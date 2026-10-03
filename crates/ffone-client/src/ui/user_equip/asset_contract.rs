//! Static asset roles, default routes and the asset path contract.

use super::asset_paths::{
    USER_EQUIP_BACKDROP_PATH, USER_EQUIP_BOOST_ICON_PATH, USER_EQUIP_BUTTON_HOVER_PATH,
    USER_EQUIP_BUTTON_NORMAL_PATH, USER_EQUIP_CALCULATOR_BACK_PATH, USER_EQUIP_CLOSE_HOVER_PATH,
    USER_EQUIP_CLOSE_PATH, USER_EQUIP_CLOTHES_PANEL_PATH, USER_EQUIP_COMBINED_PATH,
    USER_EQUIP_DEXLABS_PATH, USER_EQUIP_EQUIP_INFO_PATH, USER_EQUIP_EQUIP_POPUP_PATH,
    USER_EQUIP_EQUIP_TITLE_PATH, USER_EQUIP_FONT_PATH, USER_EQUIP_FUSION_MATTER_BAR_PATH,
    USER_EQUIP_GENERAL_DIALOG_PATH, USER_EQUIP_GUIDE_BOX_PATH, USER_EQUIP_HELP_HOVER_PATH,
    USER_EQUIP_HELP_PATH, USER_EQUIP_HP_BACK_PATH, USER_EQUIP_HP_BAR_PATH,
    USER_EQUIP_INVENTORY_PANEL_PATH, USER_EQUIP_INVENTORY_SHADOW_PATH,
    USER_EQUIP_ITEM_TAB_HOVER_PATH, USER_EQUIP_ITEM_TAB_PATH, USER_EQUIP_NANO_BACK_PATH,
    USER_EQUIP_NANO_BLUE_PATH, USER_EQUIP_NANO_DIALOG_PATH, USER_EQUIP_NANO_FM_BAR_PATH,
    USER_EQUIP_NANO_ITEM_BAR_PATH, USER_EQUIP_NANO_POPUP_EQUIPPED_PATH,
    USER_EQUIP_NANO_POPUP_NEXT_PATH, USER_EQUIP_NANO_POPUP_PATH, USER_EQUIP_NANO_RED_PATH,
    USER_EQUIP_NANO_TAB_HOVER_PATH, USER_EQUIP_NANO_TAB_PATH, USER_EQUIP_NANO_YELLOW_PATH,
    USER_EQUIP_POTION_ICON_PATH, USER_EQUIP_RED_BUTTON_HOVER_PATH, USER_EQUIP_RED_BUTTON_PATH,
    USER_EQUIP_RIGHT_PANEL_PATH, USER_EQUIP_SCROLL_DOWN_PATH, USER_EQUIP_SCROLL_THUMB_PATH,
    USER_EQUIP_SCROLL_TRACK_PATH, USER_EQUIP_SCROLL_UP_PATH, USER_EQUIP_SLOT_EMPTY_PATH,
    USER_EQUIP_SLOT_OCCUPIED_PATH, USER_EQUIP_TAROS_PATH, USER_EQUIP_TRASH_HOVER_PATH,
    USER_EQUIP_TRASH_PATH, USER_EQUIP_TURN_LEFT_HOVER_PATH, USER_EQUIP_TURN_LEFT_PATH,
    USER_EQUIP_TURN_RIGHT_HOVER_PATH, USER_EQUIP_TURN_RIGHT_PATH, USER_EQUIP_UNEQUIP_POPUP_PATH,
    USER_EQUIP_USE_DIALOG_PATH, USER_EQUIP_USER_STATUS_PANEL_PATH,
};
use bevy::prelude::*;
use std::{array, error::Error, fmt};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UserEquipStaticAssetRole {
    Backdrop,
    ClothesPanel,
    RightPanel,
    InventoryPanel,
    SlotOccupied,
    SlotEmpty,
    EquipTitle,
    NanoTab,
    Close,
    Trash,
    Help,
    Combined,
    ItemTab,
    BoostIcon,
    PotionIcon,
    Taros,
    EquipPopup,
    UnequipPopup,
    ItemTabHover,
    NanoTabHover,
    CloseHover,
    TrashHover,
    HelpHover,
    UserStatusPanel,
    HpBack,
    HpBar,
    FusionMatterBar,
    GuideBox,
    InventoryShadow,
    Dexlabs,
    NanoBack,
    NanoDialog,
    NanoBlue,
    NanoRed,
    NanoYellow,
    NanoPopup,
    NanoPopupNext,
    TurnLeft,
    TurnLeftHover,
    TurnRight,
    TurnRightHover,
    EquipInfo,
    NanoPopupEquipped,
    ButtonNormal,
    ButtonHover,
    NanoFmBar,
    NanoItemBar,
    ScrollTrack,
    ScrollUp,
    ScrollDown,
    ScrollThumb,
    UseDialog,
    GeneralDialog,
    CalculatorBack,
    RedButton,
    RedButtonHover,
    Font,
}

