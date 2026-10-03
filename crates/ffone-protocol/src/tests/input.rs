use super::*;

#[test]
fn pc_enter_success_retains_exact_0104_load_blob() {
    let mut load = PcLoadData0104::zeroed();
    load.as_bytes_mut()
        [PcLoadData0104::USER_LEVEL_OFFSET..PcLoadData0104::USER_LEVEL_OFFSET + 2]
        .copy_from_slice(&5i16.to_le_bytes());
    write_i64(
        load.as_bytes_mut(),
        PcLoadData0104::STYLE_OFFSET,
        0x0102_0304_0506_0708,
    );
    load.as_bytes_mut()[PcLoadData0104::STYLE_OFFSET + 8] = 1;
    write_utf16(
        load.as_bytes_mut(),
        PcLoadData0104::STYLE_OFFSET + 10,
        &FixedUtf16::<9>::from_str("Test").unwrap(),
    );
    write_utf16(
        load.as_bytes_mut(),
        PcLoadData0104::STYLE_OFFSET + 28,
        &FixedUtf16::<17>::from_str("Ser").unwrap(),
    );
    load.as_bytes_mut()[PcLoadData0104::STYLE_OFFSET + 62..PcLoadData0104::STYLE_OFFSET + 70]
        .copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
    write_i32(load.as_bytes_mut(), PcLoadData0104::STYLE_OFFSET + 72, 9);
    load.as_bytes_mut()
        [PcLoadData0104::STYLE_FLAGS_OFFSET..PcLoadData0104::STYLE_FLAGS_OFFSET + 3]
        .copy_from_slice(&[10, 11, 12]);
    load.as_bytes_mut()[PcLoadData0104::LEVEL_OFFSET..PcLoadData0104::LEVEL_OFFSET + 2]
        .copy_from_slice(&36i16.to_le_bytes());
    load.as_bytes_mut()[PcLoadData0104::MENTOR_OFFSET..PcLoadData0104::MENTOR_OFFSET + 2]
        .copy_from_slice(&7i16.to_le_bytes());
    load.as_bytes_mut()
        [PcLoadData0104::MENTOR_COUNT_OFFSET..PcLoadData0104::MENTOR_COUNT_OFFSET + 2]
        .copy_from_slice(&8i16.to_le_bytes());
    write_i32(load.as_bytes_mut(), PcLoadData0104::HP_OFFSET, 500);
    write_i32(
        load.as_bytes_mut(),
        PcLoadData0104::WEAPON_BATTERY_OFFSET,
        600,
    );
    write_i32(
        load.as_bytes_mut(),
        PcLoadData0104::NANO_BATTERY_OFFSET,
        700,
    );
    write_i32(
        load.as_bytes_mut(),
        PcLoadData0104::FUSION_MATTER_OFFSET,
        12_345,
    );
    load.as_bytes_mut()[PcLoadData0104::SPECIAL_STATE_OFFSET] = 9;
    write_i32(load.as_bytes_mut(), PcLoadData0104::X_OFFSET, 1000);
    write_i32(load.as_bytes_mut(), PcLoadData0104::Y_OFFSET, 2000);
    write_i32(load.as_bytes_mut(), PcLoadData0104::Z_OFFSET, 3000);
    let head_offset = PcLoadData0104::EQUIPMENT_OFFSET
        + CharacterEquipSlot0104::Head as usize * ItemBase0104::SIZE;
    load.as_bytes_mut()[head_offset..head_offset + 2].copy_from_slice(&4i16.to_le_bytes());
    load.as_bytes_mut()[head_offset + 2..head_offset + 4]
        .copy_from_slice(&123i16.to_le_bytes());
    write_i32(load.as_bytes_mut(), head_offset + 4, 456);
    write_i32(load.as_bytes_mut(), head_offset + 8, 789);
    let inventory_offset = PcLoadData0104::INVENTORY_OFFSET + 42 * ItemBase0104::SIZE;
    load.as_bytes_mut()[inventory_offset..inventory_offset + 2]
        .copy_from_slice(&7i16.to_le_bytes());
    load.as_bytes_mut()[inventory_offset + 2..inventory_offset + 4]
        .copy_from_slice(&134i16.to_le_bytes());
    write_i32(load.as_bytes_mut(), inventory_offset + 4, 2);
    write_i32(load.as_bytes_mut(), inventory_offset + 8, 0);
    let quest_inventory_offset =
        PcLoadData0104::QUEST_INVENTORY_OFFSET + 3 * ItemBase0104::SIZE;
    load.as_bytes_mut()[quest_inventory_offset..quest_inventory_offset + 2]
        .copy_from_slice(&7i16.to_le_bytes());
    load.as_bytes_mut()[quest_inventory_offset + 2..quest_inventory_offset + 4]
        .copy_from_slice(&411i16.to_le_bytes());
    write_i32(load.as_bytes_mut(), quest_inventory_offset + 4, 2);
    write_i32(load.as_bytes_mut(), quest_inventory_offset + 8, 0);
    let nano_offset = PcLoadData0104::NANO_BANK_OFFSET + 4 * Nano0104::SIZE;
    load.as_bytes_mut()[nano_offset..nano_offset + 2].copy_from_slice(&17i16.to_le_bytes());
    load.as_bytes_mut()[nano_offset + 2..nano_offset + 4].copy_from_slice(&23i16.to_le_bytes());
    load.as_bytes_mut()[nano_offset + 4..nano_offset + 6]
        .copy_from_slice(&999i16.to_le_bytes());
    for (index, nano_id) in [17i16, 3, -1].into_iter().enumerate() {
        let offset = PcLoadData0104::NANO_SLOTS_OFFSET + index * size_of::<i16>();
        load.as_bytes_mut()[offset..offset + 2].copy_from_slice(&nano_id.to_le_bytes());
    }
    load.as_bytes_mut()
        [PcLoadData0104::ACTIVE_NANO_SLOT_OFFSET..PcLoadData0104::ACTIVE_NANO_SLOT_OFFSET + 2]
        .copy_from_slice(&1i16.to_le_bytes());
    let packet = PcEnterSuccess {
        id: 77,
        load,
        server_time: 0x8877_6655_4433_2211,
    };
    let bytes = packet.encode();
    assert_eq!(bytes.len(), 2700);
    assert_eq!(read_i32(&bytes, 4 + 108), 12_345);
    assert_eq!(&bytes[2692..], &packet.server_time.to_le_bytes());
    let decoded = PcEnterSuccess::decode(&bytes).unwrap();
    assert_eq!(decoded, packet);
    assert_eq!(decoded.load.position(), [1000, 2000, 3000]);
    assert_eq!(decoded.load.style().pc_uid, 0x0102_0304_0506_0708);
    assert_eq!(decoded.load.style().first_name.to_string_lossy(), "Test");
    assert_eq!(decoded.load.style().last_name.to_string_lossy(), "Ser");
    assert_eq!(decoded.load.style().gender, 1);
    assert_eq!(decoded.load.style().body, 8);
    assert_eq!(decoded.load.style().class, 9);
    assert_eq!(decoded.load.style_flags(), [10, 11, 12]);
    assert_eq!(decoded.load.user_level(), 5);
    assert_eq!(decoded.load.level(), 36);
    assert_eq!(decoded.load.mentor(), 7);
    assert_eq!(decoded.load.mentor_count(), 8);
    assert_eq!(decoded.load.weapon_battery(), 600);
    assert_eq!(decoded.load.nano_battery(), 700);
    assert_eq!(decoded.load.special_state(), 9);
    assert_eq!(
        decoded.load.equipment()[CharacterEquipSlot0104::Head as usize],
        ItemBase0104 {
            item_type: 4,
            item_id: 123,
            option: 456,
            time_limit: 789,
        }
    );
    assert_eq!(
        decoded.load.inventory()[42],
        ItemBase0104 {
            item_type: 7,
            item_id: 134,
            option: 2,
            time_limit: 0,
        }
    );
    assert_eq!(
        decoded.load.quest_inventory()[3],
        ItemBase0104 {
            item_type: 7,
            item_id: 411,
            option: 2,
            time_limit: 0,
        }
    );
    assert_eq!(
        decoded.load.nano_bank()[4],
        Nano0104 {
            id: 17,
            skill_id: 23,
            stamina: 999,
        }
    );
    assert_eq!(decoded.load.nano_slots(), [17, 3, -1]);
    assert_eq!(decoded.load.active_nano_slot(), 1);
}

