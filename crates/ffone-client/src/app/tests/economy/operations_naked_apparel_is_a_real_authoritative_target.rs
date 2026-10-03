use super::*;

#[test]
fn cashmall_production_open_and_close_retain_exact_local_boundaries() {
    let mut state = CashmallUiState0104::default();
    let mut outbox = CashmallUiOutbox0104::default();
    state.open_from(CashmallOpenSource0104::HiddenChatCommand, true, &mut outbox);
    assert_eq!(state.phase(), CashmallLifecyclePhase0104::Opening);
    assert_eq!(
        outbox.pop_front(),
        Some(CashmallLocalEffect0104::EnterMode {
            ui_input_event: [11, 0],
            ui_input_enter_value: 1,
            force_inventory_tab: 0,
            force_cursor_unlocked: true,
        })
    );

    state
        .request_close(
            CashmallCloseSource0104::ConfigurableKey4,
            CashmallModalState0104::default(),
            CashmallCloseGate0104 {
                mode_accepts_escape: true,
                exit_arbitration_clear: true,
            },
            &mut outbox,
        )
        .unwrap();
    assert_eq!(state.phase(), CashmallLifecyclePhase0104::Hidden);
    let Some(CashmallLocalEffect0104::Close(boundary)) = outbox.pop_front() else {
        panic!("Cashmall close must emit its exact local boundary");
    };
    assert_eq!(boundary.ui_input_event, [11, 0]);
    assert_eq!(boundary.ui_input_exit_value, 10);
    assert!(boundary.restore_cursor_locked);
    assert_eq!(boundary.notify_game_mode_exit, [2, 1]);
    assert!(boundary.stop_ui_mode_sound);
    assert!(boundary.request_asset_gc);
    assert!(!boundary.loaded_textures_actually_cleared);
}

#[test]
fn naked_apparel_is_a_real_authoritative_target_and_new_root_reattaches_hand() {
    let equipped = ItemBase0104 {
        item_type: 0,
        item_id: 43,
        option: 0,
        time_limit: 0,
    };
    let active = [
        ItemBase0104 {
            item_type: 1,
            item_id: 100,
            option: 0,
            time_limit: 0,
        },
        ItemBase0104 {
            item_type: 2,
            item_id: 200,
            option: 0,
            time_limit: 0,
        },
        ItemBase0104 {
            item_type: 3,
            item_id: 300,
            option: 0,
            time_limit: 0,
        },
        ItemBase0104 {
            item_type: 4,
            item_id: 400,
            option: 0,
            time_limit: 0,
        },
        ItemBase0104 {
            item_type: 5,
            item_id: 500,
            option: 0,
            time_limit: 0,
        },
        ItemBase0104 {
            item_type: 6,
            item_id: 600,
            option: 0,
            time_limit: 0,
        },
    ];
    let naked = [
        ItemBase0104 {
            item_type: 1,
            item_id: 0,
            option: 0,
            time_limit: 0,
        },
        ItemBase0104 {
            item_type: 2,
            item_id: 0,
            option: 0,
            time_limit: 0,
        },
        ItemBase0104 {
            item_type: 3,
            item_id: 0,
            option: 0,
            time_limit: 0,
        },
        ItemBase0104 {
            item_type: 4,
            item_id: 0,
            option: 0,
            time_limit: 0,
        },
        ItemBase0104 {
            item_type: 5,
            item_id: 0,
            option: 0,
            time_limit: 0,
        },
        ItemBase0104 {
            item_type: 6,
            item_id: 0,
            option: 0,
            time_limit: 0,
        },
    ];
    assert_eq!(
        world_player_apparel_refresh_plan_0104(Some(active), None, None, naked),
        WorldPlayerApparelRefreshPlan::Spawn
    );

    let old_root = Entity::from_bits(1);
    let replacement_root = Entity::from_bits(2);
    assert!(!world_player_hand_projection_required_0104(
        Some(equipped),
        Some(old_root),
        old_root,
        equipped,
    ));
    assert!(world_player_hand_projection_required_0104(
        Some(equipped),
        Some(old_root),
        replacement_root,
        equipped,
    ));
}

