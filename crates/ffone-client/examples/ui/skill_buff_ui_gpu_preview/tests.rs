use super::*;

#[test]
fn preview_exercises_all_three_clean_rows_without_local_stim_icons() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    let project_assets = AssetLocator::open(&asset_root).unwrap();
    let content = TutorialMissionContent::open(&project_assets).unwrap();
    let catalog = SkillBuffUiCatalog::open(&content, &project_assets).unwrap();
    let view = skill_buff_ui_view(CLIENT_AREA_WIDTH, &sample_model(), &catalog);
    assert_eq!(view.local.len(), 7);
    assert_eq!(view.cash.len(), 2);
    assert_eq!(view.target.len(), 1);
    assert_eq!(view.cash[0].cash_time.as_deref(), Some("2m"));
    assert_eq!(view.cash[1].cash_time.as_deref(), Some("1h"));
    assert_eq!(view.target[0].buff_id, 21);
    assert_eq!(view.target[0].icon_number, 66);
}
