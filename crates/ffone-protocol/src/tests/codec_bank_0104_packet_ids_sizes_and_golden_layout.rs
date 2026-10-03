use super::*;

pub(super) const OPENFUSION_CHAR_SELECT_FRAME: [u8; 16] = [
    0x0c, 0x00, 0x00, 0x00, 0x73, 0x7e, 0xe6, 0x60, 0x5c, 0x79, 0x27, 0x54, 0x66, 0x3d, 0x26,
    0x67,
];

#[test]
fn packet_ids_and_sizes_match_openfusion_0104() {
    assert_eq!(packet::P_CL2LS_REQ_LOGIN, 301_989_889);
    assert_eq!(packet::P_CL2LS_REQ_CHAR_SELECT, 301_989_893);
    assert_eq!(packet::P_CL2LS_REQ_SAVE_CHAR_TUTOR, 301_989_898);
    assert_eq!(packet::P_CL2FE_REQ_PC_ENTER, 318_767_105);
    assert_eq!(packet::P_CL2FE_REQ_PC_MOVE, 318_767_107);
    assert_eq!(packet::P_CL2FE_REQ_PC_STOP, 318_767_108);
    assert_eq!(packet::P_CL2FE_REQ_PC_JUMP, 318_767_109);
    assert_eq!(packet::P_CL2FE_REQ_PC_LOADING_COMPLETE, 318_767_245);
    assert_eq!(packet::P_LS2CL_REP_LOGIN_SUCC, 553_648_129);
    assert_eq!(packet::P_LS2CL_REP_SHARD_SELECT_SUCC, 553_648_143);
    assert_eq!(packet::P_FE2CL_REP_PC_ENTER_SUCC, 822_083_586);
    assert_eq!(packet::P_FE2CL_PC_AROUND, 822_083_591);
    assert_eq!(packet::P_FE2CL_PC_MOVE, 822_083_592);
    assert_eq!(packet::P_FE2CL_PC_STOP, 822_083_593);
    assert_eq!(packet::P_FE2CL_PC_JUMP, 822_083_594);
    assert_eq!(packet::P_FE2CL_NPC_AROUND, 822_083_599);
    assert_eq!(packet::P_FE2CL_TRANSPORTATION_ENTER, 822_083_739);
    assert_eq!(packet::P_FE2CL_TRANSPORTATION_EXIT, 822_083_740);
    assert_eq!(packet::P_FE2CL_TRANSPORTATION_MOVE, 822_083_741);
    assert_eq!(packet::P_FE2CL_TRANSPORTATION_NEW, 822_083_742);
    assert_eq!(packet::P_FE2CL_TRANSPORTATION_AROUND, 822_083_743);
    assert_eq!(packet::P_FE2CL_AROUND_DEL_TRANSPORTATION, 822_083_744);
    assert_eq!(packet::P_FE2CL_SHINY_ENTER, 822_083_757);
    assert_eq!(packet::P_FE2CL_SHINY_EXIT, 822_083_758);
    assert_eq!(packet::P_FE2CL_SHINY_NEW, 822_083_759);
    assert_eq!(packet::P_FE2CL_SHINY_AROUND, 822_083_760);
    assert_eq!(packet::P_FE2CL_AROUND_DEL_SHINY, 822_083_761);
    assert_eq!(packet::P_FE2CL_REP_PC_LOADING_COMPLETE_SUCC, 822_083_833);
    assert_eq!(packet::P_FE2CL_REP_NANO_BOOK_SUBSET, 822_083_892);
    assert_eq!(fixed_payload_size(packet::P_CL2LS_REQ_LOGIN), Some(468));
    assert_eq!(
        fixed_payload_size(packet::P_CL2LS_REQ_SAVE_CHAR_TUTOR),
        Some(12)
    );
    assert_eq!(fixed_payload_size(packet::P_CL2FE_REQ_PC_MOVE), Some(44));
    assert_eq!(fixed_payload_size(packet::P_CL2FE_REQ_PC_STOP), Some(20));
    assert_eq!(fixed_payload_size(packet::P_CL2FE_REQ_PC_JUMP), Some(44));
    assert_eq!(fixed_payload_size(packet::P_FE2CL_PC_MOVE), Some(56));
    assert_eq!(fixed_payload_size(packet::P_FE2CL_PC_STOP), Some(32));
    assert_eq!(fixed_payload_size(packet::P_FE2CL_PC_JUMP), Some(56));
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_REP_PC_ENTER_SUCC),
        Some(2700)
    );
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_REP_NANO_BOOK_SUBSET),
        Some(76)
    );
}

