use super::*;

#[test]
fn primary_blackhole_nif_material_animation_is_bound_on_every_published_hole() {
    let contract = blackhole_nif_animation_contract().unwrap();
    assert_eq!(contract.clip.float_curves.len(), 40);
    assert_eq!(contract.model_bindings.len(), 7);
    assert_eq!(
        contract
            .clip
            .float_curves
            .iter()
            .filter(|curve| curve.attribute.starts_with("_MainTex.offset."))
            .count(),
        4
    );
    assert_eq!(
        contract
            .clip
            .float_curves
            .iter()
            .filter(|curve| curve.target_path.starts_with("Plane"))
            .count(),
        24
    );

    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let expected_tiles = [
        "map_06_06",
        "map_07_06",
        "map_07_08",
        "map_08_07",
        "map_11_01",
        "map_12_01",
        "map_12_03",
        "map_13_02",
        "map_15_12",
    ];
    for tile in expected_tiles {
        let document =
            load_native_world_behaviours(&asset_root, NativeWorldScope::WorldMap, tile)
                .unwrap();
        let emitters = document
            .effect_emitters
            .iter()
            .filter(|record| record.effect_name.as_deref() == Some("blackhole"))
            .collect::<Vec<_>>();
        let [emitter] = emitters.as_slice() else {
            panic!("{tile} must publish exactly one primary blackhole emitter");
        };
        assert_eq!(emitter.models.len(), 7, "{tile}");
        assert_eq!(
            emitter
                .nif_object
                .as_ref()
                .and_then(|pointer| pointer.get("pathId"))
                .and_then(serde_json::Value::as_i64),
            Some(9_377),
            "{tile}"
        );
        assert_eq!(emitter.max_timer, -4.0, "{tile}");

        let mut entities_by_model = HashMap::new();
        for (index, model) in emitter.models.iter().enumerate() {
            entities_by_model.insert(
                model.clone(),
                vec![Entity::from_raw_u32(index as u32).unwrap()],
            );
        }
        let (mut player, tracks) = blackhole_nif_animation_player(emitter, &entities_by_model)
            .unwrap()
            .expect("blackhole owns its exact NIF player");
        assert!(
            tracks.is_empty(),
            "material-only NIF clip must not sample TRS"
        );
        assert!(player.play_automatically);
        assert_eq!(player.wrap_mode, 2);
        assert_eq!(player.targets.len(), 7);
        assert_eq!(player.model_entities.len(), 7);
        assert_eq!(player.active_clip().unwrap().float_curves.len(), 40);
        let curve = player
            .active_clip()
            .unwrap()
            .float_curves
            .iter()
            .find(|curve| {
                curve.target_path == "Cylinder01" && curve.attribute == "_MainTex.offset.x"
            })
            .unwrap()
            .clone();
        player.elapsed_seconds = (contract.clip.duration + 1.25) as f32;
        let second_lap = sample_world_player_material_curve(&player, &curve).unwrap();
        assert!((second_lap - 0.312_5).abs() < 1.0e-5, "{tile}");
    }
}

#[test]
fn blackhole_material_curve_wraps_at_the_clean_four_second_boundary() {
    let contract = blackhole_nif_animation_contract().unwrap();
    let curve = contract
        .clip
        .float_curves
        .iter()
        .find(|curve| {
            curve.target_path == "Cylinder01" && curve.attribute == "_MainTex.offset.x"
        })
        .unwrap();
    let before_wrap =
        sample_world_material_curve(curve, contract.clip.duration - 0.25).unwrap();
    let after_wrap = sample_world_material_curve(curve, contract.clip.duration + 0.25).unwrap();
    assert!((before_wrap - 0.937_499_4).abs() < 1.0e-5);
    assert!((after_wrap - 0.062_5).abs() < 1.0e-5);
    // Runtime must pass elapsed time through unchanged. A historical
    // `min(duration)` at this call site returned exactly the first key on
    // every later frame because Infinity::Cycle wraps the endpoint.
    let second_lap = sample_world_material_curve(curve, contract.clip.duration + 1.25).unwrap();
    assert!((second_lap - 0.312_5).abs() < 1.0e-5);
}

