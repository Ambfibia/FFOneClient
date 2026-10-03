use super::*;

/// Resolves one player move against one collider that changed pose this frame.
///
/// For the source road cars the pose delta is translation plus a Y-axis turn.
/// Mapping the capsule endpoint through `previous * inverse(current)` makes
/// that a standard sweep against the previous triangle mesh: a stationary
/// avatar acquires the opposite relative velocity when the car approaches.
/// Mapping the resolved point forward again produces the kinematic push. If
/// no contact occurred the two mappings cancel, so nearby actors are never
/// carried by a vehicle they did not touch.
pub(super) fn resolve_runtime_authored_collider_motion(
    previous_feet: Vec3,
    current_feet: Vec3,
    collider: &AuthoredTriMeshCollider,
    current_bounds: &AuthoredColliderWorldBounds,
    motion: RuntimeAuthoredColliderMotion,
) -> Vec3 {
    let previous_world_from_local = motion.previous_world_from_local;
    let current_world_from_local = motion.current_world_from_local;
    if !previous_feet.is_finite()
        || !current_feet.is_finite()
        || !previous_world_from_local.is_finite()
        || !current_world_from_local.is_finite()
        || previous_world_from_local.abs_diff_eq(current_world_from_local, f32::EPSILON)
    {
        return current_feet;
    }
    // Reject distant runtime actors before transforming or collecting any
    // triangles. Endpoint AABBs alone do not bound the corner of a long car
    // during a sharp yaw. A sphere around the authored local origin does: its
    // radius contains every AABB corner under either endpoint scale, while
    // the two origin endpoints bound the source root's linear frame motion.
    let Some((previous_minimum, previous_maximum)) =
        collider_world_bounds(collider, previous_world_from_local)
    else {
        return current_feet;
    };
    let local_radius = collider
        .local_min
        .abs()
        .max(collider.local_max.abs())
        .length();
    let maximum_scale = [previous_world_from_local, current_world_from_local]
        .into_iter()
        .flat_map(|matrix| {
            [
                matrix.x_axis.truncate().length(),
                matrix.y_axis.truncate().length(),
                matrix.z_axis.truncate().length(),
            ]
        })
        .fold(0.0_f32, f32::max);
    let swept_radius = local_radius * maximum_scale;
    let previous_origin = previous_world_from_local.transform_point3(Vec3::ZERO);
    let current_origin = current_world_from_local.transform_point3(Vec3::ZERO);
    if !swept_radius.is_finite() || !previous_origin.is_finite() || !current_origin.is_finite() {
        return current_feet;
    }
    let rotation_padding = Vec3::splat(swept_radius);
    let swept_minimum = previous_minimum
        .min(current_bounds.minimum)
        .min(previous_origin.min(current_origin) - rotation_padding);
    let swept_maximum = previous_maximum
        .max(current_bounds.maximum)
        .max(previous_origin.max(current_origin) + rotation_padding);
    let radius = AUTHORED_CHARACTER_CONTROLLER_RADIUS - AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH;
    let player_minimum = previous_feet.min(current_feet) - Vec3::new(radius, radius, radius);
    let player_maximum = previous_feet.max(current_feet)
        + Vec3::new(
            radius,
            AUTHORED_CHARACTER_CONTROLLER_HEIGHT + radius,
            radius,
        );
    if !aabb_overlaps(swept_minimum, swept_maximum, player_minimum, player_maximum) {
        return current_feet;
    }

    // Matrix inversion and triangle collection are intentionally below the
    // swept broad phase: Downtown can have roughly a hundred live traffic
    // actors, while only the one touching this player needs narrow phase.
    let current_from_world = current_world_from_local.inverse();
    let previous_from_world = previous_world_from_local.inverse();
    if !current_from_world.is_finite() || !previous_from_world.is_finite() {
        return current_feet;
    }

    let previous_from_current = previous_world_from_local * current_from_world;
    let current_from_previous = current_world_from_local * previous_from_world;
    let relative_target = previous_from_current.transform_point3(current_feet);
    if !relative_target.is_finite() {
        return current_feet;
    }
    let previous_pose_colliders = [(previous_world_from_local, collider)];
    let resolved_previous = resolve_authored_wall_motion(
        previous_feet,
        relative_target - previous_feet,
        &previous_pose_colliders,
    );
    let resolved_current = current_from_previous.transform_point3(resolved_previous);
    if !resolved_current.is_finite() {
        return current_feet;
    }

    // Finish against the exact current triangles. This handles curved/turned
    // paths and numerical skin-width overlap while retaining the ordinary
    // front-face contract: entry is blocked, but an avatar already behind a
    // face can still walk back out instead of being trapped in the vehicle.
    let current_pose_colliders = [(current_world_from_local, collider)];
    resolve_authored_wall_motion(resolved_current, Vec3::ZERO, &current_pose_colliders)
}

