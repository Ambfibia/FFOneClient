//! Production selection controls for paired text/geometry measurements at 1264x681.
//! Synthetic roster; avatar/portrait rendering is deliberately outside this capture.
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    text::TextLayoutInfo,
    window::{PresentMode, WindowResolution},
};
use ffone_client::{
    character_selection_ui::*,
    localization::{Localization, LocalizationPlugin},
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

#[path = "../quit_menu_gpu_preview/metrics.rs"]
mod metrics;

#[derive(Resource)]
struct Capture {
    output: PathBuf,
    start: Instant,
    ready: Option<Instant>,
    issued: bool,
    interactive: bool,
}

fn main() {
    let mut args = std::env::args().skip(1);
    let locale = args.next().unwrap_or_else(|| "en".into());
    assert!(matches!(locale.as_str(), "en" | "ru"));
    let output = args.next().map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(format!(
            "target/ui-parity/character-selection-text-{locale}.png"
        ))
    });
    let interactive = match args.next().as_deref() {
        None => false,
        Some("--interactive") => true,
        _ => panic!("usage: character_selection_text_gpu_preview [en|ru] [OUTPUT.png] [--interactive]"),
    };
    assert!(args.next().is_none(), "too many arguments");
    let asset_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game")
        .canonicalize()
        .unwrap();
    let (localization, language) = Localization::open(&asset_root, &locale).unwrap();
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(localization)
        .insert_resource(language)
        .insert_resource(Capture {
            output,
            start: Instant::now(),
            ready: None,
            issued: false,
            interactive,
        })
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne selection text measurement (synthetic roster)".into(),
                        resolution: WindowResolution::new(1264, 681),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((LocalizationPlugin, NativeCharacterSelectionUiPlugin))
        .add_systems(Startup, setup)
        .add_systems(PostUpdate, capture.after(bevy::ui::UiSystems::Layout))
        .run();
}

fn setup(mut model: ResMut<CharacterSelectionUiModel>) {
    model.visible = true;
    model.music_enabled = false;
    model.create = CharacterSelectionCapability::Enabled;
    model.delete = CharacterSelectionCapability::Enabled;
    model.preview = CharacterPreviewStatus::Ready;
    model.set_slots(
        [
            slot(1, "Ambfi Omnik", 3, "PEACH CREEK COMMONS"),
            slot(2, "Clancy Galactichammer", 2, "POKEY OAKS NORTH"),
            slot(3, "Bran Neutronburke", 1, "TECH SQUARE"),
            CharacterSlotUi::Empty,
        ],
        Some(2),
    );
}

fn slot(pc_uid: i64, name: &str, level: i16, district: &str) -> CharacterSlotUi {
    CharacterSlotUi::Occupied(OccupiedCharacterSlotUi {
        pc_uid,
        display_name: name.into(),
        level,
        district: district.into(),
        zone: "THE FUTURE".into(),
        background: CharacterLocationBackground::Future,
    })
}

fn capture(
    mut commands: Commands,
    mut capture: ResMut<Capture>,
    model: Res<CharacterSelectionUiModel>,
    images: Res<Assets<Image>>,
    texts: Query<(
        &Text,
        &TextLayoutInfo,
        &ComputedNode,
        &UiGlobalTransform,
        &InheritedVisibility,
    )>,
    mut exit: MessageWriter<AppExit>,
) {
    if capture.issued {
        return;
    }
    if capture.start.elapsed() > Duration::from_secs(30) {
        eprintln!("selection capture timed out: {:?}", model.asset_status);
        exit.write(AppExit::error());
        return;
    }
    if model.asset_status != CharacterSelectionAssetStatus::Ready {
        return;
    }
    let visible = texts
        .iter()
        .filter(|(text, info, _, _, visibility)| {
            visibility.get() && !text.0.is_empty() && !info.glyphs.is_empty()
        })
        .count();
    if visible < 15 {
        return;
    }
    let ready = capture.ready.get_or_insert_with(Instant::now);
    if ready.elapsed() < Duration::from_secs(1) {
        return;
    }
    for (text, info, node, transform, visibility) in &texts {
        if !visibility.get() || text.0.is_empty() || info.glyphs.is_empty() {
            continue;
        }
        let to_client = |point: Vec2| {
            transform
                .affine()
                .transform_point2(node.content_box().min + point)
        };
        let mut ink_min = Vec2::splat(f32::INFINITY);
        let mut ink_max = Vec2::splat(f32::NEG_INFINITY);
        let mut pens = std::collections::BTreeMap::<usize, (f32, f32)>::new();
        for glyph in &info.glyphs {
            let y = to_client(metrics::glyph_pen_position(glyph)).y;
            let range = pens.entry(glyph.line_index).or_insert((y, y));
            range.0 = range.0.min(y);
            range.1 = range.1.max(y);
            if let Some(ink) = metrics::glyph_ink_bounds(glyph, &images).unwrap() {
                ink_min = ink_min.min(to_client(ink.min));
                ink_max = ink_max.max(to_client(ink.max));
            }
        }
        println!(
            "{:?}: scale={}, layout={:?}, client-alpha-ink={:?}..{:?}, client-pen-y-by-line={:?}",
            text.0, info.scale_factor, info.size, ink_min, ink_max, pens
        );
    }
    capture.issued = true;
    commands.spawn(Screenshot::primary_window()).observe(save);
}

fn save(event: On<ScreenshotCaptured>, capture: Res<Capture>, mut exit: MessageWriter<AppExit>) {
    if let Some(parent) = capture.output.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    event
        .image
        .clone()
        .try_into_dynamic()
        .unwrap()
        .save(&capture.output)
        .unwrap();
    println!("{}", capture.output.display());
    if !capture.interactive {
        exit.write(AppExit::Success);
    }
}
