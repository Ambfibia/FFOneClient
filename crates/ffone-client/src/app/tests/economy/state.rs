use super::*;

#[test]
fn inventory_computer_events_select_exact_clean_effect_and_delay_by_vehicle_family() {
    assert_eq!(
        user_equip_computer_effect(LegacyVehiclePresentationFamily::None),
        (425, 0.25)
    );
    assert_eq!(
        user_equip_computer_effect(LegacyVehiclePresentationFamily::Board),
        (833, 0.3)
    );
    assert_eq!(
        user_equip_computer_effect(LegacyVehiclePresentationFamily::Scooter),
        (834, 0.3)
    );
}

#[test]
fn cashmall_production_projection_uses_real_item_mode_and_no_fake_slot9_rows() {
    let mut shared = UserEquipItemModeProjection::default();
    shared.owner_pc_id = 77;
    let mut projection = CashmallModeProjection0104::default();

    sync_cashmall_projection_0104(&mut projection, &shared, 12_345);

    assert_eq!(projection.shared_item_mode.owner_pc_id, 77);
    assert_eq!(projection.taros, 12_345);
    assert!(projection.player_inventory.is_empty());
    assert!(projection.rows().is_empty());
}

#[test]
fn user_equip_preview_uses_selected_style_and_only_authoritative_equipment() {
    let mut character = entry_test_character(9, 1, 0);
    character.equipment[ffone_protocol::CharacterEquipSlot0104::UpperBody as usize] =
        ffone_protocol::EquippedItem0104 {
            item_type: 1,
            item_id: 10,
            option: 11,
            time_limit: 12,
        };
    let stale_roster_equipment = character.equipment;
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_id: Some(77),
            ..default()
        },
        roster: RuntimeRosterStatus {
            characters: vec![character.clone()],
            selected_uid: Some(character.pc_uid),
            ..default()
        },
        ..default()
    };
    let authoritative = ItemBase0104 {
        item_type: 1,
        item_id: 99,
        option: 123,
        time_limit: 456,
    };
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    let offset = ffone_protocol::PcLoadData0104::EQUIPMENT_OFFSET
        + ffone_protocol::CharacterEquipSlot0104::UpperBody as usize
            * ffone_protocol::ItemBase0104::SIZE;
    load.as_bytes_mut()[offset..offset + 2].copy_from_slice(&authoritative.item_type.to_le_bytes());
    load.as_bytes_mut()[offset + 2..offset + 4]
        .copy_from_slice(&authoritative.item_id.to_le_bytes());
    load.as_bytes_mut()[offset + 4..offset + 8]
        .copy_from_slice(&authoritative.option.to_le_bytes());
    load.as_bytes_mut()[offset + 8..offset + 12]
        .copy_from_slice(&authoritative.time_limit.to_le_bytes());
    let inventory = InventoryRuntime0104::from_pc_load(77, &load);

    let resolved = authoritative_user_equip_preview_character(&runtime, &inventory).unwrap();
    assert_eq!(resolved.pc_uid, character.pc_uid);
    assert_eq!(resolved.style, character.style);
    assert_eq!(
        resolved.equipment[ffone_protocol::CharacterEquipSlot0104::UpperBody as usize],
        equipped_item_from_inventory_item(authoritative)
    );
    assert_eq!(
        runtime.roster.characters[0].equipment, stale_roster_equipment,
        "preview projection must not optimistically mutate the roster"
    );

    runtime.player_id = Some(88);
    assert!(authoritative_user_equip_preview_character(&runtime, &inventory).is_err());
    runtime.player_id = Some(77);
    runtime.roster.selected_uid = Some(404);
    assert!(authoritative_user_equip_preview_character(&runtime, &inventory).is_err());
}

