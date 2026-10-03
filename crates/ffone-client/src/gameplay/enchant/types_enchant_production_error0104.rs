use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnchantPlayerAuthority0104 {
    pub owner_pc_id: i32,
    pub taros: i32,
    pub weapon_battery: i32,
    pub nano_battery: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnchantServiceSource0104 {
    pub runtime_npc_id: i32,
    pub table_npc_id: i32,
    pub npc_type: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnchantOpenContext0104 {
    pub source: EnchantServiceSource0104,
    pub player: EnchantPlayerAuthority0104,
    pub npc_button_type: i32,
    pub game_mode: i32,
    pub old_cursor_lock: bool,
}

impl EnchantOpenContext0104 {
    #[must_use]
    pub const fn clean(
        runtime_npc_id: i32,
        table_npc_id: i32,
        player: EnchantPlayerAuthority0104,
        old_cursor_lock: bool,
    ) -> Self {
        Self {
            source: EnchantServiceSource0104 {
                runtime_npc_id,
                table_npc_id,
                npc_type: ENCHANT_NPC_TYPE_0104,
            },
            player,
            npc_button_type: ENCHANT_NPC_BUTTON_TYPE_0104,
            game_mode: ENCHANT_GAME_MODE_0104,
            old_cursor_lock,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnchantAuthoritySnapshot0104 {
    pub(super) owner_pc_id: i32,
    pub(super) inventory: [ItemBase0104; INVENTORY_SLOT_COUNT_0104],
    pub(super) equipment: [ItemBase0104; EQUIPMENT_SLOT_COUNT_0104],
    pub(super) taros: i32,
    pub(super) weapon_battery: i32,
    pub(super) nano_battery: i32,
}

impl EnchantAuthoritySnapshot0104 {
    #[must_use]
    pub const fn owner_pc_id(&self) -> i32 {
        self.owner_pc_id
    }

    #[must_use]
    pub const fn inventory(&self) -> &[ItemBase0104; INVENTORY_SLOT_COUNT_0104] {
        &self.inventory
    }

    #[must_use]
    pub const fn equipment(&self) -> &[ItemBase0104; EQUIPMENT_SLOT_COUNT_0104] {
        &self.equipment
    }

    #[must_use]
    pub const fn taros(&self) -> i32 {
        self.taros
    }

    #[must_use]
    pub const fn weapon_battery(&self) -> i32 {
        self.weapon_battery
    }

    #[must_use]
    pub const fn nano_battery(&self) -> i32 {
        self.nano_battery
    }

    #[must_use]
    pub const fn item_is_empty(item: ItemBase0104) -> bool {
        item.item_id <= 0
    }
}

/// Main-owned item metadata boundary.  The controller verifies the item and
/// slot against its authority snapshot; this resolver supplies presentation
/// data only and cannot redefine wire identity or quantity.
pub trait EnchantPresentationCatalog0104 {
    fn presentation_for(&self, item: ItemBase0104) -> Option<EnchantItemPresentation0104>;
}

impl<F> EnchantPresentationCatalog0104 for F
where
    F: Fn(ItemBase0104) -> Option<EnchantItemPresentation0104>,
{
    fn presentation_for(&self, item: ItemBase0104) -> Option<EnchantItemPresentation0104> {
        self(item)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnchantOperation0104 {
    Enchant,
    Delete,
    Disassemble,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnchantAuthoritativeCommit0104 {
    pub(super) operation: EnchantOperation0104,
    pub(super) success_flag: Option<i32>,
    pub(super) inventory_writes: Vec<EnchantInventoryWrite0104>,
    pub(super) snapshot_after: EnchantAuthoritySnapshot0104,
}

impl EnchantAuthoritativeCommit0104 {
    #[must_use]
    pub const fn operation(&self) -> EnchantOperation0104 {
        self.operation
    }

    #[must_use]
    pub const fn success_flag(&self) -> Option<i32> {
        self.success_flag
    }

    pub fn inventory_writes(&self) -> impl Iterator<Item = EnchantInventoryWrite0104> + '_ {
        self.inventory_writes.iter().copied()
    }

    #[must_use]
    pub const fn snapshot_after(&self) -> &EnchantAuthoritySnapshot0104 {
        &self.snapshot_after
    }

    #[must_use]
    pub const fn taros_after(&self) -> i32 {
        self.snapshot_after.taros
    }

    #[must_use]
    pub const fn weapon_battery_after(&self) -> i32 {
        self.snapshot_after.weapon_battery
    }

    #[must_use]
    pub const fn nano_battery_after(&self) -> i32 {
        self.snapshot_after.nano_battery
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnchantModalChoice0104 {
    Accept,
    Dismiss,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnchantInputEffect0104 {
    SetCursorLock(bool),
    InventoryCloseDecisionRequested,
    InventoryCloseRejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnchantShellEffect0104 {
    ModeOpened(EnchantOpenContext0104),
    ModeClosed(EnchantOpenContext0104),
    Audio(EnchantAudioIntent0104),
    Animation(crate::enchant_ui::EnchantAnimationIntent0104),
    Camera(EnchantCameraIntent0104),
    Input(EnchantInputEffect0104),
    Lifecycle(EnchantLifecycleIntent0104),
    Popup(EnchantPopupIntent0104),
    RuntimeModal(EnchantRuntimeModal0104),
    Selection(EnchantSelectionIntent0104),
    DragCaptured {
        inventory_index: usize,
        available_quantity: i32,
    },
    HelpRequested,
    RefreshInventory,
    TransportFailure(EnchantFailure0104),
    AuxiliaryUnlock(u32),
    IgnoredSuccessFlag(i32),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EnchantProductionOutput0104 {
    pub request: Option<RegisteredGameplayRequest0104>,
    pub commit: Option<EnchantAuthoritativeCommit0104>,
    pub effects: Vec<EnchantShellEffect0104>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnchantGameplayFrame0104 {
    Passthrough(DecodedFrame),
    Malformed {
        frame: DecodedFrame,
        error: PayloadError,
    },
    Decoded {
        frame: DecodedFrame,
        packet: EnchantReplyPacket0104,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum EnchantProductionError0104 {
    AlreadyActive,
    NotActive,
    WrongNpcType {
        expected: i32,
        actual: i32,
    },
    WrongNpcButtonType {
        expected: i32,
        actual: i32,
    },
    WrongGameMode {
        expected: i32,
        actual: i32,
    },
    InventoryOwnerMismatch {
        expected_pc_id: i32,
        inventory_pc_id: i32,
    },
    PlayerOwnerMismatch {
        expected_pc_id: i32,
        player_pc_id: i32,
    },
    NegativeAuthority {
        field: &'static str,
        value: i32,
    },
    AuthorityRefreshBlocked {
        phase: EnchantPhase0104,
    },
    SelectedItemChanged {
        inventory_index: usize,
        expected: ItemBase0104,
        actual: ItemBase0104,
    },
    DragSourceOutOfBounds {
        inventory_index: usize,
    },
    EmptyDragSource {
        inventory_index: usize,
    },
    InvalidStackQuantity {
        inventory_index: usize,
        quantity: i32,
    },
    DragSourceAlreadyReserved {
        inventory_index: usize,
    },
    MissingPresentation {
        item_type: i16,
        item_id: i16,
    },
    MissingDragSource,
    StaleDragSource {
        inventory_index: usize,
        expected: ItemBase0104,
        actual: ItemBase0104,
    },
    ReservationSourceOutOfBounds {
        source_slot: i32,
    },
    ReservationSlotAlreadyOccupied {
        slot: EnchantAttachmentSlot0104,
    },
    ReservationMissing {
        slot: EnchantAttachmentSlot0104,
    },
    ReservationSourceMismatch {
        slot: EnchantAttachmentSlot0104,
        expected: i32,
        actual: i32,
    },
    ReservationQuantityInvalid {
        slot: EnchantAttachmentSlot0104,
        quantity: i32,
    },
    ReservationExceedsAuthority {
        inventory_index: usize,
        available: i32,
        requested: i32,
    },
    RequestAlreadyPending {
        operation: EnchantOperation0104,
    },
    NoPendingRequest {
        reply: EnchantOperation0104,
    },
    UnexpectedReply {
        pending: EnchantOperation0104,
        reply: EnchantOperation0104,
    },
    UnsupportedAuxiliaryReply {
        packet_type: u32,
    },
    ReplyIdentityMismatch {
        field: &'static str,
        expected: i32,
        actual: i32,
    },
    ReplySlotOutOfBounds {
        field: &'static str,
        value: i32,
    },
    ConflictingReplyWrites {
        inventory_index: usize,
        first: ItemBase0104,
        second: ItemBase0104,
    },
    MalformedReplyItem {
        inventory_index: usize,
        item_type: i16,
        item_id: i16,
    },
    UnexpectedAuthoritativeReceipt,
    MissingAuthoritativeReceipt,
    AuthoritativeReceiptMismatch,
    MultipleOutboundRequests,
    WrongWirePacket {
        expected: u32,
        actual: u32,
    },
    WrongRequestToken {
        expected: u64,
        actual: u64,
    },
    InvalidRequestSlots {
        detail: &'static str,
    },
    RuntimeModalAlreadyOpen,
    NoModalToResolve,
    CloseDecisionAlreadyPending,
    NoCloseDecisionPending,
    InputBlockedByRuntime,
    RequestRegistration(RegisteredGameplayRequestError0104),
    Model(EnchantModelError0104),
    MalformedFrame {
        frame: DecodedFrame,
        error: PayloadError,
    },
}

impl fmt::Display for EnchantProductionError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "clean Enchant production transition rejected: {self:?}"
        )
    }
}

impl Error for EnchantProductionError0104 {}

impl From<EnchantModelError0104> for EnchantProductionError0104 {
    fn from(value: EnchantModelError0104) -> Self {
        Self::Model(value)
    }
}

impl From<RegisteredGameplayRequestError0104> for EnchantProductionError0104 {
    fn from(value: RegisteredGameplayRequestError0104) -> Self {
        Self::RequestRegistration(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct EnchantDrag0104 {
    pub(super) inventory_index: usize,
    pub(super) authority_item: ItemBase0104,
    pub(super) selectable: EnchantSelectableItem0104,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct EnchantReservation0104 {
    pub(super) source_slot: i32,
    pub(super) quantity: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PendingEnchant0104 {
    pub(super) request_token: u64,
    pub(super) request: EnchantRequest0104,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PendingDelete0104 {
    pub(super) inventory_index: usize,
    pub(super) item_before: ItemBase0104,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum PendingRequest0104 {
    Enchant(PendingEnchant0104),
    Delete(PendingDelete0104),
}

impl PendingRequest0104 {
    pub(super) const fn operation(&self) -> EnchantOperation0104 {
        match self {
            Self::Enchant(_) => EnchantOperation0104::Enchant,
            Self::Delete(_) => EnchantOperation0104::Delete,
        }
    }
}

#[derive(Clone, Debug)]
pub struct EnchantProductionSession0104 {
    pub(super) context: EnchantOpenContext0104,
    pub(super) model: EnchantModeModel0104,
    pub(super) snapshot: EnchantAuthoritySnapshot0104,
    pub(super) drag: Option<EnchantDrag0104>,
    pub(super) reserved_by_source: [i32; INVENTORY_SLOT_COUNT_0104],
    pub(super) reservations_by_attachment: [Option<EnchantReservation0104>; 5],
    pub(super) runtime_modal: Option<EnchantRuntimeModal0104>,
    pub(super) close_decision_pending: bool,
    pub(super) pending_request: Option<PendingRequest0104>,
}

impl EnchantProductionSession0104 {
    #[must_use]
    pub const fn context(&self) -> EnchantOpenContext0104 {
        self.context
    }

    #[must_use]
    pub const fn model(&self) -> &EnchantModeModel0104 {
        &self.model
    }

    #[must_use]
    pub const fn snapshot(&self) -> &EnchantAuthoritySnapshot0104 {
        &self.snapshot
    }

    #[must_use]
    pub fn pending_operation(&self) -> Option<EnchantOperation0104> {
        self.pending_request
            .as_ref()
            .map(PendingRequest0104::operation)
    }

    #[must_use]
    pub const fn runtime_modal(&self) -> Option<&EnchantRuntimeModal0104> {
        self.runtime_modal.as_ref()
    }

    #[must_use]
    pub const fn close_decision_pending(&self) -> bool {
        self.close_decision_pending
    }

    #[must_use]
    pub fn inventory_slot_reserved(&self, inventory_index: usize) -> bool {
        self.reserved_by_source
            .get(inventory_index)
            .is_some_and(|quantity| *quantity > 0)
    }
}

#[derive(Clone, Debug, Default, Resource)]
pub struct EnchantProductionRuntime0104 {
    pub(super) session: Option<EnchantProductionSession0104>,
}
