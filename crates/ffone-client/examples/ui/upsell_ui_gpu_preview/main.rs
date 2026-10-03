//! Deterministic 1264x681 GPU acceptance frames for the clean-Retrobution
//! `cnUpsell` UI.
//!
//! `news` exercises the only clean `ReceiveInit` route and deliberately leaves
//! the external news provider empty. `upgrade` is an explicit capture of the
//! retained, clean-unreached level-four mode.

use std::{
    env,
    ffi::OsString,
    fs,
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
    upsell_ui::{
        UPSELL_ACTION_SUCCESS_SOUND_PATH, UPSELL_ADVERTIS_PATH, UPSELL_CLOSE_HOVER_PATH,
        UPSELL_CLOSE_NORMAL_PATH, UPSELL_CONTINUE_HOVER_PATH, UPSELL_CONTINUE_NORMAL_PATH,
        UPSELL_FONT_PATH, UPSELL_GET_HOVER_PATH, UPSELL_GET_NORMAL_PATH, UPSELL_LEVEL_IMAGE_PATHS,
        UPSELL_NEWS_BUTTON_HOVER_PATH, UPSELL_NEWS_BUTTON_NORMAL_PATH, UPSELL_NEWS_CLOSE_RECT,
        UPSELL_NEWS_CONTINUE_RECT, UPSELL_NOT_NOW_HOVER_PATH, UPSELL_NOT_NOW_NORMAL_PATH,
        UPSELL_PANELBACK_PATH, UpsellInputBoundary, UpsellScaledGroup, UpsellUiBackdrop,
        UpsellUiButton, UpsellUiButtonKind, UpsellUiDialog, UpsellUiMode, UpsellUiModel,
        UpsellUiPlugin, UpsellUiRect, UpsellUiRoot, UpsellUiSet, clean_upsell_ui_scale,
        upsell_ui_layout, upsell_upgrade_geometry,
    },
};

const CLIENT_AREA_WIDTH: u32 = 1_264;
const CLIENT_AREA_HEIGHT: u32 = 681;
const PREVIEW_LEVEL: i32 = 4;
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(20);
const MIN_BLUE_PIXELS: usize = 80_000;
const MIN_UPGRADE_BRIGHT_PIXELS: usize = 20_000;

const PREVIEW_IMAGE_PATHS: [&str; 16] = [
    UPSELL_PANELBACK_PATH,
    UPSELL_LEVEL_IMAGE_PATHS[0],
    UPSELL_LEVEL_IMAGE_PATHS[1],
    UPSELL_LEVEL_IMAGE_PATHS[2],
    UPSELL_LEVEL_IMAGE_PATHS[3],
    UPSELL_CLOSE_NORMAL_PATH,
    UPSELL_CLOSE_HOVER_PATH,
    UPSELL_GET_NORMAL_PATH,
    UPSELL_GET_HOVER_PATH,
    UPSELL_CONTINUE_NORMAL_PATH,
    UPSELL_CONTINUE_HOVER_PATH,
    UPSELL_NOT_NOW_NORMAL_PATH,
    UPSELL_NOT_NOW_HOVER_PATH,
    UPSELL_ADVERTIS_PATH,
    UPSELL_NEWS_BUTTON_NORMAL_PATH,
    UPSELL_NEWS_BUTTON_HOVER_PATH,
];

const EXPECTED_INPUT_BOUNDARY: UpsellInputBoundary = UpsellInputBoundary {
    blocks_lower_ui: true,
    blocks_gameplay_input: true,
    requires_pointer: true,
    mouse_controls_enabled: true,
    escape_dismiss_enabled: false,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
enum PreviewMode {
    News,
    Upgrade,
}

impl PreviewMode {
    const fn cli_name(self) -> &'static str {
        match self {
            Self::News => "news",
            Self::Upgrade => "upgrade",
        }
    }

    const fn default_output(self) -> &'static str {
        match self {
            Self::News => "target/ui-parity/upsell-news-loading-1264x681.png",
            Self::Upgrade => "target/ui-parity/upsell-upgrade-level4-1264x681.png",
        }
    }

    const fn expected_ui_mode(self) -> UpsellUiMode {
        match self {
            Self::News => UpsellUiMode::NewsFreeZone,
            Self::Upgrade => UpsellUiMode::Upgrade,
        }
    }

    const fn hover_target(self) -> UpsellUiButtonKind {
        match self {
            Self::News => UpsellUiButtonKind::Continue,
            Self::Upgrade => UpsellUiButtonKind::GetUpgrade,
        }
    }
}

