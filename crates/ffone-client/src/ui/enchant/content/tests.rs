use super::*;
use crate::localization::Localization;

#[test]
fn enchant_production_item_details_and_localized_success_are_complete() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let content =
        TutorialMissionContent::open(&crate::assets::AssetLocator::open(&root).unwrap())
            .unwrap();
    let item = ItemBase0104 {
        item_type: 0,
        item_id: 328,
        option: 2,
        time_limit: 0,
    };
    let presentation = enchant_item_presentation_from_content(&content, item).unwrap();
    let detail = content.gameplay_user_equip_item_detail(0, 328).unwrap();
    assert_eq!(presentation.point_rating, detail.point_rating);
    assert!(presentation.point_rating > 0);
    assert_eq!(presentation.group_rating, detail.group_rating);
    assert_eq!(presentation.defense_rating, detail.defense_rating);
    assert!(!presentation.description.is_empty());
    assert!(
        presentation
            .icon_path
            .as_ref()
            .is_some_and(|path| root.join(path).is_file())
    );
    assert!(!presentation.type_label.is_empty());
    assert!(!presentation.range_label.is_empty());
    assert!(!presentation.rarity_label.is_empty());
    assert!(!presentation.trade_label.is_empty());
    let projection = EnchantModeProjection0104 {
        success: Some(EnchantSuccessPresentation0104 {
            item,
            presentation,
            displayed_enchant_level: 1,
        }),
        ..default()
    };
    for locale in ["en", "ru"] {
        let (localization, language) = Localization::open(&root, locale).unwrap();
        for element in [
            EnchantUiElement0104::SuccessName,
            EnchantUiElement0104::SuccessDescription,
            EnchantUiElement0104::SuccessTypeValue,
            EnchantUiElement0104::SuccessRangeValue,
            EnchantUiElement0104::SuccessRarityValue,
            EnchantUiElement0104::SuccessTradeValue,
        ] {
            let text = localized_field(&content, &projection, element).unwrap();
            assert_ne!(text.key, "ui.content.passthrough");
            assert!(!localization.text(&language, &text).is_empty());
        }
    }
    for id in [
        ENCHANT_WEAPON_MATERIAL_ID_0104,
        ENCHANT_ARMOR_MATERIAL_ID_0104,
        ENCHANT_HELP_ITEM_1_ID_0104,
        ENCHANT_HELP_ITEM_2_ID_0104,
    ] {
        assert!(
            enchant_item_presentation_from_content(
                &content,
                ItemBase0104 {
                    item_type: 7,
                    item_id: id,
                    option: 1,
                    time_limit: 0
                }
            )
            .is_some()
        );
    }
}
