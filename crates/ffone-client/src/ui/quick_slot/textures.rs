use super::*;

pub const QUICK_SLOT_COOLDOWN_TEXTURE_PATH_ID: i64 = 546;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum QuickSlotTextureRole {
    Background,
    OccupiedStyle,
    EmptyStyle,
    Cooldown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QuickSlotSourceTextureEvidence {
    pub role: QuickSlotTextureRole,
    pub converted_path: &'static str,
    pub runtime_path: &'static str,
    pub sha256: &'static str,
    pub source_path_id: i64,
    pub width: u32,
    pub height: u32,
}

#[must_use]
pub fn is_semantic_png_path(path: &str) -> bool {
    (path.starts_with("ui/") || path.starts_with("icons/")) && is_relative_png_path(path)
}

pub(super) fn is_semantic_ui_png_path(path: &str) -> bool {
    path.starts_with("ui/") && is_relative_png_path(path)
}

pub(super) fn is_relative_png_path(path: &str) -> bool {
    path.ends_with(".png")
        && !path.contains('\\')
        && !has_content_hash_suffix(path)
        && !path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "..")
}
