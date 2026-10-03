//! Email production drive, system messages, SFX, output owners, update polling and new-mail notices.

use super::combi::restore_combi_cursor_0104;
use super::email_catalog::{EMAIL_UI_MODE_SOUND_GAIN_0104, EMAIL_UI_MODE_SOUND_TRUE_NAME_0104};
use super::email_production::{
    EmailProductionDrive0104, EmailProductionShell0104, PendingEmailSystemAction0104,
    PendingEmailSystemMessage0104, dispatch_email_request_0104, email_inventory_after_commit_0104,
    synchronize_email_authority_0104,
};
use super::local_inventory::LocalInventoryRuntime;
use super::option_runtime::{
    OptionProductionRuntime, clear_option_action_press, option_action_just_pressed,
    option_channel_gain,
};
use super::runtime_status::RuntimeStatus;
use super::{LocalPlayer, WorldSliceEntity};
use bevy::{
    audio::Volume,
    ecs::system::SystemParam,
    prelude::*,
    window::{CursorOptions, PrimaryWindow},
};
use ffone_client::{
    buddy_ui::{BUDDY_MAX_SLOTS, BuddyTarget, BuddyUiAction, BuddyUiModel, BuddyUiOutbox},
    email_runtime::{EmailProductionRuntime0104, email_inventory_authority_0104},
    email_ui::{
        EmailCloseSource, EmailNetworkRuntime0104, EmailTransportOutbox, EmailUiAction,
        EmailUiAudioCue, EmailUiAudioOutbox, EmailUiModel, EmailUiOutbox, confirm_delete_email,
    },
    gameplay_ui::GameplayUiModel,
    localization::LocalizedText,
    mission_ui::MissionUiModel,
    movement::LegacyWorldColliderPending,
    network::{NetworkBridge, NetworkCommand},
    option_ui::LegacyOptionAction,
    semantic_audio::{NativeAudioCatalog, NativeAudioCategory},
    system_message_ui::{
        SystemMessageButtonType, SystemMessageChoice, SystemMessageRequest, SystemMessageUiAction,
        SystemMessageUiModel, SystemMessageUiOutbox,
    },
    tutorial_mission_content::TutorialMissionContent,
};
use std::collections::VecDeque;

pub(super) fn drive_email_production_pre_interaction_0104(
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    option_runtime: Res<OptionProductionRuntime>,
    mut owners: EmailProductionDrive0104,
) {
    synchronize_email_authority_0104(&mut owners);
    let close = [
        (LegacyOptionAction::Escape, EmailCloseSource::Escape),
        (LegacyOptionAction::Email, EmailCloseSource::EmailKey),
    ]
    .into_iter()
    .find(|(action, _)| {
        option_action_just_pressed(&option_runtime.input, *action, &keyboard, &mouse)
    });
    if owners.runtime.modal_active()
        && let Some((action, source)) = close
    {
        clear_option_action_press(&option_runtime.input, action, &mut keyboard, &mut mouse);
        match owners.runtime.request_close(
            source,
            &mut owners.model,
            &mut owners.actions,
            &mut owners.audio,
        ) {
            Ok(Some(output)) => owners.shell.push_output(output),
            Ok(None) => {}
            Err(error) => owners.status.message = format!("Email hotkey close rejected: {error}"),
        }
    }
    dispatch_email_request_0104(&mut owners);
}

pub(super) fn adopt_email_raw_close_intent_0104(owners: &mut EmailProductionDrive0104) {
    let mut retained = VecDeque::new();
    let mut raw_close = None;
    while let Some(action) = owners.actions.pop() {
        match action {
            EmailUiAction::RequestEscapeCloseGate {
                event_group,
                event_function,
            } if raw_close.is_none() => {
                raw_close = Some((EmailCloseSource::Escape, event_group, event_function));
            }
            EmailUiAction::QueryComputressExitGate {
                event_group,
                event_function,
            } if raw_close.is_none() => {
                raw_close = Some((
                    EmailCloseSource::RightPanelClose,
                    event_group,
                    event_function,
                ));
            }
            action => retained.push_back(action),
        }
    }
    owners.actions.0 = retained;
    let Some((source, event_group, event_function)) = raw_close else {
        return;
    };
    let route_is_exact = match source {
        EmailCloseSource::Escape => event_group == 2 && event_function == 24,
        EmailCloseSource::RightPanelClose => event_group == 11 && event_function == 13,
        EmailCloseSource::EmailKey => false,
    };
    if !route_is_exact {
        owners.status.message =
            format!("Email ignored non-clean close gate ({event_group},{event_function})");
        return;
    }
    owners.model.escape_gate_pending = false;
    owners.model.computress_query_pending = false;
    match owners.runtime.request_close(
        source,
        &mut owners.model,
        &mut owners.actions,
        &mut owners.audio,
    ) {
        Ok(Some(output)) => owners.shell.push_output(output),
        Ok(None) => {}
        Err(error) => owners.status.message = format!("Email close intent rejected: {error}"),
    }
}

