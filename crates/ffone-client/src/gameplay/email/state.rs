use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EmailRuntimeDelivery0104 {
    Correlated(EmailReply),
    /// `P_FE2CL_REP_PC_NEW_EMAIL` is also a push notification and therefore
    /// remains valid while no update-check request is pending.
    UnsolicitedNewEmail(EmailReply),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EmailRuntimeError0104 {
    Busy {
        pending: EmailPending0104,
    },
    Encode(EmailEncodeError0104),
    Decode(EmailDecodeError0104),
    ReplyWithoutPendingRequest {
        reply: EmailReply,
    },
    ReplyDoesNotMatchPending {
        pending: EmailPending0104,
        reply: EmailReply,
    },
}

impl fmt::Display for EmailRuntimeError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Busy { pending } => {
                write!(
                    formatter,
                    "EmailMode already has a pending request: {pending:?}"
                )
            }
            Self::Encode(error) => error.fmt(formatter),
            Self::Decode(error) => error.fmt(formatter),
            Self::ReplyWithoutPendingRequest { reply } => {
                write!(formatter, "EmailMode received a stray reply: {reply:?}")
            }
            Self::ReplyDoesNotMatchPending { pending, reply } => write!(
                formatter,
                "EmailMode reply {reply:?} does not match pending request {pending:?}"
            ),
        }
    }
}

impl Error for EmailRuntimeError0104 {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Encode(error) => Some(error),
            Self::Decode(error) => Some(error),
            _ => None,
        }
    }
}

impl From<EmailEncodeError0104> for EmailRuntimeError0104 {
    fn from(value: EmailEncodeError0104) -> Self {
        Self::Encode(value)
    }
}

impl From<EmailDecodeError0104> for EmailRuntimeError0104 {
    fn from(value: EmailDecodeError0104) -> Self {
        Self::Decode(value)
    }
}

/// Clean `Panel_PCStuffScript.ActiveInventoryTab` value on EmailMode open.
pub const EMAIL_ACTIVE_INVENTORY_TAB_0104: i32 = 0;

/// Clean inventory event value while EmailMode owns the item panel.
pub const EMAIL_INVENTORY_MAIL_MODE_OPEN_0104: i32 = 4;

/// Clean inventory event value restored by `EmailMode.Exit`.
pub const EMAIL_INVENTORY_MAIL_MODE_CLOSED_0104: i32 = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmailModeLease0104 {
    pub game_mode: u8,
    pub active_inventory_tab: i32,
    pub inventory_mail_mode: i32,
    pub blocks_gameplay_input: bool,
    pub cursor_locked_during_mode: bool,
    pub cursor_locked_after_exit: bool,
    pub normal_exit_sends_packet: bool,
}

impl From<EmailOpenContext0104> for EmailModeLease0104 {
    fn from(context: EmailOpenContext0104) -> Self {
        Self {
            game_mode: EMAIL_UI_GAME_MODE_VALUE,
            active_inventory_tab: EMAIL_ACTIVE_INVENTORY_TAB_0104,
            inventory_mail_mode: EMAIL_INVENTORY_MAIL_MODE_OPEN_0104,
            blocks_gameplay_input: true,
            cursor_locked_during_mode: false,
            cursor_locked_after_exit: context.cursor_was_locked,
            normal_exit_sends_packet: EMAIL_NORMAL_EXIT_SENDS_PACKET_0104,
        }
    }
}

/// Exact local-player general-inventory feed. The owner and all fifty raw
/// `sItemBase` records remain available for request validation; empty records
/// are deliberately not canonicalized.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailInventoryAuthority0104 {
    pub(super) owner_pc_id: i32,
    pub(super) slots: [ItemBase0104; EMAIL_INVENTORY_SLOT_COUNT],
}

impl EmailInventoryAuthority0104 {
    #[must_use]
    pub fn from_inventory_runtime(runtime: &InventoryRuntime0104) -> Self {
        Self {
            owner_pc_id: runtime.owner_pc_id(),
            slots: *runtime.inventory(),
        }
    }

    #[must_use]
    pub const fn owner_pc_id(&self) -> i32 {
        self.owner_pc_id
    }

    #[must_use]
    pub const fn slots(&self) -> &[ItemBase0104; EMAIL_INVENTORY_SLOT_COUNT] {
        &self.slots
    }

    #[must_use]
    pub const fn slot(&self, index: usize) -> Option<ItemBase0104> {
        if index < EMAIL_INVENTORY_SLOT_COUNT {
            Some(self.slots[index])
        } else {
            None
        }
    }

    #[must_use]
    pub fn free_slot_count(&self) -> usize {
        self.slots
            .iter()
            .filter(|item| InventoryRuntime0104::item_is_empty(**item))
            .count()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmailInventoryWrite0104 {
    pub inventory_slot: usize,
    pub item: ItemBase0104,
}

#[must_use]
pub fn email_inventory_authority_0104(
    runtime: &InventoryRuntime0104,
) -> EmailInventoryAuthority0104 {
    EmailInventoryAuthority0104::from_inventory_runtime(runtime)
}
