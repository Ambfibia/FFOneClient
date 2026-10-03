use super::*;

pub(super) fn object<'a>(value: &'a JsonValue, key: &str) -> Result<&'a JsonValue, String> {
    value
        .get(key)
        .filter(|value| value.is_object())
        .ok_or_else(|| format!("{key} is absent or is not an object"))
}

/// Unity's `Vector3.Slerp` interpolates direction spherically and magnitude
/// linearly. The Stag points do not normally produce antiparallel vectors;
/// the linear fallback only protects malformed or degenerate frames.
pub(super) fn unity_vector_slerp(start: Vec3, end: Vec3, fraction: f32) -> Vec3 {
    let start_length = start.length();
    let end_length = end.length();
    if start_length <= f32::EPSILON || end_length <= f32::EPSILON {
        return start.lerp(end, fraction);
    }
    let start_direction = start / start_length;
    let end_direction = end / end_length;
    let dot = start_direction.dot(end_direction).clamp(-1.0, 1.0);
    let angle = dot.acos();
    let sine = angle.sin();
    let direction = if sine.abs() <= 1.0e-5 {
        start_direction
            .lerp(end_direction, fraction)
            .try_normalize()
            .unwrap_or(start_direction)
    } else {
        (start_direction * ((1.0 - fraction) * angle).sin()
            + end_direction * (fraction * angle).sin())
            / sine
    };
    direction * start_length.lerp(end_length, fraction)
}

/// Exact `ParticleEmitterController.ConfigureParticleSettings` evaluation.
///
/// All of the source formulas, including Quaternion.Euler and the unusual
/// near-zero component multipliers, run before the exporter reflection
/// `H = diag(-1, 1, 1)`. Applying H only to the completed state is equivalent
/// to Unity's result followed by the repository-wide native coordinate
/// contract; reflecting selected inputs earlier is not equivalent for the
/// random Euler rotation.
pub(super) fn legacy_particle_initial_state_in_unity(
    plan: &EmitterPlan,
    random: &mut NativeParticleRandomStream,
) -> (Vec3, Vec3) {
    use EmitterGeneration::*;

    let position = match plan.generation {
        Plane => Vec3::new(
            (random.unit() - 0.5) * plan.plane.x,
            (random.unit() - 0.5) * plan.plane.y,
            (random.unit() - 0.5) * plan.plane.z,
        ),
        Point => {
            let direction = random.on_unit_sphere();
            direction * (random.unit() * plan.random_position)
        }
        InversePoint => random.on_unit_sphere() * plan.random_position,
        XPlaneDonut | InverseXPlaneDonut | OutwardXPlaneDonut => {
            let radius = plan.plane.x + (plan.plane.z - plan.plane.x) * random.unit();
            let direction = random.inside_unit_circle_direction();
            Vec3::new(0.0, direction.x, direction.y) * radius
        }
        YPlaneDonut | InverseYPlaneDonut | OutwardYPlaneDonut => {
            let direction = random.inside_unit_circle_direction();
            let radius = plan.plane.x + (plan.plane.z - plan.plane.x) * random.unit();
            Vec3::new(direction.x, 0.0, direction.y) * radius
        }
        ZPlaneDonut | InverseZPlaneDonut | OutwardZPlaneDonut => {
            let radius = plan.plane.x + (plan.plane.z - plan.plane.x) * random.unit();
            let direction = random.inside_unit_circle_direction();
            Vec3::new(direction.x, direction.y, 0.0) * radius
        }
    };

    let mut velocity = match plan.generation {
        InversePoint | InverseXPlaneDonut | InverseYPlaneDonut | InverseZPlaneDonut => {
            (-position).normalize_or_zero()
        }
        OutwardXPlaneDonut | OutwardYPlaneDonut | OutwardZPlaneDonut => {
            position.normalize_or_zero()
        }
        _ => Vec3::ZERO,
    };

    // Unity's Quaternion.Euler applies the serialized x/y/z angles in Z-X-Y
    // order. The original script samples radians, converts them to degrees for
    // `Quaternion.Euler`, then rotates `initVelocity`.
    let angles = Vec3::new(
        (random.unit() * 2.0 - 1.0) * plan.random_angle * std::f32::consts::PI,
        (random.unit() * 2.0 - 1.0) * plan.random_angle * std::f32::consts::PI,
        (random.unit() * 2.0 - 1.0) * plan.random_angle * std::f32::consts::PI,
    );
    let rotated =
        Quat::from_euler(EulerRot::ZXY, angles.z, angles.x, angles.y) * plan.initial_velocity;
    velocity.x += if plan.initial_velocity.x.abs() < 0.01 {
        rotated.x * plan.initial_velocity.x
    } else {
        rotated.x
    };
    velocity.y += if plan.initial_velocity.y.abs() < 0.01 {
        rotated.y * plan.initial_velocity.y
    } else {
        rotated.y
    };
    velocity.z += if plan.initial_velocity.z.abs() < 0.01 {
        rotated.z * plan.initial_velocity.z
    } else {
        rotated.z
    };
    velocity *= random.unit() * 2.0 * (1.0 - plan.random_velocity);
    (position, velocity)
}
