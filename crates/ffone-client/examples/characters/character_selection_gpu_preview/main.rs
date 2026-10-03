//! Deterministic GPU acceptance frame for the native character-selection UI.
//!
//! The harness reads only semantic Bevy assets from `assets/game`. Player
//! preview and slot portrait areas deliberately remain transparent until the
//! native modular-player assembly owns their cameras.

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
use ffone_client::character_selection_ui::{
    CHARACTER_SELECTION_BACKGROUND_FRAME_COUNT, CHARACTER_SELECTION_BLUE_BUTTON_OVER_PATH,
    CHARACTER_SELECTION_BLUE_BUTTON_PATH, CHARACTER_SELECTION_CANCEL_NORMAL_PATH,
    CHARACTER_SELECTION_CHALET_FONT_PATH, CHARACTER_SELECTION_CHROME_PATH,
    CHARACTER_SELECTION_DELETE_BACKDROP_PATH, CHARACTER_SELECTION_DELETE_WINDOW_PATH,
    CHARACTER_SELECTION_DISK_BACK_PATH, CHARACTER_SELECTION_DISK_FRONT_PATH,
    CHARACTER_SELECTION_ENTER_OVER_PATH, CHARACTER_SELECTION_ENTER_PATH,
    CHARACTER_SELECTION_FULLSCREEN_OVER_PATH, CHARACTER_SELECTION_FULLSCREEN_PATH,
    CHARACTER_SELECTION_JEFFE_FONT_PATH, CHARACTER_SELECTION_LOCK_PATH,
    CHARACTER_SELECTION_MUSIC_TOGGLE_OFF_PATH, CHARACTER_SELECTION_MUSIC_TOGGLE_ON_PATH,
    CHARACTER_SELECTION_RED_BUTTON_OVER_PATH, CHARACTER_SELECTION_RED_BUTTON_PATH,
    CHARACTER_SELECTION_ROTATE_LEFT_OVER_PATH, CHARACTER_SELECTION_ROTATE_LEFT_PATH,
    CHARACTER_SELECTION_ROTATE_RIGHT_OVER_PATH, CHARACTER_SELECTION_ROTATE_RIGHT_PATH,
    CHARACTER_SELECTION_SLOT_EMPTY_PATH, CHARACTER_SELECTION_SLOT_LOCKED_PATH,
    CHARACTER_SELECTION_SLOT_NORMAL_PATH, CHARACTER_SELECTION_SLOT_OVER_PATH,
    CHARACTER_SELECTION_WINDOWED_OVER_PATH, CHARACTER_SELECTION_WINDOWED_PATH,
    CharacterLocationBackground, CharacterPreviewStatus, CharacterSelectionCapability,
    CharacterSelectionFontRole, CharacterSelectionTextAnchor, CharacterSelectionTextStyle,
    CharacterSelectionUiModel, CharacterSlotUi, NativeCharacterSelectionRoot,
    NativeCharacterSelectionUiPlugin, OccupiedCharacterSlotUi,
};
use ffone_client::localization::{Localization, LocalizationPlugin, LocalizedText};

const WARMUP_FRAMES_AFTER_LOAD: u32 = 120;
const RETRY_FRAMES_AFTER_REJECTED_CAPTURE: u32 = 120;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(2);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(15);
const MIN_VISIBLE_PIXEL_RATIO: f32 = 0.65;

#[derive(Resource)]
struct PreviewConfig {
    output: PathBuf,
    language: String,
    stage: PreviewStage,
    visible_images: Vec<Handle<Image>>,
    visible_fonts: Vec<Handle<Font>>,
    chalet_font: Handle<Font>,
    jeffe_font: Handle<Font>,
}

#[derive(Resource)]
struct PreviewOptions {
    output: PathBuf,
    language: String,
    stage: PreviewStage,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum PreviewStage {
    #[default]
    Populated,
    Delete,
    Empty,
}

impl PreviewStage {
    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "populated" => Self::Populated,
            "delete" => Self::Delete,
            "empty" => Self::Empty,
            _ => return None,
        })
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::Populated => "populated",
            Self::Delete => "delete",
            Self::Empty => "empty",
        }
    }
}

#[derive(Resource)]
struct PreviewState {
    frames: u32,
    first_loaded_frame: Option<u32>,
    assets_loaded_at: Option<Instant>,
    last_rejected_frame: Option<u32>,
    capture_issued: bool,
    capture_saved: bool,
    text_audited: bool,
    started_at: Instant,
}

