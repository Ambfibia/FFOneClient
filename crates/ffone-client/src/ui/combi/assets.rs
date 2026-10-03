use super::*;

pub const COMBI_GAME_OBJECT_PATH_ID: i64 = 1_276;

pub const COMBI_MODE_COMPONENT_PATH_ID: i64 = 1_488;

pub const COMBI_GUI_COMPONENT_PATH_ID: i64 = 1_489;

pub const COMBI_PC_STUFF_COMPONENT_PATH_ID: i64 = 1_490;

pub const COMBI_EQUIP_COMPONENT_PATH_ID: i64 = 1_491;

pub const COMBI_SKIN_PATH_ID: i64 = 1_367;

pub const COMBI_PRIMARY_CAMERA_PATH_ID: i64 = 1_492;

pub const COMBI_WAITING_CAMERA_PATH_ID: i64 = 1_494;

pub const COMBI_UI_Z_INDEX: i32 = 21;

pub const COMBI_PANEL_PATH: &str = "ui/en/combi/panel.png";

pub const COMBI_BLACK_SHADE_PATH: &str = "ui/en/combi/black-shade.png";

pub const COMBI_LOOK_ITEM_BG_PATH: &str = "ui/en/combi/look-item-bg.png";

pub const COMBI_STAT_ITEM_BG_PATH: &str = "ui/en/combi/stat-item-bg.png";

pub const COMBI_COMBINED_PATH: &str = "ui/en/combi/combined.png";

pub const COMBI_TAROS_ICON_PATH: &str = "ui/en/combi/taros-icon.png";

pub const COMBI_SUCCESS_PATH: &str = "ui/en/combi/success.png";

pub const COMBI_NPC_ICON_PATH: &str = "ui/en/combi/npc-icon.png";

pub const COMBI_WAITING_PATH: &str = "ui/en/combi/waiting.png";

pub const COMBI_CHALET_FONT_PATH: &str = "fonts/chaletbook-regular.ttf";

pub const COMBI_RECIPE_TABLE_PATH: &str = crate::assets::TABLE_SET_PATH;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CombiStaticAssetRole {
    Backdrop,
    Panel,
    RightBackplate,
    BlackShade,
    LookItemBg,
    LookError,
    StatItemBg,
    StatError,
    Combined,
    Taros,
    Success,
    NpcIcon,
    Waiting,
    ButtonNormal,
    ButtonHover,
    Restricted,
    InventoryPanel,
    SlotOccupied,
    SlotEmpty,
    EquipTitle,
    Close,
    Trash,
    Help,
    CloseHover,
    HelpHover,
}

impl CombiStaticAssetRole {
    pub const COUNT: usize = 25;

    pub(super) const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct CombiUiAssetContract {
    pub(super) image_paths: [String; CombiStaticAssetRole::COUNT],
    pub(super) jeffe_font_path: String,
    pub(super) chalet_font_path: String,
}

impl Default for CombiUiAssetContract {
    fn default() -> Self {
        Self {
            image_paths: COMBI_UI_DEFAULT_IMAGE_PATHS.map(str::to_owned),
            jeffe_font_path: USER_EQUIP_FONT_PATH.to_owned(),
            chalet_font_path: COMBI_CHALET_FONT_PATH.to_owned(),
        }
    }
}

impl CombiUiAssetContract {
    pub fn new(
        image_paths: [String; CombiStaticAssetRole::COUNT],
        jeffe_font_path: String,
        chalet_font_path: String,
    ) -> Result<Self, CombiUiAssetContractError0104> {
        for (index, path) in image_paths.iter().enumerate() {
            if !safe_relative_asset_path(path) {
                return Err(CombiUiAssetContractError0104::UnsafeImagePath {
                    index,
                    path: path.clone(),
                });
            }
        }
        if !safe_relative_asset_path(&jeffe_font_path) {
            return Err(CombiUiAssetContractError0104::UnsafeFontPath {
                path: jeffe_font_path,
            });
        }
        if !safe_relative_asset_path(&chalet_font_path) {
            return Err(CombiUiAssetContractError0104::UnsafeFontPath {
                path: chalet_font_path,
            });
        }
        Ok(Self {
            image_paths,
            jeffe_font_path,
            chalet_font_path,
        })
    }

    #[must_use]
    pub fn image_path(&self, role: CombiStaticAssetRole) -> &str {
        &self.image_paths[role.index()]
    }

    #[must_use]
    pub fn jeffe_font_path(&self) -> &str {
        &self.jeffe_font_path
    }

    #[must_use]
    pub fn chalet_font_path(&self) -> &str {
        &self.chalet_font_path
    }

    #[must_use]
    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.image_paths.iter().map(String::as_str).chain([
            self.jeffe_font_path.as_str(),
            self.chalet_font_path.as_str(),
        ])
    }
}

pub(super) fn safe_relative_asset_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.starts_with('\\')
        && !path.contains('\\')
        && !path.split('/').any(|part| part.is_empty() || part == "..")
        && !path.contains(':')
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CombiUiAssetContractError0104 {
    UnsafeImagePath { index: usize, path: String },
    UnsafeFontPath { path: String },
}

impl fmt::Display for CombiUiAssetContractError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Combi UI asset contract rejected: {self:?}")
    }
}

impl Error for CombiUiAssetContractError0104 {}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CombiStaticAssetReadiness0104 {
    #[default]
    Loading,
    Ready,
    Failed,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct CombiUiAssetStatus0104(pub CombiStaticAssetReadiness0104);

pub(super) fn look_icon_path(look: &CombiLookProjection0104) -> Option<&str> {
    look.icon_path.as_deref()
}

pub(super) fn stats_icon_path(stats: &CombiStatsProjection0104) -> Option<&str> {
    stats.icon_path.as_deref()
}