#[test]
fn weapon_cycle_swaps_equipped_hands_and_never_searches_the_bag() {
    let empty = ItemBase0104 { item_type: 0, item_id: 0, option: 0, time_limit: 0 };
    let weapon = |item_id| ItemBase0104 { item_id, ..empty };
    let mut slots = [empty; 9];
    assert!(world_weapon_swap_request(&slots, |_| true).is_none());
    slots[0] = weapon(43);
    let request = world_weapon_swap_request(&slots, |_| true).unwrap();
    assert_eq!((request.from_location, request.from_slot_num, request.to_location, request.to_slot_num), (0, 0, 0, 7));
    slots.swap(0, 7); // authoritative post-state after holstering
    assert_eq!(world_weapon_swap_request(&slots, |_| true).unwrap().from_slot_num, 7);
    slots.swap(0, 7); // draw returns to the original one-weapon state
    slots[7] = weapon(197);
    let request = world_weapon_swap_request(&slots, |id| id == 197).unwrap();
    assert_eq!((request.from_location, request.from_slot_num, request.to_location, request.to_slot_num), (0, 7, 0, 0));
    slots[0] = empty;
    assert_eq!(world_weapon_swap_request(&slots, |_| true), Some(request));
    assert!(world_weapon_swap_request(&slots, |_| false).is_none());
    slots.swap(0, 7); // with two weapons, the next accepted change selects the previous Hand
    assert_eq!(world_weapon_swap_request(&slots, |_| true).unwrap().from_slot_num, 7);
    slots.swap(0, 7);
    slots[7] = empty;
    slots[1] = weapon(43); // unrelated equipment cannot become a weapon source
    assert!(world_weapon_swap_request(&slots, |_| true).is_none());
}

#[test]
fn local_resurrect_decoder_is_strict_and_never_claims_remote_ownership() {
    let local_dead = ffone_protocol::PcSuddenDead0104 {
        pc_id: 77,
        sudden_dead_reason: 3,
        damage: 1_500,
        hp: 0,
    };
    let local_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_PC_SUDDEN_DEAD,
        flags: 0,
        checksum: 0,
        payload: local_dead.encode(),
    };
    assert_eq!(
        decode_local_resurrect_packet_0104(&local_frame, Some(77)),
        Ok(Some(LocalResurrectPacket0104::SuddenDead { hp: 0 }))
    );
    assert_eq!(
        decode_local_resurrect_packet_0104(&local_frame, Some(88)),
        Ok(None),
        "another PC's death must remain in entity lifecycle"
    );

    let remote_regen = ffone_protocol::PcRegen0104 {
        pc_id: 77,
        hp: 1_000,
        position: [100, 200, 300],
        angle: 90,
        condition_bit_flag: 0,
        pc_state: 0,
        special_state: 0,
        nano: ffone_protocol::Nano0104 {
            id: 4,
            skill_id: 5,
            stamina: 6,
        },
    };
    assert_eq!(
        decode_local_resurrect_packet_0104(
            &DecodedFrame {
                packet_type: packet::P_FE2CL_PC_REGEN,
                flags: 0,
                checksum: 0,
                payload: remote_regen.encode(),
            },
            Some(77),
        ),
        Ok(None),
        "the remote broadcast ID never drives the local modal"
    );

    let malformed = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_REGEN_SUCC,
        flags: 0,
        checksum: 0,
        payload: vec![0; PcRegenSuccess0104::SIZE - 1],
    };
    assert!(
        decode_local_resurrect_packet_0104(&malformed, Some(77)).is_err(),
        "recognized regeneration packets require their exact ABI size"
    );
}

