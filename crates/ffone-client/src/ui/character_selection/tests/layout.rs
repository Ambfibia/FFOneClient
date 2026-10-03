use super::*;

pub(super) fn assert_node_rect(node: &Node, expected: LegacySelectionRect) {
    assert_eq!(node.left, px(expected.x));
    assert_eq!(node.top, px(expected.y));
    assert_eq!(node.width, px(expected.width));
    assert_eq!(node.height, px(expected.height));
}

#[test]
fn baseline_layout_matches_cn_gui_char_selection_exactly() {
    let layout = CharacterSelectionLayout::for_viewport(
        Vec2::new(
            CHARACTER_SELECTION_BASELINE_WIDTH,
            CHARACTER_SELECTION_BASELINE_HEIGHT,
        ),
        1.0,
    );
    assert_eq!(
        layout.chrome,
        LegacySelectionRect::new(-328.0, -379.5, 1920.0, 1440.0)
    );
    assert_eq!(
        layout.avatar_group,
        LegacySelectionRect::new(122.0, 25.5, 600.0, 600.0)
    );
    assert_eq!(
        layout.selection_group,
        LegacySelectionRect::new(662.0, 52.5, 478.0, 600.0)
    );
    assert_eq!(
        layout.quit,
        LegacySelectionRect::new(20.0, 643.0, 90.0, 23.0)
    );
    assert_eq!(
        layout.fullscreen,
        LegacySelectionRect::new(1219.0, 646.0, 35.0, 31.0)
    );
    assert_eq!(
        layout.background_frame(0, 0.0),
        LegacySelectionRect::new(0.0, 38.5, 542.0, 477.0)
    );
    assert_eq!(
        layout.background_frame(9, 0.0),
        LegacySelectionRect::new(4878.0, 38.5, 542.0, 477.0)
    );
}

