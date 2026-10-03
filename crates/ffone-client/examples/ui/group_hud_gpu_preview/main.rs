//! Deterministic GPU acceptance frame for the passive Retrobution group HUD.
//!
//! `CnGuiGroup_info` is authored in FusionFall's 1280x720 outer-window
//! reference coordinate space. The captured Unity Web Player client area was
//! measured at exactly 1264x681 after outer chrome/borders. This harness keeps
//! the UI in the original 1280x720 coordinate space and clips it through a
//! 1264x681 window; it must not rescale the source rectangles to fit.

use std::{
    collections::BTreeSet,
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
use ffone_client::{
    group_ui::{
        GROUP_HP_BAR_PATH, GROUP_INFO_PATH, GROUP_NANO_HP_FRAME_PATH, GROUP_NPC_CO_OP_PATH,
        GROUP_PROTOCOL_MAX_PC_MEMBERS, GROUP_UI_FONT_BYTES, GROUP_UI_FONT_PATH,
        GROUP_UI_FONT_SHA256, GROUP_UI_PRIMARY_CONTAINER_BYTES, GROUP_UI_PRIMARY_CONTAINER_SHA256,
        GROUP_UI_REFERENCE_HEIGHT, GROUP_UI_REFERENCE_WIDTH, GroupNanoUi, GroupNpcMemberUi,
        GroupPcMemberUi, GroupUiModel, GroupUiPlugin, GroupUiTextStyle,
    },
    localization::{Localization, LocalizationPlugin, LocalizedText},
};

const PREVIEW_SKILL_ICON_PATH: &str = "ui/en/gameplay/nano/icons/skill/skillicon_10.png";
const CLIENT_AREA_WIDTH: u32 = 1_264;
const CLIENT_AREA_HEIGHT: u32 = 681;
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(20);
const MIN_VISIBLE_PIXELS: usize = 64;

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
    let (output, language_slug) =
        parse_preview_args(std::env::args_os().skip(1)).unwrap_or_else(|error| {
            eprintln!("{error}");
            eprintln!("usage: group_hud_gpu_preview [OUTPUT.png] [--language en|ru]");
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
        .insert_resource(ClearColor(Color::BLACK))
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
                        title: "FFOne Retrobution group HUD acceptance".into(),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((GroupUiPlugin, LocalizationPlugin))
        .add_systems(Startup, setup_preview)
        .add_systems(Update, drive_capture)
        .run();
}

fn parse_preview_args(
    args: impl IntoIterator<Item = OsString>,
) -> Result<(PathBuf, String), String> {
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
        } else if output.is_none() {
            output = Some(PathBuf::from(argument));
        } else {
            return Err("too many arguments".to_owned());
        }
    }
    let output = output.unwrap_or_else(|| {
        PathBuf::from(format!(
            "target/ui-parity/group-hud-{language}-1264x681.png"
        ))
    });
    if output.extension().and_then(|value| value.to_str()) != Some("png") {
        return Err("group HUD output must use the .png extension".to_owned());
    }
    Ok((output, language))
}

fn setup_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    language: Res<PreviewLanguage>,
    mut model: ResMut<GroupUiModel>,
) {
    // GroupUiPlugin deliberately reuses the runtime UI camera. The acceptance
    // harness supplies the only camera because it runs outside the client.
    commands.spawn((Camera2d, IsDefaultUiCamera));
    *model = source_backed_sample(&language.0);

    commands.insert_resource(PreviewAssets {
        images: [
            GROUP_INFO_PATH,
            GROUP_NANO_HP_FRAME_PATH,
            GROUP_NPC_CO_OP_PATH,
            GROUP_HP_BAR_PATH,
            PREVIEW_SKILL_ICON_PATH,
        ]
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect(),
        fonts: vec![asset_server.load(GROUP_UI_FONT_PATH)],
    });
}

