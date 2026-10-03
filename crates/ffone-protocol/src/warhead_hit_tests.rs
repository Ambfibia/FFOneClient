use super::*;
#[test]
fn warhead_hit_decodes_absolute_hp_and_rejects_partial_trailers() {
    let result = AttackResult0104 { entity_type: 4, id: 21, protected: 0, damage: 50, hp: 75, hit_flag: 1 };
    let mut payload = vec![0; 24];
    write_i32(&mut payload, 0, 17);
    write_i32(&mut payload, 20, 1);
    payload.extend(result.encode());
    for packet_type in [packet::P_FE2CL_PC_ROCKET_STYLE_HIT, packet::P_FE2CL_PC_GRENADE_STYLE_HIT] {
        assert_eq!(decode_npc_combat_packet_0104(packet_type, &payload).unwrap(),
            Some(NpcCombatPacket0104::PcAttackNpcs(PcAttackNpcs0104 { pc_id: 17, results: vec![result] })));
    }
    payload.pop();
    assert!(decode_pc_warhead_hit_0104(&payload).is_err());
    assert!(decode_pc_warhead_hit_0104(&payload[..20]).is_err());
}
