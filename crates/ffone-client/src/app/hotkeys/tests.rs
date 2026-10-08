use super::*;

#[test]
fn pad_attack_obeys_modal_gate_and_releases() {
    use bevy::input::gamepad::{Gamepad, GamepadButton};
    use ffone_client::avatar_action::LegacyAvatarActionInput;
    let mut app = App::new();
    app.init_resource::<OptionProductionRuntime>()
        .add_message::<bevy::input::gamepad::GamepadConnectionEvent>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<GamepadActionState>()
        .init_resource::<ffone_client::movement::LegacyInputGate>()
        .init_resource::<ffone_client::input_focus::GameplayPointerCapture>()
        .init_resource::<LegacyAvatarActionInput>()
        .add_systems(
            Update,
            (sample_gamepad_actions, read_configured_avatar_action_input).chain(),
        );
    app.world_mut()
        .resource_mut::<OptionProductionRuntime>()
        .input
        .mappings
        .iter_mut()
        .find(|row| row.action == LegacyOptionAction::Fire1)
        .unwrap()
        .pad = LegacyInputBinding::PadButton(0);
    let entity = app.world_mut().spawn(Gamepad::default()).id();
    app.world_mut()
        .get_mut::<Gamepad>(entity)
        .unwrap()
        .digital_mut()
        .press(GamepadButton::South);
    {
        let mut gate = app
            .world_mut()
            .resource_mut::<ffone_client::movement::LegacyInputGate>();
        gate.allow_forward = false;
        gate.allow_strafe = false;
        gate.allow_jump = false;
    }
    app.update();
    let input = app.world().resource::<LegacyAvatarActionInput>();
    assert!(!input.primary_held);
    assert!(!input.primary_just_pressed);
    *app.world_mut()
        .resource_mut::<ffone_client::movement::LegacyInputGate>() = Default::default();
    app.update();
    assert!(
        app.world()
            .resource::<LegacyAvatarActionInput>()
            .primary_held
    );
    app.world_mut()
        .get_mut::<Gamepad>(entity)
        .unwrap()
        .digital_mut()
        .release(GamepadButton::South);
    app.update();
    assert!(
        !app.world()
            .resource::<LegacyAvatarActionInput>()
            .primary_held
    );
}

#[test]
fn committed_controls_drive_movement_and_jump_with_gate() {
    let mut app = App::new();
    app.init_resource::<OptionProductionRuntime>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<ffone_client::movement::LegacyInputGate>()
        .init_resource::<ffone_client::movement::LegacyInputState>()
        .init_resource::<ffone_client::movement::LegacyCameraKeyInput>()
        .add_systems(Update, read_configured_movement_input);
    {
        let mut runtime = app.world_mut().resource_mut::<OptionProductionRuntime>();
        for (action, key) in [
            (LegacyOptionAction::Up, LegacyPhysicalKey::V),
            (LegacyOptionAction::Jump, LegacyPhysicalKey::F),
            (LegacyOptionAction::LeftTurn, LegacyPhysicalKey::H),
        ] {
            let row = runtime
                .input
                .mappings
                .iter_mut()
                .find(|row| row.action == action)
                .unwrap();
            row.primary = LegacyInputBinding::Key(key);
            row.alternate = LegacyInputBinding::Unbound;
        }
        for action in [
            LegacyOptionAction::Skill2,
            LegacyOptionAction::VehicleToggle,
        ] {
            runtime
                .input
                .mappings
                .iter_mut()
                .find(|row| row.action == action)
                .unwrap()
                .primary = LegacyInputBinding::Unbound;
        }
    }
    press(&mut app, &[KeyCode::KeyW, KeyCode::Space, KeyCode::KeyQ]);
    let input = app
        .world()
        .resource::<ffone_client::movement::LegacyInputState>();
    assert_eq!(input.local_axis, Vec2::ZERO);
    assert!(!input.jump_just_pressed);
    assert!(
        !app.world()
            .resource::<ffone_client::movement::LegacyCameraKeyInput>()
            .turn_left
    );
    press(&mut app, &[KeyCode::KeyV, KeyCode::KeyF, KeyCode::KeyH]);
    let input = app
        .world()
        .resource::<ffone_client::movement::LegacyInputState>();
    assert_eq!(input.local_axis, Vec2::Y);
    assert!(input.jump_just_pressed);
    assert!(
        app.world()
            .resource::<ffone_client::movement::LegacyCameraKeyInput>()
            .turn_left
    );
    app.world_mut()
        .resource_mut::<ffone_client::movement::LegacyInputGate>()
        .allow_forward = false;
    press(&mut app, &[KeyCode::KeyV]);
    assert_eq!(
        app.world()
            .resource::<ffone_client::movement::LegacyInputState>()
            .local_axis,
        Vec2::ZERO
    );
}

