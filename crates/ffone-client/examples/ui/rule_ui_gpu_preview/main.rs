//! Deterministic 1264x681 GPU acceptance frame for clean-Retrobution
//! `eGameMode.Rule`.
//!
//! The harness uses the production module and localization bundles. It
//! supports both reachable clean RulesTable pages and keeps Back in the
//! Hovered visual state.

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

use ffone_client::{
    localization::{Localization, LocalizationPlugin},
    rule_ui::{
        RULE_UI_CHALET_FONT_PATH, RULE_UI_IMAGE_PATHS, RULE_UI_JEFFE_FONT_PATH, RulePageId,
        RuleUiButton, RuleUiButtonKind, RuleUiInputBoundary, RuleUiModel, RuleUiPlugin, RuleUiRoot,
        RuleUiSet, RuleUiTextElement, RuleUiTextRole, RuleUiWindow, clean_rule_ui_scale,
        rule_ui_layout,
    },
};

const CLIENT_AREA_WIDTH: u32 = 1_264;
const CLIENT_AREA_HEIGHT: u32 = 681;
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(25);
const MIN_WINDOW_VISIBLE_PIXELS: usize = 250_000;
const MIN_CYAN_PIXELS: usize = 100_000;
const PREVIEW_HOVER: RuleUiButtonKind = RuleUiButtonKind::Back;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
enum PreviewPage {
    Vehicle,
    Combining,
}

impl PreviewPage {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "vehicle" => Some(Self::Vehicle),
            "combining" | "combine" => Some(Self::Combining),
            _ => None,
        }
    }

    const fn rule_page(self) -> RulePageId {
        match self {
            Self::Vehicle => RulePageId::Vehicle,
            Self::Combining => RulePageId::Combining,
        }
    }

    const fn output(self) -> &'static str {
        match self {
            Self::Vehicle => "target/ui-parity/rule-vehicle-1264x681.png",
            Self::Combining => "target/ui-parity/rule-combining-1264x681.png",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PreviewCli {
    page: PreviewPage,
    output: PathBuf,
}

fn parse_cli<I>(arguments: I) -> Result<PreviewCli, &'static str>
where
    I: IntoIterator<Item = std::ffi::OsString>,
{
    let mut arguments = arguments.into_iter();
    let first = arguments.next();
    let (page, output) = match first {
        None => (PreviewPage::Vehicle, None),
        Some(value) => {
            let value = value.to_string_lossy();
            if let Some(page) = PreviewPage::parse(&value) {
                (page, arguments.next().map(PathBuf::from))
            } else if Path::new(value.as_ref())
                .extension()
                .and_then(|value| value.to_str())
                == Some("png")
            {
                (PreviewPage::Vehicle, Some(PathBuf::from(value.as_ref())))
            } else {
                return Err("usage: rule_ui_gpu_preview [vehicle|combining] [OUTPUT.png]");
            }
        }
    };
    if arguments.next().is_some() {
        return Err("usage: rule_ui_gpu_preview [vehicle|combining] [OUTPUT.png]");
    }
    let output = output.unwrap_or_else(|| PathBuf::from(page.output()));
    if output.extension().and_then(|value| value.to_str()) != Some("png") {
        return Err("usage: rule_ui_gpu_preview [vehicle|combining] [OUTPUT.png]");
    }
    Ok(PreviewCli { page, output })
}

#[derive(Resource)]
struct PreviewOutput(PathBuf);

#[derive(Resource)]
struct PreviewAssets {
    images: Vec<Handle<Image>>,
    fonts: [Handle<Font>; 2],
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
        .insert_resource(ClearColor(Color::srgb(0.08, 0.17, 0.25)))
        .insert_resource(cli.page)
        .insert_resource(PreviewOutput(cli.output))
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
                        title: "FFOne Retrobution Rule acceptance".into(),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((RuleUiPlugin, LocalizationPlugin))
        .add_systems(Startup, setup_preview)
        .add_systems(
            Update,
            force_preview_hover
                .after(RuleUiSet::Interaction)
                .before(RuleUiSet::Visuals),
        )
        .add_systems(Update, drive_capture.after(RuleUiSet::Visuals))
        .run();
}

fn setup_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    page: Res<PreviewPage>,
    mut model: ResMut<RuleUiModel>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    spawn_world_backdrop(&mut commands);
    model.open(page.rule_page());

    commands.insert_resource(PreviewAssets {
        images: RULE_UI_IMAGE_PATHS
            .into_iter()
            .map(|path| asset_server.load(path))
            .collect(),
        fonts: [RULE_UI_JEFFE_FONT_PATH, RULE_UI_CHALET_FONT_PATH]
            .map(|path| asset_server.load(path)),
    });
}

fn spawn_world_backdrop(commands: &mut Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: percent(100),
            height: percent(100),
            ..default()
        },
        BackgroundColor(Color::srgb(0.15, 0.42, 0.62)),
        GlobalZIndex(-1_000),
        Pickable::IGNORE,
    ));
}

