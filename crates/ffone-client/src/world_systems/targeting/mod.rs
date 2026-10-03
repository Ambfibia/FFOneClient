//! Native normal-world NPC cone producer.
//!
//! The geometry and filtering mirror clean Retrobution
//! `NpcContainer.GetConeList`: the camera supplies the horizontal forward
//! direction, NPC table radius/height extend the view and attack cones, and
//! normal-world talking uses `m_iSightRange`. The legacy build and decompiled
//! evidence remain offline inputs; runtime state comes only from typed native
//! TableData and network entities.

use bevy::prelude::*;

use crate::{
    avatar_action::{
        LegacyAvatarActionContext, LegacyAvatarTargetFeed, LegacyTargetKind, LegacyTargetSample,
    },
    coordinates::LegacyUnityHeadingDegrees,
    entity_lifecycle::{NetworkNpcAppearance0104, NetworkPcAppearance0104, NetworkRemotePc0104},
    movement::{LegacyOrbitCamera, SERVER_TO_CLIENT_SCALE},
    tutorial_mission_content::{GameplayNpcUiDefinition, TutorialMissionContent},
    world::{
        AUTHORED_CHARACTER_CONTROLLER_HEIGHT, AUTHORED_CHARACTER_CONTROLLER_RADIUS,
        AuthoredColliderWorldBounds, AuthoredTriMeshCollider,
        authored_collider_blocks_segment_with_bounds,
    },
};

const LEGACY_WORLD_VIEW_HALF_ANGLE_DEGREES: f32 = 15.0;
const LEGACY_WORLD_VIEW_DISTANCE: f32 = 25.0;
const LEGACY_WORLD_PC_VIEW_DISTANCE_FACTOR: f32 = 0.5;

/// RustyFusion `RANGE_INTERACT`, measured between entity roots in all three axes.
/// Visual capsule height/radius must not extend the server's interaction range.
#[must_use]
pub fn world_npc_in_interaction_range(player: Vec3, npc: Vec3) -> bool {
    if !player.is_finite() || !npc.is_finite() {
        return false;
    }
    let player = crate::coordinates::ProtocolPosition::from_native(player).raw();
    let npc = crate::coordinates::ProtocolPosition::from_native(npc).raw();
    // Match packet truncation and RustyFusion Position::distance_to, including
    // its inclusive integer boundary. Do not use the visual capsule distance.
    let delta = Vec3::new(
        player[0].abs_diff(npc[0]) as f32 / 100.0,
        player[1].abs_diff(npc[1]) as f32 / 100.0,
        player[2].abs_diff(npc[2]) as f32 / 100.0,
    );
    (delta.length() * 100.0) as u32 <= 800
}

/// The user cone's distance gate also bounds explicit pointer selection.
pub fn world_pc_in_interaction_range(distance: f32) -> bool {
    let extent = AUTHORED_CHARACTER_CONTROLLER_RADIUS.max(AUTHORED_CHARACTER_CONTROLLER_HEIGHT * 0.5);
    distance.is_finite() && distance >= 0.0
        && distance < extent + LEGACY_WORLD_VIEW_DISTANCE * LEGACY_WORLD_PC_VIEW_DISTANCE_FACTOR
}

fn horizontal_forward(transform: &Transform, camera_forward: Option<Vec3>) -> Vec3 {
    let mut forward = camera_forward.unwrap_or_else(|| transform.rotation * Vec3::NEG_Z);
    forward.y = 0.0;
    forward.try_normalize().unwrap_or(Vec3::NEG_Z)
}

fn legacy_network_npc_bottom(transform: &Transform, definition: &GameplayNpcUiDefinition) -> Vec3 {
    let mut bottom = transform.translation;
    // `NpcContainer.Add` raises ordinary NPC roots by one client unit before
    // `GetConeList`; event-scene classes (>= 100) retain the server height.
    if definition.npc_class < 100 {
        bottom.y += 1.0;
    }
    bottom
}

fn legacy_vertical_target(eye_y: f32, bottom_y: f32, height: f32) -> f32 {
    let top_y = bottom_y + height;
    if bottom_y < eye_y && eye_y < top_y {
        eye_y
    } else if bottom_y > eye_y {
        bottom_y
    } else if top_y < eye_y {
        top_y
    } else {
        // This preserves the clean strict-boundary branch (including its
        // unusual eye==top result) rather than replacing it with clamp().
        bottom_y
    }
}

