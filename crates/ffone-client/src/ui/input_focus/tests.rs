use super::*;
use crate::avatar_action::{LegacyAvatarActionInput, read_legacy_avatar_action_input};
use bevy::input::{ButtonState, InputPlugin, mouse::MouseMotion};

fn fixture() -> (App, Entity, Entity) {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        GameInputFocusPlugin,
        GameplayPointerCapturePlugin,
    ))
    .init_resource::<LegacyAvatarActionInput>()
    .add_systems(Update, read_legacy_avatar_action_input);
    let window = app
        .world_mut()
        .spawn((
            Window {
                focused: true,
                ..default()
            },
            PrimaryWindow,
        ))
        .id();
    let ui = app
        .world_mut()
        .spawn((Interaction::None, ComputedNode::default()))
        .id();
    (app, window, ui)
}

fn mouse_press(app: &mut App, window: Entity) {
    app.world_mut().write_message(MouseButtonInput {
        window,
        button: MouseButton::Left,
        state: ButtonState::Pressed,
    });
}

#[test]
fn closing_menu_consumes_held_mouse_until_release() {
    let (mut app, window, ui) = fixture();
    app.world_mut().entity_mut(window).insert(CursorOptions::default());
    app.world_mut().get_mut::<ComputedNode>(ui).unwrap().size = Vec2::splat(32.0);
    *app.world_mut().get_mut::<Interaction>(ui).unwrap() = Interaction::Pressed;
    mouse_press(&mut app, window);
    app.update();
    app.world_mut().entity_mut(ui).despawn();
    app.world_mut().entity_mut(window).insert(CursorOptions {
        visible: false, grab_mode: CursorGrabMode::Locked, ..default()
    });
    for _ in 0..3 {
        app.update();
        assert!(!app.world().resource::<LegacyAvatarActionInput>().primary_held);
    }
    app.world_mut().write_message(MouseButtonInput {
        window, button: MouseButton::Left, state: ButtonState::Released,
    });
    app.update();
    mouse_press(&mut app, window);
    app.update();
    assert!(app.world().resource::<LegacyAvatarActionInput>().primary_just_pressed);
}

#[test]
fn visible_menu_keeps_position_and_click_ownership() {
    let (mut app, window, ui) = fixture();
    app.world_mut()
        .entity_mut(window)
        .insert(CursorOptions::default());
    let position = Vec2::new(100.0, 75.0);
    app.world_mut()
        .get_mut::<Window>(window)
        .unwrap()
        .set_cursor_position(Some(position));
    app.world_mut().get_mut::<ComputedNode>(ui).unwrap().size = Vec2::splat(32.0);
    *app.world_mut().get_mut::<Interaction>(ui).unwrap() = Interaction::Pressed;
    app.add_systems(
        PreUpdate,
        (|mut windows: Query<(&mut Window, &mut CursorOptions)>| {
            for (mut window, mut cursor) in &mut windows {
                apply_gameplay_cursor(&mut window, &mut cursor, false, true);
            }
        })
        .before(UiSystems::Focus),
    );
    mouse_press(&mut app, window);
    app.update();
    assert_eq!(
        app.world().get::<Window>(window).unwrap().cursor_position(),
        Some(position)
    );
    assert_eq!(
        *app.world().get::<Interaction>(ui).unwrap(),
        Interaction::Pressed
    );
    assert!(app.world().resource::<GameplayPointerCapture>().blocked);
    assert!(
        !app.world()
            .resource::<LegacyAvatarActionInput>()
            .primary_just_pressed
    );
}

#[test]
#[cfg(windows)]
fn exit_and_window_close_release_cursor_after_gameplay_sync() {
    for app_exit in [false, true] {
        let (mut app, window, _) = fixture();
        app.world_mut()
            .entity_mut(window)
            .insert(CursorOptions::default());
        app.add_systems(
            PostUpdate,
            (|mut cursors: Query<&mut CursorOptions>| {
                for mut cursor in &mut cursors {
                    cursor.visible = false;
                    cursor.grab_mode = CursorGrabMode::Locked;
                }
            })
            .in_set(GameplayCursorSyncSet),
        );
        if app_exit {
            app.world_mut().write_message(AppExit::Success);
        } else {
            app.world_mut()
                .write_message(bevy::window::WindowCloseRequested { window });
        }
        app.update();
        let cursor = app.world().get::<CursorOptions>(window).unwrap();
        assert!(cursor.visible);
        assert_eq!(cursor.grab_mode, CursorGrabMode::None);
    }
}

#[test]
fn regain_restores_pointer_before_the_first_gameplay_click_is_captured() {
    let (mut app, window, ui) = fixture();
    app.world_mut()
        .entity_mut(window)
        .insert(CursorOptions::default());
    app.world_mut().get_mut::<ComputedNode>(ui).unwrap().size = Vec2::splat(32.0);
    // Represent a HUD node under the last absolute cursor position.
    app.add_systems(
        PreUpdate,
        (|mut interactions: Query<&mut Interaction>| {
            for mut interaction in &mut interactions {
                *interaction = Interaction::Hovered;
            }
        })
        .in_set(UiSystems::Focus),
    );
    app.add_systems(
        PreUpdate,
        (|mut cursors: Query<(&mut Window, &mut CursorOptions)>| {
            for (mut window, mut cursor) in &mut cursors {
                apply_gameplay_cursor(&mut window, &mut cursor, true, true);
            }
        })
        .after(GameInputFocusSet)
        .before(UiSystems::Focus)
        .run_if(game_input_focus_regained),
    );
    app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
    app.world_mut()
        .get_mut::<Window>(window)
        .unwrap()
        .set_cursor_position(Some(Vec2::new(10.0, 20.0)));
    app.update();
    assert!(!app.world().resource::<GameInputFocus>().regained);

    app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
    mouse_press(&mut app, window);
    app.update();
    assert!(app.world().resource::<GameInputFocus>().regained);
    assert!(!app.world().resource::<GameplayPointerCapture>().blocked);
    let restored = app.world().get::<Window>(window).unwrap();
    assert_eq!(restored.cursor_position(), Some(restored.size() * 0.5));
    assert_eq!(
        *app.world().get::<Interaction>(ui).unwrap(),
        Interaction::None
    );
    assert!(
        app.world()
            .resource::<LegacyAvatarActionInput>()
            .primary_just_pressed
    );
    app.update();
    assert!(!app.world().resource::<GameInputFocus>().regained);
}