pub(super) fn authored_capsule_is_on_triangle_front_side(
    capsule_center: Vec3,
    triangle: AuthoredCollisionTriangle,
) -> bool {
    // PhysX CCT does not request eMESH_BOTH_SIDES for its triangle sweeps.
    // Retain that authored winding contract here: an already-intersecting
    // controller can leave through a back face, while a controller on the
    // front side is still conservatively stopped before entering the mesh.
    (capsule_center - triangle.a).dot(triangle.normal) >= -AUTHORED_COLLISION_CONTACT_TOLERANCE
}

// Retained as a narrow regression oracle for the superseded wall-only solver.
// Production collision uses the complete 3D candidate-triangle capsule solver.
#[cfg(test)]
pub(super) fn collider_wall_penetration(
    collider: &AuthoredTriMeshCollider,
    world_from_local: Mat4,
    feet: Vec3,
) -> Option<Vec3> {
    let radius = AUTHORED_CHARACTER_CONTROLLER_RADIUS - AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH;
    if !collider_world_bounds_overlap(
        collider,
        world_from_local,
        feet - Vec3::new(radius, 0.0, radius),
        feet + Vec3::new(radius, AUTHORED_CHARACTER_CONTROLLER_HEIGHT, radius),
    ) {
        return None;
    }
    let mut deepest = Vec3::ZERO;
    for triangle in collider.indices.chunks_exact(3) {
        let a = world_from_local.transform_point3(collider.vertices[triangle[0] as usize]);
        let b = world_from_local.transform_point3(collider.vertices[triangle[1] as usize]);
        let c = world_from_local.transform_point3(collider.vertices[triangle[2] as usize]);
        // Match the sweep probes and Unity CharacterController stepOffset:
        // a vertical riser which ends below the 0.3 m step (plus skin) is not
        // a wall. Ground resolution lifts the capsule onto its upper face.
        // Treating it as penetration made the player snag and jitter on small
        // authored lips even though the controller is the exact 1.6/0.3
        // capsule.
        if a.y.max(b.y).max(c.y)
            <= feet.y + GROUNDED_STEP_UP + AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH
        {
            continue;
        }
        let normal = (b - a).cross(c - a).normalize_or_zero();
        if !normal.is_finite() || normal.y.abs() >= AUTHORED_WALKABLE_MIN_UP_DOT {
            continue;
        }
        let horizontal_normal = Vec3::new(normal.x, 0.0, normal.z).normalize_or_zero();
        if horizontal_normal == Vec3::ZERO {
            continue;
        }
        let capsule_start = feet + Vec3::Y * AUTHORED_CHARACTER_CONTROLLER_RADIUS;
        let capsule_end = feet
            + Vec3::Y
                * (AUTHORED_CHARACTER_CONTROLLER_HEIGHT - AUTHORED_CHARACTER_CONTROLLER_RADIUS);
        let (capsule_point, triangle_point) =
            closest_points_segment_triangle(capsule_start, capsule_end, a, b, c);
        let delta = capsule_point - triangle_point;
        let distance = delta.length();
        let capsule_center = (capsule_start + capsule_end) * 0.5;
        if !distance.is_finite()
            || distance >= radius
            || (capsule_center - a).dot(normal) < -AUTHORED_COLLISION_CONTACT_TOLERANCE
        {
            continue;
        }
        let horizontal_delta = Vec3::new(delta.x, 0.0, delta.z);
        let horizontal_distance = horizontal_delta.length();
        let allowed_horizontal_distance = (radius * radius - delta.y * delta.y).max(0.0).sqrt();
        let push_distance =
            allowed_horizontal_distance - horizontal_distance + AUTHORED_COLLISION_EPSILON;
        if push_distance <= AUTHORED_COLLISION_EPSILON {
            continue;
        }
        let direction = authored_front_side_contact_direction(horizontal_delta, horizontal_normal);
        let push = direction * push_distance;
        if push.length_squared() > deepest.length_squared() {
            deepest = push;
        }
    }
    (deepest != Vec3::ZERO).then_some(deepest)
}

