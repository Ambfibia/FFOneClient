use super::*;

#[test]
fn mesh_effect_lifetime_begins_only_after_a_renderable_surface_exists() {
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<TutorialEffectRuntime>()
        .add_systems(Update, cleanup_effect_roots);
    let name = "warp.departure";
    let instance_id = app
        .world_mut()
        .resource_mut::<TutorialEffectRuntime>()
        .allocate_instance(false, Some(name.to_owned()));
    let root = NativeEffectRoot {
        stream_owner: None,
        age: 0.0,
        destroy_after: Some(2.0),
        natural_destroy_after: Some(2.0),
        material_animation: None,
        waiting_for_mesh_surface: true,
    };
    let entity = app
        .world_mut()
        .spawn((NativeRoot { instance_id }, root))
        .id();
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs(3));
    app.update();
    assert_eq!(
        app.world().get::<NativeEffectRoot>(entity).unwrap().age,
        0.0
    );
    assert!(
        !app.world()
            .resource::<TutorialEffectRuntime>()
            .named_native_presentation_ready(name)
    );

    app.world_mut()
        .get_mut::<NativeEffectRoot>(entity)
        .unwrap()
        .waiting_for_mesh_surface = false;
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(0.25));
    app.update();
    assert_eq!(
        app.world().get::<NativeEffectRoot>(entity).unwrap().age,
        0.25
    );
    assert!(
        app.world()
            .resource::<TutorialEffectRuntime>()
            .named_native_presentation_ready(name)
    );
}

