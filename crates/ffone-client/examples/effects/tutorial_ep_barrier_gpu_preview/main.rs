//! GPU regression capture for the Retrobution EPbarrier under infected-zone fog.

use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use bevy::{
    asset::AssetPlugin,
    gltf::GltfAssetLabel,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    time::Virtual,
    window::{PresentMode, WindowResolution},
};
use ffone_client::{
    legacy_model_material::{
        LegacyMaterialApplied, LegacyMaterialMetadataError, LegacyModelMaterial,
        LegacyModelMaterialPlugin,
    },
    tutorial_ep_barrier::TutorialEpBarrierPlugin,
    world::SpawnedNativeWorldVisual,
};

const BARRIER_MODELS: [&str; 14] = [
    "objects/vehicles/motttt/models/plane11_dupe1/visual.glb",
    "objects/vehicles/motttt/models/plane11/visual.glb",
    "objects/vehicles/motttt/models/plane10_dupe1/visual.glb",
    "objects/vehicles/motttt/models/plane10/visual.glb",
    "objects/vehicles/motttt/models/plane08/visual.glb",
    "objects/vehicles/motttt/models/plane09/visual.glb",
    "objects/vehicles/motttt/models/plane07/visual.glb",
    "objects/vehicles/motttt/models/plane04/visual.glb",
    "objects/vehicles/motttt/models/plane06/visual.glb",
    "objects/vehicles/motttt/models/plane03/visual.glb",
    "objects/vehicles/motttt/models/plane01/visual.glb",
    "objects/vehicles/motttt/models/plane02/visual.glb",
    "objects/effects/epbarrier1_03_defaultsd_forcefieldgenerator1/models/epbarrier1_03_defaultsd_forcefieldgenerator1/visual.glb",
    "objects/effects/epbarrier1_03_default_forcefieldgenerator/models/epbarrier1_03_default_forcefieldgenerator/visual.glb",
];
const CAPTURE_PHASES: [f32; 5] = [0.25, 1.75, 3.25, 4.5, 6.25];
const CLIP_SECONDS: f32 = 5.999_995_7;
const EXPECTED_MATERIALS: usize = 14;

#[derive(Resource)]
struct PreviewOutput(PathBuf);

#[derive(Resource)]
struct PreviewState {
    started: Instant,
    schedule: Option<[f32; 5]>,
    next: usize,
    cyan_counts: [usize; 5],
    in_flight: bool,
    current_path: Option<PathBuf>,
    failed: bool,
}

impl Default for PreviewState {
    fn default() -> Self {
        Self {
            started: Instant::now(),
            schedule: None,
            next: 0,
            cyan_counts: [0; 5],
            in_flight: false,
            current_path: None,
            failed: false,
        }
    }
}

fn main() {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/epbarrier-gpu-preview"));
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game")
        .canonicalize()
        .expect("assets/game must exist");

    let app_exit = App::new()
        .insert_resource(ClearColor(Color::srgb(0.015, 0.002, 0.004)))
        .insert_resource(PreviewOutput(output))
        .insert_resource(PreviewState::default())
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne Retrobution EPbarrier GPU regression".into(),
                        resolution: WindowResolution::new(960, 960),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((LegacyModelMaterialPlugin, TutorialEpBarrierPlugin))
        .add_systems(Startup, setup_preview)
        .add_systems(Update, drive_capture)
        .run();
    if !app_exit.is_success() {
        std::process::exit(1);
    }
}

fn setup_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    virtual_time.set_relative_speed(4.0);
    for (index, model_path) in BARRIER_MODELS.into_iter().enumerate() {
        let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset(model_path));
        commands.spawn((
            Name::new(format!(
                "EPbarrier surface {index} [prefab:BuildPlayer-Map_12_03#13669:bundle#{index} visual]"
            )),
            SpawnedNativeWorldVisual {
                model_path: model_path.to_owned(),
                source_model_path: format!("EPbarrier1/v{index}"),
                scene: 0,
            },
            WorldAssetRoot(scene),
            source_transform(index),
        ));
    }

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 45.0).looking_at(Vec3::ZERO, Vec3::Y),
        DistanceFog {
            // Deliberately strong red ambience: an additive Unity pass must
            // fade toward black here, never add this fog color per wave.
            color: Color::srgb(0.8, 0.015, 0.01),
            directional_light_color: Color::NONE,
            falloff: FogFalloff::Exponential { density: 0.015 },
            ..default()
        },
    ));
}

