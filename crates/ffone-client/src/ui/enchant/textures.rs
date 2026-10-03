
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EnchantTextureRole0104 {
    Panel,
    RawCover,
    ItemCover,
    QuantityCover,
    XMark,
    LevelBadge,
    Waiting,
    WaitProgress,
    DexlabsBanner,
    TarosCounter,
    BoostIcon,
    PotionIcon,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnchantTextureEvidence0104 {
    pub role: EnchantTextureRole0104,
    pub source_path_id: i64,
    pub source_name: &'static str,
    pub runtime_path: &'static str,
    pub width: u32,
    pub height: u32,
    pub png_bytes: usize,
    pub png_sha256: &'static str,
}
