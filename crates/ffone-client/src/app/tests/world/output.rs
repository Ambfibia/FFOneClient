use super::*;

#[test]
fn gameplay_loading_uses_source_tip_range_and_distinct_assetloader_copy() {
    let mut loading = GameplayLoadingState::default();
    for _ in 0..512 {
        loading.begin(ResourceLoadingScope::CharacterSelection);
        assert!(loading.tip_index < GAMEPLAY_LOADING_TIPS.len() - 1);
    }
    assert_eq!(
        loading.localized_tip().fallback,
        GAMEPLAY_LOADING_TIPS[loading.tip_index].1
    );
    assert_eq!(
        loading.localized_current_resource().fallback,
        "DOWNLOADING Character Selection Data"
    );
    assert_eq!(
        loading.localized_step().fallback,
        "LOADING CHARACTER SELECTION ASSETS"
    );
}

#[test]
fn category_thirteen_switches_only_the_start_row_to_exact_cancel_copy() {
    let inactive = clean_category_service_entries(13, false);
    assert_eq!(
        inactive,
        vec![
            NpcServiceUiEntry::original(NpcServiceKind::Race),
            NpcServiceUiEntry::original(NpcServiceKind::RaceRank),
        ]
    );
    let active = clean_category_service_entries(13, true);
    assert_eq!(active[0].service, NpcServiceKind::Race);
    assert_eq!(active[0].label, " CANCEL RACE");
    assert_eq!(
        active[1],
        NpcServiceUiEntry::original(NpcServiceKind::RaceRank)
    );
    assert_eq!(
        clean_category_service_entries(14, true),
        vec![NpcServiceUiEntry::original(NpcServiceKind::RaceRank)]
    );
    assert!(clean_category_service_entries(17, true).is_empty());
}
