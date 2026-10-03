use super::operations_bind::popup_position;
use super::*;

#[test]
fn vendor_popup_follows_selected_row_and_inventory_cell_at_multiple_viewports() {
    for viewport in [Vec2::new(1020., 638.), Vec2::new(1600., 900.)] {
        let layout = vendor_mode_layout(viewport.x as u32, viewport.y as u32, 1., 0., 0., 20);
        let first = popup_position(
            layout,
            VendorItemPopupSource0104::CatalogRow { row_index: 0 },
            viewport,
            449.,
            false,
        )
        .unwrap();
        let later = popup_position(
            layout,
            VendorItemPopupSource0104::CatalogRow { row_index: 3 },
            viewport,
            449.,
            false,
        )
        .unwrap();
        assert!(later.y > first.y);
        assert_eq!(later.x, first.x);

        let inventory = popup_position(
            layout,
            VendorItemPopupSource0104::InventorySlot { inventory_slot: 6 },
            viewport,
            449.,
            false,
        )
        .unwrap();
        assert!(inventory.x >= 0. && inventory.x + 310. <= viewport.x);
        assert!(inventory.y >= 0. && inventory.y + 449. <= viewport.y);
        let close = inventory + Vec2::new(277., 0.);
        assert!(close.x >= inventory.x && close.x + 32. <= inventory.x + 310.);
    }
}

