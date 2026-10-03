use super::*;

#[derive(Debug, Default, Resource)]
pub(super) struct LocalInfectionPresentationState {
    pub(super) active_player: Option<Entity>,
}

pub(super) fn client_state_sends_movement_intents(state: ClientState) -> bool {
    matches!(state, ClientState::Tutorial | ClientState::World)
}

pub(super) const fn client_state_uses_world_nano_authority(state: ClientState) -> bool {
    // The tutorial's virtual server owns equip/activate locally. Letting the
    // shard-world presenter run there dismisses the still-loading Buttercup
    // before it can emit TutorialEvent::NanoActive.
    matches!(state, ClientState::World)
}

pub(super) const fn client_state_uses_ordinary_world_action_collector(state: ClientState) -> bool {
    matches!(state, ClientState::World)
}
