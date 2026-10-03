use super::*;

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn apply_group_and_freechat_frame(
    frame: &DecodedFrame,
    commands: &mut Commands,
    asset_server: &AssetServer,
    bridge: &NetworkBridge,
    tutorial_content: &TutorialMissionContent,
    social: &mut SocialIngress<'_>,
    system_messages: &mut SystemMessageUiModel,
    remote_appearances: &Query<&NetworkPcAppearance0104>,
    remote_players: &Query<(Entity, &NetworkRemotePc0104, &GlobalTransform)>,
    local_identities: &Query<(Entity, &LocalNetworkIdentity)>,
    remote_animations: &mut Query<(&NetworkRemotePc0104, &mut RemoteAnimation)>,
    player_commands: &mut TutorialPlayerPresentationCommandQueue,
    local_vehicle: &LocalVehiclePresentationRuntime,
    option_runtime: &OptionProductionRuntime,
    runtime: &mut RuntimeStatus,
) {
    match apply_group_production_frame_0104(
        frame,
        commands,
        asset_server,
        bridge,
        tutorial_content,
        &mut social.group_runtime,
        &mut social.group_integration,
        &mut social.group_ui,
        &social.buddy_ui,
        &mut social.nanocom_messages,
        system_messages,
        remote_appearances,
        option_runtime,
        runtime,
    ) {
        Ok(true) | Ok(false) => {}
        Err(error) => runtime.message = format!("Group production ignored {error}"),
    }

    if let Err(error) = apply_freechat_frame(
        frame,
        runtime,
        remote_appearances,
        remote_players,
        local_identities,
        &social.buddy_ui,
        &mut social.freechat_bubbles,
    ) {
        runtime.message = format!("Chat ignored {error}");
    }
    if let Err(error) = apply_menu_chat_frame(
        frame,
        runtime,
        remote_appearances,
        remote_players,
        local_identities,
        remote_animations,
        player_commands,
        local_vehicle,
        &social.buddy_ui,
        &mut social.freechat_bubbles,
    ) {
        runtime.message = format!("MenuChat ignored {error}");
    }
    if let Err(error) = apply_avatar_emote_frame(
        frame,
        runtime,
        remote_animations,
        player_commands,
        local_vehicle,
    ) {
        runtime.message = format!("Avatar emote ignored {error}");
    }
}

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn apply_buddy_chat_frame(
    frame: &DecodedFrame,
    bridge: &NetworkBridge,
    social: &mut SocialIngress<'_>,
    system_messages: &mut SystemMessageUiModel,
    remote_appearances: &Query<&NetworkPcAppearance0104>,
    gameplay_ui: &GameplayUiModel,
    runtime: &mut RuntimeStatus,
) {
    if let Err(error) = apply_all_group_freechat_frame(
        frame,
        runtime,
        &social.group_ui,
        &social.buddy_ui,
        gameplay_ui.chat.selected,
    ) {
        runtime.message = format!("Group chat ignored {error}");
    }
    if let Err(error) = apply_all_group_menu_chat_frame(
        frame,
        runtime,
        &social.group_ui,
        &social.buddy_ui,
        gameplay_ui.chat.selected,
    ) {
        runtime.message = format!("Group MenuChat ignored {error}");
    }
    if let Err(error) = apply_buddy_lifecycle_frame(
        frame,
        &mut social.buddy_ui,
        &mut social.buddy_outbox,
        &mut social.buddy_runtime,
        &mut social.group_runtime,
        system_messages,
        &mut social.nanocom_messages,
        runtime,
        gameplay_ui.chat.selected,
        remote_appearances,
        bridge,
    ) {
        runtime.message = format!("Buddy lifecycle ignored {error}");
    }
    if let Err(error) =
        apply_buddy_freechat_frame(frame, &social.buddy_ui, runtime, gameplay_ui.chat.selected)
    {
        runtime.message = format!("Buddy chat ignored {error}");
    }
    if let Err(error) =
        apply_buddy_menu_chat_frame(frame, &social.buddy_ui, runtime, gameplay_ui.chat.selected)
    {
        runtime.message = format!("Buddy MenuChat ignored {error}");
    }
}

