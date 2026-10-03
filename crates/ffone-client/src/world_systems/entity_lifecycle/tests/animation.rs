use super::*;

#[test]
fn npc_combat_packets_publish_exact_animation_phases() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    let appearance = npc_appearance(21, 4001, [0, 0, 0]);
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
        frame(P_FE2CL_NPC_ENTER, NpcEnter0104 { appearance }.encode()),
    );
    app.update();
    let entity = app
        .world()
        .resource::<NetworkNpcRegistry0104>()
        .get(21)
        .unwrap();

    let attack = NpcAttackPcs0104 {
        npc_id: 21,
        results: vec![AttackResult0104 {
            // OpenFusion leaves eCT zero in this packet family.
            entity_type: 0,
            id: 11,
            protected: 0,
            damage: 50,
            hp: 950,
            hit_flag: 1,
        }],
    };
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_NPC_ATTACK_PCS, attack.encode().unwrap()),
    );
    app.update();
    assert_eq!(
        *app.world()
            .get::<NetworkNpcCombatAnimation0104>(entity)
            .unwrap(),
        NetworkNpcCombatAnimation0104 {
            revision: 1,
            clip: NetworkNpcCombatClip0104::Melee,
        }
    );
    let remote = app
        .world()
        .resource::<RemotePcRegistry0104>()
        .get(11)
        .unwrap();
    let npc_transform = app.world().get::<Transform>(entity).unwrap();
    let remote_transform = app.world().get::<Transform>(remote).unwrap();
    let expected_facing = (remote_transform.translation - npc_transform.translation)
        .with_y(0.0)
        .normalize();
    assert!(
        npc_transform.forward().as_vec3().dot(expected_facing) > 0.999,
        "a clean NPC attack turns the attacker toward its first target"
    );
    assert_eq!(
        app.world()
            .get::<NetworkPcAppearance0104>(remote)
            .unwrap()
            .0
            .hp,
        950
    );
    assert!(!NetworkNpcCombatClip0104::Melee.repeats());
    assert!(NetworkNpcCombatClip0104::MegaReady.repeats());
    assert_eq!(
        NetworkNpcCombatClip0104::CorruptionReady.fallback_name(),
        Some("ready")
    );
    assert_eq!(
        app.world_mut()
            .resource_mut::<NetworkNpcAttackEventQueue0104>()
            .take_all(),
        VecDeque::from([attack])
    );

    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_NPC_SKILL_CORRUPTION_READY,
            npc_skill_payload(20, 21, 7),
        ),
    );
    let mut corruption = skill_target_record(40, 1, 11);
    write_i32(&mut corruption, 8, 0);
    write_i32(&mut corruption, 12, 50);
    write_i32(&mut corruption, 16, 900);
    corruption[20] = 8;
    write_i16(&mut corruption, 22, 0);
    write_i32(&mut corruption, 24, 0);
    write_i16(&mut corruption, 28, 5);
    write_i16(&mut corruption, 30, 100);
    write_i32(&mut corruption, 32, 0x40);
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_NPC_SKILL_CORRUPTION_HIT,
            npc_corruption_hit_payload(21, 7, 1, &[corruption]),
        ),
    );
    app.update();
    assert_eq!(
        app.world()
            .get::<NetworkNpcCombatAnimation0104>(entity)
            .unwrap()
            .clip,
        NetworkNpcCombatClip0104::Corruption
    );
    assert!(
        app.world()
            .get::<NetworkNpcSkillPhase0104>(entity)
            .is_none()
    );
    let remote_appearance = &app
        .world()
        .get::<NetworkPcAppearance0104>(remote)
        .unwrap()
        .0;
    assert_eq!(remote_appearance.hp, 900);
    assert_eq!(remote_appearance.condition_bit_flag, 0x40);
    assert_eq!(remote_appearance.nano.stamina, 100);

    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_NPC_SKILL_READY, npc_skill_payload(20, 21, 8)),
    );
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_NPC_SKILL_HIT, npc_skill_hit_payload(21, 8, 29, &[])),
    );
    app.update();
    assert_eq!(
        app.world()
            .get::<NetworkNpcCombatAnimation0104>(entity)
            .unwrap()
            .clip,
        NetworkNpcCombatClip0104::Mega
    );

    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_NPC_SKILL_HIT, npc_skill_hit_payload(21, 9, 29, &[])),
    );
    app.update();
    let first_skill = *app
        .world()
        .get::<NetworkNpcCombatAnimation0104>(entity)
        .unwrap();
    assert_eq!(first_skill.clip, NetworkNpcCombatClip0104::Skill);
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_NPC_SKILL_HIT, npc_skill_hit_payload(21, 9, 29, &[])),
    );
    app.update();
    assert!(
        app.world()
            .get::<NetworkNpcCombatAnimation0104>(entity)
            .unwrap()
            .revision
            > first_skill.revision,
        "repeated skill packets must restart the same clip and its AnimationEvents"
    );

    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_NPC_SKILL_CANCEL, 21i32.to_le_bytes().to_vec()),
    );
    app.update();
    assert!(
        app.world()
            .get::<NetworkNpcCombatAnimation0104>(entity)
            .is_none()
    );
}

