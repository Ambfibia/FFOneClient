//! Cancel transient desktop input when the primary game window loses focus.
use bevy::{
    input::{
        InputSystems,
        keyboard::{Key, KeyboardInput},
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseButtonInput, MouseWheel},
    },
    prelude::*,
    ui::UiSystems,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow, WindowFocused},
};

/// Snapshot pointer ownership before Update can close a dialog or lock the cursor.
#[derive(Default, Resource)]
pub struct GameplayPointerCapture {
    pub blocked: bool,
    held_by_ui: bool,
}

pub struct GameplayPointerCapturePlugin;

impl Plugin for GameplayPointerCapturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameplayPointerCapture>()
            .add_systems(PreUpdate, capture_gameplay_pointer.after(UiSystems::Focus));
    }
}

fn capture_gameplay_pointer(
    cursors: Query<&CursorOptions, With<PrimaryWindow>>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    mut ui: Query<(&mut Interaction, &ComputedNode)>,
    mut capture: ResMut<GameplayPointerCapture>,
) {
    // Bevy hit testing uses the last absolute cursor position even when the
    // game hides/locks it for relative camera input. Such HUD hover is not UI
    // ownership. Visible menu clicks still remain consumed for the whole frame.
    let gameplay_cursor = cursors
        .single()
        .is_ok_and(|cursor| !cursor.visible && cursor.grab_mode == CursorGrabMode::Locked);
    let over_ui = !gameplay_cursor
        && ui.iter().any(|(interaction, node)| {
            *interaction != Interaction::None && node.size().min_element() > 0.0
        });
    // A close button can disappear on press. Keep that gesture owned by UI
    // through release instead of turning the still-held button into gunfire.
    let held = mouse.as_ref().is_some_and(|mouse| {
        mouse.pressed(MouseButton::Left) || mouse.pressed(MouseButton::Right)
    });
    capture.blocked = over_ui || capture.held_by_ui;
    capture.held_by_ui = held && capture.blocked;
    if gameplay_cursor {
        // UI button handlers must not receive clicks at the hidden pointer's
        // last absolute position, even when that position is at screen center.
        for (mut interaction, _) in &mut ui {
            interaction.set_if_neq(Interaction::None);
        }
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameplayCursorSyncSet;

/// Apply pointer ownership and discard the old HUD position on capture/return.
pub fn apply_gameplay_cursor(
    window: &mut Mut<Window>,
    cursor: &mut Mut<CursorOptions>,
    locked: bool,
    regained: bool,
) {
    if locked && (regained || cursor.grab_mode != CursorGrabMode::Locked || cursor.visible) {
        let center = window.size() * 0.5;
        window.set_cursor_position(Some(center));
    }
    if regained {
        cursor.set_changed();
    }
    let mode = if locked {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
    if cursor.grab_mode != mode {
        cursor.grab_mode = mode;
    }
    if cursor.visible == locked {
        cursor.visible = !locked;
    }
}

/// Release the native cursor while the OS window still exists. A close request
/// can already have despawned its ECS entity by PostUpdate, so CursorOptions
/// alone cannot perform this cleanup.
#[cfg(windows)]
fn release_cursor_on_exit(
    mut exits: MessageReader<AppExit>,
    mut closes: MessageReader<bevy::window::WindowCloseRequested>,
    mut cursors: Query<(Entity, &mut CursorOptions)>,
    _main_thread: bevy::ecs::system::NonSendMarker,
) {
    let exiting = exits.read().next().is_some();
    exits.clear();
    let closing: Vec<_> = closes.read().map(|event| event.window).collect();
    if !exiting && closing.is_empty() {
        return;
    }
    for (entity, mut cursor) in &mut cursors {
        if exiting || closing.contains(&entity) {
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
        }
    }
    bevy_winit::WINIT_WINDOWS.with_borrow(|windows| {
        for (entity, id) in &windows.entity_to_winit {
            if !exiting && !closing.contains(entity) {
                continue;
            }
            if let Some(window) = windows.windows.get(id) {
                if let Err(error) = window.set_cursor_grab(winit::window::CursorGrabMode::None) {
                    warn!("Could not release cursor on exit: {error}");
                }
                window.set_cursor_visible(true);
            }
        }
    });
}

#[derive(Default, Resource)]
pub struct GameInputFocus {
    /// Also covers loss and regain delivered together before a single frame.
    pub suppressed: bool,
    /// Restore native pointer ownership before UI hit testing on this frame.
    pub regained: bool,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameInputFocusSet;

pub fn game_input_focus_regained(focus: Res<GameInputFocus>) -> bool {
    focus.regained
}

pub struct GameInputFocusPlugin;

impl Plugin for GameInputFocusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameInputFocus>()
            .add_message::<WindowFocused>()
            .add_systems(
                PreUpdate,
                cancel_unfocused_input
                    .in_set(GameInputFocusSet)
                    .after(InputSystems)
                    .before(UiSystems::Focus),
            );
        #[cfg(windows)]
        app.add_message::<AppExit>()
            .add_message::<bevy::window::WindowCloseRequested>()
            .add_systems(
                PostUpdate,
                release_cursor_on_exit.after(GameplayCursorSyncSet),
            );
    }
}

fn cancel_unfocused_input(
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    mut events: MessageReader<WindowFocused>,
    mut focus: ResMut<GameInputFocus>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut logical_keys: ResMut<ButtonInput<Key>>,
    mut motion: ResMut<AccumulatedMouseMotion>,
    mut scroll: ResMut<AccumulatedMouseScroll>,
    mut key_events: ResMut<Messages<KeyboardInput>>,
    mut mouse_events: ResMut<Messages<MouseButtonInput>>,
    mut wheel_events: ResMut<Messages<MouseWheel>>,
    mut interactions: Query<&mut Interaction>,
) {
    let Ok((primary, window)) = windows.single() else {
        events.clear();
        focus.suppressed = false;
        focus.regained = false;
        return;
    };
    let mut lost = false;
    let mut gained = false;
    for event in events.read().filter(|event| event.window == primary) {
        lost |= !event.focused;
        gained |= event.focused;
    }
    let was_suppressed = focus.suppressed;
    focus.suppressed = !window.focused || lost;
    focus.regained = !focus.suppressed && (was_suppressed || gained);
    if !focus.suppressed {
        return;
    }
    // MouseButtonInput has no focus-loss reset in Bevy. A release outside the
    // window can be absent forever. Cancel instead of synthesizing a release,
    // which could commit a drag/drop or release-triggered UI action.
    mouse.reset_all();
    keys.reset_all();
    logical_keys.reset_all();
    motion.delta = Vec2::ZERO;
    scroll.delta = Vec2::ZERO;
    key_events.clear();
    mouse_events.clear();
    wheel_events.clear();
    for mut interaction in &mut interactions {
        interaction.set_if_neq(Interaction::None);
    }
}

#[cfg(test)]
mod tests;
