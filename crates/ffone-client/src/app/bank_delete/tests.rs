use super::*;
fn fixture() -> (App, BankItemDeleteIntent0104) {
    let intent = BankItemDeleteIntent0104 {
        pc: 42,
        npc: 815,
        slot: 0,
        item: ItemBase0104 {
            item_type: 7,
            item_id: 7,
            option: 99,
            time_limit: 0,
        },
    };
    let mut projection = BankModeProjection0104::default();
    projection.owner_pc_id = 42;
    projection.npc_id = 815;
    projection.item_mode.inventory[0].item.item = intent.item;
    let mut deletion = BankItemDeleteState::default();
    deletion.mark_sent(intent);
    let mut production = UserEquipProductionRuntime0104::default();
    production
        .begin(UserEquipPendingRequest0104::Delete(request_for(intent)))
        .unwrap();
    let mut status = RuntimeStatus::default();
    status.player_id = Some(42);
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    let offset = ffone_protocol::PcLoadData0104::INVENTORY_OFFSET;
    load.as_bytes_mut()[offset..offset + 2].copy_from_slice(&7i16.to_le_bytes());
    load.as_bytes_mut()[offset + 2..offset + 4].copy_from_slice(&7i16.to_le_bytes());
    load.as_bytes_mut()[offset + 4..offset + 8].copy_from_slice(&99i32.to_le_bytes());
    let mut inventory = LocalInventoryRuntime::default();
    inventory.seed(42, &load);
    let mut app = App::new();
    app.insert_resource(State::new(ClientState::World))
        .insert_resource(status)
        .insert_resource(NetworkBridge::start())
        .insert_resource(inventory)
        .insert_resource(projection)
        .insert_resource(deletion)
        .insert_resource(production)
        .insert_resource(BankUiState {
            phase: BankLifecyclePhase::Visible,
            send_pending: true,
            ..default()
        })
        .init_resource::<BankProductionRuntime0104>()
        .add_systems(Update, consume);
    (app, intent)
}
#[test]
fn bank_delete_timeout_unlocks_without_mutation_and_retains_late_reply_ownership() {
    let (mut app, intent) = fixture();
    app.update();
    assert!(app.world().resource::<BankUiState>().send_pending);
    app.world_mut()
        .resource_mut::<UserEquipProductionRuntime0104>()
        .tick(10.1);
    app.update();
    assert!(!app.world().resource::<BankUiState>().send_pending);
    assert!(
        app.world()
            .resource::<BankItemDeleteState>()
            .sent()
            .is_none()
    );
    assert!(
        app.world()
            .resource::<UserEquipProductionRuntime0104>()
            .owns_delete_reply_family()
    );
    assert_eq!(
        app.world()
            .resource::<LocalInventoryRuntime>()
            .snapshot()
            .unwrap()
            .inventory()[0],
        intent.item
    );
}
#[test]
fn bank_delete_correlated_reply_updates_authority_and_unlocks_bank() {
    let (mut app, _) = fixture();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let content = TutorialMissionContent::open(&AssetLocator::open(&root).unwrap()).unwrap();
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_ITEM_DELETE_SUCC,
        flags: 0,
        checksum: 0,
        payload: PcItemDeleteSuccess0104 {
            item_location: 1,
            slot_num: 0,
        }
        .encode(),
    };
    let mut inventory = app
        .world_mut()
        .remove_resource::<LocalInventoryRuntime>()
        .unwrap();
    let mut production = app
        .world_mut()
        .remove_resource::<UserEquipProductionRuntime0104>()
        .unwrap();
    let mut status = app.world_mut().remove_resource::<RuntimeStatus>().unwrap();
    assert!(
        apply_user_equip_frame(
            &frame,
            &mut inventory,
            &content,
            &mut production,
            &BankProductionRuntime0104::default(),
            &VendorProductionRuntime0104::default(),
            &mut status
        )
        .unwrap()
    );
    assert!(InventoryRuntime0104::item_is_empty(
        inventory.snapshot().unwrap().inventory()[0]
    ));
    app.insert_resource(inventory)
        .insert_resource(production)
        .insert_resource(status);
    app.update();
    assert!(!app.world().resource::<BankUiState>().send_pending);
    assert!(
        app.world()
            .resource::<BankItemDeleteState>()
            .sent()
            .is_none()
    );
}
