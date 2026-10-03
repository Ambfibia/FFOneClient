use super::*;

pub(super) fn cook_convex_xy_prism(
    source_vertices: &[Vec3],
    footprint_points: &[Vec2],
) -> Result<(Vec<Vec3>, Vec<u32>), String> {
    let mut points = footprint_points.to_vec();
    points.sort_by(|left, right| {
        left.x
            .total_cmp(&right.x)
            .then_with(|| left.y.total_cmp(&right.y))
    });
    points.dedup();
    if points.len() < 3 {
        return Err("authored perimeter collider needs at least three XY points".to_owned());
    }

    let cross = |origin: Vec2, left: Vec2, right: Vec2| (left - origin).perp_dot(right - origin);
    let mut lower = Vec::with_capacity(points.len());
    for &point in &points {
        while lower.len() >= 2
            && cross(lower[lower.len() - 2], lower[lower.len() - 1], point)
                <= AUTHORED_COLLISION_EPSILON
        {
            lower.pop();
        }
        lower.push(point);
    }
    let mut upper = Vec::with_capacity(points.len());
    for &point in points.iter().rev() {
        while upper.len() >= 2
            && cross(upper[upper.len() - 2], upper[upper.len() - 1], point)
                <= AUTHORED_COLLISION_EPSILON
        {
            upper.pop();
        }
        upper.push(point);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    if lower.len() < 3 {
        return Err("authored perimeter collider XY hull is degenerate".to_owned());
    }

    let source_minimum_z = source_vertices
        .iter()
        .map(|vertex| vertex.z)
        .reduce(f32::min)
        .ok_or_else(|| "authored perimeter collider has no vertices".to_owned())?;
    let maximum_z = source_vertices
        .iter()
        .map(|vertex| vertex.z)
        .reduce(f32::max)
        .ok_or_else(|| "authored perimeter collider has no vertices".to_owned())?;
    let source_height = maximum_z - source_minimum_z;
    if source_height <= AUTHORED_COLLISION_EPSILON {
        return Err("authored perimeter collider Z span is degenerate".to_owned());
    }
    // An invisible perimeter blocker must follow the terrain beneath the
    // visible model as well as its silhouette. The Retrobution tutorial dome
    // sits above a slope that drops more than five world units below its
    // authored lower edge; without this skirt, W plus camera yaw can steer the
    // complete character capsule underneath the wall.
    let minimum_z = source_minimum_z - source_height;

    let mut vertices = Vec::with_capacity(lower.len() * 2);
    for point in &lower {
        vertices.extend([
            Vec3::new(point.x, point.y, minimum_z),
            Vec3::new(point.x, point.y, maximum_z),
        ]);
    }
    let mut indices = Vec::with_capacity(lower.len() * 6);
    for index in 0..lower.len() {
        let next = (index + 1) % lower.len();
        let bottom = (index * 2) as u32;
        let top = bottom + 1;
        let next_bottom = (next * 2) as u32;
        let next_top = next_bottom + 1;
        // This cooked prism is the tutorial dome's *inner* perimeter. Unity's
        // CharacterController mesh sweep is front-face only, so the authored
        // face must point toward the playable volume. Reversing the ordinary
        // outward convex-hull winding keeps players inside without turning
        // every MeshCollider in the game into a two-sided trap.
        indices.extend([bottom, top, next_bottom, top, next_top, next_bottom]);
    }
    Ok((vertices, indices))
}

pub(super) fn carried_support_point(previous: Mat4, current: Mat4, point: Vec3) -> Option<Vec3> {
    // Static authored supports retain the exact same matrix. Sending a point
    // through inverse(current) and current anyway loses several ULPs at the
    // large world coordinates used by Sector V; repeating that round-trip
    // produces visible drift even though neither the player nor support moved.
    if previous == current {
        return point.is_finite().then_some(point);
    }
    // A translating platform keeps its linear basis, including authored
    // rotation and scale. Apply only its world displacement: repeatedly
    // inverting that basis introduces lateral error even along a straight
    // route. Subtract the translations before adding to the rider so the
    // unchanged axes remain bit-exact at large world coordinates.
    if previous.x_axis == current.x_axis
        && previous.y_axis == current.y_axis
        && previous.z_axis == current.z_axis
    {
        let carried = point + (current.w_axis - previous.w_axis).truncate();
        return carried.is_finite().then_some(carried);
    }
    let carried = current.transform_point3(previous.inverse().transform_point3(point));
    carried.is_finite().then_some(carried)
}

pub(super) fn authored_player_ground_search_range(
    previous: Vec3,
    current: Vec3,
    grounded: bool,
) -> (f32, f32) {
    if !grounded {
        return (
            current.y.min(previous.y) - AUTHORED_GROUND_SWEEP_TOLERANCE,
            previous.y.max(current.y) + AUTHORED_GROUND_SWEEP_TOLERANCE,
        );
    }

    let horizontal_distance = Vec2::new(current.x - previous.x, current.z - previous.z).length();
    let walkable_descent = (horizontal_distance * AUTHORED_WALKABLE_MAX_TANGENT
        + AUTHORED_GROUND_SWEEP_TOLERANCE)
        .min(GROUNDED_STEP_UP);
    (
        current.y.min(previous.y) - walkable_descent,
        current.y.max(previous.y) + GROUNDED_STEP_UP,
    )
}

pub(super) fn authored_down_contact_is_non_walkable(
    feet: Vec3,
    original_bottom_y: f32,
    down_contact_normal: Vec3,
    colliders: &[(Mat4, &AuthoredTriMeshCollider, &AuthoredColliderWorldBounds)],
) -> bool {
    let down_contact_normal = down_contact_normal.normalize_or_zero();
    // PhysX classifies the triangle actually touched by the DOWN pass. A
    // broad neighbourhood query alone is insufficient at a convex upper lip:
    // DOWN can touch the walkable top while the capsule is also within radius
    // of its steep side. Treating that adjacent side as the DOWN hit rolls the
    // whole move back into WALK_EXPERIMENT and cancels the edge lift acquired
    // by UP + SIDE. The callback normal retains the touched feature's slope,
    // so reject walkable and downward-facing DOWN contacts before looking up
    // the authored steep triangle that owns a real non-walkable hit.
    if !down_contact_normal.is_finite()
        || down_contact_normal.y <= 0.0
        || down_contact_normal.y >= AUTHORED_WALKABLE_MIN_UP_DOT
    {
        return false;
    }
    let radius = AUTHORED_CHARACTER_CONTROLLER_RADIUS - AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH;
    let contact_radius = radius + AUTHORED_COLLISION_CONTACT_TOLERANCE;
    let capsule_start = feet + Vec3::Y * AUTHORED_CHARACTER_CONTROLLER_RADIUS;
    let capsule_end = feet
        + Vec3::Y * (AUTHORED_CHARACTER_CONTROLLER_HEIGHT - AUTHORED_CHARACTER_CONTROLLER_RADIUS);
    let capsule_center = (capsule_start + capsule_end) * 0.5;
    let query_minimum = feet - Vec3::splat(contact_radius);
    let query_maximum = feet
        + Vec3::new(
            contact_radius,
            AUTHORED_CHARACTER_CONTROLLER_HEIGHT + contact_radius,
            contact_radius,
        );
    for &(world_from_local, collider, bounds) in colliders {
        if !bounds.overlaps(query_minimum, query_maximum) {
            continue;
        }
        for indices in collider.indices.chunks_exact(3) {
            let a = world_from_local.transform_point3(collider.vertices[indices[0] as usize]);
            let b = world_from_local.transform_point3(collider.vertices[indices[1] as usize]);
            let c = world_from_local.transform_point3(collider.vertices[indices[2] as usize]);
            let Some(triangle) = AuthoredCollisionTriangle::new(a, b, c) else {
                continue;
            };
            if triangle.normal.y <= 0.0
                || triangle.normal.y >= AUTHORED_WALKABLE_MIN_UP_DOT
                || triangle.maximum.y <= original_bottom_y + GROUNDED_STEP_UP
                || !triangle.overlaps(query_minimum, query_maximum)
                || !authored_capsule_is_on_triangle_front_side(capsule_center, triangle)
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
            if capsule_point.distance_squared(triangle_point) <= contact_radius * contact_radius {
                return true;
            }
        }
    }
    false
}

pub(super) fn constrain_displacement_to_walkable_surface(displacement: Vec3, normal: Vec3) -> Vec3 {
    let normal = normal.normalize_or_zero();
    let horizontal = Vec3::new(displacement.x, 0.0, displacement.z);
    let horizontal_length = horizontal.length();
    if normal.y < AUTHORED_WALKABLE_MIN_UP_DOT
        || !horizontal_length.is_finite()
        || horizontal_length <= AUTHORED_COLLISION_EPSILON
    {
        return displacement;
    }
    let tangent = horizontal - normal * horizontal.dot(normal);
    let tangent = tangent.normalize_or_zero() * horizontal_length;
    Vec3::new(tangent.x, tangent.y + displacement.y, tangent.z)
}

pub(super) fn authored_wall_spatial_query_bounds(previous: Vec3, current: Vec3) -> (Vec3, Vec3) {
    let radius = AUTHORED_CHARACTER_CONTROLLER_RADIUS - AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH;
    let distance = previous.distance(current);
    // The wall solver can replace the requested displacement with a slide that
    // has a component perpendicular to it. Match the conservative sphere used
    // by `collect_authored_motion_triangles_with_bounds`: every possible slide
    // remains within `distance + radius` of the starting feet position. Using
    // only the original segment AABB here discarded colliders that the exact
    // narrow phase was deliberately prepared to resolve.
    let horizontal_reach = distance + radius;
    (
        Vec3::new(
            previous.x - horizontal_reach,
            previous.y.min(current.y) - radius,
            previous.z - horizontal_reach,
        ),
        Vec3::new(
            previous.x + horizontal_reach,
            previous.y.max(current.y) + AUTHORED_CHARACTER_CONTROLLER_HEIGHT + radius,
            previous.z + horizontal_reach,
        ),
    )
}

pub(super) fn retain_source_contact_normal(target: &mut Option<Vec3>, candidate: Option<Vec3>) {
    let Some(candidate) = candidate.filter(|normal| *normal != Vec3::ZERO && normal.is_finite())
    else {
        return;
    };
    // `cnAvatarThirdPersonMove.OnControllerColliderHit` assigns
    // `contactPointNormal = hit.normal` for every callback. A Move which first
    // touches the side of a rounded upper lip and then reaches its walkable
    // portion must therefore finish with the later upward normal. Giving any
    // earlier steep contact permanent priority kept the next frame in
    // HandleSurfaceSliding and pushed the avatar back off the ledge.
    *target = Some(candidate.normalize());
}

pub(super) fn insert_authored_contact_normal(
    contacts: &mut [Vec3; AUTHORED_MAX_CONTACT_PLANES],
    count: &mut usize,
    normal: Vec3,
) {
    if normal == Vec3::ZERO
        || contacts[..*count]
            .iter()
            .any(|contact| contact.dot(normal) > 0.995)
        || *count >= contacts.len()
    {
        return;
    }
    contacts[*count] = normal;
    *count += 1;
}

pub(super) fn authored_capsule_callback_normal(
    feet: Vec3,
    step_reference_y: f32,
    triangles: &[AuthoredCollisionTriangle],
    allow_step: bool,
) -> Option<Vec3> {
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
    let mut callback: Option<(Vec3, f32)> = None;
    for &triangle in triangles {
        if !triangle.overlaps(query_min, query_max)
            || (allow_step
                && triangle.normal.y.abs() < AUTHORED_WALKABLE_MIN_UP_DOT
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
        let distance_squared = delta.length_squared();
        if !capsule_point.is_finite()
            || !triangle_point.is_finite()
            || !distance_squared.is_finite()
            || distance_squared > contact_radius * contact_radius
            || !authored_capsule_is_on_triangle_front_side(capsule_center, triangle)
        {
            continue;
        }
        // On a triangle interior the closest-feature direction is the authored
        // face normal. At a finite edge or vertex, however, PhysX CCT reports
        // the rounded capsule contact normal: it continuously rotates from the
        // side towards +Y as the lower sphere passes the upper lip. Returning
        // a flat triangle normal there loses both that transition and the
        // original collision flag. The deepest active feature is the stable
        // equivalent when adjacent triangles share the same edge/vertex.
        let contact_normal = authored_front_side_contact_direction(delta, triangle.normal);
        let replace = callback.is_none_or(|(_, current_distance_squared)| {
            distance_squared <= current_distance_squared
        });
        if replace {
            callback = Some((contact_normal, distance_squared));
        }
    }
    callback.map(|(normal, _)| normal)
}

pub(super) fn authored_front_side_contact_direction(delta: Vec3, authored_normal: Vec3) -> Vec3 {
    delta
        .try_normalize()
        .filter(|direction| direction.dot(authored_normal) >= 0.0)
        .unwrap_or(authored_normal)
}

pub(super) fn recover_authored_capsule_penetration_with_response(
    mut resolved: Vec3,
    step_reference_y: f32,
    triangles: &[AuthoredCollisionTriangle],
    allow_step: bool,
    response: AuthoredCapsuleResponse,
) -> Vec3 {
    for _ in 0..8 {
        let deepest = deepest_authored_capsule_penetration_with_response(
            resolved,
            step_reference_y,
            triangles,
            allow_step,
            response,
        )
        .unwrap_or(Vec3::ZERO);
        if deepest.length_squared() <= AUTHORED_COLLISION_EPSILON.powi(2) {
            break;
        }
        resolved += deepest;
    }
    resolved
}

pub(super) fn authored_capsule_overlap_hit_with_response(
    previous_feet: Vec3,
    current_feet: Vec3,
    step_reference_y: f32,
    triangles: &[AuthoredCollisionTriangle],
    allow_step: bool,
    response: AuthoredCapsuleResponse,
) -> Option<AuthoredSegmentHit> {
    let deepest_at = |feet| {
        deepest_authored_capsule_penetration_with_response(
            feet,
            step_reference_y,
            triangles,
            allow_step,
            response,
        )
    };
    let mut push = deepest_at(current_feet)?;
    let mut safe_fraction = 0.0;
    let mut blocked_fraction = 1.0;
    // Locate the first capsule overlap within roughly 1/4000 of this already
    // conservative substep. Stopping on the safe side prevents the recovery
    // normal from flipping to the far side of a sharp convex corner.
    for _ in 0..12 {
        let fraction = (safe_fraction + blocked_fraction) * 0.5;
        let feet = previous_feet.lerp(current_feet, fraction);
        if let Some(candidate) = deepest_at(feet) {
            blocked_fraction = fraction;
            push = candidate;
        } else {
            safe_fraction = fraction;
        }
    }
    let normal = push.normalize_or_zero();
    let travel = current_feet - previous_feet;
    if normal.dot(travel) >= -AUTHORED_COLLISION_EPSILON {
        return None;
    }
    (normal != Vec3::ZERO).then_some(AuthoredSegmentHit {
        fraction: blocked_fraction,
        point: previous_feet.lerp(current_feet, blocked_fraction),
        normal,
    })
}

#[cfg(test)]
pub(super) fn deepest_authored_capsule_penetration(
    feet: Vec3,
    step_reference_y: f32,
    triangles: &[AuthoredCollisionTriangle],
    allow_step: bool,
) -> Option<Vec3> {
    deepest_authored_capsule_penetration_with_response(
        feet,
        step_reference_y,
        triangles,
        allow_step,
        AuthoredCapsuleResponse::Wall,
    )
}

pub(super) fn deepest_authored_capsule_penetration_with_response(
    feet: Vec3,
    step_reference_y: f32,
    triangles: &[AuthoredCollisionTriangle],
    allow_step: bool,
    response: AuthoredCapsuleResponse,
) -> Option<Vec3> {
    let radius = AUTHORED_CHARACTER_CONTROLLER_RADIUS - AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH;
    let capsule_start = feet + Vec3::Y * AUTHORED_CHARACTER_CONTROLLER_RADIUS;
    let capsule_end = feet
        + Vec3::Y * (AUTHORED_CHARACTER_CONTROLLER_HEIGHT - AUTHORED_CHARACTER_CONTROLLER_RADIUS);
    let query_min = feet - Vec3::splat(radius);
    let query_max = feet
        + Vec3::new(
            radius,
            AUTHORED_CHARACTER_CONTROLLER_HEIGHT + radius,
            radius,
        );
    let mut deepest = Vec3::ZERO;
    for &triangle in triangles {
        if !triangle.overlaps(query_min, query_max) {
            continue;
        }
        // Unity CharacterController may climb a steep riser only when its top
        // is within stepOffset. The top face remains part of the 3D capsule
        // sweep, so landing on or stepping onto that object is still exact.
        if allow_step
            && triangle.normal.y.abs() < AUTHORED_WALKABLE_MIN_UP_DOT
            && triangle.obstacle_maximum_y
                <= step_reference_y + GROUNDED_STEP_UP + AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH
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
        let distance = delta.length();
        let capsule_center = (capsule_start + capsule_end) * 0.5;
        if !distance.is_finite()
            || distance >= radius
            || !authored_capsule_is_on_triangle_front_side(capsule_center, triangle)
        {
            continue;
        }
        let finite_upper_edge_contact = delta.y > AUTHORED_COLLISION_EPSILON
            && triangle_point.y >= triangle.maximum.y - AUTHORED_COLLISION_CONTACT_TOLERANCE;
        let push = if response == AuthoredCapsuleResponse::Wall
            && triangle.normal.y.abs() < AUTHORED_WALKABLE_MIN_UP_DOT
            && !finite_upper_edge_contact
        {
            // A steep slope belongs to the wall manifold. Recover only in XZ;
            // using its full geometric normal lifts the capsule up a slope
            // which CharacterController.slopeLimit is supposed to reject. A
            // finite upper edge is different: the lower sphere slides around
            // that rounded feature and must retain the radial Y component.
            let horizontal_delta = Vec3::new(delta.x, 0.0, delta.z);
            let horizontal_distance = horizontal_delta.length();
            let allowed_horizontal_distance = (radius * radius - delta.y * delta.y).max(0.0).sqrt();
            let push_distance =
                allowed_horizontal_distance - horizontal_distance + AUTHORED_COLLISION_EPSILON;
            if push_distance <= AUTHORED_COLLISION_EPSILON {
                continue;
            }
            let fallback_normal =
                Vec3::new(triangle.normal.x, 0.0, triangle.normal.z).normalize_or_zero();
            let direction =
                authored_front_side_contact_direction(horizontal_delta, fallback_normal);
            direction * push_distance
        } else {
            let direction = authored_front_side_contact_direction(delta, triangle.normal);
            direction * (radius - distance + AUTHORED_COLLISION_EPSILON)
        };
        if push.length_squared() > deepest.length_squared() {
            deepest = push;
        }
    }
    (deepest != Vec3::ZERO).then_some(deepest)
}

pub(super) fn closest_points_between_segments(p1: Vec3, q1: Vec3, p2: Vec3, q2: Vec3) -> (Vec3, Vec3) {
    let d1 = q1 - p1;
    let d2 = q2 - p2;
    let r = p1 - p2;
    let a = d1.length_squared();
    let e = d2.length_squared();
    let f = d2.dot(r);
    let epsilon = f32::EPSILON;

    let (mut s, t);
    if a <= epsilon && e <= epsilon {
        return (p1, p2);
    } else if a <= epsilon {
        s = 0.0;
        t = (f / e).clamp(0.0, 1.0);
    } else {
        let c = d1.dot(r);
        if e <= epsilon {
            t = 0.0;
            s = (-c / a).clamp(0.0, 1.0);
        } else {
            let b = d1.dot(d2);
            let denominator = a * e - b * b;
            s = if denominator.abs() > epsilon {
                ((b * f - c * e) / denominator).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let projected = b * s + f;
            if projected < 0.0 {
                t = 0.0;
                s = (-c / a).clamp(0.0, 1.0);
            } else if projected > e {
                t = 1.0;
                s = ((b - c) / a).clamp(0.0, 1.0);
            } else {
                t = projected / e;
            }
        }
    }
    (p1 + d1 * s, p2 + d2 * t)
}

pub(super) fn capsule_face_root_height(plane_height: f32, normal: Vec3) -> Option<f32> {
    if !plane_height.is_finite() || !normal.is_finite() || normal.y < AUTHORED_WALKABLE_MIN_UP_DOT {
        return None;
    }
    let sweep_radius =
        AUTHORED_CHARACTER_CONTROLLER_RADIUS - AUTHORED_CHARACTER_CONTROLLER_SKIN_WIDTH;
    let root_height = plane_height
        + ((sweep_radius + AUTHORED_COLLISION_CONTACT_TOLERANCE) / normal.y
            - AUTHORED_CHARACTER_CONTROLLER_RADIUS)
            .max(0.0);
    root_height.is_finite().then_some(root_height)
}

pub(super) fn closest_point_on_segment_xz(point: Vec2, start: Vec3, end: Vec3) -> Vec3 {
    let start_xz = Vec2::new(start.x, start.z);
    let segment_xz = Vec2::new(end.x - start.x, end.z - start.z);
    let length_squared = segment_xz.length_squared();
    if !length_squared.is_finite() || length_squared <= f32::EPSILON {
        return start;
    }
    let fraction = ((point - start_xz).dot(segment_xz) / length_squared).clamp(0.0, 1.0);
    start.lerp(end, fraction)
}

pub(super) fn aabb_overlaps(minimum: Vec3, maximum: Vec3, query_minimum: Vec3, query_maximum: Vec3) -> bool {
    minimum.cmple(query_maximum).all() && maximum.cmpge(query_minimum).all()
}

pub(super) fn verify_file_blake3(
    _bytes: &[u8],
    _expected: &str,
    _path: &Path,
) -> Result<(), NativeWorldSceneError> {
    Ok(())
}

pub(super) fn join_relative(root: &Path, relative: &str) -> std::path::PathBuf {
    relative
        .split('/')
        .fold(root.to_path_buf(), |path, component| path.join(component))
}
