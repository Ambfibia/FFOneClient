use super::*;
fn fixture() -> App {
    let mut p = BankModeProjection0104::default();
    p.owner_pc_id = 42;
    p.npc_id = 815;
    p.item_mode.inventory[0].item.item = ItemBase0104 {
        item_type: 7,
        item_id: 7,
        option: 99,
        time_limit: 0,
    };
    p.item_mode.inventory[0].item.icon = UserEquipProjectedIcon::Resolved(
        UserEquipIconRef::new("icons/items/general/generalitemicon_16.png").unwrap(),
    );
    let mut messages = SystemMessageUiModel::default();
    let mut deletion = BankItemDeleteState::default();
    assert!(deletion.open(&p, 0, &mut messages));
    let mut app = App::new();
    app.insert_resource(p)
        .insert_resource(messages)
        .insert_resource(deletion)
        .insert_resource(BankUiState {
            phase: BankLifecyclePhase::Visible,
            ..default()
        })
        .init_resource::<BankModalState>()
        .init_resource::<SystemMessageUiOutbox>()
        .add_systems(Update, consume);
    app
}
fn choose(app: &mut App, choice: SystemMessageChoice) {
    let id = app
        .world()
        .resource::<BankItemDeleteState>()
        .confirmation
        .unwrap()
        .0;
    app.world_mut()
        .resource_mut::<SystemMessageUiOutbox>()
        .push(SystemMessageUiAction::Chosen {
            request_id: id,
            button_type: SystemMessageButtonType::DeleteItem,
            choice,
        });
}
#[test]
fn bank_delete_confirmation_carries_icon_stack_and_queues_only_after_yes() {
    let mut app = fixture();
    let before = app.world().resource::<BankModeProjection0104>().clone();
    let prompt = &app.world().resource::<SystemMessageUiModel>().stack()[0];
    assert_eq!(
        prompt.icon_path.as_deref(),
        Some("icons/items/general/generalitemicon_16.png")
    );
    assert_eq!(prompt.icon_quantity, 99);
    assert!(!app.world().resource::<BankItemDeleteState>().has_ready());
    choose(&mut app, SystemMessageChoice::Primary);
    app.update();
    let intent = app
        .world_mut()
        .resource_mut::<BankItemDeleteState>()
        .take_ready()
        .unwrap();
    assert_eq!(intent.item.option, 99);
    assert_eq!(intent.slot, 0);
    assert_eq!(*app.world().resource::<BankModeProjection0104>(), before);
    assert!(
        app.world_mut()
            .resource_mut::<BankItemDeleteState>()
            .take_ready()
            .is_none()
    );
}
#[test]
fn bank_delete_cancel_does_not_queue_and_preserves_foreign_responses() {
    let mut app = fixture();
    app.world_mut()
        .resource_mut::<SystemMessageUiOutbox>()
        .push(SystemMessageUiAction::Chosen {
            request_id: 7,
            button_type: SystemMessageButtonType::Ok,
            choice: SystemMessageChoice::Primary,
        });
    choose(&mut app, SystemMessageChoice::Secondary);
    app.update();
    assert!(!app.world().resource::<BankItemDeleteState>().has_ready());
    assert_eq!(
        app.world_mut()
            .resource_mut::<SystemMessageUiOutbox>()
            .drain()
            .count(),
        1
    );
}
#[test]
fn bank_delete_rejects_changed_owner_item_mode_or_pending_request() {
    for mutation in 0..5 {
        let mut app = fixture();
        choose(&mut app, SystemMessageChoice::Primary);
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
                    .item_mode
                    .inventory[0]
                    .item
                    .item
                    .option -= 1
            }
            3 => {
                app.world_mut().resource_mut::<BankUiState>().phase = BankLifecyclePhase::Hidden
            }
            _ => app.world_mut().resource_mut::<BankUiState>().send_pending = true,
        }
        app.update();
        assert!(!app.world().resource::<BankItemDeleteState>().has_ready());
        assert!(
            app.world()
                .resource::<SystemMessageUiModel>()
                .stack()
                .is_empty()
        );
    }
}
#[test]
fn bank_delete_reopened_confirmation_never_accepts_old_choice() {
    let mut app = fixture();
    let old_id = app
        .world()
        .resource::<BankItemDeleteState>()
        .confirmation
        .unwrap()
        .0;
    choose(&mut app, SystemMessageChoice::Secondary);
    app.update();
    let p = app.world().resource::<BankModeProjection0104>().clone();
    app.world_mut()
        .resource_scope(|world, mut deletion: Mut<BankItemDeleteState>| {
            assert!(deletion.open(&p, 0, &mut world.resource_mut::<SystemMessageUiModel>()));
        });
    let new_id = app
        .world()
        .resource::<BankItemDeleteState>()
        .confirmation
        .unwrap()
        .0;
    assert_ne!(old_id, new_id);
    app.world_mut()
        .resource_mut::<SystemMessageUiOutbox>()
        .push(SystemMessageUiAction::Chosen {
            request_id: old_id,
            button_type: SystemMessageButtonType::DeleteItem,
            choice: SystemMessageChoice::Primary,
        });
    app.update();
    assert!(!app.world().resource::<BankItemDeleteState>().has_ready());
    assert!(app.world().resource::<SystemMessageUiOutbox>().is_empty());
}
