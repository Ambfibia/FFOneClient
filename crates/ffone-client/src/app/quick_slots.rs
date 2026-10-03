//! Quick slot, skill buff and special-state frames and UI contexts.

use super::local_inventory::LocalInventoryRuntime;
use super::modal_gates::GameplayUiModalInputs;
use super::runtime_status::{RuntimeNanoSlot, RuntimeStatus};
use super::state::ClientState;
use super::user_equip::project_quick_slots_from_inventory;
use super::{LocalPlayer, movement_buffs};
use bevy::prelude::*;
use ffone_client::{
    avatar_action::{LegacyAvatarActionState, LegacyTargetKind},
    bank_ui::BankLifecyclePhase,
    entity_lifecycle::NetworkNpcAppearance0104,
    gameplay_ui::{ChatChannel, GameplayUiModel},
    inventory_runtime::{InventoryLookupError0104, InventoryRuntime0104, ItemIdentity0104},
    network::{NetworkBridge, NetworkCommand},
    quick_slot_ui::{
        LegacyQuickSlotEntry, QuickSlotActivationSource, QuickSlotUiAction, QuickSlotUiModel,
        QuickSlotUiOutbox,
    },
    quit_menu_ui::QuitMenuUiModel,
    resurrect_ui::ResurrectUiModel,
    skill_buff_ui::SkillBuffUiModel,
    tutorial_actors::TutorialActor,
    tutorial_mission_content::TutorialMissionContent,
    user_equip_runtime::{UserEquipPendingRequest0104, UserEquipProductionRuntime0104},
    vendor_ui::VendorLifecyclePhase,
    world_map::WorldMapPhase,
};
use ffone_protocol::{
    DecodedFrame, ItemUseRequest0104, QuickSlotPacket0104, TimeBuffDotDamageTick0104, WirePayload,
    decode_quick_slot_packet_0104, decode_skill_buff_packet_0104, decode_special_state_change_0104,
    packet,
};
use std::collections::BTreeSet;

pub(super) fn sync_quick_slot_ui_context(
    state: Res<State<ClientState>>,
    gameplay_ui: Res<GameplayUiModel>,
    quit_menu: Res<QuitMenuUiModel>,
    resurrect_ui: Res<ResurrectUiModel>,
    modals: GameplayUiModalInputs,
    inventory: Res<LocalInventoryRuntime>,
    content: Res<TutorialMissionContent>,
    item_use_owner: Res<UserEquipProductionRuntime0104>,
    mut quick_slots: ResMut<QuickSlotUiModel>,
) {
    let in_game = matches!(state.get(), ClientState::Tutorial | ClientState::World);
    project_quick_slots_from_inventory(&mut quick_slots, inventory.snapshot(), &content);
    if !in_game {
        quick_slots.ready_for_play = false;
    }
    // Clean `CnGuiChat.OnOwnGUI` passes bEnClick=false while FRIEND owns the
    // chat-adjacent area. Keyboard input remains independently defined.
    quick_slots.clicks_enabled = in_game
        && !quit_menu.visible
        && !resurrect_ui.visible
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
        && !modals.user_equip_ui.is_active()
        && !modals.pc2pc_ui.state.phase.renders_shell()
        && modals.world_map.model.phase() == WorldMapPhase::Closed
        && !modals.transportation_modal()
        && !modals.email_runtime.modal_active()
        && !modals.combi_runtime.modal_active()
        && !modals.cashmall_modal()
        && !modals.user_store_modal()
        && !item_use_owner.owns_use_reply_family()
        && gameplay_ui.chat.selected != ChatChannel::Buddy;
}

