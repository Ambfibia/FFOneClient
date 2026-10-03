//! GPU/runtime proof for the complete tutorial player animation adapter.
//!
//! Unlike `shared_player_rig_gpu_preview`, this goes through the real tutorial
//! spawn and readiness path, including the legacy height/shape scale samples
//! and continuously active additive body-shape clips.
//! `FFONE_PLAYER_PREVIEW_YAW` selects the camera angle in degrees.
//! `FFONE_PLAYER_PREVIEW_DISMOUNT=1` captures both mounted and returned poses;
//! `weapon-swap` captures item 328 -> item 43 -> unarmed at the same camera angle.

use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::Arc,
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
    avatar_action::{
        LegacyAvatarActionContext, LegacyAvatarActionPlugin, LegacyAvatarActionSet,
        LegacyAvatarActionState, LegacyAvatarClipBindings, LegacyAvatarPresentationContext,
        LegacyAvatarTargetFeed, LegacyAvatarTraversalPresentation, LegacyVisualCompletionQueue,
        LegacyVisualRequestQueue,
    },
    character_creation_data::{CharacterCreationData, CharacterCreationDataResource},
    character_creation_ui::{CharacterAppearance, CharacterGender},
    legacy_model_material::{
        LegacyMaterialMetadataError, LegacyModelMaterial, LegacyModelMaterialPlugin,
    },
    movement::LegacyPlayerController,
    network::CharacterSummary,
    player_shared_rig::{
        NativePlayerRigAssetCache, NativePlayerRigBones, NativePlayerRigCatalog,
        NativePlayerSharedRigPlugin,
    },
    tutorial_player_presentation::{
        PlayerWeaponAnimationCatalog, TutorialPlayerClip, TutorialPlayerPresentationCommandQueue,
        tutorial_player_gender_from_protocol,
    },
    tutorial_player_rig_runtime::{
        TutorialPlayerAnimationApplied, TutorialPlayerRigRuntimePlugin,
        TutorialPlayerWeaponAttachment, TutorialSelectedPlayerRigStatus,
        TutorialSkywayPresentation, spawn_tutorial_selected_player_rig,
    },
};
use ffone_protocol::{CHARACTER_EQUIP_SLOT_COUNT_0104, CharacterStyle0104, EquippedItem0104};
use ffone_runtime_contracts::PlayerRigGender;

