//! Isolated GPU diagnostic for Retrobution's Numbuh Two marker effects.
//!
//! This scene deliberately bypasses OpenFusion and the tutorial state machine.
//! The `full`/`far` views isolate the production NPC/effect loaders, while
//! `tutorial` adds the exact Tech Square tile, actor grounding and shallow
//! gameplay camera that expose ES668 projection failures.

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
    character_scene::{LegacyCharacterSceneDeferredReveal, LegacyCharacterSceneStatus},
    coordinates::unity_to_native_vector,
    legacy_model_material::{
        LegacyMaterialMetadataError, LegacyModelMaterial, LegacyModelMaterialPlugin,
    },
    movement::LegacyOrbitCamera,
    network_world_runtime::{NetworkNpcVisualCatalog0104, NetworkNpcVisualCatalogState0104},
    tutorial_actors::{
        TutorialActor, TutorialActorCommandQueue, TutorialActorPlugin, TutorialActorRegistry,
        TutorialActorScene, TutorialActorSet, apply_tutorial_actor_animation_playback,
        spawn_tutorial_actor_visuals, tutorial_actor_root_name,
    },
    tutorial_effects_runtime::{
        TutorialEffectLibrary, TutorialEffectPlacement, TutorialEffectRuntime,
        TutorialEffectRuntimeCommand, TutorialEffectsRuntimePlugin,
        process_tutorial_effect_runtime,
    },
    tutorial_logic::{LegacySpawnPosition, TutorialNpcSpawn},
    tutorial_mission_content::TutorialMissionContent,
    world::{
        NativeWorldCatalog, NativeWorldColliderStatus, NativeWorldPlugin, NativeWorldScope,
        load_native_world_scenes, spawn_native_world_scene,
    },
};

const ACTOR_ID: i32 = 1005;
const NUMBUH_TWO_TYPE: i32 = 2671;
const DEFAULT_READY_WARMUP_FRAMES: u32 = 180;
const TIMEOUT_FRAMES: u32 = 900;

