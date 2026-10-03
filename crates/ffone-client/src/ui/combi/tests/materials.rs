use super::*;

#[test]
fn decorations_pass_pointer_focus_to_the_controls_beneath_them() {
    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(CombiUiPlugin);
    app.update();

    let world = app.world_mut();
    let mut nodes = world.query_filtered::<(
        Option<&CombiUiElement0104>,
        Option<&CombiInteractiveControl0104>,
        Has<CombiUiRoot0104>,
        Option<&FocusPolicy>,
        Has<Interaction>,
    ), With<Node>>();
    let mut controls = 0;
    for (element, control, root, focus, interactive) in nodes.iter(world) {
        // Bevy treats a missing FocusPolicy as Block: an icon or label
        // without Pass would starve the cell or button under it.
        let expected = if control.is_some() {
            controls += 1;
            assert!(interactive, "{control:?} lost its Interaction");
            FocusPolicy::Block
        } else if root
            || matches!(
                element,
                Some(CombiUiElement0104::Backdrop | CombiUiElement0104::Shade)
            )
        {
            FocusPolicy::Block
        } else {
            FocusPolicy::Pass
        };
        assert_eq!(focus, Some(&expected), "{element:?} / {control:?}");
    }
    // Bag and equipment cells, two drop areas, two framed selection slots,
    // Clear All, Combine, Close, Help, Combine More, and Go To My Stuff.
    assert_eq!(
        controls,
        INVENTORY_SLOT_COUNT_0104 + USER_EQUIP_EQUIPMENT_STRIP_COUNT + 10
    );
}
