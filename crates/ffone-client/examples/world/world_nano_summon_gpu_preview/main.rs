//! Exercises the production summon/loading/animation path with an actual Nano GLB.
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    time::TimeUpdateStrategy,
    window::WindowResolution,
};
use ffone_client::{
    assets::AssetLocator,
    gameplay_nano_portraits::GameplayNanoPortraitCatalog,
    legacy_model_material::LegacyModelMaterialPlugin,
    tutorial_mission_content::TutorialMissionContent,
    tutorial_nano_gameplay::{
        TutorialNanoGameplayCommandQueue, TutorialNanoGameplayPlugin, TutorialNanoGameplayState,
        TutorialNanoGameplayStatus, WorldNanoGameplayPresentation,
    },
};
use std::{
    collections::VecDeque,
    env, fs,
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Resource)]
struct Probe {
    nano_id: i16,
    presentation: WorldNanoGameplayPresentation,
    output: PathBuf,
    started: Instant,
    frames: u32,
    saved: bool,
    remaining: VecDeque<(i16, WorldNanoGameplayPresentation)>,
    batch_output: Option<PathBuf>,
    owner: Option<Entity>,
    exercise_skill: bool,
}

fn main() {
    let selection = std::env::args().nth(1).unwrap_or_else(|| "24".into());
    let batch = selection == "all" || selection.contains(',');
    // Optional third argument exercises a selected skill after call completes.
    let requested_skill = std::env::args().nth(3).map(|value| {
        let slot: u8 = value.parse().expect("skill slot must be 1, 2 or 3");
        assert!((1..=3).contains(&slot));
        slot
    });
    let output = std::env::args_os().nth(2).map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(if batch {
            "target/performance/all-nano-summons"
        } else {
            "target/performance/swampfire-summon.png"
        })
    });
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game")
        .canonicalize()
        .unwrap();
    let locator = AssetLocator::open(root.clone()).unwrap();
    let content = TutorialMissionContent::open(&locator).unwrap();
    let catalog = GameplayNanoPortraitCatalog::open(&locator).unwrap();
    let ids = if selection == "all" {
        content
            .gameplay_nanos()
            .map(|nano| nano.nano_id)
            .collect::<Vec<_>>()
    } else {
        selection.split(',').map(|id| id.parse().unwrap()).collect()
    };
    let mut remaining = VecDeque::new();
    for nano_id in ids {
        let first_skill = if batch {
            1
        } else {
            requested_skill.unwrap_or(1)
        };
        for skill_slot in first_skill..=if batch { 3 } else { first_skill } {
            remaining.push_back((
                nano_id,
                WorldNanoGameplayPresentation {
                    model_path: catalog.model_path(nano_id).expect("Nano model").to_owned(),
                    style: content.gameplay_nano(nano_id).expect("Nano row").style,
                    skill_slot,
                },
            ));
        }
    }
    eprintln!("Checking {} Nano/skill summon cases", remaining.len());
    let (nano_id, presentation) = remaining.pop_front().unwrap();
    let batch_output = batch.then(|| output.clone());
    let output = if batch {
        output.join(format!("nano-{nano_id:03}-skill-1.png"))
    } else {
        output
    };
    App::new()
        .insert_resource(locator)
        .insert_resource(Probe {
            nano_id,
            presentation,
            output,
            started: Instant::now(),
            frames: 0,
            saved: false,
            remaining,
            batch_output,
            owner: None,
            exercise_skill: requested_skill.is_some(),
        })
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
        .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.055)))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne Nano summon acceptance".into(),
                        resolution: WindowResolution::new(900, 640),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((LegacyModelMaterialPlugin, TutorialNanoGameplayPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, capture)
        .run();
}

fn setup(
    mut commands: Commands,
    mut probe: ResMut<Probe>,
    mut queue: ResMut<TutorialNanoGameplayCommandQueue>,
) {
    let owner = commands
        .spawn((Transform::IDENTITY, Visibility::Inherited))
        .id();
    queue.equip_world(probe.nano_id, 1, 150, probe.presentation.clone());
    queue.summon(owner);
    probe.owner = Some(owner);
    let target = Vec3::new(0.7, 1.4, 0.0);
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.7, 1.4, -2.0).looking_at(target, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(-3.0, 5.0, 4.0).looking_at(target, Vec3::Y),
    ));
}

fn capture(
    mut commands: Commands,
    nano: Res<TutorialNanoGameplayState>,
    mut probe: ResMut<Probe>,
    mut strategy: ResMut<TimeUpdateStrategy>,
    mut exit: MessageWriter<AppExit>,
    mut queue: ResMut<TutorialNanoGameplayCommandQueue>,
) {
    if probe.saved {
        if let Some((nano_id, presentation)) = probe.remaining.pop_front() {
            probe.nano_id = nano_id;
            probe.output = probe.batch_output.as_ref().unwrap().join(format!(
                "nano-{nano_id:03}-skill-{}.png",
                presentation.skill_slot
            ));
            probe.presentation = presentation;
            probe.frames = 0;
            probe.saved = false;
            probe.started = Instant::now();
            *strategy = TimeUpdateStrategy::ManualDuration(Duration::ZERO);
            queue.equip_world(
                nano_id,
                i16::from(probe.presentation.skill_slot),
                150,
                probe.presentation.clone(),
            );
            queue.summon(probe.owner.unwrap());
        } else {
            exit.write(AppExit::Success);
        }
        return;
    }
    assert!(
        probe.started.elapsed() < Duration::from_secs(60),
        "Nano summon timed out: {:?}",
        nano.status()
    );
    assert!(
        !matches!(nano.status(), TutorialNanoGameplayStatus::Blocked(_)),
        "Nano {} skill {} blocked: {:?}",
        probe.nano_id,
        probe.presentation.skill_slot,
        nano.status()
    );
    if nano.is_active() {
        *strategy = TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0));
        probe.frames += 1;
        if probe.exercise_skill && probe.frames == 60 {
            queue.play_world_skill(probe.owner.unwrap());
        }
        if probe.frames == if probe.exercise_skill { 80 } else { 30 } {
            commands.spawn(Screenshot::primary_window()).observe(save);
        }
    }
}

fn save(event: On<ScreenshotCaptured>, mut probe: ResMut<Probe>) {
    if let Some(parent) = probe.output.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    event
        .image
        .clone()
        .try_into_dynamic()
        .unwrap()
        .save(&probe.output)
        .unwrap();
    eprintln!(
        "Nano {} active; saved {}",
        probe.nano_id,
        probe.output.display()
    );
    probe.saved = true;
}
