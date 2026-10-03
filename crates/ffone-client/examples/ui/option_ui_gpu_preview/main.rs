//! Deterministic 1264x681 GPU acceptance frames for all four reachable
//! clean-Retrobution OptionMode tabs.

use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use bevy::{
    asset::AssetPlugin,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    window::{PresentMode, WindowResolution},
};
use ffone_client::localization::{
    Language, Localization, LocalizationPlugin, LocalizedText, VoiceLanguage,
};
use ffone_client::option_ui::{
    BlockedPlayerRow, InputSettings, OPTION_REFERENCE_HEIGHT, OPTION_REFERENCE_WIDTH,
    OptionBuddySlot, OptionOpenAudioRoute, OptionPage, OptionSettings, OptionSocialDefaultsButton,
    OptionTab, OptionUiAssetGate, OptionUiModel, OptionUiOutbox, OptionUiPlugin, OptionUiRoot,
    OptionUiSet, OptionUiWindow, option_ui_layout,
};

const CLIENT_AREA_WIDTH: u32 = 1_264;
const CLIENT_AREA_HEIGHT: u32 = 681;
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(20);
const DEFAULT_SOCIAL_OUTPUT: &str = "target/ui-parity/option-social-1264x681.png";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
enum PreviewMode {
    Graphics,
    GameUi,
    Social,
    Controls,
}

impl PreviewMode {
    const fn tab(self) -> OptionTab {
        match self {
            Self::Graphics => OptionTab::Graphics,
            Self::GameUi => OptionTab::GameUi,
            Self::Social => OptionTab::Social,
            Self::Controls => OptionTab::Controls,
        }
    }

    const fn default_output(self) -> &'static str {
        match self {
            Self::Graphics => "target/ui-parity/option-graphics-1264x681.png",
            Self::GameUi => "target/ui-parity/option-game-ui-1264x681.png",
            Self::Social => DEFAULT_SOCIAL_OUTPUT,
            Self::Controls => "target/ui-parity/option-controls-1264x681.png",
        }
    }

    const fn cli_name(self) -> &'static str {
        match self {
            Self::Graphics => "graphics",
            Self::GameUi => "game-ui",
            Self::Social => "social",
            Self::Controls => "controls",
        }
    }
}

#[derive(Resource)]
struct PreviewOutput(PathBuf);

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
    let (mode, output) = parse_args(std::env::args_os().skip(1).collect());
    if output.extension().and_then(|value| value.to_str()) != Some("png") {
        eprintln!("usage: option_ui_gpu_preview [graphics|game-ui|social|controls] [OUTPUT.png]");
        std::process::exit(2);
    }

    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game")
        .canonicalize()
        .expect("assets/game must exist");

    let locale = env::var("FFONE_OPTION_PREVIEW_LOCALE").unwrap_or_else(|_| "en".into());
    let (localization, language) =
        Localization::open(&asset_root, &locale).expect("localization bundles");
    App::new()
        .insert_resource(localization)
        .insert_resource(language)
        .insert_resource(VoiceLanguage {
            requested: "en".into(),
            effective: "en".into(),
        })
        .insert_resource(ClearColor(Color::srgb(0.05, 0.13, 0.2)))
        .insert_resource(mode)
        .insert_resource(PreviewOutput(output))
        .insert_resource(PreviewState::default())
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: format!("FFOne Retrobution Option {} acceptance", mode.cli_name()),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((LocalizationPlugin, OptionUiPlugin))
        .add_systems(Startup, setup_preview)
        .add_systems(
            Update,
            force_defaults_hover
                .after(OptionUiSet::Interaction)
                .before(OptionUiSet::Visuals),
        )
        .add_systems(Update, drive_capture.after(OptionUiSet::Visuals))
        .run();
}

fn parse_args(args: Vec<std::ffi::OsString>) -> (PreviewMode, PathBuf) {
    let mut args = args.into_iter();
    let first = args.next();
    let first_text = first
        .as_ref()
        .and_then(|value| value.to_str())
        .map(str::to_owned);
    let (mode, explicit_output) = match first_text.as_deref() {
        None => (PreviewMode::Social, None),
        Some("graphics") => (PreviewMode::Graphics, args.next()),
        Some("game-ui") => (PreviewMode::GameUi, args.next()),
        Some("social") => (PreviewMode::Social, args.next()),
        Some("controls") => (PreviewMode::Controls, args.next()),
        Some(value) if value.ends_with(".png") => (PreviewMode::Social, first),
        _ => {
            eprintln!(
                "usage: option_ui_gpu_preview [graphics|game-ui|social|controls] [OUTPUT.png]"
            );
            std::process::exit(2);
        }
    };
    if args.next().is_some() {
        eprintln!("usage: option_ui_gpu_preview [graphics|game-ui|social|controls] [OUTPUT.png]");
        std::process::exit(2);
    }
    let output = explicit_output
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(mode.default_output()));
    (mode, output)
}

