//! Deterministic ordinary-world screenshot batch driven by clean Unity warp anchors.
//!
//! The manifest records protocol, Unity and native coordinates plus the exact clean
//! TableData provenance. The preview itself consumes only published native assets.

use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
    time::Duration,
};

use bevy::{
    asset::AssetPlugin,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    window::{PresentMode, WindowResolution},
};
use ffone_client::{
    legacy_model_material::LegacyModelMaterialPlugin,
    movement::{LegacyMovementPlugin, LegacyMovementSet, LegacyOrbitCamera},
    native_terrain::{NativeHeightmapCollider, NativeHeightmapTerrain, NativeTerrainMaterial},
    terrain_ambience::distance_haze_density,
    tutorial_effects_runtime::{
        TutorialEffectLibrary, TutorialEffectRuntime, TutorialEffectsRuntimePlugin,
    },
    world::{
        NativeWorldCatalog, NativeWorldPlugin, NativeWorldPresentationStatus, NativeWorldSceneRoot,
        NativeWorldScope, NativeWorldStreamingStatus, load_native_world_scenes,
    },
    world_behaviour::{NativeWorldBehaviourRoot, add_world_preview_behaviour_systems},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const MANIFEST_SCHEMA: &str = "ffone.world-visual-capture-poses.v1";
const REPORT_SCHEMA: &str = "ffone.world-visual-capture-report.v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CaptureManifest {
    schema: String,
    camera: CaptureCamera,
    poses: Vec<CapturePose>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CaptureCamera {
    width: u32,
    viewport_height: u32,
    fov_degrees: f32,
    target_height: f32,
    distance: f32,
    pitch_degrees: f32,
    far: f32,
    warmup_frames: u32,
    timeout_frames_per_pose: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CapturePose {
    id: String,
    era: String,
    zone: String,
    area: String,
    map_number: i32,
    safe_anchor_protocol: [i32; 3],
    safe_anchor_unity: [f32; 3],
    safe_anchor_native: [f32; 3],
    gm_teleport_map_xyz_i: [i32; 3],
    gm_vertical_lift_unity: f32,
    yaw_degrees: f32,
}

impl CapturePose {
    fn native_anchor(&self) -> Vec3 {
        Vec3::from_array(self.safe_anchor_native)
    }
}

#[derive(Resource)]
struct PreviewConfig {
    manifest_path: PathBuf,
    output_root: PathBuf,
    manifest: CaptureManifest,
}

#[derive(Resource, Default)]
struct PreviewState {
    pose_index: usize,
    pose_frame: u32,
    stable_frames: u32,
    capture_issued: bool,
    capture_saved: bool,
    pending: Option<PendingCapture>,
    records: Vec<CaptureRecord>,
}

#[derive(Debug, Clone)]
struct PendingCapture {
    resident_tiles: Vec<[i32; 2]>,
    terrain_height: Option<f32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CaptureReport {
    schema: &'static str,
    source_manifest: String,
    output_root: String,
    captures: Vec<CaptureRecord>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CaptureRecord {
    pose_id: String,
    era: String,
    zone: String,
    area: String,
    map_number: i32,
    safe_anchor_protocol: [i32; 3],
    safe_anchor_native: [f32; 3],
    terrain_height: Option<f32>,
    anchor_height_above_terrain: Option<f32>,
    resident_tiles: Vec<[i32; 2]>,
    png: String,
    png_sha256: String,
    width: u32,
    height: u32,
    non_background_pixels: usize,
}

#[derive(Component)]
struct CaptureTarget;

fn main() {
    let mut args = std::env::args_os().skip(1);
    let asset_root = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("assets/game"));
    let manifest_path = args.next().map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from("target/world-capture/world-visual-capture-poses.v1.json")
    });
    let output_root = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/world-capture/bevy"));
    if args.next().is_some() {
        eprintln!("usage: world_capture_gpu_preview [ASSET_ROOT] [MANIFEST.json] [OUTPUT_DIR]");
        std::process::exit(2);
    }

    let asset_root = canonical_directory(&asset_root, "asset root");
    let manifest_path = canonical_file(&manifest_path, "capture manifest");
    let manifest_bytes = fs::read(&manifest_path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", manifest_path.display()));
    let manifest: CaptureManifest = serde_json::from_slice(&manifest_bytes)
        .unwrap_or_else(|error| panic!("invalid capture manifest: {error}"));
    validate_manifest(&manifest);
    let catalog = load_native_world_scenes(&asset_root)
        .unwrap_or_else(|error| panic!("cannot open native world catalog: {error}"));
    let effect_library = TutorialEffectLibrary::load(&asset_root)
        .unwrap_or_else(|error| panic!("cannot open native effect library: {error}"));
    validate_catalog_coverage(&catalog, &manifest);
    fs::create_dir_all(&output_root)
        .unwrap_or_else(|error| panic!("cannot create {}: {error}", output_root.display()));
    let output_root = fs::canonicalize(&output_root)
        .unwrap_or_else(|error| panic!("cannot resolve {}: {error}", output_root.display()));
    for pose in &manifest.poses {
        let output = output_root.join(format!("{}.png", pose.id));
        if output.exists() {
            fs::remove_file(&output)
                .unwrap_or_else(|error| panic!("cannot replace {}: {error}", output.display()));
        }
    }

    let camera = manifest.camera;
    let behaviour_root = NativeWorldBehaviourRoot(asset_root.clone());
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.025, 0.04, 0.055)))
        .insert_resource(PreviewConfig {
            manifest_path,
            output_root,
            manifest,
        })
        .insert_resource(PreviewState::default())
        .insert_resource::<NativeWorldCatalog>(catalog)
        .insert_resource(behaviour_root)
        .insert_resource(effect_library.clone())
        .insert_resource(TutorialEffectRuntime::with_library(effect_library))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne deterministic world capture".into(),
                        resolution: WindowResolution::new(camera.width, camera.viewport_height),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((
            LegacyMovementPlugin,
            LegacyModelMaterialPlugin,
            NativeWorldPlugin,
            TutorialEffectsRuntimePlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            apply_preview_ambience.after(LegacyMovementSet::CameraPose),
        );
    add_world_preview_behaviour_systems(&mut app);
    app.add_systems(Update, drive_capture).run();
}

