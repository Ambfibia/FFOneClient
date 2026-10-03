//! Configurable NanoCom shortcuts, ordered before the individual mode readers.
use super::*;
use ffone_client::{
    gameplay_ui::{GameplayMenuTransition, GameplayUiAudioCue},
    user_equip_ui::{UserEquipCloseSource, UserEquipMode, UserEquipOpenSource},
};

const NANOCOM_SHORTCUTS: [LegacyOptionAction; 7] = [
    LegacyOptionAction::Inventory,
    LegacyOptionAction::NanoBook,
    LegacyOptionAction::Journal,
    LegacyOptionAction::Email,
    LegacyOptionAction::WorldMap,
    LegacyOptionAction::Option,
    LegacyOptionAction::Escape,
];

pub(super) fn option_action_held(
    input: &InputSettings,
    action: LegacyOptionAction,
    keyboard: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
) -> bool {
    input
        .mappings
        .iter()
        .find(|row| row.action == action)
        .is_some_and(|row| {
            [row.primary, row.alternate]
                .into_iter()
                .any(|binding| option_binding_held(binding, keyboard, mouse))
        })
}

fn option_binding_held(
    binding: LegacyInputBinding,
    keyboard: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
) -> bool {
    match binding {
        LegacyInputBinding::Key(LegacyPhysicalKey::Mouse0) => mouse.pressed(MouseButton::Left),
        LegacyInputBinding::Key(LegacyPhysicalKey::Mouse1) => mouse.pressed(MouseButton::Right),
        LegacyInputBinding::Key(physical) => keyboard
            .get_pressed()
            .any(|key| legacy_physical_key(*key) == Some(physical)),
        _ => false,
    }
}

fn pad_has_interaction_target(state: &LegacyAvatarActionState) -> bool {
    let selection = &state.target_selection;
    selection.trigger.is_some()
        || selection
            .focused_npc
            .is_some_and(|npc| npc.talk_enabled && !selection.check_attack_target)
}

