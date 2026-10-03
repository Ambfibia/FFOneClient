//! Optional, reproducible tile transition through the production streamer.
//! FFONE_PROBE_TILE_STEP="512 0 0" moves the camera target after initial load.
use super::*;
use ffone_client::world::{PendingNativeWorldSceneSpawn, PendingNativeWorldSceneUnload};

#[derive(Resource)]
struct Transition {
    delta: Vec3,
    started: Option<Instant>,
    frames: Vec<serde_json::Value>,
    finished: bool,
}

pub(super) fn install(app: &mut App) {
    let Ok(value) = env::var("FFONE_PROBE_TILE_STEP") else {
        return;
    };
    let values = value
        .split_whitespace()
        .map(|v| v.parse::<f32>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(values.len(), 3);
    let delta = Vec3::new(values[0], values[1], values[2]);
    assert!(delta.is_finite());
    app.insert_resource(Transition {
        delta,
        started: None,
        frames: Vec::new(),
        finished: false,
    })
    .add_systems(Last, advance.before(measure));
}

fn advance(world: &mut World) {
    let mut transition = world.remove_resource::<Transition>().unwrap();
    if transition.finished {
        world.insert_resource(transition);
        return;
    }
    let now = Instant::now();
    if let Some(started) = transition.started {
        let status = world.resource::<NativeWorldStreamingStatus>().clone();
        assert!(
            status.resident_tiles <= 9,
            "transition exceeded physical tile cap"
        );
        let unloading = world
            .query::<&PendingNativeWorldSceneUnload>()
            .iter(world)
            .count();
        let admitting = world
            .query::<&PendingNativeWorldSceneSpawn>()
            .iter(world)
            .count();
        let anchor = world.resource::<Probe>().anchor;
        let expected = world
            .resource::<NativeWorldCatalog>()
            .legacy_stream_target_tiles(ffone_client::world::NativeWorldScope::WorldMap, anchor)
            .into_iter()
            .collect::<HashSet<_>>();
        let roots = world
            .query::<(&NativeWorldSceneRoot, &NativeWorldPresentationStatus)>()
            .iter(world)
            .map(|(root, status)| (root.tile, *status))
            .collect::<Vec<_>>();
        let ready = unloading == 0
            && admitting == 0
            && status.loading_colliders == 0
            && roots
                .iter()
                .all(|(_, status)| *status == NativeWorldPresentationStatus::Ready)
            && roots.iter().map(|(tile, _)| *tile).collect::<HashSet<_>>() == expected;
        transition.frames.push(serde_json::json!({
            "seconds": now.duration_since(started).as_secs_f64(),
            "frameMs": now.duration_since(world.resource::<Probe>().previous).as_secs_f64() * 1000.0,
            "residentTiles": status.resident_tiles, "admitting": admitting,
            "unloading": unloading, "loadingColliders": status.loading_colliders,
        }));
        if ready {
            let report = serde_json::json!({"delta": transition.delta.to_array(),
                "readySeconds": now.duration_since(started).as_secs_f64(), "frames": transition.frames});
            fs::write(
                world.resource::<Probe>().output.join("transition.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .unwrap();
            transition.finished = true;
        }
    } else if world
        .resource::<Probe>()
        .ready
        .is_some_and(|ready| now.duration_since(ready) >= Duration::from_secs(1))
    {
        let target = world
            .query::<&LegacyOrbitCamera>()
            .iter(world)
            .next()
            .unwrap()
            .target;
        world.get_mut::<Transform>(target).unwrap().translation += transition.delta;
        let mut probe = world.resource_mut::<Probe>();
        probe.anchor += transition.delta;
        probe.ready = None;
        probe.samples.clear();
        transition.started = Some(now);
    }
    world.insert_resource(transition);
}