pub(super) fn drive_email_production_post_interaction_0104(mut owners: EmailProductionDrive0104) {
    if !owners.runtime.modal_active() {
        if !owners.actions.0.is_empty()
            || !owners.audio.0.is_empty()
            || !owners.transport.0.is_empty()
        {
            owners.actions.0.clear();
            owners.audio.0.clear();
            owners.transport.0.clear();
            owners.status.message =
                "Email discarded preview input without a production mode lease".to_owned();
        }
        return;
    }
    adopt_email_raw_close_intent_0104(&mut owners);
    if !owners.actions.0.is_empty() || !owners.audio.0.is_empty() {
        match owners
            .runtime
            .collect_effects(&mut owners.actions, &mut owners.audio)
        {
            Ok(output) => owners.shell.push_output(output),
            Err(error) => owners.status.message = format!("Email effects rejected: {error}"),
        }
    }
    dispatch_email_request_0104(&mut owners);
}

pub(super) fn email_buddy_target_by_names_0104(
    buddy: &BuddyUiModel,
    first_name: &str,
    last_name: &str,
) -> Result<BuddyTarget, String> {
    let matches = (0..BUDDY_MAX_SLOTS)
        .filter_map(|slot| {
            buddy
                .slot(slot)
                .filter(|entry| {
                    !entry.blocked && entry.first_name == first_name && entry.last_name == last_name
                })
                .map(|entry| BuddyTarget {
                    slot,
                    pc_uid: entry.pc_uid,
                })
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [target] => Ok(*target),
        [] => Err(format!(
            "Email remove-buddy confirmation has no authoritative {first_name} {last_name} target"
        )),
        _ => Err(format!(
            "Email remove-buddy confirmation has duplicate {first_name} {last_name} targets"
        )),
    }
}

pub(super) fn email_passthrough_copy_0104(text: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", text.into())
}

pub(super) fn queue_email_system_message_0104(
    shell: &mut EmailProductionShell0104,
    messages: &mut SystemMessageUiModel,
    content: &TutorialMissionContent,
    localized: LocalizedText,
    message_id: Option<u16>,
    action: PendingEmailSystemAction0104,
) -> Result<u64, String> {
    let button_type = if let Some(message_id) = message_id {
        content
            .system_message_definition(i32::from(message_id))
            .ok_or_else(|| format!("Email SystemMessage {message_id} has no TableData row"))?
            .runtime_button_type
    } else {
        SystemMessageButtonType::Ok
    };
    let request_id = shell.next_request_id();
    let request = SystemMessageRequest::new_localized(request_id, localized, button_type);
    shell.pending_system_messages.insert(
        request_id,
        PendingEmailSystemMessage0104 {
            request: request.clone(),
            action,
        },
    );
    messages.push(request);
    Ok(request_id)
}

pub(super) fn requeue_email_system_message_0104(
    shell: &mut EmailProductionShell0104,
    messages: &mut SystemMessageUiModel,
    pending: PendingEmailSystemMessage0104,
) {
    let request_id = pending.request.request_id;
    messages.push(pending.request.clone());
    shell.pending_system_messages.insert(request_id, pending);
}

pub(super) fn consume_email_system_message_outbox_0104(
    mut outbox: ResMut<SystemMessageUiOutbox>,
    mut messages: ResMut<SystemMessageUiModel>,
    mut shell: ResMut<EmailProductionShell0104>,
    mut model: ResMut<EmailUiModel>,
    mut transport: ResMut<EmailTransportOutbox>,
    mut buddy_outbox: ResMut<BuddyUiOutbox>,
    mut status: ResMut<RuntimeStatus>,
) {
    if shell.pending_system_messages.is_empty() {
        return;
    }
    let mut unrelated = Vec::new();
    for ui_action in outbox.drain().collect::<Vec<_>>() {
        let SystemMessageUiAction::Chosen {
            request_id,
            button_type,
            choice,
        } = ui_action;
        let Some(pending) = shell.pending_system_messages.remove(&request_id) else {
            unrelated.push(ui_action);
            continue;
        };
        if pending.request.button_type != button_type {
            status.message = format!(
                "Email SystemMessage {request_id} returned stale button type {button_type:?}"
            );
            requeue_email_system_message_0104(&mut shell, &mut messages, pending);
            continue;
        }
        if choice != SystemMessageChoice::Primary {
            continue;
        }
        match pending.action {
            PendingEmailSystemAction0104::Informational => {}
            PendingEmailSystemAction0104::DeleteEmail { email_index } => {
                if !confirm_delete_email(email_index, &mut model, &mut transport) {
                    status.message = format!(
                        "Email delete confirmation {request_id} became stale for mail {email_index}"
                    );
                }
            }
            PendingEmailSystemAction0104::RemoveBuddy { target } => {
                buddy_outbox.push(BuddyUiAction::RemoveRequested(target));
            }
        }
    }
    for action in unrelated {
        outbox.push(action);
    }
}

pub(super) fn spawn_email_sfx_0104(
    commands: &mut Commands,
    asset_server: &AssetServer,
    catalog: &NativeAudioCatalog,
    gain: f32,
    true_name: &str,
    looping: bool,
) -> Result<Entity, String> {
    let candidates = catalog
        .by_true_name(true_name)
        .into_iter()
        .filter(|asset| asset.category == NativeAudioCategory::Sfx)
        .collect::<Vec<_>>();
    let [asset] = candidates.as_slice() else {
        return Err(format!(
            "Email SFX {true_name:?} resolved to {} native assets",
            candidates.len()
        ));
    };
    let settings = if looping {
        PlaybackSettings::LOOP.with_volume(Volume::Linear(gain))
    } else {
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(gain))
    };
    Ok(commands
        .spawn((
            Name::new(format!("EmailMode SFX {true_name}")),
            WorldSliceEntity,
            AudioPlayer::new(asset_server.load(asset.path.clone())),
            ffone_client::audio_channel::GameplayAudioChannel::ui_sfx(),
            settings,
        ))
        .id())
}

