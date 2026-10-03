use super::*;
use ffone_client::bank_ui::{BankItemDeleteIntent0104, BankItemDeleteState};

fn request_for(intent: BankItemDeleteIntent0104) -> PcItemDeleteRequest0104 {
    PcItemDeleteRequest0104 {
        item_location: 1,
        slot_num: intent.slot as i32,
    }
}

pub(super) fn consume(
    client: Res<State<ClientState>>,
    mut status: ResMut<RuntimeStatus>,
    bridge: Res<NetworkBridge>,
    inventory: Res<LocalInventoryRuntime>,
    projection: Res<BankModeProjection0104>,
    bank: Res<BankProductionRuntime0104>,
    mut state: ResMut<BankUiState>,
    mut deletion: ResMut<BankItemDeleteState>,
    mut production: ResMut<UserEquipProductionRuntime0104>,
) {
    if let Some(sent) = deletion.sent() {
        let same_owner = *client.get() == ClientState::World
            && status.player_id == Some(sent.pc)
            && projection.owner_pc_id == sent.pc
            && projection.npc_id == sent.npc
            && state.phase == BankLifecyclePhase::Visible;
        let pending =
            production.pending() == Some(UserEquipPendingRequest0104::Delete(request_for(sent)));
        if !same_owner || !pending {
            if !same_owner && pending && status.player_id == Some(sent.pc) {
                production.close_session_preserving_late_replies();
            }
            deletion.finish();
            if projection.owner_pc_id == sent.pc
                && projection.npc_id == sent.npc
                && !bank.request_pending()
            {
                state.send_pending = false;
            }
        } else if !state.send_pending {
            state.send_pending = true;
        }
    }
    if !deletion.has_ready() {
        return;
    }
    let Some(intent) = deletion.take_ready() else {
        return;
    };
    if *client.get() != ClientState::World
        || status.player_id != Some(intent.pc)
        || status.hp.is_some_and(|hp| hp <= 0)
        || state.phase != BankLifecyclePhase::Visible
        || state.send_pending
        || bank.request_pending()
        || production.send_pending()
        || !intent.valid(&projection)
    {
        status.message = "Bank item-delete confirmation expired before send".into();
        return;
    }
    let Some(authority) = inventory.snapshot() else {
        return;
    };
    if authority.owner_pc_id() != intent.pc
        || authority.inventory().get(intent.slot) != Some(&intent.item)
    {
        status.message = "Bank item-delete authority changed before send".into();
        return;
    }
    let Ok(request) = prepare_user_equip_delete_0104(authority, intent.slot) else {
        return;
    };
    if let Err(error) = production.begin(UserEquipPendingRequest0104::Delete(request)) {
        status.message = format!("Bank item-delete rejected: {error}");
        return;
    }
    if let Err(error) = bridge.send(NetworkCommand::DeleteInventoryItem(request)) {
        production.cancel();
        status.message = format!("Bank item-delete transport failed: {error}");
        return;
    }
    deletion.mark_sent(intent);
    state.send_pending = true;
    status.message = format!(
        "Bank sent confirmed inventory deletion for slot {}; awaiting authority",
        intent.slot
    );
}

#[cfg(test)]
mod tests;