#[test]
fn late_user_equip_delete_success_is_consumed_before_vendor_and_applies_authority() {
    let content = runtime_test_mission_content();
    let item = ItemBase0104 {
        item_type: 7,
        item_id: 134,
        option: 3,
        time_limit: 91,
    };
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    let slot_index = 4_usize;
    let offset = ffone_protocol::PcLoadData0104::INVENTORY_OFFSET + slot_index * ItemBase0104::SIZE;
    load.as_bytes_mut()[offset..offset + 2].copy_from_slice(&item.item_type.to_le_bytes());
    load.as_bytes_mut()[offset + 2..offset + 4].copy_from_slice(&item.item_id.to_le_bytes());
    load.as_bytes_mut()[offset + 4..offset + 8].copy_from_slice(&item.option.to_le_bytes());
    load.as_bytes_mut()[offset + 8..offset + 12].copy_from_slice(&item.time_limit.to_le_bytes());
    let mut inventory = LocalInventoryRuntime::default();
    inventory.seed(77, &load);
    let request = PcItemDeleteRequest0104 {
        item_location: 1,
        slot_num: slot_index as i32,
    };
    let mut production = UserEquipProductionRuntime0104::default();
    production
        .begin(UserEquipPendingRequest0104::Delete(request))
        .unwrap();
    assert_eq!(
        production.tick(ffone_client::user_equip_runtime::USER_EQUIP_REQUEST_TIMEOUT_SECONDS_0104),
        Some(UserEquipPendingRequest0104::Delete(request))
    );
    assert!(!production.send_pending(), "timeout must unlock controls");
    assert!(production.owns_delete_reply_family());
    let reply = PcItemDeleteSuccess0104 {
        item_location: 1,
        slot_num: slot_index as i32,
    };
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_ITEM_DELETE_SUCC,
        flags: 0,
        checksum: 0,
        payload: reply.encode(),
    };
    let mut runtime = RuntimeStatus::default();

    assert_eq!(
        apply_user_equip_frame(
            &frame,
            &mut inventory,
            &content,
            &mut production,
            &BankProductionRuntime0104::default(),
            &VendorProductionRuntime0104::default(),
            &mut runtime,
        ),
        Ok(true),
        "true is the routing barrier that prevents Vendor from seeing the reply"
    );
    assert!(!production.owns_delete_reply_family());
    assert_eq!(
        inventory.snapshot().unwrap().inventory()[slot_index],
        ItemBase0104 {
            item_type: 0,
            item_id: 0,
            option: 0,
            time_limit: item.time_limit,
        }
    );
}

#[test]
fn late_user_equip_move_is_applied_once_and_cannot_fall_through_to_bank() {
    let content = runtime_test_mission_content();
    let moved = ItemBase0104 {
        item_type: 7,
        item_id: 134,
        option: 3,
        time_limit: 91,
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
    let mut inventory = LocalInventoryRuntime::default();
    inventory.seed(77, &load);
    let request = ItemMoveRequest0104 {
        from_location: 1,
        from_slot_num: 4,
        to_location: 1,
        to_slot_num: 5,
    };
    let reply = ItemMoveSuccessPacket0104 {
        from_location: 1,
        from_slot_num: 5,
        from_slot_item: moved,
        to_location: 1,
        to_slot_num: 4,
        to_slot_item: empty,
    };
    let mut production = UserEquipProductionRuntime0104::default();
    production
        .begin(UserEquipPendingRequest0104::Move(request))
        .unwrap();
    production.close_session_preserving_late_replies();
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_PC_ITEM_MOVE_SUCC,
        flags: 0,
        checksum: 0,
        payload: reply.encode(),
    };
    let mut runtime = RuntimeStatus::default();

    assert_eq!(
        apply_user_equip_frame(
            &frame,
            &mut inventory,
            &content,
            &mut production,
            &BankProductionRuntime0104::default(),
            &VendorProductionRuntime0104::default(),
            &mut runtime,
        ),
        Ok(true),
        "the routing barrier must keep a late UserEquip move out of BankMode"
    );
    assert!(!production.owns_move_reply_family());
    assert_eq!(inventory.snapshot().unwrap().inventory()[4], empty);
    assert_eq!(inventory.snapshot().unwrap().inventory()[5], moved);
}

