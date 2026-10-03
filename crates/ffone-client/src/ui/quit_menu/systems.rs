use super::*;

pub(super) fn sync_quit_menu_visibility(
    model: Res<QuitMenuUiModel>,
    mut presentation: ResMut<QuitMenuPresentationState>,
    mut audio: ResMut<QuitMenuAudioOutbox>,
    mut roots: Query<&mut Node, With<QuitMenuUiRoot>>,
) {
    if presentation.visible != model.visible {
        audio.push(if model.visible {
            QuitMenuAudioCue::OpenScreen
        } else {
            QuitMenuAudioCue::CloseScreen
        });
        presentation.visible = model.visible;
    }
    for mut root in &mut roots {
        root.display = if model.visible {
            Display::Flex
        } else {
            Display::None
        };
    }
}