#[test]
fn game_guide_releases_cursor_and_close_restores_gameplay_capture() {
    let mut app = App::new();
    app.insert_resource(State::new(ClientState::World))
        .init_resource::<TutorialSession>()
        .init_resource::<MissionUiModel>()
        .init_resource::<GameGuideUiModel>()
        .init_resource::<ffone_client::input_focus::GameInputFocus>()
        .add_systems(Update, sync_legacy_gameplay_cursor);
    app.world_mut().spawn(LocalPlayer);
    let window = app
        .world_mut()
        .spawn((
            PrimaryWindow,
            Window {
                focused: true,
                ..default()
            },
            CursorOptions::default(),
        ))
        .id();
    app.update();
    assert_eq!(
        app.world().get::<CursorOptions>(window).unwrap().grab_mode,
        CursorGrabMode::Locked
    );
    app.world_mut()
        .resource_mut::<GameGuideUiModel>()
        .open_from_nanocom();
    app.update();
    let cursor = app.world().get::<CursorOptions>(window).unwrap();
    assert_eq!(cursor.grab_mode, CursorGrabMode::None);
    assert!(cursor.visible);
    app.world_mut().resource_mut::<GameGuideUiModel>().close();
    app.update();
    let cursor = app.world().get::<CursorOptions>(window).unwrap();
    assert_eq!(cursor.grab_mode, CursorGrabMode::Locked);
    assert!(!cursor.visible);
}

#[test]
fn locked_gameplay_pointer_reaches_both_action_readers_but_dialog_close_does_not() {
    use ffone_client::avatar_action::{LegacyAvatarActionInput, read_legacy_avatar_action_input};
    let mut app = App::new();
    app.add_plugins(ffone_client::input_focus::GameplayPointerCapturePlugin)
        .init_resource::<OptionProductionRuntime>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<LegacyAvatarActionInput>()
        .add_systems(
            Update,
            (
                read_legacy_avatar_action_input,
                read_configured_avatar_action_input,
            )
                .chain(),
        );
    let window = app
        .world_mut()
        .spawn((
            PrimaryWindow,
            CursorOptions {
                visible: false,
                grab_mode: CursorGrabMode::Locked,
                ..default()
            },
        ))
        .id();
    let ui = app
        .world_mut()
        .spawn((
            Interaction::Hovered,
            ComputedNode {
                size: Vec2::splat(32.0),
                ..default()
            },
        ))
        .id();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    let input = app.world().resource::<LegacyAvatarActionInput>();
    assert!(input.primary_held && input.primary_just_pressed);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    *app.world_mut().get_mut::<Interaction>(ui).unwrap() = Interaction::Pressed;
    app.update();
    let input = app.world().resource::<LegacyAvatarActionInput>();
    assert!(input.primary_held && !input.primary_just_pressed);

    *app.world_mut().get_mut::<CursorOptions>(window).unwrap() = CursorOptions::default();
    // Hit testing resumes when the pointer becomes visible; the locked
    // frame deliberately cleared its stale UI interaction.
    *app.world_mut().get_mut::<Interaction>(ui).unwrap() = Interaction::Pressed;
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .reset_all();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    assert_eq!(
        *app.world().resource::<LegacyAvatarActionInput>(),
        LegacyAvatarActionInput::default()
    );

    // Closing a UI in Update must not turn its consumed click into a shot.
    app.add_systems(
        Update,
        (|mut cursors: Query<&mut CursorOptions>| {
            for mut cursor in &mut cursors {
                cursor.visible = false;
                cursor.grab_mode = CursorGrabMode::Locked;
            }
        })
        .before(read_legacy_avatar_action_input),
    );
    app.update();
    assert_eq!(
        *app.world().resource::<LegacyAvatarActionInput>(),
        LegacyAvatarActionInput::default()
    );
    app.update();
    assert_eq!(
        *app.world().resource::<LegacyAvatarActionInput>(),
        LegacyAvatarActionInput::default()
    );
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    assert!(
        app.world()
            .resource::<LegacyAvatarActionInput>()
            .primary_held
    );
}

