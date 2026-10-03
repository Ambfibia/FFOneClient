use super::*;

pub const BUDDY_LARGE_LIST_BACKGROUND_SOURCE_TEXTURE_PATH_ID: i64 = 610;

pub const BUDDY_SCROLL_TRACK_SOURCE_TEXTURE_PATH_ID: i64 = 374;

pub const BUDDY_SCROLL_THUMB_SOURCE_TEXTURE_PATH_ID: i64 = 324;

pub const BUDDY_SCROLL_UP_SOURCE_TEXTURE_PATH_ID: i64 = 63;

pub const BUDDY_SCROLL_DOWN_SOURCE_TEXTURE_PATH_ID: i64 = 415;

pub(super) fn color_from_rgba([red, green, blue, alpha]: [f32; 4]) -> Color {
    Color::srgba(red, green, blue, alpha)
}
