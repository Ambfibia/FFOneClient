//! Captures the production shared input tree in either authored locale.
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    window::WindowResolution,
};
use ffone_client::{
    localization::{Localization, LocalizationPlugin, LocalizedText},
    shared_input_ui::*,
};
use std::{path::PathBuf, time::Instant};

#[derive(Resource)]
struct Capture {
    output: PathBuf,
    started: Instant,
    requested: bool,
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let locale = args.get(1).map(String::as_str).unwrap_or("en");
    let output = args
        .get(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("target/ui-parity/shared-input-{locale}.png")));
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game")
        .canonicalize()
        .unwrap();
    let (localization, language) = Localization::open(&root, locale).unwrap();
    App::new()
        .insert_resource(localization)
        .insert_resource(language)
        .insert_resource(Capture {
            output,
            started: Instant::now(),
            requested: false,
        })
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Shared input UI verification".into(),
                        resolution: WindowResolution::new(1280, 720),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((LocalizationPlugin, SharedInputUiPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, capture)
        .run();
}

fn setup(
    mut commands: Commands,
    mut input: ResMut<SharedInputDialog>,
    mut capture: ResMut<Capture>,
) {
    capture.started = Instant::now();
    commands.spawn((Camera2d, IsDefaultUiCamera));
    input.open(SharedInputRequest {
        owner: 1,
        max_utf16: 32,
        title: LocalizedText::new("ui.enchant.redeem_code", "REDEEM CODE"),
        instruction: LocalizedText::new(
            "ui.shared_input.redeem_instruction",
            "Have a code? Try entering it below to get an exclusive item!",
        ),
        submit: LocalizedText::new("ui.shared_input.redeem", "REDEEM"),
    });
    input.append("BeMore");
}

fn capture(mut commands: Commands, mut state: ResMut<Capture>, mut frames: Local<u32>) {
    *frames += 1;
    assert!(state.started.elapsed().as_secs() < 30, "capture timeout");
    if state.requested || *frames < 120 || state.started.elapsed().as_secs() < 3 {
        return;
    }
    state.requested = true;
    commands.spawn(Screenshot::primary_window()).observe(save);
}

fn save(
    event: On<ScreenshotCaptured>,
    state: Res<Capture>,
    mut exit: MessageWriter<AppExit>,
    texts: Query<(&LocalizedText, &Text)>,
) {
    if let Some(parent) = state.output.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let rows: Vec<_> = texts
        .iter()
        .map(|(key, text)| serde_json::json!({"key":key.key,"text":text.0}))
        .collect();
    std::fs::write(
        state.output.with_extension("json"),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
    let image = event.image.clone().try_into_dynamic().unwrap();
    let rgba = image.to_rgba8();
    let input_ink = rgba
        .enumerate_pixels()
        .filter(|(x, y, pixel)| {
            (453..500).contains(x) && (352..372).contains(y) && pixel.0[0] > 50 && pixel.0[1] > 80
        })
        .count();
    assert!(
        input_ink > 20,
        "prefilled code has no visible glyphs: {input_ink}"
    );
    image.save(&state.output).unwrap();
    println!("{}", state.output.display());
    exit.write(AppExit::Success);
}
