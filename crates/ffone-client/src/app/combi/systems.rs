use super::*;

pub(in super::super) fn sync_combi_presentation_0104(
    runtime: Res<CombiProductionRuntime0104>,
    mut state: ResMut<CombiUiState0104>,
    mut projection: ResMut<CombiModeProjection0104>,
) {
    runtime.write_presentation(&mut state, &mut projection);
    if !runtime.modal_active() {
        state.external_modal = CombiExternalModalState0104::default();
    }
}
