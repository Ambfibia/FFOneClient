use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in super::super) struct RaceNpcSession {
    pub(in super::super) runtime_npc_id: i32,
    pub(in super::super) table_npc_id: i32,
    pub(in super::super) voice_owner: String,
    pub(in super::super) camera_subtarget_active: bool,
}

pub(in super::super) fn reset_race_session(
    mut mode: ResMut<RaceModeModel>,
    mut mode_commands: ResMut<RaceModeUiCommandOutbox>,
    mut reward: ResMut<RaceRewardPresentation>,
    mut rank: ResMut<RaceRankModel>,
    mut rank_commands: ResMut<RaceRankUiCommandOutbox>,
    mut production: ResMut<RaceProductionRuntime>,
    mut frames: ResMut<RaceNetworkFrameInbox>,
    mut hud: ResMut<RaceHudState>,
) {
    reset_race_shell(
        &mut mode,
        &mut mode_commands,
        &mut reward,
        &mut rank,
        &mut rank_commands,
        &mut production,
        &mut frames,
    );
    *hud = RaceHudState::default();
}
