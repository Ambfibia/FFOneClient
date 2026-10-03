//! Focused GPU regression for Retrobution tutorial attack VFX.
//!
//! Each actor case plays the production animation and waits for the native
//! playback-clock bridge to emit the exact Unity `AnimationEvent` payload,
//! then captures an early and late frame through the production renderer.

use std::{
    env, fs,
    path::{Path, PathBuf},
    str::FromStr,
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
    movement::LegacyOrbitCamera,
    network_world_runtime::{
        NetworkNpcVisualCatalog0104, NetworkNpcVisualCatalogState0104,
        bind_network_npc_texture_variants_0104,
    },
    tutorial_actors::{
        TutorialActor, TutorialActorCommandQueue, TutorialActorEvent, TutorialActorEventQueue,
        TutorialActorPlugin, TutorialActorRegistry, TutorialActorSet,
        apply_tutorial_actor_animation_playback, spawn_tutorial_actor_visuals,
    },
    tutorial_choreography::{ClientVec3, EntityRef, RotationExpr},
    tutorial_choreography_formula::{TutorialCameraCaptureStore, resolve_rotation},
    tutorial_effects_runtime::{
        TutorialEffectLibrary, TutorialEffectPlacement, TutorialEffectRuntime,
        TutorialEffectRuntimeCommand, TutorialEffectsRuntimePlugin, TutorialProjectileMotion,
        TutorialProjectileVisualReadiness, process_tutorial_effect_runtime,
        tutorial_oni_velocity_samples_from_unity_unit_draws,
    },
    tutorial_logic::{LegacySpawnPosition, TutorialNpcSpawn},
    tutorial_mission_content::TutorialMissionContent,
};

const ACTOR_ID: i32 = 9001;
const TIMEOUT_FRAMES: u32 = 1_200;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PreviewCase {
    NumbuhOne,
    Dexter,
    NumbuhFive,
    SamuraiJack,
    FusionSpawnAttack,
    FusionSpawnDeath,
    FusionButtercupCorruptak,
    FusionButtercupMegatak,
    FusionBlooCorruptak,
    FusionBlooMegatak,
    FusionMacCorruptak,
    FusionMacMegatak,
    InfectionAura,
    InfectionDamage,
    InfectionProtection,
    NanoSummon,
    NpcSkill(i32),
    Rifle,
}

