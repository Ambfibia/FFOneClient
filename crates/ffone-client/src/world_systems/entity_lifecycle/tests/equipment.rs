use super::*;
use ffone_protocol::{EquipChangePacket0104, ItemBase0104};

const EMPTY_ITEM: ItemBase0104 = ItemBase0104 {
    item_type: 0,
    item_id: 0,
    option: 0,
    time_limit: 0,
};

#[test]
fn remote_equipment_changes_refresh_the_existing_players_visual_request_for_every_slot() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    let original = pc_appearance(11, [700, 800, 900]);
    upsert_player(
        app.world_mut(),
        EPOCH_ONE,
        Some(LOCAL_PC_ID),
        original.clone(),
    );
    let entity = app
        .world()
        .resource::<RemotePcRegistry0104>()
        .get(11)
        .unwrap();
    for slot in 0..9 {
        for equip_slot_item in [
            ItemBase0104 {
                item_type: slot as i16,
                item_id: 100 + slot as i16,
                option: 1,
                time_limit: 123,
            },
            EMPTY_ITEM,
        ] {
            let packet = EquipChangePacket0104 {
                pc_id: 11,
                equip_slot_num: slot,
                equip_slot_item,
            };
            push_frame(
                &mut app,
                EPOCH_ONE,
                frame(packet::P_FE2CL_PC_EQUIP_CHANGE, packet.encode()),
            );
            app.update();
            assert_eq!(
                app.world().resource::<RemotePcRegistry0104>().get(11),
                Some(entity)
            );
            let appearance = &app
                .world()
                .get::<NetworkPcAppearance0104>(entity)
                .unwrap()
                .0;
            assert_eq!(appearance.equipment[slot as usize], equip_slot_item);
            assert_eq!(appearance.position, original.position);
            assert_eq!(appearance.style, original.style);
            let pending = app.world().get::<PendingPcVisual0104>(entity).unwrap();
            assert_eq!(pending, &PendingPcVisual0104::from(appearance));
            // Repeating a server broadcast must not rebuild an unchanged rig.
            let tick = app
                .world()
                .entity(entity)
                .get_ref::<PendingPcVisual0104>()
                .unwrap()
                .last_changed();
            push_frame(
                &mut app,
                EPOCH_ONE,
                frame(packet::P_FE2CL_PC_EQUIP_CHANGE, packet.encode()),
            );
            app.update();
            assert_eq!(
                app.world()
                    .entity(entity)
                    .get_ref::<PendingPcVisual0104>()
                    .unwrap()
                    .last_changed(),
                tick
            );
        }
    }
    assert!(
        app.world()
            .resource::<PassthroughLifecycleFrames0104>()
            .frames
            .is_empty()
    );
}

#[test]
fn remote_equipment_rejects_bad_slots_payloads_stale_epochs_and_unowned_players() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    let original = pc_appearance(11, [700, 800, 900]);
    upsert_player(
        app.world_mut(),
        EPOCH_ONE,
        Some(LOCAL_PC_ID),
        original.clone(),
    );
    let entity = app
        .world()
        .resource::<RemotePcRegistry0104>()
        .get(11)
        .unwrap();
    for slot in [-1, 9] {
        let packet = EquipChangePacket0104 {
            pc_id: 11,
            equip_slot_num: slot,
            equip_slot_item: EMPTY_ITEM,
        };
        push_frame(
            &mut app,
            EPOCH_ONE,
            frame(packet::P_FE2CL_PC_EQUIP_CHANGE, packet.encode()),
        );
    }
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(packet::P_FE2CL_PC_EQUIP_CHANGE, vec![0; 19]),
    );
    for (epoch, pc_id) in [(EPOCH_TWO, 11), (EPOCH_ONE, LOCAL_PC_ID), (EPOCH_ONE, 404)] {
        let packet = EquipChangePacket0104 {
            pc_id,
            equip_slot_num: 1,
            equip_slot_item: EMPTY_ITEM,
        };
        push_frame(
            &mut app,
            epoch,
            frame(packet::P_FE2CL_PC_EQUIP_CHANGE, packet.encode()),
        );
    }
    app.update();
    assert_eq!(
        app.world()
            .get::<NetworkPcAppearance0104>(entity)
            .unwrap()
            .0,
        original
    );
    assert_eq!(
        app.world()
            .resource::<MalformedLifecycleFrames0104>()
            .frames
            .len(),
        3
    );
    assert_eq!(
        app.world()
            .resource::<IgnoredLifecycleFrames0104>()
            .frames
            .len(),
        3
    );
}
