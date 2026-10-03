use super::*;

pub(in super::super) fn apply_freechat_success(
    chat: FreeChatSuccess0104,
    runtime: &mut RuntimeStatus,
    remote_sender_name: Option<String>,
    sender_blocked: bool,
) -> bool {
    // Exact `CnGuiChat.ReceiveChat`: a user with special-state bit 64 set
    // discards normal FreeChat, and an unknown sender is not fabricated.
    if !runtime.free_chat || sender_blocked {
        return false;
    }
    let sender_name = if runtime.player_id == Some(chat.pc_id) {
        (!runtime.player_name.is_empty()).then(|| runtime.player_name.clone())
    } else {
        remote_sender_name
    };
    let Some(sender_name) = sender_name else {
        return false;
    };
    // Clean `AddChatString(0, ...)` writes only to ALL and adds the localized
    // bracketed Local label. Player-authored values enter only as arguments.
    if runtime.player_id == Some(chat.pc_id) {
        runtime.chat.outgoing_audio_pending += 1;
    }
    push_world_chat_line(
        runtime,
        local_freechat_line(sender_name, chat.message.to_string_lossy()),
    );
    true
}

pub(super) fn apply_avatar_emote_code(
    pc_id: i32,
    emote_code: i32,
    runtime: &RuntimeStatus,
    remote_animations: &mut Query<(&NetworkRemotePc0104, &mut RemoteAnimation)>,
    player_commands: &mut TutorialPlayerPresentationCommandQueue,
    local_vehicle: &LocalVehiclePresentationRuntime,
) -> Result<bool, String> {
    let Some(clip) = TutorialPlayerClip::from_avatar_emote_code(emote_code) else {
        // Clean AvatarEmote(int) has no default branch: unknown codes are
        // accepted at the protocol boundary but do not claim a clip.
        return Ok(false);
    };
    if runtime.player_id == Some(pc_id) {
        if !player_commands.accept_avatar_emote_echo(emote_code) {
            return Ok(false);
        }
        if local_vehicle.family != LegacyVehiclePresentationFamily::None {
            return Ok(false);
        }
        let protocol_gender = runtime
            .player_gender
            .ok_or_else(|| "local AvatarEmote has no authoritative player gender".to_owned())?;
        let protocol_gender = i8::try_from(protocol_gender).map_err(|_| {
            format!("local AvatarEmote gender {protocol_gender} is outside protocol i8")
        })?;
        let gender = tutorial_player_gender_from_protocol(protocol_gender)
            .map_err(|error| error.to_string())?;
        player_commands
            .avatar_emote_code(gender, emote_code)
            .ok_or_else(|| format!("AvatarEmote code {emote_code} has no exact clip"))?;
        return Ok(true);
    }
    for (remote, mut animation) in remote_animations.iter_mut() {
        if remote.pc_id == pc_id {
            animation.state = RemoteAnimationState::Emoting { clip };
            return Ok(true);
        }
    }
    Ok(false)
}

pub(in super::super) fn apply_all_group_freechat_success(
    chat: AllGroupFreeChatSuccess0104,
    runtime: &mut RuntimeStatus,
    roster_sender_name: Option<String>,
    selected_channel: ChatChannel,
    sender_blocked: bool,
) {
    if !runtime.free_chat || sender_blocked {
        return;
    }
    let Some(sender_name) = roster_sender_name else {
        return;
    };
    let message = chat.message.to_string_lossy();
    // Clean `AddChatString(3, ...)` keeps the unprefixed row in GROUP and
    // mirrors it into ALL with the localized bracketed Group label.
    if runtime.player_id == Some(chat.sender_pc_id) {
        runtime.chat.outgoing_audio_pending += 1;
    }
    push_world_chat_history(
        &mut runtime.chat.world_group_chat_lines,
        group_freechat_line(false, sender_name.clone(), message.clone()),
    );
    push_world_chat_line(runtime, group_freechat_line(true, sender_name, message));
    if selected_channel != ChatChannel::Group {
        runtime.chat.world_chat_alerts[ChatChannel::Group.index()] = true;
    }
}

pub(in super::super) fn apply_buddy_list_info(
    list: ffone_protocol::BuddyListInfo0104,
    model: &mut BuddyUiModel,
    runtime: &mut RuntimeStatus,
    selected_channel: ChatChannel,
) -> Result<(), String> {
    if runtime.player_id != Some(list.pc_id) || runtime.roster.selected_uid != Some(list.pc_uid) {
        return Err(format!(
            "buddy list identity mismatch: packet ({}, {}) != local ({:?}, {:?})",
            list.pc_id, list.pc_uid, runtime.player_id, runtime.roster.selected_uid
        ));
    }
    let start = usize::try_from(list.list_num)
        .map_err(|_| format!("negative buddy list start {}", list.list_num))?;
    for (offset, entry) in list.buddies.into_iter().enumerate() {
        let slot = start + offset;
        let pc_uid = entry.pc_uid;
        let notice = model
            .set_entry(slot, Some(buddy_entry_from_protocol(entry)))
            .map_err(|error| error.to_string())?;
        runtime
            .chat
            .world_buddy_chat_by_uid
            .entry(pc_uid)
            .or_default();
        if let Some(notice) = notice {
            push_buddy_presence_notice(runtime, notice, selected_channel);
        }
    }
    model.reset_refresh_timer();
    Ok(())
}

