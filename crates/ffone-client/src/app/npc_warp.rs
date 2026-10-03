//! Normal NPC warp runtime, warp UI entries, fade overlay and authoritative teleports.

use super::guide::PAST_WARP_UPSELL_LEVEL;
use super::loading_screen::{GameplayLoadingState, ResourceLoadingScope};
use super::local_inventory::LocalInventoryRuntime;
use super::runtime_status::RuntimeStatus;
use super::warp_presentation::*;
use super::{LocalPlayer, WorldSliceEntity};
use bevy::prelude::*;
use ffone_client::{
    legacy_environment::LegacyAvatarEnvironmentState,
    localization::LocalizedText,
    mission_ui::{MissionUiModel, WarpUiEntry},
    movement::{LegacyPlayerController, LegacyWorldColliderPending, make_pc_stop_request},
    network::{NetworkBridge, NetworkCommand},
    system_message_ui::{
        SystemMessageRequest, SystemMessageUiAction, SystemMessageUiModel, SystemMessageUiOutbox,
    },
    tutorial_effects_runtime::TutorialEffectRuntime,
    tutorial_mission_content::{
        GameplayWarpInventoryLocation, GameplayWarpItemLookup, TutorialMissionContent,
        TutorialWarpTarget,
    },
    upsell_ui::{UpsellUiError, UpsellUiModel},
    world::NativeWorldStreamingStatus,
};
use ffone_protocol::{
    DecodedFrame, ItemBase0104, PcGotoSuccess0104, PcWarpUseNpcFailure0104,
    PcWarpUseNpcRequest0104, PcWarpUseNpcSuccess0104, WirePayload, packet,
};
use ffone_runtime_contracts::RETROBUTION_NPC_WARP_EFFECT_IDS;
use std::collections::BTreeMap;

pub(super) const NORMAL_NPC_WARP_DELAY_SECONDS: f32 = 1.5;
pub(super) const NORMAL_NPC_WARP_EFFECT_ID: i32 = RETROBUTION_NPC_WARP_EFFECT_IDS[0];
pub(super) const NORMAL_NPC_WARP_SOUND_TRUE_NAME: &str = "Dexbot_Warp";
pub(super) const NORMAL_NPC_WARP_FAILURE_MESSAGE_ID: i32 = 173;
pub(super) const NORMAL_NPC_WARP_SYSTEM_MESSAGE_ID_BASE: u64 = 0x5741_5250_0000_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct NormalNpcWarpIdentity {
    pub(super) npc_id: i32,
    pub(super) npc_type: i32,
    pub(super) warp_id: i32,
    pub(super) required_task_id: Option<i32>,
    pub(super) target: TutorialWarpTarget,
}

impl NormalNpcWarpIdentity {
    pub(super) fn reject_ui(self, model: &mut MissionUiModel) -> bool {
        model.reject_warp(
            self.npc_id,
            self.npc_type,
            self.warp_id,
            self.required_task_id,
            self.target,
        )
    }

    pub(super) fn confirm_ui(self, model: &mut MissionUiModel) -> bool {
        model.confirm_warp(
            self.npc_id,
            self.npc_type,
            self.warp_id,
            self.required_task_id,
            self.target,
        )
    }
}

