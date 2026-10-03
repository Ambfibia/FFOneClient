//! Bank system messages, network frames, UI context and outboxes.

use super::local_inventory::LocalInventoryRuntime;
use super::runtime_status::{RuntimeStatus, resolve_runtime_resurrection_item_slot};
use super::state::ClientState;
use super::world_intents::queue_service_farewell;
use bevy::{ecs::system::SystemParam, prelude::*};
use ffone_client::{
    bank_runtime::{
        BANK_SPECIAL_STATE_FLAG_0104, BankAuthoritativeMoveCommit0104, BankOpenIdentity0104,
        BankOutboundRequest0104, BankProductionEvent0104, BankProductionRuntime0104,
        BankUiCommandDisposition0104,
    },
    bank_ui::{
        BankFailClosedEquipEligibility, BankLifecyclePhase, BankModalState, BankModeProjection0104,
        BankSlotLocation0104, BankUiCommand0104, BankUiOutbox0104, BankUiState,
    },
    entity_lifecycle::NetworkNpcAppearance0104,
    game_guide_ui::GameGuideUiModel,
    gameplay_ui::{GameplayUiModel, GameplayUiOutbox},
    inventory_runtime::InventoryRuntime0104,
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
};
use ffone_net::{BankGameplayFrame0104, InventoryGameplayFrame0104};
use ffone_protocol::{
    DecodedFrame, InventoryPacket0104, ItemMoveSuccessPacket0104, PcSpecialStateSwitchRequest0104,
    packet,
};
use std::collections::BTreeMap;

pub(super) const BANK_SYSTEM_MESSAGE_ID_BASE: u64 = 0x4241_4e4b_0000_0000;
pub(super) const BANK_ACCESS_REQUIRED_MESSAGE: &str =
    "You must purchase access to this bank before you can use it.";

#[derive(Debug, Resource)]
pub(super) struct BankSystemMessageRuntime {
    pub(super) next_request_id: u64,
    pub(super) pending_access_required: BTreeMap<u64, BankOpenIdentity0104>,
    pub(super) special_state_active: bool,
}

impl Default for BankSystemMessageRuntime {
    fn default() -> Self {
        Self {
            next_request_id: BANK_SYSTEM_MESSAGE_ID_BASE,
            pending_access_required: BTreeMap::new(),
            special_state_active: false,
        }
    }
}

impl BankSystemMessageRuntime {
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn queue_access_required(
        &mut self,
        source: BankOpenIdentity0104,
        system_messages: &mut SystemMessageUiModel,
    ) -> u64 {
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.wrapping_add(1);
        if self.next_request_id < BANK_SYSTEM_MESSAGE_ID_BASE {
            self.next_request_id = BANK_SYSTEM_MESSAGE_ID_BASE;
        }
        self.pending_access_required.insert(request_id, source);
        // Exact literal from clean Retrobution `cnBank.ReceiveBankStart`
        // error-code 2. This is not a guessed TableData substitution.
        system_messages.push(SystemMessageRequest::new(
            request_id,
            BANK_ACCESS_REQUIRED_MESSAGE,
            SystemMessageButtonType::Ok,
        ));
        request_id
    }
}

pub(super) fn reset_bank_shell(
    state: &mut BankUiState,
    modal: &mut BankModalState,
    projection: &mut BankModeProjection0104,
    outbox: &mut BankUiOutbox0104,
    production: &mut BankProductionRuntime0104,
    system_runtime: &mut BankSystemMessageRuntime,
) {
    *state = BankUiState::default();
    *modal = BankModalState::default();
    *projection = BankModeProjection0104::default();
    outbox.clear();
    production.reset();
    system_runtime.reset();
}

pub(super) fn reset_bank_session(
    mut state: ResMut<BankUiState>,
    mut modal: ResMut<BankModalState>,
    mut projection: ResMut<BankModeProjection0104>,
    mut outbox: ResMut<BankUiOutbox0104>,
    mut production: ResMut<BankProductionRuntime0104>,
    mut system_runtime: ResMut<BankSystemMessageRuntime>,
) {
    reset_bank_shell(
        &mut state,
        &mut modal,
        &mut projection,
        &mut outbox,
        &mut production,
        &mut system_runtime,
    );
}

