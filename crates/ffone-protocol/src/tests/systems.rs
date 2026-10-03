use super::*;

#[test]
fn environment_dot_and_hp_tick_layouts_match_openfusion_0104() {
    let enabled = EnvironmentDotToggle0104 { enabled: true };
    assert_eq!(enabled.encode(), 1i32.to_le_bytes());
    assert_eq!(
        EnvironmentDotToggle0104::decode(&enabled.encode()),
        Ok(enabled)
    );
    assert_eq!(
        fixed_payload_size(packet::P_CL2FE_DOT_DAMAGE_ONOFF),
        Some(4)
    );
    assert_eq!(fixed_payload_size(packet::P_CL2FE_DOT_HEAL_ONOFF), Some(4));

    let nanos = [
        Nano0104 {
            id: 1,
            skill_id: 8,
            stamina: 130,
        },
        Nano0104 {
            id: 17,
            skill_id: 21,
            stamina: 75,
        },
        Nano0104 {
            id: 36,
            skill_id: 144,
            stamina: 0,
        },
    ];
    let mut remaining = [0; 28];
    for (index, nano) in nanos.iter().copied().enumerate() {
        let start = index * Nano0104::SIZE;
        nano.encode_into(&mut remaining[start..start + Nano0104::SIZE]);
    }
    remaining[18..20].copy_from_slice(&[0xaa, 0xbb]);
    write_i32(&mut remaining, 20, 57);
    write_i32(&mut remaining, 24, 1);
    let tick = PcTick0104 { hp: 875, remaining };
    let wire = tick.encode();
    assert_eq!(wire.len(), PcTick0104::SIZE);
    assert_eq!(&wire[4..10], &[1, 0, 8, 0, 130, 0]);
    assert_eq!(&wire[10..16], &[17, 0, 21, 0, 75, 0]);
    assert_eq!(&wire[16..22], &[36, 0, 144, 0, 0, 0]);
    assert_eq!(&wire[22..24], &[0xaa, 0xbb]);
    assert_eq!(&wire[24..28], &57_i32.to_le_bytes());
    assert_eq!(&wire[28..32], &1_i32.to_le_bytes());
    let decoded = PcTick0104::decode(&wire).unwrap();
    assert_eq!(decoded, tick);
    assert_eq!(decoded.nanos(), nanos);
    assert_eq!(decoded.nano_battery(), 57);
    assert_eq!(decoded.reset_mission_flag(), 1);
    assert_eq!(fixed_payload_size(packet::P_FE2CL_REP_PC_TICK), Some(32));

    let infection = TimeBuffDotDamageTick0104 {
        character_type: 1,
        character_id: 77,
        time_buff_id: 17,
        result_character_type: 1,
        result_character_id: 77,
        protected: false,
        damage: 150,
        hp: 725,
        stamina: 42,
        nano_deactivated: false,
        condition_bit_flag: 0x20,
    };
    let bytes = infection.encode();
    assert_eq!(&bytes[8..10], &17i16.to_le_bytes());
    assert_eq!(&bytes[24..28], &150i32.to_le_bytes());
    assert_eq!(&bytes[28..32], &725i32.to_le_bytes());
    assert_eq!(TimeBuffDotDamageTick0104::decode(&bytes), Ok(infection));
}
