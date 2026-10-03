//! Deterministic 1264×681 acceptance frame for clean-Retrobution TransportMode.

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
    time::TimeUpdateStrategy,
    window::{PresentMode, WindowResolution},
};
use ffone_client::{
    assets::AssetLocator,
    localization::{Localization, LocalizationPlugin, LocalizationSet, LocalizedText},
    transportation_ui::{
        TransportationCatalog, TransportationModel, TransportationOpenContext, TransportationPhase,
        TransportationPlayerSnapshot, TransportationPresentationAssetStatus,
        TransportationPresentationMarker, TransportationPresentationRoot,
        TransportationPresentationRouteRow, TransportationPresentationSet,
        TransportationPresentationWindow, TransportationTarget, TransportationUiPlugin,
        TransportationUiTextStyle, TransportationUnlocks, TransportationWorldPoint,
    },
    world_map::WORLD_MAP_JEFFE_FONT_PATH,
};

const CLIENT_AREA_WIDTH: u32 = 1_264;
const CLIENT_AREA_HEIGHT: u32 = 681;
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(25);
const MIN_VISIBLE_PIXELS: usize = 180_000;
const EXPECTED_TEXT_COUNT: usize = 26;
const EXPECTED_VISIBLE_TEXT_COUNT: usize = 26;

#[derive(Resource)]
struct PreviewOutput(PathBuf);

#[derive(Resource)]
struct PreviewLanguage(String);

#[derive(Resource)]
struct PreviewAssets {
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct PreviewCli {
    language: String,
    output: PathBuf,
}

fn default_output(language: &str) -> PathBuf {
    PathBuf::from(format!(
        "target/ui-parity/transportation-mode-{language}-1264x681.png"
    ))
}

fn main() {
    let cli = match parse_cli(std::env::args_os().skip(1)) {
        Ok(cli) => cli,
        Err(usage) => {
            eprintln!("{usage}");
            std::process::exit(2);
        }
    };
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game")
        .canonicalize()
        .expect("assets/game must exist");
    let model = build_preview_model(&asset_root).expect("build transportation preview model");
    let (localization, language) = Localization::open(&asset_root, &cli.language)
        .expect("open production localization bundles");

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        // Freeze the two clean repaint-only line effects at phase zero. Asset
        // IO completion must not change acceptance pixels between runs.
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
        .insert_resource(model)
        .insert_resource(localization)
        .insert_resource(language)
        .insert_resource(PreviewOutput(cli.output))
        .insert_resource(PreviewLanguage(cli.language))
        .insert_resource(PreviewState::default())
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne Retrobution Transportation acceptance".to_owned(),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((TransportationUiPlugin, LocalizationPlugin))
        .add_systems(Startup, setup_preview)
        .add_systems(
            Update,
            drive_capture
                .after(TransportationPresentationSet::Bind)
                .after(LocalizationSet::Apply),
        )
        .run();
}

fn parse_cli(args: impl IntoIterator<Item = OsString>) -> Result<PreviewCli, &'static str> {
    let mut args = args.into_iter();
    let language = args
        .next()
        .and_then(|value| value.into_string().ok())
        .unwrap_or_else(|| "en".to_owned());
    if !matches!(language.as_str(), "en" | "ru") {
        return Err("usage: transportation_ui_gpu_preview [en|ru] [OUTPUT.png]");
    }
    let output = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| default_output(&language));
    if args.next().is_some() || output.extension().and_then(|value| value.to_str()) != Some("png") {
        return Err("usage: transportation_ui_gpu_preview [en|ru] [OUTPUT.png]");
    }
    Ok(PreviewCli { language, output })
}