#[test]
fn looped_world_material_curve_with_constant_infinity_repeats_with_its_clip() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let document =
        load_native_world_behaviours(&asset_root, NativeWorldScope::WorldMap, "map_08_07")
            .unwrap();
    let clip = document
        .animation_clips
        .iter()
        .find(|clip| clip.id.ends_with("#1725"))
        .expect("primary map_08_07 pulsing material clip");
    let curve = clip
        .float_curves
        .iter()
        .find(|curve| curve.attribute == "_Emission.r")
        .unwrap();
    assert!(clip.looped);
    assert_eq!((curve.pre_infinity, curve.post_infinity), (1, 1));

    let first_lap = sample_world_clip_material_curve(clip, 0, false, 1.25, curve).unwrap();
    let second_lap =
        sample_world_clip_material_curve(clip, 0, false, clip.duration as f32 + 1.25, curve)
            .unwrap();
    assert!((first_lap - second_lap).abs() < 1.0e-5);
    assert!(first_lap > 0.5, "the pulse must not be its clamped end key");
}

#[test]
fn recovered_closed_nif_transform_cycle_also_repeats_constant_material_curves() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let document =
        load_native_world_behaviours(&asset_root, NativeWorldScope::WorldMap, "map_06_03")
            .unwrap();
    let record = document
        .animations
        .iter()
        .find(|record| record.node == "BuildPlayer-Map_06_03#33484")
        .expect("primary closed NIF animation owner");
    let clip_id = record.default_clip_id.as_deref().unwrap();
    let clip = document
        .animation_clips
        .iter()
        .find(|clip| clip.id == clip_id)
        .unwrap();
    let curve = clip
        .float_curves
        .iter()
        .find(|curve| curve.class_id == 21 && curve.pre_infinity != 2)
        .unwrap();
    assert!(!clip.looped);
    assert_ne!(record.wrap_mode, 2);
    assert!(world_animation_transform_repeats(
        Some(clip),
        record.wrap_mode
    ));

    let first_lap =
        sample_world_clip_material_curve(clip, record.wrap_mode, true, 1.25, curve).unwrap();
    let second_lap = sample_world_clip_material_curve(
        clip,
        record.wrap_mode,
        true,
        clip.duration as f32 + 1.25,
        curve,
    )
    .unwrap();
    assert!((first_lap - second_lap).abs() < 1.0e-5);
}