#[test]
fn sword_trail_mesh_preserves_retrobution_vertex_fade_and_index_tail() {
    let edges = vec![NativeSwordTrailEdge::default(); RETROBUTION_SWORD_TRAIL_LENGTH];
    let mesh = native_sword_trail_mesh(&edges);
    assert_eq!(mesh.count_vertices(), 60);
    let Indices::U32(indices) = mesh.indices().unwrap() else {
        panic!("sword trail did not use the original 32-bit index buffer");
    };
    assert_eq!(indices.len(), 180);
    assert_eq!(&indices[174..], &[0, 0, 0, 0, 0, 0]);
    let Some(bevy::mesh::VertexAttributeValues::Float32x4(colors)) =
        mesh.attribute(Mesh::ATTRIBUTE_COLOR)
    else {
        panic!("sword trail color attribute is missing");
    };
    assert_eq!(colors[0], [0.5, 0.5, 0.5, 0.5]);
    assert_eq!(colors[1], [0.0, 0.0, 0.0, 0.0]);
    assert_eq!(colors[58], [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(colors[59], [0.0, 0.0, 0.0, 0.0]);
}

#[test]
fn mesh_effect_queue_is_submitted_after_character_queue() {
    assert_eq!(mesh_effect_render_queue_sort_bias(2_000), -1_843_200.0);
    assert_eq!(mesh_effect_render_queue_sort_bias(2_900), 0.0);
    assert_eq!(mesh_effect_render_queue_sort_bias(3_000), 204_800.0);
    assert_eq!(mesh_effect_render_queue_sort_bias(3_011), 227_328.0);
}

#[test]
fn es31_grenade_uses_the_exact_looping_native_mesh_with_provenance() {
    const SCENE: &str =
        "map/shared/projectiles/effects/models/es31/models/projectile/EnergyGrenades_e01.glb";
    const GLB_BLAKE3: &str = "4089e7039ee472055eac149a5bb73f5e3831d487edf00684c30fc97bde47adaf";
    let asset_root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let library = TutorialEffectLibrary::load(&asset_root).unwrap();
    let compiled = compile_projectile_plan(31, library.projectile_effects.get(&31).unwrap());
    let plan = compiled
        .plan
        .as_ref()
        .unwrap_or_else(|| panic!("projectile 31 blockers: {:#?}", compiled.blockers));

    assert_eq!(plan.rendered_nodes, 1);
    assert!(plan.trail.is_none());
    assert_eq!(plan.mesh_scene, Some(SCENE));
    assert!(plan.material_animation.is_none());
    assert!(projectile_mesh_repeats_standard_animation(31));
    assert!(
        compiled.blockers.is_empty(),
        "ES31 retained legacy blockers: {:#?}",
        compiled.blockers
    );

    let glb = std::fs::read(asset_root.join(SCENE)).unwrap();
    assert_eq!(blake3::hash(&glb).to_hex().as_str(), GLB_BLAKE3);
    let report: JsonValue = serde_json::from_slice(
        &std::fs::read(asset_root.join(
            "map/shared/projectiles/effects/models/es31/models/projectile/EnergyGrenades_e01.publish.json",
        ))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        report
            .pointer("/contract/legacy_name")
            .and_then(JsonValue::as_str),
        Some("EnergyGrenades_e01")
    );
    assert_eq!(
        report
            .pointer("/contract/glb_blake3")
            .and_then(JsonValue::as_str),
        Some(GLB_BLAKE3)
    );
    assert_eq!(
        report
            .pointer("/contract/source/animation_channels")
            .and_then(JsonValue::as_u64),
        Some(1)
    );
    assert_eq!(
        report
            .pointer("/contract/source/animation_keyframes")
            .and_then(JsonValue::as_u64),
        Some(7)
    );
    assert_eq!(
        report
            .pointer("/semanticProof/matched")
            .and_then(JsonValue::as_bool),
        Some(true)
    );

    let source = library
        .projectile_catalog()
        .particle_effects
        .iter()
        .find(|effect| effect.effect_id == 31)
        .unwrap();
    assert_eq!(
        source.container_route,
        "prefabs/particle/effectscripts/es[31].prefab"
    );
    assert_eq!(source.root_path_id, 10_519);
    assert_eq!(
        source.closure_blake3,
        "d30a6eef1287f19ae76db3d994768c6e3a58cd68ec10c3b6380378126aa2ae06"
    );
}

#[test]
fn es718_tutorial_pistol_uses_the_exact_looping_native_mesh_with_provenance() {
    const SCENE: &str = "map/shared/projectiles/effects/models/es718/sonic_ware_ta_m01.glb";
    const GLB_BLAKE3: &str = "e055498add1f3a742b12cbaf6d4707cf7580c08cf9610bb255de1dbc3ab75980";
    const STANDARD_ANIMATION_SHA256: &str =
        "e0765be5c2280ec0e4b97cde24de17e9b3bded1e5c46442ddebf7ea6a37511a2";
    let asset_root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let library = TutorialEffectLibrary::load(&asset_root).unwrap();
    let compiled = compile_projectile_plan(718, library.projectile_effects.get(&718).unwrap());
    let plan = compiled
        .plan
        .as_ref()
        .unwrap_or_else(|| panic!("projectile 718 blockers: {:#?}", compiled.blockers));

    assert_eq!(plan.rendered_nodes, 1);
    assert!(
        plan.trail.is_none(),
        "ES718 is a point-emitter projectile and must not fabricate a GEN_TRAIL"
    );
    assert_eq!(plan.mesh_scene, Some(SCENE));
    assert!(plan.material_animation.is_none());
    assert!(projectile_mesh_repeats_standard_animation(718));
    assert!(
        compiled.blockers.is_empty(),
        "ES718 retained legacy blockers: {:#?}",
        compiled.blockers
    );

    let glb = std::fs::read(asset_root.join(SCENE)).unwrap();
    assert_eq!(blake3::hash(&glb).to_hex().as_str(), GLB_BLAKE3);
    let report: JsonValue = serde_json::from_slice(
        &std::fs::read(asset_root.join(
            "map/shared/projectiles/effects/models/es718/sonic_ware_ta_m01.publish.json",
        ))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        report
            .pointer("/contract/legacy_name")
            .and_then(JsonValue::as_str),
        Some("sonic_ware_ta_m01")
    );
    assert_eq!(
        report
            .pointer("/contract/glb_blake3")
            .and_then(JsonValue::as_str),
        Some(GLB_BLAKE3)
    );
    assert_eq!(
        report
            .pointer("/contract/source/animation_clips")
            .and_then(JsonValue::as_u64),
        Some(1)
    );
    assert_eq!(
        report
            .pointer("/contract/source/animation_channels")
            .and_then(JsonValue::as_u64),
        Some(1)
    );
    assert_eq!(
        report
            .pointer("/contract/source/animation_keyframes")
            .and_then(JsonValue::as_u64),
        Some(5)
    );
    assert_eq!(
        report
            .pointer("/semanticProof/matched")
            .and_then(JsonValue::as_bool),
        Some(true)
    );
    for scope in ["source", "emitted"] {
        assert_eq!(
            report
                .pointer(&format!("/semanticProof/{scope}/standardAnimationsSha256"))
                .and_then(JsonValue::as_str),
            Some(STANDARD_ANIMATION_SHA256)
        );
    }

    let catalog: JsonValue = serde_json::from_slice(
        &std::fs::read(asset_root.join("map/shared/projectiles/catalog.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        catalog.get("sourceBuild").and_then(JsonValue::as_str),
        Some(RETROBUTION_TUTORIAL_BUILD_ID)
    );
    let source = catalog
        .get("particleEffects")
        .and_then(JsonValue::as_array)
        .and_then(|effects| {
            effects
                .iter()
                .find(|effect| effect.get("effectId").and_then(JsonValue::as_i64) == Some(718))
        })
        .expect("ES718 clean-source provenance");
    assert_eq!(
        source.get("containerRoute").and_then(JsonValue::as_str),
        Some("prefabs/particle/effectscripts/es[718].prefab")
    );
    assert_eq!(
        source.get("rootAsset").and_then(JsonValue::as_str),
        Some("CustomAssetBundle-fa9dbcf4604f64024b06ff1d5e375918")
    );
    assert_eq!(
        source.get("rootPathId").and_then(JsonValue::as_i64),
        Some(10_387)
    );
    assert_eq!(
        source.get("closureBlake3").and_then(JsonValue::as_str),
        Some("8e875dc2e142aade98c6e500191f5e57899f6b47c0c7613eafa7657c29c18a3d")
    );
}

#[test]
fn es742_and_es771_use_their_exact_native_mesh_packages() {
    let asset_root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let library = TutorialEffectLibrary::load(asset_root).unwrap();
    for (effect_id, expected_scene) in [
        (742, "map/shared/effects/models/es742/T_electric gun.glb"),
        (771, "map/shared/effects/models/es771/tutorialstone.glb"),
    ] {
        let compiled = compile_effect_plan(effect_id, library.effects.get(&effect_id).unwrap());
        let plan = compiled
            .plan
            .as_ref()
            .unwrap_or_else(|| panic!("effect {effect_id} blockers: {:#?}", compiled.blockers));
        assert_eq!(plan.mesh_scene, Some(expected_scene));
        assert_eq!(plan.emitters.len(), 2);
        assert_eq!(plan.rendered_nodes, 3);
        assert!(plan.material_animation.is_none());
        assert!(
            compiled.blockers.is_empty(),
            "effect {effect_id} retained blockers: {:#?}",
            compiled.blockers
        );
    }
}
