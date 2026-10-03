use super::*;

#[test]
fn barber_style_refresh_preserves_remote_entity_position_and_equipment() {
    use ffone_protocol::{WirePayload, wire_0104};
    let mut app=test_app();begin(&mut app,EPOCH_ONE);
    let original=pc_appearance(11,[700,800,900]);
    upsert_player(app.world_mut(),EPOCH_ONE,Some(LOCAL_PC_ID),original.clone());
    let entity=app.world().resource::<RemotePcRegistry0104>().get(11).unwrap();
    let mut style=wire_0104::PcStyle0104::decode(&[0;76]).unwrap();
    style.pc_uid=original.style.pc_uid;style.gender=2;style.hair_style=25;
    let packet=wire_0104::PcStyleChange0104{pc_id:11,s_pc_style:style};
    push_frame(&mut app,EPOCH_ONE,frame(0x3100013a,packet.encode()));app.update();
    assert_eq!(app.world().resource::<RemotePcRegistry0104>().get(11),Some(entity));
    let updated=&app.world().get::<NetworkPcAppearance0104>(entity).unwrap().0;
    assert_eq!(updated.position,original.position);assert_eq!(updated.equipment,original.equipment);
    assert_eq!(updated.style.hair_style,25);
    let pending=app.world().get::<PendingPcVisual0104>(entity).unwrap();
    assert_eq!(pending.style.gender,2);assert_eq!(pending.equipment,original.equipment);
    let mut wrong_owner=packet;wrong_owner.s_pc_style.pc_uid+=1;wrong_owner.s_pc_style.hair_style=26;
    push_frame(&mut app,EPOCH_ONE,frame(0x3100013a,wrong_owner.encode()));app.update();
    assert_eq!(app.world().get::<NetworkPcAppearance0104>(entity).unwrap().0.style.hair_style,25);
}

#[test]
fn appearance_then_movement_in_one_tick_updates_the_same_player_entity() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    let appearance = pc_appearance(11, [100, 200, 300]);
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_PC_NEW, PcNew0104 { appearance }.encode()),
    );
    let movement = pc_move(11, [700, 800, 900]);
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_PC_MOVE, movement.encode()),
    );
    app.update();

    let entity = app
        .world()
        .resource::<RemotePcRegistry0104>()
        .get(11)
        .unwrap();
    let motion = app.world().get::<RemoteMotion>(entity).unwrap();
    assert_eq!(motion.target_position, Vec3::new(-7.0, 9.0, 8.0));
    assert_eq!(motion.last_server_time, 200);
    assert!(matches!(
        app.world().get::<RemoteAnimation>(entity).unwrap().state,
        crate::remote::RemoteAnimationState::Moving { direction_key: 1 }
    ));
    assert!(
        app.world()
            .resource::<IgnoredLifecycleFrames0104>()
            .frames
            .is_empty()
    );
}

#[test]
fn player_attack_results_update_live_npc_hp_for_targeting() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    let appearance = npc_appearance(21, 4001, [0, 0, 0]);
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_NPC_ENTER, NpcEnter0104 { appearance }.encode()),
    );
    app.update();
    let entity = app
        .world()
        .resource::<NetworkNpcRegistry0104>()
        .get(21)
        .unwrap();
    app.world_mut()
        .entity_mut(entity)
        .insert(NetworkNpcMotion0104 {
            destination: Vec3::X,
            speed: 1.0,
            move_style: 1,
        });

    let attack = PcAttackNpcsSuccess0104 {
        battery_w: 90,
        results: vec![AttackResult0104 {
            // OpenFusion's zero-initialized sAttackResult is the real
            // server shape which previously left the target HUD stale.
            entity_type: 0,
            id: 21,
            protected: 0,
            damage: 250,
            hp: 250,
            hit_flag: 1,
        }],
    };
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_PC_ATTACK_NPCS_SUCC, attack.encode().unwrap()),
    );
    app.update();

    assert_eq!(
        app.world()
            .get::<NetworkNpcAppearance0104>(entity)
            .unwrap()
            .0
            .hp,
        250
    );
    assert_eq!(
        app.world()
            .get::<NetworkNpcCombatAnimation0104>(entity)
            .unwrap()
            .clip,
        NetworkNpcCombatClip0104::Wound
    );
    assert!(app.world().get::<NetworkNpcMotion0104>(entity).is_some());

    let killing_attack = PcAttackNpcsSuccess0104 {
        battery_w: 80,
        results: vec![AttackResult0104 {
            entity_type: 0,
            id: 21,
            protected: 0,
            damage: 250,
            hp: 0,
            hit_flag: 1,
        }],
    };
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_PC_ATTACK_NPCS_SUCC,
            killing_attack.encode().unwrap(),
        ),
    );
    app.update();

    assert_eq!(
        app.world()
            .get::<NetworkNpcAppearance0104>(entity)
            .unwrap()
            .0
            .hp,
        0
    );
    assert!(app.world().get::<NetworkNpcMotion0104>(entity).is_none());
    assert!(
        app.world()
            .get::<NetworkNpcCombatAnimation0104>(entity)
            .is_none()
    );
    assert_eq!(
        app.world()
            .resource::<NetworkEntityLifecycleStats0104>()
            .npc_damage_updates,
        2
    );
}