impl PreviewCase {
    fn slug(self) -> &'static str {
        match self {
            Self::NumbuhOne => "numbuh-one",
            Self::Dexter => "dexter",
            Self::NumbuhFive => "numbuh-five",
            Self::SamuraiJack => "samurai-jack",
            Self::FusionSpawnAttack => "fusion-spawn-attack",
            Self::FusionSpawnDeath => "fusion-spawn",
            Self::FusionButtercupCorruptak => "fusion-buttercup-corruptak",
            Self::FusionButtercupMegatak => "fusion-buttercup-megatak",
            Self::FusionBlooCorruptak => "fusion-bloo-corruptak",
            Self::FusionBlooMegatak => "fusion-bloo-megatak",
            Self::FusionMacCorruptak => "fusion-mac-corruptak",
            Self::FusionMacMegatak => "fusion-mac-megatak",
            Self::InfectionAura => "infection-aura",
            Self::InfectionDamage => "infection-damage",
            Self::InfectionProtection => "infection-protection",
            Self::NanoSummon => "nano-summon",
            Self::NpcSkill(530) => "corruption-a",
            Self::NpcSkill(531) => "corruption-b",
            Self::NpcSkill(532) => "corruption-c",
            Self::NpcSkill(592) => "fusion-corruption",
            Self::NpcSkill(_) => "eruption",
            Self::Rifle => "rifle",
        }
    }

    fn actor(self) -> Option<(i32, &'static str)> {
        match self {
            Self::NumbuhOne => Some((2667, "melee1")),
            Self::Dexter => Some((2668, "melee1")),
            Self::NumbuhFive => Some((2669, "melee1")),
            Self::SamuraiJack => Some((2666, "melee1event")),
            Self::FusionSpawnAttack => Some((2674, "melee1")),
            Self::FusionSpawnDeath => Some((2674, "death")),
            Self::FusionButtercupCorruptak
            | Self::FusionButtercupMegatak
            | Self::FusionBlooCorruptak
            | Self::FusionBlooMegatak
            | Self::FusionMacCorruptak
            | Self::FusionMacMegatak
            | Self::InfectionAura
            | Self::InfectionDamage
            | Self::InfectionProtection
            | Self::NanoSummon
            | Self::NpcSkill(_)
            | Self::Rifle => None,
        }
    }

    fn animation_event_seconds(self) -> f32 {
        match self {
            Self::NumbuhOne => 0.180_000_01,
            Self::Dexter => 0.264,
            Self::NumbuhFive => 0.233_333,
            Self::SamuraiJack => 0.133_333,
            // NpcMoveController.Attack returns m_iEffect in the same update
            // that AttackMotion starts; this is not an AnimationClip event.
            Self::FusionSpawnAttack
            | Self::FusionSpawnDeath
            | Self::FusionButtercupCorruptak
            | Self::FusionButtercupMegatak
            | Self::FusionBlooCorruptak
            | Self::FusionBlooMegatak
            | Self::FusionMacCorruptak
            | Self::FusionMacMegatak
            | Self::InfectionAura
            | Self::InfectionDamage
            | Self::InfectionProtection => 0.0,
            Self::NanoSummon | Self::NpcSkill(_) => 0.0,
            Self::Rifle => 0.0,
        }
    }

    fn capture_offsets(self) -> [f32; 2] {
        match self {
            Self::NumbuhOne | Self::Dexter | Self::NumbuhFive => [0.20, 0.75],
            Self::SamuraiJack => [0.42, 1.12],
            // BulletTable 106 hides for 0.3 s, travels for 0.3 s, then emits
            // its only visible combat payload: success ES653.
            Self::FusionSpawnAttack => [0.68, 1.20],
            Self::FusionSpawnDeath => [0.12, 0.72],
            Self::FusionButtercupCorruptak => [0.06, 0.20],
            Self::FusionButtercupMegatak => [0.43, 0.60],
            Self::FusionBlooCorruptak => [0.12, 0.32],
            Self::FusionBlooMegatak => [0.36, 0.40],
            Self::FusionMacCorruptak => [0.37, 0.52],
            Self::FusionMacMegatak => [0.60, 0.72],
            Self::InfectionAura => [0.12, 0.72],
            Self::InfectionDamage => [0.06, 0.28],
            Self::InfectionProtection => [0.06, 0.30],
            Self::NanoSummon => [0.08, 0.45],
            Self::NpcSkill(_) => [0.12, 0.65],
            Self::Rifle => [0.12, 0.26],
        }
    }
}

impl FromStr for PreviewCase {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "numbuh-one" => Ok(Self::NumbuhOne),
            "dexter" => Ok(Self::Dexter),
            "numbuh-five" => Ok(Self::NumbuhFive),
            "samurai-jack" => Ok(Self::SamuraiJack),
            "fusion-spawn" | "fusion-spawn-death" => Ok(Self::FusionSpawnDeath),
            "fusion-spawn-attack" => Ok(Self::FusionSpawnAttack),
            "fusion-buttercup-corruptak" => Ok(Self::FusionButtercupCorruptak),
            "fusion-buttercup-megatak" => Ok(Self::FusionButtercupMegatak),
            "fusion-bloo-corruptak" => Ok(Self::FusionBlooCorruptak),
            "fusion-bloo-megatak" => Ok(Self::FusionBlooMegatak),
            "fusion-mac-corruptak" => Ok(Self::FusionMacCorruptak),
            "fusion-mac-megatak" => Ok(Self::FusionMacMegatak),
            "infection-aura" => Ok(Self::InfectionAura),
            "infection-damage" => Ok(Self::InfectionDamage),
            "infection-protection" => Ok(Self::InfectionProtection),
            "nano-summon" => Ok(Self::NanoSummon),
            "corruption-a" => Ok(Self::NpcSkill(530)),
            "corruption-b" => Ok(Self::NpcSkill(531)),
            "corruption-c" => Ok(Self::NpcSkill(532)),
            "fusion-corruption" => Ok(Self::NpcSkill(592)),
            "eruption" => Ok(Self::NpcSkill(766)),
            "rifle" => Ok(Self::Rifle),
            _ => Err(format!(
                "unknown case {value:?}; expected numbuh-one|dexter|numbuh-five|samurai-jack|fusion-spawn|fusion-spawn-attack|fusion-buttercup-corruptak|fusion-buttercup-megatak|fusion-bloo-corruptak|fusion-bloo-megatak|fusion-mac-corruptak|fusion-mac-megatak|infection-aura|infection-damage|infection-protection|nano-summon|rifle"
            )),
        }
    }
}