/// Projects one live network NPC into the exact normal-world cone inputs.
///
/// `segment_blocked` represents the clean static-world raycast. A row is
/// retained even when it is outside both cones so the selection layer can
/// continue to own ordering and combat/talk semantics.
#[must_use]
pub fn world_npc_target_sample(
    entity: Entity,
    appearance: &NetworkNpcAppearance0104,
    definition: &GameplayNpcUiDefinition,
    npc_transform: &Transform,
    avatar_transform: &Transform,
    avatar_forward: Vec3,
    action_context: &LegacyAvatarActionContext,
    segment_blocked: impl Fn(Vec3, Vec3) -> bool,
) -> Option<LegacyTargetSample> {
    if appearance.0.hp <= 0 || definition.npc_class > 110 || definition.npc_class == 25 {
        return None;
    }

    let eye = avatar_transform.translation + Vec3::Y * (AUTHORED_CHARACTER_CONTROLLER_HEIGHT * 0.5);
    let bottom = legacy_network_npc_bottom(npc_transform, definition);
    let height = definition.height_server_units as f32 * SERVER_TO_CLIENT_SCALE;
    let radius = definition.radius_server_units as f32 * SERVER_TO_CLIENT_SCALE;
    let target = Vec3::new(
        bottom.x,
        legacy_vertical_target(eye.y, bottom.y, height),
        bottom.z,
    );
    let offset = target - eye;
    let distance_squared = offset.length_squared();
    let distance = distance_squared.sqrt();
    let inside_radius = distance_squared < radius * radius;
    let direction = offset.try_normalize().unwrap_or(avatar_forward);
    let forward = {
        let mut horizontal = avatar_forward;
        horizontal.y = 0.0;
        horizontal.try_normalize().unwrap_or(Vec3::NEG_Z)
    };
    let angle = direction.dot(forward).clamp(-1.0, 1.0).acos();
    let view_arc = angle <= LEGACY_WORLD_VIEW_HALF_ANGLE_DEGREES.to_radians();
    let attack_arc = angle <= action_context.attack_half_angle_degrees.to_radians();
    let nano_arc = action_context
        .nano_target_policy
        .is_some_and(|policy| angle <= policy.half_angle_degrees.to_radians());
    let view_limit = radius.max(height * 0.5) + LEGACY_WORLD_VIEW_DISTANCE;
    let attack_limit = radius.max(height * 0.5) + action_context.attack_range;
    // The result of the static-world raycast is only observable inside one
    // of the two distance-and-arc gates below. Avoid multiplying a complete
    // dense-tile collider scan by every off-screen/out-of-range NPC.
    let needs_line_of_sight = !inside_radius
        && ((view_arc && distance_squared < view_limit * view_limit)
            || (attack_arc && distance_squared < attack_limit * attack_limit));
    let line_of_sight_blocked = if needs_line_of_sight {
        let ray_end = eye + direction * (distance - radius).max(0.0);
        segment_blocked(eye, ray_end)
    } else {
        false
    };

    Some(LegacyTargetSample {
        entity,
        kind: LegacyTargetKind::Npc {
            team: definition.team,
        },
        distance,
        in_view: inside_radius
            || (view_arc && distance_squared < view_limit * view_limit && !line_of_sight_blocked),
        in_attack_arc: inside_radius || attack_arc,
        in_nano_arc: inside_radius || nano_arc,
        in_attack_cone: inside_radius
            || (attack_arc
                && distance_squared < attack_limit * attack_limit
                && !line_of_sight_blocked),
        talk_enabled: definition.team == 1
            && distance < definition.sight_range_server_units as f32 * SERVER_TO_CLIENT_SCALE
            && world_npc_in_interaction_range(
                avatar_transform.translation,
                npc_transform.translation,
            ),
        position: npc_transform.translation.to_array(),
        radius,
        height,
    })
}

