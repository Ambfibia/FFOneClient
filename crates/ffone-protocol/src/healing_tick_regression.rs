use super::*;
#[test]
fn healing_tick_requires_complete_matching_owner_and_nonnegative_hp() {
    let tick=TimeBuffHealTick0104 {character_type:1,character_id:77,healed_hp:100,hp:500};
    let bytes=tick.encode();assert_eq!(TimeBuffHealTick0104::decode(&bytes).unwrap(),tick);
    for length in 0..bytes.len() {assert!(TimeBuffHealTick0104::decode(&bytes[..length]).is_err());}
    for (offset,value) in [(8,17),(16,88),(20,-1),(24,-1)] {
        let mut bad=bytes.clone();write_i32(&mut bad,offset,value);assert!(TimeBuffHealTick0104::decode(&bad).is_err());
    }
}
