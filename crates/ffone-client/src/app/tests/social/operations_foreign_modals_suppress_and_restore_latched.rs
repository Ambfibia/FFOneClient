use super::*;

pub(in super::super) fn nanocom_context_test_app(state: ClientState) -> App {
    let mut app = App::new();
    app.insert_resource(State::new(state))
        .init_resource::<GameplayLoadingState>()
        .init_resource::<TutorialNanocomMessageContext>()
        .init_resource::<TutorialChoreographyPresentation>()
        .init_resource::<NanocomMessageUiModel>()
        .init_resource::<MissionUiModel>()
        .init_resource::<GameplayUiModel>()
        .init_resource::<GuideUiModel>()
        .init_resource::<GuideProductionRuntime>()
        .init_resource::<BankUiState>()
        .init_resource::<BankUiOutbox0104>()
        .init_resource::<BankProductionRuntime0104>()
        .init_resource::<VendorUiState>()
        .init_resource::<VendorUiOutbox0104>()
        .init_resource::<VendorProductionRuntime0104>()
        .init_resource::<RuleUiModel>()
        .init_resource::<RuleRuntime>()
        .init_resource::<NanoFreeTuningModel>()
        .init_resource::<NanoFreeTuningProductionRuntime>()
        .init_resource::<UpsellUiModel>()
        .init_resource::<UserEquipUiState>()
        .init_resource::<Pc2pcUiModel0104>()
        .init_resource::<WorldMapPresentation>()
        .init_resource::<RaceProductionRuntime>()
        .init_resource::<EmailProductionRuntime0104>()
        .init_resource::<CombiProductionRuntime0104>()
        .init_resource::<EnchantProductionRuntime0104>()
        .add_systems(Update, social_ingress::sync_buddy_nanocom_context);
    app
}

#[test]
fn npc_dialogue_and_journal_suspend_nanocom_without_losing_the_message() {
    use ffone_client::nanocom_message_ui::NanocomMessageSound;

    let mut app = nanocom_context_test_app(ClientState::World);
    {
        let mut mission = app.world_mut().resource_mut::<MissionUiModel>();
        mission.enabled = true;
        mission.npc_icon_mode_visible = true;
    }
    app.world_mut()
        .resource_mut::<NanocomMessageUiModel>()
        .enqueue_type_9_numbuh_two(9, "Numbuh Two", "Come in, cadet.");
    app.update();
    let initial_lifetime = app
        .world()
        .resource::<NanocomMessageUiModel>()
        .active()
        .unwrap()
        .remaining_seconds;

    // Cover an ordinary NPC conversation and each journal presentation; waiting
    // longer than the notice lifetime must not discard its queued head.
    app.insert_resource(State::new(ClientState::Tutorial));
    for journal in [
        MissionJournalUi::Hidden,
        MissionJournalUi::Allow(default()),
        MissionJournalUi::Reward {
            mission: default(),
            box1_choice: 0,
            box2_choice: 0,
        },
        MissionJournalUi::Other(default()),
    ] {
        {
            let mut mission = app.world_mut().resource_mut::<MissionUiModel>();
            mission.npc_icon_mode_visible = matches!(journal, MissionJournalUi::Hidden);
            mission.journal = journal;
        }
        app.update();
        assert!(
            app.world()
                .resource::<TutorialNanocomMessageContext>()
                .scene_event_active
        );
        let mut messages = app.world_mut().resource_mut::<NanocomMessageUiModel>();
        assert!(messages.scene_event_active());
        assert!(!messages.compact_visible());
        assert!(!messages.expanded_visible());
        assert_eq!(messages.tick(120.0), None);
        assert_eq!(messages.active().unwrap().request.request_id, 9);
        assert_eq!(
            messages.active().unwrap().remaining_seconds,
            initial_lifetime
        );
        assert_eq!(messages.reveal_parameter(), 1.0);
    }

    {
        let mut mission = app.world_mut().resource_mut::<MissionUiModel>();
        mission.journal = MissionJournalUi::Hidden;
        mission.nanocom_main_menu_visible = true;
    }
    app.update();
    assert!(
        !app.world()
            .resource::<TutorialNanocomMessageContext>()
            .scene_event_active
    );
    let mut messages = app.world_mut().resource_mut::<NanocomMessageUiModel>();
    assert!(!messages.scene_event_active());
    assert!(messages.compact_visible());
    assert!(messages.expanded());
    assert_eq!(messages.pop_sound(), Some(NanocomMessageSound::SlideIn));
    assert_eq!(messages.tick(1.0), None);
    assert_eq!(
        messages.active().unwrap().remaining_seconds,
        initial_lifetime - 1.0
    );
}

