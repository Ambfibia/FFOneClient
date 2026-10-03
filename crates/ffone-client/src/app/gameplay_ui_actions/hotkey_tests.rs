use super::*;
use ffone_client::gameplay_ui::GameplayMenuTransition;

fn hotkey_app(mentor: i16) -> App {
    let (mut app, _, _) = super::warp_tests::world_warp_app();
    app.add_plugins((bevy::app::TaskPoolPlugin::default(), AssetPlugin::default()))
        .init_asset::<AudioSource>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<GameplayMenuTransition>()
        .init_resource::<GameplayUiAudioOutbox>()
        .init_resource::<UserEquipModalState>()
        .init_resource::<UserEquipUiOutbox>()
        .init_resource::<LauncherUiModel>();
    app.world_mut()
        .spawn((PrimaryWindow, CursorOptions::default()));
    *app.world_mut().resource_mut::<GameplayUiOutbox>() = default();
    let mut mission = MissionUiModel::default();
    mission.enabled = true;
    *app.world_mut().resource_mut::<MissionUiModel>() = mission;
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    let offset = ffone_protocol::PcLoadData0104::MENTOR_OFFSET;
    load.as_bytes_mut()[offset..offset + 2].copy_from_slice(&mentor.to_le_bytes());
    let inventory_offset = ffone_protocol::PcLoadData0104::INVENTORY_OFFSET;
    load.as_bytes_mut()[inventory_offset + 2..inventory_offset + 4]
        .copy_from_slice(&1_i16.to_le_bytes());
    app.world_mut()
        .resource_mut::<GuideRuntime>()
        .load_pc_state(&load);
    app.world_mut()
        .resource_mut::<LocalInventoryRuntime>()
        .snapshot = Some(InventoryRuntime0104::from_pc_load(77, &load));
    {
        let mut status = app.world_mut().resource_mut::<RuntimeStatus>();
        status.player_id = Some(77);
        status.candy = 0;
    }
    app.add_systems(
        Update,
        (
            route_world_ui_shortcuts.before(consume_world_gameplay_ui_outbox),
            drive_email_production_pre_interaction_0104
                .after(route_world_ui_shortcuts)
                .before(consume_world_gameplay_ui_outbox),
            consume_email_production_outputs_0104.after(consume_world_gameplay_ui_outbox),
        ),
    );
    app
}

fn press(app: &mut App, key: KeyCode) {
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.reset_all();
    keys.press(key);
    app.update();
    // This headless fixture omits the production audio consumer. Consume the
    // close cue before the next opening checks its clean effect boundary.
    app.world_mut().resource_mut::<EmailUiAudioOutbox>().0.clear();
}

#[test]
fn hotkeys_p_opens_and_closes_production_mail_for_all_five_mentors() {
    for mentor in 1..=5 {
        let mut app = hotkey_app(mentor);
        for _ in 0..2 {
            press(&mut app, KeyCode::KeyP);
            assert!(
                app.world().resource::<EmailUiModel>().visible,
                "mentor {mentor}: {}",
                app.world().resource::<RuntimeStatus>().message
            );
            assert!(
                app.world()
                    .resource::<EmailProductionRuntime0104>()
                    .modal_active()
            );
            assert!(app.world().resource::<EmailUiModel>().inventory[0].is_some());
            assert!(
                app.world()
                    .resource::<EmailUiModel>()
                    .guide_messages
                    .iter()
                    .any(|message| message.mode == 2),
                "mentor {mentor}: opening the real mailbox must include available invitations"
            );
            assert_eq!(
                app.world()
                    .resource::<EmailProductionShell0104>()
                    .lease
                    .unwrap()
                    .game_mode,
                18
            );
            press(&mut app, KeyCode::KeyP);
            assert!(
                !app.world().resource::<EmailUiModel>().visible,
                "mentor {mentor}: {}",
                app.world().resource::<RuntimeStatus>().message
            );
            assert!(
                !app.world()
                    .resource::<EmailProductionRuntime0104>()
                    .modal_active()
            );
        }
    }
}

#[test]
fn hotkeys_future_mail_projects_computress_sender_and_fifth_message_column() {
    let app = hotkey_app(5);
    let catalog = app.world().resource::<EmailProductionCatalog0104>();
    let messages = email_guide_messages_0104(
        catalog,
        app.world().resource::<TutorialMissionContent>(),
        app.world().resource::<GuideRuntime>(),
        [],
        [558],
    )
    .unwrap();
    let [message] = messages.as_slice() else {
        panic!("Future task 558 must have its guide message")
    };
    assert_eq!(message.sender_npc_id, 1171);
    assert_eq!(
        message.content,
        *catalog
            .guide_rows
            .iter()
            .find(|row| row.task_id == 558)
            .unwrap()
            .mentor_copy[4]
            .as_ref()
            .unwrap()
    );
    assert!(!message.sender_name.is_empty());
}

