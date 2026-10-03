//! Deterministic 1264x681 GPU acceptance frame for clean-Retrobution login.
//!
//! The harness covers each reached clean background branch (`Manual`,
//! `WarpShard`, `AutoLogin`) in EN and RU. Manual frames use the approved
//! JEFFE/Chalet replacement fonts and audit exact serialized line spacing,
//! glyph bounds, and the loaded-background `ScaleAndCrop` branch.

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
    text::{ComputedTextBlock, LineHeight, TextLayoutInfo},
    window::{PresentMode, WindowResolution},
};

pub use ffone_client::localization;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
enum PreviewLanguage {
    En,
    Ru,
}
impl PreviewLanguage {
    fn locale(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ru => "ru",
        }
    }
}

mod world_audio {
    #[derive(bevy::prelude::Resource, Default)]
    pub struct RetrobutionLoadingAudioState {
        pub active: bool,
    }
}

mod option_ui {
    use bevy::prelude::*;

    #[derive(Clone, Debug)]
    pub struct DisplaySettings {
        pub scale_ui: bool,
    }

    impl Default for DisplaySettings {
        fn default() -> Self {
            Self { scale_ui: true }
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct OptionSettings {
        pub display: DisplaySettings,
    }

    #[derive(Clone, Debug, Default, Resource)]
    pub struct OptionUiModel {
        pub visible: bool,
        pub persisted_options: OptionSettings,
    }
}

pub use ffone_client::text_edit;

#[allow(dead_code)]
#[path = "../../../src/ui/login/mod.rs"]
mod login_ui;

use localization::{Localization, LocalizationPlugin, LocalizationSet, LocalizedText};
use login_ui::{
    LOGIN_BACKGROUND_PATH, LOGIN_BUTTON_ACTIVE_PATH, LOGIN_BUTTON_OVER_PATH, LOGIN_BUTTON_PATH,
    LOGIN_CHALET_FONT_SIZE, LOGIN_CHALET_LINE_HEIGHT, LOGIN_FALLBACK_BACKGROUND_PATH,
    LOGIN_FONT_PATH, LOGIN_JEFFE_FONT_SIZE, LOGIN_JEFFE_LINE_HEIGHT, LOGIN_MUSIC_PATH,
    LOGIN_PANEL_PATH, LOGIN_PANEL_SIZE, LOGIN_TEXT_FIELD_FONT_PATH, LOGIN_TEXT_FIELD_PATH,
    LoginCommunityButton, LoginRegisterButton, LoginSubmitButton, LoginSurface, LoginTextStyle0104,
    LoginUiModel, LoginUiSet, NativeLoginUiPlugin,
};
use option_ui::OptionUiModel;

const CLIENT_AREA_WIDTH: u32 = 1_264;
const CLIENT_AREA_HEIGHT: u32 = 681;
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(25);
const MIN_BACKGROUND_DETAIL_PIXELS: usize = 250_000;
const MIN_PANEL_PIXELS: usize = 18_000;
const MIN_LIGHT_TEXT_PIXELS: usize = 450;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
enum PreviewStage {
    #[default]
    Manual,
    WarpShard,
    AutoLogin,
}

impl PreviewStage {
    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "manual" => Self::Manual,
            "warp" => Self::WarpShard,
            "auto" => Self::AutoLogin,
            _ => return None,
        })
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::WarpShard => "warp",
            Self::AutoLogin => "auto",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PreviewCli {
    output: PathBuf,
    language: PreviewLanguage,
    stage: PreviewStage,
}

fn parse_cli<I>(arguments: I) -> Result<PreviewCli, &'static str>
where
    I: IntoIterator<Item = std::ffi::OsString>,
{
    let mut arguments = arguments.into_iter();
    let stage = match arguments.next().and_then(|value| value.into_string().ok()) {
        None => PreviewStage::Manual,
        Some(value) => PreviewStage::parse(&value).ok_or("stage must be manual, warp, or auto")?,
    };
    let language = match arguments.next().and_then(|value| value.into_string().ok()) {
        None => PreviewLanguage::En,
        Some(value) if value.eq_ignore_ascii_case("en") => PreviewLanguage::En,
        Some(value) if value.eq_ignore_ascii_case("ru") => PreviewLanguage::Ru,
        Some(_) => return Err("language must be en or ru"),
    };
    let output = arguments.next().map(PathBuf::from).unwrap_or_else(|| {
        let language = match language {
            PreviewLanguage::En => "en",
            PreviewLanguage::Ru => "ru",
        };
        PathBuf::from(format!(
            "target/ui-parity/login-{}-{language}-1264x681.png",
            stage.slug()
        ))
    });
    if arguments.next().is_some()
        || output.extension().and_then(|value| value.to_str()) != Some("png")
    {
        return Err("usage: login_ui_gpu_preview [manual|warp|auto] [en|ru] [OUTPUT.png]");
    }
    Ok(PreviewCli {
        output,
        language,
        stage,
    })
}

#[derive(Resource)]
struct PreviewOutput(PathBuf);

#[derive(Resource)]
struct PreviewAssets {
    images: Vec<Handle<Image>>,
    fonts: Vec<Handle<Font>>,
    audio: Handle<AudioSource>,
    rich_background: Handle<Image>,
    fallback_background: Handle<Image>,
    panel: Handle<Image>,
    text_field: Handle<Image>,
    jeffe_font: Handle<Font>,
    chalet_font: Handle<Font>,
}

#[derive(Resource)]
struct PreviewState {
    frames: u32,
    ready_frame: Option<u32>,
    ready_at: Option<Instant>,
    layout_reported: bool,
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
            layout_reported: false,
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