pub(in super::super) fn apply_music_frame_0104(
    frame: &DecodedFrame,
    music: &mut ffone_client::world_audio::RetrobutionMusicRequests,
) -> Result<bool, String> {
    if let Some(message) = decode_server_message_0104(frame.packet_type, &frame.payload)
        .map_err(|error| format!("malformed music server message: {error}"))?
        && matches!(message.message_type, 11 | 12)
    {
        music.request(
            message.message.to_string_lossy(),
            message.message_type == 12,
        );
        return Ok(true);
    }
    if let Some(FreeChatPacket0104::Success(chat)) =
        decode_freechat_packet_0104(frame.packet_type, &frame.payload)
            .map_err(|error| format!("malformed music chat message: {error}"))?
        && chat.emote_code == 666
    {
        music.request(chat.message.to_string_lossy(), chat.pc_id == 1);
        return Ok(true);
    }
    Ok(false)
}

pub(in super::super) fn apply_server_message_frame_0104(
    frame: &DecodedFrame,
    runtime: &mut RuntimeStatus,
    localization: &Localization,
    language: &Language,
) -> Result<bool, String> {
    let message = decode_server_message_0104(frame.packet_type, &frame.payload)
        .map_err(|error| format!("malformed protocol-0104 server message: {error}"))?;
    let Some(message) = message else {
        return Ok(false);
    };
    let server_text = message.message.to_string_lossy();
    if server_text.is_empty() {
        return Ok(true);
    }
    let server_text = super::command_text::command_reply(&server_text)
        .map(|reply| localization.text(language, &reply))
        .unwrap_or(server_text);

    // Clean CnGuiChat.ReceiveMOTD calls AddChatString(1, ...): the same
    // system-colored line is retained in ALL and GROUP, but not FRIEND.
    let text = localization.text(
        language,
        &LocalizedText::new("ui.hud.chat.gamemaster_message", "Gamemaster: {message}")
            .with_arg("message", server_text),
    );
    let line = ChatLineUi::system(text);
    push_world_chat_history(&mut runtime.chat.world_chat_lines, line.clone());
    push_world_chat_history(&mut runtime.chat.world_group_chat_lines, line);
    Ok(true)
}