fn build_preview_model(asset_root: &Path) -> Result<TransportationModel, String> {
    let assets =
        AssetLocator::open(asset_root).map_err(|error| format!("open project assets: {error}"))?;
    let catalog = TransportationCatalog::open(&assets)
        .map_err(|error| format!("load clean transportation catalog: {error}"))?;
    let mut model = TransportationModel::default();
    model
        .open(
            &catalog,
            TransportationOpenContext {
                player: TransportationPlayerSnapshot {
                    position: TransportationWorldPoint::new(2_246.94, 105.0, 2_290.48),
                    taros: 800,
                    unlocks: TransportationUnlocks {
                        warp_location_flags: u32::MAX,
                        wyvern_location_flags: [u64::MAX, u64::MAX],
                    },
                    cursor_was_locked: false,
                },
                // City Station Monkey Skyway agent: class 16, seven clean
                // TableData routes, enough to exercise the exact scrollbar.
                target: TransportationTarget::Npc {
                    npc_instance_id: 26_190,
                    npc_table_id: 2_619,
                    npc_position: TransportationWorldPoint::new(2_246.94, 105.0, 2_290.48),
                    has_move_ok_voice: true,
                },
            },
        )
        .map_err(|error| format!("open transportation model: {error:?}"))?;
    while model.pop_outbox().is_some() {}
    model
        .toggle_turbo()
        .map_err(|error| format!("enable turbo: {error:?}"))?;
    while model.pop_outbox().is_some() {}
    model
        .select_route(3)
        .map_err(|error| format!("select route: {error:?}"))?;
    for _ in 0..120 {
        model
            .advance_map_paint()
            .map_err(|error| format!("settle map: {error:?}"))?;
    }
    Ok(model)
}

fn setup_preview(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    commands.insert_resource(PreviewAssets {
        font: asset_server.load(WORLD_MAP_JEFFE_FONT_PATH),
    });
}