#[test]
fn production_card_binding_keeps_semantic_keys_across_item_and_language_changes() {
    use crate::localization::{Localization, LocalizationPlugin, LocalizedTextCase};
    use crate::{assets::AssetLocator, tutorial_mission_content::TutorialMissionContent};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let content = TutorialMissionContent::open(&AssetLocator::open(&root).unwrap()).unwrap();
    let item = |id, option| ItemBase0104 {
        item_type: 1,
        item_id: id,
        option,
        time_limit: 0,
    };
    assert!(content.gameplay_try_on_allowed(item(100, 0), 1, 0));
    assert!(!content.gameplay_try_on_allowed(item(100, 0), 2, 0));
    assert!(content.gameplay_try_on_allowed(item(100, 107 << 16), 2, 0));
    assert!(!content.gameplay_try_on_allowed(item(100, 107 << 16), 1, 0));
    assert!(content.gameplay_try_on_allowed(item(16, 0), 1, 4));
    assert!(!content.gameplay_try_on_allowed(item(16, 0), 1, 0));
    let (localization, language) = Localization::open(&root, "en").unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::asset::AssetPlugin {
            file_path: root.to_string_lossy().into(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .insert_resource(content)
        .insert_resource(localization)
        .insert_resource(language)
        .insert_resource(VendorUiAssets {
            images: array::from_fn(|_| Handle::default()),
            font: default(),
            service_font: default(),
            missing_checker: default(),
        })
        .init_resource::<VendorItemPopupState>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_plugins(LocalizationPlugin)
        .add_systems(Update, bind.before(LocalizationSet::Apply));
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    let name = app
        .world_mut()
        .spawn((
            Part::Name,
            Node::default(),
            Text::default(),
            LocalizedText::new("ui.vendor.item_name", "{name}").with_arg("name", "stale"),
        ))
        .id();
    let description = app
        .world_mut()
        .spawn((
            Part::Description,
            Node::default(),
            Text::default(),
            LocalizedText::new("ui.inventory.popup.description", "{description}")
                .with_arg("description", "stale"),
        ))
        .id();
    let icon = app
        .world_mut()
        .spawn((Part::Icon, Node::default(), ImageNode::default()))
        .id();
    let rarity = app
        .world_mut()
        .spawn((
            Part::Detail(11),
            Node::default(),
            LocalizedText::new("ui.inventory.rarity.common", "Common"),
        ))
        .id();
    let trade = app
        .world_mut()
        .spawn((
            Part::Detail(12),
            Node::default(),
            LocalizedText::new("ui.inventory.popup.trade_value", "Tradable"),
        ))
        .id();
    let badge = app
        .world_mut()
        .spawn((Part::CombinedBadge, Node::default()))
        .id();
    let stats = app.world_mut().spawn((Part::Info, Node::default())).id();
    let rating_labels: [Entity; 3] = std::array::from_fn(|index| {
        app.world_mut()
            .spawn((
                Part::Detail(index as u8 + 1),
                Node::default(),
                Text::default(),
                LocalizedText::new("ui.inventory.popup.point_value", "{value}"),
                TextColor(Color::WHITE),
            ))
            .id()
    });
    let range = app
        .world_mut()
        .spawn((
            Part::Detail(6),
            Node::default(),
            Text::default(),
            LocalizedText::new("ui.inventory.popup.range", "Range"),
        ))
        .id();
    let speed = app
        .world_mut()
        .spawn((
            Part::Detail(10),
            Node::default(),
            Text::default(),
            LocalizedText::new("ui.inventory.popup.not_available", "N/A"),
        ))
        .id();
    for (item_type, item_id, option) in [
        (1, 100, 107 << 16),
        (10, 1, 0),
        (0, 328, 0),
        (1, 100, i32::MIN),
        (7, 7, 0),
        (9, 1194, 0),
    ] {
        let mut projection = fixture(item_type == 7);
        projection.catalog_rows[0].item.item_type = item_type;
        projection.catalog_rows[0].item.item_id = item_id;
        projection.catalog_rows[0].item.option = option;
        let popup = open(&projection);
        app.insert_resource(popup);
        app.insert_resource(projection.clone());
        for locale in ["en", "ru"] {
            let (_, language) = Localization::open(&root, locale).unwrap();
            app.insert_resource(language);
            app.update();
            let world = app.world();
            assert_eq!(
                world.get::<Node>(badge).unwrap().display == Display::Flex,
                item_type == 1 && option > 0
            );
            assert_eq!(
                world.get::<Node>(stats).unwrap().display == Display::Flex,
                ![7, 9, 10].contains(&item_type)
            );
            if ![7, 9].contains(&item_type) {
                assert_eq!(
                    world.get::<LocalizedText>(range).unwrap().key,
                    if item_type == 10 {
                        "ui.inventory.popup.speed"
                    } else {
                        "ui.inventory.popup.range"
                    }
                );
                if item_type == 10 {
                    assert_eq!(
                        world.get::<LocalizedText>(speed).unwrap().key,
                        "ui.inventory.vehicle_class"
                    );
                } else if item_type != 0 {
                    assert_eq!(
                        world.get::<LocalizedText>(speed).unwrap().key,
                        "ui.inventory.popup.not_available"
                    );
                }
            }
            let item = projection.catalog_rows[0].item;
            let content = world.resource::<TutorialMissionContent>();
            let equipped = projection
                .equipment
                .iter()
                .find(|slot| slot.wire_slot_index == item_type as usize && !slot.item.empty)
                .map(|slot| slot.item.item);
            let expected = crate::user_equip_ui::user_equip_item_rating_colors(
                content,
                Color::srgb(0.8, 1., 1.),
                item,
                equipped,
                false,
            );
            for (entity, color) in rating_labels.into_iter().zip(expected) {
                assert_eq!(world.get::<TextColor>(entity).unwrap().0, color);
            }
            let is_combined = (0..=3).contains(&item_type) && (option >> 16) as i16 > 0;
            if is_combined {
                assert_eq!(
                    world.get::<LocalizedText>(rarity).unwrap().key,
                    "ui.inventory.rarity.special"
                );
                assert_eq!(
                    world.get::<LocalizedText>(trade).unwrap().key,
                    "ui.inventory.popup.not_tradable"
                );
                let base = content.resolve(ItemBase0104 { option: 0, ..item }).unwrap();
                let appearance = content
                    .resolve(ItemBase0104 {
                        item_id: 107,
                        option: 0,
                        ..item
                    })
                    .unwrap();
                let combined_metadata = content.resolve(item).unwrap();
                assert_eq!(combined_metadata.icon, appearance.icon);
                assert_eq!(combined_metadata.level, base.level);
                assert_eq!(combined_metadata.buy_price, base.buy_price);
                assert_eq!(combined_metadata.sell_price, base.sell_price);
                assert_eq!(combined_metadata.sellable, base.sellable);
            } else if ![7, 9].contains(&item_type) {
                assert_ne!(
                    world.get::<LocalizedText>(rarity).unwrap().key,
                    "ui.inventory.rarity.special"
                );
            }
            if let Some(path) = content.gameplay_item_display_icon(item) {
                let handle: Handle<Image> = world.resource::<AssetServer>().load(path.to_owned());
                assert_eq!(world.get::<ImageNode>(icon).unwrap().image, handle);
            }
            let expected = world
                .resource::<TutorialMissionContent>()
                .gameplay_user_equip_item_text(
                    item_type,
                    crate::user_equip_ui::user_equip_display_text_id(
                        projection.catalog_rows[0].item,
                    ),
                )
                .unwrap();
            for (entity, spec) in [(name, expected.0), (description, expected.1)] {
                assert_eq!(world.entity(entity).get::<LocalizedText>().unwrap(), &spec);
                let mut resolved = world
                    .resource::<Localization>()
                    .text(world.resource::<crate::localization::Language>(), &spec);
                if entity == description && item_type == 7 {
                    resolved = resolved.to_uppercase();
                }
                assert_eq!(world.entity(entity).get::<Text>().unwrap().0, resolved);
            }
            assert_eq!(
                world.entity(description).contains::<LocalizedTextCase>(),
                item_type == 7
            );
        }
    }
}

#[test]
fn try_on_draft_closes_with_card_and_permission_and_excludes_non_catalog_items() {
    let p = fixture(false);
    let before = p.clone();
    let mut popup = open(&p);
    assert!(popup.selected_try_on_item().is_some());
    popup.set_try_on_allowed(true);
    popup.try_on = true;
    assert_eq!(popup.try_on_item(), Some(p.catalog_rows[0].item));
    popup.set_try_on_allowed(false);
    popup.set_try_on_allowed(true);
    assert!(popup.try_on_item().is_none());
    popup.try_on = true;
    popup.close();
    assert!(popup.try_on_item().is_none());
    assert_eq!(p, before);
    assert!(open(&fixture(true)).selected_try_on_item().is_none());
}

#[test]
fn deferred_login_phase_does_not_require_gameplay_popup_assets() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .insert_state(crate::ui_startup::NativeUiStartupPhase::Deferred)
        .add_plugins(VendorUiPlugin);
    app.update();
    app.update();
    assert!(!app.world().contains_resource::<VendorUiAssets>());
    assert!(!app.world().resource::<VendorItemPopupState>().is_open());
}

fn fixture(general: bool) -> VendorModeProjection0104 {
    let mut p = VendorModeProjection0104::default();
    p.owner_pc_id = 42;
    p.taros = 100;
    p.catalog_rows.push(VendorCatalogRowProjection0104 {
        packet_index: 0,
        source_slot_id: 7,
        item: ItemBase0104 {
            item_type: if general { 7 } else { 0 },
            item_id: 1,
            option: 0,
            time_limit: 0,
        },
        metadata: Some(VendorItemMetadata0104 {
            name: "Test".into(),
            level: 1,
            buy_price: 10,
            sell_price: 5,
            sellable: true,
            general_item_type: Some(1),
            battery_recharge: None,
            stack_size: Some(99),
            icon: None,
        }),
        icon: VendorPresentationIcon0104::Empty,
        price: Some(10),
        affordable: Some(true),
        equip_validation: VendorEquipValidation0104::Allowed,
        vehicle_speed_class: None,
    });
    p
}
#[test]
fn chest_open_input_is_single_shot_and_rejects_stale_authority() {
    for mutation in 0..7 {
        let mut p = fixture(false);
        p.inventory[0].item.empty = false;
        p.inventory[0].item.item = ItemBase0104 {
            item_type: 9,
            item_id: 77,
            option: 1,
            time_limit: 0,
        };
        let VendorActivationOutcome0104::Popup(contract) = p.primary_inventory_activation(0) else {
            panic!()
        };
        let mut popup = VendorItemPopupState::default();
        popup.open(contract, &p);
        let before = p.inventory[0].item.item;
        match mutation {
            1 => p.owner_pc_id += 1,
            2 => p.session.accepted_npc_id += 1,
            3 => p.inventory[0].item.item.option += 1,
            4 => p.inventory[0].item.item.time_limit += 1,
            _ => (),
        }
        let mut app = App::new();
        app.insert_resource(p)
            .insert_resource(popup)
            .insert_resource(VendorUiState {
                phase: if mutation == 5 {
                    VendorLifecyclePhase::Hidden
                } else {
                    VendorLifecyclePhase::Visible
                },
                send_pending: mutation == 6,
                ..default()
            })
            .init_resource::<VendorModalState>()
            .init_resource::<VendorUiOutbox0104>()
            .init_resource::<VendorUiAudioOutbox0104>()
            .init_resource::<VendorChestOpenState>()
            .add_systems(Update, input);
        app.world_mut()
            .spawn((Button, Part::Accept, Interaction::Pressed));
        app.update();
        let ready = app
            .world_mut()
            .resource_mut::<VendorChestOpenState>()
            .take_ready();
        assert_eq!(ready.is_some(), mutation == 0, "mutation {mutation}");
        if let Some(intent) = ready {
            assert_eq!(intent.item, before);
        }
        app.update();
        assert!(!app.world().resource::<VendorChestOpenState>().has_ready());
        assert!(app.world().resource::<VendorUiOutbox0104>().is_empty());
    }
}
fn open(p: &VendorModeProjection0104) -> VendorItemPopupState {
    let VendorActivationOutcome0104::Popup(contract) =
        p.primary_row_activation(VendorTab0104::Buy, 0)
    else {
        panic!()
    };
    let mut popup = VendorItemPopupState::default();
    popup.open(contract, p);
    assert!(popup.is_open());
    popup
}
#[test]
fn quantity_requires_positive_input_caps_to_affordable_count_and_commits_once() {
    let p = fixture(true);
    let before = p.clone();
    let mut popup = open(&p);
    assert!(popup.commit(false, &p).is_none());
    assert!(popup.is_open());
    popup.digit(9);
    popup.digit(9);
    assert_eq!(popup.amount, 10);
    let (_, result) = popup.commit(false, &p).unwrap();
    assert!(matches!(result, VendorActionOutcome0104::Intent(_)));
    assert!(!popup.is_open());
    assert!(popup.commit(false, &p).is_none());
    assert_eq!(p, before, "a popup must not commit local currency or items");
}
#[test]
fn fixed_zero_buy_is_a_valid_single_item_purchase() {
    let p = fixture(false);
    let mut popup = open(&p);
    assert!(matches!(
        popup.commit(false, &p),
        Some((_, VendorActionOutcome0104::Intent(_)))
    ));
}
#[test]
fn source_replacement_character_change_and_maximum_change_invalidate_draft() {
    for mutation in 0..6 {
        let mut p = fixture(true);
        let mut popup = open(&p);
        popup.digit(1);
        match mutation {
            0 => p.catalog_rows[0].item.item_id += 1,
            1 => p.owner_pc_id += 1,
            2 => p.catalog_rows[0].source_slot_id += 1,
            3 => p.session.accepted_npc_id += 1,
            4 => {
                p.catalog_rows[0]
                    .metadata
                    .as_mut()
                    .unwrap()
                    .general_item_type = Some(2)
            }
            _ => p.taros = 0,
        }
        assert!(popup.commit(false, &p).is_none());
        assert!(!popup.is_open());
    }
}

#[test]
fn quantity_popup_keeps_item_a_when_background_row_b_is_pressed() {
    let mut p = fixture(true);
    let mut b = p.catalog_rows[0].clone();
    b.packet_index = 1;
    b.source_slot_id = 8;
    b.item.item_id = 2;
    b.metadata.as_mut().unwrap().buy_price = 20;
    b.price = Some(20);
    p.catalog_rows.push(b);
    let a = p.catalog_rows[0].item;
    let mut popup = open(&p);
    popup.digit(2);
    assert_eq!(popup.cost_label().as_deref(), Some("10 × 2 = 20"));
    let mut app = App::new();
    app.insert_resource(p.clone())
        .insert_resource(popup)
        .insert_resource(VendorUiState {
            phase: VendorLifecyclePhase::Visible,
            ..default()
        })
        .insert_resource(VendorModalState {
            // The production sync clears this before popup reconciliation.
            inventory_popup: false,
            ..default()
        })
        .init_resource::<VendorCloseGate0104>()
        .init_resource::<VendorHoverState>()
        .init_resource::<VendorUiOutbox0104>()
        .init_resource::<VendorUiAudioOutbox0104>()
        .add_systems(Update, (reconcile, collect_vendor_ui_input).chain());
    app.world_mut()
        .spawn((VendorInteractiveControl::Row(1), Interaction::Pressed));
    app.update();
    assert!(app.world().resource::<VendorModalState>().inventory_popup);
    assert!(app.world().resource::<VendorUiOutbox0104>().is_empty());
    let mut popup = app.world_mut().resource_mut::<VendorItemPopupState>();
    assert_eq!(popup.selection.as_ref().unwrap().item, a);
    assert_eq!(popup.selection.as_ref().unwrap().unit_price, Some(10));
    assert_eq!(popup.amount, 2);
    assert_eq!(popup.cost_label().as_deref(), Some("10 × 2 = 20"));
    let (_, outcome) = popup.commit(false, &p).unwrap();
    let VendorActionOutcome0104::Intent(VendorIntent0104::Buy(intent)) = outcome else {
        panic!("expected a buy intent for A");
    };
    assert_eq!(intent.item.item_id, a.item_id);
    assert_eq!(intent.item.option, 2);
}

#[test]
fn quantity_popup_rejects_changed_price_and_cancel_clears_selection() {
    let mut p = fixture(true);
    let mut popup = open(&p);
    popup.digit(2);
    p.catalog_rows[0].metadata.as_mut().unwrap().buy_price = 11;
    p.catalog_rows[0].price = Some(11);
    assert!(popup.commit(false, &p).is_none());
    assert!(!popup.is_open());
    assert_eq!(popup.amount, 0);

    let mut popup = open(&p);
    popup.digit(3);
    popup.close();
    assert!(!popup.is_open());
    assert_eq!(popup.amount, 0);
    assert!(popup.commit(false, &p).is_none());
}

#[test]
fn escape_cancels_quantity_without_sending_an_intent() {
    let p = fixture(true);
    let popup = open(&p);
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::Escape);
    let mut app = App::new();
    app.insert_resource(p)
        .insert_resource(popup)
        .insert_resource(keys)
        .insert_resource(VendorUiState {
            phase: VendorLifecyclePhase::Visible,
            ..default()
        })
        .insert_resource(VendorModalState {
            inventory_popup: true,
            ..default()
        })
        .init_resource::<VendorUiOutbox0104>()
        .init_resource::<VendorUiAudioOutbox0104>()
        .init_resource::<VendorChestOpenState>()
        .add_systems(Update, input);
    app.update();
    assert!(!app.world().resource::<VendorItemPopupState>().is_open());
    assert!(!app.world().resource::<VendorModalState>().inventory_popup);
    assert!(app.world().resource::<VendorUiOutbox0104>().is_empty());
}

