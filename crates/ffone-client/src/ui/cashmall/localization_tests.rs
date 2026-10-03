use crate::cashmall_ui::*;
use bevy::asset::AssetPlugin;
use tempfile::tempdir;

#[test]
fn production_cashmall_tree_attaches_key_first_localization_to_every_text() {
    let asset_root = tempdir().expect("temporary asset root");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(CashmallUiPlugin0104);
    app.update();

    let expected_font = app.world().resource::<CashmallUiAssets0104>().font.clone();
    let world = app.world_mut();
    let total_text = world.query::<&Text>().iter(world).count();
    let mut localized_text = world.query::<(&Text, &LocalizedText, &TextFont)>();
    let rows = localized_text.iter(world).collect::<Vec<_>>();
    assert!(!rows.is_empty());
    assert_eq!(
        rows.len(),
        total_text,
        "a Cash Mall Text entity bypassed LocalizedText"
    );
    for (text, localized, font) in rows {
        assert!(
            !localized.key.is_empty(),
            "empty localization key for {:?}",
            text.0
        );
        assert_eq!(
            font.font,
            bevy::text::FontSource::Handle(expected_font.clone())
        );
    }
}

#[test]
fn dynamic_cashmall_copy_is_template_and_argument_driven() {
    let passthrough = cashmall_passthrough_text_0104("Server item");
    assert_eq!(passthrough.key, "ui.content.passthrough");
    assert_eq!(passthrough.fallback, "{text}");
    assert_eq!(passthrough.args["text"], "Server item");

    let level = cashmall_item_level_text_0104(17);
    assert_eq!(level.key, "ui.cashmall.item.level");
    assert_eq!(level.fallback, "LEVEL {level}");
    assert_eq!(level.args["level"], "17");

    let weapon = cashmall_equipment_slot_localized_text_0104(6);
    assert_eq!(weapon.key, "ui.cashmall.equipment.slot.weapon");
    assert_eq!(weapon.fallback, "WEAPON {ordinal}");
    assert_eq!(weapon.args["ordinal"], "1");
    assert_eq!(cashmall_resolved_fallback_0104(&weapon), "WEAPON 1");
}
