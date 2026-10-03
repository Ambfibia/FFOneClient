use crate::tutorial_ep_barrier::*;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    path::Path,
    time::Duration,
};

use crate::legacy_model_material::{
    LegacyModelMaterialParams, LegacyModelRenderPlan, LegacyModelTextures, LegacyShaderKind,
};

const PRIMARY_EP_BARRIER_TILE_COUNTS: [(&str, usize); 33] = [
    ("map_01_01", 6),
    ("map_02_04", 20),
    ("map_02_12", 30),
    ("map_03_06", 16),
    ("map_04_02", 12),
    ("map_04_03", 22),
    ("map_04_05", 18),
    ("map_04_07", 28),
    ("map_04_13", 61),
    ("map_05_02", 20),
    ("map_05_03", 20),
    ("map_05_05", 27),
    ("map_05_07", 39),
    ("map_05_10", 20),
    ("map_05_12", 37),
    ("map_06_03", 38),
    ("map_06_06", 14),
    ("map_07_07", 22),
    ("map_08_06", 19),
    ("map_08_13", 32),
    ("map_09_06", 42),
    ("map_09_07", 18),
    ("map_09_11", 39),
    ("map_10_09", 24),
    ("map_10_10", 19),
    ("map_11_01", 18),
    ("map_11_11", 24),
    ("map_12_02", 18),
    ("map_12_03", 9),
    ("map_12_10", 28),
    ("map_14_01", 29),
    ("map_14_02", 24),
    ("map_15_09", 6),
];

fn game_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game")
}

fn parse_json(path: &Path) -> serde_json::Value {
    serde_json::from_slice(
        &fs::read(path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()))
}

fn read_declared_json(root: &Path, reference: &serde_json::Value) -> serde_json::Value {
    let relative = reference["path"].as_str().unwrap();
    let expected_bytes = reference["bytes"].as_u64().unwrap();
    let expected_blake3 = reference["blake3"].as_str().unwrap();
    let path = root.join(relative);
    let bytes = fs::read(&path).unwrap_or_else(|error| {
        panic!("missing declared artifact {}: {error}", path.display())
    });
    assert_eq!(bytes.len() as u64, expected_bytes, "{relative} byte count");
    assert_eq!(
        blake3::hash(&bytes).to_hex().as_str(),
        expected_blake3,
        "{relative} BLAKE3"
    );
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|error| panic!("failed to parse {relative}: {error}"))
}

fn barrier_roots(behaviour: &serde_json::Value, tile_id: &str) -> HashSet<String> {
    let mut roots = HashSet::new();
    for emitter in behaviour["effectEmitters"].as_array().unwrap() {
        if emitter["effectName"].as_str() != Some("EPbarrier") {
            continue;
        }
        assert_eq!(
            emitter["controller"].as_str(),
            Some("EffectEmitterController")
        );
        assert_eq!(emitter["enabled"].as_bool(), Some(true));
        // Unity PPtr fileId is the map asset's external-reference slot and
        // varies across the 33 containers; pathId is the stable object
        // identity in their shared CustomAssetBundle.
        assert!(
            emitter["nifObject"]["fileId"]
                .as_u64()
                .is_some_and(|file_id| file_id != 0),
            "EPbarrier nifObject must be an external Tutorial asset reference in {tile_id}"
        );
        assert_eq!(emitter["nifObject"]["pathId"].as_u64(), Some(4961));
        assert!(
            emitter["resolvedParticlePrefabs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| {
                    value.as_str()
                        == Some("CustomAssetBundle-b4f543c102ded400fbc6f1da25d9679a#5101")
                })
        );
        let node = emitter["node"].as_str().unwrap();
        let (asset, path_id) = node.rsplit_once('#').unwrap();
        let root = format!("{asset}#{}", path_id.parse::<u64>().unwrap() + 1);
        assert!(roots.insert(root), "duplicate EPbarrier root in {tile_id}");
    }
    roots
}

fn visual_prefab_root(name: &str) -> Option<&str> {
    name.split_once("[prefab:")?
        .1
        .split_once(':')
        .map(|pair| pair.0)
}

