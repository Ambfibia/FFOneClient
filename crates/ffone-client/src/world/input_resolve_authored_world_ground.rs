use super::*;

pub const LEGACY_DONG_LOAD_DISTANCE_NATIVE: f32 = 280.0;

/// Streaming covers the 340-unit camera plus the maximum 12-unit orbit offset
/// and a four-unit admission margin. Presentation overflow centers preserve
/// primary edge backdrops without increasing the strict nine-tile cap.
pub const EXTENDED_DONG_LOAD_DISTANCE_NATIVE: f32 = 356.0;

#[must_use]
pub fn load_first_native_world_scene(
    asset_root: impl AsRef<Path>,
) -> Result<NativeWorldScene, NativeWorldSceneError> {
    NativeWorldScene::open(asset_root, FIRST_NATIVE_WORLD_SCENE)
}

pub fn load_native_world_scenes(
    asset_root: impl AsRef<Path>,
) -> Result<NativeWorldCatalog, NativeWorldSceneError> {
    NativeWorldCatalog::open(asset_root)
}

pub(super) fn resolve_authored_world_ground(
    mut commands: Commands,
    time: Res<Time>,
    mut movement_intents: Option<ResMut<MovementIntentQueue>>,
    spatial_index: Res<AuthoredColliderSpatialIndex>,
    mut ground_triangles: Local<GroundTriangleCache>,
    parents: Query<&ChildOf>,
    colliders: Query<(
        Entity,
        &GlobalTransform,
        &AuthoredTriMeshCollider,
        &AuthoredColliderWorldBounds,
    )>,
    runtime_collider_motion: Query<(
        Entity,
        &AuthoredTriMeshCollider,
        &AuthoredColliderWorldBounds,
        &RuntimeAuthoredColliderMotion,
    )>,
    heightmaps: Query<(&GlobalTransform, &NativeHeightmapCollider)>,
    terrain_registry: Res<NativeTerrainSpatialRegistry>,
    scene_roots: Query<(&NativeWorldSceneRoot, &NativeWorldPresentationStatus)>,
    mut players: Query<(
        Entity,
        &mut Transform,
        &mut LegacyPlayerController,
        Option<&mut NativeWorldGroundSupport>,
    )>,
    mut remote_players: Query<
        (&mut Transform, &RemoteAnimation),
        (
            With<NetworkRemotePc0104>,
            Without<LegacyPlayerController>,
            Without<NetworkNpc0104>,
        ),
    >,
    mut npcs: Query<
        (
            Entity,
            &NetworkNpc0104,
            &NetworkNpcAppearance0104,
            &mut NetworkNpcGrounding0104,
            &mut Transform,
        ),
        (
            Without<LegacyPlayerController>,
            Without<NetworkRemotePc0104>,
        ),
    >,
) {
    ground_triangles.begin_frame(|entity| colliders.contains(entity));
    if colliders.iter().next().is_none() && terrain_registry.key_entities.is_empty() {
        return;
    }
    let mut collider_candidates = Vec::new();

    for (player_entity, mut transform, mut controller, support) in &mut players {
        if controller.collision != LegacyCollisionMode::External {
            continue;
        }
        let mut supported_collider = None;
        if let Some(mut support) = support {
            supported_collider = Some(support.collider);
            if let Ok((_, current_global, _, _)) = colliders.get(support.collider) {
                let current = current_global.to_matrix();
                if let Some(carried) = carried_support_point(
                    support.last_world_from_local,
                    current,
                    transform.translation,
                ) {
                    if !carried.abs_diff_eq(transform.translation, AUTHORED_COLLISION_EPSILON) {
                        controller.record_external_transport_motion();
                    }
                    transform.translation = carried;
                }
                support.last_world_from_local = current;
            } else {
                commands
                    .entity(player_entity)
                    .remove::<NativeWorldGroundSupport>();
            }
        }
        let (previous_position, wall_contact_normal, mut collision_flags) =
            resolve_authored_player_walls(
                time.delta_secs(),
                &colliders,
                &runtime_collider_motion,
                &spatial_index,
                &heightmaps,
                supported_collider,
                &mut transform,
                &mut controller,
            );
        let mut contact_normal = wall_contact_normal;
        // CharacterController performs an upward Move as separate UP and SIDE
        // passes. A rounded upper-lip normal produced by SIDE is still
        // CollisionFlags.Sides, never Below. Do not run the later support
        // query until the submitted vertical movement is no longer rising;
        // otherwise an upward-pointing edge normal ends the jump one frame too
        // early and pins the capsule against the lip.
        if controller.velocity.y > 0.0 {
            controller.set_external_collision_result(collision_flags, contact_normal);
            controller.set_grounded(false);
            controller.reconcile_external_collision_position(
                player_entity,
                transform.translation,
                movement_intents.as_deref_mut(),
            );
            commands
                .entity(player_entity)
                .remove::<NativeWorldGroundSupport>();
            continue;
        }

        let current_y = transform.translation.y;
        let (minimum_y, maximum_y) = authored_player_ground_search_range(
            previous_position,
            transform.translation,
            controller.grounded,
        );
        let mut ground: Option<f32> = None;
        let mut ground_normal = Vec3::Y;
        let mut ground_support: Option<(Entity, Mat4, Vec3)> = None;
        if let NativeTerrainSpatialLookup::Found(entity) =
            terrain_registry.lookup(transform.translation.x, transform.translation.z)
        {
            if let Ok((global, heightmap)) = heightmaps.get(entity) {
                if let Some((height, normal)) = terrain_capsule_ground_contact(
                    heightmap,
                    global,
                    transform.translation.x,
                    transform.translation.z,
                    minimum_y,
                    maximum_y,
                ) {
                    ground = Some(height);
                    ground_normal = normal;
                }
            }
        }
        // A dong edge belongs to two heightfields, and the registry can also
        // be transiently Missing/Duplicate while a complete tutorial scene is
        // materialized. Unity TerrainCollider still tests the actual surfaces
        // in either case. Fall back to every resident heightfield when the
        // keyed sample did not produce a crossing; each query is O(1).
        if ground.is_none() {
            for (global, heightmap) in &heightmaps {
                if let Some((height, normal)) = terrain_capsule_ground_contact(
                    heightmap,
                    global,
                    transform.translation.x,
                    transform.translation.z,
                    minimum_y,
                    maximum_y,
                ) {
                    if ground.is_none_or(|current| height > current) {
                        ground = Some(height);
                        ground_normal = normal;
                    }
                }
            }
        }
        // Recover a shallow terrain penetration left by a missing registry
        // frame, within the controller's step offset. Interiors can be authored
        // far below the outdoor heightfield (including Foster's House); an
        // unbounded upward search ejects an authoritative interior warp onto
        // the landscape before its mesh floor can provide support.
        if ground.is_none() {
            for (global, heightmap) in &heightmaps {
                if let Some((height, normal)) = terrain_capsule_ground_contact(
                    heightmap,
                    global,
                    transform.translation.x,
                    transform.translation.z,
                    current_y,
                    current_y + GROUNDED_STEP_UP,
                ) && current_y < height - GROUND_EPSILON
                {
                    if ground.is_none_or(|current| height > current) {
                        ground = Some(height);
                        ground_normal = normal;
                    }
                }
            }
        }
        let capsule_support_radius =
            AUTHORED_CHARACTER_CONTROLLER_RADIUS - AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH;
        spatial_index.candidates(
            Vec3::new(
                transform.translation.x - capsule_support_radius,
                minimum_y - capsule_support_radius,
                transform.translation.z - capsule_support_radius,
            ),
            Vec3::new(
                transform.translation.x + capsule_support_radius,
                maximum_y + capsule_support_radius,
                transform.translation.z + capsule_support_radius,
            ),
            &mut collider_candidates,
        );
        for collider_entity in collider_candidates.iter().copied() {
            let Ok((_, global, collider, bounds)) = colliders.get(collider_entity) else {
                continue;
            };
            if collider.is_trigger {
                continue;
            }
            if let Some((height, surface_normal)) = collider_capsule_ground_contact_with_bounds(
                collider,
                global.to_matrix(),
                bounds,
                transform.translation.x,
                transform.translation.z,
                previous_position.y,
                minimum_y,
                maximum_y,
            ) {
                if ground.is_none_or(|current| height > current) {
                    ground = Some(height);
                    ground_normal = surface_normal;
                    ground_support = Some((collider_entity, global.to_matrix(), surface_normal));
                }
            }
        }

        if let Some(height) = ground {
            transform.translation.y = height;
            collision_flags |= LEGACY_COLLISION_BELOW;
            // A completed downward/grounded Move reports its supporting
            // contact after the side manifold. Preserve that callback order:
            // retaining a steep side normal here made the next frame slide
            // outward even while simply walking against an object's edge.
            // Rising moves returned above before this support query, so the
            // Hero Square jump-edge boost still retains its steep normal.
            contact_normal = Some(ground_normal);
            controller.set_external_collision_result(collision_flags, contact_normal);
            // The clean controller deliberately ignores `Below` while the
            // previous frame's ordinary-object contact has activated
            // EnvironmentCollision.sliding. Its current Move is still
            // constrained to the surface, but the undoubled fVelocityZ and
            // airborne jump state survive into the following frame.
            if controller.surface_sliding() {
                controller.set_grounded(false);
            } else if controller.jumping || !controller.grounded {
                controller.land_on_external_collider();
            } else {
                controller.set_grounded(true);
            }
            if !controller.surface_sliding()
                && let Some((collider, last_world_from_local, surface_normal)) = ground_support
            {
                commands
                    .entity(player_entity)
                    .insert(NativeWorldGroundSupport {
                        collider,
                        last_world_from_local,
                        surface_normal,
                    });
            } else {
                commands
                    .entity(player_entity)
                    .remove::<NativeWorldGroundSupport>();
            }
        } else {
            controller.set_external_collision_result(collision_flags, contact_normal);
            // `cnAvatarThirdPersonMove` consumes CharacterController.Move's
            // DOWN-pass flag directly. It does not perform a second vertical
            // ray/support lookup before accepting the contact. A finite edge
            // can therefore be hit during the downward sweep even though the
            // final XZ point no longer has a face below its root ray. Requiring
            // the independent `ground` query here dropped that real Below,
            // so a forced surface-slide frame skipped straight across small
            // branch dents before it could switch back to normal movement.
            //
            // Preserve the clean callback ordering as well: the current
            // frame's `EnvironmentCollision.sliding` was selected from the
            // preceding Move and deliberately suppresses landing once. The
            // DOWN normal stored above is then consumed on the next frame.
            if collision_flags & LEGACY_COLLISION_BELOW != 0 {
                if controller.surface_sliding() {
                    controller.set_grounded(false);
                } else if controller.jumping || !controller.grounded {
                    controller.land_on_external_collider();
                } else {
                    controller.set_grounded(true);
                }
                commands
                    .entity(player_entity)
                    .remove::<NativeWorldGroundSupport>();
            } else if controller.grounded {
                controller.set_grounded(false);
                commands
                    .entity(player_entity)
                    .remove::<NativeWorldGroundSupport>();
            } else {
                commands
                    .entity(player_entity)
                    .remove::<NativeWorldGroundSupport>();
            }
        }
        // This check is late in the clean ForceUpdate, after Move's Below
        // handling. A steep concavity can keep `sliding` true while the CCT
        // itself has stopped at exactly the same height. Once terminal gravity
        // observes that unchanged Y twice, Retrobution clears the airborne and
        // sliding states so the player can stand and jump from the feature.
        if controller.settle_terminal_blocked_fall(transform.translation.y) {
            commands
                .entity(player_entity)
                .remove::<NativeWorldGroundSupport>();
        }
        controller.reconcile_external_collision_position(
            player_entity,
            transform.translation,
            movement_intents.as_deref_mut(),
        );
    }

    for (mut transform, animation) in &mut remote_players {
        if matches!(animation.state, RemoteAnimationState::Jumping { .. }) {
            continue;
        }
        let current_y = transform.translation.y;
        let minimum_y = current_y - REMOTE_GROUND_STEP_DOWN;
        let maximum_y = current_y + REMOTE_GROUND_STEP_UP;
        let mut ground: Option<f32> = None;
        if let NativeTerrainSpatialLookup::Found(entity) =
            terrain_registry.lookup(transform.translation.x, transform.translation.z)
            && let Ok((global, heightmap)) = heightmaps.get(entity)
        {
            ground = heightmap.ground_height(
                global,
                transform.translation.x,
                transform.translation.z,
                minimum_y,
                maximum_y,
            );
        }
        if ground.is_none() {
            for (global, heightmap) in &heightmaps {
                if let Some(height) = heightmap.ground_height(
                    global,
                    transform.translation.x,
                    transform.translation.z,
                    minimum_y,
                    maximum_y,
                ) {
                    ground = Some(ground.map_or(height, |current| current.max(height)));
                }
            }
        }
        spatial_index.point_candidates(
            transform.translation.x,
            transform.translation.z,
            &mut collider_candidates,
        );
        for collider_entity in collider_candidates.iter().copied() {
            let Ok((_, global, collider, bounds)) = colliders.get(collider_entity) else {
                continue;
            };
            if collider.is_trigger {
                continue;
            }
            if let Some(height) = ground_triangles.height(
                collider_entity,
                collider,
                global.to_matrix(),
                bounds,
                transform.translation.x,
                transform.translation.z,
                minimum_y,
                maximum_y,
            ) {
                ground = Some(ground.map_or(height, |current| current.max(height)));
            }
        }
        if let Some(height) = ground.filter(|height| height.is_finite()) {
            transform.translation.y = height;
        }
    }

    for (npc_entity, _npc, appearance, mut grounding, mut transform) in &mut npcs {
        if appearance.0.hp <= 0 {
            grounding.vertical_velocity = 0.0;
            continue;
        }
        let x = transform.translation.x;
        let z = transform.translation.z;
        // Terrain is intentionally decoded before the much denser static
        // scene. Without this gate an NPC can see the heightfield as "ready",
        // start source gravity, and settle below a platform whose authored
        // triangle collider is still queued. Preserve the authoritative Y
        // until the complete owning tile (including every collider) is ready.
        let owning_tile_is_loading = scene_roots.iter().any(|(root, presentation)| {
            root.scope == NativeWorldScope::WorldMap
                && legacy_dong_squared_distance_native(transform.translation, root.tile)
                    == Some(0.0)
                && *presentation != NativeWorldPresentationStatus::Ready
        });
        if owning_tile_is_loading {
            grounding.vertical_velocity = 0.0;
            continue;
        }
        // Exact NpcMoveController ray: origin = root + m_iHeight * .01,
        // distance = height + 10, so its lower endpoint is root - 10.
        let minimum_y = transform.translation.y - 10.0;
        let maximum_y = transform.translation.y + grounding.probe_height.max(0.0);
        let mut collision_ready = false;
        let mut ground: Option<f32> = None;

        if let NativeTerrainSpatialLookup::Found(entity) = terrain_registry.lookup(x, z)
            && let Ok((global, heightmap)) = heightmaps.get(entity)
        {
            collision_ready = true;
            ground = heightmap.ground_height(global, x, z, minimum_y, maximum_y);
        }
        if ground.is_none() {
            for (global, heightmap) in &heightmaps {
                if let Some(height) = heightmap.ground_height(global, x, z, minimum_y, maximum_y) {
                    collision_ready = true;
                    ground = Some(ground.map_or(height, |current| current.max(height)));
                } else if !collision_ready
                    && heightmap
                        .ground_height(
                            global,
                            x,
                            z,
                            -TERRAIN_RECOVERY_VERTICAL_LIMIT,
                            TERRAIN_RECOVERY_VERTICAL_LIMIT,
                        )
                        .is_some()
                {
                    // Keep the authoritative server Y until the relevant
                    // streamed terrain exists, then allow source gravity.
                    collision_ready = true;
                }
            }
        }
        spatial_index.point_candidates(x, z, &mut collider_candidates);
        for collider_entity in collider_candidates.iter().copied() {
            let Ok((_, global, collider, bounds)) = colliders.get(collider_entity) else {
                continue;
            };
            if collider.is_trigger || native_world_descendant(collider_entity, npc_entity, &parents)
            {
                continue;
            }
            if let Some(height) = ground_triangles.height(
                collider_entity,
                collider,
                global.to_matrix(),
                bounds,
                x,
                z,
                minimum_y,
                maximum_y,
            ) {
                collision_ready = true;
                ground = Some(ground.map_or(height, |current| current.max(height)));
            } else if !collision_ready
                && ground_triangles.height(
                    collider_entity,
                    collider,
                    global.to_matrix(),
                    bounds,
                    x,
                    z,
                    -TERRAIN_RECOVERY_VERTICAL_LIMIT,
                    TERRAIN_RECOVERY_VERTICAL_LIMIT,
                )
                .is_some()
            {
                collision_ready = true;
            }
        }
        // Network appearances can arrive before their dong finishes streaming.
        // Do not let gravity strand an NPC below a not-yet-loaded surface.
        if !collision_ready {
            continue;
        }
        (transform.translation.y, grounding.vertical_velocity) = advance_network_npc_grounding(
            transform.translation.y,
            ground,
            grounding.vertical_velocity,
            time.delta_secs().max(0.0),
        );
    }
}

