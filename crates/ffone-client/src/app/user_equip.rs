//! UserEquip system messages, avatar presentation, frames, projection and outboxes.

use super::local_inventory::LocalInventoryRuntime;
use super::runtime_status::{RuntimeStatus, resolve_runtime_resurrection_item_slot};
use super::state::ClientState;
use super::vehicle_transition;
use bevy::{ecs::system::SystemParam, prelude::*};
use ffone_client::{
    avatar_action::LegacyVehiclePresentationFamily,
    bank_runtime::BankProductionRuntime0104,
    entity_lifecycle::NetworkNpcAppearance0104,
    gameplay_audio::GameplayAudioRuntime,
    guide_runtime::{GuideRawMentor, GuideRuntime},
    guide_ui::GUIDE_MENTOR_NAME_LOCALIZATION_KEYS,
    inventory_runtime::{InventoryRuntime0104, ItemIdentity0104},
    localization::LocalizedText,
    mission_ui::MissionUiModel,
    nano_free_tuning_runtime::NanoFreeTuningBank0104,
    network::{NetworkBridge, NetworkCommand},
    quick_slot_ui::QuickSlotUiModel,
    resurrect_ui::ResurrectUiModel,
    system_message_ui::{
        SystemMessageButtonType, SystemMessageChoice, SystemMessageRequest, SystemMessageUiAction,
        SystemMessageUiModel, SystemMessageUiOutbox,
    },
    tutorial_effects_runtime::{TutorialEffectRuntime, TutorialEffectRuntimeCommand},
    tutorial_mission_content::TutorialMissionContent,
    user_equip_runtime::{
        UserEquipPendingRequest0104, UserEquipProductionRuntime0104,
        item_move_reply_matches_request, prepare_user_equip_chest_open_0104,
        prepare_user_equip_delete_0104, prepare_user_equip_move_0104, prepare_user_equip_use_0104,
    },
    user_equip_ui::{
        USER_EQUIP_GUIDE_BEN_TENNYSON_PATH, USER_EQUIP_GUIDE_COMPUTRESS_PATH,
        USER_EQUIP_GUIDE_DEXTER_PATH, USER_EQUIP_GUIDE_EDD_PATH, USER_EQUIP_GUIDE_MOJO_JOJO_PATH,
        UserEquipAvatarPreviewPresentation, UserEquipItemModeProjection, UserEquipItemPopupState,
        UserEquipModalState, UserEquipNanoEquippedAuthority, UserEquipNanoModeProjection,
        UserEquipNanoStationAction, UserEquipPresentationContext, UserEquipProjectedIcon,
        UserEquipSlotEndpoint, UserEquipUiAction, UserEquipUiActionOutcome, UserEquipUiAudioCue,
        UserEquipUiAudioOutbox, UserEquipUiOutbox, UserEquipUiState, apply_user_equip_ui_action,
        nano_station_action_allowed, user_equip_gum_target_enabled,
    },
    vendor_runtime::VendorProductionRuntime0104,
};
use ffone_protocol::{
    DecodedFrame, InventoryPacket0104, ItemBase0104, ItemChestOpenFailure0104,
    ItemChestOpenSuccess0104, ItemUsePacket0104, PcItemDeleteRequest0104, PcItemDeleteSuccess0104,
    WirePayload, decode_inventory_packet_0104, decode_item_use_packet_0104, packet,
};
use std::collections::BTreeMap;

pub(super) const USER_EQUIP_SYSTEM_MESSAGE_ID_BASE_0104: u64 = 0x5553_4551_0000_0000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PendingUserEquipDelete0104 {
    pub(super) owner_pc_id: i32,
    pub(super) request: PcItemDeleteRequest0104,
    pub(super) expected_item: ItemBase0104,
}

#[derive(Debug, Resource)]
pub(super) struct UserEquipSystemMessageRuntime0104 {
    pub(super) next_request_id: u64,
    pub(super) pending_deletes: BTreeMap<u64, PendingUserEquipDelete0104>,
}

impl Default for UserEquipSystemMessageRuntime0104 {
    fn default() -> Self {
        Self {
            next_request_id: USER_EQUIP_SYSTEM_MESSAGE_ID_BASE_0104,
            pending_deletes: BTreeMap::new(),
        }
    }
}

impl UserEquipSystemMessageRuntime0104 {
    pub(super) fn owns(&self, request_id: u64) -> bool {
        self.pending_deletes.contains_key(&request_id)
    }

    pub(super) fn take(&mut self, request_id: u64) -> Option<PendingUserEquipDelete0104> {
        self.pending_deletes.remove(&request_id)
    }

    pub(super) fn has_pending_delete(&self) -> bool {
        !self.pending_deletes.is_empty()
    }

