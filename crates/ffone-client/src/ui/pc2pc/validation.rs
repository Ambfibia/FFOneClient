use super::*;

pub(super) fn validate_trade_item(
    item: Pc2pcTradeItem0104,
    allow_empty: bool,
) -> Result<(), Pc2pcTradeItemError0104> {
    if item.is_empty() {
        return if allow_empty {
            Ok(())
        } else {
            Err(Pc2pcTradeItemError0104::MalformedIdentity {
                item_type: item.item_type,
                item_id: item.item_id,
            })
        };
    }
    if item.item_type < 0 {
        return Err(Pc2pcTradeItemError0104::MalformedIdentity {
            item_type: item.item_type,
            item_id: item.item_id,
        });
    }
    if !(0..INVENTORY_SLOT_COUNT_0104 as i32).contains(&item.inventory_slot) {
        return Err(Pc2pcTradeItemError0104::InventorySlotOutOfBounds {
            inventory_slot: item.inventory_slot,
        });
    }
    if !(0..PC2PC_OFFER_SLOT_COUNT as i32).contains(&item.offer_slot) {
        return Err(Pc2pcTradeItemError0104::OfferSlotOutOfBounds {
            offer_slot: item.offer_slot,
        });
    }
    if item.item_type == 7 && item.option <= 0 {
        return Err(Pc2pcTradeItemError0104::GeneralCountNotPositive {
            option: item.option,
        });
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Pc2pcPortraitRefError {
    EmptyPath,
    UnsafePath { runtime_path: String },
}

impl fmt::Display for Pc2pcPortraitRefError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid Pc2pc portrait reference: {self:?}")
    }
}

impl Error for Pc2pcPortraitRefError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pc2pcEquipValidation {
    NotApplicable,
    Allowed,
    Rejected,
    Unverified,
}

pub(super) fn validate_final_pending(
    state: &Pc2pcUiState,
    participant: Pc2pcParticipant0104,
    envelope: Pc2pcEnvelope0104,
) -> Result<(), Pc2pcCorrelationError0104> {
    match participant {
        Pc2pcParticipant0104::Local if state.pending.is_none() && state.local_ready && state.remote_ready
            && state.phase == Pc2pcLifecyclePhase::Completing => Ok(()),
        Pc2pcParticipant0104::Local => match pending_request(state)? {
            Pc2pcPendingRequest0104::Confirm(expected) if expected.envelope == envelope => Ok(()),
            pending => Err(Pc2pcCorrelationError0104::PendingKindMismatch {
                pending,
                outcome: Pc2pcRequestFailureKind0104::Confirm,
            }),
        },
        Pc2pcParticipant0104::Remote => {
            if !state.local_visual_ready() {
                return Err(Pc2pcCorrelationError0104::FinalWithoutLocalAcceptance);
            }
            if let Some(pending) = state.pending.as_ref().cloned() {
                if !matches!(pending, Pc2pcPendingRequest0104::Confirm(_)) {
                    return Err(
                        Pc2pcCorrelationError0104::FinalWhileUnrelatedRequestPending { pending },
                    );
                }
            }
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pc2pcAssetPathError {
    Empty,
    Absolute,
    Backslash,
    Traversal,
}

pub(super) fn validate_runtime_asset_path(path: &str) -> Result<(), Pc2pcAssetPathError> {
    if path.trim().is_empty() {
        return Err(Pc2pcAssetPathError::Empty);
    }
    if path.starts_with('/') || path.get(1..3) == Some(":\\") || path.get(1..3) == Some(":/") {
        return Err(Pc2pcAssetPathError::Absolute);
    }
    if path.contains('\\') {
        return Err(Pc2pcAssetPathError::Backslash);
    }
    if path.split('/').any(|part| part == "..") {
        return Err(Pc2pcAssetPathError::Traversal);
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Pc2pcUiAssetContractError {
    UnsafeImagePath {
        index: usize,
        path: String,
        reason: Pc2pcAssetPathError,
    },
    DuplicateImagePath {
        index: usize,
        path: String,
    },
    UnsafeFontPath {
        path: String,
        reason: Pc2pcAssetPathError,
    },
}

impl fmt::Display for Pc2pcUiAssetContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid Pc2pc UI asset contract: {self:?}")
    }
}

impl Error for Pc2pcUiAssetContractError {}