pub(in super::super) fn apply_buddy_freechat_success(
    chat: BuddyFreeChatSuccess0104,
    model: &BuddyUiModel,
    runtime: &mut RuntimeStatus,
    selected_channel: ChatChannel,
) {
    if !runtime.free_chat {
        return;
    }
    let Some(local_pc_uid) = runtime.roster.selected_uid else {
        return;
    };
    let incoming = chat.from_pc_uid != local_pc_uid;
    let (sender_name, target_uid) = if !incoming {
        (
            (!runtime.player_name.is_empty()).then(|| runtime.player_name.clone()),
            chat.to_pc_uid,
        )
    } else if chat.to_pc_uid == local_pc_uid {
        (
            buddy_target_by_uid(model, chat.from_pc_uid).and_then(|target| {
                model
                    .slot(target.slot)
                    .filter(|entry| !entry.blocked)
                    .map(BuddyEntry::display_name)
            }),
            chat.from_pc_uid,
        )
    } else {
        return;
    };
    if buddy_target_by_uid(model, target_uid).is_none() {
        return;
    }
    let Some(sender_name) = sender_name else {
        return;
    };
    let message = chat.message.to_string_lossy();
    // AddChatString(2) plays Incoming_Tell; the receive method then plays
    // its own incoming/outgoing cue, even while the Buddy tab is selected.
    runtime.chat.pending_social_sfx.extend([
        "Incoming_Tell",
        if incoming { "Incoming_Tell" } else { "Outgoing_Chat" },
    ]);
    let line = buddy_freechat_line(false, sender_name.clone(), message.clone());
    push_world_chat_history(&mut runtime.chat.world_buddy_chat_lines, line.clone());
    push_world_chat_history(
        runtime
            .chat
            .world_buddy_chat_by_uid
            .entry(target_uid)
            .or_default(),
        line.clone(),
    );
    push_world_chat_line(runtime, buddy_freechat_line(true, sender_name, message));
    if incoming && selected_channel != ChatChannel::Buddy {
        runtime.chat.world_chat_alerts[ChatChannel::Buddy.index()] = true;
    }
}

pub(super) fn apply_buddy_menu_chat_success(
    chat: BuddyFreeChatSuccess0104,
    model: &BuddyUiModel,
    runtime: &mut RuntimeStatus,
    selected_channel: ChatChannel,
) {
    let Some(local_pc_uid) = runtime.roster.selected_uid else {
        return;
    };
    let incoming = chat.from_pc_uid != local_pc_uid;
    let (sender_name, target_uid) = if !incoming {
        (
            (!runtime.player_name.is_empty()).then(|| runtime.player_name.clone()),
            chat.to_pc_uid,
        )
    } else if chat.to_pc_uid == local_pc_uid {
        (
            buddy_target_by_uid(model, chat.from_pc_uid).and_then(|target| {
                model
                    .slot(target.slot)
                    .filter(|entry| !entry.blocked)
                    .map(BuddyEntry::display_name)
            }),
            chat.from_pc_uid,
        )
    } else {
        return;
    };
    if buddy_target_by_uid(model, target_uid).is_none() {
        return;
    }
    let Some(sender_name) = sender_name else {
        return;
    };
    let message = chat.message.to_string_lossy();
    runtime.chat.pending_social_sfx.extend([
        "Incoming_Tell",
        if incoming { "Incoming_Tell" } else { "Outgoing_Chat" },
    ]);
    let line = buddy_freechat_line(false, sender_name.clone(), message.clone());
    push_world_chat_history(&mut runtime.chat.world_buddy_chat_lines, line.clone());
    push_world_chat_history(
        runtime
            .chat
            .world_buddy_chat_by_uid
            .entry(target_uid)
            .or_default(),
        line,
    );
    push_world_chat_line(runtime, buddy_freechat_line(true, sender_name, message));
    if incoming && selected_channel != ChatChannel::Buddy {
        runtime.chat.world_chat_alerts[ChatChannel::Buddy.index()] = true;
    }
}

