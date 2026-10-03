use super::*;

pub(super) fn append_skill_result_damage(bytes: &mut Vec<u8>, record: SkillResultDamage0104) {
    let start = bytes.len();
    bytes.resize(start + SkillResultDamage0104::SIZE, 0);
    write_i32(bytes, start, record.character_type);
    write_i32(bytes, start + 4, record.character_id);
    write_i32(bytes, start + 8, record.protected);
    write_i32(bytes, start + 12, record.damage);
    write_i32(bytes, start + 16, record.hp);
}

pub(super) fn append_skill_result_heal_hp(bytes: &mut Vec<u8>, record: SkillResultHealHp0104) {
    let start = bytes.len();
    bytes.resize(start + SkillResultHealHp0104::SIZE, 0);
    write_i32(bytes, start, record.character_type);
    write_i32(bytes, start + 4, record.character_id);
    write_i32(bytes, start + 8, record.healed_hp);
    write_i32(bytes, start + 12, record.hp);
}

pub(super) fn append_skill_result_buff(bytes: &mut Vec<u8>, record: SkillResultBuff0104) {
    let start = bytes.len();
    bytes.resize(start + SkillResultBuff0104::SIZE, 0);
    write_i32(bytes, start, record.character_type);
    write_i32(bytes, start + 4, record.character_id);
    write_i32(bytes, start + 8, record.protected);
    write_i32(bytes, start + 12, record.condition_bit_flag);
}

#[test]
fn ep_movement_requests_round_trip_at_the_clean_pack4_sizes() {
    macro_rules! round_trip {
        ($packet:expr, $value:expr, $ty:ty) => {{
            let value: $ty = $value;
            let encoded = value.encode();
            assert_eq!(encoded.len(), <$ty>::SIZE);
            assert_eq!(fixed_payload_size($packet), Some(<$ty>::SIZE));
            assert_eq!(<$ty>::decode(&encoded).unwrap(), value);
        }};
    }
    round_trip!(
        packet::P_CL2FE_REQ_PC_JUMPPAD,
        PcJumppadRequest0104 {
            client_time: 1_300,
            position: [1, 2, 3],
            velocity: [4, 5, 6],
            angle: 7,
            key_value: 8,
        },
        PcJumppadRequest0104
    );
    round_trip!(
        packet::P_CL2FE_REQ_PC_LAUNCHER,
        PcLauncherRequest0104 {
            client_time: 9,
            position: [10, 11, 12],
            velocity: [13, 14, 15],
            angle: 16,
            speed: 17,
        },
        PcLauncherRequest0104
    );
    round_trip!(
        packet::P_CL2FE_REQ_PC_MOVEPLATFORM,
        PcMovePlatformRequest0104 {
            client_time: 18,
            local_position: [19, 20, 21],
            position: [22, 23, 24],
            velocity: [1.25, -2.5, 3.75],
            down: 1,
            platform_id: 25,
            angle: 26,
            key_value: 3,
            speed: 27,
        },
        PcMovePlatformRequest0104
    );
    round_trip!(
        packet::P_CL2FE_REQ_PC_MOVETRANSPORTATION,
        PcMoveTransportationRequest0104 {
            client_time: 28,
            local_position: [29, 30, 31],
            position: [32, 33, 34],
            velocity: [-1.0, 2.0, -3.0],
            transportation_id: 35,
            angle: 36,
            key_value: 4,
            speed: 37,
        },
        PcMoveTransportationRequest0104
    );
    round_trip!(
        packet::P_CL2FE_REQ_PC_SLOPE,
        PcSlopeRequest0104 {
            client_time: 38,
            position: [39, 40, 41],
            angle: 42,
            speed: 43,
            key_value: 5,
            velocity: [4.5, 5.5, 6.5],
            slope_id: 44,
        },
        PcSlopeRequest0104
    );
    round_trip!(
        packet::P_CL2FE_REQ_PC_ZIPLINE,
        PcZiplineRequest0104 {
            client_time: 45,
            start_position: [46, 47, 48],
            moved_distance: 1.5,
            maximum_distance: 9.5,
            dummy: 0.0,
            position: [49, 50, 51],
            velocity: [7.0, 8.0, 9.0],
            down: 1,
            roll_max: 52,
            roll: 6,
            angle: 53,
            speed: 54,
        },
        PcZiplineRequest0104
    );
}

