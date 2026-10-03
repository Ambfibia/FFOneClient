//! Deterministic 1264x681 GPU acceptance frame for clean-Retrobution
//! `BankMode`.
//!
//! The harness injects a complete authoritative bank-open reply plus current
//! 9+50 inventory authority. It exercises no production network wiring and
//! emits no speculative item move. Usage:
//! `bank_ui_gpu_preview [LANG] [OUTPUT.png]`, where LANG is `en` or `ru`.

use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use bevy::{
    asset::{AssetPlugin, LoadState},
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    text::{ComputedTextBlock, LineHeight, TextLayoutInfo},
    window::{PresentMode, WindowResolution},
};
use ffone_client::{
    bank_ui::{
        BANK_BUTTON_FONT_SIZE, BANK_BUTTON_LINE_HEIGHT, BANK_BUTTON_PADDING_BOTTOM,
        BANK_BUTTON_PADDING_TOP, BANK_EQUIP_FONT_SIZE, BANK_EQUIP_LINE_HEIGHT,
        BANK_LABEL_FONT_SIZE, BANK_LABEL_LINE_HEIGHT, BANK_OPEN_SECONDS,
        BANK_TEXT_REPLACEMENT_Y_OFFSET, BANK_UI_DEFAULT_IMAGE_PATHS, BankEquipEligibility,
        BankLifecyclePhase, BankModalState, BankModeProjection0104, BankPcStuffAuthority0104,
        BankUiElement, BankUiPlugin, BankUiRoot, BankUiSet, BankUiState, BankUiTextStyle,
    },
    inventory_runtime::InventoryRuntime0104,
    localization::{Localization, LocalizationPlugin, LocalizationSet, LocalizedText},
    user_equip_ui::{
        UserEquipCatalogKind, UserEquipCatalogQuery, UserEquipIconRef, UserEquipItemCatalog,
    },
};
use ffone_protocol::{BANK_SLOT_COUNT_0104, ItemBase0104, PcBankOpenSuccess0104, PcLoadData0104};

fn main() {
    let cli = match parse_cli(std::env::args_os().skip(1)) {
        Ok(cli) => cli,
        Err(usage) => {
            eprintln!("{usage}");
            std::process::exit(2);
        }
    };
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game")
        .canonicalize()
        .expect("assets/game must exist");
    let (localization, language) = Localization::open(&asset_root, &cli.language)
        .expect("open production localization bundles");

    let content = ffone_client::tutorial_mission_content::TutorialMissionContent::open(
        &ffone_client::assets::AssetLocator::open(&asset_root).expect("production asset catalog"),
    )
    .expect("production item details");
    App::new()
        .insert_resource(if env::var_os("FFONE_SERVICE_CONTROL").is_some() {
            bevy::time::TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0))
        } else {
            bevy::time::TimeUpdateStrategy::Automatic
        })
        .insert_resource(content)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(PreviewOutput(cli.output))
        .insert_resource(PreviewLanguage(cli.language))
        .insert_resource(PreviewState::default())
        .insert_resource(localization)
        .insert_resource(language)
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne Retrobution BankMode acceptance".into(),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((BankUiPlugin, LocalizationPlugin))
        .add_plugins((
            ffone_client::game_guide_ui::GameGuideUiPlugin,
            ffone_client::shared_input_ui::SharedInputUiPlugin,
            ffone_client::system_message_ui::SystemMessageUiPlugin,
        ))
        .add_systems(
            PreUpdate,
            (exercise_bank_search, exercise_service_controls)
                .after(bevy::input::InputSystems)
                .before(bevy::ui::UiSystems::Focus),
        )
        .add_systems(
            PostUpdate,
            |mouse: Res<ButtonInput<MouseButton>>,
             q: Query<(&Interaction, Option<&Name>, Option<&BankUiElement>)>| {
                if env::var_os("FFONE_SERVICE_CONTROL").is_some()
                    && mouse.just_pressed(MouseButton::Left)
                {
                    for (i, n, e) in &q {
                        if *i != Interaction::None {
                            println!("Pointer hit: {i:?} {n:?} {e:?}");
                        }
                    }
                }
            },
        )
        .add_systems(Startup, setup_preview)
        .add_systems(
            Update,
            sync_preview_system_modal.before(BankUiSet::Interaction),
        )
        .add_systems(
            PreUpdate,
            exercise_pointer
                .after(bevy::input::InputSystems)
                .before(bevy::ui::UiSystems::Focus),
        )
        .add_systems(
            Update,
            drive_capture
                .after(BankUiSet::Bind)
                .after(LocalizationSet::Apply),
        )
        .run();
}

#[cfg(test)]
mod tests;

mod layout;
mod input;
mod constants;
mod types;
mod localization_preview_language;
mod state;
mod operations;
mod systems;
mod assets;
mod interaction;
mod output;
mod validation;

use layout::{CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT};
use input::{WARMUP_FRAMES_AFTER_LOAD, parse_cli};
use constants::{
    GPU_UPLOAD_GRACE, CAPTURE_TIMEOUT, MIN_VISIBLE_PIXELS, MIN_MISSING_CHECKER_PIXELS,
    EXPECTED_ELEMENT_COUNT, EXPECTED_TEXT_COUNT, EXPECTED_VISIBLE_TEXT_COUNT, ICON_GENERAL_00,
    ICON_GENERAL_01, ICON_WEAPON_01, ICON_WEAPON_02, ICON_COSMETIC_00, ICON_COSMETIC_01,
    ICON_COSMETIC_02, ICON_COSMETIC_03, ICON_COSMETIC_04, ICON_COSMETIC_05, ICON_VEHICLE_00,
    PREVIEW_ICON_PATHS
};
use types::{PreviewOutput, PreviewAssets, PreviewCli, PreviewEligibility};
use localization_preview_language::PreviewLanguage;
use state::PreviewState;
use operations::{
    default_output, preview_projection, item, setup_preview, drive_capture, absolute_display,
    exercise_service_controls, exercise_bank_search
};
use systems::sync_preview_system_modal;
use assets::PreviewCatalog;
use interaction::exercise_pointer;
use output::{write_item, save_screenshot};
use validation::audit_bank_text;
