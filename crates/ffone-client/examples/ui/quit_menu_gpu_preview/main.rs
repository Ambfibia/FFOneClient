//! Deterministic 1264x681 GPU acceptance frame for the clean-Retrobution
//! gameplay QuitMenu.
//!
//! The harness uses the production module and localization bundles. The second
//! button is held in `Hovered`; the remaining standard buttons and the distinct
//! Cancel style stay in their normal states.

use std::{
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
    localization::{Localization, LocalizationPlugin, LocalizedText},
    quit_menu_ui::{
        QUIT_MENU_BACKDROP_PATH, QUIT_MENU_BUTTON_HOVER_PATH, QUIT_MENU_BUTTON_NORMAL_PATH,
        QUIT_MENU_BUTTON_SOUND_PATHS, QUIT_MENU_CANCEL_HOVER_PATH, QUIT_MENU_CANCEL_NORMAL_PATH,
        QUIT_MENU_CLOSE_SOUND_PATH, QUIT_MENU_DIALOG_PATH, QUIT_MENU_FONT_PATH,
        QUIT_MENU_OPEN_SOUND_PATH, QuitMenuButton, QuitMenuButtonKind, QuitMenuButtonVisual,
        QuitMenuDialog, QuitMenuInputBoundary, QuitMenuTextStyle, QuitMenuUiModel,
        QuitMenuUiPlugin, QuitMenuUiRoot, QuitMenuUiSet, clean_quit_menu_ui_scale,
        quit_menu_ui_layout,
    },
};
use ffone_ui_layout::quit_menu::{QUIT_MENU_DOCUMENT_PATH, QuitMenuDocument};
mod metrics;

const CLIENT_AREA_WIDTH: u32 = 1_264;
const CLIENT_AREA_HEIGHT: u32 = 681;
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(20);
const MIN_DIALOG_VISIBLE_PIXELS: usize = 4_000;
const MIN_CYAN_PIXELS: usize = 120;
const PREVIEW_HOVER: QuitMenuButtonKind = QuitMenuButtonKind::QuitGame;

#[derive(Resource)]
struct PreviewOutput(PathBuf);

#[derive(Resource)]
struct PreviewDocument(QuitMenuDocument);

#[derive(Resource)]
struct PreviewOptions {
    output: PathBuf,
    language: String,
    interactive: bool,
}

#[derive(Resource)]
struct PreviewAssets {
    images: Vec<Handle<Image>>,
    font: Handle<Font>,
    audio: Vec<Handle<AudioSource>>,
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
    let (localization, language) = Localization::open(&asset_root, &options.language)
        .expect("open production localization bundles");
    let document = QuitMenuDocument::from_json(
        &fs::read(asset_root.join(QUIT_MENU_DOCUMENT_PATH)).expect("read native quit menu"),
    )
    .expect("validate native quit menu");

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.08, 0.17, 0.25)))
        .insert_resource(PreviewOutput(options.output.clone()))
        .insert_resource(options)
        .insert_resource(PreviewState::default())
        .insert_resource(PreviewDocument(document))
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
                        title: "FFOne Retrobution QuitMenu acceptance".into(),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((QuitMenuUiPlugin, LocalizationPlugin))
        .add_systems(Startup, setup_preview)
        .add_systems(
            Update,
            force_preview_hover
                .after(QuitMenuUiSet::Interaction)
                .before(QuitMenuUiSet::Visuals),
        )
        .add_systems(Update, drive_capture.after(QuitMenuUiSet::Visuals))
        .run();
}

fn parse_options() -> PreviewOptions {
    let mut output = None;
    let mut language = "en".to_owned();
    let mut interactive = false;
    let mut args = std::env::args_os().skip(1);
    while let Some(argument) = args.next() {
        if argument == "--interactive" {
            interactive = true;
        } else if argument == "--language" {
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
            eprintln!(
                "usage: quit_menu_gpu_preview [OUTPUT.png] [--language en|ru] [--interactive]"
            );
            std::process::exit(2);
        }
    }
    let output = output.unwrap_or_else(|| {
        PathBuf::from(format!(
            "target/ui-parity/quit-menu-{language}-1264x681.png"
        ))
    });
    if output.extension().and_then(|value| value.to_str()) != Some("png") {
        eprintln!("output must end in .png");
        std::process::exit(2);
    }
    PreviewOptions {
        output,
        language,
        interactive,
    }
}

