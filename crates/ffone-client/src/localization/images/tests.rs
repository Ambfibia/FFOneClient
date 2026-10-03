use super::*;

fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game")
}

#[test]
fn production_translations_preserve_dimensions_and_have_english_fallback() {
    let root = root();
    let catalog = UiImageCatalog::open(&root).unwrap();
    for required in [
        "character/selection/controls/musicon.png",
        "character/selection/controls/musicoff.png",
    ] {
        assert!(catalog.russian.contains(required));
    }
    for relative in &catalog.russian {
        let en = format!("ui/en/{relative}");
        let ru = format!("ui/ru/{relative}");
        assert_eq!(
            image::image_dimensions(root.join(&en)).unwrap(),
            image::image_dimensions(root.join(&ru)).unwrap()
        );
        assert_eq!(catalog.resolve(&en, "ru"), Some(ru.clone()));
        assert_eq!(catalog.resolve(&ru, "en"), Some(en));
        assert_eq!(catalog.resolve(&ru, "ru"), None);
    }
    assert_eq!(catalog.resolve("ui/en/option/panel.png", "ru"), None);
    assert_eq!(catalog.resolve("icons/npcs/example.png", "ru"), None);
}

#[test]
fn language_switch_and_state_writer_use_translated_handles_without_idle_writes() {
    let root = root();
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin {
            file_path: root.to_string_lossy().into_owned(),
            ..default()
        },
    ))
    .init_asset::<Image>()
    .insert_resource(AssetLocator::open(&root).unwrap())
    .insert_resource(Language {
        requested: "ru".into(),
        effective: "ru".into(),
    });
    register(&mut app);
    let on = "ui/en/character/selection/controls/musicon.png";
    let off = "ui/en/character/selection/controls/musicoff.png";
    let handle = app.world().resource::<AssetServer>().load(on);
    let entity = app.world_mut().spawn(ImageNode::new(handle)).id();
    app.update();
    let path = |app: &App| {
        app.world()
            .get::<ImageNode>(entity)
            .unwrap()
            .image
            .path()
            .unwrap()
            .path()
            .to_string_lossy()
            .into_owned()
    };
    assert_eq!(path(&app), on.replacen("/en/", "/ru/", 1));
    let tick = app
        .world()
        .entity(entity)
        .get_ref::<ImageNode>()
        .unwrap()
        .last_changed();
    app.update();
    assert_eq!(
        app.world()
            .entity(entity)
            .get_ref::<ImageNode>()
            .unwrap()
            .last_changed(),
        tick
    );
    let handle = app.world().resource::<AssetServer>().load(off);
    app.world_mut().get_mut::<ImageNode>(entity).unwrap().image = handle;
    app.update();
    assert_eq!(path(&app), off.replacen("/en/", "/ru/", 1));
    app.world_mut().resource_mut::<Language>().effective = "en".into();
    app.update();
    assert_eq!(path(&app), off);
}