fn is_direct_effect(case: PreviewCase) -> bool {
    matches!(
        case,
        PreviewCase::FusionButtercupCorruptak
            | PreviewCase::FusionButtercupMegatak
            | PreviewCase::FusionBlooCorruptak
            | PreviewCase::FusionBlooMegatak
            | PreviewCase::FusionMacCorruptak
            | PreviewCase::FusionMacMegatak
            | PreviewCase::InfectionAura
            | PreviewCase::InfectionDamage
            | PreviewCase::InfectionProtection
            | PreviewCase::NanoSummon
            | PreviewCase::NpcSkill(_)
    )
}

#[derive(Resource)]
struct PreviewConfig {
    case: PreviewCase,
    outputs: Vec<PathBuf>,
    capture_times: Vec<f32>,
}

#[derive(Resource, Default)]
struct PreviewState {
    frames: u32,
    preload_requested: bool,
    actor_ready_at: Option<f32>,
    playback_started_at: Option<f32>,
    effect_started_at: Option<f32>,
    capture_index: usize,
    capture_pending: bool,
    effect_ready_at: Option<f32>,
    pose_samples: Vec<serde_json::Value>,
}

fn main() {
    let mut args = std::env::args_os().skip(1);
    let asset_root = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("assets/game"));
    let case = args
        .next()
        .and_then(|value| value.to_str().map(str::to_owned))
        .unwrap_or_else(|| "numbuh-one".to_owned())
        .parse::<PreviewCase>()
        .unwrap_or_else(|error| panic!("{error}"));
    let early_output = args.next().map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(format!(
            "target/tutorial-combat-vfx-{}-early.png",
            case.slug()
        ))
    });
    if args.next().is_some()
        || early_output.extension().and_then(|value| value.to_str()) != Some("png")
    {
        eprintln!("usage: tutorial_combat_vfx_gpu_preview [ASSET_ROOT] [CASE] [EARLY_OUTPUT.png]");
        std::process::exit(2);
    }
    let late_output = suffixed_png(&early_output, "late");
    let capture_times = if case == PreviewCase::NpcSkill(766) {
        vec![0.02, 0.12, 0.35, 0.65, 1.05, 1.60, 2.10]
    } else {
        case.capture_offsets().to_vec()
    };
    let mut outputs = vec![early_output.clone(), late_output];
    for index in 2..capture_times.len() {
        outputs.push(suffixed_png(&early_output, &format!("phase-{index}")));
    }
    let asset_root = fs::canonicalize(&asset_root).unwrap_or_else(|error| {
        panic!(
            "cannot resolve asset root {}: {error}",
            asset_root.display()
        )
    });
    for output in &outputs {
        if output.exists() {
            fs::remove_file(output)
                .unwrap_or_else(|error| panic!("cannot replace {}: {error}", output.display()));
        }
    }
    let effect_library = TutorialEffectLibrary::load(&asset_root)
        .unwrap_or_else(|error| panic!("cannot load tutorial effect library: {error}"));
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

    let mut effect_runtime = TutorialEffectRuntime::with_library(effect_library);
    effect_runtime
        .load_native_particle_catalog(&locator)
        .unwrap();
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.018, 0.025, 0.045)))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.55, 0.62, 0.78),
            brightness: 260.0,
            affects_lightmapped_meshes: true,
        })
        .insert_resource(PreviewConfig {
            case,
            outputs,
            capture_times,
        })
        .insert_resource(if case == PreviewCase::NpcSkill(766) {
            bevy::time::TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0))
        } else {
            bevy::time::TimeUpdateStrategy::default()
        })
        .insert_resource(PreviewState::default())
        .insert_resource(effect_runtime)
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
                        title: format!("FFOne tutorial combat VFX audit: {}", case.slug()),
                        resolution: WindowResolution::new(960, 640),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((
            LegacyModelMaterialPlugin,
            TutorialActorPlugin,
            TutorialEffectsRuntimePlugin,
        ))
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
            (hold_debug_floor, drive_preview)
                .chain()
                .after(process_tutorial_effect_runtime),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    config: Res<PreviewConfig>,
    mut actors: ResMut<TutorialActorCommandQueue>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Some((npc_type, _)) = config.case.actor() {
        actors.spawn(TutorialNpcSpawn::new(
            ACTOR_ID,
            npc_type,
            LegacySpawnPosition::centiunits(0, 0, 0),
            Some(if config.case == PreviewCase::SamuraiJack {
                194
            } else if matches!(
                config.case,
                PreviewCase::FusionSpawnAttack | PreviewCase::FusionSpawnDeath
            ) {
                // The production Combat A Spawn turns toward the player
                // before attacking. Face this focused camera so the eye and
                // high-layer animation regression observes the same side.
                0
            } else {
                180
            }),
        ));
    }

    let (camera_focus, camera_transform) = if config.case == PreviewCase::SamuraiJack {
        // BasicMove targets (883,-32,693) while Jack stands at
        // (883,-34,693): preserve the exact two-unit vertical offset.
        let focus = Vec3::new(0.0, 2.0, 0.0);
        let orbit = resolve_rotation(
            RotationExpr::Euler(ClientVec3::new(10.0, 194.0, 0.0)),
            focus,
            &mut |_entity: EntityRef| None,
            &TutorialCameraCaptureStore::default(),
        )
        .expect("exact Samurai Jack tutorial camera rotation");
        let position = focus + orbit * Vec3::new(0.0, 0.0, -4.0);
        (
            focus,
            Transform::from_translation(position).looking_at(focus, Vec3::Y),
        )
    } else if matches!(
        config.case,
        PreviewCase::FusionSpawnAttack | PreviewCase::FusionSpawnDeath
    ) {
        let focus = Vec3::new(1.2, 0.9, 0.0);
        (
            focus,
            Transform::from_xyz(3.8, 2.5, -5.8).looking_at(focus, Vec3::Y),
        )
    } else if config.case == PreviewCase::FusionButtercupCorruptak {
        let focus = Vec3::new(0.0, 1.2, 1.5);
        (
            focus,
            Transform::from_xyz(2.6, 2.35, -4.6).looking_at(focus, Vec3::Y),
        )
    } else if config.case == PreviewCase::NpcSkill(766) {
        let focus = Vec3::new(0.0, 4.0, 0.0);
        (
            focus,
            Transform::from_xyz(11.0, 8.0, -19.0).looking_at(focus, Vec3::Y),
        )
    } else if is_direct_effect(config.case) {
        let focus = Vec3::new(0.0, 0.9, 0.8);
        (
            focus,
            Transform::from_xyz(1.25, 1.55, -2.35).looking_at(focus, Vec3::Y),
        )
    } else {
        let focus = Vec3::new(0.0, 1.2, 1.5);
        (
            focus,
            Transform::from_xyz(7.4, 4.8, -13.0).looking_at(focus, Vec3::Y),
        )
    };
    let camera_target = commands
        .spawn(Transform::from_translation(camera_focus))
        .id();
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            // Production tutorial camera projection in main.rs.
            fov: 45_f32.to_radians(),
            ..default()
        }),
        camera_transform,
        LegacyOrbitCamera::new(camera_target),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(-4.0, 8.0, -6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(24.0, 24.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.055, 0.075, 0.11),
            perceptual_roughness: 0.92,
            ..default()
        })),
        Transform::IDENTITY,
    ));
    if matches!(
        config.case,
        PreviewCase::Rifle | PreviewCase::FusionSpawnAttack | PreviewCase::FusionSpawnDeath
    ) {
        let marker_mesh = meshes.add(Sphere::new(0.14).mesh().uv(20, 12));
        let source_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.75, 1.0),
            emissive: LinearRgba::rgb(0.1, 1.5, 4.0),
            ..default()
        });
        let target_material = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.25, 0.15),
            emissive: LinearRgba::rgb(3.0, 0.2, 0.05),
            ..default()
        });
        let source = if matches!(
            config.case,
            PreviewCase::FusionSpawnAttack | PreviewCase::FusionSpawnDeath
        ) {
            fusion_spawn_source()
        } else {
            rifle_source()
        };
        let target = if matches!(
            config.case,
            PreviewCase::FusionSpawnAttack | PreviewCase::FusionSpawnDeath
        ) {
            fusion_spawn_target()
        } else {
            rifle_target()
        };
        commands.spawn((
            Mesh3d(marker_mesh.clone()),
            MeshMaterial3d(source_material),
            Transform::from_translation(source),
        ));
        commands.spawn((
            Mesh3d(marker_mesh),
            MeshMaterial3d(target_material),
            Transform::from_translation(target),
        ));
    }
}