pub(super) fn resolve_authored_player_walls(
    delta_seconds: f32,
    colliders: &Query<(
        Entity,
        &GlobalTransform,
        &AuthoredTriMeshCollider,
        &AuthoredColliderWorldBounds,
    )>,
    runtime_collider_motion: &Query<(
        Entity,
        &AuthoredTriMeshCollider,
        &AuthoredColliderWorldBounds,
        &RuntimeAuthoredColliderMotion,
    )>,
    spatial_index: &AuthoredColliderSpatialIndex,
    heightmaps: &Query<(&GlobalTransform, &NativeHeightmapCollider)>,
    supported_collider: Option<Entity>,
    transform: &mut Transform,
    controller: &mut LegacyPlayerController,
) -> (Vec3, Option<Vec3>, u8) {
    let requested_displacement = controller.last_move_displacement();
    let requested_position = transform.translation;
    let previous = requested_position - requested_displacement;
    if !delta_seconds.is_finite() || delta_seconds <= 0.0 {
        return (previous, None, 0);
    }
    // Unity's CharacterController constrains a grounded horizontal Move to
    // the supporting walkable plane. Preserve the requested three-dimensional
    // speed while changing only the collision-owned position; MovePacket has
    // already sampled the unmodified source kMovement at this point.
    let constrained_displacement = controller
        .walkable_support_normal()
        .map_or(requested_displacement, |normal| {
            constrain_displacement_to_walkable_surface(requested_displacement, normal)
        });
    let mut current = previous + constrained_displacement;
    // A moving kinematic mesh has velocity of its own. Solving only the
    // player's requested displacement lets a road car cross a stationary
    // capsule and leave it behind the collider's one-sided faces. Resolve each
    // such interaction in the mesh's previous local frame before the ordinary
    // current-pose wall manifold. A collider already carrying this avatar as
    // ground support is skipped: `carried_support_point` above has applied the
    // same pose delta exactly once.
    for (entity, collider, bounds, motion) in runtime_collider_motion {
        if Some(entity) == supported_collider || collider.is_trigger {
            continue;
        }
        current =
            resolve_runtime_authored_collider_motion(previous, current, collider, bounds, *motion);
    }
    // Bound the terrain cell query before allocating its patch, just as the
    // individual capsule passes bound their substep work. UP/SIDE/DOWN can
    // each consume one pass budget.
    if !previous.is_finite()
        || !current.is_finite()
        || previous.distance(current)
            > 3.0 * AUTHORED_COLLISION_MAX_SUBSTEP * AUTHORED_COLLISION_MAX_SUBSTEPS as f32
    {
        transform.translation = previous;
        controller.velocity.x = 0.0;
        controller.velocity.z = 0.0;
        return (previous, None, 0);
    }
    let (query_minimum, query_maximum) = authored_wall_spatial_query_bounds(previous, current);
    // Heightfields must participate in UP/SIDE/DOWN, not only the final
    // vertical support sample. A fast SIDE pass can otherwise end below an
    // uphill face, outside the shallow ground recovery window.
    let terrain_patch =
        authored_terrain_motion_patch(heightmaps.iter(), query_minimum, query_maximum);
    let mut candidates = Vec::new();
    spatial_index.candidates(query_minimum, query_maximum, &mut candidates);
    let mut active_colliders = candidates
        .into_iter()
        .filter_map(|entity| colliders.get(entity).ok())
        .filter(|(_, _, collider, _)| !collider.is_trigger)
        .map(|(_, global, collider, bounds)| (global.to_matrix(), collider, bounds))
        .collect::<Vec<_>>();
    if let Some((collider, bounds)) = &terrain_patch {
        active_colliders.push((Mat4::IDENTITY, collider, bounds));
    }
    let resolved = resolve_authored_controller_motion_with_bounds(
        previous,
        current - previous,
        requested_displacement.y,
        &active_colliders,
        requested_displacement.y > 0.0,
    );
    transform.translation = resolved.position;
    controller.velocity.x = (resolved.position.x - previous.x) / delta_seconds;
    controller.velocity.z = (resolved.position.z - previous.z) / delta_seconds;
    (previous, resolved.contact_normal, resolved.collision_flags)
}