#[derive(SystemParam)]
pub(super) struct EmailProductionOutputOwners0104<'w, 's> {
    pub(super) runtime: ResMut<'w, EmailProductionRuntime0104>,
    pub(super) model: ResMut<'w, EmailUiModel>,
    pub(super) actions: ResMut<'w, EmailUiOutbox>,
    pub(super) audio: ResMut<'w, EmailUiAudioOutbox>,
    pub(super) network: ResMut<'w, EmailNetworkRuntime0104>,
    pub(super) shell: ResMut<'w, EmailProductionShell0104>,
    pub(super) inventory: ResMut<'w, LocalInventoryRuntime>,
    pub(super) status: ResMut<'w, RuntimeStatus>,
    pub(super) messages: ResMut<'w, SystemMessageUiModel>,
    pub(super) mission_ui: ResMut<'w, MissionUiModel>,
    pub(super) buddy: Res<'w, BuddyUiModel>,
    pub(super) content: Res<'w, TutorialMissionContent>,
    pub(super) gameplay_ui: ResMut<'w, GameplayUiModel>,
    pub(super) new_mail_alarm: Option<ResMut<'w, ffone_client::gameplay_ui::MinimapNewMailAlarm>>,
    pub(super) option_runtime: Res<'w, OptionProductionRuntime>,
    pub(super) audio_catalog: Res<'w, NativeAudioCatalog>,
    pub(super) cursors: Query<'w, 's, &'static mut CursorOptions, With<PrimaryWindow>>,
}

