use super::*;

#[test]
fn combi_authoritative_commit_is_atomic_and_failure_still_commits_taros() {
    let catalog = runtime_test_combi_catalog_0104();
    let (mut runtime, inventory, style, stats) = pending_combi_test_runtime_0104(&catalog);
    let combined = ItemBase0104 {
        item_type: style.item_type,
        item_id: stats.item_id,
        option: i32::from(style.item_id) << 16,
        time_limit: style.time_limit,
    };
    let output = runtime
        .apply_reply(CombiReplyPacket0104::Success(CombiSuccessReply0104 {
            new_item_slot: 4,
            new_item: combined,
            stat_item_slot: 7,
            cash_item_slot_1: 0,
            cash_item_slot_2: 0,
            taros_after: 91_234,
            success_flag: 1,
        }))
        .unwrap();
    let commit = output.commit.expect("authoritative success commit");
    let next = combi_inventory_after_commit_0104(&inventory, &commit).unwrap();
    assert_eq!(inventory.inventory()[4], style, "input stays immutable");
    assert_eq!(inventory.inventory()[7], stats, "input stays immutable");
    assert_eq!(next.inventory()[4], combined);
    assert_eq!(next.inventory()[7].item_id, 0);
    assert_eq!(commit.taros_after(), 91_234);

    let stale = enchant_test_inventory_0104(&[
        (
            4,
            ItemBase0104 {
                item_id: 4,
                ..style
            },
        ),
        (7, stats),
    ]);
    assert!(combi_inventory_after_commit_0104(&stale, &commit).is_err());

    let (mut runtime, inventory, style, _) = pending_combi_test_runtime_0104(&catalog);
    let output = runtime
        .apply_reply(CombiReplyPacket0104::Success(CombiSuccessReply0104 {
            new_item_slot: 4,
            new_item: style,
            stat_item_slot: 7,
            cash_item_slot_1: 0,
            cash_item_slot_2: 0,
            taros_after: 88_765,
            success_flag: 0,
        }))
        .unwrap();
    let commit = output.commit.expect("authoritative failed-attempt commit");
    let next = combi_inventory_after_commit_0104(&inventory, &commit).unwrap();
    assert_eq!(next.inventory(), inventory.inventory());
    assert_eq!(commit.taros_after(), 88_765);
    assert!(output.effects.iter().any(|effect| matches!(
        effect,
        CombiShellEffect0104::SystemMessage(CombiSystemMessage0104::Modal {
            modal: CombiSystemModal0104::CombinationFailed,
            ..
        })
    )));
}

#[test]
fn vendor_redeem_uses_shared_owner_and_preserves_modal_gate() {
    use ffone_client::shared_input_ui::{RedeemSource, SharedInputDialog, SharedRedeemCode};
    let state = VendorUiState {
        phase: VendorLifecyclePhase::Visible,
        ..default()
    };
    let projection = VendorModeProjection0104::default();
    let mut input = SharedInputDialog::default();
    let mut redeem = SharedRedeemCode::default();
    assert!(!shared_redeem::open_vendor_redeem(
        &state,
        VendorModalState {
            system_popup: true,
            ..default()
        },
        &projection,
        &mut redeem,
        &mut input
    ));
    assert!(input.owner().is_none());
    assert!(shared_redeem::open_vendor_redeem(
        &state,
        VendorModalState::default(),
        &projection,
        &mut redeem,
        &mut input
    ));
    assert_eq!(
        redeem.source,
        Some(RedeemSource::Vendor {
            pc: projection.owner_pc_id,
            npc: projection.session.requested_npc_id
        })
    );
}

