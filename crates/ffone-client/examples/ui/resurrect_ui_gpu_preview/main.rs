//! Deterministic 1264x681 GPU acceptance frames for clean-Retrobution
//! `ResurrectMode`.
//!
//! `self` renders GO plus Phoenix Self. `group-item` deliberately supplies
//! both Phoenix capability bits to prove the clean Group-before-Self priority,
//! then renders USE ITEM last in the overlapping Phoenix rectangle.

use std::{
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use bevy::{
    asset::{AssetPlugin, LoadState},
    ecs::system::SystemParam,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    text::{LineHeight, TextLayoutInfo},
    window::{PresentMode, WindowResolution},
};
use ffone_client::{
    localization::{Language, Localization, LocalizationPlugin, LocalizedText},
    resurrect_ui::{
        RESURRECT_BODY_FONT_PATH, RESURRECT_BUTTON_FONT_PATH, RESURRECT_COUNTDOWN_LOCALIZATION_KEY,
        RESURRECT_GO_BUTTON_RECT, RESURRECT_GO_LABEL, RESURRECT_GO_LOCALIZATION_KEY,
        RESURRECT_GRIM_RECT, RESURRECT_PHOENIX_BUTTON_RECT, RESURRECT_PHOENIX_SKILL_BACK_RECT,
        RESURRECT_PHOENIX_SKILL_RECT, RESURRECT_QUESTION, RESURRECT_QUESTION_LOCALIZATION_KEY,
        RESURRECT_REVIVE_LABEL, RESURRECT_REVIVE_LOCALIZATION_KEY, RESURRECT_TEXTURE_CONTRACTS,
        RESURRECT_TIMEOUT_SECONDS, RESURRECT_TITLE, RESURRECT_TITLE_LOCALIZATION_KEY,
        RESURRECT_UI_Z_INDEX, RESURRECT_USE_ITEM_LABEL, RESURRECT_USE_ITEM_LOCALIZATION_KEY,
        ResurrectChoice, ResurrectFontRole, ResurrectInputBoundary, ResurrectTextStyle,
        ResurrectTextureRole, ResurrectUiContext, ResurrectUiModel, ResurrectUiOutbox,
        ResurrectUiPlugin, ResurrectUiRect, ResurrectUiSet, clean_resurrect_ui_scale,
        resurrect_ui_layout,
    },
};

fn main() -> AppExit {
    let (mode, language_slug, output) =
        parse_preview_args(std::env::args_os().skip(1)).unwrap_or_else(|error| {
            eprintln!("{error}");
            eprintln!(
                "usage: resurrect_ui_gpu_preview [self|group-item] [OUTPUT.png] [--language en|ru]"
            );
            std::process::exit(2);
        });
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game")
        .canonicalize()
        .expect("assets/game must exist");
    let (localization, language) = Localization::open(&asset_root, &language_slug)
        .expect("open production localization bundles");

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.08, 0.17, 0.25)))
        .insert_resource(mode)
        .insert_resource(PreviewOutput(output))
        .insert_resource(PreviewLanguage(language_slug))
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
                        title: format!(
                            "FFOne Retrobution Resurrect {} acceptance",
                            mode.cli_name()
                        ),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((ResurrectUiPlugin, LocalizationPlugin))
        .add_systems(Startup, setup_preview)
        .add_systems(
            Update,
            force_preview_hover
                .after(ResurrectUiSet::Bind)
                .before(ResurrectUiSet::Visuals),
        )
        .add_systems(Update, drive_capture.after(ResurrectUiSet::Visuals))
        .run()
}

#[cfg(test)]
mod tests;

mod layout;
mod input;
mod constants;
mod interaction;
mod state;
mod types;
mod localization_preview_language;
mod operations;
mod view_spawn_world_backdrop;
mod validation;
mod output;

use layout::{CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT, node_matches_rect};
use input::{WARMUP_FRAMES_AFTER_LOAD, parse_preview_args};
use constants::{
    GPU_UPLOAD_GRACE, CAPTURE_TIMEOUT, MIN_DIALOG_VISIBLE_PIXELS, MIN_CYAN_PIXELS,
    MIN_RED_COUNTDOWN_PIXELS
};
use interaction::{EXPECTED_INPUT_BOUNDARY, force_preview_hover, button_contract_is_exact};
use state::{PreviewMode, PreviewState};
use types::{PreviewOutput, PreviewImage, PreviewAssets, CaptureQueries};
use localization_preview_language::PreviewLanguage;
use operations::{setup_preview, drive_capture, phoenix_visual_contract_is_exact};
use view_spawn_world_backdrop::spawn_world_backdrop;
use validation::audit_text_contract;
use output::save_screenshot;
