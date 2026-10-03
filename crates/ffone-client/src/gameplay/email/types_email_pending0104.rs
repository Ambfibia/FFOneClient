use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EmailPending0104 {
    UpdateCheck,
    Read {
        email_index: i64,
    },
    PageList {
        page: i8,
    },
    Delete {
        email_indices: [i64; EMAIL_DELETE_BATCH_COUNT],
    },
    Send {
        recipient_pc_uid: i64,
    },
    ReceiveItem {
        email_index: i64,
        inventory_slot: i32,
        email_item_slot: i32,
    },
    ReceiveCash {
        email_index: i64,
    },
    ReceiveAllItems {
        email_index: i64,
    },
}

impl EmailPending0104 {
    #[must_use]
    pub fn from_request(request: &EmailRequest) -> Self {
        match request {
            EmailRequest::UpdateCheck => Self::UpdateCheck,
            EmailRequest::Read { email_index } => Self::Read {
                email_index: *email_index,
            },
            EmailRequest::PageList { page } => Self::PageList { page: *page },
            EmailRequest::Delete { email_indices } => Self::Delete {
                email_indices: *email_indices,
            },
            EmailRequest::Send {
                recipient_pc_uid, ..
            } => Self::Send {
                recipient_pc_uid: *recipient_pc_uid,
            },
            EmailRequest::ReceiveItem {
                email_index,
                inventory_slot,
                email_item_slot,
            } => Self::ReceiveItem {
                email_index: *email_index,
                inventory_slot: *inventory_slot,
                email_item_slot: *email_item_slot,
            },
            EmailRequest::ReceiveCash { email_index } => Self::ReceiveCash {
                email_index: *email_index,
            },
            EmailRequest::ReceiveAllItems { email_index } => Self::ReceiveAllItems {
                email_index: *email_index,
            },
        }
    }

