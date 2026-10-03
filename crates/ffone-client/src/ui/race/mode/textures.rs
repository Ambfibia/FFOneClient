
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RaceResultTextureRole {
    Background,
    Black,
    ButtonNormal,
    ButtonHover,
    ButtonActive,
    FusionMatter,
    ItemBar,
    Star,
    StarEmpty,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaceResultTextureContract {
    pub role: RaceResultTextureRole,
    pub runtime_path: &'static str,
    pub source_path_id: i64,
    pub source_name: &'static str,
    pub source_width: u32,
    pub source_height: u32,
    pub sha256: &'static str,
}