#[test]
fn production_modal_gate_keeps_chat_active_while_nanocom_is_open() {
    let mut app = App::new();
    app.init_resource::<MissionUiModel>()
        .init_resource::<GuideUiModel>()
        .init_resource::<GuideProductionRuntime>()
        .init_resource::<BankUiState>()
        .init_resource::<BankUiOutbox0104>()
        .init_resource::<BankProductionRuntime0104>()
        .init_resource::<VendorUiState>()
        .init_resource::<VendorUiOutbox0104>()
        .init_resource::<VendorProductionRuntime0104>()
        .init_resource::<RuleUiModel>()
        .init_resource::<RuleRuntime>()
        .init_resource::<NanoFreeTuningModel>()
        .init_resource::<NanoFreeTuningProductionRuntime>()
        .init_resource::<UpsellUiModel>()
        .init_resource::<UserEquipUiState>()
        .init_resource::<Pc2pcUiModel0104>()
        .init_resource::<WorldMapPresentation>()
        .init_resource::<RaceProductionRuntime>()
        .init_resource::<EmailProductionRuntime0104>()
        .init_resource::<CombiProductionRuntime0104>()
        .init_resource::<EnchantProductionRuntime0104>()
        .init_resource::<OptionUiModel>()
        .init_resource::<SystemMessageUiModel>()
        .init_resource::<QuitMenuUiModel>()
        .init_resource::<QuitMenuRuntime>()
        .init_resource::<ResurrectUiModel>()
        .init_resource::<GameplayUiModel>()
        .add_systems(Update, sync_guide_chat_input_gate);
    {
        let mut mission = app.world_mut().resource_mut::<MissionUiModel>();
        mission.enabled = true;
        mission.nanocom_main_menu_visible = true;
    }
    {
        let mut gameplay = app.world_mut().resource_mut::<GameplayUiModel>();
        gameplay.visible = true;
        gameplay.chat.visible = true;
        gameplay.chat.input_enabled = true;
        gameplay.chat.active = true;
        gameplay.chat.input = "Привет".to_owned();
    }

    app.update();

    let gameplay = app.world().resource::<GameplayUiModel>();
    assert!(gameplay.chat.input_enabled);
    assert!(gameplay.chat.active);
    assert_eq!(gameplay.chat.input, "Привет");
}

