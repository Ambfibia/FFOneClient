use super::*;

pub const CHARACTER_SELECTION_BACKGROUND_FRAME_WIDTH: f32 = 542.0;

pub const CHARACTER_SELECTION_BACKGROUND_FRAME_HEIGHT: f32 = 477.0;

pub const CHARACTER_SELECTION_BACKGROUND_FRAME_COUNT: usize = 10;

#[derive(Component)]
pub(super) struct SelectionBackgroundFrame(pub(super) usize);
