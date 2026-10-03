use super::*;

/// Exact clean `FusionFallInvenSkin` GUIStyle role carried by every BankMode
/// `Text`. Localization changes copy only; it never changes the primary style
/// or source Rect.
#[derive(Clone, Copy, Component, Debug, Eq, Hash, PartialEq)]
pub enum BankUiTextStyle {
    SearchUpperLeft,
    LabelUpperLeft,
    BlankBoxUpperLeft,
    BlankBoxMiddleRight,
    ButtonMiddleCenter,
    EquipBarMiddleCenter,
    EquipFontMiddleRight,
}

impl BankUiTextStyle {
    #[must_use]
    pub const fn source_style_name(self) -> &'static str {
        match self {
            Self::SearchUpperLeft => "textfield",
            Self::LabelUpperLeft => "label",
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
                BANK_LABEL_SOURCE_FONT_PATH_ID
            }
            Self::SearchUpperLeft => 949,
            Self::ButtonMiddleCenter => BANK_BUTTON_SOURCE_FONT_PATH_ID,
            Self::EquipBarMiddleCenter | Self::EquipFontMiddleRight => {
                BANK_EQUIP_SOURCE_FONT_PATH_ID
            }
        }
    }

    /// Unity `TextAnchor` numeric value used by the source style after any
    /// clean runtime clone override (`blankbox` Taros digits become 5).
    #[must_use]
    pub const fn legacy_alignment(self) -> i32 {
        match self {
            Self::SearchUpperLeft | Self::LabelUpperLeft | Self::BlankBoxUpperLeft => 0,
            Self::ButtonMiddleCenter | Self::EquipBarMiddleCenter => 4,
            Self::BlankBoxMiddleRight => 5,
            Self::EquipFontMiddleRight => 5,
        }
    }

    /// Serialized `[left, right, top, bottom]` `RectOffset`.
    #[must_use]
    pub const fn padding(self) -> [f32; 4] {
        match self {
            Self::SearchUpperLeft => [3.0, 3.0, 1.0, 3.0],
            Self::LabelUpperLeft => [0.0, 0.0, BANK_LABEL_PADDING_TOP, BANK_LABEL_PADDING_BOTTOM],
            Self::ButtonMiddleCenter => [
                BANK_BUTTON_PADDING_LEFT,
                BANK_BUTTON_PADDING_RIGHT,
                BANK_BUTTON_PADDING_TOP,
                BANK_BUTTON_PADDING_BOTTOM,
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
        matches!(self, Self::LabelUpperLeft)
    }

    #[must_use]
    pub const fn font_size(self) -> f32 {
        match self {
            Self::LabelUpperLeft | Self::BlankBoxUpperLeft | Self::BlankBoxMiddleRight => {
                BANK_LABEL_FONT_SIZE
            }
            Self::SearchUpperLeft => 12.0,
            Self::ButtonMiddleCenter => BANK_BUTTON_FONT_SIZE,
            Self::EquipBarMiddleCenter | Self::EquipFontMiddleRight => BANK_EQUIP_FONT_SIZE,
        }
    }

    #[must_use]
    pub const fn line_height(self) -> f32 {
        match self {
            Self::LabelUpperLeft | Self::BlankBoxUpperLeft | Self::BlankBoxMiddleRight => {
                BANK_LABEL_LINE_HEIGHT
            }
            Self::SearchUpperLeft => 13.56000042,
            Self::ButtonMiddleCenter => BANK_BUTTON_LINE_HEIGHT,
            Self::EquipBarMiddleCenter | Self::EquipFontMiddleRight => BANK_EQUIP_LINE_HEIGHT,
        }
    }

    #[must_use]
    pub const fn replacement_y_offset(self) -> f32 {
        BANK_TEXT_REPLACEMENT_Y_OFFSET
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
            Self::SearchUpperLeft | Self::LabelUpperLeft | Self::BlankBoxUpperLeft => Justify::Left,
            Self::BlankBoxMiddleRight | Self::EquipFontMiddleRight => Justify::Right,
            Self::ButtonMiddleCenter | Self::EquipBarMiddleCenter => Justify::Center,
        };
        let linebreak = if self.word_wrap() {
            LineBreak::WordBoundary
        } else {
            LineBreak::NoWrap
        };
        TextLayout::new(justify, linebreak)
    }

    pub(super) fn apply_to_container(self, node: &mut Node) {
        node.justify_content = match self {
            Self::SearchUpperLeft | Self::LabelUpperLeft | Self::BlankBoxUpperLeft => {
                JustifyContent::FlexStart
            }
            Self::BlankBoxMiddleRight | Self::EquipFontMiddleRight => JustifyContent::FlexEnd,
            Self::ButtonMiddleCenter | Self::EquipBarMiddleCenter => JustifyContent::Center,
        };
        node.align_items = match self {
            Self::SearchUpperLeft | Self::LabelUpperLeft | Self::BlankBoxUpperLeft => {
                AlignItems::FlexStart
            }
            Self::BlankBoxMiddleRight | Self::ButtonMiddleCenter | Self::EquipBarMiddleCenter => {
                AlignItems::Center
            }
            Self::EquipFontMiddleRight => AlignItems::Center,
        };
        let [left, right, top, bottom] = self.padding();
        node.padding = UiRect::new(px(left), px(right), px(top), px(bottom));
        node.overflow = match self {
            Self::SearchUpperLeft | Self::LabelUpperLeft | Self::ButtonMiddleCenter => {
                Overflow::clip()
            }
            Self::BlankBoxUpperLeft
            | Self::BlankBoxMiddleRight
            | Self::EquipBarMiddleCenter
            | Self::EquipFontMiddleRight => Overflow::visible(),
        };
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum BankSlotLocation0104 {
    Inventory = 1,
    Bank = 3,
}

impl BankSlotLocation0104 {
    #[must_use]
    pub const fn wire_value(self) -> i32 {
        self as i32
    }

    #[must_use]
    pub const fn capacity(self) -> usize {
        match self {
            Self::Inventory => INVENTORY_SLOT_COUNT_0104,
            Self::Bank => BANK_SLOT_COUNT_0104,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BankSlotRef0104 {
    pub(super) location: BankSlotLocation0104,
    pub(super) index: usize,
}

impl BankSlotRef0104 {
    pub fn new(location: BankSlotLocation0104, index: usize) -> Result<Self, BankSlotRefError0104> {
        if index >= location.capacity() {
            return Err(BankSlotRefError0104::OutOfBounds {
                location,
                index,
                capacity: location.capacity(),
            });
        }
        Ok(Self { location, index })
    }

    pub fn from_wire(raw_location: i32, raw_index: i32) -> Result<Self, BankSlotRefError0104> {
        let location = match raw_location {
            1 => BankSlotLocation0104::Inventory,
            3 => BankSlotLocation0104::Bank,
            _ => {
                return Err(BankSlotRefError0104::UnsupportedLocation { raw_location });
            }
        };
        if raw_index < 0 {
            return Err(BankSlotRefError0104::NegativeIndex {
                location,
                raw_index,
            });
        }
        Self::new(location, raw_index as usize)
    }

    #[must_use]
    pub const fn location(self) -> BankSlotLocation0104 {
        self.location
    }

    #[must_use]
    pub const fn index(self) -> usize {
        self.index
    }

    #[must_use]
    pub const fn wire_index(self) -> i32 {
        self.index as i32
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankSlotRefError0104 {
    UnsupportedLocation {
        raw_location: i32,
    },
    NegativeIndex {
        location: BankSlotLocation0104,
        raw_index: i32,
    },
    OutOfBounds {
        location: BankSlotLocation0104,
        index: usize,
        capacity: usize,
    },
}

impl fmt::Display for BankSlotRefError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedLocation { raw_location } => {
                write!(f, "unsupported bank-mode item location {raw_location}")
            }
            Self::NegativeIndex {
                location,
                raw_index,
            } => write!(f, "{location:?} slot index is negative: {raw_index}"),
            Self::OutOfBounds {
                location,
                index,
                capacity,
            } => write!(
                f,
                "{location:?} slot {index} is outside capacity {capacity}"
            ),
        }
    }
}

impl Error for BankSlotRefError0104 {}

pub trait BankEquipEligibility {
    fn enable_equip(&self, item: ItemBase0104) -> Option<bool>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BankFailClosedEquipEligibility;

impl BankEquipEligibility for BankFailClosedEquipEligibility {
    fn enable_equip(&self, _item: ItemBase0104) -> Option<bool> {
        None
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BankSlotProjection0104 {
    pub slot_index: usize,
    pub column: usize,
    pub row: usize,
    pub item: ItemBase0104,
    pub empty: bool,
    pub locked: bool,
    pub ids: UserEquipItemIds,
    pub icon: UserEquipProjectedIcon,
    pub equip_validation: BankEquipValidation,
    pub show_combined_badge: bool,
    pub count_label: Option<String>,
}

impl BankSlotProjection0104 {
    pub(super) fn from_authoritative(
        slot_index: usize,
        item: ItemBase0104,
        locked: bool,
        catalog: &impl UserEquipItemCatalog,
        equip_eligibility: &impl BankEquipEligibility,
    ) -> Self {
        let empty = InventoryRuntime0104::item_is_empty(item);
        let ids = UserEquipItemIds::from_item(item);
        let icon = project_bank_icon(item, catalog);
        let equip_validation = if empty || !bank_item_needs_equip_validation(item.item_type) {
            BankEquipValidation::NotApplicable
        } else {
            match equip_eligibility.enable_equip(item) {
                Some(true) => BankEquipValidation::Allowed,
                Some(false) => BankEquipValidation::Rejected,
                None => BankEquipValidation::Unverified,
            }
        };
        Self {
            slot_index,
            column: slot_index % BANK_GRID_COLUMNS,
            row: slot_index / BANK_GRID_COLUMNS,
            item,
            empty,
            locked,
            ids,
            icon,
            equip_validation,
            show_combined_badge: !empty
                && (0..4).contains(&item.item_type)
                && ids.combined_look_id.is_some(),
            count_label: (!empty && item.item_type == 7).then(|| item.option.to_string()),
        }
    }
}

/// Complete item authority needed while BankMode is open.
///
/// Outbound intents never touch this snapshot. Only an exact bank-open reply
/// or the two post-state `sItemBase` values in `PC_ITEM_MOVE_SUCC` may change
/// it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BankAuthoritativeSnapshot0104 {
    pub(super) owner_pc_id: i32,
    pub(super) npc_id: i32,
    pub(super) extra_bank: i32,
    pub(super) bank: [ItemBase0104; BANK_SLOT_COUNT_0104],
    pub(super) inventory: [ItemBase0104; INVENTORY_SLOT_COUNT_0104],
    pub(super) equipment: [ItemBase0104; EQUIPMENT_SLOT_COUNT_0104],
}

impl BankAuthoritativeSnapshot0104 {
    pub fn from_open_success(
        owner_pc_id: i32,
        npc_id: i32,
        open: &PcBankOpenSuccess0104,
        inventory: &InventoryRuntime0104,
    ) -> Result<Self, BankProjectionError0104> {
        if inventory.owner_pc_id() != owner_pc_id {
            return Err(BankProjectionError0104::OwnerMismatch {
                expected_pc_id: owner_pc_id,
                inventory_pc_id: inventory.owner_pc_id(),
            });
        }
        Ok(Self {
            owner_pc_id,
            npc_id,
            extra_bank: open.extra_bank,
            bank: open.bank_items,
            inventory: *inventory.inventory(),
            equipment: *inventory.equipment(),
        })
    }

    #[must_use]
    pub const fn owner_pc_id(&self) -> i32 {
        self.owner_pc_id
    }

    #[must_use]
    pub const fn npc_id(&self) -> i32 {
        self.npc_id
    }

    #[must_use]
    pub const fn extra_bank(&self) -> i32 {
        self.extra_bank
    }

    #[must_use]
    pub const fn bank(&self) -> &[ItemBase0104; BANK_SLOT_COUNT_0104] {
        &self.bank
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
    pub fn item_at(&self, slot: BankSlotRef0104) -> ItemBase0104 {
        match slot.location {
            BankSlotLocation0104::Inventory => self.inventory[slot.index],
            BankSlotLocation0104::Bank => self.bank[slot.index],
        }
    }

    /// Atomically applies only the authoritative post-state values carried by
    /// `PC_ITEM_MOVE_SUCC`. No request-side swap or stack arithmetic occurs.
    pub fn apply_item_move_success(
        &mut self,
        packet: ItemMoveSuccessPacket0104,
    ) -> Result<BankAuthorityMutationReceipt0104, BankAuthorityMutationError0104> {
        let from = BankSlotRef0104::from_wire(packet.from_location, packet.from_slot_num)
            .map_err(BankAuthorityMutationError0104::Slot)?;
        let to = BankSlotRef0104::from_wire(packet.to_location, packet.to_slot_num)
            .map_err(BankAuthorityMutationError0104::Slot)?;
        validate_authoritative_bank_item(packet.from_slot_item, from)?;
        validate_authoritative_bank_item(packet.to_slot_item, to)?;
        if from == to && packet.from_slot_item != packet.to_slot_item {
            return Err(BankAuthorityMutationError0104::ConflictingSameSlotValues {
                slot: from,
                first: packet.from_slot_item,
                second: packet.to_slot_item,
            });
        }

        self.write_authoritative_item(from, packet.from_slot_item);
        if to != from {
            self.write_authoritative_item(to, packet.to_slot_item);
        }
        Ok(BankAuthorityMutationReceipt0104 {
            primary: from,
            secondary: (to != from).then_some(to),
        })
    }

    pub(super) fn write_authoritative_item(&mut self, slot: BankSlotRef0104, item: ItemBase0104) {
        match slot.location {
            BankSlotLocation0104::Inventory => self.inventory[slot.index] = item,
            BankSlotLocation0104::Bank => self.bank[slot.index] = item,
        }
    }

    pub(super) fn inventory_runtime(&self) -> InventoryRuntime0104 {
        let mut load = PcLoadData0104::zeroed();
        for (slot, item) in self.equipment.iter().copied().enumerate() {
            write_item_base_into_pc_load(
                load.as_bytes_mut(),
                PcLoadData0104::EQUIPMENT_OFFSET + slot * ItemBase0104::SIZE,
                item,
            );
        }
        for (slot, item) in self.inventory.iter().copied().enumerate() {
            write_item_base_into_pc_load(
                load.as_bytes_mut(),
                PcLoadData0104::INVENTORY_OFFSET + slot * ItemBase0104::SIZE,
                item,
            );
        }
        InventoryRuntime0104::from_pc_load(self.owner_pc_id, &load)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BankAuthorityMutationReceipt0104 {
    pub primary: BankSlotRef0104,
    pub secondary: Option<BankSlotRef0104>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankAuthorityMutationError0104 {
    Slot(BankSlotRefError0104),
    MalformedItemIdentity {
        slot: BankSlotRef0104,
        item_type: i16,
        item_id: i16,
    },
    ConflictingSameSlotValues {
        slot: BankSlotRef0104,
        first: ItemBase0104,
        second: ItemBase0104,
    },
}

impl fmt::Display for BankAuthorityMutationError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Slot(error) => write!(f, "authoritative bank move has invalid slot: {error}"),
            Self::MalformedItemIdentity {
                slot,
                item_type,
                item_id,
            } => write!(
                f,
                "authoritative bank move has malformed type={item_type} id={item_id} for {:?} slot {}",
                slot.location(),
                slot.index()
            ),
            Self::ConflictingSameSlotValues {
                slot,
                first,
                second,
            } => write!(
                f,
                "authoritative bank move carries two values for {:?} slot {}: {first:?} and {second:?}",
                slot.location(),
                slot.index()
            ),
        }
    }
}

impl Error for BankAuthorityMutationError0104 {}

/// Authoritative currency consumed by the shared clean
/// `Panel_PCStuffScript` owner while BankMode is open.
///
/// Clean `cnOwnAvatarStatus.SetLoadData` copies `sPCLoadData2CL.iCandy` into
/// `iTaros`; `Panel_PCStuffScript.OnGUI` then reads that value without local
/// arithmetic. Keeping it separate from the 9+50 item snapshot prevents a
/// bank move projection rebuild from inventing or resetting currency state.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct BankPcStuffAuthority0104 {
    pub(super) taros: i32,
}

impl BankPcStuffAuthority0104 {
    #[must_use]
    pub const fn from_authoritative_taros(taros: i32) -> Self {
        Self { taros }
    }

    #[must_use]
    pub fn from_pc_load(load: &PcLoadData0104) -> Self {
        Self::from_authoritative_taros(load.candy())
    }

    pub fn replace_from_pc_load(&mut self, load: &PcLoadData0104) {
        *self = Self::from_pc_load(load);
    }

    pub fn replace_authoritative_taros(&mut self, taros: i32) {
        self.taros = taros;
    }

    #[must_use]
    pub const fn taros(self) -> i32 {
        self.taros
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankProjectionError0104 {
    OwnerMismatch {
        expected_pc_id: i32,
        inventory_pc_id: i32,
    },
}

impl fmt::Display for BankProjectionError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OwnerMismatch {
                expected_pc_id,
                inventory_pc_id,
            } => write!(
                f,
                "bank projection belongs to PC {expected_pc_id}, inventory belongs to PC {inventory_pc_id}"
            ),
        }
    }
}

impl Error for BankProjectionError0104 {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BankTransferIntent0104 {
    pub from: BankSlotRef0104,
    pub to: BankSlotRef0104,
}

impl BankTransferIntent0104 {
    #[must_use]
    pub const fn wire_request(self) -> ItemMoveRequest0104 {
        ItemMoveRequest0104 {
            from_location: self.from.location.wire_value(),
            from_slot_num: self.from.wire_index(),
            to_location: self.to.location.wire_value(),
            to_slot_num: self.to.wire_index(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankTransferError0104 {
    SameSlotNoOp { slot: BankSlotRef0104 },
    LockedBankSlot { slot: BankSlotRef0104 },
    EmptySource { slot: BankSlotRef0104 },
    InventoryFull,
    BankFull,
}

impl fmt::Display for BankTransferError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SameSlotNoOp { slot } => write!(
                f,
                "bank move is a no-op at {:?} slot {}",
                slot.location(),
                slot.index()
            ),
            Self::LockedBankSlot { slot } => {
                write!(f, "bank slot {} is visible but locked", slot.index())
            }
            Self::EmptySource { slot } => write!(
                f,
                "{:?} slot {} has no authoritative source item",
                slot.location(),
                slot.index()
            ),
            Self::InventoryFull => f.write_str("general inventory has no empty slot"),
            Self::BankFull => f.write_str("accessible bank has no empty slot"),
        }
    }
}

impl Error for BankTransferError0104 {}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BankLifecyclePhase {
    #[default]
    Hidden,
    Opening,
    Visible,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankUiCommand0104 {
    Open(PcBankOpenRequest0104),
    ItemMove(ItemMoveRequest0104),
    ExitMode,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct BankUiOutbox0104 {
    pub(super) commands: VecDeque<BankUiCommand0104>,
}

impl BankUiOutbox0104 {
    pub fn push(&mut self, command: BankUiCommand0104) {
        self.commands.push_back(command);
    }

    pub fn pop_front(&mut self) -> Option<BankUiCommand0104> {
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BankItemView {
    pub frame_visual: BankSlotFrameVisual,
    pub icon: UserEquipPresentationIcon,
    pub show_combined_badge: bool,
    pub count_label: Option<String>,
}

#[derive(Clone, Resource)]
pub(super) struct BankUiAssets {
    pub(super) images: [Handle<Image>; BankStaticAssetRole::COUNT],
    pub(super) font: Handle<Font>,
    pub(super) search_font: Handle<Font>,
    pub(super) missing_checker: Handle<Image>,
}

impl BankUiAssets {
    pub(super) fn load(
        asset_server: &AssetServer,
        images: &mut Assets<Image>,
        contract: &BankUiAssetContract,
    ) -> Self {
        Self {
            images: array::from_fn(|index| asset_server.load(contract.image_paths[index].clone())),
            font: asset_server.load(contract.font_path.clone()),
            search_font: asset_server.load(BANK_SEARCH_FONT_PATH),
            missing_checker: images.add(Image::new(
                Extent3d {
                    width: USER_EQUIP_MISSING_CHECKER_SIZE,
                    height: USER_EQUIP_MISSING_CHECKER_SIZE,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                user_equip_missing_checker_rgba(),
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            )),
        }
    }

    pub(super) fn image(&self, role: BankStaticAssetRole) -> Handle<Image> {
        self.images[role.index()].clone()
    }

    pub(super) fn all_loaded(&self, asset_server: &AssetServer) -> bool {
        self.images
            .iter()
            .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
            && matches!(asset_server.load_state(self.font.id()), LoadState::Loaded)
            && matches!(
                asset_server.load_state(self.search_font.id()),
                LoadState::Loaded
            )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct BankUiRoot;
