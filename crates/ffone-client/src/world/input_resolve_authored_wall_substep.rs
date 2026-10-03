use super::*;

pub(super) fn resolve_authored_wall_motion_with_bounds(
    previous: Vec3,
    displacement: Vec3,
    colliders: &[(Mat4, &AuthoredTriMeshCollider, &AuthoredColliderWorldBounds)],
) -> Vec3 {
    resolve_authored_wall_motion_detailed_with_bounds(previous, displacement, colliders, true)
        .position
}

pub(super) fn resolve_authored_wall_motion_detailed_with_bounds(
    previous: Vec3,
    displacement: Vec3,
    colliders: &[(Mat4, &AuthoredTriMeshCollider, &AuthoredColliderWorldBounds)],
    allow_step: bool,
) -> AuthoredWallMotionResult {
    resolve_authored_capsule_motion_detailed_with_bounds(
        previous,
        displacement,
        colliders,
        allow_step,
        AuthoredCapsuleResponse::Wall,
    )
}

pub(super) fn resolve_authored_downward_motion_detailed_with_bounds(
    previous: Vec3,
    displacement: Vec3,
    colliders: &[(Mat4, &AuthoredTriMeshCollider, &AuthoredColliderWorldBounds)],
) -> AuthoredWallMotionResult {
    resolve_authored_capsule_motion_detailed_with_bounds(
        previous,
        displacement,
        colliders,
        false,
        AuthoredCapsuleResponse::DownwardSweep,
    )
}

pub(super) fn resolve_authored_capsule_motion_detailed_with_bounds(
    previous: Vec3,
    displacement: Vec3,
    colliders: &[(Mat4, &AuthoredTriMeshCollider, &AuthoredColliderWorldBounds)],
    allow_step: bool,
    response: AuthoredCapsuleResponse,
) -> AuthoredWallMotionResult {
    let distance = displacement.length();
    if !previous.is_finite() || !displacement.is_finite() || !distance.is_finite() {
        return AuthoredWallMotionResult {
            position: previous,
            contact_normal: None,
        };
    }
    let required_substeps = ((distance / AUTHORED_COLLISION_MAX_SUBSTEP).ceil() as usize).max(1);
    // A corrupt velocity must never turn collision protection into an
    // unbounded frame stall or a teleport through the authored world.
    if required_substeps > AUTHORED_COLLISION_MAX_SUBSTEPS {
        return AuthoredWallMotionResult {
            position: previous,
            contact_normal: None,
        };
    }
    let collision_triangles =
        collect_authored_motion_triangles_with_bounds(previous, displacement, colliders);
    let substep = displacement / required_substeps as f32;
    let mut resolved = previous;
    let mut source_contact_normal = None;
    for _ in 0..required_substeps {
        let substep_result = resolve_authored_wall_substep(
            resolved,
            substep,
            collision_triangles.as_slice(),
            allow_step,
            response,
        );
        resolved = substep_result.position;
        retain_source_contact_normal(&mut source_contact_normal, substep_result.contact_normal);
        if response == AuthoredCapsuleResponse::DownwardSweep
            && substep_result.contact_normal.is_some()
        {
            break;
        }
    }
    AuthoredWallMotionResult {
        position: resolved,
        contact_normal: source_contact_normal,
    }
}

pub(super) fn resolve_authored_wall_motion(
    previous: Vec3,
    displacement: Vec3,
    colliders: &[(Mat4, &AuthoredTriMeshCollider)],
) -> Vec3 {
    let bounds = colliders
        .iter()
        .map(|(world_from_local, collider)| {
            let (minimum, maximum) = collider_world_bounds(collider, *world_from_local)
                .expect("test collider bounds must be finite");
            AuthoredColliderWorldBounds { minimum, maximum }
        })
        .collect::<Vec<_>>();
    let colliders = colliders
        .iter()
        .zip(&bounds)
        .map(|((world_from_local, collider), bounds)| (*world_from_local, *collider, bounds))
        .collect::<Vec<_>>();
    resolve_authored_wall_motion_with_bounds(previous, displacement, &colliders)
}

