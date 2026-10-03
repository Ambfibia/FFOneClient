use super::*;

pub(in super::super) fn combi_table_i32_0104(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &'static str,
    context: &str,
) -> Result<i32, String> {
    let value = object
        .get(field)
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| format!("{context}.{field} is not an integer"))?;
    i32::try_from(value).map_err(|_| format!("{context}.{field} does not fit i32"))
}

pub(in super::super) fn combi_table_i32_default_zero_0104(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &'static str,
    context: &str,
) -> Result<i32, String> {
    let Some(value) = object.get(field) else {
        return Ok(0);
    };
    let value = value
        .as_i64()
        .ok_or_else(|| format!("{context}.{field} is not an integer"))?;
    i32::try_from(value).map_err(|_| format!("{context}.{field} does not fit i32"))
}

pub(in super::super) fn combi_table_string_0104<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &'static str,
    context: &str,
) -> Result<&'a str, String> {
    object
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("{context}.{field} is not a string"))
}

pub(in super::super) fn combi_player_authority_0104(
    runtime: &RuntimeStatus,
    guide: &GuideRuntime,
) -> Result<CombiPlayerAuthority0104, String> {
    let owner_pc_id = runtime
        .player_id
        .ok_or_else(|| "Combi has no authoritative local PC ID".to_owned())?;
    let gender = runtime
        .player_gender
        .filter(|gender| matches!(gender, 1 | 2))
        .ok_or_else(|| "Combi has no supported authoritative player gender".to_owned())?;
    let guide = guide
        .authoritative()
        .map(|state| i32::from(state.raw_mentor()))
        .ok_or_else(|| "Combi has no authoritative Guide state".to_owned())?;
    Ok(CombiPlayerAuthority0104 {
        owner_pc_id,
        gender,
        level: i32::from(runtime.player_level),
        guide,
        taros: runtime.candy,
    })
}

pub(in super::super) fn reset_combi_shell_0104(
    commands: &mut Commands,
    runtime: &mut CombiProductionRuntime0104,
    state: &mut CombiUiState0104,
    projection: &mut CombiModeProjection0104,
    outbox: &mut CombiUiOutbox0104,
    shell: &mut CombiProductionShell0104,
    frames: &mut CombiNetworkFrameInbox0104,
    messages: &mut SystemMessageUiModel,
) {
    runtime.reset();
    *state = CombiUiState0104::default();
    *projection = CombiModeProjection0104::default();
    while outbox.pop_front().is_some() {}
    frames.clear();
    shell.reset(commands, messages);
}

pub(in super::super) fn consume_combi_network_frames_0104(
    mut inbox: ResMut<CombiNetworkFrameInbox0104>,
    mut runtime: ResMut<CombiProductionRuntime0104>,
    mut shell: ResMut<CombiProductionShell0104>,
    mut status: ResMut<RuntimeStatus>,
) {
    while let Some(frame) = inbox.pop_front() {
        let packet_type = frame.packet_type;
        match runtime.apply_frame(frame) {
            Ok(Some(output)) => shell.push_output(output),
            Ok(None) => {
                status.message = format!(
                    "CombiMode rejected an owned packet {packet_type:#010x} as passthrough"
                );
            }
            Err(error) => {
                // A malformed owned reply cannot be passed to a generic
                // inventory consumer and cannot release the pending lease.
                status.message = format!("CombiMode authoritative reply rejected: {error}");
            }
        }
    }
}

