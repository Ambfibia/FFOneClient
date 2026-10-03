//! Enchant production shell, network frames, system messages and outputs.

use super::WorldSliceEntity;
use super::local_inventory::LocalInventoryRuntime;
use super::option_runtime::{OptionProductionRuntime, option_channel_gain};
use super::runtime_status::{RuntimeStatus, resolve_runtime_resurrection_item_slot};
use bevy::{
    audio::Volume,
    ecs::system::SystemParam,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use ffone_client::{
    enchant_runtime::{
        EnchantAuthoritativeCommit0104, EnchantInputEffect0104, EnchantModalChoice0104,
        EnchantOperation0104, EnchantPlayerAuthority0104, EnchantProductionError0104,
        EnchantProductionOutput0104, EnchantProductionRuntime0104, EnchantRuntimeModal0104,
        EnchantShellEffect0104,
    },
    enchant_ui::{
        ENCHANT_ARMOR_MATERIAL_ID_0104, ENCHANT_DELETE_SUCCESS_PACKET_ID_0104,
        ENCHANT_DISASSEMBLE_FAILURE_PACKET_ID_0104, ENCHANT_DISASSEMBLE_SUCCESS_PACKET_ID_0104,
        ENCHANT_FAILURE_PACKET_ID_0104, ENCHANT_HELP_ITEM_1_ID_0104, ENCHANT_HELP_ITEM_2_ID_0104,
        ENCHANT_SUCCESS_PACKET_ID_0104, ENCHANT_WEAPON_MATERIAL_ID_0104,
        EnchantAnimationIntent0104, EnchantAudioIntent0104, EnchantCameraIntent0104,
        EnchantExternalGates0104, EnchantInventorySlotProjection0104, EnchantItemPresentation0104,
        EnchantLifecycleIntent0104, EnchantModeProjection0104, EnchantPhase0104,
        EnchantPopupIntent0104, EnchantSelectionIntent0104, EnchantSupportPresentation0104,
        EnchantTargetKind0104, EnchantUiOutbox0104,
    },
    inventory_runtime::{InventoryLocation0104, InventoryRuntime0104},
    localization::LocalizedText,
    mission_ui::MissionUiModel,
    network::{NetworkBridge, NetworkCommand},
    semantic_audio::{NativeAudioCatalog, NativeAudioCategory},
    system_message_ui::{
        SystemMessageButtonType, SystemMessageChoice, SystemMessageRequest, SystemMessageUiAction,
        SystemMessageUiModel, SystemMessageUiOutbox,
    },
    tutorial_mission_content::TutorialMissionContent,
    user_equip_ui::{UserEquipItemIds, UserEquipModalState, UserEquipUiState},
};
use ffone_protocol::{DecodedFrame, ItemBase0104, ItemMoveSuccessPacket0104};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub(super) const ENCHANT_SYSTEM_MESSAGE_ID_BASE_0104: u64 = 0x454e_4348_0000_0000;
pub(super) const ENCHANT_INPUT_OWNER: u64 = 0x454e_494e_0000_0000;
pub(super) const ENCHANT_PREVIEW_OWNER_GAP_0104: &str =
    "Enchant item preview remains fail-closed: no production PopupControll item owner is available";
pub(super) const ENCHANT_HELP_OWNER_GAP_0104: &str =
    "Enchant Help remains fail-closed: no production Help-mode page owner is available";
pub(super) const ENCHANT_ANIMATION_OWNER_GAP_0104: &str = "Enchant retained the exact NPC event-animation intent; no source-equivalent normal-world NPC animation owner is available";

#[derive(Clone, Copy, Debug)]
pub(super) enum PendingEnchantSystemAction0104 {
    Model,
    RuntimeDelete,
    RedeemSpaceError,
}

#[derive(Clone, Debug)]
pub(super) struct PendingEnchantSystemMessage0104 {
    pub(super) request: SystemMessageRequest,
    pub(super) action: PendingEnchantSystemAction0104,
}

#[derive(Debug, Resource)]
pub(super) struct EnchantProductionShell0104 {
    pub(super) pending_outputs: VecDeque<EnchantProductionOutput0104>,
    pub(super) pending_system_messages: BTreeMap<u64, PendingEnchantSystemMessage0104>,
    pub(super) next_system_message_id: u64,
    pub(super) ui_mode_audio_active: bool,
    pub(super) inventory_mode_active: bool,
    pub(super) redeem_code_active: bool,
    pub(super) redeem_space_error_active: bool,
    pub(super) primary_camera_intent: Option<EnchantCameraIntent0104>,
    pub(super) waiting_camera_intent: Option<EnchantCameraIntent0104>,
    pub(super) last_animation_intent: Option<EnchantAnimationIntent0104>,
    pub(super) last_preview_intent: Option<EnchantPopupIntent0104>,
    pub(super) last_selection_intent: Option<EnchantSelectionIntent0104>,
    pub(super) legacy_events: Vec<(i32, i32, Option<i32>, Option<i32>)>,
    pub(super) first_use_checks: Vec<i32>,
}

impl Default for EnchantProductionShell0104 {
    fn default() -> Self {
        Self {
            pending_outputs: VecDeque::new(),
            pending_system_messages: BTreeMap::new(),
            next_system_message_id: ENCHANT_SYSTEM_MESSAGE_ID_BASE_0104,
            ui_mode_audio_active: false,
            inventory_mode_active: false,
            redeem_code_active: false,
            redeem_space_error_active: false,
            primary_camera_intent: None,
            waiting_camera_intent: None,
            last_animation_intent: None,
            last_preview_intent: None,
            last_selection_intent: None,
            legacy_events: Vec::new(),
            first_use_checks: Vec::new(),
        }
    }
}

impl EnchantProductionShell0104 {
    pub(super) fn push_output(&mut self, output: EnchantProductionOutput0104) {
        self.pending_outputs.push_back(output);
    }

    pub(super) fn next_request_id(&mut self) -> u64 {
        let request_id = self.next_system_message_id;
        self.next_system_message_id = self.next_system_message_id.wrapping_add(1);
        if self.next_system_message_id < ENCHANT_SYSTEM_MESSAGE_ID_BASE_0104 {
            self.next_system_message_id = ENCHANT_SYSTEM_MESSAGE_ID_BASE_0104;
        }
        request_id
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

    pub(super) fn reset(&mut self, messages: &mut SystemMessageUiModel) {
        self.clear_owned_messages(messages);
        *self = Self::default();
    }
}

#[must_use]
pub(super) const fn enchant_frame_operation_0104(packet_type: u32) -> Option<EnchantOperation0104> {
    match packet_type {
        ENCHANT_SUCCESS_PACKET_ID_0104 | ENCHANT_FAILURE_PACKET_ID_0104 => {
            Some(EnchantOperation0104::Enchant)
        }
        ENCHANT_DELETE_SUCCESS_PACKET_ID_0104 => Some(EnchantOperation0104::Delete),
        ENCHANT_DISASSEMBLE_SUCCESS_PACKET_ID_0104 | ENCHANT_DISASSEMBLE_FAILURE_PACKET_ID_0104 => {
            Some(EnchantOperation0104::Disassemble)
        }
        _ => None,
    }
}

#[must_use]
pub(super) fn enchant_frame_owned_0104(runtime: &EnchantProductionRuntime0104, packet_type: u32) -> bool {
    let pending = runtime
        .session()
        .and_then(|session| session.pending_operation());
    enchant_frame_operation_0104(packet_type).is_some_and(|operation| pending == Some(operation))
}

#[derive(Debug, Default, Resource)]
pub(super) struct EnchantNetworkFrameInbox0104(pub(super) VecDeque<DecodedFrame>);

impl EnchantNetworkFrameInbox0104 {
    pub(super) fn push_if_owned(
        &mut self,
        runtime: &EnchantProductionRuntime0104,
        frame: DecodedFrame,
    ) -> bool {
        if !enchant_frame_owned_0104(runtime, frame.packet_type) {
            return false;
        }
        self.0.push_back(frame);
        true
    }

    pub(super) fn pop_front(&mut self) -> Option<DecodedFrame> {
        self.0.pop_front()
    }

    pub(super) fn clear(&mut self) {
        self.0.clear();
    }
}

pub(super) fn reset_enchant_shell_0104(
    runtime: &mut EnchantProductionRuntime0104,
    projection: &mut EnchantModeProjection0104,
    outbox: &mut EnchantUiOutbox0104,
    shell: &mut EnchantProductionShell0104,
    frames: &mut EnchantNetworkFrameInbox0104,
    messages: &mut SystemMessageUiModel,
) {
    runtime.reset();
    *projection = EnchantModeProjection0104::default();
    while outbox.pop_front().is_some() {}
    frames.clear();
    shell.reset(messages);
}

pub(super) fn reset_enchant_session_0104(
    mut runtime: ResMut<EnchantProductionRuntime0104>,
    mut projection: ResMut<EnchantModeProjection0104>,
    mut outbox: ResMut<EnchantUiOutbox0104>,
    mut shell: ResMut<EnchantProductionShell0104>,
    mut frames: ResMut<EnchantNetworkFrameInbox0104>,
    mut messages: ResMut<SystemMessageUiModel>,
) {
    reset_enchant_shell_0104(
        &mut runtime,
        &mut projection,
        &mut outbox,
        &mut shell,
        &mut frames,
        &mut messages,
    );
}

pub(super) fn enchant_item_presentation_0104(
    content: &TutorialMissionContent,
    item: ItemBase0104,
) -> Option<EnchantItemPresentation0104> {
    ffone_client::enchant_ui::enchant_item_presentation_from_content(content, item)
}

pub(super) fn enchant_support_item_0104(item_id: i16) -> ItemBase0104 {
    ItemBase0104 {
        item_type: 7,
        item_id,
        option: 1,
        time_limit: 0,
    }
}

pub(super) fn enchant_support_presentation_0104(
    content: &TutorialMissionContent,
) -> EnchantSupportPresentation0104 {
    EnchantSupportPresentation0104 {
        weapon_material: enchant_item_presentation_0104(
            content,
            enchant_support_item_0104(ENCHANT_WEAPON_MATERIAL_ID_0104),
        )
        .unwrap_or_default(),
        armor_material: enchant_item_presentation_0104(
            content,
            enchant_support_item_0104(ENCHANT_ARMOR_MATERIAL_ID_0104),
        )
        .unwrap_or_default(),
        helper_1: enchant_item_presentation_0104(
            content,
            enchant_support_item_0104(ENCHANT_HELP_ITEM_1_ID_0104),
        )
        .unwrap_or_default(),
        helper_2: enchant_item_presentation_0104(
            content,
            enchant_support_item_0104(ENCHANT_HELP_ITEM_2_ID_0104),
        )
        .unwrap_or_default(),
    }
}

pub(super) fn enchant_inventory_slot_projection_0104(
    content: &TutorialMissionContent,
    item: ItemBase0104,
    hidden_by_selection_overlay: bool,
) -> EnchantInventorySlotProjection0104 {
    if InventoryRuntime0104::item_is_empty(item) {
        return EnchantInventorySlotProjection0104::default();
    }
    let presentation = enchant_item_presentation_0104(content, item);
    let target_or_material =
        EnchantTargetKind0104::from_item_type(item.item_type).is_some() || item.item_type == 7;
    EnchantInventorySlotProjection0104 {
        item: Some(item),
        icon_path: presentation
            .as_ref()
            .and_then(|presentation| presentation.icon_path.clone()),
        restricted: !target_or_material || presentation.is_none(),
        show_combined_badge: UserEquipItemIds::from_item(item).combined_look_id.is_some(),
        quantity_label: (item.item_type == 7 && item.option > 0).then(|| item.option.to_string()),
        hidden_by_selection_overlay,
    }
}

pub(super) fn enchant_player_authority_0104(runtime: &RuntimeStatus) -> Option<EnchantPlayerAuthority0104> {
    Some(EnchantPlayerAuthority0104 {
        owner_pc_id: runtime.player_id?,
        taros: runtime.candy,
        weapon_battery: runtime.weapon_battery,
        nano_battery: runtime.nano_battery,
    })
}

pub(super) fn consume_enchant_network_frames_0104(
    mut inbox: ResMut<EnchantNetworkFrameInbox0104>,
    mut runtime: ResMut<EnchantProductionRuntime0104>,
    mut shell: ResMut<EnchantProductionShell0104>,
    mut status: ResMut<RuntimeStatus>,
) {
    while let Some(frame) = inbox.pop_front() {
        let packet_type = frame.packet_type;
        match runtime.apply_frame(frame) {
            Ok(Some(output)) => shell.push_output(output),
            Ok(None) => {
                status.message = format!(
                    "EnchantMode rejected an owned packet {packet_type:#010x} as passthrough"
                );
            }
            Err(error) => {
                status.message = format!("EnchantMode authoritative reply rejected: {error}");
            }
        }
    }
}

pub(super) fn enchant_inventory_close_decision_0104(modal: &mut UserEquipModalState) -> bool {
    if modal.inventory_popup_modal || modal.item_popup_active {
        modal.inventory_popup_modal = false;
        modal.item_popup_active = false;
        return false;
    }
    !modal.send_pending
        && !modal.help_active
        && !modal.system_popup_active
        && !modal.redeem_code_view
        && !modal.external_exit_blocked
}

pub(super) fn drive_enchant_production_0104(
    time: Res<Time>,
    content: Res<TutorialMissionContent>,
    inventory: Res<LocalInventoryRuntime>,
    system_messages: Res<SystemMessageUiModel>,
    mission_ui: Res<MissionUiModel>,
    modal: Res<UserEquipModalState>,
    mut runtime: ResMut<EnchantProductionRuntime0104>,
    mut outbox: ResMut<EnchantUiOutbox0104>,
    mut shell: ResMut<EnchantProductionShell0104>,
    mut status: ResMut<RuntimeStatus>,
) {
    if !runtime.is_active() {
        if outbox.pop_front().is_some() {
            while outbox.pop_front().is_some() {}
            status.message =
                "EnchantMode discarded UI input without an active production session".to_owned();
        }
        return;
    }
    let gates = EnchantExternalGates0104 {
        help_open: modal.help_active,
        inventory_popup_open: modal.inventory_popup_modal
            || modal.redeem_code_view
            || shell.redeem_code_active,
        system_popup_open: modal.system_popup_active
            || system_messages.is_popup()
            || mission_ui.system_popup_active()
            || shell.redeem_space_error_active,
        item_popup_open: modal.item_popup_active,
    };
    if let Err(error) = runtime.set_external_gates(gates) {
        status.message = format!("EnchantMode gate synchronization rejected: {error}");
        return;
    }
    if let (Some(snapshot), Some(player)) =
        (inventory.snapshot(), enchant_player_authority_0104(&status))
        && let Err(error) = runtime.refresh_authority(snapshot, player)
        && !matches!(
            error,
            EnchantProductionError0104::AuthorityRefreshBlocked { .. }
        )
    {
        status.message = format!("EnchantMode authority refresh rejected: {error}");
    }

    let catalog = |item| enchant_item_presentation_0104(&content, item);
    while let Some(result) = runtime.apply_next_ui_command(&mut outbox, &catalog) {
        match result {
            Ok(output) => shell.push_output(output),
            Err(error) => {
                status.message = format!("EnchantMode UI command rejected: {error}");
            }
        }
    }
    if runtime
        .session()
        .is_some_and(|session| matches!(session.model().phase(), EnchantPhase0104::Waiting { .. }))
    {
        match runtime.tick(time.delta_secs().max(0.0)) {
            Ok(output) => shell.push_output(output),
            Err(error) => status.message = format!("EnchantMode wait rejected: {error}"),
        }
    }
}

pub(super) fn queue_enchant_table_system_message_0104(
    shell: &mut EnchantProductionShell0104,
    messages: &mut SystemMessageUiModel,
    content: &TutorialMissionContent,
    message_id: i32,
    action: PendingEnchantSystemAction0104,
) -> Result<u64, String> {
    let definition = content
        .system_message_definition(message_id)
        .ok_or_else(|| format!("Enchant SystemMessage {message_id} has no source TableData row"))?;
    let request_id = shell.next_request_id();
    let request = SystemMessageRequest::new(
        request_id,
        definition.exact_text.clone(),
        definition.runtime_button_type,
    );
    shell.pending_system_messages.insert(
        request_id,
        PendingEnchantSystemMessage0104 {
            request: request.clone(),
            action,
        },
    );
    messages.push(request);
    Ok(request_id)
}

pub(super) fn queue_enchant_localized_system_message_0104(
    shell: &mut EnchantProductionShell0104,
    messages: &mut SystemMessageUiModel,
    localized: LocalizedText,
    action: PendingEnchantSystemAction0104,
) -> u64 {
    let request_id = shell.next_request_id();
    let request =
        SystemMessageRequest::new_localized(request_id, localized, SystemMessageButtonType::Ok);
    shell.pending_system_messages.insert(
        request_id,
        PendingEnchantSystemMessage0104 {
            request: request.clone(),
            action,
        },
    );
    messages.push(request);
    request_id
}

pub(super) fn requeue_enchant_system_message_0104(
    shell: &mut EnchantProductionShell0104,
    messages: &mut SystemMessageUiModel,
    pending: PendingEnchantSystemMessage0104,
) {
    let request_id = pending.request.request_id;
    messages.push(pending.request.clone());
    shell.pending_system_messages.insert(request_id, pending);
}

pub(super) fn consume_enchant_system_message_outbox_0104(
    mut outbox: ResMut<SystemMessageUiOutbox>,
    mut messages: ResMut<SystemMessageUiModel>,
    mut runtime: ResMut<EnchantProductionRuntime0104>,
    mut shell: ResMut<EnchantProductionShell0104>,
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
            status.message = format!(
                "EnchantMode rejected mismatched SystemMessage button type {button_type:?}"
            );
            requeue_enchant_system_message_0104(&mut shell, &mut messages, pending);
            continue;
        }
        let choice = match choice {
            SystemMessageChoice::Primary => EnchantModalChoice0104::Accept,
            SystemMessageChoice::Secondary => EnchantModalChoice0104::Dismiss,
        };
        let result = match pending.action {
            PendingEnchantSystemAction0104::Model
            | PendingEnchantSystemAction0104::RuntimeDelete => runtime.resolve_modal(choice),
            PendingEnchantSystemAction0104::RedeemSpaceError => {
                shell.redeem_space_error_active = false;
                if let Some(session) = runtime.session() {
                    let mut gates = session.model().external_gates();
                    gates.system_popup_open = false;
                    runtime
                        .set_external_gates(gates)
                        .map(|_| EnchantProductionOutput0104::default())
                } else {
                    Err(EnchantProductionError0104::NotActive)
                }
            }
        };
        match result {
            Ok(output) => shell.push_output(output),
            Err(error) => {
                // `0x130000A4` is intentionally absent from the pinned shard.
                // `resolve_modal` is transactional, so putting the exact modal
                // back lets the player dismiss it without inventing a request.
                status.message = format!("EnchantMode confirmation failed closed: {error}");
                requeue_enchant_system_message_0104(&mut shell, &mut messages, pending);
            }
        }
    }
}