pub(super) fn collect_authored_motion_triangles_with_bounds(
    previous: Vec3,
    displacement: Vec3,
    colliders: &[(Mat4, &AuthoredTriMeshCollider, &AuthoredColliderWorldBounds)],
) -> Vec<AuthoredCollisionTriangle> {
    let current = previous + displacement;
    // Keep the streamed spatial-index query and this exact-bounds rejection on
    // one conservative contract so neither layer can discard a slide target.
    let (query_min, query_max) = authored_wall_spatial_query_bounds(previous, current);
    let mut triangles = Vec::new();
    for &(world_from_local, collider, bounds) in colliders {
        if !bounds.overlaps(query_min, query_max) {
            continue;
        }
        for triangle in collider.indices.chunks_exact(3) {
            let a = world_from_local.transform_point3(collider.vertices[triangle[0] as usize]);
            let b = world_from_local.transform_point3(collider.vertices[triangle[1] as usize]);
            let c = world_from_local.transform_point3(collider.vertices[triangle[2] as usize]);
            let Some(triangle) = AuthoredCollisionTriangle::new(a, b, c) else {
                continue;
            };
            let triangle = triangle.with_obstacle_maximum_y(bounds.maximum.y);
            if triangle.overlaps(query_min, query_max) {
                triangles.push(triangle);
            }
        }
    }
    triangles
}

#[cfg(test)]
pub(super) fn collect_authored_motion_triangles(
    previous: Vec3,
    displacement: Vec3,
    colliders: &[(Mat4, &AuthoredTriMeshCollider)],
) -> Vec<AuthoredCollisionTriangle> {
    let bounds = colliders
        .iter()
        .map(|(world_from_local, collider)| {
            let (minimum, maximum) = collider_world_bounds(collider, *world_from_local)
                .expect("test collider bounds must be finite");
            AuthoredColliderWorldBounds { minimum, maximum }
        })
        .collect::<Vec<_>>();
    let colliders = colliders
        .iter()
        .zip(&bounds)
        .map(|((world_from_local, collider), bounds)| (*world_from_local, *collider, bounds))
        .collect::<Vec<_>>();
    collect_authored_motion_triangles_with_bounds(previous, displacement, &colliders)
}

