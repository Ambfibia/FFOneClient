//! Deterministic 1264x681 GPU acceptance frame for the clean-Retrobution
//! skill-buff icon rows.

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
    window::{PresentMode, WindowResolution},
};
use ffone_client::{
    assets::AssetLocator,
    localization::{Localization, LocalizationPlugin, LocalizedText},
    skill_buff_ui::{
        SKILL_BUFF_BACK_PATH, SKILL_BUFF_FONT_PATH, SKILL_BUFF_ICON_SIZE, SKILL_BUFF_MAX_ICONS,
        SkillBuffTargetUi, SkillBuffTextAnchor, SkillBuffTextStyle, SkillBuffUiCatalog,
        SkillBuffUiModel, SkillBuffUiPlugin, SkillBuffUiRoot, skill_buff_ui_view,
    },
    tutorial_mission_content::TutorialMissionContent,
};

const CLIENT_AREA_WIDTH: u32 = 1_264;
const CLIENT_AREA_HEIGHT: u32 = 681;
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(20);
const MIN_VISIBLE_PIXELS: usize = 250;
const EXPECTED_TEXT_COUNT: usize = SKILL_BUFF_MAX_ICONS;
const EXPECTED_VISIBLE_TEXT_COUNT: usize = 2;

#[derive(Resource)]
struct PreviewOutput(PathBuf);

#[derive(Resource)]
struct PreviewOptions {
    output: PathBuf,
    language: String,
}

#[derive(Resource)]
struct PreviewAssets {
    images: Vec<Handle<Image>>,
    font: Handle<Font>,
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
    let options = parse_options();

    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game")
        .canonicalize()
        .expect("assets/game must exist");
    let project_assets =
        AssetLocator::open(&asset_root).expect("open native Retrobution project assets");
    let content = TutorialMissionContent::open(&project_assets)
        .expect("load native Retrobution TableData projection");
    let catalog = SkillBuffUiCatalog::open(&content, &project_assets)
        .expect("resolve exact skill-buff icon catalog");
    let (localization, language) = Localization::open(&asset_root, &options.language)
        .expect("open production localization bundles");

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(PreviewOutput(options.output.clone()))
        .insert_resource(options)
        .insert_resource(PreviewState::default())
        .insert_resource(catalog)
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
                        title: "FFOne Retrobution skill-buff UI acceptance".into(),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((SkillBuffUiPlugin, LocalizationPlugin))
        .add_systems(Startup, setup_preview)
        .add_systems(Update, drive_capture)
        .run();
}

fn parse_options() -> PreviewOptions {
    let mut output = None;
    let mut language = "en".to_owned();
    let mut args = std::env::args_os().skip(1);
    while let Some(argument) = args.next() {
        if argument == "--language" {
            let Some(value) = args.next().and_then(|value| value.into_string().ok()) else {
                eprintln!("--language requires en or ru");
                std::process::exit(2);
            };
            if value != "en" && value != "ru" {
                eprintln!("--language requires en or ru");
                std::process::exit(2);
            }
            language = value;
        } else if output.is_none() {
            output = Some(PathBuf::from(argument));
        } else {
            eprintln!("usage: skill_buff_ui_gpu_preview [OUTPUT.png] [--language en|ru]");
            std::process::exit(2);
        }
    }
    let output = output.unwrap_or_else(|| {
        PathBuf::from(format!(
            "target/ui-parity/skill-buffs-{language}-1264x681.png"
        ))
    });
    if output.extension().and_then(|value| value.to_str()) != Some("png") {
        eprintln!("output must end in .png");
        std::process::exit(2);
    }
    PreviewOptions { output, language }
}

