//! Email production shell, authority sync, request dispatch and network frames.

use super::email_catalog::{
    EMAIL_SYSTEM_MESSAGE_ID_BASE_0104, EmailProductionCatalog0104, available_email_task_ids_0104,
    email_guide_messages_0104,
};
use super::email_outputs::{EmailUpdateCheckPoll0104, email_passthrough_copy_0104};
use super::local_inventory::LocalInventoryRuntime;
use super::runtime_status::RuntimeStatus;
use super::tutorial_session::TutorialMissionRuntime;
use bevy::{ecs::system::SystemParam, prelude::*};
use ffone_client::{
    buddy_ui::{BuddyTarget, BuddyUiModel},
    email_runtime::{
        EmailAttachmentRejection0104, EmailAuthoritativeCommit0104, EmailFrameDisposition0104,
        EmailInventoryAuthority0104, EmailModeLease0104, EmailPlayerAuthority0104,
        EmailProductionError0104, EmailProductionOutput0104, EmailProductionRuntime0104,
        email_buddies_from_buddy_ui_0104, email_inventory_authority_0104,
    },
    email_ui::{
        EmailNetworkInbox0104, EmailNetworkRuntime0104, EmailRequest, EmailSystemTime,
        EmailTransportOutbox, EmailUiAction, EmailUiAudioOutbox, EmailUiModel, EmailUiOutbox,
    },
    guide_runtime::GuideRuntime,
    inventory_runtime::{InventoryLocation0104, InventoryRuntime0104},
    mission_ui::MissionUiModel,
    movement::advance_native_xorshift32,
    nano_free_tuning_runtime::NanoFreeTuningBank0104,
    system_message_ui::{SystemMessageRequest, SystemMessageUiModel},
    tutorial_mission_content::TutorialMissionContent,
    user_equip_ui::UserEquipModalState,
    world_mission_runtime::WorldMissionRuntime,
};
use ffone_protocol::{DecodedFrame, ItemMoveSuccessPacket0104};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PendingEmailSystemAction0104 {
    Informational,
    DeleteEmail { email_index: i64 },
    RemoveBuddy { target: BuddyTarget },
}

#[derive(Clone, Debug)]
pub(super) struct PendingEmailSystemMessage0104 {
    pub(super) request: SystemMessageRequest,
    pub(super) action: PendingEmailSystemAction0104,
}

#[derive(Debug, Default)]
pub(super) struct NpcMailNotices0104 {
    pub(super) owner: Option<i32>,
    pub(super) letters: BTreeSet<(i32, i32)>,
}

impl NpcMailNotices0104 {
    pub(super) fn observe(
        &mut self,
        owner: Option<i32>,
        letters: impl IntoIterator<Item = (i32, i32)>,
    ) -> bool {
        if self.owner != owner {
            self.letters.clear();
            self.owner = owner;
        }
        let letters = letters.into_iter().collect::<BTreeSet<_>>();
        let arrived = letters.iter().any(|letter| !self.letters.contains(letter));
        self.letters = letters;
        arrived
    }
}

#[derive(Debug, Resource)]
pub(super) struct EmailProductionShell0104 {
    pub(super) pending_outputs: VecDeque<EmailProductionOutput0104>,
    pub(super) pending_system_messages: BTreeMap<u64, PendingEmailSystemMessage0104>,
    pub(super) next_system_message_id: u64,
    pub(super) lease: Option<EmailModeLease0104>,
    pub(super) cursor_was_locked: Option<bool>,
    pub(super) ui_mode_audio: Option<Entity>,
    pub(super) inventory_mail_mode: Option<i32>,
    pub(super) last_inventory: Option<EmailInventoryAuthority0104>,
    pub(super) rejected_frames: VecDeque<DecodedFrame>,
    pub(super) new_email_count: i32,
    pub(super) npc_mail_notices: NpcMailNotices0104,
    pub(super) update_check_poll: EmailUpdateCheckPoll0104,
    pub(super) button_random_state: u32,
}