#[test]
fn openfusion_frame_matches_golden_vector() {
    let payload = CharacterSelectRequest {
        pc_uid: 0x0102_0304_0506_0708,
    }
    .encode();
    let encoded =
        encode_server_frame(packet::P_CL2LS_REQ_CHAR_SELECT, &payload, DEFAULT_KEY).unwrap();
    assert_eq!(encoded, OPENFUSION_CHAR_SELECT_FRAME);

    let decoded = decode_server_frame(&encoded, DEFAULT_KEY).unwrap();
    assert_eq!(decoded.packet_type, packet::P_CL2LS_REQ_CHAR_SELECT);
    assert_eq!(decoded.flags, 0x0c24);
    assert_eq!(decoded.checksum, 0x0c24);
    assert_eq!(
        CharacterSelectRequest::decode(&decoded.payload)
            .unwrap()
            .pc_uid,
        0x0102_0304_0506_0708
    );
}

#[test]
fn tutorial_save_request_matches_openfusion_0104_pack4_golden_payload() {
    let request = CharacterTutorialSaveRequest0104 {
        pc_uid: 0x0102_0304_0506_0708,
        tutorial_flag: 1,
    };
    let encoded = request.encode();

    assert_eq!(CharacterTutorialSaveRequest0104::SIZE, 12);
    assert_eq!(
        encoded,
        [
            0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, 0x01, 0x00, 0x00, 0x00,
        ]
    );
    assert_eq!(
        CharacterTutorialSaveRequest0104::decode(&encoded),
        Ok(request)
    );
    assert_eq!(
        CharacterTutorialSaveRequest0104::decode(&encoded[..11]),
        Err(PayloadError::WrongSize {
            expected: 12,
            actual: 11,
        })
    );
}

#[test]
fn character_creation_packet_ids_and_c_struct_sizes_match_openfusion_0104() {
    assert_eq!(packet::P_CL2LS_REQ_CHECK_CHAR_NAME, 0x1200_0002);
    assert_eq!(packet::P_CL2LS_REQ_SAVE_CHAR_NAME, 0x1200_0003);
    assert_eq!(packet::P_CL2LS_REQ_CHAR_CREATE, 0x1200_0004);
    assert_eq!(packet::P_CL2LS_REQ_CHAR_DELETE, 0x1200_0006);
    assert_eq!(packet::P_LS2CL_REP_CHECK_CHAR_NAME_SUCC, 0x2100_0005);
    assert_eq!(packet::P_LS2CL_REP_CHECK_CHAR_NAME_FAIL, 0x2100_0006);
    assert_eq!(packet::P_LS2CL_REP_SAVE_CHAR_NAME_SUCC, 0x2100_0007);
    assert_eq!(packet::P_LS2CL_REP_SAVE_CHAR_NAME_FAIL, 0x2100_0008);
    assert_eq!(packet::P_LS2CL_REP_CHAR_CREATE_SUCC, 0x2100_0009);
    assert_eq!(packet::P_LS2CL_REP_CHAR_CREATE_FAIL, 0x2100_000a);
    assert_eq!(packet::P_LS2CL_REP_CHAR_DELETE_SUCC, 0x2100_000d);
    assert_eq!(packet::P_LS2CL_REP_CHAR_DELETE_FAIL, 0x2100_000e);

    assert_eq!(CharacterNameCheckRequest0104::SIZE, 64);
    assert_eq!(CharacterNameCheckSuccess0104::SIZE, 52);
    assert_eq!(CharacterNameCheckFailure0104::SIZE, 4);
    assert_eq!(CharacterNameSaveRequest0104::SIZE, 68);
    assert_eq!(CharacterNameSaveSuccess0104::SIZE, 64);
    assert_eq!(CharacterNameSaveFailure0104::SIZE, 4);
    assert_eq!(CharacterCreateRequest0104::SIZE, 100);
    assert_eq!(CharacterCreateSuccess0104::SIZE, 100);
    assert_eq!(CharacterCreateFailure0104::SIZE, 4);
    assert_eq!(CharacterDeleteRequest0104::SIZE, 8);
    assert_eq!(CharacterDeleteSuccess0104::SIZE, 1);
    assert_eq!(CharacterDeleteFailure0104::SIZE, 4);

    assert_eq!(
        fixed_payload_size(packet::P_CL2LS_REQ_CHECK_CHAR_NAME),
        Some(64)
    );
    assert_eq!(
        fixed_payload_size(packet::P_CL2LS_REQ_SAVE_CHAR_NAME),
        Some(68)
    );
    assert_eq!(
        fixed_payload_size(packet::P_CL2LS_REQ_CHAR_CREATE),
        Some(100)
    );
    assert_eq!(
        fixed_payload_size(packet::P_LS2CL_REP_CHAR_CREATE_SUCC),
        Some(100)
    );
    assert_eq!(fixed_payload_size(packet::P_CL2LS_REQ_CHAR_DELETE), Some(8));
    assert_eq!(
        fixed_payload_size(packet::P_LS2CL_REP_CHAR_DELETE_SUCC),
        Some(1)
    );
}

