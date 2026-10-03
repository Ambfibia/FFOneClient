use crate::group_runtime::*;

fn frame(packet_type: u32, payload: Vec<u8>) -> DecodedFrame {
    DecodedFrame {
        packet_type,
        flags: 0x1234,
        checksum: 0x5678,
        payload,
    }
}

fn remote(pc_id: i32) -> GroupRemotePlayer0104 {
    GroupRemotePlayer0104::new(pc_id, i64::from(pc_id) * 100, format!("Player {pc_id}"))
}

fn visible_authority(host: GroupRemotePlayer0104) -> GroupInboundAuthority0104 {
    GroupInboundAuthority0104 {
        allow_group_invites: true,
        host_blocked: false,
        visible_host: Some(host),
    }
}

fn write_utf16<const N: usize>(payload: &mut [u8], offset: usize, value: &str) {
    for (index, unit) in value.encode_utf16().take(N.saturating_sub(1)).enumerate() {
        payload[offset + index * 2..offset + index * 2 + 2]
            .copy_from_slice(&unit.to_le_bytes());
    }
}

fn roster_payload(context_id: i32, members: &[(i32, i64, &str, &str)]) -> Vec<u8> {
    let mut payload = vec![0; 12 + members.len() * GroupPcMemberInfo0104::SIZE];
    payload[0..4].copy_from_slice(&context_id.to_le_bytes());
    payload[4..8].copy_from_slice(&(members.len() as i32).to_le_bytes());
    for (index, (pc_id, pc_uid, first, last)) in members.iter().copied().enumerate() {
        let start = 12 + index * GroupPcMemberInfo0104::SIZE;
        payload[start..start + 4].copy_from_slice(&pc_id.to_le_bytes());
        payload[start + 4..start + 12].copy_from_slice(&pc_uid.to_le_bytes());
        payload[start + 12] = 1;
        write_utf16::<9>(&mut payload, start + 14, first);
        write_utf16::<17>(&mut payload, start + 32, last);
        payload[start + 68..start + 70].copy_from_slice(&10_i16.to_le_bytes());
        payload[start + 72..start + 76].copy_from_slice(&500_i32.to_le_bytes());
        payload[start + 76..start + 80].copy_from_slice(&1_000_i32.to_le_bytes());
    }
    payload
}

#[test]
fn registered_requests_use_exact_openfusion_ids_and_bodies() {
    let mut runtime = GroupProductionRuntime0104::default();
    runtime.begin_session(10, 1_000);

    let invite = runtime
        .request_invite(
            remote(20),
            GroupInviteOwner0104::PlayerMenu,
            GroupOwnerReachability0104::Proven,
        )
        .unwrap();
    assert_eq!(invite.packet_type(), P_CL2FE_REQ_PC_GROUP_INVITE_0104);
    assert_eq!(invite.payload(), 20_i32.to_le_bytes());
    runtime.release_transport_failure(&invite);

    let incoming = frame(P_FE2CL_PC_GROUP_INVITE_0104, 20_i32.to_le_bytes().to_vec());
    assert!(
        runtime
            .ingest(incoming, visible_authority(remote(20)))
            .owned()
    );
    let join = runtime
        .resolve_invitation(20, GroupInviteResolution0104::Accepted)
        .unwrap();
    assert_eq!(join.packet_type(), P_CL2FE_REQ_PC_GROUP_JOIN_0104);
    assert_eq!(join.payload(), 20_i32.to_le_bytes());
    runtime.release_transport_failure(&join);

    assert!(
        runtime
            .ingest(
                frame(P_FE2CL_PC_GROUP_INVITE_0104, 20_i32.to_le_bytes().to_vec()),
                visible_authority(remote(20)),
            )
            .owned()
    );
    let decline = runtime
        .resolve_invitation(20, GroupInviteResolution0104::TimedOut)
        .unwrap();
    assert_eq!(
        decline.packet_type(),
        P_CL2FE_REQ_PC_GROUP_INVITE_REFUSE_0104
    );
    assert_eq!(decline.payload(), 20_i32.to_le_bytes());

    let leave = runtime
        .request_leave(
            GroupLeaveOwner0104::BuddyWarp,
            GroupOwnerReachability0104::Proven,
        )
        .unwrap();
    assert_eq!(leave.packet_type(), P_CL2FE_REQ_PC_GROUP_LEAVE_0104);
    assert_eq!(leave.payload(), [0]);
}

