//! GPU acceptance for multiline NPC mission names.
use bevy::{
    asset::AssetPlugin,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    text::TextLayoutInfo,
    window::WindowResolution,
};
use ffone_client::{
    assets::AssetLocator,
    gameplay_nano_portraits::{JournalNanoPortraitImage, JournalNanoPortraitRequest},
    localization::{Localization, LocalizationPlugin},
    mission_ui::{MissionUiEntry, MissionUiModel, MissionUiPlugin, NpcInteractionUi},
    tutorial_mission_content::TutorialMissionContent,
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
#[derive(Resource)]
struct Capture {
    path: PathBuf,
    start: Instant,
    sent: bool,
}
fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game")
        .canonicalize()
        .unwrap();
    let (localization, language) = Localization::open(&root, "ru").unwrap();
    let path = PathBuf::from("target/performance/menu-bugs/mission-long-ru.png");
    App::new()
        .insert_resource(Capture {
            path,
            start: Instant::now(),
            sent: false,
        })
        .insert_resource(TutorialMissionContent::open(&AssetLocator::open(&root).unwrap()).unwrap())
        .insert_resource(localization)
        .insert_resource(language)
        .init_resource::<JournalNanoPortraitImage>()
        .init_resource::<JournalNanoPortraitRequest>()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: root.to_string_lossy().into(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Mission dialogue acceptance".into(),
                        resolution: WindowResolution::new(1264, 681),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((LocalizationPlugin, MissionUiPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, capture)
        .run();
}
fn setup(mut commands: Commands, mut model: ResMut<MissionUiModel>) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    model.enabled = true;
    model.npc_icon_mode_visible = true;
    model.npc_interaction=Some(NpcInteractionUi{npc_id:1,npc_type:0,name:"Проверка диалога".into(),available_missions:vec![
        MissionUiEntry{task_id:999999,mission_type:3,title:"Очень длинное название миссии: найти потерянное оборудование и вернуться к персонажу за наградой".into(),..default()},
        MissionUiEntry{task_id:999998,mission_type:3,title:"Следующая миссия".into(),..default()}],..default()});
}
fn capture(
    mut commands: Commands,
    texts: Query<(&Text, &TextLayoutInfo, &ComputedNode)>,
    mut capture: ResMut<Capture>,
) {
    if capture.sent {
        return;
    }
    assert!(
        capture.start.elapsed() < Duration::from_secs(60),
        "mission wrapping timeout"
    );
    if capture.start.elapsed() < Duration::from_secs(5) {
        return;
    }
    for (text, layout, node) in &texts {
        if text.0.contains("Очень длинное название") && layout.size.y > 35. {
            assert!(
                layout.size.y <= node.size().y + 2.,
                "mission glyphs must fit growing text node"
            );
            println!(
                "Multiline mission: glyphs={:?} node={:?}",
                layout.size,
                node.size()
            );
            capture.sent = true;
            commands.spawn(Screenshot::primary_window()).observe(save);
            return;
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