pub(super) const BANK_OPEN_SCREEN_AUDIO_PATH: &str = "audio/sfx/ui/open_screen.ogg";
pub(super) const BANK_CLOSE_SCREEN_AUDIO_PATH: &str = "audio/sfx/ui/close_screen.ogg";

pub(super) fn bank_network_command(request: BankOutboundRequest0104) -> NetworkCommand {
    match request {
        BankOutboundRequest0104::Open(request) => NetworkCommand::OpenBank(request),
        BankOutboundRequest0104::ItemMove(request) => NetworkCommand::MoveItem(request),
    }
}

pub(super) fn play_bank_screen_audio(commands: &mut Commands, asset_server: &AssetServer, path: &'static str) {
    commands.spawn((
        Name::new(format!("BankMode audio {path}")),
        ffone_client::audio_channel::GameplayAudioChannel::ui_sfx(),
        AudioPlayer::new(asset_server.load(path)),
        PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(0.7)),
    ));
}

pub(super) fn switch_bank_special_state(
    bridge: &NetworkBridge,
    player_id: Option<i32>,
    runtime: &mut BankSystemMessageRuntime,
    active: bool,
) -> Result<(), String> {
    if runtime.special_state_active == active {
        return Ok(());
    }
    let player_id =
        player_id.ok_or_else(|| "BankMode has no authoritative local PC ID".to_owned())?;
    bridge.send(NetworkCommand::SwitchSpecialState(
        PcSpecialStateSwitchRequest0104 {
            pc_id: player_id,
            special_state_flag: i8::try_from(BANK_SPECIAL_STATE_FLAG_0104)
                .expect("BankMode special-state flag must fit the beta-20100104 wire field"),
        },
    ))?;
    // InventoryManager toggles the same flag on both mode edges. This bit is
    // transport ownership only; the shard reply remains authoritative.
    runtime.special_state_active = active;
    Ok(())
}

