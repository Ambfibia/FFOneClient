use super::*;

pub(super) fn send_player_emote_continuation(
    mut continuation: ResMut<ffone_client::player_emote::PlayerEmoteContinuation>,
    mut player_commands: ResMut<TutorialPlayerPresentationCommandQueue>,
    mut runtime: ResMut<RuntimeStatus>,
    bridge: Res<NetworkBridge>,
    vehicle: Res<LocalVehiclePresentationRuntime>,
    owners: Query<(), With<LegacyPlayerController>>,
) {
    for (owner, emote_code) in continuation.pending.drain(..) {
        if !owners.contains(owner) || vehicle.family != LegacyVehiclePresentationFamily::None {
            continue;
        }
        let Some(pc_id) = runtime.player_id else {
            continue;
        };
        let request = ffone_protocol::RegisteredGameplayRequest0104::new(
            packet::P_CL2FE_REQ_PC_AVATAR_EMOTES_CHAT,
            AvatarEmoteChat0104 { pc_id, emote_code }.encode(),
        )
        .expect("emote continuation uses the registered protocol ABI");
        if let Err(error) = bridge.send(NetworkCommand::SendRegisteredGameplay0104(request)) {
            runtime.message = error;
        } else {
            player_commands.record_emote_continuation_sent(emote_code);
        }
    }
}

pub(in super::super) fn legacy_chat_sender_name(style: &ffone_protocol::PcStyle0104) -> Option<String> {
    let first = style.first_name.to_string_lossy();
    let last = style.last_name.to_string_lossy();
    let name = format!("{} {}", first.trim(), last.trim())
        .trim()
        .to_owned();
    (!name.is_empty()).then_some(name)
}

pub(in super::super) fn push_world_chat_history(lines: &mut Vec<ChatLineUi>, line: ChatLineUi) {
    lines.push(line);
    if lines.len() > TUTORIAL_CHAT_HISTORY_LIMIT {
        let overflow = lines.len() - TUTORIAL_CHAT_HISTORY_LIMIT;
        lines.drain(..overflow);
    }
}

pub(in super::super) fn push_world_chat_line(runtime: &mut RuntimeStatus, line: ChatLineUi) {
    push_world_chat_history(&mut runtime.chat.world_chat_lines, line);
}

pub(in super::super) fn receive_npc_chat(
    mut events: MessageReader<NpcChatEvent>,
    state: Res<State<ClientState>>,
    options: Res<OptionProductionRuntime>,
    content: Res<TutorialMissionContent>,
    localization: Res<Localization>,
    language: Res<Language>,
    mut runtime: ResMut<RuntimeStatus>,
    mut tutorial: ResMut<TutorialMissionRuntime>,
) {
    for event in events.read() {
        // AddEventString(4) is controlled by NPC/events-in-chat, independently
        // of speech balloons and the currently selected chat channel.
        if !options.options.display.npc_messages_in_chat
            || !matches!(state.get(), ClientState::World | ClientState::Tutorial)
            || runtime.player_id.is_none()
        {
            continue;
        }
        let Some(npc) = content.gameplay_npc(event.npc_type) else {
            continue;
        };
        let sender = localization.text(
            &language,
            &ffone_client::localization::localized_tabledata_npc_name(npc.npc_type, &npc.name),
        );
        let message = localization.text(&language, &event.message);
        if message.is_empty() || message == " " {
            continue;
        }
        // Capture the spoken locale at receipt, as with other historical chat.
        let localized = LocalizedText::new("ui.hud.chat.message.direct", "{sender}: {message}")
            .with_arg("sender", &sender)
            .with_arg("message", &message);
        let line = ChatLineUi::npc(format!("{sender}: {message}")).with_localized(localized);
        if *state.get() == ClientState::Tutorial {
            tutorial.push_chat_line(line);
        } else {
            push_world_chat_line(&mut runtime, line);
        }
    }
}