#[test]
fn hotkeys_combat_and_nano_slots_follow_committed_primary_and_alternate_bindings() {
    use ffone_client::avatar_action::LegacyAvatarActionInput;
    let mut app = App::new();
    app.add_plugins(ffone_client::input_focus::GameplayPointerCapturePlugin)
        .init_resource::<OptionProductionRuntime>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<LegacyAvatarActionInput>()
        .add_systems(Update, read_configured_avatar_action_input);
    for (action, original, replacement) in [
        (
            LegacyOptionAction::Fire1,
            KeyCode::KeyZ,
            LegacyPhysicalKey::V,
        ),
        (
            LegacyOptionAction::Fire2,
            KeyCode::KeyX,
            LegacyPhysicalKey::C,
        ),
        (
            LegacyOptionAction::WeaponChange,
            KeyCode::Tab,
            LegacyPhysicalKey::F,
        ),
    ] {
        {
            let mut options = app.world_mut().resource_mut::<OptionProductionRuntime>();
            let row = options
                .input
                .mappings
                .iter_mut()
                .find(|row| row.action == action)
                .unwrap();
            row.primary = LegacyInputBinding::Key(replacement);
            row.alternate = LegacyInputBinding::Key(LegacyPhysicalKey::Mouse1);
        }
        press(&mut app, &[original]);
        assert_eq!(
            *app.world().resource::<LegacyAvatarActionInput>(),
            LegacyAvatarActionInput::default()
        );
        for key in [
            Some(match replacement {
                LegacyPhysicalKey::V => KeyCode::KeyV,
                LegacyPhysicalKey::C => KeyCode::KeyC,
                _ => KeyCode::KeyF,
            }),
            None,
        ] {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .reset_all();
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .reset_all();
            if let Some(key) = key {
                app.world_mut()
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .press(key);
            } else {
                app.world_mut()
                    .resource_mut::<ButtonInput<MouseButton>>()
                    .press(MouseButton::Right);
            }
            app.update();
            let input = app.world().resource::<LegacyAvatarActionInput>();
            assert!(match action {
                LegacyOptionAction::Fire1 => input.primary_held && input.primary_just_pressed,
                LegacyOptionAction::Fire2 => input.nano_just_pressed,
                _ => input.weapon_cycle_just_pressed,
            });
        }
        *app.world_mut().resource_mut::<OptionProductionRuntime>() = default();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .reset_all();
    }
    let mut settings = InputSettings::default();
    let row = settings
        .mappings
        .iter_mut()
        .find(|row| row.action == LegacyOptionAction::Nano2)
        .unwrap();
    row.primary = LegacyInputBinding::Key(LegacyPhysicalKey::V);
    let mut keys = ButtonInput::default();
    keys.press(KeyCode::KeyV);
    assert_eq!(
        requested_world_nano_slot(&settings, &keys, &default()),
        Some(1)
    );
    keys.reset_all();
    keys.press(KeyCode::Digit2);
    assert_eq!(
        requested_world_nano_slot(&settings, &keys, &default()),
        None
    );
}