pub(in super::super) fn apply_freechat_frame(
    frame: &DecodedFrame,
    runtime: &mut RuntimeStatus,
    remote_appearances: &Query<&NetworkPcAppearance0104>,
    remote_players: &Query<(Entity, &NetworkRemotePc0104, &GlobalTransform)>,
    local_identities: &Query<(Entity, &LocalNetworkIdentity)>,
    buddy_ui: &BuddyUiModel,
    bubbles: &mut PlayerFreeChatBubbleRuntime,
) -> Result<bool, String> {
    let packet = decode_freechat_packet_0104(frame.packet_type, &frame.payload)
        .map_err(|error| format!("malformed protocol-0104 FreeChat packet: {error}"))?;
    let Some(packet) = packet else {
        return Ok(false);
    };
    let FreeChatPacket0104::Success(chat) = packet else {
        return Ok(true);
    };
    let remote_sender_name = if runtime.player_id == Some(chat.pc_id) {
        None
    } else {
        remote_appearances
            .iter()
            .find(|appearance| appearance.0.id == chat.pc_id)
            .and_then(|appearance| legacy_chat_sender_name(&appearance.0.style))
    };
    let sender_blocked =
        runtime.player_id != Some(chat.pc_id) && buddy_ui.is_blocked_runtime_pc_id(chat.pc_id);
    let pc_id = chat.pc_id;
    let message = chat.message.to_string_lossy();
    if apply_freechat_success(chat, runtime, remote_sender_name, sender_blocked) {
        let owner = if runtime.player_id == Some(pc_id) {
            local_identities
                .iter()
                .find(|(_, identity)| identity.player_id == pc_id)
                .map(|(entity, _)| entity)
        } else {
            remote_players
                .iter()
                .find(|(_, remote, _)| remote.pc_id == pc_id)
                .map(|(entity, _, _)| entity)
        };
        if let Some(owner) = owner {
            bubbles.request_message(owner, message);
        }
    }
    Ok(true)
}

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn apply_menu_chat_frame(
    frame: &DecodedFrame,
    runtime: &mut RuntimeStatus,
    remote_appearances: &Query<&NetworkPcAppearance0104>,
    remote_players: &Query<(Entity, &NetworkRemotePc0104, &GlobalTransform)>,
    local_identities: &Query<(Entity, &LocalNetworkIdentity)>,
    remote_animations: &mut Query<(&NetworkRemotePc0104, &mut RemoteAnimation)>,
    player_commands: &mut TutorialPlayerPresentationCommandQueue,
    local_vehicle: &LocalVehiclePresentationRuntime,
    buddy_ui: &BuddyUiModel,
    bubbles: &mut PlayerFreeChatBubbleRuntime,
) -> Result<bool, String> {
    let packet = decode_menu_chat_packet_0104(frame.packet_type, &frame.payload)
        .map_err(|error| format!("malformed protocol-0104 MenuChat packet: {error}"))?;
    let Some(packet) = packet else {
        return Ok(false);
    };
    let MenuChatPacket0104::Success(chat) = packet else {
        return Ok(true);
    };
    let sender_blocked =
        runtime.player_id != Some(chat.pc_id) && buddy_ui.is_blocked_runtime_pc_id(chat.pc_id);
    if sender_blocked {
        return Ok(true);
    }
    let remote_sender_name = if runtime.player_id == Some(chat.pc_id) {
        None
    } else {
        remote_appearances
            .iter()
            .find(|appearance| appearance.0.id == chat.pc_id)
            .and_then(|appearance| legacy_chat_sender_name(&appearance.0.style))
    };
    let sender_name = if runtime.player_id == Some(chat.pc_id) {
        (!runtime.player_name.is_empty()).then(|| runtime.player_name.clone())
    } else {
        remote_sender_name
    };
    let Some(sender_name) = sender_name else {
        return Ok(true);
    };
    let pc_id = chat.pc_id;
    let emote_code = chat.emote_code;
    let message = chat.message.to_string_lossy();
    if runtime.player_id == Some(pc_id) {
        runtime.chat.outgoing_audio_pending += 1;
    }
    push_world_chat_line(runtime, local_freechat_line(sender_name, message.clone()));
    let owner = if runtime.player_id == Some(pc_id) {
        local_identities
            .iter()
            .find(|(_, identity)| identity.player_id == pc_id)
            .map(|(entity, _)| entity)
    } else {
        remote_players
            .iter()
            .find(|(_, remote, _)| remote.pc_id == pc_id)
            .map(|(entity, _, _)| entity)
    };
    if let Some(owner) = owner {
        bubbles.request_message(owner, message);
    }
    if emote_code > 0 {
        let _ = apply_avatar_emote_code(
            pc_id,
            emote_code,
            runtime,
            remote_animations,
            player_commands,
            local_vehicle,
        )?;
    }
    Ok(true)
}

pub(in super::super) fn apply_avatar_emote_frame(
    frame: &DecodedFrame,
    runtime: &RuntimeStatus,
    remote_animations: &mut Query<(&NetworkRemotePc0104, &mut RemoteAnimation)>,
    player_commands: &mut TutorialPlayerPresentationCommandQueue,
    local_vehicle: &LocalVehiclePresentationRuntime,
) -> Result<bool, String> {
    let packet = decode_avatar_emote_chat_0104(frame.packet_type, &frame.payload)
        .map_err(|error| format!("malformed protocol-0104 AvatarEmote packet: {error}"))?;
    let Some(packet) = packet else {
        return Ok(false);
    };
    apply_avatar_emote_code(
        packet.pc_id,
        packet.emote_code,
        runtime,
        remote_animations,
        player_commands,
        local_vehicle,
    )?;
    Ok(true)
}