fn canonical_directory(path: &Path, label: &str) -> PathBuf {
    let resolved = fs::canonicalize(path)
        .unwrap_or_else(|error| panic!("cannot resolve {label} {}: {error}", path.display()));
    assert!(
        resolved.is_dir(),
        "{label} is not a directory: {}",
        resolved.display()
    );
    resolved
}

fn canonical_file(path: &Path, label: &str) -> PathBuf {
    let resolved = fs::canonicalize(path)
        .unwrap_or_else(|error| panic!("cannot resolve {label} {}: {error}", path.display()));
    assert!(
        resolved.is_file(),
        "{label} is not a file: {}",
        resolved.display()
    );
    resolved
}

fn validate_manifest(manifest: &CaptureManifest) {
    assert_eq!(
        manifest.schema, MANIFEST_SCHEMA,
        "unsupported manifest schema"
    );
    assert!(!manifest.poses.is_empty(), "capture manifest has no poses");
    let camera = manifest.camera;
    assert!(camera.width > 0 && camera.viewport_height > 0);
    assert!(camera.fov_degrees.is_finite() && (1.0..179.0).contains(&camera.fov_degrees));
    assert!(camera.target_height.is_finite());
    assert!(camera.distance.is_finite() && camera.distance > 1.0);
    assert!(camera.pitch_degrees.is_finite());
    assert!(camera.far.is_finite() && camera.far > camera.distance);
    assert!(camera.warmup_frames > 0 && camera.timeout_frames_per_pose > camera.warmup_frames);

    let mut ids = BTreeSet::new();
    for pose in &manifest.poses {
        assert!(
            ids.insert(pose.id.as_str()),
            "duplicate pose id {:?}",
            pose.id
        );
        assert!(!pose.era.trim().is_empty());
        assert!(!pose.zone.trim().is_empty());
        assert!(!pose.area.trim().is_empty());
        assert_eq!(pose.map_number, 0, "ordinary-world capture must use map 0");
        assert!(pose.yaw_degrees.is_finite());
        assert!(pose.gm_vertical_lift_unity >= 1.0);
        let protocol = pose.safe_anchor_protocol;
        let expected_unity = [
            protocol[0] as f32 / 100.0,
            protocol[2] as f32 / 100.0,
            protocol[1] as f32 / 100.0,
        ];
        let expected_native = [-expected_unity[0], expected_unity[1], expected_unity[2]];
        assert_vector_close(pose.safe_anchor_unity, expected_unity, &pose.id, "Unity");
        assert_vector_close(pose.safe_anchor_native, expected_native, &pose.id, "native");
        let gm = pose.gm_teleport_map_xyz_i;
        assert!((gm[0] as f32 - expected_unity[0]).abs() <= 0.51);
        assert!((gm[1] as f32 - expected_unity[2]).abs() <= 0.51);
        assert!(gm[2] as f32 - expected_unity[1] >= 1.0);
    }
}