#[test]
fn user_equip_delete_confirmation_is_keyed_correlated_and_resettable() {
    let mut owner = UserEquipSystemMessageRuntime0104::default();
    let mut messages = SystemMessageUiModel::default();
    let request = PcItemDeleteRequest0104 {
        item_location: 1,
        slot_num: 8,
    };
    let item = ItemBase0104 {
        item_type: 7,
        item_id: 134,
        option: 12,
        time_limit: 0,
    };
    let request_id = owner.queue_delete(
        77,
        request,
        item,
        Some("items/general/example.png".to_owned()),
        &mut messages,
    );

    let prompt = messages.current().unwrap();
    assert_eq!(request_id, USER_EQUIP_SYSTEM_MESSAGE_ID_BASE_0104);
    assert_eq!(prompt.localized.key, "ui.inventory.popup.confirm_delete");
    assert_eq!(prompt.button_type, SystemMessageButtonType::DeleteItem);
    assert_eq!(prompt.icon_quantity, 12);
    assert!(owner.owns(request_id));

    owner.reset(&mut messages);
    assert!(!owner.owns(request_id));
    assert!(messages.is_empty());
}

#[test]
fn local_equip_change_reprojects_authoritative_visual_slot_without_speculation() {
    let content = runtime_test_mission_content();
    let mut inventory = LocalInventoryRuntime::default();
    inventory.seed(77, &ffone_protocol::PcLoadData0104::zeroed());
    let equipped = ffone_protocol::ItemBase0104 {
        item_type: 4,
        item_id: 1,
        option: 0,
        time_limit: 0,
    };
    let packet = ffone_protocol::EquipChangePacket0104 {
        pc_id: 77,
        equip_slot_num: 4,
        equip_slot_item: equipped,
    };
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_PC_EQUIP_CHANGE,
        flags: 0,
        checksum: 0,
        payload: packet.encode(),
    };
    let mut runtime = RuntimeStatus::default();
    let mut projection = UserEquipItemModeProjection::default();

    assert_eq!(
        apply_inventory_frame(&frame, &mut inventory, &content, &mut runtime),
        Ok(true)
    );
    assert!(
        projection.equipment[0].item.empty,
        "UI projection cannot mutate before an authoritative refresh"
    );
    assert!(refresh_user_equip_projection(
        &mut projection,
        inventory.snapshot(),
        &content,
    ));
    assert_eq!(
        projection.equipment[0].item.item, equipped,
        "visual Head slot must project authoritative wire equipment slot 4"
    );
}

#[test]
fn nano_cooldown_projects_to_the_equipped_slot_and_clears_stale_lifecycles() {
    let runtime_slots = [
        RuntimeNanoSlot::default(),
        RuntimeNanoSlot {
            nano_id: Some(1),
            skill_id: 1,
            stamina: 80,
            // Slot-owned cooldown remains visible after dismiss.
            active: false,
        },
        RuntimeNanoSlot::default(),
    ];
    let mut nano_wheel = NanoWheelTransientUi::default();
    nano_wheel.slots[0].gumball_enabled = true;
    for slot in &mut nano_wheel.slots {
        slot.active_skill_cooldown_remaining = Some(0.25);
    }

    project_nano_skill_cooldown(
        ClientState::Tutorial,
        &runtime_slots,
        Some(TutorialNanoGameplayLoadout {
            nano_id: 1,
            skill_id: 1,
        }),
        Some(0.75),
        [None; 3],
        &mut nano_wheel,
    );

    assert_eq!(
        nano_wheel
            .slots
            .map(|slot| slot.active_skill_cooldown_remaining),
        [None, Some(0.75), None]
    );
    assert!(nano_wheel.slots[0].gumball_enabled);

    project_nano_skill_cooldown(
        ClientState::World,
        &runtime_slots,
        Some(TutorialNanoGameplayLoadout {
            nano_id: 1,
            skill_id: 1,
        }),
        Some(0.25),
        [None, Some(0.5), None],
        &mut nano_wheel,
    );
    assert_eq!(
        nano_wheel
            .slots
            .map(|slot| slot.active_skill_cooldown_remaining),
        [None, Some(0.5), None],
        "the same slot-owned cooldown drives the ordinary-world HUD"
    );

    nano_wheel.slots[2].active_skill_cooldown_remaining = Some(1.0);
    project_nano_skill_cooldown(
        ClientState::CharacterSelect,
        &runtime_slots,
        Some(TutorialNanoGameplayLoadout {
            nano_id: 1,
            skill_id: 1,
        }),
        Some(0.25),
        [Some(1.0); 3],
        &mut nano_wheel,
    );
    assert_eq!(
        nano_wheel
            .slots
            .map(|slot| slot.active_skill_cooldown_remaining),
        [None; 3],
        "cooldown state must not leak into a non-gameplay lifecycle"
    );

    nano_wheel.slots[2].active_skill_cooldown_remaining = Some(1.0);
    project_nano_skill_cooldown(
        ClientState::Tutorial,
        &runtime_slots,
        Some(TutorialNanoGameplayLoadout {
            nano_id: 2,
            skill_id: 7,
        }),
        Some(f32::NAN),
        [None; 3],
        &mut nano_wheel,
    );
    assert_eq!(
        nano_wheel
            .slots
            .map(|slot| slot.active_skill_cooldown_remaining),
        [None; 3],
        "mismatched or malformed cooldown state fails closed"
    );
}

