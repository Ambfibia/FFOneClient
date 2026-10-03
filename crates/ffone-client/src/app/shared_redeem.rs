use super::*;
use ffone_client::shared_input_ui::{
    REDEEM_INPUT_OWNER, RedeemCodeError, RedeemSource, SharedInputAction, SharedInputDialog,
    SharedRedeemCode, redeem_code_request,
};

const REDEEM_SPACE_MESSAGE: u64 = 0x5245_4445_454d_0002;

#[derive(SystemParam)]
pub(super) struct InventoryRedeemOwner<'w> {
    state: Res<'w, UserEquipUiState>,
    projection: Res<'w, UserEquipItemModeProjection>,
    modal: ResMut<'w, UserEquipModalState>,
}

pub(super) fn open_vendor_redeem(
    state: &VendorUiState,
    modal: VendorModalState,
    projection: &VendorModeProjection0104,
    redeem: &mut SharedRedeemCode,
    input: &mut SharedInputDialog,
) -> bool {
    state.input_capabilities(modal).pc_stuff_controls
        && redeem.open(
            RedeemSource::Vendor {
                pc: projection.owner_pc_id,
                npc: projection.session.requested_npc_id,
            },
            input,
        )
}

pub(super) fn consume_shared_redeem(
    client: Res<State<ClientState>>,
    mut status: ResMut<RuntimeStatus>,
    bridge: Res<NetworkBridge>,
    bank: Res<BankUiState>,
    bank_projection: Res<BankModeProjection0104>,
    vendor: Res<VendorUiState>,
    vendor_projection: Res<VendorModeProjection0104>,
    mut bank_modal: ResMut<BankModalState>,
    mut vendor_modal: ResMut<VendorModalState>,
    mut input: ResMut<SharedInputDialog>,
    mut redeem: ResMut<SharedRedeemCode>,
    mut messages: ResMut<SystemMessageUiModel>,
    mut results: ResMut<SystemMessageUiOutbox>,
    audio: Option<ResMut<GameplayAudioRuntime>>,
    mut inventory: InventoryRedeemOwner,
) {
    if !results.is_empty() {
        results.drain_matching(|action| matches!(action, SystemMessageUiAction::Chosen { request_id, .. } if *request_id == REDEEM_SPACE_MESSAGE));
    }
    let Some(source) = redeem.source else {
        return;
    };
    let valid = *client.get() == ClientState::World
        && status.hp.is_none_or(|hp| hp > 0)
        && match source {
            RedeemSource::Inventory { pc } => {
                status.player_id == Some(pc)
                    && inventory.state.phase()
                        == ffone_client::user_equip_ui::UserEquipLifecyclePhase::Visible
                    && inventory.projection.owner_pc_id == pc
            }
            RedeemSource::Bank { pc, npc } => {
                status.player_id == Some(pc)
                    && bank.phase == BankLifecyclePhase::Visible
                    && bank_projection.owner_pc_id == pc
                    && bank_projection.npc_id == npc
            }
            RedeemSource::Vendor { pc, npc } => {
                status.player_id == Some(pc)
                    && vendor.phase == VendorLifecyclePhase::Visible
                    && vendor_projection.owner_pc_id == pc
                    && vendor_projection.session.requested_npc_id == npc
            }
        };
    if !valid || input.owner() != Some(REDEEM_INPUT_OWNER) {
        input.close(REDEEM_INPUT_OWNER);
        match source {
            RedeemSource::Inventory { .. } => inventory.modal.redeem_code_view = false,
            RedeemSource::Bank { .. } => bank_modal.inventory_popup = false,
            RedeemSource::Vendor { .. } => vendor_modal.inventory_popup = false,
        }
        redeem.source = None;
        return;
    }
    let mut cue = None;
    while let Some(action) = input.pop_for(REDEEM_INPUT_OWNER) {
        match action {
            SharedInputAction::Cancel { .. } => {
                cue = Some("No_Button");
                input.close(REDEEM_INPUT_OWNER);
            }
            SharedInputAction::Submit { value, .. } => {
                cue = Some("Yes_Button");
                match redeem_code_request(&value) {
                    Err(RedeemCodeError::TooShort | RedeemCodeError::TooLong) => continue,
                    Err(RedeemCodeError::ContainsSpace) => {
                        messages.push(SystemMessageRequest::new_localized(
                            REDEEM_SPACE_MESSAGE,
                            LocalizedText::new(
                                "ui.shared_input.redeem_space_error",
                                "Codes cannot contain spaces.",
                            ),
                            SystemMessageButtonType::Ok,
                        ));
                    }
                    Ok(request) => {
                        match ffone_protocol::RegisteredGameplayRequest0104::new(
                            318_767_111,
                            request.encode(),
                        ) {
                            Ok(request) => {
                                if let Err(error) =
                                    bridge.send(NetworkCommand::SendRegisteredGameplay0104(request))
                                {
                                    status.message =
                                        format!("Redeem request transport failed: {error}");
                                }
                            }
                            Err(error) => {
                                status.message = format!("Redeem request rejected: {error}")
                            }
                        }
                    }
                }
                input.close(REDEEM_INPUT_OWNER);
            }
        }
    }
    if let (Some(cue), Some(mut audio)) = (cue, audio) {
        audio.queue_gameplay_ui_sound(cue);
    }
    let open = input.owner() == Some(REDEEM_INPUT_OWNER);
    match source {
        RedeemSource::Inventory { .. } => {
            if inventory.modal.redeem_code_view != open {
                inventory.modal.redeem_code_view = open;
            }
        }
        RedeemSource::Bank { .. } => {
            if bank_modal.inventory_popup != open {
                bank_modal.inventory_popup = open;
            }
        }
        RedeemSource::Vendor { .. } => {
            if vendor_modal.inventory_popup != open {
                vendor_modal.inventory_popup = open;
            }
        }
    }
    if !open {
        redeem.source = None;
    }
}

#[cfg(test)]
mod tests;
