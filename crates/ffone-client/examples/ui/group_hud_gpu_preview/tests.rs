use ffone_client::group_ui::group_ui_view;
use sha2::{Digest, Sha256};

use super::*;

#[test]
fn sample_exercises_remote_nano_freechat_and_cooperative_npc_gap() {
    let view = group_ui_view(&source_backed_sample("en"));

    assert!(view.visible);
    assert_eq!(view.legacy_group_size, 3);
    assert_eq!(view.pc_rows.len(), 2);
    assert_eq!(view.pc_rows[0].rect.top, 100.0);
    assert!(view.pc_rows[0].free_chat);
    assert_eq!(
        view.pc_rows[0]
            .nano
            .as_ref()
            .and_then(|nano| nano.skill_icon_path.as_deref()),
        Some(PREVIEW_SKILL_ICON_PATH)
    );
    assert_eq!(view.pc_rows[1].rect.top, 150.0);
    assert_eq!(view.npc_row.unwrap().rect.top, 250.0);
}

#[test]
fn cli_selects_language_and_exact_capture_names() {
    assert_eq!(
        parse_preview_args([]).unwrap(),
        (
            PathBuf::from("target/ui-parity/group-hud-en-1264x681.png"),
            "en".to_owned()
        )
    );
    assert_eq!(
        parse_preview_args([OsString::from("--language"), OsString::from("ru")]).unwrap(),
        (
            PathBuf::from("target/ui-parity/group-hud-ru-1264x681.png"),
            "ru".to_owned()
        )
    );
    assert!(
        parse_preview_args([
            OsString::from("capture.jpg"),
            OsString::from("--language"),
            OsString::from("en")
        ])
        .is_err()
    );
    assert!(parse_preview_args([OsString::from("--language"), OsString::from("de")]).is_err());
}

#[test]
fn measured_client_area_does_not_replace_reference_space() {
    assert_eq!((CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT), (1_264, 681));
    assert_eq!(
        (GROUP_UI_REFERENCE_WIDTH, GROUP_UI_REFERENCE_HEIGHT),
        (1_280.0, 720.0)
    );
}

#[test]
fn production_bundles_and_replacement_font_pin_group_text_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for language in ["en", "ru"] {
        let runtime = fs::read(
            root.join("assets/game/localization")
                .join(format!("{language}.json")),
        )
        .unwrap();
        let document = serde_json::from_slice::<serde_json::Value>(&runtime).unwrap();
        let entries = document["entries"].as_object().unwrap();
        assert_eq!(entries["ui.group.level"], "{level}");
        assert_eq!(entries["ui.content.passthrough"], "{text}");
    }

    let font = fs::read(root.join("assets/game").join(GROUP_UI_FONT_PATH)).unwrap();
    assert_eq!(font.len(), GROUP_UI_FONT_BYTES);
    assert_eq!(format!("{:x}", Sha256::digest(font)), GROUP_UI_FONT_SHA256);
}