#[test]
fn keyboard_quantity_and_enter_send_the_selected_item_once() {
    let p = fixture(true);
    let item_id = p.catalog_rows[0].item.item_id;
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::Numpad2);
    let mut app = App::new();
    app.insert_resource(p.clone())
        .insert_resource(open(&p))
        .insert_resource(keys)
        .insert_resource(VendorUiState {
            phase: VendorLifecyclePhase::Visible,
            ..default()
        })
        .insert_resource(VendorModalState {
            inventory_popup: true,
            ..default()
        })
        .init_resource::<VendorUiOutbox0104>()
        .init_resource::<VendorUiAudioOutbox0104>()
        .init_resource::<VendorChestOpenState>()
        .add_systems(Update, input);
    app.update();
    assert_eq!(app.world().resource::<VendorItemPopupState>().amount, 2);
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.clear();
    keys.press(KeyCode::NumpadEnter);
    drop(keys);
    app.update();
    let command = app
        .world_mut()
        .resource_mut::<VendorUiOutbox0104>()
        .pop_front();
    assert!(matches!(
        command,
        Some(VendorUiCommand0104::Intent(VendorIntent0104::Buy(intent)))
            if intent.item.item_id == item_id && intent.item.option == 2
    ));
    app.update();
    assert!(app.world().resource::<VendorUiOutbox0104>().is_empty());
}
#[test]
fn locale_bundles_own_popup_actions() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game/localization");
    let read = |locale: &str| -> serde_json::Value {
        serde_json::from_slice(&std::fs::read(root.join(format!("{locale}.json"))).unwrap())
            .unwrap()
    };
    let en = read("en");
    let ru = read("ru");
    let en = en["entries"].as_object().unwrap();
    let ru = ru["entries"].as_object().unwrap();
    assert_eq!(en.keys().collect::<Vec<_>>(), ru.keys().collect::<Vec<_>>());
    for key in [
        "ui.vendor.popup.buy",
        "ui.vendor.popup.sell",
        "ui.vendor.popup.buyback",
        "ui.vendor.popup.try_on",
        "ui.inventory.action.open",
    ] {
        assert!(en[key].as_str().unwrap().len() > 0);
        assert!(ru[key].as_str().unwrap().len() > 0);
    }
}

