#[test]
fn unavailable_help_subtopics_hide_the_button_not_only_its_text() {
    use super::*;
    let mut app = App::new();
    let mut model = GameGuideUiModel::default();
    assert!(model.open_first_use(24));
    app.insert_resource(model)
        .init_resource::<GameGuideCatalog>()
        .add_systems(Update, sync_game_guide_labels);
    let shown = app
        .world_mut()
        .spawn((GameGuideControl::Sub(1), Node::default()))
        .id();
    let hidden = app
        .world_mut()
        .spawn((GameGuideControl::Sub(2), Node::default()))
        .id();
    app.update();
    assert_eq!(
        app.world().get::<Node>(shown).unwrap().display,
        Display::Flex
    );
    assert_eq!(
        app.world().get::<Node>(hidden).unwrap().display,
        Display::None
    );
}
#[test]
fn service_help_routes_absolute_pages_and_preserves_invalid_selection() {
    let mut model = super::GameGuideUiModel::default();
    let mut world = bevy::prelude::World::new();
    use bevy::prelude::FromWorld;
    let catalog = super::GameGuideCatalog::from_world(&mut world);
    for (event, page) in [(15, 23), (24, 25), (44, 5)] {
        model.content_scroll_y = 200.0;
        assert!(model.open_first_use(event));
        assert!(model.visible);
        assert_eq!(super::current_page_id(&catalog, &model), Some(page));
        assert_eq!(model.content_scroll_y, 0.0);
    }
    let before = model.clone();
    assert!(!model.open_first_use(usize::MAX));
    assert!(!model.open_first_use(0));
    assert_eq!(before, model);
}
use super::*;

#[test]
fn nanocom_game_guide_opens_help_initial_page_not_mentor_selection() {
    let mut model = GameGuideUiModel::default();
    model.selected_main_topic = 6;
    model.selected_sub_topic = 3;
    model.content_scroll_y = 90.0;
    model.open_from_nanocom();
    assert!(model.visible);
    assert_eq!(model.selected_main_topic, 1);
    assert_eq!(model.selected_sub_topic, 0);
    assert_eq!(model.content_scroll_y, 0.0);
}

#[test]
fn native_help_table_still_has_all_clean_topics_and_initial_content() {
    let root: Value = crate::xdt::from_slice(include_bytes!(
        "../../../../../assets/game/data/tables/xdt.json"
    ))
    .unwrap();
    let help = &root["tables"][0]["value"]["m_pHelpTable"];
    assert_eq!(help["m_pHelpData"].as_array().unwrap().len(), 8);
    assert_eq!(help["m_pHelpPageString"][1]["m_strName"], "INTRODUCTION");
    assert_eq!(help["m_pHelpPageString"][3]["m_strComment"], "Login.jpg");
}

#[test]
fn clean_help_geometry_preserves_three_subtitle_columns_and_image_rect() {
    assert_eq!(MAX_SUB_TOPICS, 15);
    assert_eq!(
        GAME_GUIDE_SUB_BUTTON_RECT,
        GuideUiRect::new(200.0, 78.0, 256.0, 16.0)
    );
    assert_eq!(
        GAME_GUIDE_CONTENT_VIEW_RECT,
        GuideUiRect::new(199.0, 200.0, 810.0, 380.0)
    );
    assert_eq!(GAME_GUIDE_SCREENSHOT_RECT.width, 350.0);
    assert_eq!(GAME_GUIDE_SCREENSHOT_RECT.height, 262.0);
    assert_eq!(GAME_GUIDE_HELP_BUTTON_TEXT_Y_OFFSET, 3.0);
    assert!((GAME_GUIDE_NAV_BUTTON_TEXT_Y_OFFSET - 5.83).abs() < f32::EPSILON);
}

#[test]
fn screenshot_routes_preserve_distinct_space_and_underscore_names() {
    assert_eq!(
        screenshot_asset_path("mission giver.jpg").as_deref(),
        Some("ui/en/gameplay/game-guide/screenshots/mission-giver.png")
    );
    assert_eq!(
        screenshot_asset_path("mission_giver.jpg").as_deref(),
        Some("ui/en/gameplay/game-guide/screenshots/mission_giver.png")
    );
    assert_eq!(screenshot_asset_path("lteminfo.jpg"), None);
}

#[test]
fn every_resolved_clean_help_screenshot_is_published() {
    let root: Value = crate::xdt::from_slice(include_bytes!(
        "../../../../../assets/game/data/tables/xdt.json"
    ))
    .unwrap();
    let help = &root["tables"][0]["value"]["m_pHelpTable"];
    let strings = help["m_pHelpPageString"].as_array().unwrap();
    let assets_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    let mut unresolved = Vec::new();
    for entry in help["m_pHelpPageDescData"].as_array().unwrap() {
        if entry["m_iType"].as_u64() != Some(2) {
            continue;
        }
        let string_id = entry["m_iString"].as_u64().unwrap() as usize;
        let filename = strings[string_id]["m_strComment"].as_str().unwrap();
        match screenshot_asset_path(filename) {
            Some(path) => assert!(
                assets_root.join(&path).is_file(),
                "missing published Game Guide screenshot {path}"
            ),
            None => unresolved.push(filename),
        }
    }
    assert_eq!(unresolved, ["lteminfo.jpg"]);
}