pub(super) fn resolve_authored_wall_substep(
    mut resolved: Vec3,
    displacement: Vec3,
    triangles: &[AuthoredCollisionTriangle],
    allow_step: bool,
    response: AuthoredCapsuleResponse,
) -> AuthoredWallMotionResult {
    let step_reference_y = resolved.y;
    let mut source_contact_normal = None;
    let initial_penetration = deepest_authored_capsule_penetration_with_response(
        resolved,
        step_reference_y,
        triangles,
        allow_step,
        response,
    );
    if let Some(push) = initial_penetration {
        retain_source_contact_normal(
            &mut source_contact_normal,
            authored_capsule_callback_normal(resolved, step_reference_y, triangles, allow_step)
                .or_else(|| Some(push.normalize_or_zero())),
        );
    }
    // Complete-capsule recovery is still required for finite triangle edges.
    // The original game's extra upward movement near an ordinary object is
    // applied on the following frame by HandleSurfaceSliding; recovery here
    // supplies the rounded CCT motion, while the callback path below retains
    // the actual ControllerColliderHit feature normal instead of inventing a
    // separate unconditional edge boost.
    resolved = recover_authored_capsule_penetration_with_response(
        resolved,
        step_reference_y,
        triangles,
        allow_step,
        response,
    );
    if response == AuthoredCapsuleResponse::DownwardSweep && initial_penetration.is_some() {
        return AuthoredWallMotionResult {
            position: resolved,
            contact_normal: source_contact_normal,
        };
    }
    let mut contact_normals = [Vec3::ZERO; AUTHORED_MAX_CONTACT_PLANES];
    let mut contact_count = collect_authored_capsule_wall_contacts(
        resolved,
        step_reference_y,
        triangles,
        allow_step,
        response,
        &mut contact_normals,
    );
    // A support point that begins a Move on a wall plane produces a t=0
    // intersection, which the segment test intentionally ignores. Preserve
    // that touching plane before resolving the new input so changing from W
    // to A/D at a mesh corner cannot discard the first face and step through
    // it while the adjacent face supplies a different slide normal.
    let mut remainder =
        clip_displacement_against_contacts(displacement, &contact_normals[..contact_count]);
    if !remainder.abs_diff_eq(displacement, AUTHORED_COLLISION_EPSILON)
        && let Some(normal) = contact_normals[..contact_count]
            .iter()
            .copied()
            .min_by(|left, right| left.dot(displacement).total_cmp(&right.dot(displacement)))
    {
        retain_source_contact_normal(
            &mut source_contact_normal,
            authored_capsule_callback_normal(resolved, step_reference_y, triangles, allow_step)
                .or(Some(normal)),
        );
    }
    if response == AuthoredCapsuleResponse::DownwardSweep
        && !remainder.abs_diff_eq(displacement, AUTHORED_COLLISION_EPSILON)
    {
        return AuthoredWallMotionResult {
            position: resolved,
            contact_normal: source_contact_normal,
        };
    }
    // CharacterController.Move can contact more than one plane in one frame.
    // Keep every distinct plane in a small contact manifold: resolving a later
    // face in isolation can reintroduce velocity through the first face, which
    // is the classic "hold into wall, strafe through its corner" failure.
    for _ in 0..8 {
        if remainder.length_squared() <= AUTHORED_COLLISION_EPSILON.powi(2) {
            break;
        }
        let target = resolved + remainder;
        let mut earliest: Option<(AuthoredSegmentHit, bool)> = None;
        if response == AuthoredCapsuleResponse::Wall {
            for &triangle in triangles {
                let Some(hit) = triangle_wall_hit(
                    triangle,
                    resolved,
                    target,
                    AUTHORED_CHARACTER_CONTROLLER_HEIGHT,
                ) else {
                    continue;
                };
                if earliest.is_none_or(|(current, _)| hit.fraction < current.fraction) {
                    earliest = Some((hit, false));
                }
            }
        }
        if let Some(hit) = authored_capsule_overlap_hit_with_response(
            resolved,
            target,
            step_reference_y,
            triangles,
            allow_step,
            response,
        ) && earliest.is_none_or(|(current, _)| hit.fraction < current.fraction)
        {
            earliest = Some((hit, true));
        }
        let Some((hit, transient_edge_contact)) = earliest else {
            resolved = target;
            break;
        };

        // Orient the authored triangle normal against movement, remove only
        // the inward component, and preserve the tangential component.
        let mut hit_normal = hit.normal.normalize_or_zero();
        if hit_normal == Vec3::ZERO {
            resolved = target;
            break;
        }
        if hit_normal.dot(remainder) > 0.0 {
            hit_normal = -hit_normal;
        }
        let callback_normal = transient_edge_contact
            .then(|| {
                authored_capsule_callback_normal(hit.point, step_reference_y, triangles, allow_step)
            })
            .flatten()
            .unwrap_or(hit_normal);
        retain_source_contact_normal(&mut source_contact_normal, Some(callback_normal));
        let travel_fraction = (hit.fraction - AUTHORED_COLLISION_EPSILON).max(0.0);
        let travelled = remainder * travel_fraction;
        resolved += travelled;
        if response == AuthoredCapsuleResponse::DownwardSweep {
            return AuthoredWallMotionResult {
                position: resolved,
                contact_normal: source_contact_normal,
            };
        }
        let untravelled = remainder - travelled;
        if transient_edge_contact {
            // The radial normal of a rounded capsule against a finite convex
            // edge changes continuously as it slides around that edge. Keeping
            // every old radial normal turns the contact cone into a false
            // corner and eventually projects motion to zero. Use the current
            // radial normal for this iteration while retaining real planar
            // wall contacts collected above.
            let persistent_contact_count = contact_count;
            insert_authored_contact_normal(&mut contact_normals, &mut contact_count, hit_normal);
            remainder =
                clip_displacement_against_contacts(untravelled, &contact_normals[..contact_count]);
            contact_count = persistent_contact_count;
        } else {
            insert_authored_contact_normal(&mut contact_normals, &mut contact_count, hit_normal);
            remainder =
                clip_displacement_against_contacts(untravelled, &contact_normals[..contact_count]);
        }
    }
    // A plane sweep alone cannot recover an avatar that begins a frame inside
    // a thin triangle, nor can it cover the finite edge/corner of a triangle
    // when the support point misses the face. Unity CharacterController
    // performs overlap recovery as part of Move, so resolve the deepest
    // capsule/triangle overlap after every conservative substep. Doing this
    // only at the final frame position allowed a fast capsule to enter and
    // leave a narrow edge's contact volume without ever observing it.
    if let Some(push) = deepest_authored_capsule_penetration_with_response(
        resolved,
        step_reference_y,
        triangles,
        allow_step,
        response,
    ) {
        retain_source_contact_normal(
            &mut source_contact_normal,
            authored_capsule_callback_normal(resolved, step_reference_y, triangles, allow_step)
                .or_else(|| Some(push.normalize_or_zero())),
        );
    }
    AuthoredWallMotionResult {
        position: recover_authored_capsule_penetration_with_response(
            resolved,
            step_reference_y,
            triangles,
            allow_step,
            response,
        ),
        contact_normal: source_contact_normal,
    }
}