    pub(super) fn queue_delete(
        &mut self,
        owner_pc_id: i32,
        request: PcItemDeleteRequest0104,
        item: ItemBase0104,
        icon_path: Option<String>,
        system_messages: &mut SystemMessageUiModel,
    ) -> u64 {
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.wrapping_add(1);
        if self.next_request_id < USER_EQUIP_SYSTEM_MESSAGE_ID_BASE_0104 {
            self.next_request_id = USER_EQUIP_SYSTEM_MESSAGE_ID_BASE_0104;
        }
        let mut prompt = SystemMessageRequest::new_localized(
            request_id,
            LocalizedText::new("ui.inventory.popup.confirm_delete", "DELETE THIS ITEM?"),
            SystemMessageButtonType::DeleteItem,
        );
        if let Some(icon_path) = icon_path {
            prompt = prompt.with_icon_path(icon_path);
        }
        if item.item_type == 7 {
            prompt = prompt.with_icon_quantity(item.option.max(0));
        }
        self.pending_deletes.insert(
            request_id,
            PendingUserEquipDelete0104 {
                owner_pc_id,
                request,
                expected_item: item,
            },
        );
        system_messages.push(prompt);
        request_id
    }

    pub(super) fn reset(&mut self, system_messages: &mut SystemMessageUiModel) {
        for request_id in self.pending_deletes.keys().copied().collect::<Vec<_>>() {
            system_messages.remove(request_id);
        }
        *self = Self::default();
    }
}

pub(super) const USER_EQUIP_SPECIAL_STATE_FLAG_0104: i8 = 16;
pub(super) const USER_EQUIP_COMPUTER_EFFECT_NAME: &str = "UserEquip player computer";

/// Local ownership for clean `CnEquip.SetEquipMode` plus
/// `cnAvatarAnimation.SetInvenMode`. The authoritative special-state bitfield
/// still comes from the shard; this resource only prevents duplicate toggle
/// requests and drives the delayed AnimationEvent particle callback.
#[derive(Debug, Resource, Default)]
pub(super) struct UserEquipAvatarPresentationRuntime {
    pub(super) active_family: Option<LegacyVehiclePresentationFamily>,
    pub(super) effect_elapsed_seconds: f32,
    pub(super) effect_spawned: bool,
    pub(super) mode_open: bool,
    pub(super) special_state_active: bool,
}

/// Server-confirmed personal vehicle state. `pending` prevents key-repeat and
/// UI clicks from issuing overlapping on/off requests; `family` changes only
/// on the clean `PC_VEHICLE_*_SUCC` replies.
#[derive(Debug, Resource, Default)]
pub(super) struct LocalVehiclePresentationRuntime {
    pub(super) family: LegacyVehiclePresentationFamily,
    pub(super) pending: Option<vehicle_transition::PendingVehicleRequest>,
}

