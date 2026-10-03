use super::*;

/// Pure mapping of the exact `UnityEngine.Random.Range` call sequence in
/// `work/ilspy-retrobution-csharp/OniMoveScript.cs:24-35`.
///
/// Each value is a captured normalized Unity draw in `[0, 1]`. Normal mode
/// consumes `[p0.x, p0.z, p0.y, p1.x, p1.z, p1.y]`; reverse consumes
/// `[p0.x, p0.z, p1.x, p1.z]` because the original does not draw Y.
pub fn tutorial_oni_velocity_samples_from_unity_unit_draws(
    reverse: bool,
    draws: &[f32],
) -> Option<[Vec3; 2]> {
    let expected = if reverse { 4 } else { 6 };
    if draws.len() != expected
        || draws
            .iter()
            .any(|draw| !draw.is_finite() || !(0.0..=1.0).contains(draw))
    {
        return None;
    }
    let velocity = |x: f32, z: f32, y: Option<f32>| {
        Vec3::new(
            x * 20.0 - 10.0,
            y.map_or(0.0, |draw| draw * 5.0 + 5.0),
            z * 20.0 - 10.0,
        )
    };
    Some(if reverse {
        [
            velocity(draws[0], draws[1], None),
            velocity(draws[2], draws[3], None),
        ]
    } else {
        [
            velocity(draws[0], draws[1], Some(draws[2])),
            velocity(draws[3], draws[4], Some(draws[5])),
        ]
    })
}