#[test]
fn pc_around_golden_payload_uses_232_byte_data_offset() {
    let mut payload = vec![0u8; PcAppearance0104::SIZE * 3];
    write_i32(&mut payload, 0, 2);

    let first = PcAppearance0104::SIZE;
    write_i32(&mut payload, first, 77);
    write_i64(&mut payload, first + 4, 0x0102_0304_0506_0708);
    payload[first + 12] = 1;
    write_utf16(
        &mut payload,
        first + 14,
        &FixedUtf16::<9>::from_str("Dexter").unwrap(),
    );
    write_utf16(
        &mut payload,
        first + 32,
        &FixedUtf16::<17>::from_str("Morgan").unwrap(),
    );
    payload[first + 66] = 1;
    payload[first + 67] = 2;
    payload[first + 68] = 3;
    payload[first + 69] = 4;
    payload[first + 70] = 5;
    payload[first + 71] = 6;
    payload[first + 72] = 7;
    payload[first + 73] = 8;
    write_i32(&mut payload, first + 76, 9);
    write_i32(&mut payload, first + 80, 0x1234);
    payload[first + 84] = 10;
    payload[first + 85] = 11;
    write_test_i16(&mut payload, first + 86, 36);
    write_i32(&mut payload, first + 88, 500);
    write_i32(&mut payload, first + 92, 14);
    write_i32(&mut payload, first + 96, 1000);
    write_i32(&mut payload, first + 100, 2000);
    write_i32(&mut payload, first + 104, 3000);
    write_i32(&mut payload, first + 108, 90);
    write_test_i16(&mut payload, first + 112, 4);
    write_test_i16(&mut payload, first + 114, 123);
    write_i32(&mut payload, first + 116, 456);
    write_i32(&mut payload, first + 120, 789);
    write_test_i16(&mut payload, first + 220, 7);
    write_test_i16(&mut payload, first + 222, 8);
    write_test_i16(&mut payload, first + 224, 9);
    write_i32(&mut payload, first + 228, 3);

    let second = first + PcAppearance0104::SIZE;
    write_i32(&mut payload, second, 88);
    write_i32(&mut payload, second + 96, -100);
    write_i32(&mut payload, second + 100, -200);
    write_i32(&mut payload, second + 104, -300);

    let players = decode_pc_around_0104(&payload).unwrap();
    assert_eq!(players.len(), 2);
    assert_eq!(players[0].id, 77);
    assert_eq!(players[0].style.pc_uid, 0x0102_0304_0506_0708);
    assert_eq!(players[0].style.first_name.to_string_lossy(), "Dexter");
    assert_eq!(players[0].style.last_name.to_string_lossy(), "Morgan");
    assert_eq!(players[0].style.class, 9);
    assert_eq!(players[0].level, 36);
    assert_eq!(players[0].position, [1000, 2000, 3000]);
    assert_eq!(players[0].equipment[0].item_type, 4);
    assert_eq!(players[0].equipment[0].item_id, 123);
    assert_eq!(players[0].equipment[0].option, 456);
    assert_eq!(players[0].equipment[0].time_limit, 789);
    assert_eq!(players[0].nano.id, 7);
    assert_eq!(players[0].nano.skill_id, 8);
    assert_eq!(players[0].nano.stamina, 9);
    assert_eq!(players[0].render_type, 3);
    assert_eq!(players[1].id, 88);
    assert_eq!(players[1].position, [-100, -200, -300]);
}