pub(super) fn closest_points_segment_triangle(
    segment_start: Vec3,
    segment_end: Vec3,
    a: Vec3,
    b: Vec3,
    c: Vec3,
) -> (Vec3, Vec3) {
    if let Some(hit) = segment_triangle_hit(segment_start, segment_end, a, b, c) {
        return (hit.point, hit.point);
    }
    let mut best = (
        segment_start,
        closest_point_on_triangle(segment_start, a, b, c),
    );
    let endpoint = (segment_end, closest_point_on_triangle(segment_end, a, b, c));
    if endpoint.0.distance_squared(endpoint.1) < best.0.distance_squared(best.1) {
        best = endpoint;
    }
    for (edge_start, edge_end) in [(a, b), (b, c), (c, a)] {
        let candidate =
            closest_points_between_segments(segment_start, segment_end, edge_start, edge_end);
        if candidate.0.distance_squared(candidate.1) < best.0.distance_squared(best.1) {
            best = candidate;
        }
    }
    best
}

pub(super) fn closest_point_on_triangle(point: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    let ab = b - a;
    let ac = c - a;
    let ap = point - a;
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }

    let bp = point - b;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let weight = d1 / (d1 - d3);
        return a + ab * weight;
    }

    let cp = point - c;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let weight = d2 / (d2 - d6);
        return a + ac * weight;
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && d4 - d3 >= 0.0 && d5 - d6 >= 0.0 {
        let weight = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return b + (c - b) * weight;
    }
    let denominator = (va + vb + vc).recip();
    a + ab * (vb * denominator) + ac * (vc * denominator)
}

#[cfg(test)]
pub(super) fn collider_wall_hit(
    collider: &AuthoredTriMeshCollider,
    world_from_local: Mat4,
    previous_feet: Vec3,
    current_feet: Vec3,
    height: f32,
) -> Option<AuthoredSegmentHit> {
    let mut earliest: Option<AuthoredSegmentHit> = None;
    let probe_radius =
        AUTHORED_CHARACTER_CONTROLLER_RADIUS - AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH;
    let minimum_x = previous_feet.x.min(current_feet.x) - probe_radius;
    let maximum_x = previous_feet.x.max(current_feet.x) + probe_radius;
    let minimum_z = previous_feet.z.min(current_feet.z) - probe_radius;
    let maximum_z = previous_feet.z.max(current_feet.z) + probe_radius;
    let minimum_y = previous_feet.y;
    let maximum_y = previous_feet.y + height;
    if !collider_world_bounds_overlap(
        collider,
        world_from_local,
        Vec3::new(minimum_x, minimum_y, minimum_z),
        Vec3::new(maximum_x, maximum_y, maximum_z),
    ) {
        return None;
    }
    for triangle in collider.indices.chunks_exact(3) {
        let a = world_from_local.transform_point3(collider.vertices[triangle[0] as usize]);
        let b = world_from_local.transform_point3(collider.vertices[triangle[1] as usize]);
        let c = world_from_local.transform_point3(collider.vertices[triangle[2] as usize]);
        let Some(triangle) = AuthoredCollisionTriangle::new(a, b, c) else {
            continue;
        };
        let Some(hit) = triangle_wall_hit(triangle, previous_feet, current_feet, height) else {
            continue;
        };
        if earliest.is_none_or(|current| hit.fraction < current.fraction) {
            earliest = Some(hit);
        }
    }
    earliest
}

