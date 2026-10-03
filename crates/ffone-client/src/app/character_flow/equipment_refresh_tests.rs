use super::*;
use crate::app::runtime_status::{RuntimePlayerStatus, RuntimeRosterStatus};

fn character(uid: i64, slot: i8) -> CharacterSummary {
    CharacterSummary {
        pc_uid: uid,
        slot,
        level: 1,
        first_name: "Equip".into(),
        last_name: "Refresh".into(),
        position: [0; 3],
        style: ffone_protocol::CharacterStyle0104 {
            name_check: 1,
            gender: 1,
            face_style: 1,
            hair_style: 1,
            hair_color: 1,
            skin_color: 1,
            eye_color: 6,
            height: 2,
            body: 1,
            class: 0,
            appearance_flag: 1,
            tutorial_flag: 1,
            payzone_flag: 0,
        },
        equipment: [Default::default(); ffone_protocol::CHARACTER_EQUIP_SLOT_COUNT_0104],
    }
}

#[test]
fn equipment_and_location_survive_alternating_characters_and_retained_roster_refreshes() {
    let login = vec![character(9, 1), character(10, 2)];
    let mut runtime = RuntimeStatus {
        roster: RuntimeRosterStatus {
            characters: login.clone(),
            ..default()
        },
        ..default()
    };
    for (uid, pc_id) in [(9, 77), (10, 88), (9, 99)] {
        runtime.player_id = Some(pc_id);
        runtime.roster.selected_uid = Some(uid);
        let mut inventory =
            InventoryRuntime0104::from_pc_load(pc_id, &ffone_protocol::PcLoadData0104::zeroed());
        for slot in 0..ffone_protocol::CHARACTER_EQUIP_SLOT_COUNT_0104 {
            inventory
                .apply_equip_change(ffone_protocol::EquipChangePacket0104 {
                    pc_id,
                    equip_slot_num: slot as i32,
                    equip_slot_item: ItemBase0104 {
                        item_type: slot as i16,
                        item_id: pc_id as i16 + slot as i16,
                        option: 0x1234_0001,
                        time_limit: 12345,
                    },
                })
                .unwrap();
        }
        runtime.retain_local_equipment(&inventory);
        let position = [pc_id * 100, pc_id * 200, 300];
        runtime.retain_local_position(position);
        let expected = runtime.roster.characters.clone();
        // The last equip reply and return may arrive in one network poll.
        runtime.clear_world();
        runtime
            .roster
            .replace_retained_login_characters(login.iter().rev().cloned().collect());
        for old in expected {
            let current = runtime
                .roster
                .characters
                .iter()
                .find(|c| c.pc_uid == old.pc_uid)
                .unwrap();
            assert_eq!(current.equipment, old.equipment);
            assert_eq!(current.position, old.position);
        }
        let current = runtime
            .roster
            .characters
            .iter()
            .find(|c| c.pc_uid == uid)
            .unwrap();
        assert_eq!(current.position, position);
        assert_eq!(
            current.equipment,
            (*inventory.equipment()).map(equipped_item_from_inventory_item)
        );
    }
    // Removing an item must overwrite the nonempty login slot as well.
    runtime.player_id = Some(99);
    runtime.roster.selected_uid = Some(9);
    runtime.retain_local_equipment(&InventoryRuntime0104::from_pc_load(
        99,
        &ffone_protocol::PcLoadData0104::zeroed(),
    ));
    runtime.clear_world();
    runtime.roster.replace_retained_login_characters(login);
    assert!(
        runtime.roster.characters[0]
            .equipment
            .iter()
            .all(|item| item.item_id == 0)
    );
    assert_ne!(runtime.roster.characters[1].equipment[1].item_id, 0);
}

#[test]
fn equipment_refresh_respects_creation_owner_deletion_and_new_login() {
    let original = character(9, 1);
    let mut created = original.clone();
    created.equipment[1].item_id = 10;
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_id: Some(77),
            ..default()
        },
        roster: RuntimeRosterStatus {
            characters: vec![original],
            selected_uid: Some(9),
            ..default()
        },
        ..default()
    };
    // SAVE_CHAR creates starter clothing for the same UID as the reserved name.
    runtime
        .roster
        .replace_retained_login_characters(vec![created.clone()]);
    assert_eq!(runtime.roster.characters[0].equipment, created.equipment);
    let empty = InventoryRuntime0104::from_pc_load(88, &ffone_protocol::PcLoadData0104::zeroed());
    runtime.retain_local_equipment(&empty);
    assert!(runtime.roster.world_snapshot_uids.is_empty());
    assert_eq!(runtime.roster.characters[0].equipment, created.equipment);
    runtime.player_id = Some(88);
    runtime.retain_local_equipment(&empty);
    runtime.roster.replace_retained_login_characters(Vec::new());
    assert!(runtime.roster.world_snapshot_uids.is_empty());
    runtime
        .roster
        .replace_retained_login_characters(vec![created.clone()]);
    assert_eq!(runtime.roster.characters[0].equipment, created.equipment);
    runtime.retain_local_equipment(&empty);
    runtime.clear_world();
    runtime.roster = default(); // Connecting starts a new authenticated roster.
    runtime
        .roster
        .replace_retained_login_characters(vec![created.clone()]);
    assert_eq!(runtime.roster.characters[0].equipment, created.equipment);
}

