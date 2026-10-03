use super::*;

#[test]
fn cashmall_hidden_command_preserves_clean_access_and_space_split() {
    assert!(is_cashmall_hidden_chat_command_0104("/cashmall", 50));
    assert!(is_cashmall_hidden_chat_command_0104(
        "/cashmall retained arguments",
        0
    ));
    assert!(!is_cashmall_hidden_chat_command_0104("/cashmall", 51));
    assert!(!is_cashmall_hidden_chat_command_0104("/Cashmall", 50));
    assert!(!is_cashmall_hidden_chat_command_0104(" /cashmall", 50));
    assert!(!is_cashmall_hidden_chat_command_0104(
        "/cashmall\targument",
        50
    ));
}

#[test]
fn user_equip_chest_ack_releases_the_dedicated_request_owner() {
    let content = runtime_test_mission_content();
    let mut inventory = LocalInventoryRuntime::default();
    inventory.seed(77, &ffone_protocol::PcLoadData0104::zeroed());
    let request = ffone_protocol::ItemChestOpenRequest0104 {
        item_location: 1,
        slot_num: 7,
        chest_item: ItemBase0104 {
            item_type: 9,
            item_id: 424,
            option: 1,
            time_limit: 0,
        },
    };
    let mut production = UserEquipProductionRuntime0104::default();
    production
        .begin(UserEquipPendingRequest0104::ChestOpen(request))
        .unwrap();
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_ITEM_CHEST_OPEN_SUCC,
        flags: 0,
        checksum: 0,
        payload: ItemChestOpenSuccess0104 { slot_num: 7 }.encode(),
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
        Ok(true)
    );
    assert!(!production.send_pending());
    assert!(!production.owns_chest_open_reply_family());
    assert!(runtime.message.contains("accepted chest open"));
}

#[test]
fn user_equip_delete_rejects_mismatched_reply_without_leaking_or_mutating() {
    let content = runtime_test_mission_content();
    let mut inventory = LocalInventoryRuntime::default();
    inventory.seed(77, &ffone_protocol::PcLoadData0104::zeroed());
    let request = PcItemDeleteRequest0104 {
        item_location: 1,
        slot_num: 4,
    };
    let mut production = UserEquipProductionRuntime0104::default();
    production
        .begin(UserEquipPendingRequest0104::Delete(request))
        .unwrap();
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_ITEM_DELETE_SUCC,
        flags: 0,
        checksum: 0,
        payload: PcItemDeleteSuccess0104 {
            item_location: 1,
            slot_num: 5,
        }
        .encode(),
    };
    let before = inventory.snapshot().unwrap().clone();
    let mut runtime = RuntimeStatus::default();

    assert!(
        apply_user_equip_frame(
            &frame,
            &mut inventory,
            &content,
            &mut production,
            &BankProductionRuntime0104::default(),
            &VendorProductionRuntime0104::default(),
            &mut runtime,
        )
        .is_err()
    );
    assert!(production.send_pending());
    assert_eq!(inventory.snapshot(), Some(&before));
}

#[test]
fn email_local_request_rejection_releases_only_the_undispatched_front_request() {
    let request = EmailRequest::Send {
        recipient_pc_uid: 55,
        subject: "Subject".to_owned(),
        content: "Body".to_owned(),
        items: default(),
        cash: 0,
    };
    let mut model = EmailUiModel::default();
    model.draft.subject = "Subject".to_owned();
    model.draft.content = "Body".to_owned();
    model.send_in_flight = true;
    model.mail_send_in_flight = true;
    let draft = model.draft.clone();
    let mut transport = EmailTransportOutbox::default();
    transport.push(request.clone());

    let error = EmailProductionError0104::RequestNotReachable {
        request: request.clone(),
    };
    assert!(email_dispatch_error_rejects_front_request_0104(&error));
    assert_eq!(
        release_rejected_email_request_0104(&mut model, &mut transport),
        Some(request)
    );
    assert!(transport.0.is_empty());
    assert!(!model.send_in_flight);
    assert!(!model.mail_send_in_flight);
    assert_eq!(model.draft, draft, "the player must be able to correct it");

    assert!(!email_dispatch_error_rejects_front_request_0104(
        &EmailProductionError0104::NotActive
    ));

    let item = ItemBase0104 {
        item_type: 0,
        item_id: 1,
        option: 0,
        time_limit: 0,
    };
    let content = runtime_test_mission_content();
    let output = email_local_rejection_output_0104(
        &EmailProductionError0104::AttachmentRejected {
            inventory_slot: 4,
            item,
            reason: EmailAttachmentRejection0104::NotTradeable,
        },
        &content,
    )
    .expect("clean non-tradeable attachment message");
    let [
        EmailUiAction::SystemMessage {
            message_id,
            fallback,
            ..
        },
    ] = output.actions.as_slice()
    else {
        panic!("non-tradeable rejection did not preserve clean SystemMessage 177")
    };
    assert_eq!(*message_id, Some(177));
    assert_eq!(fallback, "SORRY!\nThis item cannot be traded or emailed.");
    assert!(
        email_local_rejection_output_0104(
            &EmailProductionError0104::AttachmentRejected {
                inventory_slot: 4,
                item,
                reason: EmailAttachmentRejection0104::Chest,
            },
            &content,
        )
        .is_none(),
        "the clean chest branch returns silently before SystemMessage 177"
    );
}

