//! Replay the production NanoPowerB finale with its authored camera and actors.
use super::*;
use ffone_client::tutorial_logic::{LegacySpawnPosition, TutorialNpcSpawn};

#[derive(Resource, Default)]
struct FinaleProbe { warmup: u32, sample: usize, pending: usize }

pub(super) fn install(app: &mut App) {
    app.init_resource::<FinaleProbe>()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0)))
        .add_systems(Last, drive.before(super::measure));
}

fn drive(world: &mut World) {
    if *world.resource::<State<ClientState>>().get() != ClientState::Tutorial
        || world.resource::<GameplayLoadingState>().visible { return; }
    let frame = { let mut p = world.resource_mut::<FinaleProbe>(); p.warmup += 1; p.warmup };
    if frame == 1 {
        let mut actors = world.resource_mut::<TutorialActorCommandQueue>();
        actors.spawn(TutorialNpcSpawn::new(5100, 2671,
            LegacySpawnPosition::centiunits(90_700, 68_500, 1_100), Some(180)));
        actors.spawn(TutorialNpcSpawn::new(5101, 2800,
            LegacySpawnPosition::centiunits(90_800, 68_200, 1_100), Some(104)));
    }
    if frame == 120 {
        let mut tutorial = world.resource_mut::<TutorialSession>();
        tutorial.init_chapter(5).unwrap();
        tutorial.init_step(8);
        tutorial.scene = TutorialScene::NanoPowerB;
    }
    if frame < 120 { return; }
    let elapsed = world.resource::<TutorialChoreographyPlayer>().presentation_elapsed_seconds();
    let times = [25.8, 26.5, 27.8, 28.5, 30.0, 33.0, 36.0];
    let probe = world.resource::<FinaleProbe>();
    if probe.sample == times.len() {
        if probe.pending == 0 { world.write_message(AppExit::Success); }
        return;
    }
    if elapsed < times[probe.sample] { return; }
    let path = world.resource::<Capture>().output.join(format!("finale-{:04}.png", (times[probe.sample] * 10.0) as u32));
    { let mut p = world.resource_mut::<FinaleProbe>(); p.sample += 1; p.pending += 1; }
    world.spawn(Screenshot::primary_window()).observe(
        move |event: On<ScreenshotCaptured>, mut probe: ResMut<FinaleProbe>| {
            event.image.clone().try_into_dynamic().unwrap().save(&path).unwrap();
            probe.pending -= 1;
        });
}
