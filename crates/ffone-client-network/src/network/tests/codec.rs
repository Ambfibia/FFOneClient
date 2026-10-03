use super::*;

pub(super) fn read_tutorial_wire(stream: &mut TcpStream) -> Vec<u8> {
    let mut length = [0u8; 4];
    stream.read_exact(&mut length).unwrap();
    let body_len = u32::from_le_bytes(length) as usize;
    let mut frame = vec![0u8; body_len + 4];
    frame[..4].copy_from_slice(&length);
    stream.read_exact(&mut frame[4..]).unwrap();
    frame
}

#[test]
fn normal_world_nano_and_vehicle_commands_encode_exact_registered_requests() {
    let equip = NanoEquipRequest0104 {
        nano_id: 36,
        nano_slot: 2,
    };
    let unequip = NanoUnequipRequest0104 { nano_slot: 1 };
    let active = NanoActiveRequest0104 { nano_slot: -1 };
    let vehicle_off = PcVehicleOffRequest0104 { unused: 0 };
    let skill = NanoSkillUseRequest0104 {
        bullet_id: 7,
        arg1: 100,
        arg2: -200,
        arg3: 300,
        target_ids: vec![81, 82],
    };

    let registered = [
        registered_fixed_world_action_0104(packet::P_CL2FE_REQ_NANO_EQUIP, &equip).unwrap(),
        registered_fixed_world_action_0104(packet::P_CL2FE_REQ_NANO_UNEQUIP, &unequip).unwrap(),
        registered_fixed_world_action_0104(packet::P_CL2FE_REQ_NANO_ACTIVE, &active).unwrap(),
        registered_nano_skill_use_request_0104(&skill).unwrap(),
        registered_fixed_world_action_0104(packet::P_CL2FE_REQ_PC_VEHICLE_OFF, &vehicle_off)
            .unwrap(),
    ];

    for (sequence, request) in registered.iter().enumerate() {
        let wire = encode_client_frame(
            request.packet_type(),
            request.payload(),
            DEFAULT_KEY,
            sequence as u16,
        )
        .unwrap();
        let decoded = decode_client_frame(&wire, DEFAULT_KEY, sequence as u16).unwrap();
        assert_eq!(decoded.packet_type, request.packet_type());
        assert_eq!(decoded.payload, request.payload());
        assert_eq!(
            decoded.payload.len(),
            match request.packet_type() {
                packet::P_CL2FE_REQ_NANO_EQUIP => NanoEquipRequest0104::SIZE,
                packet::P_CL2FE_REQ_NANO_UNEQUIP => NanoUnequipRequest0104::SIZE,
                packet::P_CL2FE_REQ_NANO_ACTIVE => NanoActiveRequest0104::SIZE,
                packet::P_CL2FE_REQ_NANO_SKILL_USE => {
                    NanoSkillUseRequest0104::HEADER_SIZE + 2 * size_of::<i32>()
                }
                packet::P_CL2FE_REQ_PC_VEHICLE_OFF => PcVehicleOffRequest0104::SIZE,
                packet_type => panic!("unexpected packet {packet_type:#010x}"),
            }
        );
    }

    assert_eq!(
        NanoEquipRequest0104::decode(registered[0].payload()),
        Ok(equip)
    );
    assert_eq!(
        NanoUnequipRequest0104::decode(registered[1].payload()),
        Ok(unequip)
    );
    assert_eq!(
        NanoActiveRequest0104::decode(registered[2].payload()),
        Ok(active)
    );
    assert_eq!(
        NanoSkillUseRequest0104::decode(registered[3].payload()),
        Ok(skill)
    );
    assert_eq!(
        PcVehicleOffRequest0104::decode(registered[4].payload()),
        Ok(vehicle_off)
    );
}