fn assert_vector_close(actual: [f32; 3], expected: [f32; 3], id: &str, space: &str) {
    for index in 0..3 {
        assert!(
            (actual[index] - expected[index]).abs() <= 0.000_1,
            "pose {id:?} has an invalid {space} coordinate at axis {index}"
        );
    }
}

fn validate_catalog_coverage(catalog: &NativeWorldCatalog, manifest: &CaptureManifest) {
    for pose in &manifest.poses {
        let scene = catalog
            .select_in_scope(NativeWorldScope::WorldMap, pose.native_anchor())
            .unwrap_or_else(|| panic!("pose {:?} has no published native world tile", pose.id));
        assert_eq!(
            scene.scope,
            Some(NativeWorldScope::WorldMap),
            "pose {:?} resolved outside the ordinary world",
            pose.id
        );
    }
}

fn setup(mut commands: Commands, config: Res<PreviewConfig>) {
    let pose = &config.manifest.poses[0];
    let target = commands
        .spawn((
            Name::new("deterministic capture target"),
            CaptureTarget,
            Transform::from_translation(pose.native_anchor()),
            Visibility::Hidden,
        ))
        .id();
    let camera = config.manifest.camera;
    let mut orbit = LegacyOrbitCamera::new(target).with_yaw(pose.yaw_degrees);
    orbit.height = camera.target_height;
    orbit.distance = camera.distance;
    orbit.default_distance = camera.distance;
    orbit.maximum_distance = camera.distance.max(12.0);
    orbit.pitch_degrees = camera.pitch_degrees;
    orbit.default_pitch_degrees = camera.pitch_degrees;
    commands.spawn((
        Name::new("deterministic world capture camera"),
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: camera.fov_degrees.to_radians(),
            far: camera.far,
            ..default()
        }),
        orbit,
        Transform::default(),
    ));
    commands.spawn((
        Name::new("Retrobution world capture light"),
        DirectionalLight {
            illuminance: 8_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 20.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    println!(
        "capture 1/{}: {} / {} ({})",
        config.manifest.poses.len(),
        pose.zone,
        pose.area,
        pose.id
    );
}

fn apply_preview_ambience(
    mut commands: Commands,
    catalog: Res<NativeWorldCatalog>,
    cameras: Query<(Entity, &Transform, &Projection), (With<Camera3d>, With<LegacyOrbitCamera>)>,
    mut lights: Query<&mut DirectionalLight>,
    mut clear: ResMut<ClearColor>,
    mut terrain_materials: ResMut<Assets<NativeTerrainMaterial>>,
) {
    let Ok((camera, camera_transform, projection)) = cameras.single() else {
        return;
    };
    let camera_far = projection.far();
    let sample = catalog.sample_ambience(camera_transform.translation, false);
    let applied = catalog.applied_ambience(camera_transform.translation, false);
    // Same owner-requested distance haze the gameplay schedule applies, so a
    // capture stays a picture of the production renderer.
    let fog_density = distance_haze_density(&applied, camera_far);
    clear.0 = Color::srgba(
        applied.sky_color[0],
        applied.sky_color[1],
        applied.sky_color[2],
        applied.sky_color[3],
    );
    commands.entity(camera).insert(DistanceFog {
        color: Color::srgba(
            sample.fog_color[0],
            sample.fog_color[1],
            sample.fog_color[2],
            sample.fog_color[3],
        ),
        directional_light_color: Color::NONE,
        falloff: FogFalloff::Exponential {
            density: fog_density,
        },
        ..default()
    });
    let light_color = Color::srgba(
        applied.light_color[0],
        applied.light_color[1],
        applied.light_color[2],
        applied.light_color[3],
    );
    for mut light in &mut lights {
        light.color = light_color;
    }
    for (_, material) in terrain_materials.iter_mut() {
        material.uniform.ambience_light = Vec4::from_array(applied.light_color);
        material.uniform.ambience_fog = Vec4::new(
            sample.fog_color[0],
            sample.fog_color[1],
            sample.fog_color[2],
            fog_density,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn drive_capture(
    mut commands: Commands,
    config: Res<PreviewConfig>,
    mut state: ResMut<PreviewState>,
    catalog: Res<NativeWorldCatalog>,
    streaming: Res<NativeWorldStreamingStatus>,
    mut targets: Query<&mut Transform, (With<CaptureTarget>, Without<LegacyOrbitCamera>)>,
    mut cameras: Query<&mut LegacyOrbitCamera>,
    roots: Query<(&NativeWorldSceneRoot, &NativeWorldPresentationStatus)>,
    heightmaps: Query<(&GlobalTransform, &NativeHeightmapCollider), With<NativeHeightmapTerrain>>,
    mut exit: MessageWriter<AppExit>,
) {
    if state.capture_saved {
        state.capture_saved = false;
        state.capture_issued = false;
        state.pending = None;
        state.pose_index += 1;
        state.pose_frame = 0;
        state.stable_frames = 0;
        if state.pose_index >= config.manifest.poses.len() {
            write_report(&config, &state.records);
            exit.write(AppExit::Success);
            return;
        }
        let pose = &config.manifest.poses[state.pose_index];
        let Ok(mut target) = targets.single_mut() else {
            eprintln!("capture target is missing or duplicated");
            exit.write(AppExit::error());
            return;
        };
        target.translation = pose.native_anchor();
        let Ok(mut camera) = cameras.single_mut() else {
            eprintln!("capture camera is missing or duplicated");
            exit.write(AppExit::error());
            return;
        };
        camera.yaw_degrees = pose.yaw_degrees;
        println!(
            "capture {}/{}: {} / {} ({})",
            state.pose_index + 1,
            config.manifest.poses.len(),
            pose.zone,
            pose.area,
            pose.id
        );
        return;
    }

    state.pose_frame = state.pose_frame.saturating_add(1);
    let pose = &config.manifest.poses[state.pose_index];
    if let Some(blocker) = &streaming.blocker {
        eprintln!("world streaming blocked at {}: {blocker}", pose.id);
        exit.write(AppExit::error());
        return;
    }
    if state.pose_frame >= config.manifest.camera.timeout_frames_per_pose {
        eprintln!(
            "world capture timed out at {}: resident={}, target={}, loadingColliders={}",
            pose.id, streaming.resident_tiles, streaming.target_tiles, streaming.loading_colliders
        );
        exit.write(AppExit::error());
        return;
    }

    let expected = catalog
        .legacy_stream_target_tiles(NativeWorldScope::WorldMap, pose.native_anchor())
        .into_iter()
        .collect::<BTreeSet<_>>();
    let loaded = roots
        .iter()
        .filter(|(root, _)| root.scope == NativeWorldScope::WorldMap)
        .map(|(root, _)| root.tile)
        .collect::<BTreeSet<_>>();
    let presentation_ready = roots
        .iter()
        .filter(|(root, _)| root.scope == NativeWorldScope::WorldMap)
        .all(|(_, status)| *status == NativeWorldPresentationStatus::Ready);
    let ready = !expected.is_empty()
        && loaded == expected
        && presentation_ready
        && streaming.loading_colliders == 0
        && !streaming.resident_budget_exceeded;
    if !ready {
        state.stable_frames = 0;
        std::thread::sleep(Duration::from_millis(16));
        return;
    }
    state.stable_frames = state.stable_frames.saturating_add(1);
    if state.capture_issued || state.stable_frames < config.manifest.camera.warmup_frames {
        std::thread::sleep(Duration::from_millis(16));
        return;
    }

    let anchor = pose.native_anchor();
    let terrain_height = heightmaps
        .iter()
        .filter_map(|(global, heightmap)| {
            heightmap.ground_height(global, anchor.x, anchor.z, -1_000_000.0, 1_000_000.0)
        })
        .max_by(f32::total_cmp);
    state.pending = Some(PendingCapture {
        resident_tiles: loaded.into_iter().collect(),
        terrain_height,
    });
    state.capture_issued = true;
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_screenshot);
}

fn save_screenshot(
    event: On<ScreenshotCaptured>,
    config: Res<PreviewConfig>,
    mut state: ResMut<PreviewState>,
) {
    let pose = &config.manifest.poses[state.pose_index];
    let pending = state
        .pending
        .take()
        .expect("capture was issued without readiness evidence");
    let image = event
        .image
        .clone()
        .try_into_dynamic()
        .expect("GPU screenshot must convert to a dynamic image");
    let rgba = image.to_rgba8();
    let background = *rgba.get_pixel(0, 0);
    let non_background_pixels = rgba.pixels().filter(|pixel| **pixel != background).count();
    assert!(
        non_background_pixels >= 1024,
        "rejecting nearly empty capture for {}: {non_background_pixels} non-background pixels",
        pose.id
    );
    let output = config.output_root.join(format!("{}.png", pose.id));
    image
        .save(&output)
        .unwrap_or_else(|error| panic!("cannot save {}: {error}", output.display()));
    let bytes = fs::read(&output)
        .unwrap_or_else(|error| panic!("cannot hash {}: {error}", output.display()));
    let png_sha256 = format!("{:X}", Sha256::digest(&bytes));
    let terrain_height = pending.terrain_height;
    state.records.push(CaptureRecord {
        pose_id: pose.id.clone(),
        era: pose.era.clone(),
        zone: pose.zone.clone(),
        area: pose.area.clone(),
        map_number: pose.map_number,
        safe_anchor_protocol: pose.safe_anchor_protocol,
        safe_anchor_native: pose.safe_anchor_native,
        terrain_height,
        anchor_height_above_terrain: terrain_height
            .map(|height| pose.safe_anchor_native[1] - height),
        resident_tiles: pending.resident_tiles,
        png: output.display().to_string(),
        png_sha256,
        width: rgba.width(),
        height: rgba.height(),
        non_background_pixels,
    });
    println!("saved {}", output.display());
    state.capture_saved = true;
}

fn write_report(config: &PreviewConfig, records: &[CaptureRecord]) {
    let report = CaptureReport {
        schema: REPORT_SCHEMA,
        source_manifest: config.manifest_path.display().to_string(),
        output_root: config.output_root.display().to_string(),
        captures: records.to_vec(),
    };
    let bytes = serde_json::to_vec_pretty(&report).expect("capture report must serialize");
    let path = config.output_root.join("capture-report.json");
    fs::write(&path, bytes)
        .unwrap_or_else(|error| panic!("cannot write {}: {error}", path.display()));
    println!("report {}", path.display());
}