    let (localization, language) = Localization::open(&asset_root, cli.language.locale()).unwrap();
    App::new()
        .insert_resource(localization)
        .insert_resource(language)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(PreviewOutput(cli.output))
        .insert_resource(cli.language)
        .insert_resource(cli.stage)
        .insert_resource(PreviewState::default())
        .init_resource::<OptionUiModel>()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne Retrobution Login acceptance".into(),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((NativeLoginUiPlugin, LocalizationPlugin))
        .add_systems(Startup, setup_preview)
        .add_systems(
            Update,
            drive_capture
                .after(LoginUiSet::Bind)
                .after(LocalizationSet::Apply),
        )
        .run();
}

fn setup_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    stage: Res<PreviewStage>,
    mut model: ResMut<LoginUiModel>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    model.visible = true;
    model.surface = match *stage {
        PreviewStage::Manual => LoginSurface::Manual,
        PreviewStage::WarpShard => LoginSurface::WarpShard,
        PreviewStage::AutoLogin => LoginSurface::AutoLogin,
    };
    model.username = env::var("FFONE_LOGIN_PREVIEW_TEXT").unwrap_or_else(|_| "Dexter".into());
    if env::var_os("FFONE_TEXT_SELECTION_PREVIEW").is_some() {
        model.username_edit.anchor = 0;
        model.username_edit.cursor = model.username.chars().count();
    }
    model.password = "nano".into();

    let rich_background = asset_server.load(LOGIN_BACKGROUND_PATH);
    let fallback_background = asset_server.load(LOGIN_FALLBACK_BACKGROUND_PATH);
    let panel = asset_server.load(LOGIN_PANEL_PATH);
    let text_field = asset_server.load(LOGIN_TEXT_FIELD_PATH);
    let jeffe_font = asset_server.load(LOGIN_FONT_PATH);
    let chalet_font = asset_server.load(LOGIN_TEXT_FIELD_FONT_PATH);
    commands.insert_resource(PreviewAssets {
        images: [
            LOGIN_BACKGROUND_PATH,
            LOGIN_FALLBACK_BACKGROUND_PATH,
            LOGIN_PANEL_PATH,
            LOGIN_BUTTON_PATH,
            LOGIN_BUTTON_OVER_PATH,
            LOGIN_BUTTON_ACTIVE_PATH,
            LOGIN_TEXT_FIELD_PATH,
        ]
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect(),
        fonts: [LOGIN_FONT_PATH, LOGIN_TEXT_FIELD_FONT_PATH]
            .into_iter()
            .map(|path| asset_server.load(path))
            .collect(),
        audio: asset_server.load(LOGIN_MUSIC_PATH),
        rich_background,
        fallback_background,
        panel,
        text_field,
        jeffe_font,
        chalet_font,
    });
}