pub(super) fn sync_skill_buff_ui_context(
    runtime: Res<RuntimeStatus>,
    gameplay_ui: Res<GameplayUiModel>,
    content: Res<TutorialMissionContent>,
    players: Query<&LegacyAvatarActionState, With<LocalPlayer>>,
    npc_appearances: Query<Ref<NetworkNpcAppearance0104>>,
    tutorial_actors: Query<&TutorialActor>,
    mut skill_buffs: ResMut<SkillBuffUiModel>,
) {
    let mut next = skill_buffs.clone();
    next.visible = gameplay_ui.visible;
    next.ui_scale = gameplay_ui.ui_scale;
    next.nano_styles = skill_buff_nano_styles(runtime.nano_slots, &content);
    let live_targets = npc_appearances
        .iter()
        .map(|appearance| (2, appearance.0.npc_id))
        .chain(tutorial_actors.iter().map(|actor| (2, actor.id)))
        .collect::<BTreeSet<_>>();
    next.retain_target_conditions(&live_targets);
    let focused_npc = players
        .iter()
        .find_map(|state| state.target_selection.focused_npc)
        // `cnAvatarAttack.UpdateTargetUI` hides a friendly MobInfo panel until
        // the exact talk-range result is active; the buff row shares that
        // selected-status visibility rather than the broader view cone.
        .filter(|target| skill_buff_target_visible(target.kind, target.talk_enabled));
    if let Some((appearance, appearance_changed)) = focused_npc
        .and_then(|target| npc_appearances.get(target.entity).ok())
        .map(|appearance| {
            let changed = appearance.is_changed();
            (appearance, changed)
        })
    {
        // OpenFusion normalizes mob time-buff timeout broadcasts to the
        // clean client's NPC character type before this UI sees them.
        next.select_target_from_appearance(
            2,
            appearance.0.npc_id,
            appearance.0.condition_bit_flag as u32,
            appearance_changed,
        );
    } else if let Some(actor) =
        focused_npc.and_then(|target| tutorial_actors.get(target.entity).ok())
    {
        // The isolated tutorial actors have no server Status component. A
        // zero appearance seed keeps their target row source-honest while
        // still allowing later condition-bearing packets to update the same
        // owner cache if the transport supplies one.
        next.select_target_from_appearance(2, actor.id, 0, false);
    } else {
        next.clear_target_selection();
    }
    if *skill_buffs != next {
        *skill_buffs = next;
    }
}

pub(super) const fn skill_buff_target_visible(kind: LegacyTargetKind, talk_enabled: bool) -> bool {
    !matches!(kind, LegacyTargetKind::Npc { team: 1 }) || talk_enabled
}

pub(super) fn skill_buff_nano_styles(
    slots: [RuntimeNanoSlot; 3],
    content: &TutorialMissionContent,
) -> [Option<u8>; 3] {
    slots.map(|slot| match slot.nano_id {
        // Clean indexes Nano row zero for an empty iNanoSlot. That row's
        // m_iStyle is zero, so active Stim bits retain one icon per slot.
        None => Some(0),
        Some(nano_id) => content.gameplay_nano(nano_id).map(|nano| nano.style),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum QuickSlotItemUseResolutionError {
    MissingLegacySlot(i32),
    MalformedIdentity { item_type: i32, item_id: i32 },
    NonGeneralItem(i16),
    CooldownActive,
    Inventory(InventoryLookupError0104),
}

impl std::fmt::Display for QuickSlotItemUseResolutionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingLegacySlot(slot) => {
                write!(
                    formatter,
                    "legacy event index {slot} resolves no clean slot"
                )
            }
            Self::MalformedIdentity { item_type, item_id } => {
                write!(
                    formatter,
                    "malformed item identity type={item_type} id={item_id}"
                )
            }
            Self::NonGeneralItem(item_type) => {
                write!(formatter, "item type {item_type} is not GeneralItem 7")
            }
            Self::CooldownActive => formatter.write_str("item remains on clean cooldown"),
            Self::Inventory(error) => write!(formatter, "{error}"),
        }
    }
}