/// Replace the movement plugin's legacy fixed keys with committed Controls
/// mappings before its camera and simulation stages consume the input state.
pub(super) fn read_configured_movement_input(
    options: Res<OptionProductionRuntime>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    pad: Option<Res<GamepadActionState>>,
    gate: Res<ffone_client::movement::LegacyInputGate>,
    mut input: ResMut<ffone_client::movement::LegacyInputState>,
    mut camera_keys: ResMut<ffone_client::movement::LegacyCameraKeyInput>,
    targets: Query<&LegacyAvatarActionState, With<LocalPlayer>>,
    zipline: Query<
        (),
        (
            With<LocalPlayer>,
            With<ffone_client::world_behaviour::WorldZiplineTraversal>,
        ),
    >,
) {
    let pad_value = |action| pad.as_ref().map_or(0.0, |pad| pad.value(action));
    let pad_held = |action| pad.as_ref().is_some_and(|pad| pad.held(action));
    let pad_pressed = |action| pad.as_ref().is_some_and(|pad| pad.just_pressed(action));
    let held = |action| option_action_held(&options.input, action, &keyboard, &mouse);
    let slot_held = |action, alternate| {
        options
            .input
            .mappings
            .iter()
            .find(|row| row.action == action)
            .is_some_and(|row| {
                let binding = if alternate {
                    row.alternate
                } else {
                    row.primary
                };
                option_binding_held(binding, &keyboard, &mouse)
            })
    };
    let horizontal = if gate.allow_strafe {
        if held(LegacyOptionAction::Right) {
            1.0
        } else if held(LegacyOptionAction::Left) {
            -1.0
        } else {
            0.0
        }
    } else {
        0.0
    };
    let vertical_slot = |alternate| {
        if gate.allow_backward && slot_held(LegacyOptionAction::Down, alternate) {
            -1.0_f32
        } else if gate.allow_forward && slot_held(LegacyOptionAction::Up, alternate) {
            1.0_f32
        } else {
            0.0_f32
        }
    };
    let pad_x = pad_value(LegacyOptionAction::Right) - pad_value(LegacyOptionAction::Left);
    let pad_y = pad_value(LegacyOptionAction::Up) - pad_value(LegacyOptionAction::Down);
    input.local_axis = Vec2::new(
        if gate.allow_strafe {
            (horizontal + pad_x).clamp(-1.0, 1.0)
        } else {
            0.0
        },
        (vertical_slot(false)
            + vertical_slot(true)
            + if pad_y > 0.0 && gate.allow_forward || pad_y < 0.0 && gate.allow_backward {
                pad_y
            } else {
                0.0
            })
        .clamp(-1.0, 1.0),
    );
    let pad_interaction = zipline.is_empty()
        && targets
            .single()
            .is_ok_and(|state| pad_has_interaction_target(state));
    input.jump_just_pressed = gate.allow_jump
        && option_action_just_pressed(&options.input, LegacyOptionAction::Jump, &keyboard, &mouse)
        || gate.allow_jump && !pad_interaction && pad_pressed(LegacyOptionAction::Jump);
    input.flight_ascend_held = gate.allow_jump
        && (held(LegacyOptionAction::Jump)
            || !pad_interaction && pad_held(LegacyOptionAction::Jump));
    input.free_camera_held =
        held(LegacyOptionAction::FreeCamera) || pad_held(LegacyOptionAction::FreeCamera);
    input.auto_run_just_pressed = gate.allow_forward
        && option_action_just_pressed(
            &options.input,
            LegacyOptionAction::AutoRun,
            &keyboard,
            &mouse,
        )
        || gate.allow_forward && pad_pressed(LegacyOptionAction::AutoRun);
    camera_keys.turn_left =
        held(LegacyOptionAction::LeftTurn) || pad_held(LegacyOptionAction::LeftTurn);
    camera_keys.turn_right =
        held(LegacyOptionAction::RightTurn) || pad_held(LegacyOptionAction::RightTurn);
    camera_keys.recenter = [
        LegacyOptionAction::Up,
        LegacyOptionAction::Down,
        LegacyOptionAction::LeftTurn,
        LegacyOptionAction::RightTurn,
    ]
    .into_iter()
    .any(|action| slot_held(action, true));
    camera_keys.pad_axis = if gate.allow_mouse_camera {
        Vec2::new(
            pad_value(LegacyOptionAction::CameraRight) - pad_value(LegacyOptionAction::CameraLeft),
            pad_value(LegacyOptionAction::CameraUp) - pad_value(LegacyOptionAction::CameraDown),
        ) * Vec2::new(
            1.0,
            if options.input.pad_invert_y {
                -1.0
            } else {
                1.0
            },
        ) * (options.input.pad_camera_sensitivity.clamp(1.0, 10.0) / 5.0)
    } else {
        Vec2::ZERO
    };
    camera_keys.pad_zoom = if gate.allow_mouse_camera {
        pad_value(LegacyOptionAction::ZoomOut) - pad_value(LegacyOptionAction::ZoomIn)
    } else {
        0.0
    };
}

