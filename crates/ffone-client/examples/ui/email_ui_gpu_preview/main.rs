//! Deterministic 1264x681 GPU acceptance frame for clean-Retrobution
//! `eGameMode.Email` on the player-mail tab.
//!
//! The harness imports the module by path so its native UI and typed packet
//! boundary can be reviewed independently of production-shell lifecycle
//! wiring. It injects server-authoritative page/read replies and never mutates
//! inventory, attached items, or Taros speculatively.

#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

pub use ffone_client::ui_startup;
use std::{
    collections::{BTreeMap, BTreeSet},
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
use serde::Deserialize;

#[allow(dead_code)]
mod email_runtime {
    use std::{error::Error, fmt};

    use crate::email_ui::{EmailReply, EmailRequest};

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum EmailRuntimeDelivery0104 {
        Correlated(EmailReply),
        UnsolicitedNewEmail(EmailReply),
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct EmailWirePacket0104 {
        pub packet_id: u32,
        pub body: Vec<u8>,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct EmailRuntimeProbeError;

    impl fmt::Display for EmailRuntimeProbeError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("standalone Email runtime probe error")
        }
    }

    impl Error for EmailRuntimeProbeError {}

    #[derive(Clone, Debug, Default, Eq, PartialEq)]
    pub struct EmailTransportRuntime0104 {
        pending: bool,
    }

    impl EmailTransportRuntime0104 {
        pub fn pending(&self) -> Option<()> {
            self.pending.then_some(())
        }

        pub fn begin(
            &mut self,
            _request: &EmailRequest,
        ) -> Result<EmailWirePacket0104, EmailRuntimeProbeError> {
            self.pending = true;
            Ok(EmailWirePacket0104 {
                packet_id: 0,
                body: Vec::new(),
            })
        }

        pub fn cancel_pending(&mut self) -> Option<()> {
            std::mem::take(&mut self.pending).then_some(())
        }

        pub fn accept(
            &mut self,
            _packet_id: u32,
            _body: &[u8],
        ) -> Result<Option<EmailRuntimeDelivery0104>, EmailRuntimeProbeError> {
            Ok(None)
        }
    }
}
pub use ffone_client::localization;

#[allow(dead_code)]
#[path = "../../../src/ui/email/mod.rs"]
mod email_ui;

use localization::{Localization, LocalizationPlugin, LocalizationSet, LocalizedText};

use email_ui::{
    EMAIL_UI_BODY_FONT_PATH, EMAIL_UI_CHALET_SMALL_FONT_SIZE, EMAIL_UI_CHALET_SMALL_LINE_HEIGHT,
    EMAIL_UI_FONT_PATH, EMAIL_UI_IMAGE_PATHS, EMAIL_UI_JEFFE_06_FONT_SIZE,
    EMAIL_UI_JEFFE_06_LINE_HEIGHT, EMAIL_UI_JEFFE_12_FONT_SIZE, EMAIL_UI_JEFFE_12_LINE_HEIGHT,
    EMAIL_UI_JEFFE_14_FONT_SIZE, EMAIL_UI_JEFFE_14_LINE_HEIGHT, EMAIL_UI_JEFFE_16_FONT_SIZE,
    EMAIL_UI_JEFFE_16_LINE_HEIGHT, EmailFolder, EmailGuideMessage, EmailInputBoundary,
    EmailInventorySlotView, EmailOutgoingItem, EmailPopup, EmailReadMessage, EmailReply,
    EmailScreen, EmailSummary, EmailSystemTime, EmailTransportOutbox, EmailUiAudioOutbox,
    EmailUiButton, EmailUiButtonKind, EmailUiComposePanel, EmailUiListPanel, EmailUiModel,
    EmailUiOutbox, EmailUiPlugin, EmailUiRightPanel, EmailUiRoot, EmailUiSet, EmailUiTextElement,
    EmailUiTextRole, EmailUiTextStyle, EmailWireItem, apply_email_reply, begin_email_compose,
    clean_email_ui_scale, email_ui_layout, open_email_ui, switch_email_folder,
};

fn main() {
    let cli = parse_cli(std::env::args_os().skip(1)).unwrap_or_else(|usage| {
        eprintln!("{usage}");
        std::process::exit(2);
    });
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game")
        .canonicalize()
        .expect("assets/game must exist");
    let (localization, language) = Localization::open(&asset_root, &cli.language)
        .expect("open and validate production EN/RU localization bundles");

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.02, 0.07)))
        .insert_resource(localization)
        .insert_resource(language)
        .insert_resource(cli)
        .insert_resource(PreviewState::default())
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne Retrobution EmailMode acceptance".into(),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((EmailUiPlugin, LocalizationPlugin))
        .add_systems(Startup, setup_preview)
        .add_systems(
            Update,
            force_preview_hover
                .after(EmailUiSet::Interaction)
                .before(EmailUiSet::Visuals),
        )
        .add_systems(
            Update,
            drive_capture
                .after(EmailUiSet::Visuals)
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
mod state;
mod operations;
mod view_spawn_world_backdrop;
mod interaction;
mod validation;
mod output;

use layout::{CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT};
use input::{WARMUP_FRAMES_AFTER_LOAD, parse_cli};
use constants::{
    GPU_UPLOAD_GRACE, CAPTURE_TIMEOUT, MIN_WINDOW_VISIBLE_PIXELS, MIN_CYAN_PIXELS,
    ICON_GENERAL, ICON_WEAPON, ICON_COSMETIC, PREVIEW_ICON_PATHS
};
use types::{PreviewScene, PreviewCli, PreviewAssets};
use state::{PreviewState, inventory_item};
use operations::{
    setup_preview, preview_buddies, setup_player_scene, setup_compose_scene,
    preview_summaries, drive_capture, expected_text_metrics
};
use view_spawn_world_backdrop::spawn_world_backdrop;
use interaction::force_preview_hover;
use validation::audit_visible_text;
use output::save_screenshot;
