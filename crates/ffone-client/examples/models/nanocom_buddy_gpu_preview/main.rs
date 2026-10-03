//! Deterministic 1264x681 GPU acceptance harness for every production-reached
//! clean NanoCom presentation: passive type 9, Buddy type 13, and Group type
//! 14. Types 10/12/15/16 deliberately have no preview state because the
//! native production client has no producer for them.

#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

#[path = "../../../tests/nanocom_message_ui_standalone/main.rs"]
mod nanocom_standalone;

pub mod localization {
    pub use crate::nanocom_standalone::localization::*;
}

pub mod semantic_audio {
    pub use crate::nanocom_standalone::semantic_audio::*;
}

pub mod ui_startup {
    pub use crate::nanocom_standalone::ui_startup::*;
}

use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use bevy::{
    asset::{AssetPlugin, LoadState},
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    text::{LineHeight, TextLayoutInfo},
    time::Virtual,
    window::{PresentMode, WindowResolution},
};
use nanocom_standalone::{
    localization::{Localization, LocalizationPlugin, LocalizedText},
    nanocom_message_ui::{
        NANOCOM_ACCEPT_LOCALIZATION_KEY, NANOCOM_BLUE_BUTTON_OVER_PATH, NANOCOM_BLUE_BUTTON_PATH,
        NANOCOM_BUDDY_FRAME_PATH, NANOCOM_BUDDY_ICON_PATH, NANOCOM_CHALET_FONT_PATH,
        NANOCOM_COMPACT_BODY_LOCALIZATION_KEY, NANOCOM_COMPACT_TITLE_LOCALIZATION_KEY,
        NANOCOM_DECLINE_LOCALIZATION_KEY, NANOCOM_DIALOG_PATH,
        NANOCOM_EXPANDED_TITLE_LOCALIZATION_KEY, NANOCOM_EXPIRATION_LOCALIZATION_KEY,
        NANOCOM_GROUP_COMPACT_BODY_LOCALIZATION_KEY, NANOCOM_GROUP_COMPACT_TITLE_LOCALIZATION_KEY,
        NANOCOM_GROUP_EXPANDED_TITLE_LOCALIZATION_KEY, NANOCOM_GROUP_EXPIRATION_LOCALIZATION_KEY,
        NANOCOM_GROUP_ICON_PATH, NANOCOM_GROUP_INVITATION_LOCALIZATION_KEY,
        NANOCOM_INVITATION_LOCALIZATION_KEY, NANOCOM_JEFFE_FONT_PATH, NANOCOM_MESSAGE_AREA_PATH,
        NANOCOM_NUMBUH_TWO_ICON_PATH, NANOCOM_RED_BUTTON_OVER_PATH, NANOCOM_RED_BUTTON_PATH,
        NANOCOM_TYPE_9_FRAME_PATH, NanocomGuiStyleRole, NanocomMessageCompactPanel,
        NanocomMessageExpandedDialog, NanocomMessageKind, NanocomMessageRequest,
        NanocomMessageUiModel, NanocomMessageUiPlugin, nanocom_gui_style,
    },
};

const CLIENT_AREA_WIDTH: u32 = 1_264;
const CLIENT_AREA_HEIGHT: u32 = 681;
const WARMUP_FRAMES_AFTER_LOAD: u32 = 90;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(2);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(20);
const NANOCOM_COMPUTRESS_ICON_PATH: &str = "ui/en/gameplay/guide/compu_icon.png";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum PreviewStage {
    NpcCompact,
    RecallCompact,
    ComputressCompact,
    #[default]
    BuddyCompact,
    BuddyModal,
    GroupCompact,
    GroupModal,
}

impl PreviewStage {
    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "npc-compact" => Self::NpcCompact,
            "recall-compact" => Self::RecallCompact,
            "computress-compact" => Self::ComputressCompact,
            "buddy-compact" => Self::BuddyCompact,
            "buddy-modal" => Self::BuddyModal,
            "group-compact" => Self::GroupCompact,
            "group-modal" => Self::GroupModal,
            _ => return None,
        })
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::NpcCompact => "npc-compact",
            Self::RecallCompact => "recall-compact",
            Self::ComputressCompact => "computress-compact",
            Self::BuddyCompact => "buddy-compact",
            Self::BuddyModal => "buddy-modal",
            Self::GroupCompact => "group-compact",
            Self::GroupModal => "group-modal",
        }
    }

    const fn is_modal(self) -> bool {
        matches!(self, Self::BuddyModal | Self::GroupModal)
    }

    const fn expected_kind(self) -> NanocomMessageKind {
        match self {
            Self::NpcCompact | Self::ComputressCompact | Self::RecallCompact => {
                NanocomMessageKind::Npc
            }
            Self::BuddyCompact | Self::BuddyModal => NanocomMessageKind::BuddyInvite,
            Self::GroupCompact | Self::GroupModal => NanocomMessageKind::GroupInvite,
        }
    }
}