pub(in super::super) fn sync_buddy_nanocom_context(
    state: Res<State<ClientState>>,
    gameplay_loading: Res<GameplayLoadingState>,
    gameplay_ui: Res<GameplayUiModel>,
    mission_ui: Res<MissionUiModel>,
    presentation: Res<TutorialChoreographyPresentation>,
    mut tutorial_nanocom_context: ResMut<TutorialNanocomMessageContext>,
    modals: GameplayUiModalInputs,
    mut nanocom_messages: ResMut<NanocomMessageUiModel>,
) {
    let gameplay_active = matches!(state.get(), ClientState::Tutorial | ClientState::World);
    let guide_modal = modals.guide_production.modal_active(&modals.guide_ui)
        || modals.bank_state.phase != BankLifecyclePhase::Hidden
        || modals.bank_production.modal_active()
        || modals.vendor_state.phase != VendorLifecyclePhase::Hidden
        || modals.vendor_production.modal_active()
        || modals.rule_runtime.modal_active(&modals.rule_ui)
        || modals
            .nano_free_tuning_production
            .modal_active(&modals.nano_free_tuning)
        || modals.upsell_ui.visible()
        || modals.user_equip_ui.is_active()
        || modals.pc2pc_ui.state.phase.renders_shell()
        || modals.world_map.model.phase() != WorldMapPhase::Closed
        || modals.transportation_modal()
        || modals.email_runtime.modal_active()
        || modals.combi_runtime.modal_active()
        || modals.cashmall_modal()
        || modals.user_store_modal();
    let mission_modal = mission_ui.npc_icon_mode_visible
        || !matches!(mission_ui.journal, MissionJournalUi::Hidden)
        || mission_ui.system_popup_active()
        || mission_ui.nanocom_foreign_modal_suppressed();
    let suspended = !gameplay_active
        || gameplay_loading.visible
        || (*state.get() == ClientState::Tutorial && presentation.event_scene)
        || guide_modal
        || mission_modal;
    nanocom_messages.set_ui_scale(gameplay_ui.ui_scale);
    // Preserve the head and its remaining lifetime while a dialogue owns the
    // screen. The existing suspension path also defers reveal/voice sounds.
    nanocom_messages.set_scene_event_active(suspended);
    nanocom_messages.set_expanded(!suspended && mission_ui.nanocom_main_menu_visible);
    tutorial_nanocom_context.scene_event_active =
        suspended || *state.get() != ClientState::Tutorial;
}

pub(in super::super) fn sync_buddy_ui_context(
    state: Res<State<ClientState>>,
    mut gameplay_ui: ResMut<GameplayUiModel>,
    chat_transition: Option<Res<GameplayMenuTransition>>,
    group_ui: Res<GroupUiModel>,
    modals: GameplayUiModalInputs,
    local_player: Query<&LegacyPlayerController, With<LocalPlayer>>,
    mut buddy_ui: ResMut<BuddyUiModel>,
) {
    let in_world = *state.get() == ClientState::World;
    let chat_controls_visible = gameplay_ui.chat.active
        || chat_transition
            .as_deref()
            .is_some_and(|transition| transition.visible_or_transitioning());
    let buddy_count = buddy_ui.visible_count();
    if gameplay_ui.chat.buddy_count != buddy_count {
        gameplay_ui.chat.buddy_count = buddy_count;
    }
    buddy_ui.set_visible(
        in_world
            && !modals.guide_production.modal_active(&modals.guide_ui)
            && modals.bank_state.phase == BankLifecyclePhase::Hidden
            && !modals.bank_production.modal_active()
            && modals.vendor_state.phase == VendorLifecyclePhase::Hidden
            && !modals.vendor_production.modal_active()
            && !modals.rule_runtime.modal_active(&modals.rule_ui)
            && !modals
                .nano_free_tuning_production
                .modal_active(&modals.nano_free_tuning)
            && !modals.upsell_ui.visible()
            && modals.world_map.model.phase() == WorldMapPhase::Closed
            && !modals.pc2pc_ui.state.phase.renders_shell()
            && !modals.transportation_modal()
            && !modals.email_runtime.modal_active()
            && !modals.combi_runtime.modal_active()
            && !modals.cashmall_modal()
            && !modals.user_store_modal()
            && gameplay_ui.chat.selected == ChatChannel::Buddy
            && chat_controls_visible,
    );
    buddy_ui.set_chat_window_style(BuddyChatWindowStyle::Large);
    buddy_ui.set_large_chat_width(gameplay_ui.chat.window_size.x);
    buddy_ui.set_quick_slot_active(false);
    buddy_ui.set_ui_scale(gameplay_ui.ui_scale);
    buddy_ui.set_player_moving(
        local_player
            .single()
            .is_ok_and(|controller| controller.velocity.length_squared() > 0.000_001),
    );
    buddy_ui.set_group(
        group_ui.pc_members.len().max(1),
        group_ui.pc_members.iter().map(|member| member.pc_uid),
    );
}
