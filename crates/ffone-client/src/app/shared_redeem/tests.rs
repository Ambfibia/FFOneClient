use super::*;

fn fixture() -> App {
    let mut app = App::new();
    let mut input = SharedInputDialog::default();
    let mut redeem = SharedRedeemCode::default();
    let mut status = RuntimeStatus::default();
    status.player_id = Some(42);
    assert!(redeem.open(RedeemSource::Bank { pc: 42, npc: 815 }, &mut input));
    app.insert_resource(State::new(ClientState::World))
        .insert_resource(status)
        .insert_resource(NetworkBridge::start())
        .insert_resource(BankUiState {
            phase: BankLifecyclePhase::Visible,
            ..default()
        })
        .insert_resource(BankModeProjection0104 {
            owner_pc_id: 42,
            npc_id: 815,
            ..default()
        })
        .init_resource::<VendorUiState>()
        .init_resource::<UserEquipUiState>()
        .init_resource::<UserEquipItemModeProjection>()
        .init_resource::<UserEquipModalState>()
        .init_resource::<VendorModeProjection0104>()
        .init_resource::<BankModalState>()
        .init_resource::<VendorModalState>()
        .insert_resource(input)
        .insert_resource(redeem)
        .init_resource::<SystemMessageUiModel>()
        .init_resource::<SystemMessageUiOutbox>()
        .add_systems(Update, consume_shared_redeem);
    app
}

#[test]
fn shared_redeem_short_stays_open_and_space_opens_localized_error() {
    let mut app = fixture();
    {
        let mut input = app.world_mut().resource_mut::<SharedInputDialog>();
        input.append("ab");
        input.submit();
    }
    app.update();
    assert_eq!(
        app.world().resource::<SharedInputDialog>().owner(),
        Some(REDEEM_INPUT_OWNER)
    );
    assert!(app.world().resource::<BankModalState>().inventory_popup);
    {
        let mut input = app.world_mut().resource_mut::<SharedInputDialog>();
        input.append(" c");
        input.submit();
    }
    app.update();
    assert!(
        app.world()
            .resource::<SharedInputDialog>()
            .owner()
            .is_none()
    );
    assert_eq!(
        app.world()
            .resource::<SystemMessageUiModel>()
            .current()
            .unwrap()
            .localized
            .key,
        "ui.shared_input.redeem_space_error"
    );
}

#[test]
fn shared_redeem_discards_submission_after_character_change() {
    let mut app = fixture();
    {
        let mut input = app.world_mut().resource_mut::<SharedInputDialog>();
        input.append("BeMore");
        input.submit();
    }
    app.world_mut().resource_mut::<RuntimeStatus>().player_id = Some(43);
    app.update();
    assert!(
        app.world()
            .resource::<SharedInputDialog>()
            .owner()
            .is_none()
    );
    assert!(app.world().resource::<SharedRedeemCode>().source.is_none());
    assert!(app.world().resource::<SystemMessageUiModel>().is_empty());
}

#[test]
fn inventory_redeem_cancel_and_owner_loss_release_the_modal() {
    for owner_lost in [false, true] {
        let mut app = fixture();
        {
            let mut state = app.world_mut().resource_mut::<UserEquipUiState>();
            state.open_item_mode();
            state.tick(1.0);
        }
        app.world_mut()
            .resource_mut::<UserEquipItemModeProjection>()
            .owner_pc_id = 42;
        app.world_mut()
            .resource_mut::<UserEquipModalState>()
            .redeem_code_view = true;
        app.world_mut().resource_mut::<SharedRedeemCode>().source =
            Some(RedeemSource::Inventory { pc: 42 });
        if owner_lost {
            app.world_mut().resource_mut::<RuntimeStatus>().player_id = Some(43);
        } else {
            app.world_mut().resource_mut::<SharedInputDialog>().cancel();
        }
        app.update();
        assert!(app.world().resource::<SharedRedeemCode>().source.is_none());
        assert!(
            !app.world()
                .resource::<UserEquipModalState>()
                .redeem_code_view
        );
        assert!(
            app.world()
                .resource::<SharedInputDialog>()
                .owner()
                .is_none()
        );
    }
}
