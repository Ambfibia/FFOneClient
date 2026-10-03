use super::*;

#[test]
fn card_item_types_follow_protocol_ids_and_localize_in_both_bundles() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    for locale in ["en", "ru"] {
        let (localization, language) =
            crate::localization::Localization::open(&root, locale).unwrap();
        for (kind, suffix, english) in [
            (1, "body", "Body"),
            (2, "legs", "Legs"),
            (3, "shoes", "Shoes"),
            (4, "hat", "Hat"),
            (5, "glasses", "Glasses"),
            (6, "backpack", "Backpack"),
        ] {
            let text = user_equip_item_type_localized(kind);
            assert_eq!(text.key, format!("ui.inventory.item_type.{suffix}"));
            let rendered = localization.text(&language, &text);
            assert!(!rendered.is_empty());
            if locale == "en" {
                assert_eq!(rendered, english);
            } else {
                assert_ne!(rendered, english);
            }
        }
    }
    for kind in 0..=10 {
        let item = ffone_protocol::ItemBase0104 {
            item_type: kind,
            item_id: 100,
            option: 107 << 16,
            time_limit: 0,
        };
        assert_eq!(
            user_equip_display_text_id(item),
            if kind <= 3 { 107 } else { 100 }
        );
        assert_eq!(
            user_equip_display_text_id(ffone_protocol::ItemBase0104 {
                option: i32::MIN,
                ..item
            }),
            100
        );
    }
}

pub(super) fn item(item_type: i16, item_id: i16, option: i32, time_limit: i32) -> ItemBase0104 {
    ItemBase0104 {
        item_type,
        item_id,
        option,
        time_limit,
    }
}

pub(super) fn empty() -> ItemBase0104 {
    item(0, 0, 0, 0)
}

pub(super) fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.000_1,
        "expected {expected}, got {actual}"
    );
}

pub(super) fn checker_pixel(rgba: &[u8], x: u32, y: u32) -> [u8; 4] {
    let offset = ((y * USER_EQUIP_MISSING_CHECKER_SIZE + x) * 4) as usize;
    rgba[offset..offset + 4].try_into().unwrap()
}

#[test]
fn missing_icon_checker_matches_clean_avatar_util_quadrants() {
    let rgba = user_equip_missing_checker_rgba();
    assert_eq!(rgba.len(), 64 * 64 * 4);
    assert_eq!(checker_pixel(&rgba, 0, 0), [255, 0, 255, 255]);
    assert_eq!(checker_pixel(&rgba, 31, 31), [255, 0, 255, 255]);
    assert_eq!(checker_pixel(&rgba, 32, 0), [0, 0, 0, 255]);
    assert_eq!(checker_pixel(&rgba, 0, 32), [0, 0, 0, 255]);
    assert_eq!(checker_pixel(&rgba, 63, 63), [255, 0, 255, 255]);
}