pub(super) fn resolve_quick_slot_item_use_request(
    action: QuickSlotUiAction,
    model: &QuickSlotUiModel,
    inventory: &InventoryRuntime0104,
) -> Result<(ItemUseRequest0104, ItemIdentity0104), QuickSlotItemUseResolutionError> {
    // Preserve the clean pointer quirk: DoQuick sends `i + 1`, while the
    // keyboard path sends `i`; ReceiveUseQuick indexes that event value
    // directly rather than normalizing it back to the painted slot.
    let entry = usize::try_from(action.legacy_event_index)
        .ok()
        .and_then(|slot| model.slots.get(slot))
        .ok_or(QuickSlotItemUseResolutionError::MissingLegacySlot(
            action.legacy_event_index,
        ))?;
    let identity = i16::try_from(entry.item_type)
        .ok()
        .zip(i16::try_from(entry.item_id).ok())
        .and_then(|(item_type, item_id)| ItemIdentity0104::new(item_type, item_id).ok())
        .ok_or(QuickSlotItemUseResolutionError::MalformedIdentity {
            item_type: entry.item_type,
            item_id: entry.item_id,
        })?;
    if identity.item_type() != 7 {
        return Err(QuickSlotItemUseResolutionError::NonGeneralItem(
            identity.item_type(),
        ));
    }
    if entry.cooldown_fraction() > 0.0 {
        return Err(QuickSlotItemUseResolutionError::CooldownActive);
    }
    let slot = inventory
        .find_unique_inventory_slot(identity)
        .map_err(QuickSlotItemUseResolutionError::Inventory)?;
    Ok((
        ItemUseRequest0104 {
            item_location: 1,
            slot_num: slot.wire_index(),
            nano_slot: 0,
        },
        identity,
    ))
}

pub(super) fn consume_quick_slot_ui_outbox(
    mut outbox: ResMut<QuickSlotUiOutbox>,
    quit_menu: Res<QuitMenuUiModel>,
    resurrect_ui: Res<ResurrectUiModel>,
    modals: GameplayUiModalInputs,
    quick_slots: Res<QuickSlotUiModel>,
    inventory: Res<LocalInventoryRuntime>,
    bridge: Res<NetworkBridge>,
    mut item_use_owner: ResMut<UserEquipProductionRuntime0104>,
    mut runtime: ResMut<RuntimeStatus>,
) {
    if quit_menu.visible
        || resurrect_ui.visible
        || modals.guide_production.modal_active(&modals.guide_ui)
        || modals.bank_state.phase != BankLifecyclePhase::Hidden
        || modals.bank_production.modal_active()
        || !modals.bank_outbox.is_empty()
        || modals.vendor_state.phase != VendorLifecyclePhase::Hidden
        || modals.vendor_production.modal_active()
        || !modals.vendor_outbox.is_empty()
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
        || modals.user_store_modal()
    {
        let _ = outbox.drain().count();
        return;
    }
    for action in outbox.drain() {
        let source = match action.source {
            QuickSlotActivationSource::Hotkey => "hotkey",
            QuickSlotActivationSource::Pointer => "pointer",
        };
        let Some(inventory) = inventory.snapshot() else {
            runtime.message = "QuickSlot blocked before authoritative inventory load".to_owned();
            continue;
        };
        let (request, identity) =
            match resolve_quick_slot_item_use_request(action, &quick_slots, inventory) {
                Ok(resolved) => resolved,
                Err(error) => {
                    runtime.message = format!("QuickSlot {source} rejected: {error}");
                    continue;
                }
            };
        // ItemUseFailure0104 has no request endpoint. QuickSlot and UserEquip
        // must therefore share one family owner; otherwise a late failure
        // from either sender could release the other's in-flight action.
        if let Err(error) = item_use_owner.begin(UserEquipPendingRequest0104::Use(request)) {
            runtime.message = format!("QuickSlot {source} item use blocked: {error}");
            continue;
        }
        if let Err(error) = bridge.send(NetworkCommand::UseItem(request)) {
            item_use_owner.cancel();
            runtime.message = format!("QuickSlot item-use transport failed: {error}");
        } else {
            runtime.message = format!(
                "QuickSlot {source} sent GeneralItem {} from inventory slot {}",
                identity.item_id(),
                request.slot_num
            );
        }
    }
}