pub(super) fn triangle_wall_hit(
    triangle: AuthoredCollisionTriangle,
    previous_feet: Vec3,
    current_feet: Vec3,
    height: f32,
) -> Option<AuthoredSegmentHit> {
    let probe_radius =
        AUTHORED_CHARACTER_CONTROLLER_RADIUS - AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH;
    let travel = current_feet - previous_feet;
    let query_min = Vec3::new(
        previous_feet.x.min(current_feet.x) - probe_radius,
        previous_feet.y,
        previous_feet.z.min(current_feet.z) - probe_radius,
    );
    let query_max = Vec3::new(
        previous_feet.x.max(current_feet.x) + probe_radius,
        previous_feet.y + height,
        previous_feet.z.max(current_feet.z) + probe_radius,
    );
    if !triangle.overlaps(query_min, query_max)
        || triangle.normal.y.abs() >= AUTHORED_WALKABLE_MIN_UP_DOT
    {
        return None;
    }
    let horizontal_normal =
        Vec3::new(triangle.normal.x, 0.0, triangle.normal.z).normalize_or_zero();
    if horizontal_normal == Vec3::ZERO || triangle.normal.dot(travel) >= -AUTHORED_COLLISION_EPSILON
    {
        return None;
    }
    // Sweep the cylinder support point toward this triangle's plane.
    // Offsetting only in the movement direction (the old three-ray
    // approximation) underestimates radius on oblique walls and creates gaps
    // at authored boundary corners.
    let contact_offset = -horizontal_normal * probe_radius;
    let minimum_blocking_height =
        (GROUNDED_STEP_UP + AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH).min(height);
    let top_cylinder_height =
        (height - AUTHORED_CHARACTER_CONTROLLER_RADIUS).max(minimum_blocking_height);
    let mut earliest: Option<AuthoredSegmentHit> = None;
    for probe_height in [
        minimum_blocking_height,
        height * 0.4,
        height * 0.6,
        top_cylinder_height,
    ] {
        let offset = Vec3::Y * probe_height.clamp(0.0, height) + contact_offset;
        let Some(mut hit) = segment_triangle_hit(
            previous_feet + offset,
            current_feet + offset,
            triangle.a,
            triangle.b,
            triangle.c,
        ) else {
            continue;
        };
        // This analytic branch represents the cylindrical wall face. Vertical
        // floor/ceiling response comes from the complete capsule overlap sweep,
        // so never let a steep triangle inject lift here.
        hit.normal = horizontal_normal;
        if earliest.is_none_or(|current| hit.fraction < current.fraction) {
            earliest = Some(hit);
        }
    }
    earliest
}

pub(super) fn collider_segment_hit(
    collider: &AuthoredTriMeshCollider,
    world_from_local: Mat4,
    start: Vec3,
    end: Vec3,
) -> Option<AuthoredSegmentHit> {
    let (minimum, maximum) = collider_world_bounds(collider, world_from_local)?;
    collider_segment_hit_with_bounds(
        collider,
        world_from_local,
        &AuthoredColliderWorldBounds { minimum, maximum },
        start,
        end,
    )
}

