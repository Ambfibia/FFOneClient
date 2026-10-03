//! Isolated GPU regression for Retrobution NPC run -> ready transitions.
//!
//! The scene bypasses OpenFusion, terrain streaming, and tutorial playback. It
//! drives the production Ben/Numbuh Five actor loaders through the same FIFO
//! sequence used at the end (and on skip) of `BasicMoveEvent`.

use std::{
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
    assets::AssetLocator,
    character_scene::LegacyCharacterSceneStatus,
    legacy_model_material::{
        LegacyMaterialMetadataError, LegacyModelMaterial, LegacyModelMaterialPlugin,
    },
    network_world_runtime::{
        NetworkNpcVisualCatalog0104, NetworkNpcVisualCatalogState0104,
        bind_network_npc_texture_variants_0104,
    },
    tutorial_actors::{
        TutorialActor, TutorialActorAnimationPlayback, TutorialActorCommandQueue,
        TutorialActorMotion, TutorialActorPlugin, TutorialActorPose, TutorialActorRegistry,
        TutorialActorSet, apply_tutorial_actor_animation_playback, spawn_tutorial_actor_visuals,
    },
    tutorial_logic::{LegacySpawnPosition, TutorialNpcSpawn},
    tutorial_mission_content::TutorialMissionContent,
};

const NUMBUH_FIVE_ID: i32 = 100;
const BEN_ID: i32 = 101;
const NUMBUH_FIVE_TYPE: i32 = 2669;
const BEN_TYPE: i32 = 2670;
const RUN_FRAMES: u32 = 60;
const READY_BLEND_FRAMES: u32 = 180;
const TIMEOUT_FRAMES: u32 = 900;

#[derive(Resource)]
struct PreviewConfig {
    output: PathBuf,
}

#[derive(Resource, Default)]
struct PreviewState {
    frames: u32,
    motion_started: bool,
    run_applied: bool,
    ready_requested_frame: Option<u32>,
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
        .unwrap_or_else(|| PathBuf::from("target/tutorial-actor-animation-preview.png"));
    if args.next().is_some() {
        eprintln!("usage: tutorial_actor_animation_gpu_preview [ASSET_ROOT] [OUTPUT.png]");
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
    let locator = AssetLocator::open(asset_root.clone())
        .unwrap_or_else(|error| panic!("cannot open asset root: {error}"));
    let content = TutorialMissionContent::open(&locator)
        .unwrap_or_else(|error| panic!("cannot open tutorial content: {error}"));
    let mut visual_catalog = NetworkNpcVisualCatalogState0104::default();
    visual_catalog.attempted = true;
    visual_catalog.catalog = Some(
        NetworkNpcVisualCatalog0104::open(&locator)
            .unwrap_or_else(|error| panic!("cannot open NPC visual catalog: {error}")),
    );

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.055)))
        .insert_resource(PreviewConfig { output })
        .insert_resource(PreviewState::default())
        .insert_resource(content)
        .insert_resource(visual_catalog)
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne BasicMove NPC animation audit".into(),
                        resolution: WindowResolution::new(900, 640),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((LegacyModelMaterialPlugin, TutorialActorPlugin))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                spawn_tutorial_actor_visuals,
                bind_network_npc_texture_variants_0104,
                apply_tutorial_actor_animation_playback,
            )
                .chain()
                .after(TutorialActorSet::AdvanceCombatAnimation),
        )
        .add_systems(
            Update,
            (
                hold_debug_floor,
                drive_animation_regression.after(apply_tutorial_actor_animation_playback),
            ),
        )
        .run();
}

fn setup(mut commands: Commands, mut actors: ResMut<TutorialActorCommandQueue>) {
    actors.spawn(TutorialNpcSpawn::new(
        NUMBUH_FIVE_ID,
        NUMBUH_FIVE_TYPE,
        LegacySpawnPosition::centiunits(-100, 0, 0),
        Some(180),
    ));
    actors.spawn(TutorialNpcSpawn::new(
        BEN_ID,
        BEN_TYPE,
        LegacySpawnPosition::centiunits(100, 0, 0),
        Some(180),
    ));

    let target = Vec3::new(0.0, 1.1, 0.0);
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 42_f32.to_radians(),
            ..default()
        }),
        Transform::from_xyz(0.0, 2.0, -12.0).looking_at(target, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 9_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(-2.0, 5.0, -4.0).looking_at(target, Vec3::Y),
    ));
    commands.spawn((
        PointLight {
            intensity: 800_000.0,
            range: 20.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(1.0, 2.5, -3.0),
    ));
}

