//! Deterministic 1264x681 GPU acceptance capture for clean-Retrobution
//! Player User Store / street-stall mode 28.
//!
//! Usage: `user_store_ui_gpu_preview [MODE] [LANG] [OUTPUT.png]`. LANG is
//! `en` or `ru`; popup modes exercise the clean GumPopup presentation. The
//! preview consumes native assets and production localization bundles only;
//! it never reads a Unity container at runtime.

#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

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
    text::{ComputedTextBlock, LineHeight, TextLayoutInfo},
    window::{PresentMode, WindowResolution},
};
use serde::Deserialize;

#[path = "../../../src/ui/startup/mod.rs"]
mod ui_startup;

pub use ffone_client::localization;

#[allow(dead_code)]
#[path = "../../../src/ui/user_store/mod.rs"]
mod user_store_ui;

use localization::{Localization, LocalizationPlugin, LocalizedText};
use user_store_ui::{
    USER_STORE_BODY_FONT_PATH, USER_STORE_CHALET_SMALL_FONT_SIZE,
    USER_STORE_CHALET_SMALL_LINE_HEIGHT, USER_STORE_FONT_PATH, USER_STORE_IMAGE_ASSET_PATHS,
    USER_STORE_JEFFE_12_FONT_SIZE, USER_STORE_JEFFE_12_LINE_HEIGHT, USER_STORE_JEFFE_14_FONT_SIZE,
    USER_STORE_JEFFE_14_LINE_HEIGHT, USER_STORE_JEFFE_16_FONT_SIZE,
    USER_STORE_JEFFE_16_LINE_HEIGHT, USER_STORE_POPUP_RECT, UserStoreAuthority0104,
    UserStoreItemCatalog0104, UserStorePopupPresentation0104, UserStorePopupTextStyle0104,
    UserStorePreviewMode0104, UserStoreStaticAssetReadiness0104, UserStoreUiAssetStatus0104,
    UserStoreUiElement0104, UserStoreUiPlugin0104, UserStoreUiProjection0104, UserStoreUiSet0104,
    UserStoreUiState0104, UserStoreUiTextRole0104, seed_user_store_popup_preview_0104,
    seed_user_store_preview_0104,
};

const CLIENT_AREA_WIDTH: u32 = 1_264;
const CLIENT_AREA_HEIGHT: u32 = 681;
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(25);
const MIN_VISIBLE_PIXELS: usize = 180_000;
const MIN_CYAN_PANEL_PIXELS: usize = 500;

#[derive(Clone, Debug, Eq, PartialEq)]
struct PreviewCli {
    mode: UserStorePreviewMode0104,
    language: String,
    output: PathBuf,
}

fn default_output(mode: UserStorePreviewMode0104, language: &str) -> PathBuf {
    PathBuf::from(format!(
        "target/ui-parity/user-store-{}-{}-1264x681.png",
        mode.as_str(),
        language
    ))
}