pub(super) fn enchant_inventory_after_commit_0104(
    current: &InventoryRuntime0104,
    commit: &EnchantAuthoritativeCommit0104,
) -> Result<InventoryRuntime0104, String> {
    if current.owner_pc_id() != commit.snapshot_after().owner_pc_id() {
        return Err(format!(
            "EnchantMode commit belongs to PC {}, global inventory belongs to {}",
            commit.snapshot_after().owner_pc_id(),
            current.owner_pc_id()
        ));
    }
    let mut next = current.clone();
    for write in commit.inventory_writes() {
        next.apply_item_move_success(ItemMoveSuccessPacket0104 {
            from_location: InventoryLocation0104::Inventory.wire_value(),
            from_slot_num: write.inventory_index as i32,
            from_slot_item: write.item,
            to_location: InventoryLocation0104::Inventory.wire_value(),
            to_slot_num: write.inventory_index as i32,
            to_slot_item: write.item,
        })
        .map_err(|error| error.to_string())?;
    }
    if next.inventory() != commit.snapshot_after().inventory()
        || next.equipment() != commit.snapshot_after().equipment()
    {
        return Err(
            "EnchantMode staged global inventory differs from its authoritative post-state"
                .to_owned(),
        );
    }
    Ok(next)
}

pub(super) fn enchant_audio_true_name_0104(intent: EnchantAudioIntent0104) -> Option<&'static str> {
    match intent {
        EnchantAudioIntent0104::PlayUiMode | EnchantAudioIntent0104::StopUiMode => None,
        EnchantAudioIntent0104::CrocPotSuccess => Some("CrocPot_success"),
        EnchantAudioIntent0104::CrocPotFail => Some("CrocPot_fail"),
        EnchantAudioIntent0104::Button => None,
        EnchantAudioIntent0104::NoButton => Some("No_Button"),
        EnchantAudioIntent0104::YesButton => Some("Yes_Button"),
        EnchantAudioIntent0104::ActionSuccess => Some("Action_Sucess"),
    }
}