#[test]
fn hotkeys_nano_charge_encodes_request_rate_limits_and_applies_only_server_reply() {
    use ffone_protocol::wire_0104::ChargeNanoStaminaReply0104;
    let mut status = RuntimeStatus::default();
    status.player_id = Some(77);
    status.nano_battery = 200;
    status.nano_slots[0] = RuntimeNanoSlot {
        nano_id: Some(1),
        skill_id: 1,
        stamina: 30,
        active: true,
    };
    let context = LegacyAvatarActionContext {
        ready_for_play: true,
        input_enabled: true,
        ..default()
    };
    let mut last = 0.0;
    let request = nano_charge_request(&status, &context, 1.0, &mut last).unwrap();
    assert_eq!(
        request.packet_type(),
        packet::P_CL2FE_REQ_CHARGE_NANO_STAMINA
    );
    assert_eq!(request.payload(), &77_i32.to_le_bytes());
    assert_eq!(status.nano_slots[0].stamina, 30);
    assert_eq!(status.nano_battery, 200);
    assert!(nano_charge_request(&status, &context, 1.99, &mut last).is_none());
    let blocked = LegacyAvatarActionContext {
        time_buff_condition: 5,
        ..context.clone()
    };
    assert!(nano_charge_request(&status, &blocked, 2.0, &mut last).is_none());
    assert!(nano_charge_request(&status, &context, 2.0, &mut last).is_some());
    let mut bank = NanoFreeTuningBank0104::default();
    bank.apply_create_success(ffone_protocol::PcNanoCreateSuccess0104 {
        nano: ffone_protocol::Nano0104 {
            id: 1,
            skill_id: 1,
            stamina: 30,
        },
        fusion_matter: 0,
        quest_item_slot: -1,
        quest_item: ItemBase0104 {
            item_type: 0,
            item_id: 0,
            option: 0,
            time_limit: 0,
        },
        player_level: 1,
    })
    .unwrap();
    let mut audio = GameplayAudioRuntime::default();
    let mut frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_CHARGE_NANO_STAMINA,
        flags: 0,
        checksum: 0,
        payload: ChargeNanoStaminaReply0104 {
            battery_n: 80,
            nano_id: 1,
            nano_stamina: 150,
        }
        .encode(),
    };
    apply_nano_charge_frame(&frame, &mut status, &mut bank, &mut audio).unwrap();
    assert_eq!(status.nano_battery, 80);
    assert_eq!(status.nano_slots[0].stamina, 150);
    assert_eq!(bank.entries()[1].stamina, 150);
    frame.payload.pop();
    assert!(apply_nano_charge_frame(&frame, &mut status, &mut bank, &mut audio).is_err());
    assert_eq!(status.nano_battery, 80);
    assert_eq!(bank.entries()[1].stamina, 150);
}

fn app() -> App {
    let mut app = App::new();
    let mut mission = MissionUiModel::default();
    mission.enabled = true;
    app.insert_resource(State::new(ClientState::World))
        .init_resource::<OptionProductionRuntime>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<GameplayUiModel>()
        .init_resource::<GameGuideUiModel>()
        .init_resource::<GameplayMenuTransition>()
        .init_resource::<UserEquipUiState>()
        .init_resource::<UserEquipModalState>()
        .init_resource::<UserEquipUiOutbox>()
        .init_resource::<GameplayUiOutbox>()
        .init_resource::<GameplayUiAudioOutbox>()
        .init_resource::<LauncherUiModel>()
        .insert_resource(mission)
        .add_systems(Update, route_world_ui_shortcuts);
    app
}

fn press(app: &mut App, keys: &[KeyCode]) {
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.reset_all();
    for key in keys {
        input.press(*key);
    }
    app.update();
}

