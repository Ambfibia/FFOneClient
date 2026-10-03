use super::*;

fn fixture() -> (App, Entity) {
    let mut app = App::new();
    app.init_resource::<UserEquipUiState>()
        .init_resource::<UserEquipItemModeProjection>()
        .init_resource::<UserEquipModalState>()
        .init_resource::<SharedInputDialog>()
        .init_resource::<SharedRedeemCode>()
        .init_resource::<UserEquipUiAudioOutbox>()
        .add_systems(Update, collect_actions);
    app.world_mut()
        .resource_mut::<UserEquipItemModeProjection>()
        .owner_pc_id = 42;
    let mut state = app.world_mut().resource_mut::<UserEquipUiState>();
    state.open_item_mode();
    state.tick(1.0);
    let button = app
        .world_mut()
        .spawn((RedeemControl, Interaction::Pressed))
        .id();
    (app, button)
}

#[test]
fn inventory_redeem_opens_for_its_character_and_blocks_other_controls() {
    let (mut app, _) = fixture();
    app.update();
    assert_eq!(
        app.world().resource::<SharedRedeemCode>().source,
        Some(RedeemSource::Inventory { pc: 42 })
    );
    let modal = *app.world().resource::<UserEquipModalState>();
    assert!(modal.redeem_code_view);
    assert!(
        !app.world()
            .resource::<UserEquipUiState>()
            .input_capabilities(modal)
            .panel_controls
    );
}

#[test]
fn inventory_redeem_cannot_replace_another_dialog_or_bypass_a_modal() {
    for blocked_by_dialog in [false, true] {
        let (mut app, _) = fixture();
        if blocked_by_dialog {
            let mut input = app.world_mut().resource_mut::<SharedInputDialog>();
            SharedRedeemCode::default()
                .open(RedeemSource::Bank { pc: 42, npc: 815 }, &mut input);
        } else {
            app.world_mut()
                .resource_mut::<UserEquipModalState>()
                .system_popup_active = true;
        }
        app.update();
        assert!(app.world().resource::<SharedRedeemCode>().source.is_none());
        assert!(
            !app.world()
                .resource::<UserEquipModalState>()
                .redeem_code_view
        );
    }
}

#[test]
fn inventory_redeem_label_exists_in_both_production_bundles() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game/localization");
    for (locale, expected) in [("en", "REDEEM CODE"), ("ru", "АКТИВИРОВАТЬ КОД")]
    {
        let bundle: serde_json::Value = serde_json::from_slice(
            &std::fs::read(root.join(format!("{locale}.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(bundle["entries"]["ui.inventory.redeem_code"], expected);
    }
}
