use super::*;

/// Exact clean `FusionFallInvenSkin` text role after runtime clone overrides.
/// Replacement fonts change glyph coverage only; serialized font ownership,
/// alignment, wrapping, padding, content offset and source Rect remain typed.
#[derive(Clone, Copy, Component, Debug, Eq, Hash, PartialEq)]
pub enum VendorUiTextStyle {
    LabelUpperLeft,
    /// `Panel_Vendor.RenderNpc` clones `label`, then swaps only its font to
    /// `leftLabel.font` (Chalet pathId 949) and alignment to UpperLeft.
    LabelUpperLeftService,
    BlankBoxUpperLeft,
    BlankBoxMiddleRight,
    ButtonMiddleCenter,
    EquipBarMiddleCenter,
    EquipFontMiddleRight,
}

impl VendorUiTextStyle {
    #[must_use]
    pub const fn source_style_name(self) -> &'static str {
        match self {
            Self::LabelUpperLeft | Self::LabelUpperLeftService => "label",
            Self::BlankBoxUpperLeft | Self::BlankBoxMiddleRight => "blankbox",
            Self::ButtonMiddleCenter => "button",
            Self::EquipBarMiddleCenter => "equipbar",
            Self::EquipFontMiddleRight => "equipfont",
        }
    }

    #[must_use]
    pub const fn source_font_path_id(self) -> i64 {
        match self {
            Self::LabelUpperLeft | Self::BlankBoxUpperLeft | Self::BlankBoxMiddleRight => {
                VENDOR_LABEL_SOURCE_FONT_PATH_ID
            }
            Self::LabelUpperLeftService => VENDOR_SERVICE_SOURCE_FONT_PATH_ID,
            Self::ButtonMiddleCenter => VENDOR_BUTTON_SOURCE_FONT_PATH_ID,
            Self::EquipBarMiddleCenter | Self::EquipFontMiddleRight => {
                VENDOR_EQUIP_SOURCE_FONT_PATH_ID
            }
        }
    }

    #[must_use]
    pub const fn legacy_alignment(self) -> i32 {
        match self {
            Self::LabelUpperLeft | Self::LabelUpperLeftService | Self::BlankBoxUpperLeft => 0,
            Self::ButtonMiddleCenter | Self::EquipBarMiddleCenter => 4,
            Self::BlankBoxMiddleRight | Self::EquipFontMiddleRight => 5,
        }
    }

    /// Serialized `[left, right, top, bottom]` `RectOffset` after the clean
    /// service clone. Every used style has zero content offset.
    #[must_use]
    pub const fn padding(self) -> [f32; 4] {
        match self {
            Self::LabelUpperLeft | Self::LabelUpperLeftService => [
                0.0,
                0.0,
                VENDOR_LABEL_PADDING_TOP,
                VENDOR_LABEL_PADDING_BOTTOM,
            ],
            Self::ButtonMiddleCenter => [
                VENDOR_BUTTON_PADDING_LEFT,
                VENDOR_BUTTON_PADDING_RIGHT,
                VENDOR_BUTTON_PADDING_TOP,
                VENDOR_BUTTON_PADDING_BOTTOM,
            ],
            Self::BlankBoxUpperLeft
            | Self::BlankBoxMiddleRight
            | Self::EquipBarMiddleCenter
            | Self::EquipFontMiddleRight => [0.0; 4],
        }
    }

    #[must_use]
    pub const fn content_offset(self) -> [f32; 2] {
        [0.0, 0.0]
    }

    #[must_use]
    pub const fn word_wrap(self) -> bool {
        matches!(self, Self::LabelUpperLeft | Self::LabelUpperLeftService)
    }

    #[must_use]
    pub const fn font_size(self) -> f32 {
        match self {
            Self::LabelUpperLeft | Self::BlankBoxUpperLeft | Self::BlankBoxMiddleRight => {
                VENDOR_LABEL_FONT_SIZE
            }
            Self::LabelUpperLeftService => VENDOR_SERVICE_FONT_SIZE,
            Self::ButtonMiddleCenter => VENDOR_BUTTON_FONT_SIZE,
            Self::EquipBarMiddleCenter | Self::EquipFontMiddleRight => VENDOR_EQUIP_FONT_SIZE,
        }
    }

    #[must_use]
    pub const fn line_height(self) -> f32 {
        match self {
            Self::LabelUpperLeft | Self::BlankBoxUpperLeft | Self::BlankBoxMiddleRight => {
                VENDOR_LABEL_LINE_HEIGHT
            }
            Self::LabelUpperLeftService => VENDOR_SERVICE_LINE_HEIGHT,
            Self::ButtonMiddleCenter => VENDOR_BUTTON_LINE_HEIGHT,
            Self::EquipBarMiddleCenter | Self::EquipFontMiddleRight => VENDOR_EQUIP_LINE_HEIGHT,
        }
    }

    #[must_use]
    pub const fn replacement_y_offset(self) -> f32 {
        VENDOR_TEXT_REPLACEMENT_Y_OFFSET
    }

    pub(super) fn font(self, font: Handle<Font>) -> (TextFont, LineHeight) {
        (
            TextFont {
                font: (font).into(),
                font_size: (self.font_size()).into(),
                ..default()
            },
            LineHeight::Px(self.line_height()),
        )
    }

    pub(super) fn layout(self) -> TextLayout {
        let justify = match self {
            Self::LabelUpperLeft | Self::LabelUpperLeftService | Self::BlankBoxUpperLeft => {
                Justify::Left
            }
            Self::BlankBoxMiddleRight | Self::EquipFontMiddleRight => Justify::Right,
            Self::ButtonMiddleCenter | Self::EquipBarMiddleCenter => Justify::Center,
        };
        TextLayout::new(
            justify,
            if self.word_wrap() {
                LineBreak::WordBoundary
            } else {
                LineBreak::NoWrap
            },
        )
    }

    pub(super) fn apply_to_container(self, node: &mut Node) {
        node.justify_content = match self {
            Self::LabelUpperLeft | Self::LabelUpperLeftService | Self::BlankBoxUpperLeft => {
                JustifyContent::FlexStart
            }
            Self::BlankBoxMiddleRight | Self::EquipFontMiddleRight => JustifyContent::FlexEnd,
            Self::ButtonMiddleCenter | Self::EquipBarMiddleCenter => JustifyContent::Center,
        };
        node.align_items = match self {
            Self::LabelUpperLeft | Self::LabelUpperLeftService | Self::BlankBoxUpperLeft => {
                AlignItems::FlexStart
            }
            Self::BlankBoxMiddleRight
            | Self::ButtonMiddleCenter
            | Self::EquipBarMiddleCenter
            | Self::EquipFontMiddleRight => AlignItems::Center,
        };
        let [left, right, top, bottom] = self.padding();
        node.padding = UiRect::new(px(left), px(right), px(top), px(bottom));
        node.overflow = if matches!(
            self,
            Self::LabelUpperLeft | Self::LabelUpperLeftService | Self::ButtonMiddleCenter
        ) {
            Overflow::clip()
        } else {
            Overflow::visible()
        };
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum VendorTab0104 {
    Buy = 0,
    Buyback = 3,
}

impl Default for VendorTab0104 {
    fn default() -> Self {
        Self::Buy
    }
}

impl VendorTab0104 {
    #[must_use]
    pub const fn legacy_mode(self) -> i32 {
        self as i32
    }
}

/// The two IDs passed into clean `SetVendorNpc`, plus the NPC identity
/// accepted by `VENDOR_START_SUCC`. The runtime NPC and vendor table IDs
/// stay separate for proximity and catalog lookup.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct VendorSession0104 {
    pub requested_npc_id: i32,
    pub table_vendor_id: i32,
    pub accepted_npc_id: i32,
}

