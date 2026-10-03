use super::*;

/// Converts the legacy `AddNpc` packet fields and `AngleNpc` argument into the
/// native gameplay-root transform.
#[cfg(test)]
#[must_use]
pub fn tutorial_spawn_transform(spawn: TutorialNpcSpawn, npc_class: i32) -> Transform {
    tutorial_spawn_transform_for_class(spawn, npc_class)
}

pub(super) fn tutorial_spawn_transform_for_class(spawn: TutorialNpcSpawn, npc_class: i32) -> Transform {
    let mut position =
        ProtocolPosition::new([spawn.position.x, spawn.position.y, spawn.position.z]).to_native();
    // Exact `NpcContainer.Add`: ordinary NPC-table classes are instantiated
    // one Unity unit above their server position, then `NpcMoveController`
    // settles them through its downward ray and gravity.
    if npc_class < 100 {
        position.y += 1.0;
    }
    let rotation = spawn
        .angle
        .map(|angle| ProtocolYawDegrees::new(i32::from(angle)).native_root_rotation())
        .unwrap_or(Quat::IDENTITY);
    Transform::from_translation(position).with_rotation(rotation)
}

#[must_use]
pub fn tutorial_actor_root_name(id: i32) -> String {
    format!("Tutorial actor {id}")
}

/// Every NPC type which tutorial code can instantiate, derived from the
/// choreography that owns those spawns. This is a residency/coverage index,
/// not a second visual catalog.
#[must_use]
pub fn tutorial_actor_npc_types() -> BTreeSet<i32> {
    fn collect_npc_action(action: NpcAction, types: &mut BTreeSet<i32>) {
        match action {
            NpcAction::Spawn(spawn) => {
                types.insert(spawn.npc_type);
            }
            NpcAction::SpawnBatch(spawns) => {
                types.extend(spawns.iter().map(|spawn| spawn.npc_type));
            }
            _ => {}
        }
    }

    fn collect_auxiliary_action(action: TutorialAuxiliaryAction, types: &mut BTreeSet<i32>) {
        if let TutorialAuxiliaryAction::SpawnNpc { npc_type, .. } = action {
            types.insert(npc_type);
        }
    }

    let mut types = BTreeSet::from([
        TUTORIAL_INITIALIZATION.initial_npc.npc_type,
        // Spawned directly by tutorial_logic state transitions rather than
        // by a scene or auxiliary choreography action.
        2374,
        2375,
        2376,
        2377,
        2378,
        2671,
        2672,
        2694,
        2800,
    ]);
    for scene in TUTORIAL_SCENE_CHOREOGRAPHIES {
        for timed in scene.actions {
            if let ChoreographyAction::Npc(action) = timed.action {
                collect_npc_action(action, &mut types);
            }
        }
        for sourced in scene
            .skip
            .scene_specific
            .iter()
            .chain(scene.skip.common_cleanup.iter())
        {
            if let ChoreographyAction::Npc(action) = sourced.action {
                collect_npc_action(action, &mut types);
            }
        }
    }
    for definition in TUTORIAL_AUXILIARY_DEFINITIONS {
        for timed in definition.actions {
            collect_auxiliary_action(timed.action, &mut types);
        }
        if let Some(repeating) = definition.repeating {
            for action in repeating.actions {
                collect_auxiliary_action(*action, &mut types);
            }
        }
    }
    types
}

/// `NpcContainer.AddNpc` eventually calls `NpcAnimation.SetModel`, whose
/// default branch immediately executes `StandMotion`. Forced animations sent
/// before the asynchronous model arrives set `iCurrentLowMode=18` and suppress
/// that stand selection, so this must run only after the whole FIFO command
/// batch has been applied.
pub(super) fn start_unforced_actor_stand_poses(world: &mut World) {
    let ids = {
        let mut query = world.query::<(&TutorialActor, &TutorialActorPose)>();
        query
            .iter(world)
            .filter_map(|(actor, pose)| pose.clip.is_none().then_some(actor.id))
            .collect::<Vec<_>>()
    };
    for id in ids {
        play_actor_pose(world, id, "idle", false);
    }
}

