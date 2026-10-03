//! Deterministic 1264x681 GPU acceptance frames for clean-Retrobution
//! `cnGuideMode`.
//!
//! The harness uses the production Guide renderer and production localization
//! bundles. It covers the Past-Warp warning, direct initial selection, normal
//! guide change and both confirmation variants.

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
    guide_ui::{
        GUIDE_ALERT_ICON_PATH, GUIDE_BLACK_TEXTURE_PATH, GUIDE_CANCEL_BUTTON_PATH,
        GUIDE_CARD_FRAME_PATH, GUIDE_CHANGE_BACKGROUND_PATH, GUIDE_CLOSE_BUTTON_OVER_PATH,
        GUIDE_CLOSE_BUTTON_PATH, GUIDE_COMPUTRESS_FRAME_PATH, GUIDE_COMPUTRESS_ICON_PATH,
        GUIDE_CONFIRM_ICON_PATH, GUIDE_COST_BAR_PATH, GUIDE_CURRENT_FRAME_PATH, GUIDE_DIALOG_PATH,
        GUIDE_FUSION_MATTER_ICON_PATH, GUIDE_HELP_BUTTON_OVER_PATH, GUIDE_HELP_BUTTON_PATH,
        GUIDE_ICON_FRAME_PATH, GUIDE_JEFFE_FONT_PATH, GUIDE_MEMBER_BACK_PATH, GUIDE_PANEL_PATH,
        GUIDE_SELECT_BACKGROUND_PATH, GUIDE_SELECTED_EFFECT_PATH, GUIDE_TOGGLE_OFF_PATH,
        GUIDE_TOGGLE_ON_PATH, GuideMentor, GuideUiConfirmationWindow, GuideUiModel, GuideUiPhase,
        GuideUiPlugin, GuideUiPurpose, GuideUiRoot, GuideUiSelectionWindow, GuideUiTextElement,
        GuideUiTextRole, GuideUiWarpWindow,
    },
    localization::{Localization, LocalizationPlugin, LocalizedText},
};

const CLIENT_AREA_WIDTH: u32 = 1_264;
const CLIENT_AREA_HEIGHT: u32 = 681;
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(25);
const MIN_NON_BACKDROP_PIXELS: usize = 35_000;