#[allow(clippy::too_many_arguments)]
fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    fonts: Res<Assets<Font>>,
    preview_assets: Res<PreviewAssets>,
    language: Res<PreviewLanguage>,
    stage: Res<PreviewStage>,
    mut state: ResMut<PreviewState>,
    image_nodes: Query<(
        &ImageNode,
        &Node,
        &ComputedNode,
        &UiGlobalTransform,
        &InheritedVisibility,
    )>,
    all_texts: Query<&Text>,
    texts: Query<(
        Entity,
        &LocalizedText,
        &Text,
        &LoginTextStyle0104,
        (&TextFont, &LineHeight, &ComputedTextBlock),
        &TextLayoutInfo,
        &ComputedNode,
        &UiGlobalTransform,
        &InheritedVisibility,
    )>,
    edit_visuals: Query<&text_edit::EditVisual>,
    black_surfaces: Query<(&BackgroundColor, &ComputedNode, &InheritedVisibility)>,
    buttons: Query<(
        &ComputedNode,
        &UiGlobalTransform,
        Option<&LoginSubmitButton>,
        Option<&LoginCommunityButton>,
        Option<&LoginRegisterButton>,
    )>,
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
            .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
        || matches!(
            asset_server.load_state(preview_assets.audio.id()),
            LoadState::Failed(_)
        );
    if failed {
        eprintln!("Login acceptance asset failed to load");
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
            .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
        && matches!(
            asset_server.load_state(preview_assets.audio.id()),
            LoadState::Loaded
        );
    let cpu_assets_present = preview_assets
        .images
        .iter()
        .all(|handle| images.get(handle).is_some())
        && preview_assets
            .fonts
            .iter()
            .all(|handle| fonts.get(handle).is_some());
    let image_rect = |target: &Handle<Image>| {
        image_nodes
            .iter()
            .find(|(image, _, _, _, _)| image.image == *target)
            .map(|(_, node, computed, transform, visibility)| {
                (
                    transform.translation - computed.size() * 0.5,
                    computed.size(),
                    visibility.get() && node.display != Display::None,
                )
            })
    };
    let panel_rect = image_rect(&preview_assets.panel);
    let rich_rect = image_rect(&preview_assets.rich_background);
    let fallback_rect = image_rect(&preview_assets.fallback_background);
    let panel_exact = panel_rect.is_some_and(|(origin, size, visible)| {
        visible && origin == Vec2::new(447.0, 203.0) && size == LOGIN_PANEL_SIZE
    });
    let field_rects = image_nodes.iter()
        .filter(|(image, _, _, _, _)| image.image == preview_assets.text_field)
        .map(|(image, _, computed, transform, visibility)| {
            (transform.translation - computed.size() * 0.5, computed.size(),
             visibility.get(), image.visual_box)
        })
        .collect::<Vec<_>>();
    let fields_exact = field_rects.len() == 2
        && [Vec2::new(492.0, 243.0), Vec2::new(492.0, 303.0)].iter().all(|expected| {
            field_rects.iter().any(|(origin, size, visible, visual_box)| {
                *visible && *origin == *expected && *size == Vec2::new(280.0, 25.0)
                    && *visual_box == bevy::ui::VisualBox::BorderBox
            })
        });
    let rich_background_exact = rich_rect.is_some_and(|(origin, size, visible)| {
        visible
            && (origin - Vec2::new(0.0, -15.0)).abs().max_element() <= 0.01
            && (size - Vec2::new(1_264.0, 711.0)).abs().max_element() <= 0.01
    });
    let fallback_background_exact = fallback_rect.is_some_and(|(origin, size, visible)| {
        visible
            // Taffy resolves source y=7.3833/h=666.2333 to this GPU pixel box.
            && origin == Vec2::new(0.0, 7.0)
            && size == Vec2::new(1_264.0, 667.0)
    });
    let root_black_exact = black_surfaces.iter().any(|(color, computed, visibility)| {
        visibility.get() && color.0 == Color::BLACK && computed.size() == Vec2::new(1_264.0, 681.0)
    });
    let all_text_count = all_texts.iter().count();
    let owned_text_count = texts.iter().count();
    let mut visible_text_count = 0usize;
    let mut glyph_count = 0usize;
    let mut line_count = 0usize;
    let mut saw_cyrillic = false;
    let mut text_metrics_exact = all_text_count == 8 && owned_text_count == all_text_count;
    for (entity, localized, text, style, font, layout, computed, _, visibility) in &texts {
        if localized.key.trim().is_empty() {
            eprintln!("Text {entity:?} has no semantic localization key");
            text_metrics_exact = false;
        }
        let (expected_font, expected_size, expected_line_height) = match style {
            LoginTextStyle0104::TextField => (
                &preview_assets.chalet_font,
                LOGIN_CHALET_FONT_SIZE,
                LOGIN_CHALET_LINE_HEIGHT,
            ),
            LoginTextStyle0104::Label | LoginTextStyle0104::Button => (
                &preview_assets.jeffe_font,
                LOGIN_JEFFE_FONT_SIZE,
                LOGIN_JEFFE_LINE_HEIGHT,
            ),
            LoginTextStyle0104::StatusAdapter => {
                (&preview_assets.jeffe_font, 11.0, LOGIN_JEFFE_LINE_HEIGHT)
            }
        };
        text_metrics_exact &= font.0.font == expected_font.clone().into()
            && font.0.font_size.eval(Vec2::ZERO, 16.0) == expected_size
            && (*font.1) == LineHeight::Px(expected_line_height);
        if *stage != PreviewStage::Manual
            || !visibility.get()
            || text.0.is_empty()
            || layout.glyphs.is_empty()
        {
            continue;
        }
        visible_text_count += 1;
        glyph_count += layout.glyphs.len();
        line_count += layout.run_geometry.len();
        saw_cyrillic |= text
            .0
            .chars()
            .any(|character| ('\u{0400}'..='\u{052f}').contains(&character));
        let mut minimum = Vec2::splat(f32::INFINITY);
        let mut maximum = Vec2::splat(f32::NEG_INFINITY);
        for glyph in &layout.glyphs {
            minimum = minimum.min(glyph.position - glyph.atlas_info.rect.size() * 0.5);
            maximum = maximum.max(glyph.position + glyph.atlas_info.rect.size() * 0.5);
        }
        text_metrics_exact &= minimum.x >= -2.0
            && minimum.y >= -2.0
            && (*style == LoginTextStyle0104::TextField || maximum.x <= computed.size().x + 2.0)
            && maximum.y <= computed.size().y + 2.0;
        // Parley decoration bounds include ascent/descent even for negative leading.
        for shaped_line in font.2.buffer().lines() {
            let line = shaped_line.metrics();
            text_metrics_exact &= line.baseline.is_finite()
                && line.line_height.is_finite()
                && (line.line_height - expected_line_height).abs() <= 0.55;
        }
    }

    let expected = match *language {
        PreviewLanguage::En => [
            "Username :",
            "Password :",
            "Log In",
            "How Do I Register?",
        ],
        PreviewLanguage::Ru => [
            "ЛОГИН :",
            "ПАРОЛЬ :",
            "ВОЙТИ",
            "КАК ЗАРЕГИСТРИРОВАТЬСЯ?",
        ],
    };
    let localized_exact = expected.iter().all(|expected| {
        texts
            .iter()
            .any(|(_, _, text, _, _, _, _, _, _)| text.0.as_str() == *expected)
    });
    let fixed_baseline_layout_exact = [
        ("ui.login.username", 497.0, 697.0, 218.0, 20.0),
        ("ui.login.password", 497.0, 697.0, 278.0, 20.0),
        ("ui.login.submit", 492.0, 772.0, 347.0, 17.0),
    ]
    .into_iter()
    .all(
        |(key, control_left, control_right, line_top, line_height)| {
            texts
                .iter()
                .find(|(_, localized, _, _, _, _, _, _, _)| localized.key == key)
                .is_some_and(|(_, _, _, _, _, _, node, transform, _)| {
                    let origin = transform.translation - node.size() * 0.5;
                    (origin.y - line_top).abs() <= 0.01
                        && (node.size().y - line_height).abs() <= 0.01
                        && origin.x >= control_left
                        && origin.x + node.size().x <= control_right
                })
        },
    );
    let button_rect = |role: usize| {
        buttons
            .iter()
            .find(|(_, _, submit, community, register)| match role {
                0 => submit.is_some(),
                1 => community.is_some(),
                2 => register.is_some(),
                _ => false,
            })
            .map(|(node, transform, _, _, _)| {
                let origin = transform.translation - node.size() * 0.5;
                (origin.x, origin.y, node.size().x, node.size().y)
            })
    };
    // Native login actions form a 280x35 column with eight-pixel gaps.
    let lower_layout_exact = button_rect(1).is_none()
        && button_rect(2).is_some_and(|register| {
            (register.1 - 424.0).abs() <= 0.01
                && (register.3 - 35.0).abs() <= 0.01
                && (register.2 - 280.0).abs() <= 0.01
                && (register.0 + register.2 * 0.5 - 632.0).abs() <= 0.51
                && texts.iter()
                    .find(|(_, localized, _, _, _, _, _, _, _)| localized.key == "ui.login.register")
                    .is_some_and(|(_, _, _, _, _, _, node, transform, _)| {
                        let origin = transform.translation - node.size() * 0.5;
                        origin.x >= register.0
                            && origin.x + node.size().x <= register.0 + register.2
                            && origin.y >= register.1
                            && origin.y + node.size().y <= register.1 + register.3
                    })
        });
    let branch_exact = match *stage {
        PreviewStage::Manual => {
            panel_exact
                && fields_exact
                && rich_background_exact
                && fallback_rect.is_some_and(|(_, _, visible)| !visible)
                && visible_text_count == 7
                && localized_exact
                && fixed_baseline_layout_exact
                && lower_layout_exact
                && text_metrics_exact
                && (*language != PreviewLanguage::Ru || saw_cyrillic)
        }
        PreviewStage::WarpShard => {
            panel_rect.is_some_and(|(_, _, visible)| !visible)
                && rich_rect.is_some_and(|(_, _, visible)| !visible)
                && fallback_background_exact
                && visible_text_count == 0
                && text_metrics_exact
        }
        PreviewStage::AutoLogin => {
            panel_rect.is_some_and(|(_, _, visible)| !visible)
                && rich_rect.is_some_and(|(_, _, visible)| !visible)
                && fallback_rect.is_some_and(|(_, _, visible)| !visible)
                && root_black_exact
                && visible_text_count == 0
                && text_metrics_exact
        }
    };
    let ready = assets_loaded && cpu_assets_present && root_black_exact && branch_exact;
    if assets_loaded && branch_exact && !state.layout_reported {
        state.layout_reported = true;
        println!(
            "Login GPU audit: stage={:?}, locale={:?}, key-first={}/{}, visible={}, glyphs={}, line-boxes={}, panel={panel_rect:?}, loaded={rich_rect:?}, fallback={fallback_rect:?}",
            *stage,
            *language,
            owned_text_count,
            all_text_count,
            visible_text_count,
            glyph_count,
            line_count,
        );
        if *stage == PreviewStage::Manual {
            for (_, localized, text, style, _, _, node, transform, visibility) in &texts {
                if !visibility.get() {
                    continue;
                }
                if matches!(
                    localized.key.as_str(),
                    "ui.login.username"
                        | "ui.login.password"
                        | "ui.login.submit"
                        | "ui.login.discord"
                        | "ui.login.register"
                ) {
                    let line_box_origin = transform.translation - node.size() * 0.5;
                    println!(
                        "text-layout key={} style={style:?} value={:?} line-box=({:.1},{:.1},{:.1},{:.1})",
                        localized.key,
                        text.0,
                        line_box_origin.x,
                        line_box_origin.y,
                        node.size().x,
                        node.size().y,
                    );
                }
            }
            for (node, transform, submit, community, register) in &buttons {
                let role = if submit.is_some() {
                    "submit"
                } else if community.is_some() {
                    "community"
                } else if register.is_some() {
                    "register"
                } else {
                    continue;
                };
                let origin = transform.translation - node.size() * 0.5;
                println!(
                    "button-layout role={role} rect=({:.1},{:.1},{:.1},{:.1})",
                    origin.x,
                    origin.y,
                    node.size().x,
                    node.size().y,
                );
            }
        }
    }
    if ready && *stage == PreviewStage::Manual {
        for (entity, _, text, style, font, _, computed, transform, _) in &texts {
            if *style != LoginTextStyle0104::TextField {
                continue;
            }
            let visual = edit_visuals
                .get(entity)
                .expect("login fields own edit visuals");
            if visual.shaped_text != text.0 {
                continue;
            }
            let scale = computed.inverse_scale_factor().recip();
            let positions = text_edit::advances(font.2, &text.0, scale);
            for (index, x) in positions.iter().enumerate() {
                let local = computed.content_box().min + Vec2::new(x * scale, 0.0);
                let cursor = transform.transform_point2(local);
                assert_eq!(
                    text_edit::hit_position(
                        font.2, computed, transform, &text.0, visual, cursor, 0.0
                    ),
                    Some(index),
                    "input glyph boundary {index} must map back to its scalar cursor"
                );
            }
        }
    }
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
            "Login capture timed out: stage={:?}, assets={assets_loaded}, cpu={cpu_assets_present}, root-black={root_black_exact}, branch={branch_exact}, panel={panel_exact}/{panel_rect:?}, loaded={rich_background_exact}/{rich_rect:?}, fallback={fallback_background_exact}/{fallback_rect:?}, text={text_metrics_exact}, visible={visible_text_count}, localized={localized_exact}, fixed={fixed_baseline_layout_exact}, lower={lower_layout_exact}, cyrillic={saw_cyrillic}",
            *stage
        );
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