pub(super) fn apply_user_equip_frame(
    frame: &DecodedFrame,
    inventory: &mut LocalInventoryRuntime,
    content: &TutorialMissionContent,
    production: &mut UserEquipProductionRuntime0104,
    bank_production: &BankProductionRuntime0104,
    vendor_production: &VendorProductionRuntime0104,
    runtime: &mut RuntimeStatus,
) -> Result<bool, String> {
    if production.owns_chest_open_reply_family() {
        match frame.packet_type {
            packet::P_FE2CL_REP_ITEM_CHEST_OPEN_SUCC => {
                let reply = ItemChestOpenSuccess0104::decode(&frame.payload)
                    .map_err(|error| format!("UserEquip malformed chest-open success: {error}"))?;
                if production.observe_chest_open_success(reply) {
                    runtime.message = format!(
                        "OpenFusion accepted chest open in inventory slot {}; authoritative reward synchronized",
                        reply.slot_num
                    );
                    return Ok(true);
                }
            }
            packet::P_FE2CL_REP_ITEM_CHEST_OPEN_FAIL => {
                let reply = ItemChestOpenFailure0104::decode(&frame.payload)
                    .map_err(|error| format!("UserEquip malformed chest-open failure: {error}"))?;
                if production.observe_chest_open_failure(reply) {
                    runtime.message = format!(
                        "OpenFusion rejected chest open in inventory slot {} with error {}",
                        reply.slot_num, reply.error_code
                    );
                    return Ok(true);
                }
            }
            _ => {}
        }
    }
    if frame.packet_type == packet::P_FE2CL_REP_PC_ITEM_DELETE_SUCC
        && production.owns_delete_reply_family()
    {
        let reply = PcItemDeleteSuccess0104::decode(&frame.payload)
            .map_err(|error| format!("UserEquip malformed item-delete success: {error}"))?;
        let user_delete_is_active = matches!(
            production.pending(),
            Some(UserEquipPendingRequest0104::Delete(_))
        );
        let vendor_owns_reply = vendor_production
            .pending_delete_request()
            .is_some_and(|request| {
                request.item_location == reply.item_location && request.slot_num == reply.slot_num
            });
        if !user_delete_is_active && vendor_owns_reply {
            // A new Vendor delete supersedes a timed-out UserEquip tombstone
            // for the identical endpoint. Yield so Vendor can release its
            // active lock and apply the one authoritative post-state.
            return Ok(false);
        }
        if !production.observe_delete_success(reply) {
            // Active and timed-out UserEquip deletes both retain ownership of
            // this packet family. Never leak a mismatch into Vendor.
            return Err(format!(
                "UserEquip rejected uncorrelated delete success at location {} slot {}",
                reply.item_location, reply.slot_num
            ));
        }
        let snapshot = inventory
            .snapshot_mut()
            .ok_or_else(|| "UserEquip delete reply arrived before PC load".to_owned())?;
        snapshot
            .apply_item_delete_success(reply)
            .map_err(|error| format!("UserEquip delete post-state rejected: {error}"))?;
        runtime.resurrection_item_slot = resolve_runtime_resurrection_item_slot(snapshot, content);
        runtime.message =
            "OpenFusion accepted UserEquip item deletion; authoritative inventory synchronized"
                .to_owned();
        return Ok(true);
    }
    if production.owns_move_reply_family() {
        let packet = decode_inventory_packet_0104(frame.packet_type, &frame.payload)
            .map_err(|error| format!("UserEquip malformed inventory reply: {error}"))?;
        if let Some(InventoryPacket0104::ItemMoveSuccess(reply)) = packet {
            let active_user_move_matches = matches!(
                production.pending(),
                Some(UserEquipPendingRequest0104::Move(request))
                    if item_move_reply_matches_request(request, reply)
            );
            if !active_user_move_matches && bank_production.owns_item_move_reply(reply) {
                // A currently pending Bank move with the exact same endpoint
                // pair supersedes only an old UserEquip tombstone. Let Bank
                // commit and release its own lock exactly once.
                return Ok(false);
            }
            if production.observe_inventory_packet(&InventoryPacket0104::ItemMoveSuccess(reply)) {
                let snapshot = inventory
                    .snapshot_mut()
                    .ok_or_else(|| "UserEquip item-move reply arrived before PC load".to_owned())?;
                snapshot
                    .apply_item_move_success(reply)
                    .map_err(|error| format!("UserEquip move post-state rejected: {error}"))?;
                runtime.resurrection_item_slot =
                    resolve_runtime_resurrection_item_slot(snapshot, content);
                // Consume here instead of falling through to BankMode. A late
                // UserEquip reply must never unlock or mutate a newer Bank
                // operation, while the authoritative post-state is still
                // applied once.
                runtime.message =
                    "OpenFusion accepted UserEquip move; authoritative slots synchronized"
                        .to_owned();
                return Ok(true);
            }
        }
    }
    if production.owns_use_reply_family() {
        let packet = decode_item_use_packet_0104(frame.packet_type, &frame.payload)
            .map_err(|error| format!("UserEquip malformed item-use reply: {error}"))?;
        let owner_pc_id = inventory.snapshot().map(InventoryRuntime0104::owner_pc_id);
        if let Some(packet) = packet
            && production.observe_item_use_packet_for_owner(&packet, owner_pc_id)
        {
            runtime.message = match packet {
                ItemUsePacket0104::Success(_) => {
                    "OpenFusion accepted item use; authoritative inventory synchronized".to_owned()
                }
                ItemUsePacket0104::Failure(failure) => format!(
                    "OpenFusion rejected item use with error {}",
                    failure.error_code
                ),
                ItemUsePacket0104::Broadcast(_) => unreachable!(
                    "UserEquip item-use broadcasts do not correlate with its local request"
                ),
            };
        }
    }
    Ok(false)
}

#[must_use]
pub(super) const fn user_equip_computer_effect(family: LegacyVehiclePresentationFamily) -> (i32, f32) {
    match family {
        LegacyVehiclePresentationFamily::None => (425, 0.25),
        LegacyVehiclePresentationFamily::Board => (833, 0.3),
        LegacyVehiclePresentationFamily::Scooter => (834, 0.3),
    }
}

pub(super) fn project_quick_slots_from_inventory(
    model: &mut QuickSlotUiModel,
    inventory: Option<&InventoryRuntime0104>,
    content: &TutorialMissionContent,
) {
    for entry in &mut model.slots {
        if !entry.has_item_id() {
            entry.icon_path = None;
            entry.inventory_empty = false;
            continue;
        }

        // Clean QuickInfo always resolves through GeneralItem's icon table.
        entry.icon_path = i16::try_from(entry.item_id)
            .ok()
            .and_then(|item_id| content.general_item_icon_path(item_id))
            .map(str::to_owned);
        let identity = i16::try_from(entry.item_type)
            .ok()
            .zip(i16::try_from(entry.item_id).ok())
            .and_then(|(item_type, item_id)| ItemIdentity0104::new(item_type, item_id).ok());
        entry.inventory_empty = match (inventory, identity) {
            (Some(inventory), Some(identity)) => {
                inventory.find_unique_inventory_slot(identity).is_err()
            }
            _ => true,
        };
    }
}

pub(super) fn refresh_user_equip_projection(
    projection: &mut UserEquipItemModeProjection,
    inventory: Option<&InventoryRuntime0104>,
    content: &TutorialMissionContent,
) -> bool {
    let next = inventory.map_or_else(UserEquipItemModeProjection::default, |inventory| {
        UserEquipItemModeProjection::from_authoritative(inventory, content)
    });
    if *projection == next {
        false
    } else {
        *projection = next;
        true
    }
}

