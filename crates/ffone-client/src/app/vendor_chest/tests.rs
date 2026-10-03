use super::*;
#[test]
fn vendor_chest_pending_identity_uses_wire_time_limit() {
    let intent = VendorChestOpenIntent {
        pc: 42,
        session: default(),
        slot: 3,
        item: ItemBase0104 {
            item_type: 9,
            item_id: 77,
            option: 1,
            time_limit: 123,
        },
    };
    assert_eq!(request_for(intent).chest_item.time_limit, 0);
    assert_eq!(intent.item.time_limit, 123);
}
fn fixture() -> (App, VendorChestOpenIntent) {
    let item = ItemBase0104 {
        item_type: 9,
        item_id: 77,
        option: 1,
        time_limit: 0,
    };
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    let offset = ffone_protocol::PcLoadData0104::INVENTORY_OFFSET;
    load.as_bytes_mut()[offset..offset + 2].copy_from_slice(&9i16.to_le_bytes());
    load.as_bytes_mut()[offset + 2..offset + 4].copy_from_slice(&77i16.to_le_bytes());
    load.as_bytes_mut()[offset + 4..offset + 8].copy_from_slice(&1i32.to_le_bytes());
    let mut inventory = LocalInventoryRuntime::default();
    inventory.seed(42, &load);
    let mut vendor = VendorProductionRuntime0104::default();
    vendor
        .begin_start(ActiveVendorSourceNpc0104 {
            runtime_npc_id: 815,
            table_npc_id: 27,
            ai_type: 1,
        })
        .unwrap();
    vendor
        .apply_packet(
            ffone_protocol::VendorPacket0104::StartSuccess(
                ffone_protocol::VendorStartSuccess0104 {
                    npc_id: 27,
                    vendor_id: 27,
                },
            ),
            inventory.snapshot_mut().unwrap(),
        )
        .unwrap();
    vendor
        .apply_packet(
            ffone_protocol::VendorPacket0104::TableSuccess(
                ffone_protocol::VendorTableUpdateSuccess0104 {
                    items: [ffone_protocol::ItemVendor0104 {
                        vendor_id: 27,
                        buy_cost: 0.0,
                        item: ItemBase0104 {
                            item_type: 0,
                            item_id: 0,
                            option: 0,
                            time_limit: 0,
                        },
                        sort_num: 0,
                    };
                        ffone_protocol::VENDOR_TABLE_ITEM_COUNT_0104],
                },
            ),
            inventory.snapshot_mut().unwrap(),
        )
        .unwrap();
    let intent = VendorChestOpenIntent {
        pc: 42,
        session: vendor.session().unwrap(),
        slot: 0,
        item,
    };
    let mut projection = VendorModeProjection0104::default();
    projection.owner_pc_id = 42;
    projection.session = intent.session;
    projection.inventory[0].item.empty = false;
    projection.inventory[0].item.item = item;
    let mut chest = VendorChestOpenState::default();
    chest.mark_sent(intent);
    let mut production = UserEquipProductionRuntime0104::default();
    production
        .begin(UserEquipPendingRequest0104::ChestOpen(request_for(intent)))
        .unwrap();
    let mut status = RuntimeStatus::default();
    status.player_id = Some(42);
    let mut app = App::new();
    app.insert_resource(State::new(ClientState::World))
        .insert_resource(status)
        .insert_resource(NetworkBridge::start())
        .insert_resource(inventory)
        .insert_resource(projection)
        .insert_resource(vendor)
        .insert_resource(chest)
        .insert_resource(production)
        .insert_resource(VendorUiState {
            phase: VendorLifecyclePhase::Visible,
            send_pending: true,
            ..default()
        })
        .add_systems(Update, consume);
    (app, intent)
}
#[test]
fn vendor_chest_close_retires_pending_without_mutating_inventory() {
    let (mut app, intent) = fixture();
    app.update();
    assert!(
        app.world()
            .resource::<VendorChestOpenState>()
            .sent()
            .is_some()
    );
    app.world_mut().resource_mut::<VendorUiState>().phase = VendorLifecyclePhase::Hidden;
    app.update();
    assert!(
        app.world()
            .resource::<VendorChestOpenState>()
            .sent()
            .is_none()
    );
    let production = app.world().resource::<UserEquipProductionRuntime0104>();
    assert!(!production.send_pending());
    assert!(production.owns_chest_open_reply_family());
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
fn vendor_chest_ack_releases_shared_owner_without_speculative_reward() {
    let (mut app, intent) = fixture();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let content = TutorialMissionContent::open(&AssetLocator::open(&root).unwrap()).unwrap();
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_ITEM_CHEST_OPEN_SUCC,
        flags: 0,
        checksum: 0,
        payload: ItemChestOpenSuccess0104 { slot_num: 0 }.encode(),
    };
    let mut inventory = app
        .world_mut()
        .remove_resource::<LocalInventoryRuntime>()
        .unwrap();
    let mut production = app
        .world_mut()
        .remove_resource::<UserEquipProductionRuntime0104>()
        .unwrap();
    assert_eq!(
        apply_user_equip_frame(
            &frame,
            &mut inventory,
            &content,
            &mut production,
            &BankProductionRuntime0104::default(),
            app.world().resource::<VendorProductionRuntime0104>(),
            &mut RuntimeStatus::default()
        ),
        Ok(true)
    );
    assert_eq!(inventory.snapshot().unwrap().inventory()[0], intent.item);
    app.insert_resource(inventory).insert_resource(production);
    app.update();
    assert!(!app.world().resource::<VendorUiState>().send_pending);
    assert!(
        app.world()
            .resource::<VendorChestOpenState>()
            .sent()
            .is_none()
    );
}