/// Writes the chat copies recorded by `NanocomMessageUiModel::enqueue`.
/// Echoes wait while World/Tutorial is being entered, so notices queued from
/// bootstrap frames still reach the chat that owns them.
pub(in super::super) fn receive_nanocom_chat(
    state: Res<State<ClientState>>,
    options: Res<OptionProductionRuntime>,
    localization: Res<Localization>,
    language: Res<Language>,
    mut messages: ResMut<NanocomMessageUiModel>,
    mut runtime: ResMut<RuntimeStatus>,
    mut tutorial: ResMut<TutorialMissionRuntime>,
) {
    let tutorial_chat = match state.get() {
        ClientState::World => false,
        ClientState::Tutorial => true,
        _ => return,
    };
    if !messages.has_chat_echo() {
        return;
    }
    while let Some(echo) = messages.pop_chat_echo() {
        // AddEventString(4) follows NPC/events-in-chat; type 1 is unconditional.
        if !echo.interactive && !options.options.display.npc_messages_in_chat {
            continue;
        }
        let title = localization.text(&language, &echo.title);
        let body = localization.text(&language, &echo.body);
        let localized = LocalizedText::new("ui.hud.chat.message.direct", "{sender}: {message}")
            .with_arg("sender", &title)
            .with_arg("message", &body);
        let text = format!("{title}: {body}");
        let line = if echo.interactive {
            ChatLineUi::system(text)
        } else {
            ChatLineUi::npc(text)
        }
        .with_localized(localized);
        if tutorial_chat {
            tutorial.push_chat_line(line);
        } else if echo.interactive {
            // AddChatString(1) retains the line in ALL and GROUP, not FRIEND.
            push_world_chat_history(&mut runtime.chat.world_chat_lines, line.clone());
            push_world_chat_history(&mut runtime.chat.world_group_chat_lines, line);
        } else {
            push_world_chat_line(&mut runtime, line);
        }
    }
}

/// Server-authored world chat is deliberately separated from tutorial copy:
/// the latter must remain key-first through `Localization`, while this text is
/// composed from protocol names/messages plus clean TextManager labels.
#[cfg(test)]
pub(in super::super) fn world_chat_line(text: impl Into<String>) -> ChatLineUi {
    ChatLineUi {
        text: text.into(),
        ..default()
    }
}

pub(in super::super) fn local_freechat_line(
    sender: impl Into<String>,
    message: impl Into<String>,
) -> ChatLineUi {
    localized_player_chat_line(
        ChatLineKind::Normal,
        "ui.hud.chat.message.local",
        "[Local] {sender}: {message}",
        sender,
        message,
    )
}

pub(in super::super) fn group_freechat_line(
    prefixed: bool,
    sender: impl Into<String>,
    message: impl Into<String>,
) -> ChatLineUi {
    let (key, fallback) = if prefixed {
        ("ui.hud.chat.message.group", "[Group] {sender}: {message}")
    } else {
        ("ui.hud.chat.message.direct", "{sender}: {message}")
    };
    localized_player_chat_line(ChatLineKind::Group, key, fallback, sender, message)
}

pub(in super::super) fn buddy_freechat_line(
    prefixed: bool,
    sender: impl Into<String>,
    message: impl Into<String>,
) -> ChatLineUi {
    let (key, fallback) = if prefixed {
        ("ui.hud.chat.message.buddy", "[Buddy] {sender}: {message}")
    } else {
        ("ui.hud.chat.message.direct", "{sender}: {message}")
    };
    localized_player_chat_line(ChatLineKind::Buddy, key, fallback, sender, message)
}

pub(in super::super) fn buddy_entry_from_protocol(entry: BuddyBaseInfo0104) -> BuddyEntry {
    BuddyEntry {
        runtime_pc_id: entry.pc_id,
        pc_uid: entry.pc_uid,
        blocked: entry.blocked != 0,
        free_chat: entry.free_chat != 0,
        presence: BuddyPresence::from_legacy(entry.pc_state),
        first_name: entry.first_name.to_string_lossy(),
        last_name: entry.last_name.to_string_lossy(),
        gender: entry.gender,
        name_check_flag: entry.name_check_flag,
    }
}

pub(in super::super) fn buddy_target_by_uid(model: &BuddyUiModel, pc_uid: i64) -> Option<BuddyTarget> {
    (0..BUDDY_MAX_SLOTS).find_map(|slot| {
        model
            .slot(slot)
            .filter(|entry| entry.pc_uid == pc_uid)
            .map(|_| BuddyTarget { slot, pc_uid })
    })
}

