use super::*;

#[test]
fn inventory_item_move_frame_updates_authoritative_projection_and_resurrection_slot() {
    let content = runtime_test_mission_content();
    let resurrection_item = ffone_protocol::ItemBase0104 {
        item_type: 7,
        item_id: 134,
        option: 19,
        time_limit: 23,
    };
    let empty = ffone_protocol::ItemBase0104 {
        item_type: 0,
        item_id: 0,
        option: 0,
        time_limit: 0,
    };
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    let source_offset =
        ffone_protocol::PcLoadData0104::INVENTORY_OFFSET + 2 * ffone_protocol::ItemBase0104::SIZE;
    load.as_bytes_mut()[source_offset..source_offset + 2]
        .copy_from_slice(&resurrection_item.item_type.to_le_bytes());
    load.as_bytes_mut()[source_offset + 2..source_offset + 4]
        .copy_from_slice(&resurrection_item.item_id.to_le_bytes());
    load.as_bytes_mut()[source_offset + 4..source_offset + 8]
        .copy_from_slice(&resurrection_item.option.to_le_bytes());
    load.as_bytes_mut()[source_offset + 8..source_offset + 12]
        .copy_from_slice(&resurrection_item.time_limit.to_le_bytes());
    let mut inventory = LocalInventoryRuntime::default();
    inventory.seed(77, &load);
    let mut projection = UserEquipItemModeProjection::default();
    assert!(refresh_user_equip_projection(
        &mut projection,
        inventory.snapshot(),
        &content,
    ));
    assert_eq!(projection.inventory[2].item.item, resurrection_item);
    let mut runtime = RuntimeStatus::default();
    runtime.resurrection_item_slot = Some(2);
    let packet = ffone_protocol::ItemMoveSuccessPacket0104 {
        from_location: 1,
        from_slot_num: 2,
        from_slot_item: empty,
        to_location: 1,
        to_slot_num: 49,
        to_slot_item: resurrection_item,
    };
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_PC_ITEM_MOVE_SUCC,
        flags: 0,
        checksum: 0,
        payload: packet.encode(),
    };

    assert_eq!(
        apply_inventory_frame(&frame, &mut inventory, &content, &mut runtime),
        Ok(true)
    );
    let snapshot = inventory.snapshot().unwrap();
    assert_eq!(snapshot.inventory()[2], empty);
    assert_eq!(snapshot.inventory()[49], resurrection_item);
    assert_eq!(runtime.resurrection_item_slot, Some(49));
    assert!(refresh_user_equip_projection(
        &mut projection,
        inventory.snapshot(),
        &content,
    ));
    assert!(projection.inventory[2].item.empty);
    assert_eq!(projection.inventory[49].item.item, resurrection_item);
}

#[test]
fn remote_equip_change_frame_is_left_for_entity_lifecycle() {
    let content = runtime_test_mission_content();
    let mut inventory = LocalInventoryRuntime::default();
    inventory.seed(77, &ffone_protocol::PcLoadData0104::zeroed());
    let before = *inventory.snapshot().unwrap().equipment();
    let packet = ffone_protocol::EquipChangePacket0104 {
        pc_id: 88,
        equip_slot_num: 0,
        equip_slot_item: ffone_protocol::ItemBase0104 {
            item_type: 0,
            item_id: 134,
            option: 0,
            time_limit: 0,
        },
    };
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_PC_EQUIP_CHANGE,
        flags: 0,
        checksum: 0,
        payload: packet.encode(),
    };
    let mut runtime = RuntimeStatus::default();

    assert_eq!(
        apply_inventory_frame(&frame, &mut inventory, &content, &mut runtime),
        Ok(false)
    );
    assert_eq!(*inventory.snapshot().unwrap().equipment(), before);
}