fn hold_debug_floor(mut actors: Query<&mut Transform, With<TutorialActor>>) {
    for mut transform in &mut actors {
        transform.translation.y = 0.0;
    }
}

#[allow(clippy::too_many_arguments)]
fn drive_preview(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<PreviewConfig>,
    mut state: ResMut<PreviewState>,
    registry: Res<TutorialActorRegistry>,
    scene_statuses: Query<&LegacyCharacterSceneStatus>,
    legacy_materials: Res<Assets<LegacyModelMaterial>>,
    legacy_surfaces: Query<(
        &Name,
        &GlobalTransform,
        &MeshMaterial3d<LegacyModelMaterial>,
    )>,
    named_transforms: Query<(&Name, &GlobalTransform)>,
    material_errors: Query<&LegacyMaterialMetadataError>,
    mut actors: ResMut<TutorialActorCommandQueue>,
    mut actor_events: ResMut<TutorialActorEventQueue>,
    mut effects: ResMut<TutorialEffectRuntime>,
    projectile_visual_readiness: Res<TutorialProjectileVisualReadiness>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);
    std::thread::sleep(Duration::from_millis(16));
    if let Some(error) = material_errors.iter().next() {
        eprintln!("material error: {}", error.0);
        exit.write(AppExit::error());
        return;
    }
    let now = time.elapsed_secs();
    if state.playback_started_at.is_none() {
        if config.case == PreviewCase::Rifle && !projectile_visual_readiness.ready {
            return;
        }
        if let Some(effect_id) = exact_mesh_effect(config.case) {
            if !state.preload_requested {
                effects.enqueue(TutorialEffectRuntimeCommand::Preload {
                    effect_id,
                    source_line: line!(),
                });
                state.preload_requested = true;
                return;
            }
            if !effects.is_native_preload_complete(effect_id) {
                return;
            }
        }
        let actor_ready = if let Some((_, clip)) = config.case.actor() {
            if registry.entity(ACTOR_ID).is_none()
                || !scene_statuses
                    .iter()
                    .any(|status| matches!(status, LegacyCharacterSceneStatus::Ready { .. }))
                || legacy_materials.is_empty()
            {
                false
            } else {
                let ready_at = state.actor_ready_at.get_or_insert(now);
                if now - *ready_at < 2.05 {
                    return;
                }
                if config.case == PreviewCase::FusionSpawnDeath {
                    actors.damage(ACTOR_ID, i32::MAX);
                } else {
                    actors.play_pose(ACTOR_ID, clip, false);
                }
                true
            }
        } else {
            true
        };
        if actor_ready {
            state.playback_started_at = Some(now);
        }
        return;
    }

    let playback_age = now - state.playback_started_at.unwrap();
    for event in actor_events.take_all() {
        if matches!(
            event,
            TutorialActorEvent::PlayerKillDeathPresentation { id: ACTOR_ID, .. }
        ) {
            let sampled_initial_velocity = tutorial_oni_velocity_samples_from_unity_unit_draws(
                false,
                &[0.08, 0.24, 0.41, 0.59, 0.76, 0.92],
            )
            .expect("fixed GPU audit draws satisfy the normal Oni contract");
            effects.enqueue(TutorialEffectRuntimeCommand::ProjectilePairSampled {
                types: [76, 77],
                source: fusion_spawn_source(),
                target: fusion_spawn_target(),
                oni: true,
                priority: 0,
                reverse: false,
                sampled_initial_velocity,
                source_line: 103,
            });
        }
    }
    let expected_effect = exact_mesh_effect(config.case);
    for record in effects.drain_records() {
        if matches!(
            record.command,
            TutorialEffectRuntimeCommand::Add { effect_id, .. }
                if Some(effect_id) == expected_effect
        ) {
            state.effect_started_at.get_or_insert(now);
        }
    }
    if state.effect_started_at.is_none()
        && (matches!(
            config.case,
            PreviewCase::FusionSpawnAttack | PreviewCase::Rifle
        ) || is_direct_effect(config.case))
        && playback_age >= config.case.animation_event_seconds()
    {
        enqueue_non_clip_case_effect(config.case, &mut effects);
        state.effect_started_at = Some(now);
        return;
    }

    let Some(effect_started_at) = state.effect_started_at else {
        return;
    };
    let effect_started_at = if config.case == PreviewCase::NpcSkill(766) {
        if state.effect_ready_at.is_none() {
            if !effects.named_native_presentation_ready("GPU audit ES766") {
                if now - effect_started_at < 2.0 {
                    return;
                }
                assert!(
                    env::var_os("FFONE_VFX_ALLOW_UNREADY_BASELINE").is_some(),
                    "eruption mesh never passed animation/material readiness; surface names: {:?}",
                    legacy_surfaces
                        .iter()
                        .map(|(name, _, _)| name.as_str())
                        .collect::<Vec<_>>()
                );
            }
            state.effect_ready_at = Some(now);
        }
        state.effect_ready_at.unwrap()
    } else {
        effect_started_at
    };
    if state.capture_index < config.outputs.len()
        && !state.capture_pending
        && now - effect_started_at >= config.capture_times[state.capture_index]
    {
        let index = state.capture_index;
        state.pose_samples.push(serde_json::json!({
            "capture": index, "age": now - effect_started_at,
            "ready": effects.named_native_presentation_ready("GPU audit ES766"),
            "surfaces": legacy_surfaces.iter().map(|(name, transform, handle)| serde_json::json!({
                "name": name.as_str(), "position": transform.translation().to_array(),
                "scale": transform.to_scale_rotation_translation().0.to_array(),
                "uv": legacy_materials.get(&handle.0).map(|m| m.uniform.uv_scale_offset.to_array()),
                "uvRotation": legacy_materials.get(&handle.0).map(|m| m.uniform.uv_pivot_rotation.z)
            })).collect::<Vec<_>>()
        }));
        let actor_effect_root_name = match config.case {
            PreviewCase::NumbuhOne | PreviewCase::Dexter | PreviewCase::NumbuhFive => {
                Some(format!("AnimationEvent actor {ACTOR_ID} ES734"))
            }
            PreviewCase::SamuraiJack => Some(format!("AnimationEvent actor {ACTOR_ID} ES751")),
            PreviewCase::FusionSpawnAttack
            | PreviewCase::FusionSpawnDeath
            | PreviewCase::FusionButtercupCorruptak
            | PreviewCase::FusionButtercupMegatak
            | PreviewCase::FusionBlooCorruptak
            | PreviewCase::FusionBlooMegatak
            | PreviewCase::FusionMacCorruptak
            | PreviewCase::FusionMacMegatak
            | PreviewCase::InfectionAura
            | PreviewCase::InfectionDamage
            | PreviewCase::InfectionProtection
            | PreviewCase::NanoSummon
            | PreviewCase::NpcSkill(_)
            | PreviewCase::Rifle => None,
        };
        if let Some(root_name) = actor_effect_root_name {
            let simultaneous_roots = named_transforms
                .iter()
                .filter(|(name, _)| name.as_str() == root_name.as_str())
                .count();
            println!(
                "capture={} currentEffect={root_name:?} simultaneous_roots={simultaneous_roots}",
                state.capture_index
            );
            if simultaneous_roots > 1 {
                eprintln!(
                    "AnimationEventHandler.currentEffect parity failed: {simultaneous_roots} simultaneous {root_name} roots"
                );
                exit.write(AppExit::error());
                return;
            }
        }
        for (name, transform) in &named_transforms {
            if matches!(name.as_str(), "Cylinder01" | "Cylinder02" | "Cylinder03")
                || name.as_str().starts_with("Tutorial projectile")
            {
                println!(
                    "capture={} node={:?} position={:?} scale={:?}",
                    state.capture_index,
                    name.as_str(),
                    transform.translation(),
                    transform.to_scale_rotation_translation().0,
                );
            }
        }
        for (name, transform, handle) in &legacy_surfaces {
            let Some(material) = legacy_materials.get(&handle.0) else {
                continue;
            };
            if !matches!(config.case, PreviewCase::Rifle)
                && !matches!(
                    config.case,
                    PreviewCase::FusionSpawnAttack | PreviewCase::FusionSpawnDeath
                )
                && !is_direct_effect(config.case)
                && !name.as_str().starts_with("Cylinder03.")
                && !name.as_str().starts_with("PolyMesh.")
                && !name.as_str().starts_with("Editable Poly.")
                && !name.as_str().starts_with("Editable Poly1.")
            {
                continue;
            }
            println!(
                "capture={} surface={:?} position={:?} scale={:?} base_color={:?} family={} bump={} effect_map={}",
                state.capture_index,
                name.as_str(),
                transform.translation(),
                transform.to_scale_rotation_translation().0,
                material.uniform.base_color,
                material.uniform.light_direction_family.w,
                material.bump_texture.is_some(),
                material.effect_map.is_some(),
            );
        }
        state.capture_pending = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }
    if state.capture_index >= config.outputs.len() {
        fs::write(
            config.outputs[0].with_extension("json"),
            serde_json::to_vec_pretty(&state.pose_samples).unwrap(),
        )
        .unwrap();
        let issues = effects.drain_issues().collect::<Vec<_>>();
        if issues.is_empty() {
            exit.write(AppExit::Success);
        } else {
            eprintln!("tutorial effect issues: {issues:#?}");
            exit.write(AppExit::error());
        }
    } else if state.frames >= TIMEOUT_FRAMES {
        eprintln!(
            "tutorial combat VFX audit timed out for {} after {} captures",
            config.case.slug(),
            state.capture_index
        );
        exit.write(AppExit::error());
    }
}

