use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CombiRgb0104 {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl CombiRgb0104 {
    #[must_use]
    pub const fn new(red: f32, green: f32, blue: f32) -> Self {
        Self { red, green, blue }
    }

    #[must_use]
    pub(super) fn bevy(self) -> Color {
        Color::srgb(self.red, self.green, self.blue)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CombiSourceLocation0104 {
    Inventory,
    Equipment,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CombiItemMetadata0104 {
    pub name: String,
    pub description: String,
    pub minimum_level: i32,
    pub required_gender: i32,
    pub rarity: i32,
    pub rarity_label: String,
    pub mentor: i32,
    pub cashable: i32,
    pub item_price: i32,
    pub point_rating: i32,
    pub group_rating: i32,
    pub defense_rating: i32,
    pub delay_time: i32,
    pub equip_type: i32,
    pub target_mode: i32,
    pub type_label: String,
    pub trade_label: String,
    pub icon_path: Option<String>,
}

pub trait CombiItemCatalog0104 {
    /// Exact TableData row for `(sItemType, item ID)`.
    fn resolve(&self, item_type: i16, item_id: i16) -> Option<CombiItemMetadata0104>;

    /// The bag includes non-combinable items too; their art is independent
    /// of the four equipment tables admitted by the recipe validator.
    fn resolve_icon(&self, item_type: i16, item_id: i16) -> Option<String> {
        self.resolve(item_type, item_id).and_then(|row| row.icon_path)
    }

    /// Clean `EnableEquip`, used only for the red frame on each selected item.
    fn enable_equip(&self, _item: ItemBase0104) -> Option<bool> {
        None
    }

    /// Clean `EnableEquipCombi`. `None` is kept fail-closed as a red frame,
    /// but is deliberately not promoted to a combination-blocking error.
    fn enable_equip_combi(&self, _item: ItemBase0104) -> Option<bool> {
        None
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CombiAuthoritativeSnapshot0104 {
    pub owner_pc_id: i32,
    pub gender: i32,
    pub level: i32,
    pub guide: i32,
    pub taros: i32,
    pub equipment: [ItemBase0104; EQUIPMENT_SLOT_COUNT_0104],
    pub inventory: [ItemBase0104; INVENTORY_SLOT_COUNT_0104],
}

impl CombiAuthoritativeSnapshot0104 {
    #[must_use]
    pub fn from_inventory_runtime(
        runtime: &InventoryRuntime0104,
        gender: i32,
        level: i32,
        guide: i32,
        taros: i32,
    ) -> Self {
        Self {
            owner_pc_id: runtime.owner_pc_id(),
            gender,
            level,
            guide,
            taros,
            equipment: *runtime.equipment(),
            inventory: *runtime.inventory(),
        }
    }

    #[must_use]
    pub const fn item_is_empty(item: ItemBase0104) -> bool {
        item.item_id <= 0
    }

    pub fn apply_authoritative_receipt(
        &mut self,
        receipt: &CombiAuthorityReceipt0104,
    ) -> Result<(), CombiReceiptCommitError0104> {
        match receipt {
            CombiAuthorityReceipt0104::Success {
                owner_pc_id,
                style_slot,
                stats_slot,
                style_before,
                stats_before,
                style_after,
                stats_after,
                taros_after,
            } => {
                if self.owner_pc_id != *owner_pc_id {
                    return Err(CombiReceiptCommitError0104::OwnerMismatch {
                        expected: *owner_pc_id,
                        actual: self.owner_pc_id,
                    });
                }
                self.validate_receipt_preconditions(
                    *style_slot,
                    *stats_slot,
                    *style_before,
                    *stats_before,
                )?;
                self.inventory[*style_slot] = *style_after;
                self.inventory[*stats_slot] = *stats_after;
                self.taros = *taros_after;
            }
            CombiAuthorityReceipt0104::Failure {
                owner_pc_id,
                style_slot,
                stats_slot,
                style_unchanged,
                stats_unchanged,
                taros_after,
            } => {
                if self.owner_pc_id != *owner_pc_id {
                    return Err(CombiReceiptCommitError0104::OwnerMismatch {
                        expected: *owner_pc_id,
                        actual: self.owner_pc_id,
                    });
                }
                self.validate_receipt_preconditions(
                    *style_slot,
                    *stats_slot,
                    *style_unchanged,
                    *stats_unchanged,
                )?;
                self.taros = *taros_after;
            }
        }
        Ok(())
    }

    pub(super) fn validate_receipt_preconditions(
        &self,
        style_slot: usize,
        stats_slot: usize,
        expected_style: ItemBase0104,
        expected_stats: ItemBase0104,
    ) -> Result<(), CombiReceiptCommitError0104> {
        if style_slot >= INVENTORY_SLOT_COUNT_0104 || stats_slot >= INVENTORY_SLOT_COUNT_0104 {
            return Err(CombiReceiptCommitError0104::SlotOutOfBounds {
                style_slot,
                stats_slot,
            });
        }
        if style_slot == stats_slot {
            return Err(CombiReceiptCommitError0104::AliasedSlots { style_slot });
        }
        if self.inventory[style_slot] != expected_style {
            return Err(CombiReceiptCommitError0104::StaleStylePrecondition {
                slot: style_slot,
                expected: expected_style,
                actual: self.inventory[style_slot],
            });
        }
        if self.inventory[stats_slot] != expected_stats {
            return Err(CombiReceiptCommitError0104::StaleStatsPrecondition {
                slot: stats_slot,
                expected: expected_stats,
                actual: self.inventory[stats_slot],
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CombiReceiptCommitError0104 {
    OwnerMismatch {
        expected: i32,
        actual: i32,
    },
    SlotOutOfBounds {
        style_slot: usize,
        stats_slot: usize,
    },
    AliasedSlots {
        style_slot: usize,
    },
    StaleStylePrecondition {
        slot: usize,
        expected: ItemBase0104,
        actual: ItemBase0104,
    },
    StaleStatsPrecondition {
        slot: usize,
        expected: ItemBase0104,
        actual: ItemBase0104,
    },
}

impl fmt::Display for CombiReceiptCommitError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "authoritative Combi receipt commit rejected: {self:?}")
    }
}

impl Error for CombiReceiptCommitError0104 {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CombiRecipe0104 {
    pub level_gap: i32,
    pub level_gap_standard: f32,
    pub same_grade: f32,
    pub one_grade: f32,
    pub two_grade: f32,
    pub three_grade: f32,
    pub look_constant: i32,
    pub stat_constant: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CombiRecipeTable0104 {
    pub(super) rows: Vec<CombiRecipe0104>,
}

impl CombiRecipeTable0104 {
    pub fn from_table_set_bytes(bytes: &[u8]) -> Result<Self, CombiRecipeTableError0104> {
        let root: Value = serde_json::from_slice(bytes)
            .map_err(|error| CombiRecipeTableError0104::MalformedJson(error.to_string()))?;
        let mut candidates = Vec::new();
        find_combining_arrays(&root, &mut candidates);
        let [rows] = candidates.as_slice() else {
            return Err(CombiRecipeTableError0104::CombiningTableCount {
                found: candidates.len(),
            });
        };
        if rows.len() != COMBI_RECIPE_ROW_COUNT_0104 {
            return Err(CombiRecipeTableError0104::WrongRowCount {
                expected: COMBI_RECIPE_ROW_COUNT_0104,
                actual: rows.len(),
            });
        }
        let mut parsed = Vec::with_capacity(rows.len());
        for (index, row) in rows.iter().enumerate() {
            let parsed_row = parse_recipe_row(index, row)?;
            if index == 0 {
                if parsed_row.level_gap != 0
                    || parsed_row.level_gap_standard != 0.0
                    || parsed_row.same_grade != 0.0
                    || parsed_row.one_grade != 0.0
                    || parsed_row.two_grade != 0.0
                    || parsed_row.three_grade != 0.0
                    || parsed_row.look_constant != 0
                    || parsed_row.stat_constant != 0
                {
                    return Err(CombiRecipeTableError0104::InvalidSentinel);
                }
            } else if parsed_row.level_gap != (index - 1) as i32 {
                return Err(CombiRecipeTableError0104::UnexpectedLevelGap {
                    index,
                    expected: (index - 1) as i32,
                    actual: parsed_row.level_gap,
                });
            }
            parsed.push(parsed_row);
        }
        Ok(Self { rows: parsed })
    }

    #[must_use]
    pub fn row_for_level_gap(&self, level_gap: usize) -> Option<&CombiRecipe0104> {
        if level_gap > COMBI_RECIPE_MAX_LEVEL_GAP_0104 {
            return None;
        }
        self.rows.get(level_gap + 1)
    }

    #[must_use]
    pub fn rows(&self) -> &[CombiRecipe0104] {
        &self.rows
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CombiRecipeTableError0104 {
    MalformedJson(String),
    CombiningTableCount {
        found: usize,
    },
    WrongRowCount {
        expected: usize,
        actual: usize,
    },
    RowNotObject {
        index: usize,
    },
    InvalidField {
        index: usize,
        field: &'static str,
    },
    InvalidSentinel,
    UnexpectedLevelGap {
        index: usize,
        expected: i32,
        actual: i32,
    },
}

impl fmt::Display for CombiRecipeTableError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "clean Combi recipe table rejected: {self:?}")
    }
}

impl Error for CombiRecipeTableError0104 {}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CombiLookError0104 {
    #[default]
    None,
    GenderMismatch,
    UnsupportedItemType,
    StatsTypeMismatch,
}

impl CombiLookError0104 {
    #[must_use]
    pub const fn legacy_code(self) -> i32 {
        match self {
            Self::None => 0,
            Self::GenderMismatch => 1,
            Self::UnsupportedItemType => 3,
            Self::StatsTypeMismatch => 4,
        }
    }

    #[must_use]
    pub const fn label(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::GenderMismatch => Some("DOES NOT MATCH PLAYER GENDER"),
            Self::UnsupportedItemType => Some("ONLY WEAPONS OR ARMOR CAN BE COMBINED"),
            Self::StatsTypeMismatch => Some("DOES NOT MATCH STATS ITEM TYPE"),
        }
    }

    #[must_use]
    pub(super) fn localized_text(self) -> Option<LocalizedText> {
        let (key, fallback) = match self {
            Self::None => return None,
            Self::GenderMismatch => (
                "ui.combi.error.look.gender_mismatch",
                "DOES NOT MATCH PLAYER GENDER",
            ),
            Self::UnsupportedItemType => (
                "ui.combi.error.unsupported_item_type",
                "ONLY WEAPONS OR ARMOR CAN BE COMBINED",
            ),
            Self::StatsTypeMismatch => (
                "ui.combi.error.look.stats_type_mismatch",
                "DOES NOT MATCH STATS ITEM TYPE",
            ),
        };
        Some(LocalizedText::new(key, fallback))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CombiStatsError0104 {
    #[default]
    None,
    GuideItem,
    ShopItem,
    UnsupportedItemType,
    StyleTypeMismatch,
}

impl CombiStatsError0104 {
    #[must_use]
    pub const fn legacy_code(self) -> i32 {
        match self {
            Self::None => 0,
            Self::GuideItem => 1,
            Self::ShopItem => 2,
            Self::UnsupportedItemType => 3,
            Self::StyleTypeMismatch => 4,
        }
    }

    #[must_use]
    pub const fn label(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::GuideItem => Some("CANNOT PLACE GUIDE ITEM HERE"),
            Self::ShopItem => Some("CANNOT PLACE SHOP ITEM HERE"),
            Self::UnsupportedItemType => Some("ONLY WEAPONS OR ARMOR CAN BE COMBINED"),
            Self::StyleTypeMismatch => Some("DOES NOT MATCH STYLE ITEM TYPE"),
        }
    }

    #[must_use]
    pub(super) fn localized_text(self) -> Option<LocalizedText> {
        let (key, fallback) = match self {
            Self::None => return None,
            Self::GuideItem => (
                "ui.combi.error.stats.guide_item",
                "CANNOT PLACE GUIDE ITEM HERE",
            ),
            Self::ShopItem => (
                "ui.combi.error.stats.shop_item",
                "CANNOT PLACE SHOP ITEM HERE",
            ),
            Self::UnsupportedItemType => (
                "ui.combi.error.unsupported_item_type",
                "ONLY WEAPONS OR ARMOR CAN BE COMBINED",
            ),
            Self::StyleTypeMismatch => (
                "ui.combi.error.stats.style_type_mismatch",
                "DOES NOT MATCH STYLE ITEM TYPE",
            ),
        };
        Some(LocalizedText::new(key, fallback))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CombiStatComparison0104 {
    Lower,
    #[default]
    Equal,
    Higher,
}

impl CombiStatComparison0104 {
    #[must_use]
    pub const fn color(self) -> CombiRgb0104 {
        match self {
            Self::Lower => COMBI_COLOR_RED,
            Self::Equal => COMBI_COLOR_DEFAULT,
            Self::Higher => COMBI_COLOR_GREEN,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CombiChance0104 {
    NotReady,
    NotPossible,
    VeryLow { raw_percent: f32 },
    Low { raw_percent: f32 },
    Medium { raw_percent: f32 },
    Good { raw_percent: f32 },
    VeryGood { raw_percent: f32 },
}

impl Default for CombiChance0104 {
    fn default() -> Self {
        Self::NotReady
    }
}

impl CombiChance0104 {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::NotReady => "NOT\nREADY",
            Self::NotPossible => "NOT\nPOSSIBLE",
            Self::VeryLow { .. } => "VERY\nLOW",
            Self::Low { .. } => "LOW",
            Self::Medium { .. } => "MEDIUM",
            Self::Good { .. } => "GOOD",
            Self::VeryGood { .. } => "VERY\nGOOD",
        }
    }

    #[must_use]
    pub(super) fn localized_text(self) -> LocalizedText {
        let (key, fallback) = match self {
            Self::NotReady => ("ui.combi.chance.not_ready", "NOT\nREADY"),
            Self::NotPossible => ("ui.combi.chance.not_possible", "NOT\nPOSSIBLE"),
            Self::VeryLow { .. } => ("ui.combi.chance.very_low", "VERY\nLOW"),
            Self::Low { .. } => ("ui.combi.chance.low", "LOW"),
            Self::Medium { .. } => ("ui.combi.chance.medium", "MEDIUM"),
            Self::Good { .. } => ("ui.combi.chance.good", "GOOD"),
            Self::VeryGood { .. } => ("ui.combi.chance.very_good", "VERY\nGOOD"),
        };
        LocalizedText::new(key, fallback)
    }

    #[must_use]
    pub const fn level(self) -> i32 {
        match self {
            Self::NotReady => -1,
            Self::NotPossible => 0,
            Self::VeryLow { .. } => 1,
            Self::Low { .. } => 2,
            Self::Medium { .. } => 3,
            Self::Good { .. } => 4,
            Self::VeryGood { .. } => 5,
        }
    }

    #[must_use]
    pub const fn color(self) -> CombiRgb0104 {
        match self {
            Self::NotReady => COMBI_COLOR_NOT_READY,
            Self::NotPossible => COMBI_COLOR_NOT_POSSIBLE,
            Self::VeryLow { .. } => CombiRgb0104::new(1.0, 0.0, 0.0),
            Self::Low { .. } => CombiRgb0104::new(0.8, 0.6, 0.0),
            Self::Medium { .. } => CombiRgb0104::new(1.0, 1.0, 0.0),
            Self::Good { .. } => CombiRgb0104::new(0.2, 1.0, 0.0),
            Self::VeryGood { .. } => CombiRgb0104::new(0.0, 0.8, 0.2),
        }
    }

    #[must_use]
    pub const fn raw_percent(self) -> Option<f32> {
        match self {
            Self::NotReady | Self::NotPossible => None,
            Self::VeryLow { raw_percent }
            | Self::Low { raw_percent }
            | Self::Medium { raw_percent }
            | Self::Good { raw_percent }
            | Self::VeryGood { raw_percent } => Some(raw_percent),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CombiLookProjection0104 {
    pub inventory_index: usize,
    pub item: ItemBase0104,
    pub icon_path: Option<String>,
    pub base_metadata: Option<CombiItemMetadata0104>,
    pub appearance_item_id: i16,
    pub appearance_metadata: Option<CombiItemMetadata0104>,
    pub error: CombiLookError0104,
    pub cannot_equip: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CombiStatsProjection0104 {
    pub inventory_index: usize,
    pub item: ItemBase0104,
    pub icon_path: Option<String>,
    pub metadata: Option<CombiItemMetadata0104>,
    pub error: CombiStatsError0104,
    pub cannot_equip: bool,
    pub point_comparison: CombiStatComparison0104,
    pub group_comparison: CombiStatComparison0104,
    pub defense_comparison: CombiStatComparison0104,
    pub range_label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CombiProjectionError0104 {
    SelectionOutOfBounds { index: usize },
    SelectedSlotEmpty { index: usize },
    MissingBaseMetadata { item_type: i16, item_id: i16 },
    MissingAppearanceMetadata { item_type: i16, item_id: i16 },
    MissingRecipeForLevelGap { level_gap: usize },
    CostOverflow { raw_cost: i64 },
    NegativeCost { cost: i32 },
    InvalidChance { raw_percent: f32 },
    UnsupportedChanceRetention { raw_percent: f32 },
}

impl fmt::Display for CombiProjectionError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "clean Combi projection rejected: {self:?}")
    }
}

impl Error for CombiProjectionError0104 {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CombiRequest0104 {
    pub costume_item_slot: i32,
    pub stat_item_slot: i32,
    pub cash_item_slot_1: i32,
    pub cash_item_slot_2: i32,
}

impl CombiRequest0104 {
    #[must_use]
    pub const fn new(style_inventory_index: usize, stats_inventory_index: usize) -> Self {
        Self {
            costume_item_slot: style_inventory_index as i32,
            stat_item_slot: stats_inventory_index as i32,
            cash_item_slot_1: 0,
            cash_item_slot_2: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CombiSuccessReply0104 {
    pub new_item_slot: i32,
    pub new_item: ItemBase0104,
    pub stat_item_slot: i32,
    pub cash_item_slot_1: i32,
    pub cash_item_slot_2: i32,
    pub taros_after: i32,
    pub success_flag: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CombiFailureReply0104 {
    pub error_code: i32,
    pub costume_item_slot: i32,
    pub stat_item_slot: i32,
    pub cash_item_slot_1: i32,
    pub cash_item_slot_2: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CombiAuthorityReceipt0104 {
    Success {
        owner_pc_id: i32,
        style_slot: usize,
        stats_slot: usize,
        style_before: ItemBase0104,
        stats_before: ItemBase0104,
        style_after: ItemBase0104,
        stats_after: ItemBase0104,
        taros_after: i32,
    },
    Failure {
        owner_pc_id: i32,
        style_slot: usize,
        stats_slot: usize,
        style_unchanged: ItemBase0104,
        stats_unchanged: ItemBase0104,
        taros_after: i32,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct CombiSuccessPresentation0104 {
    pub look: CombiLookProjection0104,
    pub stats: CombiStatsProjection0104,
    pub combined_item_cannot_equip: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CombiSystemModal0104 {
    AttemptConfirmation,
    NotEnoughTaros,
    CombinationFailed,
}

impl CombiSystemModal0104 {
    #[must_use]
    pub const fn message_id(self) -> i32 {
        match self {
            Self::AttemptConfirmation => 254,
            Self::CombinationFailed => 255,
            Self::NotEnoughTaros => 256,
        }
    }

    #[must_use]
    pub const fn exact_text(self) -> &'static str {
        match self {
            Self::AttemptConfirmation => COMBI_MESSAGE_254,
            Self::CombinationFailed => COMBI_MESSAGE_255,
            Self::NotEnoughTaros => COMBI_MESSAGE_256,
        }
    }

    #[must_use]
    pub fn localized_text(self) -> LocalizedText {
        let key = match self {
            Self::AttemptConfirmation => "ui.combi.modal.attempt_confirmation",
            Self::CombinationFailed => "ui.combi.modal.combination_failed",
            Self::NotEnoughTaros => "ui.combi.modal.not_enough_taros",
        };
        LocalizedText::new(key, self.exact_text())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CombiModalChoice0104 {
    Continue,
    Cancel,
    Ok,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum CombiPhase0104 {
    #[default]
    Hidden,
    Ready,
    Modal(CombiSystemModal0104),
    Waiting {
        elapsed_seconds: f32,
    },
    AwaitingAuthoritativeReply,
    Success(CombiSuccessPresentation0104),
}

impl CombiPhase0104 {
    #[must_use]
    pub const fn sending_locked(&self) -> bool {
        matches!(
            self,
            Self::Waiting { .. }
                | Self::AwaitingAuthoritativeReply
                | Self::Success(_)
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CombiPendingAttempt0104 {
    pub(super) style_slot: usize,
    pub(super) stats_slot: usize,
    pub(super) style_before: ItemBase0104,
    pub(super) stats_before: ItemBase0104,
    pub(super) cost: i32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CombiMachine0104 {
    pub phase: CombiPhase0104,
    pub selection: CombiSelectionOverlay0104,
    pub(super) pending: Option<CombiPendingAttempt0104>,
}