pub(super) fn actor_entity(world: &mut World, id: i32) -> Option<Entity> {
    let entity = world.resource::<TutorialActorRegistry>().entity(id);
    let Some(entity) = entity else {
        world
            .resource_mut::<TutorialActorIssueQueue>()
            .push(TutorialActorIssue::MissingActor { id });
        return None;
    };
    if world.get_entity(entity).is_err() {
        world
            .resource_mut::<TutorialActorRegistry>()
            .by_id
            .remove(&id);
        world
            .resource_mut::<TutorialActorIssueQueue>()
            .push(TutorialActorIssue::MissingActor { id });
        return None;
    }
    Some(entity)
}

pub(super) fn warp_actor_native(world: &mut World, id: i32, target: Vec3) {
    let Some(entity) = actor_entity(world, id) else {
        return;
    };
    if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
        if let Some(mut transform) = entity_mut.get_mut::<Transform>() {
            transform.translation = target;
        }
        entity_mut.insert(TutorialActorGrounding::default());
        entity_mut.remove::<TutorialActorMotion>();
    }
}

pub(super) fn translate_actor_native(world: &mut World, id: i32, delta: Vec3) {
    let Some(entity) = actor_entity(world, id) else {
        return;
    };
    if let Some(mut transform) = world.get_mut::<Transform>(entity) {
        transform.translation += delta;
    }
}

pub(super) fn move_actor_native(world: &mut World, id: i32, target: Vec3, speed_units_per_second: f32) {
    if !speed_units_per_second.is_finite() || speed_units_per_second <= 0.0 {
        world.resource_mut::<TutorialActorIssueQueue>().push(
            TutorialActorIssue::InvalidMotionSpeed {
                id,
                speed_units_per_second,
            },
        );
        return;
    }
    let Some(entity) = actor_entity(world, id) else {
        return;
    };
    face_actor_native_position(world, id, target);
    if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
        entity_mut.insert(TutorialActorMotion {
            target,
            speed_units_per_second,
        });
    }
    let should_enter_motion = world.get::<TutorialActorPose>(entity).is_none_or(|pose| {
        pose.state == TutorialActorPoseState::Playing
            && (pose.clip.is_none()
                || matches!(
                    pose.role,
                    LegacyNpcAnimationRole::Stand
                        | LegacyNpcAnimationRole::Ready
                        | LegacyNpcAnimationRole::Locomotion
                ))
    });
    if should_enter_motion {
        play_actor_pose_with_contract(
            world,
            id,
            tutorial_actor_move_clip(speed_units_per_second),
            false,
            LegacyNpcAnimationRole::Locomotion,
            LegacyAnimationBlend::CrossFade300Ms,
        );
    }
}

pub(super) fn stop_actor_motion(world: &mut World, id: i32) {
    let Some(entity) = actor_entity(world, id) else {
        return;
    };
    if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
        entity_mut.remove::<TutorialActorMotion>();
    }
}

pub(super) fn rotate_actor_native(world: &mut World, id: i32, rotation: Quat) {
    let Some(entity) = actor_entity(world, id) else {
        return;
    };
    if let Some(mut transform) = world.get_mut::<Transform>(entity) {
        transform.rotation = rotation.normalize();
    }
}

pub(super) fn face_actor_native_position(world: &mut World, id: i32, target: Vec3) {
    let Some(entity) = actor_entity(world, id) else {
        return;
    };
    let Some(mut transform) = world.get_mut::<Transform>(entity) else {
        return;
    };
    let direction = target - transform.translation;
    let horizontal = Vec3::new(direction.x, 0.0, direction.z);
    if horizontal.length_squared() > 0.000_001 {
        transform.rotation = Transform::IDENTITY
            .looking_to(horizontal.normalize(), Vec3::Y)
            .rotation;
    }
}

pub(super) fn face_actor(world: &mut World, id: i32, target: i32) {
    let Some(target_entity) = actor_entity(world, target) else {
        return;
    };
    let Some(target_position) = world
        .get::<Transform>(target_entity)
        .map(|value| value.translation)
    else {
        return;
    };
    face_actor_native_position(world, id, target_position);
}

/// Exact `NpcAnimation.SetStandMotion` weight buckets. Unity's integer overload
/// of `Random.Range(0, 100)` is maximum-exclusive.
pub(super) fn tutorial_actor_idle_stand_for_roll(roll: u32, previous_clip: Option<&str>) -> &'static str {
    debug_assert!(roll < 100, "stand roll must be in Random.Range(0, 100)");
    let selected = match roll {
        0..=39 => TUTORIAL_ACTOR_STAND_CLIPS[0],
        40..=69 => TUTORIAL_ACTOR_STAND_CLIPS[1],
        70..=89 => TUTORIAL_ACTOR_STAND_CLIPS[2],
        _ => TUTORIAL_ACTOR_STAND_CLIPS[3],
    };
    // The original avoids immediately repeating only alternate stands. A
    // repeated stand1 remains valid; stand2..4 fall back to stand1.
    if selected != "stand1" && previous_clip == Some(selected) {
        "stand1"
    } else {
        selected
    }
}