pub(super) fn user_equip_authority<'a>(
    inventory: &'a LocalInventoryRuntime,
    projection: &UserEquipItemModeProjection,
    player_id: Option<i32>,
) -> Result<&'a InventoryRuntime0104, String> {
    let snapshot = inventory
        .snapshot()
        .ok_or_else(|| "authoritative inventory is not loaded".to_owned())?;
    if player_id != Some(snapshot.owner_pc_id()) {
        return Err(format!(
            "inventory owner {} does not match local PC {player_id:?}",
            snapshot.owner_pc_id()
        ));
    }
    if projection.owner_pc_id != snapshot.owner_pc_id() {
        return Err(format!(
            "UI projection owner {} does not match inventory owner {}",
            projection.owner_pc_id,
            snapshot.owner_pc_id()
        ));
    }
    Ok(snapshot)
}

pub(super) fn user_equip_use_nano_slot(
    item: ItemBase0104,
    content: &TutorialMissionContent,
) -> Result<i16, String> {
    match item.item_type {
        7 => match content.general_item_type(item.item_id) {
            // Clean one-click `DoClick` allow-list.
            Some(4 | 5 | 7 | 8 | 9 | 50) => Ok(0),
            // Popup subtype 3 owns an explicit `UseInventoryItemOnNano`
            // request. Never guess a Nano slot for the one-click route.
            Some(3) => Err(format!(
                "GeneralItem {} is a targeted stim and requires a selected Nano slot",
                item.item_id
            )),
            Some(item_type) => Err(format!(
                "GeneralItem {} subtype {item_type} has no clean UserEquip use route",
                item.item_id
            )),
            None => Err(format!(
                "GeneralItem {} is missing authoritative subtype metadata",
                item.item_id
            )),
        },
        item_type => Err(format!(
            "item type {item_type} has no clean UserEquip use route"
        )),
    }
}

pub(super) fn reset_user_equip_shell(
    state: &mut UserEquipUiState,
    modal: &mut UserEquipModalState,
    projection: &mut UserEquipItemModeProjection,
    outbox: &mut UserEquipUiOutbox,
) {
    state.close();
    *modal = UserEquipModalState::default();
    projection.reset();
    outbox.clear();
}

#[derive(SystemParam)]
pub(super) struct UserEquipSyncOwners<'w> {
    pub(super) bank_delete: Option<Res<'w, ffone_client::bank_ui::BankItemDeleteState>>,
    pub(super) system_messages: ResMut<'w, SystemMessageUiModel>,
    pub(super) runtime: ResMut<'w, RuntimeStatus>,
    pub(super) state: ResMut<'w, UserEquipUiState>,
    pub(super) modal: ResMut<'w, UserEquipModalState>,
    pub(super) projection: ResMut<'w, UserEquipItemModeProjection>,
    pub(super) nano_projection: ResMut<'w, UserEquipNanoModeProjection>,
    pub(super) popup: ResMut<'w, UserEquipItemPopupState>,
    pub(super) presentation: ResMut<'w, UserEquipPresentationContext>,
    pub(super) outbox: ResMut<'w, UserEquipUiOutbox>,
    pub(super) production: ResMut<'w, UserEquipProductionRuntime0104>,
    pub(super) system_runtime: ResMut<'w, UserEquipSystemMessageRuntime0104>,
}

pub(super) fn reset_user_equip_session(
    mut owners: UserEquipSyncOwners,
    mut audio_outbox: ResMut<UserEquipUiAudioOutbox>,
    mut avatar_mode: ResMut<UserEquipAvatarPresentationRuntime>,
    mut avatar_preview: ResMut<UserEquipAvatarPreviewPresentation>,
    mut effects: ResMut<TutorialEffectRuntime>,
) {
    reset_user_equip_shell(
        &mut owners.state,
        &mut owners.modal,
        &mut owners.projection,
        &mut owners.outbox,
    );
    owners.nano_projection.reset();
    audio_outbox.clear();
    owners.popup.close();
    *owners.presentation = UserEquipPresentationContext::default();
    owners.production.reset();
    owners.system_runtime.reset(&mut owners.system_messages);
    if avatar_mode.effect_spawned {
        effects.enqueue(TutorialEffectRuntimeCommand::DestroyNamed {
            name: USER_EQUIP_COMPUTER_EFFECT_NAME.to_owned(),
            source_line: line!(),
        });
    }
    *avatar_mode = UserEquipAvatarPresentationRuntime::default();
    *avatar_preview = UserEquipAvatarPreviewPresentation::default();
}