#[test]
fn missing_production_owners_and_unsupported_mutations_fail_closed() {
    let mut runtime = GroupProductionRuntime0104::default();
    runtime.begin_session(10, 1_000);
    assert_eq!(
        runtime.request_invite(
            remote(20),
            GroupInviteOwner0104::PlayerMenu,
            GroupOwnerReachability0104::Unavailable,
        ),
        Err(GroupActionError0104::ProductionOwnerUnavailable {
            owner: "NpcIcon player menu"
        })
    );
    assert_eq!(
        runtime.request_leave(
            GroupLeaveOwner0104::MenuChat,
            GroupOwnerReachability0104::Unavailable,
        ),
        Err(GroupActionError0104::ProductionOwnerUnavailable {
            owner: "MenuChat leave confirmation"
        })
    );
    for action in [
        UnsupportedGroupAction0104::KickMember,
        UnsupportedGroupAction0104::TransferLeader,
        UnsupportedGroupAction0104::MutateBlockList,
    ] {
        assert_eq!(
            runtime.unsupported_action(action),
            Err(GroupActionError0104::UnsupportedAction(action))
        );
    }
}

#[test]
fn incoming_invite_follows_social_block_and_live_player_authority() {
    let invite = || frame(P_FE2CL_PC_GROUP_INVITE_0104, 20_i32.to_le_bytes().to_vec());
    let mut runtime = GroupProductionRuntime0104::default();
    runtime.begin_session(10, 1_000);

    let GroupFrameDisposition0104::Owned { output, .. } = runtime.ingest(
        invite(),
        GroupInboundAuthority0104 {
            allow_group_invites: false,
            ..Default::default()
        },
    ) else {
        panic!("social-off invitation must be owned and refused");
    };
    assert_eq!(output.requests.len(), 1);
    assert_eq!(
        output.requests[0].packet_type(),
        P_CL2FE_REQ_PC_GROUP_INVITE_REFUSE_0104
    );
    assert!(output.effects.is_empty());

    let GroupFrameDisposition0104::Owned { output, .. } = runtime.ingest(
        invite(),
        GroupInboundAuthority0104 {
            allow_group_invites: true,
            host_blocked: true,
            visible_host: Some(remote(20)),
        },
    ) else {
        panic!("blocked invitation must take the clean silent branch");
    };
    assert!(output.requests.is_empty());
    assert!(output.effects.is_empty());
    assert!(runtime.pending_invitation(20).is_none());

    let GroupFrameDisposition0104::Owned { output, .. } =
        runtime.ingest(invite(), visible_authority(remote(20)))
    else {
        panic!("visible invitation must be owned");
    };
    assert_eq!(
        output.effects,
        vec![GroupEffect0104::IncomingInvitation {
            host: remote(20),
            message_id: GROUP_INVITATION_MESSAGE_ID_0104,
        }]
    );
    assert!(runtime.pending_invitation(20).is_some());
}

#[test]
fn join_success_requires_pending_host_and_authoritative_roster_identity() {
    let mut runtime = GroupProductionRuntime0104::default();
    runtime.begin_session(10, 1_000);
    runtime.ingest(
        frame(P_FE2CL_PC_GROUP_INVITE_0104, 20_i32.to_le_bytes().to_vec()),
        visible_authority(remote(20)),
    );
    runtime
        .resolve_invitation(20, GroupInviteResolution0104::Accepted)
        .unwrap();

    let mismatch = frame(
        P_FE2CL_PC_GROUP_JOIN_SUCC_0104,
        roster_payload(
            10,
            &[
                (10, 1_000, "Local", "Player"),
                (30, 3_000, "Other", "Player"),
            ],
        ),
    );
    let original = mismatch.clone();
    assert_eq!(
        runtime.ingest(mismatch, GroupInboundAuthority0104::default()),
        GroupFrameDisposition0104::Passthrough(original)
    );
    assert!(runtime.roster().is_none());

    let success = frame(
        P_FE2CL_PC_GROUP_JOIN_SUCC_0104,
        roster_payload(
            10,
            &[
                (20, 2_000, "Host", "Player"),
                (10, 1_000, "Local", "Player"),
            ],
        ),
    );
    let GroupFrameDisposition0104::Owned { output, .. } =
        runtime.ingest(success, GroupInboundAuthority0104::default())
    else {
        panic!("correlated join success must be owned");
    };
    assert!(matches!(
        output.effects.as_slice(),
        [GroupEffect0104::RosterCommitted(roster)] if roster.pc_members.len() == 2
    ));
    assert_eq!(runtime.roster().unwrap().pc_members.len(), 2);
}