pub(in super::super) fn drive_combi_production_0104(
    time: Res<Time>,
    catalog: Res<CombiProductionCatalog0104>,
    inventory: Res<LocalInventoryRuntime>,
    guide: Res<GuideRuntime>,
    system_messages: Res<SystemMessageUiModel>,
    mission_ui: Res<MissionUiModel>,
    user_equip_modal: Res<UserEquipModalState>,
    mut state: ResMut<CombiUiState0104>,
    mut runtime: ResMut<CombiProductionRuntime0104>,
    mut outbox: ResMut<CombiUiOutbox0104>,
    mut shell: ResMut<CombiProductionShell0104>,
    mut status: ResMut<RuntimeStatus>,
) {
    state.external_modal = CombiExternalModalState0104 {
        system_popup: system_messages.is_popup() || mission_ui.system_popup_active(),
        help: user_equip_modal.help_active,
        inventory_popup_modal: user_equip_modal.inventory_popup_modal,
        popup_controller: user_equip_modal.item_popup_active,
    };
    if !runtime.modal_active() {
        if outbox.pop_front().is_some() {
            while outbox.pop_front().is_some() {}
            status.message =
                "CombiMode discarded UI input without an active production session".to_owned();
        }
        return;
    }
    let player = match combi_player_authority_0104(&status, &guide) {
        Ok(player) => player,
        Err(error) => {
            status.message = format!("CombiMode authority unavailable: {error}");
            return;
        }
    };
    let runtime_catalog = RuntimeCombiItemCatalog0104 {
        source: &catalog,
        player,
    };
    if let Some(snapshot) = inventory.snapshot()
        && let Err(error) =
            runtime.refresh_authority(snapshot, player, &runtime_catalog, &catalog.recipes)
        && !matches!(
            error,
            CombiProductionError0104::AuthorityRefreshBlocked { .. }
        )
    {
        status.message = format!("CombiMode authority refresh rejected: {error}");
    }

    while let Some(result) =
        runtime.apply_next_ui_command(&mut outbox, &runtime_catalog, &catalog.recipes)
    {
        match result {
            Ok(output) => shell.push_output(output),
            Err(error) => status.message = format!("CombiMode UI command rejected: {error}"),
        }
    }
    if runtime
        .session()
        .is_some_and(|session| matches!(session.machine().phase, CombiPhase0104::Waiting { .. }))
    {
        match runtime.tick(time.delta_secs().max(0.0)) {
            Ok(output) => shell.push_output(output),
            Err(error) => status.message = format!("CombiMode wait rejected: {error}"),
        }
    }
}

pub(in super::super) fn queue_combi_system_message_0104(
    shell: &mut CombiProductionShell0104,
    messages: &mut SystemMessageUiModel,
    content: &TutorialMissionContent,
    effect: CombiSystemMessage0104,
) -> Result<u64, String> {
    let request_id = shell.next_request_id();
    let (localized, button_type, action) = match effect {
        CombiSystemMessage0104::Modal { modal, .. } => {
            let localized = modal.localized_text();
            let definition = content
                .system_message_definition(modal.message_id())
                .ok_or_else(|| {
                    format!(
                        "Combi SystemMessage {} has no source TableData row",
                        modal.message_id()
                    )
                })?;
            if definition.exact_text != localized.fallback {
                return Err(format!(
                    "Combi SystemMessage {} fallback differs from source TableData",
                    modal.message_id()
                ));
            }
            (
                localized,
                definition.runtime_button_type,
                PendingCombiSystemAction0104::Resolve(modal),
            )
        }
        CombiSystemMessage0104::EquippedItem { message_id, .. } => {
            let localized = localized_combi_equipped_item_message();
            let definition = content
                .system_message_definition(message_id)
                .ok_or_else(|| {
                    format!("Combi SystemMessage {message_id} has no source TableData row")
                })?;
            if definition.exact_text != localized.fallback {
                return Err(format!(
                    "Combi SystemMessage {message_id} fallback differs from source TableData"
                ));
            }
            (
                localized,
                definition.runtime_button_type,
                PendingCombiSystemAction0104::Informational,
            )
        }
        CombiSystemMessage0104::WireFailure { error_code } => (
            localized_combi_wire_error(error_code),
            SystemMessageButtonType::Ok,
            PendingCombiSystemAction0104::Informational,
        ),
    };
    let request = SystemMessageRequest::new_localized(request_id, localized, button_type);
    shell.pending_system_messages.insert(
        request_id,
        PendingCombiSystemMessage0104 {
            request: request.clone(),
            action,
        },
    );
    messages.push(request);
    Ok(request_id)
}

pub(in super::super) fn requeue_combi_system_message_0104(
    shell: &mut CombiProductionShell0104,
    messages: &mut SystemMessageUiModel,
    pending: PendingCombiSystemMessage0104,
) {
    let request_id = pending.request.request_id;
    messages.push(pending.request.clone());
    shell.pending_system_messages.insert(request_id, pending);
}