#[test]
fn skill_buff_target_visibility_matches_the_combat_target_panel() {
    assert!(skill_buff_target_visible(
        LegacyTargetKind::Npc { team: 2 },
        false
    ));
    assert!(!skill_buff_target_visible(
        LegacyTargetKind::Npc { team: 1 },
        false
    ));
    assert!(skill_buff_target_visible(
        LegacyTargetKind::Npc { team: 1 },
        true
    ));
    assert!(
        skill_buff_target_visible(LegacyTargetKind::Player, false),
        "the helper remains status-owner generic even though the current feed selects NPCs"
    );
}

#[test]
fn vendor_system_messages_keep_clean_text_and_button_contracts() {
    let content = runtime_test_mission_content();
    let expected = [
        (
            VendorSystemMessageId0104::InventoryFull,
            "Inventory is full.",
            0,
            SystemMessageButtonType::None,
        ),
        (
            VendorSystemMessageId0104::StartFailed,
            "You cannot use this vendor currently.",
            10,
            SystemMessageButtonType::Ok,
        ),
        (
            VendorSystemMessageId0104::TableUpdateFailed,
            "Could not retrieve item information.",
            10,
            SystemMessageButtonType::Ok,
        ),
        (
            VendorSystemMessageId0104::ConfirmDelete,
            "DELETE THIS ITEM?",
            12,
            SystemMessageButtonType::DeleteItem,
        ),
        (
            VendorSystemMessageId0104::ConfirmDisassemble,
            "Alert! One or more of your vehicle rentals has expired.",
            0,
            SystemMessageButtonType::None,
        ),
    ];
    for (message_id, text, raw_button_type, button_type) in expected {
        let definition = content.system_message_definition(message_id.id()).unwrap();
        assert_eq!(definition.exact_text, text);
        assert_eq!(definition.raw_button_type, raw_button_type);
        assert_eq!(definition.runtime_button_type, button_type);
    }
}

#[test]
fn npc_icon_vendor_service_is_first_and_uses_exact_enter_store_label() {
    let content = runtime_test_mission_content();
    let services = world_npc_service_entries(&content, 650, 18, None, Some(0), false);
    assert_eq!(services[0].service, NpcServiceKind::Vendor);
    assert_eq!(services[0].label, "ENTER STORE");
    assert_eq!(services[1].service, NpcServiceKind::GuideChanger);
}