fn parse_cli<I>(arguments: I) -> Result<PreviewCli, &'static str>
where
    I: IntoIterator<Item = std::ffi::OsString>,
{
    let arguments = arguments
        .into_iter()
        .map(|value| value.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let usage = "usage: user_store_ui_gpu_preview [MODE] [en|ru] [OUTPUT.png]";
    let default_mode = UserStorePreviewMode0104::PopupQuantity;
    let parse_mode = |value: &str| UserStorePreviewMode0104::parse(value).ok_or(usage);
    let validate_output = |value: &str| {
        let output = PathBuf::from(value);
        (output.extension().and_then(|value| value.to_str()) == Some("png"))
            .then_some(output)
            .ok_or(usage)
    };
    match arguments.as_slice() {
        [] => Ok(PreviewCli {
            mode: default_mode,
            language: "en".to_owned(),
            output: default_output(default_mode, "en"),
        }),
        [one] if one.ends_with(".png") => Ok(PreviewCli {
            mode: default_mode,
            language: "en".to_owned(),
            output: validate_output(one)?,
        }),
        [mode] => {
            let mode = parse_mode(mode)?;
            Ok(PreviewCli {
                mode,
                language: "en".to_owned(),
                output: default_output(mode, "en"),
            })
        }
        [mode, language] if matches!(language.as_str(), "en" | "ru") => {
            let mode = parse_mode(mode)?;
            Ok(PreviewCli {
                mode,
                language: language.clone(),
                output: default_output(mode, language),
            })
        }
        [mode, output] => Ok(PreviewCli {
            mode: parse_mode(mode)?,
            language: "en".to_owned(),
            output: validate_output(output)?,
        }),
        [mode, language, output] if matches!(language.as_str(), "en" | "ru") => Ok(PreviewCli {
            mode: parse_mode(mode)?,
            language: language.clone(),
            output: validate_output(output)?,
        }),
        _ => Err(usage),
    }
}

#[derive(Resource)]
struct PreviewOutput(PathBuf);

#[derive(Resource)]
struct PreviewMode(UserStorePreviewMode0104);

#[derive(Resource)]
struct PreviewStaticAssets {
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
    text_audited: bool,
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
            text_audited: false,
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
    let (localization, language) = Localization::open(&asset_root, &cli.language)
        .expect("open production localization bundles");

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(localization)
        .insert_resource(language)
        .insert_resource(PreviewOutput(cli.output))
        .insert_resource(PreviewMode(cli.mode))
        .insert_resource(PreviewState::default())
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: format!(
                            "FFOne Retrobution User Store: {} ({})",
                            cli.mode.as_str(),
                            cli.language
                        ),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((LocalizationPlugin, UserStoreUiPlugin0104))
        .add_systems(Startup, setup_preview)
        .add_systems(Update, drive_capture.after(UserStoreUiSet0104::Bind))
        .run();
}

fn setup_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mode: Res<PreviewMode>,
    mut state: ResMut<UserStoreUiState0104>,
    mut authority: ResMut<UserStoreAuthority0104>,
    mut catalog: ResMut<UserStoreItemCatalog0104>,
    mut popup: ResMut<UserStorePopupPresentation0104>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    seed_user_store_preview_0104(mode.0, &mut state, &mut authority, &mut catalog);
    seed_user_store_popup_preview_0104(mode.0, &mut state, &authority, &mut popup);
    commands.insert_resource(PreviewStaticAssets {
        images: USER_STORE_IMAGE_ASSET_PATHS
            .iter()
            .map(|path| asset_server.load(*path))
            .collect(),
        fonts: [
            asset_server.load(USER_STORE_FONT_PATH),
            asset_server.load(USER_STORE_BODY_FONT_PATH),
        ],
    });
}

