//! Native barber chrome, localization and shared player-rig GPU acceptance.
use bevy::{
    asset::AssetPlugin,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    window::WindowResolution,
};
use ffone_client::{
    barber::{BarberModel, BarberPhase},
    character_creation_data::CharacterCreationData,
    character_creation_ui::{CharacterAppearance, barber_ui::BarberUiPlugin},
    legacy_model_material::LegacyModelMaterialPlugin,
    localization::{Localization, LocalizationPlugin},
    player_preview::{
        NativePlayerPreviewModel, NativePlayerPreviewPlugin, NativePlayerPreviewStage,
        NativePlayerPreviewStatus,
    },
    player_shared_rig::{NativePlayerRigCatalog, NativePlayerSharedRigPlugin},
};
use ffone_protocol::{
    WirePayload,
    wire_0104::{PcBarberOpenSuccess0104, PcStyle0104},
};
use ffone_runtime_contracts::CharacterAppearanceCategory;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Resource)]
struct Data(CharacterCreationData);
#[derive(Resource)]
struct Capture {
    path: PathBuf,
    start: Instant,
    ready: Option<Instant>,
    sent: bool,
}
fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game")
        .canonicalize()
        .unwrap();
    let locale = std::env::var("FFONE_REVIEW_LANGUAGE").unwrap_or("en".into());
    let (localization, language) = Localization::open(&root, &locale).unwrap();
    let path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(format!("target/performance/menu-bugs/barber-{locale}.png"))
        });
    App::new()
        .insert_resource(Data(CharacterCreationData::open(&root).unwrap()))
        .insert_resource(NativePlayerRigCatalog::open(&root).unwrap())
        .insert_resource(Capture {
            path,
            start: Instant::now(),
            ready: None,
            sent: false,
        })
        .insert_resource(localization)
        .insert_resource(language)
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: root.to_string_lossy().into(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Barber acceptance".into(),
                        resolution: WindowResolution::new(1264, 681),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((
            LocalizationPlugin,
            LegacyModelMaterialPlugin,
            NativePlayerSharedRigPlugin,
            NativePlayerPreviewPlugin,
            BarberUiPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, capture)
        .run();
}
fn setup(
    mut commands: Commands,
    data: Res<Data>,
    mut model: ResMut<BarberModel>,
    mut preview: ResMut<NativePlayerPreviewModel>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    let mut appearance = CharacterAppearance::default();
    appearance.hair_color = 6;
    appearance.eye_color = 3;
    let resolved = data
        .0
        .resolve_creator(1, 0, "Test", "Hero", &appearance)
        .unwrap();
    let a = resolved.style;
    let mut style = PcStyle0104::decode(&[0; 76]).unwrap();
    style.gender = a.gender;
    style.hair_style = a.hair_style;
    style.face_style = a.face_style;
    style.hair_color = a.hair_color;
    style.skin_color = a.skin_color;
    style.eye_color = a.eye_color;
    style.body = a.body;
    style.height = a.height;
    let mut prices = PcBarberOpenSuccess0104::decode(&[0; 288]).unwrap();
    prices.change_gender = 1;
    prices.hair_color_cost = 50;
    prices.eye_color_cost = 50;
    model.phase = BarberPhase::Editing;
    model.draft = Some(style.clone());
    style.hair_color = 1;
    style.eye_color = 1;
    model.original = Some(style);
    model.prices = Some(prices);
    model.taros = 1000;
    model.palettes = data.0.ui_palettes();
    for choice in &data.0.appearance_document().choices {
        let gender = usize::from(choice.gender.protocol_code() == 2);
        let list = match choice.category {
            CharacterAppearanceCategory::Hair => &mut model.hair[gender],
            CharacterAppearanceCategory::Face => &mut model.face[gender],
            _ => continue,
        };
        list.push((choice.value as i8, choice.label.clone()));
        let (field, category) = if choice.category == CharacterAppearanceCategory::Hair {
            (ffone_client::barber::BarberField::Hair, "hair")
        } else {
            (ffone_client::barber::BarberField::Face, "face")
        };
        model.appearance_keys.insert(
            (
                choice.gender.protocol_code() as i8,
                field as u8,
                choice.value as i8,
            ),
            format!(
                "content.appearance.{}.{category}.{}.name",
                if gender == 0 { "male" } else { "female" },
                choice.creation_index
            ),
        );
    }
    preview.set_look(resolved.look).unwrap();
    preview.visible = true;
    preview.stage = NativePlayerPreviewStage::Barber;
}
fn capture(
    mut commands: Commands,
    preview: Res<NativePlayerPreviewModel>,
    mut capture: ResMut<Capture>,
) {
    if capture.sent {
        return;
    }
    assert!(
        capture.start.elapsed() < Duration::from_secs(120),
        "preview timeout: {:?} {:?}",
        preview.status,
        preview.loading_detail
    );
    if matches!(
        preview.status,
        NativePlayerPreviewStatus::ReadyAnimated { .. }
    ) {
        let ready = *capture.ready.get_or_insert_with(Instant::now);
        if ready.elapsed() > Duration::from_secs(3) {
            capture.sent = true;
            commands.spawn(Screenshot::primary_window()).observe(save);
        }
    }
}
fn save(event: On<ScreenshotCaptured>, capture: Res<Capture>, mut exit: MessageWriter<AppExit>) {
    std::fs::create_dir_all(capture.path.parent().unwrap()).unwrap();
    event
        .image
        .clone()
        .try_into_dynamic()
        .unwrap()
        .save(&capture.path)
        .unwrap();
    println!("Saved {}", capture.path.display());
    exit.write(AppExit::Success);
}