#[allow(clippy::too_many_arguments)]
fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    fonts: Res<Assets<Font>>,
    preview_assets: Res<PreviewAssets>,
    status: Res<TransportationPresentationAssetStatus>,
    model: Res<TransportationModel>,
    language: Res<PreviewLanguage>,
    mut state: ResMut<PreviewState>,
    roots: Query<&ComputedNode, With<TransportationPresentationRoot>>,
    windows: Query<
        &ComputedNode,
        (
            With<TransportationPresentationWindow>,
            Without<TransportationPresentationRoot>,
        ),
    >,
    rows: Query<&TransportationPresentationRouteRow>,
    markers: Query<&TransportationPresentationMarker>,
    texts: Query<(
        Entity,
        &Text,
        &LocalizedText,
        &TransportationUiTextStyle,
        (&TextFont, &LineHeight),
        &TextLayout,
        &TextLayoutInfo,
        &Node,
        &ComputedNode,
        &UiTransform,
    )>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);
    if let TransportationPresentationAssetStatus::Failed { asset_path } = *status {
        eprintln!("transportation source asset failed to load: {asset_path}");
        exit.write(AppExit::error());
        return;
    }
    if matches!(
        asset_server.load_state(preview_assets.font.id()),
        LoadState::Failed(_)
    ) {
        eprintln!("transportation replacement font failed to load");
        exit.write(AppExit::error());
        return;
    }
    let font_loaded = matches!(
        asset_server.load_state(preview_assets.font.id()),
        LoadState::Loaded
    ) && fonts.get(&preview_assets.font).is_some();
    let layout_ready = roots
        .single()
        .is_ok_and(|node| node.size() == Vec2::new(1_264.0, 681.0))
        && windows
            .single()
            .is_ok_and(|node| node.size() == Vec2::new(1_036.0, 654.0));
    let loaded = matches!(*status, TransportationPresentationAssetStatus::Ready)
        && model.phase() == TransportationPhase::Browsing
        && layout_ready
        && rows.iter().count() == model.routes().len()
        && markers.iter().count() >= 3;
    let text_audit = audit_transportation_text(&texts, &preview_assets, &language.0);
    if font_loaded && !state.text_audited {
        match &text_audit {
            Ok(report) => {
                println!("{report}");
                state.text_audited = true;
            }
            Err(error) if state.frames % 120 == 0 => {
                eprintln!("TransportMode text audit waiting: {error}");
            }
            Err(_) => {}
        }
    }
    let loaded = loaded && font_loaded && text_audit.is_ok();
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
    if loaded && warmed && !state.capture_issued {
        state.capture_issued = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }
    if state.capture_saved {
        exit.write(AppExit::Success);
    } else if state.capture_failed || state.started_at.elapsed() >= CAPTURE_TIMEOUT {
        if !state.capture_failed {
            eprintln!("transportation capture timed out");
        }
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

fn audit_transportation_text(
    texts: &Query<
        '_,
        '_,
        (
            Entity,
            &Text,
            &LocalizedText,
            &TransportationUiTextStyle,
            (&TextFont, &LineHeight),
            &TextLayout,
            &TextLayoutInfo,
            &Node,
            &ComputedNode,
            &UiTransform,
        ),
    >,
    preview_assets: &PreviewAssets,
    language: &str,
) -> Result<String, String> {
    let rows = texts.iter().collect::<Vec<_>>();
    if rows.len() != EXPECTED_TEXT_COUNT {
        return Err(format!(
            "key-first Text count {}, expected {EXPECTED_TEXT_COUNT}",
            rows.len()
        ));
    }

    let mut visible = 0usize;
    let mut glyphs = 0usize;
    let mut line_boxes = 0usize;
    let mut saw_cyrillic = false;
    let mut styles = Vec::new();
    let mut glyph_min_y = f32::INFINITY;
    let mut glyph_max_y = f32::NEG_INFINITY;
    for (_entity, text, localized, style, font, text_layout, layout, node, computed, transform) in
        rows
    {
        let (expected_args, expected_style): (&[&str], TransportationUiTextStyle) =
            match localized.key.as_str() {
                "ui.transportation.title" | "ui.transportation.where_to" => {
                    (&[], TransportationUiTextStyle::BigFont16UpperLeft)
                }
                "ui.transportation.subtitle" => (
                    &["name", "region"],
                    TransportationUiTextStyle::BigFont14UpperLeft,
                ),
                "ui.transportation.subtitle.empty"
                | "ui.transportation.turbo_travel"
                | "ui.transportation.route.name"
                | "ui.transportation.route.region" => {
                    let args: &[&str] = match localized.key.as_str() {
                        "ui.transportation.route.name" => &["name"],
                        "ui.transportation.route.region" => &["region"],
                        _ => &[],
                    };
                    (args, TransportationUiTextStyle::BigFont14UpperLeft)
                }
                "ui.transportation.go_now" => (&[], TransportationUiTextStyle::ButtonMiddleCenter),
                "ui.transportation.cost" => {
                    (&["amount"], TransportationUiTextStyle::RightTextUpperRight)
                }
                "ui.transportation.unregistered" => {
                    (&[], TransportationUiTextStyle::RightTextUpperRight)
                }
                key => return Err(format!("unexpected Transportation key {key}")),
            };
        if localized
            .args
            .keys()
            .map(String::as_str)
            .ne(expected_args.iter().copied())
        {
            return Err(format!(
                "{} template args {:?} != {:?}",
                localized.key,
                localized.args.keys().collect::<Vec<_>>(),
                expected_args
            ));
        }
        if *style != expected_style {
            return Err(format!(
                "{} uses {style:?}, expected {expected_style:?}",
                localized.key
            ));
        }
        if font.0.font != bevy::text::FontSource::Handle(preview_assets.font.clone()) {
            return Err(format!(
                "{} uses the wrong JEFFE replacement font",
                localized.key
            ));
        }
        if (font.0.font_size.eval(Vec2::ZERO, 16.0) - style.font_size()).abs() > 0.01 {
            return Err(format!(
                "{} font size {} != {}",
                localized.key,
                font.0.font_size.eval(Vec2::ZERO, 16.0),
                style.font_size()
            ));
        }
        match (*font.1) {
            LineHeight::Px(actual) if (actual - style.line_height()).abs() <= 0.01 => {}
            actual => {
                return Err(format!(
                    "{} line height {actual:?} != {}",
                    localized.key,
                    style.line_height()
                ));
            }
        }
        if transform.translation != Val2::px(0.0, style.replacement_y_offset()) {
            return Err(format!(
                "{} has unexpected replacement-font offset {:?}",
                localized.key, transform.translation
            ));
        }
        let [left, right, top, bottom] = style.padding();
        if node.padding != UiRect::new(px(left), px(right), px(top), px(bottom)) {
            return Err(format!(
                "{} padding {:?} != {:?}",
                localized.key,
                node.padding,
                style.padding()
            ));
        }
        if style.content_offset() != [0.0, 0.0] {
            return Err(format!("{} has nonzero clean contentOffset", localized.key));
        }
        let expected_justify = match style {
            TransportationUiTextStyle::BigFont14UpperLeft
            | TransportationUiTextStyle::BigFont16UpperLeft => Justify::Left,
            TransportationUiTextStyle::RightTextUpperRight => Justify::Right,
            TransportationUiTextStyle::ButtonMiddleCenter => Justify::Center,
        };
        let expected_linebreak = if style.word_wrap() {
            LineBreak::WordBoundary
        } else {
            LineBreak::NoWrap
        };
        if text_layout.justify != expected_justify || text_layout.linebreak != expected_linebreak {
            return Err(format!(
                "{} layout {:?} does not match {style:?}",
                localized.key, text_layout
            ));
        }

        if text.0.is_empty() || computed.size().x <= 0.0 || layout.glyphs.is_empty() {
            continue;
        }
        visible += 1;
        glyphs += layout.glyphs.len();
        line_boxes += layout.run_geometry.len();
        saw_cyrillic |= text.0.chars().any(|character| {
            ('\u{0400}'..='\u{052f}').contains(&character)
                || ('\u{2de0}'..='\u{2dff}').contains(&character)
        });
        if !styles.contains(style) {
            styles.push(*style);
        }
        if layout.run_geometry.len() != 1 {
            return Err(format!(
                "{} produced {} lines inside the primary Rect",
                localized.key,
                layout.run_geometry.len()
            ));
        }
        if !layout.size.is_finite()
            || layout.size.x > computed.size().x + 1.0
            || layout.size.y > computed.size().y + 1.0
        {
            return Err(format!(
                "{} layout {:?} exceeds text node {:?}",
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
                "{} glyphs {:?}..{:?} exceed text node {:?}",
                localized.key,
                minimum,
                maximum,
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
                || (line.height() - style.line_height()).abs() > 0.75
            {
                return Err(format!(
                    "{} invalid line box {:?} for {}",
                    localized.key,
                    line,
                    style.line_height()
                ));
            }
        }
    }
    if visible != EXPECTED_VISIBLE_TEXT_COUNT || glyphs == 0 || line_boxes == 0 {
        return Err(format!(
            "visible TransportMode GPU text count {visible}, expected {EXPECTED_VISIBLE_TEXT_COUNT}"
        ));
    }
    for required in [
        TransportationUiTextStyle::BigFont14UpperLeft,
        TransportationUiTextStyle::BigFont16UpperLeft,
        TransportationUiTextStyle::RightTextUpperRight,
        TransportationUiTextStyle::ButtonMiddleCenter,
    ] {
        if !styles.contains(&required) {
            return Err(format!("visible TransportMode has no {required:?}"));
        }
    }
    if language == "ru" && !saw_cyrillic {
        return Err("RU TransportMode produced no visible Cyrillic glyphs".to_owned());
    }
    styles.sort_by_key(|style| format!("{style:?}"));
    Ok(format!(
        "TransportMode GPU text audit: locale={language}, key-first={}, visible={visible}, \
         glyphs={glyphs}, line-boxes={line_boxes}, glyph-y={glyph_min_y:.1}..{glyph_max_y:.1}, \
         styles={styles:?}",
        EXPECTED_TEXT_COUNT
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
    if image.width() != CLIENT_AREA_WIDTH || image.height() != CLIENT_AREA_HEIGHT {
        eprintln!(
            "rejecting transportation capture at {}x{}",
            image.width(),
            image.height()
        );
        state.capture_failed = true;
        return;
    }
    let visible_pixels = image
        .to_rgba8()
        .pixels()
        .filter(|pixel| {
            let [red, green, blue, _alpha] = pixel.0;
            u16::from(red) + u16::from(green) + u16::from(blue) > 35
        })
        .count();
    if visible_pixels < MIN_VISIBLE_PIXELS {
        eprintln!("rejecting empty transportation capture: {visible_pixels} visible pixels");
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