pub(in super::super) fn apply_all_group_freechat_frame(
    frame: &DecodedFrame,
    runtime: &mut RuntimeStatus,
    group_ui: &GroupUiModel,
    buddy_ui: &BuddyUiModel,
    selected_channel: ChatChannel,
) -> Result<bool, String> {
    let packet = decode_all_group_freechat_packet_0104(frame.packet_type, &frame.payload)
        .map_err(|error| format!("malformed protocol-0104 ALL GROUP FreeChat packet: {error}"))?;
    let Some(packet) = packet else {
        return Ok(false);
    };
    let AllGroupFreeChatPacket0104::Success(chat) = packet else {
        return Ok(true);
    };
    // `ReceiveGroupChat` resolves the sender only through the current group
    // roster, including the local member. A world entity outside the roster
    // must not be accepted as a group-chat sender.
    let roster_sender_name = group_ui
        .pc_members
        .iter()
        .find(|member| member.pc_id == chat.sender_pc_id)
        .map(|member| {
            let first = member.first_name.trim();
            let last = member.last_name.trim();
            let approved = format!("{first} {last}").trim().to_owned();
            if approved.is_empty() {
                format!("Player {}", member.pc_uid)
            } else {
                approved
            }
        });
    let sender_blocked = buddy_ui.is_blocked_runtime_pc_id(chat.sender_pc_id);
    apply_all_group_freechat_success(
        chat,
        runtime,
        roster_sender_name,
        selected_channel,
        sender_blocked,
    );
    Ok(true)
}