#[allow(clippy::too_many_arguments)]
fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    preview_assets: Res<PreviewStaticAssets>,
    mode: Res<PreviewMode>,
    language: Res<localization::Language>,
    localization: Res<localization::Localization>,
    asset_status: Res<UserStoreUiAssetStatus0104>,
    projection: Res<UserStoreUiProjection0104>,
    popup: Res<UserStorePopupPresentation0104>,
    mut state: ResMut<PreviewState>,
    elements: Query<(&UserStoreUiElement0104, &ComputedNode, &Node)>,
    texts: Query<(&UserStoreUiTextRole0104, &Text)>,
    all_texts: Query<(&Text, &LocalizedText)>,
    popup_texts: Query<(
        Entity,
        &Text,
        &LocalizedText,
        &UserStorePopupTextStyle0104,
        (&TextFont, &LineHeight, &ComputedTextBlock),
        &TextLayoutInfo,
        &ComputedNode,
        &UiTransform,
        &ChildOf,
    )>,
    computed_nodes: Query<&ComputedNode>,
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
        || asset_status.0 == UserStoreStaticAssetReadiness0104::Failed;
    if failed {
        eprintln!("User Store acceptance asset failed to load");
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
        && asset_status.0 == UserStoreStaticAssetReadiness0104::Ready;
    let root_exact = elements.iter().any(|(element, computed, node)| {
        *element == UserStoreUiElement0104::Root
            && node.display == Display::Flex
            && computed.size() == Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32)
    });
    let panel_exact = elements.iter().any(|(element, computed, node)| {
        *element == UserStoreUiElement0104::StorePanel
            && computed.size() == Vec2::new(498.0, 638.0)
            && node.left == px(122)
            && node.top == px(21)
    });
    let rows_exact = projection.listing_rows.len()
        == match mode.0 {
            UserStorePreviewMode0104::MyReady
            | UserStorePreviewMode0104::MyItems
            | UserStorePreviewMode0104::MyOpen
            | UserStorePreviewMode0104::PopupQuantity
            | UserStorePreviewMode0104::PopupPrice
            | UserStorePreviewMode0104::PopupUnregister
            | UserStorePreviewMode0104::Busy
            | UserStorePreviewMode0104::Error => 5,
            UserStorePreviewMode0104::UserList
            | UserStorePreviewMode0104::UserSold
            | UserStorePreviewMode0104::PopupBuy => 3,
        };
    let popup_exact = if let Some(contract) = popup.popup {
        let expected = user_store_ui::user_store_popup_rect_0104(
            CLIENT_AREA_WIDTH,
            CLIENT_AREA_HEIGHT,
            contract,
        );
        elements.iter().any(|(element, computed, node)| {
            *element == UserStoreUiElement0104::Popup
                && computed.size()
                    == Vec2::new(USER_STORE_POPUP_RECT.width, USER_STORE_POPUP_RECT.height)
                && node.display == Display::Flex
                && node.left == px(expected.left)
                && node.top == px(expected.top)
        })
    } else {
        true
    };
    let mode_text_exact = match mode.0 {
        UserStorePreviewMode0104::Busy => texts.iter().any(|(role, text)| {
            *role == UserStoreUiTextRole0104::Busy
                && text.0
                    == localization
                        .text(&language, &LocalizedText::new("ui.user_store.busy", "BUSY"))
        }),
        UserStorePreviewMode0104::Error => texts.iter().any(|(role, text)| {
            *role == UserStoreUiTextRole0104::Error
                && text.0
                    == localization.text(
                        &language,
                        &LocalizedText::new("ui.user_store.error", "STORE ERROR {code}")
                            .with_arg("code", "13"),
                    )
        }),
        UserStorePreviewMode0104::MyOpen => texts.iter().any(|(role, text)| {
            *role == UserStoreUiTextRole0104::PrimaryButton
                && text.0
                    == localization.text(
                        &language,
                        &LocalizedText::new("ui.user_store.close_store", "CLOSE STORE"),
                    )
        }),
        UserStorePreviewMode0104::PopupQuantity
        | UserStorePreviewMode0104::PopupPrice
        | UserStorePreviewMode0104::PopupUnregister
        | UserStorePreviewMode0104::PopupBuy => {
            let expected = match (mode.0, language.effective.as_str()) {
                (UserStorePreviewMode0104::PopupQuantity, "ru") => "ДОБАВИТЬ",
                (UserStorePreviewMode0104::PopupQuantity, _) => "ADD",
                (UserStorePreviewMode0104::PopupPrice, "ru") => "Таро",
                (UserStorePreviewMode0104::PopupPrice, _) => "Taros",
                (UserStorePreviewMode0104::PopupUnregister, "ru") => "УБРАТЬ ИЗ МАГАЗИНА",
                (UserStorePreviewMode0104::PopupUnregister, _) => "REMOVE FROM STORE",
                (UserStorePreviewMode0104::PopupBuy, "ru") => "КУПИТЬ",
                (UserStorePreviewMode0104::PopupBuy, _) => "BUY",
                _ => unreachable!(),
            };
            texts.iter().any(|(role, text)| {
                *role == UserStoreUiTextRole0104::PopupAction && text.0 == expected
            })
        }
        _ => true,
    };
    let text_audit = audit_user_store_text_0104(
        &all_texts,
        &popup_texts,
        &computed_nodes,
        &preview_assets,
        popup.popup.is_some(),
        &language.effective,
    );
    if assets_loaded && !state.text_audited {
        match &text_audit {
            Ok(report) => {
                println!("{report}");
                state.text_audited = true;
            }
            Err(error) if state.frames % 120 == 0 => {
                eprintln!("UserStore text audit waiting: {error}")
            }
            Err(_) => {}
        }
    }
    let ready = assets_loaded
        && root_exact
        && panel_exact
        && rows_exact
        && popup_exact
        && mode_text_exact
        && text_audit.is_ok();
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
        eprintln!("User Store capture timed out");
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

