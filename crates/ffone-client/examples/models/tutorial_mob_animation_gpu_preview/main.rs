//! GPU regression for the four tutorial mobs whose combat poses exercise the
//! recovered Retrobution NPC high animation layer.

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
    legacy_npc_nano_animation::LegacyNpcAnimationRole,
    network_world_runtime::{
        NetworkNpcVisualCatalog0104, NetworkNpcVisualCatalogState0104,
        bind_network_npc_texture_variants_0104,
    },
    tutorial_actors::{
        TutorialActor, TutorialActorAnimationPlayback, TutorialActorCommandQueue,
        TutorialActorPlugin, TutorialActorPose, TutorialActorRegistry, TutorialActorSet,
        apply_tutorial_actor_animation_playback, spawn_tutorial_actor_visuals,
    },
    tutorial_logic::{LegacySpawnPosition, TutorialNpcSpawn},
    tutorial_mission_content::TutorialMissionContent,
};

const MOB_IDS_AND_TYPES: [(i32, i32); 4] = [
    (267_400, 2674),
    (267_500, 2675),
    (267_600, 2676),
    (267_800, 2678),
];
const COMBAT_CAPTURE_DELAY_FRAMES: u32 = 14;
const TIMEOUT_FRAMES: u32 = 900;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PreviewMode {
    Attack,
    Damage,
}

#[derive(Resource)]
struct PreviewConfig {
    output: PathBuf,
    mode: PreviewMode,
}