    pub(super) fn accepts(&self, reply: &EmailReply) -> bool {
        match (self, reply) {
            (Self::UpdateCheck, EmailReply::NewEmail { .. }) => true,
            (
                Self::Read { email_index },
                EmailReply::ReadSuccess(EmailReadMessage {
                    email_index: actual,
                    ..
                })
                | EmailReply::ReadFailure {
                    email_index: actual,
                    ..
                },
            ) => email_index == actual,
            (
                Self::PageList { page },
                EmailReply::PageListSuccess { page: actual, .. }
                | EmailReply::PageListFailure { page: actual, .. },
            ) => page == actual,
            (
                Self::Delete { email_indices },
                EmailReply::DeleteSuccess {
                    email_indices: actual,
                }
                | EmailReply::DeleteFailure {
                    email_indices: actual,
                    ..
                },
            ) => email_indices == actual,
            (
                Self::Send { recipient_pc_uid },
                EmailReply::SendSuccess {
                    recipient_pc_uid: actual,
                    ..
                }
                | EmailReply::SendFailure {
                    recipient_pc_uid: actual,
                    ..
                },
            ) => recipient_pc_uid == actual,
            (
                Self::ReceiveItem {
                    email_index,
                    inventory_slot,
                    email_item_slot,
                },
                EmailReply::ReceiveItemSuccess {
                    email_index: actual_index,
                    inventory_slot: actual_inventory,
                    email_item_slot: actual_item,
                }
                | EmailReply::ReceiveItemFailure {
                    email_index: actual_index,
                    inventory_slot: actual_inventory,
                    email_item_slot: actual_item,
                    ..
                },
            ) => {
                email_index == actual_index
                    && inventory_slot == actual_inventory
                    && email_item_slot == actual_item
            }
            (
                Self::ReceiveCash { email_index },
                EmailReply::ReceiveCashSuccess {
                    email_index: actual,
                    ..
                }
                | EmailReply::ReceiveCashFailure {
                    email_index: actual,
                    ..
                },
            )
            | (
                Self::ReceiveAllItems { email_index },
                EmailReply::ReceiveAllItemsSuccess {
                    email_index: actual,
                }
                | EmailReply::ReceiveAllItemsFailure {
                    email_index: actual,
                    ..
                },
            ) => email_index == actual,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EmailTransportRuntime0104 {
    pub(super) pending: Option<EmailPending0104>,
}

impl EmailTransportRuntime0104 {
    #[must_use]
    pub const fn pending(&self) -> Option<&EmailPending0104> {
        self.pending.as_ref()
    }

    pub fn begin(
        &mut self,
        request: &EmailRequest,
    ) -> Result<EmailWirePacket0104, EmailRuntimeError0104> {
        if let Some(pending) = self.pending.clone() {
            return Err(EmailRuntimeError0104::Busy { pending });
        }
        let packet = encode_email_request_0104(request)?;
        self.pending = Some(EmailPending0104::from_request(request));
        Ok(packet)
    }

    pub fn cancel_pending(&mut self) -> Option<EmailPending0104> {
        self.pending.take()
    }

    pub fn accept(
        &mut self,
        packet_id: u32,
        body: &[u8],
    ) -> Result<Option<EmailRuntimeDelivery0104>, EmailRuntimeError0104> {
        let Some(reply) = decode_email_reply_0104(packet_id, body)? else {
            return Ok(None);
        };

        if matches!(reply, EmailReply::NewEmail { .. }) {
            if self.pending == Some(EmailPending0104::UpdateCheck) {
                self.pending = None;
                return Ok(Some(EmailRuntimeDelivery0104::Correlated(reply)));
            }
            return Ok(Some(EmailRuntimeDelivery0104::UnsolicitedNewEmail(reply)));
        }

        let Some(pending) = self.pending.as_ref() else {
            return Err(EmailRuntimeError0104::ReplyWithoutPendingRequest { reply });
        };
        if !pending.accepts(&reply) {
            return Err(EmailRuntimeError0104::ReplyDoesNotMatchPending {
                pending: pending.clone(),
                reply,
            });
        }

        self.pending = None;
        Ok(Some(EmailRuntimeDelivery0104::Correlated(reply)))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EmailItemFeaturePolicy0104 {
    /// Clean `localized.combine == 1` branch.
    pub combine_enabled: bool,
    /// Clean `localized.korEnchant == 1` branch.
    pub korean_enchant_enabled: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmailPlayerAuthority0104 {
    pub owner_pc_id: i32,
    pub taros: i32,
    pub current_local_time: EmailSystemTime,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmailOpenContext0104 {
    pub player: EmailPlayerAuthority0104,
    /// `EmailMode.ReceiveInit` saves this bit and `Exit` restores it.
    pub cursor_was_locked: bool,
    pub item_policy: EmailItemFeaturePolicy0104,
}

pub trait EmailItemCatalog0104 {
    fn resolve(&self, item: ItemBase0104) -> Option<EmailItemCatalogMetadata0104>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmailAttachmentRejection0104 {
    Chest,
    UnsupportedItemType,
    GeneralEmailSubtype,
    NotTradeable,
    CombinedLook,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmailAuthoritySource0104 {
    SendSuccess,
    ReceiveItemSuccess {
        email_index: i64,
        inventory_slot: i32,
        email_item_slot: i32,
    },
    ReceiveCashSuccess {
        email_index: i64,
    },
    ReceiveAllItemsSuccess {
        email_index: i64,
    },
}

/// Authority produced by one correlated Email reply.
///
/// Receive-item replies contain no `sItemBase`; OpenFusion sends separate
/// registered `PC_GIVE_ITEM_SUCC` frames. Those replies therefore set
/// `inventory_refresh_required` and intentionally carry no guessed write.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailAuthoritativeCommit0104 {
    pub(super) owner_pc_id: i32,
    pub(super) source: EmailAuthoritySource0104,
    pub(super) inventory_writes: [Option<EmailInventoryWrite0104>; EMAIL_ATTACHMENT_COUNT],
    pub(super) taros_after: Option<i32>,
    pub(super) inventory_refresh_required: bool,
}

impl EmailAuthoritativeCommit0104 {
    #[must_use]
    pub const fn owner_pc_id(&self) -> i32 {
        self.owner_pc_id
    }

    #[must_use]
    pub const fn source(&self) -> EmailAuthoritySource0104 {
        self.source
    }

    pub fn inventory_writes(&self) -> impl Iterator<Item = EmailInventoryWrite0104> + '_ {
        self.inventory_writes.iter().flatten().copied()
    }

    #[must_use]
    pub const fn taros_after(&self) -> Option<i32> {
        self.taros_after
    }

    #[must_use]
    pub const fn inventory_refresh_required(&self) -> bool {
        self.inventory_refresh_required
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EmailProductionOutput0104 {
    pub request: Option<RegisteredGameplayRequest0104>,
    pub commit: Option<EmailAuthoritativeCommit0104>,
    pub actions: Vec<EmailUiAction>,
    pub audio: Vec<EmailUiAudioCue>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EmailProductionError0104 {
    AlreadyActive,
    NotActive,
    ModelAlreadyVisible,
    DirtyTransportBoundary,
    DirtyEffectBoundary,
    InventoryOwnerMismatch {
        expected_pc_id: i32,
        inventory_pc_id: i32,
    },
    NegativeTaros {
        taros: i32,
    },
    TooManyBuddies {
        actual: usize,
        maximum: usize,
    },
    InvalidBuddy {
        index: usize,
        pc_uid: i64,
    },
    DuplicateBuddy {
        first_index: usize,
        second_index: usize,
        pc_uid: i64,
    },
    MalformedInventoryItem {
        inventory_slot: usize,
        item: ItemBase0104,
    },
    MissingCatalogItem {
        inventory_slot: usize,
        item: ItemBase0104,
    },
    InvalidCatalogIcon {
        inventory_slot: usize,
        icon_path: String,
    },
    MissingTradeMetadata {
        inventory_slot: usize,
        item: ItemBase0104,
    },
    MissingGeneralSubtype {
        inventory_slot: usize,
        item: ItemBase0104,
    },
    AttachmentRejected {
        inventory_slot: usize,
        item: ItemBase0104,
        reason: EmailAttachmentRejection0104,
    },
    AttachmentSourceChanged {
        inventory_slot: usize,
        expected: EmailWireItem,
        actual: ItemBase0104,
    },
    InventoryAuthorityStale,
    TarosProjectionMismatch {
        authoritative: i32,
        projected: i32,
    },
    RequestNotReachable {
        request: EmailRequest,
    },
    RequestAlreadyPending {
        request: EmailRequest,
    },
    TransportStateMismatch {
        controller: Option<EmailPending0104>,
        transport: Option<EmailPending0104>,
    },
    NoPendingRequest,
    ReplyDoesNotMatchRequest {
        request: EmailRequest,
        reply: EmailReply,
    },
    NegativeAuthoritativeTaros {
        taros: i32,
    },
    MissingSendCommitAction,
    UnexpectedSendCommitAction,
    CloseGateSourceMismatch,
    Transport(EmailRuntimeError0104),
    Registration(RegisteredGameplayRequestError0104),
}

impl fmt::Display for EmailProductionError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "clean EmailMode production transition rejected: {self:?}"
        )
    }
}

impl Error for EmailProductionError0104 {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Transport(error) => Some(error),
            Self::Registration(error) => Some(error),
            _ => None,
        }
    }
}

impl From<EmailRuntimeError0104> for EmailProductionError0104 {
    fn from(value: EmailRuntimeError0104) -> Self {
        Self::Transport(value)
    }
}

impl From<RegisteredGameplayRequestError0104> for EmailProductionError0104 {
    fn from(value: RegisteredGameplayRequestError0104) -> Self {
        Self::Registration(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailProductionSession0104 {
    pub(super) context: EmailOpenContext0104,
    pub(super) lease: EmailModeLease0104,
    pub(super) inventory: EmailInventoryAuthority0104,
    pub(super) inventory_projection_stale: bool,
    pub(super) close_source: Option<EmailCloseSource>,
}

impl EmailProductionSession0104 {
    #[must_use]
    pub const fn context(&self) -> EmailOpenContext0104 {
        self.context
    }

    #[must_use]
    pub const fn lease(&self) -> EmailModeLease0104 {
        self.lease
    }

    #[must_use]
    pub const fn inventory(&self) -> &EmailInventoryAuthority0104 {
        &self.inventory
    }

    #[must_use]
    pub const fn inventory_projection_stale(&self) -> bool {
        self.inventory_projection_stale
    }
}

#[derive(Clone, Debug, Default, Resource)]
pub struct EmailProductionRuntime0104 {
    pub(super) session: Option<EmailProductionSession0104>,
    pub(super) pending_request: Option<EmailRequest>,
}