#[test]
fn equipment_refresh_rebinds_inventory_hud_selection_preview_and_all_roster_portraits() {
    let data = Arc::new(
        CharacterCreationData::open(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"),
        )
        .unwrap(),
    );
    let login = vec![character(9, 1), character(10, 2)];
    let shirt = data
        .resolve_creator(
            9,
            1,
            "Equip",
            "Refresh",
            &CharacterAppearance::default(),
        )
        .unwrap()
        .equipped
        .upper_body_id;
    let mut app = App::new();
    app.insert_resource(State::new(ClientState::World))
        .insert_resource(RuntimeStatus {
            core: RuntimePlayerStatus {
                player_id: Some(77),
                ..default()
            },
            roster: RuntimeRosterStatus {
                characters: login.clone(),
                selected_uid: Some(9),
                ..default()
            },
            ..default()
        })
        .insert_resource(LoadedCharacterCreationData(data.clone()))
        .init_resource::<OptionProductionRuntime>()
        .init_resource::<LocalInventoryRuntime>()
        .init_resource::<UserEquipUiState>()
        .init_resource::<ffone_client::barber::BarberModel>()
        .init_resource::<LocalVehiclePresentationRuntime>()
        .init_resource::<UserEquipAvatarPreviewPresentation>()
        .init_resource::<CharacterCreationSession>()
        .init_resource::<CharacterCreationUiModel>()
        .init_resource::<CharacterSelectionUiModel>()
        .init_resource::<CharacterSelectionPortraitsModel>()
        .init_resource::<GameplayPlayerPortraitModel>()
        .init_resource::<NativePlayerPreviewModel>()
        .add_systems(
            Update,
            (sync_character_selection_ui, sync_native_player_preview).chain(),
        );
    app.world_mut()
        .resource_mut::<UserEquipUiState>()
        .open_item_mode();
    app.world_mut()
        .resource_mut::<LocalInventoryRuntime>()
        .seed(77, &ffone_protocol::PcLoadData0104::zeroed());
    for item_id in [shirt, 0] {
        app.insert_resource(State::new(ClientState::World));
        app.world_mut().resource_mut::<RuntimeStatus>().player_id = Some(77);
        app.world_mut()
            .resource_mut::<RuntimeStatus>()
            .roster
            .selected_uid = Some(9);
        app.world_mut()
            .resource_mut::<LocalInventoryRuntime>()
            .snapshot_mut()
            .unwrap()
            .apply_equip_change(ffone_protocol::EquipChangePacket0104 {
                pc_id: 77,
                equip_slot_num: 1,
                equip_slot_item: ItemBase0104 {
                    item_id,
                    item_type: 1,
                    option: 0,
                    time_limit: 0,
                },
            })
            .unwrap();
        app.world_mut()
            .resource_scope(|world, mut runtime: Mut<RuntimeStatus>| {
                runtime.retain_local_equipment(
                    world
                        .resource::<LocalInventoryRuntime>()
                        .snapshot()
                        .unwrap(),
                );
                runtime.retain_local_position([680000, 80000, 300]);
            });
        app.update();
        let expected = data
            .resolve_character_summary(
                &app.world().resource::<RuntimeStatus>().roster.characters[0],
            )
            .unwrap();
        assert_eq!(
            app.world().resource::<NativePlayerPreviewModel>().look(),
            Some(&expected)
        );
        assert_eq!(
            app.world()
                .resource::<CharacterSelectionPortraitsModel>()
                .slots[0]
                .look(),
            Some(&expected)
        );
        let before = character_summary_ui_slot(
            &app.world().resource::<RuntimeStatus>().roster.characters[0],
        );
        assert_ne!(before, character_summary_ui_slot(&login[0]));
        {
            let mut runtime = app.world_mut().resource_mut::<RuntimeStatus>();
            runtime.clear_world();
            runtime
                .roster
                .replace_retained_login_characters(login.clone());
        }
        app.insert_resource(State::new(ClientState::CharacterSelect));
        for uid in [10, 9] {
            app.world_mut()
                .resource_mut::<RuntimeStatus>()
                .roster
                .selected_uid = Some(uid);
            app.update();
            let expected_selected = data
                .resolve_character_summary(
                    app.world()
                        .resource::<RuntimeStatus>()
                        .roster
                        .characters
                        .iter()
                        .find(|c| c.pc_uid == uid)
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(
                app.world().resource::<NativePlayerPreviewModel>().look(),
                Some(&expected_selected)
            );
            assert_eq!(
                app.world()
                    .resource::<CharacterSelectionPortraitsModel>()
                    .slots[0]
                    .look(),
                Some(&expected)
            );
            assert_eq!(
                app.world()
                    .resource::<CharacterSelectionPortraitsModel>()
                    .slots[1]
                    .look(),
                Some(&data.resolve_character_summary(&login[1]).unwrap())
            );
            assert_eq!(
                character_summary_ui_slot(
                    &app.world().resource::<RuntimeStatus>().roster.characters[0]
                ),
                before
            );
        }
    }
}