pub(crate) fn collider_segment_hit_with_bounds(
    collider: &AuthoredTriMeshCollider,
    world_from_local: Mat4,
    bounds: &AuthoredColliderWorldBounds,
    start: Vec3,
    end: Vec3,
) -> Option<AuthoredSegmentHit> {
    let epsilon = Vec3::splat(AUTHORED_COLLISION_EPSILON);
    if !bounds.overlaps(start.min(end) - epsilon, start.max(end) + epsilon) {
        return None;
    }
    let mut nearest: Option<AuthoredSegmentHit> = None;
    for triangle in collider.indices.chunks_exact(3) {
        let a = world_from_local.transform_point3(collider.vertices[triangle[0] as usize]);
        let b = world_from_local.transform_point3(collider.vertices[triangle[1] as usize]);
        let c = world_from_local.transform_point3(collider.vertices[triangle[2] as usize]);
        let Some(hit) = segment_triangle_hit(start, end, a, b, c) else {
            continue;
        };
        if nearest.is_none_or(|current| hit.fraction < current.fraction) {
            nearest = Some(hit);
        }
    }
    nearest
}

/// Line-of-sight query shared by camera and combat acquisition. Trigger
/// volumes never occlude, matching the legacy physics-layer raycasts.
#[must_use]
pub fn authored_collider_blocks_segment(
    collider: &AuthoredTriMeshCollider,
    global: &GlobalTransform,
    start: Vec3,
    end: Vec3,
) -> bool {
    !collider.is_trigger && collider_segment_hit(collider, global.to_matrix(), start, end).is_some()
}

/// Cached-bounds variant used by dense ordinary-world targeting.
#[must_use]
pub fn authored_collider_blocks_segment_with_bounds(
    collider: &AuthoredTriMeshCollider,
    global: &GlobalTransform,
    bounds: &AuthoredColliderWorldBounds,
    start: Vec3,
    end: Vec3,
) -> bool {
    !collider.is_trigger
        && collider_segment_hit_with_bounds(collider, global.to_matrix(), bounds, start, end)
            .is_some()
}

pub(super) fn segment_triangle_hit(
    start: Vec3,
    end: Vec3,
    a: Vec3,
    b: Vec3,
    c: Vec3,
) -> Option<AuthoredSegmentHit> {
    if !start.is_finite() || !end.is_finite() || !a.is_finite() || !b.is_finite() || !c.is_finite()
    {
        return None;
    }
    let direction = end - start;
    let edge_ab = b - a;
    let edge_ac = c - a;
    let cross = direction.cross(edge_ac);
    let determinant = edge_ab.dot(cross);
    if determinant.abs() <= AUTHORED_COLLISION_EPSILON {
        return None;
    }
    let inverse = determinant.recip();
    let from_a = start - a;
    let u = from_a.dot(cross) * inverse;
    if !(-AUTHORED_COLLISION_EPSILON..=1.0 + AUTHORED_COLLISION_EPSILON).contains(&u) {
        return None;
    }
    let q = from_a.cross(edge_ab);
    let v = direction.dot(q) * inverse;
    if v < -AUTHORED_COLLISION_EPSILON || u + v > 1.0 + AUTHORED_COLLISION_EPSILON {
        return None;
    }
    let fraction = edge_ac.dot(q) * inverse;
    if !(AUTHORED_COLLISION_EPSILON..=1.0).contains(&fraction) {
        return None;
    }
    let normal = edge_ab.cross(edge_ac).normalize_or_zero();
    (normal != Vec3::ZERO).then_some(AuthoredSegmentHit {
        fraction,
        point: start + direction * fraction,
        normal,
    })
}

pub(crate) fn collider_ground_height(
    collider: &AuthoredTriMeshCollider,
    world_from_local: Mat4,
    x: f32,
    z: f32,
    minimum_y: f32,
    maximum_y: f32,
) -> Option<f32> {
    let (minimum, maximum) = collider_world_bounds(collider, world_from_local)?;
    collider_ground_height_with_bounds(
        collider,
        world_from_local,
        &AuthoredColliderWorldBounds { minimum, maximum },
        x,
        z,
        minimum_y,
        maximum_y,
    )
}