#[test]
fn pc_load_exposes_initial_skill_buff_state_at_exact_offsets() {
    assert_eq!(PcLoadData0104::ACTIVE_NANO_SLOT_OFFSET, 1672);
    assert_eq!(PcLoadData0104::CONDITION_BIT_FLAG_OFFSET, 1676);
    assert_eq!(PcLoadData0104::ADDITIONAL_TIME_BUFF_ID_OFFSET, 1680);
    assert_eq!(PcLoadData0104::INITIAL_TIME_BUFF_OFFSET, 1684);

    let mut load = PcLoadData0104::zeroed();
    write_i32(
        load.as_bytes_mut(),
        PcLoadData0104::CONDITION_BIT_FLAG_OFFSET,
        0x0081_0080,
    );
    write_i32(
        load.as_bytes_mut(),
        PcLoadData0104::ADDITIONAL_TIME_BUFF_ID_OFFSET,
        17,
    );
    let initial = TimeBuff0104 {
        time_limit: 9_000,
        time_duration: 8_000,
        time_repeat: 7,
        value: 6,
        confirm_number: 5,
    };
    load.as_bytes_mut()[PcLoadData0104::INITIAL_TIME_BUFF_OFFSET
        ..PcLoadData0104::INITIAL_TIME_BUFF_OFFSET + TimeBuff0104::SIZE]
        .copy_from_slice(&initial.encode());

    assert_eq!(load.condition_bit_flag(), 0x0081_0080);
    assert_eq!(load.additional_time_buff_id(), 17);
    assert_eq!(load.initial_time_buff(), initial);
}

