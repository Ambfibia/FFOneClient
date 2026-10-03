use crate::enchant_ui::*;
use bevy::asset::AssetPlugin;
use tempfile::tempdir;

#[test]
fn dynamic_enchant_copy_uses_semantic_templates_and_named_arguments() {
    let passthrough = enchant_passthrough_text_0104("server item name");
    assert_eq!(passthrough.key, "ui.content.passthrough");
    assert_eq!(passthrough.fallback, "{text}");
    assert_eq!(
        passthrough.args.get("text").map(String::as_str),
        Some("server item name")
    );

    let cost = enchant_cost_text_0104(12_345);
    assert_eq!(cost.key, "ui.enchant.cost");
    assert_eq!(cost.fallback, "{cost}");
    assert_eq!(cost.args.get("cost").map(String::as_str), Some("12345"));

    let level = enchant_item_level_text_0104(17);
    assert_eq!(level.key, "ui.enchant.item.level");
    assert_eq!(level.fallback, "LEVEL {level}");
    assert_eq!(level.args.get("level").map(String::as_str), Some("17"));

    let chance = enchant_chance_text_0104(EnchantChance0104::VeryHigh);
    assert_eq!(chance.key, "ui.enchant.chance.very_high");
    assert_eq!(chance.fallback, "5(Very High)");
    assert!(chance.args.is_empty());

    let weapon_two = enchant_equipment_slot_text_0104(7);
    assert_eq!(weapon_two.key, "ui.enchant.equipment.slot.weapon");
    assert_eq!(weapon_two.fallback, "WEAPON {ordinal}");
    assert_eq!(
        weapon_two.args.get("ordinal").map(String::as_str),
        Some("2")
    );

    let redeem_error = EnchantPopupIntent0104::RedeemCodeSpaceError {
        message: ENCHANT_REDEEM_SPACE_ERROR_TEXT_0104,
    }
    .localized_message()
    .unwrap();
    assert_eq!(redeem_error.key, "ui.enchant.redeem.space_error");
    assert_eq!(redeem_error.fallback, ENCHANT_REDEEM_SPACE_ERROR_TEXT_0104);
    assert!(redeem_error.args.is_empty());
}

#[test]
fn pending_item_icon_is_owned_until_loading_or_selection_cancellation() {
    #[derive(Resource)]
    struct PendingPath(Option<&'static str>);
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .insert_resource(PendingPath(Some("pending-item.png")))
        .add_systems(
            Update,
            |server: Res<AssetServer>,
             path: Res<PendingPath>,
             mut icons: Query<(&mut Node, &mut ImageNode)>| {
                for (mut node, image) in &mut icons {
                    enchant_bind_dynamic_icon_0104(
                        &mut node,
                        Some(image),
                        path.0,
                        1.0,
                        &server,
                    );
                }
            },
        );
    let entity = app
        .world_mut()
        .spawn((Node::default(), ImageNode::default()))
        .id();
    app.update();
    let image = app.world().get::<ImageNode>(entity).unwrap();
    assert_ne!(image.image, Handle::<Image>::default());
    assert_eq!(
        app.world().get::<Node>(entity).unwrap().display,
        Display::None
    );
    assert_eq!(
        app.world()
            .resource::<AssetServer>()
            .get_path(image.image.id())
            .unwrap()
            .path(),
        std::path::Path::new("pending-item.png")
    );
    app.world_mut().resource_mut::<PendingPath>().0 = None;
    app.update();
    assert_eq!(
        app.world().get::<ImageNode>(entity).unwrap().image,
        Handle::<Image>::default()
    );
}

#[test]
fn every_spawned_enchant_text_entity_is_key_first() {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(EnchantUiPlugin0104);
    app.update();

    let world = app.world_mut();
    let mut all_text =
        world.query_filtered::<(Entity, Option<&LocalizedText>, &Text), With<Text>>();
    let rows = all_text.iter(world).collect::<Vec<_>>();
    assert_eq!(
        rows.len(),
        125,
        "native enchant hierarchy text count changed"
    );
    for (entity, localized, _) in rows {
        assert!(
            localized.is_some(),
            "enchant Text entity {entity:?} is missing LocalizedText"
        );
    }
    // Interaction uses Bevy UI focus, which is independent of Pickable.
    // Every decorative child must let a click reach the owning button.
    let mut decorations = world.query::<(&Pickable, Option<&bevy::ui::FocusPolicy>)>();
    for (pickable, focus) in decorations.iter(world) {
        if *pickable == Pickable::IGNORE {
            assert_eq!(focus, Some(&bevy::ui::FocusPolicy::Pass));
        }
    }
}