pub(in super::super) fn push_buddy_presence_notice(
    runtime: &mut RuntimeStatus,
    notice: BuddyUiNotice,
    selected_channel: ChatChannel,
) {
    let BuddyUiNotice::PresenceChanged {
        display_name,
        presence,
        ..
    } = notice;
    let state = match presence {
        BuddyPresence::Online => "online",
        BuddyPresence::Offline => "offline",
    };
    push_world_chat_history(
        &mut runtime.chat.world_buddy_chat_lines,
        ChatLineUi {
            text: format!("{display_name} is now {state}."),
            kind: ChatLineKind::System,
            localized: None,
        },
    );
    if selected_channel != ChatChannel::Buddy {
        runtime.chat.world_chat_alerts[ChatChannel::Buddy.index()] = true;
    }
}

pub(in super::super) fn queue_buddy_invite(
    invite: BuddyInvite,
    model: &mut BuddyUiModel,
    outbox: &mut BuddyUiOutbox,
    integration: &mut BuddyRuntimeIntegration,
    nanocom_messages: &mut NanocomMessageUiModel,
) {
    let invite_id = invite.invite_id;
    let display_name = invite.display_name();
    match model.receive_invite(invite) {
        BuddyInviteDisposition::Queued => {
            integration.push_nanocom_action(
                nanocom_messages,
                PendingBuddyNanocomAction::Invite { invite_id },
                display_name,
            );
        }
        BuddyInviteDisposition::AutoDeclined(action) => outbox.push(action),
        BuddyInviteDisposition::IgnoredBlocked => {}
    }
}

pub(in super::super) fn buddy_confirmation_message(
    model: &BuddyUiModel,
    confirmation: BuddyConfirmation,
) -> (String, SystemMessageButtonType) {
    let target = match confirmation {
        BuddyConfirmation::Remove(target)
        | BuddyConfirmation::Warp(target)
        | BuddyConfirmation::LeaveGroupForWarp(target) => target,
    };
    let display_name = model
        .slot(target.slot)
        .filter(|entry| entry.pc_uid == target.pc_uid)
        .map(BuddyEntry::display_name)
        .unwrap_or_else(|| format!("Player {}", target.pc_uid));
    match confirmation {
        BuddyConfirmation::Remove(_) => (
            format!(
                "Delete Buddy?\nDo you really want to delete {display_name} from your buddy list?"
            ),
            SystemMessageButtonType::YesNo,
        ),
        BuddyConfirmation::Warp(_) => (
            format!("ALERT!\nAre you ready to warp to {display_name}?"),
            SystemMessageButtonType::CancelWarp,
        ),
        BuddyConfirmation::LeaveGroupForWarp(_) => (
            "WARNING!\nIf you choose to warp, you will be dropped from your current group."
                .to_owned(),
            SystemMessageButtonType::OkCancel,
        ),
    }
}

