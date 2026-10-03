use super::*;

pub(super) fn assert_sliced_border(mode: &NodeImageMode, expected: BorderRect) {
    match mode {
        NodeImageMode::Sliced(slicer) => assert_eq!(slicer.border, expected),
        _ => panic!("expected a nine-sliced Option image"),
    }
}

#[test]
fn three_completed_pages_preserve_clean_absolute_geometry() {
    assert_eq!(
        OPTION_GRAPHICS_LEFT_HEADER_RECT,
        OptionUiRect::new(10.0, 20.0, 450.0, 45.0)
    );
    assert_eq!(
        OPTION_GRAPHICS_RIGHT_SIDE_RECT,
        OptionUiRect::new(700.0, 64.0, 240.0, 446.0)
    );
    assert_eq!(
        OPTION_DISPLAY_HEADER_RECT,
        OptionUiRect::new(10.0, 20.0, 935.0, 45.0)
    );
    assert_eq!(
        OPTION_CHAT_HEADER_RECT,
        OptionUiRect::new(10.0, 300.0, 935.0, 45.0)
    );
    assert_eq!(
        OPTION_KEYMAP_VIEW_RECT,
        OptionUiRect::new(25.0, 230.0, 900.0, 260.0)
    );
    assert_eq!(OPTION_CHAT_PALETTE_RGB.len(), 18);
    assert_eq!(OPTION_CHAT_PALETTE_RGB[0], (1.0, 0.0, 0.0));
    assert_eq!(OPTION_CHAT_PALETTE_RGB[17], (1.0, 0.8, 0.6));
}
