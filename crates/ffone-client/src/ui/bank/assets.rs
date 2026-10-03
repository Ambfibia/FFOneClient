use super::*;

pub const BANK_GAME_OBJECT_PATH_ID: i64 = 1_279;

pub const BANK_CONTROLLER_COMPONENT_PATH_ID: i64 = 1_586;

pub const BANK_CONTROLLER_SCRIPT_PATH_ID: i64 = 919;

pub const BANK_PANEL_COMPONENT_PATH_ID: i64 = 1_587;

pub const BANK_PANEL_SCRIPT_PATH_ID: i64 = 1_162;

pub const BANK_PC_STUFF_COMPONENT_PATH_ID: i64 = 1_588;

pub const BANK_PC_STUFF_SCRIPT_PATH_ID: i64 = 1_045;

pub const BANK_EQUIP_COMPONENT_PATH_ID: i64 = 1_589;

pub const BANK_EQUIP_SCRIPT_PATH_ID: i64 = 1_027;

pub const BANK_INVENTORY_MANAGER_COMPONENT_PATH_ID: i64 = 1_419;

pub const BANK_INVENTORY_SKIN_PATH_ID: i64 = 1_366;

pub const BANK_TRADE_BACK_PATH: &str = "ui/en/bank-mode/trade-back.png";

pub const BANK_PANEL_PATH: &str = "ui/en/bank-mode/bank-panel.png";

pub const BANK_INFO_PATH: &str = "ui/en/bank-mode/bank-info.png";

pub const BANK_LOCKED_SLOT_PATH: &str = "ui/en/bank-mode/locked-slot.png";

/// Clean `Panel_PCStuffScript` reuses these two serialized
/// `InventoryManagerScript` textures in the actually-called
/// `localized.newInv == 1` Bank branch.
pub const BANK_DEXLABS_PATH: &str = "ui/en/enchant/dexlabsbut.png";

pub const BANK_DEXLABS_SOURCE_PATH_ID: i64 = 451;

pub const BANK_TAROS_COUNTER_PATH: &str = "ui/en/enchant/Taros.png";

pub const BANK_TAROS_COUNTER_SOURCE_PATH_ID: i64 = 326;

pub const BANK_UI_Z_INDEX: i32 = 19;

pub const BANK_LABEL_SOURCE_FONT_PATH_ID: i64 = 977;

pub const BANK_EQUIP_SOURCE_FONT_PATH_ID: i64 = 970;

/// Serialized `bankslotback` owns Chalet pathId 949, but `BeginGroup` receives
/// no text. Every actually rendered BankMode string instead comes from
/// `label`, `blankbox`, `equipbar`, or `equipfont` and uses JEFFE 12/06.
pub const BANK_BACKGROUND_ONLY_FONT_PATH_ID: i64 = 949;

pub(super) struct EmptyBankCatalog;

impl UserEquipItemCatalog for EmptyBankCatalog {
    fn resolve_icon(&self, _query: UserEquipCatalogQuery) -> Option<UserEquipIconRef> {
        None
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BankStaticAssetRole {
    Backdrop,
    TradeBack,
    RightBackplate,
    BankPanel,
    BankInfo,
    BankSlotButton,
    InventoryPanel,
    SlotOccupied,
    SlotEmpty,
    LockedSlot,
    Combined,
    ScrollTrack,
    ScrollThumb,
    ScrollUp,
    ScrollDown,
    ScrollShadow,
    EquipTitle,
    DexlabsBanner,
    TarosCounter,
    Close,
    Trash,
    Help,
    ButtonHover,
    SearchField,
    SearchInputBackground,
}

impl BankStaticAssetRole {
    pub const COUNT: usize = 25;

    pub(super) const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct BankUiAssetContract {
    pub(super) image_paths: [String; BankStaticAssetRole::COUNT],
    pub(super) font_path: String,
}

impl Default for BankUiAssetContract {
    fn default() -> Self {
        Self {
            image_paths: BANK_UI_DEFAULT_IMAGE_PATHS.map(str::to_owned),
            font_path: USER_EQUIP_FONT_PATH.to_owned(),
        }
    }
}

impl BankUiAssetContract {
    pub fn new(
        image_paths: [String; BankStaticAssetRole::COUNT],
        font_path: String,
    ) -> Result<Self, BankUiAssetContractError> {
        for (index, path) in image_paths.iter().enumerate() {
            if !is_safe_relative_asset_path(path) {
                return Err(BankUiAssetContractError::UnsafeImagePath {
                    index,
                    path: path.clone(),
                });
            }
        }
        if !is_safe_relative_asset_path(&font_path) {
            return Err(BankUiAssetContractError::UnsafeFontPath { path: font_path });
        }
        Ok(Self {
            image_paths,
            font_path,
        })
    }

    #[must_use]
    pub fn image_path(&self, role: BankStaticAssetRole) -> &str {
        &self.image_paths[role.index()]
    }

    #[must_use]
    pub fn font_path(&self) -> &str {
        &self.font_path
    }

    #[must_use]
    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.image_paths
            .iter()
            .map(String::as_str)
            .chain(std::iter::once(self.font_path.as_str()))
            .chain(std::iter::once(BANK_SEARCH_FONT_PATH))
    }
}

pub(super) fn is_safe_relative_asset_path(path: &str) -> bool {
    !path.is_empty()
        && !path.contains('\\')
        && !path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
}

pub const BANK_SEARCH_FONT_PATH: &str = "fonts/chaletbook-regular.ttf";
