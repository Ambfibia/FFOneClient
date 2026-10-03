use super::*;
use ffone_client::bank_ui::{BankSlotFrameVisual, bank_mode_view_with_pc_stuff};

#[test]
fn cli_defaults_to_required_acceptance_path_and_validates_png_shape() {
    assert_eq!(
        parse_cli(Vec::<std::ffi::OsString>::new()).unwrap(),
        PreviewCli {
            language: "en".to_owned(),
            output: default_output("en"),
        }
    );
    assert_eq!(
        parse_cli([std::ffi::OsString::from("ru")]).unwrap(),
        PreviewCli {
            language: "ru".to_owned(),
            output: default_output("ru"),
        }
    );
    assert!(parse_cli([std::ffi::OsString::from("de")]).is_err());
    assert!(
        parse_cli([
            std::ffi::OsString::from("en"),
            std::ffi::OsString::from("wrong.jpg")
        ])
        .is_err()
    );
    assert!(
        parse_cli([
            std::ffi::OsString::from("en"),
            std::ffi::OsString::from("one.png"),
            std::ffi::OsString::from("two.png")
        ])
        .is_err()
    );
}

#[test]
fn preview_is_final_authoritative_full_access_with_checker_fallback() {
    let projection = preview_projection();
    assert!(projection.has_full_access());
    let state = BankUiState {
        phase: BankLifecyclePhase::Visible,
        opening_elapsed_seconds: BANK_OPEN_SECONDS,
        ..default()
    };
    let view = bank_mode_view_with_pc_stuff(
        CLIENT_AREA_WIDTH,
        CLIENT_AREA_HEIGHT,
        state,
        BankModalState::default(),
        &projection,
        BankPcStuffAuthority0104::from_authoritative_taros(12_345_678),
        true,
    )
    .unwrap();
    assert_eq!(
        view.layout.bank_panel,
        ffone_client::bank_ui::BankUiRect::new(122.0, 21.0, 495.0, 638.0)
    );
    assert_eq!(view.bank[0].count_label.as_deref(), Some("99"));
    assert_eq!(view.bank[0].frame_visual, BankSlotFrameVisual::Occupied);
    assert_eq!(
        view.taros_digits,
        ["0", "1", "2", "3", "4", "5", "6", "7", "8"].map(str::to_owned)
    );
    assert!(matches!(
        view.bank[2].icon,
        ffone_client::user_equip_ui::UserEquipPresentationIcon::MissingChecker
    ));
}