#[derive(Resource)]
struct PreviewOutput(PathBuf);

#[derive(Resource)]
struct PreviewAssets {
    images: Vec<Handle<Image>>,
    font: Handle<Font>,
    action_success: Handle<AudioSource>,
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
    let (mode, output) = parse_preview_args(std::env::args_os().skip(1)).unwrap_or_else(|error| {
        eprintln!("{error}");
        eprintln!("usage: upsell_ui_gpu_preview [news|upgrade] [OUTPUT.png]");
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
        .insert_resource(mode)
        .insert_resource(PreviewOutput(output))
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
                        title: format!("FFOne Retrobution Upsell {} acceptance", mode.cli_name()),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((UpsellUiPlugin, LocalizationPlugin))
        .add_systems(Startup, setup_preview)
        .add_systems(
            Update,
            force_preview_hover
                .after(UpsellUiSet::Bind)
                .before(UpsellUiSet::Visuals),
        )
        .add_systems(Update, drive_capture.after(UpsellUiSet::Visuals))
        .run();
}

fn parse_preview_args(
    args: impl IntoIterator<Item = OsString>,
) -> Result<(PreviewMode, PathBuf), String> {
    let mut args = args.into_iter();
    let first = args.next();
    let (mode, output) = match first.as_deref().and_then(|value| value.to_str()) {
        None => (
            PreviewMode::News,
            PathBuf::from(PreviewMode::News.default_output()),
        ),
        Some("news") => {
            let mode = PreviewMode::News;
            let output = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(mode.default_output()));
            (mode, output)
        }
        Some("upgrade") => {
            let mode = PreviewMode::Upgrade;
            let output = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(mode.default_output()));
            (mode, output)
        }
        Some(_) => (
            PreviewMode::News,
            PathBuf::from(first.expect("the matched argument exists")),
        ),
    };
    if args.next().is_some() {
        return Err("too many arguments".to_owned());
    }
    if output.extension().and_then(|value| value.to_str()) != Some("png") {
        return Err(format!("output must be a .png path: {}", output.display()));
    }
    Ok((mode, output))
}

fn setup_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mode: Res<PreviewMode>,
    mut model: ResMut<UpsellUiModel>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    spawn_world_backdrop(&mut commands);

    match *mode {
        PreviewMode::News => {
            model
                .receive_init(4, false)
                .expect("clean Upsell ReceiveInit must accept level four");
            // The clean asynchronous provider is intentionally unresolved:
            // no local image, including `upsellpage`, is a proven fallback.
            assert_eq!(model.news_page_count(), 0);
            assert_eq!(model.current_news_page_path(), None);
        }
        PreviewMode::Upgrade => model
            .open_retained_upgrade(4)
            .expect("retained Upsell upgrade must accept level four"),
    }

    commands.insert_resource(PreviewAssets {
        images: PREVIEW_IMAGE_PATHS
            .into_iter()
            .map(|path| asset_server.load(path))
            .collect(),
        font: asset_server.load(UPSELL_FONT_PATH),
        action_success: asset_server.load(UPSELL_ACTION_SUCCESS_SOUND_PATH),
    });
}

fn spawn_world_backdrop(commands: &mut Commands) {
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
            BackgroundColor(Color::srgb(0.15, 0.42, 0.62)),
            GlobalZIndex(-1_000),
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            for (left, top, width, height, color) in [
                (55.0, 55.0, 285.0, 170.0, Color::srgb(0.86, 0.43, 0.16)),
                (905.0, 65.0, 295.0, 185.0, Color::srgb(0.24, 0.76, 0.38)),
                (65.0, 455.0, 315.0, 155.0, Color::srgb(0.66, 0.24, 0.72)),
                (865.0, 445.0, 330.0, 165.0, Color::srgb(0.9, 0.73, 0.16)),
            ] {
                root.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(left),
                        top: px(top),
                        width: px(width),
                        height: px(height),
                        border: UiRect::all(px(3)),
                        ..default()
                    },
                    BackgroundColor(color),
                    BorderColor::all(Color::WHITE),
                    Pickable::IGNORE,
                ));
            }
        });
}

