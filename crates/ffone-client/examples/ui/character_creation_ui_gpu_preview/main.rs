//! Standalone 1264x681 GPU acceptance frame for clean Character Creation UI.
//!
//! Run once per state/locale. The full `character_creation_gpu_preview`
//! remains the production-catalog/player-material acceptance harness; this
//! direct-source harness keeps chrome and localized text independently
//! verifiable while unrelated client modules are in progress.

use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use bevy::{
    asset::{AssetPlugin, LoadState},
    ecs::system::SystemParam,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    text::{ComputedTextBlock, LineHeight, TextLayoutInfo},
    window::{PresentMode, WindowResolution},
};
use ffone_client::{
    character_creation_ui::{
        CHARACTER_CREATION_DISPLAY_FONT_PATH, CHARACTER_CREATION_ENGINE_IMAGE_SPECS,
        CHARACTER_CREATION_FONT_PATH, CHARACTER_CREATION_IMAGE_SPECS,
        CHARACTER_CREATION_SHARED_IMAGE_SPECS, CharacterCreationCapability,
        CharacterCreationFontRole, CharacterCreationScreen, CharacterCreationTextAnchor,
        CharacterCreationTextStyle, CharacterCreationUiModel, CharacterNameLists,
        CharacterNameMode, NativeCharacterCreationRoot, NativeCharacterCreationUiPlugin,
    },
    localization::{Language, Localization, LocalizationPlugin, LocalizedText},
};
use serde::Deserialize;

const WIDTH: u32 = 1_264;
const HEIGHT: u32 = 681;
const WARMUP_FRAMES: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(20);
const MIN_VISIBLE_PIXELS: usize = 350_000;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
enum PreviewStage {
    #[default]
    Generated,
    Custom,
    Appearance,
}

impl PreviewStage {
    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "generated" => Self::Generated,
            "custom" => Self::Custom,
            "appearance" => Self::Appearance,
            _ => return None,
        })
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::Generated => "generated",
            Self::Custom => "custom",
            Self::Appearance => "appearance",
        }
    }
}

#[derive(Resource)]
struct PreviewOptions {
    stage: PreviewStage,
    locale: String,
    output: PathBuf,
}

#[derive(Resource)]
struct PreviewAssets {
    images: Vec<Handle<Image>>,
    jeffe_font: Handle<Font>,
    chalet_font: Handle<Font>,
}

#[derive(Clone, Resource)]
struct PreviewCatalog {
    hair_label: String,
    face_label: String,
    icon_paths: [[String; 5]; 3],
}

#[derive(Deserialize)]
struct AppearanceDocument {
    choices: Vec<AppearanceChoice>,
    maxima: AppearanceMaxima,
}

#[derive(Deserialize)]
struct AppearanceChoice {
    category: String,
    #[serde(rename = "creationIndex")]
    creation_index: u8,
    gender: String,
    icon: Option<NativeIcon>,
    label: String,
}

#[derive(Deserialize)]
struct AppearanceMaxima {
    #[serde(rename = "maleShirts")]
    male_shirts: u8,
    #[serde(rename = "malePants")]
    male_pants: u8,
    #[serde(rename = "maleShoes")]
    male_shoes: u8,
}

#[derive(Deserialize)]
struct NativeIcon {
    status: String,
    candidates: Vec<NativeAssetReference>,
}

