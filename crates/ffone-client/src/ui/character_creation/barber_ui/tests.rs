use super::*;
#[test]
fn every_barber_label_has_a_production_key_in_both_languages() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .init_asset::<Font>()
        .init_asset::<AudioSource>()
        .add_plugins(BarberUiPlugin);
    app.world_mut()
        .resource_mut::<BarberModel>()
        .request_open(8);
    app.update();
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game/localization");
    let en: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("en.json")).unwrap()).unwrap();
    let ru: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("ru.json")).unwrap()).unwrap();
    let world = app.world_mut();
    let mut query = world.query::<(&Text, Option<&LocalizedText>)>();
    let mut count = 0;
    for (_, localized) in query.iter(world) {
        let localized = localized.expect("Barber text must be keyed");
        assert!(
            en["entries"].get(&localized.key).is_some(),
            "{}",
            localized.key
        );
        assert!(
            ru["entries"].get(&localized.key).is_some(),
            "{}",
            localized.key
        );
        count += 1;
    }
    assert!(count >= 20);
}
