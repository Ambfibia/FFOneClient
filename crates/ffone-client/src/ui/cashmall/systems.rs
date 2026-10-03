use super::*;

pub(super) fn advance_cashmall_lifecycle_0104(time: Res<Time>, mut state: ResMut<CashmallUiState0104>) {
    state.tick(time.delta_secs());
}