pub(super) fn read_configured_avatar_action_input(
    options: Res<OptionProductionRuntime>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    pad: Option<Res<GamepadActionState>>,
    gate: Option<Res<ffone_client::movement::LegacyInputGate>>,
    pointer: Res<ffone_client::input_focus::GameplayPointerCapture>,
    mut input: ResMut<ffone_client::avatar_action::LegacyAvatarActionInput>,
    targets: Query<&LegacyAvatarActionState, With<LocalPlayer>>,
) {
    let pad_allowed = gate
        .as_ref()
        .is_none_or(|gate| gate.allow_jump || gate.allow_forward || gate.allow_strafe);
    let pad_held = |action| pad_allowed && pad.as_ref().is_some_and(|pad| pad.held(action));
    let pad_pressed =
        |action| pad_allowed && pad.as_ref().is_some_and(|pad| pad.just_pressed(action));
    // Keep the pointer ownership boundary used by the avatar plugin when
    // projecting the committed (rather than the options dialog's draft) keys.
    let empty_mouse = ButtonInput::default();
    let mouse = if pointer.blocked {
        &empty_mouse
    } else {
        &*mouse
    };
    input.primary_held =
        option_action_held(&options.input, LegacyOptionAction::Fire1, &keyboard, mouse)
            || pad_held(LegacyOptionAction::Fire1);
    input.primary_just_pressed =
        option_action_just_pressed(&options.input, LegacyOptionAction::Fire1, &keyboard, mouse)
            || pad_pressed(LegacyOptionAction::Fire1)
            || pad_pressed(LegacyOptionAction::Jump)
                && targets.single().is_ok_and(pad_has_interaction_target);
    input.nano_just_pressed =
        option_action_just_pressed(&options.input, LegacyOptionAction::Fire2, &keyboard, mouse)
            || pad_pressed(LegacyOptionAction::Fire2);
    input.weapon_cycle_just_pressed = option_action_just_pressed(
        &options.input,
        LegacyOptionAction::WeaponChange,
        &keyboard,
        mouse,
    ) || pad_pressed(LegacyOptionAction::WeaponChange);
}

fn nano_charge_request(
    status: &RuntimeStatus,
    context: &LegacyAvatarActionContext,
    now: f64,
    last_attempt: &mut f64,
) -> Option<ffone_protocol::RegisteredGameplayRequest0104> {
    if !context.ready_for_play
        || !context.input_enabled
        || context.system_popup
        || matches!(context.time_buff_condition, 5 | 6)
        || !status
            .nano_slots
            .iter()
            .any(|slot| slot.active && slot.nano_id.is_some())
        || now - *last_attempt < 1.0
    {
        return None;
    }
    // cnAvatarAttack.NanoCharge calls IsCallCoolTime(5) before checking fuel.
    *last_attempt = now;
    if status.nano_battery <= 0 {
        return None;
    }
    ffone_protocol::RegisteredGameplayRequest0104::new(
        packet::P_CL2FE_REQ_CHARGE_NANO_STAMINA,
        ffone_protocol::wire_0104::ChargeNanoStaminaRequest0104 {
            pc_id: status.player_id?,
        }
        .encode(),
    )
    .ok()
}

pub(super) fn charge_world_nano_from_keyboard(
    time: Res<Time>,
    options: Res<OptionProductionRuntime>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    pad: Option<Res<GamepadActionState>>,
    players: Query<&LegacyAvatarActionContext, With<LocalPlayer>>,
    bridge: Res<NetworkBridge>,
    mut status: ResMut<RuntimeStatus>,
    mut last_attempt: Local<f64>,
) {
    if !pad
        .as_ref()
        .is_some_and(|pad| pad.just_pressed(LegacyOptionAction::NanoCharge))
        && !option_action_just_pressed(
            &options.input,
            LegacyOptionAction::NanoCharge,
            &keyboard,
            &mouse,
        )
    {
        return;
    }
    let Ok(context) = players.single() else {
        return;
    };
    if let Some(request) =
        nano_charge_request(&status, context, time.elapsed_secs_f64(), &mut last_attempt)
        && let Err(error) = bridge.send(NetworkCommand::SendRegisteredGameplay0104(request))
    {
        status.message = format!("Nano charge send failed: {error}");
    }
}

pub(super) fn apply_nano_charge_frame(
    frame: &DecodedFrame,
    status: &mut RuntimeStatus,
    bank: &mut NanoFreeTuningBank0104,
    audio: &mut GameplayAudioRuntime,
) -> Result<(), String> {
    let reply = ffone_protocol::wire_0104::ChargeNanoStaminaReply0104::decode(&frame.payload)
        .map_err(|error| format!("Nano charge reply rejected: {error}"))?;
    bank.apply_stamina(reply.nano_id, reply.nano_stamina)
        .map_err(|error| error.to_string())?;
    status.nano_battery = reply.battery_n;
    for slot in &mut status.nano_slots {
        if slot.nano_id == Some(reply.nano_id) {
            slot.stamina = reply.nano_stamina;
        }
    }
    audio.queue_gameplay_ui_sound("Nano_Potion");
    Ok(())
}