impl UserEquipStaticAssetRole {
    pub const ALL: [Self; 57] = [
        Self::Backdrop,
        Self::ClothesPanel,
        Self::RightPanel,
        Self::InventoryPanel,
        Self::SlotOccupied,
        Self::SlotEmpty,
        Self::EquipTitle,
        Self::NanoTab,
        Self::Close,
        Self::Trash,
        Self::Help,
        Self::Combined,
        Self::ItemTab,
        Self::BoostIcon,
        Self::PotionIcon,
        Self::Taros,
        Self::EquipPopup,
        Self::UnequipPopup,
        Self::ItemTabHover,
        Self::NanoTabHover,
        Self::CloseHover,
        Self::TrashHover,
        Self::HelpHover,
        Self::UserStatusPanel,
        Self::HpBack,
        Self::HpBar,
        Self::FusionMatterBar,
        Self::GuideBox,
        Self::InventoryShadow,
        Self::Dexlabs,
        Self::NanoBack,
        Self::NanoDialog,
        Self::NanoBlue,
        Self::NanoRed,
        Self::NanoYellow,
        Self::NanoPopup,
        Self::NanoPopupNext,
        Self::TurnLeft,
        Self::TurnLeftHover,
        Self::TurnRight,
        Self::TurnRightHover,
        Self::EquipInfo,
        Self::NanoPopupEquipped,
        Self::ButtonNormal,
        Self::ButtonHover,
        Self::NanoFmBar,
        Self::NanoItemBar,
        Self::ScrollTrack,
        Self::ScrollUp,
        Self::ScrollDown,
        Self::ScrollThumb,
        Self::UseDialog,
        Self::GeneralDialog,
        Self::CalculatorBack,
        Self::RedButton,
        Self::RedButtonHover,
        Self::Font,
    ];

    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Backdrop => 0,
            Self::ClothesPanel => 1,
            Self::RightPanel => 2,
            Self::InventoryPanel => 3,
            Self::SlotOccupied => 4,
            Self::SlotEmpty => 5,
            Self::EquipTitle => 6,
            Self::NanoTab => 7,
            Self::Close => 8,
            Self::Trash => 9,
            Self::Help => 10,
            Self::Combined => 11,
            Self::ItemTab => 12,
            Self::BoostIcon => 13,
            Self::PotionIcon => 14,
            Self::Taros => 15,
            Self::EquipPopup => 16,
            Self::UnequipPopup => 17,
            Self::ItemTabHover => 18,
            Self::NanoTabHover => 19,
            Self::CloseHover => 20,
            Self::TrashHover => 21,
            Self::HelpHover => 22,
            Self::UserStatusPanel => 23,
            Self::HpBack => 24,
            Self::HpBar => 25,
            Self::FusionMatterBar => 26,
            Self::GuideBox => 27,
            Self::InventoryShadow => 28,
            Self::Dexlabs => 29,
            Self::NanoBack => 30,
            Self::NanoDialog => 31,
            Self::NanoBlue => 32,
            Self::NanoRed => 33,
            Self::NanoYellow => 34,
            Self::NanoPopup => 35,
            Self::NanoPopupNext => 36,
            Self::TurnLeft => 37,
            Self::TurnLeftHover => 38,
            Self::TurnRight => 39,
            Self::TurnRightHover => 40,
            Self::EquipInfo => 41,
            Self::NanoPopupEquipped => 42,
            Self::ButtonNormal => 43,
            Self::ButtonHover => 44,
            Self::NanoFmBar => 45,
            Self::NanoItemBar => 46,
            Self::ScrollTrack => 47,
            Self::ScrollUp => 48,
            Self::ScrollDown => 49,
            Self::ScrollThumb => 50,
            Self::UseDialog => 51,
            Self::GeneralDialog => 52,
            Self::CalculatorBack => 53,
            Self::RedButton => 54,
            Self::RedButtonHover => 55,
            Self::Font => 56,
        }
    }

    #[must_use]
    pub const fn is_font(self) -> bool {
        matches!(self, Self::Font)
    }
}

