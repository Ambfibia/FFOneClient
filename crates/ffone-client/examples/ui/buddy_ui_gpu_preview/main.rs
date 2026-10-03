//! Deterministic GPU acceptance frame for the clean-Retrobution buddy panel.

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
    time::TimeUpdateStrategy,
    window::{PresentMode, WindowResolution},
};
use ffone_client::buddy_ui::{
    BUDDY_ADD_DIALOG_PATH, BUDDY_ADD_OVERLAY_PATH, BUDDY_BLUE_BUTTON_OVER_PATH,
    BUDDY_BLUE_BUTTON_PATH, BUDDY_BOX_PATH, BUDDY_CANCEL_BUTTON_PATH, BUDDY_CHALET_FONT_PATH,
    BUDDY_FONT_PATH, BUDDY_FREECHAT_PATH, BUDDY_LARGE_LIST_BACKGROUND_PATH,
    BUDDY_LIST_BACKGROUND_PATH, BUDDY_RED_BUTTON_OVER_PATH, BUDDY_RED_BUTTON_PATH,
    BUDDY_SCROLL_DOWN_PATH, BUDDY_SCROLL_THUMB_PATH, BUDDY_SCROLL_TRACK_PATH, BUDDY_SCROLL_UP_PATH,
    BUDDY_SELECT_PATH, BuddyChatWindowStyle, BuddyEntry, BuddyFontRole, BuddyPresence,
    BuddyScrollbarPart, BuddyTextAnchor, BuddyTextStyle, BuddyUiModel, BuddyUiPlugin, BuddyUiSet,
};
use ffone_client::localization::{
    Localization, LocalizationPlugin, LocalizationSet, LocalizedText,
};

const CLIENT_AREA_WIDTH: u32 = 1_264;
const CLIENT_AREA_HEIGHT: u32 = 681;
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(20);
const MIN_VISIBLE_PIXELS: usize = 200;

#[derive(Resource)]
struct PreviewOutput(PathBuf);

#[derive(Resource)]
struct PreviewOptions {
    output: PathBuf,
    language: String,
    add_modal: bool,
    large: bool,
    overflow: bool,
    quick_slot: bool,
}

#[derive(Resource)]
struct PreviewAssets {
    images: Vec<Handle<Image>>,
    fonts: Vec<Handle<Font>>,
    jeffe_font: Handle<Font>,
    chalet_font: Handle<Font>,
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

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.05)))
        .insert_resource(PreviewOutput(options.output.clone()))
        .insert_resource(options)
        .insert_resource(PreviewState::default())
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
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
                        title: "FFOne Retrobution buddy UI acceptance".into(),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((BuddyUiPlugin, LocalizationPlugin))
        .add_systems(Startup, setup_preview)
        .add_systems(
            Update,
            drive_capture
                .after(BuddyUiSet::Bind)
                .after(LocalizationSet::Apply),
        )
        .run();
}

fn parse_options() -> PreviewOptions {
    let mut output = None;
    let mut language = "en".to_owned();
    let mut add_modal = false;
    let mut large = false;
    let mut overflow = false;
    let mut quick_slot = false;
    let mut args = std::env::args_os().skip(1);
    while let Some(argument) = args.next() {
        if argument == "--add-modal" {
            add_modal = true;
        } else if argument == "--large" {
            large = true;
        } else if argument == "--overflow" {
            overflow = true;
        } else if argument == "--quick-slot" {
            quick_slot = true;
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
                "usage: buddy_ui_gpu_preview [OUTPUT.png] [--language en|ru] [--add-modal] [--large] [--overflow] [--quick-slot]"
            );
            std::process::exit(2);
        }
    }
    if large && quick_slot {
        eprintln!(
            "--quick-slot is a reached small-chat branch and cannot be combined with --large"
        );
        std::process::exit(2);
    }
    let mut suffix = String::new();
    if add_modal {
        suffix.push_str("-add-modal");
    }
    if large {
        suffix.push_str("-large");
    }
    if overflow {
        suffix.push_str("-overflow");
    }
    if quick_slot {
        suffix.push_str("-quick-slot");
    }
    let output = output.unwrap_or_else(|| {
        PathBuf::from(format!(
            "target/ui-parity/buddy-{language}{suffix}-1264x681.png"
        ))
    });
    if output.extension().and_then(|value| value.to_str()) != Some("png") {
        eprintln!("output must end in .png");
        std::process::exit(2);
    }
    PreviewOptions {
        output,
        language,
        add_modal,
        large,
        overflow,
        quick_slot,
    }
}

