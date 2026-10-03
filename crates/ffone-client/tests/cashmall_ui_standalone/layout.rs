use super::*;

#[test]
fn final_1264_by_681_layout_matches_the_clean_shell() {
    let layout = cashmall_mode_layout_0104(1_264, 681, CASHMALL_OPEN_SECONDS, 0.0);
    assert_eq!(
        layout.cashmall_backplate,
        CashmallUiRect0104::new(114.0, 14.0, 585.0, 653.0)
    );
    assert_eq!(
        layout.right_backplate,
        CashmallUiRect0104::new(699.0, 14.0, 451.0, 653.0)
    );
    assert_eq!(
        layout.cashmall_panel,
        CashmallUiRect0104::new(122.0, 21.0, 498.0, 638.0)
    );
    assert_eq!(
        CashmallUiRect0104::from(layout.item_mode.pc_stuff_panel),
        CashmallUiRect0104::new(707.0, 21.0, 380.0, 632.0)
    );
    assert_eq!(
        CashmallUiRect0104::from(layout.item_mode.equipment_panel),
        CashmallUiRect0104::new(626.0, 21.0, 66.0, 639.0)
    );
    assert_eq!(layout.go_to_stuff.left, 435.0);
    assert_eq!(layout.go_to_stuff.top, 616.0);
    assert_eq!(CASHMALL_GO_TO_STUFF_RECT.top, 595.0);
    assert_eq!(layout.list_content.height, 0.0);
}

#[test]
fn scale_and_crop_backdrop_covers_1264_by_681_without_distortion() {
    let layout = cashmall_mode_layout_0104(1_264, 681, 1.0, 0.0);
    assert!((layout.full_backdrop.left - 0.0).abs() < 0.001);
    assert!((layout.full_backdrop.top + 133.5).abs() < 0.001);
    assert!((layout.full_backdrop.width - 1_264.0).abs() < 0.001);
    assert!((layout.full_backdrop.height - 948.0).abs() < 0.001);
    assert!((layout.full_backdrop.width / layout.full_backdrop.height - 4.0 / 3.0).abs() < 0.001);
}

#[test]
fn tab_backgrounds_use_the_serialized_nonzero_nine_slice_borders() {
    assert_eq!(
        cashmall_tab_border_0104(CashmallTab0104::New),
        CASHMALL_FIRST_TAB_BORDER
    );
    for tab in [
        CashmallTab0104::Scroll,
        CashmallTab0104::Potion,
        CashmallTab0104::Equipment,
        CashmallTab0104::Etc,
    ] {
        assert_eq!(cashmall_tab_border_0104(tab), CASHMALL_SECOND_TAB_BORDER);
    }
    assert_eq!(
        CASHMALL_FIRST_TAB_BORDER,
        BorderRect {
            min_inset: bevy::math::Vec2::new(8.0, 0.0),
            max_inset: bevy::math::Vec2::new(32.0, 0.0)
        }
    );
    assert_eq!(
        CASHMALL_SECOND_TAB_BORDER,
        BorderRect {
            min_inset: bevy::math::Vec2::new(36.0, 0.0),
            max_inset: bevy::math::Vec2::new(45.0, 0.0)
        }
    );
}

#[test]
fn clean_zero_height_catalog_does_not_publish_automatic_scrollbar_assets() {
    assert_eq!(CASHMALL_CACHED_ITEM_COUNT_0104, 0);
    assert_eq!(CASHMALL_LEGACY_SCROLL_CONTENT_HEIGHT, 0.0);
    assert!(!CASHMALL_AUTOMATIC_SCROLLBAR_REACHABLE);
    let paths = CASHMALL_UI_DEFAULT_IMAGE_PATHS_0104;
    for dead in [
        "scroll-track.png",
        "scroll-thumb.png",
        "scroll-up.png",
        "scroll-down.png",
    ] {
        assert!(
            !paths.iter().any(|path| path.ends_with(dead)),
            "dead automatic scrollbar asset leaked: {dead}"
        );
    }
    assert!(paths.contains(&crate::vendor_ui::VENDOR_SCROLL_SHADOW_PATH));
}
