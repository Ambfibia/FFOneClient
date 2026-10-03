//! Deterministic EN/RU 1264x681 GPU acceptance frame for clean-Retrobution
//! `CnGuiNanoFreeTuning.OnGUI`.

use std::{
    collections::BTreeMap,
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
    localization::{Localization, LocalizationPlugin, LocalizationSet, LocalizedText},
    nano_free_tuning_ui::{
        NANO_FREE_TUNING_FONT_PATH, NanoFreeTuningContent, NanoFreeTuningIconLabelStyle,
        NanoFreeTuningModel, NanoFreeTuningOpenContext, NanoFreeTuningPhase, NanoFreeTuningPower,
        NanoFreeTuningPowerButton, NanoFreeTuningPresentationAssetStatus,
        NanoFreeTuningPresentationPanel, NanoFreeTuningPresentationRoot,
        NanoFreeTuningPresentationSet, NanoFreeTuningTransform, NanoFreeTuningUiPlugin,
        NanoFreeTuningUiTextStyle, NanoFreeTuningWorldSnapshot, nano_free_tuning_panel_rect,
    },
};

const CLIENT_AREA_WIDTH: u32 = 1_264;
const CLIENT_AREA_HEIGHT: u32 = 681;
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(25);
const MIN_VISIBLE_PIXELS: usize = 70_000;
const EXPECTED_TEXT_COUNT: usize = 16;

#[derive(Resource)]
struct PreviewOutput(PathBuf);

#[derive(Resource)]
struct PreviewLanguage(String);

#[derive(Resource)]
struct PreviewButtonState(String);

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
        "target/ui-parity/nano-free-tuning-{language}-1264x681.png"
    ))
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
    let (localization, language) =
        Localization::open(&asset_root, &cli.language).expect("open production localization");
    let button_state = env::var("FFONE_NANO_BUTTON_STATE").unwrap_or_else(|_| "normal".to_owned());
    assert!(matches!(
        button_state.as_str(),
        "normal" | "hover" | "pressed" | "disabled"
    ));
    let mut model = build_preview_model();
    if button_state == "disabled" {
        model
            .click_power(0)
            .expect("queue request for disabled capture");
    }

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.035, 0.12, 0.16)))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
        .insert_resource(model)
        .insert_resource(PreviewButtonState(button_state))
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
                        title: "FFOne Retrobution NanoFreeTuning acceptance".to_owned(),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((NanoFreeTuningUiPlugin, LocalizationPlugin))
        .add_systems(Startup, setup_preview)
        .add_systems(
            PreUpdate,
            drive_pointer
                .after(bevy::input::InputSystems)
                .before(bevy::ui::UiSystems::Focus),
        )
        .add_systems(
            Update,
            drive_capture
                .after(NanoFreeTuningPresentationSet::Bind)
                .after(NanoFreeTuningPresentationSet::Visuals)
                .after(LocalizationSet::Apply),
        )
        .run();
}

// Exercise actual UI hit-testing, including the text over the button, instead
// of injecting Interaction::Hovered and bypassing focus/layering bugs.
fn drive_pointer(
    state: Res<PreviewButtonState>,
    mut windows: Query<&mut Window>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    buttons: Query<&ComputedNode, With<NanoFreeTuningPowerButton>>,
) {
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    let panel = nano_free_tuning_panel_rect(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32);
    window.set_cursor_position(Some(if state.0 == "normal" {
        Vec2::new(10.0, 200.0)
    } else {
        Vec2::new(panel.x + 237.0 + 48.5, panel.y + 145.0 + 10.0)
    }));
    if state.0 == "pressed" && buttons.iter().any(|node| node.size().x > 0.0) {
        mouse.press(MouseButton::Left);
    }
}

fn parse_cli(args: impl IntoIterator<Item = OsString>) -> Result<PreviewCli, &'static str> {
    let mut args = args.into_iter();
    let language = args
        .next()
        .and_then(|value| value.into_string().ok())
        .unwrap_or_else(|| "en".to_owned());
    if !matches!(language.as_str(), "en" | "ru") {
        return Err("usage: nano_free_tuning_gpu_preview [en|ru] [OUTPUT.png]");
    }
    let output = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| default_output(&language));
    if args.next().is_some() || output.extension().and_then(|value| value.to_str()) != Some("png") {
        return Err("usage: nano_free_tuning_gpu_preview [en|ru] [OUTPUT.png]");
    }
    Ok(PreviewCli { language, output })
}