fn force_preview_hover(mut buttons: Query<(&RuleUiButton, &mut Interaction)>) {
    for (button, mut interaction) in &mut buttons {
        *interaction = if button.kind == PREVIEW_HOVER {
            Interaction::Hovered
        } else {
            Interaction::None
        };
    }
}

#[allow(clippy::too_many_arguments)]
fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    fonts: Res<Assets<Font>>,
    preview_assets: Res<PreviewAssets>,
    page: Res<PreviewPage>,
    model: Res<RuleUiModel>,
    mut state: ResMut<PreviewState>,
    roots: Query<&ComputedNode, With<RuleUiRoot>>,
    windows: Query<(&ComputedNode, &UiTransform), With<RuleUiWindow>>,
    buttons: Query<(&RuleUiButton, &Interaction, &ComputedNode, &Node)>,
    texts: Query<(&RuleUiTextElement, &Text)>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);

    let failed = preview_assets
        .images
        .iter()
        .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
        || preview_assets
            .fonts
            .iter()
            .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)));
    if failed {
        eprintln!("Rule acceptance asset failed to load");
        exit.write(AppExit::error());
        return;
    }

    let assets_loaded = preview_assets
        .images
        .iter()
        .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
        && preview_assets
            .fonts
            .iter()
            .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded));
    let cpu_assets_present = preview_assets
        .images
        .iter()
        .all(|handle| images.get(handle).is_some())
        && preview_assets
            .fonts
            .iter()
            .all(|handle| fonts.get(handle).is_some());
    let root_exact = roots.single().is_ok_and(|node| {
        node.size() == Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32)
    });
    let window_exact = windows.single().is_ok_and(|(node, transform)| {
        node.size() == Vec2::new(1_020.0, 638.0) && transform.scale == Vec2::ONE
    });
    let button_contract = buttons.iter().count() == 4
        && buttons.iter().any(|(button, interaction, node, _)| {
            button.kind == RuleUiButtonKind::Back
                && *interaction == Interaction::Hovered
                && node.size() == Vec2::new(80.0, 25.0)
        })
        && buttons.iter().any(|(button, _, node, _)| {
            button.kind == RuleUiButtonKind::Close && node.size() == Vec2::new(30.0, 30.0)
        })
        && buttons.iter().all(|(button, _, _, node)| {
            !matches!(
                button.kind,
                RuleUiButtonKind::Previous | RuleUiButtonKind::Next
            ) || node.display == Display::None
        });
    let title = page.rule_page().spec().strings[0];
    let page_contract = model.current_page == Some(page.rule_page())
        && texts.iter().any(|(element, text)| {
            element.role == RuleUiTextRole::Title && text.0.as_str() == title
        });
    let boundary_exact = model.input_boundary()
        == RuleUiInputBoundary {
            blocks_lower_ui: true,
            blocks_gameplay_input: true,
            requires_pointer: true,
            cursor_locked_while_visible: false,
            cursor_locked_after_exit: false,
            mouse_controls_enabled: true,
            escape_close_gate_enabled: true,
        };
    let ready = assets_loaded
        && cpu_assets_present
        && root_exact
        && window_exact
        && button_contract
        && page_contract
        && boundary_exact;
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
        eprintln!("Rule capture timed out");
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
            "rejecting Rule capture with unexpected size {:?}",
            rgba.dimensions()
        );
        state.capture_failed = true;
        return;
    }

    let layout = rule_ui_layout(
        Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32),
        clean_rule_ui_scale(CLIENT_AREA_HEIGHT as f32),
    );
    let window = layout.window.painted;
    let x_start = window.x.max(0.0) as u32;
    let x_end = (window.x + window.width).min(CLIENT_AREA_WIDTH as f32) as u32;
    let y_start = window.y.max(0.0) as u32;
    let y_end = (window.y + window.height).min(CLIENT_AREA_HEIGHT as f32) as u32;
    let mut visible_pixels = 0;
    let mut cyan_pixels = 0;
    for y in y_start..y_end {
        for x in x_start..x_end {
            let [red, green, blue, alpha] = rgba.get_pixel(x, y).0;
            if alpha > 0 && u16::from(red) + u16::from(green) + u16::from(blue) > 90 {
                visible_pixels += 1;
            }
            if alpha > 0 && green > 120 && blue > 120 && green > red.saturating_add(35) {
                cyan_pixels += 1;
            }
        }
    }
    if visible_pixels < MIN_WINDOW_VISIBLE_PIXELS || cyan_pixels < MIN_CYAN_PIXELS {
        eprintln!(
            "rejecting incomplete Rule capture: visible={visible_pixels} \
             (min {MIN_WINDOW_VISIBLE_PIXELS}), cyan={cyan_pixels} \
             (min {MIN_CYAN_PIXELS})"
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
