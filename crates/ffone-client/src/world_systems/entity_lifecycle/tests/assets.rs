use super::*;

#[test]
fn coco_predicted_pickup_exit_and_server_respawn_keep_registry_consistent() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    let egg = ShinyAppearance0104 {
        shiny_id: 81,
        shiny_type: 17,
        map_number: 0,
        position: [100, 200, 300],
    };
    upsert_shiny(app.world_mut(), EPOCH_ONE, egg);
    let first = app
        .world()
        .resource::<NetworkShinyRegistry0104>()
        .get(81)
        .unwrap();
    let visual = app.world_mut().spawn(ChildOf(first)).id();
    despawn_shiny(app.world_mut(), 81);
    assert!(
        app.world()
            .resource::<NetworkShinyRegistry0104>()
            .is_empty()
    );
    assert!(app.world().get_entity(visual).is_err());
    despawn_shiny(app.world_mut(), 81); // delayed authoritative EXIT
    assert_eq!(
        app.world()
            .resource::<NetworkEntityLifecycleStats0104>()
            .shiny_despawns,
        1
    );
    upsert_shiny(app.world_mut(), EPOCH_ONE, egg); // authoritative respawn
    let next = app
        .world()
        .resource::<NetworkShinyRegistry0104>()
        .get(81)
        .unwrap();
    assert_ne!(first, next);
    assert_eq!(
        app.world()
            .get::<NetworkShinyAppearance0104>(next)
            .unwrap()
            .0,
        egg
    );
}

#[test]
fn every_exit_and_around_del_path_removes_registered_players_and_npcs() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    app.world_mut()
        .resource_mut::<NetworkEntityLifecycleIngress0104>()
        .push_bootstrap(
            EPOCH_ONE,
            InitialAroundPacket0104::Players(vec![
                pc_appearance(11, [0, 0, 0]),
                pc_appearance(12, [0, 0, 0]),
            ]),
        );
    app.world_mut()
        .resource_mut::<NetworkEntityLifecycleIngress0104>()
        .push_bootstrap(
            EPOCH_ONE,
            InitialAroundPacket0104::Npcs(vec![
                npc_appearance(21, 4001, [0, 0, 0]),
                npc_appearance(22, 4002, [0, 0, 0]),
            ]),
        );
    app.update();
    let old_entities = app
        .world()
        .resource::<RemotePcRegistry0104>()
        .iter()
        .chain(app.world().resource::<NetworkNpcRegistry0104>().iter())
        .map(|(_, entity)| entity)
        .collect::<Vec<_>>();

    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_PC_EXIT,
            PcExit0104 {
                pc_id: 11,
                exit_type: 7,
            }
            .encode(),
        ),
    );
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_AROUND_DEL_PC,
            AroundDelPc0104 { pc_ids: vec![12] }.encode().unwrap(),
        ),
    );
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_NPC_EXIT, NpcExit0104 { npc_id: 21 }.encode()),
    );
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_AROUND_DEL_NPC,
            AroundDelNpc0104 { npc_ids: vec![22] }.encode().unwrap(),
        ),
    );
    app.update();

    assert!(app.world().resource::<RemotePcRegistry0104>().is_empty());
    assert!(app.world().resource::<NetworkNpcRegistry0104>().is_empty());
    assert!(
        old_entities
            .into_iter()
            .all(|entity| app.world().get_entity(entity).is_err())
    );
    let stats = app.world().resource::<NetworkEntityLifecycleStats0104>();
    assert_eq!(stats.player_despawns, 2);
    assert_eq!(stats.npc_despawns, 2);
}

