use super::*;

pub(super) fn retrobution_actor_current_effect_name(actor_id: i32, effect_id: i32) -> String {
    format!("AnimationEvent actor {actor_id} ES{effect_id}")
}

pub(super) fn exact_projectile_inputs(
    library: &TutorialEffectLibrary,
    types: [i32; 2],
    oni: bool,
    priority: i32,
) -> Option<[(ValidatedEffect, f32); 2]> {
    if types != [76, 77] || !oni || priority != 0 {
        return None;
    }
    let resolve = |bullet_type: i32| {
        let row = library
            .projectile_catalog
            .rows
            .iter()
            .find(|entry| entry.bullet_type == bullet_type)?;
        let effect = library
            .projectile_effects
            .get(&row.parameters.particle_script)?
            .clone();
        // The validated tutorial rows require this source value to be exactly
        // 1.0, so the render-space f32 conversion is lossless.
        Some((effect, row.parameters.bullet_model_scale as f32))
    };
    Some([resolve(types[0])?, resolve(types[1])?])
}

/// `BulletMoveScript.GetSuccScale` first applies the unconditional 1.3 gain,
/// then `Crash` applies the Nano style matchup matrix. Invalid styles (the
/// null-target sentinel included) receive no matchup multiplier.
pub(super) fn exact_projectile_success_scale(base_scale: f32, source_style: i32, target_style: i32) -> f32 {
    let style_multiplier = match (source_style, target_style) {
        (0, 0) | (1, 1) | (2, 2) => 1.0,
        (0, 1) | (1, 2) | (2, 0) => 1.2,
        (0, 2) | (1, 0) | (2, 1) => 0.7,
        _ => 1.0,
    };
    base_scale * 1.3 * style_multiplier
}

pub(super) fn exact_projectile_success_script(target_exists: bool, success_script: i32) -> Option<i32> {
    (target_exists && success_script > 0).then_some(success_script)
}

/// ES100 is the clean `EmptyBullet` carrier used by melee BulletTable rows.
/// Its exact closure has an EffectEmitterController but deliberately contains
/// no ParticleEmitterController, MeshRenderer or trail. The carrier still
/// advances and owns the delayed success effect; lack of a renderer is not a
/// reason to reject the complete attack command.
pub(super) const fn exact_invisible_projectile_carrier(effect_id: i32) -> bool {
    effect_id == 100
}

pub(super) fn valid_oni_velocity_sample(sample: Vec3, reverse: bool) -> bool {
    sample.is_finite()
        && (-10.0..=10.0).contains(&sample.x)
        && (-10.0..=10.0).contains(&sample.z)
        && if reverse {
            sample.y == 0.0
        } else {
            (5.0..=10.0).contains(&sample.y)
        }
}

pub(super) fn canonical_json(value: &JsonValue) -> JsonValue {
    match value {
        JsonValue::Object(object) => {
            let mut keys = object.keys().collect::<Vec<_>>();
            keys.sort();
            let mut canonical = JsonMap::new();
            for key in keys {
                canonical.insert(key.clone(), canonical_json(&object[key]));
            }
            JsonValue::Object(canonical)
        }
        JsonValue::Array(array) => {
            JsonValue::Array(array.iter().map(canonical_json).collect::<Vec<_>>())
        }
        _ => value.clone(),
    }
}

pub(super) fn blake3_hash(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

pub(super) fn fail<T>(message: impl Into<String>) -> Result<T, TutorialEffectLibraryError> {
    Err(TutorialEffectLibraryError(message.into()))
}
