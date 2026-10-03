use super::*;

#[test]
fn paint_order_and_serialized_overflow_match_the_clean_option_skin() {
    assert!(
        OPTION_NORMAL_TAB_Z_INDEX < OPTION_FRAME_Z_INDEX
            && OPTION_FRAME_Z_INDEX < OPTION_PAGE_Z_INDEX
            && OPTION_PAGE_Z_INDEX < OPTION_SELECTED_TAB_Z_INDEX
            && OPTION_SELECTED_TAB_Z_INDEX < OPTION_CHROME_Z_INDEX
            && OPTION_CHROME_Z_INDEX < OPTION_DROPDOWN_BUTTON_Z_INDEX
            && OPTION_DROPDOWN_BUTTON_Z_INDEX < OPTION_DROPDOWN_PANEL_Z_INDEX
            && OPTION_DROPDOWN_PANEL_Z_INDEX < OPTION_DROPDOWN_OPEN_BUTTON_Z_INDEX
    );
    assert_eq!(OPTION_SCROLLBAR_VISUAL_WIDTH, 18.0);
    assert_eq!(OPTION_SCROLL_THUMB_VISUAL_WIDTH, 17.0);
    assert_eq!(OPTION_SCROLL_THUMB_OVERFLOW, 2.0);
    assert_eq!(OPTION_DROPDOWN_ITEM_OVERFLOW_LEFT, 2.0);
    assert_eq!(OPTION_DROPDOWN_ITEM_OVERFLOW_RIGHT, 3.0);

    assert_eq!(
        OptionTextAnchor::UpperLeft.align_items(),
        AlignItems::FlexStart
    );
    assert_eq!(
        OptionTextAnchor::UpperLeft.justify_content(),
        JustifyContent::Start
    );
    assert_eq!(OptionTextAnchor::UpperLeft.justify(), Justify::Left);
    assert_eq!(
        OptionTextAnchor::UpperRight.align_items(),
        AlignItems::FlexStart
    );
    assert_eq!(
        OptionTextAnchor::UpperRight.justify_content(),
        JustifyContent::End
    );
    assert_eq!(OptionTextAnchor::UpperRight.justify(), Justify::Right);
    assert_eq!(
        OptionTextAnchor::MiddleLeft.align_items(),
        AlignItems::Center
    );
    assert_eq!(
        OptionTextAnchor::MiddleLeft.justify_content(),
        JustifyContent::Start
    );
    assert_eq!(OptionTextAnchor::MiddleLeft.justify(), Justify::Left);
    assert_eq!(
        OptionTextAnchor::MiddleRight.align_items(),
        AlignItems::Center
    );
    assert_eq!(
        OptionTextAnchor::MiddleRight.justify_content(),
        JustifyContent::End
    );
    assert_eq!(OptionTextAnchor::MiddleRight.justify(), Justify::Right);
    assert_eq!(OPTION_JEFFE_12_LINE_HEIGHT, 13.56);
    assert_eq!(OPTION_JEFFE_13_LINE_HEIGHT, 14.69);
    assert_eq!(OPTION_JEFFE_14_LINE_HEIGHT, 11.3);
    assert_eq!(OPTION_JEFFE_16_LINE_HEIGHT, 13.56);
    assert_eq!(OPTION_COMIC_LINE_HEIGHT, 18.08);
    assert_eq!(OPTION_CHALET_SMALL_LINE_HEIGHT, 13.56);
    assert_eq!(OPTION_CHALET_LINE_HEIGHT, 15.82);
    assert_eq!(OPTION_JEFFE_TAB_FONT_SIZE, 12.0);
    assert_eq!(OPTION_JEFFE_BUTTON_FONT_SIZE, 11.3);
    assert_eq!(OPTION_JEFFE_TITLE_FONT_SIZE, 11.0);
    assert_eq!(OPTION_CHALET_WHITE_LABEL_FONT_SIZE, 13.1);
    assert_eq!(OPTION_CHALET_WHITE_LABEL_TOP_OFFSET, 4.0);
    assert_eq!(OPTION_GRAPHICS_TAB_TEXT_INSET, 10.0);

    let button = option_button_node(OptionUiRect::new(1.0, 2.0, 3.0, 4.0));
    assert_eq!(button.justify_content, JustifyContent::Center);
    assert_eq!(button.align_items, AlignItems::Center);
    assert_eq!(
        button.padding,
        UiRect {
            left: px(6.0),
            right: px(6.0),
            top: px(3.0),
            bottom: px(6.0),
        }
    );

    assert_sliced_border(
        &option_tab_image_mode(OptionTab::Graphics, false),
        OPTION_GRAPHICS_TAB_BORDER,
    );
    assert!(matches!(
        option_tab_image_mode(OptionTab::Graphics, true),
        NodeImageMode::Stretch
    ));
    assert!(matches!(
        option_tab_image_mode(OptionTab::GameUi, false),
        NodeImageMode::Stretch
    ));
    assert_sliced_border(
        &sliced_image(Handle::<Image>::default(), OPTION_CLOSE_BORDER).image_mode,
        OPTION_CLOSE_BORDER,
    );
    assert_sliced_border(
        &sliced_image(Handle::<Image>::default(), OPTION_TOGGLE_BORDER).image_mode,
        OPTION_TOGGLE_BORDER,
    );
    // Zero-border `darkbox2` and `darkbox2_inter` remain deliberate Stretch cases.
    assert!(matches!(
        stretched_image(Handle::<Image>::default()).image_mode,
        NodeImageMode::Stretch
    ));

    assert_eq!(
        option_tab_content_style(OptionTab::Graphics, false),
        (
            JustifyContent::Start,
            UiRect {
                left: px(OPTION_GRAPHICS_TAB_TEXT_INSET),
                top: px(-8.0),
                ..default()
            },
        )
    );
    assert_eq!(
        option_tab_content_style(OptionTab::Graphics, true),
        (
            JustifyContent::Start,
            UiRect {
                left: px(OPTION_GRAPHICS_TAB_TEXT_INSET),
                top: px(-10.0),
                ..default()
            },
        )
    );
    assert_eq!(
        option_tab_content_style(OptionTab::GameUi, false),
        (JustifyContent::Center, UiRect::default())
    );
    assert_eq!(
        option_tab_content_style(OptionTab::GameUi, true),
        (
            JustifyContent::Center,
            UiRect {
                top: px(-10.0),
                ..default()
            },
        )
    );
    assert_eq!(
        option_tab_content_style(OptionTab::Social, false),
        (
            JustifyContent::Center,
            UiRect {
                top: px(-3.0),
                ..default()
            },
        )
    );
    assert_eq!(
        option_tab_content_style(OptionTab::Social, true),
        (
            JustifyContent::Center,
            UiRect {
                left: px(-4.0),
                top: px(-12.0),
                ..default()
            },
        )
    );
    assert_eq!(
        option_tab_content_style(OptionTab::Controls, false),
        (
            JustifyContent::End,
            UiRect {
                right: px(24.0),
                top: px(-11.0),
                ..default()
            },
        )
    );
    assert_eq!(
        option_tab_content_style(OptionTab::Controls, true),
        (
            JustifyContent::End,
            UiRect {
                right: px(24.0),
                top: px(3.0),
                ..default()
            },
        )
    );
}