#[test]
fn foreign_modals_suppress_and_restore_latched_nanocom_and_chat() {
    fn restore_ready_chat_input(mut gameplay: ResMut<GameplayUiModel>) {
        gameplay.chat.input_enabled = true;
    }

    fn assert_foreign_modal_state(app: &App, suppressed: bool) {
        let mission = app.world().resource::<MissionUiModel>();
        assert!(mission.nanocom_main_menu_visible);
        assert_eq!(mission.nanocom_foreign_modal_suppressed(), suppressed);
        assert_eq!(mission.nanocom_menu_presented(), !suppressed);
        assert_eq!(
            app.world().resource::<GameplayUiModel>().chat.input_enabled,
            !suppressed
        );
    }

    let mut app = App::new();
    app.init_resource::<MissionUiModel>()
        .init_resource::<GuideUiModel>()
        .init_resource::<GuideProductionRuntime>()
        .init_resource::<BankUiState>()
        .init_resource::<BankUiOutbox0104>()
        .init_resource::<BankProductionRuntime0104>()
        .init_resource::<VendorUiState>()
        .init_resource::<VendorUiOutbox0104>()
        .init_resource::<VendorProductionRuntime0104>()
        .init_resource::<RuleUiModel>()
        .init_resource::<RuleRuntime>()
        .init_resource::<NanoFreeTuningModel>()
        .init_resource::<NanoFreeTuningProductionRuntime>()
        .init_resource::<UpsellUiModel>()
        .init_resource::<UserEquipUiState>()
        .init_resource::<Pc2pcUiModel0104>()
        .init_resource::<WorldMapPresentation>()
        .init_resource::<RaceProductionRuntime>()
        .init_resource::<EmailProductionRuntime0104>()
        .init_resource::<CombiProductionRuntime0104>()
        .init_resource::<EnchantProductionRuntime0104>()
        .init_resource::<OptionUiModel>()
        .init_resource::<SystemMessageUiModel>()
        .init_resource::<QuitMenuUiModel>()
        .init_resource::<QuitMenuRuntime>()
        .init_resource::<ResurrectUiModel>()
        .init_resource::<GameplayUiModel>()
        .add_systems(
            Update,
            (
                restore_ready_chat_input,
                sync_nanocom_foreign_modal_suppression,
                sync_guide_chat_input_gate,
            )
                .chain(),
        );
    {
        let mut mission = app.world_mut().resource_mut::<MissionUiModel>();
        mission.enabled = true;
        mission.nanocom_main_menu_visible = true;
    }
    {
        let mut gameplay = app.world_mut().resource_mut::<GameplayUiModel>();
        gameplay.visible = true;
        gameplay.chat.visible = true;
        gameplay.chat.active = true;
        gameplay.chat.input = "latched NanoCom chat".to_owned();
    }

    app.update();
    assert_foreign_modal_state(&app, false);

    app.world_mut().resource_mut::<GuideUiModel>().visible = true;
    app.update();
    assert_foreign_modal_state(&app, true);
    app.world_mut().resource_mut::<GuideUiModel>().visible = false;
    app.update();
    assert_foreign_modal_state(&app, false);

    app.world_mut().resource_mut::<QuitMenuUiModel>().open();
    app.update();
    assert_foreign_modal_state(&app, true);
    app.world_mut().resource_mut::<QuitMenuUiModel>().close();
    app.update();
    assert_foreign_modal_state(&app, false);

    app.world_mut()
        .resource_mut::<SystemMessageUiModel>()
        .set_focus_out(true);
    app.update();
    assert_foreign_modal_state(&app, true);
    app.world_mut()
        .resource_mut::<SystemMessageUiModel>()
        .set_focus_out(false);
    app.update();
    assert_foreign_modal_state(&app, false);

    app.world_mut().resource_mut::<ResurrectUiModel>().visible = true;
    app.update();
    assert_foreign_modal_state(&app, true);
    app.world_mut().resource_mut::<ResurrectUiModel>().visible = false;
    app.update();
    assert_foreign_modal_state(&app, false);

    let catalog = runtime_test_combi_catalog_0104();
    let inventory = enchant_test_inventory_0104(&[]);
    let player = CombiPlayerAuthority0104 {
        owner_pc_id: 77,
        gender: 1,
        level: 36,
        guide: 0,
        taros: 10_000,
    };
    let runtime_catalog = RuntimeCombiItemCatalog0104 {
        source: &catalog,
        player,
    };
    app.world_mut()
        .resource_mut::<CombiProductionRuntime0104>()
        .open(
            CombiOpenContext0104::clean(9_001, 6_501, player),
            &inventory,
            &runtime_catalog,
            &catalog.recipes,
        )
        .unwrap();
    app.update();
    assert_foreign_modal_state(&app, true);
    app.world_mut()
        .resource_mut::<CombiProductionRuntime0104>()
        .reset();
    app.update();
    assert_foreign_modal_state(&app, false);

    let inventory = enchant_test_inventory_0104(&[]);
    let inventory = email_inventory_authority_0104(&inventory);
    let catalog = runtime_test_email_catalog_0104();
    let mut email_model = EmailUiModel::default();
    let mut email_actions = EmailUiOutbox::default();
    let mut email_audio = EmailUiAudioOutbox::default();
    let email_network = EmailNetworkRuntime0104::default();
    let email_inbox = EmailNetworkInbox0104::default();
    let email_transport = EmailTransportOutbox::default();
    app.world_mut()
        .resource_mut::<EmailProductionRuntime0104>()
        .open(
            EmailOpenContext0104 {
                player: EmailPlayerAuthority0104 {
                    owner_pc_id: 77,
                    taros: 10_000,
                    current_local_time: EmailSystemTime::default(),
                },
                cursor_was_locked: true,
                item_policy: EmailItemFeaturePolicy0104 {
                    combine_enabled: true,
                    korean_enchant_enabled: false,
                },
            },
            Vec::new(),
            Vec::new(),
            &inventory,
            &catalog,
            &mut email_model,
            &mut email_actions,
            &mut email_audio,
            &email_network,
            &email_inbox,
            &email_transport,
        )
        .unwrap();
    app.update();
    assert_foreign_modal_state(&app, true);
    *app.world_mut().resource_mut::<EmailProductionRuntime0104>() =
        EmailProductionRuntime0104::default();
    app.update();
    assert_foreign_modal_state(&app, false);
}