#[test]
fn npc_animation_requests_survive_completion_and_keep_independent_lanes() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    upsert_npc(app.world_mut(), EPOCH_ONE, npc_appearance(21, 461, [0; 3]));
    let npc = app
        .world()
        .resource::<NetworkNpcRegistry0104>()
        .get(21)
        .unwrap();
    request_network_npc_combat_animation(
        app.world_mut(),
        21,
        NetworkNpcCombatClip0104::MegaReady,
    );
    app.world_mut()
        .entity_mut(npc)
        .insert(NetworkNpcSkillPhase0104::MegaReady);
    request_network_npc_combat_animation(app.world_mut(), 21, NetworkNpcCombatClip0104::Melee);
    let first = *app
        .world()
        .get::<NetworkNpcCombatAnimation0104>(npc)
        .unwrap();
    apply_network_npc_hp_animation(app.world_mut(), npc, 21, 100, 90);
    let layers = app
        .world()
        .get::<NetworkNpcAnimationLayers0104>(npc)
        .unwrap();
    assert_eq!(
        layers.low.unwrap().clip,
        NetworkNpcCombatClip0104::MegaReady
    );
    assert_eq!(layers.high[0], Some(first));
    assert_eq!(
        layers.high[1].unwrap().clip,
        NetworkNpcCombatClip0104::Wound
    );
    assert!(app.world().get::<NetworkNpcSkillPhase0104>(npc).is_some());
    app.world_mut()
        .entity_mut(npc)
        .remove::<NetworkNpcCombatAnimation0104>();
    request_network_npc_combat_animation(app.world_mut(), 21, NetworkNpcCombatClip0104::Melee);
    assert!(
        app.world()
            .get::<NetworkNpcCombatAnimation0104>(npc)
            .unwrap()
            .revision
            > first.revision
    );
    app.world_mut()
        .get_mut::<NetworkNpcAppearance0104>(npc)
        .unwrap()
        .0
        .hp = 0;
    apply_network_npc_hp_animation(app.world_mut(), npc, 21, 90, 0);
    request_network_npc_combat_animation(app.world_mut(), 21, NetworkNpcCombatClip0104::Melee);
    assert!(
        app.world()
            .get::<NetworkNpcCombatAnimation0104>(npc)
            .is_none(),
        "dead NPC ignores later attack packets"
    );
    let layers = app
        .world()
        .get::<NetworkNpcAnimationLayers0104>(npc)
        .unwrap();
    assert!(!layers.stand_attack);
    assert!(layers.low.is_none());
    assert!(
        layers.high_owner.is_none(),
        "death wound cannot return to ready"
    );
}

