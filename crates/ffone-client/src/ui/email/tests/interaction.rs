use super::*;

#[test]
fn ui_sfx_subject_limit_sounds_once_per_text_event_and_respects_modals() {
    use bevy::{input::{ButtonState, keyboard::{Key, KeyboardInput}}, prelude::*};
    use crate::email_ui::interaction::handle_email_keyboard;
    let mut app = App::new();
    let mut model = EmailUiModel {
        visible: true,
        screen: EmailScreen::Compose,
        compose_focus: EmailComposeFocus::Subject,
        opening_elapsed_seconds: EMAIL_UI_OPEN_SECONDS,
        ..default()
    };
    model.draft.set_subject("x".repeat(EMAIL_SUBJECT_INPUT_LIMIT - 1));
    app.insert_resource(model)
        .init_resource::<EmailUiOutbox>()
        .init_resource::<EmailUiAudioOutbox>()
        .add_message::<KeyboardInput>()
        .add_systems(Update, handle_email_keyboard);
    let input = |text: &str| KeyboardInput {
        key_code: KeyCode::KeyA,
        logical_key: Key::Character(text.into()),
        text: Some(text.into()),
        state: ButtonState::Pressed,
        repeat: false,
        window: Entity::PLACEHOLDER,
    };
    app.world_mut().write_message(input("а"));
    app.update();
    assert_eq!(app.world_mut().resource_mut::<EmailUiAudioOutbox>().pop(), None);
    app.world_mut().write_message(input("ещё текст"));
    app.update();
    assert_eq!(app.world().resource::<EmailUiModel>().draft.subject.encode_utf16().count(), EMAIL_SUBJECT_INPUT_LIMIT);
    assert_eq!(app.world_mut().resource_mut::<EmailUiAudioOutbox>().pop(), Some(EmailUiAudioCue::CharacterLimitMax));
    assert_eq!(app.world_mut().resource_mut::<EmailUiAudioOutbox>().pop(), None);
    app.world_mut().resource_mut::<EmailUiModel>().popup = EmailPopup::BuddyList;
    app.world_mut().write_message(input("з"));
    app.update();
    assert_eq!(app.world_mut().resource_mut::<EmailUiAudioOutbox>().pop(), None);
}

#[test]
fn buddy_scroll_applies_the_clean_two_stage_clamp_and_content_limit() {
    let mut model = EmailUiModel {
        visible: true,
        screen: EmailScreen::Compose,
        popup: EmailPopup::BuddyList,
        opening_elapsed_seconds: EMAIL_UI_OPEN_SECONDS,
        buddies: vec![EmailBuddy::default(); 10],
        ..default()
    };
    assert_eq!(email_buddy_scroll_max(model.buddies.len()), 122.0);
    assert!(model.apply_buddy_scroll_axis(-1.0));
    assert_eq!(model.buddy_scroll_y(), 30.0);
    for _ in 0..10 {
        assert!(model.apply_buddy_scroll_axis(-10.0));
    }
    assert_eq!(model.buddy_scroll_y(), 122.0);
    assert!(model.apply_buddy_scroll_axis(1.0));
    assert_eq!(model.buddy_scroll_y(), 92.0);
}