fn audit_user_store_text_0104(
    all_texts: &Query<'_, '_, (&Text, &LocalizedText)>,
    popup_texts: &Query<
        '_,
        '_,
        (
            Entity,
            &Text,
            &LocalizedText,
            &UserStorePopupTextStyle0104,
            (&TextFont, &LineHeight, &ComputedTextBlock),
            &TextLayoutInfo,
            &ComputedNode,
            &UiTransform,
            &ChildOf,
        ),
    >,
    computed_nodes: &Query<'_, '_, &ComputedNode>,
    assets: &PreviewStaticAssets,
    popup_visible: bool,
    language: &str,
) -> Result<String, String> {
    let all_rows = all_texts.iter().collect::<Vec<_>>();
    if all_rows.is_empty() {
        return Err("UserStore has no Text entities".to_owned());
    }
    if let Some((_, localized)) = all_rows
        .iter()
        .find(|(_, localized)| localized.key.trim().is_empty())
    {
        return Err(format!("empty localization key on {}", localized.fallback));
    }
    let styled_count = popup_texts.iter().count();
    if styled_count != 17 {
        return Err(format!(
            "GumPopup style ownership incomplete: styled={styled_count}, expected=17"
        ));
    }
    if !popup_visible {
        return Ok(format!(
            "UserStore GPU text audit: locale={language}, key-first={}",
            all_rows.len()
        ));
    }

    let mut visible = 0usize;
    let mut glyphs = 0usize;
    let mut lines = 0usize;
    let mut fitted = 0usize;
    let mut styles = Vec::new();
    let mut saw_cyrillic = false;
    let mut glyph_min_y = f32::INFINITY;
    let mut glyph_max_y = f32::NEG_INFINITY;
    for (entity, text, localized, style, font, layout, computed, transform, parent) in popup_texts {
        let (font_index, maximum_size, maximum_line_height) = expected_popup_text_metrics(*style);
        if font.0.font != bevy::text::FontSource::Handle(assets.fonts[font_index].clone()) {
            return Err(format!(
                "{} uses wrong replacement font for {style:?}",
                localized.key
            ));
        }
        let minimum_size = (maximum_size * 0.7).max(6.0).min(maximum_size);
        if font.0.font_size.eval(Vec2::ZERO, 16.0) < minimum_size - 0.01
            || font.0.font_size.eval(Vec2::ZERO, 16.0) > maximum_size + 0.01
        {
            return Err(format!(
                "{} font size {} outside {minimum_size}..={maximum_size}",
                localized.key,
                font.0.font_size.eval(Vec2::ZERO, 16.0)
            ));
        }
        fitted += usize::from(font.0.font_size.eval(Vec2::ZERO, 16.0) + 0.01 < maximum_size);
        let expected_line_height =
            maximum_line_height * font.0.font_size.eval(Vec2::ZERO, 16.0) / maximum_size;
        match *font.1 {
            LineHeight::Px(actual) if (actual - expected_line_height).abs() <= 0.02 => {}
            actual => {
                return Err(format!(
                    "{} line height {actual:?} != {expected_line_height}",
                    localized.key
                ));
            }
        }
        if transform.translation.y != px(0) {
            return Err(format!(
                "{} has unexpected replacement baseline offset {:?}",
                localized.key, transform.translation.y
            ));
        }
        if computed.size().x <= 0.0
            || computed.size().y <= 0.0
            || text.0.is_empty()
            || layout.glyphs.is_empty()
        {
            continue;
        }
        visible += 1;
        glyphs += layout.glyphs.len();
        lines += layout.run_geometry.len();
        saw_cyrillic |= text.0.chars().any(|character| {
            ('\u{0400}'..='\u{052f}').contains(&character)
                || ('\u{2de0}'..='\u{2dff}').contains(&character)
        });
        if !styles.contains(style) {
            styles.push(*style);
        }
        let parent_computed = computed_nodes
            .get(parent.parent())
            .map_err(|_| format!("Text {entity:?} has no source-Rect parent"))?;
        if computed.size().x > parent_computed.size().x + 1.0
            || computed.size().y > parent_computed.size().y + 1.0
        {
            return Err(format!(
                "{} text node {:?} exceeds source Rect {:?}",
                localized.key,
                computed.size(),
                parent_computed.size()
            ));
        }
        if !layout.size.is_finite()
            || layout.size.x > computed.size().x + 1.0
            || layout.size.y > computed.size().y + 1.0
        {
            return Err(format!(
                "{} GPU layout {:?} exceeds text node {:?}",
                localized.key,
                layout.size,
                computed.size()
            ));
        }
        let mut minimum = Vec2::splat(f32::INFINITY);
        let mut maximum = Vec2::splat(f32::NEG_INFINITY);
        for glyph in &layout.glyphs {
            minimum = minimum.min(glyph.position - glyph.atlas_info.rect.size() * 0.5);
            maximum = maximum.max(glyph.position + glyph.atlas_info.rect.size() * 0.5);
        }
        if minimum.x < -2.0
            || minimum.y < -2.0
            || maximum.x > computed.size().x + 2.0
            || maximum.y > computed.size().y + 2.0
        {
            return Err(format!(
                "{} glyph bounds {:?}..{:?} exceed text node {:?}",
                localized.key,
                minimum,
                maximum,
                computed.size()
            ));
        }
        glyph_min_y = glyph_min_y.min(minimum.y);
        glyph_max_y = glyph_max_y.max(maximum.y);
        // Parley decoration bounds include ascent/descent even for negative leading.
        for shaped_line in font.2.buffer().lines() {
            let line = shaped_line.metrics();
            if !line.baseline.is_finite()
                || !line.line_height.is_finite()
                || (line.line_height - expected_line_height).abs() > 0.75
            {
                return Err(format!(
                    "{} invalid baseline/line box {:?} for {expected_line_height}",
                    localized.key, line
                ));
            }
        }
    }
    if visible == 0 || glyphs == 0 || lines == 0 {
        return Err("visible GumPopup GPU text has not been laid out yet".to_owned());
    }
    for required in [
        UserStorePopupTextStyle0104::LabelUpperLeft,
        UserStorePopupTextStyle0104::LabelMiddleRight,
        UserStorePopupTextStyle0104::CenterLabel,
        UserStorePopupTextStyle0104::CalculatorButton,
        UserStorePopupTextStyle0104::Button,
    ] {
        if !styles.contains(&required) {
            return Err(format!("visible GumPopup has no {required:?} text"));
        }
    }
    if language == "ru" && !saw_cyrillic {
        return Err("RU GumPopup produced no visible Cyrillic glyphs".to_owned());
    }
    styles.sort_by_key(|style| format!("{style:?}"));
    Ok(format!(
        "UserStore GPU text audit: locale={language}, key-first={}, popup-styled={styled_count}, \
         visible={visible}, fitted={fitted}, glyphs={glyphs}, line-boxes={lines}, \
         glyph-y={glyph_min_y:.1}..{glyph_max_y:.1}, styles={styles:?}",
        all_rows.len()
    ))
}

