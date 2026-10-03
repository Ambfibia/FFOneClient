//! Translate Bevy's semantic gamepad controls into the saved legacy pad column.
use bevy::input::gamepad::{Gamepad, GamepadAxis, GamepadButton};
use bevy::prelude::*;
use ffone_client::option_ui::{
    InputSettings, LegacyAxisDirection, LegacyInputBinding, LegacyOptionAction, LegacyPadProfile,
};
use std::collections::HashMap;

const PAD_DEAD_ZONE: f32 = 0.22;

#[derive(Clone, Copy, Default)]
struct PadActionValue {
    value: f32,
    just_pressed: bool,
    just_released: bool,
}

/// One device owns the frame. Removal drops all values before gameplay readers run.
#[derive(Resource, Default)]
pub(super) struct GamepadActionState {
    active: Option<Entity>,
    actions: HashMap<LegacyOptionAction, PadActionValue>,
    ui_consumed: bool,
    ui_stick: Vec2,
    pub(super) buttons: ButtonInput<GamepadButton>,
}

impl GamepadActionState {
    pub(super) fn just_released(&self, action: LegacyOptionAction) -> bool {
        !self.ui_consumed && self.actions.get(&action).is_some_and(|state| state.just_released)
    }
    pub(super) fn value(&self, action: LegacyOptionAction) -> f32 {
        if self.ui_consumed {
            return 0.0;
        }
        self.actions.get(&action).map_or(0.0, |state| state.value)
    }

    pub(super) fn held(&self, action: LegacyOptionAction) -> bool {
        self.value(action) > 0.0
    }

    pub(super) fn just_pressed(&self, action: LegacyOptionAction) -> bool {
        !self.ui_consumed
            && self
                .actions
                .get(&action)
                .is_some_and(|state| state.just_pressed)
    }

    pub(super) fn connected(&self) -> bool {
        self.active.is_some()
    }

    pub(super) fn consume_ui_actions(&mut self) {
        self.ui_consumed = true;
    }

    pub(super) fn ui_stick_direction(&self) -> Vec2 {
        self.ui_stick
    }

    pub(super) fn sample(&mut self, settings: &InputSettings, gamepad: Option<(Entity, &Gamepad)>) {
        self.ui_consumed = false;
        let previous = std::mem::take(&mut self.actions);
        self.buttons = gamepad.map_or_else(ButtonInput::default, |(_, pad)| pad.digital().clone());
        self.ui_stick = gamepad.map_or(Vec2::ZERO, |(_, pad)| {
            let x = pad.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
            let y = pad.get(GamepadAxis::LeftStickY).unwrap_or(0.0);
            Vec2::new(
                if x.abs() >= 0.55 { x.signum() } else { 0.0 },
                if y.abs() >= 0.55 { -y.signum() } else { 0.0 },
            )
        });
        let previous_device = self.active;
        self.active = gamepad.map(|(entity, _)| entity);
        let Some((entity, gamepad)) = gamepad else {
            return;
        };
        for row in &settings.mappings {
            let value = pad_binding_value(row.pad, settings.pad_profile, gamepad);
            let was_held = previous_device == Some(entity)
                && previous
                    .get(&row.action)
                    .is_some_and(|state| state.value > 0.0);
            self.actions.insert(
                row.action,
                PadActionValue {
                    value,
                    just_pressed: value > 0.0 && !was_held,
                    just_released: value == 0.0 && was_held,
                },
            );
        }
    }
}

pub(super) fn sample_gamepad_actions(
    options: Res<super::OptionProductionRuntime>,
    gamepads: Query<(Entity, &Gamepad)>,
    mut state: ResMut<GamepadActionState>,
    mut players: Query<&mut ffone_client::movement::LegacyPlayerController, With<super::LocalPlayer>>,
) {
    let active = state
        .active
        .and_then(|entity| gamepads.get(entity).ok())
        .or_else(|| gamepads.iter().min_by_key(|(entity, _)| entity.index()));
    let removed = state.active.is_some() && state.active != active.map(|(entity, _)| entity);
    state.sample(&options.input, active);
    if removed {
        for mut player in &mut players {
            player.set_auto_run(false);
        }
    }
}

