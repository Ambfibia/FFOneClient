use super::*;
use ffone_protocol::WirePayload;
#[test]
fn shared_redeem_validates_exact_free_chat_without_service_dependency() {
    assert_eq!(redeem_code_request("ab"), Err(RedeemCodeError::TooShort));
    assert_eq!(
        redeem_code_request(&"x".repeat(33)),
        Err(RedeemCodeError::TooLong)
    );
    assert_eq!(
        redeem_code_request("a b"),
        Err(RedeemCodeError::ContainsSpace)
    );
    let request = redeem_code_request("BeMore").unwrap();
    let existing = crate::enchant_ui::enchant_redeem_wire_0104("BeMore").unwrap();
    assert_eq!(request.encode(), existing.payload);
    let mut input = SharedInputDialog::default();
    let mut redeem = SharedRedeemCode::default();
    let bank = RedeemSource::Bank { pc: 1, npc: 2 };
    assert!(redeem.open(bank, &mut input));
    assert!(!redeem.open(RedeemSource::Vendor { pc: 3, npc: 4 }, &mut input));
    assert_eq!(redeem.source, Some(bank));
    assert_eq!(input.owner(), Some(REDEEM_INPUT_OWNER));
    for locale in ["en", "ru"] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../assets/game/localization/{locale}.json"));
        let bundle: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        for key in [
            "ui.shared_input.redeem_title",
            "ui.shared_input.redeem_space_error",
        ] {
            assert!(bundle["entries"][key].as_str().is_some());
        }
    }
}
#[test]
fn entry_preserves_owner_and_utf16_limit_without_splitting_surrogates() {
    let mut model = SharedInputDialog::default();
    let request = SharedInputRequest {
        owner: 1,
        title: LocalizedText::new("title", ""),
        instruction: LocalizedText::new("instruction", ""),
        submit: LocalizedText::new("submit", ""),
        max_utf16: 4,
    };
    assert!(model.open(request.clone()));
    assert!(!model.open(request));
    model.append("я😀xZ\n");
    assert_eq!(model.value, "я😀x");
    model.close(2);
    assert_eq!(model.owner(), Some(1));
    model.close(1);
    assert_eq!(model.owner(), None);
    assert!(model.value.is_empty());
}

#[test]
fn production_entry_labels_have_matching_localized_placeholders() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game/localization");
    let en: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("en.json")).unwrap()).unwrap();
    let ru: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("ru.json")).unwrap()).unwrap();
    let en = en["entries"].as_object().unwrap();
    let ru = ru["entries"].as_object().unwrap();
    assert_eq!(en.keys().collect::<Vec<_>>(), ru.keys().collect::<Vec<_>>());
    for key in [
        "ui.shared_input.value",
        "ui.shared_input.cancel",
        "ui.shared_input.submit",
        "ui.shared_input.redeem",
        "ui.shared_input.redeem_instruction",
        "ui.enchant.redeem_code",
    ] {
        assert!(en[key].is_string() && ru[key].is_string());
    }
    assert_eq!(en["ui.shared_input.value"], "{value}");
    assert_eq!(ru["ui.shared_input.value"], "{value}");
}