#[test]
fn key_and_cipher_golden_values_match_openfusion() {
    assert_eq!(DEFAULT_KEY, 0x5121_7e54_7224_3e62);
    assert_eq!(
        derive_key(0x1122_3344_5566_7788, 3, 7),
        0x043f_235c_2996_4200
    );
    assert_eq!(derive_frontend_key(44), 0x3989_a24f_b8c5_6068);

    let mut body = [
        0x05, 0x40, 0xc2, 0x12, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01,
    ];
    encrypt_in_place(&mut body, DEFAULT_KEY);
    assert_eq!(
        body,
        [
            0x73, 0x7e, 0xe6, 0x60, 0x5c, 0x79, 0x27, 0x54, 0x66, 0x3d, 0x26, 0x67,
        ]
    );
    decrypt_in_place(&mut body, DEFAULT_KEY);
    assert_eq!(
        body,
        [
            0x05, 0x40, 0xc2, 0x12, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01,
        ]
    );
}

#[test]
fn movement_payloads_match_openfusion_pack4_layout() {
    let movement = PcMoveRequest0104 {
        client_time: 0x0102_0304_0506_0708,
        position: [100, -200, 300],
        velocity: [1.25, -2.5, 3.75],
        angle: -45,
        key_value: 8,
        speed: 650,
    };
    let encoded = movement.encode();
    assert_eq!(encoded.len(), 44);
    assert_eq!(&encoded[..8], &movement.client_time.to_le_bytes());
    assert_eq!(&encoded[8..12], &100i32.to_le_bytes());
    assert_eq!(&encoded[20..24], &1.25f32.to_le_bytes());
    assert_eq!(encoded[36], 8);
    assert_eq!(&encoded[37..40], &[0, 0, 0]);
    assert_eq!(&encoded[40..44], &650i32.to_le_bytes());
    assert_eq!(PcMoveRequest0104::decode(&encoded).unwrap(), movement);

    let stop = PcStopRequest0104 {
        client_time: 0,
        position: [1, 2, 3],
    };
    assert_eq!(PcStopRequest0104::decode(&stop.encode()).unwrap(), stop);

    let jump = PcJumpRequest0104 {
        client_time: 9,
        position: [10, 20, 30],
        velocity: [40, 50, 60],
        angle: 70,
        key_value: 2,
        speed: 800,
    };
    assert_eq!(PcJumpRequest0104::decode(&jump.encode()).unwrap(), jump);

    let broadcast = PcMove0104 {
        movement,
        pc_id: 1234,
        server_time: 0x1112_1314_1516_1718,
    };
    assert_eq!(PcMove0104::decode(&broadcast.encode()).unwrap(), broadcast);
}

#[test]
fn cipher_round_trips_all_block_shapes() {
    for length in 0..=128 {
        let mut bytes = (0..length).map(|value| value as u8).collect::<Vec<_>>();
        let original = bytes.clone();
        encrypt_in_place(&mut bytes, 0x8877_6655_4433_2211);
        decrypt_in_place(&mut bytes, 0x8877_6655_4433_2211);
        assert_eq!(bytes, original, "length {length}");
    }
}

#[test]
fn legacy_client_xors_sequence_into_checksum_flags() {
    let payload = 0x0102_0304_0506_0708i64.to_le_bytes();
    let encoded =
        encode_client_frame(packet::P_CL2LS_REQ_CHAR_SELECT, &payload, DEFAULT_KEY, 7).unwrap();
    assert_eq!(
        encoded,
        [
            0x0c, 0x00, 0x00, 0x00, 0x73, 0x0e, 0xe6, 0x60, 0x5c, 0x79, 0x27, 0x54, 0x66, 0x3d,
            0x26, 0x67,
        ]
    );
    let decoded = decode_client_frame(&encoded, DEFAULT_KEY, 7).unwrap();
    assert_eq!(decoded.flags, 0x0c23);
    assert_eq!(decoded.checksum, 0x0c24);
}

