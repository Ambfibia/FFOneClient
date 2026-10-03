//! Deterministic GPU acceptance frame for clean-Retrobution `WorldMapMode`.
//!
//! The preview only supplies a completed pure model/marker projection to the
//! read-only presentation plugin. It deliberately owns no click-to-waypoint,
//! warp, packet, or server behavior.

use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use bevy::{
    asset::AssetPlugin,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    text::{LineHeight, TextLayoutInfo},
    window::{PresentMode, WindowResolution},
};
use ffone_client::{
    localization::{Localization, LocalizationPlugin, LocalizedText},
    world_map::{
        WorldMapCatalog, WorldMapCatalogLookup, WorldMapInputGates, WorldMapMarker,
        WorldMapMissionAvailability, WorldMapModel, WorldMapNpcCatalogEntry, WorldMapNpcSource,
        WorldMapOpenContext, WorldMapPhase, WorldMapPlayer, WorldMapPoint, WorldMapPresentation,
        WorldMapPresentationAssetStatus, WorldMapPresentationControl, WorldMapPresentationHover,
        WorldMapPresentationMarker, WorldMapPresentationPlugin, WorldMapPresentationRoot,
        WorldMapPresentationWindow, WorldMapScanAnimation, WorldMapTextStyle, WorldMapUiPoint,
        WorldMapZoom, WorldMapZoomDirection,
    },
};

const CLIENT_AREA_WIDTH: u32 = 1_252;
const CLIENT_AREA_HEIGHT: u32 = 667;
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(25);
const MIN_VISIBLE_PIXELS: usize = 250_000;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
enum PreviewMode {
    Type1,
    Type2,
    Type3,
    Type4,
}

impl PreviewMode {
    const fn cli_name(self) -> &'static str {
        match self {
            Self::Type1 => "type1",
            Self::Type2 => "type2",
            Self::Type3 => "type3",
            Self::Type4 => "type4",
        }
    }

    fn default_output(self, language: &str) -> PathBuf {
        let view = match self {
            Self::Type1 => "world",
            Self::Type2 => "mid",
            Self::Type3 => "region",
            Self::Type4 => "local",
        };
        PathBuf::from(format!(
            "target/ui-parity/world-map-{view}-{language}-1252x667.png"
        ))
    }
}

#[derive(Resource)]
struct PreviewOutput(PathBuf);

#[derive(Resource)]
struct PreviewLanguage(String);

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

#[derive(Default)]
struct PreviewCatalog {
    entries: BTreeMap<i32, WorldMapNpcCatalogEntry>,
}

impl WorldMapCatalog for PreviewCatalog {
    fn npc(&self, npc_type: i32) -> WorldMapCatalogLookup<WorldMapNpcCatalogEntry> {
        self.entries
            .get(&npc_type)
            .cloned()
            .map(WorldMapCatalogLookup::Unique)
            .unwrap_or(WorldMapCatalogLookup::Missing)
    }
}

fn main() {
    let (mode, output, language_slug) =
        parse_preview_args(std::env::args_os().skip(1)).unwrap_or_else(|error| {
            eprintln!("{error}");
            eprintln!(
            "usage: world_map_gpu_preview [type1|type2|type3|type4] [OUTPUT.png] [--language en|ru]"
        );
            std::process::exit(2);
        });
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game")
        .canonicalize()
        .expect("assets/game must exist");
    let presentation = build_preview_presentation(mode).expect("build pure world-map projection");
    let (localization, language) = Localization::open(&asset_root, &language_slug)
        .expect("open production localization bundles");

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(mode)
        .insert_resource(PreviewOutput(output))
        .insert_resource(PreviewLanguage(language_slug))
        .insert_resource(PreviewState::default())
        // Freeze only the acceptance harness. Production preserves the clean
        // 0.15/0.5 phase rates and edge cropping from WorldMapMode.cs.
        .insert_resource(WorldMapScanAnimation::frozen(0.42, 0.42))
        .insert_resource(presentation)
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
                        title: format!(
                            "FFOne Retrobution World Map {} acceptance",
                            mode.cli_name()
                        ),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((WorldMapPresentationPlugin, LocalizationPlugin))
        .add_systems(Startup, setup_preview)
        .add_systems(Update, drive_capture)
        .run();
}

