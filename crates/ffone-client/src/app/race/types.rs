use super::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in super::super) struct RaceInstanceMapInfo0104 {
    pub(in super::super) bounds: [i32; 4],
    pub(in super::super) ep_id: i32,
    pub(in super::super) top_record: ffone_client::race_ui::mode::RaceTopRecord,
    pub(in super::super) switch_count: i32,
}

#[derive(SystemParam)]
pub(in super::super) struct RaceProductionOwners<'w, 's> {
    pub(in super::super) production: ResMut<'w, RaceProductionRuntime>,
    pub(in super::super) content: Res<'w, TutorialMissionContent>,
    pub(in super::super) system_messages: ResMut<'w, SystemMessageUiModel>,
    pub(in super::super) mission_ui: ResMut<'w, MissionUiModel>,
    pub(in super::super) inventory: ResMut<'w, LocalInventoryRuntime>,
    pub(in super::super) audio_catalog: Res<'w, NativeAudioCatalog>,
    pub(in super::super) voice_language: Res<'w, VoiceLanguage>,
    pub(in super::super) option_runtime: Res<'w, OptionProductionRuntime>,
    pub(in super::super) bridge: Res<'w, NetworkBridge>,
    pub(in super::super) status: ResMut<'w, RuntimeStatus>,
    pub(in super::super) cursors: Query<'w, 's, &'static mut CursorOptions, With<PrimaryWindow>>,
}