pub fn collider_ground_height_with_bounds(
    collider: &AuthoredTriMeshCollider,
    world_from_local: Mat4,
    bounds: &AuthoredColliderWorldBounds,
    x: f32,
    z: f32,
    minimum_y: f32,
    maximum_y: f32,
) -> Option<f32> {
    collider_ground_contact_with_bounds(
        collider,
        world_from_local,
        bounds,
        x,
        z,
        minimum_y,
        maximum_y,
    )
    .map(|(height, _)| height)
}

pub(super) fn collider_ground_contact_with_bounds(
    collider: &AuthoredTriMeshCollider,
    world_from_local: Mat4,
    bounds: &AuthoredColliderWorldBounds,
    x: f32,
    z: f32,
    minimum_y: f32,
    maximum_y: f32,
) -> Option<(f32, Vec3)> {
    let epsilon = Vec3::splat(GROUND_EPSILON);
    if !bounds.overlaps(
        Vec3::new(x, minimum_y, z) - epsilon,
        Vec3::new(x, maximum_y, z) + epsilon,
    ) {
        return None;
    }
    let mut highest: Option<(f32, Vec3)> = None;
    for triangle in collider.indices.chunks_exact(3) {
        let a = world_from_local.transform_point3(collider.vertices[triangle[0] as usize]);
        let b = world_from_local.transform_point3(collider.vertices[triangle[1] as usize]);
        let c = world_from_local.transform_point3(collider.vertices[triangle[2] as usize]);
        let normal = (b - a).cross(c - a).normalize_or_zero();
        if !normal.is_finite() || normal.y < AUTHORED_WALKABLE_MIN_UP_DOT {
            continue;
        }
        let Some(height) = triangle_height_at_xz(a, b, c, x, z) else {
            continue;
        };
        if height < minimum_y - GROUND_EPSILON || height > maximum_y + GROUND_EPSILON {
            continue;
        }
        if highest.is_none_or(|(current, _)| height > current) {
            highest = Some((height, normal));
        }
    }
    highest
}

