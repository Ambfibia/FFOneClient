use crate::app::*;

/// A destination reload needs a fresh shard acknowledgment. Initial entry
/// already completes that handshake in the network worker.
#[derive(Component)]
pub(super) struct WarpLoadingAcknowledgment;

pub(super) fn warp_loading_complete_request() -> ffone_protocol::RegisteredGameplayRequest0104 {
    ffone_protocol::RegisteredGameplayRequest0104::new(
        ffone_protocol::packet::P_CL2FE_REQ_PC_LOADING_COMPLETE,
        ffone_protocol::PcLoadingCompleteRequest { pc_id: 0 }.encode(),
    )
    .expect("loading-complete has a registered protocol-0104 ABI")
}

pub(super) fn acknowledge_warp_loading(
    mut commands: Commands,
    state: Res<State<ClientState>>,
    loading: Res<GameplayLoadingState>,
    bridge: Res<NetworkBridge>,
    mut status: ResMut<RuntimeStatus>,
    players: Query<
        Entity,
        (
            With<LocalPlayer>,
            With<WarpLoadingAcknowledgment>,
            Without<LegacyWorldColliderPending>,
        ),
    >,
) {
    if !client_state_sends_movement_intents(*state.get()) || loading.visible {
        return;
    }
    let Ok(player) = players.single() else {
        return;
    };
    // The shard sends INSTANCE_MAP_INFO only after this notification. Keep
    // the marker on queue failure so a ready destination can retry.
    match bridge.send(NetworkCommand::SendRegisteredGameplay0104(
        warp_loading_complete_request(),
    )) {
        Ok(()) => {
            commands
                .entity(player)
                .remove::<WarpLoadingAcknowledgment>();
        }
        Err(error) => status.message = format!("Warp loading acknowledgment failed: {error}"),
    }
}

/// Keep the departure's screen ownership across the authoritative reply and
/// destination loading. A failure releases it back to the existing NPC mode.
pub(super) fn sync_warp_presentation(
    state: Res<State<ClientState>>,
    loading: Res<GameplayLoadingState>,
    normal: Res<NormalNpcWarpRuntime>,
    guide: Res<GuideProductionRuntime>,
    transportation: Res<TransportationModel>,
    mut mission: ResMut<MissionUiModel>,
) {
    let active = client_state_sends_movement_intents(*state.get())
        && (mission.pending_warp.is_some()
            || normal.pending.is_some()
            || guide.pending_warp.is_some()
            || matches!(
                transportation.phase(),
                TransportationPhase::PendingWarp | TransportationPhase::AwaitingServer
            )
            || (mission.warp_transition_active
                && loading.visible
                && loading.scope == Some(ResourceLoadingScope::World)));
    if mission.warp_transition_active != active {
        mission.warp_transition_active = active;
    }
}