#[test]
fn hotkeys_open_inventory_nano_book_and_email_with_configured_priority() {
    for (key, source) in [
        (KeyCode::KeyI, UserEquipOpenSource::InventoryShortcut),
        (KeyCode::KeyN, UserEquipOpenSource::NanoBookShortcut),
    ] {
        let mut app = app();
        press(&mut app, &[key, KeyCode::KeyJ, KeyCode::Escape]);
        let actions: Vec<_> = app
            .world_mut()
            .resource_mut::<GameplayUiOutbox>()
            .drain()
            .collect();
        assert_eq!(
            actions,
            vec![GameplayUiAction::OpenUserEquipItemMode { source }]
        );
        assert!(
            !app.world()
                .resource::<ButtonInput<KeyCode>>()
                .just_pressed(KeyCode::Escape)
        );
        let mut equip = UserEquipUiState::default();
        equip.open_from(source);
        assert_eq!(
            equip.mode(),
            if key == KeyCode::KeyI {
                UserEquipMode::Item
            } else {
                UserEquipMode::Nano
            }
        );
    }
    let mut app = app();
    press(&mut app, &[KeyCode::KeyP]);
    assert_eq!(
        app.world_mut()
            .resource_mut::<GameplayUiOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![GameplayUiAction::OpenEmailFromNanocom]
    );
}

#[test]
fn hotkeys_journal_opens_and_closes_without_opening_the_nanocom_menu() {
    let mut app = app();
    press(&mut app, &[KeyCode::KeyJ]);
    let mission = app.world().resource::<MissionUiModel>();
    assert!(matches!(mission.journal, MissionJournalUi::Other(_)));
    assert!(!mission.nanocom_main_menu_visible);
    press(&mut app, &[KeyCode::KeyJ, KeyCode::Escape]);
    assert!(matches!(
        app.world().resource::<MissionUiModel>().journal,
        MissionJournalUi::Hidden
    ));
    assert!(
        !app.world()
            .resource::<ButtonInput<KeyCode>>()
            .just_pressed(KeyCode::Escape)
    );
}

#[test]
fn gamepad_escape_closes_npc_dialogue_through_the_normal_close_intent() {
    let mut app = app();
    app.world_mut()
        .resource_mut::<MissionUiModel>()
        .show_npc_interaction(NpcInteractionUi {
            npc_id: 9001,
            npc_type: 650,
            name: "NPC".into(),
            available_missions: vec![],
            completed_missions: vec![],
            warp: None,
            services: vec![],
        });
    press(&mut app, &[KeyCode::Escape]);
    assert!(
        !app.world()
            .resource::<MissionUiModel>()
            .npc_icon_mode_visible
    );
    assert_eq!(
        app.world_mut()
            .resource_mut::<GameplayUiOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![GameplayUiAction::NpcIconClose { npc_id: 9001 }]
    );
    assert!(
        !app.world()
            .resource::<ButtonInput<KeyCode>>()
            .just_pressed(KeyCode::Escape)
    );
}

#[test]
fn hotkeys_active_book_switches_tab_then_requests_arbitrated_close() {
    let mut app = app();
    {
        let mut equip = app.world_mut().resource_mut::<UserEquipUiState>();
        equip.open_from(UserEquipOpenSource::NanoBookShortcut);
    }
    press(&mut app, &[KeyCode::KeyI]);
    let action = app
        .world_mut()
        .resource_mut::<UserEquipUiOutbox>()
        .drain()
        .next()
        .unwrap();
    assert_eq!(
        action,
        UserEquipUiAction::SelectTab {
            mode: UserEquipMode::Item
        }
    );
    let outcome = apply_user_equip_ui_action(
        &mut app.world_mut().resource_mut::<UserEquipUiState>(),
        &mut UserEquipModalState::default(),
        action,
    );
    assert_eq!(
        outcome,
        UserEquipUiActionOutcome::TabSelected(UserEquipMode::Item)
    );
    press(&mut app, &[KeyCode::KeyI]);
    assert_eq!(
        app.world_mut()
            .resource_mut::<UserEquipUiOutbox>()
            .drain()
            .next(),
        Some(UserEquipUiAction::RequestClose {
            source: UserEquipCloseSource::ActiveTabHotkey
        })
    );
}

