use super::*;

#[test]
fn scrollbar_thumb_geometry_is_shared_by_drawing_and_dragging() {
    for mode in [UserEquipMode::Item, UserEquipMode::Nano] {
        let metrics = UserEquipScrollbarMetrics::for_mode(mode);
        assert!(metrics.thumb_height >= 15.0);
        assert_eq!(
            metrics.thumb_height + metrics.travel,
            USER_EQUIP_SCROLL_TRACK_RECT.height
        );
        assert_eq!(metrics.thumb_offset(0.0), 0.0);
        if metrics.scroll_max > 0.0 {
            assert!((metrics.thumb_offset(metrics.scroll_max) - metrics.travel).abs() < 1e-3);
            let half = metrics.drag_value(0.0, metrics.travel * 0.5);
            assert!((metrics.thumb_offset(half) - metrics.travel * 0.5).abs() < 1e-3);
            assert_eq!(metrics.drag_value(half, 1.0e6), metrics.scroll_max);
            assert_eq!(metrics.drag_value(half, -1.0e6), 0.0);
            assert_eq!(metrics.drag_value(half, f32::NAN), half);
        }
    }
    assert_eq!(
        UserEquipScrollbarMetrics::for_mode(UserEquipMode::Item).scroll_max,
        user_equip_scroll_max()
    );
    assert_eq!(
        UserEquipScrollbarMetrics::for_mode(UserEquipMode::Nano).scroll_max,
        user_equip_nano_scroll_max()
    );
}

#[test]
fn geometry_centers_exact_panels_grid_and_scroll_extent() {
    let layout = user_equip_item_mode_layout(1_020, 638, 1.0, 0.0);
    assert_eq!(
        layout.user_clothes_panel,
        UserEquipUiRect::new(0.0, 0.0, 512.0, 649.0)
    );
    assert_eq!(
        layout.pc_stuff_panel,
        UserEquipUiRect::new(585.0, 0.0, 380.0, 632.0)
    );
    assert_eq!(
        layout.equipment_panel,
        UserEquipUiRect::new(504.0, 0.0, 66.0, 639.0)
    );
    assert_eq!(
        layout.inventory_viewport,
        UserEquipUiRect::new(591.0, 36.0, 366.0, 500.0)
    );
    assert_eq!(
        layout.inventory_slot_rect(0),
        Some(UserEquipUiRect::new(591.0, 36.0, 67.0, 67.0))
    );
    assert_eq!(
        layout.inventory_slot_rect(49),
        Some(UserEquipUiRect::new(867.0, 657.0, 67.0, 67.0))
    );
    assert_eq!(layout.inventory_slot_rect(50), None);
    assert_eq!(
        layout.equipment_slot_rect(8),
        Some(UserEquipUiRect::new(504.0, 510.0, 64.0, 64.0))
    );
    assert_close(user_equip_scroll_max(), 190.0);

    let scrolled = user_equip_item_mode_layout(1_020, 638, 1.0, 999.0);
    assert_close(scrolled.scroll_y, 190.0);
    assert_eq!(
        scrolled.inventory_slot_rect(49),
        Some(UserEquipUiRect::new(867.0, 467.0, 67.0, 67.0))
    );

    let centered = user_equip_item_mode_layout(1_280, 720, 1.0, 0.0);
    assert_eq!(centered.user_clothes_panel.left, 130.0);
    assert_eq!(centered.user_clothes_panel.top, 41.0);
    assert_eq!(centered.pc_stuff_panel.left, 715.0);
    assert_eq!(centered.equipment_panel.left, 634.0);

    let short = user_equip_item_mode_layout(800, 500, 1.0, 0.0);
    assert_eq!(short.user_clothes_panel.left, 0.0);
    assert_eq!(short.user_clothes_panel.top, 0.0);
    assert_eq!(short.pc_stuff_panel.left, 585.0);
}