#[test]
fn buddy_find_name_0104_has_exact_pack2_and_pack4_utf16_layouts() {
    let first_name = FixedUtf16::from_str("\u{0414}\u{044d}\u{043a}\u{0441}").unwrap();
    let last_name =
        FixedUtf16::from_str("\u{041c}\u{0430}\u{043d}\u{0434}\u{0430}\u{0440}\u{043a}")
            .unwrap();
    let request = BuddyFindNameRequest0104 {
        first_name: first_name.clone(),
        last_name: last_name.clone(),
    };
    let request_bytes = request.encode();
    assert_eq!(request_bytes.len(), 52);
    assert_eq!(
        BuddyFindNameRequest0104::decode(&request_bytes),
        Ok(request.clone())
    );
    assert_eq!(
        decode_buddy_lifecycle_packet_0104(
            packet::P_CL2FE_REQ_PC_FIND_NAME_MAKE_BUDDY,
            &request_bytes,
        ),
        Ok(Some(BuddyLifecyclePacket0104::FindNameRequest(request)))
    );

    let success = BuddyFindNameSuccess0104 {
        first_name: first_name.clone(),
        last_name: last_name.clone(),
        pc_uid: 0x0102_0304_0506_0708,
        name_check_flag: 1,
    };
    let success_bytes = success.encode();
    assert_eq!(success_bytes.len(), 64);
    assert_eq!(
        &success_bytes[52..60],
        &0x0102_0304_0506_0708i64.to_le_bytes()
    );
    assert_eq!(success_bytes[60], 1);
    assert_eq!(&success_bytes[61..64], &[0, 0, 0]);
    assert_eq!(
        decode_buddy_lifecycle_packet_0104(
            packet::P_FE2CL_REP_PC_FIND_NAME_MAKE_BUDDY_SUCC,
            &success_bytes,
        ),
        Ok(Some(BuddyLifecyclePacket0104::FindNameSuccess(success)))
    );

    let failure = BuddyFindNameFailure0104 {
        first_name: first_name.clone(),
        last_name: last_name.clone(),
        error_code: 5,
    };
    let failure_bytes = failure.encode();
    assert_eq!(failure_bytes.len(), 56);
    assert_eq!(&failure_bytes[52..56], &5i32.to_le_bytes());
    assert_eq!(
        decode_buddy_lifecycle_packet_0104(
            packet::P_FE2CL_REP_PC_FIND_NAME_MAKE_BUDDY_FAIL,
            &failure_bytes,
        ),
        Ok(Some(BuddyLifecyclePacket0104::FindNameFailure(failure)))
    );
    assert_eq!(
        BuddyFindNameSuccess0104::decode(&success_bytes[..63]),
        Err(PayloadError::WrongSize {
            expected: BuddyFindNameSuccess0104::SIZE,
            actual: 63,
        })
    );

    let accept = BuddyFindNameAcceptRequest0104 {
        accept_flag: 1,
        buddy_pc_uid: 0x0102_0304_0506_0708,
        first_name: first_name.clone(),
        last_name: last_name.clone(),
    };
    let accept_bytes = accept.encode();
    assert_eq!(accept_bytes.len(), 64);
    assert_eq!(&accept_bytes[0..4], &1i32.to_le_bytes());
    assert_eq!(
        &accept_bytes[4..12],
        &0x0102_0304_0506_0708i64.to_le_bytes()
    );
    assert_eq!(
        BuddyFindNameAcceptRequest0104::decode(&accept_bytes),
        Ok(accept.clone())
    );
    assert_eq!(
        decode_buddy_lifecycle_packet_0104(
            packet::P_CL2FE_REQ_PC_FIND_NAME_ACCEPT_BUDDY,
            &accept_bytes,
        ),
        Ok(Some(BuddyLifecyclePacket0104::FindNameAcceptRequest(
            accept
        )))
    );

    let accept_failure = BuddyFindNameAcceptFailure0104 {
        first_name,
        last_name,
        pc_uid: 0x0102_0304_0506_0708,
        name_check_flag: 1,
        error_code: 6,
    };
    let accept_failure_bytes = accept_failure.encode();
    assert_eq!(accept_failure_bytes.len(), 68);
    assert_eq!(
        &accept_failure_bytes[52..60],
        &0x0102_0304_0506_0708i64.to_le_bytes()
    );
    assert_eq!(accept_failure_bytes[60], 1);
    assert_eq!(&accept_failure_bytes[61..64], &[0, 0, 0]);
    assert_eq!(&accept_failure_bytes[64..68], &6i32.to_le_bytes());
    assert_eq!(
        decode_buddy_lifecycle_packet_0104(
            packet::P_FE2CL_REP_PC_FIND_NAME_ACCEPT_BUDDY_FAIL,
            &accept_failure_bytes,
        ),
        Ok(Some(BuddyLifecyclePacket0104::FindNameAcceptFailure(
            accept_failure
        )))
    );
    assert_eq!(
        BuddyFindNameAcceptFailure0104::decode(&accept_failure_bytes[..67]),
        Err(PayloadError::WrongSize {
            expected: BuddyFindNameAcceptFailure0104::SIZE,
            actual: 67,
        })
    );
}

