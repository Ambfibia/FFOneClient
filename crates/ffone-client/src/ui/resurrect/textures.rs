use super::*;

pub const RESURRECT_GRIM_TEXTURE_PATH_ID: i64 = 55;

pub const RESURRECT_ICON_TEXTURE_PATH_ID: i64 = 405;

pub const RESURRECT_DIALOG_TEXTURE_PATH_ID: i64 = 398;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ResurrectTextureRole {
    Grim,
    ResurrectEmIcon,
    SkillIconBack,
    BlackBackdrop,
    Dialog,
    ButtonNormal,
    ButtonHover,
    PhoenixSelf,
    PhoenixGroup,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResurrectTextureContract {
    pub role: ResurrectTextureRole,
    pub runtime_path: &'static str,
    pub sha256: &'static str,
    pub source_path_id: Option<i64>,
    pub source_width: u32,
    pub source_height: u32,
}

pub const RESURRECT_TEXTURE_CONTRACTS: [ResurrectTextureContract; 9] = [
    ResurrectTextureContract {
        role: ResurrectTextureRole::Grim,
        runtime_path: RESURRECT_GRIM_PATH,
        sha256: RESURRECT_GRIM_SHA256,
        source_path_id: Some(RESURRECT_GRIM_TEXTURE_PATH_ID),
        source_width: 438,
        source_height: 362,
    },
    ResurrectTextureContract {
        role: ResurrectTextureRole::ResurrectEmIcon,
        runtime_path: RESURRECT_ICON_PATH,
        sha256: RESURRECT_ICON_SHA256,
        source_path_id: Some(RESURRECT_ICON_TEXTURE_PATH_ID),
        source_width: 46,
        source_height: 71,
    },
    ResurrectTextureContract {
        role: ResurrectTextureRole::SkillIconBack,
        runtime_path: RESURRECT_SKILL_ICON_BACK_PATH,
        sha256: RESURRECT_SKILL_ICON_BACK_SHA256,
        source_path_id: Some(RESURRECT_SKILL_ICON_BACK_PATH_ID),
        source_width: 21,
        source_height: 21,
    },
    ResurrectTextureContract {
        role: ResurrectTextureRole::BlackBackdrop,
        runtime_path: RESURRECT_BLACK_BACK_PATH,
        sha256: RESURRECT_BLACK_BACK_SHA256,
        source_path_id: Some(RESURRECT_BLACK_BACK_PATH_ID),
        source_width: 8,
        source_height: 8,
    },
    ResurrectTextureContract {
        role: ResurrectTextureRole::Dialog,
        runtime_path: RESURRECT_DIALOG_PATH,
        sha256: RESURRECT_DIALOG_SHA256,
        source_path_id: Some(RESURRECT_DIALOG_TEXTURE_PATH_ID),
        source_width: 110,
        source_height: 164,
    },
    ResurrectTextureContract {
        role: ResurrectTextureRole::ButtonNormal,
        runtime_path: RESURRECT_BUTTON_NORMAL_PATH,
        sha256: RESURRECT_BUTTON_NORMAL_SHA256,
        source_path_id: Some(RESURRECT_BUTTON_NORMAL_PATH_ID),
        source_width: 20,
        source_height: 25,
    },
    ResurrectTextureContract {
        role: ResurrectTextureRole::ButtonHover,
        runtime_path: RESURRECT_BUTTON_HOVER_PATH,
        sha256: RESURRECT_BUTTON_HOVER_SHA256,
        source_path_id: Some(RESURRECT_BUTTON_HOVER_PATH_ID),
        source_width: 20,
        source_height: 25,
    },
    ResurrectTextureContract {
        role: ResurrectTextureRole::PhoenixSelf,
        runtime_path: RESURRECT_PHOENIX_SELF_ICON_PATH,
        sha256: RESURRECT_PHOENIX_SELF_ICON_SHA256,
        source_path_id: None,
        source_width: 32,
        source_height: 32,
    },
    ResurrectTextureContract {
        role: ResurrectTextureRole::PhoenixGroup,
        runtime_path: RESURRECT_PHOENIX_GROUP_ICON_PATH,
        sha256: RESURRECT_PHOENIX_GROUP_ICON_SHA256,
        source_path_id: None,
        source_width: 32,
        source_height: 32,
    },
];