impl Default for EmailProductionShell0104 {
    fn default() -> Self {
        Self {
            pending_outputs: VecDeque::new(),
            pending_system_messages: BTreeMap::new(),
            next_system_message_id: EMAIL_SYSTEM_MESSAGE_ID_BASE_0104,
            lease: None,
            cursor_was_locked: None,
            ui_mode_audio: None,
            inventory_mail_mode: None,
            last_inventory: None,
            rejected_frames: VecDeque::new(),
            new_email_count: 0,
            npc_mail_notices: default(),
            update_check_poll: EmailUpdateCheckPoll0104::default(),
            button_random_state: 0x656d_6169,
        }
    }
}

impl EmailProductionShell0104 {
    pub(super) fn push_output(&mut self, output: EmailProductionOutput0104) {
        self.pending_outputs.push_back(output);
    }

    pub(super) fn next_request_id(&mut self) -> u64 {
        let request_id = self.next_system_message_id;
        self.next_system_message_id = self.next_system_message_id.wrapping_add(1);
        if self.next_system_message_id < EMAIL_SYSTEM_MESSAGE_ID_BASE_0104 {
            self.next_system_message_id = EMAIL_SYSTEM_MESSAGE_ID_BASE_0104;
        }
        request_id
    }

    pub(super) fn next_button_true_name(&mut self) -> &'static str {
        match advance_native_xorshift32(&mut self.button_random_state) % 5 {
            0 => "Mouse_Click01",
            1 => "Mouse_Click02",
            2 => "Mouse_Click03",
            3 => "Mouse_Click04",
            _ => "Mouse_Click05",
        }
    }

    pub(super) fn retain_rejected_frame(&mut self, frame: DecodedFrame) {
        const RETAINED_EMAIL_FRAME_LIMIT: usize = 32;
        if self.rejected_frames.len() == RETAINED_EMAIL_FRAME_LIMIT {
            self.rejected_frames.pop_front();
        }
        self.rejected_frames.push_back(frame);
    }

    pub(super) fn clear_owned_messages(&mut self, messages: &mut SystemMessageUiModel) {
        for request_id in self
            .pending_system_messages
            .keys()
            .copied()
            .collect::<Vec<_>>()
        {
            messages.remove(request_id);
        }
        self.pending_system_messages.clear();
    }

    pub(super) fn stop_ui_mode_audio(&mut self, commands: &mut Commands) {
        if let Some(entity) = self.ui_mode_audio.take() {
            commands.entity(entity).despawn();
        }
    }
}

pub(super) fn email_system_time_utc_0104(now: SystemTime) -> EmailSystemTime {
    let elapsed = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    let seconds = i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX);
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    // Howard Hinnant's civil-from-days transform. `SystemTime` has no safe
    // local-zone API; UTC is the only non-invented clock available to this
    // dependency boundary and remains deterministic on every supported host.
    let shifted = days + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    EmailSystemTime {
        year: i32::try_from(year).unwrap_or(i32::MAX),
        month: i32::try_from(month).unwrap_or_default(),
        day_of_week: i32::try_from((days + 4).rem_euclid(7)).unwrap_or_default(),
        day: i32::try_from(day).unwrap_or_default(),
        hour: i32::try_from(seconds_of_day / 3_600).unwrap_or_default(),
        minute: i32::try_from((seconds_of_day % 3_600) / 60).unwrap_or_default(),
        second: i32::try_from(seconds_of_day % 60).unwrap_or_default(),
        milliseconds: i32::try_from(elapsed.subsec_millis()).unwrap_or_default(),
    }
}

pub(super) fn email_player_authority_0104(
    runtime: &RuntimeStatus,
) -> Result<EmailPlayerAuthority0104, String> {
    Ok(EmailPlayerAuthority0104 {
        owner_pc_id: runtime
            .player_id
            .ok_or_else(|| "Email has no authoritative local PC ID".to_owned())?,
        taros: runtime.candy,
        current_local_time: email_system_time_utc_0104(SystemTime::now()),
    })
}