#[test]
fn expanded_buddy_nanocom_keeps_the_gameplay_pointer_unlocked() {
    let mut nanocom = NanocomMessageUiModel::default();
    nanocom.enqueue_buddy_invite(1, "Dexter");
    nanocom.set_expanded(true);

    assert!(nanocom.expanded_visible());
    assert!(!legacy_gameplay_cursor_locked(
        ClientState::World,
        true,
        nanocom.expanded_visible(),
    ));
}

#[test]
fn direct_buddy_invites_share_one_fifo_with_nanocom_correlation() {
    let invite = |invite_id, requester_pc_id, requester_pc_uid, first_name: &str| BuddyInvite {
        invite_id,
        requester_pc_id,
        requester_pc_uid,
        first_name: first_name.to_owned(),
        last_name: "Hero".to_owned(),
        name_check_flag: 1,
    };
    let first = invite(1, 10, 100, "Dexter");
    let second = invite(2, 20, 200, "Ben");
    let mut buddy_ui = BuddyUiModel::default();
    let mut buddy_outbox = BuddyUiOutbox::default();
    let mut integration = BuddyRuntimeIntegration::default();
    let mut nanocom = NanocomMessageUiModel::default();

    queue_buddy_invite(
        first.clone(),
        &mut buddy_ui,
        &mut buddy_outbox,
        &mut integration,
        &mut nanocom,
    );
    queue_buddy_invite(
        second.clone(),
        &mut buddy_ui,
        &mut buddy_outbox,
        &mut integration,
        &mut nanocom,
    );

    assert!(buddy_outbox.is_empty());
    assert_eq!(buddy_ui.current_invite(), Some(&first));
    let request_ids = nanocom
        .queued()
        .iter()
        .map(|queued| queued.request.request_id)
        .collect::<Vec<_>>();
    assert_eq!(
        request_ids,
        vec![
            BUDDY_NANOCOM_MESSAGE_ID_BASE,
            BUDDY_NANOCOM_MESSAGE_ID_BASE + 1,
        ]
    );
    assert_eq!(
        integration.pending_nanocom_actions.get(&request_ids[0]),
        Some(&PendingBuddyNanocomAction::Invite { invite_id: 1 })
    );
    assert_eq!(
        integration.pending_nanocom_actions.get(&request_ids[1]),
        Some(&PendingBuddyNanocomAction::Invite { invite_id: 2 })
    );
    assert!(integration.pending_system_actions.is_empty());

    assert!(buddy_nanocom_resolution_accepted(
        NanocomMessageResolution::Accepted
    ));
    assert!(!buddy_nanocom_resolution_accepted(
        NanocomMessageResolution::Declined
    ));
    assert!(!buddy_nanocom_resolution_accepted(
        NanocomMessageResolution::TimedOut
    ));
    assert_eq!(
        buddy_ui.respond_to_invite(1, true),
        Ok(BuddyUiAction::InviteResponse {
            invite_id: 1,
            requester_pc_id: 10,
            requester_pc_uid: 100,
            accepted: true,
        })
    );
    assert_eq!(
        buddy_ui.respond_to_invite(2, false),
        Ok(BuddyUiAction::InviteResponse {
            invite_id: 2,
            requester_pc_id: 20,
            requester_pc_uid: 200,
            accepted: false,
        })
    );
}