#[test]
fn shared_redeem_entry_keeps_short_codes_open_and_routes_valid_submission() {
    use ffone_client::shared_input_ui::{SharedInputDialog, SharedInputRequest};
    let inventory = enchant_test_inventory_0104(&[]);
    let mut runtime = open_enchant_test_runtime_0104(&inventory);
    runtime
        .apply_ui_command(EnchantUiCommand0104::OpenRedeemCode, &|_: ItemBase0104| {
            None
        })
        .unwrap();
    let mut input = SharedInputDialog::default();
    input.open(SharedInputRequest {
        owner: ENCHANT_INPUT_OWNER,
        title: LocalizedText::new("ui.enchant.redeem_code", "REDEEM CODE"),
        instruction: LocalizedText::new(
            "ui.shared_input.redeem_instruction",
            "Have a code? Try entering it below to get an exclusive item!",
        ),
        submit: LocalizedText::new("ui.shared_input.redeem", "REDEEM"),
        max_utf16: 32,
    });
    input.append("ab");
    input.submit();
    let mut app = App::new();
    app.insert_resource(runtime)
        .insert_resource(input)
        .insert_resource(EnchantProductionShell0104 {
            redeem_code_active: true,
            ..default()
        })
        .init_resource::<RuntimeStatus>()
        .add_systems(Update, consume_enchant_text_input);
    app.update();
    assert_eq!(
        app.world().resource::<SharedInputDialog>().owner(),
        Some(ENCHANT_INPUT_OWNER)
    );
    assert!(
        app.world()
            .resource::<EnchantProductionShell0104>()
            .pending_outputs
            .iter()
            .all(|output| output.request.is_none())
    );
    app.world_mut()
        .resource_mut::<EnchantProductionShell0104>()
        .pending_outputs
        .clear();
    let mut input = app.world_mut().resource_mut::<SharedInputDialog>();
    input.append("c");
    input.submit();
    drop(input);
    app.update();
    assert_eq!(app.world().resource::<SharedInputDialog>().owner(), None);
    let shell = app.world().resource::<EnchantProductionShell0104>();
    assert!(!shell.redeem_code_active);
    assert_eq!(
        shell
            .pending_outputs
            .iter()
            .filter(|output| output.request.is_some())
            .count(),
        1
    );
    assert!(
        shell
            .pending_outputs
            .iter()
            .all(|output| output.commit.is_none())
    );
}

#[test]
fn bank_reset_restores_all_modal_and_transport_gates() {
    let inventory =
        InventoryRuntime0104::from_pc_load(77, &ffone_protocol::PcLoadData0104::zeroed());
    let mut state = BankUiState::default();
    let mut modal = BankModalState {
        help: true,
        inventory_popup: true,
        system_popup: true,
        generic_popup: true,
    };
    let mut projection = BankModeProjection0104::default();
    let mut outbox = BankUiOutbox0104::default();
    state.begin_open(77, 9_001, &mut outbox);
    let mut production = BankProductionRuntime0104::default();
    production
        .begin_open(
            ffone_protocol::PcBankOpenRequest0104 {
                pc_id: 77,
                npc_id: 9_001,
            },
            &inventory,
        )
        .unwrap();
    let mut system_runtime = BankSystemMessageRuntime::default();
    system_runtime.special_state_active = true;
    system_runtime.pending_access_required.insert(
        BANK_SYSTEM_MESSAGE_ID_BASE,
        BankOpenIdentity0104 {
            owner_pc_id: 77,
            npc_id: 9_001,
        },
    );

    reset_bank_shell(
        &mut state,
        &mut modal,
        &mut projection,
        &mut outbox,
        &mut production,
        &mut system_runtime,
    );

    assert_eq!(state, BankUiState::default());
    assert_eq!(modal, BankModalState::default());
    assert_eq!(projection, BankModeProjection0104::default());
    assert!(outbox.is_empty());
    assert!(!production.modal_active());
    assert!(!system_runtime.special_state_active);
    assert!(system_runtime.pending_access_required.is_empty());
}

#[test]
fn nano_create_commits_quest_bank_and_progression_as_one_transaction() {
    let load = ffone_protocol::PcLoadData0104::zeroed();
    let mut inventory = LocalInventoryRuntime::default();
    inventory.seed(77, &load);
    let mut bank = NanoFreeTuningBank0104::default();
    bank.seed(&load);
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_level: 4,
            fusion_matter: 900,
            ..default()
        },
        ..default()
    };
    let packet = PcNanoCreateSuccess0104 {
        fusion_matter: 750,
        quest_item_slot: 3,
        quest_item: ItemBase0104 {
            item_type: 7,
            item_id: 411,
            option: 2,
            time_limit: 0,
        },
        nano: ffone_protocol::Nano0104 {
            id: 6,
            skill_id: 0,
            stamina: 150,
        },
        player_level: 5,
    };

    apply_nano_create_commit(packet, &mut inventory, &mut bank, &mut runtime).unwrap();

    assert_eq!(
        inventory.quest_inventory.as_ref().unwrap()[3],
        packet.quest_item
    );
    assert_eq!(bank.entries()[6], packet.nano);
    assert_eq!(runtime.player_level, 5);
    assert_eq!(runtime.fusion_matter, 750);
}