#[test]
fn plugin_owns_no_camera_and_missing_static_files_fail_closed() {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(UserEquipUiPlugin);
    app.world_mut().spawn((
        Window {
            resolution: WindowResolution::new(1_264, 681),
            ..default()
        },
        PrimaryWindow,
    ));
    app.insert_resource(UserEquipItemModeProjection::from_authoritative(
        &runtime_with(&[], &[]),
        &AllCatalog,
    ));
    {
        let mut state = app.world_mut().resource_mut::<UserEquipUiState>();
        state.open_item_mode();
        state.tick(USER_EQUIP_OPEN_SECONDS);
    }
    app.update();

    let world = app.world_mut();
    let mut roots = world.query_filtered::<(&Node, &Visibility), With<UserEquipUiRoot>>();
    let (node, visibility) = roots.single(world).unwrap();
    assert_eq!(*visibility, Visibility::Hidden);
    assert_eq!((node.width, node.height), (px(1_264), px(681)));
    let mut elements = world.query::<&UserEquipUiElement>();
    // The semantic shell may grow as source-owned controls are recovered;
    // acceptance below remains tied to required owners, not a brittle
    // total entity count.
    assert!(elements.iter(world).count() >= 415);
    let mut close = world.query_filtered::<(), With<UserEquipCloseControl>>();
    assert_eq!(close.iter(world).count(), 1);
    let mut cameras = world.query::<&Camera>();
    assert_eq!(cameras.iter(world).count(), 0);

    let mut texts = world.query::<(&Text, &LocalizedText, (&TextFont, &LineHeight))>();
    let texts = texts.iter(world).collect::<Vec<_>>();
    assert!(texts.len() >= 103);
    assert!(
        texts
            .iter()
            .all(|(_, localized, _)| !localized.key.is_empty())
    );
    let (_, _, tab_font) = texts
        .iter()
        .find(|(_, localized, _)| localized.key == "ui.inventory.tab.equipment")
        .copied()
        .unwrap();
    assert_eq!(
        tab_font.0.font_size.eval(Vec2::ZERO, 16.0),
        USER_EQUIP_TAB_FONT_SIZE
    );
    assert_eq!(
        (*tab_font.1),
        LineHeight::Px(USER_EQUIP_REGULAR_FONT_LINE_HEIGHT)
    );
    let (_, _, slot_font) = texts
        .iter()
        .find(|(_, localized, _)| localized.key == "ui.inventory.slot.head")
        .copied()
        .unwrap();
    assert_eq!(
        slot_font.0.font_size.eval(Vec2::ZERO, 16.0),
        USER_EQUIP_SMALL_FONT_SIZE
    );
    assert_eq!(
        (*slot_font.1),
        LineHeight::Px(USER_EQUIP_SMALL_FONT_LINE_HEIGHT)
    );
    let (_, _, guide_font) = texts
        .iter()
        .find(|(_, localized, _)| localized.key == "ui.inventory.status.guide")
        .copied()
        .unwrap();
    assert_eq!(
        guide_font.0.font_size.eval(Vec2::ZERO, 16.0),
        USER_EQUIP_SMALL_FONT_SIZE
    );
    assert_eq!(
        (*guide_font.1),
        LineHeight::Px(USER_EQUIP_SMALL_FONT_LINE_HEIGHT)
    );
    drop(texts);

    let mut positioned = world.query::<(&UserEquipUiElement, &Node)>();
    for slot in 0..3 {
        let (_, portrait) = positioned
            .iter(world)
            .find(|(element, _)| **element == UserEquipUiElement::NanoStatusPortrait(slot))
            .unwrap();
        let expected = USER_EQUIP_NANO_PREVIEW_RECTS[slot];
        assert_eq!(
            (portrait.left, portrait.top, portrait.width, portrait.height),
            (
                px(expected.left),
                px(expected.top),
                px(expected.width),
                px(expected.height)
            )
        );
    }
}

#[test]
fn hard_system_modal_keeps_underlying_help_and_item_popup_unchanged() {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .insert_resource(ButtonInput::<KeyCode>::default())
        .add_plugins(UserEquipUiPlugin);
    app.world_mut().spawn((
        Window {
            resolution: WindowResolution::new(1_264, 681),
            ..default()
        },
        PrimaryWindow,
    ));
    {
        let mut state = app.world_mut().resource_mut::<UserEquipUiState>();
        state.open_item_mode();
        state.tick(USER_EQUIP_OPEN_SECONDS);
    }
    *app.world_mut().resource_mut::<UserEquipModalState>() = UserEquipModalState {
        help_active: true,
        item_popup_active: true,
        system_popup_active: true,
        ..default()
    };
    app.world_mut()
        .resource_mut::<UserEquipItemPopupState>()
        .open(UserEquipSlotEndpoint::Inventory { slot_index: 0 });
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Escape);

    app.update();

    let modal = *app.world().resource::<UserEquipModalState>();
    assert!(modal.help_active);
    assert!(modal.item_popup_active);
    assert!(modal.system_popup_active);
    assert_eq!(
        app.world().resource::<UserEquipItemPopupState>().selected(),
        Some(UserEquipSlotEndpoint::Inventory { slot_index: 0 })
    );
    assert!(app.world().resource::<UserEquipUiOutbox>().is_empty());
}

