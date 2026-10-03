use super::*;

#[test]
fn exact_frame_tabs_close_and_footer_geometry_is_preserved() {
    assert_eq!(
        OPTION_BACKDROP_RECT,
        OptionUiRect::new(-450.0, -401.0, 1_920.0, 1_440.0)
    );
    assert_eq!(OPTION_FRAME_BORDER, BorderRect::all(10.0));
    assert_eq!(
        OPTION_BIG_LABEL_BORDER,
        BorderRect {
            min_inset: Vec2::new(8.0, 5.0),
            max_inset: Vec2::new(8.0, 5.0)
        }
    );
    assert_eq!(
        OPTION_PANEL_CONNECTOR_BORDER,
        BorderRect {
            min_inset: Vec2::new(6.0, 0.0),
            max_inset: Vec2::new(35.0, 0.0)
        }
    );
    assert_eq!(OPTION_TEXT_FIELD_BORDER, BorderRect::all(2.0));
    assert_eq!(OPTION_CLOSE_BORDER, OPTION_BIG_LABEL_BORDER);
    assert_eq!(
        OPTION_GRAPHICS_TAB_BORDER,
        BorderRect {
            min_inset: Vec2::new(0.0, 0.0),
            max_inset: Vec2::new(0.0, 3.0)
        }
    );
    assert_eq!(
        OPTION_TOGGLE_BORDER,
        BorderRect {
            min_inset: Vec2::new(17.0, 0.0),
            max_inset: Vec2::new(0.0, 0.0)
        }
    );
    assert_eq!(OPTION_DARK_BOX_BORDER, BorderRect::all(10.0));
    assert_eq!(
        OPTION_SCROLLBAR_BORDER,
        BorderRect {
            min_inset: Vec2::new(2.0, 4.0),
            max_inset: Vec2::new(2.0, 4.0)
        }
    );
    assert_eq!(OPTION_SCROLL_THUMB_BORDER, OPTION_SCROLLBAR_BORDER);
    assert_eq!(
        OPTION_FRAME_RECT,
        OptionUiRect::new(10.0, 50.0, 980.0, 540.0)
    );
    assert_eq!(
        OPTION_PAGE_RECT,
        OptionUiRect::new(20.0, 60.0, 960.0, 520.0)
    );
    assert_eq!(OPTION_CLOSE_RECT, OptionUiRect::new(987.0, 0.0, 30.0, 30.0));
    assert_eq!(
        OPTION_APPLY_RECT,
        OptionUiRect::new(500.0, 600.0, 180.0, 25.0)
    );
    assert_eq!(
        OPTION_SAVE_RECT,
        OptionUiRect::new(695.0, 600.0, 180.0, 25.0)
    );
    assert_eq!(
        OptionTab::Controls.selected_rect(),
        OptionUiRect::new(799.0, 3.0, 194.0, 61.0)
    );
    assert_eq!(
        OptionTab::Social.normal_rect(),
        OptionUiRect::new(560.0, 21.0, 189.0, 33.0)
    );
}

#[test]
fn hidden_option_tree_is_not_dirtied_on_an_idle_frame() {
    let empty_assets = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(AssetPlugin {
            file_path: empty_assets.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(OptionUiPlugin);

    app.update();
    app.world_mut().clear_trackers();
    app.update();

    let world = app.world_mut();
    let mut pages = world.query_filtered::<Ref<Node>, With<OptionPage>>();
    assert!(pages.iter(world).all(|node| !node.is_changed()));
    let mut labels = world.query_filtered::<Ref<TextColor>, With<OptionButtonLabel>>();
    assert!(labels.iter(world).all(|color| !color.is_changed()));
    let mut roots = world.query_filtered::<Ref<Visibility>, With<OptionUiRoot>>();
    assert!(!roots.single(world).unwrap().is_changed());
}