fn hold_debug_floor(mut actors: Query<&mut Transform, With<TutorialActor>>) {
    for mut transform in &mut actors {
        transform.translation.y = 0.0;
    }
}

#[allow(clippy::too_many_arguments)]
fn drive_animation_regression(
    mut commands: Commands,
    mut state: ResMut<PreviewState>,
    registry: Res<TutorialActorRegistry>,
    actor_transforms: Query<&Transform, With<TutorialActor>>,
    poses: Query<&TutorialActorPose>,
    motions: Query<&TutorialActorMotion>,
    playback: Query<&TutorialActorAnimationPlayback>,
    scene_statuses: Query<&LegacyCharacterSceneStatus>,
    materials: Res<Assets<LegacyModelMaterial>>,
    material_errors: Query<&LegacyMaterialMetadataError>,
    mut actor_commands: ResMut<TutorialActorCommandQueue>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);
    if let Some(error) = material_errors.iter().next() {
        eprintln!("material error: {}", error.0);
        exit.write(AppExit::error());
        return;
    }
    let Some(numbuh_five) = registry.entity(NUMBUH_FIVE_ID) else {
        return;
    };
    let Some(ben) = registry.entity(BEN_ID) else {
        return;
    };

    if !state.motion_started {
        let Ok(numbuh_five_transform) = actor_transforms.get(numbuh_five) else {
            return;
        };
        let Ok(ben_transform) = actor_transforms.get(ben) else {
            return;
        };
        actor_commands.move_native(
            NUMBUH_FIVE_ID,
            numbuh_five_transform.translation + Vec3::X * 50.0,
            7.0,
        );
        actor_commands.move_native(BEN_ID, ben_transform.translation - Vec3::X * 50.0, 7.0);
        state.motion_started = true;
        return;
    }

    let run_applied = [numbuh_five, ben].into_iter().all(|actor_root| {
        playback
            .iter()
            .any(|applied| applied.actor_root == actor_root && applied.resolved_clip == Some("run"))
    });
    state.run_applied |= run_applied;
    if state.ready_requested_frame.is_none() && state.run_applied && state.frames >= RUN_FRAMES {
        for id in [NUMBUH_FIVE_ID, BEN_ID] {
            actor_commands.stop_motion(id);
            actor_commands.play_pose(id, "ready", false);
        }
        state.ready_requested_frame = Some(state.frames);
        return;
    }

    let Some(ready_requested_frame) = state.ready_requested_frame else {
        return;
    };
    let ready_pose = [numbuh_five, ben].into_iter().all(|entity| {
        poses
            .get(entity)
            .is_ok_and(|pose| pose.resolved_clip == Some("ready"))
    });
    let ready_applied = [numbuh_five, ben].into_iter().all(|actor_root| {
        playback.iter().any(|applied| {
            applied.actor_root == actor_root && applied.resolved_clip == Some("ready")
        })
    });
    let motion_stopped = [numbuh_five, ben]
        .into_iter()
        .all(|entity| motions.get(entity).is_err());
    let ready_scenes = scene_statuses
        .iter()
        .filter(|status| matches!(status, LegacyCharacterSceneStatus::Ready { .. }))
        .count();

    if !state.capture_issued
        && ready_pose
        && ready_applied
        && motion_stopped
        && ready_scenes >= 2
        && !materials.is_empty()
        && state.frames.saturating_sub(ready_requested_frame) >= READY_BLEND_FRAMES
    {
        println!(
            "runApplied={}, readyApplied={}, motionStopped={}, readyScenes={}",
            state.run_applied, ready_applied, motion_stopped, ready_scenes
        );
        state.capture_issued = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }
    if state.capture_saved {
        exit.write(AppExit::Success);
    } else if state.frames >= TIMEOUT_FRAMES {
        eprintln!(
            "NPC animation audit timed out: runApplied={}, readyPose={}, readyApplied={}, motionStopped={}",
            state.run_applied, ready_pose, ready_applied, motion_stopped
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