/// Applies Retrobution's `NpcMoveController` vertical ray and gravity policy to
/// every live NPC which is not under direct coroutine transform control.
pub fn ground_tutorial_actors(
    time: Res<Time>,
    content: Res<TutorialMissionContent>,
    loading_players: Query<
        (),
        (
            With<crate::movement::LegacyPlayerController>,
            With<crate::movement::LegacyWorldColliderPending>,
        ),
    >,
    terrain_registry: Option<Res<NativeTerrainSpatialRegistry>>,
    scene_roots: Query<(&NativeWorldSceneRoot, &NativeWorldPresentationStatus)>,
    heightmaps: Query<(&GlobalTransform, &NativeHeightmapCollider)>,
    colliders: Query<(&GlobalTransform, &AuthoredTriMeshCollider)>,
    mut actors: Query<
        (&TutorialActor, &mut TutorialActorGrounding, &mut Transform),
        Without<TutorialActorForceUpdate>,
    >,
) {
    // NpcMoveController.UpdateMove applies vertical motion only while
    // GameFrame.IsReadyForPlay(). Actors created while world collision is
    // loading must not fall below the floor before it becomes available.
    // Keep both Y and velocity
    // until the same loading barrier used by the local player is released.
    if !loading_players.is_empty() {
        return;
    }
    let delta_seconds = time.delta_secs().max(0.0);
    for (actor, mut grounding, mut transform) in &mut actors {
        if !actor.is_alive() {
            grounding.vertical_velocity = 0.0;
            continue;
        }
        let Some(definition) = content.gameplay_npc(actor.npc_type) else {
            continue;
        };
        let x = transform.translation.x;
        let z = transform.translation.z;
        // Player readiness does not cover actors in a neighboring streamed
        // tile. Terrain can arrive before the platform's triangle collider.
        if scene_roots.iter().any(|(root, status)| {
            legacy_dong_squared_distance_native(transform.translation, root.tile) == Some(0.0)
                && *status != NativeWorldPresentationStatus::Ready
        }) {
            grounding.vertical_velocity = 0.0;
            continue;
        }
        // Exact Retrobution ray: origin = root + npc height, distance =
        // npc height + 10, therefore the lower endpoint is root - 10.
        let minimum_y = transform.translation.y - 10.0;
        let maximum_y = transform.translation.y + definition.height().max(0.0);
        let mut ground: Option<f32> = None;
        let mut collision_ready = false;

        if let Some(registry) = terrain_registry.as_deref()
            && let NativeTerrainSpatialLookup::Found(entity) = registry.lookup(x, z)
            && let Ok((global, heightmap)) = heightmaps.get(entity)
        {
            ground = heightmap.ground_height(global, x, z, minimum_y, maximum_y);
            collision_ready = true;
        }
        if ground.is_none() {
            for (global, heightmap) in &heightmaps {
                if let Some(height) = heightmap.ground_height(global, x, z, minimum_y, maximum_y) {
                    collision_ready = true;
                    ground = Some(ground.map_or(height, |current| current.max(height)));
                } else if !collision_ready
                    && heightmap.ground_height(global, x, z, -1_000_000.0, 1_000_000.0).is_some()
                {
                    // A resident floor below the short source ray still allows
                    // ordinary falling; an absent local floor must preserve Y.
                    collision_ready = true;
                }
            }
        }
        for (global, collider) in &colliders {
            if collider.is_trigger() {
                continue;
            }
            if let Some(height) =
                collider_ground_height(collider, global.to_matrix(), x, z, minimum_y, maximum_y)
            {
                collision_ready = true;
                ground = Some(ground.map_or(height, |current| current.max(height)));
            } else if !collision_ready && collider_ground_height(
                collider, global.to_matrix(), x, z, -1_000_000.0, 1_000_000.0,
            ).is_some() {
                collision_ready = true;
            }
        }
        // Actors outlive terrain residency. Without this guard a shared unload
        // makes every actor fall below the ray's upper endpoint, preventing
        // the reloaded floor from catching either the portal or nearby mobs.
        if !collision_ready {
            grounding.vertical_velocity = 0.0;
            continue;
        }
        (transform.translation.y, grounding.vertical_velocity) = advance_tutorial_actor_grounding(
            transform.translation.y,
            ground,
            grounding.vertical_velocity,
            delta_seconds,
        );
    }
}