fn main() {
    let mut args = std::env::args_os().skip(1);
    let asset_root = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("assets/game"));
    let output = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/tutorial-player-rig-preview.png"));
    let height = parse_selector(args.next(), "height", 2, 4);
    let body = parse_selector(args.next(), "body", 1, 2);
    let direction = parse_selector(args.next(), "direction", 0, 8);
    let case = match args.next().and_then(|value| value.into_string().ok()) {
        None => PreviewCase::Locomotion,
        Some(value) => match value.as_str() {
            "locomotion" => PreviewCase::Locomotion,
            "standup" => PreviewCase::Standup,
            "startup-landing" => PreviewCase::StartupLanding,
            "attack-run" => PreviewCase::AttackRun,
            "attack-fall" => PreviewCase::AttackFall,
            "skyway" => PreviewCase::Skyway,
            "zipline" => PreviewCase::Zipline,
            "vehicle-board" => PreviewCase::VehicleBoard,
            "vehicle-scooter" => PreviewCase::VehicleScooter,
            "weapon-swap" => PreviewCase::WeaponSwap,
            "dance" => PreviewCase::Dance,
            "beach" => PreviewCase::Beach,
            _ => panic!("unknown preview case {value:?}"),
        },
    };
    let selected_gender = match args
        .next()
        .and_then(|value| value.into_string().ok())
        .as_deref()
    {
        None | Some("male") => CharacterGender::Boy,
        Some("female") => CharacterGender::Girl,
        Some(value) => panic!("unknown preview gender {value:?}"),
    };
    if args.next().is_some() {
        eprintln!(
            "usage: tutorial_player_rig_gpu_preview [ASSET_ROOT] [OUTPUT.png] \
             [HEIGHT 0..4] [BODY 0..2] [DIRECTION 0..8] \
             [locomotion|standup|startup-landing|attack-run|attack-fall|skyway|zipline|\
              vehicle-board|vehicle-scooter|weapon-swap|dance|beach] [male|female]"
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
    let character_data = Arc::new(
        CharacterCreationData::open(&asset_root)
            .unwrap_or_else(|error| panic!("cannot open character-creation data: {error}")),
    );
    if output.exists() {
        fs::remove_file(&output)
            .unwrap_or_else(|error| panic!("cannot replace {}: {error}", output.display()));
    }

    App::new()
        .insert_resource(
            ffone_client::tutorial_mission_content::TutorialMissionContent::open(
                &ffone_client::assets::AssetLocator::open(&asset_root).unwrap(),
            )
            .unwrap(),
        )
        .insert_resource(ffone_client::assets::AssetLocator::open(&asset_root).unwrap())
        .insert_resource(
            PlayerWeaponAnimationCatalog::open(
                &ffone_client::assets::AssetLocator::open(&asset_root).unwrap(),
            )
            .unwrap(),
        )
        .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.055)))
        .insert_resource(PreviewState::default())
        .insert_resource(catalog)
        .insert_resource(CharacterCreationDataResource(character_data.clone()))
        .init_resource::<LegacyVisualRequestQueue>()
        .init_resource::<LegacyVisualCompletionQueue>()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne tutorial player rig audit".into(),
                        resolution: WindowResolution::new(768, 768),
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((
            LegacyModelMaterialPlugin,
            NativePlayerSharedRigPlugin,
            LegacyAvatarActionPlugin,
            TutorialPlayerRigRuntimePlugin,
            ffone_client::tutorial_effects_runtime::TutorialEffectsRuntimePlugin,
        ))
        .add_systems(
            Startup,
            move |mut commands: Commands,
                  assets: Res<AssetServer>,
                  mut asset_cache: ResMut<NativePlayerRigAssetCache>,
                  catalog: Res<NativePlayerRigCatalog>,
                  weapon_animations: Res<PlayerWeaponAnimationCatalog>| {
                setup(
                    &mut commands,
                    &assets,
                    &mut asset_cache,
                    &catalog,
                    &weapon_animations,
                    &character_data,
                    output.clone(),
                    height,
                    body,
                    direction,
                    case,
                    selected_gender,
                );
            },
        )
        .add_systems(
            Update,
            drive_attack_input.before(LegacyAvatarActionSet::ReadInput),
        )
        .add_systems(Update, drive_vehicle_trail_probe)
        .add_systems(Update, drive_capture)
        .add_systems(Update, weapon_swap::drive_weapon_swap)
        .add_systems(
            PostUpdate,
            operations_setup::disable_preview_outlines
                .before(bevy::camera::visibility::VisibilitySystems::VisibilityPropagate),
        )
        .run();
}

mod codec;
mod commands;
mod constants;
mod input;
mod operations_drive_capture;
mod operations_setup;
mod output;
mod state;
mod types;
mod weapon_swap;

use codec::MAX_END_EVENT_FRAME_OVERSHOOT_SECONDS;
use commands::PRIMARY_STANDUP_END_EVENT_SECONDS;
use constants::{
    INJECT_PAUSE_AFTER_RUN_FRAMES, INJECT_STOP_AFTER_RUN_FRAMES, MAX_FROZEN_RUN_FRAMES,
    MAX_MISSING_BASE_FRAMES, READY_WARMUP_FRAMES, TIMEOUT_FRAMES,
};
use input::parse_selector;
use operations_drive_capture::{absolute_display, drive_capture};
use operations_setup::{drive_attack_input, drive_vehicle_trail_probe, setup};
use output::{save_dismount_screenshot, save_screenshot};
use state::PreviewState;
use types::{PreviewCase, PreviewConfig, PreviewTravelResources};