#[derive(Resource, Default)]
struct PreviewState {
    frames: u32,
    ready_requested_frame: Option<u32>,
    ready_observed: bool,
    action_requested_frame: Option<u32>,
    action_applied: bool,
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
        .unwrap_or_else(|| PathBuf::from("target/tutorial-mob-animation-preview.png"));
    let mode = match args
        .next()
        .and_then(|value| value.to_str().map(str::to_owned))
    {
        None => PreviewMode::Attack,
        Some(value) if value == "attack" => PreviewMode::Attack,
        Some(value) if value == "damage" => PreviewMode::Damage,
        Some(value) => {
            eprintln!("unknown preview mode {value:?}; expected attack or damage");
            std::process::exit(2);
        }
    };
    if args.next().is_some() {
        eprintln!(
            "usage: tutorial_mob_animation_gpu_preview [ASSET_ROOT] [OUTPUT.png] [attack|damage]"
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
    if output.exists() {
        fs::remove_file(&output)
            .unwrap_or_else(|error| panic!("cannot replace {}: {error}", output.display()));
    }

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.055)))
        .insert_resource(PreviewConfig { output, mode })
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
                        title: "FFOne tutorial mob animation audit".into(),
                        resolution: WindowResolution::new(1100, 700),
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
    for ((id, npc_type), x) in MOB_IDS_AND_TYPES.into_iter().zip([-675, -225, 225, 675]) {
        actors.spawn(TutorialNpcSpawn::new(
            id,
            npc_type,
            LegacySpawnPosition::centiunits(x, 0, 0),
            Some(if npc_type == 2674 { 0 } else { 180 }),
        ));
    }

    let target = Vec3::new(0.0, 1.5, 0.0);
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 43_f32.to_radians(),
            ..default()
        }),
        Transform::from_xyz(0.0, 3.0, -17.0).looking_at(target, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(-3.0, 6.0, -4.0).looking_at(target, Vec3::Y),
    ));
    commands.spawn((
        PointLight {
            intensity: 1_000_000.0,
            range: 25.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(2.0, 4.0, -5.0),
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
    config: Res<PreviewConfig>,
    mut state: ResMut<PreviewState>,
    registry: Res<TutorialActorRegistry>,
    transforms: Query<&Transform, With<TutorialActor>>,
    poses: Query<&TutorialActorPose>,
    playback: Query<(&TutorialActorAnimationPlayback, &AnimationPlayer)>,
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
    let Some(entities) = MOB_IDS_AND_TYPES
        .map(|(id, _)| registry.entity(id))
        .into_iter()
        .collect::<Option<Vec<_>>>()
    else {
        return;
    };

    if state.ready_requested_frame.is_none() {
        for (id, _) in MOB_IDS_AND_TYPES {
            actor_commands.play_pose(id, "ready", false);
        }
        state.ready_requested_frame = Some(state.frames);
        return;
    }

    let all_scenes_ready = scene_statuses
        .iter()
        .filter(|status| matches!(status, LegacyCharacterSceneStatus::Ready { .. }))
        .count()
        >= MOB_IDS_AND_TYPES.len();
    let ready_applied = entities.iter().all(|actor_root| {
        playback.iter().any(|(applied, _)| {
            applied.actor_root == *actor_root && applied.resolved_clip == Some("ready")
        })
    });
    let every_base_node_live = entities.iter().all(|actor_root| {
        playback.iter().any(|(applied, player)| {
            applied.actor_root == *actor_root
                && !applied.terminally_unavailable
                && applied
                    .node
                    .is_some_and(|node| player.animation(node).is_some())
        })
    });
    if state.ready_observed && !every_base_node_live {
        eprintln!("mob animation audit observed an empty base-animation frame");
        exit.write(AppExit::error());
        return;
    }
    if state.action_requested_frame.is_none() {
        state.ready_observed |= ready_applied;
    }
    if state.action_requested_frame.is_none()
        && all_scenes_ready
        && ready_applied
        && !materials.is_empty()
        && state
            .ready_requested_frame
            .is_some_and(|frame| state.frames.saturating_sub(frame) >= 30)
    {
        for ((id, _), entity) in MOB_IDS_AND_TYPES.into_iter().zip(&entities) {
            match config.mode {
                PreviewMode::Attack => {
                    let player_position = transforms
                        .get(*entity)
                        .expect("spawned mob must keep its transform")
                        .translation;
                    actor_commands.attempt_player_attack(id, player_position);
                }
                PreviewMode::Damage => actor_commands.damage(id, 1),
            }
        }
        state.action_requested_frame = Some(state.frames);
        return;
    }

    let Some(action_requested_frame) = state.action_requested_frame else {
        return;
    };
    let action_role = match config.mode {
        PreviewMode::Attack => LegacyNpcAnimationRole::Melee,
        PreviewMode::Damage => LegacyNpcAnimationRole::Wound,
    };
    let action_pose = entities.iter().all(|entity| {
        poses
            .get(*entity)
            .is_ok_and(|pose| pose.role == action_role)
    });
    let action_applied = entities.iter().all(|actor_root| {
        playback.iter().any(|(applied, _)| {
            applied.actor_root == *actor_root
                && match config.mode {
                    PreviewMode::Attack => {
                        matches!(applied.resolved_clip, Some("melee1" | "melee2"))
                    }
                    PreviewMode::Damage => applied.resolved_clip == Some("wound"),
                }
        })
    });
    state.action_applied |= action_pose && action_applied;
    if !state.capture_issued
        && state.action_applied
        && state.frames.saturating_sub(action_requested_frame) >= COMBAT_CAPTURE_DELAY_FRAMES
    {
        println!(
            "mode={:?}, readyObserved={}, actionPose={action_pose}, actionApplied={action_applied}",
            config.mode, state.ready_observed
        );
        state.capture_issued = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }

    let returned_to_ready = state.capture_saved
        && entities.iter().all(|entity| {
            poses
                .get(*entity)
                .is_ok_and(|pose| pose.role == LegacyNpcAnimationRole::Ready)
        })
        && ready_applied;
    if returned_to_ready {
        println!(
            "mode={:?}, actionApplied={}, returnedReady={returned_to_ready}",
            config.mode, state.action_applied
        );
        exit.write(AppExit::Success);
    } else if state.frames >= TIMEOUT_FRAMES {
        eprintln!(
            "mob animation audit timed out: mode={:?}, scenesReady={all_scenes_ready}, readyApplied={ready_applied}, actionPose={action_pose}, actionApplied={action_applied}",
            config.mode
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