fn setup_preview(
    mut commands: Commands,
    mode: Res<PreviewMode>,
    mut model: ResMut<OptionUiModel>,
    mut outbox: ResMut<OptionUiOutbox>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    spawn_world_backdrop(&mut commands);

    model.open(
        OptionSettings::default(),
        InputSettings::default(),
        OptionOpenAudioRoute::default(),
        &mut outbox,
    );
    model.selected_tab = mode.tab();
    model.buddy_slots[1] = OptionBuddySlot {
        pc_uid: 5_412,
        blocked: true,
        name_check_flag: 1,
        first_name: "Dexter".into(),
        last_name: "Morgan".into(),
    };
    model.buddy_slots[8] = OptionBuddySlot {
        pc_uid: 7_777,
        blocked: true,
        name_check_flag: 0,
        first_name: "Hidden".into(),
        last_name: "UntilVerified".into(),
    };
    model.buddy_slots[23] = OptionBuddySlot {
        pc_uid: 9_100,
        blocked: true,
        name_check_flag: 1,
        first_name: "Ben".into(),
        last_name: "Tennyson".into(),
    };
    model.selected_blocked_slot = Some(8);
    model.draft_options.social.allow_trade_requests = false;
    model.draft_options.graphics.visibility = 0.8;
    model.draft_options.graphics.particle_level = 3;
    model.draft_options.sound.master.volume = 0.8;
    model.draft_options.sound.effects.enabled = false;
    model.draft_options.display.new_chat = false;
    model.draft_options.text_colors.general = 13;
    model.draft_input.camera_sensitivity = 7.0;
    model.draft_input.invert_y = true;
    match env::var("FFONE_OPTION_PREVIEW_DROPDOWN").as_deref() {
        Ok("text") => model.modal.translation_dropdown = true,
        Ok("voice") => model.modal.voice_dropdown = true,
        _ => {}
    }
    model.clean = false;
    outbox.clear();
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
        BackgroundColor(Color::srgb(0.1, 0.31, 0.47)),
        GlobalZIndex(-1_000),
        Pickable::IGNORE,
    ));
}

fn force_defaults_hover(mut buttons: Query<&mut Interaction, With<OptionSocialDefaultsButton>>) {
    for mut interaction in &mut buttons {
        *interaction = Interaction::Hovered;
    }
}

#[allow(clippy::too_many_arguments)]
fn drive_capture(
    mut commands: Commands,
    gate: Res<OptionUiAssetGate>,
    mode: Res<PreviewMode>,
    model: Res<OptionUiModel>,
    mut state: ResMut<PreviewState>,
    roots: Query<(&Visibility, &ComputedNode), With<OptionUiRoot>>,
    windows: Query<(&ComputedNode, &UiTransform), With<OptionUiWindow>>,
    pages: Query<(&OptionPage, &Node)>,
    texts: Query<(&Text, Option<&LocalizedText>)>,
    localization: Res<Localization>,
    language: Res<Language>,
    mut exit: MessageWriter<AppExit>,
    output: Res<PreviewOutput>,
    text_metrics: Query<(
        &Text,
        &TextFont,
        &bevy::text::TextLayoutInfo,
        &ComputedNode,
        &UiGlobalTransform,
    )>,
) {
    state.frames = state.frames.saturating_add(1);
    if gate.failed {
        eprintln!("OptionMode acceptance asset closure failed");
        exit.write(AppExit::error());
        return;
    }

    let layout = option_ui_layout(
        Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32),
        model.draft_options.display.scale_ui,
    );
    let root_exact = roots.single().is_ok_and(|(visibility, computed)| {
        *visibility == Visibility::Visible
            && computed.size() == Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32)
    });
    let window_exact = windows.single().is_ok_and(|(computed, transform)| {
        computed.size() == Vec2::new(OPTION_REFERENCE_WIDTH, OPTION_REFERENCE_HEIGHT)
            && transform.scale == Vec2::ONE
    });
    let pages_exact = pages
        .iter()
        .filter(|(_, node)| node.display == Display::Flex)
        .count()
        == 1
        && pages
            .iter()
            .any(|(page, node)| page.0 == mode.tab() && node.display == Display::Flex);
    let expected_rows = [
        BlockedPlayerRow {
            slot: 1,
            pc_uid: 5_412,
            display_name: "Dexter Morgan".into(),
        },
        BlockedPlayerRow {
            slot: 8,
            pc_uid: 7_777,
            display_name: "Player 7777".into(),
        },
        BlockedPlayerRow {
            slot: 23,
            pc_uid: 9_100,
            display_name: "Ben Tennyson".into(),
        },
    ];
    let page_model_exact = match *mode {
        PreviewMode::Graphics => {
            model.draft_options.graphics.visibility == 0.8
                && model.draft_options.graphics.particle_level == 3
                && model.draft_options.sound.master.volume == 0.8
                && !model.draft_options.sound.effects.enabled
        }
        PreviewMode::GameUi => {
            !model.draft_options.display.new_chat && model.draft_options.text_colors.general == 13
        }
        PreviewMode::Social => {
            model.blocked_projection() == expected_rows
                && model.selected_blocked_slot == Some(8)
                && !model.draft_options.social.allow_trade_requests
        }
        PreviewMode::Controls => {
            model.draft_input.camera_sensitivity == 7.0 && model.draft_input.invert_y
        }
    };
    let model_exact =
        model.visible && model.selected_tab == mode.tab() && page_model_exact && !model.clean;
    let common_text_exact = [
        "GRAPHICS & SOUND",
        "GAME UI",
        "SOCIAL",
        "CONTROLS",
        "APPLY CHANGE",
        "SAVE AND EXIT",
    ]
    .into_iter()
    .all(|expected| {
        texts.iter().any(|(text, localized)| {
            text.0 == expected
                || localized.is_some_and(|value| {
                    value.fallback == expected && text.0 == localization.text(&language, value)
                })
        })
    });
    let page_text_exact = expected_page_text(*mode).iter().all(|expected| {
        texts.iter().any(|(text, localized)| {
            text.0 == *expected
                || localized.is_some_and(|value| {
                    value.fallback == *expected && text.0 == localization.text(&language, value)
                })
        })
    });
    let text_exact = common_text_exact && page_text_exact;
    let ready = gate.ready
        && layout.scale == Vec2::ONE
        && layout.node_left == 122.0
        && layout.node_top == 21.5
        && root_exact
        && window_exact
        && pages_exact
        && model_exact
        && text_exact;
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
        let metrics: Vec<_> = text_metrics
            .iter()
            .map(|(text, font, layout, node, transform)| {
                let transform = bevy::math::Affine2::from(transform);
                serde_json::json!({"text": text.0, "fontSize": font.font_size.eval(Vec2::ZERO, 16.0),
                "layoutSize": layout.size.to_array(), "nodeSize": node.size().to_array(),
                "glyphMinX": layout.glyphs.iter().map(|glyph| glyph.position.x - glyph.atlas_info.rect.size().x * 0.5).reduce(f32::min),
                "glyphMaxX": layout.glyphs.iter().map(|glyph| glyph.position.x + glyph.atlas_info.rect.size().x * 0.5).reduce(f32::max),
                "center": transform.translation.to_array()})
            })
            .collect();
        if let Some(parent) = output.0.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(
            output.0.with_extension("text.json"),
            serde_json::to_vec_pretty(&metrics).unwrap(),
        )
        .unwrap();
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
            "OptionMode capture timed out: gate={:?} root={root_exact} \
             window={window_exact} pages={pages_exact} model={model_exact} text={text_exact}",
            *gate
        );
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

