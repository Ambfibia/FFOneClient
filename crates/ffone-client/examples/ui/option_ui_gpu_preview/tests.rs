use super::*;

#[test]
fn preview_contract_covers_all_four_reachable_tabs() {
    assert_eq!((CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT), (1_264, 681));
    assert_eq!(
        PreviewMode::Social.default_output(),
        "target/ui-parity/option-social-1264x681.png"
    );
    assert_eq!(PreviewMode::Graphics.tab(), OptionTab::Graphics);
    assert_eq!(PreviewMode::GameUi.tab(), OptionTab::GameUi);
    assert_eq!(PreviewMode::Social.tab(), OptionTab::Social);
    assert_eq!(PreviewMode::Controls.tab(), OptionTab::Controls);
    for mode in [
        PreviewMode::Graphics,
        PreviewMode::GameUi,
        PreviewMode::Social,
        PreviewMode::Controls,
    ] {
        assert!(!expected_page_text(mode).is_empty());
        assert!(mode.default_output().ends_with("-1264x681.png"));
    }
    let layout = option_ui_layout(Vec2::new(1_264.0, 681.0), true);
    assert_eq!(layout.scale, Vec2::ONE);
    assert_eq!((layout.node_left, layout.node_top), (122.0, 21.5));
}
