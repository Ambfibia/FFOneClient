//! Launcher production bridge and fail-closed owner gaps.

use super::hotkeys;
use super::gamepad::GamepadActionState;
use super::option_runtime::{
    OptionProductionRuntime, option_action_just_pressed, option_action_just_released,
};
use super::runtime_status::RuntimeStatus;
use bevy::prelude::*;
use ffone_client::{
    launcher_ui::{LauncherUiEffect, LauncherUiExternalState, LauncherUiModel, LauncherUiOutbox},
    mission_ui::MissionUiModel,
    option_ui::LegacyOptionAction,
    system_message_ui::SystemMessageUiModel,
};
use std::collections::VecDeque;

pub(super) const LAUNCHER_COMBAT_ICON_OWNER_GAP: &str =
    "Launcher supports only hidden icon -1; visible interaction cues belong to target selection";
pub(super) const LAUNCHER_NAME_VISIBILITY_OWNER_GAP: &str =
    "Launcher local PrintName owner is absent; visible name cannot be restored without faking it";
pub(super) const LAUNCHER_ESCAPE_ROUTE_GAP: &str =
    "Launcher rejected a non-clean or stale Computress escape-gate request";

pub(super) fn launcher_system_popup_active(
    system_messages: &SystemMessageUiModel,
    mission_ui: &MissionUiModel,
) -> bool {
    system_messages.is_popup() || mission_ui.system_popup_active()
}

pub(super) fn launcher_combat_icon_fail_closed(icon: i32) -> Result<(), &'static str> {
    // Clean `cnLauncher.Update` addresses the local avatar's `PrintName` and
    // sends only `-1`. The local trigger cue now derives its hidden state
    // directly from LauncherUiModel. Non-negative launcher requests still
    // must not override either the trigger cue or the NPC target icon.
    if icon == -1 {
        Ok(())
    } else {
        Err(LAUNCHER_COMBAT_ICON_OWNER_GAP)
    }
}

pub(super) fn launcher_name_visibility_fail_closed(visible: bool) -> Result<(), &'static str> {
    // `ReceiveLuncherInit` disables the local avatar's PrintName Behaviour and
    // every clean exit enables that same component. There is no corresponding
    // native local-nameplate component yet. Absence is safely hidden; showing
    // the HUD player-name label instead would target the wrong owner.
    if visible {
        Err(LAUNCHER_NAME_VISIBILITY_OWNER_GAP)
    } else {
        Ok(())
    }
}

pub(super) fn read_launcher_configured_aim(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    options: Res<OptionProductionRuntime>,
    pad: Option<Res<GamepadActionState>>,
    model: Res<LauncherUiModel>,
    mut external: ResMut<LauncherUiExternalState>,
) {
    if !model.visible() { return; }
    // Entering the cannon disables walking. Read its independent configured
    // axes before each aim step, rather than reusing the blocked walking feed.
    let held = |action| (hotkeys::option_action_held(&options.input, action, &keyboard, &mouse) as u8 as f32
        + pad.as_ref().map_or(0.0, |pad| pad.value(action))).clamp(0.0, 1.0);
    external.aim_vertical_axis = held(LegacyOptionAction::Up) - held(LegacyOptionAction::Down);
    external.aim_horizontal_axis = held(LegacyOptionAction::Right) - held(LegacyOptionAction::Left);
}

pub(super) fn prepare_launcher_production_context(
    runtime: Res<RuntimeStatus>,
    system_messages: Res<SystemMessageUiModel>,
    mission_ui: Res<MissionUiModel>,
    model: Res<LauncherUiModel>,
    mut external: ResMut<LauncherUiExternalState>,
) {
    external.current_hp = runtime.hp.unwrap_or_default();
    let popup_active = launcher_system_popup_active(&system_messages, &mission_ui);
    // The standalone plugin retains its raw Space/Escape compatibility reader.
    // Mask that reader while production owns the visible mode; the ordered
    // driver below restores the real popup state and applies ConfigurableInput
    // Key(5)/Key(4) through OptionProductionRuntime instead.
    external.system_popup_active = popup_active || model.visible();
}

