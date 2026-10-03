//! Mission-owned NPC group requests; the server still owns NPC movement.
use crate::app::*;
mod state;
use state::EscortRequests;

pub(super) fn sync_escort_requests(
    time: Res<Time>,
    session: Res<NetworkLifecycleSession>,
    mission: Res<WorldMissionRuntime>,
    content: Res<TutorialMissionContent>,
    group: Res<GroupProductionRuntime0104>,
    npcs: Query<&NetworkNpcAppearance0104>,
    bridge: Res<NetworkBridge>,
    mut state: Local<EscortRequests>,
) {
    let tasks = mission
        .active_tasks()
        .iter()
        .filter_map(|active| {
            let task = content.mission(active.task_id).ok()?;
            if task.provenance.task_type != 6 || task.provenance.escort_def_npc_type <= 0 {
                return None;
            }
            let npc = npcs
                .iter()
                .find(|npc| npc.0.npc_type == task.provenance.escort_def_npc_type)
                .map(|npc| npc.0.npc_id);
            Some((active.task_id, npc))
        })
        .collect::<Vec<_>>();
    let has_npc = group
        .roster()
        .is_some_and(|roster| !roster.npc_members.is_empty());
    for npc_id in state.update(
        session.active.map(|epoch| epoch.0),
        time.elapsed_secs_f64(),
        &tasks,
        has_npc,
    ) {
        if let Err(error) = bridge.send(NetworkCommand::InviteEscortNpc(npc_id)) {
            warn!("Escort group request send failed: {error}");
        }
    }
}
