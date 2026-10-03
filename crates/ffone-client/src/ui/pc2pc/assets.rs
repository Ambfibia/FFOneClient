use super::*;

pub const PC2PC_GAME_OBJECT_PATH_ID: i64 = 1_301;

pub const PC2PC_PANEL_COMPONENT_PATH_ID: i64 = 1_610;

pub const PC2PC_PANEL_SCRIPT_PATH_ID: i64 = 1_029;

pub const PC2PC_PC_STUFF_COMPONENT_PATH_ID: i64 = 1_611;

pub const PC2PC_MENU_CHAT_COMPONENT_PATH_ID: i64 = 1_612;

pub const PC2PC_EQUIP_COMPONENT_PATH_ID: i64 = 1_613;

pub const PC2PC_INVENTORY_SKIN_PATH_ID: i64 = 1_366;

pub const PC2PC_CHAT_SKIN_PATH_ID: i64 = 1_368;

pub const PC2PC_TRADE_BACK_PATH: &str = "ui/en/pc2pc/trade-back.png";

pub const PC2PC_TRADE_AREA_PATH: &str = "ui/en/pc2pc/trade-area.png";

pub const PC2PC_LOCAL_OFFER_PATH: &str = "ui/en/pc2pc/local-offer.png";

pub const PC2PC_LOCAL_OFFER_READY_PATH: &str = "ui/en/pc2pc/local-offer-ready.png";

pub const PC2PC_REMOTE_OFFER_PATH: &str = "ui/en/pc2pc/remote-offer.png";

pub const PC2PC_REMOTE_OFFER_READY_PATH: &str = "ui/en/pc2pc/remote-offer-ready.png";

pub const PC2PC_CHAT_BOX_PATH: &str = "ui/en/pc2pc/chat-box.png";

pub const PC2PC_MONEY_BACK_PATH: &str = "ui/en/pc2pc/money-back.png";

pub const PC2PC_CHAT_SEND_PATH: &str = "ui/en/pc2pc/chat-send.png";

pub const PC2PC_TAROS_PATH: &str = "ui/en/pc2pc/taros.png";

pub const PC2PC_FREE_CHAT_PATH: &str = "ui/en/pc2pc/free-chat.png";

pub const PC2PC_PORTRAIT_BACK_PATH: &str = "ui/en/pc2pc/portrait-back.png";

pub const PC2PC_TEXT_FIELD_PATH: &str = "ui/en/pc2pc/text-field.png";

/// Approved Cyrillic-capable replacement for the clean JEFFE bitmap Fonts.
pub const PC2PC_JEFFE_FONT_PATH: &str = USER_EQUIP_FONT_PATH;

/// Approved Cyrillic-capable replacement for clean `ChaletBook-Regular Small`.
pub const PC2PC_CHALET_FONT_PATH: &str = "fonts/chaletbook-regular.ttf";

pub const PC2PC_UI_Z_INDEX: i32 = 19;

// `FusionFallInventorySkin` path ID 1366 is the authority for these
// fixed-raster Font references and GUIStyle metrics. Font sizes below are the
// validated outline-font replacements; line heights retain the serialized
// fixed-raster metrics. The chat log uses the clean explicit 15 px row stride.
pub const PC2PC_JEFFE_12_SOURCE_FONT_PATH_ID: i64 = 977;

pub const PC2PC_JEFFE_14_SOURCE_FONT_PATH_ID: i64 = 933;

pub const PC2PC_JEFFE_16_SOURCE_FONT_PATH_ID: i64 = 1_008;

pub const PC2PC_CHALET_SMALL_SOURCE_FONT_PATH_ID: i64 = 949;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(usize)]
pub enum Pc2pcStaticAssetRole {
    TradeBack = 0,
    TradeArea = 1,
    LocalOffer = 2,
    LocalOfferReady = 3,
    RemoteOffer = 4,
    RemoteOfferReady = 5,
    ChatBox = 6,
    MoneyBack = 7,
    ChatSend = 8,
    ChatSendHover = 9,
    Taros = 10,
    FreeChat = 11,
    PortraitBack = 12,
    OfferRejected = 13,
    Button = 14,
    ButtonHover = 15,
    TextField = 16,
    Backdrop = 17,
    RightPanel = 18,
    InventoryPanel = 19,
    SlotOccupied = 20,
    SlotEmpty = 21,
    EquipTitle = 22,
    NanoTab = 23,
    Close = 24,
    Trash = 25,
    Help = 26,
    Combined = 27,
}

impl Pc2pcStaticAssetRole {
    pub const COUNT: usize = 28;
}

#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct Pc2pcUiAssetContract {
    pub image_paths: [String; Pc2pcStaticAssetRole::COUNT],
    pub font_path: String,
    pub chalet_font_path: String,
}

impl Default for Pc2pcUiAssetContract {
    fn default() -> Self {
        Self {
            image_paths: PC2PC_UI_DEFAULT_IMAGE_PATHS.map(str::to_owned),
            font_path: PC2PC_JEFFE_FONT_PATH.to_owned(),
            chalet_font_path: PC2PC_CHALET_FONT_PATH.to_owned(),
        }
    }
}

impl Pc2pcUiAssetContract {
    pub fn validate(&self) -> Result<(), Pc2pcUiAssetContractError> {
        for (index, path) in self.image_paths.iter().enumerate() {
            validate_runtime_asset_path(path).map_err(|reason| {
                Pc2pcUiAssetContractError::UnsafeImagePath {
                    index,
                    path: path.clone(),
                    reason,
                }
            })?;
            if self.image_paths[..index].contains(path) {
                return Err(Pc2pcUiAssetContractError::DuplicateImagePath {
                    index,
                    path: path.clone(),
                });
            }
        }
        validate_runtime_asset_path(&self.font_path).map_err(|reason| {
            Pc2pcUiAssetContractError::UnsafeFontPath {
                path: self.font_path.clone(),
                reason,
            }
        })?;
        validate_runtime_asset_path(&self.chalet_font_path).map_err(|reason| {
            Pc2pcUiAssetContractError::UnsafeFontPath {
                path: self.chalet_font_path.clone(),
                reason,
            }
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub enum Pc2pcStaticAssetReadiness {
    #[default]
    Loading,
    Ready,
    Rejected,
}
