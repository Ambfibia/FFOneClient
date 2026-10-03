use super::*;

pub const CASHMALL_GAME_OBJECT_PATH_ID: i64 = 1_359;

pub const CASHMALL_TRANSFORM_PATH_ID: i64 = 1_243;

pub const CASHMALL_MODE_COMPONENT_PATH_ID: i64 = 1_573;

pub const CASHMALL_PANEL_COMPONENT_PATH_ID: i64 = 1_574;

pub const CASHMALL_PC_STUFF_COMPONENT_PATH_ID: i64 = 1_575;

pub const CASHMALL_EQUIP_COMPONENT_PATH_ID: i64 = 1_576;

pub const CASHMALL_MODE_SCRIPT_PATH_ID: i64 = 1_120;

pub const CASHMALL_PANEL_SCRIPT_PATH_ID: i64 = 1_077;

pub const CASHMALL_PC_STUFF_SCRIPT_PATH_ID: i64 = 1_045;

pub const CASHMALL_EQUIP_SCRIPT_PATH_ID: i64 = 1_027;

pub const CASHMALL_SKIN_PATH_ID: i64 = 1_376;

pub const CASHMALL_INVENTORY_SKIN_PATH_ID: i64 = 1_366;

pub const CASHMALL_ITEM_BAR_PATH_ID: i64 = 0;

pub const CASHMALL_TAB_VISUAL_FONT_PATH_ID: i64 = 903;

pub const CASHMALL_LABEL_FONT_PATH_ID: i64 = 977;

pub const CASHMALL_SMALL_FONT_PATH_ID: i64 = 970;

pub const CASHMALL_UI_Z_INDEX: i32 = 20;

pub const CASHMALL_BACK_BAR_PATH: &str = "ui/en/cashmall/back-bar.png";

pub const CASHMALL_CASH_PATH: &str = "ui/en/cashmall/cash.png";

pub const CASHMALL_FIRST_TAB_SELECTED_PATH: &str = "ui/en/cashmall/first-tab.png";

pub const CASHMALL_FIRST_TAB_NORMAL_PATH: &str = "ui/en/cashmall/first-tab-button.png";

pub const CASHMALL_SECOND_TAB_SELECTED_PATH: &str = "ui/en/cashmall/second-tab.png";

pub const CASHMALL_SECOND_TAB_NORMAL_PATH: &str = "ui/en/cashmall/second-tab-button.png";

pub const CASHMALL_DEXLABS_PATH: &str = "ui/en/enchant/dexlabsbut.png";

pub const CASHMALL_TAROS_COUNTER_PATH: &str = "ui/en/enchant/Taros.png";