pub(super) fn active_email_task_ids_0104(
    world: &WorldMissionRuntime,
    tutorial: &TutorialMissionRuntime,
) -> Vec<i32> {
    world
        .active_tasks()
        .iter()
        .map(|task| task.task_id)
        .chain(tutorial.active_tasks.iter().copied())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn email_inventory_after_commit_0104(
    current: &InventoryRuntime0104,
    commit: &EmailAuthoritativeCommit0104,
) -> Result<InventoryRuntime0104, String> {
    if current.owner_pc_id() != commit.owner_pc_id() {
        return Err(format!(
            "Email commit belongs to PC {}, global inventory belongs to {}",
            commit.owner_pc_id(),
            current.owner_pc_id()
        ));
    }
    let mut next = current.clone();
    for write in commit.inventory_writes() {
        next.apply_item_move_success(ItemMoveSuccessPacket0104 {
            from_location: InventoryLocation0104::Inventory.wire_value(),
            from_slot_num: write.inventory_slot as i32,
            from_slot_item: write.item,
            to_location: InventoryLocation0104::Inventory.wire_value(),
            to_slot_num: write.inventory_slot as i32,
            to_slot_item: write.item,
        })
        .map_err(|error| error.to_string())?;
    }
    Ok(next)
}

pub(super) fn reset_email_shell_0104(
    commands: &mut Commands,
    runtime: &mut EmailProductionRuntime0104,
    model: &mut EmailUiModel,
    actions: &mut EmailUiOutbox,
    audio: &mut EmailUiAudioOutbox,
    transport: &mut EmailTransportOutbox,
    network: &mut EmailNetworkRuntime0104,
    inbox: &mut EmailNetworkInbox0104,
    shell: &mut EmailProductionShell0104,
    messages: &mut SystemMessageUiModel,
) {
    runtime.reset_boundary(model, actions, audio, transport, network, inbox);
    shell.stop_ui_mode_audio(commands);
    shell.clear_owned_messages(messages);
    *shell = EmailProductionShell0104::default();
}

#[allow(clippy::too_many_arguments)]
pub(super) fn reset_email_session_0104(
    mut commands: Commands,
    mut runtime: ResMut<EmailProductionRuntime0104>,
    mut model: ResMut<EmailUiModel>,
    mut actions: ResMut<EmailUiOutbox>,
    mut audio: ResMut<EmailUiAudioOutbox>,
    mut transport: ResMut<EmailTransportOutbox>,
    mut network: ResMut<EmailNetworkRuntime0104>,
    mut inbox: ResMut<EmailNetworkInbox0104>,
    mut shell: ResMut<EmailProductionShell0104>,
    mut messages: ResMut<SystemMessageUiModel>,
) {
    reset_email_shell_0104(
        &mut commands,
        &mut runtime,
        &mut model,
        &mut actions,
        &mut audio,
        &mut transport,
        &mut network,
        &mut inbox,
        &mut shell,
        &mut messages,
    );
}

#[derive(SystemParam)]
pub(super) struct EmailProductionDrive0104<'w> {
    pub(super) runtime: ResMut<'w, EmailProductionRuntime0104>,
    pub(super) model: ResMut<'w, EmailUiModel>,
    pub(super) actions: ResMut<'w, EmailUiOutbox>,
    pub(super) audio: ResMut<'w, EmailUiAudioOutbox>,
    pub(super) transport: ResMut<'w, EmailTransportOutbox>,
    pub(super) network: ResMut<'w, EmailNetworkRuntime0104>,
    pub(super) shell: ResMut<'w, EmailProductionShell0104>,
    pub(super) catalog: Res<'w, EmailProductionCatalog0104>,
    pub(super) nano_bank: Res<'w, NanoFreeTuningBank0104>,
    pub(super) inventory: Res<'w, LocalInventoryRuntime>,
    pub(super) buddy: Res<'w, BuddyUiModel>,
    pub(super) guide: Res<'w, GuideRuntime>,
    pub(super) world_mission: Res<'w, WorldMissionRuntime>,
    pub(super) tutorial_mission: Res<'w, TutorialMissionRuntime>,
    pub(super) content: Res<'w, TutorialMissionContent>,
    pub(super) system_messages: Res<'w, SystemMessageUiModel>,
    pub(super) mission_ui: Res<'w, MissionUiModel>,
    pub(super) user_equip_modal: Res<'w, UserEquipModalState>,
    pub(super) status: ResMut<'w, RuntimeStatus>,
}

