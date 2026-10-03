use super::*;

#[test]
fn group_and_buddy_menuchat_bypass_the_free_typing_gate() {
    let group_ui = GroupUiModel {
        local_pc_uid: Some(8_100),
        pc_members: vec![GroupPcMemberUi {
            pc_id: 82,
            pc_uid: 8_200,
            first_name: "Remote".to_owned(),
            last_name: "Group".to_owned(),
            ..default()
        }],
        ..default()
    };
    let mut buddy_ui = BuddyUiModel::default();
    buddy_ui
        .set_entry(
            7,
            Some(BuddyEntry {
                runtime_pc_id: 83,
                pc_uid: 8_300,
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
            free_chat: false,
            ..default()
        },
        roster: RuntimeRosterStatus {
            selected_uid: Some(8_100),
            ..default()
        },
        ..default()
    };

    let group_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_SEND_ALL_GROUP_MENUCHAT_MESSAGE_SUCC,
        flags: 0,
        checksum: 0,
        payload: AllGroupFreeChatSuccess0104 {
            sender_pc_id: 82,
            message: FixedUtf16::from_str("Regroup here").unwrap(),
            emote_code: 17,
        }
        .encode(),
    };
    assert_eq!(
        apply_all_group_menu_chat_frame(
            &group_frame,
            &mut runtime,
            &group_ui,
            &buddy_ui,
            ChatChannel::All,
        ),
        Ok(true)
    );
    assert_eq!(
        runtime.chat.world_group_chat_lines,
        vec![group_freechat_line(false, "Remote Group", "Regroup here")]
    );
    assert_eq!(
        runtime.chat.world_chat_lines,
        vec![group_freechat_line(true, "Remote Group", "Regroup here")]
    );
    assert!(runtime.chat.world_chat_alerts[ChatChannel::Group.index()]);

    let buddy_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_SEND_BUDDY_MENUCHAT_MESSAGE_SUCC,
        flags: 0,
        checksum: 0,
        payload: BuddyFreeChatSuccess0104 {
            from_pc_uid: 8_300,
            to_pc_uid: 8_100,
            message: FixedUtf16::from_str("Hello there").unwrap(),
            emote_code: 4,
        }
        .encode(),
    };
    assert_eq!(
        apply_buddy_menu_chat_frame(&buddy_frame, &buddy_ui, &mut runtime, ChatChannel::All),
        Ok(true)
    );
    assert_eq!(
        runtime.chat.world_buddy_chat_lines,
        vec![buddy_freechat_line(false, "Remote Buddy", "Hello there")]
    );
    assert_eq!(
        runtime.chat.world_buddy_chat_by_uid[&8_300],
        vec![buddy_freechat_line(false, "Remote Buddy", "Hello there")]
    );
    assert_eq!(
        runtime.chat.world_chat_lines.last(),
        Some(&buddy_freechat_line(true, "Remote Buddy", "Hello there"))
    );
    assert!(runtime.chat.world_chat_alerts[ChatChannel::Buddy.index()]);
}

#[test]
fn all_group_freechat_routes_group_and_prefixed_all_histories() {
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_id: Some(81),
            player_name: "Local Player".to_owned(),
            free_chat: true,
            ..default()
        },
        ..default()
    };
    apply_all_group_freechat_success(
        AllGroupFreeChatSuccess0104 {
            sender_pc_id: 82,
            message: FixedUtf16::from_str("Regroup here").unwrap(),
            emote_code: 0,
        },
        &mut runtime,
        Some("Remote Player".to_owned()),
        ChatChannel::All,
        false,
    );
    assert_eq!(
        runtime.chat.world_group_chat_lines,
        vec![group_freechat_line(false, "Remote Player", "Regroup here")]
    );
    assert_eq!(
        runtime.chat.world_chat_lines,
        vec![group_freechat_line(true, "Remote Player", "Regroup here")]
    );
    assert!(runtime.chat.world_buddy_chat_lines.is_empty());
    assert!(runtime.chat.world_chat_alerts[ChatChannel::Group.index()]);
    runtime.chat.world_chat_alerts[ChatChannel::Group.index()] = false;
    apply_all_group_freechat_success(
        AllGroupFreeChatSuccess0104 {
            sender_pc_id: 82,
            message: FixedUtf16::from_str("Visible group tab").unwrap(),
            emote_code: 0,
        },
        &mut runtime,
        Some("Remote Player".to_owned()),
        ChatChannel::Group,
        false,
    );
    assert!(!runtime.chat.world_chat_alerts[ChatChannel::Group.index()]);

    let retained_group = runtime.chat.world_group_chat_lines.clone();
    let retained_all = runtime.chat.world_chat_lines.clone();
    runtime.free_chat = false;
    apply_all_group_freechat_success(
        AllGroupFreeChatSuccess0104 {
            sender_pc_id: 82,
            message: FixedUtf16::from_str("blocked").unwrap(),
            emote_code: 0,
        },
        &mut runtime,
        Some("Remote Player".to_owned()),
        ChatChannel::All,
        false,
    );
    assert_eq!(runtime.chat.world_group_chat_lines, retained_group);
    assert_eq!(runtime.chat.world_chat_lines, retained_all);
}

