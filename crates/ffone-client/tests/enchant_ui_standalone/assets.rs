use super::*;

#[test]
fn clean_recipe_index_class_switch_and_probability_buckets_are_preserved() {
    assert_eq!(CLEAN_ENCHANT_RECIPES_0104.len(), 11);
    let first = enchant_requirements_0104(item(0, 501, 0)).unwrap();
    assert_eq!(first.current_encoded_level, 1);
    assert_eq!(first.recipe_index, 2);
    assert_eq!(first.weapon_material_count, 12);
    assert_eq!(first.armor_material_id, 0);
    assert_eq!(first.cost, 200);
    assert_eq!(first.chance, Some(EnchantChance0104::VeryHigh));

    let class_one = enchant_requirements_0104(item(1, 502, 3)).unwrap();
    assert_eq!(class_one.recipe_index, 4);
    assert_eq!(class_one.weapon_material_id, 101);
    assert_eq!(class_one.armor_material_id, 102);
    assert_eq!(class_one.weapon_material_count, 10);
    assert_eq!(class_one.armor_material_count, 10);
    assert_eq!(class_one.chance, Some(EnchantChance0104::High));

    assert_eq!(
        EnchantChance0104::from_probability(9),
        EnchantChance0104::VeryLow
    );
    assert_eq!(
        EnchantChance0104::from_probability(10),
        EnchantChance0104::Low
    );
    assert_eq!(
        EnchantChance0104::from_probability(30),
        EnchantChance0104::Medium
    );
    assert_eq!(
        EnchantChance0104::from_probability(70),
        EnchantChance0104::High
    );
    assert_eq!(
        EnchantChance0104::from_probability(90),
        EnchantChance0104::VeryHigh
    );
    assert_eq!(
        EnchantChance0104::from_probability(101),
        EnchantChance0104::LegacyUnassigned
    );
}