fn find_set_artifact<'a>(
    manifest: &'a serde_json::Value,
    wanted: &str,
) -> Option<&'a serde_json::Value> {
    for member in manifest["members"].as_array()? {
        if member["definition"]["path"].as_str() == Some(wanted) {
            return Some(&member["definition"]);
        }
        if let Some(files) = member["files"].as_array()
            && let Some(file) = files
                .iter()
                .find(|file| file["path"].as_str() == Some(wanted))
        {
            return Some(file);
        }
    }
    manifest["textures"].as_array().and_then(|textures| {
        textures
            .iter()
            .find(|file| file["path"].as_str() == Some(wanted))
    })
}

fn verify_set_artifact(root: &Path, artifact: &serde_json::Value) {
    let relative = artifact["path"].as_str().unwrap();
    let bytes = fs::read(root.join(relative))
        .unwrap_or_else(|error| panic!("missing set artifact {relative}: {error}"));
    assert_eq!(
        bytes.len() as u64,
        artifact["bytes"].as_u64().unwrap(),
        "{relative}"
    );
    assert_eq!(
        blake3::hash(&bytes).to_hex().as_str(),
        artifact["blake3"].as_str().unwrap(),
        "{relative}"
    );
}

#[test]
fn exact_published_model_paths_restore_the_complete_barrier_clip() {
    for (path, expected) in EP_BARRIER_ANIMATED_SURFACES {
        assert_eq!(classify_surface(path), Some(expected), "{path}");
        let windows_path = path.replace('/', &char::from(92).to_string());
        assert_eq!(classify_surface(&windows_path), Some(expected));
    }
    for phase in 0..6 {
        assert_eq!(
            EP_BARRIER_ANIMATED_SURFACES
                .iter()
                .filter(|entry| entry.1 == EpBarrierSurfaceKind::Ring { phase })
                .count(),
            2
        );
    }
    assert_eq!(
        EP_BARRIER_ANIMATED_SURFACES
            .iter()
            .filter(|entry| entry.1 == EpBarrierSurfaceKind::Center)
            .count(),
        1
    );
    for path in EP_BARRIER_STATIC_MODEL_PATHS {
        assert!(classify_surface(path).is_none(), "{path} is static");
    }
    assert!(classify_surface("models/world/tutorial/tile_01_01/v-00013-test.glb").is_none());
}