pub(super) fn pad_binding_value(
    binding: LegacyInputBinding,
    profile: LegacyPadProfile,
    pad: &Gamepad,
) -> f32 {
    match binding {
        LegacyInputBinding::PadButton(index) => {
            pad_button(index, profile).is_some_and(|button| pad.pressed(button)) as u8 as f32
        }
        LegacyInputBinding::PadAxis { axis, direction } => {
            let Some(value) = pad_axis_value(axis, profile, pad) else {
                return 0.0;
            };
            let directed = match direction {
                LegacyAxisDirection::Positive => value,
                LegacyAxisDirection::Negative => -value,
            };
            ((directed - PAD_DEAD_ZONE) / (1.0 - PAD_DEAD_ZONE)).clamp(0.0, 1.0)
        }
        _ => 0.0,
    }
}

// Clean joystick axes use downward-positive Y; Bevy uses upward-positive Y.
fn pad_axis_value(index: u8, profile: LegacyPadProfile, pad: &Gamepad) -> Option<f32> {
    let axis = match (profile, index) {
        (_, 3) => GamepadAxis::LeftStickX,
        (_, 4) => GamepadAxis::LeftStickY,
        (LegacyPadProfile::Xbox360 | LegacyPadProfile::PlayStation2, 6)
        | (LegacyPadProfile::RumblePad2, 5) => GamepadAxis::RightStickX,
        (LegacyPadProfile::Xbox360, 7)
        | (LegacyPadProfile::RumblePad2, 6)
        | (LegacyPadProfile::PlayStation2, 5) => GamepadAxis::RightStickY,
        (LegacyPadProfile::Xbox360, 8) | (_, 7) => {
            return Some(if pad.pressed(GamepadButton::RightThumb) {
                1.0
            } else {
                0.0
            });
        }
        (LegacyPadProfile::Xbox360, 9) | (_, 8) => {
            return Some(
                pad.get(GamepadButton::RightTrigger2).unwrap_or(0.0)
                    - pad.get(GamepadButton::LeftTrigger2).unwrap_or(0.0),
            );
        }
        _ => return None,
    };
    Some(
        pad.get(axis).unwrap_or(0.0)
            * if matches!(axis, GamepadAxis::LeftStickY | GamepadAxis::RightStickY) {
                -1.0
            } else {
                1.0
            },
    )
}