#[test]
fn combined_ids_split_unsigned_high_and_low_bits_only_for_equipment() {
    let combined = item(0, 42, 0x1234_0002_u32 as i32, 0);
    let ids = UserEquipItemIds::from_item(combined);
    assert_eq!(ids.base_item_id, 42);
    assert_eq!(ids.combined_look_id, Some(0x1234));
    assert_eq!(ids.low_option_bits, 2);
    assert_eq!(ids.icon_item_row_id(), 0x1234);

    let query = UserEquipCatalogQuery::from_non_empty_item(combined).unwrap();
    assert_eq!(
        query.kind,
        UserEquipCatalogKind::Equipment { item_table: 25 }
    );
    assert_eq!(query.base_item_id, 42);
    assert_eq!(query.item_row_id, 0x1234);

    let general = item(7, 42, 0x1234_0002_u32 as i32, 0);
    let general_ids = UserEquipItemIds::from_item(general);
    assert_eq!(general_ids.combined_look_id, None);
    assert_eq!(general_ids.icon_item_row_id(), 42);
    assert_eq!(
        UserEquipCatalogQuery::from_non_empty_item(general)
            .unwrap()
            .kind,
        UserEquipCatalogKind::General
    );

    let runtime = runtime_with(&[(4, item(4, 9, 0x0011_0000, 0))], &[(0, combined)]);
    let projection = UserEquipItemModeProjection::from_authoritative(&runtime, &AllCatalog);
    assert!(projection.inventory[0].item.show_combined_badge);
    assert!(
        projection.equipment[0].item.show_combined_badge,
        "equip strip draws its combined badge without the inventory type<Head gate"
    );
}

#[test]
fn projection_changes_only_when_rebuilt_from_authoritative_snapshot() {
    let moved = item(0, 77, 0, 0);
    let mut runtime = runtime_with(&[], &[(0, moved)]);
    let mut projection = UserEquipItemModeProjection::from_authoritative(&runtime, &AllCatalog);
    let before = projection.clone();

    runtime
        .apply_item_move_success(ItemMoveSuccessPacket0104 {
            from_location: 1,
            from_slot_num: 0,
            from_slot_item: empty(),
            to_location: 0,
            to_slot_num: 0,
            to_slot_item: moved,
        })
        .unwrap();

    assert_eq!(
        projection, before,
        "authoritative runtime mutation cannot mutate an existing UI projection"
    );
    projection.rebuild_from_authoritative(&runtime, &AllCatalog);
    assert!(projection.inventory[0].item.empty);
    assert_eq!(
        projection.equipment[6].item.item, moved,
        "visual weapon slot 1 projects authoritative wire slot 0"
    );
}

#[test]
fn taros_uses_clean_nine_digit_counter_and_exact_cells() {
    assert_eq!(user_equip_taros_digits(112), "000000112");
    assert_eq!(user_equip_taros_digits(-5), "000000000");
    assert_eq!(user_equip_taros_digits(i64::MAX), "999999999");
    assert_eq!(
        user_equip_taros_digit_rect(0),
        UserEquipUiRect::new(22.0, 564.0, 12.0, 20.0)
    );
    assert_eq!(
        user_equip_taros_digit_rect(8),
        UserEquipUiRect::new(118.0, 564.0, 12.0, 20.0)
    );
}