pub(super) fn validate_normal_npc_warp_ui_action(
    model: &mut MissionUiModel,
    identity: NormalNpcWarpIdentity,
) -> bool {
    let exact = model.pending_warp_matches(
        identity.npc_id,
        identity.npc_type,
        identity.warp_id,
        identity.required_task_id,
        identity.target,
    ) && model
        .npc_interaction
        .as_ref()
        .is_some_and(|interaction| interaction.npc_id == identity.npc_id);
    if !exact {
        // A stale/forged outbox action must not leave an inherited pending
        // warp latch that permanently disables NpcIconMode input.
        model.clear_npc_interaction_locally();
    }
    exact
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct PendingNormalNpcWarp {
    pub(super) identity: NormalNpcWarpIdentity,
    pub(super) request: PcWarpUseNpcRequest0104,
    pub(super) elapsed_seconds: f32,
    pub(super) sent: bool,
}

impl PendingNormalNpcWarp {
    #[must_use]
    pub(super) fn advance(&mut self, delta_seconds: f32) -> Option<PcWarpUseNpcRequest0104> {
        if self.sent {
            return None;
        }
        self.elapsed_seconds += delta_seconds.max(0.0);
        if self.elapsed_seconds < NORMAL_NPC_WARP_DELAY_SECONDS {
            return None;
        }
        self.sent = true;
        Some(self.request)
    }
}

#[derive(Debug, Resource)]
pub(super) struct NormalNpcWarpRuntime {
    pub(super) departure_clock: WarpDepartureClock,
    pub(super) pending: Option<PendingNormalNpcWarp>,
    pub(super) movement_packet_emission: bool,
    pub(super) window_in_fade_alpha: f32,
    pub(super) pending_system_messages: BTreeMap<u64, i32>,
    pub(super) next_system_message_id: u64,
}

impl Default for NormalNpcWarpRuntime {
    fn default() -> Self {
        Self {
            departure_clock: WarpDepartureClock::default(),
            pending: None,
            movement_packet_emission: true,
            window_in_fade_alpha: 0.0,
            pending_system_messages: BTreeMap::new(),
            next_system_message_id: NORMAL_NPC_WARP_SYSTEM_MESSAGE_ID_BASE,
        }
    }
}

pub(super) const NORMAL_NPC_WARP_PRESENTATION: &str = "local-player.npc-warp.departure";
pub(super) const TRANSPORTATION_WARP_PRESENTATION: &str = "local-player.transportation.departure";

#[derive(Debug, Default)]
pub(super) struct WarpDepartureClock {
    pub(super) started: bool,
    pub(super) elapsed_seconds: f32,
}

impl WarpDepartureClock {
    pub(super) fn delta(&mut self, presentation_ready: bool, effect_alive: bool, delta_seconds: f32) -> f32 {
        if !self.started {
            self.started = presentation_ready;
            // This frame's delta began before the ready edge. Do not charge
            // loading or the input frame to the visible departure animation.
            return 0.0;
        }
        self.elapsed_seconds += delta_seconds.max(0.0);
        // ES394's particles outlive the original 1.5-second request delay.
        // Release elapsed time to the request timer only when that exact
        // departure instance has finished, so a fast reply cannot cut it off.
        if effect_alive {
            return 0.0;
        }
        std::mem::take(&mut self.elapsed_seconds)
    }
}

impl NormalNpcWarpRuntime {
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    #[must_use]
    pub(super) fn correlated_sent(&self) -> Option<PendingNormalNpcWarp> {
        self.pending.filter(|pending| pending.sent)
    }

    pub(super) fn take_correlated_sent(&mut self) -> Option<PendingNormalNpcWarp> {
        self.correlated_sent()?;
        self.movement_packet_emission = true;
        self.pending.take()
    }

    pub(super) fn start_window_in_fade(&mut self) {
        self.window_in_fade_alpha = 1.0;
    }

    pub(super) fn advance_window_in_fade(&mut self, unscaled_delta_seconds: f32) {
        self.window_in_fade_alpha =
            (self.window_in_fade_alpha - unscaled_delta_seconds.max(0.0) * 2.0).max(0.0);
    }

    pub(super) fn queue_system_message(
        &mut self,
        content: &TutorialMissionContent,
        messages: &mut SystemMessageUiModel,
        legacy_message_id: i32,
    ) -> Result<u64, String> {
        let definition = content
            .system_message_definition(legacy_message_id)
            .ok_or_else(|| {
                format!(
                    "normal NPC warp SystemMessage {legacy_message_id} has no source TableData row"
                )
            })?;
        let request_id = self.next_system_message_id;
        self.next_system_message_id = self.next_system_message_id.wrapping_add(1);
        if self.next_system_message_id < NORMAL_NPC_WARP_SYSTEM_MESSAGE_ID_BASE {
            self.next_system_message_id = NORMAL_NPC_WARP_SYSTEM_MESSAGE_ID_BASE;
        }
        self.pending_system_messages
            .insert(request_id, legacy_message_id);
        messages.push(SystemMessageRequest::new_localized(
            request_id,
            LocalizedText::new(
                format!("content.tabledata.message.message.{legacy_message_id}.sz_string"),
                definition.exact_text.clone(),
            ),
            definition.runtime_button_type,
        ));
        Ok(request_id)
    }
}

pub(super) fn normal_npc_warp_item_slot(
    inventory: &LocalInventoryRuntime,
    lookup: GameplayWarpItemLookup,
) -> Option<i32> {
    let start = usize::try_from(lookup.start_slot).ok()?;
    let matches = |item: ItemBase0104| {
        item.item_id > 0
            && i32::from(item.item_id) == lookup.item_id
            && i32::from(item.item_type) == lookup.item_type
    };
    let index = match lookup.item_location {
        GameplayWarpInventoryLocation::Inventory => inventory
            .snapshot()?
            .inventory()
            .iter()
            .copied()
            .enumerate()
            .skip(start)
            .find_map(|(index, item)| matches(item).then_some(index)),
        GameplayWarpInventoryLocation::Quest => inventory
            .quest_inventory
            .as_ref()?
            .iter()
            .copied()
            .enumerate()
            .skip(start)
            .find_map(|(index, item)| matches(item).then_some(index)),
    }?;
    i32::try_from(index).ok()
}

pub(super) fn normal_npc_warp_ui_entry(
    content: &TutorialMissionContent,
    runtime_npc_id: i32,
    npc_type: i32,
) -> Option<WarpUiEntry> {
    content
        .normal_gameplay_warp_for_npc(npc_type)
        .map(|warp| WarpUiEntry {
            npc_id: runtime_npc_id,
            npc_type,
            warp_id: warp.warp_id,
            required_task_id: (warp.limit_task_id != 0).then_some(warp.limit_task_id),
            target: warp.target,
            // The clean action/icon supplies the semantic label; destination
            // ownership remains the exact first serialized WarpTable row.
            label: "WARP".to_owned(),
        })
}

pub(super) fn open_unpaid_past_warp_upsell(model: &mut UpsellUiModel) -> Result<(), UpsellUiError> {
    // The only clean NpcIconMode callsite supplies level 4 and reaches
    // `ReceiveInit`; an existing MainGame camera outside a paid zone selects
    // News mode 2. Upgrade mode 0 is retained data, not this route.
    model.receive_init(PAST_WARP_UPSELL_LEVEL, false)
}

pub(super) fn advance_pending_normal_npc_warp(
    time: Res<Time>,
    effects: Res<TutorialEffectRuntime>,
    bridge: Res<NetworkBridge>,
    mut production: ResMut<NormalNpcWarpRuntime>,
    mut mission_ui: ResMut<MissionUiModel>,
    mut players: Query<(&Transform, &mut LegacyPlayerController), With<LocalPlayer>>,
    mut status: ResMut<RuntimeStatus>,
) {
    let Some(mut pending) = production.pending else {
        return;
    };
    let delta = production.departure_clock.delta(
        effects.named_native_presentation_ready(NORMAL_NPC_WARP_PRESENTATION),
        effects.has_named_native_instance(NORMAL_NPC_WARP_PRESENTATION),
        time.delta_secs(),
    );
    let Some(request) = pending.advance(delta) else {
        production.pending = Some(pending);
        return;
    };
    let Ok((transform, mut controller)) = players.single_mut() else {
        pending.identity.reject_ui(&mut mission_ui);
        production.pending = None;
        status.message =
            "Normal NPC warp blocked without one authoritative local player".to_owned();
        return;
    };
    // The interaction UI can stop local movement before its last scheduled
    // movement packet is sent. Send the exact departure position first so the
    // server validates the warp against the point the player actually used.
    // Both commands go through the same FIFO worker; the server still checks
    // the NPC ID, instance and normal 800-unit interaction range.
    let [position_sync, warp_request] =
        normal_npc_warp_departure_commands(transform.translation, request);
    if let Err(error) = bridge.send(position_sync) {
        pending.identity.reject_ui(&mut mission_ui);
        production.pending = None;
        status.message = format!("Normal NPC warp position sync failed: {error}");
        return;
    }
    match bridge.send(warp_request) {
        Ok(()) => {
            // Clean `NpcIconMode.Update` flips bEnableSendPacket immediately
            // before P_CL2FE_REQ_PC_WARP_USE_NPC and keeps it disabled until
            // failure or destination loading completes.
            production.movement_packet_emission = false;
            production.start_window_in_fade();
            controller.movement_enabled = false;
            production.pending = Some(pending);
            status.message = format!(
                "Normal NPC warp {} sent to OpenFusion after departure effect completion",
                request.warp_id
            );
        }
        Err(error) => {
            pending.identity.reject_ui(&mut mission_ui);
            production.pending = None;
            status.message = format!("Normal NPC warp transport failed: {error}");
        }
    }
}

pub(super) fn normal_npc_warp_departure_commands(
    position: Vec3,
    request: PcWarpUseNpcRequest0104,
) -> [NetworkCommand; 2] {
    [
        NetworkCommand::Stop(make_pc_stop_request(position)),
        NetworkCommand::UseNpcWarp(request),
    ]
}

#[derive(Component)]
pub(super) struct NormalNpcWarpWindowInFadeOverlay;

pub(super) fn render_normal_npc_warp_window_in_fade(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut production: ResMut<NormalNpcWarpRuntime>,
    mut overlays: Query<(Entity, &mut BackgroundColor), With<NormalNpcWarpWindowInFadeOverlay>>,
) {
    let alpha = production.window_in_fade_alpha;
    let mut existing = overlays.iter_mut();
    if alpha > 0.0 {
        if let Some((_, mut color)) = existing.next() {
            color.0 = Color::srgba(1.0, 1.0, 1.0, alpha);
        } else {
            commands.spawn((
                Name::new("NpcIconMode WIn white fade"),
                NormalNpcWarpWindowInFadeOverlay,
                WorldSliceEntity,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::ZERO,
                    top: Val::ZERO,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, alpha)),
                ZIndex(10_000),
                Pickable::IGNORE,
            ));
        }
    }
    for (entity, _) in existing {
        commands.entity(entity).despawn();
    }
    // `GameFrame.WIn` draws the current alpha before subtracting this
    // frame's unscaled delta, so the send edge visibly starts at pure white.
    if alpha > 0.0 {
        production.advance_window_in_fade(time.delta_secs());
    }
}