pub(super) fn spawn_enchant_sfx_0104(
    commands: &mut Commands,
    asset_server: &AssetServer,
    catalog: &NativeAudioCatalog,
    gain: f32,
    true_name: &'static str,
) -> Result<(), String> {
    let candidates = catalog
        .by_true_name(true_name)
        .into_iter()
        .filter(|asset| asset.category == NativeAudioCategory::Sfx)
        .collect::<Vec<_>>();
    let [asset] = candidates.as_slice() else {
        return Err(format!(
            "Enchant SFX {true_name:?} resolved to {} native assets",
            candidates.len()
        ));
    };
    commands.spawn((
        Name::new(format!("EnchantMode SFX {true_name}")),
        WorldSliceEntity,
        AudioPlayer::new(asset_server.load(asset.path.clone())),
        ffone_client::audio_channel::GameplayAudioChannel::ui_sfx(),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(gain)),
    ));
    Ok(())
}

#[derive(SystemParam)]
pub(super) struct EnchantProductionOwners0104<'w, 's> {
    pub(super) audio: ResMut<'w, ffone_client::gameplay_audio::GameplayAudioRuntime>,
    pub(super) input: ResMut<'w, ffone_client::shared_input_ui::SharedInputDialog>,
    pub(super) runtime: ResMut<'w, EnchantProductionRuntime0104>,
    pub(super) shell: ResMut<'w, EnchantProductionShell0104>,
    pub(super) inventory: ResMut<'w, LocalInventoryRuntime>,
    pub(super) system_messages: ResMut<'w, SystemMessageUiModel>,
    pub(super) content: Res<'w, TutorialMissionContent>,
    pub(super) modal: ResMut<'w, UserEquipModalState>,
    pub(super) user_equip: ResMut<'w, UserEquipUiState>,
    pub(super) option_runtime: Res<'w, OptionProductionRuntime>,
    pub(super) audio_catalog: Res<'w, NativeAudioCatalog>,
    pub(super) status: ResMut<'w, RuntimeStatus>,
    pub(super) cursors: Query<'w, 's, &'static mut CursorOptions, With<PrimaryWindow>>,
}

