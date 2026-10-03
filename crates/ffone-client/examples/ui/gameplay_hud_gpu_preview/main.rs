//! Deterministic GPU acceptance frame for the Retrobution gameplay HUD.

use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use bevy::{
    asset::AssetPlugin,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    window::{PresentMode, WindowResolution},
};
use ffone_client::gameplay_ui::NpcServiceKind;
use ffone_client::gameplay_ui::{
    ChatChannel, ChatLineUi, CurrentObjectiveProgressUi, CurrentObjectiveUi, GameplayHud,
    GameplayMenuTransition, GameplayUiModel, GameplayUiPlugin, MinimapMarkerIcon,
    MinimapMarkerSample, NanoSlotUi, NanoWheelTransientUi, NpcBarkerBubbleRuntime,
    PlayerFreeChatBubbleRuntime, QuickChatMenuMode,
};
use ffone_client::{
    assets::AssetLocator,
    avatar_action::{
        LegacyAttackTarget, LegacyAvatarActionContext, LegacyAvatarActionState,
        LegacyFocusedTarget, LegacyTargetKind,
    },
    buddy_ui::{BuddyChatWindowStyle, BuddyUiModel, BuddyUiPlugin},
    character_creation_data::CharacterCreationData,
    character_creation_ui::CharacterAppearance,
    character_selection_portraits::{
        CharacterSelectionPortraitStatus, CharacterSelectionPortraitsModel,
        GameplayPlayerPortraitModel, NativeCharacterSelectionPortraitsPlugin,
    },
    character_selection_ui::CharacterSelectionUiModel,
    entity_lifecycle::NetworkNpcAppearance0104,
    gameplay_nano_portraits::{GameplayNanoPortraitCatalog, GameplayNanoPortraitPlugin},
    legacy_model_material::LegacyModelMaterialPlugin,
    localization::{
        Language, Localization, LocalizationPlugin, LocalizedText, localized_tutorial_instruction,
        localized_tutorial_literal, localized_tutorial_scene_text,
    },
    mission_ui::{
        JournalListTab, JournalOtherUi, MissionJournalUi, MissionUiEntry, MissionUiModel,
        MissionUiPlugin, MissionUiRewards, NpcInteractionUi, NpcServiceUiEntry,
    },
    movement::LegacyOrbitCamera,
    nanocom_message_ui::{NanocomMessageUiModel, NanocomMessageUiPlugin},
    player_preview::NativePlayerLook,
    player_shared_rig::{NativePlayerRigCatalog, NativePlayerSharedRigPlugin},
    tutorial_actors::TutorialActor,
    tutorial_mission_content::TutorialMissionContent,
    tutorial_nanocom_message::{TutorialNanocomMessagePlugin, TutorialNanocomMessageQueue},
    tutorial_overlay_ui::{
        TutorialArrowCue, TutorialArrowDirection, TutorialOverlayUiModel, TutorialOverlayUiPlugin,
    },
};
use ffone_protocol::NpcAppearance0104;

fn main() {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/ffone-gameplay-hud-current.png"));
    if output.extension().and_then(|value| value.to_str()) != Some("png") {
        eprintln!("usage: gameplay_hud_gpu_preview [OUTPUT.png]");
        std::process::exit(2);
    }
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game")
        .canonicalize()
        .expect("assets/game must exist");
    let data =
        CharacterCreationData::open(&asset_root).expect("open native character-creation data");
    let catalog =
        NativePlayerRigCatalog::open(&asset_root).expect("open native shared player-rig catalog");
    let project_assets = AssetLocator::open(&asset_root).expect("open project assets");
    let nano_portrait_catalog = GameplayNanoPortraitCatalog::open(&project_assets)
        .expect("resolve exact Nano portrait catalog");
    let mission_content = TutorialMissionContent::open(&project_assets)
        .expect("resolve exact tutorial mission content");
    let requested_language = env::var("FFONE_LANGUAGE").unwrap_or_else(|_| "en".to_owned());
    let (localization, language) = Localization::open(&asset_root, &requested_language)
        .expect("open native localization bundles");
    let player_look = data
        .resolve_creator(1, 1, "Test", "Ser", &CharacterAppearance::default())
        .expect("resolve exact preview player")
        .look;

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.055, 0.10, 0.13)))
        .insert_resource(PreviewOutput(output))
        .insert_resource(PreviewPlayerLook(player_look))
        .insert_resource(PreviewState::default())
        .insert_resource(catalog)
        .insert_resource(nano_portrait_catalog)
        .insert_resource(mission_content)
        .insert_resource(localization)
        .insert_resource(language)
        .insert_resource(CharacterSelectionUiModel::default())
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne Retrobution HUD acceptance".into(),
                        resolution: WindowResolution::new(1264, 681),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((
            LegacyModelMaterialPlugin,
            GameplayUiPlugin,
            BuddyUiPlugin,
            TutorialOverlayUiPlugin,
            NativePlayerSharedRigPlugin,
            NativeCharacterSelectionPortraitsPlugin,
            GameplayNanoPortraitPlugin,
            TutorialNanocomMessagePlugin,
            NanocomMessageUiPlugin,
            MissionUiPlugin,
            LocalizationPlugin,
        ))
        .add_systems(Startup, setup_preview)
        .add_systems(
            Update,
            drive_reward_preview.before(ffone_client::gameplay_ui::GameplayUiSet::Rewards),
        )
        .add_systems(Update, (sync_preview_warp_hud, drive_capture).chain())
        .run();
}

mod constants;
mod types;
mod state;
mod systems;
mod operations_setup_preview;
mod operations_drive_capture;
mod interaction;
mod output;

use constants::{PORTRAIT_WARMUP, CAPTURE_TIMEOUT};
use types::{PreviewOutput, PreviewPlayerLook};
use state::PreviewState;
use systems::sync_preview_warp_hud;
use operations_setup_preview::{drive_reward_preview, setup_preview};
use operations_drive_capture::drive_capture;
use interaction::ScrollPointerProbe;
use output::save_screenshot;