#[test]
fn tcp_buffer_handles_split_and_coalesced_frames() {
    let all = OPENFUSION_CHAR_SELECT_FRAME.repeat(2);
    let mut buffer = FrameBuffer::default();
    buffer.push(&all[..7]);
    assert!(buffer.next(DEFAULT_KEY).unwrap().is_none());
    buffer.push(&all[7..]);
    assert!(buffer.next(DEFAULT_KEY).unwrap().is_some());
    assert!(buffer.next(DEFAULT_KEY).unwrap().is_some());
    assert!(buffer.next(DEFAULT_KEY).unwrap().is_none());
    assert_eq!(buffer.buffered_len(), 0);
}

#[test]
fn login_success_places_u64_at_pack4_offset_four() {
    let packet = LoginSuccess {
        character_count: 2,
        selected_slot: 1,
        payment_flag: 1,
        packing_byte: 0,
        server_time: 0x1122_3344_5566_7788,
        id: FixedUtf16::from_str("Ambfibia").unwrap(),
        open_beta_flag: 0,
    };
    let bytes = packet.encode();
    assert_eq!(bytes.len(), 84);
    assert_eq!(&bytes[4..12], &packet.server_time.to_le_bytes());
    assert_eq!(LoginSuccess::decode(&bytes).unwrap(), packet);
    assert_eq!(
        derive_login_e_key(packet.server_time, 2, 1),
        derive_key(packet.server_time, 3, 2)
    );
}

#[test]
fn character_name_packets_follow_openfusion_0104_pack_offsets() {
    let check = CharacterNameCheckRequest0104::new(
        "Proto",
        "Hero",
        0x1112_1314,
        0x3132_3334,
        0x2122_2324,
    )
    .unwrap();
    let bytes = check.encode();
    assert_eq!(&bytes[0..4], &0x1112_1314i32.to_le_bytes());
    assert_eq!(&bytes[4..8], &0x2122_2324i32.to_le_bytes());
    assert_eq!(&bytes[8..12], &0x3132_3334i32.to_le_bytes());
    assert_eq!(&bytes[12..24], b"P\0r\0o\0t\0o\0\0\0");
    assert_eq!(&bytes[30..40], b"H\0e\0r\0o\0\0\0");
    assert_eq!(
        CharacterNameCheckRequest0104::decode(&bytes),
        Ok(check.clone())
    );

    let checked = CharacterNameCheckSuccess0104 {
        first_name: check.first_name.clone(),
        last_name: check.last_name.clone(),
    };
    let checked_bytes = checked.encode();
    assert_eq!(&checked_bytes[0..12], b"P\0r\0o\0t\0o\0\0\0");
    assert_eq!(&checked_bytes[18..28], b"H\0e\0r\0o\0\0\0");
    assert_eq!(
        CharacterNameCheckSuccess0104::decode(&checked_bytes),
        Ok(checked.clone())
    );

    let save = CharacterNameSaveRequest0104::from_check(3, 2, &check, &checked);
    let save_bytes = save.encode();
    assert_eq!(&save_bytes[0..4], &[3, 2, 0, 0]);
    assert_eq!(&save_bytes[4..8], &0x1112_1314i32.to_le_bytes());
    assert_eq!(&save_bytes[8..12], &0x2122_2324i32.to_le_bytes());
    assert_eq!(&save_bytes[12..16], &0x3132_3334i32.to_le_bytes());
    assert_eq!(&save_bytes[16..28], b"P\0r\0o\0t\0o\0\0\0");
    assert_eq!(CharacterNameSaveRequest0104::decode(&save_bytes), Ok(save));

    let saved = CharacterNameSaveSuccess0104 {
        pc_uid: 0x0102_0304_0506_0708,
        slot: 3,
        gender: 2,
        first_name: checked.first_name,
        last_name: checked.last_name,
    };
    let saved_bytes = saved.encode();
    assert_eq!(&saved_bytes[0..8], &0x0102_0304_0506_0708i64.to_le_bytes());
    assert_eq!(&saved_bytes[8..12], &[3, 2, b'P', 0]);
    assert_eq!(&saved_bytes[62..64], &[0, 0]);
    assert_eq!(
        CharacterNameSaveSuccess0104::decode(&saved_bytes),
        Ok(saved)
    );
}