pub(super) fn delete_actor(world: &mut World, id: i32) {
    let entity = world
        .resource_mut::<TutorialActorRegistry>()
        .by_id
        .remove(&id);
    let Some(entity) = entity else {
        world
            .resource_mut::<TutorialActorIssueQueue>()
            .push(TutorialActorIssue::MissingActor { id });
        return;
    };
    let _ = world.despawn(entity);
}

pub(super) fn damage_actor(world: &mut World, id: i32, amount: i32) {
    if amount <= 0 {
        world
            .resource_mut::<TutorialActorIssueQueue>()
            .push(TutorialActorIssue::InvalidDamage { id, amount });
        return;
    }
    let entity = world.resource::<TutorialActorRegistry>().entity(id);
    let Some(entity) = entity else {
        world
            .resource_mut::<TutorialActorIssueQueue>()
            .push(TutorialActorIssue::MissingActor { id });
        return;
    };

    let Some(mut actor) = world.get_mut::<TutorialActor>(entity) else {
        world
            .resource_mut::<TutorialActorRegistry>()
            .by_id
            .remove(&id);
        world
            .resource_mut::<TutorialActorIssueQueue>()
            .push(TutorialActorIssue::MissingActor { id });
        return;
    };
    if actor.hp <= 0 {
        return;
    }

    let first_hit = !actor.damaged;
    let applied = if actor.invulnerable {
        0
    } else {
        amount.min(actor.hp)
    };
    actor.hp -= applied;
    actor.damaged = true;
    let remaining_hp = actor.hp;
    let died = actor.hp == 0;
    let npc_type = actor.npc_type;
    drop(actor);

    let mut events = world.resource_mut::<TutorialActorEventQueue>();
    events.push(TutorialActorEvent::Damaged {
        id,
        entity,
        amount: applied,
        remaining_hp,
        first_hit,
    });
    if died {
        events.push(TutorialActorEvent::Dead { id, entity });
    }
    drop(events);

    if died {
        // `NpcAnimation.DeadMotion`: death is a once/clamp animation. Deleting
        // the gameplay root only after AnimationPlayer reports completion
        // preserves the visible death while still matching tutorial cleanup.
        play_actor_combat_pose(world, id, "death", TutorialActorCombatCompletion::Despawn);
        // Primary mob_spawn.kfm has a 1.6 s death clip. DeadMotion waits until
        // death.length - 1 before spawning the player-kill presentation. Keep
        // this marker separate from scripted AnimationNpc("death") calls,
        // which never invoke DeadMotion.Dead and own their effects explicitly.
        if matches!(npc_type, 2674 | 2897)
            && let Some(request_serial) = world
                .get::<TutorialActorPose>(entity)
                .map(|pose| pose.request_serial)
        {
            world
                .entity_mut(entity)
                .insert(TutorialActorPlayerKillDeathPresentation { request_serial });
        }
    } else {
        // `NpcAnimation.DamageMotion` plays the exact wound overlay and returns
        // the mob to AttackReady when that one-shot finishes.
        play_actor_combat_pose(world, id, "wound", TutorialActorCombatCompletion::Ready);
    }
}

pub(super) fn clear_actor_interactions(world: &mut World) {
    let mut changes = Vec::new();
    let mut query = world.query::<(Entity, &mut TutorialActor)>();
    for (entity, mut actor) in query.iter_mut(world) {
        if actor.interacting {
            actor.interacting = false;
            changes.push((actor.id, entity));
        }
    }
    changes.sort_by_key(|(id, _)| *id);
    let mut events = world.resource_mut::<TutorialActorEventQueue>();
    for (id, entity) in changes {
        events.push(TutorialActorEvent::Interaction {
            id,
            entity,
            interacting: false,
        });
    }
}

pub(super) fn configure_demo_monster(world: &mut World, dont_kill: bool) {
    world
        .resource_mut::<TutorialActorCombatConfig>()
        .demo_dont_kill = dont_kill;
    if let Some(entity) = world
        .resource::<TutorialActorRegistry>()
        .entity(DEMO_MONSTER_ID)
        && let Some(mut actor) = world.get_mut::<TutorialActor>(entity)
    {
        actor.invulnerable = dont_kill;
    }
}