#[test]
fn npc_around_golden_payload_uses_36_byte_data_offset() {
    let mut payload = vec![0u8; NpcAppearance0104::SIZE * 3];
    write_i32(&mut payload, 0, 2);
    let first = NpcAppearance0104::SIZE;
    for (index, value) in [41, 1234, 900, 0x55, 10, 20, 30, 180, 7]
        .into_iter()
        .enumerate()
    {
        write_i32(&mut payload, first + index * 4, value);
    }
    write_i32(&mut payload, first + NpcAppearance0104::SIZE, 42);

    let npcs = decode_npc_around_0104(&payload).unwrap();
    assert_eq!(npcs.len(), 2);
    assert_eq!(npcs[0].npc_id, 41);
    assert_eq!(npcs[0].npc_type, 1234);
    assert_eq!(npcs[0].hp, 900);
    assert_eq!(npcs[0].condition_bit_flag, 0x55);
    assert_eq!(npcs[0].position, [10, 20, 30]);
    assert_eq!(npcs[0].angle, 180);
    assert_eq!(npcs[0].barker_type, 7);
    assert_eq!(npcs[1].npc_id, 42);
}

#[test]
fn transportation_around_golden_payload_uses_24_byte_data_offset() {
    let mut payload = vec![0u8; TransportationAppearance0104::SIZE * 2];
    write_i32(&mut payload, 0, 1);
    let entry = TransportationAppearance0104::SIZE;
    for (index, value) in [3, 501, 12, -10, -20, -30].into_iter().enumerate() {
        write_i32(&mut payload, entry + index * 4, value);
    }

    let vehicles = decode_transportation_around_0104(&payload).unwrap();
    assert_eq!(
        vehicles,
        [TransportationAppearance0104 {
            transportation_kind: 3,
            id: 501,
            transportation_type: 12,
            position: [-10, -20, -30],
        }]
    );
}

#[test]
fn shiny_around_golden_payload_uses_normal_four_byte_header() {
    let mut payload = vec![0u8; 4 + ShinyAppearance0104::SIZE * 2];
    write_i32(&mut payload, 0, 2);
    for (index, value) in [601, 2, 14, 111, 222, 333].into_iter().enumerate() {
        write_i32(&mut payload, 4 + index * 4, value);
    }
    write_i32(&mut payload, 4 + ShinyAppearance0104::SIZE, 602);

    let shinies = decode_shiny_around_0104(&payload).unwrap();
    assert_eq!(shinies.len(), 2);
    assert_eq!(shinies[0].shiny_id, 601);
    assert_eq!(shinies[0].shiny_type, 2);
    assert_eq!(shinies[0].map_number, 14);
    assert_eq!(shinies[0].position, [111, 222, 333]);
    assert_eq!(shinies[1].shiny_id, 602);
}

#[test]
fn npc_combat_packet_ids_and_pack4_sizes_match_openfusion_0104() {
    assert_eq!(packet::P_CL2FE_REQ_PC_ATTACK_NPCS, 0x1300_0006);
    assert_eq!(packet::P_CL2FE_REQ_PC_ROCKET_STYLE_FIRE, 0x1300_001c);
    assert_eq!(packet::P_CL2FE_REQ_PC_GRENADE_STYLE_FIRE, 0x1300_001f);
    assert_eq!(packet::P_CL2FE_REQ_PC_COMBAT_BEGIN, 0x1300_0033);
    assert_eq!(packet::P_CL2FE_REQ_PC_COMBAT_END, 0x1300_0034);
    assert_eq!(packet::P_FE2CL_PC_NEW, 0x3100_0003);
    assert_eq!(packet::P_FE2CL_PC_EXIT, 0x3100_0006);
    assert_eq!(packet::P_FE2CL_AROUND_DEL_PC, 0x3100_0010);
    assert_eq!(packet::P_FE2CL_NPC_ENTER, 0x3100_000b);
    assert_eq!(packet::P_FE2CL_NPC_EXIT, 0x3100_000c);
    assert_eq!(packet::P_FE2CL_NPC_MOVE, 0x3100_000d);
    assert_eq!(packet::P_FE2CL_NPC_NEW, 0x3100_000e);
    assert_eq!(packet::P_FE2CL_AROUND_DEL_NPC, 0x3100_0011);
    assert_eq!(packet::P_FE2CL_PC_ATTACK_NPCS_SUCC, 0x3100_0014);
    assert_eq!(packet::P_FE2CL_PC_ATTACK_NPCS, 0x3100_0015);
    assert_eq!(packet::P_FE2CL_NPC_ATTACK_PCS, 0x3100_0016);

    assert_eq!(NpcEnter0104::SIZE, 36);
    assert_eq!(NpcExit0104::SIZE, 4);
    assert_eq!(NpcMove0104::SIZE, 24);
    assert_eq!(NpcNew0104::SIZE, 36);
    assert_eq!(PcNew0104::SIZE, 232);
    assert_eq!(PcExit0104::SIZE, 8);
    assert_eq!(PcCombatStateRequest0104::SIZE, 4);
    assert_eq!(PcRocketStyleFireRequest0104::SIZE, 28);
    assert_eq!(PcGrenadeStyleFireRequest0104::SIZE, 16);
    assert_eq!(AttackResult0104::SIZE, 24);
    assert_eq!(fixed_payload_size(packet::P_FE2CL_NPC_MOVE), Some(24));
    assert_eq!(fixed_payload_size(packet::P_FE2CL_PC_NEW), Some(232));
    assert_eq!(fixed_payload_size(packet::P_FE2CL_PC_EXIT), Some(8));
    assert_eq!(
        fixed_payload_size(packet::P_CL2FE_REQ_PC_COMBAT_BEGIN),
        Some(4)
    );
    assert_eq!(fixed_payload_size(packet::P_CL2FE_REQ_PC_ATTACK_NPCS), None);
    assert_eq!(
        fixed_payload_size(packet::P_CL2FE_REQ_PC_ROCKET_STYLE_FIRE),
        Some(28)
    );
    assert_eq!(
        fixed_payload_size(packet::P_CL2FE_REQ_PC_GRENADE_STYLE_FIRE),
        Some(16)
    );
}

