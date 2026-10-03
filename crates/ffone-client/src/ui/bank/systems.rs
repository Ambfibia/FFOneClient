use super::*;

pub(super) fn advance_bank_lifecycle(time: Res<Time>, mut state: ResMut<BankUiState>) {
    state.tick(time.delta_secs());
}
