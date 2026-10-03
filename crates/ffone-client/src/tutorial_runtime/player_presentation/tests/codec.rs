use super::*;

#[test]
fn equipment_does_not_suppress_same_frame_runtime_animation() {
    let mut queue = TutorialPlayerPresentationCommandQueue::default();
    queue.tutorial_weapon(true, 0).unwrap();
    assert_eq!(queue.pending_weapon_item_id(), Some(328));
    assert!(!queue.has_pending_direct_pose_override());

    queue.push_animation(TutorialPlayerAnimationRequest::runtime_cross_fade(
        PlayerRigGender::Male,
        TutorialPlayerClip::RifleAttack1,
    ));
    assert!(!queue.has_pending_direct_pose_override());
}

#[test]
fn consumer_drains_full_and_upper_attack_in_the_same_presentation_frame() {
    let gender = PlayerRigGender::Male;
    let capabilities = TutorialPlayerRigCapabilities::from_contract(
        gender,
        &[
            clip_contract(
                gender,
                TutorialPlayerClip::RifleAttack1,
                1,
                TUTORIAL_PLAYER_RUNTIME_READY_STATUS,
            ),
            clip_contract(
                gender,
                TutorialPlayerClip::RifleAttack1Upper,
                2,
                TUTORIAL_PLAYER_RUNTIME_READY_STATUS,
            ),
        ],
    )
    .unwrap();
    let consumer = TutorialPlayerPresentationConsumer::new(capabilities);
    let mut queue = TutorialPlayerPresentationCommandQueue::default();
    queue.push_animation(TutorialPlayerAnimationRequest::runtime_cross_fade(
        gender,
        TutorialPlayerClip::RifleAttack1,
    ));
    queue.push_animation(TutorialPlayerAnimationRequest::runtime_cross_fade(
        gender,
        TutorialPlayerClip::RifleAttack1Upper,
    ));

    let resolved = consumer.consume_available(&mut queue);

    assert_eq!(resolved.len(), 2);
    assert!(queue.is_empty());
    assert_eq!(
        resolved
            .iter()
            .filter_map(|resolution| match resolution {
                TutorialPlayerPresentationResolution::AnimationContractResolved(animation) =>
                    Some(animation.request.clip),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [
            TutorialPlayerClip::RifleAttack1,
            TutorialPlayerClip::RifleAttack1Upper,
        ]
    );
}