const GUIDE_PREVIEW_IMAGE_PATHS: [&str; 23] = [
    GUIDE_SELECT_BACKGROUND_PATH,
    GUIDE_CHANGE_BACKGROUND_PATH,
    GUIDE_PANEL_PATH,
    GUIDE_MEMBER_BACK_PATH,
    GUIDE_SELECTED_EFFECT_PATH,
    GUIDE_ALERT_ICON_PATH,
    GUIDE_CONFIRM_ICON_PATH,
    GUIDE_BLACK_TEXTURE_PATH,
    GUIDE_COST_BAR_PATH,
    GUIDE_FUSION_MATTER_ICON_PATH,
    GUIDE_COMPUTRESS_ICON_PATH,
    GUIDE_CARD_FRAME_PATH,
    GUIDE_CURRENT_FRAME_PATH,
    GUIDE_COMPUTRESS_FRAME_PATH,
    GUIDE_ICON_FRAME_PATH,
    GUIDE_DIALOG_PATH,
    GUIDE_TOGGLE_OFF_PATH,
    GUIDE_TOGGLE_ON_PATH,
    ffone_client::guide_ui::GUIDE_BLUE_BUTTON_PATH,
    ffone_client::guide_ui::GUIDE_BLUE_BUTTON_OVER_PATH,
    GUIDE_CANCEL_BUTTON_PATH,
    GUIDE_CLOSE_BUTTON_PATH,
    GUIDE_CLOSE_BUTTON_OVER_PATH,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
enum PreviewScene {
    Warp,
    InitialSelection,
    Change,
    InitialConfirmation,
    ChangeConfirmation,
}

impl PreviewScene {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "warp" => Some(Self::Warp),
            "selection" | "initial" => Some(Self::InitialSelection),
            "change" => Some(Self::Change),
            "confirm-initial" => Some(Self::InitialConfirmation),
            "confirm-change" => Some(Self::ChangeConfirmation),
            _ => None,
        }
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::Warp => "warp",
            Self::InitialSelection => "selection",
            Self::Change => "change",
            Self::InitialConfirmation => "confirm-initial",
            Self::ChangeConfirmation => "confirm-change",
        }
    }

    fn apply(self, model: &mut GuideUiModel) {
        match self {
            Self::Warp => model.open_initial_selection(),
            Self::InitialSelection => {
                model.open_initial_mentor_selection();
                model.selected = Some(GuideMentor::MojoJojo);
            }
            Self::Change => {
                model.open_change(GuideMentor::Dexter);
                model.selected = Some(GuideMentor::Edd);
            }
            Self::InitialConfirmation => {
                model.open_initial_mentor_selection();
                model.selected = Some(GuideMentor::BenTennyson);
                model.confirmation_open = true;
            }
            Self::ChangeConfirmation => {
                model.open_change(GuideMentor::Dexter);
                model.selected = Some(GuideMentor::Edd);
                model.confirmation_open = true;
            }
        }
    }

    const fn expected_heading_key(self) -> Option<&'static str> {
        match self {
            Self::Warp => None,
            Self::InitialSelection | Self::InitialConfirmation => Some("ui.guide.heading.choose"),
            Self::Change | Self::ChangeConfirmation => Some("ui.guide.heading.change"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PreviewCli {
    scene: PreviewScene,
    locale: String,
    output: PathBuf,
}

fn parse_cli<I>(arguments: I) -> Result<PreviewCli, &'static str>
where
    I: IntoIterator<Item = std::ffi::OsString>,
{
    let usage = "usage: guide_ui_gpu_preview [warp|selection|change|confirm-initial|confirm-change] [en|ru] [OUTPUT.png]";
    let mut arguments = arguments.into_iter();
    let scene = arguments
        .next()
        .map(|value| PreviewScene::parse(&value.to_string_lossy()).ok_or(usage))
        .transpose()?
        .unwrap_or(PreviewScene::ChangeConfirmation);
    let locale = arguments
        .next()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "en".to_owned());
    if locale != "en" && locale != "ru" {
        return Err(usage);
    }
    let output = arguments.next().map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(format!(
            "target/ui-parity/guide-{}-{}-1264x681.png",
            scene.slug(),
            locale
        ))
    });
    if arguments.next().is_some()
        || output.extension().and_then(|value| value.to_str()) != Some("png")
    {
        return Err(usage);
    }
    Ok(PreviewCli {
        scene,
        locale,
        output,
    })
}

#[derive(Resource)]
struct PreviewOutput(PathBuf);

#[derive(Resource)]
struct PreviewAssets {
    images: Vec<Handle<Image>>,
    fonts: Vec<Handle<Font>>,
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
        Localization::open(&asset_root, &cli.locale).expect("open production localization bundles");

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.15, 0.42, 0.62)))
        .insert_resource(cli.scene)
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
                        title: "FFOne Retrobution Guide acceptance".into(),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((GuideUiPlugin, LocalizationPlugin))
        .add_systems(Startup, setup_preview)
        .add_systems(Update, drive_capture)
        .run();
}

