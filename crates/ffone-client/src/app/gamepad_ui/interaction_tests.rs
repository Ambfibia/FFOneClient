use super::*;
use bevy::input::gamepad::Gamepad;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;

#[derive(Component, Default)]
struct Presses(usize);

fn pointer_focus(mut buttons: Query<&mut Interaction>) {
    for mut button in &mut buttons {
        button.set_if_neq(Interaction::None);
    }
}

fn count_presses(mut buttons: Query<(&Interaction, &mut Presses), Changed<Interaction>>) {
    for (interaction, mut presses) in &mut buttons {
        if *interaction == Interaction::Pressed {
            presses.0 += 1;
        }
    }
}

fn fixture(state: ClientState) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )))
        .insert_state(state)
        .init_resource::<GamepadActionState>()
        .init_resource::<PadUiFocus>()
        .init_resource::<ControllerUiInput>()
        .init_resource::<MissionUiModel>()
        .init_resource::<TutorialSession>()
        .init_resource::<LauncherUiModel>()
        .init_resource::<GameplayControllerMenuInput>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_message::<KeyboardInput>()
        .add_systems(
            Update,
            (
                prepare_gamepad_ui,
                pointer_focus,
                navigate_gamepad_ui,
                highlight_gamepad_ui,
                count_presses,
            )
                .chain(),
        );
    app
}

fn button(app: &mut App, parent: Entity, x: f32, y: f32, stack: u32) -> Entity {
    app.world_mut()
        .spawn((
            Button,
            Node::default(),
            ComputedNode {
                size: Vec2::splat(30.0),
                ..default()
            },
            UiGlobalTransform::from_translation(Vec2::new(x, y)),
            ComputedStackIndex(stack),
            ChildOf(parent),
            Presses::default(),
        ))
        .id()
}

fn frame(app: &mut App, buttons: &[GamepadButton]) {
    let previous = app.world().resource::<GamepadActionState>().buttons.clone();
    let mut pad = Gamepad::default();
    *pad.digital_mut() = previous;
    pad.digital_mut().clear();
    let held: Vec<_> = pad.get_pressed().copied().collect();
    for button in held {
        if !buttons.contains(&button) {
            pad.digital_mut().release(button);
        }
    }
    for button in buttons {
        pad.digital_mut().press(*button);
    }
    app.world_mut()
        .resource_mut::<GamepadActionState>()
        .sample(&InputSettings::default(), Some((Entity::PLACEHOLDER, &pad)));
    app.update();
}

#[test]
fn gamepad_confirm_releases_represses_and_holds_without_repeated_click_edges() {
    let mut app = fixture(ClientState::CharacterCreate);
    let root = app.world_mut().spawn(Node::default()).id();
    let target = button(&mut app, root, 30.0, 30.0, 1);
    frame(&mut app, &[]);
    frame(&mut app, &[GamepadButton::South]);
    assert_eq!(app.world().get::<Presses>(target).unwrap().0, 1);
    for _ in 0..8 {
        frame(&mut app, &[GamepadButton::South]);
    }
    assert_eq!(
        *app.world().get::<Interaction>(target).unwrap(),
        Interaction::Pressed
    );
    assert_eq!(app.world().get::<Presses>(target).unwrap().0, 1);
    frame(&mut app, &[]);
    assert_eq!(
        *app.world().get::<Interaction>(target).unwrap(),
        Interaction::Hovered
    );
    frame(&mut app, &[GamepadButton::South]);
    assert_eq!(app.world().get::<Presses>(target).unwrap().0, 2);
    assert!(app.world().get::<PadUiHighlight>(target).is_some());
}

