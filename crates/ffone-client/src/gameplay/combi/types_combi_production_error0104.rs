use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CombiPlayerAuthority0104 {
    pub owner_pc_id: i32,
    pub gender: i32,
    pub level: i32,
    pub guide: i32,
    pub taros: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CombiServiceSource0104 {
    /// Runtime NPC instance that owned the reachable NpcIcon service menu.
    pub runtime_npc_id: i32,
    /// Table NPC row backing that runtime instance.
    pub table_npc_id: i32,
    /// Clean `NpcIconMode` exposes Combine only for type 26.
    pub npc_type: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CombiOpenContext0104 {
    pub source: CombiServiceSource0104,
    pub player: CombiPlayerAuthority0104,
    pub npc_button_type: i32,
    pub game_mode: i32,
}

impl CombiOpenContext0104 {
    #[must_use]
    pub const fn clean(
        runtime_npc_id: i32,
        table_npc_id: i32,
        player: CombiPlayerAuthority0104,
    ) -> Self {
        Self {
            source: CombiServiceSource0104 {
                runtime_npc_id,
                table_npc_id,
                npc_type: COMBI_NPC_TYPE_0104,
            },
            player,
            npc_button_type: COMBI_NPC_BUTTON_TYPE_0104,
            game_mode: COMBI_GAME_MODE_0104,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CombiNpcAnimation0104 {
    MakingInOut,
    Stand,
}

impl CombiNpcAnimation0104 {
    #[must_use]
    pub const fn legacy_event_name(self) -> Option<&'static str> {
        match self {
            Self::MakingInOut => Some(COMBI_MAKING_ANIMATION_EVENT_0104),
            Self::Stand => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CombiCloseReason0104 {
    Close,
    GoToMyStuff,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CombiSystemMessage0104 {
    Modal {
        modal: CombiSystemModal0104,
        style_item: Option<ItemBase0104>,
        stats_item: Option<ItemBase0104>,
    },
    EquippedItem {
        message_id: i32,
        equipment_index: usize,
        item: ItemBase0104,
    },
    WireFailure {
        error_code: i32,
    },
}

impl CombiSystemMessage0104 {
    #[must_use]
    pub const fn message_id(self) -> Option<i32> {
        match self {
            Self::Modal { modal, .. } => Some(modal.message_id()),
            Self::EquippedItem { message_id, .. } => Some(message_id),
            Self::WireFailure { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CombiShellEffect0104 {
    ModeOpened {
        context: CombiOpenContext0104,
        lease: CombiModeLease0104,
    },
    DragCaptured {
        inventory_index: usize,
    },
    SelectionChanged(CombiSelectionChange0104),
    SystemMessage(CombiSystemMessage0104),
    NpcAnimation {
        npc_id: i32,
        animation: CombiNpcAnimation0104,
    },
    PlaySuccessSound {
        true_name: &'static str,
    },
    HelpRequested,
    ModeClosed {
        context: CombiOpenContext0104,
        reason: CombiCloseReason0104,
    },
    /// Semantic boundary for the clean event chain that exits Combi, opens
    /// My Stuff mode 6, checks first-use condition 3, and refreshes inventory.
    GoToMyStuffRequested {
        game_mode: i32,
        first_use_condition: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CombiAuthoritativeCommit0104 {
    pub(super) receipt: CombiAuthorityReceipt0104,
    pub(super) inventory_writes: [Option<CombiInventoryWrite0104>; 2],
    pub(super) taros_after: i32,
    pub(super) snapshot_after: CombiAuthoritativeSnapshot0104,
}

impl CombiAuthoritativeCommit0104 {
    #[must_use]
    pub const fn receipt(&self) -> &CombiAuthorityReceipt0104 {
        &self.receipt
    }

    #[must_use]
    pub const fn owner_pc_id(&self) -> i32 {
        self.snapshot_after.owner_pc_id
    }

    #[must_use]
    pub const fn taros_after(&self) -> i32 {
        self.taros_after
    }

    #[must_use]
    pub const fn snapshot_after(&self) -> &CombiAuthoritativeSnapshot0104 {
        &self.snapshot_after
    }

    pub fn inventory_writes(&self) -> impl Iterator<Item = CombiInventoryWrite0104> + '_ {
        self.inventory_writes.iter().flatten().copied()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CombiProductionOutput0104 {
    pub request: Option<RegisteredGameplayRequest0104>,
    pub commit: Option<CombiAuthoritativeCommit0104>,
    pub effects: Vec<CombiShellEffect0104>,
}

impl CombiProductionOutput0104 {
    pub(super) fn from_effect(effect: CombiShellEffect0104) -> Self {
        Self {
            effects: vec![effect],
            ..Self::default()
        }
    }

    pub(super) fn add_selection_changes(&mut self, changes: Vec<CombiSelectionChange0104>) {
        self.effects.extend(
            changes
                .into_iter()
                .map(CombiShellEffect0104::SelectionChanged),
        );
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CombiGameplayFrame0104 {
    Passthrough(DecodedFrame),
    Malformed {
        frame: DecodedFrame,
        error: PayloadError,
    },
    Decoded {
        frame: DecodedFrame,
        packet: CombiReplyPacket0104,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum CombiProductionError0104 {
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
    NegativeTaros {
        taros: i32,
    },
    AuthorityRefreshBlocked {
        phase: CombiPhase0104,
    },
    SelectedItemChanged {
        slot: CombiSelectionSlot0104,
        inventory_index: usize,
        expected: ItemBase0104,
        actual: ItemBase0104,
    },
    MissingDragSource,
    DragSourceOutOfBounds {
        inventory_index: usize,
    },
    EmptyDragSource {
        inventory_index: usize,
    },
    DragSourceAlreadySelected {
        inventory_index: usize,
    },
    WrongEquippedMessageId {
        expected: i32,
        actual: i32,
    },
    EquipmentSlotOutOfBounds {
        equipment_index: usize,
    },
    EmptyEquipmentSlot {
        equipment_index: usize,
    },
    RequestAlreadyPending,
    NoPendingRequest,
    State(CombiStateError0104),
    Projection(CombiProjectionError0104),
    Reply(CombiReplyError0104),
    Receipt(CombiReceiptCommitError0104),
    RequestRegistration(RegisteredGameplayRequestError0104),
    MalformedFrame {
        frame: DecodedFrame,
        error: PayloadError,
    },
}

impl fmt::Display for CombiProductionError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "clean Combi production transition rejected: {self:?}"
        )
    }
}

impl Error for CombiProductionError0104 {}

impl From<CombiStateError0104> for CombiProductionError0104 {
    fn from(value: CombiStateError0104) -> Self {
        Self::State(value)
    }
}

impl From<CombiProjectionError0104> for CombiProductionError0104 {
    fn from(value: CombiProjectionError0104) -> Self {
        Self::Projection(value)
    }
}

impl From<CombiReplyError0104> for CombiProductionError0104 {
    fn from(value: CombiReplyError0104) -> Self {
        Self::Reply(value)
    }
}

impl From<CombiReceiptCommitError0104> for CombiProductionError0104 {
    fn from(value: CombiReceiptCommitError0104) -> Self {
        Self::Receipt(value)
    }
}

impl From<RegisteredGameplayRequestError0104> for CombiProductionError0104 {
    fn from(value: RegisteredGameplayRequestError0104) -> Self {
        Self::RequestRegistration(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CombiProductionSession0104 {
    pub(super) context: CombiOpenContext0104,
    pub(super) lease: CombiModeLease0104,
    pub(super) machine: CombiMachine0104,
    pub(super) snapshot: CombiAuthoritativeSnapshot0104,
    pub(super) projection: CombiModeProjection0104,
    pub(super) drag_source_inventory_index: Option<usize>,
    pub(super) pending_request: Option<CombiRequest0104>,
}

impl CombiProductionSession0104 {
    #[must_use]
    pub const fn context(&self) -> CombiOpenContext0104 {
        self.context
    }

    #[must_use]
    pub const fn lease(&self) -> CombiModeLease0104 {
        self.lease
    }

    #[must_use]
    pub const fn machine(&self) -> &CombiMachine0104 {
        &self.machine
    }

    #[must_use]
    pub const fn snapshot(&self) -> &CombiAuthoritativeSnapshot0104 {
        &self.snapshot
    }

    #[must_use]
    pub const fn projection(&self) -> &CombiModeProjection0104 {
        &self.projection
    }

    #[must_use]
    pub const fn drag_source_inventory_index(&self) -> Option<usize> {
        self.drag_source_inventory_index
    }

    #[must_use]
    pub const fn pending_request(&self) -> Option<CombiRequest0104> {
        self.pending_request
    }
}

#[derive(Clone, Debug, Default, Resource)]
pub struct CombiProductionRuntime0104 {
    pub(super) session: Option<CombiProductionSession0104>,
}
