use super::*;
const PRODUCTION: &[u8] =
    include_bytes!("../../../../assets/game/ui/en/gameplay/quit-menu/menu.ffquit.json");
#[test]
fn background_policy_preserves_old_documents_and_rejects_padding_box_shrinkage() {
    let mut value: serde_json::Value = serde_json::from_slice(PRODUCTION).unwrap();
    for style in value["styles"].as_array_mut().unwrap() {
        style.as_object_mut().unwrap().remove("backgroundBox");
    }
    let old = QuitMenuDocument::from_json(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(
        old.styles
            .iter()
            .all(|style| style.background_box == QuitMenuBackgroundBox::BorderBox)
    );
    value["styles"][0]["backgroundBox"] = serde_json::json!("content-box");
    assert!(QuitMenuDocument::from_json(&serde_json::to_vec(&value).unwrap()).is_err());
}
#[test]
fn production_document_preserves_accepted_actions_fonts_and_locales() {
    let d = QuitMenuDocument::from_json(PRODUCTION).unwrap();
    assert_eq!(d.dialog_size, [206.0, 188.0]);
    assert_eq!(d.font, "fonts/jeffe.otf");
    assert_eq!(
        d.buttons.map(|b| b.action),
        [
            QuitMenuAction::ChangeCharacter,
            QuitMenuAction::QuitGame,
            QuitMenuAction::Cancel
        ]
    );
    for bytes in [
        include_bytes!("../../../../assets/game/localization/en.json").as_slice(),
        include_bytes!("../../../../assets/game/localization/ru.json").as_slice(),
    ] {
        let locale: serde_json::Value = serde_json::from_slice(bytes).unwrap();
        let d = QuitMenuDocument::from_json(PRODUCTION).unwrap();
        for b in d.buttons {
            assert!(locale["entries"].get(&b.localization_key).is_some());
        }
    }
}
#[test]
fn unknown_behavior_unsafe_paths_and_duplicate_actions_fail() {
    for (field, value) in [
        ("schema", serde_json::json!("future")),
        ("font", serde_json::json!("../font.otf")),
        ("dialogSize", serde_json::json!([0, 188])),
    ] {
        let mut d: serde_json::Value = serde_json::from_slice(PRODUCTION).unwrap();
        d[field] = value;
        assert!(QuitMenuDocument::from_json(&serde_json::to_vec(&d).unwrap()).is_err());
    }
    let mut d: serde_json::Value = serde_json::from_slice(PRODUCTION).unwrap();
    d["buttons"][0]["action"] = serde_json::json!("invented-action");
    assert!(QuitMenuDocument::from_json(&serde_json::to_vec(&d).unwrap()).is_err());
    let mut d = QuitMenuDocument::from_json(PRODUCTION).unwrap();
    d.buttons[1].action = d.buttons[0].action;
    assert!(d.validate().is_err());
}

#[test]
fn compensation_is_separate_from_source_offset_and_validated() {
    let mut d = QuitMenuDocument::from_json(PRODUCTION).unwrap();
    assert_eq!(d.styles[0].font_compensation, [0.0; 2]);
    d.styles[0].content_offset = [2.0, 1.0];
    d.styles[0].font_compensation = [0.0, -3.0];
    assert_eq!(d.styles[0].text_offset(), [2.0, -2.0]);
    assert_eq!(d.styles[0].content_offset, [2.0, 1.0]);
    d.styles[0].active = Some("../unapproved.png".into());
    assert!(d.validate().is_err());
    d.styles[0].active = None;
    d.styles[0].font_compensation[1] = f32::NAN;
    assert!(d.validate().is_err());
}