/// Reproduces the pass ordering used by PhysX CharacterController::Move.
///
/// The distinction matters at a reachable convex upper edge. A single
/// diagonal capsule sweep reaches the vertical face before its lower sphere
/// has gained the frame's vertical height and can pin the avatar below the
/// lip. The source CCT first consumes the complete upward component and only
/// then sweeps the horizontal component from that raised position. At the
/// apex and during descent it instead performs an artificial stepOffset UP,
/// then SIDE, then restores that offset together with gravity in DOWN.
/// Collision flags describe the pass which hit, not the Y component of its
/// contact normal: an upward-facing rounded normal in SIDE remains `Sides`.
pub(super) fn resolve_authored_controller_motion_with_bounds(
    previous: Vec3,
    displacement: Vec3,
    submitted_vertical_displacement: f32,
    colliders: &[(Mat4, &AuthoredTriMeshCollider, &AuthoredColliderWorldBounds)],
    moving_up: bool,
) -> AuthoredControllerMotionResult {
    let side_displacement = Vec3::new(displacement.x, 0.0, displacement.z);
    let side_is_zero = side_displacement.length_squared() <= AUTHORED_COLLISION_EPSILON.powi(2);
    // PhysX cancels stepOffset while dir.dot(up) > 0. Otherwise every Move
    // with real lateral input first raises the capsule by the serialized step
    // offset, sweeps SIDE from that raised position, then removes the height
    // again in DOWN. This is not restricted to grounded movement: the same
    // decomposition is what lets a descending capsule acquire a platform lip.
    let requested_step_offset = if moving_up || side_is_zero {
        0.0
    } else {
        GROUNDED_STEP_UP
    };
    let upward_displacement = if moving_up {
        submitted_vertical_displacement.max(0.0)
    } else {
        requested_step_offset
    };
    let upward = if upward_displacement.abs() > AUTHORED_COLLISION_EPSILON {
        resolve_authored_wall_motion_detailed_with_bounds(
            previous,
            Vec3::new(0.0, upward_displacement, 0.0),
            colliders,
            false,
        )
    } else {
        AuthoredWallMotionResult {
            position: previous,
            contact_normal: None,
        }
    };
    let mut contact_normal = upward.contact_normal;
    let mut collision_flags = if upward.contact_normal.is_some() {
        crate::movement::LEGACY_COLLISION_ABOVE
    } else {
        0
    };
    let side = if side_is_zero {
        AuthoredWallMotionResult {
            position: upward.position,
            contact_normal: None,
        }
    } else {
        resolve_authored_wall_motion_detailed_with_bounds(
            upward.position,
            side_displacement,
            colliders,
            false,
        )
    };
    if let Some(side_normal) = side.contact_normal {
        contact_normal = Some(side_normal);
        collision_flags |= crate::movement::LEGACY_COLLISION_SIDES;
    }
    // A ceiling can clamp the artificial UP pass. Undo only the height which
    // was actually gained, matching PhysX's `Delta < stepOffset` clamp.
    let applied_step_offset = if moving_up {
        0.0
    } else {
        (upward.position.y - previous.y).clamp(0.0, requested_step_offset)
    };
    let downward_displacement = if moving_up {
        0.0
    } else {
        submitted_vertical_displacement.min(0.0) - applied_step_offset
    };
    let downward = if downward_displacement.abs() > AUTHORED_COLLISION_EPSILON {
        resolve_authored_downward_motion_detailed_with_bounds(
            side.position,
            Vec3::new(0.0, downward_displacement, 0.0),
            colliders,
        )
    } else {
        AuthoredWallMotionResult {
            position: side.position,
            contact_normal: None,
        }
    };
    // The first CCT pass aborts when DOWN identifies a steep triangle whose
    // top lies above stepOffset. `Controller::move` then restores the original
    // capsule pose and repeats the *whole* move with STF_WALK_EXPERIMENT: the
    // artificial UP pass is skipped, SIDE runs directly from the original
    // height, and DOWN still includes the stepOffset recovery. This is what
    // prevents a narrow concavity from alternating between its two faces.
    if !moving_up
        && let Some(down_normal) = downward.contact_normal
        && authored_down_contact_is_non_walkable(
            downward.position,
            previous.y,
            down_normal,
            colliders,
        )
    {
        return resolve_authored_controller_walk_experiment_with_bounds(
            previous,
            side_displacement,
            submitted_vertical_displacement,
            requested_step_offset,
            colliders,
        );
    }
    if let Some(down_normal) = downward.contact_normal {
        contact_normal = Some(down_normal);
        collision_flags |= LEGACY_COLLISION_BELOW;
    }
    // PhysX runs the ordinary DOWN pass with maxIterDown=1. It advances the
    // vertical sweep to its first contact but does not consume the tangential
    // response calculated after that hit. Our general multi-plane solver is
    // intentionally iterative, so without this pass boundary it applied an
    // extra downhill XZ displacement and reduced speed on every walkable
    // slope. SIDE exclusively owns the horizontal result.
    let position = Vec3::new(side.position.x, downward.position.y, side.position.z);
    AuthoredControllerMotionResult {
        position,
        contact_normal,
        collision_flags,
    }
}