#[derive(Resource)]
struct PreviewConfig {
    output: PathBuf,
    marker: MarkerKind,
    view: PreviewView,
    ready_warmup_frames: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PreviewView {
    Full,
    Far,
    Tutorial,
}

impl PreviewView {
    fn parse(value: Option<&str>) -> Option<Self> {
        match value.unwrap_or("full") {
            "full" => Some(Self::Full),
            "far" => Some(Self::Far),
            "tutorial" => Some(Self::Tutorial),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MarkerKind {
    Quest,
    Progress,
    Minimap,
    Smart,
    Bank,
    Scamper,
}

impl MarkerKind {
    fn parse(value: Option<&str>) -> Option<Self> {
        match value.unwrap_or("quest") {
            "quest" => Some(Self::Quest),
            "progress" => Some(Self::Progress),
            "minimap" => Some(Self::Minimap),
            "smart" => Some(Self::Smart),
            "bank" => Some(Self::Bank),
            "scamper" => Some(Self::Scamper),
            _ => None,
        }
    }

    const fn effect_id(self) -> i32 {
        match self {
            Self::Quest => 866,
            Self::Progress => 865,
            Self::Minimap | Self::Smart => 668,
            Self::Bank => 679,
            Self::Scamper => 683,
        }
    }
}

#[derive(Resource, Default)]
struct PreviewState {
    frames: u32,
    marker_enqueued: bool,
    marker_ready_frame: Option<u32>,
    attachment_match_count: usize,
    last_actor_y: Option<f32>,
    stable_actor_frames: u32,
    issues: Vec<String>,
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
        .unwrap_or_else(|| PathBuf::from("target/tutorial-mission-indicator-preview.png"));
    let marker_argument = args.next();
    let marker = MarkerKind::parse(marker_argument.as_deref().and_then(|value| value.to_str()))
        .unwrap_or_else(|| {
            eprintln!("marker must be quest, progress, minimap, smart, bank or scamper");
            std::process::exit(2);
        });
    let view = PreviewView::parse(args.next().as_deref().and_then(|value| value.to_str()))
        .unwrap_or_else(|| {
            eprintln!("view must be full, far or tutorial");
            std::process::exit(2);
        });
    let ready_warmup_frames = args
        .next()
        .map(|value| {
            value
                .to_str()
                .and_then(|value| value.parse::<u32>().ok())
                .unwrap_or_else(|| {
                    eprintln!("warmup frames must be an unsigned integer");
                    std::process::exit(2);
                })
        })
        .unwrap_or(DEFAULT_READY_WARMUP_FRAMES);
    if args.next().is_some() {
        eprintln!(
            "usage: tutorial_mission_indicator_gpu_preview [ASSET_ROOT] [OUTPUT.png] [quest|progress|minimap|smart|bank|scamper] [full|far|tutorial] [WARMUP_FRAMES]"
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
    let library = TutorialEffectLibrary::load(&asset_root)
        .unwrap_or_else(|error| panic!("cannot open tutorial effect catalog: {error}"));
    let asset_locator = AssetLocator::open(&asset_root)
        .unwrap_or_else(|error| panic!("cannot open asset locator: {error}"));
    let mission_content = TutorialMissionContent::open(&asset_locator)
        .unwrap_or_else(|error| panic!("cannot open tutorial mission content: {error}"));
    let mut npc_visual_state = NetworkNpcVisualCatalogState0104::default();
    npc_visual_state.catalog = Some(
        NetworkNpcVisualCatalog0104::open(&asset_locator)
            .unwrap_or_else(|error| panic!("cannot open NPC visual catalog: {error}")),
    );
    let world_catalog = load_native_world_scenes(&asset_root)
        .unwrap_or_else(|error| panic!("cannot open native world catalog: {error}"));
    if output.exists() {
        fs::remove_file(&output)
            .unwrap_or_else(|error| panic!("cannot replace {}: {error}", output.display()));
    }

    App::new()
        // A bright world-like background exposes contrast and projection
        // failures which the old black diagnostic background concealed.
        .insert_resource(ClearColor(match marker {
            MarkerKind::Quest | MarkerKind::Progress | MarkerKind::Bank | MarkerKind::Scamper => {
                Color::srgb(0.025, 0.04, 0.055)
            }
            MarkerKind::Minimap | MarkerKind::Smart => Color::srgb(0.16, 0.52, 0.48),
        }))
        .insert_resource(PreviewConfig {
            output,
            marker,
            view,
            ready_warmup_frames,
        })
        .insert_resource(PreviewState::default())
        .insert_resource(TutorialEffectRuntime::with_library(library))
        .insert_resource(mission_content)
        .insert_resource(npc_visual_state)
        .insert_resource::<NativeWorldCatalog>(world_catalog)
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: format!("FFOne ES{} isolated diagnostic", marker.effect_id()),
                        resolution: WindowResolution::new(768, 768),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((
            LegacyModelMaterialPlugin,
            NativeWorldPlugin,
            TutorialActorPlugin,
            TutorialEffectsRuntimePlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                spawn_tutorial_actor_visuals,
                apply_tutorial_actor_animation_playback,
                force_preview_actor_reveal,
            )
                .chain()
                .after(TutorialActorSet::AdvanceCombatAnimation),
        )
        .add_systems(
            Update,
            (
                lock_actor_to_debug_origin,
                enqueue_exact_marker
                    .after(spawn_tutorial_actor_visuals)
                    .before(process_tutorial_effect_runtime),
                collect_runtime_issues.after(process_tutorial_effect_runtime),
                drive_capture,
            ),
        )
        .run();
}

fn force_preview_actor_reveal(
    mut commands: Commands,
    mut scenes: Query<
        (Entity, &mut Visibility),
        (
            With<TutorialActorScene>,
            With<LegacyCharacterSceneDeferredReveal>,
        ),
    >,
) {
    for (entity, mut visibility) in &mut scenes {
        *visibility = Visibility::Inherited;
        commands
            .entity(entity)
            .remove::<LegacyCharacterSceneDeferredReveal>();
    }
}

fn setup(
    mut commands: Commands,
    config: Res<PreviewConfig>,
    asset_server: Res<AssetServer>,
    catalog: Res<NativeWorldCatalog>,
    mut actors: ResMut<TutorialActorCommandQueue>,
    mut effects: ResMut<TutorialEffectRuntime>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    effects.enqueue(TutorialEffectRuntimeCommand::Preload {
        effect_id: config.marker.effect_id(),
        source_line: line!(),
    });
    let world_view = config.view == PreviewView::Tutorial;
    let actor_spawn = if world_view {
        let actor_position = Vec3::new(-651.0, -82.0, 739.0);
        let scene = catalog
            .select_in_scope(NativeWorldScope::Tutorial, actor_position)
            .expect("tutorial tile_01_01 must cover Numbuh Two");
        spawn_native_world_scene(&mut commands, &asset_server, &catalog, scene)
            .expect("complete tutorial tile_01_01 must spawn");
        TutorialNpcSpawn::new(
            ACTOR_ID,
            NUMBUH_TWO_TYPE,
            LegacySpawnPosition::centiunits(65_100, 73_900, -8_200),
            Some(68),
        )
    } else {
        TutorialNpcSpawn::new(
            ACTOR_ID,
            NUMBUH_TWO_TYPE,
            LegacySpawnPosition::centiunits(0, 0, 0),
            Some(0),
        )
    };
    actors.spawn(actor_spawn);

    let camera_target = if world_view {
        Vec3::new(-651.0, -75.5, 739.0)
    } else {
        match config.marker {
            MarkerKind::Quest | MarkerKind::Progress | MarkerKind::Bank | MarkerKind::Scamper => {
                Vec3::new(0.0, 1.25, 0.0)
            }
            // Frame the complete ES668 column, not only its rigid marker. The
            // source object sits around y=14 after MinimapEvent's -1 offset while
            // waypoint/waypoint1/waypoint2 project onto the destination plane.
            MarkerKind::Minimap => Vec3::new(0.0, 6.5, 0.0),
            // SetParticleScale(2.4) scales the complete hierarchy, including the
            // authored y=15 device position, to y=36.
            MarkerKind::Smart => Vec3::new(0.0, 17.5, 0.0),
        }
    };
    let camera_translation = match (config.marker, config.view) {
        (_, PreviewView::Tutorial) => Vec3::new(-634.0, -78.0, 722.0),
        (MarkerKind::Quest | MarkerKind::Progress | MarkerKind::Bank | MarkerKind::Scamper, _) => {
            camera_target + Vec3::new(0.0, 0.4, -4.5)
        }
        (MarkerKind::Minimap, PreviewView::Far) => camera_target + Vec3::new(24.0, 13.0, -50.0),
        (MarkerKind::Minimap, PreviewView::Full) => camera_target + Vec3::new(9.5, 5.5, -21.0),
        (MarkerKind::Smart, PreviewView::Far) => camera_target + Vec3::new(32.0, 18.0, -72.0),
        (MarkerKind::Smart, PreviewView::Full) => camera_target + Vec3::new(18.0, 10.0, -52.0),
    };
    let target = commands
        .spawn((
            Name::new(format!(
                "ES{} debug camera target",
                config.marker.effect_id()
            )),
            Transform::from_translation(camera_target),
        ))
        .id();
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 45_f32.to_radians(),
            ..default()
        }),
        Transform::from_translation(camera_translation).looking_at(camera_target, Vec3::Y),
        LegacyOrbitCamera::new(target),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 8_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(-2.0, 4.0, -3.0).looking_at(Vec3::Y, Vec3::Y),
    ));
    if !matches!(
        config.marker,
        MarkerKind::Quest | MarkerKind::Progress | MarkerKind::Bank | MarkerKind::Scamper
    ) && !world_view
    {
        commands.spawn((
            Name::new("ES668 world-contrast road"),
            Mesh3d(meshes.add(Plane3d::default().mesh().size(80.0, 80.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.18, 0.21, 0.20),
                perceptual_roughness: 0.92,
                ..default()
            })),
            Transform::IDENTITY,
        ));
    }
}

fn lock_actor_to_debug_origin(
    config: Res<PreviewConfig>,
    mut actors: Query<&mut Transform, With<TutorialActor>>,
) {
    if config.view == PreviewView::Tutorial {
        return;
    }
    for mut transform in &mut actors {
        transform.translation = Vec3::ZERO;
    }
}

fn enqueue_exact_marker(
    config: Res<PreviewConfig>,
    content: Res<TutorialMissionContent>,
    registry: Res<TutorialActorRegistry>,
    actors: Query<&Transform, With<TutorialActor>>,
    names: Query<&Name>,
    children: Query<&Children>,
    scene_statuses: Query<&LegacyCharacterSceneStatus>,
    collider_statuses: Query<&NativeWorldColliderStatus>,
    mut runtime: ResMut<TutorialEffectRuntime>,
    mut state: ResMut<PreviewState>,
) {
    if state.marker_enqueued {
        return;
    }
    let Some(actor) = registry.entity(ACTOR_ID) else {
        return;
    };
    if config.view == PreviewView::Tutorial {
        if collider_statuses.is_empty()
            || collider_statuses
                .iter()
                .any(|status| matches!(status, NativeWorldColliderStatus::Loading))
        {
            return;
        }
        let Ok(transform) = actors.get(actor) else {
            return;
        };
        let previous_y = state.last_actor_y.replace(transform.translation.y);
        if previous_y.is_some_and(|value| (value - transform.translation.y).abs() <= 0.000_1) {
            state.stable_actor_frames = state.stable_actor_frames.saturating_add(1);
        } else {
            state.stable_actor_frames = 0;
        }
        if state.stable_actor_frames < 30 {
            return;
        }
        println!(
            "tutorial actor settled at {:?}; ES668 root y={:.4}",
            transform.translation,
            transform.translation.y - 1.0
        );
    }
    let placement = match config.marker {
        MarkerKind::Quest | MarkerKind::Progress => {
            if !descendant_scene_is_ready(actor, &children, &scene_statuses) {
                return;
            }
            let mut matches = Vec::new();
            collect_exact_named(actor, "GameIcon", &names, &children, &mut matches);
            state.attachment_match_count = matches.len();
            let (node_name, local_translation) = if matches.len() == 1 {
                ("GameIcon".to_owned(), Vec3::ZERO)
            } else {
                state.attachment_match_count = 1;
                let height = content
                    .gameplay_npc(NUMBUH_TWO_TYPE)
                    .expect("Numbuh Two gameplay row must exist")
                    .height();
                (
                    tutorial_actor_root_name(ACTOR_ID),
                    Vec3::Y * (height * 0.85),
                )
            };
            TutorialEffectPlacement::ExactBone {
                actor_id: ACTOR_ID,
                node_name,
                spawn_world_rotation: Quat::IDENTITY,
                local_translation_after_parenting: local_translation,
                local_rotation_after_parenting: Quat::IDENTITY,
            }
        }
        MarkerKind::Bank | MarkerKind::Scamper => {
            if !descendant_scene_is_ready(actor, &children, &scene_statuses) {
                return;
            }
            let mut matches = Vec::new();
            collect_exact_named(actor, "Bip01 Head", &names, &children, &mut matches);
            state.attachment_match_count = matches.len();
            if matches.len() != 1 {
                return;
            }
            TutorialEffectPlacement::ExactEntityBone {
                root_entity: actor,
                node_name: "Bip01 Head".to_owned(),
                spawn_world_rotation: Quat::IDENTITY,
                local_translation_after_parenting: unity_to_native_vector(Vec3::new(
                    -0.6, 0.0, 0.0,
                )),
                local_rotation_after_parenting: Quat::IDENTITY,
            }
        }
        MarkerKind::Minimap => {
            let Ok(transform) = actors.get(actor) else {
                return;
            };
            TutorialEffectPlacement::World {
                position: transform.translation + unity_to_native_vector(Vec3::new(0.0, -1.0, 0.0)),
                rotation: Quat::IDENTITY,
            }
        }
        MarkerKind::Smart => {
            let Ok(transform) = actors.get(actor) else {
                return;
            };
            state.attachment_match_count = 1;
            TutorialEffectPlacement::ExactBone {
                actor_id: ACTOR_ID,
                node_name: tutorial_actor_root_name(ACTOR_ID),
                spawn_world_rotation: transform.rotation,
                local_translation_after_parenting: Vec3::ZERO,
                local_rotation_after_parenting: Quat::IDENTITY,
            }
        }
    };
    let scale = match config.marker {
        MarkerKind::Smart => 136.0,
        MarkerKind::Quest
        | MarkerKind::Progress
        | MarkerKind::Minimap
        | MarkerKind::Bank
        | MarkerKind::Scamper => 1.0,
    };

    runtime.enqueue(TutorialEffectRuntimeCommand::Add {
        effect_id: config.marker.effect_id(),
        placement,
        scale,
        tracked: config.marker == MarkerKind::Minimap,
        name: Some(format!(
            "isolated Numbuh Two ES{}",
            config.marker.effect_id()
        )),
        destroy_after_seconds: None,
        source_line: line!(),
    });
    state.marker_enqueued = true;
    state.marker_ready_frame = Some(state.frames);
}

fn collect_exact_named(
    entity: Entity,
    target: &str,
    names: &Query<&Name>,
    children: &Query<&Children>,
    matches: &mut Vec<Entity>,
) {
    if names.get(entity).is_ok_and(|name| name.as_str() == target) {
        matches.push(entity);
    }
    if let Ok(descendants) = children.get(entity) {
        for child in descendants.iter() {
            collect_exact_named(child, target, names, children, matches);
        }
    }
}

fn descendant_scene_is_ready(
    entity: Entity,
    children: &Query<&Children>,
    statuses: &Query<&LegacyCharacterSceneStatus>,
) -> bool {
    if statuses
        .get(entity)
        .is_ok_and(|status| matches!(status, LegacyCharacterSceneStatus::Ready { .. }))
    {
        return true;
    }
    children.get(entity).is_ok_and(|descendants| {
        descendants
            .iter()
            .any(|child| descendant_scene_is_ready(child, children, statuses))
    })
}

fn collect_runtime_issues(
    mut runtime: ResMut<TutorialEffectRuntime>,
    mut state: ResMut<PreviewState>,
) {
    state
        .issues
        .extend(runtime.drain_issues().map(|issue| format!("{issue:?}")));
}

fn drive_capture(
    mut commands: Commands,
    config: Res<PreviewConfig>,
    mut state: ResMut<PreviewState>,
    runtime: Res<TutorialEffectRuntime>,
    materials: Res<Assets<LegacyModelMaterial>>,
    material_errors: Query<&LegacyMaterialMetadataError>,
    names: Query<&Name>,
    parents: Query<&ChildOf>,
    players: Query<(Entity, &AnimationPlayer)>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);
    if let Some(error) = material_errors.iter().next() {
        eprintln!("material error: {}", error.0);
        exit.write(AppExit::error());
        return;
    }
    if !state.issues.is_empty() {
        eprintln!(
            "ES{} runtime issues: {:#?}",
            config.marker.effect_id(),
            state.issues
        );
        exit.write(AppExit::error());
        return;
    }
    let warmed_up = state
        .marker_ready_frame
        .is_some_and(|ready| state.frames.saturating_sub(ready) >= config.ready_warmup_frames);
    let effect_mesh_name = format!("Original Unity mesh effect es{}", config.marker.effect_id());
    let effect_animation_playing = players.iter().any(|(entity, player)| {
        player.playing_animations().next().is_some()
            && ancestor_has_name(entity, &effect_mesh_name, &parents, &names)
    });
    if !state.capture_issued
        && warmed_up
        && runtime.active_native_instance_count() == 1
        && !materials.is_empty()
        && effect_animation_playing
    {
        println!(
            "ES{} ready: attachmentMatches={}, activeInstances={}, effectAnimationPlaying=true, warmupFrames={}",
            config.marker.effect_id(),
            state.attachment_match_count,
            runtime.active_native_instance_count(),
            config.ready_warmup_frames
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
            "ES{} diagnostic timed out: enqueued={}, attachmentMatches={}, activeInstances={}, issues={:?}",
            config.marker.effect_id(),
            state.marker_enqueued,
            state.attachment_match_count,
            runtime.active_native_instance_count(),
            state.issues
        );
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(16));
}

fn ancestor_has_name(
    mut entity: Entity,
    target: &str,
    parents: &Query<&ChildOf>,
    names: &Query<&Name>,
) -> bool {
    loop {
        if names.get(entity).is_ok_and(|name| name.as_str() == target) {
            return true;
        }
        let Ok(parent) = parents.get(entity) else {
            return false;
        };
        entity = parent.parent();
    }
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
