use super::*;
use ffone_client::vendor_ui::{
    VendorEquipValidation0104, VendorPresentationIcon0104, vendor_mode_view,
};

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
fn preview_is_final_buy_tab_with_restriction_and_checker_fallback() {
    let projection = preview_projection();
    let state = VendorUiState {
        phase: VendorLifecyclePhase::Visible,
        opening_elapsed_seconds: VENDOR_OPEN_SECONDS,
        ..default()
    };
    let view = vendor_mode_view(
        CLIENT_AREA_WIDTH,
        CLIENT_AREA_HEIGHT,
        state,
        VendorModalState::default(),
        &projection,
        true,
    )
    .unwrap();
    assert_eq!(view.rows.len(), 8);
    assert_eq!(
        view.rows[4].equip_validation,
        VendorEquipValidation0104::Rejected
    );
    assert!(matches!(
        view.rows[7].icon,
        VendorPresentationIcon0104::MissingChecker(_)
    ));
    assert_eq!(view.rows[4].vehicle_speed_class, Some(700));
    assert_eq!(view.inventory[2].quest_item_id, Some(300));
    assert_eq!((view.weapon_battery, view.nano_battery), (25, 40));
    // Camera availability is owned by the production service portrait system.
}
