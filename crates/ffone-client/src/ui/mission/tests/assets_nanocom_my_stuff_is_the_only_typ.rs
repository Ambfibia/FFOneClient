use super::*;

pub(super) fn asset_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game")
}

#[test]
fn nanocom_my_stuff_is_the_only_typed_user_equip_entry_route() {
    let mut model = MissionUiModel {
        enabled: true,
        ..default()
    };
    let mut outbox = GameplayUiOutbox::default();

    assert!(model.open_nanocom_menu(&mut outbox));
    assert!(model.open_user_equip_from_nanocom(&mut outbox));
    assert!(!model.nanocom_main_menu_visible);
    assert!(!model.open_user_equip_from_nanocom(&mut outbox));
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![
            GameplayUiAction::OpenNanocomMenu,
            GameplayUiAction::OpenUserEquipItemMode {
                source: UserEquipOpenSource::NanocomMyStuff,
            },
            GameplayUiAction::CloseNanocomMenu,
        ]
    );
}