#[test]
fn menuchat_and_avatar_emote_0104_use_the_clean_packet_ids_and_layouts() {
    assert_eq!(packet::P_CL2FE_REQ_SEND_MENUCHAT_MESSAGE, 0x1300_0008);
    assert_eq!(packet::P_FE2CL_REP_SEND_MENUCHAT_MESSAGE_SUCC, 0x3100_0018);
    assert_eq!(packet::P_CL2FE_REQ_SEND_BUDDY_MENUCHAT_MESSAGE, 0x1300_0038);
    assert_eq!(
        packet::P_FE2CL_REP_SEND_BUDDY_MENUCHAT_MESSAGE_SUCC,
        0x3100_006b
    );
    assert_eq!(packet::P_CL2FE_REQ_PC_AVATAR_EMOTES_CHAT, 0x1300_0050);
    assert_eq!(packet::P_FE2CL_REP_PC_AVATAR_EMOTES_CHAT, 0x3100_0092);
    assert_eq!(
        packet::P_CL2FE_REQ_SEND_ALL_GROUP_MENUCHAT_MESSAGE,
        0x1300_0066
    );
    assert_eq!(
        packet::P_FE2CL_REP_SEND_ALL_GROUP_MENUCHAT_MESSAGE_SUCC,
        0x3100_00ba
    );
    for (packet_id, expected) in [
        (packet::P_CL2FE_REQ_SEND_MENUCHAT_MESSAGE, 260),
        (packet::P_FE2CL_REP_SEND_MENUCHAT_MESSAGE_SUCC, 264),
        (packet::P_CL2FE_REQ_SEND_BUDDY_MENUCHAT_MESSAGE, 272),
        (packet::P_FE2CL_REP_SEND_BUDDY_MENUCHAT_MESSAGE_SUCC, 276),
        (packet::P_CL2FE_REQ_PC_AVATAR_EMOTES_CHAT, 8),
        (packet::P_FE2CL_REP_PC_AVATAR_EMOTES_CHAT, 8),
        (packet::P_CL2FE_REQ_SEND_ALL_GROUP_MENUCHAT_MESSAGE, 260),
        (
            packet::P_FE2CL_REP_SEND_ALL_GROUP_MENUCHAT_MESSAGE_SUCC,
            264,
        ),
    ] {
        assert_eq!(fixed_payload_size(packet_id), Some(expected));
    }

    let message = FixedUtf16::<128>::from_str("Quick hello").unwrap();
    let request = MenuChatRequest0104 {
        message: message.clone(),
        emote_code: 4,
    };
    assert_eq!(
        decode_menu_chat_packet_0104(
            packet::P_CL2FE_REQ_SEND_MENUCHAT_MESSAGE,
            &request.encode(),
        ),
        Ok(Some(MenuChatPacket0104::Request(request)))
    );
    let success = MenuChatSuccess0104 {
        pc_id: 42,
        message: message.clone(),
        emote_code: 4,
    };
    assert_eq!(
        decode_menu_chat_packet_0104(
            packet::P_FE2CL_REP_SEND_MENUCHAT_MESSAGE_SUCC,
            &success.encode(),
        ),
        Ok(Some(MenuChatPacket0104::Success(success)))
    );

    let emote = AvatarEmoteChat0104 {
        pc_id: 42,
        emote_code: 20,
    };
    let bytes = emote.encode();
    assert_eq!(&bytes[..4], &42i32.to_le_bytes());
    assert_eq!(&bytes[4..], &20i32.to_le_bytes());
    assert_eq!(
        decode_avatar_emote_chat_0104(packet::P_FE2CL_REP_PC_AVATAR_EMOTES_CHAT, &bytes),
        Ok(Some(emote))
    );

    let buddy = BuddyMenuChatRequest0104 {
        message: message.clone(),
        emote_code: 0,
        buddy_pc_uid: 8_200,
        buddy_slot: 4,
    };
    assert_eq!(
        decode_buddy_menu_chat_packet_0104(
            packet::P_CL2FE_REQ_SEND_BUDDY_MENUCHAT_MESSAGE,
            &buddy.encode(),
        ),
        Ok(Some(BuddyMenuChatPacket0104::Request(buddy)))
    );
    let group = AllGroupMenuChatRequest0104 {
        message,
        emote_code: 0,
    };
    assert_eq!(
        decode_all_group_menu_chat_packet_0104(
            packet::P_CL2FE_REQ_SEND_ALL_GROUP_MENUCHAT_MESSAGE,
            &group.encode(),
        ),
        Ok(Some(AllGroupMenuChatPacket0104::Request(group)))
    );
}

