use super::*;
#[test]
fn production_fields_follow_selected_item_and_clear_on_close() {
    use crate::user_equip_ui::{
        UserEquipItemModeProjection, UserEquipItemPopupState, UserEquipSlotEndpoint,
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let (loc, language) = Localization::open(&root, "ru").unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(loc)
        .insert_resource(language)
        .init_resource::<UserEquipItemModeProjection>()
        .init_resource::<UserEquipItemPopupState>()
        .add_plugins(ItemCardPlugin);
    let rental = app
        .world_mut()
        .spawn((
            CardOwner::Inventory,
            Field::Rental,
            Node::default(),
            LocalizedText::new("ui.card.rental_value", "{value}"),
        ))
        .id();
    let expiry = app
        .world_mut()
        .spawn((
            CardOwner::Inventory,
            Field::Expiry,
            Node::default(),
            LocalizedText::new("ui.card.expiry_compact", "{date} {time}"),
        ))
        .id();
    app.world_mut()
        .resource_mut::<UserEquipItemPopupState>()
        .open(UserEquipSlotEndpoint::Inventory { slot_index: 0 });
    for (kind, limit, show_rental, show_expiry) in [
        (10, 1800000000, true, false),
        (10, 0, false, false),
        (1, 1800000000, false, true),
        (1, 0, false, false),
    ] {
        let mut p = app
            .world_mut()
            .resource_mut::<UserEquipItemModeProjection>();
        p.inventory[0].item.item = ItemBase0104 {
            item_type: kind,
            item_id: 1,
            option: 0,
            time_limit: limit,
        };
        p.inventory[0].item.empty = false;
        drop(p);
        app.update();
        assert_eq!(
            app.world().get::<Node>(rental).unwrap().display == Display::Flex,
            show_rental
        );
        assert_eq!(
            app.world().get::<Node>(expiry).unwrap().display == Display::Flex,
            show_expiry
        );
    }
    app.world_mut()
        .resource_mut::<UserEquipItemPopupState>()
        .close();
    app.update();
    assert_eq!(
        app.world().get::<Node>(rental).unwrap().display,
        Display::None
    );
    assert_eq!(
        app.world().get::<Node>(expiry).unwrap().display,
        Display::None
    );
}
#[test]
fn rental_contract_distinguishes_duration_from_expiry_and_localizes_every_unit() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    for locale in ["en", "ru"] {
        let (loc, lang) = Localization::open(&root, locale).unwrap();
        let item = ItemBase0104 {
            item_type: 10,
            item_id: 1,
            option: 0,
            time_limit: 90061,
        };
        let s = CardSnapshot {
            item,
            catalog: true,
            price: None,
        };
        let text = rental(s, &loc, &lang);
        assert_eq!(text.key, "ui.card.rental_from_purchase");
        let output = loc.text(&lang, &text);
        assert!(!output.contains('{'));
        assert_eq!(output.matches('1').count(), 4);
        let expiry = rental(
            CardSnapshot {
                catalog: false,
                ..s
            },
            &loc,
            &lang,
        );
        assert_eq!(expiry.key, "ui.card.rental_expires");
        assert!(!loc.text(&lang, &expiry).contains('{'));
    }
    assert_eq!(duration_parts(86400).map(|p| p.0), [1, 0, 0, 0]);
    assert_eq!(duration_parts(59).map(|p| p.0), [0, 0, 0, 59]);
    assert_eq!(duration_parts(0).map(|p| p.0), [0, 0, 0, 0]);
}
