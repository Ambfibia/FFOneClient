use super::*;

pub const SYSTEM_MESSAGE_DIALOG_PATH: &str = "ui/en/gameplay/system/systemDialogBox.png";

pub const SYSTEM_MESSAGE_ITEM_BOX_PATH: &str = "ui/en/gameplay/journal/equipbox.png";

pub const SYSTEM_MESSAGE_WARNING_ICON_PATH: &str = "ui/en/gameplay/journal/messicon_warning.png";

pub const SYSTEM_MESSAGE_TRADE_ICON_PATH: &str = "ui/en/gameplay/system/messicon_trade.png";

pub const SYSTEM_MESSAGE_BUDDY_ICON_PATH: &str = "ui/en/gameplay/nanocom/messicon_buddy.png";

pub const SYSTEM_MESSAGE_GROUP_ICON_PATH: &str = "ui/en/gameplay/system/messicon_group.png";

pub const SYSTEM_MESSAGE_COMBI_ICON_PATH: &str = "ui/en/gameplay/system/combi_icon.png";

pub const SYSTEM_MESSAGE_COMBINED_BADGE_PATH: &str = "ui/en/user-equip/combineditemicon.png";

pub const SYSTEM_MESSAGE_MANAGER_COMPONENT_PATH_ID: i64 = 1_468;

pub const SYSTEM_MESSAGE_SKIN_PATH_ID: i64 = 1_380;

pub const SYSTEM_MESSAGE_ITEM_BOX_PATH_ID: i64 = 256;

pub const SYSTEM_MESSAGE_COMBINED_BADGE_PATH_ID: i64 = 636;

pub const SYSTEM_MESSAGE_FONT_PATH: &str = "fonts/jeffe.otf";

pub const SYSTEM_MESSAGE_BODY_FONT_PATH: &str = "fonts/chaletbook-regular.ttf";

pub const SYSTEM_MESSAGE_JEFFE_14_PATH_ID: i64 = 903;

pub const SYSTEM_MESSAGE_JEFFE_12_PATH_ID: i64 = 953;

pub const SYSTEM_MESSAGE_CHALET_SMALL_PATH_ID: i64 = 1_018;

pub const SYSTEM_MESSAGE_UI_Z_INDEX: i32 = i32::MAX - 24;

/// Exact serialized `cnSystemMessageManager.IconIndex` slots.
///
/// Clean path ID 1468 assigns textures only to warning/trade/buddy/group and
/// combi. Monkey, Scamp and Endsville/Past are deliberately null and must not
/// be filled from similarly named but unused textures.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum SystemMessageIconIndex {
    Warning = 0,
    Trade = 1,
    Buddy = 2,
    Group = 3,
    Monkey = 4,
    Scamp = 5,
    EndsvillePast = 6,
    Combi = 7,
}

impl SystemMessageIconIndex {
    #[must_use]
    pub const fn runtime_path(self) -> Option<&'static str> {
        match self {
            Self::Warning => Some(SYSTEM_MESSAGE_WARNING_ICON_PATH),
            Self::Trade => Some(SYSTEM_MESSAGE_TRADE_ICON_PATH),
            Self::Buddy => Some(SYSTEM_MESSAGE_BUDDY_ICON_PATH),
            Self::Group => Some(SYSTEM_MESSAGE_GROUP_ICON_PATH),
            Self::Monkey | Self::Scamp | Self::EndsvillePast => None,
            Self::Combi => Some(SYSTEM_MESSAGE_COMBI_ICON_PATH),
        }
    }
}

impl TryFrom<i32> for SystemMessageIconIndex {
    type Error = SystemMessageIconIndexError;

    fn try_from(raw: i32) -> Result<Self, Self::Error> {
        match raw {
            0 => Ok(Self::Warning),
            1 => Ok(Self::Trade),
            2 => Ok(Self::Buddy),
            3 => Ok(Self::Group),
            4 => Ok(Self::Monkey),
            5 => Ok(Self::Scamp),
            6 => Ok(Self::EndsvillePast),
            7 => Ok(Self::Combi),
            _ => Err(SystemMessageIconIndexError(raw)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemMessageIconAssetContract {
    pub index: SystemMessageIconIndex,
    pub source_path_id: i64,
    pub runtime_path: &'static str,
    pub png_sha256: &'static str,
}

pub const SYSTEM_MESSAGE_ICON_ASSET_CONTRACTS: [SystemMessageIconAssetContract; 5] = [
    SystemMessageIconAssetContract {
        index: SystemMessageIconIndex::Warning,
        source_path_id: 220,
        runtime_path: SYSTEM_MESSAGE_WARNING_ICON_PATH,
        png_sha256: "989F1CCC954E7EF3FD4A371FEC7B04350C0B084F2E894300C84EA2AE933CD71A",
    },
    SystemMessageIconAssetContract {
        index: SystemMessageIconIndex::Trade,
        source_path_id: 674,
        runtime_path: SYSTEM_MESSAGE_TRADE_ICON_PATH,
        png_sha256: "4D4BCC5991C7588841A24E2521292E29989D344703E8B46E6C0F6D47B102D1E8",
    },
    SystemMessageIconAssetContract {
        index: SystemMessageIconIndex::Buddy,
        source_path_id: 79,
        runtime_path: SYSTEM_MESSAGE_BUDDY_ICON_PATH,
        png_sha256: "D18035DB5DFC41245E124C523D85E709F3A05DE7B4A4696AD07A2CCF860BD763",
    },
    SystemMessageIconAssetContract {
        index: SystemMessageIconIndex::Group,
        source_path_id: 26,
        runtime_path: SYSTEM_MESSAGE_GROUP_ICON_PATH,
        png_sha256: "1D1DA5991350860038947855A6BCA12F08B6CFB3FD92A0AB22F9C80C407FC11A",
    },
    SystemMessageIconAssetContract {
        index: SystemMessageIconIndex::Combi,
        source_path_id: 288,
        runtime_path: SYSTEM_MESSAGE_COMBI_ICON_PATH,
        png_sha256: "6FF2AAB9ABFACB6C52CD14B34E3748D389A69BBB0ECAD8CEACF3D9F5B3466A79",
    },
];