#[test]
fn production_button_input_blocks_system_modal_and_emits_one_vendor_request() {
    let p = fixture(true);
    let popup = open(&p);
    let mut app = App::new();
    app.insert_resource(p)
        .insert_resource(popup)
        .insert_resource(VendorUiState {
            phase: VendorLifecyclePhase::Visible,
            ..default()
        })
        .insert_resource(VendorModalState {
            inventory_popup: true,
            system_popup: true,
            ..default()
        })
        .init_resource::<VendorUiOutbox0104>()
        .init_resource::<VendorUiAudioOutbox0104>()
        .init_resource::<VendorChestOpenState>()
        .add_systems(Update, input);
    let button = app
        .world_mut()
        .spawn((Button, Part::Digit(2), Interaction::Pressed))
        .id();
    app.update();
    assert_eq!(app.world().resource::<VendorItemPopupState>().amount, 0);
    app.world_mut()
        .resource_mut::<VendorModalState>()
        .system_popup = false;
    app.world_mut().entity_mut(button).insert(Interaction::None);
    app.update();
    app.world_mut()
        .entity_mut(button)
        .insert(Interaction::Pressed);
    app.update();
    assert_eq!(app.world().resource::<VendorItemPopupState>().amount, 2);
    app.world_mut()
        .entity_mut(button)
        .insert((Part::Accept, Interaction::None));
    app.update();
    app.world_mut()
        .entity_mut(button)
        .insert(Interaction::Pressed);
    app.update();
    assert!(!app.world().resource::<VendorItemPopupState>().is_open());
    assert!(matches!(
        app.world_mut()
            .resource_mut::<VendorUiOutbox0104>()
            .pop_front(),
        Some(VendorUiCommand0104::Intent(_))
    ));
    app.update();
    assert!(app.world().resource::<VendorUiOutbox0104>().is_empty());
}