#[test]
fn pc_load_mission_regions_keep_exact_offsets_and_counts() {
    let mut load = PcLoadData0104::zeroed();
    write_i64(
        load.as_bytes_mut(),
        PcLoadData0104::QUEST_FLAGS_OFFSET + 3 * 8,
        1_i64 << 17,
    );
    write_i64(
        load.as_bytes_mut(),
        PcLoadData0104::REPEAT_QUEST_FLAGS_OFFSET + 2 * 8,
        1_i64 << 5,
    );
    let running = RunningQuest0104 {
        task_id: 2248,
        kill_npc_ids: [2676, 0, 0],
        remaining_kill_counts: [2, 0, 0],
        needed_item_ids: [120, 0, 0],
        needed_item_counts: [1, 0, 0],
    };
    let running_offset = PcLoadData0104::RUNNING_QUESTS_OFFSET + 4 * RunningQuest0104::SIZE;
    load.as_bytes_mut()[running_offset..running_offset + RunningQuest0104::SIZE]
        .copy_from_slice(&running.encode());
    write_i32(
        load.as_bytes_mut(),
        PcLoadData0104::CURRENT_MISSION_ID_OFFSET,
        937,
    );
    write_i32(
        load.as_bytes_mut(),
        PcLoadData0104::WARP_LOCATION_FLAG_OFFSET,
        0x4000_0001,
    );
    write_i64(
        load.as_bytes_mut(),
        PcLoadData0104::WYVERN_LOCATION_FLAGS_OFFSET,
        i64::MIN | 3,
    );
    write_i64(
        load.as_bytes_mut(),
        PcLoadData0104::WYVERN_LOCATION_FLAGS_OFFSET + 8,
        1_i64 << 17,
    );

    assert_eq!(load.quest_flags()[3], 1_i64 << 17);
    assert_eq!(load.repeat_quest_flags()[2], 1_i64 << 5);
    assert_eq!(load.running_quests()[4], running);
    assert_eq!(load.current_mission_id(), 937);
    assert_eq!(load.warp_location_flag(), 0x4000_0001);
    assert_eq!(
        load.wyvern_location_flags(),
        [(i64::MIN | 3) as u64, 1 << 17]
    );
    assert_eq!(PcLoadData0104::QUEST_FLAG_COUNT, 32);
    assert_eq!(PcLoadData0104::REPEAT_QUEST_FLAG_COUNT, 8);
    assert_eq!(PcLoadData0104::RUNNING_QUEST_COUNT, 9);
    assert_eq!(
        PcLoadData0104::CURRENT_MISSION_ID_OFFSET,
        PcLoadData0104::RUNNING_QUESTS_OFFSET
            + PcLoadData0104::RUNNING_QUEST_COUNT * RunningQuest0104::SIZE
    );
    assert_eq!(
        PcLoadData0104::WARP_LOCATION_FLAG_OFFSET,
        PcLoadData0104::CURRENT_MISSION_ID_OFFSET + 4
    );
    assert_eq!(
        PcLoadData0104::WYVERN_LOCATION_FLAGS_OFFSET,
        PcLoadData0104::WARP_LOCATION_FLAG_OFFSET + 4
    );
}
