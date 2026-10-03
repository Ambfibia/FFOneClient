//! Vendor system messages, requests, network frames, UI context and outboxes.

use super::local_inventory::LocalInventoryRuntime;
use super::runtime_status::{RuntimeStatus, resolve_runtime_resurrection_item_slot};
use super::shared_redeem;
use super::state::ClientState;
use super::world_intents::queue_service_farewell;
use bevy::{ecs::system::SystemParam, prelude::*};
use ffone_client::{
    entity_lifecycle::NetworkNpcAppearance0104,
    game_guide_ui::GameGuideUiModel,
    gameplay_audio::GameplayAudioRuntime,
    gameplay_ui::{GameplayUiModel, GameplayUiOutbox},
    inventory_runtime::InventoryRuntime0104,
    localization::{Language, Localization, localized_tabledata_npc_name},
    mission_ui::MissionUiModel,
    nanocom_message_ui::NanocomMessageUiModel,
    network::{NetworkBridge, NetworkCommand},
    resurrect_ui::ResurrectUiModel,
    system_message_ui::{
        SystemMessageButtonType, SystemMessageChoice, SystemMessageRequest, SystemMessageUiAction,
        SystemMessageUiModel, SystemMessageUiOutbox,
    },
    tutorial_mission_content::TutorialMissionContent,
    user_equip_ui::UserEquipUiState,
    vendor_runtime::{
        ActiveVendorSourceNpc0104, VendorOperation0104, VendorOutboundRequest0104,
        VendorProductionEvent0104, VendorProductionRuntime0104,
    },
    vendor_ui::{
        VendorActionOutcome0104, VendorCloseGate0104, VendorConfirmation0104,
        VendorFailClosedEquipEligibility0104, VendorLifecyclePhase, VendorModalState,
        VendorModeProjection0104, VendorPresentationIcon0104, VendorSystemMessage0104,
        VendorSystemMessageCallback0104, VendorUiAudioCue0104, VendorUiAudioOutbox0104,
        VendorUiCommand0104, VendorUiOutbox0104, VendorUiState, vendor_server_failure_outcome,
    },
};
use ffone_net::VendorGameplayFrame0104;
use ffone_protocol::DecodedFrame;
use std::collections::BTreeMap;

pub(super) const VENDOR_SYSTEM_MESSAGE_ID_BASE: u64 = 0x5645_4e44_0000_0000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum PendingVendorSystemAction {
    Message {
        callback: Option<VendorSystemMessageCallback0104>,
        expected_button_type: SystemMessageButtonType,
    },
    Confirmation {
        confirmation: VendorConfirmation0104,
        expected_button_type: SystemMessageButtonType,
    },
}

#[derive(Debug, Resource)]
pub(super) struct VendorSystemMessageRuntime {
    pub(super) next_request_id: u64,
    pub(super) pending: BTreeMap<u64, PendingVendorSystemAction>,
}

impl Default for VendorSystemMessageRuntime {
    fn default() -> Self {
        Self {
            next_request_id: VENDOR_SYSTEM_MESSAGE_ID_BASE,
            pending: BTreeMap::new(),
        }
    }
}

impl VendorSystemMessageRuntime {
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn next_request_id(&mut self) -> u64 {
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.wrapping_add(1);
        if self.next_request_id < VENDOR_SYSTEM_MESSAGE_ID_BASE {
            self.next_request_id = VENDOR_SYSTEM_MESSAGE_ID_BASE;
        }
        request_id
    }
}