#[test]
fn malformed_skill_tails_are_atomic_for_authority_and_animation() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    for pc_id in [11, 12] {
        push_frame(
            &mut app,
            EPOCH_ONE,
            frame(
                P_FE2CL_PC_NEW,
                PcNew0104 {
                    appearance: pc_appearance(pc_id, [0, 0, 0]),
                }
                .encode(),
            ),
        );
    }
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
    let pc_11 = app
        .world()
        .resource::<RemotePcRegistry0104>()
        .get(11)
        .unwrap();
    let pc_12 = app
        .world()
        .resource::<RemotePcRegistry0104>()
        .get(12)
        .unwrap();
    let npc = app
        .world()
        .resource::<NetworkNpcRegistry0104>()
        .get(21)
        .unwrap();
    let pc_11_before = app
        .world()
        .get::<NetworkPcAppearance0104>(pc_11)
        .unwrap()
        .clone();
    let pc_12_before = app
        .world()
        .get::<NetworkPcAppearance0104>(pc_12)
        .unwrap()
        .clone();
    let npc_before = *app.world().get::<NetworkNpcAppearance0104>(npc).unwrap();

    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_NPC_SKILL_CORRUPTION_READY,
            npc_skill_payload(20, 21, 7),
        ),
    );
    app.update();
    let ready_animation = *app
        .world()
        .get::<NetworkNpcCombatAnimation0104>(npc)
        .unwrap();
    assert_eq!(
        ready_animation.clip,
        NetworkNpcCombatClip0104::CorruptionReady
    );

    let mut corruption = skill_target_record(40, 1, 11);
    corruption[20] = 8;
    write_i16(&mut corruption, 22, 0);
    let mut malformed_corruption = npc_corruption_hit_payload(21, 7, 1, &[corruption]);
    malformed_corruption.pop();
    let malformed_corruption_frame =
        frame(P_FE2CL_NPC_SKILL_CORRUPTION_HIT, malformed_corruption);
    push_frame(&mut app, EPOCH_ONE, malformed_corruption_frame.clone());
    app.update();

    assert_eq!(
        *app.world()
            .get::<NetworkNpcCombatAnimation0104>(npc)
            .unwrap(),
        ready_animation,
        "a rejected hit must not consume the ready phase or publish hit animation"
    );
    assert_eq!(
        app.world().get::<NetworkPcAppearance0104>(pc_11).unwrap(),
        &pc_11_before
    );
    assert_eq!(
        app.world().get::<NetworkNpcAppearance0104>(npc).unwrap(),
        &npc_before
    );

    let mut first = skill_target_record(20, 1, 12);
    write_i32(&mut first, 8, 0);
    write_i32(&mut first, 12, 200);
    write_i32(&mut first, 16, 800);
    let mut invalid_second = skill_target_record(20, 2, 21);
    write_i32(&mut invalid_second, 8, 2);
    write_i32(&mut invalid_second, 12, 200);
    write_i32(&mut invalid_second, 16, 300);
    let malformed_nano_frame = frame(
        P_FE2CL_NANO_SKILL_USE,
        nano_skill_payload(11, 5, 6, 90, false, 1, &[first, invalid_second]),
    );
    push_frame(&mut app, EPOCH_ONE, malformed_nano_frame.clone());
    app.update();

    assert_eq!(
        app.world().get::<NetworkPcAppearance0104>(pc_11).unwrap(),
        &pc_11_before,
        "caster Nano prefix must not commit before every target validates"
    );
    assert_eq!(
        app.world().get::<NetworkPcAppearance0104>(pc_12).unwrap(),
        &pc_12_before,
        "the first valid record must not commit before a later record validates"
    );
    assert_eq!(
        app.world().get::<NetworkNpcAppearance0104>(npc).unwrap(),
        &npc_before
    );

    let malformed = &app
        .world()
        .resource::<MalformedLifecycleFrames0104>()
        .frames;
    assert_eq!(malformed.len(), 2);
    assert_eq!(malformed[0].frame, malformed_corruption_frame);
    assert!(matches!(
        malformed[0].error,
        EntityLifecycleDecodeError0104::NpcSkillAuthority(_)
    ));
    assert_eq!(malformed[1].frame, malformed_nano_frame);
    assert!(matches!(
        malformed[1].error,
        EntityLifecycleDecodeError0104::NanoSkillAuthority(
            WorldNanoProjectionError0104::InvalidBoolean { .. }
        )
    ));
}

#[test]
fn warhead_reply_updates_npc_health_and_hit_animation() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    push_frame(&mut app, EPOCH_ONE, frame(P_FE2CL_NPC_ENTER,
        NpcEnter0104 { appearance: npc_appearance(21, 4001, [0,0,0]) }.encode()));
    app.update();
    let entity = app.world().resource::<NetworkNpcRegistry0104>().get(21).unwrap();
    let mut payload = vec![0;24];
    payload[20..24].copy_from_slice(&1_i32.to_le_bytes());
    payload.extend(AttackResult0104 { entity_type: 0, id: 21, protected: 0, damage: 450, hp: 50, hit_flag: 1 }.encode());
    push_frame(&mut app, EPOCH_ONE, frame(ffone_protocol::packet::P_FE2CL_PC_GRENADE_STYLE_HIT, payload));
    app.update();
    assert_eq!(app.world().get::<NetworkNpcAppearance0104>(entity).unwrap().0.hp, 50);
    assert_eq!(app.world().get::<NetworkNpcCombatAnimation0104>(entity).unwrap().clip, NetworkNpcCombatClip0104::Wound);
}