fn setup_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    options: Res<PreviewOptions>,
    mut model: ResMut<BuddyUiModel>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    model.set_visible(true);
    if options.large {
        model.set_chat_window_style(BuddyChatWindowStyle::Large);
    }
    model.set_quick_slot_active(options.quick_slot);
    model
        .set_entry(
            0,
            Some(sample_entry(
                4_002,
                "Gaia",
                "Roundbreath",
                1,
                BuddyPresence::Online,
                true,
            )),
        )
        .expect("sample slot");
    model
        .set_entry(
            4,
            Some(sample_entry(
                8_198,
                "Hidden",
                "LegacyName",
                0,
                BuddyPresence::Offline,
                false,
            )),
        )
        .expect("sample slot");
    model
        .set_entry(
            11,
            Some(sample_entry(
                9_003,
                "Dexter",
                "Hero",
                1,
                BuddyPresence::Online,
                false,
            )),
        )
        .expect("sample slot");
    model.select_slot(11).expect("sample selection");
    if options.overflow {
        for slot in [1usize, 2, 3, 5, 6, 7, 8, 9, 10, 12] {
            model
                .set_entry(
                    slot,
                    Some(sample_entry(
                        20_000 + slot as i64,
                        "Buddy",
                        &slot.to_string(),
                        1,
                        if slot % 2 == 0 {
                            BuddyPresence::Online
                        } else {
                            BuddyPresence::Offline
                        },
                        false,
                    )),
                )
                .expect("overflow sample slot");
        }
    }
    if options.add_modal {
        model.open_add_dialog();
        model.set_add_name_input("Dexter Hero");
    }

    let jeffe_font = asset_server.load(BUDDY_FONT_PATH);
    let chalet_font = asset_server.load(BUDDY_CHALET_FONT_PATH);

    commands.insert_resource(PreviewAssets {
        images: [
            BUDDY_BOX_PATH,
            BUDDY_SELECT_PATH,
            BUDDY_FREECHAT_PATH,
            BUDDY_LIST_BACKGROUND_PATH,
            BUDDY_LARGE_LIST_BACKGROUND_PATH,
            BUDDY_BLUE_BUTTON_PATH,
            BUDDY_BLUE_BUTTON_OVER_PATH,
            BUDDY_RED_BUTTON_PATH,
            BUDDY_RED_BUTTON_OVER_PATH,
            BUDDY_ADD_DIALOG_PATH,
            BUDDY_ADD_OVERLAY_PATH,
            BUDDY_CANCEL_BUTTON_PATH,
            BUDDY_SCROLL_TRACK_PATH,
            BUDDY_SCROLL_THUMB_PATH,
            BUDDY_SCROLL_UP_PATH,
            BUDDY_SCROLL_DOWN_PATH,
        ]
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect(),
        fonts: vec![jeffe_font.clone(), chalet_font.clone()],
        jeffe_font,
        chalet_font,
    });
}

fn sample_entry(
    pc_uid: i64,
    first_name: &str,
    last_name: &str,
    name_check_flag: i8,
    presence: BuddyPresence,
    free_chat: bool,
) -> BuddyEntry {
    BuddyEntry {
        runtime_pc_id: pc_uid as i32,
        pc_uid,
        free_chat,
        presence,
        first_name: first_name.to_owned(),
        last_name: last_name.to_owned(),
        name_check_flag,
        ..default()
    }
}

fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    fonts: Res<Assets<Font>>,
    options: Res<PreviewOptions>,
    preview_assets: Res<PreviewAssets>,
    mut state: ResMut<PreviewState>,
    computed_nodes: Query<&ComputedNode>,
    all_texts: Query<&Text>,
    styled_texts: Query<(
        Entity,
        &Node,
        &Text,
        &LocalizedText,
        &BuddyTextStyle,
        (&TextFont, &LineHeight),
        &TextLayout,
        &TextLayoutInfo,
        &ComputedNode,
        &InheritedVisibility,
        &ChildOf,
        &UiTransform,
    )>,
    scrollbars: Query<(&BuddyScrollbarPart, &ComputedNode, &InheritedVisibility)>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);
    let failed = preview_assets
        .images
        .iter()
        .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
        || preview_assets
            .fonts
            .iter()
            .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)));
    if failed {
        eprintln!("buddy UI source asset failed to load");
        exit.write(AppExit::error());
        return;
    }
    let loaded = preview_assets
        .images
        .iter()
        .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
        && preview_assets
            .fonts
            .iter()
            .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
        && preview_assets
            .images
            .iter()
            .all(|handle| images.get(handle).is_some())
        && preview_assets
            .fonts
            .iter()
            .all(|handle| fonts.get(handle).is_some())
        && computed_nodes
            .iter()
            .any(|node| node.size().x >= 293.0 && node.size().y >= 150.0);
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
            &scrollbars,
            &preview_assets,
            &options,
        ) {
            Ok(summary) => {
                println!("{summary}");
                state.text_audited = true;
            }
            Err(error) => {
                eprintln!("buddy UI GPU text audit failed: {error}");
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
            eprintln!("buddy UI capture timed out");
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
            &Node,
            &Text,
            &LocalizedText,
            &BuddyTextStyle,
            (&TextFont, &LineHeight),
            &TextLayout,
            &TextLayoutInfo,
            &ComputedNode,
            &InheritedVisibility,
            &ChildOf,
            &UiTransform,
        ),
    >,
    computed_nodes: &Query<'_, '_, &ComputedNode>,
    scrollbars: &Query<'_, '_, (&BuddyScrollbarPart, &ComputedNode, &InheritedVisibility)>,
    assets: &PreviewAssets,
    options: &PreviewOptions,
) -> Result<String, String> {
    let all_count = all_texts.iter().count();
    let styled_count = styled_texts.iter().count();
    if all_count != 59 || styled_count != all_count {
        return Err(format!(
            "every-Text ownership incomplete: styled={styled_count}, all={all_count}, expected=59"
        ));
    }

    let mut visible_count = 0usize;
    let mut glyph_count = 0usize;
    let mut line_count = 0usize;
    let mut saw_cyrillic = false;
    let mut saw_title = false;
    let mut saw_unknown_row = false;
    let mut saw_verified_row = false;
    let mut saw_empty_row = false;
    let mut saw_add_input = false;
    let mut saw_add_title = false;
    let mut visible_styles = Vec::new();
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
        inherited_visibility,
        parent,
        transform,
    ) in styled_texts
    {
        if localized.key.trim().is_empty() {
            return Err(format!("Text {entity:?} has an empty semantic key"));
        }
        if localized.key == "ui.content.passthrough" {
            return Err(format!(
                "Text {entity:?} uses the compatibility passthrough key"
            ));
        }
        let spec = style.spec();
        let expected_font = match spec.font_role {
            BuddyFontRole::Jeffe => &assets.jeffe_font,
            BuddyFontRole::Chalet => &assets.chalet_font,
        };
        if font.0.font != bevy::text::FontSource::Handle(expected_font.clone())
            || font.0.font_size.eval(Vec2::ZERO, 16.0) != spec.font_size
            || (*font.1) != LineHeight::Px(spec.line_height)
        {
            return Err(format!(
                "{} has wrong replacement metrics for {style:?}: size={}, line={:?}",
                localized.key,
                font.0.font_size.eval(Vec2::ZERO, 16.0),
                (*font.1)
            ));
        }
        let expected_justify = match spec.anchor {
            BuddyTextAnchor::UpperLeft | BuddyTextAnchor::MiddleLeft => Justify::Left,
            BuddyTextAnchor::MiddleCenter => Justify::Center,
        };
        if layout.justify != expected_justify {
            return Err(format!("{} has wrong {style:?} alignment", localized.key));
        }
        if transform.translation != Val2::px(0.0, spec.y_offset) {
            return Err(format!(
                "{} has wrong {style:?} replacement baseline transform {:?}",
                localized.key, transform.translation
            ));
        }
        if localized.key == "ui.buddy.title" {
            let expected = if options.language == "ru" {
                " СПИСОК ДРУЗЕЙ"
            } else {
                " BUDDY LIST"
            };
            if text.0 != expected {
                return Err(format!(
                    "localized title {:?} does not match {expected:?}",
                    text.0
                ));
            }
            saw_title = true;
        }
        if localized.key == "ui.buddy.row.player" {
            let expected = if options.language == "ru" {
                "    Игрок 8198"
            } else {
                "    Player 8198"
            };
            if text.0 == expected {
                saw_unknown_row = true;
            }
        }
        if localized.key == "ui.buddy.row.verified" {
            let display_name = localized.args.get("display_name").map(String::as_str);
            let fixture_name = display_name.is_some_and(|name| {
                matches!(name, "Gaia Roundbreath" | "Dexter Hero")
                    || (options.overflow
                        && [1usize, 2, 3, 5, 6, 7, 8, 9, 10, 12]
                            .iter()
                            .any(|slot| name == format!("Buddy {slot}")))
            });
            if localized.fallback != "    {display_name}"
                || !fixture_name
                || localized.args.len() != 1
                || text.0 != format!("    {}", display_name.unwrap_or_default())
            {
                return Err("verified Buddy row template/args are not exact".to_owned());
            }
            saw_verified_row |= display_name == Some("Gaia Roundbreath");
        }
        if localized.key == "ui.buddy.row.empty" {
            if !localized.fallback.is_empty() || !localized.args.is_empty() || !text.0.is_empty() {
                return Err("empty Buddy row template is not exact".to_owned());
            }
            saw_empty_row = true;
        }
        if localized.key == "ui.buddy.add_name_input" {
            let expected_name = if options.add_modal { "Dexter Hero" } else { "" };
            if localized.fallback != "{name}"
                || localized.args.get("name").map(String::as_str) != Some(expected_name)
                || text.0 != expected_name
            {
                return Err("ADD name input template/args are not exact".to_owned());
            }
            saw_add_input = true;
        }
        if localized.key == "ui.buddy.add_title" {
            let expected = if options.language == "ru" {
                "ДОБАВИТЬ ДРУГА"
            } else {
                "ADD BUDDY"
            };
            if text.0 != expected {
                return Err(format!(
                    "localized ADD title {:?} does not match {expected:?}",
                    text.0
                ));
            }
        }

        if !inherited_visibility.get()
            || computed.size().x <= 0.0
            || computed.size().y <= 0.0
            || text.0.is_empty()
            || layout_info.glyphs.is_empty()
        {
            continue;
        }
        visible_count += 1;
        saw_add_title |= localized.key == "ui.buddy.add_title";
        glyph_count += layout_info.glyphs.len();
        line_count += layout_info.run_geometry.len();
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
        if !layout_info.size.is_finite()
            || layout_info.size.x > source_rect.size().x + 1.0
            || layout_info.size.y > source_rect.size().y + 1.0
        {
            return Err(format!(
                "{} GPU text bounds {:?} exceed source Rect {:?}",
                localized.key,
                layout_info.size,
                source_rect.size()
            ));
        }
        let mut minimum = Vec2::splat(f32::INFINITY);
        let mut maximum = Vec2::splat(f32::NEG_INFINITY);
        for glyph in &layout_info.glyphs {
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
                    "{} invalid GPU line/baseline box {:?} for line height {}",
                    localized.key, line, spec.line_height
                ));
            }
        }
    }
    if !saw_title
        || !saw_unknown_row
        || !saw_verified_row
        || !saw_empty_row
        || !saw_add_input
        || visible_count == 0
        || glyph_count == 0
        || line_count == 0
    {
        return Err("localized visible GPU text has not been laid out".to_owned());
    }
    if options.add_modal != saw_add_title {
        return Err(format!(
            "ADD popup visibility mismatch: requested={}, visible={saw_add_title}",
            options.add_modal
        ));
    }
    if options.language == "ru" && !saw_cyrillic {
        return Err("RU production bundle produced no visible Cyrillic glyphs".to_owned());
    }
    let visible_scrollbar_parts = scrollbars
        .iter()
        .filter(|(_, computed, inherited)| {
            inherited.get() && computed.size().x > 0.0 && computed.size().y > 0.0
        })
        .count();
    let expected_scrollbar_parts = if options.overflow { 4 } else { 0 };
    if visible_scrollbar_parts != expected_scrollbar_parts {
        return Err(format!(
            "scrollbar visibility mismatch: visible={visible_scrollbar_parts}, expected={expected_scrollbar_parts}"
        ));
    }
    visible_styles.sort_by_key(|style| format!("{style:?}"));
    Ok(format!(
        "Buddy GPU text audit: locale={}, key-first={}/{}, visible={}, glyphs={}, line-boxes={}, add-modal={}, large={}, overflow={}, quick-slot={}, scrollbar-parts={}, styles={visible_styles:?}",
        options.language,
        styled_count,
        all_count,
        visible_count,
        glyph_count,
        line_count,
        options.add_modal,
        options.large,
        options.overflow,
        options.quick_slot,
        visible_scrollbar_parts,
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
        eprintln!("rejecting empty buddy UI capture: {visible_pixels} visible pixels");
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
