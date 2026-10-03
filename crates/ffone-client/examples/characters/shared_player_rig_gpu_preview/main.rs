//! GPU/runtime proof for the native shared player rig.
//!
//! Success requires all certified part skins to be rebound to the common actor
//! skeleton, the real named `stand1` clip to be playing, and an animated bone
//! rotation to change before the screenshot is accepted.

use std::{
    env, fs,
    path::{Path, PathBuf},
    time::Duration,
};

use bevy::{
    asset::AssetPlugin,
    camera::visibility::RenderLayers,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    window::{PresentMode, WindowResolution},
};
use ffone_client::{
    character_scene::{NativeSceneRole, native_scene_container_transform},
    legacy_model_material::{
        LegacyMaterialMetadataError, LegacyModelMaterial, LegacyModelMaterialPlugin,
    },
    player_shared_rig::{
        NativePlayerRigAssetCache, NativePlayerRigBones, NativePlayerRigCatalog,
        NativePlayerRigSpawnRequest, NativePlayerRigStand1Playback, NativePlayerRigStatus,
        NativePlayerSharedRigPlugin, spawn_native_player_rig,
    },
};
use ffone_runtime_contracts::{CharacterAppearanceCategory, PlayerRigGender};

const READY_WARMUP_FRAMES: u32 = 45;
const TIMEOUT_FRAMES: u32 = 900;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CreatorPartMode {
    Default,
    Random,
}

#[derive(Resource)]
struct PreviewConfig {
    output: PathBuf,
    rig_root: Entity,
}

#[derive(Resource, Default)]
struct PreviewState {
    frames: u32,
    ready_frame: Option<u32>,
    initial_spine_rotation: Option<Quat>,
    motion_observed: bool,
    capture_issued: bool,
    capture_saved: bool,
}

fn main() {
    let mut args = std::env::args_os().skip(1);
    let asset_root = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("assets/game"));
    let output = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/shared-player-rig-preview.png"));
    let gender = match args
        .next()
        .and_then(|value| value.into_string().ok())
        .as_deref()
    {
        None | Some("male") => PlayerRigGender::Male,
        Some("female") => PlayerRigGender::Female,
        Some(other) => {
            eprintln!("invalid gender {other:?}; expected male or female");
            std::process::exit(2);
        }
    };
    let part_mode = match args
        .next()
        .and_then(|value| value.into_string().ok())
        .as_deref()
    {
        None | Some("default") => CreatorPartMode::Default,
        Some("random") => CreatorPartMode::Random,
        Some(other) => {
            eprintln!("invalid creator part mode {other:?}; expected default or random");
            std::process::exit(2);
        }
    };
    if args.next().is_some() {
        eprintln!(
            "usage: shared_player_rig_gpu_preview [ASSET_ROOT] [OUTPUT.png] [male|female] [default|random]"
        );
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
    let catalog = NativePlayerRigCatalog::open(&asset_root)
        .unwrap_or_else(|error| panic!("cannot open native shared-rig catalog: {error}"));
    if output.exists() {
        fs::remove_file(&output)
            .unwrap_or_else(|error| panic!("cannot replace {}: {error}", output.display()));
    }

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.055)))
        .insert_resource(PreviewState::default())
        .insert_resource(catalog)
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne native shared player rig audit".into(),
                        resolution: WindowResolution::new(768, 768),
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((LegacyModelMaterialPlugin, NativePlayerSharedRigPlugin))
        .add_systems(
            Startup,
            move |mut commands: Commands,
                  assets: Res<AssetServer>,
                  mut asset_cache: ResMut<NativePlayerRigAssetCache>,
                  catalog: Res<NativePlayerRigCatalog>| {
                setup(
                    &mut commands,
                    &assets,
                    &mut asset_cache,
                    &catalog,
                    output.clone(),
                    gender,
                    part_mode,
                );
            },
        )
        .add_systems(Update, drive_capture)
        .run();
}

