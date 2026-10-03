use super::*;

pub(super) const fn upper_event_releases_immediately(
    clip: TutorialPlayerClip,
    locomotion: LegacyLocomotionState,
) -> bool {
    matches!(clip, TutorialPlayerClip::Attack1Upper)
        && matches!(
            locomotion,
            LegacyLocomotionState::Run
                | LegacyLocomotionState::RunBack
                | LegacyLocomotionState::JumpStart
                | LegacyLocomotionState::Jump
                | LegacyLocomotionState::Landing
                | LegacyLocomotionState::LandingRun
        )
}
