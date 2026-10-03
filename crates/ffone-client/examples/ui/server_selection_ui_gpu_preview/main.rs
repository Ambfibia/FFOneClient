//! Deterministic 1264x681 GPU acceptance frame for clean-Retrobution
//! `eGameMode::ServerSelection`.
//!
//! The standalone screen is opened with Server 1 expanded, all four status
//! colours represented, Channel 7 selected, and a small deterministic scroll
//! offset. The exact clean background and GUI textures are used; no synthetic
//! world content is introduced.

#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

pub use ffone_client::ui_startup;
use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use bevy::{
    asset::{AssetPlugin, LoadState},
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    window::{PresentMode, WindowResolution},
};

mod localization {
    use std::collections::BTreeMap;

    use bevy::prelude::*;

    #[derive(Clone, Debug, Resource)]
    pub struct Localization;

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
    pub enum LocalizationSet {
        Apply,
    }

    #[derive(Clone, Debug, Component, Eq, PartialEq)]
    pub struct LocalizedText {
        pub key: String,
        pub fallback: String,
        pub args: BTreeMap<String, String>,
    }

    impl LocalizedText {
        pub fn new(key: impl Into<String>, fallback: impl Into<String>) -> Self {
            Self {
                key: key.into(),
                fallback: fallback.into(),
                args: BTreeMap::new(),
            }
        }

        pub fn with_arg(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
            self.args.insert(name.into(), value.into());
            self
        }
    }
}

#[allow(dead_code)]
#[path = "../../../src/ui/server_selection/mod.rs"]
mod server_selection_ui;

use server_selection_ui::{
    SERVER_SELECTION_FONT_PATH, SERVER_SELECTION_IMAGE_PATHS,
    SERVER_SELECTION_REP_SHARD_LIST_PACKET_ID, ServerSelectionInit, ServerSelectionInputBoundary,
    ServerSelectionLoginSnapshot, ServerSelectionUiElement, ServerSelectionUiModel,
    ServerSelectionUiOutbox, ServerSelectionUiPlugin, ServerSelectionUiSet,
    ServerSelectionUiTextRole,
};

const CLIENT_AREA_WIDTH: u32 = 1_264;
const CLIENT_AREA_HEIGHT: u32 = 681;
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(25);
const MIN_BACKGROUND_DETAIL_PIXELS: usize = 250_000;
const MIN_PANEL_CYAN_PIXELS: usize = 1_200;
const MIN_STATUS_COLOUR_PIXELS: usize = 80;

#[derive(Clone, Debug, Eq, PartialEq)]
struct PreviewCli {
    output: PathBuf,
}

fn parse_cli<I>(arguments: I) -> Result<PreviewCli, &'static str>
where
    I: IntoIterator<Item = std::ffi::OsString>,
{
    let mut arguments = arguments.into_iter();
    let output = arguments.next().map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from("target/ui-parity/server-selection-expanded-1264x681.png")
    });
    if arguments.next().is_some()
        || output.extension().and_then(|value| value.to_str()) != Some("png")
    {
        return Err("usage: server_selection_ui_gpu_preview [OUTPUT.png]");
    }
    Ok(PreviewCli { output })
}

#[derive(Resource)]
struct PreviewOutput(PathBuf);

#[derive(Resource)]
struct PreviewAssets {
    images: Vec<Handle<Image>>,
    font: Handle<Font>,
}

#[derive(Resource)]
struct PreviewState {
    frames: u32,
    ready_frame: Option<u32>,
    ready_at: Option<Instant>,
    capture_issued: bool,
    capture_saved: bool,
    capture_failed: bool,
    started_at: Instant,
}

impl Default for PreviewState {
    fn default() -> Self {
        Self {
            frames: 0,
            ready_frame: None,
            ready_at: None,
            capture_issued: false,
            capture_saved: false,
            capture_failed: false,
            started_at: Instant::now(),
        }
    }
}

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

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(PreviewOutput(cli.output))
        .insert_resource(PreviewState::default())
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne Retrobution Server Selection acceptance".into(),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(ServerSelectionUiPlugin)
        .add_systems(Startup, setup_preview)
        .add_systems(Update, drive_capture.after(ServerSelectionUiSet::Bind))
        .run();
}

fn setup_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut model: ResMut<ServerSelectionUiModel>,
    mut outbox: ResMut<ServerSelectionUiOutbox>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    model.open(
        ServerSelectionInit {
            login: ServerSelectionLoginSnapshot {
                character_count: 2,
                account_id: "Retrobution".into(),
                ..default()
            },
            auto_login: false,
            warp_shard: false,
        },
        &mut outbox,
    );
    outbox.clear();
    model.toggle_server();
    let mut statuses = [0_u8; 26];
    for (index, status) in statuses.iter_mut().enumerate().skip(1) {
        *status = (index % 4) as u8;
    }
    assert!(model.receive_shard_list(SERVER_SELECTION_REP_SHARD_LIST_PACKET_ID, statuses));
    assert!(model.select_shard(1, 7));
    model.scroll_by(36.0);

    commands.insert_resource(PreviewAssets {
        images: SERVER_SELECTION_IMAGE_PATHS
            .into_iter()
            .map(|path| asset_server.load(path))
            .collect(),
        font: asset_server.load(SERVER_SELECTION_FONT_PATH),
    });
}

