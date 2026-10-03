use super::*;

pub(in super::super) fn reset_combi_session_0104(
    mut commands: Commands,
    mut runtime: ResMut<CombiProductionRuntime0104>,
    mut state: ResMut<CombiUiState0104>,
    mut projection: ResMut<CombiModeProjection0104>,
    mut outbox: ResMut<CombiUiOutbox0104>,
    mut shell: ResMut<CombiProductionShell0104>,
    mut frames: ResMut<CombiNetworkFrameInbox0104>,
    mut messages: ResMut<SystemMessageUiModel>,
) {
    reset_combi_shell_0104(
        &mut commands,
        &mut runtime,
        &mut state,
        &mut projection,
        &mut outbox,
        &mut shell,
        &mut frames,
        &mut messages,
    );
}
