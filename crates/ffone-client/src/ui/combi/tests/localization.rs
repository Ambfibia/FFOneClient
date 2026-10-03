use super::*;

#[test]
fn production_combi_tree_attaches_key_first_localization_to_every_text() {
    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(CombiUiPlugin);
    app.update();

    let assets = app.world().resource::<CombiUiAssets>().clone();
    let world = app.world_mut();
    let mut texts = world.query::<(&Text, &LocalizedText, (&TextFont, &LineHeight))>();
    let text_rows: Vec<_> = texts
        .iter(world)
        .map(|(text, localized, font)| {
            (text.0.clone(), localized.key.clone(), font.0.font.clone())
        })
        .collect();
    assert!(!text_rows.is_empty());
    assert_eq!(
        text_rows.len(),
        world.query::<&Text>().iter(world).count(),
        "a Combi Text entity bypassed LocalizedText"
    );
    for expected_key in [
        "ui.combi.title",
        "ui.combi.intro",
        "ui.combi.style",
        "ui.combi.stats",
        "ui.combi.cost",
        "ui.combi.taros",
        "ui.combi.chance.title",
        "ui.combi.chance.not_ready",
        "ui.combi.chance.level",
        "ui.combi.new_combined_item",
        "ui.combi.look.empty",
        "ui.combi.item.level",
        "ui.combi.item.stat_value",
        "ui.combi.info",
        "ui.combi.info.type",
        "ui.combi.info.range",
        "ui.combi.info.rarity",
        "ui.combi.info.trade_availability",
        "ui.combi.stats.empty",
        "ui.combi.clear_all",
        "ui.combi.combine",
        "ui.combi.inventory.equipment",
        "ui.combi.success.hooray",
        "ui.combi.success.message",
        "ui.combi.success.combine_more",
        "ui.combi.success.go_to_my_stuff",
    ] {
        assert!(
            text_rows.iter().any(|(_, key, _)| key == expected_key),
            "missing semantic Combi text role {expected_key}"
        );
    }
    for (text, key, font) in text_rows {
        assert!(!key.is_empty(), "empty localization key for {text:?}");
        assert!(
            font == bevy::text::FontSource::Handle(assets.jeffe_font.clone())
                || font == bevy::text::FontSource::Handle(assets.chalet_font.clone()),
            "Combi text stopped using one of the two validated replacement fonts"
        );
    }
}

#[test]
fn source_audit_keeps_text_creation_behind_localized_helpers() {
    let source = concat!(
        include_str!("../constants.rs"),
        "\n",
        include_str!("../containers.rs"),
        "\n",
        include_str!("../assets.rs"),
        "\n",
        include_str!("../state.rs"),
        "\n",
        include_str!("../interaction.rs"),
        "\n",
        include_str!("../frame.rs"),
        "\n",
        include_str!("../commands.rs"),
        "\n",
        include_str!("../validation.rs"),
        "\n",
        include_str!("../layout.rs"),
        "\n",
        include_str!("../types_combi_authoritative_snapshot0104.rs"),
        "\n",
        include_str!("../types_combi_machine0104.rs"),
        "\n",
        include_str!("../input_find_combining_arrays.rs"),
        "\n",
        include_str!("../operations_classify_clean_client_chance.rs"),
        "\n",
        include_str!("../binding_combi_ui.rs"),
        "\n",
        include_str!("../projection.rs"),
        "\n",
        include_str!("../localization_bind_optional_localized_text.rs"),
        "\n",
        include_str!("../view_combi_main_group.rs"),
        "\n",
        include_str!("../view_combi_overlays.rs"),
        "\n",
        include_str!("../mod.rs")
    ).split("#[cfg(test)]").next().unwrap();
    let text_constructor = ["Text", "::new("].concat();
    let text_constructors: Vec<_> = source
        .match_indices(&text_constructor)
        .filter(|(offset, _)| {
            *offset == 0
                || !source.as_bytes()[offset - 1].is_ascii_alphanumeric()
                    && source.as_bytes()[offset - 1] != b'_'
        })
        .collect();
    assert_eq!(
        text_constructors.len(),
        2,
        "new Combi Text must go through spawn_combi_text or spawn_combi_button"
    );
    assert!(source.contains(".before(LocalizationSet::Apply)"));
    for (offset, _) in text_constructors {
        let tail = &source[offset..source.len().min(offset + 160)];
        assert!(
            tail.contains("localized"),
            "Text constructor at byte {offset} has no adjacent LocalizedText component"
        );
    }
    let raw_metadata_constructor = ["Text", "::new(metadata."].concat();
    for forbidden in [
        "bind_optional_text(".to_owned(),
        "bind_text(text".to_owned(),
        "format!(\"LEVEL {}\"".to_owned(),
        raw_metadata_constructor,
    ] {
        assert!(
            !source.contains(&forbidden),
            "Combi source bypasses keyed dynamic localization via {forbidden:?}"
        );
    }
    let binder = &source[source.find("fn bind_combi_ui").unwrap()
        ..source.find("fn bind_dynamic_icon").unwrap()];
    assert!(!binder.contains("Mut<Text>"));
    assert!(!binder.contains("text.0"));
}