#[allow(clippy::too_many_arguments)]
fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    fonts: Res<Assets<Font>>,
    preview_assets: Res<PreviewAssets>,
    model: Res<ServerSelectionUiModel>,
    mut state: ResMut<PreviewState>,
    elements: Query<(&ServerSelectionUiElement, &ComputedNode, &Node)>,
    texts: Query<(&ServerSelectionUiTextRole, &Text)>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);
    let failed = preview_assets
        .images
        .iter()
        .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
        || matches!(
            asset_server.load_state(preview_assets.font.id()),
            LoadState::Failed(_)
        );
    if failed {
        eprintln!("Server Selection acceptance asset failed to load");
        exit.write(AppExit::error());
        return;
    }

    let assets_loaded = preview_assets
        .images
        .iter()
        .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
        && matches!(
            asset_server.load_state(preview_assets.font.id()),
            LoadState::Loaded
        );
    let cpu_assets_present = preview_assets
        .images
        .iter()
        .all(|handle| images.get(handle).is_some())
        && fonts.get(&preview_assets.font).is_some();
    let root_exact = elements.iter().any(|(element, computed, node)| {
        *element == ServerSelectionUiElement::Root
            && node.display == Display::Flex
            && computed.size() == Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32)
    });
    let panel_exact = elements.iter().any(|(element, computed, node)| {
        *element == ServerSelectionUiElement::Panel
            && computed.size() == Vec2::new(375.0, 330.0)
            && node.left == px(705)
            && node.top == px(180)
    });
    let selected_text_exact = texts
        .iter()
        .any(|(role, text)| *role == ServerSelectionUiTextRole::ShardStatus(7) && text.0 == "BUSY");
    let boundary_exact = model.input_boundary()
        == ServerSelectionInputBoundary {
            blocks_lower_ui: true,
            blocks_gameplay_input: true,
            requires_pointer: true,
            escape_enabled: false,
        };
    let ready = assets_loaded
        && cpu_assets_present
        && root_exact
        && panel_exact
        && selected_text_exact
        && boundary_exact
        && model.server_expanded
        && model.selected_shard == 7
        && (model.scroll_y - 36.0).abs() <= 0.001;
    if ready && state.ready_frame.is_none() {
        state.ready_frame = Some(state.frames);
        state.ready_at = Some(Instant::now());
    }

    let warmed = state
        .ready_frame
        .is_some_and(|frame| state.frames.saturating_sub(frame) >= WARMUP_FRAMES_AFTER_LOAD)
        && state
            .ready_at
            .is_some_and(|ready_at| ready_at.elapsed() >= GPU_UPLOAD_GRACE);
    if ready && warmed && !state.capture_issued {
        state.capture_issued = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }

    if state.capture_saved {
        exit.write(AppExit::Success);
    } else if state.capture_failed {
        exit.write(AppExit::error());
    } else if state.started_at.elapsed() >= CAPTURE_TIMEOUT {
        eprintln!("Server Selection capture timed out");
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

fn save_screenshot(
    event: On<ScreenshotCaptured>,
    output: Res<PreviewOutput>,
    mut state: ResMut<PreviewState>,
) {
    let image = event
        .image
        .clone()
        .try_into_dynamic()
        .expect("GPU screenshot must convert to a dynamic image");
    let rgba = image.to_rgba8();
    if rgba.dimensions() != (CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT) {
        eprintln!(
            "rejecting Server Selection capture with unexpected size {:?}",
            rgba.dimensions()
        );
        state.capture_failed = true;
        return;
    }

    let mut background_detail = 0;
    let mut panel_cyan = 0;
    let mut status_colours = 0;
    for (x, y, pixel) in rgba.enumerate_pixels() {
        let [red, green, blue, alpha] = pixel.0;
        if alpha > 0 && u16::from(red) + u16::from(green) + u16::from(blue) > 90 {
            background_detail += 1;
        }
        if alpha > 0
            && (700..1_085).contains(&x)
            && (175..615).contains(&y)
            && green > 100
            && blue > 110
            && blue > red.saturating_add(20)
        {
            panel_cyan += 1;
        }
        if alpha > 0
            && (870..1_000).contains(&x)
            && (205..455).contains(&y)
            && ((red > 180 && green < 80)
                || (green > 180 && red < 100)
                || (red > 180 && green > 150 && blue < 100))
        {
            status_colours += 1;
        }
    }
    if background_detail < MIN_BACKGROUND_DETAIL_PIXELS
        || panel_cyan < MIN_PANEL_CYAN_PIXELS
        || status_colours < MIN_STATUS_COLOUR_PIXELS
    {
        eprintln!(
            "rejecting incomplete Server Selection capture: background={background_detail} (min {MIN_BACKGROUND_DETAIL_PIXELS}), panel={panel_cyan} (min {MIN_PANEL_CYAN_PIXELS}), status={status_colours} (min {MIN_STATUS_COLOUR_PIXELS})"
        );
        state.capture_failed = true;
        return;
    }

    if let Some(parent) = output
        .0
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("cannot create {}: {error}", parent.display()));
    }
    image
        .save(&output.0)
        .unwrap_or_else(|error| panic!("cannot save {}: {error}", output.0.display()));
    println!(
        "{}",
        output
            .0
            .canonicalize()
            .unwrap_or_else(|_| output.0.clone())
            .display()
    );
    state.capture_saved = true;
}

#[cfg(test)]
mod tests;