#[test]
fn fixed_replies_mutate_only_the_matching_single_pending_operation() {
    let mut runtime = GroupProductionRuntime0104::default();
    runtime.begin_session(10, 1_000);
    runtime
        .request_invite(
            remote(20),
            GroupInviteOwner0104::PlayerMenu,
            GroupOwnerReachability0104::Proven,
        )
        .unwrap();

    let wrong_refusal = frame(
        P_FE2CL_PC_GROUP_INVITE_REFUSE_0104,
        30_i32.to_le_bytes().to_vec(),
    );
    assert_eq!(
        runtime.ingest(wrong_refusal.clone(), GroupInboundAuthority0104::default()),
        GroupFrameDisposition0104::Passthrough(wrong_refusal)
    );
    let GroupFrameDisposition0104::Owned { output, .. } = runtime.ingest(
        frame(
            P_FE2CL_PC_GROUP_INVITE_REFUSE_0104,
            20_i32.to_le_bytes().to_vec(),
        ),
        GroupInboundAuthority0104::default(),
    ) else {
        panic!("matching refusal must be owned");
    };
    assert_eq!(
        output.effects,
        vec![
            GroupEffect0104::PassiveNotice {
                message_id: GROUP_INVITE_DECLINED_MESSAGE_ID_0104,
                arguments: vec!["Player 20".to_owned()],
            },
            GroupEffect0104::PlayActionFailure,
        ]
    );

    runtime.ingest(
        frame(P_FE2CL_PC_GROUP_INVITE_0104, 20_i32.to_le_bytes().to_vec()),
        visible_authority(remote(20)),
    );
    runtime
        .resolve_invitation(20, GroupInviteResolution0104::Accepted)
        .unwrap();
    let wrong_join_failure = frame(
        P_FE2CL_PC_GROUP_JOIN_FAIL_0104,
        [30_i32.to_le_bytes(), 7_i32.to_le_bytes()].concat(),
    );
    assert_eq!(
        runtime.ingest(
            wrong_join_failure.clone(),
            GroupInboundAuthority0104::default()
        ),
        GroupFrameDisposition0104::Passthrough(wrong_join_failure)
    );
    let GroupFrameDisposition0104::Owned { output, .. } = runtime.ingest(
        frame(P_FE2CL_PC_GROUP_JOIN_FAIL_0104, vec![0; 8]),
        GroupInboundAuthority0104::default(),
    ) else {
        panic!("OpenFusion's zeroed correlated join failure must be owned");
    };
    assert_eq!(
        output.effects,
        vec![
            GroupEffect0104::SystemMessage {
                message_id: GROUP_JOIN_FAILURE_MESSAGE_ID_0104,
            },
            GroupEffect0104::PlayActionFailure,
        ]
    );

    runtime
        .request_leave(
            GroupLeaveOwner0104::BuddyWarp,
            GroupOwnerReachability0104::Proven,
        )
        .unwrap();
    let wrong_leave_failure = frame(
        P_FE2CL_PC_GROUP_LEAVE_FAIL_0104,
        [30_i32.to_le_bytes(), 8_i32.to_le_bytes()].concat(),
    );
    assert_eq!(
        runtime.ingest(
            wrong_leave_failure.clone(),
            GroupInboundAuthority0104::default()
        ),
        GroupFrameDisposition0104::Passthrough(wrong_leave_failure)
    );
    assert_eq!(
        runtime.ingest(
            frame(P_FE2CL_PC_GROUP_LEAVE_FAIL_0104, vec![0; 8]),
            GroupInboundAuthority0104::default(),
        ),
        GroupFrameDisposition0104::Owned {
            frame: frame(P_FE2CL_PC_GROUP_LEAVE_FAIL_0104, vec![0; 8]),
            output: GroupFrameOutput0104 {
                requests: Vec::new(),
                effects: vec![GroupEffect0104::PlayActionFailure],
            },
        }
    );
    assert_eq!(runtime.pending_leave_owner(), None);
}