fn setup_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    scene: Res<PreviewScene>,
    mut model: ResMut<GuideUiModel>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    scene.apply(&mut model);

    let mut image_paths = GUIDE_PREVIEW_IMAGE_PATHS.to_vec();
    image_paths.extend([GUIDE_HELP_BUTTON_PATH, GUIDE_HELP_BUTTON_OVER_PATH]);
    for mentor in GuideMentor::CLEAN_ORDER {
        image_paths.extend([
            mentor.portrait_path(),
            mentor.icon_path(),
            mentor.confirm_path(),
        ]);
    }
    commands.insert_resource(PreviewAssets {
        images: image_paths
            .into_iter()
            .map(|path| asset_server.load(path))
            .collect(),
        fonts: [
            GUIDE_JEFFE_FONT_PATH,
            ffone_client::guide_ui::GUIDE_CHALET_FONT_PATH,
        ]
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect(),
    });
}

#[allow(clippy::too_many_arguments)]
fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    fonts: Res<Assets<Font>>,
    preview_assets: Res<PreviewAssets>,
    scene: Res<PreviewScene>,
    model: Res<GuideUiModel>,
    mut state: ResMut<PreviewState>,
    roots: Query<&ComputedNode, With<GuideUiRoot>>,
    selection: Query<&Node, With<GuideUiSelectionWindow>>,
    warp: Query<&Node, With<GuideUiWarpWindow>>,
    confirmation: Query<&Node, With<GuideUiConfirmationWindow>>,
    texts: Query<(&GuideUiTextElement, &Text, &LocalizedText)>,
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
        eprintln!("Guide acceptance asset failed to load");
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
    let expected_selection = !matches!(*scene, PreviewScene::Warp);
    let expected_confirmation = matches!(
        *scene,
        PreviewScene::InitialConfirmation | PreviewScene::ChangeConfirmation
    );
    let visibility_exact = selection
        .single()
        .is_ok_and(|node| (node.display == Display::Flex) == expected_selection)
        && warp
            .single()
            .is_ok_and(|node| (node.display == Display::Flex) == !expected_selection)
        && confirmation
            .single()
            .is_ok_and(|node| (node.display == Display::Flex) == expected_confirmation);
    let semantic_exact = scene.expected_heading_key().is_none_or(|key| {
        texts.iter().any(|(element, text, localized)| {
            element.role == GuideUiTextRole::SelectionHeading
                && localized.key == key
                && !text.0.is_empty()
        })
    });
    let model_exact = model.visible
        && model.phase
            == if *scene == PreviewScene::Warp {
                GuideUiPhase::WarpWarning
            } else {
                GuideUiPhase::MentorSelection
            }
        && model.confirmation_open == expected_confirmation
        && model.purpose
            == if matches!(
                *scene,
                PreviewScene::Change | PreviewScene::ChangeConfirmation
            ) {
                GuideUiPurpose::ChangeMentor
            } else {
                GuideUiPurpose::InitialSelection
            };
    let ready = assets_loaded
        && cpu_assets_present
        && root_exact
        && visibility_exact
        && semantic_exact
        && model_exact;
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
        // Keep the production UI available for pointer/state acceptance checks.
        if env::var_os("FFONE_GUIDE_PREVIEW_INTERACTIVE").is_none() {
            exit.write(AppExit::Success);
        }
    } else if state.capture_failed {
        exit.write(AppExit::error());
    } else if state.started_at.elapsed() >= CAPTURE_TIMEOUT {
        eprintln!("Guide capture timed out");
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
        eprintln!("rejecting Guide capture with unexpected size");
        state.capture_failed = true;
        return;
    }
    let backdrop = [38_u8, 107_u8, 158_u8];
    let non_backdrop_pixels = rgba
        .pixels()
        .filter(|pixel| {
            let [red, green, blue, alpha] = pixel.0;
            alpha > 0
                && (red.abs_diff(backdrop[0]) > 12
                    || green.abs_diff(backdrop[1]) > 12
                    || blue.abs_diff(backdrop[2]) > 12)
        })
        .count();
    if non_backdrop_pixels < MIN_NON_BACKDROP_PIXELS {
        eprintln!(
            "rejecting incomplete Guide capture: non-backdrop={non_backdrop_pixels} (min {MIN_NON_BACKDROP_PIXELS})"
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
