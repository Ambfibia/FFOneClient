use crate::app::gameplay_ui_actions::*;

pub(super) fn handle_modes(
    action: ModeAction,
    gameplay_modal: bool,
    context: &mut WorldGameplayActionContext<'_, '_>,
) {
    let WorldGameplayActionContext {
        gameplay_ui,
        buddy_ui,
        normal_warp,
        mission_ui,
        nanocom_messages,
        content,
        guide_ui,
        game_guide_ui,
        modal_owners,
        guide_runtime,
        guide_production,
        user_equip_ui,
        runtime,
        ..
    } = context;
    for action in std::iter::once(action) {
        match action {
            ModeAction::OpenNanocomMenu => {
                if gameplay_modal || user_equip_ui.is_active() {
                    if user_equip_ui.is_active() {
                        let mut ignored = GameplayUiOutbox::default();
                        mission_ui.close_nanocom_menu(&mut ignored);
                        nanocom_messages.set_expanded(false);
                    }
                    continue;
                }
                let mut ignored = GameplayUiOutbox::default();
                mission_ui.open_nanocom_menu(&mut ignored);
                nanocom_messages.set_expanded(mission_ui.nanocom_main_menu_visible);
            }
            ModeAction::CloseNanocomMenu => {
                let mut ignored = GameplayUiOutbox::default();
                mission_ui.close_nanocom_menu(&mut ignored);
                nanocom_messages.set_expanded(mission_ui.nanocom_main_menu_visible);
                gameplay_ui.chat.close_nanocom_menu();
            }
            ModeAction::OpenUserEquipItemMode { source } => {
                if gameplay_modal || user_equip_ui.is_active() || runtime.player_id.is_none() {
                    continue;
                }
                let mut ignored = GameplayUiOutbox::default();
                mission_ui.close_nanocom_menu(&mut ignored);
                nanocom_messages.set_expanded(false);
                gameplay_ui.chat.active = false;
                gameplay_ui.chat.input.clear();
                user_equip_ui.open_from(source);
            }
            ModeAction::OpenOptionFromNanocomSettings => {
                // `cnGUINanocom.OnOption` requests mode 12 before its shared
                // menu-close event. The model has already latched that close,
                // so the production boundary only mirrors it to the expanded
                // Nanocom owner before opening OptionMode.
                nanocom_messages.set_expanded(false);
                let blocked = gameplay_modal
                    || user_equip_ui.is_active()
                    || modal_owners.shell.system_messages.is_popup()
                    || mission_ui.system_popup_active()
                    || modal_owners.shell.quit_model.visible
                    || modal_owners.shell.quit_runtime.is_waiting_for_server()
                    || modal_owners.shell.resurrect_ui.visible;
                if blocked {
                    runtime.message =
                        "NanoCom SETTINGS ignored while another clean modal owns input".to_owned();
                    continue;
                }
                if modal_owners.modes.option_asset_gate.failed {
                    runtime.message =
                        "NanoCom SETTINGS blocked by missing exact OptionMode assets".to_owned();
                    continue;
                }
                if !modal_owners.modes.option_asset_gate.ready {
                    runtime.message =
                        "NanoCom SETTINGS is waiting for exact OptionMode assets".to_owned();
                    continue;
                }
                if !modal_owners
                    .modes
                    .option_runtime
                    .initialized_from_live_runtime
                {
                    runtime.message =
                        "NanoCom SETTINGS is waiting for live window and camera owners".to_owned();
                    continue;
                }
                gameplay_ui.chat.input_enabled = false;
                gameplay_ui.chat.active = false;
                gameplay_ui.chat.input.clear();
                modal_owners.modes.option_model.open(
                    modal_owners.modes.option_runtime.options.clone(),
                    modal_owners.modes.option_runtime.input.clone(),
                    OptionOpenAudioRoute {
                        main_game_transition: true,
                        inventory_transition: false,
                    },
                    &mut modal_owners.modes.option_outbox,
                );
            }
            ModeAction::OpenEmailFromNanocom => {
                nanocom_messages.set_expanded(false);
                let another_modal_owns_input = gameplay_modal
                    || buddy_ui.add_dialog_open()
                    || mission_ui.chat_input_blocked()
                    || user_equip_ui.is_active()
                    || modal_owners.shell.system_messages.is_popup()
                    || mission_ui.system_popup_active()
                    || modal_owners.shell.quit_model.visible
                    || modal_owners.shell.quit_runtime.is_waiting_for_server()
                    || modal_owners.shell.resurrect_ui.visible;
                if another_modal_owns_input {
                    runtime.message =
                        "NanoCom E-MAIL ignored while another clean modal owns input".to_owned();
                    continue;
                }
                let Some(inventory) = normal_warp.inventory.snapshot() else {
                    runtime.message =
                        "NanoCom E-MAIL is waiting for authoritative inventory".to_owned();
                    continue;
                };
                let inventory = email_inventory_authority_0104(inventory);
                let buddies = match email_buddies_from_buddy_ui_0104(&buddy_ui) {
                    Ok(buddies) => buddies,
                    Err(error) => {
                        runtime.message =
                            format!("NanoCom E-MAIL buddy authority rejected: {error}");
                        continue;
                    }
                };
                let guide_messages = match email_guide_messages_0104(
                    &modal_owners.email.catalog,
                    &content,
                    &guide_runtime,
                    active_email_task_ids_0104(
                        &modal_owners.mission.world_mission,
                        &modal_owners.mission.tutorial,
                    ),
                    available_email_task_ids_0104(
                        &modal_owners.email.catalog,
                        &content,
                        &modal_owners.mission.world_mission,
                        &guide_runtime,
                        &modal_owners.email.nano_bank,
                        &normal_warp.inventory,
                        i32::from(runtime.player_level),
                    ),
                ) {
                    Ok(messages) => messages,
                    Err(error) => {
                        runtime.message =
                            format!("NanoCom E-MAIL guide authority rejected: {error}");
                        continue;
                    }
                };
                let player = match email_player_authority_0104(&runtime) {
                    Ok(player) => player,
                    Err(error) => {
                        runtime.message = error;
                        continue;
                    }
                };
                let cursor_was_locked = match modal_owners.shell.cursors.single_mut() {
                    Ok(cursor) => cursor.grab_mode != CursorGrabMode::None,
                    Err(_) => {
                        runtime.message =
                            "NanoCom E-MAIL has no unique production cursor owner".to_owned();
                        continue;
                    }
                };
                let context = EmailOpenContext0104 {
                    player,
                    cursor_was_locked,
                    item_policy: EmailItemFeaturePolicy0104 {
                        combine_enabled: true,
                        korean_enchant_enabled: false,
                    },
                };
                match modal_owners.email.runtime.open(
                    context,
                    guide_messages,
                    buddies,
                    &inventory,
                    &*modal_owners.email.catalog,
                    &mut modal_owners.email.model,
                    &mut modal_owners.email.actions,
                    &mut modal_owners.email.audio,
                    &modal_owners.email.network,
                    &modal_owners.email.inbox,
                    &modal_owners.email.transport,
                ) {
                    Ok(mut output) => {
                        output.audio.push(EmailUiAudioCue::Open);
                        modal_owners.email.shell.cursor_was_locked = Some(cursor_was_locked);
                        modal_owners.email.shell.lease = modal_owners.email.runtime.mode_lease();
                        modal_owners.email.shell.last_inventory = Some(inventory);
                        modal_owners.email.shell.push_output(output);
                        gameplay_ui.chat.input_enabled = false;
                        gameplay_ui.chat.active = false;
                        gameplay_ui.chat.input.clear();
                        runtime.message = "EmailMode 18 opened from NanoCom event 2/3".to_owned();
                    }
                    Err(error) => {
                        runtime.message = format!("NanoCom E-MAIL open rejected: {error}")
                    }
                }
            }
            ModeAction::OpenWorldMapFromNanocom => {
                // `OnWorldMap` requests mode 15 before the shared menu-close
                // event. Mirror the already-latched model close immediately;
                // the following CloseNanocomMenu action remains idempotent.
                nanocom_messages.set_expanded(false);
                let another_modal_owns_input = gameplay_modal
                    || buddy_ui.add_dialog_open()
                    || mission_ui.chat_input_blocked()
                    || user_equip_ui.is_active()
                    || modal_owners.shell.system_messages.is_popup()
                    || modal_owners.shell.quit_model.visible
                    || modal_owners.shell.quit_runtime.is_waiting_for_server()
                    || modal_owners.shell.resurrect_ui.visible;
                if another_modal_owns_input {
                    runtime.message =
                        "NanoCom WorldMap ignored while another clean modal owns input".to_owned();
                    continue;
                }
                let player = modal_owners.shell.players.single_mut().ok().and_then(
                    |(transform, controller, collider_pending)| {
                        collider_pending
                            .is_none()
                            .then(|| world_map_player_from_native(transform, &controller))
                    },
                );
                match open_world_map_for_ready_player(
                    &mut modal_owners.modes.world_map,
                    &modal_owners.modes.world_map_asset_status,
                    player,
                    runtime.map_number,
                    &modal_owners.race.catalog,
                ) {
                    Ok(()) => {
                        gameplay_ui.chat.active = false;
                        gameplay_ui.chat.input.clear();
                    }
                    Err(message) => runtime.message = message,
                }
            }
            ModeAction::OpenGameGuideFromNanocom => {
                let another_modal_owns_input = gameplay_modal
                    || buddy_ui.add_dialog_open()
                    || mission_ui.chat_input_blocked()
                    || user_equip_ui.is_active()
                    || modal_owners.shell.system_messages.is_popup()
                    || modal_owners.shell.quit_model.visible
                    || modal_owners.shell.quit_runtime.is_waiting_for_server()
                    || modal_owners.shell.resurrect_ui.visible
                    || guide_production.active_source_npc.is_some()
                    || guide_production.special_state_active;
                if another_modal_owns_input {
                    runtime.message =
                        "NanoCom Guide ignored while another clean modal owns input".to_owned();
                    continue;
                }
                // `cnGUINanocom.OnGuide` sends `(2, 18)` to GameFrame, whose
                // `ReceiveGameGuidePressed` opens `cnHelpMode.InitMode(1, 0)`.
                // Mode 20 / `cnGuideMode` is the unrelated mentor changer.
                game_guide_ui.open_from_nanocom();
                modal_owners
                    .audio
                    .runtime
                    .queue_gameplay_ui_sound("Open_Screen");
                gameplay_ui.chat.active = false;
                gameplay_ui.chat.input.clear();
                debug_assert!(!guide_ui.visible);
                debug_assert!(guide_production.active_source_npc.is_none());
                debug_assert!(!guide_production.special_state_active);
                runtime.message = "Game Guide opened from NanoCom event 2/18".to_owned();
            }
            ModeAction::OpenQuitFromNanocom => {
                let another_modal_owns_input = gameplay_modal
                    || buddy_ui.add_dialog_open()
                    || mission_ui.chat_input_blocked()
                    || user_equip_ui.is_active()
                    || modal_owners.shell.system_messages.is_popup()
                    || modal_owners.shell.resurrect_ui.visible;
                if open_quit_menu_from_nanocom(
                    &mut modal_owners.shell.quit_model,
                    &modal_owners.shell.quit_runtime,
                    another_modal_owns_input,
                ) {
                    gameplay_ui.chat.input_enabled = false;
                    gameplay_ui.chat.active = false;
                    gameplay_ui.chat.input.clear();
                    runtime.message = "QuitMenu opened from NanoCom mode 23".to_owned();
                } else {
                    runtime.message =
                        "NanoCom Quit ignored while another clean modal owns input".to_owned();
                }
            }
        }
    }
}