pub(super) fn queue_vendor_system_message(
    content: &TutorialMissionContent,
    message: VendorSystemMessage0104,
    system_runtime: &mut VendorSystemMessageRuntime,
    system_messages: &mut SystemMessageUiModel,
) -> Result<u64, String> {
    let request_id = system_runtime.next_request_id();
    let definition = content
        .system_message_definition(message.message_id.id())
        .ok_or_else(|| {
            format!(
                "VendorMode SystemMessage {} has no source TableData row",
                message.message_id.id()
            )
        })?;
    let request = SystemMessageRequest::new(
        request_id,
        definition.exact_text.clone(),
        definition.runtime_button_type,
    );
    system_runtime.pending.insert(
        request_id,
        PendingVendorSystemAction::Message {
            callback: message.callback,
            expected_button_type: definition.runtime_button_type,
        },
    );
    system_messages.push(request);
    Ok(request_id)
}

pub(super) fn queue_vendor_confirmation(
    content: &TutorialMissionContent,
    confirmation: VendorConfirmation0104,
    system_runtime: &mut VendorSystemMessageRuntime,
    system_messages: &mut SystemMessageUiModel,
) -> Result<u64, String> {
    let request_id = system_runtime.next_request_id();
    let definition = content
        .system_message_definition(confirmation.message_id.id())
        .ok_or_else(|| {
            format!(
                "VendorMode confirmation {} has no source TableData row",
                confirmation.message_id.id()
            )
        })?;
    let mut request = SystemMessageRequest::new(
        request_id,
        definition.exact_text.clone(),
        definition.runtime_button_type,
    );
    if let VendorPresentationIcon0104::Resolved(icon) = &confirmation.icon {
        request = request.with_icon_path(icon.runtime_path());
    }
    system_runtime.pending.insert(
        request_id,
        PendingVendorSystemAction::Confirmation {
            confirmation,
            expected_button_type: definition.runtime_button_type,
        },
    );
    system_messages.push(request);
    Ok(request_id)
}

pub(super) fn vendor_network_command(request: VendorOutboundRequest0104) -> NetworkCommand {
    match request {
        VendorOutboundRequest0104::Start(request) => NetworkCommand::VendorStart(request),
        VendorOutboundRequest0104::TableUpdate(request) => {
            NetworkCommand::VendorTableUpdate(request)
        }
        VendorOutboundRequest0104::Buy(request) => NetworkCommand::VendorBuy(request),
        VendorOutboundRequest0104::BatteryBuy(request) => NetworkCommand::VendorBatteryBuy(request),
        VendorOutboundRequest0104::Sell(request) => NetworkCommand::VendorSell(request),
        VendorOutboundRequest0104::Restore(request) => NetworkCommand::VendorRestore(request),
        VendorOutboundRequest0104::Delete(request) => NetworkCommand::DeleteInventoryItem(request),
        VendorOutboundRequest0104::Disassemble(request) => {
            NetworkCommand::DisassembleInventoryItem(request)
        }
    }
}

pub(super) fn reset_vendor_shell(
    state: &mut VendorUiState,
    modal: &mut VendorModalState,
    projection: &mut VendorModeProjection0104,
    outbox: &mut VendorUiOutbox0104,
    production: &mut VendorProductionRuntime0104,
    system_runtime: &mut VendorSystemMessageRuntime,
    disconnect: bool,
) {
    *state = VendorUiState::default();
    *modal = VendorModalState::default();
    *projection = VendorModeProjection0104::default();
    outbox.clear();
    if disconnect {
        production.reset();
    } else {
        production.close_mode();
    }
    system_runtime.reset();
}

pub(super) fn reset_vendor_session(
    mut state: ResMut<VendorUiState>,
    mut modal: ResMut<VendorModalState>,
    mut projection: ResMut<VendorModeProjection0104>,
    mut outbox: ResMut<VendorUiOutbox0104>,
    mut production: ResMut<VendorProductionRuntime0104>,
    mut system_runtime: ResMut<VendorSystemMessageRuntime>,
) {
    reset_vendor_shell(
        &mut state,
        &mut modal,
        &mut projection,
        &mut outbox,
        &mut production,
        &mut system_runtime,
        true,
    );
}

