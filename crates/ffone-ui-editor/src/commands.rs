
#[derive(Clone, Copy)]
pub(super) enum AlignAction {
    Left,
    HorizontalCenter,
    Right,
    Top,
    VerticalCenter,
    Bottom,
    MatchWidth,
    MatchHeight,
}