#[test]
fn positive_tail_item_use_success_updates_authoritative_inventory_projection() {
    let content = runtime_test_mission_content();
    let resurrection_item = ffone_protocol::ItemBase0104 {
        item_type: 7,
        item_id: 134,
        option: 0,
        time_limit: 0,
    };
    let remaining_item = ffone_protocol::ItemBase0104 {
        item_type: 0,
        item_id: 0,
        option: 77,
        time_limit: 88,
    };
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    let slot = 49_usize;
    let offset = ffone_protocol::PcLoadData0104::INVENTORY_OFFSET
        + slot * ffone_protocol::ItemBase0104::SIZE;
    load.as_bytes_mut()[offset..offset + 2]
        .copy_from_slice(&resurrection_item.item_type.to_le_bytes());
    load.as_bytes_mut()[offset + 2..offset + 4]
        .copy_from_slice(&resurrection_item.item_id.to_le_bytes());
    let mut inventory = LocalInventoryRuntime::default();
    inventory.seed(77, &load);
    let mut projection = UserEquipItemModeProjection::default();
    refresh_user_equip_projection(&mut projection, inventory.snapshot(), &content);
    assert_eq!(projection.inventory[slot].item.item, resurrection_item);
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            resurrection_item_slot: Some(slot as i32),
            ..default()
        },
        ..default()
    };
    let mut payload = ffone_protocol::ItemUseSuccessPrefix0104 {
        pc_id: 77,
        item_location: 1,
        slot_num: slot as i32,
        remaining_item,
        skill_id: 12,
        pack_padding: [0; 2],
        skill_type: 10,
        target_count: 1,
    }
    .encode_prefix();
    // eST 10 uses one exact 16-byte sSkillResult_Buff record.
    payload.extend_from_slice(&[0; ffone_protocol::SkillResultBuff0104::SIZE]);
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_ITEM_USE_SUCC,
        flags: 0,
        checksum: 0,
        payload,
    };

    assert_eq!(
        apply_inventory_frame(&frame, &mut inventory, &content, &mut runtime),
        Ok(true)
    );
    assert_eq!(
        inventory.snapshot().unwrap().inventory()[slot],
        remaining_item
    );
    assert_eq!(runtime.resurrection_item_slot, None);
    assert!(refresh_user_equip_projection(
        &mut projection,
        inventory.snapshot(),
        &content,
    ));
    assert_eq!(projection.inventory[slot].item.item, remaining_item);
}

#[test]
fn quick_slot_projection_resolves_clean_general_icon_and_inventory_emptiness() {
    let content = runtime_test_mission_content();
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    let offset =
        ffone_protocol::PcLoadData0104::INVENTORY_OFFSET + 12 * ffone_protocol::ItemBase0104::SIZE;
    load.as_bytes_mut()[offset..offset + 2].copy_from_slice(&7_i16.to_le_bytes());
    load.as_bytes_mut()[offset + 2..offset + 4].copy_from_slice(&134_i16.to_le_bytes());
    let inventory = InventoryRuntime0104::from_pc_load(77, &load);
    let mut model = QuickSlotUiModel::default();
    model.slots[0] = LegacyQuickSlotEntry {
        item_type: 7,
        item_id: 134,
        ..default()
    };
    model.slots[1] = LegacyQuickSlotEntry {
        item_type: 7,
        item_id: 167,
        ..default()
    };

    project_quick_slots_from_inventory(&mut model, Some(&inventory), &content);

    assert!(
        model.slots[0]
            .icon_path
            .as_deref()
            .is_some_and(|path| path.starts_with("icons/items/general/generalitemicon_"))
    );
    assert!(!model.slots[0].inventory_empty);
    assert!(model.slots[1].inventory_empty);
}

