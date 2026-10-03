use super::*;

#[test]
fn death_clips_keep_primary_gender_ownership_and_source_playback() {
    for (gender, die_path_id, death_path_id) in [
        (PlayerRigGender::Male, 34_531, 34_462),
        (PlayerRigGender::Female, 34_444, 34_537),
    ] {
        assert_eq!(TutorialPlayerClip::Die.name(), "die");
        assert_eq!(TutorialPlayerClip::Death.name(), "death");
        assert_eq!(TutorialPlayerClip::Die.source_path_id(gender), die_path_id);
        assert_eq!(
            TutorialPlayerClip::Death.source_path_id(gender),
            death_path_id
        );
        assert_eq!(
            TutorialPlayerClip::Die.playback(),
            TutorialPlayerClipPlayback::Clamp
        );
        assert_eq!(
            TutorialPlayerClip::Death.playback(),
            TutorialPlayerClipPlayback::Loop
        );
    }
    assert_eq!(
        TutorialPlayerClip::from_exact_name("die"),
        Some(TutorialPlayerClip::Die)
    );
    assert_eq!(
        TutorialPlayerClip::from_exact_name("death"),
        Some(TutorialPlayerClip::Death)
    );
}

#[test]
fn protocol_gender_mapping_is_strict() {
    assert_eq!(
        tutorial_player_gender_from_protocol(1),
        Ok(PlayerRigGender::Male)
    );
    assert_eq!(
        tutorial_player_gender_from_protocol(2),
        Ok(PlayerRigGender::Female)
    );
    for unsupported in [i8::MIN, -1, 0, 3, i8::MAX] {
        assert!(tutorial_player_gender_from_protocol(unsupported).is_err());
    }
}

#[test]
fn avatar_emote_codes_map_to_exact_clean_gender_clips_and_crossfade_contract() {
    let expected = [
        (1, "cry", 34_471, 34_494),
        (2, "angry", 34_304, 34_161),
        (3, "shocked", 34_383, 34_629),
        (4, "hello", 34_598, 34_192),
        (5, "thank", 34_362, 34_476),
        (6, "dance1", 34_566, 34_283),
        (7, "kiss", 34_427, 34_602),
        (8, "agree", 34_319, 34_201),
        (9, "laugh", 34_353, 34_377),
        (10, "no", 34_410, 34_250),
        (11, "flex", 34_558, 34_218),
        (12, "tease", 34_623, 34_547),
        (13, "ok", 34_423, 34_431),
        (14, "applaud", 34_217, 34_149),
        (15, "cheer", 34_405, 34_515),
        (16, "cheer", 34_405, 34_515),
        (17, "dance2", 34_285, 34_430),
        (18, "dance3", 34_225, 34_536),
        (19, "dance4", 34_635, 34_634),
        (20, "dance5", 34_316, 34_628),
        (21, "goodbye", 34_310, 34_575),
        (22, "beach1", 34_211, 34_573),
        (23, "beach2", 34_402, 34_563),
        (24, "beach3", 34_433, 34_552),
    ];
    for (code, name, male_path_id, female_path_id) in expected {
        let clip = TutorialPlayerClip::from_avatar_emote_code(code)
            .unwrap_or_else(|| panic!("missing clean AvatarEmote code {code}"));
        assert_eq!(clip.name(), name);
        assert_eq!(clip.playback(), TutorialPlayerClipPlayback::Clamp);
        for (gender, source_path_id) in [
            (PlayerRigGender::Male, male_path_id),
            (PlayerRigGender::Female, female_path_id),
        ] {
            let request = TutorialPlayerAnimationRequest::avatar_emote_code(gender, code)
                .expect("exact AvatarEmote request");
            assert_eq!(request.clip, clip);
            assert_eq!(request.source_path_id, source_path_id);
            assert_eq!(
                request.dispatch,
                TutorialPlayerAnimationDispatch::CrossFade(Duration::from_secs_f32(
                    AVATAR_EMOTE_CODE_CROSS_FADE_SECONDS,
                ))
            );
            assert!(request.sets_emote_state);
            assert!(request.hide_hand_attachment);
            assert!(!request.reset_move_direction);
        }
    }
    for unsupported in [i32::MIN, -1, 0, 25, 26, 27, 28, 29, 30, 45, i32::MAX] {
        assert!(TutorialPlayerClip::from_avatar_emote_code(unsupported).is_none());
        assert!(
            TutorialPlayerAnimationRequest::avatar_emote_code(PlayerRigGender::Male, unsupported,)
                .is_none()
        );
    }
}