#[test]
fn email_all_production_npc_letters_have_explicit_en_ru_keys() {
    let content = runtime_test_mission_content();
    let catalog = runtime_test_email_catalog_0104();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let bundles = ["en", "ru"].map(|locale| {
        serde_json::from_slice::<serde_json::Value>(
            &std::fs::read(root.join(format!("localization/{locale}.json"))).unwrap(),
        )
        .unwrap()
    });
    assert_eq!(
        bundles[0]["entries"]
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>(),
        bundles[1]["entries"]
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>()
    );
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    let offset = ffone_protocol::PcLoadData0104::MENTOR_OFFSET;
    for mentor in 1_i16..=5 {
        load.as_bytes_mut()[offset..offset + 2].copy_from_slice(&mentor.to_le_bytes());
        let mut guide = GuideRuntime::new(GuideServerProfile::OpenFusion0104);
        guide.load_pc_state(&load);
        let messages = email_guide_messages_0104(
            &catalog,
            &content,
            &guide,
            catalog.guide_rows.iter().map(|row| row.task_id),
            [],
        )
        .unwrap();
        assert!(messages.len() >= 151);
        let invitations = email_guide_messages_0104(
            &catalog,
            &content,
            &guide,
            [],
            catalog.guide_rows.iter().map(|row| row.task_id),
        )
        .unwrap();
        assert!(!invitations.is_empty());
        for message in messages.iter().chain(&invitations) {
            for text in [
                message.localized_sender(),
                message.localized_subject(),
                message.localized_content(),
            ] {
                for bundle in &bundles {
                    assert!(
                        bundle["entries"].get(&text.key).is_some(),
                        "missing {} in {}",
                        text.key,
                        bundle["locale"]
                    );
                    assert!(!bundle["entries"][&text.key].as_str().unwrap().is_empty());
                }
            }
        }
    }
}

#[test]
fn email_available_invitations_follow_progress_and_are_limited_by_category() {
    let content = runtime_test_mission_content();
    let catalog = runtime_test_email_catalog_0104();
    let nano_bank = NanoFreeTuningBank0104::default();
    let inventory = LocalInventoryRuntime::default();
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    let offset = ffone_protocol::PcLoadData0104::MENTOR_OFFSET;
    for mentor in 1_i16..=5 {
        load.as_bytes_mut()[offset..offset + 2].copy_from_slice(&mentor.to_le_bytes());
        let mut guide = GuideRuntime::new(GuideServerProfile::OpenFusion0104);
        guide.load_pc_state(&load);
        let mut world = WorldMissionRuntime::default();
        world.seed(&load, &content).unwrap();
        let tasks = available_email_task_ids_0104(
            &catalog, &content, &world, &guide, &nano_bank, &inventory, 100,
        );
        assert!(!tasks.is_empty(), "mentor {mentor} has no invitations");
        assert!(tasks.len() <= 2);
        let letters =
            email_guide_messages_0104(&catalog, &content, &guide, [], tasks.clone()).unwrap();
        assert_eq!(letters.len(), tasks.len());
        assert!(letters.iter().all(|message| message.mode == 2));
        for task in tasks {
            let definition = content.mission(task).unwrap();
            let mut active_load = load.clone();
            let slot = usize::from(definition.mission_type != TutorialMissionType::Nano);
            let offset = ffone_protocol::PcLoadData0104::RUNNING_QUESTS_OFFSET
                + slot * ffone_protocol::RunningQuest0104::SIZE;
            active_load.as_bytes_mut()[offset..offset + 4].copy_from_slice(&task.to_le_bytes());
            let mut active_world = WorldMissionRuntime::default();
            active_world.seed(&active_load, &content).unwrap();
            assert!(
                available_email_task_ids_0104(
                    &catalog,
                    &content,
                    &active_world,
                    &guide,
                    &nano_bank,
                    &inventory,
                    100,
                )
                .iter()
                .all(|id| content.mission(*id).unwrap().provenance.mission_id
                    != definition.provenance.mission_id)
            );
            world
                .accept_gm_mission_completion(definition.provenance.mission_id)
                .unwrap();
            assert!(
                !available_email_task_ids_0104(
                    &catalog, &content, &world, &guide, &nano_bank, &inventory, 100
                )
                .contains(&task)
            );
        }
    }
}

#[test]
fn email_npc_arrivals_notify_once_and_reset_between_characters() {
    let mut notices = NpcMailNotices0104::default();
    assert!(notices.observe(Some(1), [(1, 198)]));
    assert!(!notices.observe(Some(1), [(1, 198)]));
    assert!(notices.observe(Some(1), [(1, 198), (2, 255)]));
    assert!(!notices.observe(Some(1), [(2, 255)]));
    assert!(notices.observe(Some(2), [(2, 255)]));
    assert!(!notices.observe(Some(2), []));
}