fn setup(
    commands: &mut Commands,
    assets: &AssetServer,
    asset_cache: &mut NativePlayerRigAssetCache,
    catalog: &NativePlayerRigCatalog,
    output: PathBuf,
    gender: PlayerRigGender,
    part_mode: CreatorPartMode,
) {
    let routes = creator_part_routes(catalog, gender, part_mode);
    println!("gender={gender:?}, partMode={part_mode:?}, exactRoutes={routes:?}");
    let mut request = NativePlayerRigSpawnRequest::new(
        format!("{gender:?} GPU proof"),
        1,
        gender,
        RenderLayers::default(),
    )
    .use_part_routes(catalog, routes)
    .expect("production creator routes");
    request.transform = native_scene_container_transform(NativeSceneRole::CharacterGameplay);
    let spawned = spawn_native_player_rig(commands, assets, asset_cache, catalog, request)
        .expect("spawn native shared player rig");

    // cnSimpleCharRenderCamera source values: Distance=2.3, fHeight=0.8,
    // vAngle.x=-10, FOV=45. Position the camera so its look vector is exactly
    // ten degrees down toward the authored height target.
    let target = Vec3::new(0.0, 0.8, 0.0);
    let camera_y = target.y + 2.3 * 10_f32.to_radians().tan();
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 45_f32.to_radians(),
            ..default()
        }),
        Transform::from_xyz(0.0, camera_y, -2.3).looking_at(target, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 8_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(-2.0, 4.0, -3.0).looking_at(target, Vec3::Y),
    ));
    commands.spawn((
        PointLight {
            intensity: 900_000.0,
            range: 8.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(1.2, 1.8, -2.0),
    ));
    commands.insert_resource(PreviewConfig {
        output,
        rig_root: spawned.root,
    });
}

fn creator_part_routes(
    catalog: &NativePlayerRigCatalog,
    gender: PlayerRigGender,
    mode: CreatorPartMode,
) -> Vec<String> {
    if mode == CreatorPartMode::Default {
        return catalog
            .default_creator_routes(gender)
            .expect("default creator routes");
    }
    let contract = catalog.gender(gender).expect("gender contract");
    [
        CharacterAppearanceCategory::Face,
        CharacterAppearanceCategory::Hair,
        CharacterAppearanceCategory::Shirt,
        CharacterAppearanceCategory::Pants,
        CharacterAppearanceCategory::Shoes,
    ]
    .into_iter()
    .enumerate()
    .map(|(ordinal, category)| {
        let mut routes = contract
            .creator_choices
            .iter()
            .filter(|choice| choice.appearance_category == category)
            .map(|choice| choice.exact_route.as_str())
            .collect::<Vec<_>>();
        routes.sort_unstable();
        routes.dedup();
        if gender == PlayerRigGender::Female && category == CharacterAppearanceCategory::Pants {
            return routes
                .iter()
                .find(|route| **route == "wear/f_pants_gothgirl.nif")
                .expect("recovered female Goth Girl pants route")
                .to_string();
        }
        let gender_offset = match gender {
            PlayerRigGender::Male => 7,
            PlayerRigGender::Female => 19,
        };
        routes[(gender_offset + ordinal * 13) % routes.len()].to_owned()
    })
    .collect()
}

fn drive_capture(
    mut commands: Commands,
    config: Res<PreviewConfig>,
    mut state: ResMut<PreviewState>,
    rigs: Query<(
        &NativePlayerRigStatus,
        &NativePlayerRigBones,
        &NativePlayerRigStand1Playback,
    )>,
    transforms: Query<&Transform>,
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
    let Ok((status, bones, playback)) = rigs.get(config.rig_root) else {
        if state.frames >= TIMEOUT_FRAMES {
            eprintln!("native shared-rig components did not become available");
            exit.write(AppExit::error());
        }
        std::thread::sleep(Duration::from_millis(16));
        return;
    };
    if let NativePlayerRigStatus::Blocked(error) = status {
        eprintln!("native shared-rig blocked: {error}");
        exit.write(AppExit::error());
        return;
    }
    if !status.is_ready() {
        if state.frames >= TIMEOUT_FRAMES {
            eprintln!("native shared-rig timed out in status {status:?}");
            exit.write(AppExit::error());
        }
        std::thread::sleep(Duration::from_millis(16));
        return;
    }

    let spine = bones
        .unique_by_true_name("Bip01 Spine")
        .expect("shared actor skeleton must have one Bip01 Spine");
    let rotation = transforms
        .get(spine)
        .expect("animated spine must retain Transform")
        .rotation;
    let frames = state.frames;
    let ready_frame = *state.ready_frame.get_or_insert(frames);
    let initial = *state.initial_spine_rotation.get_or_insert(rotation);
    if initial.angle_between(rotation) > 0.000_01 {
        state.motion_observed = true;
    }

    if !state.capture_issued
        && state.motion_observed
        && state.frames.saturating_sub(ready_frame) >= READY_WARMUP_FRAMES
        && !materials.is_empty()
    {
        state.capture_issued = true;
        println!(
            "ready={status:?}, animationPlayer={:?}, graph={:?}, motionRadians={:.8}",
            playback.animation_player,
            playback.animation_graph.id(),
            initial.angle_between(rotation)
        );
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }
    if state.capture_saved {
        exit.write(AppExit::Success);
    } else if state.frames >= TIMEOUT_FRAMES {
        eprintln!(
            "native shared-rig GPU proof timed out: status={status:?}, motionObserved={}",
            state.motion_observed
        );
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(16));
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
