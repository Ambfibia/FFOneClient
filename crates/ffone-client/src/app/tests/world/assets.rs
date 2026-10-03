use super::*;

#[test]
fn option_route_uses_live_clean_mappings_and_consumes_only_its_press() {
    let mut input = InputSettings::default();
    let mut keyboard = ButtonInput::<KeyCode>::default();
    let mut mouse = ButtonInput::<MouseButton>::default();
    keyboard.press(KeyCode::Quote);
    keyboard.press(KeyCode::KeyW);

    assert!(option_action_just_pressed(
        &input,
        LegacyOptionAction::Option,
        &keyboard,
        &mouse,
    ));
    clear_option_action_press(
        &input,
        LegacyOptionAction::Option,
        &mut keyboard,
        &mut mouse,
    );
    assert!(!keyboard.just_pressed(KeyCode::Quote));
    assert!(keyboard.just_pressed(KeyCode::KeyW));

    let row = input
        .mappings
        .iter_mut()
        .find(|row| row.action == LegacyOptionAction::Option)
        .unwrap();
    row.primary = LegacyInputBinding::Key(LegacyPhysicalKey::Mouse1);
    row.alternate = LegacyInputBinding::Unbound;
    mouse.press(MouseButton::Right);
    assert!(option_action_just_pressed(
        &input,
        LegacyOptionAction::Option,
        &keyboard,
        &mouse,
    ));
}

#[test]
fn unpaid_past_warp_opens_only_the_clean_level_four_news_route() {
    let service = guide_service_entry(23, Some(0)).expect("category 23 must expose Past Warp");
    assert_eq!(service.service, NpcServiceKind::PastWarp);

    let mut model = UpsellUiModel::default();
    open_unpaid_past_warp_upsell(&mut model).unwrap();

    assert!(model.visible());
    assert_eq!(model.level(), Some(4));
    assert_eq!(model.active_mode(), Some(UpsellUiMode::NewsFreeZone));
    assert_eq!(model.news_page_count(), 0);
    assert_eq!(model.current_news_page_path(), None);
}

#[test]
fn npc_icon_rule_service_uses_numeric_table_route_and_page_label() {
    let content = runtime_test_mission_content();
    let vehicle = world_npc_service_entries(&content, 650, 0, Some(9), None, false);
    assert_eq!(
        vehicle.last(),
        Some(&NpcServiceUiEntry {
            service: NpcServiceKind::Rule,
            label: " WHAT ARE VEHICLES?".to_owned(),
        })
    );
    let combining = world_npc_service_entries(&content, 650, 0, Some(10), None, false);
    assert_eq!(
        combining.last(),
        Some(&NpcServiceUiEntry {
            service: NpcServiceKind::Rule,
            label: " WHAT IS COMBINING?".to_owned(),
        })
    );
    assert!(
        world_npc_service_entries(&content, 650, 0, Some(8), None, false)
            .iter()
            .all(|entry| entry.service != NpcServiceKind::Rule)
    );
}

#[test]
fn npc_warp_level_gate_uses_maintained_en_ru_message_key() {
    let content = runtime_test_mission_content();
    let mut runtime = NormalNpcWarpRuntime::default();
    let mut messages = SystemMessageUiModel::default();
    runtime.queue_system_message(&content, &mut messages, 111).unwrap();
    let request = messages.current().unwrap();
    assert_eq!(request.localized.key, "content.tabledata.message.message.111.sz_string");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let (localization, ru) = Localization::open(&root, "ru").unwrap();
    let (_, en) = Localization::open(&root, "en").unwrap();
    assert_eq!(localization.text(&en, &request.localized), "You need to be a higher level before traveling to this destination.");
    assert!(localization.text(&ru, &request.localized).contains("уровня"));
}