#[test]
fn leave_broadcast_uses_previous_roster_name_and_success_honors_buddy_warp_suppression() {
    let mut runtime = GroupProductionRuntime0104::default();
    runtime.begin_session(10, 1_000);
    runtime.roster = Some(
        match decode_group_packet_0104(&frame(
            P_FE2CL_PC_GROUP_MEMBER_INFO_0104,
            roster_payload(
                10,
                &[
                    (10, 1_000, "Local", "Player"),
                    (20, 2_000, "Host", "Player"),
                ],
            ),
        ))
        .unwrap()
        .unwrap()
        {
            GroupPacket0104::Roster(roster) => roster,
            GroupPacket0104::LeaveSuccess => unreachable!(),
        },
    );

    let leave = frame(
        P_FE2CL_PC_GROUP_LEAVE_0104,
        roster_payload(20, &[(10, 1_000, "Local", "Player")]),
    );
    let GroupFrameDisposition0104::Owned { output, .. } =
        runtime.ingest(leave, GroupInboundAuthority0104::default())
    else {
        panic!("authoritative leave broadcast must be owned");
    };
    assert!(output.effects.contains(&GroupEffect0104::PassiveNotice {
        message_id: GROUP_MEMBER_LEFT_MESSAGE_ID_0104,
        arguments: vec!["Host Player".to_owned()],
    }));

    runtime
        .request_leave(
            GroupLeaveOwner0104::BuddyWarp,
            GroupOwnerReachability0104::Proven,
        )
        .unwrap();
    let GroupFrameDisposition0104::Owned { output, .. } = runtime.ingest(
        frame(P_FE2CL_PC_GROUP_LEAVE_SUCC_0104, vec![0]),
        GroupInboundAuthority0104::default(),
    ) else {
        panic!("leave success must be authoritative");
    };
    assert_eq!(output.effects, vec![GroupEffect0104::RosterCleared]);

    let GroupFrameDisposition0104::Owned { output, .. } = runtime.ingest(
        frame(P_FE2CL_PC_GROUP_LEAVE_SUCC_0104, vec![0]),
        GroupInboundAuthority0104::default(),
    ) else {
        panic!("unsolicited disband success must remain authoritative");
    };
    assert_eq!(
        output.effects,
        vec![
            GroupEffect0104::RosterCleared,
            GroupEffect0104::PassiveNotice {
                message_id: GROUP_LOCAL_LEFT_MESSAGE_ID_0104,
                arguments: Vec::new(),
            },
        ]
    );
}

#[test]
fn uncorrelated_and_malformed_frames_are_lossless_and_do_not_mutate_state() {
    let mut runtime = GroupProductionRuntime0104::default();
    runtime.begin_session(10, 1_000);
    let before = runtime.clone();

    let unknown = frame(0x3100_fefe, vec![9, 8, 7, 6]);
    assert_eq!(
        runtime.ingest(unknown.clone(), GroupInboundAuthority0104::default()),
        GroupFrameDisposition0104::Passthrough(unknown)
    );
    assert_eq!(runtime, before);

    let uncorrelated = frame(P_FE2CL_PC_GROUP_INVITE_FAIL_0104, vec![0; 4]);
    assert_eq!(
        runtime.ingest(uncorrelated.clone(), GroupInboundAuthority0104::default()),
        GroupFrameDisposition0104::Passthrough(uncorrelated)
    );
    assert_eq!(runtime, before);

    let malformed = frame(P_FE2CL_PC_GROUP_INVITE_0104, vec![1, 2, 3]);
    let GroupFrameDisposition0104::Malformed { frame, error } =
        runtime.ingest(malformed.clone(), GroupInboundAuthority0104::default())
    else {
        panic!("known malformed group frame must be retained");
    };
    assert_eq!(frame, malformed);
    assert_eq!(
        error,
        GroupFrameDecodeError0104::Fixed(PayloadError::WrongSize {
            expected: 4,
            actual: 3,
        })
    );
    assert_eq!(runtime, before);
}