#[test]
fn email_guide_projection_uses_serialized_mentor_order_not_wire_id_order() {
    for (mentor, email_index, npc_type) in [
        (GuideMentor::Edd, 0, 707),
        (GuideMentor::Dexter, 1, 728),
        (GuideMentor::MojoJojo, 2, 731),
        (GuideMentor::BenTennyson, 3, 732),
    ] {
        assert_eq!(email_mentor_email_index_0104(mentor), email_index);
        assert_eq!(email_mentor_npc_type_0104(mentor), npc_type);
    }

    let content = runtime_test_mission_content();
    let catalog = runtime_test_email_catalog_0104();
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    load.as_bytes_mut()[ffone_protocol::PcLoadData0104::MENTOR_OFFSET
        ..ffone_protocol::PcLoadData0104::MENTOR_OFFSET + 2]
        .copy_from_slice(&GuideMentor::Edd.wire_id().to_le_bytes());
    let mut guide = GuideRuntime::new(GuideServerProfile::OpenFusion0104);
    guide.load_pc_state(&load);

    let messages = email_guide_messages_0104(&catalog, &content, &guide, [], [255]).unwrap();
    let [message] = messages.as_slice() else {
        panic!("available Edd task 255 must project one clean guide email")
    };
    assert_eq!(message.mode, 2);
    assert_eq!(message.mission_task_id, 255);
    assert_eq!(message.sender_npc_id, 707);
    assert_eq!(message.sender_name, "Edd");
    assert_eq!(message.subject, "Don't Be a Drip");
    assert_eq!(
        message.content,
        "Hey, pal! We need some assistance with an ice cream delivery! Think you can lend a hand? Speak to Urban Ranger Ralphie in Hero Square. -Edd"
    );
    assert!(!message.auto_delete_note);

    // `Panel_EmailList` resolves `GetEquipIconElement(30, GuideNpcNum[...])`
    // for every selectable mentor and the Future Computress.
    let icons = [707_i16, 728, 731, 732, 1171].map(|npc_type| {
        UserEquipCatalogQuery::from_non_empty_item(ItemBase0104 {
            item_type: 30,
            item_id: npc_type,
            option: 0,
            time_limit: 0,
        })
        .ok()
        .and_then(|query| UserEquipItemCatalog::resolve_icon(&content, query))
        .map(|icon| icon.runtime_path().to_owned())
        .unwrap_or_else(|| panic!("guide NPC {npc_type} has no type-30 icon"))
    });
    for icon in &icons {
        assert!(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../assets/game")
                .join(icon)
                .is_file(),
            "{icon}"
        );
    }
    assert_eq!(message.sender_icon_path.as_deref(), Some(icons[0].as_str()));
}

#[test]
fn combi_frame_inbox_claims_only_a_correlated_owned_reply_and_keeps_malformed_pending() {
    let catalog = runtime_test_combi_catalog_0104();
    let idle = CombiProductionRuntime0104::default();
    let mut inbox = CombiNetworkFrameInbox0104::default();
    let malformed = DecodedFrame {
        packet_type: COMBI_SUCCESS_PACKET_ID_0104,
        flags: 0,
        checksum: 0,
        payload: Vec::new(),
    };
    assert!(!inbox.push_if_owned(&idle, malformed.clone()));

    let (mut runtime, _, _, _) = pending_combi_test_runtime_0104(&catalog);
    assert!(!inbox.push_if_owned(
        &runtime,
        DecodedFrame {
            packet_type: 0xdead_beef,
            flags: 0,
            checksum: 0,
            payload: Vec::new(),
        },
    ));
    assert!(inbox.push_if_owned(&runtime, malformed));
    let error = runtime.apply_frame(inbox.pop_front().unwrap()).unwrap_err();
    assert!(matches!(
        error,
        CombiProductionError0104::MalformedFrame { .. }
    ));
    assert!(runtime.request_pending());
    assert!(inbox.pop_front().is_none());
}