fn sample_model() -> SkillBuffUiModel {
    let mut model = SkillBuffUiModel::default();
    model.visible = true;
    model.ui_scale = 1.0;
    // Three debuffs followed by four buffs. Stim flags are deliberately
    // absent because the clean local row redirects them to Nano gumballs.
    model.local_condition_bit_flag = 128 | 512 | 65_536 | 1 | 4 | 64 | 8_388_608;
    // Cash draws only the buff list and is the only row with countdowns.
    model.cash_condition_bit_flag = 8_192 | 32_768;
    // A target Stim flag resolves through the local Nano style.
    model.target = Some(SkillBuffTargetUi {
        character_type: 2,
        character_id: 77,
        condition_bit_flag: 1_048_576,
    });
    model.nano_styles = [Some(0), Some(1), Some(2)];
    model.set_cash_remaining_ms(14, 125_000);
    // Keep the sample away from a unit boundary while the real clean coarse
    // countdown advances during GPU warmup.
    model.set_cash_remaining_ms(16, 5_400_000);
    model
}

fn setup_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    catalog: Res<SkillBuffUiCatalog>,
    mut model: ResMut<SkillBuffUiModel>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    *model = sample_model();

    let mut paths = BTreeSet::from([SKILL_BUFF_BACK_PATH.to_owned()]);
    let view = skill_buff_ui_view(CLIENT_AREA_WIDTH, &model, &catalog);
    for icon in view.local.iter().chain(&view.cash).chain(&view.target) {
        paths.insert(icon.icon_path.clone());
    }
    commands.insert_resource(PreviewAssets {
        images: paths
            .into_iter()
            .map(|path| asset_server.load(path))
            .collect(),
        font: asset_server.load(SKILL_BUFF_FONT_PATH),
    });
}

fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    fonts: Res<Assets<Font>>,
    options: Res<PreviewOptions>,
    preview_assets: Res<PreviewAssets>,
    roots: Query<&ComputedNode, With<SkillBuffUiRoot>>,
    all_texts: Query<&Text>,
    styled_texts: Query<(
        Entity,
        &Text,
        &LocalizedText,
        &SkillBuffTextStyle,
        (&TextFont, &LineHeight),
        &TextLayout,
        &TextLayoutInfo,
        &ComputedNode,
        &UiTransform,
        &UiGlobalTransform,
        &InheritedVisibility,
        &ChildOf,
    )>,
    computed_nodes: Query<(&ComputedNode, &UiGlobalTransform, Option<&ChildOf>)>,
    mut state: ResMut<PreviewState>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);
    let failed = preview_assets
        .images
        .iter()
        .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
        || matches!(
            asset_server.load_state(preview_assets.font.id()),
            LoadState::Failed(_)
        );
    if failed {
        eprintln!("skill-buff UI source asset failed to load");
        exit.write(AppExit::error());
        return;
    }

    let loaded = preview_assets
        .images
        .iter()
        .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
        && matches!(
            asset_server.load_state(preview_assets.font.id()),
            LoadState::Loaded
        )
        && preview_assets
            .images
            .iter()
            .all(|handle| images.get(handle).is_some())
        && fonts.get(&preview_assets.font).is_some()
        && roots.iter().any(|node| {
            node.size() == Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32)
        });
    if loaded && state.ready_frame.is_none() {
        state.ready_frame = Some(state.frames);
        state.ready_at = Some(Instant::now());
    }
    let warmed = state
        .ready_frame
        .is_some_and(|frame| state.frames.saturating_sub(frame) >= WARMUP_FRAMES_AFTER_LOAD)
        && state
            .ready_at
            .is_some_and(|ready_at| ready_at.elapsed() >= GPU_UPLOAD_GRACE);
    if loaded && warmed && !state.text_audited {
        match audit_text(
            &all_texts,
            &styled_texts,
            &computed_nodes,
            &preview_assets,
            &options,
        ) {
            Ok(summary) => {
                println!("{summary}");
                state.text_audited = true;
            }
            Err(error) => {
                eprintln!("skill-buff UI GPU text audit failed: {error}");
                state.capture_failed = true;
            }
        }
    }
    if loaded && warmed && state.text_audited && !state.capture_issued {
        state.capture_issued = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }
    if state.capture_saved {
        exit.write(AppExit::Success);
    } else if state.capture_failed || state.started_at.elapsed() >= CAPTURE_TIMEOUT {
        if !state.capture_failed {
            eprintln!("skill-buff UI capture timed out");
        }
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

fn audit_text(
    all_texts: &Query<'_, '_, &Text>,
    styled_texts: &Query<
        '_,
        '_,
        (
            Entity,
            &Text,
            &LocalizedText,
            &SkillBuffTextStyle,
            (&TextFont, &LineHeight),
            &TextLayout,
            &TextLayoutInfo,
            &ComputedNode,
            &UiTransform,
            &UiGlobalTransform,
            &InheritedVisibility,
            &ChildOf,
        ),
    >,
    computed_nodes: &Query<'_, '_, (&ComputedNode, &UiGlobalTransform, Option<&ChildOf>)>,
    assets: &PreviewAssets,
    options: &PreviewOptions,
) -> Result<String, String> {
    let all_count = all_texts.iter().count();
    let rows = styled_texts.iter().collect::<Vec<_>>();
    if all_count != EXPECTED_TEXT_COUNT || rows.len() != all_count {
        return Err(format!(
            "every-Text ownership incomplete: styled={}, all={all_count}, expected={EXPECTED_TEXT_COUNT}",
            rows.len()
        ));
    }

    let mut visible = 0usize;
    let mut glyphs = 0usize;
    let mut line_boxes = 0usize;
    let mut visible_values = BTreeSet::new();
    let mut glyph_min_y = f32::INFINITY;
    let mut glyph_max_y = f32::NEG_INFINITY;
    for (
        entity,
        text,
        localized,
        style,
        font,
        text_layout,
        layout,
        computed,
        transform,
        global,
        inherited_visibility,
        parent,
    ) in rows
    {
        let spec = style.spec();
        if localized.key != "ui.skill_buff.cash_time"
            || localized.fallback != "{time}"
            || localized.args.keys().map(String::as_str).ne(["time"])
        {
            return Err(format!(
                "{entity:?} bypasses the semantic cash-time template: {localized:?}"
            ));
        }
        if *style != SkillBuffTextStyle::HudLabel
            || spec.anchor != SkillBuffTextAnchor::MiddleLeft
            || spec.source_style != "label"
            || spec.source_skin_path_id != 1_372
            || spec.source_font_path_id != 1_018
            || spec.padding != [0.0; 4]
            || !spec.word_wrap
            || !spec.text_clipping
            || spec.y_offset != 0.0
        {
            return Err(format!("{entity:?} lost the clean HUD label contract"));
        }
        if font.0.font != bevy::text::FontSource::Handle(assets.font.clone())
            || (font.0.font_size.eval(Vec2::ZERO, 16.0) - spec.font_size).abs() > 0.01
            || (*font.1) != LineHeight::Px(spec.line_height)
            || text_layout.justify != spec.justify
            || text_layout.linebreak != spec.linebreak
            || transform.translation != Val2::px(0.0, 0.0)
        {
            return Err(format!(
                "{entity:?} has wrong replacement font/layout metrics: size={}, line={:?}, translation={:?}",
                font.0.font_size.eval(Vec2::ZERO, 16.0),
                (*font.1),
                transform.translation
            ));
        }

        if !inherited_visibility.get()
            || text.0.is_empty()
            || computed.size().x <= 0.0
            || layout.glyphs.is_empty()
        {
            continue;
        }
        visible += 1;
        visible_values.insert(text.0.clone());
        glyphs += layout.glyphs.len();
        line_boxes += layout.run_geometry.len();
        if layout.run_geometry.len() != 1 {
            return Err(format!(
                "{} wrapped to {} lines inside the clean 26x26 Rect",
                localized.key,
                layout.run_geometry.len()
            ));
        }

        let (label_node, label_global, label_parent) = computed_nodes
            .get(parent.parent())
            .map_err(|_| format!("{entity:?} has no cash label Rect"))?;
        let label_parent = label_parent
            .ok_or_else(|| format!("{entity:?} cash label Rect has no icon-slot parent"))?;
        let (slot_node, slot_global, _) = computed_nodes
            .get(label_parent.parent())
            .map_err(|_| format!("{entity:?} has no icon-slot Rect"))?;
        if (label_node.size() - Vec2::splat(SKILL_BUFF_ICON_SIZE))
            .abs()
            .max_element()
            > 0.01
            || (slot_node.size() - Vec2::splat(SKILL_BUFF_ICON_SIZE))
                .abs()
                .max_element()
                > 0.01
        {
            return Err(format!(
                "{entity:?} source Rects are not exact 26x26: label={:?}, slot={:?}",
                label_node.size(),
                slot_node.size()
            ));
        }
        let label_top_left = label_global.translation - label_node.size() * 0.5;
        let slot_top_left = slot_global.translation - slot_node.size() * 0.5;
        let label_offset = label_top_left - slot_top_left;
        if (label_offset - Vec2::new(2.0, 19.0 + spec.y_offset))
            .abs()
            .max_element()
            > 0.75
        {
            return Err(format!(
                "{entity:?} cash label Rect offset {label_offset:?} != clean (2,19)"
            ));
        }
        let text_top_left = global.translation - computed.size() * 0.5;
        let actual_text_offset = text_top_left - label_top_left;
        let expected_text_offset = Vec2::new(0.0, (label_node.size().y - computed.size().y) * 0.5);
        if (actual_text_offset - expected_text_offset)
            .abs()
            .max_element()
            > 0.75
        {
            return Err(format!(
                "{entity:?} MiddleLeft baseline offset {actual_text_offset:?} != {expected_text_offset:?}"
            ));
        }
        if !layout.size.is_finite()
            || layout.size.x > label_node.size().x + 1.0
            || layout.size.y > label_node.size().y + 1.0
        {
            return Err(format!(
                "{} layout {:?} exceeds source Rect {:?}",
                localized.key,
                layout.size,
                label_node.size()
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
                "{} glyph bounds {minimum:?}..{maximum:?} exceed text node {:?}",
                localized.key,
                computed.size()
            ));
        }
        glyph_min_y = glyph_min_y.min(minimum.y);
        glyph_max_y = glyph_max_y.max(maximum.y);
        for run in &layout.run_geometry {
            let line = run.bounds;
            if !line.min.is_finite()
                || !line.max.is_finite()
                || line.min.y < -0.5
                || line.max.y > layout.size.y + 0.5
                || (line.height() - spec.line_height).abs() > 0.75
            {
                return Err(format!(
                    "{} invalid line box {line:?} for {}",
                    localized.key, spec.line_height
                ));
            }
        }
    }

    if visible != EXPECTED_VISIBLE_TEXT_COUNT
        || glyphs == 0
        || line_boxes != EXPECTED_VISIBLE_TEXT_COUNT
        || visible_values != BTreeSet::from(["1h".to_owned(), "2m".to_owned()])
    {
        return Err(format!(
            "visible cash timers mismatch: visible={visible}, glyphs={glyphs}, lines={line_boxes}, values={visible_values:?}"
        ));
    }
    Ok(format!(
        "Skill-buff GPU text audit: locale={}, key-first={}/{}, visible={}, glyphs={}, line-boxes={}, glyph-y={glyph_min_y:.1}..{glyph_max_y:.1}, style=FusionFallHUDSkin.label, y-offset=0",
        options.language, all_count, EXPECTED_TEXT_COUNT, visible, glyphs, line_boxes
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
    let visible_pixels = image
        .to_rgba8()
        .pixels()
        .filter(|pixel| {
            let [red, green, blue, _alpha] = pixel.0;
            u16::from(red) + u16::from(green) + u16::from(blue) > 35
        })
        .count();
    if visible_pixels < MIN_VISIBLE_PIXELS {
        eprintln!("rejecting empty skill-buff UI capture: {visible_pixels} visible pixels");
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