#[derive(Deserialize)]
struct NativeAssetReference {
    bytes: u64,
    path: String,
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

#[derive(SystemParam)]
struct TextAuditQueries<'w, 's> {
    all: Query<'w, 's, &'static Text>,
    styled: Query<
        'w,
        's,
        (
            Entity,
            &'static Node,
            &'static Text,
            &'static LocalizedText,
            &'static CharacterCreationTextStyle,
            (
                &'static TextFont,
                &'static LineHeight,
                &'static ComputedTextBlock,
            ),
            &'static TextLayout,
            &'static TextLayoutInfo,
            &'static ComputedNode,
            &'static UiTransform,
            &'static InheritedVisibility,
            &'static ChildOf,
        ),
    >,
    computed: Query<'w, 's, &'static ComputedNode>,
}

fn main() {
    let mut args = std::env::args().skip(1);
    let stage = args
        .next()
        .as_deref()
        .and_then(PreviewStage::parse)
        .unwrap_or_default();
    let locale = args.next().unwrap_or_else(|| "en".to_owned());
    if !matches!(locale.as_str(), "en" | "ru") {
        eprintln!("locale must be en or ru");
        std::process::exit(2);
    }
    let output = args.next().map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(format!(
            "target/ui-parity/character-creation-{}-{}-1264x681.png",
            stage.slug(),
            locale
        ))
    });
    if args.next().is_some() || output.extension().and_then(|value| value.to_str()) != Some("png") {
        eprintln!(
            "usage: character_creation_ui_gpu_preview [generated|custom|appearance] [en|ru] [OUTPUT.png]"
        );
        std::process::exit(2);
    }

    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game")
        .canonicalize()
        .expect("assets/game must exist");
    let (localization, language) =
        Localization::open(&asset_root, &locale).expect("open native EN/RU localization");
    let catalog = load_preview_catalog(&asset_root).expect("open exact creator projection");
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(PreviewOptions {
            stage,
            locale,
            output,
        })
        .insert_resource(PreviewState::default())
        .insert_resource(localization)
        .insert_resource(language)
        .insert_resource(catalog)
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne Character Creation UI acceptance".into(),
                        resolution: WindowResolution::new(WIDTH, HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((LocalizationPlugin, NativeCharacterCreationUiPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, drive_capture)
        .run();
}

fn setup(
    mut commands: Commands,
    options: Res<PreviewOptions>,
    catalog: Res<PreviewCatalog>,
    asset_server: Res<AssetServer>,
    mut model: ResMut<CharacterCreationUiModel>,
    mut names: ResMut<CharacterNameLists>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    model.visible = true;
    model.music_enabled = false;
    model.screen = if options.stage == PreviewStage::Appearance {
        CharacterCreationScreen::Appearance
    } else {
        CharacterCreationScreen::Name
    };
    model.name_mode = if options.stage == PreviewStage::Custom {
        CharacterNameMode::Custom
    } else {
        CharacterNameMode::Generated
    };
    model.name_indices = [1, 2, 1];
    model.custom_name = "Astra Nova".to_owned();
    model.name_table = CharacterCreationCapability::Enabled;
    model.custom_name_filter = CharacterCreationCapability::Enabled;
    model.creation_items = CharacterCreationCapability::Enabled;
    model.starter_icons = CharacterCreationCapability::Enabled;
    model.hair_label.clone_from(&catalog.hair_label);
    model.face_label.clone_from(&catalog.face_label);
    for (target_row, source_row) in model.starter_icon_paths.iter_mut().zip(&catalog.icon_paths) {
        for (target, source) in target_row.iter_mut().zip(source_row) {
            *target = Some(source.clone());
        }
    }
    *names = CharacterNameLists {
        first: ["", "Zoom", "Zora", "Abbey", "Abner", "Acacia"]
            .map(str::to_owned)
            .into(),
        middle: ["", "Zoom", "Zort", " ", "Able", "Acorn"]
            .map(str::to_owned)
            .into(),
        last: ["", "Zombie", "Zon", " ", "Abnormal", "Abyss"]
            .map(str::to_owned)
            .into(),
    };
    let mut images = CHARACTER_CREATION_IMAGE_SPECS
        .iter()
        .chain(CHARACTER_CREATION_SHARED_IMAGE_SPECS.iter())
        .chain(CHARACTER_CREATION_ENGINE_IMAGE_SPECS.iter())
        .map(|spec| asset_server.load(spec.path))
        .collect::<Vec<_>>();
    images.extend(
        catalog
            .icon_paths
            .iter()
            .flatten()
            .map(|path| asset_server.load(path.clone())),
    );
    commands.insert_resource(PreviewAssets {
        images,
        jeffe_font: asset_server.load(CHARACTER_CREATION_DISPLAY_FONT_PATH),
        chalet_font: asset_server.load(CHARACTER_CREATION_FONT_PATH),
    });
}

fn load_preview_catalog(asset_root: &Path) -> Result<PreviewCatalog, String> {
    let path = asset_root.join("data/character_creation/appearance.json");
    let document: AppearanceDocument = serde_json::from_slice(
        &fs::read(&path).map_err(|error| format!("cannot read {}: {error}", path.display()))?,
    )
    .map_err(|error| format!("invalid {}: {error}", path.display()))?;
    let choice = |category: &str, selector: u8| {
        document
            .choices
            .iter()
            .find(|choice| {
                choice.gender == "male"
                    && choice.category == category
                    && choice.creation_index == selector
            })
            .ok_or_else(|| format!("male/{category}/{selector} is absent from appearance.json"))
    };
    let icon_window = |category: &str, maximum_count: u8| -> Result<[String; 5], String> {
        let minimum = 2_u8;
        let maximum = maximum_count + 1;
        let span = i16::from(maximum - minimum + 1);
        let center = 2_i16;
        let mut paths = Vec::with_capacity(5);
        for offset in -2_i16..=2 {
            let selector = minimum + (center - i16::from(minimum) + offset).rem_euclid(span) as u8;
            let icon = choice(category, selector)?
                .icon
                .as_ref()
                .ok_or_else(|| format!("male/{category}/{selector} has no icon"))?;
            if icon.status != "verified_unique" || icon.candidates.len() != 1 {
                return Err(format!(
                    "male/{category}/{selector} icon is {} with {} candidates",
                    icon.status,
                    icon.candidates.len()
                ));
            }
            let reference = &icon.candidates[0];
            let native = asset_root.join(&reference.path);
            let actual_bytes = fs::metadata(&native)
                .map_err(|error| format!("cannot stat {}: {error}", native.display()))?
                .len();
            if actual_bytes != reference.bytes {
                return Err(format!(
                    "{} bytes changed: expected {}, got {actual_bytes}",
                    native.display(),
                    reference.bytes
                ));
            }
            paths.push(reference.path.clone());
        }
        paths
            .try_into()
            .map_err(|paths: Vec<String>| format!("icon window has {} entries", paths.len()))
    };
    Ok(PreviewCatalog {
        hair_label: choice("hair", 2)?.label.clone(),
        face_label: choice("face", 2)?.label.clone(),
        icon_paths: [
            icon_window("shirt", document.maxima.male_shirts)?,
            icon_window("pants", document.maxima.male_pants)?,
            icon_window("shoes", document.maxima.male_shoes)?,
        ],
    })
}

#[allow(clippy::too_many_arguments)]
fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    fonts: Res<Assets<Font>>,
    options: Res<PreviewOptions>,
    preview_assets: Res<PreviewAssets>,
    language: Res<Language>,
    mut state: ResMut<PreviewState>,
    roots: Query<(&ComputedNode, &Visibility), With<NativeCharacterCreationRoot>>,
    texts: TextAuditQueries,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);
    let failed = preview_assets
        .images
        .iter()
        .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
        || [
            preview_assets.jeffe_font.id(),
            preview_assets.chalet_font.id(),
        ]
        .into_iter()
        .any(|id| matches!(asset_server.load_state(id), LoadState::Failed(_)));
    if failed {
        eprintln!("Character Creation acceptance asset failed to load");
        exit.write(AppExit::error());
        return;
    }
    let assets_loaded = preview_assets
        .images
        .iter()
        .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
        && [
            preview_assets.jeffe_font.id(),
            preview_assets.chalet_font.id(),
        ]
        .into_iter()
        .all(|id| matches!(asset_server.load_state(id), LoadState::Loaded));
    let cpu_assets_present = preview_assets
        .images
        .iter()
        .all(|handle| images.get(handle).is_some())
        && fonts.get(&preview_assets.jeffe_font).is_some()
        && fonts.get(&preview_assets.chalet_font).is_some();
    let root_exact = roots.iter().any(|(node, visibility)| {
        *visibility != Visibility::Hidden && node.size() == Vec2::new(WIDTH as f32, HEIGHT as f32)
    });
    let text_audit = audit_text(&texts, &preview_assets, &options, &language);
    let ready = assets_loaded && cpu_assets_present && root_exact && text_audit.is_ok();
    if ready && state.ready_frame.is_none() {
        state.ready_frame = Some(state.frames);
        state.ready_at = Some(Instant::now());
    }
    let warmed = state
        .ready_frame
        .is_some_and(|frame| state.frames.saturating_sub(frame) >= WARMUP_FRAMES)
        && state
            .ready_at
            .is_some_and(|ready_at| ready_at.elapsed() >= GPU_UPLOAD_GRACE);
    if ready && warmed && !state.capture_issued {
        println!("{}", text_audit.as_ref().expect("ready text audit"));
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
            "Character Creation UI capture timed out: stage={:?}, locale={}, text={:?}",
            options.stage,
            options.locale,
            text_audit.err()
        );
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

