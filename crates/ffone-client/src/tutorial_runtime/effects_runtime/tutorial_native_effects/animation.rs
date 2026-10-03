use super::*;

#[derive(Clone, Debug, Default)]
pub(super) struct AnimationCurve(pub(super) Vec<CurveKey>);

pub(super) fn mesh_effect_repeats_standard_animation(effect_id: i32) -> bool {
    // World EP mesh prefabs serialize WrapMode.Loop directly. Tutorial
    // `EffectEmitterController.Start` also overrides every instantiated
    // non-null `nifObject` to Loop:
    //
    //     go.animation.wrapMode = (WrapMode)2; // Loop
    //
    // Therefore the controller contract, rather than the serialized
    // Animation.wrapMode value, owns playback for every published mesh
    // effect. This is significant for ES734 and ES751: their clips are
    // shorter than EffectEmitterController.maxTimer and holding the last
    // sample leaves the projectile/slash geometry frozen on screen.
    exact_effect_mesh_scene(effect_id).is_some()
        || matches!(effect_id, 429 | 411 | 434 | 714 | 412 | 805 | 435 | 766)
}

pub(super) fn projectile_mesh_repeats_standard_animation(effect_id: i32) -> bool {
    // These exact legacy Animation components autoplay `nif-default` with
    // UnityEngine.WrapMode.Loop. ES15/ES391 use material animation metadata;
    // ES718 publishes its authored Cylinder01 scale curve as a standard clip.
    matches!(
        effect_id,
        15 | 31 | 379 | 391 | 718 | 729 | 755 | 787 | 790 | 791 | 792 | 793
    )
}

impl AnimationCurve {
    pub(super) fn evaluate(&self, time: f32, fallback: f32) -> f32 {
        let Some(first) = self.0.first() else {
            return fallback;
        };
        if time <= first.time {
            return first.value;
        }
        let last = self.0.last().unwrap();
        if time >= last.time {
            return last.value;
        }
        let pair = self
            .0
            .windows(2)
            .find(|pair| time >= pair[0].time && time <= pair[1].time)
            .unwrap();
        let duration = pair[1].time - pair[0].time;
        if duration <= f32::EPSILON {
            return pair[1].value;
        }
        let t = (time - pair[0].time) / duration;
        let (t2, t3) = (t * t, t * t * t);
        (2.0 * t3 - 3.0 * t2 + 1.0) * pair[0].value
            + (t3 - 2.0 * t2 + t) * duration * pair[0].out_slope
            + (-2.0 * t3 + 3.0 * t2) * pair[1].value
            + (t3 - t2) * duration * pair[1].in_slope
    }
}

pub(super) const PARTICLE_COLOR_ANIMATION_STEPS: f32 = 16.0;