#[test]
fn gamepad_reconnect_resets_the_direction_repeat_delay() {
    let mut app = fixture(ClientState::CharacterCreate);
    let root = app.world_mut().spawn(Node::default()).id();
    let first = button(&mut app, root, 30.0, 30.0, 1);
    let second = button(&mut app, root, 30.0, 90.0, 2);
    frame(&mut app, &[GamepadButton::DPadDown]);
    assert_eq!(app.world().resource::<PadUiFocus>().entity, Some(second));
    app.world_mut().resource_mut::<GamepadActionState>().sample(&InputSettings::default(), None);
    app.update();
    assert_eq!(app.world().resource::<PadUiFocus>().entity, None);
    frame(&mut app, &[GamepadButton::DPadDown]);
    assert_ne!(app.world().resource::<PadUiFocus>().entity, Some(first));
    assert_eq!(app.world().resource::<PadUiFocus>().entity, Some(second));
}

#[test]
fn gamepad_repeat_is_opt_in_and_does_not_press_a_new_target_while_held() {
    let mut app = fixture(ClientState::CharacterCreate);
    let root = app.world_mut().spawn(Node::default()).id();
    let arrow = button(&mut app, root, 30.0, 30.0, 1);
    app.world_mut().entity_mut(arrow).insert(ControllerUiRepeat);
    let other = button(&mut app, root, 90.0, 30.0, 2);
    frame(&mut app, &[GamepadButton::South]);
    for _ in 0..8 {
        frame(&mut app, &[GamepadButton::South]);
    }
    assert!(app.world().get::<Presses>(arrow).unwrap().0 >= 3);
    frame(&mut app, &[GamepadButton::South, GamepadButton::DPadRight]);
    assert_eq!(app.world().resource::<PadUiFocus>().entity, Some(other));
    assert_eq!(app.world().get::<Presses>(other).unwrap().0, 0);
    assert!(app.world().get::<PadUiHighlight>(arrow).is_none());
    assert!(app.world().get::<PadUiHighlight>(other).is_some());
}

#[test]
fn gamepad_selection_unites_render_roots_but_keeps_modal_focus_isolated() {
    let mut app = fixture(ClientState::CharacterSelect);
    let base = app
        .world_mut()
        .spawn((Node::default(), ControllerUiScope::CharacterSelection))
        .id();
    let overlay = app
        .world_mut()
        .spawn((Node::default(), ControllerUiScope::CharacterSelection))
        .id();
    let slot = button(&mut app, base, 30.0, 30.0, 1);
    app.world_mut().entity_mut(slot).insert(ControllerUiDefault);
    let delete = button(&mut app, overlay, 30.0, 100.0, 5);
    frame(&mut app, &[]);
    assert_eq!(app.world().resource::<PadUiFocus>().entity, Some(slot));
    frame(&mut app, &[GamepadButton::DPadDown]);
    assert_eq!(app.world().resource::<PadUiFocus>().entity, Some(delete));
    let modal = app.world_mut().spawn(Node::default()).id();
    let cancel = button(&mut app, modal, 30.0, 200.0, 10);
    frame(&mut app, &[]);
    assert_eq!(app.world().resource::<PadUiFocus>().entity, Some(cancel));
}

#[test]
fn gamepad_tutorial_preserves_gameplay_and_dialog_prefers_first_option() {
    let mut app = fixture(ClientState::Tutorial);
    let root = app.world_mut().spawn(Node::default()).id();
    let chat = button(&mut app, root, 30.0, 30.0, 1);
    app.world_mut().entity_mut(chat).insert(ControllerUiIgnore);
    frame(&mut app, &[GamepadButton::South]);
    assert!(
        app.world()
            .resource::<GamepadActionState>()
            .just_pressed(LegacyOptionAction::Jump)
    );
    assert_eq!(app.world().resource::<PadUiFocus>().entity, None);
    app.world_mut()
        .resource_mut::<MissionUiModel>()
        .npc_icon_mode_visible = true;
    let close = button(&mut app, root, 120.0, 10.0, 2);
    app.world_mut().entity_mut(close).insert(ControllerUiClose);
    let option = button(&mut app, root, 30.0, 80.0, 3);
    app.world_mut()
        .entity_mut(option)
        .insert(ControllerUiDefault);
    frame(&mut app, &[]);
    assert_eq!(app.world().resource::<PadUiFocus>().entity, Some(option));
    frame(&mut app, &[GamepadButton::South]);
    assert!(
        !app.world()
            .resource::<GamepadActionState>()
            .held(LegacyOptionAction::Jump)
    );
    assert_eq!(app.world().get::<Presses>(option).unwrap().0, 1);
    assert_eq!(app.world().get::<Presses>(chat).unwrap().0, 0);
}