impl VendorSession0104 {
    #[must_use]
    pub const fn request_identity(self) -> VendorRequestIdentity0104 {
        VendorRequestIdentity0104 {
            npc_id: self.accepted_npc_id,
            vendor_id: self.table_vendor_id,
            // Exact unchecked C# `(sbyte)iVendorID` semantics, widened again
            // only so the semantic intent remains independent of wire types.
            list_id: (self.table_vendor_id as i8) as i32,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct VendorIconRef {
    pub(super) runtime_path: String,
}

impl VendorIconRef {
    pub fn new(runtime_path: impl Into<String>) -> Result<Self, VendorIconRefError> {
        let runtime_path = runtime_path.into();
        if !is_safe_relative_asset_path(&runtime_path) {
            return Err(VendorIconRefError::UnsafeRuntimePath { runtime_path });
        }
        Ok(Self { runtime_path })
    }

    #[must_use]
    pub fn runtime_path(&self) -> &str {
        &self.runtime_path
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VendorItemMetadata0104 {
    pub name: String,
    pub level: i32,
    pub buy_price: i32,
    pub sell_price: i32,
    pub sellable: bool,
    /// Clean `GeneralItemElement.m_iItemType`; value `2` selects
    /// `cnVendor.BuyGeneral`.
    pub general_item_type: Option<i32>,
    /// Clean `GeneralItemElement.m_iBatteryRecharge`; zero means the
    /// battery-buy channel delivers an inventory item and therefore needs an
    /// empty slot.
    pub battery_recharge: Option<i32>,
    /// Clean `GeneralItemElement.m_iStackNumber`. Non-general tables and
    /// compact source subsets keep this unresolved.
    pub stack_size: Option<i32>,
    pub icon: Option<VendorIconRef>,
}

pub trait VendorItemCatalog0104 {
    fn resolve(&self, item: ItemBase0104) -> Option<VendorItemMetadata0104>;

    /// Clean vehicle row copy comes from table 26 `m_iUp_runSpeed`, not from
    /// packet item fields. Catalogs that do not own that table fail closed and
    /// suppress only the conditional speed label.
    fn vehicle_speed_class(&self, _item: ItemBase0104) -> Option<i32> {
        None
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VendorMissingCatalog0104;

impl VendorItemCatalog0104 for VendorMissingCatalog0104 {
    fn resolve(&self, _item: ItemBase0104) -> Option<VendorItemMetadata0104> {
        None
    }
}

pub trait VendorEquipEligibility0104 {
    fn enable_equip(&self, item: ItemBase0104) -> Option<bool>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VendorFailClosedEquipEligibility0104;

impl VendorEquipEligibility0104 for VendorFailClosedEquipEligibility0104 {
    fn enable_equip(&self, _item: ItemBase0104) -> Option<bool> {
        None
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VendorEquipValidation0104 {
    NotApplicable,
    Allowed,
    Rejected,
    Unverified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VendorMissingIconReason0104 {
    InvalidIdentity { item_type: i16, item_id: i16 },
    CatalogMiss { item_type: i16, item_id: i16 },
    CatalogWithoutIcon { item_type: i16, item_id: i16 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VendorPresentationIcon0104 {
    Empty,
    Resolved(VendorIconRef),
    MissingChecker(VendorMissingIconReason0104),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VendorRecentBuyEntry0104 {
    /// Clean local slot type 12 FIFO `SlotID`; restore uses `SlotID + 1`.
    pub source_slot_id: usize,
    pub item: ItemBase0104,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VendorRecentBuyRowProjection0104 {
    pub fifo_index: usize,
    pub source_slot_id: usize,
    pub restore_list_id: i32,
    pub item: ItemBase0104,
    pub metadata: Option<VendorItemMetadata0104>,
    pub icon: VendorPresentationIcon0104,
    pub price: Option<i32>,
    pub affordable: Option<bool>,
    pub vehicle_speed_class: Option<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VendorProjectedItem0104 {
    pub item: ItemBase0104,
    pub empty: bool,
    pub icon: VendorPresentationIcon0104,
    pub show_combined_badge: bool,
    pub count_label: Option<String>,
    pub quest_item_id: Option<i16>,
    pub metadata: Option<VendorItemMetadata0104>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VendorEquipmentSlotProjection0104 {
    pub visual_index: usize,
    pub wire_slot_index: usize,
    pub item: VendorProjectedItem0104,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VendorProjectionError0104 {
    OwnerMismatch {
        expected_pc_id: i32,
        inventory_pc_id: i32,
    },
}

impl fmt::Display for VendorProjectionError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OwnerMismatch {
                expected_pc_id,
                inventory_pc_id,
            } => write!(
                f,
                "VendorMode belongs to PC {expected_pc_id}, inventory belongs to PC {inventory_pc_id}"
            ),
        }
    }
}

impl Error for VendorProjectionError0104 {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VendorBuyIntent0104 {
    pub identity: VendorRequestIdentity0104,
    pub item: ItemBase0104,
    pub inventory_slot: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VendorBuyGeneralIntent0104 {
    pub identity: VendorRequestIdentity0104,
    pub item: ItemBase0104,
    pub inventory_slot: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VendorBatteryIntent0104 {
    pub identity: VendorRequestIdentity0104,
    pub item: ItemBase0104,
    pub battery_recharge: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VendorRestoreIntent0104 {
    pub identity: VendorRequestIdentity0104,
    pub restore_list_id: i32,
    pub item: ItemBase0104,
    pub inventory_slot: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VendorSellIntent0104 {
    pub item: ItemBase0104,
    pub inventory_slot: usize,
    pub count: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VendorDeleteIntent0104 {
    pub location: VendorInventoryLocation0104,
    pub inventory_slot: usize,
    pub item: ItemBase0104,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VendorDisassembleIntent0104 {
    pub inventory_slot: usize,
    pub item: ItemBase0104,
}

/// Semantic request boundary only. No variant promises a packet ID, layout,
/// or encoding; the production transport owner must map it against the
/// selected legacy protocol.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VendorIntent0104 {
    Buy(VendorBuyIntent0104),
    BuyGeneral(VendorBuyGeneralIntent0104),
    Battery(VendorBatteryIntent0104),
    Restore(VendorRestoreIntent0104),
    Sell(VendorSellIntent0104),
    Delete(VendorDeleteIntent0104),
    Disassemble(VendorDisassembleIntent0104),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum VendorSystemMessageId0104 {
    InventoryFull = 12,
    StartFailed = 21,
    TableUpdateFailed = 22,
    BuyFailed = 23,
    BatteryFailed = 24,
    SellFailed = 25,
    CannotSellChest = 107,
    CannotSellQuestItem = 108,
    ItemNotSellable = 109,
    ConfirmDelete = 153,
    ConfirmDisassemble = 257,
    PurchaseRestricted = 264,
}

impl VendorSystemMessageId0104 {
    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VendorSystemMessageCallback0104 {
    Exit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VendorSystemMessage0104 {
    pub message_id: VendorSystemMessageId0104,
    pub callback: Option<VendorSystemMessageCallback0104>,
}

impl VendorSystemMessage0104 {
    #[must_use]
    pub const fn plain(message_id: VendorSystemMessageId0104) -> Self {
        Self {
            message_id,
            callback: None,
        }
    }

    #[must_use]
    pub const fn exit(message_id: VendorSystemMessageId0104) -> Self {
        Self {
            message_id,
            callback: Some(VendorSystemMessageCallback0104::Exit),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VendorConfirmationCallback0104 {
    DeleteItemOk,
    HammerItemOk,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VendorConfirmation0104 {
    pub message_id: VendorSystemMessageId0104,
    pub callback: VendorConfirmationCallback0104,
    pub icon: VendorPresentationIcon0104,
    pub delete_count: Option<i32>,
    pub(super) intent: VendorIntent0104,
}

impl VendorConfirmation0104 {
    #[must_use]
    pub fn confirm(self) -> VendorIntent0104 {
        self.intent
    }

    #[must_use]
    pub fn intent(&self) -> VendorIntent0104 {
        self.intent
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VendorSilentBlock0104 {
    InvalidCatalogRow {
        row_index: usize,
    },
    InvalidRecentBuyRow {
        row_index: usize,
    },
    InvalidInventorySlot {
        inventory_slot: usize,
    },
    EmptyInventorySlot {
        inventory_slot: usize,
    },
    MissingCatalogMetadata {
        item_type: i16,
        item_id: i16,
    },
    MissingBatteryMetadata {
        item_id: i16,
    },
    MissingStackMetadata {
        item_id: i16,
    },
    InventoryFullForGeneralBuy,
    InventoryFullForRestore,
    InvalidCount {
        requested: i32,
        maximum: Option<i32>,
    },
    DisassembleIneligible {
        item_type: i16,
    },
    PopupActionUnavailable,
}

/// Clean calculator/fixed-value contract shown by the legacy item popups.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VendorQuantityContract0104 {
    /// Equip/Turing popups submit the exact fixed value without a calculator.
    Fixed(i32),
    /// GumPopup accepts a positive decimal value capped at this maximum.
    Calculator { maximum: i32 },
}

impl VendorQuantityContract0104 {
    #[must_use]
    pub const fn accepts(self, value: i32) -> bool {
        match self {
            Self::Fixed(expected) => value == expected,
            Self::Calculator { maximum } => value > 0 && value <= maximum,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VendorItemPopupSource0104 {
    CatalogRow { row_index: usize },
    BuybackRow { row_index: usize },
    InventorySlot { inventory_slot: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VendorPopupCommit0104 {
    Buy { selected_option: i32 },
    Buyback,
    Sell { count: i32 },
    Delete,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VendorActivationOutcome0104 {
    Popup(VendorItemActionPopup0104),
    Action(VendorActionOutcome0104),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VendorServerFailure0104 {
    Start,
    TableUpdate,
    Buy { error_code: i32 },
    Battery,
    Sell,
    Restore { error_code: i32 },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum VendorLifecyclePhase {
    #[default]
    Hidden,
    Opening,
    Visible,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct VendorCloseGate0104 {
    pub mode_accepts_escape: bool,
    pub target_action_idle: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VendorUiCommand0104 {
    StartSession {
        requested_npc_id: i32,
        table_vendor_id: i32,
    },
    Intent(VendorIntent0104),
    ShowSystemMessage(VendorSystemMessage0104),
    OpenConfirmation(VendorConfirmation0104),
    OpenItemActionPopup(VendorItemActionPopup0104),
    OpenHelp {
        event_id: i32,
    },
    OpenRedeemCode,
    GoToMyStuff,
    ExitMode,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct VendorUiOutbox0104 {
    pub(super) commands: VecDeque<VendorUiCommand0104>,
}

impl VendorUiOutbox0104 {
    pub fn push(&mut self, command: VendorUiCommand0104) {
        self.commands.push_back(command);
    }

    pub fn pop_front(&mut self) -> Option<VendorUiCommand0104> {
        self.commands.pop_front()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VendorRowView0104 {
    pub name: String,
    pub level: Option<i32>,
    pub level_label: String,
    pub vehicle_speed_class: Option<i32>,
    pub price: Option<i32>,
    pub price_label: String,
    pub affordable: Option<bool>,
    pub equip_validation: VendorEquipValidation0104,
    pub icon: VendorPresentationIcon0104,
    pub count_label: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VendorSlotView0104 {
    pub frame_visual: VendorSlotFrameVisual0104,
    pub icon: VendorPresentationIcon0104,
    pub show_combined_badge: bool,
    pub count_label: Option<String>,
    pub quest_item_id: Option<i16>,
}

#[derive(Clone, Resource)]
pub(super) struct VendorUiAssets {
    pub(super) images: [Handle<Image>; VendorStaticAssetRole::COUNT],
    pub(super) font: Handle<Font>,
    pub(super) service_font: Handle<Font>,
    pub(super) missing_checker: Handle<Image>,
}