impl Default for PreviewState {
    fn default() -> Self {
        Self {
            frames: 0,
            first_loaded_frame: None,
            assets_loaded_at: None,
            last_rejected_frame: None,
            capture_issued: false,
            capture_saved: false,
            text_audited: false,
            started_at: Instant::now(),
        }
    }
}

fn main() -> AppExit {
    let options = parse_options();

    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game")
        .canonicalize()
        .expect("assets/game must exist");
    let (localization, language) =
        Localization::open(&asset_root, &options.language).expect("open native localization");

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(PreviewState::default())
        .insert_resource(options)
        .insert_resource(localization)
        .insert_resource(language)
        .insert_resource(ffone_client::assets::AssetLocator::open(&asset_root).unwrap())
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne character selection acceptance".into(),
                        resolution: WindowResolution::new(1264, 681),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((NativeCharacterSelectionUiPlugin, LocalizationPlugin))
        .add_systems(Startup, setup_preview)
        .add_systems(Update, drive_capture)
        .run()
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
            "target/ui-parity/character-selection-{}-{language}-1264x681.png",
            stage.slug()
        ))
    });
    if args.next().is_some() || output.extension().and_then(|value| value.to_str()) != Some("png") {
        eprintln!(
            "usage: character_selection_gpu_preview [populated|delete|empty] [en|ru] [OUTPUT.png]"
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
    assets: Res<AssetServer>,
    options: Res<PreviewOptions>,
    mut model: ResMut<CharacterSelectionUiModel>,
    mut time: ResMut<Time<Virtual>>,
) {
    time.pause();
    // Production has exactly one `GameplayUiCamera` marked as the default UI
    // camera. Preserve that ownership so the dedicated higher-order modal
    // camera does not accidentally become the default for the base screen.
    commands.spawn((Camera2d, IsDefaultUiCamera));

    model.visible = true;
    let populated = [
        CharacterSlotUi::Occupied(OccupiedCharacterSlotUi {
            pc_uid: 1,
            display_name: "Test Ser".to_owned(),
            level: 1,
            district: "TECH SQUARE".to_owned(),
            zone: "THE FUTURE".to_owned(),
            background: CharacterLocationBackground::Future,
        }),
        CharacterSlotUi::Occupied(OccupiedCharacterSlotUi {
            pc_uid: 2,
            display_name: "Dexter Bell".to_owned(),
            level: 12,
            district: "POKÉ PLAZA".to_owned(),
            zone: "THE SUBURBS".to_owned(),
            background: CharacterLocationBackground::Suburbs,
        }),
        CharacterSlotUi::SubscriptionLockedOccupied(OccupiedCharacterSlotUi {
            pc_uid: 3,
            display_name: "Gaia Roundbreath".to_owned(),
            level: 36,
            district: "GENIUS GROVE".to_owned(),
            zone: "THE FUTURE".to_owned(),
            background: CharacterLocationBackground::Future,
        }),
        CharacterSlotUi::SubscriptionLocked,
    ];
    match options.stage {
        PreviewStage::Populated | PreviewStage::Delete => {
            model.slots = populated;
            model.selected_slot = Some(0);
        }
        PreviewStage::Empty => {
            model.slots = std::array::from_fn(|_| CharacterSlotUi::Empty);
            model.selected_slot = None;
        }
    }
    model.music_enabled = env::var("FFONE_SELECTION_MUSIC").as_deref() != Ok("off");
    model.fullscreen = false;
    model.create = CharacterSelectionCapability::Enabled;
    model.delete = CharacterSelectionCapability::Enabled;
    model.preview = CharacterPreviewStatus::PlayerAssemblyPending;
    if options.stage == PreviewStage::Delete {
        model.delete_confirmation_pc_uid = Some(1);
        model.delete_name_input = "Test".to_owned();
    }

    let mut visible_images: Vec<Handle<Image>> = [
        CHARACTER_SELECTION_CHROME_PATH,
        CHARACTER_SELECTION_SLOT_EMPTY_PATH,
        CHARACTER_SELECTION_SLOT_NORMAL_PATH,
        CHARACTER_SELECTION_SLOT_OVER_PATH,
        CHARACTER_SELECTION_SLOT_LOCKED_PATH,
        CHARACTER_SELECTION_LOCK_PATH,
        CHARACTER_SELECTION_ENTER_PATH,
        CHARACTER_SELECTION_ENTER_OVER_PATH,
        CHARACTER_SELECTION_FULLSCREEN_PATH,
        CHARACTER_SELECTION_FULLSCREEN_OVER_PATH,
        CHARACTER_SELECTION_WINDOWED_PATH,
        CHARACTER_SELECTION_WINDOWED_OVER_PATH,
        CHARACTER_SELECTION_MUSIC_TOGGLE_ON_PATH,
        CHARACTER_SELECTION_MUSIC_TOGGLE_OFF_PATH,
        CHARACTER_SELECTION_RED_BUTTON_PATH,
        CHARACTER_SELECTION_RED_BUTTON_OVER_PATH,
        CHARACTER_SELECTION_BLUE_BUTTON_PATH,
        CHARACTER_SELECTION_BLUE_BUTTON_OVER_PATH,
        CHARACTER_SELECTION_DELETE_BACKDROP_PATH,
        CHARACTER_SELECTION_DELETE_WINDOW_PATH,
        CHARACTER_SELECTION_CANCEL_NORMAL_PATH,
        CHARACTER_SELECTION_ROTATE_LEFT_PATH,
        CHARACTER_SELECTION_ROTATE_LEFT_OVER_PATH,
        CHARACTER_SELECTION_ROTATE_RIGHT_PATH,
        CHARACTER_SELECTION_ROTATE_RIGHT_OVER_PATH,
        CHARACTER_SELECTION_DISK_BACK_PATH,
        CHARACTER_SELECTION_DISK_FRONT_PATH,
    ]
    .into_iter()
    .map(|path| assets.load(path))
    .collect();
    visible_images.extend(
        (1..=CHARACTER_SELECTION_BACKGROUND_FRAME_COUNT)
            .map(|frame| assets.load(CharacterLocationBackground::Future.asset_path(frame))),
    );

    let chalet_font = assets.load(CHARACTER_SELECTION_CHALET_FONT_PATH);
    let jeffe_font = assets.load(CHARACTER_SELECTION_JEFFE_FONT_PATH);
    commands.insert_resource(PreviewConfig {
        output: options.output.clone(),
        language: options.language.clone(),
        stage: options.stage,
        visible_images,
        visible_fonts: vec![chalet_font.clone(), jeffe_font.clone()],
        chalet_font,
        jeffe_font,
    });
}

fn drive_capture(
    mut commands: Commands,
    assets: Res<AssetServer>,
    images: Res<Assets<Image>>,
    fonts: Res<Assets<Font>>,
    config: Res<PreviewConfig>,
    mut state: ResMut<PreviewState>,
    roots: Query<(&Visibility, &ComputedNode), With<NativeCharacterSelectionRoot>>,
    all_texts: Query<&Text>,
    styled_texts: Query<(
        Entity,
        &Node,
        &Text,
        &LocalizedText,
        &CharacterSelectionTextStyle,
        (&TextFont, &LineHeight),
        &TextLayout,
        &TextLayoutInfo,
        &ComputedNode,
        &InheritedVisibility,
        &ChildOf,
    )>,
    computed_nodes: Query<&ComputedNode>,
    cameras: Query<&Camera>,
    targeted_ui_roots: Query<(
        &UiTargetCamera,
        &ComputedUiTargetCamera,
        &ComputedNode,
        &UiGlobalTransform,
        &Visibility,
        Option<&ChildOf>,
    )>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);
    let source_assets_loaded = config
        .visible_images
        .iter()
        .all(|handle| matches!(assets.load_state(handle.id()), LoadState::Loaded))
        && config
            .visible_fonts
            .iter()
            .all(|handle| matches!(assets.load_state(handle.id()), LoadState::Loaded));
    let cpu_assets_present = config
        .visible_images
        .iter()
        .all(|handle| images.get(handle).is_some())
        && config
            .visible_fonts
            .iter()
            .all(|handle| fonts.get(handle).is_some());
    let root_laid_out = roots.single().is_ok_and(|(visibility, computed)| {
        *visibility != Visibility::Hidden
            && computed.size().x >= 1264.0
            && computed.size().y >= 681.0
    });
    let loaded = source_assets_loaded && cpu_assets_present && root_laid_out;
    if loaded && state.first_loaded_frame.is_none() {
        state.first_loaded_frame = Some(state.frames);
        state.assets_loaded_at = Some(Instant::now());
    }
    let gpu_grace_elapsed = state
        .assets_loaded_at
        .is_some_and(|loaded_at| loaded_at.elapsed() >= GPU_UPLOAD_GRACE);
    let retry_ready = state.last_rejected_frame.is_none_or(|frame| {
        state.frames.saturating_sub(frame) >= RETRY_FRAMES_AFTER_REJECTED_CAPTURE
    });
    let capture_ready = !state.capture_issued
        && loaded
        && gpu_grace_elapsed
        && retry_ready
        && state
            .first_loaded_frame
            .is_some_and(|frame| state.frames.saturating_sub(frame) >= WARMUP_FRAMES_AFTER_LOAD);
    if capture_ready && !state.text_audited {
        match audit_visible_text(
            &all_texts,
            &styled_texts,
            &computed_nodes,
            &cameras,
            &targeted_ui_roots,
            &config,
        ) {
            Ok(summary) => {
                println!("{summary}");
                state.text_audited = true;
            }
            Err(error) => {
                eprintln!("character-selection GPU text audit failed: {error}");
                exit.write(AppExit::error());
                return;
            }
        }
    }
    if capture_ready && state.text_audited {
        state.capture_issued = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }
    if state.capture_saved {
        exit.write(AppExit::Success);
    } else if state.started_at.elapsed() >= CAPTURE_TIMEOUT {
        eprintln!("character-selection capture timed out");
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

fn audit_visible_text(
    all_texts: &Query<'_, '_, &Text>,
    styled_texts: &Query<
        '_,
        '_,
        (
            Entity,
            &Node,
            &Text,
            &LocalizedText,
            &CharacterSelectionTextStyle,
            (&TextFont, &LineHeight),
            &TextLayout,
            &TextLayoutInfo,
            &ComputedNode,
            &InheritedVisibility,
            &ChildOf,
        ),
    >,
    computed_nodes: &Query<'_, '_, &ComputedNode>,
    cameras: &Query<'_, '_, &Camera>,
    targeted_ui_roots: &Query<
        '_,
        '_,
        (
            &UiTargetCamera,
            &ComputedUiTargetCamera,
            &ComputedNode,
            &UiGlobalTransform,
            &Visibility,
            Option<&ChildOf>,
        ),
    >,
    config: &PreviewConfig,
) -> Result<String, String> {
    let all_count = all_texts.iter().count();
    let styled_count = styled_texts.iter().count();
    if all_count != 27 || styled_count != all_count {
        return Err(format!(
            "every-Text ownership incomplete: styled={styled_count}, all={all_count}, expected=27"
        ));
    }

    let mut visible_count = 0usize;
    let mut glyph_count = 0usize;
    let mut line_count = 0usize;
    let mut fitted_count = 0usize;
    let mut saw_cyrillic = false;
    let mut saw_title = false;
    let mut visible_styles = Vec::new();
    let mut visible_keys = BTreeSet::new();
    for (
        entity,
        node,
        text,
        localized,
        style,
        font,
        text_layout,
        layout,
        computed,
        inherited_visibility,
        parent,
    ) in styled_texts
    {
        if localized.key.trim().is_empty() {
            return Err(format!("Text {entity:?} has an empty semantic key"));
        }
        let spec = style.spec();
        let expected_font = match spec.font_role {
            CharacterSelectionFontRole::Jeffe => &config.jeffe_font,
            CharacterSelectionFontRole::Chalet => &config.chalet_font,
        };
        // The production UiTextAutoFit uses a 6 px floor. The old standalone
        // localization stub used 70%, so it cannot define this runtime audit.
        let minimum_font_size = 6.0_f32.min(spec.font_size);
        if font.0.font_size.eval(Vec2::ZERO, 16.0) + f32::EPSILON < minimum_font_size
            || font.0.font_size.eval(Vec2::ZERO, 16.0) > spec.font_size + f32::EPSILON
        {
            return Err(format!(
                "{} has replacement size {} outside {}..={} for {style:?}",
                localized.key,
                font.0.font_size.eval(Vec2::ZERO, 16.0),
                minimum_font_size,
                spec.font_size
            ));
        }
        if font.0.font_size.eval(Vec2::ZERO, 16.0) + f32::EPSILON < spec.font_size {
            fitted_count += 1;
        }
        let current_line_height =
            spec.line_height * font.0.font_size.eval(Vec2::ZERO, 16.0) / spec.font_size;
        if font.0.font != expected_font.clone().into()
            || !matches!((*font.1), LineHeight::Px(actual) if (actual - current_line_height).abs() < 0.0001)
        {
            return Err(format!(
                "{} has wrong replacement metrics for {style:?}: size={}, line={:?}",
                localized.key,
                font.0.font_size.eval(Vec2::ZERO, 16.0),
                (*font.1)
            ));
        }
        let expected_justify = match spec.anchor {
            CharacterSelectionTextAnchor::MiddleLeft => Justify::Left,
            CharacterSelectionTextAnchor::MiddleCenter => Justify::Center,
        };
        if text_layout.justify != expected_justify {
            return Err(format!("{} has wrong {style:?} alignment", localized.key));
        }
        if localized.key == "ui.character_select.title" {
            let expected = if config.language == "ru" {
                "ВЫБЕРИТЕ ПЕРСОНАЖА"
            } else {
                "SELECT A CHARACTER"
            };
            if text.0 != expected {
                return Err(format!(
                    "localized title {:?} does not match {expected:?}",
                    text.0
                ));
            }
            saw_title = true;
        }

        if !inherited_visibility.get()
            || computed.size().x <= 0.0
            || computed.size().y <= 0.0
            || text.0.is_empty()
            || layout.glyphs.is_empty()
        {
            continue;
        }
        visible_count += 1;
        visible_keys.insert(localized.key.as_str());
        glyph_count += layout.glyphs.len();
        line_count += layout.run_geometry.len();
        saw_cyrillic |= text.0.chars().any(|character| {
            ('\u{0400}'..='\u{052f}').contains(&character)
                || ('\u{2de0}'..='\u{2dff}').contains(&character)
        });
        if !visible_styles.contains(style) {
            visible_styles.push(*style);
        }

        let source_rect = if node.position_type == PositionType::Absolute {
            computed
        } else {
            computed_nodes
                .get(parent.parent())
                .map_err(|_| format!("{} has no computed source-Rect parent", localized.key))?
        };
        if !layout.size.is_finite()
            || layout.size.x > source_rect.size().x + 1.0
            || layout.size.y > source_rect.size().y + 1.0
        {
            return Err(format!(
                "{} GPU text bounds {:?} exceed source Rect {:?}",
                localized.key,
                layout.size,
                source_rect.size()
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
        for run in &layout.run_geometry {
            let line = run.bounds;
            // Bevy 0.19 replaced the per-section line rect with `run_geometry`,
            // whose `bounds` is the decoration/selection box, not the line box.
            // Parley clamps that box to at least `ascent + descent`, so an
            // authored line height tighter than the font's em box legitimately
            // reports a taller box that overhangs the line: a 12 px font on a
            // 13.71 px line height measures 15 px starting at -1.0. The authored
            // height stays under contract on the `LineHeight` component checked
            // above, so allow the decoration box exactly that documented
            // overhang and still fail a collapsed or oversized run.
            let overhang =
                (font.0.font_size.eval(Vec2::ZERO, 16.0) * 1.5 - current_line_height).max(0.0) + 0.55;
            if !line.min.is_finite()
                || !line.max.is_finite()
                || line.min.y < -overhang
                || line.max.y > layout.size.y + overhang
                || line.height() + 0.55 < current_line_height
                || line.height() > current_line_height + overhang
            {
                return Err(format!(
                    "{} invalid GPU line/baseline box {:?} for line height {}",
                    localized.key, line, current_line_height
                ));
            }
        }
    }
    if !saw_title || visible_count == 0 || glyph_count == 0 || line_count == 0 {
        return Err("localized visible GPU text has not been laid out".to_owned());
    }
    if config.language == "ru" && !saw_cyrillic {
        return Err("RU production bundle produced no visible Cyrillic glyphs".to_owned());
    }
    let required_keys: &[&str] = match config.stage {
        PreviewStage::Populated => &[
            "ui.character_select.title",
            "ui.character_select.level",
            "ui.character_select.location.spaced",
            "ui.character_select.location.hyphen",
            "ui.character_select.unlimited_only",
            "ui.character_select.delete",
            "ui.character_select.enter",
            "ui.common.quit",
            "ui.content.passthrough",
        ],
        PreviewStage::Delete => &[
            "ui.character_select.title",
            "ui.character_select.level",
            "ui.character_select.location.spaced",
            "ui.character_select.location.hyphen",
            "ui.character_select.unlimited_only",
            "ui.character_select.delete",
            "ui.character_select.enter",
            "ui.character_select.delete_prompt",
            "ui.common.cancel",
            "ui.common.delete",
            "ui.common.quit",
            "ui.content.passthrough",
        ],
        PreviewStage::Empty => &[
            "ui.character_select.title",
            "ui.character_select.empty",
            "ui.character_select.create",
            "ui.common.quit",
        ],
    };
    for required in required_keys {
        if !visible_keys.contains(required) {
            return Err(format!(
                "{:?} capture is missing visible key {required}",
                config.stage
            ));
        }
    }
    let required_styles: &[CharacterSelectionTextStyle] = match config.stage {
        PreviewStage::Populated | PreviewStage::Delete => &[
            CharacterSelectionTextStyle::Transparent2,
            CharacterSelectionTextStyle::Transparent3,
            CharacterSelectionTextStyle::CharNameUp,
            CharacterSelectionTextStyle::CharNameDown,
            CharacterSelectionTextStyle::CharLevelUp,
            CharacterSelectionTextStyle::AvatarName,
            CharacterSelectionTextStyle::QuitButton,
            CharacterSelectionTextStyle::EnterGame,
        ],
        PreviewStage::Empty => &[
            CharacterSelectionTextStyle::Transparent2,
            CharacterSelectionTextStyle::Transparent3,
            CharacterSelectionTextStyle::QuitButton,
            CharacterSelectionTextStyle::CreateButton,
        ],
    };
    for required in required_styles {
        if !visible_styles.contains(required) {
            return Err(format!(
                "{:?} capture is missing visible {required:?} glyph/line bounds",
                config.stage
            ));
        }
    }
    if config.stage == PreviewStage::Delete {
        for required in [
            CharacterSelectionTextStyle::DeleteText,
            CharacterSelectionTextStyle::Cancel,
        ] {
            if !visible_styles.contains(&required) {
                return Err(format!(
                    "delete capture is missing visible {required:?} glyph/line bounds"
                ));
            }
        }
    }
    let mut modal_roots = targeted_ui_roots.iter().filter(|(target, ..)| {
        cameras.get(target.0).is_ok_and(|camera| {
            camera.order
                == ffone_client::character_selection_ui::CHARACTER_SELECTION_MODAL_CAMERA_ORDER
        })
    });
    let (target, computed_target, modal_root, modal_transform, modal_visibility, parent) =
        modal_roots.next().ok_or("modal target root is missing")?;
    if modal_roots.next().is_some() {
        return Err("modal target root is not unique".to_owned());
    }
    if parent.is_some() || computed_target.get() != Some(target.0) {
        return Err("modal UiTargetCamera is not active on a detached root".to_owned());
    }
    let expected_modal_visibility = if config.stage == PreviewStage::Delete {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    if *modal_visibility != expected_modal_visibility {
        return Err(format!(
            "{:?} modal visibility is {:?}, expected {:?}",
            config.stage, modal_visibility, expected_modal_visibility
        ));
    }
    visible_styles.sort_by_key(|style| format!("{style:?}"));
    Ok(format!(
        "Character Selection GPU text audit: stage={:?}, locale={}, key-first={}/{}, visible={}, auto-fit={}, glyphs={}, line-boxes={}, modal-root={:?}@{:?}/{:?}, styles={visible_styles:?}",
        config.stage,
        config.language,
        styled_count,
        all_count,
        visible_count,
        fitted_count,
        glyph_count,
        line_count,
        modal_root.size(),
        modal_transform.translation,
        modal_visibility
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
    let rgba = image.to_rgba8();
    let visible_pixels = rgba
        .pixels()
        .filter(|pixel| {
            let [red, green, blue, _alpha] = pixel.0;
            u16::from(red) + u16::from(green) + u16::from(blue) > 20
        })
        .count();
    let visible_ratio = visible_pixels as f32 / (rgba.width() * rgba.height()) as f32;
    if visible_ratio < MIN_VISIBLE_PIXEL_RATIO {
        eprintln!(
            "rejecting incomplete character-selection capture: visible pixel ratio \
             {visible_ratio:.3} < {MIN_VISIBLE_PIXEL_RATIO:.3}"
        );
        state.capture_issued = false;
        state.last_rejected_frame = Some(state.frames);
        return;
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