pub(super) fn resolve_authored_controller_walk_experiment_with_bounds(
    previous: Vec3,
    side_displacement: Vec3,
    submitted_vertical_displacement: f32,
    step_offset: f32,
    colliders: &[(Mat4, &AuthoredTriMeshCollider, &AuthoredColliderWorldBounds)],
) -> AuthoredControllerMotionResult {
    let side = if side_displacement.length_squared() <= AUTHORED_COLLISION_EPSILON.powi(2) {
        AuthoredWallMotionResult {
            position: previous,
            contact_normal: None,
        }
    } else {
        resolve_authored_wall_motion_detailed_with_bounds(
            previous,
            side_displacement,
            colliders,
            false,
        )
    };
    let mut contact_normal = side.contact_normal;
    let mut collision_flags = if side.contact_normal.is_some() {
        crate::movement::LEGACY_COLLISION_SIDES
    } else {
        0
    };
    // PhysX still subtracts stepOffset in WALK_EXPERIMENT even though that
    // mode deliberately skipped the matching artificial UP displacement.
    let downward_displacement = submitted_vertical_displacement.min(0.0) - step_offset;
    let downward = if downward_displacement.abs() > AUTHORED_COLLISION_EPSILON {
        resolve_authored_downward_motion_detailed_with_bounds(
            side.position,
            Vec3::new(0.0, downward_displacement, 0.0),
            colliders,
        )
    } else {
        AuthoredWallMotionResult {
            position: side.position,
            contact_normal: None,
        }
    };
    if let Some(down_normal) = downward.contact_normal {
        contact_normal = Some(down_normal);
        collision_flags |= LEGACY_COLLISION_BELOW;
    }
    AuthoredControllerMotionResult {
        position: Vec3::new(side.position.x, downward.position.y, side.position.z),
        contact_normal,
        collision_flags,
    }
}