pub(super) fn collect_authored_capsule_wall_contacts(
    feet: Vec3,
    step_reference_y: f32,
    triangles: &[AuthoredCollisionTriangle],
    allow_step: bool,
    response: AuthoredCapsuleResponse,
    contacts: &mut [Vec3; AUTHORED_MAX_CONTACT_PLANES],
) -> usize {
    let radius = AUTHORED_CHARACTER_CONTROLLER_RADIUS - AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH;
    let contact_radius = radius + AUTHORED_COLLISION_CONTACT_TOLERANCE;
    let capsule_start = feet + Vec3::Y * AUTHORED_CHARACTER_CONTROLLER_RADIUS;
    let capsule_end = feet
        + Vec3::Y * (AUTHORED_CHARACTER_CONTROLLER_HEIGHT - AUTHORED_CHARACTER_CONTROLLER_RADIUS);
    let query_min = feet - Vec3::splat(contact_radius);
    let query_max = feet
        + Vec3::new(
            contact_radius,
            AUTHORED_CHARACTER_CONTROLLER_HEIGHT + contact_radius,
            contact_radius,
        );
    let capsule_center = (capsule_start + capsule_end) * 0.5;
    let mut count = 0;
    for &triangle in triangles {
        if count == contacts.len()
            || !triangle.overlaps(query_min, query_max)
            || triangle.normal.y.abs() >= AUTHORED_WALKABLE_MIN_UP_DOT
            || (allow_step
                && triangle.obstacle_maximum_y
                    <= step_reference_y
                        + GROUNDED_STEP_UP
                        + AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH)
        {
            continue;
        }
        let (capsule_point, triangle_point) = closest_points_segment_triangle(
            capsule_start,
            capsule_end,
            triangle.a,
            triangle.b,
            triangle.c,
        );
        let delta = capsule_point - triangle_point;
        if !delta.is_finite()
            || delta.length_squared() > contact_radius * contact_radius
            || !authored_capsule_is_on_triangle_front_side(capsule_center, triangle)
        {
            continue;
        }
        // A finite wall ends in a rounded capsule/edge contact. Keeping its
        // horizontal face plane active above that endpoint prevents the lower
        // sphere from rolling over a reachable lip and makes success depend on
        // the exact jump frame. The transient overlap sweep below owns it.
        if delta.y > AUTHORED_COLLISION_EPSILON
            && triangle_point.y >= triangle.maximum.y - AUTHORED_COLLISION_CONTACT_TOLERANCE
        {
            continue;
        }
        let normal = if response == AuthoredCapsuleResponse::DownwardSweep {
            // DOWN needs the actual slope plane. Flattening this normal lets
            // gravity move through a steep face and prevents two opposing
            // faces in a small concavity from forming a supporting manifold.
            authored_front_side_contact_direction(delta, triangle.normal)
        } else {
            let horizontal_delta = Vec3::new(delta.x, 0.0, delta.z);
            let fallback = Vec3::new(triangle.normal.x, 0.0, triangle.normal.z).normalize_or_zero();
            authored_front_side_contact_direction(horizontal_delta, fallback)
        };
        insert_authored_contact_normal(contacts, &mut count, normal);
    }
    count
}