#[test]
fn hotkeys_inventory_nano_and_help_reach_production_modes() {
    use ffone_client::user_equip_ui::UserEquipMode;
    for (key, mode) in [
        (KeyCode::KeyI, UserEquipMode::Item),
        (KeyCode::KeyN, UserEquipMode::Nano),
    ] {
        let mut app = hotkey_app(5);
        press(&mut app, key);
        let state = app.world().resource::<UserEquipUiState>();
        assert!(state.is_active());
        assert_eq!(state.mode(), mode);
    }
    let mut app = hotkey_app(5);
    press(&mut app, KeyCode::KeyH);
    assert!(!app.world().resource::<GameGuideUiModel>().modal_active());
    app.world_mut().resource_mut::<OptionProductionRuntime>()
        .input.mappings.iter_mut()
        .find(|row| row.action == LegacyOptionAction::Help).unwrap()
        .primary = LegacyInputBinding::Key(LegacyPhysicalKey::F);
    app.world_mut().resource_mut::<OptionProductionRuntime>()
        .input.mappings.iter_mut()
        .find(|row| row.action == LegacyOptionAction::Skill2).unwrap()
        .primary = LegacyInputBinding::Unbound;
    press(&mut app, KeyCode::KeyF);
    assert!(app.world().resource::<GameGuideUiModel>().modal_active());
    press(&mut app, KeyCode::KeyF);
    assert!(!app.world().resource::<GameGuideUiModel>().modal_active());
}

#[test]
fn hotkeys_mail_compose_keeps_p_as_text_and_closes_with_configured_escape() {
    use ffone_client::email_ui::EmailScreen;
    let mut app = hotkey_app(5);
    press(&mut app, KeyCode::KeyP);
    assert!(app.world().resource::<EmailUiModel>().visible);
    app.world_mut().resource_mut::<EmailUiModel>().screen = EmailScreen::Compose;
    press(&mut app, KeyCode::KeyP);
    assert!(app.world().resource::<EmailUiModel>().visible);
    press(&mut app, KeyCode::Backquote);
    assert!(!app.world().resource::<EmailUiModel>().visible);
    assert!(
        !app.world()
            .resource::<EmailProductionRuntime0104>()
            .modal_active()
    );
}

#[test]
fn email_opens_for_characters_without_a_supported_mentor() {
    for mentor in [0, -1, 6] {
        let mut app = hotkey_app(mentor);
        press(&mut app, KeyCode::KeyP);
        assert!(app.world().resource::<EmailUiModel>().visible,
            "mentor {mentor}: {}", app.world().resource::<RuntimeStatus>().message);
        press(&mut app, KeyCode::KeyP);
        assert!(!app.world().resource::<EmailUiModel>().visible);
    }
}

#[test]
fn email_npc_arrival_arms_hud_while_mailbox_is_closed() {
    let mut app = hotkey_app(5);
    app.init_resource::<ffone_client::gameplay_ui::MinimapNewMailAlarm>();
    app.update();
    assert!(!app.world().resource::<EmailUiModel>().visible);
    assert!(app.world().resource::<MissionUiModel>().nanocom_new_mail_notice_visible);
    assert!(app.world().resource::<ffone_client::gameplay_ui::MinimapNewMailAlarm>().active());
    *app.world_mut().resource_mut::<ffone_client::gameplay_ui::MinimapNewMailAlarm>() = default();
    app.update();
    assert!(!app.world().resource::<ffone_client::gameplay_ui::MinimapNewMailAlarm>().active());
}

#[test]
fn email_opens_with_reported_vehicle_and_bank_card_inventories_and_unknown_artwork() {
    for (mentor, items) in [
        (4_i16, vec![(10_i16, 105_i16), (10, 42)]),
        (1, vec![(7, 5), (7, 29), (7, 32), (7, 30)]),
        (1, vec![(7, 32767)]),
    ] {
        let mut app = hotkey_app(mentor);
        let mut load = ffone_protocol::PcLoadData0104::zeroed();
        for (slot, &(item_type, item_id)) in items.iter().enumerate() {
            let offset = ffone_protocol::PcLoadData0104::INVENTORY_OFFSET + slot * ItemBase0104::SIZE;
            let bytes = &mut load.as_bytes_mut()[offset..offset + ItemBase0104::SIZE];
            bytes[..2].copy_from_slice(&item_type.to_le_bytes());
            bytes[2..4].copy_from_slice(&item_id.to_le_bytes());
            bytes[4..8].copy_from_slice(&1_i32.to_le_bytes());
        }
        app.world_mut().resource_mut::<LocalInventoryRuntime>().snapshot =
            Some(InventoryRuntime0104::from_pc_load(77, &load));
        press(&mut app, KeyCode::KeyP);
        let model = app.world().resource::<EmailUiModel>();
        assert!(model.visible, "{}", app.world().resource::<RuntimeStatus>().message);
        for (slot, &(_, item_id)) in items.iter().enumerate() {
            let item = model.inventory[slot].as_ref().unwrap();
            assert_eq!(item.icon_path.is_some(), item_id != 32767);
        }
        press(&mut app, KeyCode::KeyP);
        assert!(!app.world().resource::<EmailUiModel>().visible);
    }
}