#[test]
fn character_create_and_delete_packets_follow_nested_pack_offsets() {
    let style = PcStyle0104 {
        pc_uid: 0x0102_0304_0506_0708,
        name_check: 1,
        first_name: FixedUtf16::from_str("Proto").unwrap(),
        last_name: FixedUtf16::from_str("Hero").unwrap(),
        gender: 2,
        face_style: 7,
        hair_style: 29,
        hair_color: 13,
        skin_color: 8,
        eye_color: 4,
        height: 3,
        body: 2,
        class: 0x1112_1314,
    };
    let equipped = OnItem0104 {
        hand_id: 0,
        upper_body_id: 101,
        lower_body_id: 202,
        foot_id: 303,
        head_id: 0,
        face_id: 0,
        back_id: 0,
    };
    let selected_indices = OnItemIndex0104 {
        upper_body_index: 4,
        lower_body_index: 5,
        foot_index: 6,
        face_style_index: 2,
        hair_style_index: 9,
    };
    let request = CharacterCreateRequest0104 {
        style: style.clone(),
        equipped,
        selected_indices,
    };
    let request_bytes = request.encode();
    assert_eq!(&request_bytes[0..8], &style.pc_uid.to_le_bytes());
    assert_eq!(request_bytes[8], 1);
    assert_eq!(request_bytes[9], 0);
    assert_eq!(&request_bytes[10..22], b"P\0r\0o\0t\0o\0\0\0");
    assert_eq!(&request_bytes[62..70], &[2, 7, 29, 13, 8, 4, 3, 2]);
    assert_eq!(&request_bytes[72..76], &style.class.to_le_bytes());
    assert_eq!(&request_bytes[76..78], &0i16.to_le_bytes());
    assert_eq!(&request_bytes[78..80], &101i16.to_le_bytes());
    assert_eq!(&request_bytes[90..92], &4i16.to_le_bytes());
    assert_eq!(&request_bytes[98..100], &9i16.to_le_bytes());
    assert_eq!(
        CharacterCreateRequest0104::decode(&request_bytes),
        Ok(request)
    );

    let success = CharacterCreateSuccess0104 {
        level: 1,
        style,
        style2: PcStyle2Flags0104 {
            appearance_flag: 1,
            tutorial_flag: 0,
            payzone_flag: 0,
        },
        equipped,
    };
    let success_bytes = success.encode();
    assert_eq!(&success_bytes[0..4], &[1, 0, 0, 0]);
    assert_eq!(
        &success_bytes[4..12],
        &0x0102_0304_0506_0708i64.to_le_bytes()
    );
    assert_eq!(&success_bytes[80..84], &[1, 0, 0, 0]);
    assert_eq!(&success_bytes[84..86], &0i16.to_le_bytes());
    assert_eq!(&success_bytes[86..88], &101i16.to_le_bytes());
    assert_eq!(&success_bytes[98..100], &[0, 0]);
    assert_eq!(
        CharacterCreateSuccess0104::decode(&success_bytes),
        Ok(success)
    );

    let delete = CharacterDeleteRequest0104 {
        pc_uid: 0x0102_0304_0506_0708,
    };
    assert_eq!(delete.encode(), 0x0102_0304_0506_0708i64.to_le_bytes());
    assert_eq!(
        CharacterDeleteRequest0104::decode(&delete.encode()),
        Ok(delete)
    );
    let deleted = CharacterDeleteSuccess0104 { slot: 3 };
    assert_eq!(deleted.encode(), [3]);
    assert_eq!(
        CharacterDeleteSuccess0104::decode(&deleted.encode()),
        Ok(deleted)
    );
}

