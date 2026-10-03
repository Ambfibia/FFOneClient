use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankEquipValidation {
    NotApplicable,
    Allowed,
    Rejected,
    Unverified,
}

pub(super) fn bank_item_needs_equip_validation(item_type: i16) -> bool {
    (0..=6).contains(&item_type) || item_type == 10
}

pub(super) fn validate_authoritative_bank_item(
    item: ItemBase0104,
    slot: BankSlotRef0104,
) -> Result<(), BankAuthorityMutationError0104> {
    if item.item_id > 0 && item.item_type < 0 {
        return Err(BankAuthorityMutationError0104::MalformedItemIdentity {
            slot,
            item_type: item.item_type,
            item_id: item.item_id,
        });
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BankUiAssetContractError {
    UnsafeImagePath { index: usize, path: String },
    UnsafeFontPath { path: String },
}

impl fmt::Display for BankUiAssetContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsafeImagePath { index, path } => {
                write!(f, "BankMode image path {index} is unsafe: {path}")
            }
            Self::UnsafeFontPath { path } => {
                write!(f, "BankMode font path is unsafe: {path}")
            }
        }
    }
}

impl Error for BankUiAssetContractError {}