/// Projects one live remote PC through clean `UserContainer.GetConeList`.
/// User targeting deliberately has neither the NPC inside-radius shortcut nor
/// the static-world LOS raycast, and compresses vertical separation by half
/// before storing/sorting the target distance.
#[must_use]
pub fn world_pc_target_sample(
    entity: Entity,
    appearance: &NetworkPcAppearance0104,
    pc_transform: &Transform,
    avatar_transform: &Transform,
    avatar_forward: Vec3,
    action_context: &LegacyAvatarActionContext,
) -> Option<LegacyTargetSample> {
    if appearance.0.hp <= 0 || appearance.0.special_state as u8 & 0x06 != 0 {
        return None;
    }

    let eye = avatar_transform.translation + Vec3::Y * (AUTHORED_CHARACTER_CONTROLLER_HEIGHT * 0.5);
    let bottom = pc_transform.translation;
    let mut offset = Vec3::new(
        bottom.x,
        legacy_vertical_target(eye.y, bottom.y, AUTHORED_CHARACTER_CONTROLLER_HEIGHT),
        bottom.z,
    ) - eye;
    offset.y *= 0.5;
    let distance_squared = offset.length_squared();
    let distance = distance_squared.sqrt();
    let forward = {
        let mut horizontal = avatar_forward;
        horizontal.y = 0.0;
        horizontal.try_normalize().unwrap_or(Vec3::NEG_Z)
    };
    let direction = offset.try_normalize().unwrap_or(forward);
    let angle = direction.dot(forward).clamp(-1.0, 1.0).acos();
    let nano_arc = action_context
        .nano_target_policy
        .is_some_and(|policy| angle < policy.half_angle_degrees.to_radians());
    let extent =
        AUTHORED_CHARACTER_CONTROLLER_RADIUS.max(AUTHORED_CHARACTER_CONTROLLER_HEIGHT * 0.5);
    let view_limit = extent + LEGACY_WORLD_VIEW_DISTANCE * LEGACY_WORLD_PC_VIEW_DISTANCE_FACTOR;
    let attack_limit = extent + action_context.attack_range;

    Some(LegacyTargetSample {
        entity,
        kind: LegacyTargetKind::Player,
        distance,
        in_view: angle <= LEGACY_WORLD_VIEW_HALF_ANGLE_DEGREES.to_radians()
            && distance_squared < view_limit * view_limit,
        in_attack_arc: angle < action_context.attack_half_angle_degrees.to_radians(),
        in_nano_arc: nano_arc,
        in_attack_cone: angle < action_context.attack_half_angle_degrees.to_radians()
            && distance_squared < attack_limit * attack_limit,
        // User talk has no second range gate; membership in the view list is
        // the complete clean enablement condition when PvP is disabled.
        talk_enabled: true,
        position: pc_transform.translation.to_array(),
        radius: AUTHORED_CHARACTER_CONTROLLER_RADIUS,
        height: AUTHORED_CHARACTER_CONTROLLER_HEIGHT,
    })
}

/// Populates the local normal-world avatar feed from live protocol-0104 NPC
/// entities. `ai_type == 0` is intentionally not a filter: clean Guide
/// Changers and the paid-zone warp service are static but fully talkable.
pub fn produce_world_avatar_target_feed(
    content: Res<TutorialMissionContent>,
    colliders: Query<(
        &GlobalTransform,
        &AuthoredTriMeshCollider,
        &AuthoredColliderWorldBounds,
    )>,
    cameras: Query<&LegacyOrbitCamera, Without<LegacyAvatarTargetFeed>>,
    npcs: Query<(Entity, &NetworkNpcAppearance0104, &Transform), Without<LegacyAvatarTargetFeed>>,
    pcs: Query<
        (
            Entity,
            &NetworkRemotePc0104,
            &NetworkPcAppearance0104,
            &Transform,
        ),
        Without<LegacyAvatarTargetFeed>,
    >,
    mut avatars: Query<(
        Entity,
        &Transform,
        &LegacyAvatarActionContext,
        &mut LegacyAvatarTargetFeed,
    )>,
) {
    let mut npc_rows: Vec<_> = npcs.iter().collect();
    npc_rows.sort_by_key(|(entity, appearance, _)| (appearance.0.npc_id, entity.to_bits()));
    let mut pc_rows: Vec<_> = pcs.iter().collect();
    pc_rows.sort_by_key(|(entity, remote, _, _)| (remote.pc_id, entity.to_bits()));

    for (avatar_entity, avatar_transform, action_context, mut feed) in &mut avatars {
        let camera_forward = cameras
            .iter()
            .find(|camera| camera.target == avatar_entity)
            .map(|camera| {
                LegacyUnityHeadingDegrees::new(camera.yaw_degrees).native_root_rotation()
                    * Vec3::NEG_Z
            });
        let forward = horizontal_forward(avatar_transform, camera_forward);
        feed.source_connected = true;
        feed.trigger = None;
        feed.samples.clear();
        feed.samples.extend(
            npc_rows
                .iter()
                .filter_map(|(entity, appearance, npc_transform)| {
                    let definition = content.gameplay_npc(appearance.0.npc_type)?;
                    world_npc_target_sample(
                        *entity,
                        appearance,
                        definition,
                        npc_transform,
                        avatar_transform,
                        forward,
                        action_context,
                        |start, end| {
                            colliders.iter().any(|(global, collider, bounds)| {
                                authored_collider_blocks_segment_with_bounds(
                                    collider, global, bounds, start, end,
                                )
                            })
                        },
                    )
                }),
        );
        feed.samples.extend(
            pc_rows
                .iter()
                .filter_map(|(entity, _, appearance, pc_transform)| {
                    world_pc_target_sample(
                        *entity,
                        appearance,
                        pc_transform,
                        avatar_transform,
                        forward,
                        action_context,
                    )
                }),
        );
    }
}

#[cfg(test)]
mod tests;
