use super::*;

#[test]
fn group_runtime_malformed_frame_preserves_hud_and_leave_effect_clears_it() {
    let content = runtime_test_mission_content();
    let mut production = GroupProductionRuntime0104::default();
    production.begin_session(7, 900);
    let mut model = GroupUiModel {
        local_pc_uid: Some(900),
        pc_members: vec![GroupPcMemberUi {
            pc_uid: 700,
            first_name: "Remote".to_owned(),
            ..default()
        }],
        npc_members: vec![GroupNpcMemberUi {
            npc_type: 0,
            name: "Escort".to_owned(),
            hp: 1,
            max_hp: 2,
        }],
    };
    let malformed = DecodedFrame {
        packet_type: packet::P_FE2CL_PC_GROUP_MEMBER_INFO,
        flags: 0,
        checksum: 0,
        payload: Vec::new(),
    };
    let last_good = model.clone();
    assert!(matches!(
        production.ingest(malformed, GroupInboundAuthority0104::default()),
        GroupFrameDisposition0104::Malformed { .. }
    ));
    assert_eq!(model, last_good);

    let leave = DecodedFrame {
        packet_type: packet::P_FE2CL_PC_GROUP_LEAVE_SUCC,
        flags: 0,
        checksum: 0,
        payload: vec![0],
    };
    let GroupFrameDisposition0104::Owned { output, .. } =
        production.ingest(leave, GroupInboundAuthority0104::default())
    else {
        panic!("authoritative group leave success must be owned");
    };
    assert!(output.effects.contains(&GroupEffect0104::RosterCleared));
    if output.effects.contains(&GroupEffect0104::RosterCleared) {
        let local_pc_uid = model.local_pc_uid.or(production.local_pc_uid());
        model = GroupUiModel {
            local_pc_uid,
            ..default()
        };
    }
    assert_eq!(
        model,
        GroupUiModel {
            local_pc_uid: Some(900),
            ..default()
        }
    );
    assert!(content.system_message_definition(147).is_some());
}

#[test]
fn accepted_freechat_frame_targets_the_exact_remote_player_bubble_owner() {
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
        ))
        .id();
    let mut queries = bevy::ecs::system::SystemState::<(
        Query<&NetworkPcAppearance0104>,
        Query<(Entity, &NetworkRemotePc0104, &GlobalTransform)>,
        Query<(Entity, &LocalNetworkIdentity)>,
    )>::new(app.world_mut());
    let (appearances, players, local_identities) = queries.get(app.world()).unwrap();
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_id: Some(81),
            player_name: "Local Player".to_owned(),
            free_chat: true,
            ..default()
        },
        ..default()
    };
    let mut bubbles = PlayerFreeChatBubbleRuntime::default();
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_FREECHAT_SUCC,
        flags: 0,
        checksum: 0,
        payload: FreeChatSuccess0104 {
            pc_id: 82,
            message: FixedUtf16::from_str("Bubble me").unwrap(),
            emote_code: 0,
        }
        .encode(),
    };

    assert_eq!(
        apply_freechat_frame(
            &frame,
            &mut runtime,
            &appearances,
            &players,
            &local_identities,
            &BuddyUiModel::default(),
            &mut bubbles,
        ),
        Ok(true)
    );
    assert_eq!(bubbles.pending_len(), 1);
    assert!(bubbles.has_pending_message(remote, "Bubble me"));
    assert_eq!(
        runtime.chat.world_chat_lines,
        vec![local_freechat_line("Remote Player", "Bubble me")]
    );
}

#[test]
fn all_group_freechat_frame_accepts_only_unblocked_roster_sender_and_uid_fallback() {
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_id: Some(81),
            player_name: "Local Player".to_owned(),
            free_chat: true,
            ..default()
        },
        ..default()
    };
    let group_ui = GroupUiModel {
        local_pc_uid: Some(810),
        pc_members: vec![GroupPcMemberUi {
            pc_id: 82,
            pc_uid: 8_200,
            ..default()
        }],
        ..default()
    };
    let mut buddy_ui = BuddyUiModel::default();
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_SEND_ALL_GROUP_FREECHAT_MESSAGE_SUCC,
        flags: 3,
        checksum: 4,
        payload: AllGroupFreeChatSuccess0104 {
            sender_pc_id: 82,
            message: FixedUtf16::from_str("Roster only").unwrap(),
            emote_code: 99,
        }
        .encode(),
    };
    assert_eq!(
        apply_all_group_freechat_frame(
            &frame,
            &mut runtime,
            &group_ui,
            &buddy_ui,
            ChatChannel::All,
        ),
        Ok(true)
    );
    assert_eq!(
        runtime.chat.world_group_chat_lines,
        vec![group_freechat_line(false, "Player 8200", "Roster only")]
    );

    buddy_ui
        .set_entry(
            4,
            Some(BuddyEntry {
                runtime_pc_id: 82,
                pc_uid: 8_200,
                blocked: true,
                ..default()
            }),
        )
        .unwrap();
    let before = runtime.chat.world_chat_lines.clone();
    assert_eq!(
        apply_all_group_freechat_frame(
            &frame,
            &mut runtime,
            &group_ui,
            &buddy_ui,
            ChatChannel::All,
        ),
        Ok(true)
    );
    assert_eq!(runtime.chat.world_chat_lines, before);

    let unknown = DecodedFrame {
        payload: AllGroupFreeChatSuccess0104 {
            sender_pc_id: 83,
            message: FixedUtf16::from_str("Nearby but not grouped").unwrap(),
            emote_code: 0,
        }
        .encode(),
        ..frame
    };
    assert_eq!(
        apply_all_group_freechat_frame(
            &unknown,
            &mut runtime,
            &group_ui,
            &buddy_ui,
            ChatChannel::All,
        ),
        Ok(true)
    );
    assert_eq!(runtime.chat.world_chat_lines, before);
}