pub(super) fn attempt_player_attack(world: &mut World, id: i32, player_position: Vec3) {
    let Some(entity) = world.resource::<TutorialActorRegistry>().entity(id) else {
        world
            .resource_mut::<TutorialActorIssueQueue>()
            .push(TutorialActorIssue::MissingActor { id });
        return;
    };
    let Some(actor) = world.get::<TutorialActor>(entity).copied() else {
        return;
    };
    let Some(transform) = world.get::<Transform>(entity) else {
        return;
    };
    let Some(attack_range) = world
        .resource::<TutorialMissionContent>()
        .gameplay_npc(actor.npc_type)
        .and_then(GameplayNpcUiDefinition::attack_range)
    else {
        return;
    };
    if actor.is_alive()
        && attack_range > 0.0
        && transform.translation.distance(player_position) <= attack_range
    {
        let melee_roll = world
            .resource_mut::<TutorialActorStandRandomStream>()
            .draw_percent();
        let melee_clip = tutorial_actor_attack_clip(actor.npc_type, melee_roll);
        face_actor_native_position(world, id, player_position);
        play_actor_combat_pose(world, id, melee_clip, TutorialActorCombatCompletion::Ready);
        world
            .resource_mut::<TutorialActorEventQueue>()
            .push(TutorialActorEvent::AttackedPlayer {
                id,
                entity,
                damage: 50,
            });
    }
}

/// Exact four-by-four style matrix in `cnVirtualServer`.
#[must_use]
pub fn tutorial_virtual_server_damage(
    attacker_style: i32,
    defender_style: i32,
    dont_kill: bool,
) -> i32 {
    const STYLE_EFFECT: [[f32; 4]; 4] = [
        [1.0, 0.5, 0.5, 0.5],
        [1.5, 1.0, 0.5, 1.5],
        [1.5, 1.5, 1.0, 0.5],
        [1.5, 0.5, 1.5, 1.0],
    ];
    if dont_kill {
        return 0;
    }
    let attacker = usize::try_from(attacker_style + 1).ok();
    let defender = usize::try_from(defender_style + 1).ok();
    match (attacker, defender) {
        (Some(attacker), Some(defender)) if attacker < 4 && defender < 4 => {
            (200.0 * STYLE_EFFECT[attacker][defender]) as i32
        }
        _ => 0,
    }
}

/// Removes all tutorial-owned actor state on the state transition boundary.
pub fn cleanup_tutorial_actors(world: &mut World) {
    let entities: Vec<_> = world
        .resource::<TutorialActorRegistry>()
        .iter()
        .map(|(_, entity)| entity)
        .collect();
    for entity in entities {
        let _ = world.despawn(entity);
    }
    *world.resource_mut::<TutorialActorRegistry>() = TutorialActorRegistry::default();
    *world.resource_mut::<TutorialActorCommandQueue>() = TutorialActorCommandQueue::default();
    *world.resource_mut::<TutorialActorEventQueue>() = TutorialActorEventQueue::default();
    *world.resource_mut::<TutorialActorIssueQueue>() = TutorialActorIssueQueue::default();
    *world.resource_mut::<TutorialNpcObservationSnapshot>() =
        TutorialNpcObservationSnapshot::default();
    *world.resource_mut::<TutorialActorCombatConfig>() = TutorialActorCombatConfig::default();
    let mut feeds = world.query::<&mut LegacyAvatarTargetFeed>();
    for mut feed in feeds.iter_mut(world) {
        *feed = LegacyAvatarTargetFeed::default();
    }
}

pub(super) fn set_actor_interacting(world: &mut World, id: i32, interacting: bool) {
    if world
        .resource::<TutorialActorRegistry>()
        .entity(id)
        .is_none()
    {
        world
            .resource_mut::<TutorialActorIssueQueue>()
            .push(TutorialActorIssue::MissingActor { id });
        return;
    }

    let mut changes = Vec::new();
    let mut query = world.query::<(Entity, &mut TutorialActor)>();
    for (entity, mut actor) in query.iter_mut(world) {
        let next = if interacting {
            actor.id == id && actor.is_alive()
        } else if actor.id == id {
            false
        } else {
            actor.interacting
        };
        if next != actor.interacting {
            actor.interacting = next;
            changes.push((actor.id, entity, next));
        }
    }
    changes.sort_by_key(|(actor_id, _, _)| *actor_id);
    let mut events = world.resource_mut::<TutorialActorEventQueue>();
    for (actor_id, entity, active) in changes {
        events.push(TutorialActorEvent::Interaction {
            id: actor_id,
            entity,
            interacting: active,
        });
    }
}