#[allow(clippy::too_many_arguments)]
pub(super) fn advance_launcher_production_frame(
    model: &mut LauncherUiModel,
    outbox: &mut LauncherUiOutbox,
    delta_seconds: f32,
    fire_pressed: bool,
    fire_released: bool,
    escape_pressed: bool,
    current_hp: i32,
    system_popup_active: bool,
) {
    if !model.visible() || system_popup_active {
        return;
    }
    if fire_pressed {
        model.press_fire(false, outbox);
    }
    model.advance_power(delta_seconds, false, outbox);
    if fire_released {
        model.release_fire(false, outbox);
    }
    if !model.visible() {
        return;
    }
    if current_hp <= 0 {
        model.cancel_for_death(false, outbox);
    } else if escape_pressed {
        model.request_escape_close(false, outbox);
    }
}

pub(super) fn bridge_launcher_production_effects(
    model: &mut LauncherUiModel,
    outbox: &mut LauncherUiOutbox,
    system_popup_active: bool,
) -> Vec<&'static str> {
    let mut forwarded = VecDeque::new();
    let mut issues = Vec::new();
    while let Some(effect) = outbox.pop_front() {
        match effect {
            LauncherUiEffect::SetCombatIcon(icon) => {
                if let Err(issue) = launcher_combat_icon_fail_closed(icon) {
                    issues.push(issue);
                }
            }
            LauncherUiEffect::SetNameVisible(visible) => {
                if let Err(issue) = launcher_name_visibility_fail_closed(visible) {
                    issues.push(issue);
                }
            }
            LauncherUiEffect::RequestEscapeCloseGate {
                event_group,
                event_function,
            } => {
                let exact_route = event_group == 2 && event_function == 24;
                // `GameFrame.RecieveCloseFirstUseComputress` returns one when
                // its absent Computress owner returns false. A real popup that
                // appeared between input and consumption remains fail-closed.
                let accepted = exact_route && !system_popup_active;
                let resolved = model.resolve_escape_close_gate(accepted, outbox);
                if !exact_route || !resolved {
                    issues.push(LAUNCHER_ESCAPE_ROUTE_GAP);
                }
            }
            effect => forwarded.push_back(effect),
        }
    }
    for effect in forwarded {
        outbox.push(effect);
    }
    issues
}

#[allow(clippy::too_many_arguments)]
pub(super) fn drive_launcher_production(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    option_runtime: Res<OptionProductionRuntime>,
    pad: Option<Res<GamepadActionState>>,
    mut pad_charging: Local<bool>,
    system_messages: Res<SystemMessageUiModel>,
    mission_ui: Res<MissionUiModel>,
    mut external: ResMut<LauncherUiExternalState>,
    mut model: ResMut<LauncherUiModel>,
    mut outbox: ResMut<LauncherUiOutbox>,
    mut status: ResMut<RuntimeStatus>,
) {
    let current_hp = status.hp.unwrap_or_default();
    let system_popup_active = launcher_system_popup_active(&system_messages, &mission_ui);
    external.current_hp = current_hp;
    external.system_popup_active = system_popup_active;

    if *pad_charging && !pad.as_ref().is_some_and(|pad| pad.connected()) {
        // Losing the device cancels its charge; it must never launch on disconnect.
        model.request_escape_close(false, &mut outbox);
        *pad_charging = false;
    }
    let pad_pressed = pad.as_ref().is_some_and(|pad| pad.just_pressed(LegacyOptionAction::Jump));
    let pad_released = pad.as_ref().is_some_and(|pad| pad.just_released(LegacyOptionAction::Jump));
    if pad_pressed && model.visible() && !system_popup_active { *pad_charging = true; }
    if pad_released || !model.visible() { *pad_charging = false; }
    let fire_pressed = pad_pressed || option_action_just_pressed(
        &option_runtime.input,
        LegacyOptionAction::Jump,
        &keyboard,
        &mouse,
    );
    let fire_released = pad_released || option_action_just_released(
        &option_runtime.input,
        LegacyOptionAction::Jump,
        &keyboard,
        &mouse,
    );
    let escape_pressed = pad.as_ref().is_some_and(|pad| pad.buttons.just_pressed(bevy::input::gamepad::GamepadButton::East)) || option_action_just_pressed(
        &option_runtime.input,
        LegacyOptionAction::Escape,
        &keyboard,
        &mouse,
    );
    advance_launcher_production_frame(
        &mut model,
        &mut outbox,
        time.delta_secs(),
        fire_pressed,
        fire_released,
        escape_pressed,
        current_hp,
        system_popup_active,
    );
    if let Some(issue) =
        bridge_launcher_production_effects(&mut model, &mut outbox, system_popup_active).last()
    {
        status.message = (*issue).to_owned();
    }
}