pub(super) fn synchronize_email_authority_0104(owners: &mut EmailProductionDrive0104) {
    owners.model.system_popup_active =
        owners.system_messages.is_popup() || owners.mission_ui.system_popup_active();
    owners.model.help_active = owners.user_equip_modal.help_active;
    owners.model.external_inventory_popup_active = owners.user_equip_modal.inventory_popup_modal
        || owners.user_equip_modal.item_popup_active
        || owners.user_equip_modal.redeem_code_view;
    // Mission mail is local; it never produces a server NEW_EMAIL packet.
    // Observe authority changes even while the mailbox is closed.
    if owners.status.player_id.is_some()
        && (owners.world_mission.is_changed()
            || owners.tutorial_mission.is_changed()
            || owners.guide.is_changed()
            || owners.nano_bank.is_changed()
            || owners.inventory.is_changed()
            || owners.shell.npc_mail_notices.owner != owners.status.player_id)
    {
        if let Ok(messages) = email_guide_messages_0104(
            &owners.catalog,
            &owners.content,
            &owners.guide,
            active_email_task_ids_0104(&owners.world_mission, &owners.tutorial_mission),
            available_email_task_ids_0104(
                &owners.catalog,
                &owners.content,
                &owners.world_mission,
                &owners.guide,
                &owners.nano_bank,
                &owners.inventory,
                i32::from(owners.status.player_level),
            ),
        ) {
            if owners.shell.npc_mail_notices.observe(
                owners.status.player_id,
                messages
                    .iter()
                    .map(|mail| (mail.mode, mail.mission_task_id)),
            ) {
                owners.shell.push_output(EmailProductionOutput0104 {
                    actions: vec![EmailUiAction::NewEmailCount(1)],
                    ..default()
                });
            }
        }
    }
    if !owners.runtime.modal_active() {
        return;
    }

    match email_buddies_from_buddy_ui_0104(&owners.buddy) {
        Ok(buddies) => {
            if let Err(error) = owners
                .runtime
                .refresh_buddy_projection(buddies, &mut owners.model)
            {
                owners.status.message = format!("Email buddy projection rejected: {error}");
            }
        }
        Err(error) => owners.status.message = format!("Email buddy authority rejected: {error}"),
    }
    let active_tasks = active_email_task_ids_0104(&owners.world_mission, &owners.tutorial_mission);
    match email_guide_messages_0104(
        &owners.catalog,
        &owners.content,
        &owners.guide,
        active_tasks,
        available_email_task_ids_0104(
            &owners.catalog,
            &owners.content,
            &owners.world_mission,
            &owners.guide,
            &owners.nano_bank,
            &owners.inventory,
            i32::from(owners.status.player_level),
        ),
    ) {
        Ok(messages) => {
            if let Err(error) = owners
                .runtime
                .refresh_guide_projection(messages, &mut owners.model)
            {
                owners.status.message = format!("Email guide projection rejected: {error}");
            }
        }
        Err(error) => owners.status.message = error,
    }

    let Some(inventory) = owners.inventory.snapshot() else {
        owners.status.message = "Email lost its authoritative inventory feed".to_owned();
        return;
    };
    let inventory = email_inventory_authority_0104(inventory);
    let inventory_changed = owners.shell.last_inventory.as_ref() != Some(&inventory);
    let session_stale = owners
        .runtime
        .session()
        .is_some_and(|session| session.inventory_projection_stale());
    if (inventory_changed || !session_stale)
        && let Ok(player) = email_player_authority_0104(&owners.status)
    {
        match owners.runtime.refresh_inventory_projection(
            player,
            &inventory,
            &*owners.catalog,
            &mut owners.model,
        ) {
            Ok(()) => owners.shell.last_inventory = Some(inventory),
            Err(error) => {
                owners.status.message = format!("Email inventory projection rejected: {error}")
            }
        }
    }
}