#[test]
fn all_primary_epbarriers_are_owned_and_their_published_assets_are_verified() {
    let root = game_root();
    let tile_root = root.join("map/tiles");
    let all_paths = EP_BARRIER_ANIMATED_SURFACES
        .iter()
        .map(|(path, _)| *path)
        .chain(EP_BARRIER_STATIC_MODEL_PATHS)
        .collect::<HashSet<_>>();
    assert_eq!(all_paths.len(), 15);

    let mut occurrences = all_paths
        .iter()
        .map(|path| ((*path).to_owned(), 0usize))
        .collect::<BTreeMap<_, _>>();
    let mut model_hashes = BTreeMap::<String, String>::new();
    let mut total_emitters = 0usize;
    let mut barrier_tiles = 0usize;
    let mut classified_surfaces = 0usize;
    let mut tile_dirs = fs::read_dir(&tile_root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir() && path.join("tile.json").is_file())
        .collect::<Vec<_>>();
    tile_dirs.sort();

    for tile_dir in tile_dirs {
        let tile_id = tile_dir.file_name().unwrap().to_str().unwrap();
        let manifest = parse_json(&tile_dir.join("tile.json"));
        let expected_emitters = PRIMARY_EP_BARRIER_TILE_COUNTS
            .iter()
            .find_map(|(expected_tile, count)| (*expected_tile == tile_id).then_some(*count));
        let roots = if let Some(expected_emitters) = expected_emitters {
            let behaviour = read_declared_json(&root, &manifest["behaviour"]);
            let roots = barrier_roots(&behaviour, tile_id);
            assert_eq!(roots.len(), expected_emitters, "{tile_id}");
            total_emitters += roots.len();
            barrier_tiles += 1;
            roots
        } else {
            HashSet::new()
        };

        let scene = read_declared_json(&root, &manifest["scene"]);
        let models = scene["models"]
            .as_array()
            .unwrap()
            .iter()
            .map(|model| {
                (
                    model["id"].as_str().unwrap().to_owned(),
                    (
                        model["path"].as_str().unwrap().to_owned(),
                        model["blake3"].as_str().unwrap().to_owned(),
                    ),
                )
            })
            .collect::<HashMap<_, _>>();
        let mut root_visual_counts = roots
            .iter()
            .map(|root| (root.clone(), 0usize))
            .collect::<HashMap<_, _>>();

        for visual in scene["visuals"].as_array().unwrap() {
            let model_id = visual["model"].as_str().unwrap();
            let (model_path, model_blake3) = models.get(model_id).unwrap();
            let prefab_root = visual["name"].as_str().and_then(visual_prefab_root);
            let owned_root = prefab_root.and_then(|root| roots.contains(root).then_some(root));
            let is_barrier_path = all_paths.contains(model_path.as_str());

            if let Some(root) = owned_root {
                *root_visual_counts.get_mut(root).unwrap() += 1;
                assert!(
                    is_barrier_path,
                    "unexpected model {model_path} under {root} in {tile_id}"
                );
            }
            if is_barrier_path {
                assert!(
                    owned_root.is_some(),
                    "barrier model {model_path} escaped EPbarrier ownership in {tile_id}"
                );
                *occurrences.get_mut(model_path).unwrap() += 1;
                if let Some(previous) =
                    model_hashes.insert(model_path.clone(), model_blake3.clone())
                {
                    assert_eq!(
                        previous, *model_blake3,
                        "conflicting scene-catalog hash for {model_path}"
                    );
                }
            }
            if classify_surface(model_path).is_some() {
                assert!(
                    owned_root.is_some(),
                    "animated surface {model_path} is not an EPbarrier child"
                );
                classified_surfaces += 1;
            }
        }
        for (prefab_root, count) in root_visual_counts {
            assert_eq!(
                count, 15,
                "{tile_id} {prefab_root} must retain all prefab visuals"
            );
        }
    }

    assert_eq!(barrier_tiles, PRIMARY_EP_BARRIER_TILE_COUNTS.len());
    assert_eq!(total_emitters, 799);
    assert_eq!(classified_surfaces, 799 * 13);
    assert_eq!(model_hashes.len(), 15);
    for (model_path, count) in &occurrences {
        assert_eq!(*count, 799, "{model_path}");
    }

    let mut verified_sets = HashSet::new();
    for (model_path, scene_blake3) in model_hashes {
        let parts = model_path.split('/').collect::<Vec<_>>();
        assert!(parts.len() >= 4 && parts[0] == "objects", "{model_path}");
        let set_relative = format!("{}/{}/{}/set.json", parts[0], parts[1], parts[2]);
        let set = parse_json(&root.join(&set_relative));
        let artifact = find_set_artifact(&set, &model_path)
            .unwrap_or_else(|| panic!("{model_path} missing from {set_relative}"));
        assert_eq!(
            artifact["blake3"].as_str(),
            Some(scene_blake3.as_str()),
            "{model_path}"
        );
        verify_set_artifact(&root, artifact);
        if verified_sets.insert(set_relative) {
            for texture in set["textures"].as_array().unwrap() {
                verify_set_artifact(&root, texture);
            }
        }
    }
}

#[test]
fn authored_local_scale_keeps_external_scene_placement_fixed() {
    let baseline = Transform {
        translation: Vec3::new(3.0, 4.0, 5.0),
        rotation: Quat::from_rotation_y(0.37),
        scale: Vec3::new(2.0, 3.0, 4.0),
    };
    let animated = scale_from_authored_local_pivot(baseline, 95.0);

    assert_eq!(animated.translation, baseline.translation);
    assert_eq!(animated.rotation, baseline.rotation);
    assert_eq!(animated.scale, baseline.scale * 95.0);
    assert_eq!(scale_from_authored_local_pivot(baseline, 1.0), baseline);
}