pub(super) fn send_vendor_request(
    bridge: &NetworkBridge,
    production: &mut VendorProductionRuntime0104,
    state: &mut VendorUiState,
    request: VendorOutboundRequest0104,
) -> Result<(), String> {
    let operation = request.operation();
    if let Err(error) = bridge.send(vendor_network_command(request)) {
        let cancelled = production.cancel_pending();
        if matches!(
            operation,
            VendorOperation0104::Start | VendorOperation0104::TableUpdate
        ) {
            state.close_after_failure();
            production.close_mode();
        } else {
            state.accept_request_reply();
        }
        return Err(format!(
            "VendorMode {operation:?} transport failed after cancelling {cancelled:?}: {error}"
        ));
    }
    Ok(())
}

pub(super) fn begin_vendor_intent_and_send(
    intent: ffone_client::vendor_ui::VendorIntent0104,
    inventory: &InventoryRuntime0104,
    bridge: &NetworkBridge,
    production: &mut VendorProductionRuntime0104,
    state: &mut VendorUiState,
) -> Result<VendorOperation0104, String> {
    let request = production
        .begin_intent(intent, inventory)
        .map_err(|error| format!("VendorMode intent rejected: {error}"))?;
    let operation = request.operation();
    state.send_pending = true;
    send_vendor_request(bridge, production, state, request)?;
    Ok(operation)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_vendor_frame(
    frame: &DecodedFrame,
    bridge: &NetworkBridge,
    content: &TutorialMissionContent,
    inventory: &mut LocalInventoryRuntime,
    state: &mut VendorUiState,
    production: &mut VendorProductionRuntime0104,
    system_runtime: &mut VendorSystemMessageRuntime,
    system_messages: &mut SystemMessageUiModel,
    runtime: &mut RuntimeStatus,
    audio: &mut GameplayAudioRuntime,
) -> Result<bool, String> {
    let packet = match VendorGameplayFrame0104::decode(frame.clone()) {
        VendorGameplayFrame0104::Passthrough(_) => return Ok(false),
        VendorGameplayFrame0104::Malformed { error, .. } => {
            return Err(format!(
                "VendorMode rejected malformed known reply without releasing its pending request: {error:?}"
            ));
        }
        VendorGameplayFrame0104::Decoded { packet, .. } => packet,
    };
    let inventory_snapshot = inventory
        .snapshot_mut()
        .ok_or_else(|| "VendorMode reply arrived without authoritative inventory".to_owned())?;
    let event = production
        .apply_packet(packet, inventory_snapshot)
        .map_err(|error| format!("VendorMode rejected uncorrelated reply: {error}"))?;

    if matches!(&event,
        VendorProductionEvent0104::BuyAccepted { .. }
        | VendorProductionEvent0104::BatteryAccepted { .. }
        | VendorProductionEvent0104::SellAccepted { .. }
        | VendorProductionEvent0104::RestoreAccepted { .. }) {
        audio.queue_legacy_money_transfer_sound();
    }

    match event {
        VendorProductionEvent0104::StartAccepted { table_request } => {
            state.accept_start_success();
            send_vendor_request(
                bridge,
                production,
                state,
                VendorOutboundRequest0104::TableUpdate(table_request),
            )?;
            runtime.message =
                "OpenFusion accepted VendorMode start; exact table request sent".to_owned();
        }
        VendorProductionEvent0104::TableAccepted => {
            state.accept_authoritative_table();
            runtime.message = format!(
                "OpenFusion supplied {} authoritative VendorMode item(s)",
                production.catalog_entries().len()
            );
        }
        VendorProductionEvent0104::BuyAccepted { candy } => {
            state.accept_request_reply();
            runtime.candy = candy;
            runtime.message =
                "OpenFusion accepted vendor purchase; inventory and Taros synchronized".to_owned();
        }
        VendorProductionEvent0104::BatteryAccepted {
            candy,
            weapon_battery,
            nano_battery,
        } => {
            state.accept_request_reply();
            runtime.candy = candy;
            runtime.weapon_battery = weapon_battery;
            runtime.nano_battery = nano_battery;
            runtime.message =
                "OpenFusion accepted battery purchase; Taros and batteries synchronized".to_owned();
        }
        VendorProductionEvent0104::SellAccepted { candy, sold_item } => {
            state.accept_request_reply();
            runtime.candy = candy;
            runtime.message = format!(
                "OpenFusion accepted vendor sale of type {} item {}; inventory, buyback and Taros synchronized",
                sold_item.item_type, sold_item.item_id
            );
        }
        VendorProductionEvent0104::RestoreAccepted { candy } => {
            state.accept_request_reply();
            runtime.candy = candy;
            runtime.message =
                "OpenFusion accepted vendor buyback; inventory, FIFO and Taros synchronized"
                    .to_owned();
        }
        VendorProductionEvent0104::DeleteAccepted => {
            state.accept_request_reply();
            runtime.message =
                "OpenFusion accepted item deletion; authoritative inventory synchronized"
                    .to_owned();
        }
        VendorProductionEvent0104::DisassembleAccepted => {
            state.accept_request_reply();
            runtime.message =
                "OpenFusion accepted item disassembly; authoritative inventory synchronized"
                    .to_owned();
        }
        VendorProductionEvent0104::Failed {
            operation,
            error_code,
            ui_failure,
        } => {
            state.accept_request_reply();
            if let Some(ui_failure) = ui_failure {
                let VendorActionOutcome0104::SystemMessage(message) =
                    vendor_server_failure_outcome(ui_failure)
                else {
                    return Err(
                        "VendorMode failure resolver produced a non-message outcome".to_owned()
                    );
                };
                queue_vendor_system_message(content, message, system_runtime, system_messages)?;
            }
            runtime.message =
                format!("OpenFusion rejected VendorMode {operation:?} with error {error_code}");
        }
    }

    runtime.resurrection_item_slot = inventory
        .snapshot()
        .and_then(|inventory| resolve_runtime_resurrection_item_slot(inventory, content));
    Ok(true)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn sync_vendor_ui_context(
    client_state: Res<State<ClientState>>,
    inventory: Res<LocalInventoryRuntime>,
    content: Res<TutorialMissionContent>,
    localization: Res<Localization>,
    language: Res<Language>,
    system_messages: Res<SystemMessageUiModel>,
    resurrect_ui: Res<ResurrectUiModel>,
    user_equip_ui: Res<UserEquipUiState>,
    runtime: Res<RuntimeStatus>,
    mut state: ResMut<VendorUiState>,
    mut modal: ResMut<VendorModalState>,
    mut close_gate: ResMut<VendorCloseGate0104>,
    mut projection: ResMut<VendorModeProjection0104>,
    mut outbox: ResMut<VendorUiOutbox0104>,
    mut production: ResMut<VendorProductionRuntime0104>,
    mut system_runtime: ResMut<VendorSystemMessageRuntime>,
) {
    if *client_state.get() != ClientState::World
        || inventory.snapshot().is_none()
        || resurrect_ui.visible
        || runtime.hp.is_some_and(|hp| hp <= 0)
        || user_equip_ui.is_active()
    {
        if state.phase != VendorLifecyclePhase::Hidden
            || production.modal_active()
            || !outbox.is_empty()
        {
            reset_vendor_shell(
                &mut state,
                &mut modal,
                &mut projection,
                &mut outbox,
                &mut production,
                &mut system_runtime,
                false,
            );
        }
        return;
    }

    modal.help = false;
    modal.inventory_popup = false;
    modal.generic_popup = false;
    modal.system_popup = system_messages.is_popup();
    *close_gate = VendorCloseGate0104 {
        mode_accepts_escape: true,
        // Native VendorMode has no independent InventoryManager popup owner
        // yet. With that owner absent, `(11, 13)` has the clean idle result
        // zero; any future popup must set `inventory_popup` and close this gate.
        target_action_idle: !modal.inventory_popup,
    };

    let next_projection = production
        .active_source_npc()
        .zip(production.session())
        .zip(runtime.player_id)
        .zip(inventory.snapshot())
        .and_then(|(((source, session), owner_pc_id), inventory)| {
            let definition = content.gameplay_npc(source.table_npc_id)?;
            let recent_entries = production.recent_entries().copied().collect::<Vec<_>>();
            VendorModeProjection0104::from_authoritative(
                owner_pc_id,
                session,
                runtime.candy,
                runtime.weapon_battery,
                runtime.nano_battery,
                localization.text(
                    &language,
                    &localized_tabledata_npc_name(definition.npc_type, &definition.name),
                ),
                content
                    .gameplay_npc_service(source.table_npc_id)
                    .unwrap_or_default(),
                production.catalog_entries(),
                &recent_entries,
                inventory,
                &*content,
                &VendorFailClosedEquipEligibility0104,
            )
            .ok()
        })
        .unwrap_or_default();
    if *projection != next_projection {
        *projection = next_projection;
    }
}

#[derive(SystemParam)]
pub(super) struct VendorRedeemCodeOwners0104<'w> {
    pub(super) modal: Res<'w, VendorModalState>,
    pub(super) item_popup: ResMut<'w, ffone_client::vendor_ui::VendorItemPopupState>,
    pub(super) projection: Res<'w, VendorModeProjection0104>,
    pub(super) help: ResMut<'w, GameGuideUiModel>,
    pub(super) audio: Option<ResMut<'w, GameplayAudioRuntime>>,
    pub(super) input: ResMut<'w, ffone_client::shared_input_ui::SharedInputDialog>,
    pub(super) redeem: ResMut<'w, ffone_client::shared_input_ui::SharedRedeemCode>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn consume_vendor_ui_outbox(
    mut commands: Commands,
    bridge: Res<NetworkBridge>,
    content: Res<TutorialMissionContent>,
    npc_appearances: Query<&NetworkNpcAppearance0104>,
    inventory: Res<LocalInventoryRuntime>,
    mut gameplay_ui: ResMut<GameplayUiModel>,
    mut mission_ui: ResMut<MissionUiModel>,
    mut nanocom_messages: ResMut<NanocomMessageUiModel>,
    mut user_equip_ui: ResMut<UserEquipUiState>,
    mut state: ResMut<VendorUiState>,
    mut outbox: ResMut<VendorUiOutbox0104>,
    mut production: ResMut<VendorProductionRuntime0104>,
    mut system_runtime: ResMut<VendorSystemMessageRuntime>,
    mut system_messages: ResMut<SystemMessageUiModel>,
    mut redeem_owners: VendorRedeemCodeOwners0104,
    mut runtime: ResMut<RuntimeStatus>,
) {
    while let Some(command) = outbox.pop_front() {
        match command {
            VendorUiCommand0104::StartSession {
                requested_npc_id,
                table_vendor_id,
            } => {
                let mut matches = npc_appearances
                    .iter()
                    .filter(|appearance| appearance.0.npc_id == requested_npc_id);
                let Some(appearance) = matches.next() else {
                    state.close_after_failure();
                    production.close_mode();
                    runtime.message =
                        format!("VendorMode start rejected missing runtime NPC {requested_npc_id}");
                    continue;
                };
                if matches.next().is_some()
                    || appearance.0.npc_type != table_vendor_id
                    || !content.gameplay_npc_is_vendor(table_vendor_id)
                {
                    state.close_after_failure();
                    production.close_mode();
                    runtime.message = format!(
                        "VendorMode start rejected runtime/table identity {requested_npc_id}/{table_vendor_id}"
                    );
                    continue;
                }
                let Some(definition) = content.gameplay_npc(table_vendor_id) else {
                    state.close_after_failure();
                    production.close_mode();
                    runtime.message =
                        format!("VendorMode start rejected missing NPC table {table_vendor_id}");
                    continue;
                };
                let request = match production.begin_start(ActiveVendorSourceNpc0104 {
                    runtime_npc_id: requested_npc_id,
                    table_npc_id: table_vendor_id,
                    ai_type: definition.ai_type,
                }) {
                    Ok(request) => request,
                    Err(error) => {
                        state.close_after_failure();
                        production.close_mode();
                        runtime.message = format!("VendorMode start rejected: {error}");
                        continue;
                    }
                };
                match send_vendor_request(&bridge, &mut production, &mut state, request) {
                    Ok(()) => {
                        runtime.message = format!(
                            "Requesting VendorMode from runtime NPC {requested_npc_id} (table {table_vendor_id})"
                        );
                    }
                    Err(error) => runtime.message = error,
                }
            }
            VendorUiCommand0104::Intent(intent) => {
                let Some(inventory) = inventory.snapshot() else {
                    state.accept_request_reply();
                    runtime.message =
                        "VendorMode intent blocked before authoritative inventory load".to_owned();
                    continue;
                };
                match begin_vendor_intent_and_send(
                    intent,
                    inventory,
                    &bridge,
                    &mut production,
                    &mut state,
                ) {
                    Ok(operation) => {
                        runtime.message =
                            format!("VendorMode {operation:?} request sent to OpenFusion");
                    }
                    Err(error) => {
                        state.accept_request_reply();
                        runtime.message = error;
                    }
                }
            }
            VendorUiCommand0104::ShowSystemMessage(message) => {
                if let Err(error) = queue_vendor_system_message(
                    &content,
                    message,
                    &mut system_runtime,
                    &mut system_messages,
                ) {
                    runtime.message = error;
                }
            }
            VendorUiCommand0104::OpenConfirmation(confirmation) => {
                if let Err(error) = queue_vendor_confirmation(
                    &content,
                    confirmation,
                    &mut system_runtime,
                    &mut system_messages,
                ) {
                    runtime.message = error;
                }
            }
            VendorUiCommand0104::OpenItemActionPopup(popup) => {
                redeem_owners
                    .item_popup
                    .open(popup, &redeem_owners.projection);
            }
            VendorUiCommand0104::OpenHelp { event_id } => {
                if usize::try_from(event_id).is_ok_and(|id| redeem_owners.help.open_first_use(id))
                    && let Some(audio) = redeem_owners.audio.as_mut()
                {
                    audio.queue_gameplay_ui_sound("Open_Screen");
                }
            }
            VendorUiCommand0104::OpenRedeemCode => {
                shared_redeem::open_vendor_redeem(
                    &state,
                    *redeem_owners.modal,
                    &redeem_owners.projection,
                    &mut redeem_owners.redeem,
                    &mut redeem_owners.input,
                );
            }
            VendorUiCommand0104::GoToMyStuff => {
                production.close_mode();
                let mut ignored = GameplayUiOutbox::default();
                mission_ui.close_nanocom_menu(&mut ignored);
                nanocom_messages.set_expanded(false);
                gameplay_ui.chat.active = false;
                gameplay_ui.chat.input.clear();
                user_equip_ui.open_item_mode();
                runtime.message =
                    "VendorMode GO TO MY STUFF returned to MainGame Item mode".to_owned();
            }
            VendorUiCommand0104::ExitMode => {
                queue_service_farewell(&mut commands, redeem_owners.projection.session.accepted_npc_id);
                if let Some(audio) = redeem_owners.audio.as_mut() {
                    audio.queue_gameplay_ui_sound("Close_Screen");
                }
                production.close_mode();
                runtime.message = "VendorMode closed without a protocol packet".to_owned();
            }
        }
    }
}

pub(super) fn consume_vendor_ui_audio_outbox(
    mut outbox: ResMut<VendorUiAudioOutbox0104>,
    mut audio: ResMut<GameplayAudioRuntime>,
) {
    while let Some(cue) = outbox.pop_front() {
        match cue {
            VendorUiAudioCue0104::WeaponEquipped => {
                audio.queue_gameplay_ui_sound("Weapon_Equipped")
            }
            VendorUiAudioCue0104::ClothingEquipped => {
                audio.queue_gameplay_ui_sound("Clothing_Equipped")
            }
            VendorUiAudioCue0104::ButtonSound => audio.queue_legacy_button_sound(),
            VendorUiAudioCue0104::TabClick01 => audio.queue_gameplay_ui_sound("Tab_Click01"),
            VendorUiAudioCue0104::Purchase { random_pitch } => audio.queue_legacy_purchase_sound(random_pitch),
            VendorUiAudioCue0104::Money => audio.queue_legacy_money_sound(),
        }
    }
}

pub(super) fn consume_vendor_system_message_outbox(
    mut system_outbox: ResMut<SystemMessageUiOutbox>,
    inventory: Res<LocalInventoryRuntime>,
    bridge: Res<NetworkBridge>,
    mut state: ResMut<VendorUiState>,
    mut production: ResMut<VendorProductionRuntime0104>,
    mut system_runtime: ResMut<VendorSystemMessageRuntime>,
    mut runtime: ResMut<RuntimeStatus>,
) {
    if system_runtime.pending.is_empty() {
        return;
    }
    let mut unrelated = Vec::new();
    for action in system_outbox.drain().collect::<Vec<_>>() {
        let SystemMessageUiAction::Chosen {
            request_id,
            button_type,
            choice,
        } = action;
        let Some(pending) = system_runtime.pending.remove(&request_id) else {
            unrelated.push(action);
            continue;
        };
        let (message_id, expected_button_type) = match &pending {
            PendingVendorSystemAction::Message {
                expected_button_type,
                ..
            } => (None, *expected_button_type),
            PendingVendorSystemAction::Confirmation {
                confirmation,
                expected_button_type,
            } => (Some(confirmation.message_id), *expected_button_type),
        };
        if expected_button_type != button_type {
            runtime.message = match message_id {
                Some(message_id) => format!(
                    "VendorMode confirmation {} rejected mismatched button type {button_type:?}",
                    message_id.id()
                ),
                None => format!(
                    "VendorMode SystemMessage rejected mismatched button type {button_type:?}"
                ),
            };
            continue;
        }
        match pending {
            PendingVendorSystemAction::Message { callback, .. } => {
                if choice == SystemMessageChoice::Primary
                    && callback == Some(VendorSystemMessageCallback0104::Exit)
                {
                    state.close_after_failure();
                    production.close_mode();
                    runtime.message =
                        "VendorMode closed by its correlated SystemMessage callback".to_owned();
                }
            }
            PendingVendorSystemAction::Confirmation { confirmation, .. } => {
                if choice != SystemMessageChoice::Primary {
                    runtime.message = format!(
                        "VendorMode confirmation {} cancelled",
                        confirmation.message_id.id()
                    );
                    continue;
                }
                let Some(inventory) = inventory.snapshot() else {
                    state.accept_request_reply();
                    runtime.message =
                        "VendorMode confirmation blocked without authoritative inventory"
                            .to_owned();
                    continue;
                };
                match begin_vendor_intent_and_send(
                    confirmation.confirm(),
                    inventory,
                    &bridge,
                    &mut production,
                    &mut state,
                ) {
                    Ok(operation) => {
                        runtime.message = format!(
                            "VendorMode confirmed {operation:?} request sent to OpenFusion"
                        );
                    }
                    Err(error) => {
                        state.accept_request_reply();
                        runtime.message = error;
                    }
                }
            }
        }
    }
    for action in unrelated {
        system_outbox.push(action);
    }
}