#[test]
fn avatar_target_and_repeat_buttons_match_the_1264x681_source_geometry() {
    let layout = user_equip_item_mode_layout(1_264, 681, USER_EQUIP_OPEN_SECONDS, 0.0);
    assert_eq!(
        USER_EQUIP_AVATAR_PREVIEW_RECT.translated(
            layout.user_clothes_panel.left,
            layout.user_clothes_panel.top,
        ),
        UserEquipUiRect::new(122.0, -29.0, 500.0, 564.0)
    );
    assert_eq!(
        USER_EQUIP_TURN_LEFT_POSITIONED_RECT.translated(
            layout.user_clothes_panel.left,
            layout.user_clothes_panel.top,
        ),
        UserEquipUiRect::new(242.0, 461.0, 43.0, 78.0)
    );
    assert_eq!(
        USER_EQUIP_TURN_RIGHT_POSITIONED_RECT.translated(
            layout.user_clothes_panel.left,
            layout.user_clothes_panel.top,
        ),
        UserEquipUiRect::new(462.0, 461.0, 43.0, 78.0)
    );

    let mut presentation = UserEquipAvatarPreviewPresentation::default();
    assert_eq!(presentation.avatar_rect(), USER_EQUIP_AVATAR_PREVIEW_RECT);
    presentation.set_vehicle_mounted(true);
    assert_eq!(
        presentation.avatar_rect(),
        UserEquipUiRect::new(0.0, 50.0, 500.0, 564.0)
    );
}

#[test]
fn repeat_button_positions_preserve_crossed_styles_and_signed_degrees() {
    assert_eq!(
        UserEquipAvatarTurnDirection::LeftPositioned.yaw_delta_degrees(),
        1.0
    );
    assert_eq!(
        UserEquipAvatarTurnDirection::RightPositioned.yaw_delta_degrees(),
        -1.0
    );
    assert_eq!(
        user_equip_avatar_turn_asset_role(UserEquipAvatarTurnDirection::LeftPositioned, false,),
        UserEquipStaticAssetRole::TurnRight
    );
    assert_eq!(
        user_equip_avatar_turn_asset_role(UserEquipAvatarTurnDirection::LeftPositioned, true,),
        UserEquipStaticAssetRole::TurnRightHover
    );
    assert_eq!(
        user_equip_avatar_turn_asset_role(UserEquipAvatarTurnDirection::RightPositioned, false,),
        UserEquipStaticAssetRole::TurnLeft
    );
    assert_eq!(
        user_equip_avatar_turn_asset_role(UserEquipAvatarTurnDirection::RightPositioned, true,),
        UserEquipStaticAssetRole::TurnLeftHover
    );
}

#[test]
fn final_item_mode_view_matches_1264x681_geometry_and_slot_overlays() {
    let runtime = runtime_with(
        &[(4, item(4, 91, 0x0011_0000, 0))],
        &[
            (0, item(0, 42, 0x1234_0002_u32 as i32, 0)),
            (1, item(7, 77, 25, 0)),
            (2, item(8, 88, 0, 0)),
        ],
    );
    let projection = UserEquipItemModeProjection::from_authoritative(&runtime, &AllCatalog);
    let mut state = UserEquipUiState::default();
    state.open_item_mode();
    state.tick(USER_EQUIP_OPEN_SECONDS);
    let view = user_equip_item_mode_view(
        1_264,
        681,
        state,
        UserEquipModalState::default(),
        &projection,
        true,
    )
    .unwrap();

    assert_eq!(
        view.layout.full_backdrop,
        UserEquipUiRect::new(-328.0, -379.0, 1_920.0, 1_440.0)
    );
    assert_eq!(
        view.layout.clothes_backplate,
        UserEquipUiRect::new(114.0, 14.0, 585.0, 653.0)
    );
    assert_eq!(
        view.layout.pc_stuff_backplate,
        UserEquipUiRect::new(699.0, 14.0, 451.0, 653.0)
    );
    assert_eq!(
        view.layout.user_clothes_panel,
        UserEquipUiRect::new(122.0, 21.0, 512.0, 649.0)
    );
    assert_eq!(
        view.layout.pc_stuff_panel,
        UserEquipUiRect::new(707.0, 21.0, 380.0, 632.0)
    );
    assert_eq!(
        view.layout.equipment_panel,
        UserEquipUiRect::new(626.0, 21.0, 66.0, 639.0)
    );
    assert_eq!(
        view.layout.inventory_viewport,
        UserEquipUiRect::new(713.0, 57.0, 366.0, 500.0)
    );
    assert_eq!(
        view.layout.close_button,
        UserEquipUiRect::new(1_107.0, 26.0, 30.0, 30.0)
    );
    assert_eq!(
        view.layout.trash_button,
        UserEquipUiRect::new(1_097.0, 576.0, 32.0, 32.0)
    );
    assert_eq!(
        view.layout.help_button,
        UserEquipUiRect::new(1_097.0, 611.0, 32.0, 32.0)
    );
    assert!(view.controls_enabled);

    assert_eq!(
        view.inventory[0].frame_visual,
        UserEquipSlotFrameVisual::Occupied
    );
    assert!(matches!(
        view.inventory[0].icon,
        UserEquipPresentationIcon::Resolved(_)
    ));
    assert_eq!(
        view.inventory[0].combined_badge,
        Some(UserEquipUiRect::new(749.0, 93.0, 26.0, 26.0))
    );
    assert_eq!(view.inventory[1].count_label.as_deref(), Some("25"));
    assert_eq!(
        view.inventory[2].icon,
        UserEquipPresentationIcon::MissingChecker
    );
    assert_eq!(view.inventory[2].count_label.as_deref(), Some("Quest 88"));
    assert_eq!(
        view.inventory[3].frame_visual,
        UserEquipSlotFrameVisual::Empty
    );
    assert_eq!(view.inventory[3].icon, UserEquipPresentationIcon::Empty);
    assert_eq!(view.equipment[0].label, "HEAD");
    assert_eq!(view.equipment[6].label, "WEAPON 1");
    assert_eq!(view.equipment[7].label, "WEAPON 2");
    assert_eq!(view.equipment[8].label, "VEHICLE");
    assert!(view.equipment[0].combined_badge.is_some());
}