#[test]
fn runtime_plugin_expands_and_fades_a_published_ring_in_place() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default())
        .insert_resource(Assets::<LegacyModelMaterial>::default())
        .add_plugins(TutorialEpBarrierPlugin);

    let baseline = Transform {
        translation: Vec3::new(-596.185_06, -79.424_36, 708.693_24),
        rotation: Quat::from_rotation_y(std::f32::consts::PI),
        scale: Vec3::splat(0.136_336_21),
    };
    let root = app
        .world_mut()
        .spawn((
            Name::new(
                "Plane01 [prefab:BuildPlayer-Map_12_03#13669:\
                 CustomAssetBundle-b4f543c102ded400fbc6f1da25d9679a#5529 visual]",
            ),
            SpawnedNativeWorldVisual {
                model_path: "objects/vehicles/motttt/models/plane01/visual.glb".into(),
                source_model_path: "models/world/maps/map_12_03/v-00032-arbitrary.glb".into(),
                scene: 0,
            },
            baseline,
        ))
        .id();

    let shader = LegacyShaderKind::SrcAlphaAdditiveTwoSided;
    let material = LegacyModelMaterialParams::for_shader(shader)
        .material_for_pass(
            LegacyModelRenderPlan::for_shader(shader).passes[0],
            &LegacyModelTextures::default(),
        )
        .unwrap();
    let material_handle = app
        .world_mut()
        .resource_mut::<Assets<LegacyModelMaterial>>()
        .add(material);
    let material_entity = app
        .world_mut()
        .spawn((MeshMaterial3d(material_handle.clone()), ChildOf(root)))
        .id();

    app.update();
    assert!(
        app.world().get::<TutorialEpBarrierSurface>(root).is_some(),
        "the production plugin must recognize the published model_path"
    );

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(1.7));
    app.update();

    let expected_scale = evaluate_curve(RING_SCALE_0, 1.7, 1.0);
    let transform = app.world().get::<Transform>(root).unwrap();
    assert_eq!(transform.translation, baseline.translation);
    assert_eq!(transform.rotation, baseline.rotation);
    assert!(
        transform
            .scale
            .abs_diff_eq(baseline.scale * expected_scale, 0.000_1),
        "{:?} != {:?}",
        transform.scale,
        baseline.scale * expected_scale
    );

    let instance_handle = app
        .world()
        .get::<MeshMaterial3d<LegacyModelMaterial>>(material_entity)
        .unwrap()
        .0
        .clone();
    assert_ne!(instance_handle, material_handle);
    assert!(
        app.world()
            .get::<TutorialEpBarrierMaterialInstance>(material_entity)
            .is_some()
    );
    let material = app
        .world()
        .resource::<Assets<LegacyModelMaterial>>()
        .get(&instance_handle)
        .unwrap();
    let expected_color = evaluate_curve(RING_COLOR_0, 1.7, 1.0);
    let expected_emission = evaluate_curve(RING_EMISSION_0, 1.7, 0.0);
    assert!((material.uniform.base_color.red - expected_color).abs() < 0.000_1);
    assert!((material.uniform.emission.red - expected_emission).abs() < 0.000_1);
}

#[test]
fn separately_started_barriers_do_not_overwrite_one_shared_gltf_material() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default())
        .insert_resource(Assets::<LegacyModelMaterial>::default())
        .add_plugins(TutorialEpBarrierPlugin);

    let shader = LegacyShaderKind::SrcAlphaAdditiveTwoSided;
    let shared = app
        .world_mut()
        .resource_mut::<Assets<LegacyModelMaterial>>()
        .add(
            LegacyModelMaterialParams::for_shader(shader)
                .material_for_pass(
                    LegacyModelRenderPlan::for_shader(shader).passes[0],
                    &LegacyModelTextures::default(),
                )
                .unwrap(),
        );
    let spawn = |app: &mut App, owner: u32, shared: &Handle<LegacyModelMaterial>| {
        let root = app
            .world_mut()
            .spawn((
                Name::new(format!(
                    "Plane01 [prefab:BuildPlayer-Map_12_03#{owner}:bundle#5529 visual]"
                )),
                SpawnedNativeWorldVisual {
                    model_path: "objects/vehicles/motttt/models/plane01/visual.glb".into(),
                    source_model_path: "models/world/maps/map_12_03/test.glb".into(),
                    scene: 0,
                },
                Transform::IDENTITY,
            ))
            .id();
        app.world_mut()
            .spawn((MeshMaterial3d(shared.clone()), ChildOf(root)))
            .id()
    };

    let first = spawn(&mut app, 13669, &shared);
    app.update();
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(0.5));
    let second = spawn(&mut app, 13700, &shared);
    app.update();
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(0.5));
    app.update();

    let first_handle = app
        .world()
        .get::<MeshMaterial3d<LegacyModelMaterial>>(first)
        .unwrap()
        .0
        .clone();
    let second_handle = app
        .world()
        .get::<MeshMaterial3d<LegacyModelMaterial>>(second)
        .unwrap()
        .0
        .clone();
    assert_ne!(first_handle, shared);
    assert_ne!(second_handle, shared);
    assert_ne!(first_handle, second_handle);

    let materials = app.world().resource::<Assets<LegacyModelMaterial>>();
    let first_color = materials.get(&first_handle).unwrap().uniform.base_color.red;
    let second_color = materials
        .get(&second_handle)
        .unwrap()
        .uniform
        .base_color
        .red;
    assert!((first_color - evaluate_curve(RING_COLOR_0, 1.0, 1.0)).abs() < 0.000_1);
    assert!((second_color - evaluate_curve(RING_COLOR_0, 0.5, 1.0)).abs() < 0.000_1);
    assert_ne!(first_color, second_color);
}

