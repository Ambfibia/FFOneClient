use super::*;

#[test]
fn bank_full_inventory_reports_localized_modal_without_move() {
    use crate::system_message_ui::{
        SystemMessageChoice, SystemMessageUiModel, SystemMessageUiOutbox,
    };
    let (mut app, _, _) = pointer_test_app();
    let full = (0..INVENTORY_SLOT_COUNT_0104)
        .map(|slot| (slot, item(7, 77, 2, 0)))
        .collect::<Vec<_>>();
    app.insert_resource(projection(1, &[(0, item(7, 77, 2, 0))], &full))
        .init_resource::<SystemMessageUiModel>()
        .init_resource::<SystemMessageUiOutbox>()
        .add_systems(
            Update,
            collect_bank_local_controls.after(collect_bank_slot_input),
        );
    pointer_edge(&mut app, MouseButton::Right, true);
    pointer_edge(&mut app, MouseButton::Right, false);
    assert!(app.world().resource::<BankUiOutbox0104>().is_empty());
    assert!(!app.world().resource::<BankUiState>().send_pending);
    assert!(app.world().resource::<BankModalState>().system_popup);
    let current = app
        .world()
        .resource::<SystemMessageUiModel>()
        .current()
        .unwrap();
    assert_eq!(
        current.localized,
        bank_full_localized(BankSlotLocation0104::Inventory)
    );
    let choice = app
        .world_mut()
        .resource_mut::<SystemMessageUiModel>()
        .choose(SystemMessageChoice::Primary)
        .unwrap();
    app.world_mut()
        .resource_mut::<SystemMessageUiOutbox>()
        .push(choice);
    app.update();
    assert!(app.world().resource::<SystemMessageUiOutbox>().is_empty());
    for locale in ["en", "ru"] {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game/localization");
        let bundle: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join(format!("{locale}.json"))).unwrap())
                .unwrap();
        for location in [BankSlotLocation0104::Inventory, BankSlotLocation0104::Bank] {
            assert!(
                bundle["entries"][bank_full_localized(location).key]
                    .as_str()
                    .is_some()
            );
        }
    }
}