pub(super) fn consume_enchant_production_outputs_0104(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    bridge: Res<NetworkBridge>,
    mut owners: EnchantProductionOwners0104,
) {
    let effects_gain = option_channel_gain(owners.option_runtime.options.sound.effects);
    while let Some(output) = owners.shell.pending_outputs.pop_front() {
        if let Some(request) = output.request {
            if let Err(error) = bridge.send(NetworkCommand::SendRegisteredGameplay0104(request)) {
                owners.status.message = format!("EnchantMode request send failed: {error}");
                let old_cursor_lock = owners
                    .runtime
                    .session()
                    .is_some_and(|session| session.context().old_cursor_lock);
                owners.runtime.reset();
                owners
                    .shell
                    .clear_owned_messages(&mut owners.system_messages);
                if let Ok(mut cursor) = owners.cursors.single_mut() {
                    cursor.grab_mode = if old_cursor_lock {
                        CursorGrabMode::Locked
                    } else {
                        CursorGrabMode::None
                    };
                    cursor.visible = !old_cursor_lock;
                }
                continue;
            }
        }
        if let Some(commit) = output.commit {
            let result = owners
                .inventory
                .snapshot()
                .ok_or_else(|| "EnchantMode commit has no global inventory owner".to_owned())
                .and_then(|current| enchant_inventory_after_commit_0104(current, &commit));
            match result {
                Ok(next) => {
                    owners.inventory.snapshot = Some(next);
                    owners.status.candy = commit.taros_after();
                    owners.status.weapon_battery = commit.weapon_battery_after();
                    owners.status.nano_battery = commit.nano_battery_after();
                    owners.status.resurrection_item_slot =
                        owners.inventory.snapshot().and_then(|inventory| {
                            resolve_runtime_resurrection_item_slot(inventory, &owners.content)
                        });
                }
                Err(error) => {
                    owners.status.message = format!("EnchantMode commit rejected: {error}");
                    owners.runtime.reset();
                    owners
                        .shell
                        .clear_owned_messages(&mut owners.system_messages);
                    continue;
                }
            }
        }
        for effect in output.effects {
            match effect {
                EnchantShellEffect0104::ModeOpened(context) => {
                    owners.status.message = format!(
                        "EnchantMode 29 opened from category-27 runtime NPC {} (table {})",
                        context.source.runtime_npc_id, context.source.table_npc_id
                    );
                }
                EnchantShellEffect0104::ModeClosed(context) => {
                    owners
                        .shell
                        .clear_owned_messages(&mut owners.system_messages);
                    owners.shell.inventory_mode_active = false;
                    owners.shell.redeem_code_active = false;
                    owners.shell.redeem_space_error_active = false;
                    owners.status.message = format!(
                        "EnchantMode closed for runtime NPC {}",
                        context.source.runtime_npc_id
                    );
                }
                EnchantShellEffect0104::Audio(intent) => match intent {
                    EnchantAudioIntent0104::PlayUiMode => {
                        owners.shell.ui_mode_audio_active = true;
                    }
                    EnchantAudioIntent0104::StopUiMode => {
                        owners.shell.ui_mode_audio_active = false;
                    }
                    EnchantAudioIntent0104::Button => owners.audio.queue_legacy_button_sound(),
                    intent => {
                        let true_name = enchant_audio_true_name_0104(intent)
                            .expect("non-mode Enchant audio has a semantic true name");
                        if let Err(error) = spawn_enchant_sfx_0104(
                            &mut commands,
                            &asset_server,
                            &owners.audio_catalog,
                            0.7 * effects_gain,
                            true_name,
                        ) {
                            owners.status.message = error;
                        }
                    }
                },
                EnchantShellEffect0104::Animation(intent) => {
                    owners.shell.last_animation_intent = Some(intent);
                    owners.status.message = ENCHANT_ANIMATION_OWNER_GAP_0104.to_owned();
                }
                EnchantShellEffect0104::Camera(intent) => {
                    match intent {
                        EnchantCameraIntent0104::BindPrimaryNpcAvatar { .. } => {
                            owners.shell.primary_camera_intent = Some(intent);
                        }
                        EnchantCameraIntent0104::BindWaitingNpcAvatar { .. } => {
                            owners.shell.waiting_camera_intent = Some(intent);
                        }
                    }
                    owners.status.message = "Enchant live NPC portrait requested".to_owned();
                }
                EnchantShellEffect0104::Input(EnchantInputEffect0104::SetCursorLock(locked)) => {
                    if let Ok(mut cursor) = owners.cursors.single_mut() {
                        cursor.grab_mode = if locked {
                            CursorGrabMode::Locked
                        } else {
                            CursorGrabMode::None
                        };
                        cursor.visible = !locked;
                    } else {
                        owners.status.message =
                            "EnchantMode cursor effect has no unique production owner".to_owned();
                    }
                }
                EnchantShellEffect0104::Input(
                    EnchantInputEffect0104::InventoryCloseDecisionRequested,
                ) => {
                    let accepted = enchant_inventory_close_decision_0104(&mut owners.modal);
                    match owners.runtime.resolve_close(accepted) {
                        Ok(output) => owners.shell.push_output(output),
                        Err(error) => {
                            owners.status.message =
                                format!("EnchantMode close arbitration rejected: {error}");
                        }
                    }
                }
                EnchantShellEffect0104::Input(EnchantInputEffect0104::InventoryCloseRejected) => {
                    owners.status.message =
                        "EnchantMode close was consumed by the shared inventory popup owner"
                            .to_owned();
                }
                EnchantShellEffect0104::Lifecycle(intent) => match intent {
                    EnchantLifecycleIntent0104::InitializeInventoryMode {
                        gui_mode,
                        active_inventory_tab,
                        show_inventory_panel,
                        show_equipment_panel,
                        inventory_event_dispatch,
                    } => {
                        owners.shell.inventory_mode_active = gui_mode == 9
                            && active_inventory_tab == 0
                            && show_inventory_panel
                            && show_equipment_panel
                            && inventory_event_dispatch == 8;
                    }
                    EnchantLifecycleIntent0104::LoadPanelBackdrop { .. } => {}
                    EnchantLifecycleIntent0104::RestoreGameplayInventory {
                        inventory_event_dispatch,
                    } => {
                        owners.shell.inventory_mode_active = inventory_event_dispatch != 10;
                    }
                    EnchantLifecycleIntent0104::SetCursorLock(locked) => {
                        if let Ok(mut cursor) = owners.cursors.single_mut() {
                            cursor.grab_mode = if locked {
                                CursorGrabMode::Locked
                            } else {
                                CursorGrabMode::None
                            };
                            cursor.visible = !locked;
                        }
                    }
                    EnchantLifecycleIntent0104::ExitToMainGame => {}
                    EnchantLifecycleIntent0104::GoToMyStuff => {}
                    EnchantLifecycleIntent0104::EnterGameMode(game_mode) => {
                        if game_mode == 6 {
                            owners.user_equip.open_item_mode();
                        } else {
                            owners.status.message = format!(
                                "EnchantMode rejected unsupported successor game mode {game_mode}"
                            );
                        }
                    }
                    EnchantLifecycleIntent0104::DispatchLegacyEvent {
                        channel,
                        element_func,
                        argument,
                        dispatch,
                    } => {
                        owners
                            .shell
                            .legacy_events
                            .push((channel, element_func, argument, dispatch))
                    }
                    EnchantLifecycleIntent0104::CheckFirstUse(condition) => {
                        owners.shell.first_use_checks.push(condition);
                    }
                },
                EnchantShellEffect0104::Popup(intent) => {
                    match intent {
                        EnchantPopupIntent0104::SystemMessage { message_id, .. } => {
                            if let Err(error) = queue_enchant_table_system_message_0104(
                                &mut owners.shell,
                                &mut owners.system_messages,
                                &owners.content,
                                message_id,
                                PendingEnchantSystemAction0104::Model,
                            ) {
                                owners.status.message = error;
                            }
                        }
                        intent @ EnchantPopupIntent0104::Preview { .. } => {
                            owners.shell.last_preview_intent = Some(intent);
                            owners.status.message = ENCHANT_PREVIEW_OWNER_GAP_0104.to_owned();
                        }
                        EnchantPopupIntent0104::RedeemCode { .. } => {
                            owners.shell.redeem_code_active = owners.input.open(ffone_client::shared_input_ui::SharedInputRequest {
                            owner: ENCHANT_INPUT_OWNER,
                            title: LocalizedText::new("ui.enchant.redeem_code", "REDEEM CODE"),
                            instruction: LocalizedText::new("ui.shared_input.redeem_instruction", "Have a code? Try entering it below to get an exclusive item!"),
                            submit: LocalizedText::new("ui.shared_input.redeem", "REDEEM"),
                            max_utf16: 32,
                        });
                            if !owners.shell.redeem_code_active {
                                match owners.runtime.cancel_redeem_code() {
                                    Ok(output) => owners.shell.push_output(output),
                                    Err(error) => {
                                        owners.status.message =
                                            format!("Redeem dialog ownership rejected: {error}")
                                    }
                                }
                            }
                        }
                        intent @ EnchantPopupIntent0104::RedeemCodeSpaceError { .. } => {
                            owners.shell.redeem_space_error_active = true;
                            if let Some(localized) = intent.localized_message() {
                                queue_enchant_localized_system_message_0104(
                                    &mut owners.shell,
                                    &mut owners.system_messages,
                                    localized,
                                    PendingEnchantSystemAction0104::RedeemSpaceError,
                                );
                            }
                        }
                    }
                }
                EnchantShellEffect0104::RuntimeModal(modal) => {
                    let message_id = match modal {
                        EnchantRuntimeModal0104::DeleteItem { message_id, .. } => message_id,
                    };
                    if let Err(error) = queue_enchant_table_system_message_0104(
                        &mut owners.shell,
                        &mut owners.system_messages,
                        &owners.content,
                        message_id,
                        PendingEnchantSystemAction0104::RuntimeDelete,
                    ) {
                        owners.status.message = error;
                    }
                }
                EnchantShellEffect0104::Selection(intent) => {
                    owners.shell.last_selection_intent = Some(intent);
                }
                EnchantShellEffect0104::DragCaptured { .. } => {}
                EnchantShellEffect0104::HelpRequested => {
                    owners.status.message = ENCHANT_HELP_OWNER_GAP_0104.to_owned();
                }
                EnchantShellEffect0104::RefreshInventory => {}
                EnchantShellEffect0104::TransportFailure(failure) => {
                    owners.status.message = format!(
                        "EnchantMode authoritative failure code {} for target slot {}",
                        failure.error_code, failure.enchant_item_slot
                    );
                }
                EnchantShellEffect0104::AuxiliaryUnlock(packet_type) => {
                    owners.status.message =
                        format!("EnchantMode accepted auxiliary unlock {packet_type:#010x}");
                }
                EnchantShellEffect0104::IgnoredSuccessFlag(flag) => {
                    owners.status.message =
                        format!("EnchantMode preserved clean ignored success flag {flag}");
                }
            }
        }
    }
}