#[test]
fn gamepad_shoulders_activate_visible_tabs_and_cancel_emits_one_escape() {
    let mut app = fixture(ClientState::CharacterCreate);
    let root = app.world_mut().spawn(Node::default()).id();
    let left = button(&mut app, root, 30.0, 30.0, 1);
    let right = button(&mut app, root, 90.0, 30.0, 2);
    app.world_mut().entity_mut(left).insert(ControllerUiTab(0));
    app.world_mut().entity_mut(right).insert(ControllerUiTab(1));
    frame(&mut app, &[GamepadButton::RightTrigger]);
    assert_eq!(app.world().get::<Presses>(right).unwrap().0, 1);
    frame(&mut app, &[GamepadButton::RightTrigger]);
    assert_eq!(app.world().get::<Presses>(right).unwrap().0, 1);
    frame(&mut app, &[GamepadButton::LeftTrigger]);
    assert_eq!(app.world().get::<Presses>(left).unwrap().0, 1);
    frame(&mut app, &[GamepadButton::East]);
    assert!(
        app.world()
            .resource::<ButtonInput<KeyCode>>()
            .just_pressed(KeyCode::Escape)
    );
    frame(&mut app, &[GamepadButton::East]);
    assert!(
        !app.world()
            .resource::<ButtonInput<KeyCode>>()
            .pressed(KeyCode::Escape)
    );
}

#[test]
fn gamepad_cutscenes_receive_a_instead_of_hidden_hud_controls() {
    for state in [
        ClientState::CharacterCreateIntro,
        ClientState::TutorialIntro,
        ClientState::Tutorial,
    ] {
        let mut app = fixture(state);
        app.world_mut().resource_mut::<TutorialSession>().scene = TutorialScene::BasicMove;
        let root = app.world_mut().spawn(Node::default()).id();
        let hud = button(&mut app, root, 30.0, 30.0, 1);
        frame(&mut app, &[GamepadButton::South]);
        assert_eq!(app.world().resource::<PadUiFocus>().entity, None);
        assert!(
            app.world()
                .resource::<GamepadActionState>()
                .just_pressed(LegacyOptionAction::Jump)
        );
        assert_eq!(app.world().get::<Presses>(hud).unwrap().0, 0);
    }
}

#[test]
fn gamepad_child_popup_captures_focus_and_hides_parent_tabs_from_shoulders() {
    let mut app = fixture(ClientState::CharacterCreate);
    let root = app.world_mut().spawn(Node::default()).id();
    let tab = button(&mut app, root, 30.0, 30.0, 1);
    app.world_mut().entity_mut(tab).insert(ControllerUiTab(1));
    let popup = app
        .world_mut()
        .spawn((Node::default(), ControllerUiBoundary, ChildOf(root)))
        .id();
    let action = button(&mut app, popup, 30.0, 100.0, 2);
    frame(&mut app, &[GamepadButton::RightTrigger]);
    assert_eq!(app.world().resource::<PadUiFocus>().entity, Some(action));
    assert_eq!(app.world().get::<Presses>(tab).unwrap().0, 0);
}