#[test]
fn enchant_frame_inbox_owns_only_the_correlated_pending_operation() {
    let item = ItemBase0104 {
        item_type: 7,
        item_id: 134,
        option: 3,
        time_limit: 91,
    };
    let inventory = enchant_test_inventory_0104(&[(4, item)]);
    let runtime = open_enchant_test_runtime_0104(&inventory);
    for packet_type in [
        ENCHANT_SUCCESS_PACKET_ID_0104,
        ENCHANT_FAILURE_PACKET_ID_0104,
        ENCHANT_DELETE_SUCCESS_PACKET_ID_0104,
        ENCHANT_DISASSEMBLE_SUCCESS_PACKET_ID_0104,
        ENCHANT_DISASSEMBLE_FAILURE_PACKET_ID_0104,
    ] {
        assert!(!enchant_frame_owned_0104(&runtime, packet_type));
    }

    let (runtime, _) = pending_enchant_delete_test_runtime_0104(item);
    assert!(enchant_frame_owned_0104(
        &runtime,
        ENCHANT_DELETE_SUCCESS_PACKET_ID_0104
    ));
    assert!(!enchant_frame_owned_0104(
        &runtime,
        ENCHANT_SUCCESS_PACKET_ID_0104
    ));
    assert!(!enchant_frame_owned_0104(
        &runtime,
        ENCHANT_FAILURE_PACKET_ID_0104
    ));
    assert!(!enchant_frame_owned_0104(
        &runtime,
        ENCHANT_DISASSEMBLE_SUCCESS_PACKET_ID_0104
    ));

    let mut inbox = EnchantNetworkFrameInbox0104::default();
    assert!(
        inbox.push_if_owned(
            &runtime,
            DecodedFrame {
                packet_type: ENCHANT_DELETE_SUCCESS_PACKET_ID_0104,
                flags: 0,
                checksum: 0,
                payload: ffone_protocol::PcItemDeleteSuccess0104 {
                    item_location: 1,
                    slot_num: 4,
                }
                .encode(),
            }
        )
    );
    assert!(!inbox.push_if_owned(
        &runtime,
        DecodedFrame {
            packet_type: ENCHANT_DISASSEMBLE_SUCCESS_PACKET_ID_0104,
            flags: 0,
            checksum: 0,
            payload: Vec::new(),
        }
    ));
    assert_eq!(inbox.0.len(), 1);
}

#[test]
fn user_store_main_guard_drops_manual_packet_without_transport_or_authority_mutation() {
    let mut app = App::new();
    app.init_resource::<RuntimeStatus>()
        .init_resource::<UserStoreProductionRuntime0104>()
        .init_resource::<UserStoreUiState0104>()
        .init_resource::<UserStoreAuthority0104>()
        .init_resource::<UserStorePopupPresentation0104>()
        .init_resource::<UserStoreUiOutbox0104>()
        .add_systems(Update, guard_user_store_before_interaction_0104);

    {
        let mut authority = app.world_mut().resource_mut::<UserStoreAuthority0104>();
        authority.owner_pc_id = 7;
        authority.taros = 1234;
    }
    *app.world_mut().resource_mut::<UserStoreUiState0104>() = UserStoreUiState0104 {
        active: true,
        pending: Some(
            ffone_client::user_store_ui::UserStorePendingRequest0104::ItemList { target_pc_id: 19 },
        ),
        ..default()
    };
    app.world_mut()
        .resource_mut::<UserStoreUiOutbox0104>()
        .push(
            ffone_client::user_store_ui::UserStoreUiCommand0104::SendPacket(
                ffone_client::user_store_ui::UserStorePacket0104::item_list(19),
            ),
        );

    assert!(!app.world().contains_resource::<NetworkBridge>());
    app.update();

    assert_eq!(
        *app.world().resource::<UserStoreUiState0104>(),
        UserStoreUiState0104::default()
    );
    assert!(app.world().resource::<UserStoreUiOutbox0104>().0.is_empty());
    let authority = app.world().resource::<UserStoreAuthority0104>();
    assert_eq!((authority.owner_pc_id, authority.taros), (7, 1234));
    let rejection = app
        .world()
        .resource::<UserStoreProductionRuntime0104>()
        .last_rejection()
        .expect("manual UserStore shell must be rejected");
    assert_eq!(rejection.dropped_commands, 1);
    assert!(matches!(
        rejection.error,
        ffone_client::user_store_runtime::UserStoreProductionError0104::NoRegisteredProductionSession
    ));
    assert!(
        app.world()
            .resource::<RuntimeStatus>()
            .message
            .contains("no registry-proven production session")
    );
    assert!(!app.world().contains_resource::<NetworkBridge>());
}