#[test]
fn group_roster_projects_protocol_names_nano_and_npc_table_data() {
    let content = runtime_test_mission_content();
    let roster = GroupRoster0104 {
        context_id: 41,
        npc_id: None,
        pc_members: vec![ffone_protocol::GroupPcMemberInfo0104 {
            pc_id: 7,
            pc_uid: 700,
            name_check: 1,
            first_name: FixedUtf16::from_str("Remote").unwrap(),
            last_name: FixedUtf16::from_str("Player").unwrap(),
            special_state: 0,
            level: 12,
            hp: 750,
            max_hp: 1_000,
            map_type: 1,
            map_number: 14,
            position: [100, 200, 300],
            nano_active: 1,
            nano: ffone_protocol::Nano0104 {
                id: 1,
                skill_id: 1,
                stamina: 75,
            },
        }],
        npc_members: vec![ffone_protocol::GroupNpcMemberInfo0104 {
            npc_id: 99,
            npc_type: 664,
            hp: 220,
            map_type: 1,
            map_number: 14,
            position: [400, 500, 600],
        }],
    };

    let model = group_ui_model_from_roster(Some(900), roster, &content);

    assert_eq!(model.local_pc_uid, Some(900));
    assert_eq!(model.pc_members.len(), 1);
    let pc = &model.pc_members[0];
    assert_eq!(
        (pc.first_name.as_str(), pc.last_name.as_str()),
        ("Remote", "Player")
    );
    assert!(pc.free_chat);
    assert_eq!(pc.map_type, 1);
    assert_eq!(pc.map_number, 14);
    assert_eq!(pc.position, [100, 200, 300]);
    assert_eq!(
        pc.nano,
        Some(GroupNanoUi {
            name: "Buttercup".to_owned(),
            stamina: 75,
            max_stamina: 150,
            skill_icon_path: Some("ui/en/gameplay/nano/icons/skill/skillicon_10.png".to_owned()),
        })
    );
    assert_eq!(
        model.npc_members,
        vec![GroupNpcMemberUi {
            npc_type: 664,
            name: "Time Squad Officer James".to_owned(),
            hp: 220,
            max_hp: 439,
        }]
    );
}

#[test]
fn group_main_owner_queues_exact_type14_nanocom_and_table_system_message() {
    let content = runtime_test_mission_content();
    let mut integration = GroupRuntimeIntegration0104::default();
    let mut nanocom = NanocomMessageUiModel::default();
    let invitation = content
        .system_message_definition(GROUP_INVITATION_MESSAGE_ID_0104)
        .unwrap();
    let body =
        format_clean_group_message_0104(&invitation.exact_text, &["Remote Player".to_owned()]);
    let request_id = integration.push_invitation(&mut nanocom, 20, body.clone());
    let request = &nanocom.active().unwrap().request;
    assert_eq!(request.request_id, request_id);
    assert_eq!(request.kind, NanocomMessageKind::GroupInvite);
    assert_eq!(request.title, "");
    assert_eq!(request.body, body);
    assert_eq!(request.compact_frame_path, NANOCOM_BUDDY_FRAME_PATH);
    assert_eq!(
        request.compact_icon_path.as_deref(),
        Some(GROUP_NANOCOM_ICON_PATH_0104)
    );
    assert_eq!(
        integration.pending_nanocom_invites.get(&request_id),
        Some(&20)
    );

    let mut messages = SystemMessageUiModel::default();
    let failure = content.system_message_definition(59).unwrap();
    let system_request_id = integration.push_system_message(
        &mut messages,
        failure.exact_text.clone(),
        failure.runtime_button_type,
    );
    assert_eq!(messages.current().unwrap().text, failure.exact_text);
    assert!(
        integration
            .pending_system_messages
            .contains(&system_request_id)
    );
}

