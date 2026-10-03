use super::*;

pub(super) fn drain_model_intents(
    session: &mut EnchantProductionSession0104,
    output: &mut EnchantProductionOutput0104,
) -> Result<Vec<EnchantAuthoritativeReceipt0104>, EnchantProductionError0104> {
    let mut receipts = Vec::new();
    while let Some(intent) = session.model.pop_intent() {
        match intent {
            EnchantIntent0104::Audio(intent) => {
                output.effects.push(EnchantShellEffect0104::Audio(intent));
            }
            EnchantIntent0104::Animation(intent) => {
                output
                    .effects
                    .push(EnchantShellEffect0104::Animation(intent));
            }
            EnchantIntent0104::Camera(intent) => {
                output.effects.push(EnchantShellEffect0104::Camera(intent));
            }
            EnchantIntent0104::Lifecycle(EnchantLifecycleIntent0104::SetCursorLock(value)) => {
                output.effects.push(EnchantShellEffect0104::Input(
                    EnchantInputEffect0104::SetCursorLock(value),
                ));
            }
            EnchantIntent0104::Lifecycle(intent) => {
                output
                    .effects
                    .push(EnchantShellEffect0104::Lifecycle(intent));
            }
            EnchantIntent0104::Popup(intent) => {
                output.effects.push(EnchantShellEffect0104::Popup(intent));
            }
            EnchantIntent0104::Selection(intent) => {
                apply_selection_intent(session, &intent)?;
                output
                    .effects
                    .push(EnchantShellEffect0104::Selection(intent));
            }
            EnchantIntent0104::Wire {
                request_token,
                packet_id,
                payload,
                request,
            } => {
                if packet_id != ENCHANT_REQUEST_PACKET_ID_0104 {
                    return Err(EnchantProductionError0104::WrongWirePacket {
                        expected: ENCHANT_REQUEST_PACKET_ID_0104,
                        actual: packet_id,
                    });
                }
                if payload != request.encode() {
                    return Err(EnchantProductionError0104::InvalidRequestSlots {
                        detail: "model wire payload disagrees with its typed request",
                    });
                }
                let EnchantPhase0104::AwaitingReply {
                    request_token: phase_token,
                } = session.model.phase()
                else {
                    return Err(EnchantProductionError0104::InvalidRequestSlots {
                        detail: "wire intent was emitted outside AwaitingReply",
                    });
                };
                if *phase_token != request_token {
                    return Err(EnchantProductionError0104::WrongRequestToken {
                        expected: *phase_token,
                        actual: request_token,
                    });
                }
                validate_request_against_reservations(session, request)?;
                ensure_no_pending_request(session)?;
                let registered = encode_enchant_request_0104(request)?;
                set_single_request(output, registered)?;
                session.pending_request = Some(PendingRequest0104::Enchant(PendingEnchant0104 {
                    request_token,
                    request,
                }));
            }
            EnchantIntent0104::RedeemWire(wire) => {
                let registered = encode_enchant_redeem_request_0104(&wire)?;
                set_single_request(output, registered)?;
            }
            EnchantIntent0104::AuthoritativeReceipt(receipt) => receipts.push(receipt),
            EnchantIntent0104::RefreshInventory => {
                output
                    .effects
                    .push(EnchantShellEffect0104::RefreshInventory);
            }
            EnchantIntent0104::TransportFailure(failure) => {
                output
                    .effects
                    .push(EnchantShellEffect0104::TransportFailure(failure));
            }
            EnchantIntent0104::AuxiliaryUnlock(packet_type) => {
                output
                    .effects
                    .push(EnchantShellEffect0104::AuxiliaryUnlock(packet_type));
            }
            EnchantIntent0104::IgnoredSuccessFlag(flag) => {
                output
                    .effects
                    .push(EnchantShellEffect0104::IgnoredSuccessFlag(flag));
            }
        }
    }
    Ok(receipts)
}
