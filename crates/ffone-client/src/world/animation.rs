use super::*;

pub(super) fn clip_displacement_against_contacts(displacement: Vec3, contact_normals: &[Vec3]) -> Vec3 {
    let is_feasible = |candidate: Vec3| {
        contact_normals
            .iter()
            .all(|normal| candidate.dot(*normal) >= -AUTHORED_COLLISION_EPSILON)
    };
    if is_feasible(displacement) {
        return displacement;
    }

    // Project onto the contact cone, considering its faces, pairwise edge
    // lines, and the zero-dimensional corner. This is both bounded and exact
    // for our 3D displacement; sequential one-plane projections are not exact
    // and can put velocity back through an earlier face at an oblique corner.
    let mut best = Vec3::ZERO;
    let mut best_error = displacement.length_squared();
    for (index, &normal) in contact_normals.iter().enumerate() {
        let face_candidate = displacement - normal * displacement.dot(normal);
        let face_error = face_candidate.distance_squared(displacement);
        if is_feasible(face_candidate) && face_error < best_error {
            best = face_candidate;
            best_error = face_error;
        }
        for &other in &contact_normals[index + 1..] {
            let axis = normal.cross(other).normalize_or_zero();
            if axis == Vec3::ZERO {
                continue;
            }
            let edge_candidate = axis * displacement.dot(axis);
            let edge_error = edge_candidate.distance_squared(displacement);
            if is_feasible(edge_candidate) && edge_error < best_error {
                best = edge_candidate;
                best_error = edge_error;
            }
        }
    }
    best
}