#[derive(Resource)]
struct PreviewOptions {
    output: PathBuf,
    language: String,
    stage: PreviewStage,
}

#[derive(Resource)]
struct PreviewConfig {
    output: PathBuf,
    language: String,
    stage: PreviewStage,
    images: Vec<Handle<Image>>,
    fonts: Vec<Handle<Font>>,
    jeffe: Handle<Font>,
    chalet: Handle<Font>,
}

#[derive(Resource)]
struct PreviewState {
    frames: u32,
    ready_frame: Option<u32>,
    ready_at: Option<Instant>,
    capture_issued: bool,
    capture_saved: bool,
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
            text_audited: false,
            started_at: Instant::now(),
        }
    }
}

fn main() {
    let options = parse_options();
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game")
        .canonicalize()
        .expect("assets/game must exist");
    let (localization, language) =
        Localization::open(&asset_root, &options.language).expect("open production localization");

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.08, 0.22, 0.38)))
        .insert_resource(PreviewState::default())
        .insert_resource(options)
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
                        title: "FFOne clean NanoCom acceptance".into(),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((NanocomMessageUiPlugin, LocalizationPlugin))
        .add_systems(Startup, setup_preview)
        .add_systems(Update, drive_capture)
        .run();
}

fn parse_options() -> PreviewOptions {
    let mut args = std::env::args().skip(1);
    let stage = args
        .next()
        .as_deref()
        .and_then(PreviewStage::parse)
        .unwrap_or_default();
    let language = args.next().unwrap_or_else(|| "en".to_owned());
    if !matches!(language.as_str(), "en" | "ru") {
        eprintln!("language must be en or ru");
        std::process::exit(2);
    }
    let output = args.next().map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(format!(
            "target/ui-parity/nanocom-{}-{language}-1264x681.png",
            stage.slug()
        ))
    });
    if args.next().is_some() || output.extension().and_then(|value| value.to_str()) != Some("png") {
        eprintln!(
            "usage: nanocom_buddy_gpu_preview [npc-compact|recall-compact|computress-compact|buddy-compact|buddy-modal|group-compact|group-modal] [en|ru] [OUTPUT.png]"
        );
        std::process::exit(2);
    }
    PreviewOptions {
        output,
        language,
        stage,
    }
}

