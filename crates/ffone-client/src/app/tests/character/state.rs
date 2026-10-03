use super::*;

#[test]
fn gameplay_loading_barrier_renders_after_character_selection_and_its_modal() {
    assert!(
        GAMEPLAY_LOADING_CAMERA_ORDER
            > ffone_client::character_selection_ui::CHARACTER_SELECTION_MODAL_CAMERA_ORDER
    );
}

#[test]
fn authoritative_world_hand_weapon_selection_ignores_presentation_lag() {
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    let hand_offset = ffone_protocol::PcLoadData0104::EQUIPMENT_OFFSET
        + ffone_protocol::CharacterEquipSlot0104::Hand as usize
            * ffone_protocol::ItemBase0104::SIZE;
    load.as_bytes_mut()[hand_offset + 2..hand_offset + 4].copy_from_slice(&197_i16.to_le_bytes());
    let mut inventory = LocalInventoryRuntime::default();
    inventory.seed(77, &load);
    let stale_presentation = WorldPlayerEquipmentProjection {
        hand: Some(ItemBase0104 {
            item_type: 0,
            item_id: 43,
            option: 0,
            time_limit: 0,
        }),
        ..default()
    };

    assert_eq!(
        authoritative_world_hand_weapon_item_id(&inventory),
        Some(197)
    );
    assert_eq!(stale_presentation.hand.unwrap().item_id, 43);
}

#[test]
fn hud_portrait_follows_authoritative_equipment_without_opening_inventory() {
    let data = CharacterCreationData::open(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"),
    )
    .unwrap();
    let mut character = entry_test_character(9, 1, 1);
    character.style.face_style = 1;
    character.style.hair_style = 1;
    character.style.eye_color = 6;
    character.style.skin_color = 1;
    character.style.hair_color = 1;
    character.style.height = 2;
    character.style.body = 1;
    let mut other = character.clone();
    other.pc_uid = 10;
    other.slot = 2;
    let initial = data.resolve_character_summary(&character).unwrap();
    let shirt = data
        .resolve_creator(9, 1, "Test", "Entry", &CharacterAppearance::default())
        .unwrap()
        .equipped
        .upper_body_id;
    let mut equipped = character.clone();
    equipped.equipment[ffone_protocol::CharacterEquipSlot0104::UpperBody as usize] =
        ffone_protocol::EquippedItem0104 {
            item_type: 1,
            item_id: shirt,
            ..default()
        };
    let expected = data.resolve_character_summary(&equipped).unwrap();
    assert_ne!(initial, expected);
    let data = Arc::new(data);

    for state in [ClientState::World, ClientState::Tutorial] {
        let mut app = App::new();
        app.insert_resource(State::new(state))
            .insert_resource(RuntimeStatus {
                core: RuntimePlayerStatus {
                    player_id: Some(77),
                    ..default()
                },
                roster: RuntimeRosterStatus {
                    characters: vec![character.clone(), other.clone()],
                    selected_uid: Some(9),
                    ..default()
                },
                ..default()
            })
            .insert_resource(LoadedCharacterCreationData(data.clone()))
            .init_resource::<OptionProductionRuntime>()
            .init_resource::<LocalInventoryRuntime>()
            .init_resource::<CharacterSelectionUiModel>()
            .init_resource::<CharacterSelectionPortraitsModel>()
            .init_resource::<GameplayPlayerPortraitModel>()
            .add_systems(Update, character_flow::sync_character_selection_ui);
        app.update();
        assert_eq!(
            app.world()
                .resource::<CharacterSelectionPortraitsModel>()
                .slots[0]
                .look(),
            Some(&initial)
        );
        app.world_mut()
            .resource_mut::<LocalInventoryRuntime>()
            .seed(77, &ffone_protocol::PcLoadData0104::zeroed());
        for item in [
            ItemBase0104 {
                item_type: 1,
                item_id: shirt,
                option: 0,
                time_limit: 0,
            },
            ItemBase0104 {
                item_type: 0,
                item_id: 0,
                option: 0,
                time_limit: 0,
            },
        ] {
            app.world_mut()
                .resource_mut::<LocalInventoryRuntime>()
                .snapshot_mut()
                .unwrap()
                .apply_equip_change(ffone_protocol::EquipChangePacket0104 {
                    pc_id: 77,
                    equip_slot_num: ffone_protocol::CharacterEquipSlot0104::UpperBody as i32,
                    equip_slot_item: item,
                })
                .unwrap();
            app.update();
            let portraits = app.world().resource::<CharacterSelectionPortraitsModel>();
            assert_eq!(
                portraits.slots[0].look(),
                Some(if item.item_id == 0 {
                    &initial
                } else {
                    &expected
                })
            );
            assert_eq!(portraits.slots[1].revision(), 1);
            let revision = portraits.slots[0].revision();
            app.update();
            assert_eq!(
                app.world()
                    .resource::<CharacterSelectionPortraitsModel>()
                    .slots[0]
                    .revision(),
                revision
            );
        }
        assert_eq!(
            app.world().resource::<RuntimeStatus>().roster.characters[0].equipment,
            character.equipment
        );
        app.world_mut().resource_mut::<RuntimeStatus>().player_id = Some(88);
        app.update();
        assert!(matches!(
            app.world()
                .resource::<CharacterSelectionPortraitsModel>()
                .slots[0]
                .status,
            CharacterSelectionPortraitStatus::Blocked(_)
        ));
        app.insert_resource(State::new(ClientState::CharacterSelect));
        app.update();
        assert_eq!(
            app.world()
                .resource::<CharacterSelectionPortraitsModel>()
                .slots[0]
                .look(),
            Some(&initial)
        );
        assert_eq!(
            *app.world().resource::<GameplayPlayerPortraitModel>(),
            GameplayPlayerPortraitModel::default()
        );
    }
}