fn parse_preview_args(
    args: impl IntoIterator<Item = OsString>,
) -> Result<(PreviewMode, PathBuf, String), String> {
    let mut mode = PreviewMode::Type3;
    let mut mode_seen = false;
    let mut output = None;
    let mut language = "en".to_owned();
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
            continue;
        }
        if !mode_seen {
            mode = match argument.to_str() {
                Some("type1") => PreviewMode::Type1,
                Some("type2") => PreviewMode::Type2,
                Some("type3") => PreviewMode::Type3,
                Some("type4") => PreviewMode::Type4,
                _ => {
                    return Err(format!(
                        "unknown world-map view: {}",
                        argument.to_string_lossy()
                    ));
                }
            };
            mode_seen = true;
        } else if output.is_none() {
            output = Some(PathBuf::from(argument));
        } else {
            return Err("too many arguments".to_owned());
        }
    }
    let output = output.unwrap_or_else(|| mode.default_output(&language));
    if output.extension().and_then(|value| value.to_str()) != Some("png") {
        return Err("world-map output must use the .png extension".to_owned());
    }
    Ok((mode, output, language))
}

fn build_preview_presentation(mode: PreviewMode) -> Result<WorldMapPresentation, String> {
    let player_position = if matches!(mode, PreviewMode::Type3 | PreviewMode::Type4) {
        // The supplied clean reference is the Future/Pokey Oaks world view
        // backed by freezone_001; its player marker resolves to this clean
        // map-space position. Type4 keeps a companion freezone_002 fixture.
        WorldMapPoint::new(6_390.0, 100.0, 1_500.0)
    } else {
        WorldMapPoint::new(4_096.0, 100.0, 4_096.0)
    };
    let player = WorldMapPlayer::new(player_position, 37.0);
    let mut model = WorldMapModel::default();
    model
        .try_open(WorldMapOpenContext::gameplay(player))
        .map_err(|error| format!("open world map: {error:?}"))?;
    model.pop_outbox();
    match mode {
        PreviewMode::Type1 => {}
        PreviewMode::Type2 => {
            model
                .zoom_button(WorldMapZoomDirection::In, WorldMapInputGates::default())
                .map_err(|error| format!("select Type2: {error:?}"))?;
        }
        // Future opens directly in the clean Type3/world-view state.
        PreviewMode::Type3 => {}
        PreviewMode::Type4 => {
            model
                .select_local_view(WorldMapInputGates::default())
                .map_err(|error| format!("select Type4: {error:?}"))?;
        }
    }
    for _ in 0..160 {
        model
            .advance_paint()
            .map_err(|error| format!("settle map view: {error:?}"))?;
    }
    if model.zoom()
        != match mode {
            PreviewMode::Type1 => WorldMapZoom::Type1,
            PreviewMode::Type2 => WorldMapZoom::Type2,
            PreviewMode::Type3 => WorldMapZoom::Type3,
            PreviewMode::Type4 => WorldMapZoom::Type4,
        }
    {
        return Err("pure model selected an unexpected zoom".to_owned());
    }

    model
        .apply_present_npc_types(true, 1, &[1_001, 1_002, 1_003])
        .map_err(|error| format!("apply present NPCs: {error:?}"))?;
    let waypoint = match mode {
        PreviewMode::Type3 => None,
        PreviewMode::Type4 => Some(WorldMapPoint::new(7_500.0, 100.0, 2_300.0)),
        PreviewMode::Type1 | PreviewMode::Type2 => {
            Some(WorldMapPoint::new(4_500.0, 106.0, 4_300.0))
        }
    };
    model
        .set_waypoint(waypoint)
        .map_err(|error| format!("set preview waypoint: {error:?}"))?;

    let catalog = PreviewCatalog {
        entries: BTreeMap::from([
            (
                1_001,
                WorldMapNpcCatalogEntry {
                    display_name: "Numbuh Two".to_owned(),
                    map_icon: 12,
                    mission: WorldMapMissionAvailability::None,
                },
            ),
            (
                1_002,
                WorldMapNpcCatalogEntry {
                    display_name: "Dexter".to_owned(),
                    map_icon: 4,
                    mission: WorldMapMissionAvailability::New,
                },
            ),
            (
                1_003,
                WorldMapNpcCatalogEntry {
                    display_name: "Edd".to_owned(),
                    map_icon: 7,
                    mission: WorldMapMissionAvailability::Advance,
                },
            ),
        ]),
    };
    let npc_positions = if matches!(mode, PreviewMode::Type3 | PreviewMode::Type4) {
        [
            WorldMapPoint::new(6_260.0, 100.0, 1_430.0),
            WorldMapPoint::new(6_520.0, 100.0, 1_680.0),
            WorldMapPoint::new(6_790.0, 100.0, 1_380.0),
        ]
    } else {
        [
            WorldMapPoint::new(3_930.0, 100.0, 4_000.0),
            WorldMapPoint::new(4_080.0, 100.0, 4_240.0),
            WorldMapPoint::new(4_280.0, 100.0, 3_980.0),
        ]
    };
    let npcs = [
        WorldMapNpcSource {
            npc_type: 1_001,
            position: npc_positions[0],
        },
        WorldMapNpcSource {
            npc_type: 1_002,
            position: npc_positions[1],
        },
        WorldMapNpcSource {
            npc_type: 1_003,
            position: npc_positions[2],
        },
    ];
    let markers: Vec<WorldMapMarker> = model
        .project_markers(&catalog, &npcs)
        .map_err(|error| format!("project preview markers: {error:?}"))?;
    let mut presentation = WorldMapPresentation::new(model, markers, "POKEY OAKS NORTH");
    presentation.hover = WorldMapPresentationHover {
        control: Some(WorldMapPresentationControl::Help),
        marker_index: None,
        pointer: WorldMapUiPoint::new(1_010.0, 623.0),
    };
    presentation
        .validate()
        .map_err(|error| format!("validate preview presentation: {error:?}"))?;
    Ok(presentation)
}