#[test]
fn special_state_change_0104_has_exact_pack4_layout_and_strict_family_decode() {
    assert_eq!(
        packet::P_FE2CL_REP_PC_SPECIAL_STATE_SWITCH_SUCC,
        0x3100_00c3
    );
    assert_eq!(packet::P_FE2CL_PC_SPECIAL_STATE_CHANGE, 0x3100_00c4);
    assert_eq!(SpecialStateChange0104::SIZE, 8);
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_PC_SPECIAL_STATE_CHANGE),
        Some(8)
    );

    let change = SpecialStateChange0104 {
        pc_id: 0x0102_0304,
        requested_flag: 64,
        special_state: 80,
    };
    let bytes = change.encode();
    assert_eq!(
        bytes,
        vec![0x04, 0x03, 0x02, 0x01, 64, 80, 0, 0],
        "the final two bytes are Pack=4 tail padding"
    );
    assert_eq!(SpecialStateChange0104::decode(&bytes), Ok(change));
    assert_eq!(
        decode_special_state_change_0104(
            packet::P_FE2CL_REP_PC_SPECIAL_STATE_SWITCH_SUCC,
            &bytes,
        ),
        Ok(Some(change))
    );
    assert_eq!(
        decode_special_state_change_0104(0x3100_0abc, &[1, 2, 3]),
        Ok(None)
    );
    assert_eq!(
        decode_special_state_change_0104(packet::P_FE2CL_PC_SPECIAL_STATE_CHANGE, &[0; 7],),
        Err(PayloadError::WrongSize {
            expected: 8,
            actual: 7,
        })
    );
}

#[test]
fn quick_slot_0104_packet_ids_and_fixed_size_registry_match_source() {
    assert_eq!(packet::P_CL2FE_REQ_ITEM_USE, 0x1300_0073);
    assert_eq!(packet::P_CL2FE_REQ_PC_REGIST_QUICK_SLOT, 0x1300_00a1);
    assert_eq!(packet::P_FE2CL_REP_PC_ITEM_USE_FAIL, 0x3100_00d2);
    assert_eq!(packet::P_FE2CL_REP_PC_ITEM_USE_SUCC, 0x3100_00d3);
    assert_eq!(packet::P_FE2CL_PC_ITEM_USE, 0x3100_00d4);
    assert_eq!(packet::P_FE2CL_PC_QUICK_SLOT_INFO, 0x3100_0126);
    assert_eq!(packet::P_FE2CL_REP_PC_REGIST_QUICK_SLOT_FAIL, 0x3100_0127);
    assert_eq!(packet::P_FE2CL_REP_PC_REGIST_QUICK_SLOT_SUCC, 0x3100_0128);

    assert_eq!(
        fixed_payload_size(packet::P_CL2FE_REQ_ITEM_USE),
        Some(ItemUseRequest0104::SIZE)
    );
    assert_eq!(
        fixed_payload_size(packet::P_CL2FE_REQ_PC_REGIST_QUICK_SLOT),
        Some(QuickSlotRegisterRequest0104::SIZE)
    );
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_REP_PC_ITEM_USE_FAIL),
        Some(ItemUseFailure0104::SIZE)
    );
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_PC_QUICK_SLOT_INFO),
        Some(QuickSlotInfo0104::SIZE)
    );
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_REP_PC_REGIST_QUICK_SLOT_FAIL),
        Some(QuickSlotRegisterFailure0104::SIZE)
    );
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_REP_PC_REGIST_QUICK_SLOT_SUCC),
        Some(QuickSlotRegisterSuccess0104::SIZE)
    );
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_REP_PC_ITEM_USE_SUCC),
        None,
        "item-use success has a variable eST-dependent result tail"
    );
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_PC_ITEM_USE),
        None,
        "item-use broadcast has a variable eST-dependent result tail"
    );
}

