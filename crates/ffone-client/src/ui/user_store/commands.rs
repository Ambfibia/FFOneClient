use super::*;

pub const USER_STORE_HELP_EVENT_ID: i32 = 24;

pub const USER_STORE_GENERIC_CALCULATOR_ACTION_AREA: UserStoreUiRect =
    UserStoreUiRect::new(0.0, 160.0, 100.0, 30.0);

pub const STREETSTALL_READY_REQUEST_SIZE: usize = 4;

pub const STREETSTALL_CANCEL_REQUEST_SIZE: usize = 4;

pub const STREETSTALL_REGISTER_REQUEST_SIZE: usize = 24;

pub const STREETSTALL_UNREGISTER_REQUEST_SIZE: usize = 4;

pub const STREETSTALL_SALE_START_REQUEST_SIZE: usize = 4;

pub const STREETSTALL_ITEM_LIST_REQUEST_SIZE: usize = 4;

pub const STREETSTALL_ITEM_BUY_REQUEST_SIZE: usize = 12;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UserStoreReplyReject0104 {
    Codec(UserStoreCodecError0104),
    NoPendingRequest {
        packet_id: u32,
    },
    UnexpectedForPending {
        packet_id: u32,
        pending: UserStorePendingRequest0104,
    },
    CorrelationMismatch(&'static str),
    Apply(UserStoreApplyError0104),
}

impl fmt::Display for UserStoreReplyReject0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "street-stall reply rejected: {self:?}")
    }
}

impl Error for UserStoreReplyReject0104 {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserStoreRequestBlock0104 {
    Inactive,
    Busy,
    WrongMode,
    StoreOpen,
    StoreClosed,
    InvalidListSlot,
    InvalidInventorySlot,
    EmptyInventorySlot,
    EmptyListSlot,
    Sold,
    InvalidQuantity,
    InvalidPrice,
    InsufficientTaros,
    InventoryFull,
    ReturnStoreUnreachable,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UserStoreActionOutcome0104 {
    Sent(UserStorePacket0104),
    Popup(UserStoreItemPopup0104),
    SystemMessage(&'static str),
    LocalExit,
    NoOp,
    Blocked(UserStoreRequestBlock0104),
}

/// Executes the clean GumPopup action against the typed UserStore state and
/// preserves the existing outbox command boundary. No packet semantics are
/// inferred here: all authoritative validation remains in the state methods.
pub fn apply_user_store_popup_action_0104(
    presentation: &mut UserStorePopupPresentation0104,
    state: &mut UserStoreUiState0104,
    authority: &UserStoreAuthority0104,
    outbox: &mut UserStoreUiOutbox0104,
) -> UserStoreActionOutcome0104 {
    let Some(popup) = presentation.popup else {
        return UserStoreActionOutcome0104::NoOp;
    };
    match popup.kind {
        UserStorePopupKind0104::RegisterQuantity => {
            let outcome =
                state.commit_register_quantity(authority, popup, presentation.value, outbox);
            if let UserStoreActionOutcome0104::Popup(next) = &outcome {
                presentation.open(*next);
            }
            outcome
        }
        UserStorePopupKind0104::RegisterPrice => {
            let value = presentation.value;
            let outcome = state.commit_register_price(authority, popup, value, outbox);
            if value > 0 {
                presentation.close();
            }
            outcome
        }
        UserStorePopupKind0104::Unregister | UserStorePopupKind0104::Buy => {
            let outcome = state.commit_store_popup(authority, popup, outbox);
            presentation.close();
            outcome
        }
    }
}

pub(super) fn user_store_popup_action_text(kind: UserStorePopupKind0104) -> LocalizedText {
    match kind {
        UserStorePopupKind0104::RegisterQuantity => {
            LocalizedText::new("ui.user_store.popup.add", "ADD")
        }
        UserStorePopupKind0104::RegisterPrice => {
            LocalizedText::new("ui.user_store.popup.taros", "Taros")
        }
        UserStorePopupKind0104::Unregister => {
            LocalizedText::new("ui.user_store.popup.remove", "REMOVE FROM STORE")
        }
        UserStorePopupKind0104::Buy => LocalizedText::new("ui.user_store.popup.buy", "BUY"),
    }
}