#[test]
fn material_animation_binding_is_amortized_across_frames() {
    assert_eq!(WORLD_ANIMATION_MATERIAL_BINDING_WORK_PER_FRAME, 64);
    assert_eq!(WORLD_ANIMATION_MATERIAL_BINDING_WORK_PER_PLAYER, 16);

    let contract = blackhole_nif_animation_contract().unwrap();
    let target_path = contract.model_bindings[0].target_path.clone();
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<Assets<crate::legacy_model_material::LegacyModelMaterial>>()
        .add_systems(Update, prepare_world_animation_material_bindings);
    let root = app
        .world_mut()
        .spawn(crate::world::NativeWorldPresentationStatus::Loading)
        .id();
    let visual = app
        .world_mut()
        .spawn(crate::world::SpawnedNativeWorldVisual {
            model_path: "models/animated.glb".to_owned(),
            source_model_path: "models/animated.glb".to_owned(),
            scene: 0,
        })
        .id();
    // Animation target sets intentionally include collider-only entities;
    // they never receive the visual WorldInstanceReady marker and must not
    // hold the whole tile hidden.
    let collider = app
        .world_mut()
        .spawn(crate::world::SpawnedNativeWorldCollider {
            model_path: "models/animated.glb".to_owned(),
            source_model_path: "models/animated.glb".to_owned(),
        })
        .id();
    let mut model_entities = vec![visual, collider];
    model_entities.extend((0..98).map(|_| app.world_mut().spawn_empty().id()));
    let player = app
        .world_mut()
        .spawn((
            WorldAnimationPlayer {
                play_automatically: true,
                animate_physics: false,
                animate_only_if_visible: true,
                wrap_mode: 2,
                clip_count: 1,
                model_entities: model_entities.clone(),
                default_clip: None,
                clip_refs: Vec::new(),
                default_clip_id: Some(contract.clip.id.clone()),
                clip_ids: vec![contract.clip.id.clone()],
                targets: HashMap::from([(
                    target_path,
                    WorldAnimationTargetBinding {
                        entity: model_entities[0],
                        model_entities: model_entities.clone(),
                    },
                )]),
                elapsed_seconds: 0.0,
                transform_repeats: false,
                active_clip: Some(Arc::new(contract.clip.clone())),
            },
            WorldAnimationMaterialBindings::default(),
            WorldAnimationVisibilityBindings::default(),
            ChildOf(root),
        ))
        .id();

    // The Animation record may be admitted before any referenced glTF
    // scene has produced descendants. It must remain pending rather than
    // caching an empty binding and allowing the tile to reveal.
    app.update();
    let waiting = app
        .world()
        .get::<WorldAnimationMaterialBindings>(player)
        .unwrap();
    assert!(!waiting.ready);
    assert!(!waiting.initialized);
    app.world_mut()
        .entity_mut(visual)
        .insert(crate::world::NativeWorldVisualSceneReady);

    for _ in 0..12 {
        app.update();
        assert!(
            !app.world()
                .get::<WorldAnimationMaterialBindings>(player)
                .unwrap()
                .ready
        );
    }
    app.update();
    assert!(
        app.world()
            .get::<WorldAnimationMaterialBindings>(player)
            .unwrap()
            .ready
    );
}

#[test]
fn material_animation_updates_only_its_visible_renderer() {
    assert_material_animation_upload_contract(false);
}

#[test]
fn material_animation_uniform_upload_preserves_values_without_asset_rebuild() {
    assert_material_animation_upload_contract(true);
}

