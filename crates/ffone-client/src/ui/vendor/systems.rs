use super::*;

pub(super) fn advance_vendor_lifecycle(time: Res<Time>, mut state: ResMut<VendorUiState>) {
    state.tick(time.delta_secs());
}