fn audit_text(
    queries: &TextAuditQueries,
    assets: &PreviewAssets,
    options: &PreviewOptions,
    language: &Language,
) -> Result<String, String> {
    if language.effective != options.locale {
        return Err(format!(
            "locale mismatch: requested={}, effective={}",
            options.locale, language.effective
        ));
    }
    let all_count = queries.all.iter().count();
    let styled_count = queries.styled.iter().count();
    if all_count == 0 || styled_count != all_count {
        return Err(format!(
            "every-Text ownership incomplete: styled={styled_count}, all={all_count}"
        ));
    }
    let mut visible = 0usize;
    let mut glyphs = 0usize;
    let mut lines = 0usize;
    let mut saw_cyrillic = false;
    let mut styles = Vec::new();
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for (
        entity,
        node,
        text,
        localized,
        style,
        font,
        layout,
        layout_info,
        computed,
        transform,
        inherited,
        parent,
    ) in &queries.styled
    {
        if localized.key.trim().is_empty() {
            return Err(format!("Text {entity:?} has no semantic key"));
        }
        let spec = style.spec();
        let expected_font = match spec.font_role {
            CharacterCreationFontRole::Jeffe => &assets.jeffe_font,
            CharacterCreationFontRole::Chalet => &assets.chalet_font,
        };
        let expected_justify = match spec.anchor {
            CharacterCreationTextAnchor::MiddleLeft => Justify::Left,
            CharacterCreationTextAnchor::MiddleCenter => Justify::Center,
        };
        let metrics_match = if *style == CharacterCreationTextStyle::Toggle {
            let ratio = font.0.font_size.eval(Vec2::ZERO, 16.0) / spec.font_size;
            (0.7..=1.0).contains(&ratio)
                && matches!(
                    *font.1,
                    LineHeight::Px(value)
                        if (value - spec.line_height * ratio).abs() <= 0.01
                )
        } else {
            font.0.font_size.eval(Vec2::ZERO, 16.0) == spec.font_size
                && (*font.1) == LineHeight::Px(spec.line_height)
        };
        if font.0.font != bevy::text::FontSource::Handle(expected_font.clone())
            || !metrics_match
            || layout.justify != expected_justify
            || transform.translation.y != px(0.0)
            || spec.content_offset != [0.0, 0.0]
            || spec.y_offset != 0.0
        {
            return Err(format!(
                "{} bypasses exact {style:?} replacement metrics: font={:?}, expected={:?}, size={:?}/{}, line={:?}/{}, justify={:?}/{:?}, y={:?}",
                localized.key,
                font.0.font,
                expected_font,
                font.0.font_size,
                spec.font_size,
                font.1,
                spec.line_height,
                layout.justify,
                expected_justify,
                transform.translation.y
            ));
        }
        if !inherited.get()
            || computed.size().x <= 0.0
            || computed.size().y <= 0.0
            || text.0.is_empty()
            || layout_info.glyphs.is_empty()
        {
            continue;
        }
        visible += 1;
        glyphs += layout_info.glyphs.len();
        lines += layout_info.run_geometry.len();
        saw_cyrillic |= text
            .0
            .chars()
            .any(|character| ('\u{0400}'..='\u{052f}').contains(&character));
        if !styles.contains(style) {
            styles.push(*style);
        }
        let source_rect = if node.position_type == PositionType::Absolute {
            computed
        } else {
            queries
                .computed
                .get(parent.parent())
                .map_err(|_| format!("{} has no source Rect parent", localized.key))?
        };
        if !layout_info.size.is_finite()
            || (!spec.clip
                && (layout_info.size.x > source_rect.size().x + 2.0
                    || layout_info.size.y > source_rect.size().y + 2.0))
        {
            return Err(format!(
                "{} GPU bounds {:?} exceed unclipped source Rect {:?}",
                localized.key,
                layout_info.size,
                source_rect.size()
            ));
        }
        if *style == CharacterCreationTextStyle::Toggle && layout_info.size.x > 60.0 {
            return Err(format!(
                "{} localized Toggle text width {} overlaps the next 72px clean hit Rect",
                localized.key, layout_info.size.x
            ));
        }
        let mut minimum = Vec2::splat(f32::INFINITY);
        let mut maximum = Vec2::splat(f32::NEG_INFINITY);
        for glyph in &layout_info.glyphs {
            minimum = minimum.min(glyph.position - glyph.atlas_info.rect.size() * 0.5);
            maximum = maximum.max(glyph.position + glyph.atlas_info.rect.size() * 0.5);
        }
        if !minimum.is_finite() || !maximum.is_finite() {
            return Err(format!("{} has non-finite glyph bounds", localized.key));
        }
        min_y = min_y.min(minimum.y);
        max_y = max_y.max(maximum.y);
        // Decoration bounds may exceed line height when Parley clamps negative leading.
        // Verify the actual shaped line metrics, not the selection/decorations box.
        for shaped_line in font.2.buffer().lines() {
            let line = shaped_line.metrics();
            let expected_line_height = match *font.1 {
                LineHeight::Px(value) => value,
                LineHeight::RelativeToFont(value) => {
                    value * font.0.font_size.eval(Vec2::ZERO, 16.0)
                }
            };
            if !line.baseline.is_finite()
                || !line.line_height.is_finite()
                || (line.line_height - expected_line_height).abs() > 0.55
            {
                return Err(format!(
                    "{} invalid line/baseline box {:?} for {}",
                    localized.key, line, expected_line_height
                ));
            }
        }
    }
    if visible == 0 || glyphs == 0 || lines == 0 {
        return Err("visible GPU text has not been laid out".to_owned());
    }
    let required: &[CharacterCreationTextStyle] = match options.stage {
        PreviewStage::Generated => &[
            CharacterCreationTextStyle::Label,
            CharacterCreationTextStyle::Transparent,
            CharacterCreationTextStyle::TabButton,
            CharacterCreationTextStyle::TabText,
            CharacterCreationTextStyle::OrText,
            CharacterCreationTextStyle::NameDisplay,
            CharacterCreationTextStyle::ButtonTabFont,
            CharacterCreationTextStyle::Transparent4,
            CharacterCreationTextStyle::ExitButton,
            CharacterCreationTextStyle::Button,
        ],
        PreviewStage::Custom => &[
            CharacterCreationTextStyle::Label,
            CharacterCreationTextStyle::TabButton,
            CharacterCreationTextStyle::TabText,
            CharacterCreationTextStyle::OrText,
            CharacterCreationTextStyle::CustomQuestion,
            CharacterCreationTextStyle::Transparent5,
            CharacterCreationTextStyle::TextField,
            CharacterCreationTextStyle::Transparent4,
            CharacterCreationTextStyle::ExitButton,
            CharacterCreationTextStyle::Button,
        ],
        PreviewStage::Appearance => &[
            CharacterCreationTextStyle::Transparent,
            CharacterCreationTextStyle::SectionLabel,
            CharacterCreationTextStyle::Toggle,
            CharacterCreationTextStyle::BodyText,
            CharacterCreationTextStyle::ButtonTabFont,
            CharacterCreationTextStyle::ExitButton,
            CharacterCreationTextStyle::Button,
        ],
    };
    for style in required {
        if !styles.contains(style) {
            return Err(format!("no visible {style:?} text"));
        }
    }
    if options.locale == "ru" && !saw_cyrillic {
        return Err("RU bundle produced no visible Cyrillic glyphs".to_owned());
    }
    styles.sort_by_key(|style| format!("{style:?}"));
    Ok(format!(
        "Character Creation UI GPU text audit: stage={}, locale={}, key-first={}/{}, visible={}, glyphs={}, line-boxes={}, glyph-y={:.1}..{:.1}, styles={styles:?}",
        options.stage.slug(),
        options.locale,
        styled_count,
        all_count,
        visible,
        glyphs,
        lines,
        min_y,
        max_y,
    ))
}

fn save_screenshot(
    event: On<ScreenshotCaptured>,
    options: Res<PreviewOptions>,
    mut state: ResMut<PreviewState>,
) {
    let image = event
        .image
        .clone()
        .try_into_dynamic()
        .expect("GPU screenshot must convert to a dynamic image");
    let rgba = image.to_rgba8();
    if rgba.dimensions() != (WIDTH, HEIGHT) {
        eprintln!("unexpected capture size {:?}", rgba.dimensions());
        state.capture_failed = true;
        return;
    }
    let visible_pixels = rgba
        .pixels()
        .filter(|pixel| {
            let [red, green, blue, _alpha] = pixel.0;
            u16::from(red) + u16::from(green) + u16::from(blue) > 25
        })
        .count();
    if visible_pixels < MIN_VISIBLE_PIXELS {
        eprintln!("capture is incomplete: {visible_pixels} visible pixels");
        state.capture_failed = true;
        return;
    }
    if let Some(parent) = options
        .output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).unwrap();
    }
    image.save(&options.output).unwrap();
    println!(
        "capture={}",
        options
            .output
            .canonicalize()
            .unwrap_or_else(|_| options.output.clone())
            .display()
    );
    state.capture_saved = true;
}
