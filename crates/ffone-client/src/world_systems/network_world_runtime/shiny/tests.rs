use super::*;

#[test]
fn repeated_packets_keep_one_visual_and_network_despawn_removes_it() {
    use bevy::asset::AssetApp;
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()))
        .init_asset::<WorldAsset>()
        .init_asset::<Gltf>()
        .init_resource::<NetworkNpcVisualCatalogState0104>()
        .add_systems(Update, resolve);
    let mut catalog = super::super::NetworkNpcVisualCatalog0104::default();
    catalog.shinies.insert(
        1,
        Ok(ShinyVisualDefinition {
            name: "shineni_Item".to_owned(),
            glb: "characters/shinies/shineni_Item/shineni_Item.glb".to_owned(),
            pickup_effect: 397,
        }),
    );
    app.world_mut()
        .resource_mut::<NetworkNpcVisualCatalogState0104>()
        .catalog = Some(catalog);
    let appearance = NetworkShiny0104 {
        shiny_id: 81,
        shiny_type: 1,
        map_number: 0,
    };
    let root = app
        .world_mut()
        .spawn((appearance, Transform::IDENTITY, Visibility::Inherited))
        .id();
    app.update();
    let first = app.world().get::<ShinyVisual>(root).unwrap().container;
    app.world_mut().entity_mut(root).insert(appearance);
    app.update();
    assert_eq!(
        app.world().get::<ShinyVisual>(root).unwrap().container,
        first
    );
    assert_eq!(app.world().get::<Children>(root).unwrap().len(), 1);
    app.world_mut().entity_mut(root).insert(NetworkShiny0104 {
        shiny_type: -1,
        ..appearance
    });
    app.update();
    assert!(app.world().get_entity(first).is_err());
    assert!(app.world().get::<ShinyVisual>(root).is_none());
    assert!(app.world().get::<ShinyVisualIssue>(root).is_some());
    app.world_mut().entity_mut(root).insert(appearance);
    app.update();
    let replacement = app.world().get::<ShinyVisual>(root).unwrap().container;
    assert_ne!(first, replacement);
    assert!(app.world().get::<ShinyVisualIssue>(root).is_none());
    app.world_mut().entity_mut(root).despawn();
    assert!(app.world().get_entity(replacement).is_err());
}

#[test]
fn production_eggs_resolve_through_shiny_table_to_loadable_native_models() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let document: Value = crate::xdt::from_slice(
        &std::fs::read(root.join("data/tables/xdt.json")).unwrap(),
    )
    .unwrap();
    let registry =
        ffone_client_foundation::asset_tables::character_models_from_document(&document)
            .unwrap();
    let definitions = definitions(&document, &registry);
    for (kind, expected) in [
        (1, "shineni_Item"),
        (2, "shineni_Item"),
        (9, "shineni_buff"),
    ] {
        let definition = definitions[&kind].as_ref().unwrap();
        assert_eq!(definition.name, expected);
        assert_eq!(definition.pickup_effect, 397);
        let bytes = std::fs::read(root.join(&definition.glb)).unwrap();
        let length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
        let gltf: Value = serde_json::from_slice(&bytes[20..20 + length]).unwrap();
        assert!(
            gltf["animations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|a| a["name"] == "stand1")
        );
        let clip = &gltf["animations"][0];
        assert_eq!(clip["extras"]["loop"], true);
        let event = &clip["extras"]["nonTrs"]["events"][0];
        assert_eq!(event["functionName"], "particle");
        assert_eq!(
            event["stringParameter"],
            if expected == "shineni_Item" {
                "398"
            } else {
                "401"
            }
        );
        assert!((event["time"].as_f64().unwrap() - 0.1).abs() < 0.00001);
        for image in gltf["images"].as_array().unwrap() {
            assert!(
                root.join(&definition.glb)
                    .parent()
                    .unwrap()
                    .join(image["uri"].as_str().unwrap())
                    .is_file()
            );
        }
    }
    assert!(definitions[&0].is_err());
    assert!(!definitions.contains_key(&-1));
}

#[test]
fn shiny_root_preserves_scale_without_character_half_turn() {
    let authored = Transform::from_xyz(3.0, 4.0, 5.0).with_scale(Vec3::splat(0.7));
    assert_eq!(
        LegacyCharacterRootPolicy::Shiny.resolve_root(authored),
        Transform::from_scale(Vec3::splat(0.7))
    );
    assert_eq!(
        crate::character_scene::native_scene_container_transform(
            NativeSceneRole::WorldAuthored
        ),
        Transform::IDENTITY
    );
}

#[test]
fn animation_waits_for_gltf_and_starts_once() {
    let mut app = App::new();
    app.init_resource::<Assets<Gltf>>()
        .init_resource::<Assets<AnimationGraph>>()
        .add_systems(Update, animate);
    let handle = app.world().resource::<Assets<Gltf>>().reserve_handle();
    let root = app.world_mut().spawn_empty().id();
    app.world_mut().entity_mut(root).insert(ShinyVisual {
        shiny_type: 1,
        container: root,
        gltf: handle.clone(),
        name: "shineni_Item".into(),
    });
    let entity = app
        .world_mut()
        .spawn((AnimationPlayer::default(), ChildOf(root)))
        .id();
    app.update();
    app.update();
    assert!(app.world().get::<PendingShinyAnimation>(entity).is_some());
    assert!(app.world().get::<AnimationGraphHandle>(entity).is_none());
    let clip = Handle::<AnimationClip>::default();
    let gltf = Gltf {
        scenes: vec![],
        named_scenes: Default::default(),
        meshes: vec![],
        named_meshes: Default::default(),
        materials: vec![],
        named_materials: Default::default(),
        nodes: vec![],
        named_nodes: Default::default(),
        skins: vec![],
        named_skins: Default::default(),
        default_scene: None,
        source: None,
        animations: vec![clip.clone()],
        named_animations: [("stand1".into(), clip)].into_iter().collect(),
    };
    app.world_mut()
        .resource_mut::<Assets<Gltf>>()
        .insert(handle.id(), gltf)
        .unwrap();
    app.update();
    assert!(app.world().get::<PendingShinyAnimation>(entity).is_none());
    let graph = app
        .world()
        .get::<AnimationGraphHandle>(entity)
        .unwrap()
        .0
        .clone();
    app.update();
    assert_eq!(
        app.world().get::<AnimationGraphHandle>(entity).unwrap().0,
        graph
    );
}