fn first_nanocom_shortcut(
    input: &InputSettings,
    keyboard: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
    pad: Option<&GamepadActionState>,
) -> Option<LegacyOptionAction> {
    NANOCOM_SHORTCUTS.into_iter().find(|action| {
        option_action_just_pressed(input, *action, keyboard, mouse)
            || pad.is_some_and(|pad| pad.just_pressed(*action))
    })
}

// Older settings snapshots contain the former default H binding. Keep the
// saved row intact, but do not route that physical key to the guide. A user
// can still assign another Help key through Controls.
fn help_shortcut_pressed(
    input: &InputSettings,
    keyboard: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
) -> bool {
    input
        .mappings
        .iter()
        .find(|row| row.action == LegacyOptionAction::Help)
        .is_some_and(|row| {
            [row.primary, row.alternate].into_iter().any(|binding| {
                binding != LegacyInputBinding::Key(LegacyPhysicalKey::H)
                    && option_binding_just_pressed(binding, keyboard, mouse)
            })
        })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn route_world_ui_shortcuts(
    state: Res<State<ClientState>>,
    runtime: Res<OptionProductionRuntime>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    pad: Option<Res<GamepadActionState>>,
    mut owners: ParamSet<(GameplayModalModels, ResMut<GameGuideUiModel>)>,
    mut mission: ResMut<MissionUiModel>,
    transition: Res<GameplayMenuTransition>,
    equip_modal: Res<UserEquipModalState>,
    mut equip_outbox: ResMut<UserEquipUiOutbox>,
    mut outbox: ResMut<GameplayUiOutbox>,
    mut audio: ResMut<GameplayUiAudioOutbox>,
    launcher: Res<LauncherUiModel>,
) {
    if *state.get() != ClientState::World {
        return;
    }
    let pressed = |action| {
        option_action_just_pressed(&runtime.input, action, &keyboard, &mouse)
            || pad.as_ref().is_some_and(|pad| pad.just_pressed(action))
    };
    let modals = owners.p0();
    if modals.system_popup()
        || mission.system_popup_active()
        || modals.chat_active(*state.get())
        || launcher.visible()
    {
        return;
    }
    if modals.game_guide_modal() {
        let close = help_shortcut_pressed(&runtime.input, &keyboard, &mouse)
            || pad
                .as_ref()
                .is_some_and(|pad| pad.just_pressed(LegacyOptionAction::Help))
            || pressed(LegacyOptionAction::Escape)
            || keyboard.just_pressed(KeyCode::Escape);
        drop(modals);
        if close {
            owners.p1().close();
            keyboard.clear_just_pressed(KeyCode::Escape);
        }
        // Help owns the frame even when it just closed. Lower mode readers
        // must not open a second window from a simultaneous shortcut.
        for action in NANOCOM_SHORTCUTS
            .into_iter()
            .chain([LegacyOptionAction::Help])
        {
            clear_option_action_press(&runtime.input, action, &mut keyboard, &mut mouse);
        }
        return;
    }
    if let Some(equip) = modals
        .user_equip_ui
        .as_ref()
        .filter(|equip| equip.is_active())
    {
        if equip.nano_station_npc().is_some()
            || !equip
                .input_capabilities(*equip_modal)
                .keyboard_close_request
        {
            return;
        }
        let requested = if pressed(LegacyOptionAction::Inventory) {
            Some(UserEquipMode::Item)
        } else if pressed(LegacyOptionAction::NanoBook) {
            Some(UserEquipMode::Nano)
        } else {
            None
        };
        if let Some(mode) = requested {
            equip_outbox.push(if mode == equip.mode() {
                UserEquipUiAction::RequestClose {
                    source: UserEquipCloseSource::ActiveTabHotkey,
                }
            } else {
                UserEquipUiAction::SelectTab { mode }
            });
            // An active mode owns this frame, including a simultaneous Escape.
            keyboard.clear_just_pressed(KeyCode::Escape);
            for action in NANOCOM_SHORTCUTS {
                clear_option_action_press(&runtime.input, action, &mut keyboard, &mut mouse);
            }
        }
        return;
    }
    let foreign_modal = modals.buddy_modal()
        || modals.quit_modal()
        || modals.option_modal()
        || modals.resurrect_modal()
        || modals.upsell_modal()
        || modals.guide_modal()
        || modals.bank_modal()
        || modals.vendor_modal()
        || modals.rule_modal()
        || modals.nano_free_tuning_modal()
        || modals.pc2pc_modal()
        || modals.world_map_modal()
        || modals.transportation_modal()
        || modals.race_modal()
        || modals.email_modal()
        || (modals.combi_modal() || modals.barber_modal())
        || modals.enchant_modal()
        || modals.cashmall_modal()
        || modals.user_store_modal();
    if foreign_modal {
        return;
    }
    if !matches!(mission.journal, MissionJournalUi::Hidden) {
        if (pressed(LegacyOptionAction::Journal)
            || pressed(LegacyOptionAction::Escape)
            || keyboard.just_pressed(KeyCode::Escape))
            && mission.close_journal(&mut outbox)
        {
            audio.push(GameplayUiAudioCue::CloseScreen);
            keyboard.clear_just_pressed(KeyCode::Escape);
            for action in NANOCOM_SHORTCUTS {
                clear_option_action_press(&runtime.input, action, &mut keyboard, &mut mouse);
            }
        }
        return;
    }
    if !mission.enabled
        || mission.gameplay_input_blocked()
        || transition.open
        || transition.remaining > 0.0
    {
        return;
    }
    let action = if help_shortcut_pressed(&runtime.input, &keyboard, &mouse)
        || pad
            .as_ref()
            .is_some_and(|pad| pad.just_pressed(LegacyOptionAction::Help))
    {
        Some(LegacyOptionAction::Help)
    } else {
        first_nanocom_shortcut(&runtime.input, &keyboard, &mouse, pad.as_deref())
    };
    match action {
        Some(LegacyOptionAction::Inventory) => {
            outbox.push(GameplayUiAction::OpenUserEquipItemMode {
                source: UserEquipOpenSource::InventoryShortcut,
            })
        }
        Some(LegacyOptionAction::NanoBook) => {
            outbox.push(GameplayUiAction::OpenUserEquipItemMode {
                source: UserEquipOpenSource::NanoBookShortcut,
            })
        }
        Some(LegacyOptionAction::Journal) => {
            if mission.open_journal_from_shortcut(&mut outbox) {
                audio.push(GameplayUiAudioCue::OpenScreen);
            }
        }
        Some(LegacyOptionAction::Email) => {
            mission.nanocom_new_mail_notice_visible = false;
            outbox.push(GameplayUiAction::OpenEmailFromNanocom);
        }
        Some(LegacyOptionAction::Help) => outbox.push(GameplayUiAction::OpenGameGuideFromNanocom),
        Some(LegacyOptionAction::WorldMap) => {
            for action in [LegacyOptionAction::Option, LegacyOptionAction::Escape] {
                clear_option_action_press(&runtime.input, action, &mut keyboard, &mut mouse);
            }
            return;
        }
        Some(LegacyOptionAction::Option) => {
            clear_option_action_press(
                &runtime.input,
                LegacyOptionAction::Escape,
                &mut keyboard,
                &mut mouse,
            );
            return;
        }
        // These modes retain their own asset-readiness and close arbitration.
        _ => return,
    }
    keyboard.clear_just_pressed(KeyCode::Escape);
    for action in NANOCOM_SHORTCUTS
        .into_iter()
        .chain([LegacyOptionAction::Help])
    {
        clear_option_action_press(&runtime.input, action, &mut keyboard, &mut mouse);
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod gamepad_camera_regressions {
    use super::*;
    use bevy::input::gamepad::{Gamepad, GamepadAxis, GamepadButton};
    #[test]
    fn pad_camera_inversion_and_speed_are_independent_of_mouse_settings() {
        let mut app = App::new();
        let mut options = OptionProductionRuntime::default();
        options.input.pad_camera_sensitivity = 2.0;
        options.input.camera_sensitivity = 10.0;
        options.input.invert_y = true;
        app.insert_resource(options)
            .init_resource::<GamepadActionState>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<ffone_client::movement::LegacyInputGate>()
            .init_resource::<ffone_client::movement::LegacyInputState>()
            .init_resource::<ffone_client::movement::LegacyCameraKeyInput>()
            .add_systems(Update, read_configured_movement_input);
        let settings = app
            .world()
            .resource::<OptionProductionRuntime>()
            .input
            .clone();
        let mut pad = Gamepad::default();
        pad.analog_mut().set(GamepadAxis::RightStickX, 1.0);
        pad.analog_mut().set(GamepadAxis::RightStickY, 1.0);
        app.world_mut()
            .resource_mut::<GamepadActionState>()
            .sample(&settings, Some((Entity::from_bits(1), &pad)));
        app.update();
        let first = app
            .world()
            .resource::<ffone_client::movement::LegacyCameraKeyInput>()
            .pad_axis;
        assert_eq!(first, Vec2::new(0.4, -0.4));
        {
            let mut options = app.world_mut().resource_mut::<OptionProductionRuntime>();
            options.input.pad_invert_y = true;
            options.input.camera_sensitivity = 1.0;
            options.input.invert_y = false;
        }
        app.update();
        let second = app
            .world()
            .resource::<ffone_client::movement::LegacyCameraKeyInput>()
            .pad_axis;
        assert_eq!(second, Vec2::new(first.x, -first.y));
    }

    #[test]
    fn pad_a_dismounts_zipline_even_with_a_nearby_interaction_target() {
        let mut app = App::new();
        app.init_resource::<OptionProductionRuntime>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<GamepadActionState>()
            .init_resource::<ffone_client::movement::LegacyInputGate>()
            .init_resource::<ffone_client::movement::LegacyInputState>()
            .init_resource::<ffone_client::movement::LegacyCameraKeyInput>()
            .add_systems(Update, read_configured_movement_input);
        let mut actions = LegacyAvatarActionState::default();
        actions.target_selection.trigger = Some(Entity::PLACEHOLDER);
        let player = app.world_mut().spawn((LocalPlayer, actions)).id();
        let settings = app
            .world()
            .resource::<OptionProductionRuntime>()
            .input
            .clone();
        let device = Entity::from_bits(1);
        let mut pad = Gamepad::default();
        pad.digital_mut().press(GamepadButton::South);
        app.world_mut()
            .resource_mut::<GamepadActionState>()
            .sample(&settings, Some((device, &pad)));
        app.update();
        assert!(
            !app.world()
                .resource::<ffone_client::movement::LegacyInputState>()
                .jump_just_pressed
        );

        pad.digital_mut().release(GamepadButton::South);
        app.world_mut()
            .resource_mut::<GamepadActionState>()
            .sample(&settings, Some((device, &pad)));
        app.world_mut().entity_mut(player).insert(
            ffone_client::world_behaviour::WorldZiplineTraversal {
                start: Vec3::ZERO,
                end: Vec3::X,
                speed: 1.0,
                travelled: 0.0,
                packet_elapsed: 0.0,
                hang_height: 0.0,
            },
        );
        pad.digital_mut().press(GamepadButton::South);
        app.world_mut()
            .resource_mut::<GamepadActionState>()
            .sample(&settings, Some((device, &pad)));
        app.update();
        assert!(
            app.world()
                .resource::<ffone_client::movement::LegacyInputState>()
                .jump_just_pressed
        );
    }
}
