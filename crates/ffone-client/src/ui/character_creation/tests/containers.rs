use super::*;

#[test]
fn reached_guiskin_text_styles_keep_exact_serialized_metrics() {
    assert_eq!(
        CharacterCreationTextStyle::Label.spec(),
        CharacterCreationTextStyleSpec {
            source_style: "label",
            source_font_path_id: CHARACTER_CREATION_JEFFE_16_PATH_ID,
            font_role: CharacterCreationFontRole::Jeffe,
            font_size: 14.0,
            line_height: 16.451_999_66,
            padding: [0.0, 0.0, 3.0, 3.0],
            anchor: CharacterCreationTextAnchor::MiddleCenter,
            normal_color: [0.799_270_1, 1.0, 1.0, 1.0],
            word_wrap: true,
            clip: true,
            content_offset: [0.0, 0.0],
            y_offset: 0.0,
        }
    );
    assert_eq!(
        CharacterCreationTextStyle::Toggle.spec().padding,
        [0.0, -30.0, 0.0, 0.0]
    );
    assert_eq!(
        CharacterCreationTextStyle::TabButton.spec().padding,
        [0.0, 0.0, 0.0, 5.0]
    );
    assert_eq!(
        CharacterCreationTextStyle::TabText.spec().padding,
        [0.0, 0.0, 4.0, 7.0]
    );
    assert_eq!(
        CharacterCreationTextStyle::NameDisplay.spec().padding,
        [10.0, 6.0, 4.0, 6.0]
    );
    assert_eq!(
        CharacterCreationTextStyle::TextField.spec().padding,
        [3.0; 4]
    );
    assert_eq!(
        CharacterCreationTextStyle::TextField.spec().anchor,
        CharacterCreationTextAnchor::MiddleLeft
    );
    assert_eq!(
        CharacterCreationTextStyle::Transparent4
            .spec()
            .source_font_path_id,
        CHARACTER_CREATION_CHALET_SMALL_PATH_ID
    );
    assert_eq!(
        CharacterCreationTextStyle::Transparent5
            .spec()
            .source_font_path_id,
        CHARACTER_CREATION_CHALET_REGULAR_PATH_ID
    );
    for style in [
        CharacterCreationTextStyle::Label,
        CharacterCreationTextStyle::Transparent,
        CharacterCreationTextStyle::SectionLabel,
        CharacterCreationTextStyle::Toggle,
        CharacterCreationTextStyle::BodyText,
        CharacterCreationTextStyle::Button,
        CharacterCreationTextStyle::ButtonTabFont,
        CharacterCreationTextStyle::ExitButton,
        CharacterCreationTextStyle::TabButton,
        CharacterCreationTextStyle::TabText,
        CharacterCreationTextStyle::OrText,
        CharacterCreationTextStyle::NameDisplay,
        CharacterCreationTextStyle::Transparent4,
        CharacterCreationTextStyle::Transparent5,
        CharacterCreationTextStyle::CustomQuestion,
        CharacterCreationTextStyle::TextField,
    ] {
        assert_eq!(style.spec().content_offset, [0.0, 0.0]);
        assert_eq!(style.spec().y_offset, 0.0);
    }
}