#[test]
fn buddy_freechat_routes_aggregate_personal_and_all_without_optimistic_echo() {
    let mut model = BuddyUiModel::default();
    model
        .set_entry(
            7,
            Some(BuddyEntry {
                runtime_pc_id: 82,
                pc_uid: 8_200,
                free_chat: true,
                presence: BuddyPresence::Online,
                first_name: "Remote".to_owned(),
                last_name: "Buddy".to_owned(),
                name_check_flag: 1,
                ..default()
            }),
        )
        .unwrap();
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_name: "Local Player".to_owned(),
            free_chat: true,
            ..default()
        },
        roster: RuntimeRosterStatus {
            selected_uid: Some(8_100),
            ..default()
        },
        ..default()
    };

    apply_buddy_freechat_success(
        BuddyFreeChatSuccess0104 {
            from_pc_uid: 8_200,
            to_pc_uid: 8_100,
            message: FixedUtf16::from_str("Meet here").unwrap(),
            emote_code: 0,
        },
        &model,
        &mut runtime,
        ChatChannel::All,
    );
    assert_eq!(
        runtime.chat.world_buddy_chat_lines,
        vec![buddy_freechat_line(false, "Remote Buddy", "Meet here")]
    );
    assert_eq!(
        runtime.chat.world_buddy_chat_by_uid[&8_200],
        vec![buddy_freechat_line(false, "Remote Buddy", "Meet here")]
    );
    assert_eq!(
        runtime.chat.world_chat_lines,
        vec![buddy_freechat_line(true, "Remote Buddy", "Meet here")]
    );
    assert!(runtime.chat.world_chat_alerts[ChatChannel::Buddy.index()]);
    assert_eq!(runtime.chat.pending_social_sfx, ["Incoming_Tell", "Incoming_Tell"]);

    runtime.chat.world_chat_alerts[ChatChannel::Buddy.index()] = false;
    apply_buddy_freechat_success(
        BuddyFreeChatSuccess0104 {
            from_pc_uid: 8_100,
            to_pc_uid: 8_200,
            message: FixedUtf16::from_str("On my way").unwrap(),
            emote_code: 0,
        },
        &model,
        &mut runtime,
        ChatChannel::All,
    );
    assert_eq!(runtime.chat.world_buddy_chat_lines.len(), 2);
    assert_eq!(runtime.chat.world_buddy_chat_by_uid[&8_200].len(), 2);
    assert_eq!(
        runtime.chat.world_chat_lines.last().unwrap().text,
        "[Buddy] Local Player: On my way"
    );
    assert!(!runtime.chat.world_chat_alerts[ChatChannel::Buddy.index()]);
    assert_eq!(runtime.chat.pending_social_sfx, ["Incoming_Tell", "Incoming_Tell", "Incoming_Tell", "Outgoing_Chat"]);

    model
        .set_blocked(
            BuddyTarget {
                slot: 7,
                pc_uid: 8_200,
            },
            true,
        )
        .unwrap();
    let retained = runtime.chat.world_buddy_chat_lines.clone();
    apply_buddy_freechat_success(
        BuddyFreeChatSuccess0104 {
            from_pc_uid: 8_200,
            to_pc_uid: 8_100,
            message: FixedUtf16::from_str("Blocked").unwrap(),
            emote_code: 0,
        },
        &model,
        &mut runtime,
        ChatChannel::Buddy,
    );
    assert_eq!(runtime.chat.world_buddy_chat_lines, retained);
    assert_eq!(runtime.chat.pending_social_sfx.len(), 4, "blocked buddy messages stay silent");
}

