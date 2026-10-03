use super::*;

#[test]
fn reached_gui_styles_preserve_serialized_metrics_without_guessed_y_shifts() {
    let expected = [
        (
            NanocomGuiStyleRole::BigFont14,
            "BIGFont14",
            903,
            3,
            NanocomGuiInsets::ZERO,
        ),
        (
            NanocomGuiStyleRole::MessageText,
            "messagetext",
            1_018,
            0,
            NanocomGuiInsets::new(10.0, 4.0, 6.0, 6.0),
        ),
        (
            NanocomGuiStyleRole::MessageTitle,
            "MessageTitle",
            934,
            3,
            NanocomGuiInsets::ZERO,
        ),
        (
            NanocomGuiStyleRole::CenterBox2,
            "centerbox2",
            1_018,
            1,
            NanocomGuiInsets::new(10.0, 4.0, 6.0, 6.0),
        ),
        (
            NanocomGuiStyleRole::Button,
            "button",
            903,
            4,
            NanocomGuiInsets::new(10.0, 3.0, 6.0, 6.0),
        ),
        (
            NanocomGuiStyleRole::RedButton,
            "RedButton",
            903,
            4,
            NanocomGuiInsets::new(10.0, 4.0, 6.0, 6.0),
        ),
    ];
    for (role, name, font_path_id, alignment, padding) in expected {
        let style = nanocom_gui_style(role);
        assert_eq!(style.skin_path_id, 1_372);
        assert_eq!(style.style_name, name);
        assert_eq!(style.source_font_path_id, font_path_id);
        assert_eq!(style.alignment, alignment);
        assert_eq!(style.padding, padding);
        assert_eq!(style.content_offset, [0.0, 0.0]);
        assert_eq!(style.replacement_y_offset, 0.0);
    }
    assert_eq!(NANOCOM_CHALET_SMALL_FONT_SIZE, 11.0);
    assert_eq!(NANOCOM_CHALET_SMALL_LINE_HEIGHT, 12.071_999_55);
    assert_eq!(NANOCOM_MESSAGE_TITLE_LINE_HEIGHT, 17.822_999_95);
    assert_eq!(NANOCOM_UI_Z_INDEX, -1);
}