#[test]
fn nano_projection_preserves_primary_order_owned_state_power_and_read_only_geometry() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    let locator = AssetLocator::open(&root).unwrap();
    let content = TutorialMissionContent::from_project_assets(&locator).unwrap();
    let empty_nano = Nano0104 {
        id: 0,
        skill_id: 0,
        stamina: 0,
    };
    let mut bank = [empty_nano; PcLoadData0104::NANO_BANK_COUNT];
    bank[4] = Nano0104 {
        id: 1,
        skill_id: 1,
        stamina: 75,
    };
    bank[9] = Nano0104 {
        id: 5,
        skill_id: 8,
        stamina: 125,
    };
    let projection = UserEquipNanoModeProjection::from_authoritative(
        &bank,
        [
            UserEquipNanoEquippedAuthority {
                nano_id: Some(1),
                skill_id: 1,
                stamina: 75,
                active: true,
            },
            UserEquipNanoEquippedAuthority {
                nano_id: Some(5),
                skill_id: 8,
                stamina: 125,
                active: false,
            },
            UserEquipNanoEquippedAuthority::default(),
        ],
        &content,
    );

    assert_eq!(content.general_item_stim_pack_attribute(119), Some(1));
    for (slot_index, status) in projection.status.iter().enumerate() {
        assert_eq!(
            user_equip_gum_target_enabled(119, slot_index, &projection, &content),
            status.nano_id.is_some() && status.style == Some(0),
            "Adaptium Gumball target gate must follow one-based Nano style"
        );
    }

    assert_eq!(projection.gallery.len(), USER_EQUIP_NANO_GALLERY_COUNT);
    assert_eq!(
        projection
            .gallery
            .iter()
            .map(|entry| entry.nano_id)
            .collect::<Vec<_>>(),
        USER_EQUIP_MERGED_NANO_ORDER
    );
    assert!(
        projection
            .gallery
            .windows(2)
            .all(|pair| pair[0].sort_number <= pair[1].sort_number)
    );
    assert!(projection.gallery.iter().all(|entry| entry.nano_id > 0));
    assert!(projection.gallery.iter().any(|entry| entry.nano_id > 36));
    let owned = projection
        .gallery
        .iter()
        .find(|entry| entry.nano_id == 1)
        .unwrap();
    assert!(owned.owned && owned.equipped);
    assert_eq!(owned.current_power, Some(1));
    assert!(matches!(
        owned.icon,
        UserEquipPresentationIcon::Resolved(ref path) if path.contains("/nanoicon_")
    ));
    let unowned = projection
        .gallery
        .iter()
        .find(|entry| entry.nano_id == 2)
        .unwrap();
    assert!(!unowned.owned && !unowned.equipped);
    assert_eq!(unowned.current_power, None);
    assert!(matches!(
        unowned.icon,
        UserEquipPresentationIcon::Resolved(ref path) if path.contains("/ready/nanoready_")
    ));
    for (name, expected_path) in [
        (
            "Belladonna",
            "icons/entities/nanos/ready/nanoready_belladonna.png",
        ),
        ("Rigby", "icons/entities/nanos/ready/nanoready_rigby.png"),
        (
            "Johnny Test",
            "icons/entities/nanos/ready/nanoready_johnny-test.png",
        ),
        (
            "Flapjack",
            "icons/entities/nanos/ready/nanoready_flapjack.png",
        ),
        ("Finn", "icons/entities/nanos/ready/nanoready_finn.png"),
        ("Rex", "icons/entities/nanos/ready/nanoready_rex.png"),
        (
            "Ben Tennyson",
            "icons/entities/nanos/ready/nanoready_ben-10.png",
        ),
        ("Jake", "icons/entities/nanos/ready/nanoready_jake.png"),
    ] {
        assert!(matches!(
            projection.gallery.iter().find(|entry| entry.name == name).unwrap().icon,
            UserEquipPresentationIcon::Resolved(ref path) if path == expected_path
        ));
    }
    let rigby = projection
        .gallery
        .iter()
        .find(|entry| entry.name == "Rigby")
        .unwrap();
    assert!(matches!(
        rigby.viewer_icon,
        UserEquipPresentationIcon::Resolved(ref path)
            if path == "icons/entities/nanos/nanoicon_rigby.png"
    ));
    assert_ne!(rigby.icon, rigby.viewer_icon);
    let ben = projection
        .gallery
        .iter()
        .find(|entry| entry.name == "Ben Tennyson")
        .unwrap();
    let rex = projection
        .gallery
        .iter()
        .find(|entry| entry.name == "Rex")
        .unwrap();
    assert_ne!(ben.viewer_icon, rex.viewer_icon);
    assert_eq!(
        ben.skills.each_ref().map(|skill| skill.skill_id),
        [210, 211, 212]
    );
    assert_eq!(ben.description, "Ben Tennyson Nano.");
    let cheese = projection
        .gallery
        .iter()
        .find(|entry| entry.name == "Cheese")
        .unwrap();
    assert_eq!(
        cheese.skills.each_ref().map(|skill| skill.skill_id),
        [286, 238, 239]
    );
    assert_eq!(
        cheese.skills.each_ref().map(|skill| skill.tune_id),
        [237, 238, 239]
    );
    assert_eq!(cheese.skills[0].name, "HEAL!");
    assert_eq!(projection.status[0].nano_id, Some(1));
    assert_eq!(projection.status[0].current_power, Some(1));
    assert_close(projection.status[0].stamina_fraction(), 0.5);
    assert_eq!(projection.status[2].nano_id, None);

    let layout = user_equip_item_mode_layout(1_020, 638, 1.0, 0.0);
    let gallery = user_equip_nano_gallery_view(layout, &projection);
    assert_eq!(gallery[0].frame.width, 67.0);
    assert_eq!(gallery[1].frame.left - gallery[0].frame.left, 69.0);
    assert_eq!(gallery[5].frame.top - gallery[0].frame.top, 69.0);
    assert_close(USER_EQUIP_NANO_CONTENT_HEIGHT, 966.0);

    let mut state = UserEquipUiState::default();
    state.open_item_mode();
    state.tick(USER_EQUIP_OPEN_SECONDS);
    state.select_nano_tab();
    state.set_scroll_y(user_equip_nano_scroll_max());
    let bottom_layout = state.layout(1_020, 638);
    assert_close(bottom_layout.scroll_y, user_equip_nano_scroll_max());
    let bottom_gallery = user_equip_nano_gallery_view(bottom_layout, &projection);
    let final_bottom =
        bottom_gallery.last().unwrap().frame.top + bottom_gallery.last().unwrap().frame.height;
    let viewport_bottom =
        bottom_layout.inventory_viewport.top + bottom_layout.inventory_viewport.height;
    assert!(final_bottom <= viewport_bottom && final_bottom >= viewport_bottom - 2.0);
}

