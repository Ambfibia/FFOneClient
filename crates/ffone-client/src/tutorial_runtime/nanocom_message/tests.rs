use crate::tutorial_nanocom_message::*;

#[test]
fn exact_type_9_contract_and_semantic_paths_are_stable() {
    assert_eq!(TUTORIAL_NANOCOM_NPC_TYPE, 2_671);
    assert_eq!(TUTORIAL_NANOCOM_MESSAGE_TYPE, 9);
    assert_eq!(TUTORIAL_NANOCOM_HEAD_LIFETIME_SECONDS, 10.0);
    assert_eq!(TUTORIAL_NANOCOM_REVEAL_SECONDS, 0.5);
    assert_eq!(
        TUTORIAL_NANOCOM_NPC_FRAME_PATH,
        "ui/en/gameplay/nanocom/message/npc.png"
    );
    assert_eq!(
        TUTORIAL_NANOCOM_NUMBUH_TWO_ICON_PATH,
        "icons/entities/npc/npcicon_87.png"
    );
    assert_eq!(TUTORIAL_NANOCOM_TITLE_STYLE.legacy_name, "MenuBigFont14");
    assert_eq!(TUTORIAL_NANOCOM_TITLE_STYLE.source_font_name, "JEFFE___14");
    assert_eq!(TUTORIAL_NANOCOM_TITLE_STYLE.source_font_path_id, 903);
    assert_eq!(TUTORIAL_NANOCOM_TITLE_STYLE.font_size, 12.0);
    assert_eq!(TUTORIAL_NANOCOM_TITLE_STYLE.line_height, 13.71);
    assert_eq!(TUTORIAL_NANOCOM_TITLE_STYLE.legacy_alignment, 3);
    assert_eq!(
        TUTORIAL_NANOCOM_TITLE_STYLE.semantic_font_path,
        TUTORIAL_NANOCOM_TITLE_FONT_PATH
    );
    assert_eq!(TUTORIAL_NANOCOM_TITLE_STYLE.color, [0.0, 1.0, 1.0, 1.0]);
    assert_eq!(TUTORIAL_NANOCOM_BODY_STYLE.legacy_name, "MenuMessageText");
    assert_eq!(
        TUTORIAL_NANOCOM_BODY_STYLE.source_font_name,
        "ChaletBook-Regular Small"
    );
    assert_eq!(TUTORIAL_NANOCOM_BODY_STYLE.source_font_path_id, 1_018);
    assert_eq!(TUTORIAL_NANOCOM_BODY_STYLE.font_size, 11.0);
    assert_eq!(TUTORIAL_NANOCOM_BODY_STYLE.line_height, 12.072);
    assert_eq!(TUTORIAL_NANOCOM_BODY_STYLE.legacy_alignment, 0);
    assert_eq!(
        TUTORIAL_NANOCOM_BODY_STYLE.padding,
        TutorialNanocomPadding::new(10.0, 4.0, 6.0, 6.0)
    );
    assert_eq!(
        TUTORIAL_NANOCOM_TEXT_CONTENT_RECT,
        TutorialNanocomRect::new(130.0, 29.0, 164.0, 60.0)
    );
}

#[test]
fn reveal_geometry_matches_the_legacy_squared_sine_slide() {
    let hidden = tutorial_nanocom_message_geometry(1_280.0, 1.0);
    assert!((hidden.panel.left - 1_158.0).abs() < 0.001);
    assert!(hidden.panel.width.abs() < 0.001);

    let shown = tutorial_nanocom_message_geometry(1_280.0, 0.0);
    assert!((shown.panel.left - 786.0).abs() < 0.001);
    assert!((shown.panel.width - 372.0).abs() < 0.001);
    assert_eq!(shown.frame, TUTORIAL_NANOCOM_NPC_FRAME_RECT);
    assert_eq!(shown.icon, TUTORIAL_NANOCOM_NPC_ICON_RECT);
    assert_eq!(shown.title, TUTORIAL_NANOCOM_TITLE_RECT);
    assert_eq!(shown.body, TUTORIAL_NANOCOM_TEXT_RECT);
    assert!(
        (1_280.0
            - shown.panel.left
            - shown.panel.width
            - TUTORIAL_NANOCOM_REVEALED_RIGHT_MARGIN)
            .abs()
            < 0.001
    );
}

#[test]
fn queue_head_lives_ten_seconds_and_emits_both_slide_sounds() {
    let mut queue = TutorialNanocomMessageQueue::default();
    queue.enqueue_type_9_numbuh_two(
        LocalizedText::new("content.npc.2671.name", "Numbuh Two"),
        LocalizedText::new("test.nanocom.body", "Report in!"),
    );
    let mut state = TutorialNanocomMessageState::default();
    let mut sounds = TutorialNanocomMessageSoundQueue::default();
    let context = TutorialNanocomMessageContext::default();

    state.tick(&mut queue, &mut sounds, context, 0.0);
    assert_eq!(
        state.active().unwrap().title,
        LocalizedText::new("content.npc.2671.name", "Numbuh Two")
    );
    assert!(queue.is_empty());
    assert_eq!(
        sounds.pop_front(),
        Some(TutorialNanocomMessageSound::SlideIn)
    );
    state.tick(&mut queue, &mut sounds, context, 0.5);
    assert_eq!(state.reveal_parameter(), 0.0);
    state.tick(&mut queue, &mut sounds, context, 9.5);
    assert_eq!(state.head_remaining_seconds(), 0.0);
    assert!(state.active().is_some());
    state.tick(&mut queue, &mut sounds, context, 0.001);
    assert!(state.active().is_none());
    assert_eq!(
        sounds.pop_front(),
        Some(TutorialNanocomMessageSound::SlideOut)
    );
}

#[test]
fn scene_event_hides_and_freezes_the_live_queue_head() {
    let mut queue = TutorialNanocomMessageQueue::default();
    queue.enqueue_type_9_numbuh_two(
        LocalizedText::new("content.npc.2671.name", "Numbuh Two"),
        LocalizedText::new("test.nanocom.body", "Frozen"),
    );
    let mut state = TutorialNanocomMessageState::default();
    let mut sounds = TutorialNanocomMessageSoundQueue::default();
    state.tick(
        &mut queue,
        &mut sounds,
        TutorialNanocomMessageContext::default(),
        1.0,
    );
    let before = state.head_remaining_seconds();
    let hidden = TutorialNanocomMessageContext {
        scene_event_active: true,
    };
    state.tick(&mut queue, &mut sounds, hidden, 30.0);
    assert_eq!(state.head_remaining_seconds(), before);
    assert!(!state.is_visible(hidden));
    assert!(state.active().is_some());
}
