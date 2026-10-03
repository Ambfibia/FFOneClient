//! Separate controller camera options, using the normal Controls draft.
use super::*;
use super::binding_controls_page::option_slider_thumb_left;

#[derive(Component)]
pub(super) struct OptionPadInvertYButton(pub bool);
#[derive(Component)]
pub(super) struct OptionPadSensitivityButton(pub u8);
#[derive(Component)]
pub(super) struct OptionPadSensitivityThumb;

pub(super) fn handle_pad_controls(
    invert: Query<(&Interaction, &OptionPadInvertYButton), Changed<Interaction>>,
    sensitivity: Query<(&Interaction, &OptionPadSensitivityButton), Changed<Interaction>>,
    mut model: ResMut<OptionUiModel>,
) {
    if !model.page_body_enabled(OptionTab::Controls) {
        return;
    }
    for (interaction, button) in &invert {
        if *interaction == Interaction::Pressed && model.draft_input.pad_invert_y != button.0 {
            model.draft_input.pad_invert_y = button.0;
            model.mark_input_edited();
        }
    }
    for (interaction, button) in &sensitivity {
        let value = f32::from(button.0.clamp(1, 10));
        if *interaction == Interaction::Pressed && model.draft_input.pad_camera_sensitivity != value
        {
            model.draft_input.pad_camera_sensitivity = value;
            model.mark_input_edited();
        }
    }
}

pub(super) fn bind_pad_controls(
    model: Res<OptionUiModel>,
    assets: Res<OptionUiAssets>,
    mut queries: ParamSet<(
        Query<(&OptionPadInvertYButton, &mut ImageNode, &mut Pickable)>,
        Query<&mut Pickable, With<OptionPadSensitivityButton>>,
    )>,
    mut thumbs: Query<&mut Node, With<OptionPadSensitivityThumb>>,
) {
    let enabled =
        model.selected_tab == OptionTab::Controls && model.page_body_enabled(OptionTab::Controls);
    for (button, mut image, mut pickable) in &mut queries.p0() {
        image.image = assets.image(if model.draft_input.pad_invert_y == button.0 {
            OptionTextureRole::RadioChecked
        } else {
            OptionTextureRole::RadioEmpty
        });
        *pickable = pickable_for(enabled);
    }
    for mut pickable in &mut queries.p1() {
        *pickable = pickable_for(enabled);
    }
    let fraction = (model.draft_input.pad_camera_sensitivity.clamp(1.0, 10.0) - 1.0) / 9.0;
    for mut thumb in &mut thumbs {
        thumb.left = px(option_slider_thumb_left(680.0, fraction));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pad_camera_buttons_commit_and_cancel_through_controls_owner() {
        let mut app = App::new();
        let mut model = OptionUiModel::default();
        model.open(OptionSettings::default(), InputSettings::default(), OptionOpenAudioRoute::default(), &mut OptionUiOutbox::default());
        model.select_tab(OptionTab::Controls);
        app.insert_resource(model).add_systems(Update, handle_pad_controls);
        app.world_mut().spawn((Interaction::Pressed, OptionPadInvertYButton(true)));
        app.world_mut().spawn((Interaction::Pressed, OptionPadSensitivityButton(2)));
        app.update();
        let model = app.world().resource::<OptionUiModel>();
        assert!(model.draft_input.pad_invert_y);
        assert_eq!(model.draft_input.pad_camera_sensitivity, 2.0);
        assert!(!model.draft_input.invert_y);
        assert_eq!(model.draft_input.camera_sensitivity, 5.0);
        let mut model = app.world_mut().resource_mut::<OptionUiModel>();
        let mut outbox = OptionUiOutbox::default();
        assert!(model.apply(&mut outbox));
        assert!(model.persisted_input.pad_invert_y);
        assert_eq!(model.persisted_input.pad_camera_sensitivity, 2.0);
        model.draft_input.pad_camera_sensitivity = 9.0;
        model.mark_input_edited();
        model.cancel(OptionCloseTrigger::Shortcut, OptionCloseAudioRoute::default(), &mut outbox);
        assert_eq!(model.draft_input.pad_camera_sensitivity, 2.0);
    }
}