fn build_preview_model() -> NanoFreeTuningModel {
    let mut model = NanoFreeTuningModel::default();
    model
        .open(NanoFreeTuningOpenContext {
            player_id: 7_001,
            killed_fusion: false,
            content: NanoFreeTuningContent {
                nano_id: 1,
                nano_style: 0,
                nano_name: "Buttercup".to_owned(),
                powers: [
                    NanoFreeTuningPower {
                        tune_id: 1,
                        skill_id: 1,
                        icon_path: "ui/en/gameplay/nano/icons/skill/skillicon_10.png".to_owned(),
                        name: "MISS FIRE".to_owned(),
                        power_type: "STUN - CONE".to_owned(),
                        description: "Mange uses fire to stun enemies in the target area."
                            .to_owned(),
                    },
                    NanoFreeTuningPower {
                        tune_id: 2,
                        skill_id: 2,
                        icon_path: "ui/en/gameplay/nano/icons/skill/skillicon_04.png".to_owned(),
                        name: "RALLYING CRY".to_owned(),
                        power_type: "HEALTH - GROUP".to_owned(),
                        description:
                            "Buttercup's warcry pumps up your group, healing their injuries."
                                .to_owned(),
                    },
                    NanoFreeTuningPower {
                        tune_id: 3,
                        skill_id: 3,
                        icon_path: "ui/en/gameplay/nano/icons/skill/skillicon_26.png".to_owned(),
                        name: "BUTTERCUP BURST".to_owned(),
                        power_type: "SCAVENGE".to_owned(),
                        description: "Collect even more Fusion Matter!".to_owned(),
                    },
                ],
            },
            world: NanoFreeTuningWorldSnapshot {
                player: NanoFreeTuningTransform {
                    position: Vec3::ZERO,
                    rotation: Quat::IDENTITY,
                },
                first_defeated_fusion: None,
            },
        })
        .expect("preview content must open");
    model.clear_intents();
    model
        .advance(2.001)
        .expect("non-kill preview must reach power selection");
    model.clear_intents();
    model
}

fn setup_preview(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    commands.insert_resource(PreviewAssets {
        font: asset_server.load(NANO_FREE_TUNING_FONT_PATH),
    });
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: percent(100),
            height: percent(100),
            ..default()
        },
        BackgroundColor(Color::srgb(0.09, 0.31, 0.38)),
        GlobalZIndex(-1_000),
        Pickable::IGNORE,
    ));
}

