//! Item selection and bounded quantity drafts. The vendor runtime owns transactions.
use super::*;
use crate::user_equip_ui::{
    USER_EQUIP_BUTTON_HOVER_PATH, USER_EQUIP_BUTTON_NORMAL_PATH, USER_EQUIP_CALCULATOR_BACK_PATH,
    USER_EQUIP_EQUIP_POPUP_PATH, USER_EQUIP_GENERAL_DIALOG_PATH, USER_EQUIP_USE_DIALOG_PATH,
};

#[cfg(test)]
mod tests;

mod state;
mod operations_spawn;
mod operations_bind;
mod types;
mod interaction;

pub use state::VendorItemPopupState;
use operations_spawn::{current_contract, reconcile, rotate_try_on, spawn};
use operations_bind::bind;
pub(super) use types::VendorItemPopupPlugin;
use types::Part;
use interaction::input;