#[test]
fn character_info_accessors_follow_nested_struct_offsets() {
    let mut packet = CharacterInfo0104::zeroed();
    packet.as_bytes_mut()[0] = 2;
    packet.as_bytes_mut()[2..4].copy_from_slice(&36i16.to_le_bytes());
    write_i64(packet.as_bytes_mut(), 4, 1234);
    packet.as_bytes_mut()[12] = 1;
    write_utf16(
        packet.as_bytes_mut(),
        14,
        &FixedUtf16::<9>::from_str("Dexter").unwrap(),
    );
    write_utf16(
        packet.as_bytes_mut(),
        32,
        &FixedUtf16::<17>::from_str("Test").unwrap(),
    );
    write_i32(packet.as_bytes_mut(), 84, 100);
    write_i32(packet.as_bytes_mut(), 88, 200);
    write_i32(packet.as_bytes_mut(), 92, 300);
    packet.as_bytes_mut()[66..74].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
    write_i32(packet.as_bytes_mut(), 76, 9);
    packet.as_bytes_mut()[80..83].copy_from_slice(&[10, 11, 12]);
    let upper_body = 96 + CharacterEquipSlot0104::UpperBody as usize * 12;
    packet.as_bytes_mut()[upper_body..upper_body + 2].copy_from_slice(&1i16.to_le_bytes());
    packet.as_bytes_mut()[upper_body + 2..upper_body + 4]
        .copy_from_slice(&321i16.to_le_bytes());
    write_i32(packet.as_bytes_mut(), upper_body + 4, 654);
    write_i32(packet.as_bytes_mut(), upper_body + 8, 987);
    let bytes = packet.encode();
    assert_eq!(bytes.len(), 204);
    let decoded = CharacterInfo0104::decode(&bytes).unwrap();
    assert_eq!(decoded.slot(), 2);
    assert_eq!(decoded.level(), 36);
    assert_eq!(decoded.pc_uid(), 1234);
    assert_eq!(decoded.first_name().to_string_lossy(), "Dexter");
    assert_eq!(decoded.position(), [100, 200, 300]);
    assert_eq!(
        decoded.style(),
        CharacterStyle0104 {
            name_check: 1,
            gender: 1,
            face_style: 2,
            hair_style: 3,
            hair_color: 4,
            skin_color: 5,
            eye_color: 6,
            height: 7,
            body: 8,
            class: 9,
            appearance_flag: 10,
            tutorial_flag: 11,
            payzone_flag: 12,
        }
    );
    assert_eq!(
        decoded.equipped_item(CharacterEquipSlot0104::UpperBody),
        EquippedItem0104 {
            item_type: 1,
            item_id: 321,
            option: 654,
            time_limit: 987,
        }
    );
    assert_eq!(
        decoded.equipment()[CharacterEquipSlot0104::UpperBody as usize],
        decoded.equipped_item(CharacterEquipSlot0104::UpperBody)
    );
    assert_eq!(
        decoded.equipment()[CharacterEquipSlot0104::Vehicle as usize],
        EquippedItem0104::default()
    );
}

#[test]
fn shard_select_and_pc_enter_use_pack4_i64_offsets() {
    let mut ip = [0u8; 16];
    ip[..9].copy_from_slice(b"127.0.0.1");
    let shard = ShardSelectSuccess {
        server_ip: ip,
        server_port: 23001,
        enter_serial_key: 0x0102_0304_0506_0708,
    };
    let bytes = shard.encode();
    assert_eq!(bytes.len(), 28);
    assert_eq!(read_i64(&bytes, 20), shard.enter_serial_key);
    assert_eq!(shard.server_ip_string(), "127.0.0.1");

    let enter = PcEnterRequest {
        id: FixedUtf16::from_str("Ambfibia").unwrap(),
        temporary_value: 0,
        enter_serial_key: shard.enter_serial_key,
    };
    let bytes = enter.encode();
    assert_eq!(bytes.len(), 80);
    assert_eq!(&bytes[66..68], &[0, 0]);
    assert_eq!(read_i64(&bytes, 72), shard.enter_serial_key);
    assert_eq!(PcEnterRequest::decode(&bytes).unwrap(), enter);
}

#[test]
fn initial_around_dispatch_accepts_openfusion_zero_count_bucket_shapes() {
    let cases = [
        (packet::P_FE2CL_PC_AROUND, PcAppearance0104::SIZE),
        (packet::P_FE2CL_NPC_AROUND, NpcAppearance0104::SIZE),
        (
            packet::P_FE2CL_TRANSPORTATION_AROUND,
            TransportationAppearance0104::SIZE,
        ),
        (packet::P_FE2CL_SHINY_AROUND, 4),
    ];
    for (packet_type, payload_size) in cases {
        let decoded = decode_initial_around_0104(packet_type, &vec![0; payload_size]).unwrap();
        assert!(decoded.is_empty());
    }

    assert_eq!(
        decode_initial_around_0104(0xdead_beef, &[]),
        Err(AroundDecodeError::UnsupportedPacket {
            packet_type: 0xdead_beef
        })
    );
}