#[test]
fn locked_cursor_hud_hover_does_not_consume_mouse_actions() {
    let (mut app, window, ui) = fixture();
    app.world_mut().entity_mut(window).insert(CursorOptions {
        visible: false,
        grab_mode: CursorGrabMode::Locked,
        ..default()
    });
    app.world_mut().get_mut::<ComputedNode>(ui).unwrap().size = Vec2::splat(32.0);
    *app.world_mut().get_mut::<Interaction>(ui).unwrap() = Interaction::Hovered;
    mouse_press(&mut app, window);
    app.update();
    let input = app.world().resource::<LegacyAvatarActionInput>();
    assert!(input.primary_held && input.primary_just_pressed);
    assert_eq!(
        *app.world().get::<Interaction>(ui).unwrap(),
        Interaction::None
    );
    *app.world_mut().get_mut::<Interaction>(ui).unwrap() = Interaction::Pressed;
    app.update();
    let input = app.world().resource::<LegacyAvatarActionInput>();
    assert!(input.primary_held && !input.primary_just_pressed);
    assert_eq!(
        *app.world().get::<Interaction>(ui).unwrap(),
        Interaction::None
    );
    app.world_mut().write_message(MouseButtonInput {
        window,
        button: MouseButton::Left,
        state: ButtonState::Released,
    });
    app.update();
    assert!(
        !app.world()
            .resource::<LegacyAvatarActionInput>()
            .primary_held
    );
    mouse_press(&mut app, window);
    app.update();
    assert!(
        app.world()
            .resource::<LegacyAvatarActionInput>()
            .primary_just_pressed
    );
}

#[test]
fn focus_loss_cancels_stuck_mouse_and_ui_and_next_click_reaches_gameplay() {
    let (mut app, window, ui) = fixture();
    mouse_press(&mut app, window);
    app.update();
    assert!(
        app.world()
            .resource::<LegacyAvatarActionInput>()
            .primary_just_pressed
    );
    *app.world_mut().get_mut::<Interaction>(ui).unwrap() = Interaction::Pressed;
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::AltLeft);
    app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
    app.world_mut().write_message(WindowFocused {
        window,
        focused: false,
    });
    app.world_mut().write_message(MouseMotion {
        delta: Vec2::splat(100.0),
    });
    app.world_mut().write_message(MouseWheel {
        phase: bevy::input::touch::TouchPhase::Moved,
        window,
        unit: bevy::input::mouse::MouseScrollUnit::Line,
        x: 0.0,
        y: 4.0,
    });
    app.update();
    assert!(
        !app.world()
            .resource::<LegacyAvatarActionInput>()
            .primary_held
    );
    assert!(
        !app.world()
            .resource::<ButtonInput<MouseButton>>()
            .just_released(MouseButton::Left)
    );
    assert_eq!(
        *app.world().get::<Interaction>(ui).unwrap(),
        Interaction::None
    );
    assert_eq!(
        app.world().resource::<AccumulatedMouseMotion>().delta,
        Vec2::ZERO
    );
    assert_eq!(
        app.world().resource::<AccumulatedMouseScroll>().delta,
        Vec2::ZERO
    );
    assert!(
        !app.world()
            .resource::<ButtonInput<KeyCode>>()
            .pressed(KeyCode::AltLeft)
    );

    // There was no release packet. Returning to the game must not require
    // an extra click to clear the old held state.
    app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
    app.world_mut().write_message(WindowFocused {
        window,
        focused: true,
    });
    mouse_press(&mut app, window);
    app.update();
    assert!(!app.world().resource::<GameInputFocus>().suppressed);
    assert!(app.world().resource::<GameInputFocus>().regained);
    assert!(
        app.world()
            .resource::<LegacyAvatarActionInput>()
            .primary_just_pressed
    );
}

#[test]
fn focus_bounce_is_cancelled_once_and_other_windows_do_not_cancel_game_input() {
    let (mut app, window, _) = fixture();
    let other = app.world_mut().spawn(Window::default()).id();
    app.world_mut().write_message(WindowFocused {
        window: other,
        focused: false,
    });
    mouse_press(&mut app, window);
    app.update();
    assert!(
        app.world()
            .resource::<LegacyAvatarActionInput>()
            .primary_just_pressed
    );
    app.world_mut().write_message(WindowFocused {
        window,
        focused: false,
    });
    app.world_mut().write_message(WindowFocused {
        window,
        focused: true,
    });
    app.update();
    assert!(app.world().resource::<GameInputFocus>().suppressed);
    assert!(!app.world().resource::<GameInputFocus>().regained);
    assert!(
        !app.world()
            .resource::<LegacyAvatarActionInput>()
            .primary_held
    );
    app.update();
    assert!(!app.world().resource::<GameInputFocus>().suppressed);
    assert!(app.world().resource::<GameInputFocus>().regained);
    mouse_press(&mut app, window);
    app.update();
    assert!(
        app.world()
            .resource::<LegacyAvatarActionInput>()
            .primary_just_pressed
    );
}
