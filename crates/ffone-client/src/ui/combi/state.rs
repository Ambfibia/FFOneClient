use super::*;

pub const COMBI_GAME_MODE_0104: i32 = 25;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum CombiSelectionSlot0104 {
    Style = COMBI_STYLE_SLOT_TYPE_0104,
    Stats = COMBI_STATS_SLOT_TYPE_0104,
}

impl CombiSelectionSlot0104 {
    #[must_use]
    pub const fn other(self) -> Self {
        match self {
            Self::Style => Self::Stats,
            Self::Stats => Self::Style,
        }
    }

    #[must_use]
    pub const fn wire_slot_type(self) -> i32 {
        self as i32
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CombiSelectionChange0104 {
    Detached {
        slot: CombiSelectionSlot0104,
        inventory_index: usize,
    },
    Attached {
        slot: CombiSelectionSlot0104,
        inventory_index: usize,
    },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CombiSelectionOverlay0104 {
    pub(super) style_inventory_index: Option<usize>,
    pub(super) stats_inventory_index: Option<usize>,
}

impl CombiSelectionOverlay0104 {
    #[must_use]
    pub const fn style_inventory_index(self) -> Option<usize> {
        self.style_inventory_index
    }

    #[must_use]
    pub const fn stats_inventory_index(self) -> Option<usize> {
        self.stats_inventory_index
    }

    #[must_use]
    pub const fn selected(self, slot: CombiSelectionSlot0104) -> Option<usize> {
        match slot {
            CombiSelectionSlot0104::Style => self.style_inventory_index,
            CombiSelectionSlot0104::Stats => self.stats_inventory_index,
        }
    }

    #[must_use]
    pub fn contains_inventory_index(self, index: usize) -> bool {
        self.style_inventory_index == Some(index) || self.stats_inventory_index == Some(index)
    }

    pub fn attach(
        &mut self,
        snapshot: &CombiAuthoritativeSnapshot0104,
        source_location: CombiSourceLocation0104,
        source_index: usize,
        destination: CombiSelectionSlot0104,
    ) -> Result<Vec<CombiSelectionChange0104>, CombiSelectionError0104> {
        match source_location {
            CombiSourceLocation0104::Equipment => {
                if source_index >= EQUIPMENT_SLOT_COUNT_0104 {
                    return Err(CombiSelectionError0104::OutOfBounds {
                        location: source_location,
                        index: source_index,
                        capacity: EQUIPMENT_SLOT_COUNT_0104,
                    });
                }
                return Err(CombiSelectionError0104::EquippedItem {
                    equipment_index: source_index,
                    system_message: COMBI_MESSAGE_260,
                });
            }
            CombiSourceLocation0104::Inventory => {
                if source_index >= INVENTORY_SLOT_COUNT_0104 {
                    return Err(CombiSelectionError0104::OutOfBounds {
                        location: source_location,
                        index: source_index,
                        capacity: INVENTORY_SLOT_COUNT_0104,
                    });
                }
            }
        }
        if CombiAuthoritativeSnapshot0104::item_is_empty(snapshot.inventory[source_index]) {
            return Err(CombiSelectionError0104::EmptyInventorySlot {
                inventory_index: source_index,
            });
        }
        if self.selected(destination) == Some(source_index) {
            return Ok(Vec::new());
        }

        let mut changes = Vec::with_capacity(3);
        // Clean moving-special-slot order: remove the moving overlay first.
        if self.selected(destination.other()) == Some(source_index) {
            self.detach_inner(destination.other(), &mut changes);
        }
        // Clean AttachCombiItem removes an occupied destination before attach.
        if self.selected(destination).is_some() {
            self.detach_inner(destination, &mut changes);
        }
        self.set(destination, Some(source_index));
        changes.push(CombiSelectionChange0104::Attached {
            slot: destination,
            inventory_index: source_index,
        });
        Ok(changes)
    }

    #[must_use]
    pub fn detach(&mut self, slot: CombiSelectionSlot0104) -> Vec<CombiSelectionChange0104> {
        let mut changes = Vec::with_capacity(1);
        self.detach_inner(slot, &mut changes);
        changes
    }

    /// Clean `ClearAll` always detaches slot 19 (Style) before slot 20 (Stats).
    #[must_use]
    pub fn clear_all(&mut self) -> Vec<CombiSelectionChange0104> {
        let mut changes = Vec::with_capacity(2);
        self.detach_inner(CombiSelectionSlot0104::Style, &mut changes);
        self.detach_inner(CombiSelectionSlot0104::Stats, &mut changes);
        changes
    }

    pub(super) fn detach_inner(
        &mut self,
        slot: CombiSelectionSlot0104,
        changes: &mut Vec<CombiSelectionChange0104>,
    ) {
        if let Some(inventory_index) = self.selected(slot) {
            self.set(slot, None);
            changes.push(CombiSelectionChange0104::Detached {
                slot,
                inventory_index,
            });
        }
    }

    pub(super) fn set(&mut self, slot: CombiSelectionSlot0104, value: Option<usize>) {
        match slot {
            CombiSelectionSlot0104::Style => self.style_inventory_index = value,
            CombiSelectionSlot0104::Stats => self.stats_inventory_index = value,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CombiSelectionError0104 {
    OutOfBounds {
        location: CombiSourceLocation0104,
        index: usize,
        capacity: usize,
    },
    EmptyInventorySlot {
        inventory_index: usize,
    },
    EquippedItem {
        equipment_index: usize,
        system_message: &'static str,
    },
}

impl fmt::Display for CombiSelectionError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfBounds {
                location,
                index,
                capacity,
            } => write!(
                f,
                "{location:?} Combi source index {index} is outside capacity {capacity}"
            ),
            Self::EmptyInventorySlot { inventory_index } => {
                write!(f, "inventory slot {inventory_index} is empty")
            }
            Self::EquippedItem {
                equipment_index, ..
            } => write!(
                f,
                "equipment slot {equipment_index} must be unequipped before Combi selection"
            ),
        }
    }
}

impl Error for CombiSelectionError0104 {}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CombiInventorySlotProjection0104 {
    pub item: Option<ItemBase0104>,
    pub icon_path: Option<String>,
    pub show_combined_badge: bool,
    pub hidden_by_selection_overlay: bool,
}

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct CombiModeProjection0104 {
    pub owner_pc_id: i32,
    pub taros: i32,
    pub selection: CombiSelectionOverlay0104,
    pub look: Option<CombiLookProjection0104>,
    pub stats: Option<CombiStatsProjection0104>,
    pub chance: CombiChance0104,
    pub cost: i32,
    /// Clean global `bCombiItemCannotEquip`, computed from the Stats item
    /// through `EnableEquipCombi` only after compatibility succeeds.
    pub combined_item_cannot_equip: bool,
    pub clear_enabled: bool,
    pub combine_enabled: bool,
    pub inventory: [CombiInventorySlotProjection0104; INVENTORY_SLOT_COUNT_0104],
    pub equipment: [CombiInventorySlotProjection0104; EQUIPMENT_SLOT_COUNT_0104],
}

impl Default for CombiModeProjection0104 {
    fn default() -> Self {
        Self {
            owner_pc_id: 0,
            taros: 0,
            selection: CombiSelectionOverlay0104::default(),
            look: None,
            stats: None,
            chance: CombiChance0104::NotReady,
            cost: 0,
            combined_item_cannot_equip: false,
            clear_enabled: false,
            combine_enabled: false,
            inventory: array::from_fn(|_| CombiInventorySlotProjection0104::default()),
            equipment: array::from_fn(|_| CombiInventorySlotProjection0104::default()),
        }
    }
}

pub(super) fn snapshot_inventory_item(
    snapshot: &CombiAuthoritativeSnapshot0104,
    index: usize,
) -> Result<ItemBase0104, CombiProjectionError0104> {
    if index >= INVENTORY_SLOT_COUNT_0104 {
        return Err(CombiProjectionError0104::SelectionOutOfBounds { index });
    }
    let item = snapshot.inventory[index];
    if CombiAuthoritativeSnapshot0104::item_is_empty(item) {
        return Err(CombiProjectionError0104::SelectedSlotEmpty { index });
    }
    Ok(item)
}

#[derive(Clone, Debug, PartialEq)]
pub enum CombiStateError0104 {
    InputBlocked {
        phase: CombiPhase0104,
    },
    Selection(CombiSelectionError0104),
    StaleProjection,
    CombinationNotReady,
    MissingPendingAttempt,
    InvalidModalChoice {
        phase: CombiPhase0104,
        choice: CombiModalChoice0104,
    },
    InvalidDelta {
        delta_seconds: f32,
    },
}

impl fmt::Display for CombiStateError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "clean Combi state transition rejected: {self:?}")
    }
}

impl Error for CombiStateError0104 {}