pub(super) fn sync_user_equip_ui_context(
    time: Res<Time>,
    client_state: Res<State<ClientState>>,
    inventory: Res<LocalInventoryRuntime>,
    content: Res<TutorialMissionContent>,
    guide_runtime: Res<GuideRuntime>,
    mission_ui: Res<MissionUiModel>,
    resurrect_ui: Res<ResurrectUiModel>,
    bank_production: Res<BankProductionRuntime0104>,
    nano_bank: Res<NanoFreeTuningBank0104>,
    npcs: Query<&NetworkNpcAppearance0104>,
    mut owners: UserEquipSyncOwners,
) {
    let inventory_changed = inventory.is_changed();
    let inventory_snapshot = inventory.snapshot();
    if let Some(npc_id) = owners.state.nano_station_npc()
        && !npcs
            .iter()
            .any(|npc| npc.0.npc_id == npc_id && npc.0.hp > 0)
    {
        owners.state.close();
        owners.outbox.clear();
    }
    if let Some(expired) = owners.production.tick(time.delta_secs()) {
        owners.runtime.message = format!(
            "UserEquip {:?} request timed out; controls unlocked without speculative mutation",
            expired.kind()
        );
    }
    if *client_state.get() != ClientState::World
        || inventory_snapshot.is_none()
        || resurrect_ui.visible
        || bank_production.modal_active()
    {
        if owners.state.is_active()
            || *owners.modal != UserEquipModalState::default()
            || owners.projection.owner_pc_id != 0
            || *owners.nano_projection != UserEquipNanoModeProjection::default()
            || !owners.outbox.is_empty()
            || owners.system_runtime.has_pending_delete()
            || *owners.popup != UserEquipItemPopupState::default()
            || *owners.presentation != UserEquipPresentationContext::default()
        {
            reset_user_equip_shell(
                &mut owners.state,
                &mut owners.modal,
                &mut owners.projection,
                &mut owners.outbox,
            );
            owners.nano_projection.reset();
            owners.popup.close();
            *owners.presentation = UserEquipPresentationContext::default();
            owners.system_runtime.reset(&mut owners.system_messages);
        }
        if *client_state.get() == ClientState::Tutorial && inventory_snapshot.is_some() {
            // QuickSlot is live in Tutorial and shares the nonce-less Use
            // reply family with UserEquip. Closing the World-only shell must
            // not erase a tutorial request that is still awaiting its reply.
        } else if *client_state.get() == ClientState::World && inventory_snapshot.is_some() {
            // Bank/Resurrect replace the shell but not the server reply that
            // may already be in flight. Preserve bounded routing ownership so
            // a late UserEquip move cannot satisfy a newer Bank request.
            if !owners
                .bank_delete
                .as_ref()
                .is_some_and(|deletion| deletion.sent().is_some())
            {
                owners.production.close_session_preserving_late_replies();
            }
        } else {
            owners.production.reset();
        }
        return;
    }
    // Preserve UI-owned Help/item-popup state while synchronizing the two
    // production-owned clean hard gates.
    owners.modal.send_pending =
        owners.production.send_pending() || owners.state.nano_station_send_pending();
    owners.modal.system_popup_active =
        mission_ui.system_popup_active() || owners.system_messages.is_popup();

    let guide_presentation = guide_runtime
        .authoritative()
        .and_then(|guide| guide.recognized_mentor())
        .map(|guide| match guide {
            GuideRawMentor::Selectable(mentor) => (
                mentor.name(),
                GUIDE_MENTOR_NAME_LOCALIZATION_KEYS[mentor.slot()],
                [
                    USER_EQUIP_GUIDE_EDD_PATH,
                    USER_EQUIP_GUIDE_DEXTER_PATH,
                    USER_EQUIP_GUIDE_MOJO_JOJO_PATH,
                    USER_EQUIP_GUIDE_BEN_TENNYSON_PATH,
                ][mentor.slot()],
            ),
            GuideRawMentor::ComputressFuture => (
                "COMPUTRESS",
                "ui.inventory.status.guide_computress",
                USER_EQUIP_GUIDE_COMPUTRESS_PATH,
            ),
        });
    *owners.presentation = UserEquipPresentationContext {
        player_name: owners.runtime.player_name.clone(),
        level: i32::from(owners.runtime.player_level),
        gender: owners.runtime.player_gender.unwrap_or(0),
        guide: guide_runtime
            .authoritative()
            .map_or(0, |guide| i32::from(guide.raw_mentor())),
        hp: owners.runtime.hp.unwrap_or_default(),
        max_hp: owners.runtime.max_hp,
        fusion_matter: owners.runtime.fusion_matter,
        max_fusion_matter: owners.runtime.max_fusion_matter,
        taros: i64::from(owners.runtime.candy),
        weapon_battery: owners.runtime.weapon_battery,
        nano_battery: owners.runtime.nano_battery,
        guide_name: guide_presentation
            .map(|(name, _, _)| name.to_owned())
            .unwrap_or_default(),
        guide_name_key: guide_presentation.map(|(_, key, _)| key.to_owned()),
        guide_icon_path: guide_presentation.map(|(_, _, icon)| icon.to_owned()),
    };

    let inventory = inventory_snapshot.expect("authoritative inventory was checked above");
    if owners.projection.owner_pc_id != inventory.owner_pc_id()
        || inventory_changed
        || content.is_changed()
    {
        refresh_user_equip_projection(&mut owners.projection, Some(inventory), &content);
    }
    let equipped = owners
        .runtime
        .nano_slots
        .map(|slot| UserEquipNanoEquippedAuthority {
            nano_id: slot.nano_id,
            skill_id: slot.skill_id,
            stamina: slot.stamina,
            active: slot.active,
        });
    let next_nano_projection =
        UserEquipNanoModeProjection::from_authoritative(nano_bank.entries(), equipped, &content);
    if *owners.nano_projection != next_nano_projection {
        *owners.nano_projection = next_nano_projection;
    }
}

