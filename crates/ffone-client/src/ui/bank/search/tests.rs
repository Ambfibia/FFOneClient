use super::*;

#[test]
fn bank_search_filters_localized_names_without_changing_authoritative_slots() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = crate::assets::AssetLocator::open(&root).unwrap();
    let content = TutorialMissionContent::open(&locator).unwrap();
    for locale in ["en", "ru"] {
        let (localization, language) = Localization::open(&root, locale).unwrap();
        let name = localization.text(
            &language,
            &content.gameplay_user_equip_item_text(7, 7).unwrap().0,
        );
        let mut app = App::new();
        app.init_resource::<BankSearch>()
            .insert_resource(BankUiState {
                phase: BankLifecyclePhase::Visible,
                ..default()
            })
            .init_resource::<BankModalState>()
            .insert_resource(content.clone())
            .insert_resource(localization)
            .insert_resource(language)
            .init_resource::<BankModeProjection0104>()
            .add_systems(Update, input);
        {
            let mut projection = app.world_mut().resource_mut::<BankModeProjection0104>();
            projection.owner_pc_id = 42;
            projection.npc_id = 815;
            for index in [5, 197] {
                projection.bank[index].empty = false;
                projection.bank[index].item = ItemBase0104 {
                    item_type: 7,
                    item_id: 7,
                    option: 3,
                    time_limit: 0,
                };
            }
        }
        app.update();
        app.world_mut().resource_mut::<BankSearch>().query = name.to_uppercase();
        // Input text and the snapshot can arrive on the same frame.
        app.world_mut()
            .resource_mut::<BankModeProjection0104>()
            .set_changed();
        app.world_mut().resource_mut::<BankUiState>().bank_scroll_y = 900.0;
        app.update();
        let search = app.world().resource::<BankSearch>();
        assert_eq!(search.visible, [5, 197]);
        assert_eq!(search.visible_index(197), Some(1));
        assert_eq!(search.maximum(), 0.0);
        assert_eq!(app.world().resource::<BankUiState>().bank_scroll_y, 0.0);
        assert_eq!(
            app.world().resource::<BankModeProjection0104>().bank[197].slot_index,
            197
        );
        app.world_mut()
            .resource_mut::<BankModeProjection0104>()
            .npc_id = 816;
        app.update();
        let search = app.world().resource::<BankSearch>();
        assert!(search.query.is_empty());
        assert_eq!(search.visible.len(), BANK_SLOT_COUNT_0104);
    }
}