fn force_preview_hover(
    mode: Res<PreviewMode>,
    mut buttons: Query<(&UpsellUiButton, &mut Interaction)>,
) {
    for (button, mut interaction) in &mut buttons {
        *interaction = if button.kind == mode.hover_target() {
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
    audio_sources: Res<Assets<AudioSource>>,
    preview_assets: Res<PreviewAssets>,
    mode: Res<PreviewMode>,
    model: Res<UpsellUiModel>,
    mut state: ResMut<PreviewState>,
    roots: Query<(&Node, &ComputedNode), With<UpsellUiRoot>>,
    backdrops: Query<(&Node, &ComputedNode, &UiTransform), With<UpsellUiBackdrop>>,
    dialogs: Query<(&Node, &ComputedNode, &UiTransform), With<UpsellUiDialog>>,
    buttons: Query<(&UpsellUiButton, &Node, &ComputedNode, &Interaction)>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);

    let asset_failed = preview_assets
        .images
        .iter()
        .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
        || matches!(
            asset_server.load_state(preview_assets.font.id()),
            LoadState::Failed(_)
        )
        || matches!(
            asset_server.load_state(preview_assets.action_success.id()),
            LoadState::Failed(_)
        );
    if asset_failed {
        eprintln!("Upsell acceptance asset failed to load");
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
        )
        && matches!(
            asset_server.load_state(preview_assets.action_success.id()),
            LoadState::Loaded
        );
    let cpu_assets_present = preview_assets
        .images
        .iter()
        .all(|handle| images.get(handle).is_some())
        && fonts.get(&preview_assets.font).is_some()
        && audio_sources.get(&preview_assets.action_success).is_some();

    let viewport = Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32);
    let expected_mode = mode.expected_ui_mode();
    let ui_scale = clean_upsell_ui_scale(CLIENT_AREA_HEIGHT as f32);
    let layout = upsell_ui_layout(viewport, ui_scale, expected_mode);
    let root_exact = roots.single().is_ok_and(|(node, computed)| {
        node.display == Display::Flex
            && node_matches_rect(
                node,
                UpsellUiRect::new(
                    0.0,
                    0.0,
                    CLIENT_AREA_WIDTH as f32,
                    CLIENT_AREA_HEIGHT as f32,
                ),
            )
            && computed.size() == viewport
    });
    let backdrop_exact = backdrops.single().is_ok_and(|(node, computed, transform)| {
        scaled_group_is_exact(node, computed, transform, layout.background)
    });
    let dialog_exact = dialogs.single().is_ok_and(|(node, computed, transform)| {
        scaled_group_is_exact(node, computed, transform, layout.dialog)
    });

    let mut button_count = 0;
    let mut visible_button_count = 0;
    let mut button_geometry_exact = true;
    for (button, node, computed, interaction) in &buttons {
        button_count += 1;
        let expected_rect = expected_button_rect(*mode, button.kind);
        let expected_interaction = if button.kind == mode.hover_target() {
            Interaction::Hovered
        } else {
            Interaction::None
        };
        button_geometry_exact &= *interaction == expected_interaction;
        if let Some(rect) = expected_rect {
            visible_button_count += 1;
            button_geometry_exact &= node.display == Display::Flex
                && node_matches_rect(node, rect)
                && computed.size() == Vec2::new(rect.width, rect.height);
        } else {
            button_geometry_exact &= node.display == Display::None;
        }
    }
    let expected_visible_button_count = match *mode {
        PreviewMode::News => 2,
        PreviewMode::Upgrade => 4,
    };
    let buttons_exact = button_count == 4
        && visible_button_count == expected_visible_button_count
        && button_geometry_exact;

    let model_exact = model.visible()
        && model.active_mode() == Some(expected_mode)
        && model.level() == Some(PREVIEW_LEVEL as u8)
        && model.news_page_count() == 0
        && model.current_news_page_path().is_none()
        && match *mode {
            PreviewMode::News => model.page_alpha() == 0.0,
            PreviewMode::Upgrade => model.page_alpha() == 1.0,
        };
    let boundary_exact = model.input_boundary() == EXPECTED_INPUT_BOUNDARY;
    let ready = assets_loaded
        && cpu_assets_present
        && ui_scale == 1.0
        && root_exact
        && backdrop_exact
        && dialog_exact
        && buttons_exact
        && model_exact
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
        eprintln!(
            "Upsell {} capture timed out: assets={assets_loaded}/{cpu_assets_present} \
             scale={ui_scale} root={root_exact} backdrop={backdrop_exact} \
             dialog={dialog_exact} buttons={buttons_exact} model={model_exact} \
             boundary={boundary_exact}",
            mode.cli_name()
        );
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

fn expected_button_rect(mode: PreviewMode, kind: UpsellUiButtonKind) -> Option<UpsellUiRect> {
    match mode {
        PreviewMode::News => match kind {
            UpsellUiButtonKind::Close => Some(UPSELL_NEWS_CLOSE_RECT),
            UpsellUiButtonKind::Continue => Some(UPSELL_NEWS_CONTINUE_RECT),
            UpsellUiButtonKind::GetUpgrade | UpsellUiButtonKind::NotRightNow => None,
        },
        PreviewMode::Upgrade => {
            let geometry =
                upsell_upgrade_geometry(PREVIEW_LEVEL as u8).expect("level four is valid");
            Some(match kind {
                UpsellUiButtonKind::Close => geometry.close,
                UpsellUiButtonKind::Continue => geometry.continue_playing,
                UpsellUiButtonKind::GetUpgrade => geometry.get_upgrade,
                UpsellUiButtonKind::NotRightNow => geometry.not_right_now,
            })
        }
    }
}

fn node_matches_rect(node: &Node, rect: UpsellUiRect) -> bool {
    node.position_type == PositionType::Absolute
        && node.left == px(rect.x)
        && node.top == px(rect.y)
        && node.width == px(rect.width)
        && node.height == px(rect.height)
}

fn scaled_group_is_exact(
    node: &Node,
    computed: &ComputedNode,
    transform: &UiTransform,
    expected: UpsellScaledGroup,
) -> bool {
    node_matches_rect(
        node,
        UpsellUiRect::new(
            expected.node_left,
            expected.node_top,
            expected.source.width,
            expected.source.height,
        ),
    ) && transform.scale == Vec2::splat(expected.scale)
        && computed.size() == Vec2::new(expected.source.width, expected.source.height)
}

fn save_screenshot(
    event: On<ScreenshotCaptured>,
    output: Res<PreviewOutput>,
    mode: Res<PreviewMode>,
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
            "rejecting Upsell capture with unexpected size {:?}",
            rgba.dimensions()
        );
        state.capture_failed = true;
        return;
    }

    let blue_pixels = rgba
        .pixels()
        .filter(|pixel| {
            let [red, green, blue, _alpha] = pixel.0;
            blue > 55 && blue > red.saturating_add(20) && green > red.saturating_add(5)
        })
        .count();
    let layout = upsell_ui_layout(
        Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32),
        1.0,
        mode.expected_ui_mode(),
    );
    let dialog = layout.dialog.painted;
    let mut upgrade_bright_pixels = 0;
    for y in
        dialog.y.max(0.0) as u32..(dialog.y + dialog.height).min(CLIENT_AREA_HEIGHT as f32) as u32
    {
        for x in
            dialog.x.max(0.0) as u32..(dialog.x + dialog.width).min(CLIENT_AREA_WIDTH as f32) as u32
        {
            let [red, green, blue, _alpha] = rgba.get_pixel(x, y).0;
            if red > 165 && green > 165 && blue > 165 {
                upgrade_bright_pixels += 1;
            }
        }
    }
    let upgrade_complete =
        *mode == PreviewMode::News || upgrade_bright_pixels >= MIN_UPGRADE_BRIGHT_PIXELS;
    if blue_pixels < MIN_BLUE_PIXELS || !upgrade_complete {
        eprintln!(
            "rejecting incomplete Upsell {} capture: blue={blue_pixels} \
             (min {MIN_BLUE_PIXELS}), upgrade-bright={upgrade_bright_pixels} \
             (min {MIN_UPGRADE_BRIGHT_PIXELS} in upgrade mode)",
            mode.cli_name()
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
