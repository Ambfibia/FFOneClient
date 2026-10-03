//! Deterministic GPU acceptance frame for the clean-Retrobution system modal.
//!
//! The frame uses the measured 1264x681 Unity Web Player client area. Two
//! messages exercise the exact LIFO stack: both dialog boxes remain visible,
//! but content and the OKAY/CANCEL controls belong only to the newest entry.

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
    text::{LineHeight, TextLayoutInfo},
    window::{PresentMode, WindowResolution},
};
use ffone_client::localization::LocalizedText;
use ffone_client::{
    localization::{Localization, LocalizationPlugin},
    system_message_ui::{
        SYSTEM_MESSAGE_BLUE_BUTTON_OVER_PATH, SYSTEM_MESSAGE_BLUE_BUTTON_PATH,
        SYSTEM_MESSAGE_BODY_FONT_PATH, SYSTEM_MESSAGE_CANCEL_BUTTON_PATH,
        SYSTEM_MESSAGE_COMBI_ICON_PATH, SYSTEM_MESSAGE_COMBINED_BADGE_PATH,
        SYSTEM_MESSAGE_DIALOG_PATH, SYSTEM_MESSAGE_FONT_PATH, SYSTEM_MESSAGE_ITEM_BOX_PATH,
        SYSTEM_MESSAGE_RED_BUTTON_OVER_PATH, SYSTEM_MESSAGE_RED_BUTTON_PATH,
        SYSTEM_MESSAGE_WARNING_ICON_PATH, SystemMessageButtonType, SystemMessageFontRole,
        SystemMessageRequest, SystemMessageTextStyle, SystemMessageUiModel, SystemMessageUiPlugin,
        SystemMessageUiRoot,
    },
};

const CLIENT_AREA_WIDTH: u32 = 1_264;
const CLIENT_AREA_HEIGHT: u32 = 681;
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(20);
const MIN_CYAN_PIXELS: usize = 300;
const MIN_DIMMED_BACKGROUND_PIXELS: usize = 100_000;

const COMPARISON_LEFT_PATH: &str = "icons/items/weapons/wpnicon_01.png";
const COMPARISON_RIGHT_PATH: &str = "icons/items/weapons/wpnicon_02.png";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
enum PreviewMode {
    Generic,
    DeleteMission,
    CombinationFailure,
}

impl PreviewMode {
    const fn cli_name(self) -> &'static str {
        match self {
            Self::Generic => "generic",
            Self::DeleteMission => "delete-mission",
            Self::CombinationFailure => "combi-failure",
        }
    }

    const fn expected_text_count(self) -> usize {
        match self {
            Self::Generic | Self::CombinationFailure => 3,
            Self::DeleteMission => 6,
        }
    }

    const fn expected_button_count(self) -> usize {
        match self {
            Self::Generic | Self::DeleteMission => 2,
            Self::CombinationFailure => 1,
        }
    }

    fn default_output(self, language: &str) -> PathBuf {
        PathBuf::from(format!(
            "target/ui-parity/system-message-{}-{language}-1264x681.png",
            self.cli_name()
        ))
    }
}

#[derive(Resource)]
struct PreviewOutput(PathBuf);

