use super::*;

pub(super) fn force_preview_hover(
    cli: Res<PreviewCli>,
    mut buttons: Query<(&EmailUiButton, &mut Interaction)>,
) {
    for (button, mut interaction) in &mut buttons {
        *interaction = if button.kind == cli.scene.hover() {
            Interaction::Hovered
        } else {
            Interaction::None
        };
    }
}