#[test]
fn retrobution_emote_codes_use_their_distinct_native_clips_for_both_genders() {
    let expected = [
        (31, "ffr_dance_02", 644, 644),
        (32, "ffr_dance_04", 624, 624),
        (33, "ffr_dance_07", 641, 641),
        (34, "ffr_dance_08", 629, 629),
        (35, "ffr_dance_10", 607, 607),
        (36, "ffr_dance_13", 604, 604),
        (37, "ffr_dance_15", 610, 610),
        (38, "ffr_dance_18", 646, 646),
        (39, "ffr_dance_19", 637, 637),
        (40, "ffr_dance_20", 585, 585),
        (41, "ffr_dance_bully", 642, 645),
        (42, "ffr_dance_tellme", 648, 648),
        (43, "ffr_emote_catpose", 639, 639),
        (44, "ffr_emote_idolpose", 623, 623),
    ];
    for (code, name, male_path_id, female_path_id) in expected {
        let clip = TutorialPlayerClip::from_avatar_emote_code(code).unwrap();
        assert_eq!(clip.name(), name);
        assert_eq!(clip.playback(), TutorialPlayerClipPlayback::Clamp);
        for (gender, source_path_id) in [
            (PlayerRigGender::Male, male_path_id),
            (PlayerRigGender::Female, female_path_id),
        ] {
            let request = TutorialPlayerAnimationRequest::avatar_emote_code(gender, code).unwrap();
            assert_eq!(request.clip, clip);
            assert_eq!(request.source_path_id, source_path_id);
            assert_eq!(
                request.dispatch,
                TutorialPlayerAnimationDispatch::CrossFade(Duration::from_secs_f32(
                    AVATAR_EMOTE_CODE_CROSS_FADE_SECONDS
                ))
            );
        }
    }
}

#[test]
fn primary_direct_avatar_emotes_all_use_immediate_play() {
    for name in TUTORIAL_DIRECT_AVATAR_EMOTE_NAMES {
        let request =
            TutorialPlayerAnimationRequest::avatar_emote(PlayerRigGender::Male, name).unwrap();
        assert_eq!(request.clip.name(), name);
        assert_eq!(
            request.dispatch,
            TutorialPlayerAnimationDispatch::PlayImmediate
        );
        assert!(request.sets_emote_state);
        assert!(request.hide_hand_attachment);
        assert!(!request.reset_move_direction);
    }
    for unsupported in ["stand1", "Standing", "RUN", ""] {
        assert!(
            TutorialPlayerAnimationRequest::avatar_emote(PlayerRigGender::Male, unsupported)
                .is_err()
        );
    }
}

#[test]
fn ffr_custom_emotes_crossfade_and_preserve_gender_sources() {
    assert_eq!(FFR_CUSTOM_AVATAR_EMOTE_NAMES.len(), 14);
    for gender in [PlayerRigGender::Male, PlayerRigGender::Female] {
        for name in FFR_CUSTOM_AVATAR_EMOTE_NAMES {
            let request = TutorialPlayerAnimationRequest::avatar_emote(gender, name).unwrap();
            assert!(request.clip.is_ffr_custom());
            assert_eq!(request.clip.playback(), TutorialPlayerClipPlayback::Clamp);
            assert_eq!(
                request.dispatch,
                TutorialPlayerAnimationDispatch::CrossFade(Duration::from_secs_f32(
                    FFR_CUSTOM_EMOTE_CROSS_FADE_SECONDS,
                ))
            );
            assert!(request.sets_emote_state);
            assert!(request.hide_hand_attachment);
            assert!(!request.reset_move_direction);
        }
    }
    assert_eq!(
        TutorialPlayerClip::FfrDanceBully.source_path_id(PlayerRigGender::Male),
        642
    );
    assert_eq!(
        TutorialPlayerClip::FfrDanceBully.source_path_id(PlayerRigGender::Female),
        645
    );
}

#[test]
fn stand_force_is_stand1_crossfade_not_staying() {
    let request = TutorialPlayerAnimationRequest::stand_force(PlayerRigGender::Female);
    assert_eq!(request.clip, TutorialPlayerClip::Stand1);
    assert_ne!(request.clip, TutorialPlayerClip::Staying);
    assert_eq!(request.source_path_id, 237_027);
    assert_eq!(
        request.dispatch,
        TutorialPlayerAnimationDispatch::CrossFade(Duration::from_secs_f32(0.15))
    );
    assert!(!request.sets_emote_state);
    assert!(!request.hide_hand_attachment);
    assert!(request.reset_move_direction);
}

