//! Resurrect UI context, outbox and packets.

use super::LocalPlayer;
use super::local_inventory::LocalInventoryRuntime;
use super::nano_free_tuning::NanoFreeTuningProductionRuntime;
use super::race::warp_away_xcom_index;
use super::runtime_status::{RuntimeStatus, resolve_runtime_resurrection_item_slot};
use super::state::ClientState;
use bevy::prelude::*;
use ffone_client::{
    coordinates::ProtocolPosition,
    gameplay_ui::GameplayUiModel,
    movement::LegacyWorldColliderPending,
    nano_free_tuning_ui::NanoFreeTuningModel,
    network::{NetworkBridge, NetworkCommand},
    resurrect_ui::{ResurrectUiAction, ResurrectUiContext, ResurrectUiModel, ResurrectUiOutbox},
    skill_buff_ui::SkillBuffUiModel,
    system_message_ui::SystemMessageUiModel,
    tutorial_mission_content::TutorialMissionContent,
};
use ffone_protocol::{
    DecodedFrame, PcRegenPacket0104, PcRegenRequest0104, PcRegenSuccess0104,
    decode_pc_regen_packet_0104,
};

pub(super) fn reset_resurrect_shell(
    model: &mut ResurrectUiModel,
    context: &mut ResurrectUiContext,
    outbox: &mut ResurrectUiOutbox,
) {
    model.reset();
    *context = ResurrectUiContext::default();
    outbox.clear();
}

pub(super) fn consume_resurrect_ui_outbox(
    bridge: Res<NetworkBridge>,
    mut outbox: ResMut<ResurrectUiOutbox>,
    mut status: ResMut<RuntimeStatus>,
) {
    while let Some(action) = outbox.pop_front() {
        match action {
            ResurrectUiAction::Entered { .. } => {
                status.message = "ResurrectMode entered from authoritative zero HP".to_owned();
            }
            ResurrectUiAction::RequestRegen(request) => {
                let packet = PcRegenRequest0104 {
                    regen_type: request.regen_type,
                    e_il: request.e_il,
                    index: request.index,
                };
                status.message = format!(
                    "Requesting {:?} regeneration from OpenFusion ({:?})...",
                    request.choice, request.origin
                );
                if let Err(error) = bridge.send(NetworkCommand::Regen(packet)) {
                    // Protocol-0104 defines no regeneration failure packet.
                    // Preserve clean request_sent ownership instead of
                    // pretending the server rejected and reopened controls.
                    status.message = format!("ResurrectMode regeneration send failed: {error}");
                }
            }
            ResurrectUiAction::RegenSucceeded { effects } => {
                if effects.restore_nano_selection {
                    status.message =
                        "Regeneration complete; NanoFreeTuning mode 22 selection restored"
                            .to_owned();
                }
            }
        }
    }
}

pub(super) fn sync_resurrect_ui_context(
    state: Res<State<ClientState>>,
    gameplay_ui: Res<GameplayUiModel>,
    system_messages: Res<SystemMessageUiModel>,
    skill_buffs: Res<SkillBuffUiModel>,
    content: Res<TutorialMissionContent>,
    inventory: Res<LocalInventoryRuntime>,
    runtime: Res<RuntimeStatus>,
    nano_free_tuning: Res<NanoFreeTuningModel>,
    nano_free_tuning_production: Res<NanoFreeTuningProductionRuntime>,
    ready_players: Query<&Transform, (With<LocalPlayer>, Without<LegacyWorldColliderPending>)>,
    mut context: ResMut<ResurrectUiContext>,
    mut model: ResMut<ResurrectUiModel>,
    mut outbox: ResMut<ResurrectUiOutbox>,
) {
    let in_game = matches!(state.get(), ClientState::Tutorial | ClientState::World);
    if !in_game {
        model.reset();
        *context = ResurrectUiContext::default();
        outbox.clear();
        return;
    }
    model.set_ui_scale(gameplay_ui.ui_scale);
    let ready_player = ready_players.single().ok();
    context.ready_for_play = ready_player.is_some();
    context.player_available = runtime.player_id.is_some();
    context.system_popup_active = system_messages.is_popup();
    // These serialized references are non-null in clean Retrobution and their
    // content-addressed native copies are startup-verified by the UI slice.
    context.skill_icon_back_available = true;
    context.phoenix_group_icon_available = true;
    context.phoenix_self_icon_available = true;
    context.phoenix_self_skill_bit_16 = skill_buffs.local_condition_bit_flag & 0x10 != 0;
    context.resurrection_item_slot = inventory
        .snapshot()
        .and_then(|inventory| resolve_runtime_resurrection_item_slot(inventory, &content));
    context.nearest_xcom_index = ready_player.and_then(|transform| {
        warp_away_xcom_index(
            &content,
            runtime.map_number,
            ProtocolPosition::from_native(transform.translation).raw(),
        )
    });
    // Group Phoenix ownership and the general-inventory lookup remain
    // explicit external inputs; this shell never fabricates them.
    if runtime.hp.is_some_and(|hp| hp <= 0) && !model.visible {
        // Covers a dead authoritative WorldReady snapshot. Live SetHP packets
        // enter immediately in `poll_network`; repeated dead status packets
        // preserve the current death's deadline and pending request.
        model.enter(
            if nano_free_tuning_production.modal_active(&nano_free_tuning) {
                22
            } else {
                0
            },
            &mut outbox,
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LocalResurrectPacket0104 {
    Success(PcRegenSuccess0104),
    SuddenDead { hp: i32 },
}

pub(super) fn decode_local_resurrect_packet_0104(
    frame: &DecodedFrame,
    local_player_id: Option<i32>,
) -> Result<Option<LocalResurrectPacket0104>, String> {
    let packet = decode_pc_regen_packet_0104(frame.packet_type, &frame.payload)
        .map_err(|error| format!("malformed protocol-0104 PC regeneration packet: {error}"))?;
    Ok(match packet {
        Some(PcRegenPacket0104::Success(success)) => {
            Some(LocalResurrectPacket0104::Success(success))
        }
        Some(PcRegenPacket0104::SuddenDead(sudden_dead))
            if local_player_id == Some(sudden_dead.pc_id) =>
        {
            Some(LocalResurrectPacket0104::SuddenDead { hp: sudden_dead.hp })
        }
        // Request packets are client-to-server only. Broadcast regeneration
        // and sudden-death packets for other PCs remain owned by the network
        // entity-lifecycle path and must never drive the local modal.
        Some(
            PcRegenPacket0104::Request(_)
            | PcRegenPacket0104::Broadcast(_)
            | PcRegenPacket0104::SuddenDead(_),
        )
        | None => None,
    })
}
