use super::*;

#[test]
fn npc_skill_animation_signals_use_clean_pack4_headers() {
    let mut ready = vec![0u8; 20];
    ready[0..4].copy_from_slice(&314i32.to_le_bytes());
    ready[4..6].copy_from_slice(&27i16.to_le_bytes());
    assert_eq!(
        decode_npc_skill_signal_0104(packet::P_FE2CL_NPC_SKILL_READY, &ready),
        Ok(Some(NpcSkillSignal0104 {
            npc_id: 314,
            skill_id: Some(27),
            kind: NpcSkillSignalKind0104::Ready,
        }))
    );

    let mut hit = vec![0u8; 28 + 17];
    hit[0..4].copy_from_slice(&314i32.to_le_bytes());
    hit[4..6].copy_from_slice(&27i16.to_le_bytes());
    assert_eq!(
        decode_npc_skill_signal_0104(packet::P_FE2CL_NPC_SKILL_HIT, &hit),
        Ok(Some(NpcSkillSignal0104 {
            npc_id: 314,
            skill_id: Some(27),
            kind: NpcSkillSignalKind0104::Hit,
        }))
    );
    assert_eq!(
        decode_npc_skill_signal_0104(packet::P_FE2CL_NPC_SKILL_HIT, &hit[..27]),
        Err(PayloadError::WrongSize {
            expected: 28,
            actual: 27,
        })
    );

    let cancel = 314i32.to_le_bytes();
    assert_eq!(
        decode_npc_skill_signal_0104(packet::P_FE2CL_NPC_SKILL_CANCEL, &cancel),
        Ok(Some(NpcSkillSignal0104 {
            npc_id: 314,
            skill_id: None,
            kind: NpcSkillSignalKind0104::Cancel,
        }))
    );

    let barker = NpcBarker0104 {
        npc_id: 90210,
        mission_string_id: 3412,
    };
    assert_eq!(NpcBarker0104::decode(&barker.encode()), Ok(barker));
    assert_eq!(barker.encode(), [98, 96, 1, 0, 84, 13, 0, 0]);
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_REP_BARKER),
        Some(NpcBarker0104::SIZE)
    );
    let request = NpcBarkerRequest0104 {
        mission_task_id: 2248,
        npc_id: 90210,
    };
    assert_eq!(NpcBarkerRequest0104::decode(&request.encode()), Ok(request));
    assert!(
        RegisteredGameplayRequest0104::new(packet::P_CL2FE_REQ_BARKER, request.encode(),)
            .is_ok()
    );
}