#[test]
fn tutorial_weapon_matrix_matches_retrobution_and_never_uses_134() {
    for class in [-1, 0, 1, 2, 3, 4, 5, 99] {
        let request = tutorial_weapon_request(true, class).unwrap();
        assert_eq!(request.slot, CharacterEquipSlot0104::Hand);
        assert_eq!(request.item.item_type, 0);
        assert_eq!(request.item.item_id, 328);
        assert_eq!(request.item.option, 0);
        assert_eq!(request.item.time_limit, 0);
    }
    assert_eq!(tutorial_weapon_request(false, 2).unwrap().item.item_id, 43);
    assert_eq!(tutorial_weapon_request(false, 3).unwrap().item.item_id, 328);
    assert_eq!(tutorial_weapon_request(false, 4).unwrap().item.item_id, 197);
    for class in [-1, 0, 1, 5, 99] {
        assert_eq!(tutorial_weapon_request(false, class), None);
    }
    for request in [
        tutorial_weapon_request(true, 0),
        tutorial_weapon_request(false, 2),
        tutorial_weapon_request(false, 3),
        tutorial_weapon_request(false, 4),
    ]
    .into_iter()
    .flatten()
    {
        assert_ne!(request.item.item_id, 134);
    }
}

#[test]
fn renderer_neutral_queue_preserves_order_without_claiming_playback() {
    let mut queue = TutorialPlayerPresentationCommandQueue::default();
    let emote = queue
        .avatar_emote(PlayerRigGender::Male, "staying")
        .unwrap();
    let stand = queue.stand_force(PlayerRigGender::Male);
    let weapon = queue.tutorial_weapon(false, 2).unwrap();
    assert_eq!(queue.len(), 3);
    assert!(queue.has_pending_direct_pose_override());
    assert_eq!(queue.pending_weapon_item_id(), Some(43));
    assert_eq!(
        queue.pop_front(),
        Some(TutorialPlayerPresentationCommand::Animation(emote))
    );
    assert_eq!(
        queue.pop_front(),
        Some(TutorialPlayerPresentationCommand::Animation(stand))
    );
    assert_eq!(
        queue.pop_front(),
        Some(TutorialPlayerPresentationCommand::Equipment(weapon))
    );
    assert!(queue.is_empty());

    assert_eq!(queue.tutorial_weapon(false, 0), None);
    assert!(queue.is_empty());
}

#[test]
fn attack_cancels_pending_player_emotes_but_preserves_scripted_poses_and_equipment() {
    for gender in [PlayerRigGender::Male, PlayerRigGender::Female] {
        for code in 1..=24 {
            let mut queue = TutorialPlayerPresentationCommandQueue::default();
            queue.avatar_emote_code(gender, code);
            queue.cancel_pending_avatar_emotes();
            assert!(queue.is_empty());
        }
        let mut queue = TutorialPlayerPresentationCommandQueue::default();
        queue.avatar_emote(gender, "staying").unwrap();
        queue.stand_force(gender);
        queue.tutorial_weapon(false, 2).unwrap();
        let before = queue.pending.clone();
        queue.cancel_pending_avatar_emotes();
        assert_eq!(queue.pending, before);
    }
}

#[test]
fn attack_discards_late_repeat_echo_once_without_blocking_new_emotes() {
    let mut queue = TutorialPlayerPresentationCommandQueue::default();
    queue.record_emote_continuation_sent(24);
    queue.cancel_pending_avatar_emotes();
    assert!(queue.accept_avatar_emote_echo(6));
    assert!(!queue.accept_avatar_emote_echo(24));
    assert!(queue.accept_avatar_emote_echo(24));
    queue.record_emote_continuation_sent(17);
    assert!(queue.accept_avatar_emote_echo(17));
    assert!(queue.emote_continuation_echoes.is_empty());
}

#[test]
fn consumer_preserves_fifo_and_does_not_claim_renderer_application() {
    let gender = PlayerRigGender::Male;
    let capabilities = TutorialPlayerRigCapabilities::from_contract(
        gender,
        &[clip_contract(
            gender,
            TutorialPlayerClip::Staying,
            7,
            TUTORIAL_PLAYER_RUNTIME_READY_STATUS,
        )],
    )
    .unwrap();
    let consumer = TutorialPlayerPresentationConsumer::new(capabilities);
    let mut queue = TutorialPlayerPresentationCommandQueue::default();
    let staying = queue.avatar_emote(gender, "staying").unwrap();
    let weapon = queue.tutorial_weapon(true, 0).unwrap();

    assert_eq!(
        consumer.consume_next(&mut queue),
        Some(
            TutorialPlayerPresentationResolution::AnimationContractResolved(
                ContractResolvedTutorialPlayerAnimation {
                    request: staying,
                    gltf_animation_index: 7,
                    playback: TutorialPlayerClipPlayback::Clamp,
                }
            )
        )
    );
    assert_eq!(
        consumer.consume_next(&mut queue),
        Some(TutorialPlayerPresentationResolution::EquipmentForwarded(
            weapon
        ))
    );
    assert_eq!(consumer.consume_next(&mut queue), None);
}
