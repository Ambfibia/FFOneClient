use super::*;

#[test]
fn unchanged_buddy_projection_does_not_dirty_hidden_option_state() {
    let mut app = App::new();
    app.init_resource::<BuddyUiModel>()
        .init_resource::<OptionUiModel>()
        .add_systems(Update, sync_option_buddy_projection);

    app.update();
    app.world_mut().clear_trackers();
    app.update();

    assert!(!app.world().resource_ref::<OptionUiModel>().is_changed());
}

#[test]
fn ordinary_nanocom_help_opens_without_mentor_or_npc_state() {
    let mut model = GameGuideUiModel::default();
    assert!(!model.modal_active());
    model.open_from_nanocom();
    assert!(model.modal_active());
    assert_eq!(model.selected_main_topic, 1);
    assert_eq!(model.selected_sub_topic, 0);
    assert_eq!(model.content_scroll_y, 0.0);
}

#[test]
fn nanocom_quick_chat_and_emote_routes_own_the_complete_clean_selection_models() {
    let mut menu = QuickChatMenuUi::default();
    menu.toggle(QuickChatMenuMode::MenuChat);
    assert_eq!(menu.visible_items(0).len(), 14);
    let combat = *menu
        .visible_items(0)
        .into_iter()
        .find(|item| item.id == 7)
        .expect("Combat container");
    assert!(menu.select(combat).is_none());
    assert_eq!(menu.visible_items(1).len(), 16);

    menu.toggle(QuickChatMenuMode::Emotes);
    assert_eq!(menu.visible_items(0).len(), 19);
    let dance = *menu
        .visible_items(0)
        .into_iter()
        .find(|item| item.id == 16)
        .expect("Dance container");
    assert!(menu.select(dance).is_none());
    assert_eq!(
        menu.visible_items(1)
            .into_iter()
            .map(|item| item.emote_code)
            .collect::<Vec<_>>(),
        vec![6, 17, 18, 19, 20]
    );
    // Clean ButtonLock keeps the selected source menu open on a second click.
    menu.toggle(QuickChatMenuMode::Emotes);
    assert_eq!(menu.mode, QuickChatMenuMode::Emotes);
    assert_eq!(menu.open_level0, Some(16));

    menu.close();
    menu.toggle(QuickChatMenuMode::Emotes);
    let retrobution = *menu
        .source_items()
        .iter()
        .find(|item| item.id == 18)
        .unwrap();
    assert!(menu.select(retrobution).is_none());
    assert_eq!(
        menu.visible_items(1)
            .iter()
            .map(|item| item.emote_code)
            .collect::<Vec<_>>(),
        (31..=44).collect::<Vec<_>>()
    );
}

#[test]
fn buddy_list_identity_and_state_updates_preserve_exact_slots() {
    let mut model = BuddyUiModel::default();
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_id: Some(81),
            ..default()
        },
        roster: RuntimeRosterStatus {
            selected_uid: Some(8_100),
            ..default()
        },
        ..default()
    };
    let list = ffone_protocol::BuddyListInfo0104 {
        pc_id: 81,
        pc_uid: 8_100,
        list_num: 2,
        pack_padding: [0; 2],
        buddies: vec![
            BuddyBaseInfo0104 {
                pc_id: 82,
                pc_uid: 8_200,
                blocked: 0,
                free_chat: 1,
                pc_state: 1,
                first_name: FixedUtf16::from_str("Remote").unwrap(),
                last_name: FixedUtf16::from_str("Buddy").unwrap(),
                gender: 1,
                name_check_flag: 1,
            },
            BuddyBaseInfo0104 {
                pc_id: 83,
                pc_uid: 8_300,
                blocked: 0,
                free_chat: 1,
                pc_state: 0,
                first_name: FixedUtf16::from_str("Ignored").unwrap(),
                last_name: FixedUtf16::from_str("Name").unwrap(),
                gender: 0,
                name_check_flag: 0,
            },
        ],
    };
    apply_buddy_list_info(list, &mut model, &mut runtime, ChatChannel::All).unwrap();
    assert_eq!(model.occupied_count(), 2);
    assert_eq!(model.slot(2).unwrap().display_name(), "Remote Buddy");
    assert_eq!(model.slot(3).unwrap().display_name(), "Player 8300");
    assert!(runtime.chat.world_buddy_chat_by_uid.contains_key(&8_200));
    assert!(runtime.chat.world_buddy_chat_by_uid.contains_key(&8_300));

    let retained = model.slot(2).unwrap().clone();
    let mismatched = ffone_protocol::BuddyListInfo0104 {
        pc_id: 81,
        pc_uid: 9_999,
        list_num: 2,
        pack_padding: [0; 2],
        buddies: vec![],
    };
    assert!(apply_buddy_list_info(mismatched, &mut model, &mut runtime, ChatChannel::All).is_err());
    assert_eq!(model.slot(2), Some(&retained));

    let mut state = BuddyStateSuccess0104 {
        buddy_ids: [0; BUDDY_MAX_SLOTS],
        buddy_states: [0; BUDDY_MAX_SLOTS],
    };
    state.buddy_ids[2] = 82;
    apply_buddy_state_success(state, &mut model, &mut runtime, ChatChannel::All).unwrap();
    assert_eq!(model.slot(2).unwrap().runtime_pc_id, 82);
    assert_eq!(model.slot(2).unwrap().presence, BuddyPresence::Offline);
    assert_eq!(
        runtime.chat.world_buddy_chat_lines.last().unwrap().text,
        "Remote Buddy is now offline."
    );
    assert!(runtime.chat.world_chat_alerts[ChatChannel::Buddy.index()]);
}

