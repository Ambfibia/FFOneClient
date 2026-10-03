use super::*;

#[test]
fn item_use_0104_all_proven_uniform_tail_families_validate_every_count() {
    let families = [
        (1, SkillResultDamage0104::SIZE),
        (2, SkillResultHealHp0104::SIZE),
        (34, SkillResultHealHp0104::SIZE),
        (3, SkillResultDamageDebuff0104::SIZE),
        (4, SkillResultDamageDebuff0104::SIZE),
        (5, SkillResultDamageDebuff0104::SIZE),
        (8, SkillResultDamageDebuff0104::SIZE),
        (6, SkillResultHealStamina0104::SIZE),
        (10, SkillResultBuff0104::SIZE),
        (11, SkillResultBuff0104::SIZE),
        (12, SkillResultBuff0104::SIZE),
        (14, SkillResultBuff0104::SIZE),
        (15, SkillResultBuff0104::SIZE),
        (16, SkillResultBuff0104::SIZE),
        (17, SkillResultBuff0104::SIZE),
        (18, SkillResultBuff0104::SIZE),
        (19, SkillResultBuff0104::SIZE),
        (20, SkillResultBuff0104::SIZE),
        (23, SkillResultBuff0104::SIZE),
        (25, SkillResultBuff0104::SIZE),
        (31, SkillResultBuff0104::SIZE),
        (32, SkillResultBuff0104::SIZE),
        (33, SkillResultBuff0104::SIZE),
        (35, SkillResultBuff0104::SIZE),
        (21, SkillResultBatteryDrain0104::SIZE),
        (26, SkillResultResurrect0104::SIZE),
        (27, SkillResultMove0104::SIZE),
        (28, SkillResultMove0104::SIZE),
    ];
    for (skill_type, record_size) in families {
        for target_count in [1, 3] {
            let prefix = ItemUseBroadcastPrefix0104 {
                pc_id: 1,
                skill_id: 2,
                pack_padding: [0, 0],
                skill_type,
                target_count,
            };
            let mut payload = prefix.encode_prefix();
            payload.resize(
                ItemUseBroadcastPrefix0104::SIZE
                    + usize::try_from(target_count).unwrap() * record_size,
                0,
            );
            let decoded = ItemUseBroadcastPacket0104::decode(&payload).unwrap();
            assert_eq!(decoded.prefix, prefix);
            assert_eq!(
                decoded.results.target_count(),
                usize::try_from(target_count).unwrap(),
                "eST={skill_type}"
            );

            let mut truncated = payload.clone();
            truncated.pop();
            assert!(matches!(
                ItemUseBroadcastPacket0104::decode(&truncated),
                Err(ItemUseDecodeError0104::Fixed(
                    PayloadError::WrongSize { .. }
                ))
            ));
            payload.push(0);
            assert!(matches!(
                ItemUseBroadcastPacket0104::decode(&payload),
                Err(ItemUseDecodeError0104::Fixed(
                    PayloadError::WrongSize { .. }
                ))
            ));
        }
    }
}
