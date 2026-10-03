use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnchantAbiScalar0104 {
    I16,
    I32,
    ItemBase,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnchantAbiField0104 {
    pub name: &'static str,
    pub offset: usize,
    pub scalar: EnchantAbiScalar0104,
    pub count: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EnchantRequest0104 {
    pub enchant_item_slot: i32,
    pub weapon_material_item_slot: i32,
    pub defence_material_item_slot: i32,
    pub cash_item_slot_1: i32,
    pub cash_item_slot_2: i32,
}

impl EnchantRequest0104 {
    #[must_use]
    pub fn encode(self) -> [u8; ENCHANT_REQUEST_PACKET_SIZE_0104] {
        let mut bytes = [0; ENCHANT_REQUEST_PACKET_SIZE_0104];
        write_i32(&mut bytes, 0, self.enchant_item_slot);
        write_i32(&mut bytes, 4, self.weapon_material_item_slot);
        write_i32(&mut bytes, 8, self.defence_material_item_slot);
        write_i32(&mut bytes, 12, self.cash_item_slot_1);
        write_i32(&mut bytes, 16, self.cash_item_slot_2);
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, EnchantPacketError0104> {
        exact_packet_size(bytes, ENCHANT_REQUEST_PACKET_SIZE_0104)?;
        Ok(Self {
            enchant_item_slot: read_i32(bytes, 0),
            weapon_material_item_slot: read_i32(bytes, 4),
            defence_material_item_slot: read_i32(bytes, 8),
            cash_item_slot_1: read_i32(bytes, 12),
            cash_item_slot_2: read_i32(bytes, 16),
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EnchantDeleteRequest0104 {
    pub inventory_location: i32,
    pub item_slot: i32,
}

impl EnchantDeleteRequest0104 {
    #[must_use]
    pub fn encode(self) -> [u8; ENCHANT_DELETE_REQUEST_PACKET_SIZE_0104] {
        let mut bytes = [0; ENCHANT_DELETE_REQUEST_PACKET_SIZE_0104];
        write_i32(&mut bytes, 0, self.inventory_location);
        write_i32(&mut bytes, 4, self.item_slot);
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, EnchantPacketError0104> {
        exact_packet_size(bytes, ENCHANT_DELETE_REQUEST_PACKET_SIZE_0104)?;
        Ok(Self {
            inventory_location: read_i32(bytes, 0),
            item_slot: read_i32(bytes, 4),
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EnchantDisassembleRequest0104 {
    pub item_slot: i32,
}

impl EnchantDisassembleRequest0104 {
    #[must_use]
    pub fn encode(self) -> [u8; ENCHANT_DISASSEMBLE_REQUEST_PACKET_SIZE_0104] {
        self.item_slot.to_le_bytes()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, EnchantPacketError0104> {
        exact_packet_size(bytes, ENCHANT_DISASSEMBLE_REQUEST_PACKET_SIZE_0104)?;
        Ok(Self {
            item_slot: read_i32(bytes, 0),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnchantRedeemError0104 {
    TooShort,
    TooLong,
    ContainsSpace,
    Utf16Capacity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnchantRedeemWire0104 {
    pub packet_id: u32,
    pub request: FreeChatRequest0104,
    pub payload: [u8; ENCHANT_REDEEM_REQUEST_PACKET_SIZE_0104],
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EnchantFailure0104 {
    pub error_code: i32,
    pub enchant_item_slot: i32,
    pub weapon_material_item_slot: i32,
    pub defence_material_item_slot: i32,
    pub cash_item_slot_1: i32,
    pub cash_item_slot_2: i32,
}

impl EnchantFailure0104 {
    #[must_use]
    pub fn encode(self) -> [u8; ENCHANT_FAILURE_PACKET_SIZE_0104] {
        let mut bytes = [0; ENCHANT_FAILURE_PACKET_SIZE_0104];
        for (offset, value) in [
            (0, self.error_code),
            (4, self.enchant_item_slot),
            (8, self.weapon_material_item_slot),
            (12, self.defence_material_item_slot),
            (16, self.cash_item_slot_1),
            (20, self.cash_item_slot_2),
        ] {
            write_i32(&mut bytes, offset, value);
        }
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, EnchantPacketError0104> {
        exact_packet_size(bytes, ENCHANT_FAILURE_PACKET_SIZE_0104)?;
        Ok(Self {
            error_code: read_i32(bytes, 0),
            enchant_item_slot: read_i32(bytes, 4),
            weapon_material_item_slot: read_i32(bytes, 8),
            defence_material_item_slot: read_i32(bytes, 12),
            cash_item_slot_1: read_i32(bytes, 16),
            cash_item_slot_2: read_i32(bytes, 20),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnchantSuccess0104 {
    pub enchant_item_slot: i32,
    pub enchant_item: ItemBase0104,
    pub weapon_material_item_slot: i32,
    pub weapon_material_item: ItemBase0104,
    pub defence_material_item_slot: i32,
    pub defence_material_item: ItemBase0104,
    pub cash_item_slot_1: i32,
    pub cash_item_slot_2: i32,
    pub taros: i32,
    pub success_flag: i32,
}

impl Default for EnchantSuccess0104 {
    fn default() -> Self {
        Self {
            enchant_item_slot: 0,
            enchant_item: empty_enchant_item_0104(),
            weapon_material_item_slot: 0,
            weapon_material_item: empty_enchant_item_0104(),
            defence_material_item_slot: 0,
            defence_material_item: empty_enchant_item_0104(),
            cash_item_slot_1: 0,
            cash_item_slot_2: 0,
            taros: 0,
            success_flag: 0,
        }
    }
}

impl EnchantSuccess0104 {
    #[must_use]
    pub fn encode(self) -> [u8; ENCHANT_SUCCESS_PACKET_SIZE_0104] {
        let mut bytes = [0; ENCHANT_SUCCESS_PACKET_SIZE_0104];
        write_i32(&mut bytes, 0, self.enchant_item_slot);
        write_item_base(&mut bytes, 4, self.enchant_item);
        write_i32(&mut bytes, 16, self.weapon_material_item_slot);
        write_item_base(&mut bytes, 20, self.weapon_material_item);
        write_i32(&mut bytes, 32, self.defence_material_item_slot);
        write_item_base(&mut bytes, 36, self.defence_material_item);
        write_i32(&mut bytes, 48, self.cash_item_slot_1);
        write_i32(&mut bytes, 52, self.cash_item_slot_2);
        write_i32(&mut bytes, 56, self.taros);
        write_i32(&mut bytes, 60, self.success_flag);
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, EnchantPacketError0104> {
        exact_packet_size(bytes, ENCHANT_SUCCESS_PACKET_SIZE_0104)?;
        Ok(Self {
            enchant_item_slot: read_i32(bytes, 0),
            enchant_item: read_item_base(bytes, 4),
            weapon_material_item_slot: read_i32(bytes, 16),
            weapon_material_item: read_item_base(bytes, 20),
            defence_material_item_slot: read_i32(bytes, 32),
            defence_material_item: read_item_base(bytes, 36),
            cash_item_slot_1: read_i32(bytes, 48),
            cash_item_slot_2: read_i32(bytes, 52),
            taros: read_i32(bytes, 56),
            success_flag: read_i32(bytes, 60),
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EnchantRecipe0104 {
    pub enchant_grade: i32,
    pub cost: i32,
    pub class: i32,
    pub weapon_matter: i32,
    pub costume_matter: i32,
    pub probability: i32,
    pub offence_up: i32,
    pub defence_up: i32,
    pub fail_type: i32,
    pub no_drop_probability: i32,
    pub one_drop_probability: i32,
    pub two_drop_probability: i32,
    pub three_drop_probability: i32,
    pub four_drop_probability: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnchantChance0104 {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
    LegacyUnassigned,
}

impl EnchantChance0104 {
    #[must_use]
    pub const fn from_probability(probability: i32) -> Self {
        if probability < 10 {
            Self::VeryLow
        } else if probability < 30 {
            Self::Low
        } else if probability < 70 {
            Self::Medium
        } else if probability < 90 {
            Self::High
        } else if probability <= 100 {
            Self::VeryHigh
        } else {
            Self::LegacyUnassigned
        }
    }

    #[must_use]
    pub const fn level(self) -> i32 {
        match self {
            Self::VeryLow => 1,
            Self::Low => 2,
            Self::Medium => 3,
            Self::High => 4,
            Self::VeryHigh => 5,
            Self::LegacyUnassigned => 0,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::VeryLow => "1(Very Low)",
            Self::Low => "2(Low)",
            Self::Medium => "3(Medium)",
            Self::High => "4(High)",
            Self::VeryHigh => "5(Very High)",
            Self::LegacyUnassigned => "",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i16)]
pub enum EnchantTargetKind0104 {
    Hand = 0,
    UpperBody = 1,
    LowerBody = 2,
    Foot = 3,
}

impl EnchantTargetKind0104 {
    #[must_use]
    pub const fn from_item_type(item_type: i16) -> Option<Self> {
        match item_type {
            0 => Some(Self::Hand),
            1 => Some(Self::UpperBody),
            2 => Some(Self::LowerBody),
            3 => Some(Self::Foot),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EnchantRequirements0104 {
    pub recipe_index: usize,
    pub current_encoded_level: i32,
    pub displayed_level: i32,
    pub weapon_material_id: i16,
    pub weapon_material_count: i32,
    pub armor_material_id: i16,
    pub armor_material_count: i32,
    pub cost: i32,
    pub probability: i32,
    pub chance: Option<EnchantChance0104>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnchantProjectionError0104 {
    UnsupportedTargetItemType(i16),
    MissingRecipe { recipe_index: usize },
}

impl fmt::Display for EnchantProjectionError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedTargetItemType(item_type) => {
                write!(formatter, "item type {item_type} cannot be enchanted")
            }
            Self::MissingRecipe { recipe_index } => {
                write!(formatter, "clean Enchant table has no row {recipe_index}")
            }
        }
    }
}

impl Error for EnchantProjectionError0104 {}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EnchantItemPresentation0104 {
    pub name: String,
    pub description: String,
    pub icon_path: Option<String>,
    pub minimum_level: i32,
    pub cashable: i32,
    pub can_equip: bool,
    pub point_rating: i32,
    pub group_rating: i32,
    pub defense_rating: i32,
    pub type_label: String,
    pub range_label: String,
    pub rarity_label: String,
    pub trade_label: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnchantSelectableItem0104 {
    pub source_slot: i32,
    pub item: ItemBase0104,
    pub presentation: EnchantItemPresentation0104,
}

impl Default for EnchantSelectableItem0104 {
    fn default() -> Self {
        Self {
            source_slot: 0,
            item: empty_enchant_item_0104(),
            presentation: EnchantItemPresentation0104::default(),
        }
    }
}

impl EnchantSelectableItem0104 {
    #[must_use]
    pub const fn quantity(&self) -> i32 {
        self.item.option
    }

    #[must_use]
    pub const fn target_kind(&self) -> Option<EnchantTargetKind0104> {
        EnchantTargetKind0104::from_item_type(self.item.item_type)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EnchantSupportPresentation0104 {
    pub weapon_material: EnchantItemPresentation0104,
    pub armor_material: EnchantItemPresentation0104,
    pub helper_1: EnchantItemPresentation0104,
    pub helper_2: EnchantItemPresentation0104,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum EnchantAttachmentSlot0104 {
    Target = 21,
    WeaponMaterial = 22,
    ArmorMaterial = 23,
    Helper1 = 24,
    Helper2 = 25,
}

impl EnchantAttachmentSlot0104 {
    pub const ALL: [Self; 5] = [
        Self::Target,
        Self::WeaponMaterial,
        Self::ArmorMaterial,
        Self::Helper1,
        Self::Helper2,
    ];

    #[must_use]
    pub const fn index(self) -> usize {
        (self as i32 - Self::Target as i32) as usize
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnchantSelection0104 {
    pub(super) slots: [Option<EnchantSelectableItem0104>; 5],
    pub(super) attached: [bool; 5],
}

impl Default for EnchantSelection0104 {
    fn default() -> Self {
        Self {
            slots: array::from_fn(|_| None),
            attached: [false; 5],
        }
    }
}

impl EnchantSelection0104 {
    #[must_use]
    pub fn visual_item(
        &self,
        slot: EnchantAttachmentSlot0104,
    ) -> Option<&EnchantSelectableItem0104> {
        self.slots[slot.index()].as_ref()
    }

    #[must_use]
    pub fn visual_item_mut(
        &mut self,
        slot: EnchantAttachmentSlot0104,
    ) -> Option<&mut EnchantSelectableItem0104> {
        self.slots[slot.index()].as_mut()
    }

    #[must_use]
    pub const fn is_attached(&self, slot: EnchantAttachmentSlot0104) -> bool {
        self.attached[slot.index()]
    }

    #[must_use]
    pub fn any_attached(&self) -> bool {
        self.attached.iter().copied().any(|value| value)
    }

    #[must_use]
    pub fn any_orphaned(&self) -> bool {
        self.slots
            .iter()
            .zip(self.attached)
            .any(|(item, attached)| item.is_some() && !attached)
    }

    pub(super) fn set(&mut self, slot: EnchantAttachmentSlot0104, item: EnchantSelectableItem0104) {
        self.slots[slot.index()] = Some(item);
        self.attached[slot.index()] = true;
    }

    pub(super) fn take_visual(
        &mut self,
        slot: EnchantAttachmentSlot0104,
    ) -> Option<EnchantSelectableItem0104> {
        self.attached[slot.index()] = false;
        self.slots[slot.index()].take()
    }

    pub(super) fn clear_flag_only(&mut self, slot: EnchantAttachmentSlot0104) {
        self.attached[slot.index()] = false;
    }

    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EnchantExternalGates0104 {
    pub help_open: bool,
    pub inventory_popup_open: bool,
    pub system_popup_open: bool,
    pub item_popup_open: bool,
}

impl EnchantExternalGates0104 {
    #[must_use]
    pub const fn any(self) -> bool {
        self.help_open
            || self.inventory_popup_open
            || self.system_popup_open
            || self.item_popup_open
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnchantSystemCallback0104 {
    None,
    EnchantConfirmed,
    EnchantFailed,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EnchantPhase0104 {
    Closed,
    Ready,
    SystemMessage {
        message_id: i32,
        argument: Option<String>,
        callback: EnchantSystemCallback0104,
    },
    Waiting {
        elapsed_seconds: f32,
        request_token: u64,
    },
    AwaitingReply {
        request_token: u64,
    },
    Success {
        reply: EnchantSuccess0104,
    },
}

impl Default for EnchantPhase0104 {
    fn default() -> Self {
        Self::Closed
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnchantAuthoritativeReceipt0104 {
    pub taros: i32,
    pub mutations: Vec<EnchantInventoryMutation0104>,
    pub success_flag: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnchantCameraIntent0104 {
    BindPrimaryNpcAvatar {
        camera_path_id: i64,
        controller_path_id: i64,
        distance_millimetres: i32,
        height_millimetres: i32,
        euler_degrees: [i32; 3],
        target: EnchantCameraTarget0104,
    },
    BindWaitingNpcAvatar {
        camera_path_id: i64,
        controller_path_id: i64,
        distance_millimetres: i32,
        height_millimetres: i32,
        euler_degrees: [i32; 3],
        target: EnchantCameraTarget0104,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnchantCameraTarget0104 {
    Neck,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnchantLifecycleIntent0104 {
    InitializeInventoryMode {
        gui_mode: i32,
        active_inventory_tab: i32,
        show_inventory_panel: bool,
        show_equipment_panel: bool,
        inventory_event_dispatch: i32,
    },
    LoadPanelBackdrop {
        logical_path: &'static str,
    },
    RestoreGameplayInventory {
        inventory_event_dispatch: i32,
    },
    SetCursorLock(bool),
    ExitToMainGame,
    GoToMyStuff,
    EnterGameMode(i32),
    DispatchLegacyEvent {
        channel: i32,
        element_func: i32,
        argument: Option<i32>,
        dispatch: Option<i32>,
    },
    CheckFirstUse(i32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnchantPopupIntent0104 {
    SystemMessage {
        message_id: i32,
        callback: EnchantSystemCallback0104,
        argument: Option<String>,
    },
    Preview {
        gui_mode: i32,
        source_slot_type: i32,
        action: i32,
        popup_rect: (i32, i32, i32, i32),
        item: EnchantSelectableItem0104,
    },
    RedeemCode {
        window_id: i32,
        max_code_chars: usize,
    },
    RedeemCodeSpaceError {
        message: &'static str,
    },
}

impl EnchantPopupIntent0104 {
    /// Key-first copy for the only popup intent that owns literal text.
    /// System-message intents remain keyed by their clean table row IDs.
    #[must_use]
    pub fn localized_message(&self) -> Option<LocalizedText> {
        match self {
            Self::RedeemCodeSpaceError { message } => Some(LocalizedText::new(
                "ui.enchant.redeem.space_error",
                *message,
            )),
            Self::SystemMessage { .. } | Self::Preview { .. } | Self::RedeemCode { .. } => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnchantIntent0104 {
    Audio(EnchantAudioIntent0104),
    Animation(EnchantAnimationIntent0104),
    Camera(EnchantCameraIntent0104),
    Lifecycle(EnchantLifecycleIntent0104),
    Popup(EnchantPopupIntent0104),
    Selection(EnchantSelectionIntent0104),
    Wire {
        request_token: u64,
        packet_id: u32,
        payload: [u8; ENCHANT_REQUEST_PACKET_SIZE_0104],
        request: EnchantRequest0104,
    },
    RedeemWire(EnchantRedeemWire0104),
    AuthoritativeReceipt(EnchantAuthoritativeReceipt0104),
    RefreshInventory,
    TransportFailure(EnchantFailure0104),
    AuxiliaryUnlock(u32),
    IgnoredSuccessFlag(i32),
}

/// Exact reached text roles from clean `FusionFallCombi` (path 1367) and
/// `FusionFallInvenSkin` (path 1366).  The approved JEFFE/Chalet files replace
/// the legacy Unity Font objects only for glyph coverage; every serialized
/// font owner, line height, alignment, padding, wrapping and clipping choice
/// remains typed on the spawned `Text` entity.
#[derive(Clone, Copy, Component, Debug, Eq, Hash, PartialEq)]
pub enum EnchantUiTextStyle0104 {
    Jeff12LightBlueUpperLeft,
    Cha12YellowMiddleLeft,
    Jeff8LightBlueMiddleCenter,
    Jeff8LightBlueUpperLeft,
    Cha10SkyBlueMiddleLeft,
    Cha10LightBlueMiddleCenter,
    Jeff12SkyBlueMiddleLeft,
    Cha10LightBlueMiddleLeft,
    Jeff12LightBlueMiddleRight,
    Jeff40LightBlueMiddleCenter,
    Char12BlueMiddleCenter,
    Cha10BlueUpperLeft,
    Cha12YellowMiddleRight,
    EnchantLevelMiddleCenter,
    EnchantDefaultLabelUpperLeft,
    EnchantDefaultButtonMiddleCenter,
    InventoryLabelUpperLeft,
    InventoryBlankBoxUpperLeft,
    InventoryBlankBoxMiddleRight,
    InventoryButtonMiddleCenter,
    InventoryEquipBarMiddleCenter,
    InventoryEquipFontMiddleRight,
}