pub(super) fn apply_special_state_frame(
    frame: &DecodedFrame,
    runtime: &mut RuntimeStatus,
) -> Result<bool, String> {
    let change = decode_special_state_change_0104(frame.packet_type, &frame.payload)
        .map_err(|error| format!("malformed protocol-0104 special-state packet: {error}"))?;
    let Some(change) = change else {
        return Ok(false);
    };
    if runtime.player_id == Some(change.pc_id) {
        runtime.special_state = change.special_state;
        runtime.free_chat = runtime.special_state & 64 == 0;
    }
    Ok(true)
}

pub(super) fn apply_quick_slot_frame(
    frame: &DecodedFrame,
    model: &mut QuickSlotUiModel,
) -> Result<bool, String> {
    let Some(packet) = decode_quick_slot_packet_0104(frame.packet_type, &frame.payload)
        .map_err(|error| error.to_string())?
    else {
        // Item-use success/broadcast ownership is handled by the dedicated
        // strict eST-dependent decoder and the inventory/entity consumers.
        return Ok(false);
    };
    match packet {
        QuickSlotPacket0104::Info(info) => {
            model.slots = info.slots.map(|slot| LegacyQuickSlotEntry {
                item_type: i32::from(slot.item_type),
                item_id: i32::from(slot.item_id),
                // The roster contains item type/ID only. Resolving the exact
                // icon and inventory slot belongs to the inventory runtime.
                icon_path: None,
                inventory_empty: false,
                cooldown_remaining_fraction: 0.0,
            });
            model.ready_for_play = true;
            Ok(true)
        }
        QuickSlotPacket0104::RegisterSuccess(success) => {
            let slot = usize::try_from(success.slot_num)
                .ok()
                .filter(|slot| *slot < model.slots.len())
                .ok_or_else(|| {
                    format!(
                        "registration success contains invalid slot {}",
                        success.slot_num
                    )
                })?;
            model.slots[slot] = LegacyQuickSlotEntry {
                item_type: i32::from(success.item_type),
                item_id: i32::from(success.item_id),
                icon_path: None,
                inventory_empty: false,
                cooldown_remaining_fraction: 0.0,
            };
            model.ready_for_play = true;
            Ok(true)
        }
        QuickSlotPacket0104::RegisterFailure(failure) => Err(format!(
            "registration rejected with error {}",
            failure.error_code
        )),
        QuickSlotPacket0104::ItemUseFailure(failure) => Err(format!(
            "item use rejected with error {}",
            failure.error_code
        )),
        QuickSlotPacket0104::RegisterRequest(_) | QuickSlotPacket0104::ItemUseRequest(_) => {
            Ok(false)
        }
    }
}

pub(super) fn apply_skill_buff_frame(
    frame: &DecodedFrame,
    model: &mut SkillBuffUiModel,
    movement: &mut movement_buffs::MovementBuffs,
) -> Result<bool, String> {
    let decoded = decode_skill_buff_packet_0104(frame.packet_type, &frame.payload)
        .map_err(|error| format!("malformed protocol-0104 skill-buff packet: {error}"))?;
    let Some(packet) = decoded else {
        if frame.packet_type != packet::P_FE2CL_CHAR_TIME_BUFF_TIME_TICK {
            return Ok(false);
        }
        if frame.payload.len() < 12 {
            return Err(format!(
                "malformed protocol-0104 time-buff tick header: expected at least 12 bytes, got {}",
                frame.payload.len()
            ));
        }
        let time_buff_id = i16::from_le_bytes(
            frame.payload[8..10]
                .try_into()
                .expect("validated time-buff ID range"),
        );
        // Only infection (17) appends sSkillResult_DotDamage and therefore
        // carries iConditionBitFlag. Bounding-ball damage (19) and heal (24)
        // have shorter, different tails and must not be rejected or
        // reinterpreted as infection state.
        if time_buff_id != 17 {
            return Ok(true);
        }
        let tick = TimeBuffDotDamageTick0104::decode(&frame.payload)
            .map_err(|error| format!("malformed protocol-0104 time-buff tick: {error}"))?;
        model.apply_dot_damage_tick(tick);
        return Ok(true);
    };
    if let ffone_protocol::SkillBuffPacket0104::Pc(update) = &packet {
        movement.apply(*update);
    }
    model.apply_packet(packet);
    Ok(true)
}
