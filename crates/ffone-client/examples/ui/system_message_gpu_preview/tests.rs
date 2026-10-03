use ffone_client::system_message_ui::{
    SYSTEM_MESSAGE_CANCEL_RECT, SYSTEM_MESSAGE_OK_RECT, SystemMessageChoice,
    system_message_button_layout, system_message_layer_layout,
};

use super::*;

#[test]
fn authority_frame_contains_two_layers_but_only_newest_buttons() {
    let older = system_message_layer_layout(
        Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32),
        0,
        1.0,
    );
    let newer = system_message_layer_layout(
        Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32),
        1,
        1.0,
    );
    assert_eq!((older.left, older.top), (342.0, 268.5));
    assert_eq!((newer.left, newer.top), (352.0, 278.5));

    let active = system_message_button_layout(SystemMessageButtonType::OkCancel);
    assert_eq!(active.primary.choice, SystemMessageChoice::Primary);
    assert_eq!(active.primary.rect, SYSTEM_MESSAGE_OK_RECT);
    assert_eq!(active.secondary.unwrap().rect, SYSTEM_MESSAGE_CANCEL_RECT);
}

#[test]
fn cli_selects_every_reached_body_branch_and_language() {
    let (mode, language, output) = parse_preview_args([
        OsString::from("delete-mission"),
        OsString::from("--language"),
        OsString::from("ru"),
    ])
    .unwrap();
    assert_eq!(mode, PreviewMode::DeleteMission);
    assert_eq!(language, "ru");
    assert_eq!(
        output,
        PathBuf::from("target/ui-parity/system-message-delete-mission-ru-1264x681.png")
    );

    assert!(parse_preview_args([OsString::from("--language")]).is_err());
    assert!(
        parse_preview_args([OsString::from("generic"), OsString::from("combi-failure")])
            .is_err()
    );
}

#[test]
fn preview_modes_cover_exact_text_and_button_counts() {
    assert_eq!(PreviewMode::Generic.expected_text_count(), 3);
    assert_eq!(PreviewMode::Generic.expected_button_count(), 2);
    assert_eq!(PreviewMode::DeleteMission.expected_text_count(), 6);
    assert_eq!(PreviewMode::DeleteMission.expected_button_count(), 2);
    assert_eq!(PreviewMode::CombinationFailure.expected_text_count(), 3);
    assert_eq!(PreviewMode::CombinationFailure.expected_button_count(), 1);
}