pub fn produce_tutorial_avatar_target_feed(
    profile: Res<TutorialTargetingProfile>,
    content: Res<TutorialMissionContent>,
    actors: Query<(Entity, &TutorialActor, &Transform)>,
    colliders: Query<(&GlobalTransform, &AuthoredTriMeshCollider)>,
    cameras: Query<&LegacyOrbitCamera, Without<LegacyAvatarTargetFeed>>,
    mut avatars: Query<
        (
            Entity,
            &Transform,
            Option<&TutorialTargetingFacing>,
            &LegacyAvatarActionContext,
            &mut LegacyAvatarTargetFeed,
        ),
        Without<TutorialActor>,
    >,
) {
    let mut actor_rows: Vec<_> = actors.iter().collect();
    actor_rows.sort_by_key(|(_, actor, _)| actor.id);

    for (avatar_entity, avatar_transform, facing, action_context, mut feed) in &mut avatars {
        let camera_forward = cameras
            .iter()
            .find(|camera| camera.target == avatar_entity)
            .map(|camera| {
                LegacyUnityHeadingDegrees::new(camera.yaw_degrees).native_root_rotation()
                    * Vec3::NEG_Z
            });
        let forward = horizontal_forward(
            avatar_transform,
            facing.map(|direction| direction.0).or(camera_forward),
        );
        feed.source_connected = true;
        feed.trigger = None;
        // `cntutorialscript` owns its local `NpcContainer` while the tutorial
        // is live. Network NPCs from the shared shard must not enter this feed:
        // tutorial damage is intentionally resolved only against TutorialActor
        // entities, so retaining a closer network duplicate makes the selected
        // Spawn impossible to attack.
        feed.samples.clear();
        let mut weapon_profile = *profile;
        weapon_profile.attack_half_angle_degrees = action_context.attack_half_angle_degrees;
        weapon_profile.attack_distance = action_context.attack_range;
        feed.samples
            .extend(actor_rows.iter().filter_map(|(entity, actor, transform)| {
                let definition = content.gameplay_npc(actor.npc_type)?;
                tutorial_target_sample_with_values(
                    *entity,
                    actor,
                    transform,
                    avatar_transform.translation,
                    forward,
                    weapon_profile,
                    definition.npc_class,
                    definition.radius(),
                    definition.height(),
                    |start, end| {
                        colliders.iter().any(|(global, collider)| {
                            authored_collider_blocks_segment(collider, global, start, end)
                        })
                    },
                )
            }));
    }
}

pub(super) fn horizontal_forward(transform: &Transform, override_direction: Option<Vec3>) -> Vec3 {
    let mut forward = override_direction.unwrap_or_else(|| transform.rotation * Vec3::NEG_Z);
    forward.y = 0.0;
    forward.try_normalize().unwrap_or(Vec3::NEG_Z)
}

#[cfg(test)]
#[must_use]
pub fn tutorial_target_sample(
    content: &TutorialMissionContent,
    entity: Entity,
    actor: &TutorialActor,
    actor_transform: &Transform,
    avatar_position: Vec3,
    avatar_forward: Vec3,
    profile: TutorialTargetingProfile,
) -> Option<LegacyTargetSample> {
    tutorial_target_sample_with_los(
        content,
        entity,
        actor,
        actor_transform,
        avatar_position,
        avatar_forward,
        profile,
        |_, _| false,
    )
}

#[cfg(test)]
pub(super) fn tutorial_target_sample_with_los(
    content: &TutorialMissionContent,
    entity: Entity,
    actor: &TutorialActor,
    actor_transform: &Transform,
    avatar_position: Vec3,
    avatar_forward: Vec3,
    profile: TutorialTargetingProfile,
    segment_blocked: impl Fn(Vec3, Vec3) -> bool,
) -> Option<LegacyTargetSample> {
    let definition = content.gameplay_npc(actor.npc_type)?;
    tutorial_target_sample_with_values(
        entity,
        actor,
        actor_transform,
        avatar_position,
        avatar_forward,
        profile,
        definition.npc_class,
        definition.radius(),
        definition.height(),
        segment_blocked,
    )
}