#[derive(Resource)]
struct PreviewLanguage(String);

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
    let (mode, language_slug, output) =
        parse_preview_args(std::env::args_os().skip(1)).unwrap_or_else(|error| {
            eprintln!("{error}");
            eprintln!(
                "usage: system_message_gpu_preview [generic|delete-mission|combi-failure] \
                 [OUTPUT.png] [--language en|ru]"
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
        .insert_resource(ClearColor(Color::srgb(0.08, 0.22, 0.38)))
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
                        title: "FFOne Retrobution system message acceptance".into(),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((SystemMessageUiPlugin, LocalizationPlugin))
        .add_systems(Startup, setup_preview)
        .add_systems(Update, drive_capture)
        .run();
}

fn parse_preview_args(
    args: impl IntoIterator<Item = OsString>,
) -> Result<(PreviewMode, String, PathBuf), String> {
    let mut mode = PreviewMode::Generic;
    let mut language = "en".to_owned();
    let mut output = None;
    let mut mode_seen = false;
    let mut args = args.into_iter();
    while let Some(argument) = args.next() {
        if argument == "--language" {
            let value = args
                .next()
                .and_then(|value| value.into_string().ok())
                .ok_or_else(|| "--language requires en or ru".to_owned())?;
            if value != "en" && value != "ru" {
                return Err("--language requires en or ru".to_owned());
            }
            language = value;
        } else if !mode_seen && argument == "generic" {
            mode = PreviewMode::Generic;
            mode_seen = true;
        } else if !mode_seen && argument == "delete-mission" {
            mode = PreviewMode::DeleteMission;
            mode_seen = true;
        } else if !mode_seen && argument == "combi-failure" {
            mode = PreviewMode::CombinationFailure;
            mode_seen = true;
        } else if output.is_none() {
            output = Some(PathBuf::from(argument));
        } else {
            return Err("too many positional arguments".to_owned());
        }
    }
    let output = output.unwrap_or_else(|| mode.default_output(&language));
    if output.extension().and_then(|value| value.to_str()) != Some("png") {
        return Err(format!("output must be a .png path: {}", output.display()));
    }
    Ok((mode, language, output))
}

fn setup_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mode: Res<PreviewMode>,
    mut model: ResMut<SystemMessageUiModel>,
) {
    // The reusable modal intentionally owns no camera; the bounded harness
    // supplies the one default UI camera.
    commands.spawn((Camera2d, IsDefaultUiCamera));
    spawn_world_backdrop(&mut commands);

    model.push(SystemMessageRequest::new(
        1,
        "Older queued message",
        SystemMessageButtonType::Ok,
    ));
    let request = match *mode {
        PreviewMode::Generic => SystemMessageRequest::new_localized(
            2,
            LocalizedText::new("ui.loading.error", "LOADING STOPPED: {error}")
                .with_arg("error", "Scene 13"),
            SystemMessageButtonType::OkCancel,
        ),
        PreviewMode::DeleteMission => SystemMessageRequest::new_localized(
            2,
            LocalizedText::new(
                "ui.mission.delete.confirm",
                "Delete\n {mission}\nThis mission will no longer appear in your mission journal,\n\
                 but you can get it again later by visiting the mission giver.",
            )
            .with_arg("mission", "Nano Evolution"),
            SystemMessageButtonType::DeleteMission,
        ),
        PreviewMode::CombinationFailure => SystemMessageRequest::new_localized(
            2,
            LocalizedText::new(
                "ui.combi.modal.combination_failed",
                "OOPS!\nThe combination failed.  But you can always try again.  Your items are \
                 still available to combine, equip, sell, or trade.",
            ),
            SystemMessageButtonType::CombinationFailure,
        )
        .try_with_legacy_icon_index(7)
        .expect("clean serialized Combi icon index must remain renderable")
        .with_comparison_icons(COMPARISON_LEFT_PATH, true, COMPARISON_RIGHT_PATH, false),
    };
    model.push(request);

    commands.insert_resource(PreviewAssets {
        images: [
            SYSTEM_MESSAGE_DIALOG_PATH,
            SYSTEM_MESSAGE_BLUE_BUTTON_PATH,
            SYSTEM_MESSAGE_BLUE_BUTTON_OVER_PATH,
            SYSTEM_MESSAGE_RED_BUTTON_PATH,
            SYSTEM_MESSAGE_RED_BUTTON_OVER_PATH,
            SYSTEM_MESSAGE_CANCEL_BUTTON_PATH,
            SYSTEM_MESSAGE_ITEM_BOX_PATH,
            SYSTEM_MESSAGE_WARNING_ICON_PATH,
            SYSTEM_MESSAGE_COMBI_ICON_PATH,
            SYSTEM_MESSAGE_COMBINED_BADGE_PATH,
            COMPARISON_LEFT_PATH,
            COMPARISON_RIGHT_PATH,
        ]
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect(),
        fonts: vec![
            asset_server.load(SYSTEM_MESSAGE_FONT_PATH),
            asset_server.load(SYSTEM_MESSAGE_BODY_FONT_PATH),
        ],
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
            BackgroundColor(Color::srgb(0.18, 0.46, 0.68)),
            GlobalZIndex(-1_000),
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            for (left, top, width, height, color) in [
                (70.0, 70.0, 260.0, 160.0, Color::srgb(0.85, 0.42, 0.16)),
                (910.0, 90.0, 250.0, 190.0, Color::srgb(0.28, 0.78, 0.36)),
                (90.0, 465.0, 300.0, 135.0, Color::srgb(0.68, 0.24, 0.72)),
                (870.0, 450.0, 310.0, 145.0, Color::srgb(0.88, 0.75, 0.18)),
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

fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    fonts: Res<Assets<Font>>,
    preview_assets: Res<PreviewAssets>,
    mode: Res<PreviewMode>,
    language_slug: Res<PreviewLanguage>,
    mut state: ResMut<PreviewState>,
    roots: Query<&ComputedNode, With<SystemMessageUiRoot>>,
    buttons: Query<(), With<Button>>,
    all_texts: Query<&Text>,
    styled_texts: Query<(
        &Text,
        &LocalizedText,
        (&TextFont, &LineHeight),
        &TextLayout,
        &TextLayoutInfo,
        &SystemMessageTextStyle,
    )>,
    model: Res<SystemMessageUiModel>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);

    let asset_failed = preview_assets
        .images
        .iter()
        .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
        || preview_assets
            .fonts
            .iter()
            .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)));
    if asset_failed {
        eprintln!("system-message source asset failed to load");
        exit.write(AppExit::error());
        return;
    }

    let source_assets_loaded = preview_assets
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
    let modal_laid_out = roots.single().is_ok_and(|computed| {
        computed.size().x >= CLIENT_AREA_WIDTH as f32
            && computed.size().y >= CLIENT_AREA_HEIGHT as f32
    });
    let lifo_contract_visible =
        model.len() == 2 && buttons.iter().count() == mode.expected_button_count();
    let text_laid_out = all_texts.iter().count() == mode.expected_text_count()
        && styled_texts.iter().count() == mode.expected_text_count()
        && styled_texts
            .iter()
            .all(|(_, _, _, _, layout, _)| layout.size.x > 0.0 && layout.size.y > 0.0);
    if source_assets_loaded
        && cpu_assets_present
        && modal_laid_out
        && lifo_contract_visible
        && text_laid_out
        && !state.text_audited
    {
        match audit_text_contract(
            &mode,
            &language_slug.0,
            &preview_assets,
            &all_texts,
            &styled_texts,
        ) {
            Ok(summary) => {
                println!("system-message text audit: {summary}");
                state.text_audited = true;
            }
            Err(error) => {
                eprintln!("system-message text audit failed: {error}");
                state.capture_failed = true;
            }
        }
    }
    let ready = source_assets_loaded
        && cpu_assets_present
        && modal_laid_out
        && lifo_contract_visible
        && text_laid_out
        && state.text_audited;
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
    if !state.capture_issued && ready && warmed {
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
            "system-message capture timed out: mode={} assets={source_assets_loaded}/{cpu_assets_present} \
             modal={modal_laid_out} lifo={lifo_contract_visible} buttons={} texts={}/{} \
             expected_texts={} layout_sizes={:?}",
            mode.cli_name(),
            buttons.iter().count(),
            styled_texts.iter().count(),
            all_texts.iter().count(),
            mode.expected_text_count(),
            styled_texts
                .iter()
                .map(|(_, _, _, _, layout, style)| (*style, layout.size))
                .collect::<Vec<_>>()
        );
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

fn audit_text_contract(
    mode: &PreviewMode,
    language: &str,
    assets: &PreviewAssets,
    all_texts: &Query<'_, '_, &Text>,
    styled_texts: &Query<
        '_,
        '_,
        (
            &Text,
            &LocalizedText,
            (&TextFont, &LineHeight),
            &TextLayout,
            &TextLayoutInfo,
            &SystemMessageTextStyle,
        ),
    >,
) -> Result<String, String> {
    let all_count = all_texts.iter().count();
    let styled_count = styled_texts.iter().count();
    if all_count != mode.expected_text_count() || styled_count != all_count {
        return Err(format!(
            "every-Text ownership incomplete: styled={styled_count}, all={all_count}, expected={}",
            mode.expected_text_count()
        ));
    }
    let mut glyph_bounds = Vec2::ZERO;
    let mut saw_cyrillic = false;
    for (text, localized, font, layout, layout_info, style) in styled_texts.iter() {
        if text.0.is_empty() || localized.key.is_empty() {
            return Err("empty rendered copy or localization identity".to_owned());
        }
        let spec = style.spec();
        let expected_font = match spec.font_role {
            SystemMessageFontRole::Jeffe => &assets.fonts[0],
            SystemMessageFontRole::Chalet => &assets.fonts[1],
        };
        if font.0.font != bevy::text::FontSource::Handle(expected_font.clone())
            || font.0.font_size.eval(Vec2::ZERO, 16.0) != spec.font_size
            || (*font.1) != LineHeight::Px(spec.line_height)
            || layout.justify != spec.justify
            || layout.linebreak != spec.linebreak
        {
            return Err(format!("{:?} lost its source font/layout contract", style));
        }
        glyph_bounds.x = glyph_bounds.x.max(layout_info.size.x);
        glyph_bounds.y = glyph_bounds.y.max(layout_info.size.y);
        saw_cyrillic |= text
            .0
            .chars()
            .any(|character| ('А'..='я').contains(&character));
    }
    if language == "ru" && !saw_cyrillic {
        return Err("RU frame contains no Cyrillic glyphs".to_owned());
    }
    Ok(format!(
        "mode={} language={language} key_first={styled_count}/{all_count} max_glyph_bounds={:.1}x{:.1}",
        mode.cli_name(),
        glyph_bounds.x,
        glyph_bounds.y
    ))
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
    assert_eq!(image.width(), CLIENT_AREA_WIDTH);
    assert_eq!(image.height(), CLIENT_AREA_HEIGHT);
    let rgba = image.to_rgba8();

    let cyan_pixels = rgba
        .pixels()
        .filter(|pixel| {
            let [red, green, blue, _alpha] = pixel.0;
            blue > 110 && green > 80 && blue > red.saturating_add(30)
        })
        .count();
    let dimmed_background_pixels = rgba
        .pixels()
        .filter(|pixel| {
            let [red, green, blue, _alpha] = pixel.0;
            let total = u16::from(red) + u16::from(green) + u16::from(blue);
            total > 20 && total < 260
        })
        .count();
    if cyan_pixels < MIN_CYAN_PIXELS || dimmed_background_pixels < MIN_DIMMED_BACKGROUND_PIXELS {
        eprintln!(
            "rejecting incomplete system-message capture: cyan={cyan_pixels} \
             (min {MIN_CYAN_PIXELS}), dimmed={dimmed_background_pixels} \
             (min {MIN_DIMMED_BACKGROUND_PIXELS})"
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
    println!("{}", absolute_display(&output.0));
    state.capture_saved = true;
}

fn absolute_display(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}

#[cfg(test)]
mod tests;