#[test]
fn inventory_0104_packet_ids_sizes_and_golden_layouts_are_exact() {
    assert_eq!(packet::P_FE2CL_PC_ITEM_MOVE_SUCC, 822_083_610);
    assert_eq!(packet::P_FE2CL_PC_ITEM_MOVE_SUCC, 0x3100_001a);
    assert_eq!(packet::P_FE2CL_PC_EQUIP_CHANGE, 822_083_611);
    assert_eq!(packet::P_FE2CL_PC_EQUIP_CHANGE, 0x3100_001b);
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_PC_ITEM_MOVE_SUCC),
        Some(ItemMoveSuccessPacket0104::SIZE)
    );
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_PC_EQUIP_CHANGE),
        Some(EquipChangePacket0104::SIZE)
    );

    let move_success = ItemMoveSuccessPacket0104 {
        from_location: 1,
        from_slot_num: 49,
        from_slot_item: ItemBase0104 {
            item_type: 141,
            item_id: 0,
            option: -7,
            time_limit: 900,
        },
        to_location: 0,
        to_slot_num: 7,
        to_slot_item: ItemBase0104 {
            item_type: 0,
            item_id: 321,
            option: 0x1020_3040,
            time_limit: -55,
        },
    };
    let move_bytes = move_success.encode();
    assert_eq!(move_bytes.len(), 40);
    assert_eq!(&move_bytes[0..4], &1i32.to_le_bytes());
    assert_eq!(&move_bytes[4..8], &49i32.to_le_bytes());
    assert_eq!(&move_bytes[8..10], &141i16.to_le_bytes());
    assert_eq!(&move_bytes[10..12], &0i16.to_le_bytes());
    assert_eq!(&move_bytes[12..16], &(-7i32).to_le_bytes());
    assert_eq!(&move_bytes[16..20], &900i32.to_le_bytes());
    assert_eq!(&move_bytes[20..24], &0i32.to_le_bytes());
    assert_eq!(&move_bytes[24..28], &7i32.to_le_bytes());
    assert_eq!(&move_bytes[28..30], &0i16.to_le_bytes());
    assert_eq!(&move_bytes[30..32], &321i16.to_le_bytes());
    assert_eq!(&move_bytes[32..36], &0x1020_3040i32.to_le_bytes());
    assert_eq!(&move_bytes[36..40], &(-55i32).to_le_bytes());
    assert_eq!(
        ItemMoveSuccessPacket0104::decode(&move_bytes),
        Ok(move_success)
    );
    assert_eq!(
        decode_inventory_packet_0104(packet::P_FE2CL_PC_ITEM_MOVE_SUCC, &move_bytes,),
        Ok(Some(InventoryPacket0104::ItemMoveSuccess(move_success)))
    );

    let equip_change = EquipChangePacket0104 {
        pc_id: 77,
        equip_slot_num: 8,
        equip_slot_item: ItemBase0104 {
            item_type: 10,
            item_id: 404,
            option: -1,
            time_limit: 60_000,
        },
    };
    let equip_bytes = equip_change.encode();
    assert_eq!(equip_bytes.len(), 20);
    assert_eq!(&equip_bytes[0..4], &77i32.to_le_bytes());
    assert_eq!(&equip_bytes[4..8], &8i32.to_le_bytes());
    assert_eq!(&equip_bytes[8..10], &10i16.to_le_bytes());
    assert_eq!(&equip_bytes[10..12], &404i16.to_le_bytes());
    assert_eq!(&equip_bytes[12..16], &(-1i32).to_le_bytes());
    assert_eq!(&equip_bytes[16..20], &60_000i32.to_le_bytes());
    assert_eq!(
        EquipChangePacket0104::decode(&equip_bytes),
        Ok(equip_change)
    );
    assert_eq!(
        decode_inventory_packet_0104(packet::P_FE2CL_PC_EQUIP_CHANGE, &equip_bytes,),
        Ok(Some(InventoryPacket0104::EquipChange(equip_change)))
    );
}