pub(super) fn consume_enchant_text_input(
    mut input: ResMut<ffone_client::shared_input_ui::SharedInputDialog>,
    mut runtime: ResMut<EnchantProductionRuntime0104>,
    mut shell: ResMut<EnchantProductionShell0104>,
    mut status: ResMut<RuntimeStatus>,
) {
    use ffone_client::shared_input_ui::SharedInputAction;
    if runtime.session().is_none() || !shell.redeem_code_active {
        input.close(ENCHANT_INPUT_OWNER);
        return;
    }
    while let Some(action) = input.pop_for(ENCHANT_INPUT_OWNER) {
        let result = match action {
            SharedInputAction::Submit { value, .. } => runtime.submit_redeem_code(&value),
            SharedInputAction::Cancel { .. } => runtime.cancel_redeem_code(),
        };
        match result {
            Ok(output) => shell.push_output(output),
            Err(error) => status.message = format!("Redeem entry rejected: {error}"),
        }
        let open = runtime
            .session()
            .is_some_and(|session| session.model().external_gates().inventory_popup_open);
        shell.redeem_code_active = open;
        if !open {
            input.close(ENCHANT_INPUT_OWNER);
        }
    }
}

pub(super) fn sync_enchant_presentation_0104(
    content: Res<TutorialMissionContent>,
    inventory: Res<LocalInventoryRuntime>,
    runtime: Res<EnchantProductionRuntime0104>,
    mut projection: ResMut<EnchantModeProjection0104>,
) {
    let Some(snapshot) = inventory.snapshot() else {
        *projection = EnchantModeProjection0104::default();
        return;
    };
    let session = runtime.session();
    let inventory_projection = std::array::from_fn(|index| {
        enchant_inventory_slot_projection_0104(
            &content,
            snapshot.inventory()[index],
            session.is_some_and(|session| session.inventory_slot_reserved(index)),
        )
    });
    let equipment_projection = std::array::from_fn(|index| {
        enchant_inventory_slot_projection_0104(&content, snapshot.equipment()[index], false)
    });
    runtime.write_presentation(
        &mut projection,
        enchant_support_presentation_0104(&content),
        inventory_projection,
        equipment_projection,
    );
}