pub(super) fn dispatch_email_request_0104(owners: &mut EmailProductionDrive0104) {
    if !owners.runtime.modal_active() {
        return;
    }
    match owners.runtime.dispatch_next_request(
        &owners.model,
        &mut owners.transport,
        &mut owners.network,
        &*owners.catalog,
    ) {
        Ok(output)
            if output.request.is_some()
                || output.commit.is_some()
                || !output.actions.is_empty()
                || !output.audio.is_empty() =>
        {
            owners.shell.push_output(output);
        }
        Ok(_) => {}
        Err(error) => {
            if let Some(output) = email_local_rejection_output_0104(&error, &owners.content) {
                owners.shell.push_output(output);
            }
            if email_dispatch_error_rejects_front_request_0104(&error) {
                release_rejected_email_request_0104(&mut owners.model, &mut owners.transport);
            }
            owners.status.message = format!("Email request rejected: {error}");
        }
    }
}

pub(super) fn email_dispatch_error_rejects_front_request_0104(error: &EmailProductionError0104) -> bool {
    matches!(
        error,
        EmailProductionError0104::MalformedInventoryItem { .. }
            | EmailProductionError0104::MissingCatalogItem { .. }
            | EmailProductionError0104::InvalidCatalogIcon { .. }
            | EmailProductionError0104::MissingTradeMetadata { .. }
            | EmailProductionError0104::MissingGeneralSubtype { .. }
            | EmailProductionError0104::AttachmentRejected { .. }
            | EmailProductionError0104::AttachmentSourceChanged { .. }
            | EmailProductionError0104::InventoryAuthorityStale
            | EmailProductionError0104::TarosProjectionMismatch { .. }
            | EmailProductionError0104::RequestNotReachable { .. }
            | EmailProductionError0104::Transport(_)
            | EmailProductionError0104::Registration(_)
    )
}

pub(super) fn email_local_rejection_output_0104(
    error: &EmailProductionError0104,
    content: &TutorialMissionContent,
) -> Option<EmailProductionOutput0104> {
    let EmailProductionError0104::AttachmentRejected { reason, .. } = error else {
        return None;
    };
    if !matches!(
        reason,
        EmailAttachmentRejection0104::NotTradeable | EmailAttachmentRejection0104::CombinedLook
    ) {
        return None;
    }
    // Clean InventoryManagerScript::PutItemInEmailSlot emits row 177 only
    // for its `m_iTradeAble == 0` branch. Chest, unsupported outer types and
    // general-email subtype 3 return silently before this branch.
    let definition = content.system_message_definition(177)?;
    let fallback = definition.exact_text.clone();
    Some(EmailProductionOutput0104 {
        actions: vec![EmailUiAction::SystemMessage {
            message_id: Some(177),
            fallback: fallback.clone(),
            localized: email_passthrough_copy_0104(fallback),
        }],
        ..default()
    })
}

pub(super) fn release_rejected_email_request_0104(
    model: &mut EmailUiModel,
    transport: &mut EmailTransportOutbox,
) -> Option<EmailRequest> {
    let request = transport.pop()?;
    model.send_in_flight = false;
    if matches!(request, EmailRequest::Send { .. }) {
        model.mail_send_in_flight = false;
    }
    Some(request)
}

pub(super) fn consume_email_network_frames_0104(
    mut runtime: ResMut<EmailProductionRuntime0104>,
    mut model: ResMut<EmailUiModel>,
    mut actions: ResMut<EmailUiOutbox>,
    mut transport: ResMut<EmailTransportOutbox>,
    mut audio: ResMut<EmailUiAudioOutbox>,
    mut network: ResMut<EmailNetworkRuntime0104>,
    mut inbox: ResMut<EmailNetworkInbox0104>,
    mut shell: ResMut<EmailProductionShell0104>,
    mut status: ResMut<RuntimeStatus>,
) {
    while let Some(disposition) = runtime.consume_next_frame(
        &mut inbox,
        &mut model,
        &mut actions,
        &mut transport,
        &mut audio,
        &mut network,
    ) {
        match disposition {
            EmailFrameDisposition0104::Applied { output, .. } => shell.push_output(output),
            EmailFrameDisposition0104::Passthrough(frame) => {
                status.message = format!(
                    "Email returned queued packet {:#010x} to passthrough",
                    frame.packet_type
                );
                shell.retain_rejected_frame(frame);
            }
            EmailFrameDisposition0104::Rejected { frame, error } => {
                status.message = format!(
                    "Email rejected owned packet {:#010x}: {error}",
                    frame.packet_type
                );
                shell.retain_rejected_frame(frame);
            }
        }
    }
}