fn setup_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    options: Res<PreviewOptions>,
    mut model: ResMut<NanocomMessageUiModel>,
    mut time: ResMut<Time<Virtual>>,
) {
    time.pause();
    commands.spawn((Camera2d, IsDefaultUiCamera));
    spawn_world_backdrop(&mut commands);

    model.set_ui_scale(1.0);
    match options.stage {
        PreviewStage::RecallCompact => {
            model.enqueue_type_9_localized_optional_icon(
                LocalizedText::new("ui.nano.recall.title", "Recall"),
                LocalizedText::new(
                    "ui.nano.recall.registered",
                    "Recall Point registered! Use your Recall Nano to return to this point.",
                ),
                None,
                None,
            );
        }
        PreviewStage::NpcCompact => {
            let (title, body) = if options.language == "ru" {
                ("Номер Два", "Выходи на связь, кадет.")
            } else {
                ("Numbuh Two", "Come in, cadet.")
            };
            model.enqueue_type_9_numbuh_two(9, title, body);
        }
        PreviewStage::ComputressCompact => {
            model.enqueue_type_9_localized(
                LocalizedText::new("content.npc.730.name", "Computress"),
                LocalizedText::new(
                    "content.tabledata.guide.guide_string.19.sz_string",
                    "Welcome back. Please check your email to learn about an important mission from me.",
                ),
                NANOCOM_COMPUTRESS_ICON_PATH,
                Some("Computress"),
            );
        }
        PreviewStage::BuddyCompact | PreviewStage::BuddyModal => {
            model.enqueue_buddy_invite(13, "Dexter");
        }
        PreviewStage::GroupCompact | PreviewStage::GroupModal => {
            model.enqueue(NanocomMessageRequest::group_invite(14, "Remote Player"));
        }
    }
    // Capture the stable fully revealed state after the exact half-second
    // clean slide interval; virtual time remains paused afterwards.
    assert!(model.tick(0.5).is_none());
    model.set_expanded(options.stage.is_modal());
    while model.pop_sound().is_some() {}

    let images = [
        NANOCOM_BUDDY_FRAME_PATH,
        NANOCOM_BUDDY_ICON_PATH,
        NANOCOM_GROUP_ICON_PATH,
        NANOCOM_TYPE_9_FRAME_PATH,
        NANOCOM_NUMBUH_TWO_ICON_PATH,
        NANOCOM_COMPUTRESS_ICON_PATH,
        NANOCOM_DIALOG_PATH,
        NANOCOM_MESSAGE_AREA_PATH,
        NANOCOM_BLUE_BUTTON_PATH,
        NANOCOM_BLUE_BUTTON_OVER_PATH,
        NANOCOM_RED_BUTTON_PATH,
        NANOCOM_RED_BUTTON_OVER_PATH,
    ]
    .into_iter()
    .map(|path| asset_server.load(path))
    .collect::<Vec<_>>();
    let jeffe = asset_server.load(NANOCOM_JEFFE_FONT_PATH);
    let chalet = asset_server.load(NANOCOM_CHALET_FONT_PATH);
    commands.insert_resource(PreviewConfig {
        output: options.output.clone(),
        language: options.language.clone(),
        stage: options.stage,
        images,
        fonts: vec![jeffe.clone(), chalet.clone()],
        jeffe,
        chalet,
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

#[allow(clippy::too_many_arguments)]
fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    fonts: Res<Assets<Font>>,
    config: Res<PreviewConfig>,
    mut state: ResMut<PreviewState>,
    compact_panels: Query<&ComputedNode, With<NanocomMessageCompactPanel>>,
    dialogs: Query<&ComputedNode, With<NanocomMessageExpandedDialog>>,
    all_texts: Query<&Text>,
    styled_texts: Query<(
        Entity,
        &Text,
        &LocalizedText,
        &NanocomGuiStyleRole,
        (&TextFont, &LineHeight),
        &TextLayoutInfo,
        &ComputedNode,
        &InheritedVisibility,
    )>,
    model: Res<NanocomMessageUiModel>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);
    let failed = config
        .images
        .iter()
        .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
        || config
            .fonts
            .iter()
            .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)));
    if failed {
        eprintln!("NanoCom source asset failed to load");
        exit.write(AppExit::error());
        return;
    }
    let loaded = config
        .images
        .iter()
        .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
        && config
            .fonts
            .iter()
            .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
        && config
            .images
            .iter()
            .all(|handle| images.get(handle).is_some())
        && config
            .fonts
            .iter()
            .all(|handle| fonts.get(handle).is_some());
    let compact_laid_out = compact_panels
        .single()
        .is_ok_and(|computed| computed.size().x >= 370.0 && computed.size().y >= 120.0);
    let modal_laid_out = !config.stage.is_modal()
        || dialogs
            .single()
            .is_ok_and(|computed| computed.size().x >= 518.0 && computed.size().y >= 162.0);
    let model_ready = model.compact_visible()
        && model.expanded_visible() == config.stage.is_modal()
        && model.active().is_some_and(|active| {
            active.request.kind == config.stage.expected_kind()
                && active.remaining_seconds
                    == if config.stage.expected_kind() == NanocomMessageKind::Npc {
                        9.5
                    } else {
                        19.5
                    }
        });
    let ready = loaded && compact_laid_out && modal_laid_out && model_ready;
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
    if ready && warmed && !state.text_audited {
        match audit_text_bounds(&all_texts, &styled_texts, &config) {
            Ok(report) => {
                println!("{report}");
                state.text_audited = true;
            }
            Err(error) => {
                eprintln!("NanoCom GPU text audit failed: {error}");
                exit.write(AppExit::error());
                return;
            }
        }
    }
    if ready && warmed && state.text_audited && !state.capture_issued {
        state.capture_issued = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }
    if state.capture_saved {
        exit.write(AppExit::Success);
    } else if state.started_at.elapsed() >= CAPTURE_TIMEOUT {
        eprintln!(
            "NanoCom {:?} capture timed out: loaded={loaded}, compact={compact_laid_out}, modal={modal_laid_out}, model={model_ready}, active={:?}, compact-visible={}, modal-visible={}, audited={}, issued={}",
            config.stage,
            model.active().map(|active| (
                active.request.kind,
                active.remaining_seconds,
                active.request.request_id
            )),
            model.compact_visible(),
            model.expanded_visible(),
            state.text_audited,
            state.capture_issued,
        );
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

fn audit_text_bounds(
    all_texts: &Query<'_, '_, &Text>,
    styled_texts: &Query<
        '_,
        '_,
        (
            Entity,
            &Text,
            &LocalizedText,
            &NanocomGuiStyleRole,
            (&TextFont, &LineHeight),
            &TextLayoutInfo,
            &ComputedNode,
            &InheritedVisibility,
        ),
    >,
    config: &PreviewConfig,
) -> Result<String, String> {
    let all_count = all_texts.iter().count();
    let styled_count = styled_texts.iter().count();
    if all_count != 7 || styled_count != all_count {
        return Err(format!(
            "every-Text ownership incomplete: styled={styled_count}, all={all_count}, expected=7"
        ));
    }

    let mut visible_count = 0usize;
    let mut glyph_count = 0usize;
    let mut line_count = 0usize;
    let mut saw_cyrillic = false;
    let mut visible_keys = BTreeSet::new();
    let mut visible_styles = BTreeSet::new();
    for (entity, text, localized, style, font, layout, computed, inherited) in styled_texts {
        if localized.key.trim().is_empty() {
            return Err(format!("Text {entity:?} has an empty semantic key"));
        }
        let spec = nanocom_gui_style(*style);
        let expected_font = match style {
            NanocomGuiStyleRole::MessageText | NanocomGuiStyleRole::CenterBox2 => &config.chalet,
            _ => &config.jeffe,
        };
        if font.0.font != expected_font.clone().into()
            || font.0.font_size.eval(Vec2::ZERO, 16.0) != spec.replacement_font_size
            || (*font.1) != LineHeight::Px(spec.source_line_height)
            || spec.content_offset != [0.0, 0.0]
            || spec.replacement_y_offset != 0.0
        {
            return Err(format!(
                "{} has wrong {:?} replacement metrics: size={}, line={:?}",
                localized.key,
                style,
                font.0.font_size.eval(Vec2::ZERO, 16.0),
                (*font.1)
            ));
        }
        if !inherited.get()
            || computed.size().x <= 0.0
            || computed.size().y <= 0.0
            || text.0.is_empty()
            || layout.glyphs.is_empty()
        {
            continue;
        }
        visible_count += 1;
        glyph_count += layout.glyphs.len();
        line_count += layout.run_geometry.len();
        visible_keys.insert(localized.key.as_str());
        visible_styles.insert(*style);
        saw_cyrillic |= text.0.chars().any(|character| {
            ('\u{0400}'..='\u{052f}').contains(&character)
                || ('\u{2de0}'..='\u{2dff}').contains(&character)
        });
        if !layout.size.is_finite()
            || layout.size.x > computed.size().x + 1.0
            || layout.size.y > computed.size().y + 1.0
        {
            return Err(format!(
                "{} GPU text bounds {:?} exceed node {:?}",
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
                "{} glyph bounds {:?}..{:?} exceed node {:?}",
                localized.key,
                minimum,
                maximum,
                computed.size()
            ));
        }
        for run in &layout.run_geometry {
            let line = run.bounds;
            if !line.min.is_finite()
                || !line.max.is_finite()
                || line.min.y < -0.5
                || line.max.y > layout.size.y + 0.5
                || (line.height() - spec.source_line_height).abs() > 0.65
            {
                return Err(format!(
                    "{} invalid {:?} line/baseline box {:?} for {}",
                    localized.key, style, line, spec.source_line_height
                ));
            }
        }
    }
    let expected_visible = if config.stage.is_modal() { 7 } else { 2 };
    if visible_count != expected_visible || glyph_count == 0 || line_count == 0 {
        return Err(format!(
            "visible GPU text incomplete: visible={visible_count}/{expected_visible}, glyphs={glyph_count}, lines={line_count}"
        ));
    }
    if config.language == "ru" && !saw_cyrillic {
        return Err("RU production bundle produced no visible Cyrillic glyphs".to_owned());
    }
    let required_keys: &[&str] = match config.stage {
        PreviewStage::NpcCompact => &["ui.content.passthrough"],
        PreviewStage::RecallCompact => &["ui.nano.recall.title", "ui.nano.recall.registered"],
        PreviewStage::ComputressCompact => &[
            "content.npc.730.name",
            "content.tabledata.guide.guide_string.19.sz_string",
        ],
        PreviewStage::BuddyCompact => &[
            NANOCOM_COMPACT_TITLE_LOCALIZATION_KEY,
            NANOCOM_COMPACT_BODY_LOCALIZATION_KEY,
        ],
        PreviewStage::BuddyModal => &[
            NANOCOM_COMPACT_TITLE_LOCALIZATION_KEY,
            NANOCOM_COMPACT_BODY_LOCALIZATION_KEY,
            NANOCOM_INVITATION_LOCALIZATION_KEY,
            NANOCOM_EXPANDED_TITLE_LOCALIZATION_KEY,
            NANOCOM_EXPIRATION_LOCALIZATION_KEY,
            NANOCOM_ACCEPT_LOCALIZATION_KEY,
            NANOCOM_DECLINE_LOCALIZATION_KEY,
        ],
        PreviewStage::GroupCompact => &[
            NANOCOM_GROUP_COMPACT_TITLE_LOCALIZATION_KEY,
            NANOCOM_GROUP_COMPACT_BODY_LOCALIZATION_KEY,
        ],
        PreviewStage::GroupModal => &[
            NANOCOM_GROUP_COMPACT_TITLE_LOCALIZATION_KEY,
            NANOCOM_GROUP_COMPACT_BODY_LOCALIZATION_KEY,
            NANOCOM_GROUP_INVITATION_LOCALIZATION_KEY,
            NANOCOM_GROUP_EXPANDED_TITLE_LOCALIZATION_KEY,
            NANOCOM_GROUP_EXPIRATION_LOCALIZATION_KEY,
            NANOCOM_ACCEPT_LOCALIZATION_KEY,
            NANOCOM_DECLINE_LOCALIZATION_KEY,
        ],
    };
    for key in required_keys {
        if !visible_keys.contains(key) {
            return Err(format!("{:?} is missing visible key {key}", config.stage));
        }
    }
    let mut required_styles = vec![
        NanocomGuiStyleRole::BigFont14,
        NanocomGuiStyleRole::MessageText,
    ];
    if config.stage.is_modal() {
        required_styles.extend([
            NanocomGuiStyleRole::MessageTitle,
            NanocomGuiStyleRole::CenterBox2,
            NanocomGuiStyleRole::Button,
            NanocomGuiStyleRole::RedButton,
        ]);
    }
    for style in required_styles {
        if !visible_styles.contains(&style) {
            return Err(format!("{:?} is missing visible {style:?}", config.stage));
        }
    }
    Ok(format!(
        "NanoCom GPU text audit: stage={:?}, locale={}, key-first={}/{}, visible={}, glyphs={}, line-boxes={}, styles={visible_styles:?}, y-offsets=0",
        config.stage,
        config.language,
        styled_count,
        all_count,
        visible_count,
        glyph_count,
        line_count
    ))
}

fn save_screenshot(
    event: On<ScreenshotCaptured>,
    config: Res<PreviewConfig>,
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
    let cyan = rgba
        .pixels()
        .filter(|pixel| {
            let [red, green, blue, _] = pixel.0;
            green > 80 && blue > 90 && blue > red.saturating_add(25)
        })
        .count();
    if cyan < 40 {
        panic!("NanoCom capture has too little cyan title/frame coverage: {cyan}");
    }
    if config.stage.is_modal() {
        let dimmed = rgba
            .pixels()
            .filter(|pixel| {
                let [red, green, blue, _] = pixel.0;
                let total = u16::from(red) + u16::from(green) + u16::from(blue);
                total > 20 && total < 300
            })
            .count();
        let red = rgba
            .pixels()
            .filter(|pixel| {
                let [red, green, blue, _] = pixel.0;
                red > 75 && red > green.saturating_add(20) && red > blue.saturating_add(15)
            })
            .count();
        if dimmed < 80_000 || red < 120 {
            panic!("incomplete modal coverage: dimmed={dimmed}, red={red}");
        }
    }
    if let Some(parent) = config
        .output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("cannot create {}: {error}", parent.display()));
    }
    image
        .save(&config.output)
        .unwrap_or_else(|error| panic!("cannot save {}: {error}", config.output.display()));
    println!("{}", absolute_display(&config.output));
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