#[test]
fn around_decoders_enforce_openfusion_bucket_limits_and_checked_lengths() {
    for (decode_error, count, maximum) in [
        (
            decode_pc_around_0104(&17i32.to_le_bytes()).unwrap_err(),
            17,
            16,
        ),
        (
            decode_npc_around_0104(&113i32.to_le_bytes()).unwrap_err(),
            113,
            112,
        ),
        (
            decode_transportation_around_0104(&170i32.to_le_bytes()).unwrap_err(),
            170,
            169,
        ),
        (
            decode_shiny_around_0104(&171i32.to_le_bytes()).unwrap_err(),
            171,
            170,
        ),
    ] {
        assert_eq!(
            decode_error,
            AroundDecodeError::CountTooLarge { count, maximum }
        );
    }

    assert_eq!(
        checked_around_payload_len(2, 1, usize::MAX),
        Err(AroundDecodeError::LengthOverflow {
            count: 2,
            element_size: usize::MAX,
            base_offset: 1,
        })
    );
    assert_eq!(
        checked_around_payload_len(1, usize::MAX, 1),
        Err(AroundDecodeError::LengthOverflow {
            count: 1,
            element_size: 1,
            base_offset: usize::MAX,
        })
    );
}

#[test]
fn pc_lifecycle_payloads_follow_openfusion_0104_pack4_layout() {
    let equipment = std::array::from_fn(|index| ItemBase0104 {
        item_type: index as i16 + 1,
        item_id: index as i16 + 101,
        option: index as i32 + 201,
        time_limit: index as i32 + 301,
    });
    let appearance = PcAppearance0104 {
        id: 0x0102_0304,
        style: PcStyle0104 {
            pc_uid: 0x0102_0304_0506_0708,
            name_check: 3,
            first_name: FixedUtf16::from_str("Dexter").unwrap(),
            last_name: FixedUtf16::from_str("Test").unwrap(),
            gender: 1,
            face_style: 2,
            hair_style: 3,
            hair_color: 4,
            skin_color: 5,
            eye_color: 6,
            height: 7,
            body: 8,
            class: 0x1112_1314,
        },
        condition_bit_flag: 0x2122_2324,
        pc_state: 9,
        special_state: 10,
        level: 0x3132,
        hp: 0x4142_4344,
        map_number: 0x5152_5354,
        position: [0x1112_1314, 0x2122_2324, 0x3132_3334],
        angle: 0x4142_4344,
        equipment,
        nano: Nano0104 {
            id: 0x5152,
            skill_id: 0x6162,
            stamina: 0x7172,
        },
        render_type: 0x6162_6364,
    };
    let packet = PcNew0104 {
        appearance: appearance.clone(),
    };
    let encoded = packet.encode();

    assert_eq!(encoded.len(), 232);
    assert_eq!(&encoded[0..4], &0x0102_0304i32.to_le_bytes());
    assert_eq!(&encoded[4..12], &0x0102_0304_0506_0708i64.to_le_bytes());
    assert_eq!(&encoded[76..80], &0x1112_1314i32.to_le_bytes());
    assert_eq!(&encoded[80..84], &0x2122_2324i32.to_le_bytes());
    assert_eq!(&encoded[86..88], &0x3132i16.to_le_bytes());
    assert_eq!(&encoded[112..114], &1i16.to_le_bytes());
    assert_eq!(&encoded[114..116], &101i16.to_le_bytes());
    assert_eq!(&encoded[220..222], &0x5152i16.to_le_bytes());
    assert_eq!(&encoded[228..232], &0x6162_6364i32.to_le_bytes());
    assert_eq!(PcNew0104::decode(&encoded), Ok(packet));

    let exit = PcExit0104 {
        pc_id: 0x0102_0304,
        exit_type: 0x1112_1314,
    };
    assert_eq!(
        exit.encode(),
        vec![0x04, 0x03, 0x02, 0x01, 0x14, 0x13, 0x12, 0x11]
    );
    assert_eq!(PcExit0104::decode(&exit.encode()), Ok(exit));
    assert_eq!(
        PcExit0104::decode(&[0; 7]),
        Err(PayloadError::WrongSize {
            expected: 8,
            actual: 7,
        })
    );
}

