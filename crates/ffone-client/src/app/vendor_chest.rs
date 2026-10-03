use super::*;
use ffone_client::vendor_ui::{VendorChestOpenIntent, VendorChestOpenState};

fn request_for(intent: VendorChestOpenIntent) -> ffone_protocol::ItemChestOpenRequest0104 {
    ffone_protocol::ItemChestOpenRequest0104 {
        item_location: 1,
        slot_num: intent.slot as i32,
        chest_item: ItemBase0104 {
            time_limit: 0,
            ..intent.item
        },
    }
}

pub(super) fn consume(
    client: Res<State<ClientState>>,
    mut status: ResMut<RuntimeStatus>,
    bridge: Res<NetworkBridge>,
    inventory: Res<LocalInventoryRuntime>,
    projection: Res<VendorModeProjection0104>,
    vendor: Res<VendorProductionRuntime0104>,
    mut state: ResMut<VendorUiState>,
    mut chest: ResMut<VendorChestOpenState>,
    mut production: ResMut<UserEquipProductionRuntime0104>,
    mut audio: Option<ResMut<GameplayAudioRuntime>>,
) {
    if let Some(sent) = chest.sent() {
        let same_owner = *client.get() == ClientState::World
            && status.player_id == Some(sent.pc)
            && projection.owner_pc_id == sent.pc
            && vendor.session() == Some(sent.session)
            && state.phase == VendorLifecyclePhase::Visible;
        let pending =
            production.pending() == Some(UserEquipPendingRequest0104::ChestOpen(request_for(sent)));
        if !same_owner || !pending {
            if !same_owner && pending && status.player_id == Some(sent.pc) {
                production.close_session_preserving_late_replies();
            }
            chest.finish();
            if projection.owner_pc_id == sent.pc
                && projection.session == sent.session
                && !vendor.request_pending()
            {
                state.send_pending = false;
            }
        } else if !state.send_pending {
            state.send_pending = true;
        }
    }
    if !chest.has_ready() {
        return;
    }
    let Some(intent) = chest.take_ready() else {
        return;
    };
    if *client.get() != ClientState::World
        || status.player_id != Some(intent.pc)
        || status.hp.is_some_and(|hp| hp <= 0)
        || state.phase != VendorLifecyclePhase::Visible
        || state.send_pending
        || vendor.request_pending()
        || vendor.session() != Some(intent.session)
        || production.send_pending()
        || !intent.valid(&projection)
    {
        status.message = "Vendor chest selection expired before send".into();
        return;
    }
    let Some(authority) = inventory.snapshot() else {
        return;
    };
    if authority.owner_pc_id() != intent.pc
        || authority.inventory().get(intent.slot) != Some(&intent.item)
    {
        return;
    }
    let Ok(request) = prepare_user_equip_chest_open_0104(authority, intent.slot) else {
        return;
    };
    if let Err(error) = production.begin(UserEquipPendingRequest0104::ChestOpen(request)) {
        status.message = format!("Vendor chest open rejected: {error}");
        return;
    }
    if let Err(error) = bridge.send(NetworkCommand::OpenChest(request)) {
        production.cancel();
        status.message = format!("Vendor chest open transport failed: {error}");
        return;
    }
    chest.mark_sent(intent);
    state.send_pending = true;
    if let Some(audio) = audio.as_mut() {
        audio.queue_gameplay_ui_sound("Container_Open01");
    }
}

#[cfg(test)]
mod tests;