#[test]
fn confirmed_first_nano_mission_edge_enqueues_type_10_nanocom_notice() {
    let content = runtime_test_mission_content();
    let mission = content
        .missions()
        .find(|mission| {
            let Some(source) = mission.start_nanocom_message.as_ref() else {
                return false;
            };
            mission.mission_type == TutorialMissionType::Nano
                && content
                    .is_first_serialized_task(mission.provenance.task_id)
                    .is_ok_and(|is_first| is_first)
                && content
                    .gameplay_npc(source.npc_type)
                    .is_some_and(|npc| !npc.move_voice_owner.is_empty())
                && i16::try_from(mission.provenance.nano_id)
                    .ok()
                    .and_then(|nano_id| content.gameplay_nano(nano_id))
                    .and_then(|nano| nano.icon_path.as_ref())
                    .is_some()
        })
        .expect("published first Nano task must own a type-10 NanoCom line");
    let source = mission.start_nanocom_message.as_ref().unwrap();
    let nano = content
        .gameplay_nano(i16::try_from(mission.provenance.nano_id).unwrap())
        .unwrap();
    let event =
        WorldMissionServerEvent0104::TaskStartSuccess(ffone_protocol::PcTaskStartSuccess0104 {
            task_id: mission.provenance.task_id,
            remaining_time: 0,
        });
    let mut messages = NanocomMessageUiModel::default();
    assert!(enqueue_world_mission_nanocom(
        &event,
        &content,
        &mut messages
    ));
    assert_eq!(messages.len(), 1);

    let active = &messages.active().unwrap().request;
    assert_eq!(active.kind, NanocomMessageKind::Nano);
    assert_eq!(
        active.compact_frame_path,
        ffone_client::nanocom_message_ui::NANOCOM_NANO_FRAME_PATH
    );
    assert_eq!(
        active.compact_title_localized().key,
        ffone_client::nanocom_message_ui::NANOCOM_NANO_MISSION_TITLE_LOCALIZATION_KEY
    );
    assert_eq!(
        active.compact_body_localized(10.0).key,
        format!(
            "content.tabledata.mission.mission_string.{}.str_name_string",
            source.string_id
        )
    );
    assert_eq!(active.compact_icon_path.as_ref(), nano.icon_path.as_ref());
    assert!(
        active.voice_true_name.is_some(),
        "type 10 must retain the source NPC CommOut owner"
    );
}

#[test]
fn mission_nanocom_survives_primary_npc_790_missing_portrait() {
    let content = runtime_test_mission_content();
    let mission = content.mission(1428).expect("primary task 1428");
    let source = mission
        .success_nanocom_message
        .as_ref()
        .expect("primary task 1428 success NanoCom line");
    assert_eq!(source.npc_type, 790);
    assert_eq!(
        content.gameplay_npc_portrait_icon_path(source.npc_type),
        None,
        "primary NPC 790 requests absent Icons/wpnicon_00"
    );

    let event = WorldMissionServerEvent0104::TaskEndSuccess(ffone_protocol::PcTaskEndSuccess0104 {
        task_id: mission.provenance.task_id,
    });
    let mut messages = NanocomMessageUiModel::default();
    assert!(enqueue_world_mission_nanocom(
        &event,
        &content,
        &mut messages
    ));
    let request = &messages.active().unwrap().request;
    assert_eq!(request.compact_icon_path, None);
    assert_eq!(
        request.compact_body_localized(10.0).key,
        format!(
            "content.tabledata.mission.mission_string.{}.str_name_string",
            source.string_id
        )
    );
}

#[test]
fn email_buddy_confirmation_requires_one_live_authoritative_name_match() {
    let mut buddies = BuddyUiModel::default();
    buddies
        .set_entry(
            7,
            Some(BuddyEntry {
                pc_uid: 8_200,
                first_name: "Remote".to_owned(),
                last_name: "Buddy".to_owned(),
                name_check_flag: 1,
                ..default()
            }),
        )
        .unwrap();
    assert_eq!(
        email_buddy_target_by_names_0104(&buddies, "Remote", "Buddy"),
        Ok(BuddyTarget {
            slot: 7,
            pc_uid: 8_200,
        })
    );
    assert!(
        email_buddy_target_by_names_0104(&buddies, "Missing", "Buddy")
            .unwrap_err()
            .contains("no authoritative")
    );

    buddies
        .set_entry(
            9,
            Some(BuddyEntry {
                pc_uid: 8_300,
                first_name: "Remote".to_owned(),
                last_name: "Buddy".to_owned(),
                name_check_flag: 1,
                ..default()
            }),
        )
        .unwrap();
    assert!(
        email_buddy_target_by_names_0104(&buddies, "Remote", "Buddy")
            .unwrap_err()
            .contains("duplicate")
    );
}
