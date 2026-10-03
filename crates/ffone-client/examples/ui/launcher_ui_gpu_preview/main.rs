//! Deterministic 1264x681 GPU acceptance frame for clean-Retrobution
//! `eGameMode::Launcher`.
//!
//! The harness imports the isolated module by path so the vertical can be
//! validated before production-shell registration. It shows an actively
//! charging launcher at 64% power over a deterministic synthetic world
//! backdrop; the backdrop is not claimed as legacy scene evidence.

#[allow(unused_imports)]
use ffone_client::{coordinates, scene_hierarchy, ui_support};

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

#[allow(dead_code)]
#[path = "../../../src/ui/launcher/mod.rs"]
mod launcher_ui;
mod localization {
    pub use ffone_client::localization::{LocalizationSet, LocalizedText};
}

use ffone_client::localization::{Localization, LocalizationPlugin, LocalizationSet};

use launcher_ui::{
    LAUNCHER_UI_FONT_PATH, LAUNCHER_UI_IMAGE_PATHS, LauncherTriggerSpec, LauncherUiElement,
    LauncherUiInputBoundary, LauncherUiModel, LauncherUiOutbox, LauncherUiPlugin,
    LauncherUiTextRole,
};

const CLIENT_AREA_WIDTH: u32 = 1_264;
const CLIENT_AREA_HEIGHT: u32 = 681;
const PREVIEW_POWER: f32 = 0.64;
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(25);
const MIN_DARK_BORDER_PIXELS: usize = 40_000;
const MIN_CROSSHAIR_DETAIL_PIXELS: usize = 8_000;
const MIN_GAUGE_DETAIL_PIXELS: usize = 20_000;

#[derive(Clone, Debug, Eq, PartialEq)]
struct PreviewCli {
    output: PathBuf,
}

fn parse_cli<I>(arguments: I) -> Result<PreviewCli, &'static str>
where
    I: IntoIterator<Item = std::ffi::OsString>,
{
    let mut arguments = arguments.into_iter();
    let output = arguments
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/ui-parity/launcher-aiming-1264x681.png"));
    if arguments.next().is_some()
        || output.extension().and_then(|value| value.to_str()) != Some("png")
    {
        return Err("usage: launcher_ui_gpu_preview [OUTPUT.png]");
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
    let (localization, language) =
        Localization::open(&asset_root, "en").expect("open production localization bundles");

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.11, 0.27, 0.43)))
        .insert_resource(localization)
        .insert_resource(language)
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
                        title: "FFOne Retrobution Launcher acceptance".into(),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((LauncherUiPlugin, LocalizationPlugin))
        .add_systems(Startup, setup_preview)
        .add_systems(
            Update,
            drive_capture
                .after(launcher_ui::LauncherUiSet::Bind)
                .after(LocalizationSet::Apply),
        )
        .run();
}

fn setup_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut model: ResMut<LauncherUiModel>,
    mut outbox: ResMut<LauncherUiOutbox>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    spawn_synthetic_world_backdrop(&mut commands);

    model
        .open(
            LauncherTriggerSpec {
                trigger_position: Vec3::new(0.0, 100.0, 0.0),
                trigger_euler_degrees: Vec3::new(0.0, 180.0, 0.0),
                min_power: 10.0,
                max_power: 30.0,
                initial_rotation_degrees: Vec3::ZERO,
                maximum_rotation_degrees: Vec3::new(20.0, 45.0, 0.0),
            },
            Vec3::ZERO,
            6.0,
            &mut outbox,
        )
        .expect("preview trigger contract must be valid");
    outbox.clear();
    model.press_fire(false, &mut outbox);
    model.advance_power(PREVIEW_POWER, false, &mut outbox);
    outbox.clear();

    commands.insert_resource(PreviewAssets {
        images: LAUNCHER_UI_IMAGE_PATHS
            .into_iter()
            .map(|path| asset_server.load(path))
            .collect(),
        font: asset_server.load(LAUNCHER_UI_FONT_PATH),
    });
}

