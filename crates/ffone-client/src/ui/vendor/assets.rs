use super::*;

pub const VENDOR_GAME_OBJECT_PATH_ID: i64 = 1_324;

pub const VENDOR_CN_VENDOR_COMPONENT_PATH_ID: i64 = 1_409;

pub const VENDOR_CN_VENDOR_SCRIPT_PATH_ID: i64 = 1_016;

pub const VENDOR_PC_STUFF_COMPONENT_PATH_ID: i64 = 1_410;

pub const VENDOR_PC_STUFF_SCRIPT_PATH_ID: i64 = 1_045;

pub const VENDOR_PANEL_COMPONENT_PATH_ID: i64 = 1_411;

pub const VENDOR_PANEL_SCRIPT_PATH_ID: i64 = 1_097;

pub const VENDOR_EQUIP_COMPONENT_PATH_ID: i64 = 1_412;

pub const VENDOR_EQUIP_SCRIPT_PATH_ID: i64 = 1_027;

pub const VENDOR_INVENTORY_MANAGER_COMPONENT_PATH_ID: i64 = 1_419;

pub const VENDOR_INVENTORY_SKIN_PATH_ID: i64 = 1_366;

pub const VENDOR_PANEL_PATH: &str = "ui/en/vendor/vendor-panel.png";

pub const VENDOR_INFO_PATH: &str = "ui/en/vendor/info.png";

pub const VENDOR_LIST_BACK_PATH: &str = "ui/en/vendor/list-back.png";

pub const VENDOR_LIST_DIVIDER_PATH: &str = "ui/en/vendor/list-divider.png";

pub const VENDOR_ITEM_ROW_PATH: &str = "ui/en/vendor/item-row.png";

pub const VENDOR_TAB_BUY_SELECTED_PATH: &str = "ui/en/vendor/tab-buy-selected.png";

pub const VENDOR_TAB_BUY_NORMAL_PATH: &str = "ui/en/vendor/tab-buy-normal.png";

pub const VENDOR_TAB_BUYBACK_SELECTED_PATH: &str = "ui/en/vendor/tab-buyback-selected.png";

pub const VENDOR_TAB_BUYBACK_NORMAL_PATH: &str = "ui/en/vendor/tab-buyback-normal.png";

pub const VENDOR_TAROS_ICON_PATH: &str = "ui/en/vendor/taros-icon.png";

/// Actually-called `localized.newInv == eUse` shared PCStuff branch.
pub const VENDOR_DEXLABS_PATH: &str = "ui/en/enchant/dexlabsbut.png";

pub const VENDOR_DEXLABS_SOURCE_PATH_ID: i64 = 451;

pub const VENDOR_TAROS_COUNTER_PATH: &str = "ui/en/enchant/Taros.png";

pub const VENDOR_TAROS_COUNTER_SOURCE_PATH_ID: i64 = 326;

pub const VENDOR_BOOST_ICON_PATH: &str = "ui/en/enchant/BoostIcon.png";

pub const VENDOR_BOOST_ICON_SOURCE_PATH_ID: i64 = 87;

pub const VENDOR_POTION_ICON_PATH: &str = "ui/en/enchant/PostionIcon.png";

pub const VENDOR_POTION_ICON_SOURCE_PATH_ID: i64 = 99;

pub const VENDOR_SERVICE_FONT_PATH: &str = "fonts/chaletbook-regular.ttf";

pub const VENDOR_LABEL_SOURCE_FONT_PATH_ID: i64 = 977;

pub const VENDOR_EQUIP_SOURCE_FONT_PATH_ID: i64 = 970;

pub const VENDOR_SERVICE_SOURCE_FONT_PATH_ID: i64 = 949;

pub const VENDOR_CATALOG_CAPACITY: usize = 50;

pub const VENDOR_UI_Z_INDEX: i32 = 20;

pub(super) fn is_safe_relative_asset_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.contains(':')
        && !path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VendorCatalogEntry0104 {
    /// Clean local slot type 9 `SlotID`. The serialized table update supplies
    /// at most fifty entries.
    pub source_slot_id: usize,
    pub item: ItemBase0104,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VendorCatalogRowProjection0104 {
    pub packet_index: usize,
    pub source_slot_id: usize,
    pub item: ItemBase0104,
    pub metadata: Option<VendorItemMetadata0104>,
    pub icon: VendorPresentationIcon0104,
    pub price: Option<i32>,
    pub affordable: Option<bool>,
    pub equip_validation: VendorEquipValidation0104,
    pub vehicle_speed_class: Option<i32>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VendorStaticAssetRole {
    Backdrop,
    VendorBackplate,
    RightBackplate,
    Info,
    ListBack,
    ListDivider,
    ItemRow,
    BuySelected,
    BuyNormal,
    BuyHover,
    BuybackSelected,
    BuybackNormal,
    BuybackHover,
    Taros,
    ButtonNormal,
    ButtonHover,
    Restricted,
    ScrollTrack,
    ScrollThumb,
    ScrollUp,
    ScrollDown,
    ScrollShadow,
    InventoryPanel,
    SlotOccupied,
    SlotEmpty,
    Combined,
    EquipTitle,
    Close,
    Trash,
    Help,
    DexlabsBanner,
    TarosCounter,
    BoostIcon,
    PotionIcon,
}

impl VendorStaticAssetRole {
    pub const COUNT: usize = 34;

    pub(super) const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct VendorUiAssetContract {
    pub(super) image_paths: [String; VendorStaticAssetRole::COUNT],
    pub(super) font_path: String,
    pub(super) service_font_path: String,
}

impl Default for VendorUiAssetContract {
    fn default() -> Self {
        Self {
            image_paths: VENDOR_UI_DEFAULT_IMAGE_PATHS.map(str::to_owned),
            font_path: USER_EQUIP_FONT_PATH.to_owned(),
            service_font_path: VENDOR_SERVICE_FONT_PATH.to_owned(),
        }
    }
}

impl VendorUiAssetContract {
    pub fn new(
        image_paths: [String; VendorStaticAssetRole::COUNT],
        font_path: String,
    ) -> Result<Self, VendorUiAssetContractError> {
        for (index, path) in image_paths.iter().enumerate() {
            if !is_safe_relative_asset_path(path) {
                return Err(VendorUiAssetContractError::UnsafeImagePath {
                    index,
                    path: path.clone(),
                });
            }
        }
        if !is_safe_relative_asset_path(&font_path) {
            return Err(VendorUiAssetContractError::UnsafeFontPath { path: font_path });
        }
        Ok(Self {
            image_paths,
            font_path,
            service_font_path: VENDOR_SERVICE_FONT_PATH.to_owned(),
        })
    }

    #[must_use]
    pub fn image_path(&self, role: VendorStaticAssetRole) -> &str {
        &self.image_paths[role.index()]
    }

    #[must_use]
    pub fn font_path(&self) -> &str {
        &self.font_path
    }

    #[must_use]
    pub fn service_font_path(&self) -> &str {
        &self.service_font_path
    }

    #[must_use]
    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.image_paths
            .iter()
            .map(String::as_str)
            .chain([self.font_path.as_str(), self.service_font_path.as_str()])
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum VendorStaticAssetReadiness {
    #[default]
    Loading,
    Ready,
    Failed,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct VendorUiAssetStatus(pub VendorStaticAssetReadiness);