#[test]
fn around_del_pc_validates_count_and_exact_trailer() {
    let packet = AroundDelPc0104 {
        pc_ids: vec![17, 23, 42],
    };
    let encoded = packet.encode().unwrap();
    assert_eq!(
        encoded,
        vec![3, 0, 0, 0, 17, 0, 0, 0, 23, 0, 0, 0, 42, 0, 0, 0]
    );
    assert_eq!(AroundDelPc0104::decode(&encoded), Ok(packet));
    assert_eq!(
        AroundDelPc0104::decode(&(-1i32).to_le_bytes()),
        Err(CountedPayloadError0104::NegativeCount { count: -1 })
    );
    assert_eq!(
        AroundDelPc0104::decode(&[1, 0, 0, 0]),
        Err(CountedPayloadError0104::WrongSize {
            expected: 8,
            actual: 4,
        })
    );
}

#[test]
fn npc_lifecycle_fixed_payloads_have_golden_layouts() {
    let appearance = NpcAppearance0104 {
        npc_id: 0x0102_0304,
        npc_type: 0x1112_1314,
        hp: 0x2122_2324,
        condition_bit_flag: 0x3132_3334,
        position: [0x4142_4344, 0x5152_5354, 0x6162_6364],
        angle: 0x7172_7374,
        barker_type: 0x0101_0101,
    };
    let enter = NpcEnter0104 { appearance };
    assert_eq!(NpcEnter0104::decode(&enter.encode()), Ok(enter));
    assert_eq!(
        NpcNew0104::decode(&appearance.encode()),
        Ok(NpcNew0104 { appearance })
    );

    let movement = NpcMove0104 {
        npc_id: 0x0102_0304,
        destination: [0x1112_1314, 0x2122_2324, 0x3132_3334],
        speed: 0x4142_4344,
        move_style: 0x5152,
    };
    assert_eq!(
        movement.encode(),
        vec![
            0x04, 0x03, 0x02, 0x01, 0x14, 0x13, 0x12, 0x11, 0x24, 0x23, 0x22, 0x21, 0x34, 0x33,
            0x32, 0x31, 0x44, 0x43, 0x42, 0x41, 0x52, 0x51, 0x00, 0x00,
        ]
    );
    assert_eq!(NpcMove0104::decode(&movement.encode()), Ok(movement));
}

#[test]
fn transportation_and_shiny_lifecycle_payloads_round_trip_exactly() {
    let transportation = TransportationAppearance0104 {
        transportation_kind: 3,
        id: 71,
        transportation_type: 9,
        position: [100, 200, 300],
    };
    assert_eq!(
        TransportationAppearance0104::decode(&transportation.encode()),
        Ok(transportation)
    );
    let movement = TransportationMove0104 {
        transportation_kind: 3,
        id: 71,
        destination: [400, 500, 600],
        speed: 700,
        move_style: 1,
    };
    assert_eq!(TransportationMove0104::SIZE, 28);
    assert_eq!(
        TransportationMove0104::decode(&movement.encode()),
        Ok(movement)
    );
    let deleted = AroundDelTransportation0104 {
        transportation_kind: 3,
        ids: vec![71, 72],
    };
    assert_eq!(
        AroundDelTransportation0104::decode(&deleted.encode().unwrap()),
        Ok(deleted)
    );

    let shiny = ShinyAppearance0104 {
        shiny_id: 81,
        shiny_type: 2,
        map_number: 0,
        position: [700, 800, 900],
    };
    assert_eq!(ShinyAppearance0104::decode(&shiny.encode()), Ok(shiny));
    let deleted = AroundDelShiny0104 {
        shiny_ids: vec![81, 82, 83],
    };
    assert_eq!(
        AroundDelShiny0104::decode(&deleted.encode().unwrap()),
        Ok(deleted)
    );
}