#[test]
fn user_equip_item_mode_unlocks_cursor_and_blocks_world_gameplay_input() {
    let mut user_equip = UserEquipUiState::default();
    user_equip.open_item_mode();
    assert!(!legacy_gameplay_cursor_locked(
        ClientState::World,
        true,
        user_equip.is_active(),
    ));

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
        .init_state::<ClientState>()
        .init_resource::<TutorialSession>()
        .init_resource::<TutorialNativeMechanics>()
        .init_resource::<TutorialChoreographyPresentation>()
        .init_resource::<MissionUiModel>()
        .init_resource::<LegacyInputGate>()
        .insert_resource(user_equip)
        .add_systems(Update, sync_tutorial_input_gate);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::World);
    app.update();

    assert_eq!(
        *app.world().resource::<LegacyInputGate>(),
        LegacyInputGate {
            allow_forward: false,
            allow_backward: false,
            allow_strafe: false,
            allow_keyboard_turning: false,
            allow_jump: false,
            allow_mouse_camera: false,
        }
    );
}

#[test]
fn malformed_or_invalid_quick_slot_frames_preserve_last_good_state() {
    let mut model = QuickSlotUiModel::default();
    model.ready_for_play = true;
    model.slots[3].item_id = 303;
    let before_ids = model
        .slots
        .iter()
        .map(|slot| slot.item_id)
        .collect::<Vec<_>>();

    let malformed = DecodedFrame {
        packet_type: packet::P_FE2CL_PC_QUICK_SLOT_INFO,
        flags: 0,
        checksum: 0,
        payload: vec![0; ffone_protocol::QuickSlotInfo0104::SIZE - 1],
    };
    assert!(apply_quick_slot_frame(&malformed, &mut model).is_err());
    assert_eq!(
        model
            .slots
            .iter()
            .map(|slot| slot.item_id)
            .collect::<Vec<_>>(),
        before_ids
    );

    let invalid_slot = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_REGIST_QUICK_SLOT_SUCC,
        flags: 0,
        checksum: 0,
        payload: ffone_protocol::QuickSlotRegisterSuccess0104 {
            slot_num: 8,
            item_type: 7,
            item_id: 808,
        }
        .encode(),
    };
    assert!(apply_quick_slot_frame(&invalid_slot, &mut model).is_err());
    assert_eq!(
        model
            .slots
            .iter()
            .map(|slot| slot.item_id)
            .collect::<Vec<_>>(),
        before_ids
    );
}