#[test]
fn one_prefab_instance_uses_one_timeline_even_when_children_arrive_late() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default())
        .insert_resource(Assets::<LegacyModelMaterial>::default())
        .add_plugins(TutorialEpBarrierPlugin);

    let spawn_surface = |app: &mut App, name: &str, model_path: &str| {
        app.world_mut()
            .spawn((
                Name::new(name.to_owned()),
                SpawnedNativeWorldVisual {
                    model_path: model_path.to_owned(),
                    source_model_path: "models/world/maps/map_12_03/test.glb".into(),
                    scene: 0,
                },
                Transform::IDENTITY,
            ))
            .id()
    };
    let first = spawn_surface(
        &mut app,
        "Plane01 [prefab:BuildPlayer-Map_12_03#13669:bundle#5529 visual]",
        "objects/vehicles/motttt/models/plane01/visual.glb",
    );
    app.update();
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(0.25));
    let late = spawn_surface(
        &mut app,
        "Plane02 [prefab:BuildPlayer-Map_12_03#13669:bundle#5530 visual]",
        "objects/vehicles/motttt/models/plane02/visual.glb",
    );
    let other = spawn_surface(
        &mut app,
        "Plane01 [prefab:BuildPlayer-Map_12_03#13700:bundle#5529 visual]",
        "objects/vehicles/motttt/models/plane01/visual.glb",
    );
    app.update();

    let first_start = app
        .world()
        .get::<TutorialEpBarrierSurface>(first)
        .unwrap()
        .started_at_seconds;
    let late_start = app
        .world()
        .get::<TutorialEpBarrierSurface>(late)
        .unwrap()
        .started_at_seconds;
    let other_start = app
        .world()
        .get::<TutorialEpBarrierSurface>(other)
        .unwrap()
        .started_at_seconds;
    assert_eq!(first_start, late_start);
    assert!((other_start - first_start - 0.25).abs() < 0.000_1);
}

#[test]
fn prefab_owner_is_the_stable_animation_instance_key() {
    assert_eq!(
        barrier_instance_key(
            "Plane01 [prefab:BuildPlayer-Map_12_03#13669:\
             CustomAssetBundle-example#5529 visual]"
        )
        .as_deref(),
        Some("BuildPlayer-Map_12_03#13669")
    );
    assert_eq!(barrier_instance_key("Plane01"), None);
}

#[test]
fn clip_uses_the_original_hermite_keys_and_constant_post_infinity() {
    assert_eq!(evaluate_curve(RING_SCALE_0, 0.0, -1.0), 1.0);
    assert_eq!(evaluate_curve(RING_SCALE_0, 1.699_999_7, -1.0), 74.213_85);
    assert_eq!(
        evaluate_curve(RING_SCALE_2, CLIP_DURATION_SECONDS, -1.0),
        95.634_964
    );
    assert_eq!(
        evaluate_curve(CENTER_PULSE, 0.533_333_36, -1.0),
        0.990_026_4
    );
    assert_eq!(
        evaluate_curve(CENTER_SCALE, CLIP_DURATION_SECONDS, -1.0),
        1.077_147_7
    );
}