pub(super) fn bank_inventory_after_commit(
    current: &InventoryRuntime0104,
    commit: &BankAuthoritativeMoveCommit0104,
) -> Result<InventoryRuntime0104, String> {
    if current.owner_pc_id() != commit.snapshot_after().owner_pc_id() {
        return Err(format!(
            "BankMode move belongs to PC {}, global inventory belongs to {}",
            commit.snapshot_after().owner_pc_id(),
            current.owner_pc_id()
        ));
    }
    let mut next = current.clone();
    for write in commit.inventory_writes() {
        // Apply each already-validated location-1 post-state to a clone by
        // using the generic runtime's authoritative same-slot form. No
        // request-side swap or stack arithmetic is introduced.
        next.apply_item_move_success(ItemMoveSuccessPacket0104 {
            from_location: BankSlotLocation0104::Inventory.wire_value(),
            from_slot_num: write.slot.wire_index(),
            from_slot_item: write.item,
            to_location: BankSlotLocation0104::Inventory.wire_value(),
            to_slot_num: write.slot.wire_index(),
            to_slot_item: write.item,
        })
        .map_err(|error| error.to_string())?;
    }
    if next.inventory() != commit.snapshot_after().inventory()
        || next.equipment() != commit.snapshot_after().equipment()
    {
        return Err(
            "BankMode staged global inventory differs from its authoritative post-state".to_owned(),
        );
    }
    Ok(next)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn finish_bank_mode(
    commands: &mut Commands,
    asset_server: &AssetServer,
    bridge: &NetworkBridge,
    player_id: Option<i32>,
    state: &mut BankUiState,
    modal: &mut BankModalState,
    projection: &mut BankModeProjection0104,
    outbox: &mut BankUiOutbox0104,
    system_runtime: &mut BankSystemMessageRuntime,
    status: &mut RuntimeStatus,
    message: String,
) {
    state.close_after_failure();
    *modal = BankModalState::default();
    *projection = BankModeProjection0104::default();
    outbox.clear();
    system_runtime.pending_access_required.clear();
    status.message = match switch_bank_special_state(bridge, player_id, system_runtime, false) {
        Ok(()) => message,
        Err(error) => format!("{message}; special-state exit failed: {error}"),
    };
    play_bank_screen_audio(commands, asset_server, BANK_CLOSE_SCREEN_AUDIO_PATH);
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_bank_network_frame(
    frame: &DecodedFrame,
    commands: &mut Commands,
    asset_server: &AssetServer,
    bridge: &NetworkBridge,
    content: &TutorialMissionContent,
    inventory: &mut LocalInventoryRuntime,
    state: &mut BankUiState,
    modal: &mut BankModalState,
    projection: &mut BankModeProjection0104,
    outbox: &mut BankUiOutbox0104,
    production: &mut BankProductionRuntime0104,
    system_runtime: &mut BankSystemMessageRuntime,
    system_messages: &mut SystemMessageUiModel,
    status: &mut RuntimeStatus,
) -> Result<bool, String> {
    let bank_frame = BankGameplayFrame0104::decode(frame.clone());
    let event = if !matches!(&bank_frame, BankGameplayFrame0104::Passthrough(_)) {
        let authoritative = inventory
            .snapshot()
            .ok_or_else(|| "BankMode reply arrived before authoritative PC load".to_owned())?;
        let mut staged = production.clone();
        let event = staged
            .apply_bank_frame(bank_frame, authoritative)
            .map_err(|error| format!("BankMode rejected reply: {error}"))?
            .ok_or_else(|| "BankMode known reply produced no runtime event".to_owned())?;
        Some((staged, event))
    } else if production.modal_active() {
        let inventory_frame = InventoryGameplayFrame0104::decode(frame.clone());
        let bank_move_owned = match &inventory_frame {
            InventoryGameplayFrame0104::Decoded {
                packet: InventoryPacket0104::ItemMoveSuccess(_),
                ..
            } => true,
            InventoryGameplayFrame0104::Malformed { frame, .. } => {
                frame.packet_type == packet::P_FE2CL_PC_ITEM_MOVE_SUCC
            }
            InventoryGameplayFrame0104::Passthrough(_)
            | InventoryGameplayFrame0104::Decoded {
                packet: InventoryPacket0104::EquipChange(_),
                ..
            } => false,
        };
        if !bank_move_owned {
            return Ok(false);
        }
        let mut staged = production.clone();
        let event = staged
            .apply_inventory_frame(inventory_frame)
            .map_err(|error| format!("BankMode rejected item-move reply: {error}"))?
            .ok_or_else(|| "BankMode item-move reply produced no runtime event".to_owned())?;
        Some((staged, event))
    } else {
        None
    };
    let Some((staged, event)) = event else {
        return Ok(false);
    };

    match event {
        BankProductionEvent0104::OpenAccepted {
            source,
            raw_extra_bank,
            access_tier,
            ..
        } => {
            let session = staged
                .session()
                .expect("accepted BankMode event owns a staged session");
            projection.rebuild_from_authoritative_snapshot(
                session.snapshot(),
                content,
                &BankFailClosedEquipEligibility,
            );
            *production = staged;
            state.accept_open_success();
            status.message = format!(
                "OpenFusion opened BankMode for NPC {} with raw extra-bank {} ({access_tier:?})",
                source.npc_id, raw_extra_bank
            );
        }
        BankProductionEvent0104::OpenFailed {
            source,
            error_code,
            show_access_required_popup,
            close_immediately,
        } => {
            *production = staged;
            state.reject_server_request();
            if show_access_required_popup {
                debug_assert!(!close_immediately);
                system_runtime.queue_access_required(source, system_messages);
                status.message =
                    "OpenFusion requires purchased access for this BankMode".to_owned();
            } else {
                debug_assert!(close_immediately);
                finish_bank_mode(
                    commands,
                    asset_server,
                    bridge,
                    status.player_id,
                    state,
                    modal,
                    projection,
                    outbox,
                    system_runtime,
                    status,
                    format!("OpenFusion rejected BankMode with error {error_code}"),
                );
            }
        }
        BankProductionEvent0104::ItemMoveCommitted(commit) => {
            let current = inventory
                .snapshot()
                .ok_or_else(|| "BankMode move arrived without global inventory".to_owned())?;
            let next_inventory = bank_inventory_after_commit(current, &commit)?;
            *inventory
                .snapshot_mut()
                .expect("BankMode global inventory was checked") = next_inventory;
            projection.rebuild_from_authoritative_snapshot(
                commit.snapshot_after(),
                content,
                &BankFailClosedEquipEligibility,
            );
            *production = staged;
            state.accept_item_move_success();
            status.resurrection_item_slot = inventory
                .snapshot()
                .and_then(|inventory| resolve_runtime_resurrection_item_slot(inventory, content));
            status.message =
                "OpenFusion committed BankMode bank/inventory post-state atomically".to_owned();
        }
        BankProductionEvent0104::ClosedByServerSuccess { source, .. } => {
            *production = staged;
            finish_bank_mode(
                commands,
                asset_server,
                bridge,
                status.player_id,
                state,
                modal,
                projection,
                outbox,
                system_runtime,
                status,
                format!("OpenFusion closed BankMode for NPC {}", source.npc_id),
            );
        }
        BankProductionEvent0104::ClosedByServerFailure { source, error_code } => {
            *production = staged;
            finish_bank_mode(
                commands,
                asset_server,
                bridge,
                status.player_id,
                state,
                modal,
                projection,
                outbox,
                system_runtime,
                status,
                format!(
                    "OpenFusion forced BankMode exit for NPC {} with error {error_code}",
                    source.npc_id
                ),
            );
        }
        BankProductionEvent0104::ClosedLocally { .. } => {
            return Err("BankMode network router received a local-only exit event".to_owned());
        }
    }
    Ok(true)
}

#[derive(SystemParam)]
pub(super) struct BankUiContextInputs<'w> {
    pub(super) client_state: Res<'w, State<ClientState>>,
    pub(super) bridge: Res<'w, NetworkBridge>,
    pub(super) inventory: Res<'w, LocalInventoryRuntime>,
    pub(super) content: Res<'w, TutorialMissionContent>,
    pub(super) mission_ui: Res<'w, MissionUiModel>,
    pub(super) system_messages: Res<'w, SystemMessageUiModel>,
    pub(super) resurrect_ui: Res<'w, ResurrectUiModel>,
    pub(super) user_equip_ui: Res<'w, UserEquipUiState>,
    pub(super) help: Res<'w, GameGuideUiModel>,
}

#[derive(SystemParam)]
pub(super) struct BankUiOutboxInputs<'w, 's> {
    pub(super) state_owner: Res<'w, State<ClientState>>,
    pub(super) bridge: Res<'w, NetworkBridge>,
    pub(super) content: Res<'w, TutorialMissionContent>,
    pub(super) npc_appearances: Query<'w, 's, &'static NetworkNpcAppearance0104>,
    pub(super) inventory: Res<'w, LocalInventoryRuntime>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn sync_bank_ui_context(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    inputs: BankUiContextInputs,
    mut status: ResMut<RuntimeStatus>,
    mut state: ResMut<BankUiState>,
    mut modal: ResMut<BankModalState>,
    mut projection: ResMut<BankModeProjection0104>,
    mut outbox: ResMut<BankUiOutbox0104>,
    mut production: ResMut<BankProductionRuntime0104>,
    mut system_runtime: ResMut<BankSystemMessageRuntime>,
) {
    let BankUiContextInputs {
        client_state,
        bridge,
        inventory,
        content,
        mission_ui,
        system_messages,
        resurrect_ui,
        user_equip_ui,
        help,
    } = inputs;

    let invalid_owner = *client_state.get() != ClientState::World
        || inventory.snapshot().is_none()
        || resurrect_ui.visible
        || status.hp.is_some_and(|hp| hp <= 0)
        || user_equip_ui.is_active();
    if invalid_owner {
        if production.modal_active()
            || state.phase != BankLifecyclePhase::Hidden
            || !outbox.is_empty()
        {
            if *client_state.get() == ClientState::World {
                let _ = switch_bank_special_state(
                    &bridge,
                    status.player_id,
                    &mut system_runtime,
                    false,
                );
                play_bank_screen_audio(&mut commands, &asset_server, BANK_CLOSE_SCREEN_AUDIO_PATH);
            }
            reset_bank_shell(
                &mut state,
                &mut modal,
                &mut projection,
                &mut outbox,
                &mut production,
                &mut system_runtime,
            );
        }
        return;
    }

    modal.help = help.modal_active();
    modal.inventory_popup = false;
    modal.generic_popup = mission_ui.system_popup_active();
    modal.system_popup = system_messages.is_popup();

    if production.session().is_some()
        && !production.request_pending()
        && (inventory.is_changed() || content.is_changed())
    {
        let authoritative = inventory
            .snapshot()
            .expect("BankMode authoritative inventory was checked");
        if let Err(error) = production.refresh_inventory_authority(authoritative) {
            status.message = format!("BankMode inventory refresh rejected: {error}");
        }
    }
    if let Some(session) = production.session() {
        projection.rebuild_from_authoritative_snapshot(
            session.snapshot(),
            &*content,
            &BankFailClosedEquipEligibility,
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn consume_bank_ui_outbox(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    inputs: BankUiOutboxInputs,
    mut gameplay_ui: ResMut<GameplayUiModel>,
    mut mission_ui: ResMut<MissionUiModel>,
    mut nanocom_messages: ResMut<NanocomMessageUiModel>,
    mut state: ResMut<BankUiState>,
    mut modal: ResMut<BankModalState>,
    mut projection: ResMut<BankModeProjection0104>,
    mut outbox: ResMut<BankUiOutbox0104>,
    mut production: ResMut<BankProductionRuntime0104>,
    mut system_runtime: ResMut<BankSystemMessageRuntime>,
    mut status: ResMut<RuntimeStatus>,
) {
    let BankUiOutboxInputs {
        state_owner,
        bridge,
        content,
        npc_appearances,
        inventory,
    } = inputs;

    while let Some(command) = outbox.pop_front() {
        let Some(authoritative) = inventory.snapshot() else {
            state.close_after_failure();
            production.reset();
            status.message = "BankMode blocked before authoritative inventory load".to_owned();
            continue;
        };
        if let BankUiCommand0104::Open(request) = command {
            let valid_player =
                *state_owner.get() == ClientState::World && status.player_id == Some(request.pc_id);
            let mut matches = npc_appearances
                .iter()
                .filter(|appearance| appearance.0.npc_id == request.npc_id);
            let first = matches.next();
            let unique = matches.next().is_none();
            let valid_npc = unique
                && first
                    .and_then(|appearance| content.gameplay_npc(appearance.0.npc_type))
                    .is_some_and(|definition| matches!(definition.service_category, 12 | 50));
            if !valid_player || !valid_npc {
                state.close_after_failure();
                production.reset();
                status.message = format!(
                    "BankMode start rejected unowned PC/NPC {}/{}",
                    request.pc_id, request.npc_id
                );
                continue;
            }
        }

        let disposition = match production.apply_ui_command(command, authoritative) {
            Ok(disposition) => disposition,
            Err(error) => {
                if matches!(command, BankUiCommand0104::Open(_)) {
                    state.close_after_failure();
                } else {
                    state.reject_server_request();
                }
                status.message = format!("BankMode command rejected: {error}");
                continue;
            }
        };
        match disposition {
            BankUiCommandDisposition0104::Send(request) => {
                let operation = request.operation();
                if let Err(error) = bridge.send(bank_network_command(request)) {
                    let cancelled = production.cancel_pending_transport();
                    if matches!(request, BankOutboundRequest0104::Open(_)) {
                        let _ = production.close_locally();
                        state.close_after_failure();
                    } else {
                        state.reject_server_request();
                    }
                    status.message = format!(
                        "BankMode {operation:?} transport failed after cancelling {cancelled:?}: {error}"
                    );
                    continue;
                }
                match request {
                    BankOutboundRequest0104::Open(request) => {
                        mission_ui.clear_npc_interaction_locally();
                        let mut ignored = GameplayUiOutbox::default();
                        mission_ui.close_nanocom_menu(&mut ignored);
                        nanocom_messages.set_expanded(false);
                        gameplay_ui.chat.input_enabled = false;
                        gameplay_ui.chat.active = false;
                        gameplay_ui.chat.input.clear();
                        let special = switch_bank_special_state(
                            &bridge,
                            status.player_id,
                            &mut system_runtime,
                            true,
                        );
                        play_bank_screen_audio(
                            &mut commands,
                            &asset_server,
                            BANK_OPEN_SCREEN_AUDIO_PATH,
                        );
                        status.message = match special {
                            Ok(()) => {
                                format!("Requesting BankMode from runtime NPC {}", request.npc_id)
                            }
                            Err(error) => format!(
                                "BankMode open sent, but special-state entry failed: {error}"
                            ),
                        };
                    }
                    BankOutboundRequest0104::ItemMove(_) => {
                        status.message = "BankMode item-move request sent to OpenFusion".to_owned();
                    }
                }
            }
            BankUiCommandDisposition0104::Local(BankProductionEvent0104::ClosedLocally {
                source,
            }) => {
                queue_service_farewell(&mut commands, source.npc_id);
                keyboard.clear_just_pressed(KeyCode::Escape);
                finish_bank_mode(
                    &mut commands,
                    &asset_server,
                    &bridge,
                    status.player_id,
                    &mut state,
                    &mut modal,
                    &mut projection,
                    &mut outbox,
                    &mut system_runtime,
                    &mut status,
                    format!(
                        "BankMode closed locally for NPC {}; no bank-close packet sent",
                        source.npc_id
                    ),
                );
            }
            BankUiCommandDisposition0104::Local(event) => {
                status.message = format!("BankMode rejected unexpected local event {event:?}");
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn consume_bank_system_message_outbox(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    bridge: Res<NetworkBridge>,
    mut system_outbox: ResMut<SystemMessageUiOutbox>,
    mut state: ResMut<BankUiState>,
    mut modal: ResMut<BankModalState>,
    mut projection: ResMut<BankModeProjection0104>,
    mut outbox: ResMut<BankUiOutbox0104>,
    mut production: ResMut<BankProductionRuntime0104>,
    mut system_runtime: ResMut<BankSystemMessageRuntime>,
    mut status: ResMut<RuntimeStatus>,
) {
    if system_runtime.pending_access_required.is_empty() {
        return;
    }
    let mut unrelated = Vec::new();
    for action in system_outbox.drain().collect::<Vec<_>>() {
        let SystemMessageUiAction::Chosen {
            request_id,
            button_type,
            choice,
        } = action;
        let Some(source) = system_runtime.pending_access_required.remove(&request_id) else {
            unrelated.push(action);
            continue;
        };
        if button_type != SystemMessageButtonType::Ok
            || choice != SystemMessageChoice::Primary
            || production.opening() != Some(source)
        {
            status.message =
                "BankMode access-required callback rejected mismatched owner/button".to_owned();
            continue;
        }
        match production.close_locally() {
            Ok(BankProductionEvent0104::ClosedLocally { .. }) => finish_bank_mode(
                &mut commands,
                &asset_server,
                &bridge,
                status.player_id,
                &mut state,
                &mut modal,
                &mut projection,
                &mut outbox,
                &mut system_runtime,
                &mut status,
                "BankMode access-required popup dismissed; mode closed locally".to_owned(),
            ),
            Ok(event) => {
                status.message = format!("BankMode popup produced unexpected event {event:?}");
            }
            Err(error) => {
                status.message = format!("BankMode popup callback rejected: {error}");
            }
        }
    }
    for action in unrelated {
        system_outbox.push(action);
    }
}