fn setup_preview(mut commands: Commands) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
}

#[allow(clippy::too_many_arguments)]
fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    status: Res<WorldMapPresentationAssetStatus>,
    presentation: Res<WorldMapPresentation>,
    language: Res<PreviewLanguage>,
    mut state: ResMut<PreviewState>,
    roots: Query<&ComputedNode, With<WorldMapPresentationRoot>>,
    map_windows: Query<
        &ComputedNode,
        (
            With<WorldMapPresentationWindow>,
            Without<WorldMapPresentationRoot>,
        ),
    >,
    markers: Query<&WorldMapPresentationMarker>,
    all_texts: Query<&Text>,
    styled_texts: Query<(
        Entity,
        &Text,
        &LocalizedText,
        &WorldMapTextStyle,
        (&TextFont, &LineHeight),
        &TextLayout,
        &TextLayoutInfo,
        &ComputedNode,
        &Node,
        &UiTransform,
        &InheritedVisibility,
    )>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);
    if let WorldMapPresentationAssetStatus::Failed { asset_path } = *status {
        eprintln!("world-map source asset failed to load: {asset_path}");
        exit.write(AppExit::error());
        return;
    }
    let layout_ready = roots.single().is_ok_and(|node| {
        node.size() == Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32)
    }) && map_windows
        .single()
        .is_ok_and(|node| node.size() == Vec2::new(1_036.0, 654.0));
    let loaded = matches!(*status, WorldMapPresentationAssetStatus::Ready)
        && presentation.model.phase() == WorldMapPhase::Open
        && presentation.validate().is_ok()
        && layout_ready
        && markers.iter().count() == presentation.markers.len();
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
        match audit_world_map_text(&asset_server, &all_texts, &styled_texts, &language.0) {
            Ok(summary) => {
                println!("{summary}");
                state.text_audited = true;
            }
            Err(error) => {
                eprintln!("world-map GPU text audit failed: {error}");
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
            eprintln!("world-map capture timed out");
        }
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

fn audit_world_map_text(
    asset_server: &AssetServer,
    all_texts: &Query<'_, '_, &Text>,
    styled_texts: &Query<
        '_,
        '_,
        (
            Entity,
            &Text,
            &LocalizedText,
            &WorldMapTextStyle,
            (&TextFont, &LineHeight),
            &TextLayout,
            &TextLayoutInfo,
            &ComputedNode,
            &Node,
            &UiTransform,
            &InheritedVisibility,
        ),
    >,
    language: &str,
) -> Result<String, String> {
    let all_count = all_texts.iter().count();
    let rows = styled_texts.iter().collect::<Vec<_>>();
    if all_count != 6 || rows.len() != all_count {
        return Err(format!(
            "every-Text ownership incomplete: styled={}, all={all_count}, expected=6",
            rows.len()
        ));
    }

    let expected_static = if language == "ru" {
        [
            ("ui.world_map.my_view", "МОЙ ВИД"),
            ("ui.world_map.world_view", "ВИД МИРА"),
            ("ui.world_map.show_filters", "ФИЛЬТРЫ"),
            ("ui.world_map.current_location", "Текущее местоположение:"),
        ]
    } else {
        [
            ("ui.world_map.my_view", "MY VIEW"),
            ("ui.world_map.world_view", "WORLD VIEW"),
            ("ui.world_map.show_filters", "SHOW FILTERS"),
            ("ui.world_map.current_location", "current location:"),
        ]
    };
    let mut seen_static = BTreeSet::new();
    let mut seen_styles = BTreeSet::new();
    let mut visible = 0usize;
    let mut glyphs = 0usize;
    let mut line_boxes = 0usize;
    let mut saw_cyrillic = false;
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
        node,
        transform,
        inherited_visibility,
    ) in rows
    {
        if localized.key.trim().is_empty() {
            return Err(format!("{entity:?} has an empty semantic key"));
        }
        if let Some((_, expected)) = expected_static
            .iter()
            .find(|(key, _)| *key == localized.key)
        {
            let expected_display = if localized.key == "ui.world_map.current_location" {
                expected.to_uppercase()
            } else {
                (*expected).to_owned()
            };
            if text.as_str() != expected_display {
                return Err(format!(
                    "{} resolved {:?}, expected {expected_display:?} for {language}",
                    localized.key,
                    text.as_str()
                ));
            }
            seen_static.insert(localized.key.clone());
        } else if !((localized.key.starts_with("content.location.world.")
            || (localized.key.starts_with("content.npc.") && localized.key.ends_with(".name")))
            && localized.args.is_empty())
            && (localized.key != "ui.content.passthrough"
                || localized.fallback != "{text}"
                || localized.args.keys().map(String::as_str).ne(["text"]))
        {
            return Err(format!(
                "{} bypasses the semantic dynamic-text template",
                localized.key
            ));
        }

        let spec = style.spec();
        let bevy::text::FontSource::Handle(font_handle) = &font.0.font else {
            return Err(format!("expected an asset font, got {:?}", font.0.font));
        };
        let actual_font_path = asset_server
            .get_path(font_handle.id())
            .map(|path| path.path().to_string_lossy().replace('\\', "/"));
        if actual_font_path.as_deref() != Some(spec.font_path)
            || (font.0.font_size.eval(Vec2::ZERO, 16.0) - spec.font_size).abs() > 0.01
            || (*font.1) != LineHeight::Px(spec.line_height)
            || text_layout.justify != spec.justify
            || text_layout.linebreak != spec.linebreak
            || transform.translation != Val2::px(spec.x_offset, spec.y_offset)
            || spec.source_skin_path_id != 1_373
        {
            return Err(format!(
                "{} lost {style:?} replacement metrics: path={actual_font_path:?}, size={}, line={:?}, transform={:?}",
                localized.key,
                font.0.font_size.eval(Vec2::ZERO, 16.0),
                (*font.1),
                transform.translation
            ));
        }
        let expected_padding = UiRect {
            left: px(spec.padding[0]),
            right: px(spec.padding[1]),
            top: px(spec.padding[2]),
            bottom: px(spec.padding[3]),
        };
        if node.padding != expected_padding || node.overflow != Overflow::clip() {
            return Err(format!("{} lost {style:?} padding/clipping", localized.key));
        }
        let expected_flex = match spec.anchor {
            ffone_client::world_map::WorldMapTextAnchor::UpperLeft => {
                (JustifyContent::FlexStart, AlignItems::FlexStart)
            }
            ffone_client::world_map::WorldMapTextAnchor::UpperCenter => {
                (JustifyContent::Center, AlignItems::FlexStart)
            }
            ffone_client::world_map::WorldMapTextAnchor::MiddleCenter => {
                (JustifyContent::Center, AlignItems::Center)
            }
        };
        if (node.justify_content, node.align_items) != expected_flex {
            return Err(format!(
                "{} lost {style:?} anchor {:?}",
                localized.key, spec.anchor
            ));
        }
        seen_styles.insert(*style);

        if !inherited_visibility.get()
            || text.0.is_empty()
            || computed.size().x <= 0.0
            || computed.size().y <= 0.0
            || layout.glyphs.is_empty()
        {
            continue;
        }
        visible += 1;
        glyphs += layout.glyphs.len();
        line_boxes += layout.run_geometry.len();
        saw_cyrillic |= text.0.chars().any(|character| {
            ('\u{0400}'..='\u{052f}').contains(&character)
                || ('\u{2de0}'..='\u{2dff}').contains(&character)
        });
        let expected_lines =
            usize::from(language == "ru" && localized.key == "ui.world_map.current_location") + 1;
        if layout.run_geometry.len() != expected_lines {
            return Err(format!(
                "{} produced {} lines inside its clean Rect, expected {expected_lines}",
                localized.key,
                layout.run_geometry.len()
            ));
        }
        if !layout.size.is_finite()
            || layout.size.x > computed.size().x + 1.0
            || layout.size.y > computed.size().y + 1.0
        {
            return Err(format!(
                "{} layout {:?} exceeds source node {:?}",
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
                "{} glyph bounds {minimum:?}..{maximum:?} exceed {:?}",
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

    let expected_line_boxes = if language == "ru" { 6 } else { 5 };
    if seen_static.len() != expected_static.len()
        || seen_styles.len() != 4
        || visible != 5
        || line_boxes != expected_line_boxes
        || glyphs == 0
    {
        return Err(format!(
            "incomplete visible map text: static={seen_static:?}, styles={seen_styles:?}, visible={visible}, glyphs={glyphs}, lines={line_boxes}"
        ));
    }
    if language == "ru" && !saw_cyrillic {
        return Err("RU production bundle produced no visible Cyrillic glyphs".to_owned());
    }
    Ok(format!(
        "WorldMap GPU text audit: locale={language}, key-first=6/6, visible={visible}, glyphs={glyphs}, line-boxes={line_boxes}, glyph-y={glyph_min_y:.1}..{glyph_max_y:.1}, styles={seen_styles:?}, calibrated-baselines=true"
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
            "rejecting world-map capture at {}x{}",
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
        eprintln!("rejecting empty world-map capture: {visible_pixels} visible pixels");
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