pub const USER_EQUIP_DEFAULT_ASSET_PATHS: [&str; 57] = [
    USER_EQUIP_BACKDROP_PATH,
    USER_EQUIP_CLOTHES_PANEL_PATH,
    USER_EQUIP_RIGHT_PANEL_PATH,
    USER_EQUIP_INVENTORY_PANEL_PATH,
    USER_EQUIP_SLOT_OCCUPIED_PATH,
    USER_EQUIP_SLOT_EMPTY_PATH,
    USER_EQUIP_EQUIP_TITLE_PATH,
    USER_EQUIP_NANO_TAB_PATH,
    USER_EQUIP_CLOSE_PATH,
    USER_EQUIP_TRASH_PATH,
    USER_EQUIP_HELP_PATH,
    USER_EQUIP_COMBINED_PATH,
    USER_EQUIP_ITEM_TAB_PATH,
    USER_EQUIP_BOOST_ICON_PATH,
    USER_EQUIP_POTION_ICON_PATH,
    USER_EQUIP_TAROS_PATH,
    USER_EQUIP_EQUIP_POPUP_PATH,
    USER_EQUIP_UNEQUIP_POPUP_PATH,
    USER_EQUIP_ITEM_TAB_HOVER_PATH,
    USER_EQUIP_NANO_TAB_HOVER_PATH,
    USER_EQUIP_CLOSE_HOVER_PATH,
    USER_EQUIP_TRASH_HOVER_PATH,
    USER_EQUIP_HELP_HOVER_PATH,
    USER_EQUIP_USER_STATUS_PANEL_PATH,
    USER_EQUIP_HP_BACK_PATH,
    USER_EQUIP_HP_BAR_PATH,
    USER_EQUIP_FUSION_MATTER_BAR_PATH,
    USER_EQUIP_GUIDE_BOX_PATH,
    USER_EQUIP_INVENTORY_SHADOW_PATH,
    USER_EQUIP_DEXLABS_PATH,
    USER_EQUIP_NANO_BACK_PATH,
    USER_EQUIP_NANO_DIALOG_PATH,
    USER_EQUIP_NANO_BLUE_PATH,
    USER_EQUIP_NANO_RED_PATH,
    USER_EQUIP_NANO_YELLOW_PATH,
    USER_EQUIP_NANO_POPUP_PATH,
    USER_EQUIP_NANO_POPUP_NEXT_PATH,
    USER_EQUIP_TURN_LEFT_PATH,
    USER_EQUIP_TURN_LEFT_HOVER_PATH,
    USER_EQUIP_TURN_RIGHT_PATH,
    USER_EQUIP_TURN_RIGHT_HOVER_PATH,
    USER_EQUIP_EQUIP_INFO_PATH,
    USER_EQUIP_NANO_POPUP_EQUIPPED_PATH,
    USER_EQUIP_BUTTON_NORMAL_PATH,
    USER_EQUIP_BUTTON_HOVER_PATH,
    USER_EQUIP_NANO_FM_BAR_PATH,
    USER_EQUIP_NANO_ITEM_BAR_PATH,
    USER_EQUIP_SCROLL_TRACK_PATH,
    USER_EQUIP_SCROLL_UP_PATH,
    USER_EQUIP_SCROLL_DOWN_PATH,
    USER_EQUIP_SCROLL_THUMB_PATH,
    USER_EQUIP_USE_DIALOG_PATH,
    USER_EQUIP_GENERAL_DIALOG_PATH,
    USER_EQUIP_CALCULATOR_BACK_PATH,
    USER_EQUIP_RED_BUTTON_PATH,
    USER_EQUIP_RED_BUTTON_HOVER_PATH,
    USER_EQUIP_FONT_PATH,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserEquipAssetContractError {
    pub role: UserEquipStaticAssetRole,
    pub path: String,
}

impl fmt::Display for UserEquipAssetContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:?} must use a relative semantic UserEquip asset path: {}",
            self.role, self.path
        )
    }
}

impl Error for UserEquipAssetContractError {}

#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct UserEquipUiAssetContract {
    pub(super) paths: [String; 57],
}

impl Default for UserEquipUiAssetContract {
    fn default() -> Self {
        Self::try_from_semantic_paths(USER_EQUIP_DEFAULT_ASSET_PATHS)
            .expect("built-in UserEquip paths must remain semantic")
    }
}

impl UserEquipUiAssetContract {
    pub fn try_from_semantic_paths<P>(paths: [P; 57]) -> Result<Self, UserEquipAssetContractError>
    where
        P: Into<String>,
    {
        let paths = paths.map(Into::into);
        for role in UserEquipStaticAssetRole::ALL {
            let path = &paths[role.index()];
            if !is_user_equip_static_asset_path(role, path) {
                return Err(UserEquipAssetContractError {
                    role,
                    path: path.clone(),
                });
            }
        }
        Ok(Self { paths })
    }

    #[must_use]
    pub fn path(&self, role: UserEquipStaticAssetRole) -> &str {
        &self.paths[role.index()]
    }

    #[must_use]
    pub fn paths(&self) -> [&str; 57] {
        array::from_fn(|index| self.paths[index].as_str())
    }
}

pub(super) fn is_user_equip_static_asset_path(role: UserEquipStaticAssetRole, path: &str) -> bool {
    if !is_safe_relative_asset_path(path) {
        return false;
    }
    if role.is_font() {
        path.starts_with("fonts/") && (path.ends_with(".otf") || path.ends_with(".ttf"))
    } else {
        path.starts_with("ui/en/user-equip/") && path.ends_with(".png")
    }
}

#[must_use]
pub fn is_user_equip_semantic_icon_path(path: &str) -> bool {
    path.starts_with("icons/") && path.ends_with(".png") && is_safe_relative_asset_path(path)
}

pub(super) fn is_safe_relative_asset_path(path: &str) -> bool {
    !path.is_empty()
        && !path.contains('\\')
        && !path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
}