#[test]
fn npc_icon_discovers_only_exact_bank_service_types_twelve_and_fifty() {
    let content = runtime_test_mission_content();
    assert_eq!(bank_service_kind(12), Some(NpcServiceKind::Bank));
    assert_eq!(bank_service_kind(50), Some(NpcServiceKind::LocalBank));
    assert_eq!(bank_service_kind(11), None);
    assert_eq!(bank_service_kind(51), None);
    assert_eq!(
        world_npc_service_entries(&content, -1, 12, None, None, false),
        vec![NpcServiceUiEntry::original(NpcServiceKind::Bank)]
    );
    assert_eq!(
        world_npc_service_entries(&content, -1, 50, None, None, false),
        vec![NpcServiceUiEntry::original(NpcServiceKind::LocalBank)]
    );
}

#[test]
fn positive_new_email_count_rings_twice_and_arms_nanocom_notices() {
    let mut mission = MissionUiModel::default();
    let mut alarm = ffone_client::gameplay_ui::MinimapNewMailAlarm::default();
    assert!(!apply_new_email_notice_0104(
        0,
        &mut mission,
        Some(&mut alarm)
    ));
    assert!(!mission.nanocom_new_mail_notice_visible);
    assert!(!alarm.active());
    assert!(apply_new_email_notice_0104(
        2,
        &mut mission,
        Some(&mut alarm)
    ));
    assert!(mission.nanocom_new_mail_notice_visible);
    assert!(alarm.active());
    assert_eq!(
        EMAIL_NEW_MAIL_ARRIVAL_CUES_0104,
        [EmailUiAudioCue::EmailArrived, EmailUiAudioCue::EmailArrived]
    );
}

#[test]
fn email_clock_projection_is_exact_utc_without_host_timezone_inference() {
    let projected = email_system_time_utc_0104(
        UNIX_EPOCH + Duration::from_secs(1_709_210_096) + Duration::from_millis(789),
    );
    assert_eq!(
        projected,
        EmailSystemTime {
            year: 2024,
            month: 2,
            day_of_week: 4,
            day: 29,
            hour: 12,
            minute: 34,
            second: 56,
            milliseconds: 789,
        }
    );
}

#[test]
fn email_receive_item_feed_applies_only_exact_registered_item_bytes() {
    let content = runtime_test_mission_content();
    let mut inventory = LocalInventoryRuntime {
        snapshot: Some(enchant_test_inventory_0104(&[])),
        ..default()
    };
    let mut runtime = RuntimeStatus::default();
    let item = ItemBase0104 {
        item_type: 7,
        item_id: 12,
        option: 3,
        time_limit: 77,
    };
    let mut payload = Vec::with_capacity(PC_GIVE_ITEM_SUCCESS_BODY_SIZE_0104);
    payload.extend_from_slice(&InventoryLocation0104::Inventory.wire_value().to_le_bytes());
    payload.extend_from_slice(&4_i32.to_le_bytes());
    payload.extend_from_slice(&item.item_type.to_le_bytes());
    payload.extend_from_slice(&item.item_id.to_le_bytes());
    payload.extend_from_slice(&item.option.to_le_bytes());
    payload.extend_from_slice(&item.time_limit.to_le_bytes());
    let frame = DecodedFrame {
        packet_type: PC_GIVE_ITEM_SUCCESS_PACKET_ID_0104,
        flags: 0,
        checksum: 0,
        payload,
    };
    assert!(
        !EmailNetworkInbox0104::owns_packet(frame.packet_type),
        "generic give-item authority must remain outside Email reply correlation"
    );
    assert_eq!(
        apply_inventory_frame(&frame, &mut inventory, &content, &mut runtime),
        Ok(true)
    );
    assert_eq!(inventory.snapshot().unwrap().inventory()[4], item);

    let before = inventory.snapshot().unwrap().clone();
    let malformed = DecodedFrame {
        payload: frame.payload[..frame.payload.len() - 1].to_vec(),
        ..frame
    };
    let error =
        apply_inventory_frame(&malformed, &mut inventory, &content, &mut runtime).unwrap_err();
    assert!(error.contains("requires exactly 20 bytes"), "{error}");
    assert_eq!(inventory.snapshot(), Some(&before));
}
