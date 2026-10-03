//! FFONE_PERF_AUDIO=1: real output sinks, synthetic offline zone transitions.
use super::*;
use bevy::audio::{AudioSink, AudioSinkPlayback};

struct Case {
    position: Vec3,
    instance: bool,
    name: String,
}

#[derive(Resource)]
struct Probe {
    cases: Vec<Case>,
    stage: usize,
    entered: bool,
    started: Option<Instant>,
    audible_since: Option<f64>,
    passed: Vec<String>,
}

pub(super) fn install(app: &mut App) {
    let root = &app.world().resource::<ClientConfig>().asset_root;
    let document: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join(ffone_client::world_audio::RETROBUTION_WORLD_AUDIO_PATH)).unwrap(),
    )
    .unwrap();
    let zones = document["musicZones"].as_array().unwrap();
    let time_machine = zones
        .iter()
        .position(|zone| zone["logicalKey"] == "music/time_machine")
        .unwrap();
    let cases = [61, 4, 61, 5, 61, time_machine, 61]
        .map(|index| {
            let zone = &zones[index];
            let points = zone["points"].as_array().unwrap();
            // The time-machine row covers separated rooms. Its overall centroid
            // is outside the authored polygon; sample the first room instead.
            let sample = if index == time_machine {
                &points[..4]
            } else {
                points.as_slice()
            };
            let center = sample
                .iter()
                .fold([0.0, 0.0], |a, p| {
                    [a[0] + p[0].as_f64().unwrap(), a[1] + p[1].as_f64().unwrap()]
                })
                .map(|v| v / sample.len() as f64);
            Case {
                position: Vec3::new(-center[0] as f32, -50.0, center[1] as f32),
                instance: zone["instance"].as_bool().unwrap(),
                name: format!(
                    "Retrobution music {}",
                    zone["sourceTrueName"].as_str().unwrap()
                ),
            }
        })
        .into_iter()
        .collect();
    app.insert_resource(Probe {
        cases,
        stage: 0,
        entered: false,
        started: None,
        audible_since: None,
        passed: Vec::new(),
    })
    .add_systems(
        Update,
        discard_offline_network_errors.before(network_ingress::poll_network),
    )
    .add_systems(Last, drive.before(measure));
}

fn discard_offline_network_errors(bridge: Res<NetworkBridge>) {
    // The offline fixture has no shard. Periodic gameplay sends otherwise
    // produce Error events that issue "stop" and invalidate an audio replay.
    for event in bridge.drain() {
        assert!(
            matches!(event, NetworkEvent::Error(_)),
            "audio fixture must stay offline: {event:?}"
        );
    }
}

fn drive(world: &mut World) {
    if *world.resource::<State<ClientState>>().get() != ClientState::World {
        return;
    }
    let mut probe = world.remove_resource::<Probe>().unwrap();
    if probe.started.is_none() && world.resource::<GameplayLoadingState>().visible {
        world.insert_resource(probe);
        return;
    }
    let started = *probe.started.get_or_insert_with(Instant::now);
    assert!(
        started.elapsed() < Duration::from_secs(150),
        "audio stage {} timed out",
        probe.stage
    );
    let case = &probe.cases[probe.stage];
    world.resource_mut::<GameplayLoadingState>().finish();
    world.resource_mut::<RetrobutionInstanceAudioState>().active = case.instance;
    if !probe.entered {
        let mut query = world.query_filtered::<(
            &mut Transform,
            &mut GlobalTransform,
            &mut LegacyPlayerController,
        ), With<LocalPlayer>>();
        let (mut transform, mut global, mut controller) = query.single_mut(world).unwrap();
        transform.translation = case.position;
        *global = GlobalTransform::from(*transform);
        controller.movement_enabled = false;
        probe.entered = true;
    }
    let active: Vec<_> = world
        .query::<(&Name, &AudioSink)>()
        .iter(world)
        .filter(|(name, sink)| name.as_str().starts_with("Retrobution music ") && !sink.empty())
        .map(|(name, _)| name.as_str().to_owned())
        .collect();
    assert!(active.len() <= 1, "music overlap: {active:?}");
    let now = world.resource::<Time>().elapsed_secs_f64();
    if active.first() == Some(&case.name) {
        let audible_since = *probe.audible_since.get_or_insert(now);
        if now - audible_since >= 1.2 {
            probe.passed.push(case.name.clone());
            probe.stage += 1;
            probe.entered = false;
            probe.audible_since = None;
            if probe.stage == probe.cases.len() {
                let output = &world.resource::<Capture>().output;
                fs::write(
                    output.join("audio.json"),
                    serde_json::to_vec_pretty(&serde_json::json!({
                        "passed": probe.passed, "actualOutputSinks": true, "musicOverlap": false,
                        "syntheticOfflineTransitions": true, "serverPortalsVerified": false,
                    }))
                    .unwrap(),
                )
                .unwrap();
                world.write_message(AppExit::Success);
            }
        }
    } else {
        probe.audible_since = None;
    }
    world.insert_resource(probe);
}