#[test]
fn bank_0104_packet_ids_sizes_and_golden_layouts_are_exact() {
    assert_eq!(packet::P_CL2FE_REQ_ITEM_MOVE, 318_767_114);
    assert_eq!(packet::P_CL2FE_REQ_PC_BANK_OPEN, 318_767_150);
    assert_eq!(packet::P_CL2FE_REQ_PC_BANK_CLOSE, 318_767_151);
    assert_eq!(packet::P_FE2CL_REP_PC_BANK_OPEN_SUCC, 822_083_670);
    assert_eq!(packet::P_FE2CL_REP_PC_BANK_OPEN_FAIL, 822_083_671);
    assert_eq!(packet::P_FE2CL_REP_PC_BANK_CLOSE_SUCC, 822_083_672);
    assert_eq!(packet::P_FE2CL_REP_PC_BANK_CLOSE_FAIL, 822_083_673);

    let move_request = ItemMoveRequest0104 {
        from_location: 3,
        from_slot_num: 199,
        to_location: 1,
        to_slot_num: 49,
    };
    assert_eq!(
        move_request.encode(),
        [3i32, 199, 1, 49]
            .into_iter()
            .flat_map(i32::to_le_bytes)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ItemMoveRequest0104::decode(&move_request.encode()),
        Ok(move_request)
    );

    let open_request = PcBankOpenRequest0104 {
        pc_id: 0x1020_3040,
        npc_id: -77,
    };
    assert_eq!(
        open_request.encode(),
        [0x1020_3040i32, -77]
            .into_iter()
            .flat_map(i32::to_le_bytes)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        PcBankOpenRequest0104::decode(&open_request.encode()),
        Ok(open_request)
    );
    let close_request = PcBankCloseRequest0104 { pc_id: 777 };
    assert_eq!(
        PcBankCloseRequest0104::decode(&close_request.encode()),
        Ok(close_request)
    );

    let open_success = PcBankOpenSuccess0104 {
        bank_items: std::array::from_fn(|index| ItemBase0104 {
            item_type: if index == 199 { 7 } else { 0 },
            item_id: if index == 199 { 321 } else { 0 },
            option: index as i32,
            time_limit: -(index as i32),
        }),
        extra_bank: 1,
    };
    let open_bytes = open_success.encode();
    assert_eq!(open_bytes.len(), 2_404);
    assert_eq!(&open_bytes[0..4], &[0, 0, 0, 0]);
    assert_eq!(&open_bytes[2_388..2_390], &7i16.to_le_bytes());
    assert_eq!(&open_bytes[2_390..2_392], &321i16.to_le_bytes());
    assert_eq!(&open_bytes[2_392..2_396], &199i32.to_le_bytes());
    assert_eq!(&open_bytes[2_396..2_400], &(-199i32).to_le_bytes());
    assert_eq!(&open_bytes[2_400..2_404], &1i32.to_le_bytes());
    assert_eq!(PcBankOpenSuccess0104::decode(&open_bytes), Ok(open_success));
    assert_eq!(
        decode_pc_bank_reply_0104(packet::P_FE2CL_REP_PC_BANK_OPEN_SUCC, &open_bytes),
        Ok(Some(PcBankReply0104::OpenSuccess(open_success)))
    );

    let open_failure = PcBankFailure0104 { error_code: 2 };
    assert_eq!(
        decode_pc_bank_reply_0104(
            packet::P_FE2CL_REP_PC_BANK_OPEN_FAIL,
            &open_failure.encode(),
        ),
        Ok(Some(PcBankReply0104::OpenFailure(open_failure)))
    );
    let close_success = PcBankCloseSuccess0104 { pc_id: 777 };
    assert_eq!(
        decode_pc_bank_reply_0104(
            packet::P_FE2CL_REP_PC_BANK_CLOSE_SUCC,
            &close_success.encode(),
        ),
        Ok(Some(PcBankReply0104::CloseSuccess(close_success)))
    );

    for (packet_id, expected) in [
        (packet::P_CL2FE_REQ_ITEM_MOVE, 16),
        (packet::P_CL2FE_REQ_PC_BANK_OPEN, 8),
        (packet::P_CL2FE_REQ_PC_BANK_CLOSE, 4),
        (packet::P_FE2CL_REP_PC_BANK_OPEN_SUCC, 2_404),
        (packet::P_FE2CL_REP_PC_BANK_OPEN_FAIL, 4),
        (packet::P_FE2CL_REP_PC_BANK_CLOSE_SUCC, 4),
        (packet::P_FE2CL_REP_PC_BANK_CLOSE_FAIL, 4),
    ] {
        assert_eq!(fixed_payload_size(packet_id), Some(expected));
    }
}
