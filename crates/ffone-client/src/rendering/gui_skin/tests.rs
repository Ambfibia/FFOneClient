use crate::gui_skin::*;

#[test]
fn editor_candidate_is_rejected_even_if_its_schema_is_relabelled() {
    let mut spoofed_candidate: serde_json::Value =
        serde_json::from_str(RETROBUTION_GUI_SKINS).unwrap();
    spoofed_candidate["evidenceLevel"] = serde_json::json!("candidate");
    spoofed_candidate["publicationAllowed"] = serde_json::json!(false);
    let error = parse_gui_skin_contract(&spoofed_candidate.to_string())
        .expect_err("an Editor candidate must never pass as the historical runtime snapshot");
    assert!(error.contains("editor-only GUI-skin candidate marker"));
    assert!(error.contains("evidenceLevel"));

    let mut wrong_schema: serde_json::Value =
        serde_json::from_str(RETROBUTION_GUI_SKINS).unwrap();
    wrong_schema["schema"] =
        serde_json::json!("fusionforge.legacy-unity-gui-skin-candidate.v1");
    let error = parse_gui_skin_contract(&wrong_schema.to_string())
        .expect_err("candidate schema must fail closed");
    assert!(error.contains("unsupported historical GUI-skin schema"));
}

#[test]
fn historical_snapshot_contains_all_clean_retrobution_skins() {
    assert_eq!(contract().skins.len(), 14);
    assert_eq!(
        gui_style("FusionFallMissionSkinR", "AllowTab")
            .unwrap()
            .border,
        GuiInsets {
            left: 10,
            right: 10,
            top: 40,
            bottom: 25,
        }
    );
    assert_eq!(
        gui_style("FusionFallMissionSkinR", "Active_Tab")
            .unwrap()
            .states["normal"]
            .background
            .asset_name
            .as_deref(),
        Some("active_tab")
    );
    assert_eq!(
        gui_effective_font("FusionFallMissionSkinR", "label")
            .unwrap()
            .asset_name
            .as_deref(),
        Some("ChaletBook-Regular Small")
    );
    assert_eq!(
        gui_style("FusionFallHUDSkin", "messagetext")
            .unwrap()
            .states["normal"]
            .text_color,
        GuiColor {
            r: 1.0,
            g: 0.995_967_745_780_944_8,
            b: 1.0,
            a: 1.0,
        }
    );
    assert_eq!(
        gui_style("FusionFallInteractionSkin", "M_Top")
            .unwrap()
            .border,
        GuiInsets {
            left: 20,
            right: 45,
            top: 0,
            bottom: 0,
        }
    );
}