pub(in super::super) fn apply_all_group_menu_chat_frame(
    frame: &DecodedFrame,
    runtime: &mut RuntimeStatus,
    group_ui: &GroupUiModel,
    buddy_ui: &BuddyUiModel,
    selected_channel: ChatChannel,
) -> Result<bool, String> {
    let packet = decode_all_group_menu_chat_packet_0104(frame.packet_type, &frame.payload)
        .map_err(|error| format!("malformed protocol-0104 ALL GROUP MenuChat packet: {error}"))?;
    let Some(packet) = packet else {
        return Ok(false);
    };
    let AllGroupMenuChatPacket0104::Success(chat) = packet else {
        return Ok(true);
    };
    if buddy_ui.is_blocked_runtime_pc_id(chat.sender_pc_id) {
        return Ok(true);
    }
    let sender_name = group_ui
        .pc_members
        .iter()
        .find(|member| member.pc_id == chat.sender_pc_id)
        .map(|member| {
            let first = member.first_name.trim();
            let last = member.last_name.trim();
            let approved = format!("{first} {last}").trim().to_owned();
            if approved.is_empty() {
                format!("Player {}", member.pc_uid)
            } else {
                approved
            }
        });
    let Some(sender_name) = sender_name else {
        return Ok(true);
    };
    if runtime.player_id == Some(chat.sender_pc_id) {
        runtime.chat.outgoing_audio_pending += 1;
    }
    let message = chat.message.to_string_lossy();
    push_world_chat_history(
        &mut runtime.chat.world_group_chat_lines,
        group_freechat_line(false, sender_name.clone(), message.clone()),
    );
    push_world_chat_line(runtime, group_freechat_line(true, sender_name, message));
    if selected_channel != ChatChannel::Group {
        runtime.chat.world_chat_alerts[ChatChannel::Group.index()] = true;
    }
    Ok(true)
}

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn apply_buddy_lifecycle_frame(
    frame: &DecodedFrame,
    model: &mut BuddyUiModel,
    outbox: &mut BuddyUiOutbox,
    integration: &mut BuddyRuntimeIntegration,
    group_runtime: &mut GroupProductionRuntime0104,
    system_messages: &mut SystemMessageUiModel,
    nanocom_messages: &mut NanocomMessageUiModel,
    runtime: &mut RuntimeStatus,
    selected_channel: ChatChannel,
    remote_appearances: &Query<&NetworkPcAppearance0104>,
    bridge: &NetworkBridge,
) -> Result<bool, String> {
    let packet = decode_buddy_lifecycle_packet_0104(frame.packet_type, &frame.payload)
        .map_err(|error| format!("malformed protocol-0104 Buddy lifecycle packet: {error}"))?;
    let Some(packet) = packet else {
        return Ok(false);
    };
    match packet {
        BuddyLifecyclePacket0104::ListInfo(list) => {
            apply_buddy_list_info(list, model, runtime, selected_channel)?;
        }
        BuddyLifecyclePacket0104::AcceptSuccess(success) => {
            let slot = usize::try_from(success.buddy_slot)
                .map_err(|_| format!("negative accepted Buddy slot {}", success.buddy_slot))?;
            let pc_uid = success.buddy.pc_uid;
            model
                .set_entry(slot, Some(buddy_entry_from_protocol(success.buddy)))
                .map_err(|error| error.to_string())?;
            runtime
                .chat
                .world_buddy_chat_by_uid
                .insert(pc_uid, Vec::new());
            model.reset_refresh_timer();
            runtime.chat.pending_social_sfx.push("Action_Sucess");
        }
        BuddyLifecyclePacket0104::StateSuccess(state) => {
            apply_buddy_state_success(state, model, runtime, selected_channel)?;
        }
        BuddyLifecyclePacket0104::BlockSuccess(success) => {
            let slot = usize::try_from(success.buddy_slot)
                .map_err(|_| format!("negative blocked Buddy slot {}", success.buddy_slot))?;
            let target = BuddyTarget {
                slot,
                pc_uid: success.buddy_pc_uid,
            };
            if integration.pending_remove == Some(target) {
                if let Err(error) =
                    bridge.send(NetworkCommand::RemoveBuddy(BuddyRemoveRequest0104 {
                        buddy_pc_uid: target.pc_uid,
                        buddy_slot: target.slot as i8,
                    }))
                {
                    integration.pending_remove = None;
                    return Err(error);
                }
            } else {
                model
                    .set_blocked(target, true)
                    .map_err(|error| error.to_string())?;
            }
        }
        BuddyLifecyclePacket0104::RemoveSuccess(success) => {
            let slot = usize::try_from(success.buddy_slot)
                .map_err(|_| format!("negative removed Buddy slot {}", success.buddy_slot))?;
            let target = BuddyTarget {
                slot,
                pc_uid: success.buddy_pc_uid,
            };
            model
                .confirm_removed(target)
                .map_err(|error| error.to_string())?;
            runtime.chat.world_buddy_chat_by_uid.remove(&target.pc_uid);
            if integration.pending_remove == Some(target) {
                integration.pending_remove = None;
            }
        }
        BuddyLifecyclePacket0104::IncomingRequest(request) => {
            if runtime.player_id != Some(request.buddy_id) {
                return Err(format!(
                    "incoming Buddy request targets runtime PC {}, local runtime PC is {:?}",
                    request.buddy_id, runtime.player_id
                ));
            }
            let appearance = remote_appearances
                .iter()
                .find(|appearance| appearance.0.id == request.request_id)
                .map(|appearance| &appearance.0);
            let Some(appearance) = appearance else {
                return Err(format!(
                    "incoming Buddy request {} cannot resolve runtime PC {} to a persistent PCUID",
                    request.request_id, request.request_id
                ));
            };
            queue_buddy_invite(
                BuddyInvite {
                    invite_id: u64::from(request.request_id as u32),
                    requester_pc_id: request.request_id,
                    requester_pc_uid: appearance.style.pc_uid,
                    first_name: appearance.style.first_name.to_string_lossy(),
                    last_name: appearance.style.last_name.to_string_lossy(),
                    name_check_flag: appearance.style.name_check,
                },
                model,
                outbox,
                integration,
                nanocom_messages,
            );
        }
        BuddyLifecyclePacket0104::MakeSuccess(success) => {
            runtime.message = format!(
                "Buddy request {} delivered to PCUID {}",
                success.request_id, success.buddy_pc_uid
            );
        }
        BuddyLifecyclePacket0104::FindNameSuccess(success) => {
            if runtime.roster.selected_uid == Some(success.pc_uid) {
                return Ok(true);
            }
            let first_name = success.first_name.to_string_lossy();
            let last_name = success.last_name.to_string_lossy();
            let display_name = if success.name_check_flag == 1 {
                format!("{first_name} {last_name}").trim().to_owned()
            } else {
                format!("Player {}", success.pc_uid)
            };
            if !model.social_buddy_enabled() {
                let first_name = FixedUtf16::<9>::from_str(&first_name)
                    .map_err(|error| format!("Buddy decline first name is invalid: {error}"))?;
                let last_name = FixedUtf16::<17>::from_str(&last_name)
                    .map_err(|error| format!("Buddy decline last name is invalid: {error}"))?;
                bridge.send(NetworkCommand::AcceptBuddyByName(
                    BuddyFindNameAcceptRequest0104 {
                        accept_flag: 0,
                        buddy_pc_uid: success.pc_uid,
                        first_name,
                        last_name,
                    },
                ))?;
                return Ok(true);
            }
            integration.push_nanocom_action(
                nanocom_messages,
                PendingBuddyNanocomAction::NameInvite {
                    pc_uid: success.pc_uid,
                    first_name,
                    last_name,
                },
                display_name,
            );
        }
        BuddyLifecyclePacket0104::ListFailure(failure) => {
            return Err(format!(
                "Buddy list rejected with error {}",
                failure.error_code
            ));
        }
        BuddyLifecyclePacket0104::MakeFailure(failure) => {
            runtime.chat.pending_social_sfx.push("Action_Failure01");
            return Err(format!(
                "Buddy request for PCUID {} rejected with error {}",
                failure.buddy_pc_uid, failure.error_code
            ));
        }
        BuddyLifecyclePacket0104::FindNameFailure(failure) => {
            return Err(format!(
                "Buddy lookup for {} {} rejected with error {}",
                failure.first_name.to_string_lossy(),
                failure.last_name.to_string_lossy(),
                failure.error_code
            ));
        }
        BuddyLifecyclePacket0104::FindNameAcceptFailure(failure) => {
            if runtime.roster.selected_uid != Some(failure.pc_uid) {
                return Err(format!(
                    "Name-based Buddy response for {} {} (PCUID {}) failed with error {}",
                    failure.first_name.to_string_lossy(),
                    failure.last_name.to_string_lossy(),
                    failure.pc_uid,
                    failure.error_code
                ));
            }
        }
        BuddyLifecyclePacket0104::AcceptFailure(failure) => {
            runtime.chat.pending_social_sfx.push("Action_Failure01");
            return Err(format!(
                "Buddy invitation for PCUID {} rejected with error {}",
                failure.buddy_pc_uid, failure.error_code
            ));
        }
        BuddyLifecyclePacket0104::StateFailure(failure) => {
            return Err(format!(
                "Buddy state refresh rejected with error {}",
                failure.error_code
            ));
        }
        BuddyLifecyclePacket0104::BlockFailure(failure) => {
            // Clean `cnBuddySystemManager` does not register this failure
            // handler, so its global block-to-remove flag is not cleared.
            return Err(format!(
                "Buddy block for PCUID {} rejected with error {}",
                failure.buddy_pc_uid, failure.error_code
            ));
        }
        BuddyLifecyclePacket0104::RemoveFailure(failure) => {
            runtime.chat.pending_social_sfx.push("Action_Failure01");
            if integration
                .pending_remove
                .is_some_and(|target| target.pc_uid == failure.buddy_pc_uid)
            {
                integration.pending_remove = None;
            }
            return Err(format!(
                "Buddy removal for PCUID {} rejected with error {}",
                failure.buddy_pc_uid, failure.error_code
            ));
        }
        BuddyLifecyclePacket0104::WarpFailure(failure) => {
            runtime.chat.pending_social_sfx.push("Action_Failure01");
            let display_name = buddy_target_by_uid(model, failure.buddy_pc_uid)
                .and_then(|target| model.slot(target.slot))
                .map(BuddyEntry::display_name)
                .unwrap_or_else(|| format!("Player {}", failure.buddy_pc_uid));
            match failure.error_code {
                3 | 4 => integration.push_system_action(
                    system_messages,
                    PendingBuddySystemAction::Dismiss,
                    format!(
                        "Warp failure!\nUh oh! {display_name} is in a location that you can't warp to! Try again later."
                    ),
                    SystemMessageButtonType::Ok,
                ),
                6 => integration.push_system_action(
                    system_messages,
                    PendingBuddySystemAction::Dismiss,
                    format!(
                        "WARP FAILURE!\nRecharge not complete. You will be ready to Buddy Warp again in {}s.",
                        model.warp_cooldown_seconds()
                    ),
                    SystemMessageButtonType::Ok,
                ),
                _ => {}
            }
            model.set_warp_cooldown_seconds(0);
            system_messages.set_focus_out(false);
            integration.pending_warp = None;
            runtime.message = format!(
                "Buddy warp to PCUID {} rejected with error {}",
                failure.buddy_pc_uid, failure.error_code
            );
        }
        BuddyLifecyclePacket0104::WarpOtherShardSuccess(success) => {
            system_messages.set_focus_out(false);
            integration.pending_warp = None;
            runtime.message = format!(
                "Buddy warp to PCUID {} requires shard {} channel {}; native shard handoff is not yet proven",
                success.buddy_pc_uid, success.shard_num, success.channel_num
            );
        }
        BuddyLifecyclePacket0104::WarpSameShardSuccess(_) => {
            system_messages.set_focus_out(false);
            if let Some(message) = complete_pending_buddy_warp_after_authoritative_teleport(
                bridge,
                integration,
                group_runtime,
                system_messages,
            )? {
                runtime.message = message;
            }
        }
        BuddyLifecyclePacket0104::MakeRequest(_)
        | BuddyLifecyclePacket0104::FindNameRequest(_)
        | BuddyLifecyclePacket0104::FindNameAcceptRequest(_)
        | BuddyLifecyclePacket0104::AcceptRequest(_)
        | BuddyLifecyclePacket0104::StateRequest(_)
        | BuddyLifecyclePacket0104::SetBlockRequest(_)
        | BuddyLifecyclePacket0104::RemoveRequest(_)
        | BuddyLifecyclePacket0104::WarpRequest(_) => {}
    }
    Ok(true)
}