pub(super) fn consume_email_production_outputs_0104(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    bridge: Res<NetworkBridge>,
    mut owners: EmailProductionOutputOwners0104,
) {
    let effects_gain = option_channel_gain(owners.option_runtime.options.sound.effects);
    while let Some(mut output) = owners.shell.pending_outputs.pop_front() {
        if let Some(request) = output.request.take()
            && let Err(error) = bridge.send(NetworkCommand::SendRegisteredGameplay0104(request))
        {
            if let Err(cancel_error) = owners
                .runtime
                .cancel_pending_transport(&mut owners.model, &mut owners.network)
            {
                owners.status.message = format!(
                    "Email request send failed: {error}; pending cancellation failed: {cancel_error}"
                );
            } else {
                owners.status.message = format!("Email request send failed: {error}");
            }
            continue;
        }
        if let Some(commit) = output.commit.take() {
            let staged = if owners.status.player_id == Some(commit.owner_pc_id()) {
                owners
                    .inventory
                    .snapshot()
                    .ok_or_else(|| "Email commit has no global inventory owner".to_owned())
                    .and_then(|current| email_inventory_after_commit_0104(current, &commit))
                    .and_then(|inventory| {
                        let taros = commit.taros_after().unwrap_or(owners.status.candy);
                        (taros >= 0)
                            .then_some((inventory, taros))
                            .ok_or_else(|| format!("Email commit carries negative Taros {taros}"))
                    })
            } else {
                Err(format!(
                    "Email commit owner {} differs from active local PC {:?}",
                    commit.owner_pc_id(),
                    owners.status.player_id
                ))
            };
            match staged {
                Ok((inventory, taros)) => {
                    owners.inventory.snapshot = Some(inventory);
                    owners.status.candy = taros;
                    owners.shell.last_inventory = owners
                        .inventory
                        .snapshot()
                        .map(email_inventory_authority_0104);
                    if commit.inventory_refresh_required() {
                        owners.status.message =
                            "Email is waiting for the authoritative inventory refresh frame"
                                .to_owned();
                    }
                }
                Err(error) => {
                    owners.status.message = format!("Email atomic commit rejected: {error}");
                    continue;
                }
            }
        }

        for action in output.actions {
            match action {
                EmailUiAction::SetCursorLocked(locked) => {
                    if let Err(error) = restore_combi_cursor_0104(&mut owners.cursors, locked) {
                        owners.status.message = error.replace("CombiMode", "EmailMode");
                    }
                }
                EmailUiAction::StartUiModeSound => {
                    owners.shell.stop_ui_mode_audio(&mut commands);
                    match spawn_email_sfx_0104(
                        &mut commands,
                        &asset_server,
                        &owners.audio_catalog,
                        effects_gain * EMAIL_UI_MODE_SOUND_GAIN_0104,
                        EMAIL_UI_MODE_SOUND_TRUE_NAME_0104,
                        true,
                    ) {
                        Ok(entity) => owners.shell.ui_mode_audio = Some(entity),
                        Err(error) => owners.status.message = error,
                    }
                }
                EmailUiAction::StopUiModeSound => owners.shell.stop_ui_mode_audio(&mut commands),
                EmailUiAction::RefreshGuideEmail {
                    event_group,
                    event_function,
                } => {
                    if (event_group, event_function) != (15, 7) {
                        owners.status.message = format!(
                            "Email rejected non-clean guide refresh ({event_group},{event_function})"
                        );
                    }
                }
                EmailUiAction::SetInventoryMailMode {
                    event_group,
                    event_function,
                    value,
                } => {
                    if (event_group, event_function) == (11, 0) && matches!(value, 4 | 10) {
                        owners.shell.inventory_mail_mode = Some(value);
                    } else {
                        owners.status.message = format!(
                            "Email rejected non-clean inventory mode ({event_group},{event_function})={value}"
                        );
                    }
                }
                EmailUiAction::RequestEscapeCloseGate {
                    event_group,
                    event_function,
                } => {
                    let accepted = (event_group, event_function) == (2, 24)
                        && !owners.messages.is_popup()
                        && !owners.mission_ui.system_popup_active();
                    match owners.runtime.resolve_escape_gate(
                        accepted,
                        &mut owners.model,
                        &mut owners.actions,
                        &mut owners.audio,
                    ) {
                        Ok(Some(next)) => owners.shell.push_output(next),
                        Ok(None) => {}
                        Err(error) => {
                            owners.status.message = format!("Email escape gate rejected: {error}")
                        }
                    }
                }
                EmailUiAction::QueryComputressExitGate {
                    event_group,
                    event_function,
                } => {
                    if (event_group, event_function) != (11, 13) {
                        owners.status.message = format!(
                            "Email rejected non-clean Computress gate ({event_group},{event_function})"
                        );
                        continue;
                    }
                    let computress_active =
                        owners.messages.is_popup() || owners.mission_ui.system_popup_active();
                    match owners.runtime.resolve_computress_gate(
                        computress_active,
                        &mut owners.model,
                        &mut owners.actions,
                        &mut owners.audio,
                    ) {
                        Ok(Some(next)) => owners.shell.push_output(next),
                        Ok(None) => {}
                        Err(error) => {
                            owners.status.message =
                                format!("Email Computress gate rejected: {error}")
                        }
                    }
                }
                EmailUiAction::ExitMode {
                    event_group,
                    event_function,
                } => {
                    if (event_group, event_function) == (2, 1) {
                        owners.shell.lease = None;
                        owners.shell.last_inventory = None;
                        owners.shell.clear_owned_messages(&mut owners.messages);
                        owners.gameplay_ui.chat.input_enabled = true;
                        owners.status.message = "EmailMode 18 returned to MainGame".to_owned();
                        owners.audio.push(EmailUiAudioCue::Close);
                    } else {
                        owners.status.message = format!(
                            "Email rejected non-clean exit ({event_group},{event_function})"
                        );
                    }
                }
                EmailUiAction::RefreshInventory {
                    event_group,
                    event_function,
                } => {
                    if (event_group, event_function) != (11, 6) {
                        owners.status.message = format!(
                            "Email rejected non-clean inventory refresh ({event_group},{event_function})"
                        );
                    }
                }
                EmailUiAction::DetachComposeItems => {}
                EmailUiAction::ApplySendSuccessItems(_) => {
                    owners.status.message =
                        "Email rejected speculative ApplySendSuccessItems outside runtime commit"
                            .to_owned();
                }
                EmailUiAction::RemoveBuddyConfirmation {
                    message_id,
                    first_name,
                    last_name,
                } => match email_buddy_target_by_names_0104(&owners.buddy, &first_name, &last_name)
                {
                    Ok(target) => {
                        let localized_name = format!("{first_name} {last_name}").trim().to_owned();
                        let text = owners
                            .content
                            .system_message_definition(i32::from(message_id))
                            .map(|definition| definition.exact_text.replace("%s", &localized_name))
                            .ok_or_else(|| {
                                format!("Email SystemMessage {message_id} has no TableData row")
                            });
                        match text.and_then(|text| {
                            queue_email_system_message_0104(
                                &mut owners.shell,
                                &mut owners.messages,
                                &owners.content,
                                email_passthrough_copy_0104(text),
                                Some(message_id),
                                PendingEmailSystemAction0104::RemoveBuddy { target },
                            )
                            .map(|_| ())
                        }) {
                            Ok(()) => {}
                            Err(error) => owners.status.message = error,
                        }
                    }
                    Err(error) => owners.status.message = error,
                },
                EmailUiAction::DeleteEmailConfirmation {
                    message_id,
                    email_index,
                } => {
                    let localized = owners
                        .content
                        .system_message_definition(i32::from(message_id))
                        .map(|definition| {
                            email_passthrough_copy_0104(definition.exact_text.clone())
                        });
                    match localized.ok_or_else(|| {
                        format!("Email SystemMessage {message_id} has no TableData row")
                    }) {
                        Ok(localized) => {
                            if let Err(error) = queue_email_system_message_0104(
                                &mut owners.shell,
                                &mut owners.messages,
                                &owners.content,
                                localized,
                                Some(message_id),
                                PendingEmailSystemAction0104::DeleteEmail { email_index },
                            ) {
                                owners.status.message = error;
                            }
                        }
                        Err(error) => owners.status.message = error,
                    }
                }
                EmailUiAction::SystemMessage {
                    message_id,
                    localized,
                    ..
                } => {
                    if let Err(error) = queue_email_system_message_0104(
                        &mut owners.shell,
                        &mut owners.messages,
                        &owners.content,
                        localized,
                        message_id,
                        PendingEmailSystemAction0104::Informational,
                    ) {
                        owners.status.message = error;
                    }
                }
                EmailUiAction::NewEmailCount(count) => {
                    owners.shell.new_email_count = count.max(0);
                    if apply_new_email_notice_0104(
                        count,
                        &mut owners.mission_ui,
                        owners.new_mail_alarm.as_deref_mut(),
                    ) {
                        output.audio.extend(EMAIL_NEW_MAIL_ARRIVAL_CUES_0104);
                    }
                }
            }
        }

        for cue in output.audio {
            let true_name = match cue {
                EmailUiAudioCue::Open => "Open_Screen",
                EmailUiAudioCue::ButtonSound => owners.shell.next_button_true_name(),
                EmailUiAudioCue::TabClick => "Tab_Click01",
                EmailUiAudioCue::EmailArrived => "Email_Arrived",
                EmailUiAudioCue::OutgoingChat => "Outgoing_Chat",
                EmailUiAudioCue::ActionSuccess => "Action_Sucess",
                EmailUiAudioCue::CharacterLimitMax => "Character_Limit_Max",
                EmailUiAudioCue::Close => "Close_Screen",
            };
            if let Err(error) = spawn_email_sfx_0104(
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
    }
}

pub(super) const EMAIL_UPDATE_CHECK_FIRST_SECONDS_0104: f32 = 60.0;
pub(super) const EMAIL_UPDATE_CHECK_INTERVAL_SECONDS_0104: f32 = 600.0;

/// Clean `cnGUINanocom.fNewUserMailTime`: the first unread-mail poll is due
/// 60 seconds after the HUD owner starts, then every 600 seconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct EmailUpdateCheckPoll0104 {
    pub(super) remaining_seconds: f32,
}

impl Default for EmailUpdateCheckPoll0104 {
    fn default() -> Self {
        Self {
            remaining_seconds: EMAIL_UPDATE_CHECK_FIRST_SECONDS_0104,
        }
    }
}

impl EmailUpdateCheckPoll0104 {
    /// Advances one ready `Update`; returns true when the request is due.
    pub(super) fn advance(&mut self, delta_seconds: f32) -> bool {
        self.remaining_seconds -= delta_seconds;
        if self.remaining_seconds < 0.0 {
            self.remaining_seconds = EMAIL_UPDATE_CHECK_INTERVAL_SECONDS_0104;
            true
        } else {
            false
        }
    }
}

/// The clean client sends this empty struct without payload bytes. Pinned
/// OpenFusion declares its one-byte C++ size (and zero-fills an empty body);
/// the registry keeps that ABI, as for `P_CL2FE_REQ_PC_GROUP_LEAVE`.
pub(super) fn email_update_check_request_0104() -> Result<ffone_protocol::RegisteredGameplayRequest0104, String>
{
    ffone_protocol::RegisteredGameplayRequest0104::new(
        ffone_client::email_ui::EMAIL_REQ_UPDATE_CHECK_ID,
        vec![0],
    )
    .map_err(|error| error.to_string())
}

/// Clean `cnGUINanocom.Update` polls unread mail outside EmailMode, and
/// OpenFusion sends `P_FE2CL_REP_PC_NEW_EMAIL` only as this poll's reply. The
/// reply is routed as an unsolicited Email frame, so the poll must not claim
/// the EmailMode request slot.
pub(super) fn poll_email_update_check_0104(
    time: Res<Time>,
    bridge: Res<NetworkBridge>,
    ready_players: Query<(), (With<LocalPlayer>, Without<LegacyWorldColliderPending>)>,
    mut shell: ResMut<EmailProductionShell0104>,
    mut status: ResMut<RuntimeStatus>,
) {
    // `Update` returns before touching the timer until `IsReadyForPlay()`.
    if ready_players.is_empty() || !shell.update_check_poll.advance(time.delta_secs()) {
        return;
    }
    let sent = email_update_check_request_0104().and_then(|request| {
        bridge
            .send(NetworkCommand::SendRegisteredGameplay0104(request))
            .map_err(|error| error.to_string())
    });
    if let Err(error) = sent {
        status.message = format!("Email update check failed: {error}");
    }
}

/// Clean `GameFrame` forwards a positive `P_FE2CL_REP_PC_NEW_EMAIL` count to
/// `cnMainGame.ReceiveNewMail`, which plays `Email_Arrived` and calls
/// `cnGUINanocom.SetNewMail`; that plays `Email_Arrived` again (two separate
/// `SoundUtil` sources), clears `bNewMailSelect` and arms the minimap alarm.
pub(super) const EMAIL_NEW_MAIL_ARRIVAL_CUES_0104: [EmailUiAudioCue; 2] = [EmailUiAudioCue::EmailArrived; 2];

pub(super) fn apply_new_email_notice_0104(
    count: i32,
    mission_ui: &mut MissionUiModel,
    alarm: Option<&mut ffone_client::gameplay_ui::MinimapNewMailAlarm>,
) -> bool {
    if count <= 0 {
        return false;
    }
    mission_ui.notify_new_email();
    if let Some(alarm) = alarm {
        alarm.arm();
    }
    true
}