#[derive(SystemParam)]
pub(super) struct UserEquipActionOwners<'w> {
    pub(super) bridge: Res<'w, NetworkBridge>,
    pub(super) inventory: Res<'w, LocalInventoryRuntime>,
    pub(super) projection: Res<'w, UserEquipItemModeProjection>,
    pub(super) nano_projection: Res<'w, UserEquipNanoModeProjection>,
    pub(super) content: Res<'w, TutorialMissionContent>,
    pub(super) production: ResMut<'w, UserEquipProductionRuntime0104>,
    pub(super) system_runtime: ResMut<'w, UserEquipSystemMessageRuntime0104>,
    pub(super) system_messages: ResMut<'w, SystemMessageUiModel>,
    pub(super) runtime: ResMut<'w, RuntimeStatus>,
}

pub(super) fn consume_user_equip_ui_outbox(
    mut outbox: ResMut<UserEquipUiOutbox>,
    mut audio_outbox: ResMut<UserEquipUiAudioOutbox>,
    mut gameplay_audio: ResMut<GameplayAudioRuntime>,
    mut state: ResMut<UserEquipUiState>,
    mut modal: ResMut<UserEquipModalState>,
    mut owners: UserEquipActionOwners,
) {
    for cue in audio_outbox.drain() {
        match cue {
            UserEquipUiAudioCue::ButtonSound => gameplay_audio.queue_legacy_button_sound(),
            UserEquipUiAudioCue::OpenScreen => {
                gameplay_audio.queue_gameplay_ui_sound("Open_Screen");
            }
            UserEquipUiAudioCue::TabClick01 => {
                gameplay_audio.queue_gameplay_ui_sound("Tab_Click01");
            }
        }
    }
    for action in outbox.drain().collect::<Vec<_>>() {
        if let UserEquipUiAction::NanoStation(request) = action {
            if !nano_station_action_allowed(&state, *modal, &owners.nano_projection, request)
                || owners.production.send_pending()
                || owners.system_runtime.has_pending_delete()
                || user_equip_authority(
                    &owners.inventory,
                    &owners.projection,
                    owners.runtime.player_id,
                )
                .is_err()
            {
                owners.runtime.message =
                    "Nano Station request rejected by ownership or modal state".to_owned();
                continue;
            }
            let command = match request {
                UserEquipNanoStationAction::Equip { nano_id, slot } => {
                    NetworkCommand::EquipNano(ffone_protocol::NanoEquipRequest0104 {
                        nano_id,
                        nano_slot: slot as i16,
                    })
                }
                UserEquipNanoStationAction::Unequip { slot, .. } => {
                    NetworkCommand::UnequipNano(ffone_protocol::NanoUnequipRequest0104 {
                        nano_slot: slot as i16,
                    })
                }
            };
            match owners.bridge.send(command) {
                Ok(()) => {
                    state.begin_nano_station_request(request);
                    modal.send_pending = true;
                }
                Err(error) => {
                    owners.runtime.message = format!("Nano Station transport failed: {error}")
                }
            }
            continue;
        }
        let outcome = apply_user_equip_ui_action(&mut state, &mut modal, action);
        match outcome {
            UserEquipUiActionOutcome::CloseBlocked(reason) => {
                owners.runtime.message = format!("UserEquip close blocked: {reason:?}");
            }
            UserEquipUiActionOutcome::Closed
            | UserEquipUiActionOutcome::DismissedPopups
            | UserEquipUiActionOutcome::Scrolled { .. }
            | UserEquipUiActionOutcome::ScrollBlocked
            | UserEquipUiActionOutcome::TabSelected(_) => {}
            UserEquipUiActionOutcome::RequestQueued => {
                if !state.is_active() {
                    owners.runtime.message =
                        "UserEquip request rejected after its mode closed".to_owned();
                    continue;
                }
                if owners.production.send_pending() {
                    owners.runtime.message =
                        "UserEquip request blocked while an authoritative reply is pending"
                            .to_owned();
                    continue;
                }
                if owners.system_runtime.has_pending_delete() {
                    owners.runtime.message =
                        "UserEquip request blocked by its item-delete confirmation".to_owned();
                    continue;
                }
                let authority = match user_equip_authority(
                    &owners.inventory,
                    &owners.projection,
                    owners.runtime.player_id,
                ) {
                    Ok(authority) => authority,
                    Err(error) => {
                        owners.runtime.message =
                            format!("UserEquip authority check rejected request: {error}");
                        continue;
                    }
                };
                match action {
                    UserEquipUiAction::MoveItem { from, to } => {
                        let request = match prepare_user_equip_move_0104(authority, from, to) {
                            Ok(request) => request,
                            Err(error) => {
                                owners.runtime.message =
                                    format!("UserEquip move rejected before send: {error}");
                                continue;
                            }
                        };
                        if let Err(error) = owners
                            .production
                            .begin(UserEquipPendingRequest0104::Move(request))
                        {
                            owners.runtime.message =
                                format!("UserEquip move rejected before send: {error}");
                            continue;
                        }
                        if let Err(error) = owners.bridge.send(NetworkCommand::MoveItem(request)) {
                            owners.production.cancel();
                            owners.runtime.message =
                                format!("UserEquip move transport failed: {error}");
                            continue;
                        }
                        modal.send_pending = true;
                        owners.runtime.message = format!(
                            "UserEquip sent item move ({}, {}) -> ({}, {}); awaiting authority",
                            request.from_location,
                            request.from_slot_num,
                            request.to_location,
                            request.to_slot_num
                        );
                    }
                    action @ (UserEquipUiAction::UseInventoryItem { .. }
                    | UserEquipUiAction::UseInventoryItemOnNano { .. }) => {
                        let (slot_index, requested_nano_slot) = match action {
                            UserEquipUiAction::UseInventoryItem { slot_index } => {
                                (slot_index, None)
                            }
                            UserEquipUiAction::UseInventoryItemOnNano {
                                slot_index,
                                nano_slot,
                            } => (slot_index, Some(nano_slot)),
                            _ => unreachable!("matched only item-use actions"),
                        };
                        let Some(item) = authority.inventory().get(slot_index).copied() else {
                            owners.runtime.message = format!(
                                "UserEquip item use rejected: inventory slot {slot_index} is out of bounds"
                            );
                            continue;
                        };
                        let nano_slot = match requested_nano_slot {
                            Some(nano_slot) => {
                                if !user_equip_gum_target_enabled(
                                    item.item_id,
                                    nano_slot,
                                    &owners.nano_projection,
                                    &owners.content,
                                ) {
                                    owners.runtime.message = format!(
                                        "UserEquip targeted item use rejected: item {} cannot target Nano slot {nano_slot}",
                                        item.item_id
                                    );
                                    continue;
                                }
                                match i16::try_from(nano_slot) {
                                    Ok(nano_slot) => nano_slot,
                                    Err(_) => {
                                        owners.runtime.message = format!(
                                            "UserEquip targeted item use rejected: Nano slot {nano_slot} does not fit protocol i16"
                                        );
                                        continue;
                                    }
                                }
                            }
                            None => match user_equip_use_nano_slot(item, &owners.content) {
                                Ok(nano_slot) => nano_slot,
                                Err(error) => {
                                    owners.runtime.message =
                                        format!("UserEquip item use rejected before send: {error}");
                                    continue;
                                }
                            },
                        };
                        let request =
                            match prepare_user_equip_use_0104(authority, slot_index, nano_slot) {
                                Ok(request) => request,
                                Err(error) => {
                                    owners.runtime.message =
                                        format!("UserEquip item use rejected before send: {error}");
                                    continue;
                                }
                            };
                        if let Err(error) = owners
                            .production
                            .begin(UserEquipPendingRequest0104::Use(request))
                        {
                            owners.runtime.message =
                                format!("UserEquip item use rejected before send: {error}");
                            continue;
                        }
                        if let Err(error) = owners.bridge.send(NetworkCommand::UseItem(request)) {
                            owners.production.cancel();
                            owners.runtime.message =
                                format!("UserEquip item-use transport failed: {error}");
                            continue;
                        }
                        modal.send_pending = true;
                        owners.runtime.message = format!(
                            "UserEquip sent item use from inventory slot {slot_index}; awaiting authority"
                        );
                    }
                    UserEquipUiAction::OpenInventoryChest { slot_index } => {
                        let request =
                            match prepare_user_equip_chest_open_0104(authority, slot_index) {
                                Ok(request) => request,
                                Err(error) => {
                                    owners.runtime.message = format!(
                                        "UserEquip chest open rejected before send: {error}"
                                    );
                                    continue;
                                }
                            };
                        if let Err(error) = owners
                            .production
                            .begin(UserEquipPendingRequest0104::ChestOpen(request))
                        {
                            owners.runtime.message =
                                format!("UserEquip chest open rejected before send: {error}");
                            continue;
                        }
                        if let Err(error) = owners.bridge.send(NetworkCommand::OpenChest(request)) {
                            owners.production.cancel();
                            owners.runtime.message =
                                format!("UserEquip chest-open transport failed: {error}");
                            continue;
                        }
                        modal.send_pending = true;
                        owners.runtime.message = format!(
                            "UserEquip sent chest open from inventory slot {slot_index}; awaiting authority"
                        );
                    }
                    UserEquipUiAction::DeleteInventoryItem { slot_index } => {
                        let request = match prepare_user_equip_delete_0104(authority, slot_index) {
                            Ok(request) => request,
                            Err(error) => {
                                owners.runtime.message =
                                    format!("UserEquip item delete rejected: {error}");
                                continue;
                            }
                        };
                        let endpoint = UserEquipSlotEndpoint::Inventory { slot_index };
                        let projected = owners.projection.item_at(endpoint);
                        let item = authority.inventory()[slot_index];
                        if projected
                            .is_none_or(|projected| projected.empty || projected.item != item)
                        {
                            owners.runtime.message =
                                "UserEquip item delete rejected: UI projection is stale".to_owned();
                            continue;
                        }
                        let icon_path = projected.and_then(|projected| match &projected.icon {
                            UserEquipProjectedIcon::Resolved(icon) => {
                                Some(icon.runtime_path().to_owned())
                            }
                            UserEquipProjectedIcon::Empty
                            | UserEquipProjectedIcon::MissingChecker(_) => None,
                        });
                        let request_id = owners.system_runtime.queue_delete(
                            authority.owner_pc_id(),
                            request,
                            item,
                            icon_path,
                            &mut owners.system_messages,
                        );
                        modal.system_popup_active = true;
                        owners.runtime.message =
                            format!("UserEquip queued clean item-delete confirmation {request_id}");
                    }
                    UserEquipUiAction::NanoStation(_)
                    | UserEquipUiAction::RequestClose { .. }
                    | UserEquipUiAction::ApplyLegacyScrollAxis { .. }
                    | UserEquipUiAction::SetScrollY { .. }
                    | UserEquipUiAction::SelectTab { .. } => {
                        unreachable!("local UserEquip actions do not yield RequestQueued")
                    }
                }
            }
        }
    }
}