#[allow(clippy::too_many_arguments)]
fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    fonts: Res<Assets<Font>>,
    preview_assets: Res<PreviewAssets>,
    status: Res<NanoFreeTuningPresentationAssetStatus>,
    model: Res<NanoFreeTuningModel>,
    language: Res<PreviewLanguage>,
    button_state: Res<PreviewButtonState>,
    mut state: ResMut<PreviewState>,
    roots: Query<&ComputedNode, With<NanoFreeTuningPresentationRoot>>,
    panels: Query<
        (&ComputedNode, &UiGlobalTransform),
        (
            With<NanoFreeTuningPresentationPanel>,
            Without<NanoFreeTuningPresentationRoot>,
        ),
    >,
    buttons: Query<(
        &Node,
        &ComputedNode,
        &NanoFreeTuningPowerButton,
        &Interaction,
        &ImageNode,
    )>,
    icon_slots: Query<(&Node, &ComputedNode, &Children), With<NanoFreeTuningIconLabelStyle>>,
    computed_nodes: Query<&ComputedNode>,
    texts: Query<(
        Entity,
        &Text,
        &LocalizedText,
        &NanoFreeTuningUiTextStyle,
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
    if let NanoFreeTuningPresentationAssetStatus::Failed { asset_path } = &*status {
        eprintln!("NanoFreeTuning source asset failed to load: {asset_path}");
        exit.write(AppExit::error());
        return;
    }
    if matches!(
        asset_server.load_state(preview_assets.font.id()),
        LoadState::Failed(_)
    ) {
        eprintln!("NanoFreeTuning replacement font failed to load");
        exit.write(AppExit::error());
        return;
    }
    let font_loaded = matches!(
        asset_server.load_state(preview_assets.font.id()),
        LoadState::Loaded
    ) && fonts.get(&preview_assets.font).is_some();
    let source_panel = nano_free_tuning_panel_rect(1_264.0, 681.0);
    let panel_ready = panels.single().is_ok_and(|(node, global)| {
        let expected_center = Vec2::new(
            source_panel.x + source_panel.width * 0.5,
            source_panel.y + source_panel.height * 0.5,
        );
        node.size() == Vec2::new(source_panel.width, source_panel.height)
            && global.translation.distance(expected_center) <= 0.75
    });
    let buttons_ready = buttons.iter().count() == 3
        && buttons.iter().all(|(node, computed, _, _, _)| {
            node.padding == UiRect::new(px(6), px(6), px(3), px(3))
                && node.overflow == Overflow::clip()
                && computed.size() == Vec2::new(97.0, 20.0)
        });
    let icons_ready = icon_slots.iter().count() == 3
        && icon_slots.iter().all(|(node, computed, children)| {
            let child_size = children
                .iter()
                .next()
                .and_then(|child| computed_nodes.get(child).ok())
                .map(ComputedNode::size);
            node.padding == UiRect::new(px(0), px(0), px(3), px(3))
                && computed.size() == Vec2::new(35.0, 35.0)
                && child_size == Some(Vec2::new(35.0, 29.0))
        });
    let layout_ready = roots
        .single()
        .is_ok_and(|node| node.size() == Vec2::new(1_264.0, 681.0))
        && panel_ready
        && buttons_ready
        && icons_ready;

    let text_audit = audit_nano_free_tuning_text(&texts, &preview_assets, &language.0);
    if font_loaded && !state.text_audited {
        match &text_audit {
            Ok(report) => {
                println!("{report}");
                state.text_audited = true;
            }
            Err(error) if state.frames % 120 == 0 => {
                eprintln!("NanoFreeTuning text audit waiting: {error}");
            }
            Err(_) => {}
        }
    }
    let ready = matches!(*status, NanoFreeTuningPresentationAssetStatus::Ready)
        && model.phase() == NanoFreeTuningPhase::PowerSelection
        && model.panel_visible()
        && font_loaded
        && layout_ready
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
        let (_, _, _, interaction, image) = buttons
            .iter()
            .find(|(_, _, button, _, _)| button.power_index == 0)
            .unwrap();
        match button_state.0.as_str() {
            "hover" => assert_eq!(
                *interaction,
                Interaction::Hovered,
                "button label must pass hover to the button"
            ),
            "pressed" => assert_eq!(*interaction, Interaction::Pressed),
            "disabled" => assert_eq!(
                image.color.alpha(),
                ffone_client::nano_free_tuning_ui::NANO_FREE_TUNING_BUTTON_DISABLED_ALPHA
            ),
            _ => assert_eq!(*interaction, Interaction::None),
        }
        println!(
            "Button state accepted through real hit test: {}",
            button_state.0
        );
        state.capture_issued = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }
    if state.capture_saved {
        exit.write(AppExit::Success);
    } else if state.capture_failed || state.started_at.elapsed() >= CAPTURE_TIMEOUT {
        if !state.capture_failed {
            eprintln!("NanoFreeTuning capture timed out");
        }
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

fn localized_arg<'a>(localized: &'a LocalizedText, name: &str) -> Result<&'a str, String> {
    localized
        .args
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| format!("{} is missing {{{name}}}", localized.key))
}

fn expected_rendered_text(localized: &LocalizedText, language: &str) -> Result<String, String> {
    Ok(match localized.key.as_str() {
        "ui.nano_free_tuning.acquired" => match language {
            "en" => "YOU HAVE ACQUIRED ".to_owned(),
            "ru" => "ВЫ ПОЛУЧИЛИ ".to_owned(),
            _ => return Err(format!("unsupported locale {language}")),
        },
        "ui.nano_free_tuning.nano_name" => {
            format!(" {} ", localized_arg(localized, "name")?)
        }
        "ui.nano_free_tuning.bang" => " !".to_owned(),
        "ui.nano_free_tuning.title" => match language {
            "en" => "SELECT A POWER".to_owned(),
            "ru" => "ВЫБЕРИТЕ СИЛУ".to_owned(),
            _ => return Err(format!("unsupported locale {language}")),
        },
        "ui.nano_free_tuning.power.name" => localized_arg(localized, "name")?.to_owned(),
        "ui.nano_free_tuning.power.type" => localized_arg(localized, "type")?.to_owned(),
        "ui.nano_free_tuning.power.description" => {
            localized_arg(localized, "description")?.to_owned()
        }
        "ui.nano_free_tuning.select" => match language {
            "en" => "SELECT".to_owned(),
            "ru" => "ВЫБРАТЬ".to_owned(),
            _ => return Err(format!("unsupported locale {language}")),
        },
        key => return Err(format!("unexpected NanoFreeTuning key {key}")),
    })
}

