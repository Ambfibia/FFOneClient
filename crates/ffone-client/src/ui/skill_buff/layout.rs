use super::*;

pub const SKILL_BUFF_FONT_LINE_HEIGHT: f32 = 12.071_999_55;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SkillBuffUiRect {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl SkillBuffUiRect {
    pub const fn new(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self {
            left,
            top,
            width,
            height,
        }
    }
}

pub(super) fn icon_rect(
    projection: IconProjection,
    viewport_width: u32,
    count: usize,
    index: usize,
    scale: f32,
) -> SkillBuffUiRect {
    let index = index as f32;
    let size = SKILL_BUFF_ICON_SIZE * scale;
    let (left, top) = match projection {
        IconProjection::Local => {
            let source_left = SKILL_BUFF_LOCAL_CENTER_X - (count / 2) as f32 * SKILL_BUFF_ICON_SIZE
                + index * SKILL_BUFF_ICON_SIZE;
            (source_left * scale, SKILL_BUFF_LOCAL_Y * scale)
        }
        IconProjection::Cash => {
            let pivot = viewport_width as f32;
            let source_left = SKILL_BUFF_CASH_X + index * SKILL_BUFF_ICON_SIZE;
            (
                pivot + (source_left - pivot) * scale,
                SKILL_BUFF_CASH_Y * scale,
            )
        }
        IconProjection::Target => {
            // Unity's Screen.width / 2 is integer division.
            let pivot = (viewport_width / 2) as f32;
            let source_left = pivot + SKILL_BUFF_TARGET_CENTER_OFFSET_X - 10.5 * count as f32
                + index * SKILL_BUFF_ICON_SIZE;
            (
                pivot + (source_left - pivot) * scale,
                SKILL_BUFF_TARGET_Y * scale,
            )
        }
    };
    SkillBuffUiRect::new(left, top, size, size)
}

pub(super) fn bind_rect(node: &mut Node, rect: SkillBuffUiRect) {
    node.left = px(rect.left);
    node.top = px(rect.top);
    node.width = px(rect.width);
    node.height = px(rect.height);
}