fn exact_mesh_effect(case: PreviewCase) -> Option<i32> {
    match case {
        PreviewCase::NumbuhOne | PreviewCase::Dexter | PreviewCase::NumbuhFive => Some(734),
        PreviewCase::SamuraiJack => Some(751),
        PreviewCase::FusionSpawnDeath => Some(372),
        PreviewCase::FusionButtercupCorruptak => Some(547),
        PreviewCase::FusionButtercupMegatak => Some(548),
        PreviewCase::FusionBlooCorruptak => Some(601),
        PreviewCase::FusionBlooMegatak => Some(602),
        PreviewCase::FusionMacCorruptak => Some(626),
        PreviewCase::FusionMacMegatak => Some(627),
        PreviewCase::InfectionAura => Some(376),
        PreviewCase::InfectionDamage => Some(385),
        PreviewCase::InfectionProtection => Some(804),
        PreviewCase::NanoSummon => Some(528),
        PreviewCase::NpcSkill(id) => Some(id),
        PreviewCase::FusionSpawnAttack | PreviewCase::Rifle => None,
    }
}

fn enqueue_non_clip_case_effect(case: PreviewCase, effects: &mut TutorialEffectRuntime) {
    match case {
        PreviewCase::FusionSpawnAttack => {
            effects.enqueue(TutorialEffectRuntimeCommand::Projectile {
                bullet_type: 106,
                source: fusion_spawn_source(),
                target: fusion_spawn_target(),
                target_exists: true,
                source_style: 2,
                target_style: -1,
                motion: TutorialProjectileMotion::BulletMove,
                source_line: line!(),
            })
        }
        PreviewCase::Rifle => effects.enqueue(TutorialEffectRuntimeCommand::Projectile {
            bullet_type: 13,
            source: rifle_source(),
            target: rifle_target(),
            target_exists: true,
            source_style: 0,
            target_style: 0,
            motion: TutorialProjectileMotion::BulletMove,
            source_line: line!(),
        }),
        direct if is_direct_effect(direct) => effects.enqueue(TutorialEffectRuntimeCommand::Add {
            effect_id: exact_mesh_effect(direct).expect("direct effect id"),
            placement: TutorialEffectPlacement::World {
                // NanoContainer parents the call effect to the Nano wrapper at
                // shoulder height. Keep the standalone audit off the floor so
                // all three additive emitters remain visible to the camera.
                position: if direct == PreviewCase::NanoSummon {
                    Vec3::Y * 1.12
                } else {
                    Vec3::ZERO
                },
                rotation: Quat::IDENTITY,
            },
            scale: if direct == PreviewCase::NpcSkill(766) {
                0.6
            } else {
                1.0
            },
            tracked: false,
            name: Some(format!(
                "GPU audit ES{}",
                exact_mesh_effect(direct).unwrap()
            )),
            destroy_after_seconds: None,
            source_line: line!(),
        }),
        PreviewCase::NumbuhOne
        | PreviewCase::Dexter
        | PreviewCase::NumbuhFive
        | PreviewCase::SamuraiJack
        | PreviewCase::FusionSpawnDeath
        | PreviewCase::FusionButtercupCorruptak
        | PreviewCase::FusionButtercupMegatak
        | PreviewCase::FusionBlooCorruptak
        | PreviewCase::FusionBlooMegatak
        | PreviewCase::FusionMacCorruptak
        | PreviewCase::FusionMacMegatak
        | PreviewCase::InfectionAura
        | PreviewCase::InfectionDamage
        | PreviewCase::InfectionProtection
        | PreviewCase::NanoSummon
        | PreviewCase::NpcSkill(_) => {
            unreachable!("actor cases are emitted by the production AnimationEvent bridge")
        }
    }
}