/// Returns the `Below` contact produced by the lower sphere of the serialized
/// CharacterController. The support is the upper envelope of the triangle
/// dilated by the controller's skin-shrunk radius: it may belong to a face or
/// to one of its finite edges.
pub(super) fn collider_capsule_ground_contact_with_bounds(
    collider: &AuthoredTriMeshCollider,
    world_from_local: Mat4,
    bounds: &AuthoredColliderWorldBounds,
    x: f32,
    z: f32,
    step_reference_y: f32,
    minimum_y: f32,
    maximum_y: f32,
) -> Option<(f32, Vec3)> {
    let sweep_radius =
        AUTHORED_CHARACTER_CONTROLLER_RADIUS - AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH;
    let query_minimum = Vec3::new(x - sweep_radius, minimum_y - sweep_radius, z - sweep_radius);
    let query_maximum = Vec3::new(x + sweep_radius, maximum_y + sweep_radius, z + sweep_radius);
    if !bounds.overlaps(query_minimum, query_maximum) {
        return None;
    }

    let mut highest: Option<(f32, Vec3)> = None;
    let root_xz = Vec2::new(x, z);
    for triangle in collider.indices.chunks_exact(3) {
        let a = world_from_local.transform_point3(collider.vertices[triangle[0] as usize]);
        let b = world_from_local.transform_point3(collider.vertices[triangle[1] as usize]);
        let c = world_from_local.transform_point3(collider.vertices[triangle[2] as usize]);
        let authored_normal = (b - a).cross(c - a).normalize_or_zero();
        if !authored_normal.is_finite() || authored_normal.y <= 0.0 {
            continue;
        }
        let face_is_walkable = authored_normal.y >= AUTHORED_WALKABLE_MIN_UP_DOT;

        // `m_Center.y - m_Height/2 == 0`, so the transform is the feet
        // convention and the lower sphere center is one serialized radius
        // above it. On a slope, placing those feet directly at the vertical
        // triangle height embeds the skin-shrunk sphere in its own support.
        // The subsequent overlap recovery then has a horizontal component and
        // repeatedly pushes an idle controller downhill. Unity's controller
        // rests where the shrunken sphere is tangent to the face instead.
        if face_is_walkable {
            let plane_height = a.y
                - (authored_normal.x * (x - a.x) + authored_normal.z * (z - a.z))
                    / authored_normal.y;
            let Some(face_root_height) = capsule_face_root_height(plane_height, authored_normal)
            else {
                continue;
            };
            let lower_sphere_center = Vec3::new(
                x,
                face_root_height + AUTHORED_CHARACTER_CONTROLLER_RADIUS,
                z,
            );
            let face_distance = authored_normal.dot(lower_sphere_center - a);
            let face_point = lower_sphere_center - authored_normal * face_distance;
            let face_belongs_to_triangle =
                triangle_height_at_xz(a, b, c, face_point.x, face_point.z)
                    .is_some_and(|height| (height - face_point.y).abs() <= GROUND_EPSILON);
            if face_belongs_to_triangle
                && face_root_height >= minimum_y - GROUND_EPSILON
                && face_root_height <= maximum_y + GROUND_EPSILON
                && highest.is_none_or(|(current, _)| face_root_height > current)
            {
                highest = Some((face_root_height, authored_normal));
            }
        }

        // PhysX tests a static triangle's geometric slope only after its DOWN
        // sweep has produced a contact. A steep triangle is non-walkable when
        // its highest vertex rises by more than stepOffset from the original
        // controller bottom. A smaller feature is retained, and the public
        // ControllerColliderHit reports the capsule/edge contact normal rather
        // than that steep geometric normal. This is what lets the serialized
        // controller settle in small branch dents and push off again. Applying
        // the slope filter before the finite-edge query discarded that upward
        // radial normal and left only EnvironmentCollision's forced 2 m/s
        // slide.
        let feature_maximum_y = a.y.max(b.y).max(c.y);
        let edge_can_support = face_is_walkable
            || feature_maximum_y
                <= step_reference_y + GROUNDED_STEP_UP + AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH;
        if !edge_can_support {
            continue;
        }

        let edge_point = closest_point_on_triangle_edges_xz(root_xz, a, b, c);
        let horizontal_delta = root_xz - Vec2::new(edge_point.x, edge_point.z);
        let horizontal_distance_squared = horizontal_delta.length_squared();
        if !horizontal_distance_squared.is_finite()
            || horizontal_distance_squared >= sweep_radius * sweep_radius
        {
            continue;
        }
        let vertical_separation =
            (sweep_radius * sweep_radius - horizontal_distance_squared).sqrt();
        let contact_delta = Vec3::new(horizontal_delta.x, vertical_separation, horizontal_delta.y);
        let contact_normal = contact_delta / sweep_radius;
        if contact_normal.y < AUTHORED_WALKABLE_MIN_UP_DOT
            || contact_delta.dot(authored_normal) < -AUTHORED_COLLISION_CONTACT_TOLERANCE
        {
            continue;
        }

        let root_height = edge_point.y - AUTHORED_CHARACTER_CONTROLLER_RADIUS + vertical_separation;
        if root_height < minimum_y - GROUND_EPSILON || root_height > maximum_y + GROUND_EPSILON {
            continue;
        }
        if highest.is_none_or(|(current, _)| root_height > current) {
            highest = Some((root_height, contact_normal));
        }
    }
    highest
}

pub(super) fn closest_point_on_triangle_edges_xz(point: Vec2, a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    [
        closest_point_on_segment_xz(point, a, b),
        closest_point_on_segment_xz(point, b, c),
        closest_point_on_segment_xz(point, c, a),
    ]
    .into_iter()
    .min_by(|left, right| {
        let left_distance = Vec2::new(left.x, left.z).distance_squared(point);
        let right_distance = Vec2::new(right.x, right.z).distance_squared(point);
        left_distance.total_cmp(&right_distance)
    })
    .unwrap_or(a)
}