#[test]
fn popup_rating_colors_reach_bound_text_children_and_reset() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let content = TutorialMissionContent::open(&AssetLocator::open(&root).unwrap()).unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(content)
        .init_resource::<UserEquipItemPopupState>()
        .init_resource::<UserEquipItemModeProjection>()
        .add_systems(Update, bind_item_card_text_case);
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn(Node::default()).with_children(|parent| {
            for index in 2..=4 {
                spawn_bound_text_styled(
                    parent,
                    UserEquipUiElement::ItemPopupField(index),
                    UserEquipUiRect::new(0., 0., 80., 20.),
                    "",
                    "ui.content.passthrough",
                    "{text}",
                    Handle::default(),
                    12.,
                    13.71,
                    Color::WHITE,
                    Justify::Left,
                );
            }
        });
    });
    for kind in 0..=3 {
        let content = app.world().resource::<TutorialMissionContent>();
        let mut candidates: Vec<_> = (1..1000)
            .filter_map(|id| {
                let detail = content.gameplay_user_equip_item_detail(kind, id)?;
                Some((
                    if kind == 0 {
                        detail.point_rating
                    } else {
                        detail.defense_rating
                    },
                    id,
                ))
            })
            .collect();
        candidates.sort_unstable();
        let low = candidates.first().unwrap();
        let high = candidates.last().unwrap();
        assert!(low.0 < high.0);
        let (low, high) = (low.1, high.1);
        for (selected_id, equipped_id, expected) in [
            (high, low, USER_EQUIP_RATING_BETTER_TINT),
            (low, high, USER_EQUIP_RATING_WORSE_TINT),
            (high, high, Color::WHITE),
        ] {
            let runtime = runtime_with(
                &[(kind as usize, item(kind, equipped_id, 0, 0))],
                &[(0, item(kind, selected_id, 0, 0))],
            );
            app.insert_resource(UserEquipItemModeProjection::from_authoritative(
                &runtime,
                &AllCatalog,
            ));
            app.world_mut()
                .resource_mut::<UserEquipItemPopupState>()
                .open(UserEquipSlotEndpoint::Inventory { slot_index: 0 });
            app.update();
            let index = if kind == 0 { 2 } else { 4 };
            let mut query = app
                .world_mut()
                .query::<(&UserEquipUiElement, &UserEquipBoundTextTarget)>();
            let target = query
                .iter(app.world())
                .find_map(|(element, target)| {
                    matches!(element, UserEquipUiElement::ItemPopupField(i) if *i == index)
                        .then_some(target.0)
                })
                .unwrap();
            let actual = app.world().get::<TextColor>(target).unwrap().0.to_srgba();
            let expected = expected.to_srgba();
            for (actual, expected) in actual
                .to_f32_array()
                .into_iter()
                .zip(expected.to_f32_array())
            {
                assert!((actual - expected).abs() < 1e-6);
            }
        }
    }
}

#[test]
fn popup_ratings_follow_clean_equip_and_unequip_value_color() {
    let base = Color::srgb(1.0, 1.0, 1.0);
    let (better, worse) = (USER_EQUIP_RATING_BETTER_TINT, USER_EQUIP_RATING_WORSE_TINT);
    // EquipPopup: an unequipped weapon/armor piece against its own slot.
    assert_eq!(
        user_equip_rating_colors(base, 0, [50, 30, 7], Some([40, 40, 7]), false),
        [better, worse, base]
    );
    // An empty same-type slot compares with zero.
    assert_eq!(
        user_equip_rating_colors(base, 3, [0, 0, 12], None, false),
        [base, base, better]
    );
    // Head, face and back items are never compared.
    assert_eq!(
        user_equip_rating_colors(base, 4, [9, 9, 9], Some([1, 1, 1]), false),
        [base; 3]
    );
    // UnequipPopup: the equipped item itself compares with zero.
    assert_eq!(
        user_equip_rating_colors(base, 1, [0, 0, 15], Some([0, 0, 15]), true),
        [base, base, better]
    );
    // Tints multiply the label color like Unity's GUI.color.
    let label = Color::srgb(0.8, 1.0, 1.0);
    let [tinted, ..] = user_equip_rating_colors(label, 0, [-1, 0, 0], None, false);
    let tinted = tinted.to_srgba();
    assert!((tinted.red - 0.72).abs() < 1e-6);
    assert!((tinted.green - 0.2).abs() < 1e-6);
    assert!(tinted.blue.abs() < 1e-6);
}