#[test]
fn normal_menuchat_echo_adds_colored_history_bubble_and_remote_emote() {
    let mut app = App::new();
    let remote = app
        .world_mut()
        .spawn((
            NetworkRemotePc0104 { pc_id: 82 },
            NetworkPcAppearance0104(ffone_protocol::PcAppearance0104 {
                id: 82,
                style: ffone_protocol::PcStyle0104 {
                    pc_uid: 820,
                    name_check: 1,
                    first_name: FixedUtf16::from_str("Remote").unwrap(),
                    last_name: FixedUtf16::from_str("Player").unwrap(),
                    gender: 1,
                    face_style: 1,
                    hair_style: 1,
                    hair_color: 1,
                    skin_color: 1,
                    eye_color: 1,
                    height: 1,
                    body: 1,
                    class: 0,
                },
                condition_bit_flag: 0,
                pc_state: 1,
                special_state: 0,
                level: 1,
                hp: 1000,
                map_number: 1,
                position: [0; 3],
                angle: 0,
                equipment: [ffone_protocol::ItemBase0104 {
                    item_type: 0,
                    item_id: 0,
                    option: 0,
                    time_limit: 0,
                }; 9],
                nano: ffone_protocol::Nano0104 {
                    id: 0,
                    skill_id: 0,
                    stamina: 0,
                },
                render_type: 1,
            }),
            GlobalTransform::IDENTITY,
            RemoteAnimation::default(),
        ))
        .id();
    let mut queries = bevy::ecs::system::SystemState::<(
        Query<&NetworkPcAppearance0104>,
        Query<(Entity, &NetworkRemotePc0104, &GlobalTransform)>,
        Query<(Entity, &LocalNetworkIdentity)>,
        Query<(&NetworkRemotePc0104, &mut RemoteAnimation)>,
    )>::new(app.world_mut());
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_id: Some(81),
            player_name: "Local Player".to_owned(),
            // MenuChat remains available when free typing is disabled.
            free_chat: false,
            ..default()
        },
        ..default()
    };
    let mut bubbles = PlayerFreeChatBubbleRuntime::default();
    let mut player_commands = TutorialPlayerPresentationCommandQueue::default();
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_SEND_MENUCHAT_MESSAGE_SUCC,
        flags: 0,
        checksum: 0,
        payload: FreeChatSuccess0104 {
            pc_id: 82,
            message: FixedUtf16::from_str("Let's Dance!").unwrap(),
            emote_code: 20,
        }
        .encode(),
    };
    {
        let (appearances, players, local_identities, mut animations) =
            queries.get_mut(app.world_mut()).unwrap();
        assert_eq!(
            apply_menu_chat_frame(
                &frame,
                &mut runtime,
                &appearances,
                &players,
                &local_identities,
                &mut animations,
                &mut player_commands,
                &LocalVehiclePresentationRuntime::default(),
                &BuddyUiModel::default(),
                &mut bubbles,
            ),
            Ok(true)
        );
    }
    queries.apply(app.world_mut());
    assert_eq!(
        app.world().get::<RemoteAnimation>(remote).unwrap().state,
        RemoteAnimationState::Emoting {
            clip: TutorialPlayerClip::Dance5,
        }
    );
    assert!(player_commands.is_empty());
    assert!(bubbles.has_pending_message(remote, "Let's Dance!"));
    assert_eq!(
        runtime.chat.world_chat_lines,
        vec![local_freechat_line("Remote Player", "Let's Dance!")]
    );
    assert_eq!(runtime.chat.world_chat_lines[0].kind, ChatLineKind::Normal);
}