pub(super) fn assert_material_animation_upload_contract(uniform_only: bool) {
    use crate::legacy_model_material::{
        LegacyModelMaterialParams, LegacyModelRenderPlan, LegacyModelTextures, LegacyShaderKind,
    };

    let mut app = App::new();
    app.add_plugins((TaskPoolPlugin::default(), AssetPlugin::default()))
        .init_asset::<crate::legacy_model_material::LegacyModelMaterial>()
        .add_systems(Update, update_world_animation_materials);
    if uniform_only {
        app.insert_resource(
            crate::legacy_model_material::NativeMaterialUniformUpdates::enabled_for_test(),
        );
    }

    let material = LegacyModelMaterialParams::for_shader(LegacyShaderKind::OpaqueNormal)
        .material_for_pass(
            LegacyModelRenderPlan::for_shader(LegacyShaderKind::OpaqueNormal).passes[0],
            &LegacyModelTextures::default(),
        )
        .unwrap();
    let visible_material = app
        .world_mut()
        .resource_mut::<Assets<crate::legacy_model_material::LegacyModelMaterial>>()
        .add(material.clone());
    let hidden_material = app
        .world_mut()
        .resource_mut::<Assets<crate::legacy_model_material::LegacyModelMaterial>>()
        .add(material);
    let visible_renderer = app
        .world_mut()
        .spawn((Mesh3d::default(), ViewVisibility::HIDDEN))
        .id();
    app.world_mut()
        .get_mut::<ViewVisibility>(visible_renderer)
        .unwrap()
        .set_visible();
    let hidden_renderer = app
        .world_mut()
        .spawn((Mesh3d::default(), ViewVisibility::HIDDEN))
        .id();

    let curve = |target_path: &str| WorldAnimationFloatCurve {
        target_path: target_path.to_owned(),
        class_id: 21,
        attribute: "_Color.r".to_owned(),
        pre_infinity: 0,
        post_infinity: 0,
        times: vec![0.0],
        values: vec![0.25],
        in_tangents: vec![0.0],
        out_tangents: vec![0.0],
    };
    let clip = WorldAnimationClip {
        id: "visibility-material-test".to_owned(),
        name: "visibility-material-test".to_owned(),
        duration: 1.0,
        sample_rate: 30.0,
        looped: true,
        channels: Vec::new(),
        float_curves: vec![curve("visible"), curve("hidden")],
        events: Vec::new(),
    };
    app.world_mut().spawn((
        WorldAnimationPlayer {
            play_automatically: true,
            animate_physics: false,
            animate_only_if_visible: false,
            wrap_mode: 2,
            clip_count: 1,
            model_entities: vec![visible_renderer, hidden_renderer],
            default_clip: None,
            clip_refs: Vec::new(),
            default_clip_id: Some(clip.id.clone()),
            clip_ids: vec![clip.id.clone()],
            targets: HashMap::new(),
            elapsed_seconds: 0.5,
            transform_repeats: false,
            active_clip: Some(Arc::new(clip)),
        },
        WorldAnimationMaterialBindings {
            ready: true,
            by_target: HashMap::from([
                (
                    "visible".to_owned(),
                    vec![WorldAnimationMaterialBinding {
                        renderer: visible_renderer,
                        material: visible_material.clone(),
                        deferred_owner: None,
                    }],
                ),
                (
                    "hidden".to_owned(),
                    vec![WorldAnimationMaterialBinding {
                        renderer: hidden_renderer,
                        material: hidden_material.clone(),
                        deferred_owner: None,
                    }],
                ),
            ]),
            ..default()
        },
        WorldAnimationVisibilityBindings {
            ready: true,
            renderers: vec![visible_renderer, hidden_renderer],
            ..default()
        },
    ));

    app.update();

    let materials = app
        .world()
        .resource::<Assets<crate::legacy_model_material::LegacyModelMaterial>>();
    assert_eq!(
        materials
            .get(&visible_material)
            .unwrap()
            .uniform
            .base_color
            .red,
        0.25
    );
    assert_eq!(
        materials
            .get(&hidden_material)
            .unwrap()
            .uniform
            .base_color
            .red,
        1.0
    );
    let events = app
        .world()
        .resource::<Messages<AssetEvent<crate::legacy_model_material::LegacyModelMaterial>>>();
    let modified = events.get_cursor().read(events).filter(|event| matches!(event, AssetEvent::Modified { id } if *id == visible_material.id())).count();
    assert_eq!(modified, usize::from(!uniform_only));
}

pub(super) fn linear_material_curve(
    target_path: &str,
    attribute: &str,
    duration: f64,
    velocity: f64,
) -> WorldAnimationFloatCurve {
    WorldAnimationFloatCurve {
        target_path: target_path.to_owned(),
        class_id: 21,
        attribute: attribute.to_owned(),
        pre_infinity: 0,
        post_infinity: 0,
        // NIF exports commonly retain a duplicate key at time zero.
        times: vec![0.0, 0.0, duration * 0.5, duration],
        values: vec![0.0, 0.0, velocity * duration * 0.5, velocity * duration],
        in_tangents: vec![velocity; 4],
        out_tangents: vec![velocity; 4],
    }
}

pub(super) fn material_clip(
    name: &str,
    looped: bool,
    curves: Vec<WorldAnimationFloatCurve>,
) -> WorldAnimationClip {
    WorldAnimationClip {
        id: "clip".to_owned(),
        name: name.to_owned(),
        duration: 100.0,
        sample_rate: 30.0,
        looped,
        channels: Vec::new(),
        float_curves: curves,
        events: Vec::new(),
    }
}
