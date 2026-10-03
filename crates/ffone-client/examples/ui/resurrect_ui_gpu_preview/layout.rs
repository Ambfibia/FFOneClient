use super::*;

pub(super) const CLIENT_AREA_WIDTH: u32 = 1_264;

pub(super) const CLIENT_AREA_HEIGHT: u32 = 681;

pub(super) fn node_matches_rect(node: &Node, rect: ResurrectUiRect) -> bool {
    node.position_type == PositionType::Absolute
        && node.left == px(rect.x)
        && node.top == px(rect.y)
        && node.width == px(rect.width)
        && node.height == px(rect.height)
}