pub(in super::super) fn consume_buddy_ui_outbox(
    bridge: Res<NetworkBridge>,
    mut buddy_ui: ResMut<BuddyUiModel>,
    mut outbox: ResMut<BuddyUiOutbox>,
    group_ui: Res<GroupUiModel>,
    mut integration: ResMut<BuddyRuntimeIntegration>,
    mut system_messages: ResMut<SystemMessageUiModel>,
    mut runtime: ResMut<RuntimeStatus>,
) {
    for action in outbox.drain().collect::<Vec<_>>() {
        match action {
            BuddyUiAction::RefreshStatesRequested => {
                if let Err(error) =
                    bridge.send(NetworkCommand::RefreshBuddyState(BuddyStateRequest0104 {
                        unused: 0,
                    }))
                {
                    runtime.message = error;
                }
            }
            BuddyUiAction::AddByNameRequested {
                first_name,
                last_name,
            } => {
                let request = FixedUtf16::<9>::from_str(&first_name).and_then(|first_name| {
                    FixedUtf16::<17>::from_str(&last_name).map(|last_name| {
                        BuddyFindNameRequest0104 {
                            first_name,
                            last_name,
                        }
                    })
                });
                match request {
                    Ok(request) => {
                        if let Err(error) = bridge.send(NetworkCommand::RequestBuddyByName(request))
                        {
                            runtime.message = error;
                        }
                    }
                    Err(error) => {
                        runtime.message = format!("Buddy name cannot be encoded: {error}");
                    }
                }
            }
            BuddyUiAction::AddNameRejected { message } => {
                runtime.message = message;
            }
            BuddyUiAction::ConfirmationRequested(confirmation) => {
                let (text, button_type) = buddy_confirmation_message(&buddy_ui, confirmation);
                integration.push_system_action(
                    &mut system_messages,
                    PendingBuddySystemAction::Confirmation(confirmation),
                    text,
                    button_type,
                );
            }
            BuddyUiAction::RemoveRequested(target) => {
                integration.pending_remove = Some(target);
                if let Err(error) =
                    bridge.send(NetworkCommand::BlockBuddy(BuddySetBlockRequest0104 {
                        buddy_pc_uid: target.pc_uid,
                        buddy_slot: target.slot as i8,
                    }))
                {
                    integration.pending_remove = None;
                    runtime.message = error;
                }
            }
            BuddyUiAction::WarpCooldownNotice { remaining_seconds } => {
                integration.push_system_action(
                    &mut system_messages,
                    PendingBuddySystemAction::Dismiss,
                    format!(
                        "WARP FAILURE!\nRecharge not complete. You will be ready to Buddy Warp again in {remaining_seconds}s."
                    ),
                    SystemMessageButtonType::Ok,
                );
            }
            BuddyUiAction::WarpRequested {
                target,
                leave_group: _,
            } => {
                let target_was_group_member = group_ui
                    .pc_members
                    .iter()
                    .any(|member| member.pc_uid == target.pc_uid);
                integration.pending_warp = Some(PendingBuddyWarp {
                    target,
                    target_was_group_member,
                });
                system_messages.set_focus_out(true);
                if let Err(error) = bridge.send(NetworkCommand::WarpToBuddy(BuddyWarpRequest0104 {
                    buddy_pc_uid: target.pc_uid,
                    buddy_slot: target.slot as i8,
                })) {
                    integration.pending_warp = None;
                    buddy_ui.set_warp_cooldown_seconds(0);
                    system_messages.set_focus_out(false);
                    runtime.message = error;
                } else {
                    runtime.chat.pending_social_sfx.push("Buddy_Warp");
                }
            }
            BuddyUiAction::InviteResponse {
                requester_pc_id,
                requester_pc_uid,
                accepted,
                ..
            } => {
                if let Err(error) =
                    bridge.send(NetworkCommand::AcceptBuddy(BuddyAcceptRequest0104 {
                        accept_flag: if accepted { 1 } else { 0 },
                        buddy_id: requester_pc_id,
                        buddy_pc_uid: requester_pc_uid,
                    }))
                {
                    runtime.message = error;
                }
            }
        }
    }
}

pub(in super::super) fn consume_buddy_system_message_outbox(
    mut system_outbox: ResMut<SystemMessageUiOutbox>,
    mut buddy_outbox: ResMut<BuddyUiOutbox>,
    mut buddy_ui: ResMut<BuddyUiModel>,
    mut integration: ResMut<BuddyRuntimeIntegration>,
    mut runtime: ResMut<RuntimeStatus>,
) {
    if integration.pending_system_actions.is_empty() {
        return;
    }
    let mut unrelated = Vec::new();
    for action in system_outbox.drain().collect::<Vec<_>>() {
        let SystemMessageUiAction::Chosen {
            request_id, choice, ..
        } = action;
        let Some(pending) = integration.pending_system_actions.remove(&request_id) else {
            unrelated.push(action);
            continue;
        };
        let accepted = choice == SystemMessageChoice::Primary;
        let resolved = match pending {
            PendingBuddySystemAction::Dismiss => Ok(None),
            PendingBuddySystemAction::Confirmation(confirmation) => {
                buddy_ui.resolve_confirmation(confirmation, accepted)
            }
        };
        match resolved {
            Ok(Some(action)) => buddy_outbox.push(action),
            Ok(None) => {}
            Err(error) => runtime.message = format!("Buddy confirmation ignored: {error}"),
        }
    }
    for action in unrelated {
        system_outbox.push(action);
    }
}