#[test]
fn freechat_receive_uses_known_unblocked_names_special_state_gate_and_fifty_line_cap() {
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_id: Some(81),
            player_name: "Local Player".to_owned(),
            free_chat: true,
            ..default()
        },
        ..default()
    };
    apply_freechat_success(
        FreeChatSuccess0104 {
            pc_id: 81,
            message: FixedUtf16::from_str("Привет 🌌").unwrap(),
            emote_code: 0,
        },
        &mut runtime,
        None,
        false,
    );
    apply_freechat_success(
        FreeChatSuccess0104 {
            pc_id: 82,
            message: FixedUtf16::from_str("Remote line").unwrap(),
            emote_code: 0,
        },
        &mut runtime,
        Some("Remote Player".to_owned()),
        false,
    );
    apply_freechat_success(
        FreeChatSuccess0104 {
            pc_id: 83,
            message: FixedUtf16::from_str("Unknown sender").unwrap(),
            emote_code: 0,
        },
        &mut runtime,
        None,
        false,
    );
    assert_eq!(
        runtime
            .chat
            .world_chat_lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>(),
        vec![
            "[Local] Local Player: Привет 🌌",
            "[Local] Remote Player: Remote line"
        ]
    );

    let blocked_retained = runtime.chat.world_chat_lines.clone();
    apply_freechat_success(
        FreeChatSuccess0104 {
            pc_id: 82,
            message: FixedUtf16::from_str("Blocked Buddy").unwrap(),
            emote_code: 0,
        },
        &mut runtime,
        Some("Remote Player".to_owned()),
        true,
    );
    assert_eq!(runtime.chat.world_chat_lines, blocked_retained);

    for index in 0..TUTORIAL_CHAT_HISTORY_LIMIT {
        push_world_chat_line(&mut runtime, world_chat_line(format!("line {index}")));
    }
    assert_eq!(
        runtime.chat.world_chat_lines.len(),
        TUTORIAL_CHAT_HISTORY_LIMIT
    );
    assert_eq!(
        runtime.chat.world_chat_lines.first().unwrap().text,
        "line 0"
    );
    assert_eq!(
        runtime.chat.world_chat_lines.last().unwrap().text,
        "line 49"
    );

    runtime.free_chat = false;
    let retained = runtime.chat.world_chat_lines.clone();
    apply_freechat_success(
        FreeChatSuccess0104 {
            pc_id: 81,
            message: FixedUtf16::from_str("blocked").unwrap(),
            emote_code: 0,
        },
        &mut runtime,
        None,
        false,
    );
    assert_eq!(runtime.chat.world_chat_lines, retained);

    let localized = local_freechat_line("Игрок", "Привет");
    let template = localized
        .localized
        .as_ref()
        .expect("Local prefix must remain key-first");
    assert_eq!(template.key, "ui.hud.chat.message.local");
    assert_eq!(template.args.get("sender").unwrap(), "Игрок");
    assert_eq!(template.args.get("message").unwrap(), "Привет");
    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let (localization, russian) = Localization::open(&asset_root, "ru").unwrap();
    assert_eq!(
        localization.text(&russian, template),
        "[Локальный] Игрок: Привет"
    );
}

#[test]
fn special_state_frames_retain_local_action_and_freechat_flags() {
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_id: Some(81),
            free_chat: true,
            ..default()
        },
        ..default()
    };
    let local_muted = DecodedFrame {
        packet_type: packet::P_FE2CL_PC_SPECIAL_STATE_CHANGE,
        flags: 0,
        checksum: 0,
        payload: ffone_protocol::SpecialStateChange0104 {
            pc_id: 81,
            requested_flag: 64,
            special_state: 64,
        }
        .encode(),
    };
    assert_eq!(
        apply_special_state_frame(&local_muted, &mut runtime),
        Ok(true)
    );
    assert!(!runtime.free_chat);
    assert_eq!(runtime.special_state, 64);

    let remote_unmuted = DecodedFrame {
        payload: ffone_protocol::SpecialStateChange0104 {
            pc_id: 82,
            requested_flag: 64,
            special_state: 0,
        }
        .encode(),
        ..local_muted
    };
    assert_eq!(
        apply_special_state_frame(&remote_unmuted, &mut runtime),
        Ok(true)
    );
    assert!(!runtime.free_chat);
    assert_eq!(runtime.special_state, 64);

    let local_action_lock = DecodedFrame {
        payload: ffone_protocol::SpecialStateChange0104 {
            pc_id: 81,
            requested_flag: 4,
            special_state: 4,
        }
        .encode(),
        ..local_muted
    };
    assert_eq!(
        apply_special_state_frame(&local_action_lock, &mut runtime),
        Ok(true)
    );
    assert!(runtime.free_chat);
    assert_eq!(runtime.special_state, 4);

    let malformed = DecodedFrame {
        payload: vec![0; 7],
        ..local_action_lock
    };
    assert!(apply_special_state_frame(&malformed, &mut runtime).is_err());
    assert!(runtime.free_chat);
    assert_eq!(runtime.special_state, 4);
}