#[must_use]
pub fn cashmall_safe_relative_asset_path_0104(path: &str) -> bool {
    !path.is_empty()
        && !path.contains('\\')
        && !path.contains(':')
        && !path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CashmallStaticAssetRole0104 {
    Backdrop,
    CashmallBackplate,
    RightBackplate,
    Info,
    ListBack,
    BackBar,
    Cash,
    FirstSelected,
    FirstNormal,
    FirstHover,
    SecondSelected,
    SecondNormal,
    SecondHover,
    ButtonNormal,
    ButtonHover,
    Restricted,
    ScrollShadow,
    InventoryPanel,
    NanoTab,
    NanoTabHover,
    SlotOccupied,
    SlotEmpty,
    Combined,
    EquipTitle,
    Close,
    Trash,
    Help,
    DexlabsBanner,
    TarosCounter,
}

impl CashmallStaticAssetRole0104 {
    pub const COUNT: usize = 29;
    pub const ALL: [Self; Self::COUNT] = [
        Self::Backdrop,
        Self::CashmallBackplate,
        Self::RightBackplate,
        Self::Info,
        Self::ListBack,
        Self::BackBar,
        Self::Cash,
        Self::FirstSelected,
        Self::FirstNormal,
        Self::FirstHover,
        Self::SecondSelected,
        Self::SecondNormal,
        Self::SecondHover,
        Self::ButtonNormal,
        Self::ButtonHover,
        Self::Restricted,
        Self::ScrollShadow,
        Self::InventoryPanel,
        Self::NanoTab,
        Self::NanoTabHover,
        Self::SlotOccupied,
        Self::SlotEmpty,
        Self::Combined,
        Self::EquipTitle,
        Self::Close,
        Self::Trash,
        Self::Help,
        Self::DexlabsBanner,
        Self::TarosCounter,
    ];

    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }

    #[must_use]
    pub const fn is_cashmall_owned(self) -> bool {
        matches!(
            self,
            Self::BackBar
                | Self::Cash
                | Self::FirstSelected
                | Self::FirstNormal
                | Self::FirstHover
                | Self::SecondSelected
                | Self::SecondNormal
                | Self::SecondHover
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct CashmallUiAssetContract0104 {
    pub(super) image_paths: [String; CashmallStaticAssetRole0104::COUNT],
    pub(super) font_path: String,
}

impl Default for CashmallUiAssetContract0104 {
    fn default() -> Self {
        Self::new(
            CASHMALL_UI_DEFAULT_IMAGE_PATHS_0104.map(str::to_owned),
            USER_EQUIP_FONT_PATH.to_owned(),
        )
        .expect("built-in Cash Mall paths must remain semantic")
    }
}

impl CashmallUiAssetContract0104 {
    pub fn new(
        image_paths: [String; CashmallStaticAssetRole0104::COUNT],
        font_path: String,
    ) -> Result<Self, CashmallUiAssetContractError0104> {
        for role in CashmallStaticAssetRole0104::ALL {
            let path = &image_paths[role.index()];
            let role_path_valid = if role.is_cashmall_owned() {
                path.starts_with("ui/en/cashmall/")
            } else {
                path.starts_with("ui/")
            };
            if !cashmall_safe_relative_asset_path_0104(path)
                || !path.ends_with(".png")
                || !role_path_valid
            {
                return Err(CashmallUiAssetContractError0104::Image {
                    role,
                    path: path.clone(),
                });
            }
        }
        if !cashmall_safe_relative_asset_path_0104(&font_path)
            || !font_path.starts_with("fonts/")
            || !(font_path.ends_with(".otf") || font_path.ends_with(".ttf"))
        {
            return Err(CashmallUiAssetContractError0104::Font(font_path));
        }
        Ok(Self {
            image_paths,
            font_path,
        })
    }

    #[must_use]
    pub fn image_path(&self, role: CashmallStaticAssetRole0104) -> &str {
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
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CashmallUiAssetContractError0104 {
    Image {
        role: CashmallStaticAssetRole0104,
        path: String,
    },
    Font(String),
}

impl fmt::Display for CashmallUiAssetContractError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Image { role, path } => {
                write!(
                    formatter,
                    "invalid Cash Mall image path for {role:?}: {path}"
                )
            }
            Self::Font(path) => write!(formatter, "invalid Cash Mall font path: {path}"),
        }
    }
}

impl Error for CashmallUiAssetContractError0104 {}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CashmallStaticAssetReadiness0104 {
    #[default]
    Loading,
    Ready,
    Failed,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct CashmallUiAssetStatus0104(pub CashmallStaticAssetReadiness0104);

#[must_use]
pub const fn cashmall_tab_asset_role_0104(
    tab: CashmallTab0104,
    visual: CashmallTabVisual0104,
) -> CashmallStaticAssetRole0104 {
    let first = matches!(tab, CashmallTab0104::New);
    match (first, visual) {
        (true, CashmallTabVisual0104::Selected) => CashmallStaticAssetRole0104::FirstSelected,
        (true, CashmallTabVisual0104::Normal) => CashmallStaticAssetRole0104::FirstNormal,
        (true, CashmallTabVisual0104::Hover) => CashmallStaticAssetRole0104::FirstHover,
        (false, CashmallTabVisual0104::Selected) => CashmallStaticAssetRole0104::SecondSelected,
        (false, CashmallTabVisual0104::Normal) => CashmallStaticAssetRole0104::SecondNormal,
        (false, CashmallTabVisual0104::Hover) => CashmallStaticAssetRole0104::SecondHover,
    }
}
