use super::*;

pub const OPTION_DROPDOWN_BUTTON_Z_INDEX: i32 = 50;

/// ...and then repaints the open dropdown's own `pulldown` box over the list.
pub const OPTION_DROPDOWN_OPEN_BUTTON_Z_INDEX: i32 = 70;

pub const OPTION_INPUT_ACTION_COUNT: usize = 39;

pub const OPTION_JEFFE_BUTTON_FONT_SIZE: f32 = 11.3;

pub const OPTION_CONTROL_SCROLL_MAX: f32 =
    OPTION_CONTROL_CONTENT_HEIGHT - OPTION_KEYMAP_CONTENT_RECT.height;

pub const OPTION_SCROLL_THUMB_OVERFLOW: f32 = 2.0;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum OptionInputMappingSlot {
    #[default]
    Primary,
    Alternate,
    Pad,
}

impl OptionInputMappingSlot {
    pub const ALL: [Self; 3] = [Self::Primary, Self::Alternate, Self::Pad];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Primary => "SETTING 1",
            Self::Alternate => "SETTING 2",
            Self::Pad => "SETTING 3",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum LegacyInputAxis {
    MouseWheel,
    MouseX,
    MouseY,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum LegacyInputBinding {
    #[default]
    Unbound,
    Key(LegacyPhysicalKey),
    Axis {
        axis: LegacyInputAxis,
        direction: LegacyAxisDirection,
    },
    PadButton(u8),
    PadAxis {
        axis: u8,
        direction: LegacyAxisDirection,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LegacyInputBindingSet {
    pub action: LegacyOptionAction,
    pub primary: LegacyInputBinding,
    pub alternate: LegacyInputBinding,
    pub pad: LegacyInputBinding,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct InputSettings {
    pub input_version: u8,
    pub camera_sensitivity: f32,
    pub invert_y: bool,
    #[serde(default)]
    pub pad_invert_y: bool,
    #[serde(default = "default_pad_camera_sensitivity")]
    pub pad_camera_sensitivity: f32,
    pub pad_profile: LegacyPadProfile,
    pub mappings: Vec<LegacyInputBindingSet>,
}

pub fn default_pad_camera_sensitivity() -> f32 { 5.0 }

impl Default for InputSettings {
    fn default() -> Self {
        let mut mappings = LegacyOptionAction::ALL
            .into_iter()
            .map(|action| LegacyInputBindingSet {
                action,
                primary: LegacyInputBinding::Unbound,
                alternate: LegacyInputBinding::Unbound,
                pad: LegacyInputBinding::Unbound,
            })
            .collect::<Vec<_>>();
        let key = LegacyInputBinding::Key;
        let set = |mappings: &mut [LegacyInputBindingSet],
                   action: LegacyOptionAction,
                   primary: LegacyInputBinding,
                   alternate: LegacyInputBinding| {
            let row = mappings
                .iter_mut()
                .find(|row| row.action == action)
                .expect("every clean option action has a binding row");
            row.primary = primary;
            row.alternate = alternate;
        };
        set(
            &mut mappings,
            LegacyOptionAction::Up,
            key(LegacyPhysicalKey::W),
            key(LegacyPhysicalKey::ArrowUp),
        );
        set(
            &mut mappings,
            LegacyOptionAction::Down,
            key(LegacyPhysicalKey::S),
            key(LegacyPhysicalKey::ArrowDown),
        );
        set(
            &mut mappings,
            LegacyOptionAction::Left,
            key(LegacyPhysicalKey::A),
            LegacyInputBinding::Unbound,
        );
        set(
            &mut mappings,
            LegacyOptionAction::Right,
            key(LegacyPhysicalKey::D),
            LegacyInputBinding::Unbound,
        );
        set(
            &mut mappings,
            LegacyOptionAction::Escape,
            key(LegacyPhysicalKey::BackQuote),
            LegacyInputBinding::Unbound,
        );
        set(
            &mut mappings,
            LegacyOptionAction::Jump,
            key(LegacyPhysicalKey::Space),
            LegacyInputBinding::Unbound,
        );
        set(
            &mut mappings,
            LegacyOptionAction::Fire1,
            key(LegacyPhysicalKey::Z),
            key(LegacyPhysicalKey::Mouse0),
        );
        set(
            &mut mappings,
            LegacyOptionAction::Fire2,
            key(LegacyPhysicalKey::X),
            key(LegacyPhysicalKey::Mouse1),
        );
        set(
            &mut mappings,
            LegacyOptionAction::Nano1,
            key(LegacyPhysicalKey::Digit1),
            key(LegacyPhysicalKey::Numpad1),
        );
        set(
            &mut mappings,
            LegacyOptionAction::Nano2,
            key(LegacyPhysicalKey::Digit2),
            key(LegacyPhysicalKey::Numpad2),
        );
        set(
            &mut mappings,
            LegacyOptionAction::Nano3,
            key(LegacyPhysicalKey::Digit3),
            key(LegacyPhysicalKey::Numpad3),
        );
        set(
            &mut mappings,
            LegacyOptionAction::WeaponChange,
            key(LegacyPhysicalKey::Tab),
            LegacyInputBinding::Unbound,
        );
        set(
            &mut mappings,
            LegacyOptionAction::WeaponCharge,
            key(LegacyPhysicalKey::R),
            LegacyInputBinding::Unbound,
        );
        set(
            &mut mappings,
            LegacyOptionAction::NanoCharge,
            key(LegacyPhysicalKey::C),
            LegacyInputBinding::Unbound,
        );
        set(
            &mut mappings,
            LegacyOptionAction::Inventory,
            key(LegacyPhysicalKey::I),
            LegacyInputBinding::Unbound,
        );
        set(
            &mut mappings,
            LegacyOptionAction::NanoBook,
            key(LegacyPhysicalKey::N),
            LegacyInputBinding::Unbound,
        );
        set(
            &mut mappings,
            LegacyOptionAction::Journal,
            key(LegacyPhysicalKey::J),
            LegacyInputBinding::Unbound,
        );
        set(
            &mut mappings,
            LegacyOptionAction::Email,
            key(LegacyPhysicalKey::P),
            LegacyInputBinding::Unbound,
        );
        set(
            &mut mappings,
            LegacyOptionAction::Help,
            LegacyInputBinding::Unbound,
            LegacyInputBinding::Unbound,
        );
        set(
            &mut mappings,
            LegacyOptionAction::AutoRun,
            key(LegacyPhysicalKey::Home),
            LegacyInputBinding::Unbound,
        );
        set(
            &mut mappings,
            LegacyOptionAction::Menu,
            key(LegacyPhysicalKey::Enter),
            key(LegacyPhysicalKey::NumpadEnter),
        );
        set(
            &mut mappings,
            LegacyOptionAction::WorldMap,
            key(LegacyPhysicalKey::M),
            LegacyInputBinding::Unbound,
        );
        set(
            &mut mappings,
            LegacyOptionAction::Option,
            key(LegacyPhysicalKey::Quote),
            LegacyInputBinding::Unbound,
        );
        set(
            &mut mappings,
            LegacyOptionAction::LeftTurn,
            key(LegacyPhysicalKey::Q),
            key(LegacyPhysicalKey::ArrowLeft),
        );
        set(
            &mut mappings,
            LegacyOptionAction::RightTurn,
            key(LegacyPhysicalKey::E),
            key(LegacyPhysicalKey::ArrowRight),
        );
        set(
            &mut mappings,
            LegacyOptionAction::FreeCamera,
            key(LegacyPhysicalKey::ControlLeft),
            key(LegacyPhysicalKey::ControlRight),
        );
        set(
            &mut mappings,
            LegacyOptionAction::ZoomIn,
            LegacyInputBinding::Axis {
                axis: LegacyInputAxis::MouseWheel,
                direction: LegacyAxisDirection::Positive,
            },
            LegacyInputBinding::Unbound,
        );
        set(
            &mut mappings,
            LegacyOptionAction::ZoomOut,
            LegacyInputBinding::Axis {
                axis: LegacyInputAxis::MouseWheel,
                direction: LegacyAxisDirection::Negative,
            },
            LegacyInputBinding::Unbound,
        );
        set(
            &mut mappings,
            LegacyOptionAction::CameraUp,
            LegacyInputBinding::Axis {
                axis: LegacyInputAxis::MouseY,
                direction: LegacyAxisDirection::Positive,
            },
            LegacyInputBinding::Unbound,
        );
        set(
            &mut mappings,
            LegacyOptionAction::CameraDown,
            LegacyInputBinding::Axis {
                axis: LegacyInputAxis::MouseY,
                direction: LegacyAxisDirection::Negative,
            },
            LegacyInputBinding::Unbound,
        );
        set(
            &mut mappings,
            LegacyOptionAction::CameraLeft,
            LegacyInputBinding::Axis {
                axis: LegacyInputAxis::MouseX,
                direction: LegacyAxisDirection::Negative,
            },
            LegacyInputBinding::Unbound,
        );
        set(
            &mut mappings,
            LegacyOptionAction::CameraRight,
            LegacyInputBinding::Axis {
                axis: LegacyInputAxis::MouseX,
                direction: LegacyAxisDirection::Positive,
            },
            LegacyInputBinding::Unbound,
        );
        set(
            &mut mappings,
            LegacyOptionAction::Skill1,
            key(LegacyPhysicalKey::R),
            LegacyInputBinding::Unbound,
        );
        set(
            &mut mappings,
            LegacyOptionAction::Skill2,
            key(LegacyPhysicalKey::F),
            LegacyInputBinding::Unbound,
        );
        set(
            &mut mappings,
            LegacyOptionAction::Skill3,
            key(LegacyPhysicalKey::Z),
            key(LegacyPhysicalKey::Mouse0),
        );
        set(
            &mut mappings,
            LegacyOptionAction::VehicleToggle,
            key(LegacyPhysicalKey::V),
            LegacyInputBinding::Unbound,
        );
        reset_pad_profile_bindings(&mut mappings, LegacyPadProfile::Xbox360);

        Self {
            input_version: 2,
            camera_sensitivity: 5.0,
            invert_y: false,
            pad_invert_y: false,
            pad_camera_sensitivity: default_pad_camera_sensitivity(),
            pad_profile: LegacyPadProfile::Xbox360,
            mappings,
        }
    }
}

impl InputSettings {
    #[must_use]
    pub fn has_complete_clean_schema(&self) -> bool {
        self.input_version == 2
            && self.mappings.len() == OPTION_INPUT_ACTION_COUNT
            && self
                .mappings
                .iter()
                .zip(LegacyOptionAction::ALL)
                .all(|(row, action)| row.action == action)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionTabButton(pub OptionTab);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub enum OptionChromeButton {
    Close,
    Apply,
    Save,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionSocialFlagButton {
    pub request: SocialRequestKind,
    pub value: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionSocialDefaultsButton;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionUnignoreButton;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub enum OptionBlockedScrollButton {
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionBlockedScrollThumb;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct OptionBlockedScrollChrome;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct OptionPageButtonVisual;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct OptionButtonLabel;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct OptionKeyButtonLabel;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionGraphicsDefaultsButton;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionGraphicsToggleButton {
    pub toggle: OptionGraphicsToggle,
    pub value: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub enum OptionGraphicsSliderButton {
    Visibility(u8),
    Particles(u8),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionDropdownButton(pub OptionDropdownKind);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionDisplayFlagButton {
    pub element: OptionDisplayElement,
    pub value: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionDisplayDefaultsButton;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionTextColorDefaultsButton;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionTextColorButton {
    pub channel: OptionTextColorChannel,
    pub index: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionInputDefaultsButton;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionKeyMappingDefaultsButton;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionInvertYButton(pub bool);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionSensitivityButton(pub u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionMappingButton {
    pub action: LegacyOptionAction,
    pub slot: OptionInputMappingSlot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub enum OptionControlsScrollButton {
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionControlsScrollContent;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionControlsScrollThumb;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionSystemPopupOkButton;

pub(super) fn option_button_node(rect: OptionUiRect) -> Node {
    Node {
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        padding: UiRect {
            left: px(6.0),
            right: px(6.0),
            top: px(3.0),
            bottom: px(6.0),
        },
        ..rect.node()
    }
}

pub(super) const fn tab_hover_role(tab: OptionTab) -> OptionTextureRole {
    match tab {
        OptionTab::Graphics => OptionTextureRole::GraphicsHover,
        OptionTab::GameUi => OptionTextureRole::GameUiHover,
        OptionTab::Social => OptionTextureRole::SocialHover,
        OptionTab::Controls => OptionTextureRole::ControlsHover,
    }
}

pub(super) fn capture_option_binding_input(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    gamepads: Query<(Entity, &bevy::input::gamepad::Gamepad)>,
    mut held_axes: Local<std::collections::HashSet<(Entity, u8, LegacyAxisDirection)>>,
    mut model: ResMut<OptionUiModel>,
) {
    use bevy::input::gamepad::GamepadAxis;
    let mut current_axes = std::collections::HashSet::new();
    let profile = model.draft_input.pad_profile;
    for (entity, pad) in &gamepads {
        let camera_axes = match profile {
            LegacyPadProfile::Xbox360 => [(6, GamepadAxis::RightStickX, 1.0), (7, GamepadAxis::RightStickY, -1.0)],
            LegacyPadProfile::RumblePad2 => [(5, GamepadAxis::RightStickX, 1.0), (6, GamepadAxis::RightStickY, -1.0)],
            LegacyPadProfile::PlayStation2 => [(6, GamepadAxis::RightStickX, 1.0), (5, GamepadAxis::RightStickY, -1.0)],
        };
        for (index, axis, sign) in [
            (3, GamepadAxis::LeftStickX, 1.0),
            (4, GamepadAxis::LeftStickY, -1.0),
            camera_axes[0], camera_axes[1],
        ] {
            let value = pad.get(axis).unwrap_or(0.0) * sign;
            if value.abs() >= 0.75 {
                current_axes.insert((entity, index,
                    if value > 0.0 { LegacyAxisDirection::Positive }
                    else { LegacyAxisDirection::Negative }));
            }
        }
        let zoom_index = if profile == LegacyPadProfile::Xbox360 { 9 } else { 8 };
        let zoom = pad.get(bevy::input::gamepad::GamepadButton::RightTrigger2).unwrap_or(0.0)
            - pad.get(bevy::input::gamepad::GamepadButton::LeftTrigger2).unwrap_or(0.0);
        if zoom.abs() >= 0.75 {
            current_axes.insert((entity, zoom_index,
                if zoom > 0.0 { LegacyAxisDirection::Positive } else { LegacyAxisDirection::Negative }));
        }
    }
    let newly_moved = current_axes.difference(&held_axes).next().copied();
    *held_axes = current_axes;
    let Some(capture) = model.key_capture else {
        return;
    };
    if capture.slot == OptionInputMappingSlot::Pad {
        use bevy::input::gamepad::GamepadButton;
        let profile = model.draft_input.pad_profile;
        for (_, pad) in &gamepads {
            let button = pad.get_just_pressed().find_map(|button| {
                let index = match button {
                    GamepadButton::South => if profile == LegacyPadProfile::Xbox360 { 0 } else { 1 },
                    GamepadButton::East => if profile == LegacyPadProfile::Xbox360 { 1 } else { 2 },
                    GamepadButton::West => if profile == LegacyPadProfile::Xbox360 { 2 } else { 0 },
                    GamepadButton::North => 3,
                    GamepadButton::LeftTrigger => 4,
                    GamepadButton::RightTrigger => 5,
                    GamepadButton::Select => if profile == LegacyPadProfile::Xbox360 { 6 } else { 8 },
                    GamepadButton::Start => if profile == LegacyPadProfile::Xbox360 { 7 } else { 9 },
                    GamepadButton::LeftThumb => 10,
                    GamepadButton::DPadUp => 12,
                    GamepadButton::DPadDown => 13,
                    GamepadButton::DPadLeft => 14,
                    GamepadButton::DPadRight => 15,
                    GamepadButton::LeftTrigger2 => 16,
                    GamepadButton::RightTrigger2 => 17,
                    GamepadButton::RightThumb => 11,
                    _ => return None,
                };
                Some(LegacyInputBinding::PadButton(index))
            });
            if let Some(binding) = button.or_else(|| newly_moved.map(|(_, axis, direction)| LegacyInputBinding::PadAxis { axis, direction })) {
                model.submit_captured_binding(binding);
                return;
            }
        }
        if keyboard.as_ref().is_some_and(|keys| keys.just_pressed(KeyCode::Backspace) || keys.just_pressed(KeyCode::Delete)) {
            model.submit_captured_binding(LegacyInputBinding::Unbound);
        }
        return;
    }
    let mouse_binding = mouse.as_ref().and_then(|buttons| {
        if buttons.just_pressed(MouseButton::Left) {
            Some(LegacyInputBinding::Key(LegacyPhysicalKey::Mouse0))
        } else if buttons.just_pressed(MouseButton::Right) {
            Some(LegacyInputBinding::Key(LegacyPhysicalKey::Mouse1))
        } else {
            None
        }
    });
    let keyboard_binding = keyboard.as_ref().and_then(|keys| {
        if keys.just_pressed(KeyCode::Backspace) || keys.just_pressed(KeyCode::Delete) {
            return Some(LegacyInputBinding::Unbound);
        }
        keys.get_just_pressed()
            .find_map(|key| legacy_physical_key(*key).map(LegacyInputBinding::Key))
    });
    if let Some(binding) = mouse_binding.or(keyboard_binding) {
        model.submit_captured_binding(binding);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_option_interactions(
    tabs: Query<(&Interaction, &OptionTabButton), Changed<Interaction>>,
    chrome: Query<(&Interaction, &OptionChromeButton), Changed<Interaction>>,
    flags: Query<(&Interaction, &OptionSocialFlagButton), Changed<Interaction>>,
    defaults: Query<&Interaction, (Changed<Interaction>, With<OptionSocialDefaultsButton>)>,
    rows: Query<(&Interaction, &OptionBlockedRow), Changed<Interaction>>,
    scroll: Query<(&Interaction, &OptionBlockedScrollButton), Changed<Interaction>>,
    unignore: Query<&Interaction, (Changed<Interaction>, With<OptionUnignoreButton>)>,
    routing: Res<OptionUiAudioRouting>,
    mut model: ResMut<OptionUiModel>,
    mut outbox: ResMut<OptionUiOutbox>,
) {
    for (interaction, tab) in &tabs {
        if *interaction == Interaction::Pressed {
            model.select_tab(tab.0);
        }
    }
    for (interaction, button) in &chrome {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match button {
            OptionChromeButton::Close => {
                model.cancel(OptionCloseTrigger::CloseButton, routing.close, &mut outbox);
            }
            OptionChromeButton::Apply => {
                model.apply(&mut outbox);
            }
            OptionChromeButton::Save => {
                model.save_and_exit(routing.close, &mut outbox);
            }
        }
    }
    for (interaction, flag) in &flags {
        if *interaction == Interaction::Pressed {
            model.set_social_request(flag.request, flag.value);
        }
    }
    for interaction in &defaults {
        if *interaction == Interaction::Pressed {
            model.restore_social_defaults();
        }
    }
    for (interaction, row) in &rows {
        if *interaction != Interaction::Pressed || !model.page_body_enabled(OptionTab::Social) {
            continue;
        }
        if let Some(projected) = model
            .blocked_projection()
            .get(model.blocked_scroll_row + row.projection_index)
        {
            model.select_blocked_slot(projected.slot);
        }
    }
    for (interaction, direction) in &scroll {
        if *interaction == Interaction::Pressed {
            model.scroll_blocked_rows(match direction {
                OptionBlockedScrollButton::Up => -1,
                OptionBlockedScrollButton::Down => 1,
            });
        }
    }
    for interaction in &unignore {
        if *interaction == Interaction::Pressed {
            model.remove_selected_buddy(&mut outbox);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_graphics_interactions(
    graphics_defaults: Query<
        &Interaction,
        (Changed<Interaction>, With<OptionGraphicsDefaultsButton>),
    >,
    sound_defaults: Query<&Interaction, (Changed<Interaction>, With<OptionSoundDefaultsButton>)>,
    graphics_toggles: Query<(&Interaction, &OptionGraphicsToggleButton), Changed<Interaction>>,
    graphics_sliders: Query<(&Interaction, &OptionGraphicsSliderButton), Changed<Interaction>>,
    sound_toggles: Query<(&Interaction, &OptionSoundToggleButton), Changed<Interaction>>,
    sound_sliders: Query<(&Interaction, &OptionSoundSliderButton), Changed<Interaction>>,
    mut model: ResMut<OptionUiModel>,
    mut outbox: ResMut<OptionUiOutbox>,
) {
    for interaction in &graphics_defaults {
        if *interaction == Interaction::Pressed && model.restore_graphics_defaults() {
            model.emit_button_click(&mut outbox);
        }
    }
    for interaction in &sound_defaults {
        if *interaction == Interaction::Pressed && model.restore_sound_defaults(&mut outbox) {
            model.emit_button_click(&mut outbox);
        }
    }
    for (interaction, button) in &graphics_toggles {
        if *interaction == Interaction::Pressed {
            model.set_graphics_toggle(button.toggle, button.value);
        }
    }
    for (interaction, button) in &graphics_sliders {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match button {
            OptionGraphicsSliderButton::Visibility(step) => {
                model.set_visibility_step(*step);
            }
            OptionGraphicsSliderButton::Particles(step) => {
                model.set_particle_level(*step);
            }
        }
    }
    for (interaction, button) in &sound_toggles {
        if *interaction == Interaction::Pressed {
            model.set_sound_enabled(button.channel, button.value, &mut outbox);
        }
    }
    for (interaction, button) in &sound_sliders {
        if *interaction == Interaction::Pressed {
            model.set_sound_volume_step(button.channel, button.step, &mut outbox);
        }
    }
}

pub(super) fn handle_game_ui_interactions(
    flags: Query<(&Interaction, &OptionDisplayFlagButton), Changed<Interaction>>,
    colors: Query<(&Interaction, &OptionTextColorButton), Changed<Interaction>>,
    display_defaults: Query<
        &Interaction,
        (Changed<Interaction>, With<OptionDisplayDefaultsButton>),
    >,
    color_defaults: Query<
        &Interaction,
        (Changed<Interaction>, With<OptionTextColorDefaultsButton>),
    >,
    mut model: ResMut<OptionUiModel>,
    mut outbox: ResMut<OptionUiOutbox>,
) {
    for (interaction, button) in &flags {
        if *interaction == Interaction::Pressed {
            model.set_display_value(button.element, button.value);
        }
    }
    for (interaction, button) in &colors {
        if *interaction == Interaction::Pressed {
            model.set_text_color(button.channel, button.index);
        }
    }
    for interaction in &display_defaults {
        if *interaction == Interaction::Pressed && model.restore_display_defaults() {
            model.emit_button_click(&mut outbox);
        }
    }
    for interaction in &color_defaults {
        if *interaction == Interaction::Pressed && model.restore_text_color_defaults() {
            model.emit_button_click(&mut outbox);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_controls_interactions(
    input_defaults: Query<&Interaction, (Changed<Interaction>, With<OptionInputDefaultsButton>)>,
    mapping_defaults: Query<
        &Interaction,
        (Changed<Interaction>, With<OptionKeyMappingDefaultsButton>),
    >,
    invert: Query<(&Interaction, &OptionInvertYButton), Changed<Interaction>>,
    sensitivity: Query<(&Interaction, &OptionSensitivityButton), Changed<Interaction>>,
    mappings: Query<(&Interaction, &OptionMappingButton), Changed<Interaction>>,
    scroll: Query<(&Interaction, &OptionControlsScrollButton), Changed<Interaction>>,
    mut model: ResMut<OptionUiModel>,
    mut outbox: ResMut<OptionUiOutbox>,
) {
    for interaction in &input_defaults {
        if *interaction == Interaction::Pressed && model.restore_input_controls() {
            model.emit_button_click(&mut outbox);
        }
    }
    for interaction in &mapping_defaults {
        if *interaction == Interaction::Pressed && model.restore_key_mapping_defaults() {
            model.emit_button_click(&mut outbox);
        }
    }
    for (interaction, button) in &invert {
        if *interaction == Interaction::Pressed {
            model.set_invert_y(button.0);
        }
    }
    for (interaction, button) in &sensitivity {
        if *interaction == Interaction::Pressed {
            model.set_camera_sensitivity(button.0);
        }
    }
    for (interaction, button) in &mappings {
        if *interaction == Interaction::Pressed {
            model.begin_binding_capture(button.action, button.slot);
        }
    }
    for (interaction, direction) in &scroll {
        if *interaction == Interaction::Pressed {
            model.scroll_controls(match direction {
                OptionControlsScrollButton::Up => -10.0,
                OptionControlsScrollButton::Down => 10.0,
            });
        }
    }
}