fn expected_page_text(mode: PreviewMode) -> &'static [&'static str] {
    match mode {
        PreviewMode::Graphics => &[
            "GRAPHICS",
            "SOUND",
            "DEFAULT GRAPHICS",
            "SCREEN MODE:",
            "DETAIL LEVEL:",
            "CUSTOM DETAIL LEVEL",
            "MASTER VOLUME:",
            "DEFAULT SOUND",
        ],
        PreviewMode::GameUi => &[
            "DISPLAY ELEMENTS",
            "CHAT TEXT COLORS",
            "OLD CHAT:",
            "ANIMATED NANOCOM:",
            "GENERAL CHAT",
            "DEFAULT DISPLAY ELEMENTS",
            "DEFAULT TEXT COLORS",
        ],
        PreviewMode::Social => &[
            "GAME LANGUAGE",
            "TEXT LANGUAGE:",
            "VOICE LANGUAGE:",
            "ALLOW REQUESTS",
            "BLOCKED PLAYERS",
            "GROUP REQUESTS:",
            "BUDDY REQUESTS:",
            "TRADE REQUESTS:",
            "DEFAULT REQUESTS",
            "Dexter Morgan",
            "Player 7777",
            "Ben Tennyson",
            "UNIGNORE",
        ],
        PreviewMode::Controls => &[
            "INPUT CONTROLS",
            "KEY MAPPING",
            "INVERT CAMERA Y-AXIS:",
            "Selected Gamepad :",
            "MOVEMENT",
            "SETTING 1",
            "FORWARD",
            "DEFAULT KEY MAPPING",
        ],
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
            "rejecting OptionMode capture with unexpected size {:?}",
            rgba.dimensions()
        );
        state.capture_failed = true;
        return;
    }

    let layout = option_ui_layout(
        Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32),
        true,
    );
    let x_start = layout.visual.x.max(0.0) as u32;
    let x_end = (layout.visual.x + layout.visual.width).min(CLIENT_AREA_WIDTH as f32) as u32;
    let y_start = layout.visual.y.max(0.0) as u32;
    let y_end = (layout.visual.y + layout.visual.height).min(CLIENT_AREA_HEIGHT as f32) as u32;
    let mut visible = 0usize;
    let mut cyan = 0usize;
    for y in y_start..y_end {
        for x in x_start..x_end {
            let [red, green, blue, alpha] = rgba.get_pixel(x, y).0;
            if alpha > 0 && u16::from(red) + u16::from(green) + u16::from(blue) > 55 {
                visible += 1;
            }
            if green > 90 && blue > 110 && blue > red.saturating_add(25) {
                cyan += 1;
            }
        }
    }
    if visible < 300_000 || cyan < 1_000 {
        eprintln!("rejecting incomplete OptionMode capture: visible={visible}, cyan={cyan}");
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