fn save_screenshot(
    event: On<ScreenshotCaptured>,
    output: Res<PreviewOutput>,
    stage: Res<PreviewStage>,
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
            "rejecting Login capture with unexpected size {:?}",
            rgba.dimensions()
        );
        state.capture_failed = true;
        return;
    }

    let mut background_detail = 0;
    let mut panel_pixels = 0;
    let mut light_text_pixels = 0;
    let mut non_black_pixels = 0;
    for (x, y, pixel) in rgba.enumerate_pixels() {
        let [red, green, blue, alpha] = pixel.0;
        if alpha > 0 && u16::from(red) + u16::from(green) + u16::from(blue) > 3 {
            non_black_pixels += 1;
        }
        if alpha > 0 && u16::from(red) + u16::from(green) + u16::from(blue) > 90 {
            background_detail += 1;
        }
        if alpha > 0
            && (440..825).contains(&x)
            && (195..490).contains(&y)
            && blue > red.saturating_add(12)
            && green > 55
        {
            panel_pixels += 1;
        }
        if alpha > 0
            && (480..790).contains(&x)
            && (210..465).contains(&y)
            && red > 175
            && green > 175
            && blue > 175
        {
            light_text_pixels += 1;
        }
    }
    let pixels_exact = match *stage {
        PreviewStage::Manual => {
            background_detail >= MIN_BACKGROUND_DETAIL_PIXELS
                && panel_pixels >= MIN_PANEL_PIXELS
                && light_text_pixels >= MIN_LIGHT_TEXT_PIXELS
        }
        PreviewStage::WarpShard => background_detail >= MIN_BACKGROUND_DETAIL_PIXELS,
        PreviewStage::AutoLogin => non_black_pixels == 0,
    };
    if !pixels_exact {
        eprintln!(
            "rejecting incomplete Login {:?} capture: background={background_detail}, panel={panel_pixels}, text={light_text_pixels}, non-black={non_black_pixels}",
            *stage
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
