use super::*;

pub(super) fn ordinary_world_action_collector_active(state: Res<State<ClientState>>) -> bool {
    client_state_uses_ordinary_world_action_collector(*state.get())
}