pub(in super::super) fn consume_combi_system_message_outbox_0104(
    mut outbox: ResMut<SystemMessageUiOutbox>,
    mut messages: ResMut<SystemMessageUiModel>,
    catalog: Res<CombiProductionCatalog0104>,
    guide: Res<GuideRuntime>,
    mut runtime: ResMut<CombiProductionRuntime0104>,
    mut shell: ResMut<CombiProductionShell0104>,
    mut status: ResMut<RuntimeStatus>,
) {
    if shell.pending_system_messages.is_empty() {
        return;
    }
    let owned = shell
        .pending_system_messages
        .keys()
        .copied()
        .collect::<BTreeSet<_>>();
    for action in outbox.drain_matching(|action| match action {
        SystemMessageUiAction::Chosen { request_id, .. } => owned.contains(request_id),
    }) {
        let SystemMessageUiAction::Chosen {
            request_id,
            button_type,
            choice,
        } = action;
        let Some(pending) = shell.pending_system_messages.remove(&request_id) else {
            continue;
        };
        if button_type != pending.request.button_type {
            status.message =
                format!("CombiMode rejected mismatched SystemMessage type {button_type:?}");
            requeue_combi_system_message_0104(&mut shell, &mut messages, pending);
            continue;
        }
        let result = match pending.action {
            PendingCombiSystemAction0104::Informational => Ok(CombiProductionOutput0104::default()),
            PendingCombiSystemAction0104::Resolve(modal) => {
                let modal_choice = match (modal, choice) {
                    (CombiSystemModal0104::AttemptConfirmation, SystemMessageChoice::Primary) => {
                        Some(CombiModalChoice0104::Continue)
                    }
                    (CombiSystemModal0104::AttemptConfirmation, SystemMessageChoice::Secondary) => {
                        Some(CombiModalChoice0104::Cancel)
                    }
                    (
                        CombiSystemModal0104::NotEnoughTaros
                        | CombiSystemModal0104::CombinationFailed,
                        SystemMessageChoice::Primary,
                    ) => Some(CombiModalChoice0104::Ok),
                    _ => None,
                };
                let Some(modal_choice) = modal_choice else {
                    status.message =
                        format!("CombiMode rejected unavailable {choice:?} choice for {modal:?}");
                    requeue_combi_system_message_0104(&mut shell, &mut messages, pending);
                    continue;
                };
                let player = match combi_player_authority_0104(&status, &guide) {
                    Ok(player) => player,
                    Err(error) => {
                        status.message = format!("CombiMode modal authority unavailable: {error}");
                        requeue_combi_system_message_0104(&mut shell, &mut messages, pending);
                        continue;
                    }
                };
                let runtime_catalog = RuntimeCombiItemCatalog0104 {
                    source: &catalog,
                    player,
                };
                runtime.resolve_modal(modal_choice, &runtime_catalog, &catalog.recipes)
            }
        };
        match result {
            Ok(output) => shell.push_output(output),
            Err(error) => {
                status.message = format!("CombiMode SystemMessage transition rejected: {error}");
                requeue_combi_system_message_0104(&mut shell, &mut messages, pending);
            }
        }
    }
}