#[test]
fn legacy_help_key_is_inert_but_custom_help_binding_and_escape_close_work() {
    let mut app = app();
    press(&mut app, &[KeyCode::KeyH]);
    assert!(app.world().resource::<GameplayUiOutbox>().is_empty());
    // A loaded v1 snapshot can still contain the old H default. Keep the
    // saved row, but suppress the physical H route in both keyboard layouts.
    {
        let mut runtime = app.world_mut().resource_mut::<OptionProductionRuntime>();
        let help = runtime
            .input
            .mappings
            .iter_mut()
            .find(|row| row.action == LegacyOptionAction::Help)
            .unwrap();
        help.primary = LegacyInputBinding::Key(LegacyPhysicalKey::H);
    }
    press(&mut app, &[KeyCode::KeyH]);
    assert!(app.world().resource::<GameplayUiOutbox>().is_empty());
    {
        let mut runtime = app.world_mut().resource_mut::<OptionProductionRuntime>();
        let help = runtime
            .input
            .mappings
            .iter_mut()
            .find(|row| row.action == LegacyOptionAction::Help)
            .unwrap();
        help.alternate = LegacyInputBinding::Key(LegacyPhysicalKey::F);
        runtime
            .input
            .mappings
            .iter_mut()
            .find(|row| row.action == LegacyOptionAction::Skill2)
            .unwrap()
            .primary = LegacyInputBinding::Unbound;
    }
    press(&mut app, &[KeyCode::KeyF]);
    assert_eq!(
        app.world_mut()
            .resource_mut::<GameplayUiOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![GameplayUiAction::OpenGameGuideFromNanocom]
    );
    app.world_mut()
        .resource_mut::<GameGuideUiModel>()
        .open_from_nanocom();
    press(&mut app, &[KeyCode::Escape]);
    assert!(!app.world().resource::<GameGuideUiModel>().modal_active());
    app.world_mut()
        .resource_mut::<UserEquipUiState>()
        .open_item_mode();
    app.world_mut()
        .resource_mut::<UserEquipModalState>()
        .send_pending = true;
    press(&mut app, &[KeyCode::KeyI]);
    assert!(app.world().resource::<UserEquipUiOutbox>().is_empty());
}

#[test]
fn hotkeys_respect_rebinding_chat_tutorial_modal_and_menu_transition() {
    let mut app = app();
    {
        let mut runtime = app.world_mut().resource_mut::<OptionProductionRuntime>();
        let mapping = runtime
            .input
            .mappings
            .iter_mut()
            .find(|row| row.action == LegacyOptionAction::Inventory)
            .unwrap();
        mapping.primary = LegacyInputBinding::Key(LegacyPhysicalKey::V);
        mapping.alternate = LegacyInputBinding::Key(LegacyPhysicalKey::F);
    }
    press(&mut app, &[KeyCode::KeyI]);
    assert!(app.world().resource::<GameplayUiOutbox>().is_empty());
    for key in [KeyCode::KeyV, KeyCode::KeyF] {
        press(&mut app, &[key]);
        assert_eq!(
            app.world_mut()
                .resource_mut::<GameplayUiOutbox>()
                .drain()
                .count(),
            1
        );
    }
    {
        let mut hud = app.world_mut().resource_mut::<GameplayUiModel>();
        hud.visible = true;
        hud.chat.active = true;
        hud.chat.input_enabled = true;
    }
    press(&mut app, &[KeyCode::KeyV]);
    assert!(app.world().resource::<GameplayUiOutbox>().is_empty());
    app.world_mut()
        .resource_mut::<GameplayUiModel>()
        .chat
        .active = false;
    app.insert_resource(State::new(ClientState::Tutorial));
    press(&mut app, &[KeyCode::KeyV]);
    assert!(app.world().resource::<GameplayUiOutbox>().is_empty());
    app.insert_resource(State::new(ClientState::World));
    app.world_mut()
        .resource_mut::<GameplayMenuTransition>()
        .remaining = 0.5;
    press(&mut app, &[KeyCode::KeyV]);
    assert!(app.world().resource::<GameplayUiOutbox>().is_empty());
    app.world_mut()
        .resource_mut::<GameplayMenuTransition>()
        .remaining = 0.0;
    app.insert_resource(OptionUiModel::default());
    app.world_mut().resource_mut::<OptionUiModel>().visible = true;
    press(&mut app, &[KeyCode::KeyV]);
    assert!(app.world().resource::<GameplayUiOutbox>().is_empty());
}