#[test]
fn email_registered_send_reply_commits_inventory_and_taros_transactionally() {
    use ffone_client::email_ui::{
        EMAIL_REP_SEND_SUCCESS_ID, EMAIL_REP_SEND_SUCCESS_SIZE, EMAIL_REQ_SEND_ID, EmailBuddy,
        EmailRequest, EmailScreen, attach_email_inventory_item, begin_email_compose,
        send_composed_email,
    };
    const EMAIL_OUTGOING_ITEM_BYTES_0104: usize = 16;

    let catalog = runtime_test_email_catalog_0104();
    let item = ItemBase0104 {
        item_type: 0,
        item_id: 1,
        option: 1,
        time_limit: 77,
    };
    let inventory = enchant_test_inventory_0104(&[(4, item)]);
    let authority = email_inventory_authority_0104(&inventory);
    let mut production = EmailProductionRuntime0104::default();
    let mut model = EmailUiModel::default();
    let mut actions = EmailUiOutbox::default();
    let mut audio = EmailUiAudioOutbox::default();
    let mut transport = EmailTransportOutbox::default();
    let mut network = EmailNetworkRuntime0104::default();
    let inbox = EmailNetworkInbox0104::default();
    production
        .open(
            EmailOpenContext0104 {
                player: EmailPlayerAuthority0104 {
                    owner_pc_id: 77,
                    taros: 1_000,
                    current_local_time: EmailSystemTime::default(),
                },
                cursor_was_locked: false,
                item_policy: EmailItemFeaturePolicy0104 {
                    combine_enabled: true,
                    korean_enchant_enabled: false,
                },
            },
            Vec::new(),
            vec![EmailBuddy {
                pc_uid: 55,
                first_name: "Remote".to_owned(),
                last_name: "Buddy".to_owned(),
                name_check_flag: 1,
            }],
            &authority,
            &catalog,
            &mut model,
            &mut actions,
            &mut audio,
            &network,
            &inbox,
            &transport,
        )
        .unwrap();
    assert!(begin_email_compose(None, &mut model, &mut audio));
    model.screen = EmailScreen::Compose;
    model.opening_elapsed_seconds = 99.0;
    model.draft.recipient_pc_uid = 55;
    model.draft.recipient_name = "RemoteBuddy".to_owned();
    model.draft.subject = "Subject".to_owned();
    model.draft.content = "Body".to_owned();
    assert!(attach_email_inventory_item(4, 0, &mut model));
    assert_eq!(
        send_composed_email(&mut model, &mut actions, &mut transport, &mut audio),
        Ok(true)
    );
    let request = transport.0.front().cloned().unwrap();
    let outbound = production
        .dispatch_next_request(&model, &mut transport, &mut network, &catalog)
        .unwrap()
        .request
        .unwrap();
    assert_eq!(outbound.packet_type(), EMAIL_REQ_SEND_ID);

    let EmailRequest::Send {
        recipient_pc_uid,
        items,
        ..
    } = request
    else {
        panic!("compose emitted a non-send request")
    };
    let mut payload = vec![0; EMAIL_REP_SEND_SUCCESS_SIZE];
    payload[0..8].copy_from_slice(&recipient_pc_uid.to_le_bytes());
    payload[8..12].copy_from_slice(&700_i32.to_le_bytes());
    for (index, outgoing) in items.iter().enumerate() {
        let offset = 12 + index * EMAIL_OUTGOING_ITEM_BYTES_0104;
        payload[offset..offset + 4].copy_from_slice(&outgoing.inventory_slot.to_le_bytes());
        payload[offset + 4..offset + 6].copy_from_slice(&outgoing.item.item_type.to_le_bytes());
        payload[offset + 6..offset + 8].copy_from_slice(&outgoing.item.item_id.to_le_bytes());
        payload[offset + 8..offset + 12].copy_from_slice(&outgoing.item.option.to_le_bytes());
        payload[offset + 12..offset + 16].copy_from_slice(&outgoing.item.time_limit.to_le_bytes());
    }
    let disposition = production.route_frame(
        DecodedFrame {
            packet_type: EMAIL_REP_SEND_SUCCESS_ID,
            flags: 0,
            checksum: 0,
            payload,
        },
        &mut model,
        &mut actions,
        &mut transport,
        &mut audio,
        &mut network,
    );
    let EmailFrameDisposition0104::Applied { output, .. } = disposition else {
        panic!("strictly correlated send success was not accepted")
    };
    assert_eq!(
        output.actions,
        vec![
            EmailUiAction::RefreshInventory {
                event_group: 11,
                event_function: 6,
            },
            EmailUiAction::RefreshGuideEmail {
                event_group: 15,
                event_function: 7,
            },
        ],
        "the production shell must preserve the clean post-send callbacks"
    );
    let commit = output.commit.expect("send success authority commit");
    assert_eq!(commit.taros_after(), Some(700));
    assert!(commit.inventory_refresh_required());
    let next = email_inventory_after_commit_0104(&inventory, &commit).unwrap();
    assert_eq!(
        inventory.inventory()[4],
        item,
        "input snapshot is immutable"
    );
    assert_eq!(next.inventory()[4], ItemBase0104 { item_id: 0, ..item });

    let other_owner =
        InventoryRuntime0104::from_pc_load(88, &ffone_protocol::PcLoadData0104::zeroed());
    assert!(email_inventory_after_commit_0104(&other_owner, &commit).is_err());
    assert!(InventoryRuntime0104::item_is_empty(
        other_owner.inventory()[4]
    ));
}

