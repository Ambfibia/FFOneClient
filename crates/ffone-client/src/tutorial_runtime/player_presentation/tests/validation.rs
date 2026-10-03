use super::*;

#[test]
fn capabilities_require_exact_identity_playback_and_runtime_ready_status() {
    let gender = PlayerRigGender::Male;
    let contracts = [
        clip_contract(
            gender,
            TutorialPlayerClip::Stand1,
            0,
            TUTORIAL_PLAYER_RUNTIME_READY_STATUS,
        ),
        clip_contract(
            gender,
            TutorialPlayerClip::Run,
            6,
            "asset-published-tutorial-player-consumer-pending",
        ),
    ];
    let capabilities =
        TutorialPlayerRigCapabilities::from_contract(gender, &contracts).unwrap();
    let stand = capabilities
        .resolve(TutorialPlayerAnimationRequest::stand_force(gender))
        .unwrap();
    assert_eq!(stand.gltf_animation_index, 0);
    assert_eq!(stand.playback, TutorialPlayerClipPlayback::Loop);

    let run =
        TutorialPlayerAnimationRequest::locomotion(gender, TutorialPlayerClip::Run).unwrap();
    assert!(matches!(
        capabilities.resolve(run),
        Err(TutorialPlayerRigCapabilityError::RuntimeStatusNotReady {
            clip: TutorialPlayerClip::Run,
            ..
        })
    ));
}
