use super::*;
use ffone_client::user_equip_ui::{
    UserEquipPresentationIcon, UserEquipSlotFrameVisual, user_equip_item_mode_view,
};

#[test]
fn cli_defaults_to_required_acceptance_path_and_validates_png_shape() {
    assert_eq!(
        parse_cli(Vec::<std::ffi::OsString>::new()).unwrap(),
        PreviewCli {
            output: PathBuf::from(DEFAULT_OUTPUT),
            nano: false,
        }
    );
    assert_eq!(
        parse_cli([std::ffi::OsString::from("custom.png")])
            .unwrap()
            .output,
        PathBuf::from("custom.png")
    );
    assert_eq!(
        parse_cli([std::ffi::OsString::from("--nano")]).unwrap(),
        PreviewCli {
            output: PathBuf::from(DEFAULT_NANO_OUTPUT),
            nano: true,
        }
    );
    assert!(parse_cli([std::ffi::OsString::from("custom.jpg")]).is_err());
    assert!(
        parse_cli([
            std::ffi::OsString::from("a.png"),
            std::ffi::OsString::from("b.png")
        ])
        .is_err()
    );
}

#[test]
fn fixture_contains_resolved_empty_combined_count_and_missing_states() {
    let projection = preview_projection();
    let mut state = UserEquipUiState::default();
    state.open_item_mode();
    state.tick(USER_EQUIP_OPEN_SECONDS);
    let view = user_equip_item_mode_view(
        CLIENT_AREA_WIDTH,
        CLIENT_AREA_HEIGHT,
        state,
        UserEquipModalState::default(),
        &projection,
        true,
    )
    .unwrap();

    assert!(matches!(
        view.inventory[0].icon,
        UserEquipPresentationIcon::Resolved(_)
    ));
    assert!(view.inventory[0].combined_badge.is_some());
    assert_eq!(view.inventory[1].count_label.as_deref(), Some("1"));
    assert!(matches!(
        view.inventory[2].icon,
        UserEquipPresentationIcon::Resolved(_)
    ));
    assert_eq!(
        view.inventory[3].frame_visual,
        UserEquipSlotFrameVisual::Empty
    );
    assert_eq!(view.inventory[3].icon, UserEquipPresentationIcon::Empty);
    assert_eq!(view.inventory[12].count_label.as_deref(), Some("Quest 401"));
    assert!(view.equipment[0].combined_badge.is_some());
    assert_eq!(
        view.equipment[4].frame_visual,
        UserEquipSlotFrameVisual::Empty
    );
}

#[test]
fn fixture_populates_status_currency_and_battery_presentation() {
    let presentation = preview_presentation_context();
    assert_eq!(presentation.player_name, "Dexter");
    assert_eq!((presentation.hp, presentation.max_hp), (1_782, 2_000));
    assert_eq!(
        (presentation.fusion_matter, presentation.max_fusion_matter),
        (8_750, 10_000)
    );
    assert_eq!(presentation.taros, 125_430);
    assert_eq!(
        (presentation.weapon_battery, presentation.nano_battery),
        (64, 29)
    );
}

#[test]
fn nano_fixture_contains_owned_unowned_equipped_power_and_three_status_slots() {
    let projection = preview_nano_projection();
    assert_eq!(projection.gallery.len(), 36);
    let owned = projection
        .gallery
        .iter()
        .find(|entry| entry.nano_id == 1)
        .unwrap();
    assert!(owned.owned && owned.equipped);
    assert_eq!(owned.current_power, Some(1));
    let unowned = projection
        .gallery
        .iter()
        .find(|entry| !entry.owned)
        .unwrap();
    assert!(matches!(
        unowned.icon,
        UserEquipPresentationIcon::Resolved(ref path) if path.contains("/ready/nanoready_")
    ));
    assert_eq!(projection.status.len(), 3);
    assert_eq!(projection.status[0].nano_id, Some(1));
    assert_eq!(projection.status[2].nano_id, None);
}
