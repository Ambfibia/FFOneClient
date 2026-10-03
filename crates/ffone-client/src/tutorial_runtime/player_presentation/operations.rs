use super::*;

/// Convert the original float32 seconds at the renderer boundary without
/// rounding it to an integer number of milliseconds.
#[must_use]
pub fn tutorial_stand_force_cross_fade() -> Duration {
    Duration::from_secs_f32(TUTORIAL_STAND_FORCE_CROSS_FADE_SECONDS)
}

/// Strict protocol-to-rig mapping used by selected player assembly.
pub fn tutorial_player_gender_from_protocol(
    protocol_gender: i8,
) -> Result<PlayerRigGender, TutorialPlayerPresentationError> {
    match protocol_gender {
        1 => Ok(PlayerRigGender::Male),
        2 => Ok(PlayerRigGender::Female),
        value => Err(TutorialPlayerPresentationError::UnsupportedProtocolGender(
            value,
        )),
    }
}
