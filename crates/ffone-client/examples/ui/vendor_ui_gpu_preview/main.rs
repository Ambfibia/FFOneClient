//! Deterministic 1264x681 GPU acceptance frame for clean-Retrobution
//! `VendorMode` on the Buy tab.
//!
//! The harness injects complete authoritative vendor/recent/inventory
//! snapshots. It owns no production network encoding and emits no speculative
//! inventory or currency change. Usage:
//! `vendor_ui_gpu_preview [LANG] [OUTPUT.png]`, where LANG is `en` or `ru`.

use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use bevy::{
    asset::{AssetPlugin, LoadState},
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    text::{LineHeight, TextLayoutInfo},
    window::{PresentMode, WindowResolution},
};
use ffone_client::{
    inventory_runtime::InventoryRuntime0104,
    localization::{Localization, LocalizationPlugin, LocalizationSet, LocalizedText},
    vendor_ui::{
        VENDOR_OPEN_SECONDS, VENDOR_SERVICE_FONT_PATH, VENDOR_UI_DEFAULT_IMAGE_PATHS,
        VendorCatalogEntry0104, VendorCloseGate0104, VendorEquipEligibility0104, VendorIconRef,
        VendorItemCatalog0104, VendorItemMetadata0104, VendorLifecyclePhase, VendorModalState,
        VendorModeProjection0104, VendorRecentBuyEntry0104, VendorStaticAssetReadiness,
        VendorUiAssetStatus, VendorUiElement, VendorUiPlugin, VendorUiRoot, VendorUiSet,
        VendorUiState, VendorUiTextStyle,
    },
};
use ffone_protocol::{ItemBase0104, PcLoadData0104};

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
        &ffone_client::assets::AssetLocator::open(&asset_root).unwrap(),
    )
    .unwrap();

    App::new()
        .insert_resource(if env::var_os("FFONE_SERVICE_CONTROL").is_some() {
            bevy::time::TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0))
        } else {
            bevy::time::TimeUpdateStrategy::Automatic
        })
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(PreviewOutput(cli.output))
        .insert_resource(PreviewLanguage(cli.language))
        .insert_resource(PreviewState::default())
        .init_resource::<TryOnChecked>()
        .insert_resource(localization)
        .insert_resource(content)
        .insert_resource(TryOnFixture(
            ffone_client::character_creation_data::CharacterCreationData::open(&asset_root)
                .unwrap(),
        ))
        .insert_resource(
            ffone_client::player_shared_rig::NativePlayerRigCatalog::open(&asset_root).unwrap(),
        )
        .insert_resource(language)
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne Retrobution VendorMode acceptance".into(),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((
            VendorUiPlugin,
            LocalizationPlugin,
            ffone_client::legacy_model_material::LegacyModelMaterialPlugin,
            ffone_client::player_shared_rig::NativePlayerSharedRigPlugin,
            ffone_client::player_preview::NativePlayerPreviewPlugin,
        ))
        .add_systems(
            Update,
            drive_try_on_preview
                .after(VendorUiSet::Interaction)
                .before(ffone_client::player_preview::NativePlayerPreviewSet::Rebuild),
        )
        .add_systems(
            PreUpdate,
            exercise_try_on_pointer
                .after(bevy::input::InputSystems)
                .before(bevy::ui::UiSystems::Focus),
        )
        .add_systems(
            PreUpdate,
            exercise_chest_pointer
                .after(bevy::input::InputSystems)
                .before(bevy::ui::UiSystems::Focus),
        )
        .add_systems(
            PreUpdate,
            exercise_scroll_pointer
                .after(bevy::input::InputSystems)
                .before(bevy::ui::UiSystems::Focus),
        )
        .add_systems(
            PostUpdate,
            |mouse: Res<ButtonInput<MouseButton>>,
             q: Query<(&Interaction, Option<&Name>, Option<&VendorUiElement>)>| {
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
            open_popup_preview
                .after(VendorUiSet::Lifecycle)
                .before(VendorUiSet::Interaction),
        )
        .add_systems(
            Update,
            drive_capture
                .after(VendorUiSet::Bind)
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
mod interaction;
mod assets;
mod output;
mod validation;

use layout::{CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT};
use input::{WARMUP_FRAMES_AFTER_LOAD, parse_cli};
use constants::{
    GPU_UPLOAD_GRACE, CAPTURE_TIMEOUT, MIN_VISIBLE_PIXELS, MIN_MISSING_CHECKER_PIXELS,
    MAX_MISSING_CHECKER_PIXELS, EXPECTED_ELEMENT_COUNT, EXPECTED_TEXT_COUNT,
    EXPECTED_VISIBLE_TEXT_COUNT, ICON_GENERAL_00, ICON_GENERAL_01, ICON_WEAPON_01,
    ICON_WEAPON_02, ICON_COSMETIC_00, ICON_COSMETIC_01, ICON_COSMETIC_02, ICON_COSMETIC_03,
    ICON_COSMETIC_04, ICON_COSMETIC_05, ICON_VEHICLE_00, PREVIEW_ICON_PATHS
};
use types::{
    PreviewOutput, PreviewAssets, PreviewCli, TryOnChecked, TryOnFixture, PreviewEligibility
};
use localization_preview_language::PreviewLanguage;
use state::PreviewState;
use operations::{
    default_output, open_popup_preview, drive_try_on_preview, preview_projection,
    vendor_entry, item, setup_preview, drive_capture, element_has_size, absolute_display
};
use interaction::{exercise_try_on_pointer, exercise_chest_pointer, exercise_scroll_pointer};
use assets::PreviewCatalog;
use output::{write_item, save_screenshot};
use validation::audit_vendor_text;