#[test]
fn enchant_unregistered_confirm_requeues_the_same_modal_without_a_request() {
    let runtime = enchant_confirmation_test_runtime_0104();
    let request_id = ENCHANT_SYSTEM_MESSAGE_ID_BASE_0104;
    let request = SystemMessageRequest::new(
        request_id,
        "Confirm enchant",
        SystemMessageButtonType::YesNo,
    );
    let mut shell = EnchantProductionShell0104::default();
    shell.pending_system_messages.insert(
        request_id,
        PendingEnchantSystemMessage0104 {
            request: request.clone(),
            action: PendingEnchantSystemAction0104::Model,
        },
    );
    let mut outbox = SystemMessageUiOutbox::default();
    outbox.push(SystemMessageUiAction::Chosen {
        request_id,
        button_type: request.button_type,
        choice: SystemMessageChoice::Primary,
    });

    let mut app = App::new();
    app.insert_resource(runtime)
        .insert_resource(shell)
        .insert_resource(outbox)
        .init_resource::<SystemMessageUiModel>()
        .init_resource::<RuntimeStatus>()
        .add_systems(Update, consume_enchant_system_message_outbox_0104);
    app.update();

    let runtime = app.world().resource::<EnchantProductionRuntime0104>();
    assert!(!runtime.request_pending());
    assert!(matches!(
        runtime.session().unwrap().model().phase(),
        EnchantPhase0104::SystemMessage { .. }
    ));
    let shell = app.world().resource::<EnchantProductionShell0104>();
    assert!(shell.pending_outputs.is_empty());
    assert_eq!(shell.pending_system_messages.len(), 1);
    assert_eq!(
        app.world()
            .resource::<SystemMessageUiModel>()
            .current()
            .map(|current| current.request_id),
        Some(request_id)
    );
    assert!(
        app.world()
            .resource::<RuntimeStatus>()
            .message
            .contains("confirmation failed closed")
    );
}

#[test]
fn bag_crate_and_egg_rewards_add_clean_event_chat_lines() {
    let content = runtime_test_mission_content();
    let chest = |kind| (0..i16::MAX).find(|id| content.reward_chest_kind(*id) == Some(kind));
    let crate_id = chest(0).expect("production TableData must contain a C.R.A.T.E row");
    let egg_id = chest(1);
    let reward = |item_type, item_id, inventory_location| ffone_protocol::ItemReward0104 {
        item: ffone_protocol::ItemBase0104 {
            item_type,
            item_id,
            option: 1,
            time_limit: 0,
        },
        inventory_location,
        slot: 0,
    };
    // Only bag (`eIL == 1`) chest items reach `RewardItem`; quest items and
    // other inventory locations never add a chest line.
    let mut items = vec![
        reward(9, crate_id, 1),
        reward(9, crate_id, 0),
        reward(8, crate_id, 1),
    ];
    items.extend(egg_id.map(|egg_id| reward(9, egg_id, 1)));
    let reply = ffone_protocol::RewardItemReply0104 {
        candy: 0,
        fusion_matter: 0,
        nano_battery: 0,
        weapon_battery: 0,
        pack_padding: [0; 3],
        fatigue: 0,
        fatigue_level: 0,
        npc_type_id: 0,
        task_id: 0,
        items,
    };
    let keys = reward_chest_chat_lines(&content, &reply)
        .into_iter()
        .map(|line| line.localized.expect("chest chat lines are keyed").key)
        .collect::<Vec<_>>();
    let mut expected = vec!["ui.hud.chat.reward.crate".to_owned()];
    expected.extend(egg_id.map(|_| "ui.hud.chat.reward.egg".to_owned()));
    assert_eq!(keys, expected);
}
