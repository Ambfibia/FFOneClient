use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameSequenceSample {
    Fade {
        channel: FadeChannel,
        frame: u16,
        value: f32,
    },
    SubtitleTypewriter {
        localization_key: &'static str,
        frame: u16,
        visible_characters: u16,
        canonical_english_chars: u16,
    },
    NpcDelta {
        id: i32,
        frame: u16,
        cumulative_translation: ClientVec3,
        rotation_each_frame: Option<RotationExpr>,
    },
    CameraShake {
        frame: u16,
        start_offset: ClientVec3,
        target_offset: ClientVec3,
    },
    NpcLerp {
        id: i32,
        frame: u16,
        value: ClientVec3,
    },
    PlayerPathAndFade {
        frame: u16,
        player_position: ClientVec3,
        fade_alpha: f32,
    },
}

pub(super) fn sequence_frame_count(sequence: FrameSequence) -> u16 {
    match sequence {
        FrameSequence::Fade(fade) => fade.steps.max(1),
        FrameSequence::SubtitleTypewriter {
            canonical_english_chars,
            first_character,
            ..
        } => canonical_english_chars
            .saturating_sub(first_character)
            .saturating_add(1)
            .max(1),
        FrameSequence::NpcDelta { frames, .. }
        | FrameSequence::CameraShake { frames, .. }
        | FrameSequence::NpcLerp { frames, .. }
        | FrameSequence::PlayerPathAndFade { frames, .. } => frames.max(1),
    }
}

pub(super) fn sequence_frame_delay(sequence: FrameSequence, frame: u16) -> f32 {
    match sequence {
        FrameSequence::Fade(fade) => f32::from(frame) * fade.seconds_per_step,
        FrameSequence::SubtitleTypewriter {
            seconds_per_character,
            ..
        } => f32::from(frame) * seconds_per_character,
        FrameSequence::NpcDelta {
            seconds_per_frame, ..
        }
        | FrameSequence::CameraShake {
            seconds_per_frame, ..
        }
        | FrameSequence::NpcLerp {
            seconds_per_frame, ..
        }
        | FrameSequence::PlayerPathAndFade {
            seconds_per_frame, ..
        } => f32::from(frame) * seconds_per_frame,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub(super) struct TutorialPanFrame(pub(super) usize);
