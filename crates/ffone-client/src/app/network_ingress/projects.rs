use super::*;

#[derive(SystemParam)]
pub(super) struct NetworkSessionIngress<'w> {
    pub(super) session_resets: MessageWriter<'w, NetworkSessionReset>,
    pub(super) world_combat: ResMut<'w, WorldCombatLifecycle>,
    pub(super) instance_audio: ResMut<'w, RetrobutionInstanceAudioState>,
    pub(super) active_credentials: ResMut<'w, ActiveLoginCredentials>,
    pub(super) app_exit: MessageWriter<'w, AppExit>,
    pub(super) character_preview: ResMut<'w, NativePlayerPreviewModel>,
    pub(super) rig_assets: ResMut<'w, NativePlayerRigAssetCache>,
    pub(super) weapon_animation_catalog: Res<'w, PlayerWeaponAnimationCatalog>,
    pub(super) gameplay_loading: ResMut<'w, GameplayLoadingState>,
    pub(super) player_commands: ResMut<'w, TutorialPlayerPresentationCommandQueue>,
    pub(super) effect_runtime: ResMut<'w, TutorialEffectRuntime>,
    pub(super) gameplay_audio: ResMut<'w, GameplayAudioRuntime>,
    pub(super) music_requests: ResMut<'w, ffone_client::world_audio::RetrobutionMusicRequests>,
}