#[test]
fn character_selection_runtime_maps_one_based_slots_names_and_worldname() {
    fn summary(
        slot: i8,
        pc_uid: i64,
        first_name: &str,
        last_name: &str,
        position: [i32; 3],
        tutorial_flag: i8,
    ) -> CharacterSummary {
        CharacterSummary {
            slot,
            level: if pc_uid == 30 { 36 } else { 1 },
            pc_uid,
            first_name: first_name.to_owned(),
            last_name: last_name.to_owned(),
            position,
            style: ffone_protocol::CharacterStyle0104 {
                name_check: 1,
                gender: 0,
                face_style: 0,
                hair_style: 0,
                hair_color: 0,
                skin_color: 0,
                eye_color: 0,
                height: 0,
                body: 0,
                class: 0,
                appearance_flag: 1,
                tutorial_flag,
                payzone_flag: 0,
            },
            equipment: [ffone_protocol::EquippedItem0104::default();
                ffone_protocol::CHARACTER_EQUIP_SLOT_COUNT_0104],
        }
    }

    let slots = character_slots_from_runtime(&[
        summary(1, 10, "Test", "Ser", [0, 0, 0], 0),
        summary(3, 30, "Gaia", "Roundbreath", [563_200, 51_200, 0], 1),
    ]);
    let CharacterSlotUi::Occupied(first) = &slots[0] else {
        panic!("one-based slot 1 was not mapped to UI slot 0");
    };
    assert_eq!(first.display_name, "Test Ser");
    assert_eq!(first.location_label(), "TECH SQUARE - THE FUTURE");
    assert!(matches!(slots[1], CharacterSlotUi::Empty));
    let CharacterSlotUi::Occupied(third) = &slots[2] else {
        panic!("one-based slot 3 was not mapped to UI slot 2");
    };
    assert_eq!(third.display_name, "Gaia Roundbreath");
    assert_eq!(third.level, 36);
    assert_eq!(third.location_label(), "GENIUS GROVE - THE FUTURE");
    assert_eq!(third.background, CharacterLocationBackground::Future);
    assert!(matches!(slots[3], CharacterSlotUi::Empty));
}

#[test]
fn character_entry_pressed_during_selection_settle_is_replayed_once() {
    let mut loading = GameplayLoadingState::default();
    loading.begin(ResourceLoadingScope::CharacterSelection);
    let mut entry = BufferedCharacterEntry::default();

    entry.queue(42, &loading);
    assert_eq!(entry.take_when_ready(&loading), None);

    loading.finish();
    assert_eq!(entry.take_when_ready(&loading), Some(42));
    assert_eq!(entry.take_when_ready(&loading), None);
}

#[test]
fn returning_to_selection_clears_a_stale_buffered_entry() {
    let mut loading = GameplayLoadingState::default();
    loading.begin(ResourceLoadingScope::CharacterSelection);
    let mut entry = BufferedCharacterEntry::default();
    entry.queue(42, &loading);

    entry.clear();
    loading.finish();
    assert_eq!(entry.take_when_ready(&loading), None);
}

#[test]
fn character_selection_handler_discards_buffered_entry_outside_selection() {
    for state in [
        ClientState::World,
        ClientState::Login,
        ClientState::CharacterCreate,
    ] {
        let mut loading = GameplayLoadingState::default();
        loading.begin(ResourceLoadingScope::CharacterSelection);
        let mut entry = BufferedCharacterEntry::default();
        entry.queue(42, &loading);
        loading.finish();
        let mut runtime = RuntimeStatus::default();
        runtime
            .roster
            .characters
            .push(entry_test_character(42, 1, 1));

        let mut app = App::new();
        app.insert_resource(State::new(state))
            .insert_resource(NextState::<ClientState>::default())
            .insert_resource(loading)
            .insert_resource(entry)
            .insert_resource(runtime)
            .insert_resource(NetworkBridge::start())
            .init_resource::<CharacterSelectionUiOutbox>()
            .init_resource::<CharacterSelectionUiModel>()
            .init_resource::<NativePlayerPreviewModel>()
            .init_resource::<CharacterCreationSession>()
            .init_resource::<TutorialSession>()
            .init_resource::<CharacterCreationUiModel>()
            .init_resource::<OptionProductionRuntime>()
            .add_message::<AppExit>()
            .add_systems(Update, handle_character_selection_ui_actions);
        app.update();

        assert!(!app.world().resource::<GameplayLoadingState>().visible);
        assert_eq!(
            app.world()
                .resource::<RuntimeStatus>()
                .roster
                .pending_character_entry_uid,
            None
        );
        assert_eq!(
            app.world_mut()
                .resource_mut::<BufferedCharacterEntry>()
                .take_when_ready(&GameplayLoadingState::default()),
            None
        );
    }
}

#[test]
fn selection_reentry_resets_loading_but_preserves_an_active_shard_handshake() {
    for pending_uid in [None, Some(42)] {
        let mut runtime = RuntimeStatus::default();
        runtime.roster.pending_character_entry_uid = pending_uid;
        let mut loading = GameplayLoadingState::default();
        loading.begin(ResourceLoadingScope::World);
        let mut app = App::new();
        app.insert_resource(runtime)
            .insert_resource(loading)
            .init_resource::<BufferedCharacterEntry>()
            .init_resource::<TutorialSession>()
            .add_systems(Update, begin_character_selection_loading);
        app.update();
        assert_eq!(
            app.world().resource::<GameplayLoadingState>().scope,
            Some(if pending_uid.is_some() {
                ResourceLoadingScope::World
            } else {
                ResourceLoadingScope::CharacterSelection
            })
        );
    }
}