fn expected_popup_text_metrics(style: UserStorePopupTextStyle0104) -> (usize, f32, f32) {
    match style {
        UserStorePopupTextStyle0104::LabelUpperLeft
        | UserStorePopupTextStyle0104::LabelMiddleRight => (
            0,
            USER_STORE_JEFFE_12_FONT_SIZE,
            USER_STORE_JEFFE_12_LINE_HEIGHT,
        ),
        UserStorePopupTextStyle0104::CenterLabel => (
            1,
            USER_STORE_CHALET_SMALL_FONT_SIZE,
            USER_STORE_CHALET_SMALL_LINE_HEIGHT,
        ),
        UserStorePopupTextStyle0104::CalculatorButton => (
            0,
            USER_STORE_JEFFE_16_FONT_SIZE,
            USER_STORE_JEFFE_16_LINE_HEIGHT,
        ),
        UserStorePopupTextStyle0104::Button => (
            0,
            USER_STORE_JEFFE_14_FONT_SIZE,
            USER_STORE_JEFFE_14_LINE_HEIGHT,
        ),
    }
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
            "rejecting User Store capture with size {:?}",
            rgba.dimensions()
        );
        state.capture_failed = true;
        return;
    }
    let mut visible = 0;
    let mut cyan_panel = 0;
    for (x, y, pixel) in rgba.enumerate_pixels() {
        let [red, green, blue, alpha] = pixel.0;
        if alpha > 0 && u16::from(red) + u16::from(green) + u16::from(blue) > 80 {
            visible += 1;
        }
        if alpha > 0
            && (100..1_160).contains(&x)
            && (10..670).contains(&y)
            && green > 90
            && blue > 100
            && blue > red.saturating_add(12)
        {
            cyan_panel += 1;
        }
    }
    if visible < MIN_VISIBLE_PIXELS || cyan_panel < MIN_CYAN_PANEL_PIXELS {
        eprintln!("rejecting incomplete User Store capture: visible={visible}, cyan={cyan_panel}");
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
