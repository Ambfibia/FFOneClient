use super::*;

pub(super) fn tutorial_target_sample_with_values(
    entity: Entity,
    actor: &TutorialActor,
    actor_transform: &Transform,
    avatar_position: Vec3,
    avatar_forward: Vec3,
    profile: TutorialTargetingProfile,
    npc_class: i32,
    radius: f32,
    height: f32,
    segment_blocked: impl Fn(Vec3, Vec3) -> bool,
) -> Option<LegacyTargetSample> {
    if !actor.is_alive() || npc_class > 110 || npc_class == 25 {
        return None;
    }

    let eye = avatar_position + Vec3::Y * (profile.player_height * 0.5);
    let actor_bottom = actor_transform.translation.y;
    let actor_top = actor_bottom + height;
    let target = Vec3::new(
        actor_transform.translation.x,
        eye.y.clamp(actor_bottom, actor_top),
        actor_transform.translation.z,
    );
    let offset = target - eye;
    let distance = offset.length();
    let inside_radius = offset.length_squared() < radius * radius;
    let direction = offset.try_normalize().unwrap_or(avatar_forward);
    let forward = {
        let mut horizontal = avatar_forward;
        horizontal.y = 0.0;
        horizontal.try_normalize().unwrap_or(Vec3::NEG_Z)
    };
    let dot = direction.dot(forward).clamp(-1.0, 1.0);
    let ray_end = eye + direction * (distance - radius).max(0.0);
    let line_of_sight_blocked = !inside_radius && segment_blocked(eye, ray_end);

    let in_view = inside_radius
        || (dot >= profile.view_half_angle_degrees.to_radians().cos()
            && distance < radius.max(height * 0.5) + profile.view_distance
            && !line_of_sight_blocked);
    let in_attack_arc =
        inside_radius || dot >= profile.attack_half_angle_degrees.to_radians().cos();
    let in_attack_cone = inside_radius
        || (in_attack_arc
            && distance < radius.max(height * 0.5) + profile.attack_distance
            && !line_of_sight_blocked);
    Some(LegacyTargetSample {
        entity,
        kind: LegacyTargetKind::Npc { team: actor.team },
        distance,
        in_view,
        in_attack_arc,
        in_nano_arc: in_attack_arc,
        in_attack_cone,
        talk_enabled: actor.team == 1 && distance < profile.talk_distance,
        position: actor_transform.translation.to_array(),
        radius,
        height,
    })
}

/// Builds all named `CheckNpcDistance` / NPC-icon observations without relying
/// on ECS iteration order. Duplicate IDs, if supplied by a malformed caller,
/// are resolved by a stable position/state ordering.
#[must_use]
pub fn build_tutorial_npc_observation(
    avatar_position: Vec3,
    samples: impl IntoIterator<Item = TutorialActorObservationSample>,
) -> TutorialNpcObservation {
    let mut rows: Vec<_> = samples.into_iter().collect();
    rows.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then_with(|| left.position.x.total_cmp(&right.position.x))
            .then_with(|| left.position.y.total_cmp(&right.position.y))
            .then_with(|| left.position.z.total_cmp(&right.position.z))
            .then_with(|| left.damaged.cmp(&right.damaged))
            .then_with(|| left.interacting.cmp(&right.interacting))
    });
    rows.dedup_by_key(|row| row.id);
    let rows: BTreeMap<_, _> = rows.into_iter().map(|row| (row.id, row)).collect();

    let observe = |id| {
        rows.get(&id)
            .map_or_else(ObservedNpc::default, |actor| ObservedNpc {
                distance: Some(actor.position.distance(avatar_position)),
                damaged: actor.damaged,
                interacting: actor.interacting,
            })
    };
    TutorialNpcObservation {
        numbuh_two: observe(NUMBUH_TWO_ID),
        mission_target: observe(MISSION_TARGET_ID),
        buttercup: observe(BUTTERCUP_ID),
        tech_square_attendant: observe(TECH_SQUARE_ATTENDANT_ID),
        fusion_portal: observe(FUSION_PORTAL_ID),
        lair_dexter: observe(LAIR_DEXTER_ID),
        fusion_buttercup: observe(FUSION_BUTTERCUP_ID),
        lair_exit: observe(LAIR_EXIT_ID),
        collapse_numbuh_two: observe(COLLAPSE_NUMBUH_TWO_ID),
        demo_monster: observe(DEMO_MONSTER_ID),
    }
}
