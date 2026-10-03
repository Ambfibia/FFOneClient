use super::*;

pub(super) fn asset_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game")
}

#[test]
fn plugin_is_hidden_and_noninteractive_until_asset_closure_is_ready() {
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
    app.world_mut().resource_mut::<OptionUiModel>().visible = true;
    app.update();

    let gate = *app.world().resource::<OptionUiAssetGate>();
    assert!(!gate.ready);
    let world = app.world_mut();
    let mut roots = world.query_filtered::<(&Visibility, &Pickable), With<OptionUiRoot>>();
    let (visibility, pickable) = roots.single(world).unwrap();
    assert_eq!(*visibility, Visibility::Hidden);
    assert_eq!(*pickable, Pickable::IGNORE);

    let mut tabs = world.query::<(&OptionTabButton, &ZIndex)>();
    let tab_layers = tabs
        .iter(world)
        .map(|(tab, z_index)| (tab.0, z_index.0))
        .collect::<HashMap<_, _>>();
    assert_eq!(
        tab_layers[&OptionTab::Graphics],
        OPTION_SELECTED_TAB_Z_INDEX
    );
    for tab in [OptionTab::GameUi, OptionTab::Social, OptionTab::Controls] {
        assert_eq!(tab_layers[&tab], OPTION_NORMAL_TAB_Z_INDEX);
    }

    let mut pages = world.query::<(&OptionPage, &ZIndex)>();
    assert!(
        pages
            .iter(world)
            .all(|(_, z_index)| z_index.0 == OPTION_PAGE_Z_INDEX)
    );

    let mut chrome = world.query::<(&OptionChromeButton, &ImageNode)>();
    let mut chrome_count = 0;
    for (button, image) in chrome.iter(world) {
        chrome_count += 1;
        let border = if *button == OptionChromeButton::Close {
            OPTION_CLOSE_BORDER
        } else {
            OPTION_BIG_LABEL_BORDER
        };
        assert_sliced_border(&image.image_mode, border);
    }
    assert_eq!(chrome_count, 3);

    let mut sky_headers = world.query_filtered::<&ImageNode, With<OptionSkyHeader>>();
    let mut sky_header_count = 0;
    for image in sky_headers.iter(world) {
        sky_header_count += 1;
        assert!(matches!(image.image_mode, NodeImageMode::Stretch));
    }
    assert_eq!(sky_header_count, 12);

    let mut graphics_toggles =
        world.query_filtered::<&ImageNode, With<OptionGraphicsToggleButton>>();
    let mut graphics_toggle_count = 0;
    for image in graphics_toggles.iter(world) {
        graphics_toggle_count += 1;
        assert_sliced_border(&image.image_mode, OPTION_TOGGLE_BORDER);
    }
    assert!(graphics_toggle_count > 0);

    let mut dropdown_backgrounds =
        world.query_filtered::<&Node, With<OptionDropdownChoiceBackground>>();
    let mut background_count = 0;
    for node in dropdown_backgrounds.iter(world) {
        background_count += 1;
        assert_eq!(node.left, px(-OPTION_DROPDOWN_ITEM_OVERFLOW_LEFT));
        assert!([175.0, 200.0].into_iter().any(|width| node.width
            == px(width
                + OPTION_DROPDOWN_ITEM_OVERFLOW_LEFT
                + OPTION_DROPDOWN_ITEM_OVERFLOW_RIGHT)));
        assert_eq!(node.height, px(20.0));
    }
    assert!(background_count > 0);
}