pub(super) fn consume_user_equip_system_message_outbox(
    mut outbox: ResMut<SystemMessageUiOutbox>,
    mut modal: ResMut<UserEquipModalState>,
    mut owners: UserEquipActionOwners,
) {
    let actions = outbox.drain_matching(|action| {
        let SystemMessageUiAction::Chosen { request_id, .. } = action;
        owners.system_runtime.owns(*request_id)
    });
    for action in actions {
        let SystemMessageUiAction::Chosen {
            request_id,
            button_type,
            choice,
        } = action;
        let Some(pending) = owners.system_runtime.take(request_id) else {
            owners.runtime.message =
                format!("UserEquip ignored stale item-delete confirmation {request_id}");
            continue;
        };
        if button_type != SystemMessageButtonType::DeleteItem {
            owners.runtime.message = format!(
                "UserEquip item-delete confirmation {request_id} returned mismatched button type {button_type:?}"
            );
            continue;
        }
        if choice != SystemMessageChoice::Primary {
            owners.runtime.message = "UserEquip item deletion cancelled".to_owned();
            continue;
        }
        if owners.production.send_pending() {
            owners.runtime.message =
                "UserEquip item deletion rejected while another reply is pending".to_owned();
            continue;
        }
        let authority = match user_equip_authority(
            &owners.inventory,
            &owners.projection,
            owners.runtime.player_id,
        ) {
            Ok(authority) => authority,
            Err(error) => {
                owners.runtime.message =
                    format!("UserEquip item deletion authority changed: {error}");
                continue;
            }
        };
        if authority.owner_pc_id() != pending.owner_pc_id {
            owners.runtime.message = format!(
                "UserEquip item deletion owner changed from {} to {}",
                pending.owner_pc_id,
                authority.owner_pc_id()
            );
            continue;
        }
        let Ok(slot_index) = usize::try_from(pending.request.slot_num) else {
            owners.runtime.message = format!(
                "UserEquip item deletion retained an invalid slot {}",
                pending.request.slot_num
            );
            continue;
        };
        let request = match prepare_user_equip_delete_0104(authority, slot_index) {
            Ok(request) => request,
            Err(error) => {
                owners.runtime.message =
                    format!("UserEquip item deletion changed before confirmation: {error}");
                continue;
            }
        };
        if request != pending.request || authority.inventory()[slot_index] != pending.expected_item
        {
            owners.runtime.message =
                "UserEquip item deletion rejected because the confirmed slot changed".to_owned();
            continue;
        }
        if let Err(error) = owners
            .production
            .begin(UserEquipPendingRequest0104::Delete(request))
        {
            owners.runtime.message =
                format!("UserEquip item deletion rejected before send: {error}");
            continue;
        }
        if let Err(error) = owners
            .bridge
            .send(NetworkCommand::DeleteInventoryItem(request))
        {
            owners.production.cancel();
            owners.runtime.message = format!("UserEquip item-delete transport failed: {error}");
            continue;
        }
        modal.send_pending = true;
        owners.runtime.message = format!(
            "UserEquip sent confirmed item deletion for inventory slot {slot_index}; awaiting authority"
        );
    }
}