#[test]
fn email_mode_uses_clean_lease_and_live_configurable_hotkey() {
    let lease = EmailModeLease0104::from(EmailOpenContext0104 {
        player: EmailPlayerAuthority0104 {
            owner_pc_id: 77,
            taros: 1_000,
            current_local_time: EmailSystemTime::default(),
        },
        cursor_was_locked: true,
        item_policy: EmailItemFeaturePolicy0104 {
            combine_enabled: true,
            korean_enchant_enabled: false,
        },
    });
    assert_eq!(lease.game_mode, 18);
    assert_eq!(lease.active_inventory_tab, 0);
    assert_eq!(lease.inventory_mail_mode, 4);
    assert!(lease.blocks_gameplay_input);
    assert!(!lease.cursor_locked_during_mode);
    assert!(lease.cursor_locked_after_exit);
    assert!(!lease.normal_exit_sends_packet);

    let mut input = InputSettings::default();
    let email = input
        .mappings
        .iter_mut()
        .find(|row| row.action == LegacyOptionAction::Email)
        .unwrap();
    email.primary = LegacyInputBinding::Key(LegacyPhysicalKey::Mouse1);
    email.alternate = LegacyInputBinding::Unbound;
    let keyboard = ButtonInput::<KeyCode>::default();
    let mut mouse = ButtonInput::<MouseButton>::default();
    mouse.press(MouseButton::Right);
    assert!(option_action_just_pressed(
        &input,
        LegacyOptionAction::Email,
        &keyboard,
        &mouse,
    ));
    clear_option_action_press(
        &input,
        LegacyOptionAction::Email,
        &mut ButtonInput::<KeyCode>::default(),
        &mut mouse,
    );
    assert!(!mouse.just_pressed(MouseButton::Right));
}

#[test]
fn enchant_delete_commit_updates_global_inventory_only_after_authoritative_reply() {
    let item = ItemBase0104 {
        item_type: 7,
        item_id: 134,
        option: 3,
        time_limit: 91,
    };
    let (mut runtime, inventory) = pending_enchant_delete_test_runtime_0104(item);
    assert_eq!(inventory.inventory()[4], item);

    let output = runtime
        .apply_reply(EnchantReplyPacket0104::DeleteSuccess(
            ffone_protocol::PcItemDeleteSuccess0104 {
                item_location: 1,
                slot_num: 4,
            },
        ))
        .unwrap();
    let commit = output.commit.unwrap();
    assert_eq!(commit.operation(), EnchantOperation0104::Delete);
    let next = enchant_inventory_after_commit_0104(&inventory, &commit).unwrap();
    assert_eq!(inventory.inventory()[4], item, "input stays immutable");
    assert_eq!(
        next.inventory()[4],
        ItemBase0104 {
            item_type: 0,
            item_id: 0,
            option: 0,
            time_limit: 91,
        }
    );
    assert_eq!(next.inventory(), commit.snapshot_after().inventory());
    assert_eq!(next.equipment(), commit.snapshot_after().equipment());
}