#[test]
fn spawned_controls_keep_clean_rects_and_imgui_paint_order() {
    let asset_root = project_asset("");
    let mut app = App::new();
    insert_test_localization(&mut app, &asset_root);
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .init_asset::<AudioSource>()
        .add_plugins(NativeCharacterSelectionUiPlugin);
    app.update();

    let world = app.world_mut();
    let root = only_entity::<NativeCharacterSelectionRoot>(world);
    let chrome = only_entity::<SelectionChrome>(world);
    let avatar = only_entity::<SelectionAvatarGroup>(world);
    let panel = only_entity::<SelectionPanel>(world);
    let portrait_overlay = only_entity::<SelectionPortraitOverlayRoot>(world);
    let delete_modal = only_entity::<SelectionDeleteModal>(world);
    let quit = only_entity::<SelectionQuit>(world);
    let fullscreen = only_entity::<SelectionFullscreen>(world);
    let delete_panel = only_entity::<SelectionDeletePanel>(world);

    assert_eq!(
        world.get::<ZIndex>(root),
        Some(&ZIndex(CHARACTER_SELECTION_BASE_INTERACTION_Z_INDEX))
    );
    assert_eq!(
        world.get::<ZIndex>(portrait_overlay),
        Some(&ZIndex(
            CHARACTER_SELECTION_PORTRAIT_OVERLAY_INTERACTION_Z_INDEX
        ))
    );
    assert_eq!(
        world.get::<ZIndex>(delete_modal),
        Some(&ZIndex(CHARACTER_SELECTION_MODAL_INTERACTION_Z_INDEX))
    );

    let mut background_entities = world
        .query::<(Entity, &SelectionBackgroundFrame)>()
        .iter(world)
        .map(|(entity, marker)| (marker.0, entity))
        .collect::<Vec<_>>();
    background_entities.sort_by_key(|(index, _)| *index);
    let root_children = world.get::<Children>(root).unwrap();
    let root_position = |entity| {
        root_children
            .iter()
            .position(|child| child == entity)
            .unwrap()
    };
    for pair in background_entities.windows(2) {
        assert!(root_position(pair[0].1) < root_position(pair[1].1));
    }
    assert!(root_position(background_entities[9].1) < root_position(chrome));
    assert!(root_position(chrome) < root_position(avatar));
    assert!(root_position(avatar) < root_position(panel));
    assert!(root_position(panel) < root_position(quit));
    assert!(root_position(quit) < root_position(fullscreen));

    assert_node_rect(
        world.get::<Node>(chrome).unwrap(),
        LegacySelectionRect::new(0.0, 0.0, 1_920.0, 1_440.0),
    );
    assert_node_rect(
        world.get::<Node>(avatar).unwrap(),
        LegacySelectionRect::new(0.0, 0.0, 600.0, 600.0),
    );
    assert_node_rect(
        world.get::<Node>(panel).unwrap(),
        LegacySelectionRect::new(0.0, 0.0, 478.0, 600.0),
    );
    assert_node_rect(
        world.get::<Node>(portrait_overlay).unwrap(),
        LegacySelectionRect::new(0.0, 0.0, 478.0, 600.0),
    );
    assert_node_rect(
        world.get::<Node>(quit).unwrap(),
        LegacySelectionRect::new(20.0, 643.0, 90.0, 23.0),
    );
    assert_node_rect(
        world.get::<Node>(fullscreen).unwrap(),
        LegacySelectionRect::new(1_219.0, 646.0, 35.0, 31.0),
    );
    assert_node_rect(
        world.get::<Node>(delete_panel).unwrap(),
        LegacySelectionRect::new(369.0, 258.5, 526.0, 164.0),
    );

    for (slot, node) in world.query::<(&SelectionSlotButton, &Node)>().iter(world) {
        assert_node_rect(node, SLOT_BUTTON_RECTS[slot.0]);
    }
    for (slot, node) in world
        .query::<(&SelectionSlotEmptyLabel, &Node)>()
        .iter(world)
    {
        assert_node_rect(
            node,
            LegacySelectionRect::new(220.0, SLOT_EMPTY_Y[slot.0], 63.0, 15.0),
        );
    }
    for (slot, child_of) in world.query::<(&SelectionSlotName, &ChildOf)>().iter(world) {
        assert_node_rect(
            world.get::<Node>(child_of.parent()).unwrap(),
            LegacySelectionRect::new(85.0, SLOT_NAME_Y[slot.0], 370.0, 16.0),
        );
    }
    for (slot, child_of) in world.query::<(&SelectionSlotLevel, &ChildOf)>().iter(world) {
        assert_node_rect(
            world.get::<Node>(child_of.parent()).unwrap(),
            LegacySelectionRect::new(98.0, SLOT_LEVEL_Y[slot.0], 97.0, 16.0),
        );
    }
    for (slot, child_of) in world
        .query::<(&SelectionSlotLocation, &ChildOf)>()
        .iter(world)
    {
        assert_node_rect(
            world.get::<Node>(child_of.parent()).unwrap(),
            LegacySelectionRect::new(98.0, SLOT_LOCATION_Y[slot.0], 250.0, 16.0),
        );
    }

    let slot_zero = world
        .query::<(Entity, &SelectionSlotButton)>()
        .iter(world)
        .find_map(|(entity, marker)| (marker.0 == 0).then_some(entity))
        .unwrap();
    let empty_zero = world
        .query::<(Entity, &SelectionSlotEmptyLabel)>()
        .iter(world)
        .find_map(|(entity, marker)| (marker.0 == 0).then_some(entity))
        .unwrap();
    let disk_back_zero = world
        .query::<(Entity, &SelectionSlotDiskBack)>()
        .iter(world)
        .find_map(|(entity, marker)| (marker.0 == 0).then_some(entity))
        .unwrap();
    let disk_front_zero = world
        .query::<(Entity, &SelectionSlotDiskFront)>()
        .iter(world)
        .find_map(|(entity, marker)| (marker.0 == 0).then_some(entity))
        .unwrap();
    let lock_zero = world
        .query::<(Entity, &SelectionSlotLock)>()
        .iter(world)
        .find_map(|(entity, marker)| (marker.0 == 0).then_some(entity))
        .unwrap();
    let delete = only_entity::<SelectionDelete>(world);
    let panel_children = world.get::<Children>(panel).unwrap();
    let panel_position = |entity| {
        panel_children
            .iter()
            .position(|child| child == entity)
            .unwrap()
    };
    assert!(panel_position(slot_zero) < panel_position(empty_zero));
    assert!(panel_position(empty_zero) < panel_position(disk_back_zero));
    for foreground in [disk_front_zero, lock_zero, delete] {
        assert_eq!(
            world.get::<ChildOf>(foreground).unwrap().parent(),
            portrait_overlay,
            "portrait foregrounds and DELETE must share the post-portrait UI pass"
        );
    }
    let overlay_children = world.get::<Children>(portrait_overlay).unwrap();
    let overlay_position = |entity| {
        overlay_children
            .iter()
            .position(|child| child == entity)
            .unwrap()
    };
    assert!(overlay_position(disk_front_zero) < overlay_position(lock_zero));
    assert!(overlay_position(lock_zero) < overlay_position(delete));
    assert_node_rect(
        world.get::<Node>(disk_front_zero).unwrap(),
        LegacySelectionRect::new(23.0, SLOT_DISK_FRONT_Y[0], 69.0, 23.0),
    );
    assert_node_rect(
        world.get::<Node>(lock_zero).unwrap(),
        LegacySelectionRect::new(42.0, SLOT_LOCK_Y[0], 32.0, 36.0),
    );
}

#[test]
fn center_and_corner_pivots_scale_like_ffguiutility() {
    let layout = CharacterSelectionLayout::for_viewport(Vec2::new(1264.0, 681.0), 0.5);
    assert_eq!(
        layout.chrome,
        LegacySelectionRect::new(152.0, -19.5, 960.0, 720.0)
    );
    assert_eq!(
        layout.selection_group,
        LegacySelectionRect::new(647.0, 196.5, 239.0, 300.0)
    );
    assert_eq!(
        layout.quit,
        LegacySelectionRect::new(10.0, 662.0, 45.0, 11.5)
    );
    assert_eq!(
        layout.fullscreen,
        LegacySelectionRect::new(1241.5, 663.5, 17.5, 15.5)
    );
}

#[test]
fn worldname_lookup_matches_future_and_region_rectangles_in_source_order() {
    assert_eq!(
        resolve_character_selection_location([563_200, 51_200, 0]),
        Some(ResolvedCharacterSelectionLocation {
            district: "Genius Grove",
            zone: "The Future",
            background: CharacterLocationBackground::Future,
        })
    );
    assert_eq!(
        resolve_character_selection_location([307_200, 307_200, 0]),
        Some(ResolvedCharacterSelectionLocation {
            district: "Genius Grove",
            zone: "The Suburbs",
            background: CharacterLocationBackground::Suburbs,
        })
    );
    assert_eq!(
        resolve_character_selection_location([466_000, 470_000, 0]),
        Some(ResolvedCharacterSelectionLocation {
            district: "Prickly Pines",
            zone: "The Wilds",
            background: CharacterLocationBackground::Wilds,
        })
    );
    assert_eq!(
        resolve_character_selection_location([900_000, 900_000, 0]),
        None
    );
}
