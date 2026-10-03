//! Opt-in offline replay of the production teleport and location loading gate.
//! FFONE_PERF_LOADING_WARP="x y z" with FFONE_PERF_OUTPUT; no server acceptance.
use super::*;
use ffone_client::world::NativeWorldLocationPresentation;

#[derive(Resource)]
struct LoadingWarpProbe {
    destination: Vec3,
    phase: u8,
    loading_observed: bool,
}

pub(super) fn install(app: &mut App) {
    let values = env::var("FFONE_PERF_LOADING_WARP")
        .unwrap()
        .split_whitespace()
        .map(|value| value.parse::<f32>().expect("native warp coordinate"))
        .collect::<Vec<_>>();
    assert!(values.len() == 3 && values.iter().all(|value| value.is_finite()));
    app.insert_resource(LoadingWarpProbe {
        destination: Vec3::from_slice(&values),
        phase: 0,
        loading_observed: false,
    })
    .add_systems(Last, verify.before(measure));
}

fn verify(
    mut commands: Commands,
    state: Res<State<ClientState>>,
    mut loading: ResMut<GameplayLoadingState>,
    location: Res<NativeWorldLocationPresentation>,
    mut capture: ResMut<Capture>,
    mut probe: ResMut<LoadingWarpProbe>,
    mut players: Query<
        (
            Entity,
            &mut Transform,
            &mut LegacyPlayerController,
            &mut Visibility,
            &mut LegacyAvatarEnvironmentState,
        ),
        With<LocalPlayer>,
    >,
) {
    if *state.get() != ClientState::World || probe.phase == 2 {
        return;
    }
    assert!(
        loading.blocked.is_none(),
        "location loader blocked: {:?}",
        loading.blocked
    );
    let Ok((player, mut transform, mut controller, mut visibility, mut environment)) =
        players.single_mut()
    else {
        return;
    };
    if probe.phase == 0 {
        if loading.visible {
            return;
        }
        assert!(
            location.ready,
            "source loader closed before location presentation"
        );
        commands.spawn(Screenshot::primary_window()).observe(
            bevy::render::view::screenshot::save_to_disk(capture.output.join("loading-source.png")),
        );
        assert!(npc_warp::apply_authoritative_world_teleport(
            &mut commands,
            &mut loading,
            player,
            &mut transform,
            &mut controller,
            &mut visibility,
            &mut environment,
            probe.destination,
            Some(1000),
        ));
        capture.position = probe.destination;
        capture.ready = None;
        capture.samples.clear();
        probe.phase = 1;
        info!("Location loading probe: production teleport entered destination barrier");
        return;
    }
    if loading.visible {
        assert!(
            !controller.movement_enabled,
            "movement enabled while destination loads"
        );
        if !probe.loading_observed {
            commands.spawn(Screenshot::primary_window()).observe(
                bevy::render::view::screenshot::save_to_disk(
                    capture.output.join("loading-destination.png"),
                ),
            );
            probe.loading_observed = true;
        }
        return;
    }
    assert!(
        probe.loading_observed,
        "destination loading barrier was skipped"
    );
    assert!(
        location.ready && location.position == Some(transform.translation),
        "destination loader closed with stale/incomplete presentation: {location:?}"
    );
    assert!(controller.movement_enabled);
    fs::write(
        capture.output.join("loading-warp.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "offline": true, "productionTeleport": true, "sourcePresentationReady": true,
            "destinationLoadingObserved": true, "movementBlockedDuringLoading": true,
            "destinationPresentationReadyBeforeLoaderClosed": true,
            "destination": transform.translation.to_array(),
        }))
        .unwrap(),
    )
    .unwrap();
    probe.phase = 2;
    info!("Location loading probe: destination admitted; loader and movement gates passed");
}
