use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameSequence {
    Fade(FadeSequence),
    SubtitleTypewriter {
        localization_key: &'static str,
        canonical_english_chars: u16,
        first_character: u16,
        seconds_per_character: f32,
    },
    NpcDelta {
        id: i32,
        frames: u16,
        seconds_per_frame: f32,
        translation_per_frame: ClientVec3,
        rotation_each_frame: Option<RotationExpr>,
    },
    CameraShake {
        frames: u16,
        seconds_per_frame: f32,
        amplitude: f32,
        start_wave: CameraShakeWave,
        target_wave: CameraShakeWave,
    },
    NpcLerp {
        id: i32,
        frames: u16,
        seconds_per_frame: f32,
        from: ClientVec3,
        to: ClientVec3,
        numerator_offset: u16,
        denominator: f32,
    },
    PlayerPathAndFade {
        frames: u16,
        seconds_per_frame: f32,
        start: ClientVec3,
        translation_per_frame: ClientVec3,
        fade_from: f32,
        fade_numerator_per_frame: f32,
        fade_divisor: f32,
    },
}