fn audit_nano_free_tuning_text(
    texts: &Query<
        '_,
        '_,
        (
            Entity,
            &Text,
            &LocalizedText,
            &NanoFreeTuningUiTextStyle,
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

    let mut key_counts = BTreeMap::<String, usize>::new();
    let mut glyphs = 0usize;
    let mut inferred_lines = 0usize;
    let mut saw_cyrillic = false;
    let mut styles = Vec::new();
    let mut glyph_min_y = f32::INFINITY;
    let mut glyph_max_y = f32::NEG_INFINITY;
    for (_entity, text, localized, style, font, text_layout, layout, node, computed, transform) in
        rows
    {
        *key_counts.entry(localized.key.clone()).or_default() += 1;
        let (expected_args, expected_style, expected_fallback): (
            &[&str],
            NanoFreeTuningUiTextStyle,
            &str,
        ) = match localized.key.as_str() {
            "ui.nano_free_tuning.acquired" => (
                &[],
                NanoFreeTuningUiTextStyle::BigBlueMiddleLeft,
                "YOU HAVE ACQUIRED ",
            ),
            "ui.nano_free_tuning.nano_name" => (
                &["name"],
                NanoFreeTuningUiTextStyle::BigYellowMiddleLeft,
                " {name} ",
            ),
            "ui.nano_free_tuning.bang" => (&[], NanoFreeTuningUiTextStyle::BigBlueMiddleLeft, " !"),
            "ui.nano_free_tuning.title" => (
                &[],
                NanoFreeTuningUiTextStyle::TransparentLightBlueUpperLeft,
                "SELECT A POWER",
            ),
            "ui.nano_free_tuning.power.name" => (
                &["name"],
                NanoFreeTuningUiTextStyle::TransparentLightBlueUpperLeft,
                "{name}",
            ),
            "ui.nano_free_tuning.power.type" => (
                &["type"],
                NanoFreeTuningUiTextStyle::TransparentYellowSmallUpperLeft,
                "{type}",
            ),
            "ui.nano_free_tuning.power.description" => (
                &["description"],
                NanoFreeTuningUiTextStyle::TransparentBlueUpperLeft,
                "{description}",
            ),
            "ui.nano_free_tuning.select" => {
                (&[], NanoFreeTuningUiTextStyle::ButtonMiddleCenter, "SELECT")
            }
            key => return Err(format!("unexpected NanoFreeTuning key {key}")),
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
        if localized.fallback != expected_fallback {
            return Err(format!(
                "{} fallback {:?} != {:?}",
                localized.key, localized.fallback, expected_fallback
            ));
        }
        let expected_text = expected_rendered_text(localized, language)?;
        if text.0 != expected_text {
            return Err(format!(
                "{} rendered {:?} != {:?}",
                localized.key, text.0, expected_text
            ));
        }
        if *style != expected_style {
            return Err(format!(
                "{} uses {style:?}, expected {expected_style:?}",
                localized.key
            ));
        }
        if style.source_skin_path_id() != 1_379
            || style.replacement_font_path() != NANO_FREE_TUNING_FONT_PATH
            || style.content_offset() != [0.0, 0.0]
        {
            return Err(format!("{} lost source GUIStyle ownership", localized.key));
        }
        if font.0.font != bevy::text::FontSource::Handle(preview_assets.font.clone()) {
            return Err(format!("{} uses the wrong JEFFE font", localized.key));
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
                "{} replacement Y {:?} != {}",
                localized.key,
                transform.translation,
                style.replacement_y_offset()
            ));
        }
        if !matches!(style, NanoFreeTuningUiTextStyle::ButtonMiddleCenter) {
            let [left, right, top, bottom] = style.padding();
            if node.padding != UiRect::new(px(left), px(right), px(top), px(bottom)) {
                return Err(format!("{} has wrong GUIStyle padding", localized.key));
            }
        }
        let expected_justify = if matches!(style, NanoFreeTuningUiTextStyle::ButtonMiddleCenter) {
            Justify::Center
        } else {
            Justify::Left
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
        if text.0.is_empty()
            || computed.size().x <= 0.0
            || computed.size().y <= 0.0
            || layout.glyphs.is_empty()
            || layout.run_geometry.is_empty()
        {
            return Err(format!("{} has no laid-out GPU glyphs", localized.key));
        }
        glyphs += layout.glyphs.len();
        saw_cyrillic |= text.0.chars().any(|character| {
            ('\u{0400}'..='\u{052f}').contains(&character)
                || ('\u{2de0}'..='\u{2dff}').contains(&character)
        });
        if !styles.contains(style) {
            styles.push(*style);
        }

        let line_ratio = layout.size.y / style.line_height();
        let line_count = line_ratio.round().max(1.0) as usize;
        if (line_ratio - line_count as f32).abs() > 0.12 {
            return Err(format!(
                "{} line-box height {} is not a multiple of {}",
                localized.key,
                layout.size.y,
                style.line_height()
            ));
        }
        if localized.key == "ui.nano_free_tuning.power.description" {
            if !(1..=2).contains(&line_count) {
                return Err(format!(
                    "{} produced {line_count} lines in its 25px clean Rect",
                    localized.key
                ));
            }
        } else if line_count != 1 {
            return Err(format!(
                "{} produced {line_count} lines in a clean single-line Rect",
                localized.key
            ));
        }
        inferred_lines += line_count;
        if !layout.size.is_finite()
            || layout.size.x > computed.size().x + 1.0
            || layout.size.y > computed.size().y + 1.0
        {
            return Err(format!(
                "{} layout {:?} exceeds clean text node {:?}",
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
                "{} glyphs {:?}..{:?} exceed clean node {:?}",
                localized.key,
                minimum,
                maximum,
                computed.size()
            ));
        }
        glyph_min_y = glyph_min_y.min(minimum.y);
        glyph_max_y = glyph_max_y.max(maximum.y);
        for run in &layout.run_geometry {
            let section = run.bounds;
            if !section.min.is_finite()
                || !section.max.is_finite()
                || section.min.x < -1.0
                || section.min.y < -1.0
                || section.max.x > layout.size.x + 1.0
                || section.max.y > layout.size.y + 1.0
            {
                return Err(format!(
                    "{} invalid section/line bounds {section:?} in {:?}",
                    localized.key, layout.size
                ));
            }
        }
    }

    let exact_counts = [
        ("ui.nano_free_tuning.acquired", 1),
        ("ui.nano_free_tuning.nano_name", 1),
        ("ui.nano_free_tuning.bang", 1),
        ("ui.nano_free_tuning.title", 1),
        ("ui.nano_free_tuning.power.name", 3),
        ("ui.nano_free_tuning.power.type", 3),
        ("ui.nano_free_tuning.power.description", 3),
        ("ui.nano_free_tuning.select", 3),
    ];
    for (key, count) in exact_counts {
        if key_counts.get(key).copied() != Some(count) {
            return Err(format!("{key} count {:?} != {count}", key_counts.get(key)));
        }
    }
    for required in [
        NanoFreeTuningUiTextStyle::BigBlueMiddleLeft,
        NanoFreeTuningUiTextStyle::BigYellowMiddleLeft,
        NanoFreeTuningUiTextStyle::TransparentLightBlueUpperLeft,
        NanoFreeTuningUiTextStyle::TransparentYellowSmallUpperLeft,
        NanoFreeTuningUiTextStyle::TransparentBlueUpperLeft,
        NanoFreeTuningUiTextStyle::ButtonMiddleCenter,
    ] {
        if !styles.contains(&required) {
            return Err(format!("visible NanoFreeTuning has no {required:?}"));
        }
    }
    if language == "ru" && !saw_cyrillic {
        return Err("RU NanoFreeTuning produced no visible Cyrillic glyphs".to_owned());
    }
    styles.sort_by_key(|style| format!("{style:?}"));
    Ok(format!(
        "NanoFreeTuning GPU text audit: locale={language}, key-first={EXPECTED_TEXT_COUNT}, \
         glyphs={glyphs}, inferred-lines={inferred_lines}, glyph-y={glyph_min_y:.1}..{glyph_max_y:.1}, \
         styles={styles:?}"
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
            "rejecting NanoFreeTuning capture at {}x{}",
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
            u16::from(red) + u16::from(green) + u16::from(blue) > 48
        })
        .count();
    if visible_pixels < MIN_VISIBLE_PIXELS {
        eprintln!("rejecting empty NanoFreeTuning capture: {visible_pixels} visible pixels");
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
        "saved {} ({} visible pixels)",
        output.0.display(),
        visible_pixels
    );
    state.capture_saved = true;
}