fn pad_button(index: u8, profile: LegacyPadProfile) -> Option<GamepadButton> {
    let extended = match index {
        10 => Some(GamepadButton::LeftThumb),
        11 => Some(GamepadButton::RightThumb),
        12 => Some(GamepadButton::DPadUp),
        13 => Some(GamepadButton::DPadDown),
        14 => Some(GamepadButton::DPadLeft),
        15 => Some(GamepadButton::DPadRight),
        16 => Some(GamepadButton::LeftTrigger2),
        17 => Some(GamepadButton::RightTrigger2),
        _ => None,
    };
    if extended.is_some() {
        return extended;
    }
    if profile == LegacyPadProfile::Xbox360 {
        return Some(match index {
            0 => GamepadButton::South,
            1 => GamepadButton::East,
            2 => GamepadButton::West,
            3 => GamepadButton::North,
            4 => GamepadButton::LeftTrigger,
            5 => GamepadButton::RightTrigger,
            6 => GamepadButton::Select,
            7 => GamepadButton::Start,
            _ => return None,
        });
    }
    Some(match index {
        0 => GamepadButton::West,
        1 => GamepadButton::South,
        2 => GamepadButton::East,
        3 => GamepadButton::North,
        4 => GamepadButton::LeftTrigger,
        5 => GamepadButton::RightTrigger,
        6 => GamepadButton::LeftTrigger2,
        7 => GamepadButton::RightTrigger2,
        8 => GamepadButton::Select,
        9 => GamepadButton::Start,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::OptionProductionRuntime;

    #[test]
    fn profiles_translate_semantic_face_buttons_and_camera_axes() {
        let mut pad = Gamepad::default();
        pad.digital_mut().press(GamepadButton::South);
        pad.analog_mut().set(GamepadAxis::RightStickY, 0.8);
        assert_eq!(
            pad_binding_value(
                LegacyInputBinding::PadButton(0),
                LegacyPadProfile::Xbox360,
                &pad
            ),
            1.0
        );
        assert_eq!(
            pad_binding_value(
                LegacyInputBinding::PadButton(1),
                LegacyPadProfile::PlayStation2,
                &pad
            ),
            1.0
        );
        assert!(
            pad_binding_value(
                LegacyInputBinding::PadAxis {
                    axis: 6,
                    direction: LegacyAxisDirection::Negative
                },
                LegacyPadProfile::RumblePad2,
                &pad
            ) > 0.7
        );
        assert!(
            pad_binding_value(
                LegacyInputBinding::PadAxis {
                    axis: 5,
                    direction: LegacyAxisDirection::Negative
                },
                LegacyPadProfile::PlayStation2,
                &pad
            ) > 0.7
        );
    }

    #[test]
    fn pad_dead_zone_button_edge_and_disconnect_release() {
        let mut app = App::new();
        app.init_resource::<OptionProductionRuntime>()
            .init_resource::<GamepadActionState>()
            .add_systems(Update, sample_gamepad_actions);
        let entity = app.world_mut().spawn(Gamepad::default()).id();
        {
            let mut pad = app.world_mut().get_mut::<Gamepad>(entity).unwrap();
            pad.analog_mut().set(GamepadAxis::LeftStickX, 0.12);
            pad.digital_mut().press(GamepadButton::South);
        }
        app.update();
        let state = app.world().resource::<GamepadActionState>();
        assert_eq!(state.value(LegacyOptionAction::Right), 0.0);
        assert!(state.just_pressed(LegacyOptionAction::Jump));
        app.update();
        assert!(
            !app.world()
                .resource::<GamepadActionState>()
                .just_pressed(LegacyOptionAction::Jump)
        );
        app.world_mut()
            .get_mut::<Gamepad>(entity)
            .unwrap()
            .analog_mut()
            .set(GamepadAxis::LeftStickX, 0.7);
        app.update();
        assert!(
            app.world()
                .resource::<GamepadActionState>()
                .value(LegacyOptionAction::Right)
                > 0.5
        );
        app.world_mut().despawn(entity);
        app.update();
        let state = app.world().resource::<GamepadActionState>();
        assert_eq!(state.value(LegacyOptionAction::Right), 0.0);
        assert!(!state.held(LegacyOptionAction::Jump));
        assert!(!state.just_pressed(LegacyOptionAction::Jump));
    }
}

#[cfg(test)]
mod hotplug_regressions {
    use super::*;
    use crate::app::OptionProductionRuntime;
    use bevy::input::gamepad::{GamepadConnection, GamepadConnectionEvent};
    use bevy::input::{InputPlugin, InputSystems};

    #[test]
    fn connection_messages_release_and_reconnect_same_device_repeatedly() {
        let mut app = App::new();
        app.add_plugins(InputPlugin)
            .init_resource::<OptionProductionRuntime>()
            .init_resource::<GamepadActionState>()
            .add_systems(PreUpdate, sample_gamepad_actions.after(InputSystems));
        let device = app.world_mut().spawn_empty().id();
        let player = app.world_mut().spawn((super::super::LocalPlayer,
            ffone_client::movement::LegacyPlayerController::from_baseline_table())).id();
        for _ in 0..5 {
            app.world_mut().write_message(GamepadConnectionEvent::new(
                device,
                GamepadConnection::Connected {
                    name: "Regression pad".into(),
                    vendor_id: None,
                    product_id: None,
                },
            ));
            app.update();
            app.world_mut()
                .get_mut::<Gamepad>(device)
                .unwrap()
                .digital_mut()
                .press(GamepadButton::West);
            app.update();
            assert!(
                app.world()
                    .resource::<GamepadActionState>()
                    .just_pressed(LegacyOptionAction::Nano1)
            );
            app.update();
            assert!(
                !app.world()
                    .resource::<GamepadActionState>()
                    .just_pressed(LegacyOptionAction::Nano1)
            );
            app.world_mut().write_message(GamepadConnectionEvent::new(
                device,
                GamepadConnection::Disconnected,
            ));
            app.world_mut().get_mut::<ffone_client::movement::LegacyPlayerController>(player)
                .unwrap().set_auto_run(true);
            app.update();
            assert!(!app.world().get::<ffone_client::movement::LegacyPlayerController>(player)
                .unwrap().is_auto_running());
            assert!(!app.world().resource::<GamepadActionState>().connected());
            assert!(
                !app.world()
                    .resource::<GamepadActionState>()
                    .held(LegacyOptionAction::Nano1)
            );
            assert!(app.world().get_entity(device).is_ok());
        }
    }

    #[test]
    fn semantic_layout_supports_both_triggers_and_all_requested_controls() {
        let settings = InputSettings::default();
        let mut pad = Gamepad::default();
        let mut state = GamepadActionState::default();
        let entity = Entity::from_bits(1);
        for (button, action) in [
            (GamepadButton::RightTrigger2, LegacyOptionAction::Fire1),
            (GamepadButton::LeftTrigger2, LegacyOptionAction::Fire2),
            (GamepadButton::DPadUp, LegacyOptionAction::ZoomIn),
            (GamepadButton::DPadDown, LegacyOptionAction::ZoomOut),
            (GamepadButton::DPadLeft, LegacyOptionAction::Journal),
            (GamepadButton::DPadRight, LegacyOptionAction::WeaponChange),
            (GamepadButton::LeftTrigger, LegacyOptionAction::FreeCamera),
            (GamepadButton::RightTrigger, LegacyOptionAction::NanoCharge),
            (GamepadButton::West, LegacyOptionAction::Nano1),
            (GamepadButton::North, LegacyOptionAction::Nano2),
            (GamepadButton::East, LegacyOptionAction::Nano3),
            (GamepadButton::South, LegacyOptionAction::Jump),
            (GamepadButton::LeftThumb, LegacyOptionAction::VehicleToggle),
            (GamepadButton::RightThumb, LegacyOptionAction::AutoRun),
            (GamepadButton::Start, LegacyOptionAction::Menu),
            (GamepadButton::Select, LegacyOptionAction::Inventory),
        ] {
            pad.digital_mut().press(button);
            state.sample(&settings, Some((entity, &pad)));
            assert!(
                state.just_pressed(action),
                "{button:?} must trigger {action:?} in this frame"
            );
            state.sample(&settings, Some((entity, &pad)));
            assert!(!state.just_pressed(action));
            pad.digital_mut().release(button);
            state.sample(&settings, Some((entity, &pad)));
        }
        pad.digital_mut().press(GamepadButton::LeftTrigger2);
        pad.digital_mut().press(GamepadButton::RightTrigger2);
        state.sample(&settings, Some((entity, &pad)));
        assert!(state.held(LegacyOptionAction::Fire1));
        assert!(state.held(LegacyOptionAction::Fire2));
    }
}

#[cfg(test)]
mod ui_ownership_regressions {
    use super::*;
    #[test]
    fn ui_consumption_does_not_repeat_held_menu_or_leak_nano_when_closed() {
        let settings = InputSettings::default();
        let mut pad = Gamepad::default();
        pad.digital_mut().press(GamepadButton::Start);
        pad.digital_mut().press(GamepadButton::East);
        let entity = Entity::from_bits(1);
        let mut state = GamepadActionState::default();
        state.sample(&settings, Some((entity, &pad)));
        assert!(state.just_pressed(LegacyOptionAction::Menu));
        state.consume_ui_actions();
        assert!(!state.just_pressed(LegacyOptionAction::Nano3));
        assert!(!state.held(LegacyOptionAction::Menu));
        state.sample(&settings, Some((entity, &pad)));
        assert!(!state.just_pressed(LegacyOptionAction::Menu));
        assert!(!state.just_pressed(LegacyOptionAction::Nano3));
    }
}
