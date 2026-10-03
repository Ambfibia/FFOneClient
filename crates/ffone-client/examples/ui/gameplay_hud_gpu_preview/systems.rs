use super::*;

pub(super) fn sync_preview_warp_hud(
    model: Res<MissionUiModel>,
    mut hud: Query<&mut Visibility, With<GameplayHud>>,
) {
    if env::var("FFONE_MISSION_UI_PREVIEW").as_deref() != Ok("warp-departure") {
        return;
    }
    for mut visibility in &mut hud {
        *visibility = if model.npc_letterbox_visible() {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}
