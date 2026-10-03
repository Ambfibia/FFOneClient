use super::*;

#[test]
fn capabilities_fail_closed_on_case_path_playback_and_duplicates() {
    let gender = PlayerRigGender::Female;
    let run =
        TutorialPlayerAnimationRequest::locomotion(gender, TutorialPlayerClip::Run).unwrap();
    let mut wrong_case = clip_contract(
        gender,
        TutorialPlayerClip::Run,
        0,
        TUTORIAL_PLAYER_RUNTIME_READY_STATUS,
    );
    wrong_case.name = "Run".to_owned();
    let capabilities =
        TutorialPlayerRigCapabilities::from_contract(gender, &[wrong_case]).unwrap();
    assert!(matches!(
        capabilities.resolve(run),
        Err(TutorialPlayerRigCapabilityError::MissingExactClip { .. })
    ));

    let mut wrong_path = clip_contract(
        gender,
        TutorialPlayerClip::Run,
        0,
        TUTORIAL_PLAYER_RUNTIME_READY_STATUS,
    );
    wrong_path.source_path_id += 1;
    let capabilities =
        TutorialPlayerRigCapabilities::from_contract(gender, &[wrong_path]).unwrap();
    assert!(matches!(
        capabilities.resolve(run),
        Err(TutorialPlayerRigCapabilityError::ContractSourcePathIdMismatch { .. })
    ));

    let mut wrong_playback = clip_contract(
        gender,
        TutorialPlayerClip::Run,
        0,
        TUTORIAL_PLAYER_RUNTIME_READY_STATUS,
    );
    wrong_playback.playback = "clamp".to_owned();
    let capabilities =
        TutorialPlayerRigCapabilities::from_contract(gender, &[wrong_playback]).unwrap();
    assert!(matches!(
        capabilities.resolve(run),
        Err(TutorialPlayerRigCapabilityError::PlaybackMismatch { .. })
    ));

    let duplicate = clip_contract(
        gender,
        TutorialPlayerClip::Run,
        0,
        TUTORIAL_PLAYER_RUNTIME_READY_STATUS,
    );
    assert!(matches!(
        TutorialPlayerRigCapabilities::from_contract(gender, &[duplicate.clone(), duplicate]),
        Err(TutorialPlayerRigCapabilityError::DuplicateExactClip {
            clip: TutorialPlayerClip::Run
        })
    ));
}
