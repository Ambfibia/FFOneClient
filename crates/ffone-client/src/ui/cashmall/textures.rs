
pub const CASHMALL_CASH_TEXTURE_PATH_ID: i64 = 551;

pub const CASHMALL_INFO_TEXTURE_PATH_ID: i64 = 493;

pub const CASHMALL_NANO_TAB_TEXTURE_PATH_ID: i64 = 98;

pub const CASHMALL_NANO_TAB_HOVER_TEXTURE_PATH_ID: i64 = 299;

pub const CASHMALL_DEXLABS_TEXTURE_PATH_ID: i64 = 451;

pub const CASHMALL_TAROS_COUNTER_TEXTURE_PATH_ID: i64 = 326;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CashmallTextureRole0104 {
    BackBar,
    Cash,
    FirstTabSelected,
    FirstTabNormal,
    FirstTabHover,
    SecondTabSelected,
    SecondTabNormal,
    SecondTabHover,
    NanoTab,
    NanoTabHover,
    DexlabsBanner,
    TarosCounter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CashmallTextureEvidence0104 {
    pub role: CashmallTextureRole0104,
    pub source_name: &'static str,
    pub path_id: i64,
    pub runtime_path: &'static str,
    pub width: u32,
    pub height: u32,
    pub png_bytes: usize,
    pub png_sha256: &'static str,
}
