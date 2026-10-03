use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VendorIconRefError {
    UnsafeRuntimePath { runtime_path: String },
}

impl fmt::Display for VendorIconRefError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsafeRuntimePath { runtime_path } => {
                write!(f, "unsafe VendorMode runtime asset path: {runtime_path}")
            }
        }
    }
}

impl Error for VendorIconRefError {}

pub(super) fn vendor_item_needs_equip_validation(item_type: i16) -> bool {
    (0..=6).contains(&item_type) || item_type == 10
}

pub(super) fn equip_validation(
    item: ItemBase0104,
    eligibility: &impl VendorEquipEligibility0104,
) -> VendorEquipValidation0104 {
    if !vendor_item_needs_equip_validation(item.item_type) {
        VendorEquipValidation0104::NotApplicable
    } else {
        match eligibility.enable_equip(item) {
            Some(true) => VendorEquipValidation0104::Allowed,
            Some(false) => VendorEquipValidation0104::Rejected,
            None => VendorEquipValidation0104::Unverified,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VendorUiAssetContractError {
    UnsafeImagePath { index: usize, path: String },
    UnsafeFontPath { path: String },
}

impl fmt::Display for VendorUiAssetContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsafeImagePath { index, path } => {
                write!(f, "VendorMode image path {index} is unsafe: {path}")
            }
            Self::UnsafeFontPath { path } => {
                write!(f, "VendorMode font path is unsafe: {path}")
            }
        }
    }
}

impl Error for VendorUiAssetContractError {}
