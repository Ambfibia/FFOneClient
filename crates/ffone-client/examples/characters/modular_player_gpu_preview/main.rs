//! Deterministic native preview for the modular protocol-0104 player look.
//!
//! This harness intentionally spawns independent audited GLBs at one player
//! root.  It exists to prove whether their authored bind poses, origins and
//! scales form one visible avatar before the shared-skeleton runtime is
//! allowed to replace the temporary composition.

use std::{
    env, fs,
    path::{Path, PathBuf},
    time::Duration,
};

use bevy::{
    asset::{AssetPlugin, LoadState},
    gltf::GltfAssetLabel,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    window::{PresentMode, WindowResolution},
};
use ffone_client::{
    character_scene::{NativeSceneRole, native_scene_container_transform},
    legacy_model_material::{
        LegacyMaterialMetadataError, LegacyModelMaterial, LegacyModelMaterialPlugin,
    },
};

const WARMUP_FRAMES: u32 = 45;
const TIMEOUT_FRAMES: u32 = 600;

#[derive(Resource)]
struct PreviewConfig {
    output: PathBuf,
    scenes: Vec<Handle<WorldAsset>>,
}

#[derive(Resource, Default)]
struct PreviewState {
    frames: u32,
    capture_issued: bool,
    capture_saved: bool,
}

#[derive(Component)]
struct PreviewPart;

fn main() {
    let mut args = std::env::args_os().skip(1);
    let asset_root = args.next().map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from("target/equipment-logical-models-full-v2-player-assembly")
    });
    let output = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/modular-player-preview.png"));
    if args.next().is_some() {
        eprintln!("usage: modular_player_gpu_preview [ASSET_ROOT] [OUTPUT.png]");
        std::process::exit(2);
    }
    if output.extension().and_then(|value| value.to_str()) != Some("png") {
        eprintln!("output must end in .png");
        std::process::exit(2);
    }
    let asset_root = fs::canonicalize(&asset_root).unwrap_or_else(|error| {
        panic!(
            "cannot resolve asset root {}: {error}",
            asset_root.display()
        )
    });
    if output.exists() {
        fs::remove_file(&output)
            .unwrap_or_else(|error| panic!("cannot replace {}: {error}", output.display()));
    }

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.055)))
        .insert_resource(PreviewState::default())
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne modular player audit".into(),
                        resolution: WindowResolution::new(768, 768),
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(LegacyModelMaterialPlugin)
        .add_systems(
            Startup,
            move |mut commands: Commands, assets: Res<AssetServer>| {
                setup(&mut commands, &assets, output.clone());
            },
        )
        .add_systems(Update, drive_capture)
        .run();
}

fn setup(commands: &mut Commands, assets: &AssetServer, output: PathBuf) {
    let models = [
        "characters/player/items/mask/m_face_001_type01/models/m_face_001_type01/model.glb",
        "characters/player/items/head/m_head_004_type01/models/m_head_004_type01/model.glb",
        "characters/player/items/collections/m_pants_light/models/m_shirt_light/model.glb",
        "characters/player/items/pants/pants_unisexpants/models/m_pants_unisexpants/model.glb",
        "characters/player/items/shoes/f_shoes_militaryboots/models/m_shoes_militaryboots/model.glb",
    ];
    let player = commands
        .spawn((
            Name::new("Test Ser modular bind-pose preview"),
            native_scene_container_transform(NativeSceneRole::CharacterGameplay),
            Visibility::Inherited,
        ))
        .id();
    let scenes = models
        .into_iter()
        .map(|model| {
            let scene = assets.load(GltfAssetLabel::Scene(0).from_asset(model.to_owned()));
            commands.spawn((
                Name::new(model.to_owned()),
                PreviewPart,
                ChildOf(player),
                WorldAssetRoot(scene.clone()),
                Transform::IDENTITY,
            ));
            scene
        })
        .collect();

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.0, -3.2).looking_at(Vec3::new(0.0, 0.82, 0.0), Vec3::Y),
    ));
    commands.insert_resource(PreviewConfig { output, scenes });
}

fn drive_capture(
    mut commands: Commands,
    assets: Res<AssetServer>,
    config: Res<PreviewConfig>,
    mut state: ResMut<PreviewState>,
    parts: Query<Entity, With<PreviewPart>>,
    materials: Res<Assets<LegacyModelMaterial>>,
    material_errors: Query<&LegacyMaterialMetadataError>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);
    if let Some(error) = material_errors.iter().next() {
        eprintln!("material error: {}", error.0);
        exit.write(AppExit::error());
        return;
    }
    let scenes_loaded = config.scenes.iter().all(|scene| {
        matches!(
            assets.load_state(scene.id()),
            LoadState::Loaded | LoadState::Loading
        )
    });
    if !state.capture_issued
        && scenes_loaded
        && parts.iter().count() == config.scenes.len()
        && !materials.is_empty()
        && state.frames >= WARMUP_FRAMES
    {
        state.capture_issued = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }
    if state.capture_saved {
        exit.write(AppExit::Success);
    } else if state.frames >= TIMEOUT_FRAMES {
        eprintln!("modular player capture timed out");
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

fn save_screenshot(
    event: On<ScreenshotCaptured>,
    config: Res<PreviewConfig>,
    mut state: ResMut<PreviewState>,
) {
    let image = event
        .image
        .clone()
        .try_into_dynamic()
        .expect("GPU screenshot must convert to a dynamic image");
    if let Some(parent) = config
        .output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("cannot create {}: {error}", parent.display()));
    }
    image
        .save(&config.output)
        .unwrap_or_else(|error| panic!("cannot save {}: {error}", config.output.display()));
    println!("{}", absolute_display(&config.output));
    state.capture_saved = true;
}

fn absolute_display(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}