#[cfg(test)]
pub(super) fn collider_world_bounds_overlap(
    collider: &AuthoredTriMeshCollider,
    world_from_local: Mat4,
    query_min: Vec3,
    query_max: Vec3,
) -> bool {
    collider_world_bounds(collider, world_from_local)
        .is_some_and(|(minimum, maximum)| aabb_overlaps(minimum, maximum, query_min, query_max))
}

pub(super) fn collider_world_bounds(
    collider: &AuthoredTriMeshCollider,
    world_from_local: Mat4,
) -> Option<(Vec3, Vec3)> {
    let mut world_min = Vec3::splat(f32::INFINITY);
    let mut world_max = Vec3::splat(f32::NEG_INFINITY);
    for x in [collider.local_min.x, collider.local_max.x] {
        for y in [collider.local_min.y, collider.local_max.y] {
            for z in [collider.local_min.z, collider.local_max.z] {
                let corner = world_from_local.transform_point3(Vec3::new(x, y, z));
                world_min = world_min.min(corner);
                world_max = world_max.max(corner);
            }
        }
    }
    (world_min.is_finite() && world_max.is_finite()).then_some((world_min, world_max))
}

pub(super) fn triangle_height_at_xz(a: Vec3, b: Vec3, c: Vec3, x: f32, z: f32) -> Option<f32> {
    if !a.is_finite() || !b.is_finite() || !c.is_finite() || !x.is_finite() || !z.is_finite() {
        return None;
    }
    let denominator = (b.z - c.z) * (a.x - c.x) + (c.x - b.x) * (a.z - c.z);
    if !denominator.is_finite() || denominator.abs() <= f32::EPSILON {
        return None;
    }
    let weight_a = ((b.z - c.z) * (x - c.x) + (c.x - b.x) * (z - c.z)) / denominator;
    let weight_b = ((c.z - a.z) * (x - c.x) + (a.x - c.x) * (z - c.z)) / denominator;
    let weight_c = 1.0 - weight_a - weight_b;
    let tolerance = -0.000_01;
    if weight_a < tolerance || weight_b < tolerance || weight_c < tolerance {
        return None;
    }
    let height = weight_a * a.y + weight_b * b.y + weight_c * c.y;
    height.is_finite().then_some(height)
}

pub(super) struct PreparedGroundTriangle {
    pub(super) a: Vec3,
    pub(super) b: Vec3,
    pub(super) c: Vec3,
}

impl PreparedGroundTriangle {
    pub(super) fn may_hit_cell(&self, minimum: Vec2, maximum: Vec2) -> bool {
        let Self { a, b, c } = *self;
        let denominator = (b.z - c.z) * (a.x - c.x) + (c.x - b.x) * (a.z - c.z);
        if !denominator.is_finite() || denominator.abs() <= f32::EPSILON {
            return false;
        }
        let x = GroundInterval {
            low: minimum.x,
            high: maximum.x,
        }
        .sub(GroundInterval::constant(c.x));
        let z = GroundInterval {
            low: minimum.y,
            high: maximum.y,
        }
        .sub(GroundInterval::constant(c.z));
        let wa = x.mul(b.z - c.z).add(z.mul(c.x - b.x)).div(denominator);
        let wb = x.mul(c.z - a.z).add(z.mul(a.x - c.x)).div(denominator);
        let wc = GroundInterval::constant(1.0).sub(wa).sub(wb);
        let tolerance = -0.000_01;
        wa.high >= tolerance && wb.high >= tolerance && wc.high >= tolerance
    }
}

pub(super) struct GroundTriangleCell {
    pub(super) minimum: Vec2,
    pub(super) maximum: Vec2,
    pub(super) triangles: Vec<usize>,
}