fn rifle_source() -> Vec3 {
    Vec3::new(-4.0, 1.7, 1.0)
}

fn rifle_target() -> Vec3 {
    Vec3::new(4.0, 1.7, 1.0)
}

fn fusion_spawn_source() -> Vec3 {
    Vec3::new(0.0, 0.9, 0.0)
}

fn fusion_spawn_target() -> Vec3 {
    Vec3::new(2.4, 0.8, 0.0)
}

fn save_screenshot(
    event: On<ScreenshotCaptured>,
    config: Res<PreviewConfig>,
    mut state: ResMut<PreviewState>,
) {
    let output = &config.outputs[state.capture_index];
    let image = event
        .image
        .clone()
        .try_into_dynamic()
        .expect("GPU screenshot must convert to a dynamic image");
    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("cannot create {}: {error}", parent.display()));
    }
    image
        .save(output)
        .unwrap_or_else(|error| panic!("cannot save {}: {error}", output.display()));
    println!("{}", absolute_display(output));
    state.capture_index += 1;
    state.capture_pending = false;
}

fn suffixed_png(path: &Path, suffix: &str) -> PathBuf {
    let stem = path.file_stem().and_then(|value| value.to_str()).unwrap();
    path.with_file_name(format!("{stem}-{suffix}.png"))
}

fn absolute_display(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}
