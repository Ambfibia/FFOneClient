
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct RetrobutionActorEffectEvent {
    pub(super) npc_type: i32,
    pub(super) clip: &'static str,
    pub(super) source_clip_path_id: i64,
    pub(super) event_seconds: f32,
    pub(super) effect_id: i32,
    pub(super) node_name: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct RetrobutionActorDeathPresentationEvent {
    pub(super) npc_type: i32,
    pub(super) source_clip_path_id: i64,
    pub(super) event_seconds: f32,
    pub(super) effect_id: i32,
}
