use crate::app::gameplay_ui_actions::*;

pub(super) fn handle_chat(
    action: ChatAction,
    gameplay_modal: bool,
    context: &mut WorldGameplayActionContext<'_, '_>,
) {
    if !gameplay_modal
        && let ChatAction::SendChat(message) = &action
        && gm_runtime::handle(message, context)
    {
        return;
    }
    let WorldGameplayActionContext {
        gameplay_ui,
        buddy_ui,
        normal_warp,
        content,
        modal_owners,
        bridge,
        vehicle,
        runtime,
        ..
    } = context;
    for action in std::iter::once(action) {
        match action {
            ChatAction::SelectChatChannel(channel) => {
                if gameplay_modal {
                    continue;
                }
                // `CnGuiChat.CheckGroup` immediately returns to ALL when the
                // local player is not in a multi-PC group.
                let selected = if channel == ChatChannel::Group
                    && normal_warp.group_ui.pc_members.len() <= 1
                {
                    ChatChannel::All
                } else {
                    channel
                };
                gameplay_ui.chat.selected = selected;
                runtime.chat.world_chat_alerts[selected.index()] = false;
            }
            ChatAction::SendChat(message) => {
                if gameplay_modal || message.is_empty() {
                    continue;
                }
                if is_cashmall_hidden_chat_command_0104(&message, runtime.user_level) {
                    let previous_cursor_locked = match modal_owners.shell.cursors.single_mut() {
                        Ok(cursor) => cursor.grab_mode == CursorGrabMode::Locked,
                        Err(_) => {
                            runtime.message =
                                "Cashmall hidden command requires one authoritative primary cursor"
                                    .to_owned();
                            continue;
                        }
                    };
                    modal_owners.commerce.cashmall_state.open_from(
                        CashmallOpenSource0104::HiddenChatCommand,
                        previous_cursor_locked,
                        &mut modal_owners.commerce.cashmall_outbox,
                    );
                    gameplay_ui.chat.input_enabled = false;
                    gameplay_ui.chat.active = false;
                    gameplay_ui.chat.input.clear();
                    continue;
                }
                if let Some(command) = parse_local_flight_command(&message) {
                    let (key, fallback) = match command {
                        FlightCommand::Invalid => {
                            ("ui.chat.command.fly.usage", "Usage: /fly [on|off]")
                        }
                        FlightCommand::Toggle | FlightCommand::Set(_) => {
                            let Ok((_, mut controller, _)) =
                                modal_owners.shell.players.single_mut()
                            else {
                                continue;
                            };
                            let enabled = match command {
                                FlightCommand::Toggle => !controller.flight_enabled(),
                                FlightCommand::Set(enabled) => enabled,
                                FlightCommand::Invalid => unreachable!(),
                            };
                            controller.set_flight_enabled(enabled);
                            if enabled {
                                (
                                    "ui.chat.command.fly.enabled",
                                    "Flight enabled. Space: up, Shift: down.",
                                )
                            } else {
                                ("ui.chat.command.fly.disabled", "Flight disabled.")
                            }
                        }
                    };
                    let feedback = modal_owners.shell.localization.text(
                        &modal_owners.shell.language,
                        &LocalizedText::new(key, fallback),
                    );
                    runtime.message.clone_from(&feedback);
                    push_world_chat_line(runtime, ChatLineUi::normal(feedback));
                    continue;
                }
                if let Some(command) = parse_gm_speed_command(&message) {
                    let feedback = match build_gm_speed_request(
                        command,
                        runtime.user_level,
                        runtime.player_id,
                    ) {
                        Err(GmSpeedRequestError::Usage) => LocalizedText::new(
                            "ui.chat.command.speed.usage",
                            "Usage: /speed <value>",
                        ),
                        Err(GmSpeedRequestError::AccessDenied) => LocalizedText::new(
                            "ui.chat.command.speed.denied",
                            "/speed requires GM access (accountLevel <= 50).",
                        ),
                        Err(GmSpeedRequestError::MissingPlayer) => LocalizedText::new(
                            "ui.chat.command.speed.failed",
                            "Could not send /speed: {error}",
                        )
                        .with_arg("error", "the local runtime player ID is unavailable"),
                        Ok(request) => {
                            let speed = request.value;
                            match bridge.send(NetworkCommand::SetGmValue(request)) {
                                Ok(()) => LocalizedText::new(
                                    "ui.chat.command.speed.requested",
                                    "Speed change requested: {speed}.",
                                )
                                .with_arg("speed", speed.to_string()),
                                Err(error) => LocalizedText::new(
                                    "ui.chat.command.speed.failed",
                                    "Could not send /speed: {error}",
                                )
                                .with_arg("error", error),
                            }
                        }
                    };
                    let feedback = modal_owners
                        .shell
                        .localization
                        .text(&modal_owners.shell.language, &feedback);
                    runtime.message.clone_from(&feedback);
                    push_world_chat_line(runtime, ChatLineUi::normal(feedback));
                    continue;
                }
                if let Some(command) = parse_gm_nano_command(&message) {
                    let feedback =
                        match build_gm_nano_request(command, runtime.user_level, |nano_id| {
                            content.gameplay_nano(nano_id).is_some()
                        }) {
                            Err(GmNanoRequestError::Usage) => LocalizedText::new(
                                "ui.chat.command.nano.usage",
                                "Usage: /nano <nano-id>",
                            ),
                            Err(GmNanoRequestError::AccessDenied) => LocalizedText::new(
                                "ui.chat.command.nano.denied",
                                "/nano requires GM access (accountLevel <= 50).",
                            ),
                            Err(GmNanoRequestError::UnknownNano(nano_id)) => LocalizedText::new(
                                "ui.chat.command.nano.unknown",
                                "Unknown Nano ID: {nano_id}.",
                            )
                            .with_arg("nano_id", nano_id.to_string()),
                            Ok(request) => {
                                let nano_id = request.nano_id;
                                let request = ffone_protocol::RegisteredGameplayRequest0104::new(
                                    packet::P_CL2FE_REQ_PC_GIVE_NANO,
                                    request.encode(),
                                )
                                .expect("the clean /nano request has the registered 0104 ABI");
                                match bridge
                                    .send(NetworkCommand::SendRegisteredGameplay0104(request))
                                {
                                    Ok(()) => LocalizedText::new(
                                        "ui.chat.command.nano.requested",
                                        "Nano requested: {nano_id}.",
                                    )
                                    .with_arg("nano_id", nano_id.to_string()),
                                    Err(error) => LocalizedText::new(
                                        "ui.chat.command.nano.failed",
                                        "Could not send /nano: {error}",
                                    )
                                    .with_arg("error", error),
                                }
                            }
                        };
                    let feedback = modal_owners
                        .shell
                        .localization
                        .text(&modal_owners.shell.language, &feedback);
                    runtime.message.clone_from(&feedback);
                    push_world_chat_line(runtime, ChatLineUi::normal(feedback));
                    continue;
                }
                if let Some(usage) = message
                    .split_whitespace()
                    .next()
                    .and_then(gm_commands::usage)
                {
                    let result = gm_commands::request(
                        &message,
                        runtime.user_level,
                        runtime.player_id,
                        |quest, id| {
                            let items: &[ItemBase0104] = if quest {
                                normal_warp.inventory.quest_inventory.as_ref()?.as_slice()
                            } else {
                                normal_warp
                                    .inventory
                                    .snapshot
                                    .as_ref()?
                                    .inventory()
                                    .as_slice()
                            };
                            if quest {
                                items
                                    .iter()
                                    .position(|item| item.item_id == id)
                                    .or_else(|| items.iter().position(|item| item.option == 0))
                            } else {
                                items.iter().position(|item| item.item_id <= 0)
                            }
                        },
                    );
                    let feedback = match result {
                        Ok(request) => {
                            match bridge.send(NetworkCommand::SendRegisteredGameplay0104(request)) {
                                Ok(()) => LocalizedText::new(
                                    "ui.chat.command.gm.requested",
                                    "Command sent: {command}",
                                )
                                .with_arg("command", message.clone()),
                                Err(error) => LocalizedText::new(
                                    "ui.chat.command.gm.failed",
                                    "Could not send command: {error}",
                                )
                                .with_arg("error", error),
                            }
                        }
                        Err(gm_commands::Error::Usage) => {
                            LocalizedText::new("ui.chat.command.gm.usage", "Usage: {usage}")
                                .with_arg("usage", usage)
                        }
                        Err(gm_commands::Error::Denied) => LocalizedText::new(
                            "ui.chat.command.gm.denied",
                            "This command requires GM access (accountLevel <= 50).",
                        ),
                        Err(gm_commands::Error::MissingPlayer) => LocalizedText::new(
                            "ui.chat.command.gm.player_missing",
                            "The local player is not ready.",
                        ),
                        Err(gm_commands::Error::Full) => LocalizedText::new(
                            "ui.chat.command.gm.inventory_full",
                            "No free inventory slot is available.",
                        ),
                    };
                    let feedback = modal_owners
                        .shell
                        .localization
                        .text(&modal_owners.shell.language, &feedback);
                    runtime.message.clone_from(&feedback);
                    push_world_chat_line(runtime, ChatLineUi::normal(feedback));
                    continue;
                }
                let server_command = is_server_chat_command_0104(&message);
                if !runtime.free_chat && !server_command {
                    continue;
                }
                let message = match FixedUtf16::<128>::from_str(&message) {
                    Ok(message) => message,
                    Err(error) => {
                        runtime.message = format!("FreeChat cannot be encoded: {error}");
                        continue;
                    }
                };
                // Command replies are retained in ALL/GROUP, never BUDDY.
                // Show the receiving history even when the command was entered
                // without a selected friend in the Buddy pane.
                if server_command && gameplay_ui.chat.selected == ChatChannel::Buddy {
                    gameplay_ui.chat.selected = ChatChannel::All;
                    runtime.chat.world_chat_alerts[ChatChannel::All.index()] = false;
                }
                // OpenFusion consumes slash commands before its FreeChat mute
                // check. Always use the normal FreeChat packet so commands do
                // not depend on the selected group/buddy tab or target.
                let command = match (server_command, gameplay_ui.chat.selected) {
                    (true, _) => NetworkCommand::SendFreeChat(FreeChatRequest0104 {
                        message,
                        emote_code: 0,
                    }),
                    (false, ChatChannel::All) => {
                        NetworkCommand::SendFreeChat(FreeChatRequest0104 {
                            message,
                            emote_code: 0,
                        })
                    }
                    (false, ChatChannel::Group) if normal_warp.group_ui.pc_members.len() > 1 => {
                        NetworkCommand::SendAllGroupFreeChat(AllGroupFreeChatRequest0104 {
                            message,
                            emote_code: 0,
                        })
                    }
                    (false, ChatChannel::Group) => {
                        gameplay_ui.chat.selected = ChatChannel::All;
                        runtime.message =
                            "Group chat requires a multi-player group; returned to ALL".to_owned();
                        continue;
                    }
                    (false, ChatChannel::Buddy) => {
                        let Some(target) = buddy_ui.selected_target() else {
                            runtime.message = "Buddy FreeChat requires a selected Buddy".to_owned();
                            continue;
                        };
                        NetworkCommand::SendBuddyFreeChat(BuddyFreeChatRequest0104 {
                            message,
                            emote_code: 0,
                            buddy_pc_uid: target.pc_uid,
                            buddy_slot: target.slot as i8,
                        })
                    }
                };
                if let Err(error) = bridge.send(command) {
                    runtime.message = error;
                }
            }
            ChatAction::ToggleMenuChat => {
                if !gameplay_modal && gameplay_ui.chat.input_enabled {
                    gameplay_ui
                        .chat
                        .quick_menu
                        .toggle(QuickChatMenuMode::MenuChat);
                }
            }
            ChatAction::ToggleEmotes => {
                if !gameplay_modal && gameplay_ui.chat.input_enabled {
                    gameplay_ui
                        .chat
                        .quick_menu
                        .toggle(QuickChatMenuMode::Emotes);
                }
            }
            ChatAction::SelectQuickChatItem(item) => {
                if gameplay_modal || !gameplay_ui.chat.input_enabled {
                    continue;
                }
                let mode = gameplay_ui.chat.quick_menu.mode;
                let Some(item) = gameplay_ui.chat.quick_menu.select(item) else {
                    continue;
                };
                let registered = match mode {
                    QuickChatMenuMode::Closed => continue,
                    QuickChatMenuMode::Emotes => {
                        if vehicle.family != LegacyVehiclePresentationFamily::None {
                            modal_owners
                                .audio
                                .runtime
                                .queue_gameplay_ui_sound("Action_Failure01");
                            continue;
                        }
                        let Some(pc_id) = runtime.player_id else {
                            runtime.message =
                                "EmoteChat requires the authoritative local PC ID".to_owned();
                            continue;
                        };
                        ffone_protocol::RegisteredGameplayRequest0104::new(
                            packet::P_CL2FE_REQ_PC_AVATAR_EMOTES_CHAT,
                            AvatarEmoteChat0104 {
                                pc_id,
                                emote_code: item.emote_code,
                            }
                            .encode(),
                        )
                    }
                    QuickChatMenuMode::MenuChat => {
                        let localized = item.localized(
                            gameplay_ui.minimap.map_name.clone(),
                            gameplay_ui.current_objective.title.clone(),
                        );
                        let text = modal_owners
                            .shell
                            .localization
                            .text(&modal_owners.shell.language, &localized);
                        let message = match FixedUtf16::<128>::from_str(&text) {
                            Ok(message) => message,
                            Err(error) => {
                                runtime.message = format!("MenuChat cannot be encoded: {error}");
                                continue;
                            }
                        };
                        let (packet_id, payload) = match gameplay_ui.chat.selected {
                            ChatChannel::All => (
                                packet::P_CL2FE_REQ_SEND_MENUCHAT_MESSAGE,
                                MenuChatRequest0104 {
                                    message,
                                    emote_code: item.emote_code,
                                }
                                .encode(),
                            ),
                            ChatChannel::Group if normal_warp.group_ui.pc_members.len() > 1 => (
                                packet::P_CL2FE_REQ_SEND_ALL_GROUP_MENUCHAT_MESSAGE,
                                AllGroupMenuChatRequest0104 {
                                    message,
                                    // Clean `CnGuiChat.MenuChat` leaves this
                                    // field zero for the group request.
                                    emote_code: 0,
                                }
                                .encode(),
                            ),
                            ChatChannel::Group => {
                                gameplay_ui.chat.selected = ChatChannel::All;
                                runtime.message =
                                    "Group chat requires a multi-player group; returned to ALL"
                                        .to_owned();
                                continue;
                            }
                            ChatChannel::Buddy => {
                                let Some(target) = buddy_ui.selected_target() else {
                                    runtime.message =
                                        "Buddy MenuChat requires a selected Buddy".to_owned();
                                    continue;
                                };
                                (
                                    packet::P_CL2FE_REQ_SEND_BUDDY_MENUCHAT_MESSAGE,
                                    BuddyMenuChatRequest0104 {
                                        message,
                                        // The clean managed handler assigns
                                        // no emote field on buddy MenuChat.
                                        emote_code: 0,
                                        buddy_pc_uid: target.pc_uid,
                                        buddy_slot: target.slot as i8,
                                    }
                                    .encode(),
                                )
                            }
                        };
                        ffone_protocol::RegisteredGameplayRequest0104::new(packet_id, payload)
                    }
                };
                match registered {
                    Ok(request) => {
                        if let Err(error) =
                            bridge.send(NetworkCommand::SendRegisteredGameplay0104(request))
                        {
                            runtime.message = error;
                        }
                    }
                    Err(error) => {
                        runtime.message =
                            format!("MenuChat/Emote protocol rejected request: {error}");
                    }
                }
            }
        }
    }
}