pub(in super::super) const fn buddy_nanocom_resolution_accepted(
    resolution: NanocomMessageResolution,
) -> bool {
    matches!(resolution, NanocomMessageResolution::Accepted)
}

pub(in super::super) fn consume_buddy_nanocom_outbox(
    bridge: Res<NetworkBridge>,
    mut nanocom_outbox: ResMut<NanocomMessageUiOutbox>,
    mut buddy_outbox: ResMut<BuddyUiOutbox>,
    mut buddy_ui: ResMut<BuddyUiModel>,
    mut integration: ResMut<BuddyRuntimeIntegration>,
    mut runtime: ResMut<RuntimeStatus>,
) {
    if integration.pending_nanocom_actions.is_empty() {
        return;
    }

    let mut unrelated = Vec::new();
    while let Some(action) = nanocom_outbox.pop_front() {
        if action.kind != NanocomMessageKind::BuddyInvite {
            unrelated.push(action);
            continue;
        }
        let Some(pending) = integration
            .pending_nanocom_actions
            .remove(&action.request_id)
        else {
            unrelated.push(action);
            continue;
        };
        let accepted = buddy_nanocom_resolution_accepted(action.resolution);
        match pending {
            PendingBuddyNanocomAction::Invite { invite_id } => {
                match buddy_ui.respond_to_invite(invite_id, accepted) {
                    Ok(action) => buddy_outbox.push(action),
                    Err(error) => {
                        runtime.message = format!("Buddy NanoCom response ignored: {error}");
                    }
                }
            }
            PendingBuddyNanocomAction::NameInvite {
                pc_uid,
                first_name,
                last_name,
            } => {
                let request = FixedUtf16::<9>::from_str(&first_name).and_then(|first_name| {
                    FixedUtf16::<17>::from_str(&last_name).map(|last_name| {
                        BuddyFindNameAcceptRequest0104 {
                            accept_flag: if accepted { 1 } else { 0 },
                            buddy_pc_uid: pc_uid,
                            first_name,
                            last_name,
                        }
                    })
                });
                match request {
                    Ok(request) => {
                        if let Err(error) = bridge.send(NetworkCommand::AcceptBuddyByName(request))
                        {
                            runtime.message = error;
                        }
                    }
                    Err(error) => {
                        runtime.message =
                            format!("Name-based Buddy response cannot be encoded: {error}");
                    }
                }
            }
        }
    }
    for action in unrelated {
        nanocom_outbox.push(action);
    }
}

pub(in super::super) fn complete_pending_buddy_warp_after_authoritative_teleport(
    bridge: &NetworkBridge,
    integration: &mut BuddyRuntimeIntegration,
    group_runtime: &mut GroupProductionRuntime0104,
    system_messages: &mut SystemMessageUiModel,
) -> Result<Option<String>, String> {
    let Some(pending) = integration.pending_warp.take() else {
        return Ok(None);
    };
    system_messages.set_focus_out(false);
    if !pending.target_was_group_member {
        let request = group_runtime
            .request_leave(
                GroupLeaveOwner0104::BuddyWarp,
                GroupOwnerReachability0104::Proven,
            )
            .map_err(|error| {
                format!(
                    "Buddy warp to PCUID {} succeeded, but the group-leave owner rejected its exact request: {error}",
                    pending.target.pc_uid
                )
            })?;
        if let Err(error) = bridge.send(NetworkCommand::SendRegisteredGameplay0104(request.clone()))
        {
            group_runtime.release_transport_failure(&request);
            return Err(format!(
                "Buddy warp to PCUID {} succeeded, but the exact follow-up group-leave request failed: {error}",
                pending.target.pc_uid
            ));
        }
        Ok(Some(format!(
            "Buddy warp to PCUID {} accepted; group leave requested",
            pending.target.pc_uid
        )))
    } else {
        Ok(Some(format!(
            "Buddy warp to PCUID {} accepted on the current shard",
            pending.target.pc_uid
        )))
    }
}