#[test]
fn bank_move_stages_global_inventory_and_bank_projection_from_one_commit() {
    let moved = ItemBase0104 {
        item_type: 7,
        item_id: 134,
        option: 3,
        time_limit: 4,
    };
    let empty = ItemBase0104 {
        item_type: 0,
        item_id: 0,
        option: 0,
        time_limit: 0,
    };
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    let offset =
        ffone_protocol::PcLoadData0104::INVENTORY_OFFSET + 4 * ffone_protocol::ItemBase0104::SIZE;
    load.as_bytes_mut()[offset..offset + 2].copy_from_slice(&moved.item_type.to_le_bytes());
    load.as_bytes_mut()[offset + 2..offset + 4].copy_from_slice(&moved.item_id.to_le_bytes());
    load.as_bytes_mut()[offset + 4..offset + 8].copy_from_slice(&moved.option.to_le_bytes());
    load.as_bytes_mut()[offset + 8..offset + 12].copy_from_slice(&moved.time_limit.to_le_bytes());
    let inventory = InventoryRuntime0104::from_pc_load(77, &load);
    let request = ffone_protocol::PcBankOpenRequest0104 {
        pc_id: 77,
        npc_id: 9_001,
    };
    let mut production = BankProductionRuntime0104::default();
    production.begin_open(request, &inventory).unwrap();
    production
        .apply_bank_reply(
            ffone_protocol::PcBankReply0104::OpenSuccess(ffone_protocol::PcBankOpenSuccess0104 {
                bank_items: [empty; ffone_protocol::BANK_SLOT_COUNT_0104],
                extra_bank: 1,
            }),
            &inventory,
        )
        .unwrap();
    production
        .begin_item_move(
            ffone_protocol::ItemMoveRequest0104 {
                from_location: 1,
                from_slot_num: 4,
                to_location: 3,
                to_slot_num: 9,
            },
            &inventory,
        )
        .unwrap();
    let event = production
        .apply_item_move_success(ItemMoveSuccessPacket0104 {
            from_location: 1,
            from_slot_num: 4,
            from_slot_item: empty,
            to_location: 3,
            to_slot_num: 9,
            to_slot_item: moved,
        })
        .unwrap();
    let BankProductionEvent0104::ItemMoveCommitted(commit) = event else {
        panic!("expected authoritative BankMode move commit");
    };
    let next = bank_inventory_after_commit(&inventory, &commit).unwrap();
    assert_eq!(inventory.inventory()[4], moved, "input stays unmodified");
    assert_eq!(next.inventory()[4], empty);
    assert_eq!(commit.snapshot_after().bank()[9], moved);
    assert_eq!(commit.snapshot_after().inventory(), next.inventory());
}

#[test]
fn vendor_subtarget_hides_local_player_until_mode_closes() {
    let mut vendor = VendorUiState::default();
    vendor.phase = VendorLifecyclePhase::Visible;
    let mut production = VendorProductionRuntime0104::default();
    production
        .begin_start(ActiveVendorSourceNpc0104 {
            runtime_npc_id: 9_001,
            table_npc_id: 650,
            ai_type: 1,
        })
        .unwrap();

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<TutorialChoreographyPresentation>()
        .init_resource::<MissionUiModel>()
        .insert_resource(vendor)
        .insert_resource(production)
        .add_systems(PostUpdate, sync_tutorial_choreography_visibility);
    let player_scene = app
        .world_mut()
        .spawn((LocalCharacterScene, Visibility::Inherited))
        .id();
    let hud = app
        .world_mut()
        .spawn((GameplayHud, Visibility::Inherited))
        .id();

    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(player_scene).unwrap(),
        Visibility::Hidden
    );
    assert_eq!(
        *app.world().get::<Visibility>(hud).unwrap(),
        Visibility::Hidden
    );

    app.world_mut().resource_mut::<VendorUiState>().phase = VendorLifecyclePhase::Hidden;
    app.world_mut()
        .resource_mut::<VendorProductionRuntime0104>()
        .close_mode();
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(player_scene).unwrap(),
        Visibility::Inherited
    );
    assert_eq!(
        *app.world().get::<Visibility>(hud).unwrap(),
        Visibility::Inherited
    );
}

#[test]
fn email_all_production_inventory_item_icons_are_installed() {
    let catalog = runtime_test_email_catalog_0104();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let mut checked = 0;
    for (&(item_type, item_id), metadata) in &catalog.items {
        if item_id == 0 { continue; }
        let path = metadata.icon_path.as_ref().unwrap_or_else(||
            panic!("missing icon for item {item_type}:{item_id}"));
        assert!(root.join(path).is_file(), "{path}");
        checked += 1;
    }
    assert!(checked >= 6427, "must cover every production inventory item, checked {checked}");
}