pub(in super::super) fn apply_buddy_menu_chat_frame(
    frame: &DecodedFrame,
    model: &BuddyUiModel,
    runtime: &mut RuntimeStatus,
    selected_channel: ChatChannel,
) -> Result<bool, String> {
    let packet = decode_buddy_menu_chat_packet_0104(frame.packet_type, &frame.payload)
        .map_err(|error| format!("malformed protocol-0104 Buddy MenuChat packet: {error}"))?;
    let Some(packet) = packet else {
        return Ok(false);
    };
    let BuddyMenuChatPacket0104::Success(chat) = packet else {
        return Ok(true);
    };
    apply_buddy_menu_chat_success(chat, model, runtime, selected_channel);
    Ok(true)
}

pub(in super::super) fn apply_buddy_freechat_frame(
    frame: &DecodedFrame,
    model: &BuddyUiModel,
    runtime: &mut RuntimeStatus,
    selected_channel: ChatChannel,
) -> Result<bool, String> {
    let packet = decode_buddy_freechat_packet_0104(frame.packet_type, &frame.payload)
        .map_err(|error| format!("malformed protocol-0104 Buddy FreeChat packet: {error}"))?;
    let Some(packet) = packet else {
        return Ok(false);
    };
    let BuddyFreeChatPacket0104::Success(chat) = packet else {
        return Ok(true);
    };
    apply_buddy_freechat_success(chat, model, runtime, selected_channel);
    Ok(true)
}