pub(super) fn resolve_authored_camera_occlusion(
    choreography: Option<
        Res<crate::tutorial_choreography_runtime::TutorialChoreographyPresentation>,
    >,
    spatial_index: Res<AuthoredColliderSpatialIndex>,
    colliders: Query<(
        Entity,
        &GlobalTransform,
        &AuthoredTriMeshCollider,
        &AuthoredColliderWorldBounds,
    )>,
    heightmaps: Query<(&GlobalTransform, &NativeHeightmapCollider)>,
    targets: Query<&Transform, (With<LegacyPlayerController>, Without<LegacyOrbitCamera>)>,
    mut cameras: Query<(&LegacyOrbitCamera, &mut Transform), Without<LegacyPlayerController>>,
) {
    // `cnPlayerCamera` replaces PositionUpdate with `pCustomControll` during
    // an event camera. Running the normal collision ray after the authored
    // coroutine camera has written its pose makes the two systems fight and
    // was the source of the visible cutscene shake/offset.
    if choreography.is_some_and(|presentation| {
        presentation.camera.mode != crate::tutorial_choreography::CameraMode::None
    }) {
        return;
    }
    let mut collider_candidates = Vec::new();
    for (camera, mut camera_transform) in &mut cameras {
        let Ok(target_transform) = targets.get(camera.target) else {
            continue;
        };
        let target = target_transform.translation + Vec3::Y * camera.height;
        let direction = (camera_transform.translation - target).normalize_or_zero();
        if direction == Vec3::ZERO {
            continue;
        }
        let ideal = target + direction * camera.distance;
        let mut nearest: Option<AuthoredSegmentHit> = None;
        let epsilon = Vec3::splat(AUTHORED_COLLISION_EPSILON);
        spatial_index.candidates(
            target.min(ideal) - epsilon,
            target.max(ideal) + epsilon,
            &mut collider_candidates,
        );
        for collider_entity in collider_candidates.iter().copied() {
            let Ok((_, global, collider, bounds)) = colliders.get(collider_entity) else {
                continue;
            };
            if collider.is_trigger {
                continue;
            }
            let Some(hit) = collider_segment_hit_with_bounds(
                collider,
                global.to_matrix(),
                bounds,
                target,
                ideal,
            ) else {
                continue;
            };
            if nearest.is_none_or(|current| hit.fraction < current.fraction) {
                nearest = Some(hit);
            }
        }
        // Unity's layer mask includes Terrain. Heightmaps are separate from
        // authored GLB MeshColliders in the native world, so omitting this
        // query let the camera pass below hills even though the avatar stood
        // on the matching TerrainCollider.
        for (global, heightmap) in &heightmaps {
            let Some(hit) = heightmap.segment_hit(global, target, ideal) else {
                continue;
            };
            let hit = AuthoredSegmentHit {
                fraction: hit.fraction,
                point: hit.point,
                normal: hit.normal,
            };
            if nearest.is_none_or(|current| hit.fraction < current.fraction) {
                nearest = Some(hit);
            }
        }
        let Some(mut hit) = nearest else {
            continue;
        };
        if hit.normal.dot(direction) > 0.0 {
            hit.normal = -hit.normal;
        }
        let hit_distance = target.distance(hit.point);
        let toward_target = (target - hit.point).normalize_or_zero();
        // Exact final correction from cnPlayerCamera.Update after Physics.Raycast.
        let correction =
            (1.0 - hit.normal.dot(toward_target)).clamp(0.0, (hit_distance - 0.1).max(0.0));
        camera_transform.translation = hit.point + toward_target * correction;
        camera_transform.look_at(target, Vec3::Y);
    }
}

pub(super) fn read_runtime_world_reference(
    asset_root: &Path,
    reference: &RuntimeWorldAssetReference,
    context: &str,
) -> Result<Vec<u8>, NativeWorldSceneError> {
    validate_relative_asset_path(&reference.path, "json")?;
    validate_blake3(&reference.blake3, context)?;
    let path = join_relative(asset_root, &reference.path);
    let bytes = fs::read(&path).map_err(|error| {
        NativeWorldSceneError::new(format!(
            "failed to read runtime {context} {}: {error}",
            path.display()
        ))
    })?;
    verify_file_blake3(&bytes, &reference.blake3, &path)?;
    Ok(bytes)
}

pub(super) fn load_runtime_environment(
    asset_root: &Path,
    reference: &RuntimeWorldAssetReference,
    entry: &RuntimeWorldRegistryEntry,
) -> Result<NativeTerrainEnvironment, NativeWorldSceneError> {
    let bytes = read_runtime_world_reference(asset_root, reference, "terrain environment")?;
    let environment: NativeTerrainEnvironment =
        serde_json::from_slice(&bytes).map_err(|error| {
            NativeWorldSceneError::new(format!(
                "invalid runtime terrain environment {:?}: {error}",
                reference.path
            ))
        })?;
    let expected_scope = match entry.scope {
        NativeWorldScope::WorldMap => "worldMap",
        NativeWorldScope::Tutorial => "tutorial",
    };
    let expected_tile_id = entry
        .id
        .strip_prefix("map_")
        .or_else(|| entry.id.strip_prefix("tile_"))
        .expect("validated runtime world id always has a stable scope prefix");
    if environment.schema != NATIVE_TERRAIN_ENVIRONMENT_SCHEMA
        || environment.status != "complete"
        || environment.scope != expected_scope
        || environment.tile_id != expected_tile_id
        || environment.ambience.status != "exactSource"
        || environment.ambience.grid_coordinates != entry.tile
    {
        return Err(NativeWorldSceneError::new(format!(
            "runtime terrain environment {:?} disagrees with registry identity",
            reference.path
        )));
    }
    Ok(environment)
}