fn source_transform(index: usize) -> Transform {
    // Clean-primary prefab placement, recentered from
    // BuildPlayer-Map_01_01#7084. The clip animates localScale on top of these
    // values; using identity makes the generator fifteen times too large and
    // depth-occludes every ring in a diagnostic scene.
    const RING_SCALES: [f32; 12] = [
        0.136_336_21,
        0.136_336_21,
        0.136_336_21,
        0.114_774_145,
        0.136_336_21,
        0.136_883_35,
        0.136_336_21,
        0.136_336_21,
        0.136_336_21,
        0.114_774_145,
        0.136_336_21,
        0.136_883_35,
    ];
    match index {
        0..=11 => Transform {
            rotation: Quat::from_xyzw(0.0, 0.0, 1.0, 4.371_139e-8),
            scale: Vec3::splat(RING_SCALES[index]),
            ..default()
        },
        12 => Transform {
            translation: Vec3::new(0.007_324_219, -0.345_573_43, 0.0),
            rotation: Quat::from_xyzw(-0.5, -0.499_999_94, -0.5, 0.500_000_06),
            scale: Vec3::splat(0.067_927_37),
        },
        13 => Transform {
            translation: Vec3::new(0.007_324_219, -0.339_439_4, 0.0),
            rotation: Quat::from_xyzw(0.499_999_97, 0.5, 0.5, -0.499_999_97),
            scale: Vec3::splat(0.066_707_39),
        },
        _ => unreachable!("EPbarrier preview model index is bounded"),
    }
}

fn drive_capture(
    mut commands: Commands,
    time: Res<Time>,
    output: Res<PreviewOutput>,
    applied: Query<(), With<LegacyMaterialApplied>>,
    errors: Query<&LegacyMaterialMetadataError>,
    roots: Query<(&SpawnedNativeWorldVisual, &Transform)>,
    material_handles: Query<&MeshMaterial3d<LegacyModelMaterial>>,
    materials: Res<Assets<LegacyModelMaterial>>,
    mut state: ResMut<PreviewState>,
    mut exit: MessageWriter<AppExit>,
) {
    if let Some(error) = errors.iter().next() {
        eprintln!("EPbarrier material error: {}", error.0);
        state.failed = true;
    }
    if state.schedule.is_none() && applied.iter().count() >= EXPECTED_MATERIALS {
        let now = time.elapsed_secs();
        let cycle_start = (now / CLIP_SECONDS).ceil() * CLIP_SECONDS;
        state.schedule = Some(CAPTURE_PHASES.map(|phase| cycle_start + phase));
    }

    if !state.failed
        && !state.in_flight
        && let Some(schedule) = state.schedule
        && state.next < schedule.len()
        && time.elapsed_secs() >= schedule[state.next]
    {
        let ring_materials = material_handles
            .iter()
            .filter_map(|handle| materials.get(&handle.0))
            .filter(|material| material.uniform.legacy_effect.w > 1.5)
            .collect::<Vec<_>>();
        let plane01_scale = roots
            .iter()
            .find(|(visual, _)| visual.model_path.contains("/plane01/"))
            .map(|(_, transform)| transform.scale);
        println!(
            "EPbarrier capture {} at {:.3}s: ringMaterials={}, plane01Scale={plane01_scale:?}, firstRing={:?}",
            state.next,
            time.elapsed_secs(),
            ring_materials.len(),
            ring_materials.first().map(|material| (
                material.uniform.base_color,
                material.uniform.emission,
                material.uniform.legacy_effect,
            )),
        );
        let path = output.0.join(format!("phase-{:02}.png", state.next));
        state.current_path = Some(path);
        state.in_flight = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }

    if state.next == CAPTURE_PHASES.len() {
        let (before_wrap, after_wrap) = (state.cyan_counts[0], state.cyan_counts[4]);
        let ratio = before_wrap.max(after_wrap) as f32 / before_wrap.min(after_wrap).max(1) as f32;
        if ratio <= 1.35 {
            println!(
                "EPbarrier loop continuity: before={before_wrap}, after={after_wrap}, ratio={ratio:.3}"
            );
            exit.write(AppExit::Success);
        } else {
            eprintln!(
                "EPbarrier loop discontinuity: before={before_wrap}, after={after_wrap}, ratio={ratio:.3}"
            );
            exit.write(AppExit::error());
        }
    } else if state.failed || state.started.elapsed() > Duration::from_secs(20) {
        if !state.failed {
            eprintln!("EPbarrier GPU capture timed out");
        }
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

fn save_screenshot(event: On<ScreenshotCaptured>, mut state: ResMut<PreviewState>) {
    let path = state
        .current_path
        .take()
        .expect("capture path must be selected before screenshot");
    let image = event
        .image
        .clone()
        .try_into_dynamic()
        .expect("GPU screenshot must convert to a dynamic image");
    let rgba = image.to_rgba8();
    let mut cyan = 0_usize;
    let mut red = 0_usize;
    for pixel in rgba.pixels() {
        let [r, g, b, _] = pixel.0;
        cyan += usize::from(b > r.saturating_add(12) && g > r.saturating_add(8));
        red += usize::from(r > b.saturating_add(24) && r > g.saturating_add(24));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("cannot create {}: {error}", parent.display()));
    }
    image
        .save(&path)
        .unwrap_or_else(|error| panic!("cannot save {}: {error}", path.display()));
    if cyan < 64 || cyan <= red {
        eprintln!(
            "rejecting EPbarrier frame {}: cyan={cyan}, red={red}",
            state.next
        );
        state.failed = true;
    } else {
        println!(
            "EPbarrier phase {}: cyan={cyan}, red={red}, {}",
            state.next,
            path.display()
        );
        let completed = state.next;
        state.cyan_counts[completed] = cyan;
        state.next += 1;
    }
    state.in_flight = false;
}