#[test]
fn popup_geometry_uses_primary_inventory_and_unequip_owners() {
    let layout = user_equip_item_mode_layout(1_020, 638, 1.0, 0.0);
    assert_eq!(
        user_equip_popup_rects(layout, UserEquipSlotEndpoint::Inventory { slot_index: 0 },),
        (
            UserEquipUiRect::new(610.0, 60.0, 310.0, 449.0),
            UserEquipUiRect::new(0.0, 14.0, 310.0, 435.0),
        )
    );
    assert_eq!(
        user_equip_popup_rects(
            layout,
            UserEquipSlotEndpoint::Equipment {
                visual_index: 0,
                wire_slot_index: 4,
            },
        ),
        (
            UserEquipUiRect::new(195.0, 75.0, 310.0, 448.0),
            UserEquipUiRect::new(0.0, 0.0, 310.0, 448.0),
        )
    );
}

#[test]
fn popup_layout_variants_follow_primary_general_subtype_dispatch() {
    let runtime = runtime_with(
        &[(4, item(4, 91, 0, 0))],
        &[
            (0, item(0, 77, 0, 0)),
            (1, item(7, 40, 1, 0)),
            (2, item(9, 41, 1, 0)),
        ],
    );
    let projection = UserEquipItemModeProjection::from_authoritative(&runtime, &AllCatalog);
    let mut popup = UserEquipItemPopupState::default();

    popup.open(UserEquipSlotEndpoint::Inventory { slot_index: 0 });
    assert_eq!(
        user_equip_popup_layout_variant(&popup, &projection, None),
        Some(UserEquipPopupLayoutVariant::Equip)
    );
    popup.open(UserEquipSlotEndpoint::Inventory { slot_index: 1 });
    assert_eq!(
        user_equip_popup_layout_variant(&popup, &projection, None),
        Some(UserEquipPopupLayoutVariant::GeneralStack)
    );
    popup.open(UserEquipSlotEndpoint::Inventory { slot_index: 2 });
    assert_eq!(
        user_equip_popup_layout_variant(&popup, &projection, None),
        Some(UserEquipPopupLayoutVariant::Chest)
    );
    popup.open(UserEquipSlotEndpoint::Equipment {
        visual_index: 0,
        wire_slot_index: 4,
    });
    assert_eq!(
        user_equip_popup_layout_variant(&popup, &projection, None),
        Some(UserEquipPopupLayoutVariant::Unequip)
    );

    assert_eq!(
        user_equip_popup_variant_layout_key(
            UserEquipUiElement::ItemPopupButton(0),
            UserEquipPopupLayoutVariant::GeneralStack,
        )
        .as_deref(),
        Some("item_general_stack_button_0")
    );
    assert_eq!(
        user_equip_popup_variant_layout_key(
            UserEquipUiElement::ItemPopupButton(0),
            UserEquipPopupLayoutVariant::Chest,
        )
        .as_deref(),
        Some("item_chest_button_0")
    );
    assert_eq!(
        user_equip_general_popup_variant(Some(3)),
        UserEquipPopupLayoutVariant::GeneralGum
    );
    for item_type in [4, 5, 7, 8, 9, 11] {
        assert_eq!(
            user_equip_general_popup_variant(Some(item_type)),
            UserEquipPopupLayoutVariant::GeneralUse
        );
    }
    assert_eq!(
        user_equip_general_popup_variant(Some(2)),
        UserEquipPopupLayoutVariant::GeneralStack
    );
    assert_eq!(
        USER_EQUIP_GUM_NANO_FRAME_RECTS,
        [
            UserEquipUiRect::new(33.0, 160.0, 62.0, 62.0),
            UserEquipUiRect::new(125.0, 160.0, 62.0, 62.0),
            UserEquipUiRect::new(215.0, 160.0, 62.0, 62.0),
        ]
    );
    assert_eq!(
        USER_EQUIP_GUM_NANO_BUTTON_RECTS,
        [
            UserEquipUiRect::new(31.0, 232.0, 66.0, 25.0),
            UserEquipUiRect::new(125.0, 232.0, 66.0, 25.0),
            UserEquipUiRect::new(215.0, 232.0, 66.0, 25.0),
        ]
    );
    assert!(user_equip_gum_attribute_accepts_style(
        Some(1),
        Some(1),
        Some(0)
    ));
    assert!(!user_equip_gum_attribute_accepts_style(
        Some(1),
        Some(5),
        Some(1)
    ));
    assert!(user_equip_gum_attribute_accepts_style(
        Some(4),
        Some(5),
        Some(2)
    ));
    assert!(!user_equip_gum_attribute_accepts_style(
        Some(4),
        None,
        Some(2)
    ));
    assert_eq!(
        equipment_slot_label_color(UserEquipSlotFrameVisual::Empty),
        Color::srgba(1.0, 1.0, 1.0, 0.8)
    );
    assert_eq!(
        equipment_slot_label_color(UserEquipSlotFrameVisual::Occupied),
        Color::srgba(0.6, 1.0, 0.0, 0.8)
    );
    assert_eq!(
        user_equip_popup_command_rect(
            UserEquipPopupCommand::EquipPrimary,
            Some(UserEquipPopupLayoutVariant::Equip),
        ),
        UserEquipUiRect::new(120.0, 374.0, 170.0, 28.0)
    );
    assert_eq!(
        user_equip_popup_command_rect(
            UserEquipPopupCommand::Unequip,
            Some(UserEquipPopupLayoutVariant::Unequip),
        ),
        UserEquipUiRect::new(160.0, 393.0, 130.0, 28.0)
    );
    assert_eq!(
        user_equip_popup_command_rect(
            UserEquipPopupCommand::Delete,
            Some(UserEquipPopupLayoutVariant::GeneralStack),
        ),
        UserEquipUiRect::new(79.0, 301.0, 150.0, 25.0)
    );
    assert_eq!(
        user_equip_popup_command_rect(
            UserEquipPopupCommand::Open,
            Some(UserEquipPopupLayoutVariant::Chest),
        ),
        UserEquipUiRect::new(160.0, 180.0, 130.0, 28.0)
    );
    assert_eq!(
        user_equip_popup_field_rect(0, Some(UserEquipPopupLayoutVariant::Equip)),
        Some(UserEquipUiRect::new(82.0, 64.0, 170.0, 20.0))
    );
    assert_eq!(
        user_equip_popup_field_rect(3, Some(UserEquipPopupLayoutVariant::Unequip)),
        Some(UserEquipUiRect::new(110.0, 232.0, 80.0, 20.0))
    );
}