fn launcher_fixture() -> App {
    use ffone_client::launcher_ui::{
        LauncherTriggerSpec, LauncherUiExternalState, LauncherUiOutbox,
    };
    let mut app = fixture(ClientState::World);
    let mut outbox = LauncherUiOutbox::default();
    app.world_mut()
        .resource_mut::<LauncherUiModel>()
        .open(
            LauncherTriggerSpec {
                trigger_position: Vec3::ZERO,
                trigger_euler_degrees: Vec3::ZERO,
                min_power: 10.0,
                max_power: 100.0,
                initial_rotation_degrees: Vec3::ZERO,
                maximum_rotation_degrees: Vec3::splat(45.0),
            },
            Vec3::ZERO,
            6.0,
            &mut outbox,
        )
        .unwrap();
    app.init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<OptionProductionRuntime>()
        .init_resource::<SystemMessageUiModel>()
        .init_resource::<LauncherUiExternalState>()
        .init_resource::<LauncherUiOutbox>()
        .init_resource::<RuntimeStatus>()
        .add_systems(
            Update,
            super::super::launcher::drive_launcher_production.after(navigate_gamepad_ui),
        );
    app.world_mut().resource_mut::<RuntimeStatus>().hp = Some(100);
    app
}

#[test]
fn gamepad_cannon_charges_while_a_is_held_and_fires_only_on_release() {
    use ffone_client::launcher_ui::{
        LauncherUiDismissalSource, LauncherUiEffect, LauncherUiOutbox,
    };
    let mut app = launcher_fixture();
    frame(&mut app, &[]);
    frame(&mut app, &[GamepadButton::South]);
    let initial = app.world().resource::<LauncherUiModel>().current_power;
    for _ in 0..3 {
        frame(&mut app, &[GamepadButton::South]);
    }
    let model = app.world().resource::<LauncherUiModel>();
    assert!(model.charging && model.current_power > initial);
    frame(&mut app, &[]);
    assert_eq!(
        app.world().resource::<LauncherUiModel>().dismissal,
        Some(LauncherUiDismissalSource::Fired)
    );
    let mut shots = 0;
    while let Some(effect) = app
        .world_mut()
        .resource_mut::<LauncherUiOutbox>()
        .pop_front()
    {
        if matches!(effect, LauncherUiEffect::StartLauncher(_)) {
            shots += 1;
        }
    }
    assert_eq!(shots, 1);
}

#[test]
fn gamepad_disconnect_cancels_cannon_charge_without_firing() {
    use ffone_client::launcher_ui::{LauncherUiEffect, LauncherUiOutbox};
    let mut app = launcher_fixture();
    frame(&mut app, &[GamepadButton::South]);
    assert!(app.world().resource::<LauncherUiModel>().charging);
    app.world_mut()
        .resource_mut::<GamepadActionState>()
        .sample(&InputSettings::default(), None);
    app.update();
    assert!(!app.world().resource::<LauncherUiModel>().visible());
    while let Some(effect) = app
        .world_mut()
        .resource_mut::<LauncherUiOutbox>()
        .pop_front()
    {
        assert!(!matches!(effect, LauncherUiEffect::StartLauncher(_)));
    }
}

#[test]
fn gamepad_same_frame_reconnect_cancels_cannon_charge_without_firing() {
    use bevy::input::{InputPlugin, InputSystems};
    use bevy::input::gamepad::{GamepadConnection, GamepadConnectionEvent};
    use ffone_client::launcher_ui::{LauncherUiEffect, LauncherUiOutbox};
    let mut app = launcher_fixture();
    app.add_plugins(InputPlugin)
        .add_systems(PreUpdate, super::super::gamepad::sample_gamepad_actions.after(InputSystems));
    let device = app.world_mut().spawn_empty().id();
    let connect = || GamepadConnectionEvent::new(device, GamepadConnection::Connected {
        name: "Cannon regression pad".into(), vendor_id: None, product_id: None,
    });
    app.world_mut().write_message(connect());
    app.update();
    app.world_mut().get_mut::<Gamepad>(device).unwrap().digital_mut().press(GamepadButton::South);
    app.update();
    assert!(app.world().resource::<LauncherUiModel>().charging);
    app.world_mut().write_message(GamepadConnectionEvent::new(device, GamepadConnection::Disconnected));
    app.world_mut().write_message(connect());
    app.update();
    assert!(!app.world().resource::<LauncherUiModel>().visible());
    while let Some(effect) = app.world_mut().resource_mut::<LauncherUiOutbox>().pop_front() {
        assert!(!matches!(effect, LauncherUiEffect::StartLauncher(_)));
    }
}