pub(super) fn consume_normal_warp_system_message_outbox(
    mut system_outbox: ResMut<SystemMessageUiOutbox>,
    mut production: ResMut<NormalNpcWarpRuntime>,
    mut mission_ui: ResMut<MissionUiModel>,
    mut status: ResMut<RuntimeStatus>,
) {
    if production.pending_system_messages.is_empty() {
        return;
    }
    let mut unrelated = Vec::new();
    for action in system_outbox.drain().collect::<Vec<_>>() {
        let SystemMessageUiAction::Chosen { request_id, .. } = action;
        let Some(message_id) = production.pending_system_messages.remove(&request_id) else {
            unrelated.push(action);
            continue;
        };
        // Clean SystemMessage 110..115/173 callbacks call NpcIconMode.EndMode
        // rather than reopening the rejected source interaction.
        mission_ui.clear_npc_interaction_locally();
        status.message = format!("Normal NPC warp SystemMessage {message_id} dismissed");
    }
    for action in unrelated {
        system_outbox.push(action);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AuthoritativeWarpReply0104 {
    NpcSuccess(PcWarpUseNpcSuccess0104),
    NpcFailure(PcWarpUseNpcFailure0104),
    GotoSuccess(PcGotoSuccess0104),
}

pub(super) fn decode_authoritative_warp_reply_0104(
    frame: &DecodedFrame,
) -> Result<Option<AuthoritativeWarpReply0104>, String> {
    match frame.packet_type {
        packet::P_FE2CL_REP_PC_WARP_USE_NPC_SUCC => PcWarpUseNpcSuccess0104::decode(&frame.payload)
            .map(AuthoritativeWarpReply0104::NpcSuccess)
            .map(Some)
            .map_err(|error| format!("malformed protocol-0104 NPC-warp success: {error}")),
        packet::P_FE2CL_REP_PC_WARP_USE_NPC_FAIL => PcWarpUseNpcFailure0104::decode(&frame.payload)
            .map(AuthoritativeWarpReply0104::NpcFailure)
            .map(Some)
            .map_err(|error| format!("malformed protocol-0104 NPC-warp failure: {error}")),
        packet::P_FE2CL_REP_PC_GOTO_SUCC => PcGotoSuccess0104::decode(&frame.payload)
            .map(AuthoritativeWarpReply0104::GotoSuccess)
            .map(Some)
            .map_err(|error| format!("malformed protocol-0104 authoritative GOTO: {error}")),
        _ => Ok(None),
    }
}

/// Re-enters the same collider/readiness barrier used by initial world entry.
///
/// OpenFusion updates its authoritative position immediately after emitting a
/// warp response. Keeping the old terrain interactive while the destination
/// dongs stream can drop the avatar through empty space and can also preserve
/// stale poison/water state from the source tile.
#[allow(clippy::too_many_arguments)]
pub(super) fn apply_authoritative_world_teleport(
    commands: &mut Commands,
    loading: &mut GameplayLoadingState,
    player: Entity,
    transform: &mut Transform,
    controller: &mut LegacyPlayerController,
    visibility: &mut Visibility,
    environment: &mut LegacyAvatarEnvironmentState,
    position: Vec3,
    hp: Option<i32>,
) -> bool {
    // `sendPlayerTo` commonly emits NPC_WARP_SUCC followed by GOTO_SUCC. The
    // second packet must still reconcile the position, but must not restart an
    // already-progressing loader for the identical destination.
    let duplicate_in_flight = loading.visible
        && loading.scope == Some(ResourceLoadingScope::World)
        && loading.blocked.is_none()
        && (transform.translation - position).length_squared() <= 0.0001;

    transform.translation = position;
    controller.apply_authoritative_teleport(position);
    controller.movement_enabled = false;
    *visibility = Visibility::Hidden;
    *environment = LegacyAvatarEnvironmentState {
        last_observed_hp: hp,
        ..default()
    };
    commands
        .entity(player)
        .insert((LegacyWorldColliderPending, WarpLoadingAcknowledgment));

    if duplicate_in_flight {
        return false;
    }
    commands.insert_resource(NativeWorldStreamingStatus::default());
    loading.begin(ResourceLoadingScope::World);
    true
}
