use super::*;

pub const RULE_UI_BACK_NORMAL_TEXTURE_PATH_ID: i64 = 178;

pub const RULE_UI_BUTTON_NORMAL_TEXTURE_PATH_ID: i64 = 640;

pub const RULE_UI_BUTTON_HOVER_TEXTURE_PATH_ID: i64 = 309;

pub const RULE_UI_CLOSE_NORMAL_TEXTURE_PATH_ID: i64 = 105;

pub const RULE_UI_CLOSE_HOVER_TEXTURE_PATH_ID: i64 = 157;

pub(super) fn rgba(value: [f32; 4]) -> Color {
    Color::srgba(value[0], value[1], value[2], value[3])
}