fn source_backed_sample(language: &str) -> GroupUiModel {
    // Names remain server/NPC-authored template arguments. The RU fixture
    // intentionally exercises Cyrillic glyphs in the validated replacement
    // font; it does not claim that locale selection translates player copy.
    let (local_first, local_last, first, last, nano, second_first, second_last, npc) =
        if language == "ru" {
            (
                "Тест",
                "Игрок",
                "Гайя",
                "Круг",
                "Лютик",
                "Декстер",
                "Герой",
                "Кооперативный НПС",
            )
        } else {
            (
                "Test",
                "Ser",
                "Gaia",
                "Roundbreath",
                "Buttercup",
                "Dexter",
                "Hero",
                "Cooperative NPC",
            )
        };
    GroupUiModel {
        local_pc_uid: Some(1_001),
        pc_members: vec![
            GroupPcMemberUi {
                pc_id: 1_001,
                pc_uid: 1_001,
                first_name: local_first.to_owned(),
                last_name: local_last.to_owned(),
                level: 12,
                hp: 1_000,
                max_hp: 1_000,
                free_chat: false,
                nano: None,
                ..default()
            },
            GroupPcMemberUi {
                pc_id: 1_002,
                pc_uid: 1_002,
                first_name: first.to_owned(),
                last_name: last.to_owned(),
                level: 18,
                hp: 735,
                max_hp: 1_000,
                free_chat: true,
                nano: Some(GroupNanoUi {
                    name: nano.to_owned(),
                    stamina: 55,
                    max_stamina: 100,
                    skill_icon_path: Some(PREVIEW_SKILL_ICON_PATH.to_owned()),
                }),
                ..default()
            },
            GroupPcMemberUi {
                pc_id: 1_003,
                pc_uid: 1_003,
                first_name: second_first.to_owned(),
                last_name: second_last.to_owned(),
                level: 9,
                hp: 420,
                max_hp: 800,
                free_chat: false,
                nano: None,
                ..default()
            },
        ],
        npc_members: vec![GroupNpcMemberUi {
            npc_type: 0, // Generic NPC fixture, without a catalog identity.
            name: npc.to_owned(),
            hp: 650,
            max_hp: 1_300,
        }],
    }
}

fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    fonts: Res<Assets<Font>>,
    preview_assets: Res<PreviewAssets>,
    mut state: ResMut<PreviewState>,
    computed_nodes: Query<&ComputedNode>,
    text_rows: Query<(
        Entity,
        &Text,
        &LocalizedText,
        &GroupUiTextStyle,
        (&TextFont, &LineHeight),
        &TextLayout,
        &TextLayoutInfo,
        &ComputedNode,
        &Node,
        &UiTransform,
        &InheritedVisibility,
    )>,
    language: Res<PreviewLanguage>,
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
        eprintln!("group HUD source asset failed to load");
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
    let reference_layout_ready = computed_nodes.iter().any(|node| {
        node.size().x >= GROUP_UI_REFERENCE_WIDTH && node.size().y >= GROUP_UI_REFERENCE_HEIGHT
    });
    let ready = source_assets_loaded && cpu_assets_present && reference_layout_ready;
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
        match audit_group_text(&asset_server, &language.0, &text_rows) {
            Ok(summary) => {
                println!("{summary}");
                state.text_audited = true;
            }
            Err(error) => {
                eprintln!("group HUD GPU text audit failed: {error}");
                state.capture_failed = true;
            }
        }
    }
    if !state.capture_issued && ready && warmed && state.text_audited {
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
        eprintln!("group HUD capture timed out");
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

fn audit_group_text(
    asset_server: &AssetServer,
    language: &str,
    rows: &Query<(
        Entity,
        &Text,
        &LocalizedText,
        &GroupUiTextStyle,
        (&TextFont, &LineHeight),
        &TextLayout,
        &TextLayoutInfo,
        &ComputedNode,
        &Node,
        &UiTransform,
        &InheritedVisibility,
    )>,
) -> Result<String, String> {
    let mut total = 0usize;
    let mut level_templates = 0usize;
    let mut passthrough_templates = 0usize;
    let mut visible = 0usize;
    let mut glyphs = 0usize;
    let mut line_boxes = 0usize;
    let mut glyph_min_y = f32::INFINITY;
    let mut glyph_max_y = f32::NEG_INFINITY;
    let mut saw_cyrillic = false;
    let mut styles = BTreeSet::new();

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
        total += 1;
        match localized.key.as_str() {
            "ui.group.level" => {
                level_templates += 1;
                if localized.fallback != "{level}"
                    || localized.args.keys().map(String::as_str).ne(["level"])
                {
                    return Err(format!("{entity:?} bypasses the level template"));
                }
            }
            "ui.content.passthrough" => {
                passthrough_templates += 1;
                if localized.fallback != "{text}"
                    || localized.args.keys().map(String::as_str).ne(["text"])
                {
                    return Err(format!("{entity:?} bypasses the content template"));
                }
            }
            key => return Err(format!("{entity:?} has unexpected semantic key {key}")),
        }

        let spec = style.spec();
        let bevy::text::FontSource::Handle(font_handle) = &font.0.font else {
            return Err(format!("expected an asset font, got {:?}", font.0.font));
        };
        let actual_font_path = asset_server
            .get_path(font_handle.id())
            .map(|path| path.path().to_string_lossy().replace('\\', "/"));
        if actual_font_path.as_deref() != Some(spec.font_path)
            || font.0.font_size.eval(Vec2::ZERO, 16.0) != spec.font_size
            || (*font.1) != LineHeight::Px(spec.line_height)
            || text_layout.justify != spec.justify
            || text_layout.linebreak != spec.linebreak
            || node.padding != UiRect::ZERO
            || node.justify_content != JustifyContent::FlexStart
            || node.align_items != AlignItems::Center
            || node.overflow != Overflow::clip()
            || transform.translation != Val2::px(0.0, 0.0)
            || spec.source_game_object_path_id != 1_352
            || spec.source_component_path_id != 1_563
            || spec.source_script_path_id != 1_135
            || spec.source_skin_path_id != 1_372
            || spec.source_style != "label"
            || spec.source_font_path_id != 1_018
            || spec.y_offset != 0.0
        {
            return Err(format!(
                "{entity:?} lost the clean HUD label contract: path={actual_font_path:?}, size={}, line={:?}, node={node:?}, transform={:?}",
                font.0.font_size.eval(Vec2::ZERO, 16.0),
                (*font.1),
                transform.translation
            ));
        }
        styles.insert(*style);

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
        if layout.run_geometry.len() != 1 {
            return Err(format!(
                "{} wrapped to {} lines inside its clean Rect",
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

    let expected_total = GROUP_PROTOCOL_MAX_PC_MEMBERS * 3 + 1;
    if total != expected_total
        || level_templates != GROUP_PROTOCOL_MAX_PC_MEMBERS
        || passthrough_templates != GROUP_PROTOCOL_MAX_PC_MEMBERS * 2 + 1
        || styles != BTreeSet::from([GroupUiTextStyle::HudLabel])
        || visible != 6
        || line_boxes != 6
        || glyphs == 0
    {
        return Err(format!(
            "incomplete Group HUD text: total={total}, level={level_templates}, passthrough={passthrough_templates}, styles={styles:?}, visible={visible}, glyphs={glyphs}, lines={line_boxes}"
        ));
    }
    if language == "ru" && !saw_cyrillic {
        return Err("RU stress fixture produced no visible Cyrillic glyphs".to_owned());
    }
    if GROUP_UI_PRIMARY_CONTAINER_BYTES != 7_000_415
        || GROUP_UI_PRIMARY_CONTAINER_SHA256.len() != 64
        || GROUP_UI_FONT_BYTES != 96_832
        || GROUP_UI_FONT_SHA256.len() != 64
    {
        return Err("Group HUD source/font provenance contract drifted".to_owned());
    }
    Ok(format!(
        "Group HUD GPU text audit: locale={language}, key-first={total}/{expected_total}, visible={visible}, glyphs={glyphs}, line-boxes={line_boxes}, styles={styles:?}, glyph-y={glyph_min_y:.1}..{glyph_max_y:.1}, y-offset=0"
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
            "rejecting group HUD capture at {}x{}",
            image.width(),
            image.height()
        );
        state.capture_failed = true;
        return;
    }
    let rgba = image.to_rgba8();
    let visible_pixels = rgba
        .pixels()
        .filter(|pixel| {
            let [red, green, blue, _alpha] = pixel.0;
            u16::from(red) + u16::from(green) + u16::from(blue) > 20
        })
        .count();
    if visible_pixels < MIN_VISIBLE_PIXELS {
        eprintln!(
            "rejecting empty group HUD capture: {visible_pixels} visible pixels \
             < {MIN_VISIBLE_PIXELS}"
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