fn spawn_synthetic_world_backdrop(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                ..default()
            },
            GlobalZIndex(-1_000),
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: percent(100),
                    height: percent(62),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.43, 0.62)),
            ));
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    bottom: px(0),
                    width: percent(100),
                    height: percent(38),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.20, 0.38, 0.26)),
            ));
            for (left, top, width, height, color) in [
                (360.0, 205.0, 170.0, 250.0, Color::srgb(0.18, 0.31, 0.42)),
                (735.0, 170.0, 145.0, 285.0, Color::srgb(0.24, 0.34, 0.44)),
                (555.0, 330.0, 155.0, 125.0, Color::srgb(0.29, 0.39, 0.34)),
            ] {
                root.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(left),
                        top: px(top),
                        width: px(width),
                        height: px(height),
                        ..default()
                    },
                    BackgroundColor(color),
                ));
            }
        });
}

#[allow(clippy::too_many_arguments)]
fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    fonts: Res<Assets<Font>>,
    preview_assets: Res<PreviewAssets>,
    model: Res<LauncherUiModel>,
    mut state: ResMut<PreviewState>,
    elements: Query<(&LauncherUiElement, &ComputedNode, &Node)>,
    texts: Query<(&LauncherUiTextRole, &Text)>,
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
        eprintln!("Launcher acceptance asset failed to load");
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
        *element == LauncherUiElement::Root
            && node.display == Display::Flex
            && computed.size() == Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32)
    });
    let cross_exact = elements.iter().any(|(element, computed, _)| {
        *element == LauncherUiElement::Crosshair && computed.size() == Vec2::splat(660.0)
    });
    let gauge_exact = elements.iter().any(|(element, computed, _)| {
        *element == LauncherUiElement::Gauge && computed.size() == Vec2::new(169.0, 595.0)
    });
    let bar_exact = elements.iter().any(|(element, computed, _)| {
        *element == LauncherUiElement::GaugeBar && computed.size() == Vec2::new(87.0, 31.0)
    });
    let text_exact = texts.iter().any(|(role, text)| {
        *role == LauncherUiTextRole::Tip
            && text.0 == "HOLD SPACE BAR TO SET POWER AND RELEASE TO FIRE"
    });
    let boundary_exact = model.input_boundary(false)
        == LauncherUiInputBoundary {
            blocks_lower_ui: true,
            blocks_gameplay_input: true,
            requires_pointer: false,
            fire_enabled: true,
            escape_enabled: true,
        };
    let ready = assets_loaded
        && cpu_assets_present
        && root_exact
        && cross_exact
        && gauge_exact
        && bar_exact
        && text_exact
        && boundary_exact
        && (model.normalized_power() - PREVIEW_POWER).abs() <= 0.000_1;
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
        eprintln!("Launcher capture timed out");
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
            "rejecting Launcher capture with unexpected size {:?}",
            rgba.dimensions()
        );
        state.capture_failed = true;
        return;
    }

    let mut dark_border_pixels = 0;
    let mut crosshair_detail_pixels = 0;
    let mut gauge_detail_pixels = 0;
    for (x, y, pixel) in rgba.enumerate_pixels() {
        let [red, green, blue, alpha] = pixel.0;
        if alpha > 0
            && (x < 302 || x >= 962 || y < 10 || y >= 670)
            && u16::from(red) + u16::from(green) + u16::from(blue) < 120
        {
            dark_border_pixels += 1;
        }
        if alpha > 0
            && (302..962).contains(&x)
            && (10..670).contains(&y)
            && red.saturating_add(green).saturating_add(blue) > 100
        {
            crosshair_detail_pixels += 1;
        }
        if alpha > 0
            && (123..292).contains(&x)
            && (40..635).contains(&y)
            && u16::from(red) + u16::from(green) + u16::from(blue) > 90
        {
            gauge_detail_pixels += 1;
        }
    }
    if dark_border_pixels < MIN_DARK_BORDER_PIXELS
        || crosshair_detail_pixels < MIN_CROSSHAIR_DETAIL_PIXELS
        || gauge_detail_pixels < MIN_GAUGE_DETAIL_PIXELS
    {
        eprintln!(
            "rejecting incomplete Launcher capture: dark={dark_border_pixels} (min {MIN_DARK_BORDER_PIXELS}), cross={crosshair_detail_pixels} (min {MIN_CROSSHAIR_DETAIL_PIXELS}), gauge={gauge_detail_pixels} (min {MIN_GAUGE_DETAIL_PIXELS})"
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