#[test]
fn editor_catalog_keys_cover_static_panels_and_repeated_slot_geometry() {
    assert_eq!(
        user_equip_editable_layout_key(UserEquipUiElement::PcStuffPanelBackground).as_deref(),
        Some("inventory_panel")
    );
    assert_eq!(
        user_equip_editable_layout_key(UserEquipUiElement::InventorySlotFrame(7)).as_deref(),
        Some("inventory_slot_07")
    );
    assert_eq!(
        user_equip_editable_layout_key(UserEquipUiElement::NanoSlotFrame(63)).as_deref(),
        Some("nano_slot_63")
    );
    assert_eq!(
        user_equip_editable_layout_key(UserEquipUiElement::EquipmentSlotFrame(8)).as_deref(),
        Some("equipment_slot_8")
    );
    assert_eq!(
        user_equip_editable_layout_key(UserEquipUiElement::ItemPopupEquipInfo).as_deref(),
        Some("item_popup_equip_info")
    );
}

#[test]
fn equipment_strip_captions_keep_primary_middle_right_anchor_and_battery_rects() {
    let label_node = user_equip_equipfont_node(UserEquipUiRect::new(0.0, -2.0, 60.0, 19.0));
    assert_eq!(label_node.left, Val::Px(0.0));
    assert_eq!(label_node.top, Val::Px(-2.0));
    assert_eq!(label_node.width, Val::Px(60.0));
    assert_eq!(label_node.height, Val::Px(19.0));
    assert_eq!(label_node.justify_content, JustifyContent::End);
    assert_eq!(label_node.align_items, AlignItems::Center);
    assert_eq!(label_node.padding, UiRect::default());
    assert_eq!(
        user_equip_default_label_top_padding(UserEquipUiElement::BoostValue),
        3.0
    );
    assert_eq!(
        user_equip_default_label_top_padding(UserEquipUiElement::PotionValue),
        3.0
    );

    let layout = user_equip_item_mode_layout(1_264, 681, USER_EQUIP_OPEN_SECONDS, 0.0);
    assert_eq!(
        layout.equipment_label_rect(0),
        Some(UserEquipUiRect::new(626.0, 33.0, 60.0, 19.0))
    );
    assert_eq!(
        layout.equipment_label_rect(8),
        Some(UserEquipUiRect::new(626.0, 529.0, 60.0, 19.0))
    );

    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    let document =
        UiLayoutDocument::load(&asset_root.join(USER_EQUIP_EDITABLE_LAYOUT_PATH)).unwrap();
    for (id, expected) in [
        ("boost_icon", [32.0, 577.0, 30.0, 28.0]),
        ("boost_label", [0.0, 570.0, 60.0, 19.0]),
        ("boost_value", [4.0, 580.0, 64.0, 30.0]),
        ("potion_icon", [32.0, 606.0, 30.0, 28.0]),
        ("potion_label", [0.0, 599.0, 60.0, 19.0]),
        ("potion_value", [4.0, 610.0, 64.0, 30.0]),
    ] {
        let element = document
            .elements
            .iter()
            .find(|element| element.id == id)
            .unwrap();
        assert!(element.override_enabled);
        assert_eq!(
            [
                element.rect.x,
                element.rect.y,
                element.rect.width,
                element.rect.height,
            ],
            expected
        );
        assert_eq!(element.rect, element.source_rect);
    }
}