pub(in super::super) fn restore_combi_cursor_0104(
    cursors: &mut Query<&mut CursorOptions, With<PrimaryWindow>>,
    locked: bool,
) -> Result<(), String> {
    let mut cursor = cursors
        .single_mut()
        .map_err(|_| "CombiMode has no unique production cursor owner".to_owned())?;
    cursor.grab_mode = if locked {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
    cursor.visible = !locked;
    Ok(())
}

pub(in super::super) fn reset_combi_after_fault_0104(commands: &mut Commands, owners: &mut CombiProductionOwners0104) {
    let cursor_was_locked = owners.shell.cursor_was_locked.unwrap_or(true);
    reset_combi_shell_0104(
        commands,
        &mut owners.runtime,
        &mut owners.state,
        &mut owners.projection,
        &mut owners.outbox,
        &mut owners.shell,
        &mut owners.frames,
        &mut owners.system_messages,
    );
    if let Err(error) = restore_combi_cursor_0104(&mut owners.cursors, cursor_was_locked) {
        owners.status.message = error;
    }
}

pub(in super::super) fn consume_combi_production_outputs_0104(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    bridge: Res<NetworkBridge>,
    mut owners: CombiProductionOwners0104,
) {
    let effects_gain = option_channel_gain(owners.option_runtime.options.sound.effects);
    while let Some(mut output) = owners.shell.pending_outputs.pop_front() {
        if let Some(request) = output.request.take()
            && let Err(error) = bridge.send(NetworkCommand::SendRegisteredGameplay0104(request))
        {
            owners.status.message = format!("CombiMode request send failed: {error}");
            reset_combi_after_fault_0104(&mut commands, &mut owners);
            continue;
        }
        if let Some(commit) = output.commit.take() {
            let result = if owners.status.player_id == Some(commit.owner_pc_id()) {
                owners
                    .inventory
                    .snapshot()
                    .ok_or_else(|| "CombiMode commit has no global inventory owner".to_owned())
                    .and_then(|current| combi_inventory_after_commit_0104(current, &commit))
            } else {
                Err(format!(
                    "CombiMode commit owner {} differs from active local PC {:?}",
                    commit.owner_pc_id(),
                    owners.status.player_id
                ))
            };
            match result {
                Ok(next) => {
                    // Inventory and Taros publish together only after the
                    // entire receipt and global post-state have validated.
                    owners.inventory.snapshot = Some(next);
                    owners.status.candy = commit.taros_after();
                }
                Err(error) => {
                    owners.status.message = format!("CombiMode commit rejected: {error}");
                    reset_combi_after_fault_0104(&mut commands, &mut owners);
                    continue;
                }
            }
        }
        for effect in output.effects {
            match effect {
                CombiShellEffect0104::ModeOpened { context, lease } => {
                    owners.shell.lease = Some(lease);
                    owners.shell.inventory_mode_active = lease.game_mode == 25
                        && lease.active_inventory_tab == 0
                        && !lease.cursor_locked_during_mode;
                    owners.shell.camera_npc_table_id = Some(lease.camera_npc_id);
                    owners
                        .shell
                        .first_use_checks
                        .push(lease.first_use_condition_checked_by_service_menu);
                    if let Err(error) = restore_combi_cursor_0104(&mut owners.cursors, false) {
                        owners.status.message = error;
                    }
                    owners.shell.stop_ui_mode_audio(&mut commands);
                    match spawn_combi_sfx_0104(
                        &mut commands,
                        &asset_server,
                        &owners.audio_catalog,
                        effects_gain * COMBI_UI_MODE_SOUND_GAIN_0104,
                        COMBI_UI_MODE_SOUND_TRUE_NAME_0104,
                        true,
                    ) {
                        Ok(entity) => owners.shell.ui_mode_audio = Some(entity),
                        Err(error) => owners.status.message = error,
                    }
                    owners.status.message = format!(
                        "CombiMode 25 opened from category-26 runtime NPC {} (table {}); live NPC portrait requested",
                        context.source.runtime_npc_id, context.source.table_npc_id
                    );
                }
                CombiShellEffect0104::DragCaptured { inventory_index } => {
                    owners.shell.last_drag_inventory_index = Some(inventory_index);
                }
                CombiShellEffect0104::SelectionChanged(change) => {
                    owners.shell.last_selection_change = Some(change);
                }
                CombiShellEffect0104::SystemMessage(message) => {
                    if let Err(error) = queue_combi_system_message_0104(
                        &mut owners.shell,
                        &mut owners.system_messages,
                        &owners.content,
                        message,
                    ) {
                        owners.status.message = error;
                    }
                }
                CombiShellEffect0104::NpcAnimation { npc_id, animation } => {
                    owners.shell.last_animation = Some((npc_id, animation));
                    owners.status.message = COMBI_ANIMATION_OWNER_GAP_0104.to_owned();
                }
                CombiShellEffect0104::PlaySuccessSound { true_name } => {
                    if let Err(error) = spawn_combi_sfx_0104(
                        &mut commands,
                        &asset_server,
                        &owners.audio_catalog,
                        0.7 * effects_gain,
                        true_name,
                        false,
                    ) {
                        owners.status.message = error;
                    }
                }
                CombiShellEffect0104::HelpRequested => {
                    owners.status.message = COMBI_HELP_OWNER_GAP_0104.to_owned();
                }
                CombiShellEffect0104::ModeClosed { context, reason } => {
                    queue_service_farewell(&mut commands, context.source.runtime_npc_id);
                    if let Err(error) = spawn_combi_sfx_0104(
                        &mut commands,
                        &asset_server,
                        &owners.audio_catalog,
                        0.7 * effects_gain,
                        "Close_Screen",
                        false,
                    ) {
                        owners.status.message = error;
                    }
                    owners
                        .shell
                        .clear_owned_messages(&mut owners.system_messages);
                    owners.shell.stop_ui_mode_audio(&mut commands);
                    owners.shell.inventory_mode_active = false;
                    owners.shell.camera_npc_table_id = None;
                    owners.shell.lease = None;
                    let cursor_was_locked = owners.shell.cursor_was_locked.take().unwrap_or(true);
                    if let Err(error) =
                        restore_combi_cursor_0104(&mut owners.cursors, cursor_was_locked)
                    {
                        owners.status.message = error;
                    }
                    owners.status.message = format!(
                        "CombiMode closed for runtime NPC {} ({reason:?})",
                        context.source.runtime_npc_id
                    );
                }
                CombiShellEffect0104::GoToMyStuffRequested {
                    game_mode,
                    first_use_condition,
                } => {
                    if game_mode == 6 {
                        owners.user_equip.open_item_mode();
                        owners.shell.first_use_checks.push(first_use_condition);
                    } else {
                        owners.status.message = format!(
                            "CombiMode rejected unsupported successor game mode {game_mode}"
                        );
                    }
                }
            }
        }
    }
}
