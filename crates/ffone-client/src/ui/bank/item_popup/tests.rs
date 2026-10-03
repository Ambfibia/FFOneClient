use super::*;

fn fixture(inventory: bool, item_type: i16) -> (App, BankSlotRef0104, Entity) {
    let slot = BankSlotRef0104::new(
        if inventory {
            BankSlotLocation0104::Inventory
        } else {
            BankSlotLocation0104::Bank
        },
        0,
    )
    .unwrap();
    let mut p = BankModeProjection0104::default();
    p.owner_pc_id = 42;
    p.npc_id = 100;
    let item = ItemBase0104 {
        item_type,
        item_id: 1,
        option: 99,
        time_limit: 123,
    };
    if inventory {
        p.item_mode.inventory[0].item.item = item;
    } else {
        p.bank[0].item = item;
    }
    let mut popup = BankItemPopupState::default();
    popup.open(&p, slot);
    let mut app = App::new();
    app.insert_resource(p)
        .insert_resource(popup)
        .insert_resource(BankUiState {
            phase: BankLifecyclePhase::Visible,
            ..default()
        })
        .init_resource::<BankModalState>()
        .init_resource::<BankUiOutbox0104>()
        .init_resource::<BankLocalRequests>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Update, (reconcile, input).chain());
    let accept = app
        .world_mut()
        .spawn((Button, Part::Accept, Interaction::None))
        .id();
    (app, slot, accept)
}

#[test]
fn bank_popup_transfers_full_items_in_both_directions_once() {
    for inventory in [false, true] {
        for item_type in [0, 7, 9] {
            let (mut app, slot, accept) = fixture(inventory, item_type);
            let before = app.world().resource::<BankModeProjection0104>().clone();
            app.update();
            assert!(app.world().resource::<BankModalState>().inventory_popup);
            *app.world_mut().get_mut::<Interaction>(accept).unwrap() = Interaction::Pressed;
            app.update();
            let expected = before.one_click_intent(slot).unwrap().wire_request();
            assert_eq!(
                app.world_mut()
                    .resource_mut::<BankUiOutbox0104>()
                    .pop_front(),
                Some(BankUiCommand0104::ItemMove(expected))
            );
            assert_eq!(*app.world().resource::<BankModeProjection0104>(), before);
            assert!(!app.world().resource::<BankItemPopupState>().is_open());
            assert!(app.world().resource::<BankUiState>().send_pending);
            app.update();
            assert!(app.world().resource::<BankUiOutbox0104>().is_empty());
        }
    }
}

#[test]
fn bank_popup_revalidates_owner_slot_and_pending_state_before_accept() {
    for mutation in 0..5 {
        let (mut app, _, accept) = fixture(false, 7);
        app.update();
        match mutation {
            0 => {
                app.world_mut()
                    .resource_mut::<BankModeProjection0104>()
                    .owner_pc_id += 1
            }
            1 => {
                app.world_mut()
                    .resource_mut::<BankModeProjection0104>()
                    .npc_id += 1
            }
            2 => {
                app.world_mut()
                    .resource_mut::<BankModeProjection0104>()
                    .bank[0]
                    .item
                    .option += 1
            }
            3 => {
                app.world_mut().resource_mut::<BankUiState>().phase = BankLifecyclePhase::Hidden
            }
            _ => app.world_mut().resource_mut::<BankUiState>().send_pending = true,
        }
        *app.world_mut().get_mut::<Interaction>(accept).unwrap() = Interaction::Pressed;
        app.update();
        assert!(!app.world().resource::<BankItemPopupState>().is_open());
        assert!(!app.world().resource::<BankModalState>().inventory_popup);
        assert!(app.world().resource::<BankUiOutbox0104>().is_empty());
    }
}

#[test]
fn bank_popup_escape_blocks_same_frame_bank_close_and_releases_next_frame() {
    let (mut app, _, _) = fixture(false, 9);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Escape);
    app.update();
    assert!(!app.world().resource::<BankItemPopupState>().is_open());
    assert!(app.world().resource::<BankModalState>().inventory_popup);
    assert!(app.world().resource::<BankUiOutbox0104>().is_empty());
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
    app.update();
    assert!(!app.world().resource::<BankModalState>().inventory_popup);
}

#[test]
fn bank_popup_keeps_other_modals_and_reports_full_destination() {
    let (mut app, _, accept) = fixture(false, 9);
    for slot in &mut app
        .world_mut()
        .resource_mut::<BankModeProjection0104>()
        .item_mode
        .inventory
    {
        slot.item.item = ItemBase0104 {
            item_type: 7,
            item_id: 1,
            option: 1,
            time_limit: 0,
        };
    }
    app.world_mut()
        .resource_mut::<BankModalState>()
        .system_popup = true;
    *app.world_mut().get_mut::<Interaction>(accept).unwrap() = Interaction::Pressed;
    app.update();
    assert!(app.world().resource::<BankItemPopupState>().is_open());
    app.world_mut()
        .resource_mut::<BankModalState>()
        .system_popup = false;
    *app.world_mut().get_mut::<Interaction>(accept).unwrap() = Interaction::Pressed;
    app.update();
    assert_eq!(
        app.world().resource::<BankLocalRequests>().full,
        Some(BankSlotLocation0104::Inventory)
    );
    assert!(!app.world().resource::<BankItemPopupState>().is_open());
    assert!(app.world().resource::<BankUiOutbox0104>().is_empty());
}

#[test]
fn bank_popup_content_text_uses_resolved_semantic_rows_and_switches_language() {
    use crate::{
        assets::AssetLocator, localization::Localization,
        tutorial_mission_content::TutorialMissionContent,
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let content = TutorialMissionContent::open(&AssetLocator::open(&root).unwrap()).unwrap();
    let (localization, en) = Localization::open(&root, "en").unwrap();
    let (_, ru) = Localization::open(&root, "ru").unwrap();
    let (name, description) = content.gameplay_user_equip_item_text(7, 7).unwrap();
    assert_eq!(
        name.key,
        "content.tabledata.general_item.item_string.7.str_name"
    );
    assert_eq!(localization.text(&en, &name), "Damage Power Item");
    assert_ne!(localization.text(&en, &name), localization.text(&ru, &name));
    assert_ne!(
        localization.text(&en, &description),
        localization.text(&ru, &description)
    );
    let (name, _) = content.gameplay_user_equip_item_text(9, 1194).unwrap();
    assert_eq!(
        name.key,
        "content.tabledata.chest_item.item_string.274.str_name"
    );
    assert_eq!(localization.text(&en, &name), "10Lv CRATE");
}

#[test]
fn bank_popup_labels_exist_in_both_production_bundles() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game/localization");
    let en: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("en.json")).unwrap()).unwrap();
    let ru: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("ru.json")).unwrap()).unwrap();
    assert_eq!(
        en["entries"]
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>(),
        ru["entries"]
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>()
    );
    for inventory in [false, true] {
        let copy = transfer_label(inventory);
        assert!(en["entries"][&copy.key].is_string());
        assert!(ru["entries"][&copy.key].is_string());
    }
}
