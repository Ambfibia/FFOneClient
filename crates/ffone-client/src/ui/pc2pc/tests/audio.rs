use super::*;
use crate::gameplay_ui::{GameplayUiAudioCue, GameplayUiAudioOutbox};
use crate::pc2pc_ui::gestures::{TarosEditor, interact};
use bevy::{input::keyboard::KeyboardInput, prelude::*};

#[test]
fn ui_sfx_trade_confirmation_requires_an_enabled_action_and_does_not_repeat() {
    for remote_ready in [false, true] {
        let mut app = App::new();
        let mut model = model_with(500, &[]);
        model.state.remote_ready = remote_ready;
        app.insert_resource(model)
            .init_resource::<Pc2pcModalState>()
            .init_resource::<Pc2pcBackendCapabilities>()
            .init_resource::<TarosEditor>()
            .init_resource::<GameplayUiAudioOutbox>()
            .add_message::<KeyboardInput>()
            .add_systems(Update, interact);
        let button = app
            .world_mut()
            .spawn((Pc2pcUiElement::MainButton, Interaction::Pressed))
            .id();
        app.world_mut()
            .resource_mut::<Pc2pcModalState>()
            .system_popup_active = true;
        app.update();
        assert_eq!(
            app.world_mut()
                .resource_mut::<GameplayUiAudioOutbox>()
                .drain()
                .count(),
            0
        );
        assert!(
            app.world()
                .resource::<Pc2pcUiModel0104>()
                .state
                .pending
                .is_none()
        );

        app.world_mut()
            .resource_mut::<Pc2pcModalState>()
            .system_popup_active = false;
        *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::None;
        app.update();
        *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Pressed;
        app.update();
        assert_eq!(
            app.world_mut()
                .resource_mut::<GameplayUiAudioOutbox>()
                .drain()
                .collect::<Vec<_>>(),
            [GameplayUiAudioCue::YesButton]
        );
        assert!(
            app.world()
                .resource::<Pc2pcUiModel0104>()
                .state
                .pending
                .is_some()
        );
        app.update();
        assert_eq!(
            app.world_mut()
                .resource_mut::<GameplayUiAudioOutbox>()
                .drain()
                .count(),
            0
        );
        // Even a second physical click cannot repeat an unacknowledged Submit.
        *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::None;
        app.update();
        *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Pressed;
        app.update();
        assert_eq!(
            app.world_mut()
                .resource_mut::<GameplayUiAudioOutbox>()
                .drain()
                .count(),
            0
        );
    }
}