fn setup_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut model: ResMut<QuitMenuUiModel>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    spawn_world_backdrop(&mut commands);
    model.open();

    commands.insert_resource(PreviewAssets {
        images: [
            QUIT_MENU_BACKDROP_PATH,
            QUIT_MENU_DIALOG_PATH,
            QUIT_MENU_BUTTON_NORMAL_PATH,
            QUIT_MENU_BUTTON_HOVER_PATH,
            QUIT_MENU_CANCEL_NORMAL_PATH,
            QUIT_MENU_CANCEL_HOVER_PATH,
        ]
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect(),
        font: asset_server.load(QUIT_MENU_FONT_PATH),
        audio: [QUIT_MENU_OPEN_SOUND_PATH, QUIT_MENU_CLOSE_SOUND_PATH]
            .into_iter()
            .chain(QUIT_MENU_BUTTON_SOUND_PATHS)
            .map(|path| asset_server.load(path))
            .collect(),
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
            BackgroundColor(Color::srgb(0.15, 0.42, 0.62)),
            GlobalZIndex(-1_000),
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            for (left, top, width, height, color) in [
                (55.0, 55.0, 285.0, 170.0, Color::srgb(0.86, 0.43, 0.16)),
                (905.0, 65.0, 295.0, 185.0, Color::srgb(0.24, 0.76, 0.38)),
                (65.0, 455.0, 315.0, 155.0, Color::srgb(0.66, 0.24, 0.72)),
                (865.0, 445.0, 330.0, 165.0, Color::srgb(0.9, 0.73, 0.16)),
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

fn force_preview_hover(
    options: Res<PreviewOptions>,
    mut buttons: Query<(&QuitMenuButton, &mut Interaction)>,
) {
    if options.interactive {
        return;
    }
    for (button, mut interaction) in &mut buttons {
        *interaction = if button.kind == PREVIEW_HOVER {
            Interaction::Hovered
        } else {
            Interaction::None
        };
    }
}

#[allow(clippy::too_many_arguments)]
fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    fonts: Res<Assets<Font>>,
    audio_sources: Res<Assets<AudioSource>>,
    preview_assets: Res<PreviewAssets>,
    configuration: (Res<PreviewOptions>, Res<PreviewDocument>),
    model: Res<QuitMenuUiModel>,
    mut state: ResMut<PreviewState>,
    roots: Query<&ComputedNode, With<QuitMenuUiRoot>>,
    dialogs: Query<(&ComputedNode, &UiTransform), With<QuitMenuDialog>>,
    buttons: Query<(&QuitMenuButton, &Interaction, &ComputedNode, &ImageNode)>,
    all_texts: Query<&Text>,
    styled_texts: Query<(
        Entity,
        &Text,
        &LocalizedText,
        &QuitMenuTextStyle,
        (&TextFont, &LineHeight),
        &TextLayout,
        &TextLayoutInfo,
        &ComputedNode,
        &UiTransform,
        &UiGlobalTransform,
        &InheritedVisibility,
        &ChildOf,
    )>,
    computed_nodes: Query<&ComputedNode>,
    mut exit: MessageWriter<AppExit>,
) {
    let (options, document) = configuration;
    if options.interactive && state.capture_saved {
        return;
    }
    state.frames = state.frames.saturating_add(1);

    let failed = preview_assets
        .images
        .iter()
        .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
        || matches!(
            asset_server.load_state(preview_assets.font.id()),
            LoadState::Failed(_)
        )
        || preview_assets
            .audio
            .iter()
            .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)));
    if failed {
        eprintln!("QuitMenu acceptance asset failed to load");
        exit.write(AppExit::error());
        return;
    }

    let assets_loaded = preview_assets
        .images
        .iter()
        .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
        && matches!(
            asset_server.load_state(preview_assets.font.id()),
            LoadState::Loaded
        )
        && preview_assets
            .audio
            .iter()
            .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded));
    let cpu_assets_present = preview_assets
        .images
        .iter()
        .all(|handle| images.get(handle).is_some())
        && fonts.get(&preview_assets.font).is_some()
        && preview_assets
            .audio
            .iter()
            .all(|handle| audio_sources.get(handle).is_some());
    let root_exact = roots.single().is_ok_and(|node| {
        node.size() == Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32)
    });
    let dialog_exact = dialogs.single().is_ok_and(|(node, transform)| {
        node.size() == Vec2::new(206.0, 188.0) && transform.scale == Vec2::ONE
    });
    let button_contract = buttons.iter().count() == 3
        && buttons.iter().all(|(_, _, node, image)| {
            node.size() == Vec2::new(175.0, 45.0)
                && image.visual_box == bevy::ui::VisualBox::BorderBox
        })
        && (options.interactive
            || buttons.iter().any(|(button, interaction, _, _)| {
                button.kind == PREVIEW_HOVER && *interaction == Interaction::Hovered
            }));
    let boundary_exact = model.input_boundary()
        == QuitMenuInputBoundary {
            blocks_lower_ui: true,
            blocks_gameplay_input: true,
            requires_pointer: true,
            mouse_controls_enabled: true,
            escape_dismiss_enabled: true,
        };
    let text_audit = audit_text(
        &all_texts,
        &styled_texts,
        &computed_nodes,
        &preview_assets,
        &options.language,
        &document.0,
        &images,
    );
    if assets_loaded && !state.text_audited {
        match &text_audit {
            Ok(report) => {
                println!("{report}");
                state.text_audited = true;
            }
            Err(error) if state.frames % 120 == 0 => {
                eprintln!("QuitMenu text audit waiting: {error}")
            }
            Err(_) => {}
        }
    }
    let ready = assets_loaded
        && cpu_assets_present
        && root_exact
        && dialog_exact
        && button_contract
        && boundary_exact
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
        if !options.interactive {
            exit.write(AppExit::Success);
        }
    } else if state.capture_failed {
        exit.write(AppExit::error());
    } else if state.started_at.elapsed() >= CAPTURE_TIMEOUT {
        eprintln!("QuitMenu capture timed out");
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
            &QuitMenuTextStyle,
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
    computed_nodes: &Query<'_, '_, &ComputedNode>,
    assets: &PreviewAssets,
    language: &str,
    document: &QuitMenuDocument,
    images: &Assets<Image>,
) -> Result<String, String> {
    let all_count = all_texts.iter().count();
    let styled_count = styled_texts.iter().count();
    if all_count != 3 || styled_count != all_count {
        return Err(format!(
            "every-Text ownership incomplete: styled={styled_count}, all={all_count}, expected=3"
        ));
    }

    let mut standard_count = 0usize;
    let mut cancel_count = 0usize;
    let mut visible_count = 0usize;
    let mut glyph_count = 0usize;
    let mut line_count = 0usize;
    let mut saw_cyrillic = false;
    let mut measurements = Vec::new();

    for (
        entity,
        text,
        localized,
        style,
        font,
        text_layout,
        layout_info,
        computed,
        transform,
        global_transform,
        inherited_visibility,
        parent,
    ) in styled_texts
    {
        let (fallback, english, russian, expected_visual) = match localized.key.as_str() {
            "ui.quit_menu.change_character" => (
                "CHANGE CHARACTER",
                "CHANGE CHARACTER",
                "СМЕНИТЬ ПЕРСОНАЖА",
                QuitMenuButtonVisual::Standard,
            ),
            "ui.quit_menu.quit_game" => (
                "QUIT GAME",
                "QUIT GAME",
                "ВЫЙТИ ИЗ ИГРЫ",
                QuitMenuButtonVisual::Standard,
            ),
            "ui.common.cancel" => ("CANCEL", "CANCEL", "ОТМЕНА", QuitMenuButtonVisual::Cancel),
            key => return Err(format!("unexpected QuitMenu localization key {key:?}")),
        };
        if localized.fallback != fallback || !localized.args.is_empty() {
            return Err(format!(
                "{} has a non-clean fallback or unexpected template args",
                localized.key
            ));
        }
        let expected_copy = if language == "ru" { russian } else { english };
        if text.0 != expected_copy {
            return Err(format!(
                "{} rendered {:?}, expected {:?}",
                localized.key, text.0, expected_copy
            ));
        }
        if style.0 != expected_visual {
            return Err(format!(
                "{} uses {:?}, expected {:?}",
                localized.key, style.0, expected_visual
            ));
        }
        match style.0 {
            QuitMenuButtonVisual::Standard => standard_count += 1,
            QuitMenuButtonVisual::Cancel => cancel_count += 1,
        }

        let spec = style.0.text_style();
        let native = &document.styles[match style.0 {
            QuitMenuButtonVisual::Standard => 0,
            QuitMenuButtonVisual::Cancel => 1,
        }];
        if font.0.font != bevy::text::FontSource::Handle(assets.font.clone())
            || font.0.font_size.eval(Vec2::ZERO, 16.0) != document.font_size
            || (*font.1) != LineHeight::Px(document.line_height)
        {
            return Err(format!(
                "{} has wrong JEFFE replacement metrics: size={}, line={:?}",
                localized.key,
                font.0.font_size.eval(Vec2::ZERO, 16.0),
                (*font.1)
            ));
        }
        let expected_linebreak = if native.word_wrap {
            LineBreak::WordBoundary
        } else {
            LineBreak::NoWrap
        };
        if text_layout.justify != Justify::Center || text_layout.linebreak != expected_linebreak {
            return Err(format!(
                "{} does not preserve {} alignment/wrapping",
                localized.key, spec.source_style
            ));
        }
        let [x, y] = native.text_offset();
        if transform.translation != Val2::px(x, y) {
            return Err(format!(
                "{} replacement baseline offset {:?} != {:?}",
                localized.key,
                transform.translation,
                native.text_offset()
            ));
        }

        if !inherited_visibility.get()
            || computed.size().x <= 0.0
            || computed.size().y <= 0.0
            || layout_info.glyphs.is_empty()
        {
            continue;
        }
        visible_count += 1;
        glyph_count += layout_info.glyphs.len();
        line_count += layout_info.run_geometry.len();
        saw_cyrillic |= text.0.chars().any(|character| {
            ('\u{0400}'..='\u{052f}').contains(&character)
                || ('\u{2de0}'..='\u{2dff}').contains(&character)
        });

        let parent_computed = computed_nodes
            .get(parent.parent())
            .map_err(|_| format!("Text {entity:?} has no source button parent"))?;
        let [left, right, top, bottom] = native.padding;
        let available = Vec2::new(
            parent_computed.size().x - left - right,
            parent_computed.size().y - top - bottom,
        );
        if !layout_info.size.is_finite()
            || layout_info.size.x > available.x + 1.0
            || layout_info.size.y > available.y + 1.0
        {
            return Err(format!(
                "{} GPU layout {:?} exceeds source content Rect {:?}",
                localized.key, layout_info.size, available
            ));
        }

        let mut minimum = Vec2::splat(f32::INFINITY);
        let mut maximum = Vec2::splat(f32::NEG_INFINITY);
        let mut pen_y_by_line = std::collections::BTreeMap::<usize, (f32, f32)>::new();
        for glyph in &layout_info.glyphs {
            let pen = metrics::glyph_pen_position(glyph);
            let screen_pen = global_transform
                .affine()
                .transform_point2(computed.content_box().min + pen);
            let range = pen_y_by_line
                .entry(glyph.line_index)
                .or_insert((screen_pen.y, screen_pen.y));
            range.0 = range.0.min(screen_pen.y);
            range.1 = range.1.max(screen_pen.y);
            if let Some(ink) = metrics::glyph_ink_bounds(glyph, images)? {
                minimum = minimum.min(ink.min);
                maximum = maximum.max(ink.max);
            }
        }
        if !minimum.is_finite() || !maximum.is_finite() {
            return Err(format!(
                "{} has no visible rasterized glyphs",
                localized.key
            ));
        }
        measurements.push(format!(
            "{}: scale={}, layout={:?}, alpha-ink={:?}..{:?}, textOffset={:?}, client-pen-y-by-line={:?}",
            localized.key,
            layout_info.scale_factor,
            layout_info.size,
            minimum,
            maximum,
            native.text_offset(),
            pen_y_by_line
        ));
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
        for run in &layout_info.run_geometry {
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
                (font.0.font_size.eval(Vec2::ZERO, 16.0) * 1.5 - spec.line_height).max(0.0) + 0.55;
            if !line.min.is_finite()
                || !line.max.is_finite()
                || line.min.y < -overhang
                || line.max.y > layout_info.size.y + overhang
                || line.height() + 0.55 < spec.line_height
                || line.height() > spec.line_height + overhang
            {
                return Err(format!(
                    "{} invalid GPU decoration/selection box {:?}",
                    localized.key, line
                ));
            }
        }
    }

    if standard_count != 2
        || cancel_count != 1
        || visible_count != 3
        || glyph_count == 0
        || line_count == 0
    {
        return Err(format!(
            "incomplete visible style tree: standard={standard_count}, cancel={cancel_count}, visible={visible_count}, glyphs={glyph_count}, lines={line_count}"
        ));
    }
    if language == "ru" && !saw_cyrillic {
        return Err("RU production bundle produced no visible Cyrillic glyphs".to_owned());
    }

    Ok(format!(
        "QuitMenu GPU text audit: locale={language}, key-first={styled_count}/{all_count}, visible={visible_count}, glyphs={glyph_count}, decoration-runs={line_count}, styles=[Button x2, CancelButton x1]\nNative glyph raster bounds (physical pixels relative to text layout, alpha > 0; GUI baseline not inferred):\n{}",
        measurements.join("\n")
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
    let rgba = image.to_rgba8();
    if rgba.dimensions() != (CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT) {
        eprintln!(
            "rejecting QuitMenu capture with unexpected size {:?}",
            rgba.dimensions()
        );
        state.capture_failed = true;
        return;
    }

    let layout = quit_menu_ui_layout(
        Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32),
        clean_quit_menu_ui_scale(CLIENT_AREA_HEIGHT as f32),
    );
    let dialog = layout.dialog.visual;
    let x_start = dialog.x.max(0.0) as u32;
    let x_end = (dialog.x + dialog.width).min(CLIENT_AREA_WIDTH as f32) as u32;
    let y_start = dialog.y.max(0.0) as u32;
    let y_end = (dialog.y + dialog.height).min(CLIENT_AREA_HEIGHT as f32) as u32;
    let mut visible_pixels = 0;
    let mut cyan_pixels = 0;
    for y in y_start..y_end {
        for x in x_start..x_end {
            let [red, green, blue, _alpha] = rgba.get_pixel(x, y).0;
            if u16::from(red) + u16::from(green) + u16::from(blue) > 55 {
                visible_pixels += 1;
            }
            if green > 80 && blue > 105 && blue > red.saturating_add(20) {
                cyan_pixels += 1;
            }
        }
    }
    if visible_pixels < MIN_DIALOG_VISIBLE_PIXELS || cyan_pixels < MIN_CYAN_PIXELS {
        eprintln!(
            "rejecting incomplete QuitMenu capture: visible={visible_pixels} \
             (min {MIN_DIALOG_VISIBLE_PIXELS}), cyan={cyan_pixels} \
             (min {MIN_CYAN_PIXELS})"
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
