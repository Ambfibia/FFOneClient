//! Native ES705 render probe. Uses the production effect compiler and renderer,
//! waits for recursive preload, then captures one second of effect playback.
use bevy::{
    asset::AssetPlugin,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    time::TimeUpdateStrategy,
    window::WindowResolution,
};
use ffone_client::{
    assets::AssetLocator,
    legacy_model_material::LegacyModelMaterialPlugin,
    tutorial_effects_runtime::{
        TutorialEffectLibrary, TutorialEffectPlacement, TutorialEffectRuntime,
        TutorialEffectRuntimeCommand, TutorialEffectsRuntimePlugin,
    },
    tutorial_mission_content::TutorialMissionContent,
};
use std::{
    env, fs,
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Resource)]
struct Probe {
    output: PathBuf,
    started: Instant,
    frame: Option<u32>,
    saved: bool,
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game")
        .canonicalize()
        .unwrap();
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/ui-parity/nano-creation-effect.png"));
    let runtime = TutorialEffectRuntime::with_library(TutorialEffectLibrary::load(&root).unwrap());
    let locator = AssetLocator::open(root.clone()).unwrap();
    let content = TutorialMissionContent::open(&locator).unwrap();
    App::new()
        .insert_resource(runtime)
        .insert_resource(content)
        .insert_resource(Probe {
            output,
            started: Instant::now(),
            frame: None,
            saved: false,
        })
        .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.055)))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne Nano creation effect probe".into(),
                        resolution: WindowResolution::new(900, 640),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((LegacyModelMaterialPlugin, TutorialEffectsRuntimePlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, capture)
        .run();
}

fn setup(mut commands: Commands, mut effects: ResMut<TutorialEffectRuntime>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 8.0).looking_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            ..default()
        },
        Transform::from_xyz(3.0, 6.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    effects.enqueue(TutorialEffectRuntimeCommand::Preload {
        effect_id: 705,
        source_line: 0,
    });
}

fn capture(
    mut commands: Commands,
    mut effects: ResMut<TutorialEffectRuntime>,
    mut probe: ResMut<Probe>,
    mut strategy: ResMut<TimeUpdateStrategy>,
    mut exit: MessageWriter<AppExit>,
) {
    let issues = effects.drain_issues().collect::<Vec<_>>();
    assert!(issues.is_empty(), "effect probe issues: {issues:?}");
    if probe.saved {
        exit.write(AppExit::Success);
        return;
    }
    assert!(
        probe.started.elapsed() < Duration::from_secs(45),
        "effect probe timed out"
    );
    if probe.frame.is_none() && effects.is_native_preload_complete(705) {
        effects.enqueue(TutorialEffectRuntimeCommand::Add {
            effect_id: 705,
            placement: TutorialEffectPlacement::World {
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
            },
            scale: 1.0,
            tracked: true,
            name: Some("nano-creation-probe".into()),
            destroy_after_seconds: None,
            source_line: 0,
        });
        *strategy = TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0));
        probe.frame = Some(0);
    } else if let Some(frame) = &mut probe.frame {
        *frame += 1;
        if *frame == 60 {
            commands.spawn(Screenshot::primary_window()).observe(save);
        }
    }
}

fn save(event: On<ScreenshotCaptured>, mut probe: ResMut<Probe>) {
    let image = event.image.clone().try_into_dynamic().unwrap();
    if let Some(parent) = probe.output.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    image.save(&probe.output).unwrap();
    eprintln!("saved {}", probe.output.display());
    probe.saved = true;
}