#[test]
fn pvp_attack_families_route_results_by_entity_type() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_PC_NEW,
            PcNew0104 {
                appearance: pc_appearance(11, [100, 0, 0]),
            }
            .encode(),
        ),
    );
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_NPC_ENTER,
            NpcEnter0104 {
                appearance: npc_appearance(21, 4001, [0, 0, 0]),
            }
            .encode(),
        ),
    );
    app.update();
    let npc = app
        .world()
        .resource::<NetworkNpcRegistry0104>()
        .get(21)
        .unwrap();
    let remote = app
        .world()
        .resource::<RemotePcRegistry0104>()
        .get(11)
        .unwrap();
    let npc_hp = |app: &App| {
        app.world()
            .get::<NetworkNpcAppearance0104>(npc)
            .unwrap()
            .0
            .hp
    };
    let remote_hp = |app: &App| {
        app.world()
            .get::<NetworkPcAppearance0104>(remote)
            .unwrap()
            .0
            .hp
    };
    let hit = |entity_type: i32, id: i32, hp: i32| AttackResult0104 {
        entity_type,
        id,
        protected: 0,
        damage: 10,
        hp,
        hit_flag: 1,
    };

    // The caster's PvP reply mixes an NPC and a player result; the clean
    // client routes them by eCT even though both IDs could collide.
    let success = PcAttackCharsSuccess0104 {
        battery_w: 500,
        results: vec![hit(4, 21, 400), hit(1, 11, 800)],
    };
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_PC_ATTACK_CHARS_SUCC, success.encode().unwrap()),
    );
    app.update();
    assert_eq!(npc_hp(&app), 400);
    assert_eq!(remote_hp(&app), 800);

    // Another player's broadcast hitting the same NPC.
    let broadcast = PcAttackChars0104 {
        pc_id: 11,
        results: vec![hit(4, 21, 300)],
    };
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_PC_ATTACK_CHARS, broadcast.encode().unwrap()),
    );
    app.update();
    assert_eq!(npc_hp(&app), 300);
    assert_eq!(remote_hp(&app), 800);

    // An NPC attacking a mixed set: the player result takes the ordinary
    // NPC_ATTACK_PCS presentation path, the unknown NPC result is ignored.
    // The damage above already advanced the NPC's animation revision, so
    // only the edge and the clip are asserted.
    let animation_revision = |app: &App| {
        app.world()
            .get::<NetworkNpcCombatAnimation0104>(npc)
            .map_or(0, |animation| animation.revision)
    };
    let before_npc_attack = animation_revision(&app);
    let npc_attack = NpcAttackChars0104 {
        npc_id: 21,
        results: vec![hit(1, 11, 700), hit(4, 99, 1)],
    };
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_NPC_ATTACK_CHARS, npc_attack.encode().unwrap()),
    );
    app.update();
    assert_eq!(remote_hp(&app), 700);
    assert_eq!(npc_hp(&app), 300);
    let after_npc_attack = app
        .world()
        .get::<NetworkNpcCombatAnimation0104>(npc)
        .unwrap();
    assert!(after_npc_attack.revision > before_npc_attack);
    assert_eq!(after_npc_attack.clip, NetworkNpcCombatClip0104::Melee);

    // An NPC-owned attack whose targets are NPCs only.
    let before_character_attack = animation_revision(&app);
    let character_attack = CharacterAttackCharacters0104 {
        entity_type: 4,
        character_id: 21,
        results: vec![hit(4, 21, 250)],
    };
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_CHARACTER_ATTACK_CHARACTERS,
            character_attack.encode().unwrap(),
        ),
    );
    app.update();
    assert_eq!(npc_hp(&app), 250);
    assert_eq!(remote_hp(&app), 700);
    assert!(animation_revision(&app) > before_character_attack);

    // A truncated PvP reply is rejected atomically: nothing changes.
    let mut truncated = success.encode().unwrap();
    truncated.truncate(30);
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_PC_ATTACK_CHARS_SUCC, truncated),
    );
    app.update();
    assert_eq!(npc_hp(&app), 250);
    assert_eq!(remote_hp(&app), 700);
}

#[test]
fn counted_pc_delete_rejects_negative_count_without_touching_registry() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_AROUND_DEL_PC, (-1i32).to_le_bytes().to_vec()),
    );
    app.update();

    assert!(matches!(
        app.world()
            .resource::<MalformedLifecycleFrames0104>()
            .frames[0]
            .error,
        EntityLifecycleDecodeError0104::Counted(CountedPayloadError0104::NegativeCount {
            count: -1
        })
    ));
    assert!(app.world().resource::<RemotePcRegistry0104>().is_empty());
}

#[test]
fn remote_attack_broadcast_starts_animation_only_after_strict_decode() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    push_frame(&mut app, EPOCH_ONE, frame(ffone_protocol::packet::P_FE2CL_PC_NEW, pc_appearance(17, [0,0,0]).encode()));
    app.update();
    let entity = app.world().resource::<RemotePcRegistry0104>().get(17).unwrap();
    let payload = ffone_protocol::PcAttackNpcs0104 { pc_id: 17, results: vec![] }.encode().unwrap();
    push_frame(&mut app, EPOCH_ONE, frame(ffone_protocol::packet::P_FE2CL_PC_ATTACK_NPCS, payload.clone()));
    app.update();
    assert_eq!(app.world().get::<crate::remote::RemoteAnimation>(entity).unwrap().state, crate::remote::RemoteAnimationState::Attacking);
    app.world_mut().entity_mut(entity).insert(crate::remote::RemoteAnimation::default());
    push_frame(&mut app, EPOCH_ONE, frame(ffone_protocol::packet::P_FE2CL_PC_ATTACK_NPCS, payload[..payload.len()-1].to_vec()));
    app.update();
    assert_eq!(app.world().get::<crate::remote::RemoteAnimation>(entity).unwrap().state, crate::remote::RemoteAnimationState::Idle);
}

#[test]
fn retiring_pc_releases_id_and_authority_before_fade_and_session_clears_it() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    let pc = pc_appearance(17, [0, 0, 0]);
    super::super::operations::upsert_player(app.world_mut(), EPOCH_ONE, Some(LOCAL_PC_ID), pc.clone());
    let old = app.world().resource::<RemotePcRegistry0104>().get(17).unwrap();
    let rig = app.world_mut().spawn(ChildOf(old)).id();
    app.world_mut().get_mut::<RemotePcVisibility0104>(old).unwrap().alpha = 0.4;
    super::super::entities::despawn_player(app.world_mut(), Some(LOCAL_PC_ID), 17);
    assert!(app.world().resource::<RemotePcRegistry0104>().get(17).is_none());
    assert!(app.world().get::<NetworkPcAppearance0104>(old).is_none());
    assert!(app.world().get::<crate::remote::RemoteMotion>(old).is_none());
    assert!(app.world().get::<RemotePcVisibility0104>(old).unwrap().retiring);
    super::super::operations::upsert_player(app.world_mut(), EPOCH_ONE, Some(LOCAL_PC_ID), pc);
    assert_ne!(app.world().resource::<RemotePcRegistry0104>().get(17).unwrap(), old);
    super::super::projects::disconnect_session(app.world_mut(), EPOCH_ONE);
    assert!(app.world().get_entity(old).is_err());
    assert!(app.world().get_entity(rig).is_err());
}
