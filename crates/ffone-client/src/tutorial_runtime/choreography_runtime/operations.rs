use super::*;

pub(super) fn blocking_wait_start(wait: BlockingWait) -> f32 {
    match wait {
        BlockingWait::AssetPreload {
            reached_at_seconds, ..
        }
        | BlockingWait::EffectInstantiationRetry {
            reached_at_seconds, ..
        } => reached_at_seconds,
    }
}

pub(super) fn blocking_wait_continuation_source_line(wait: BlockingWait) -> u32 {
    match wait {
        BlockingWait::AssetPreload {
            continuation_source_line,
            ..
        }
        | BlockingWait::EffectInstantiationRetry {
            continuation_source_line,
            ..
        } => continuation_source_line,
    }
}

pub(super) fn next_blocking_wait(playback: &ActivePlayback) -> Option<BlockingWait> {
    playback
        .choreography
        .blocking_waits
        .get(playback.blocking_wait_cursor)
        .copied()
}

pub(super) fn sample_sequence(
    sequence: FrameSequence,
    frame: u16,
    sample_seconds: f32,
) -> FrameSequenceSample {
    match sequence {
        FrameSequence::Fade(fade) => {
            let divisor = if fade.divisor.abs() <= f32::EPSILON {
                1.0
            } else {
                fade.divisor
            };
            let value = fade.from + (fade.to - fade.from) * f32::from(frame) / divisor;
            FrameSequenceSample::Fade {
                channel: fade.channel,
                frame,
                value,
            }
        }
        FrameSequence::SubtitleTypewriter {
            localization_key,
            canonical_english_chars,
            first_character,
            ..
        } => FrameSequenceSample::SubtitleTypewriter {
            localization_key,
            frame,
            visible_characters: first_character
                .saturating_add(frame)
                .min(canonical_english_chars),
            canonical_english_chars,
        },
        FrameSequence::NpcDelta {
            id,
            translation_per_frame,
            rotation_each_frame,
            ..
        } => {
            let count = f32::from(frame.saturating_add(1));
            FrameSequenceSample::NpcDelta {
                id,
                frame,
                cumulative_translation: ClientVec3::new(
                    translation_per_frame.x * count,
                    translation_per_frame.y * count,
                    translation_per_frame.z * count,
                ),
                rotation_each_frame,
            }
        }
        FrameSequence::CameraShake {
            amplitude,
            start_wave,
            target_wave,
            ..
        } => FrameSequenceSample::CameraShake {
            frame,
            start_offset: sample_camera_shake_wave(start_wave, sample_seconds, amplitude),
            target_offset: sample_camera_shake_wave(target_wave, sample_seconds, amplitude),
        },
        FrameSequence::NpcLerp {
            id,
            from,
            to,
            numerator_offset,
            denominator,
            ..
        } => {
            let denominator = if denominator.abs() <= f32::EPSILON {
                1.0
            } else {
                denominator
            };
            let amount = f32::from(frame.saturating_add(numerator_offset)) / denominator;
            FrameSequenceSample::NpcLerp {
                id,
                frame,
                value: lerp_client_vec3(from, to, amount),
            }
        }
        FrameSequence::PlayerPathAndFade {
            start,
            translation_per_frame,
            fade_from,
            fade_numerator_per_frame,
            fade_divisor,
            ..
        } => {
            let source_index = f32::from(frame);
            let divisor = if fade_divisor.abs() <= f32::EPSILON {
                1.0
            } else {
                fade_divisor
            };
            FrameSequenceSample::PlayerPathAndFade {
                frame,
                player_position: ClientVec3::new(
                    start.x + translation_per_frame.x * source_index,
                    start.y + translation_per_frame.y * source_index,
                    start.z + translation_per_frame.z * source_index,
                ),
                fade_alpha: fade_from + source_index * fade_numerator_per_frame / divisor,
            }
        }
    }
}

pub(super) fn sample_camera_shake_wave(
    wave: crate::tutorial_choreography::CameraShakeWave,
    sample_seconds: f32,
    amplitude: f32,
) -> ClientVec3 {
    ClientVec3::new(
        (sample_seconds * wave.x_sin_multiplier).sin() * amplitude,
        (sample_seconds * wave.y_cos_multiplier).cos() * amplitude,
        ((sample_seconds * wave.z_sin_multiplier).sin()
            + (sample_seconds * wave.z_cos_multiplier).cos())
            * amplitude,
    )
}

pub(super) fn lerp_client_vec3(from: ClientVec3, to: ClientVec3, amount: f32) -> ClientVec3 {
    ClientVec3::new(
        from.x + (to.x - from.x) * amount,
        from.y + (to.y - from.y) * amount,
        from.z + (to.z - from.z) * amount,
    )
}

pub(super) fn client_vec3(value: ClientVec3) -> Vec3 {
    // The choreography tables retain legacy Unity coordinates. Camera-shake
    // samples must cross the same H=diag(-1,1,1) boundary as every other
    // native runtime position; otherwise each shake is mirrored horizontally.
    Vec3::new(-value.x, value.y, value.z)
}
