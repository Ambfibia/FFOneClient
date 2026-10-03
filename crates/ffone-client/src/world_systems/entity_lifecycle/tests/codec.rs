use super::*;

pub(super) fn frame(packet_type: u32, payload: Vec<u8>) -> DecodedFrame {
    DecodedFrame {
        packet_type,
        flags: 0,
        checksum: 0,
        payload,
    }
}

pub(super) fn npc_skill_payload(bytes: usize, npc_id: i32, skill_id: i16) -> Vec<u8> {
    let mut payload = vec![0u8; bytes];
    payload[0..4].copy_from_slice(&npc_id.to_le_bytes());
    if bytes >= 6 {
        payload[4..6].copy_from_slice(&skill_id.to_le_bytes());
    }
    payload
}

pub(super) fn npc_skill_hit_payload(
    npc_id: i32,
    skill_id: i16,
    skill_type: i32,
    records: &[Vec<u8>],
) -> Vec<u8> {
    let mut payload = NpcSkillHitPrefix0104 {
        npc_id,
        skill_id,
        pack_padding: [0; 2],
        position: [0; 3],
        skill_type,
        target_count: i32::try_from(records.len()).unwrap(),
    }
    .encode_prefix();
    for record in records {
        payload.extend_from_slice(record);
    }
    payload
}

pub(super) fn npc_corruption_hit_payload(
    npc_id: i32,
    skill_id: i16,
    style: i16,
    records: &[Vec<u8>],
) -> Vec<u8> {
    let mut payload = NpcSkillCorruptionHitPrefix0104 {
        npc_id,
        skill_id,
        style,
        position: [0; 3],
        target_count: i32::try_from(records.len()).unwrap(),
    }
    .encode_prefix();
    for record in records {
        payload.extend_from_slice(record);
    }
    payload
}

pub(super) fn nano_skill_payload(
    pc_id: i32,
    nano_id: i16,
    skill_id: i16,
    stamina: i16,
    deactivated: bool,
    skill_type: i32,
    records: &[Vec<u8>],
) -> Vec<u8> {
    let mut payload = NanoSkillUseSuccessPrefix0104 {
        pc_id,
        bullet_id: 0,
        pack_padding: [0],
        skill_id,
        arg1: 0,
        arg2: 0,
        arg3: 0,
        nano_deactivated: i32::from(deactivated),
        nano_id,
        nano_stamina: stamina,
        skill_type,
        target_count: i32::try_from(records.len()).unwrap(),
    }
    .encode_prefix();
    for record in records {
        payload.extend_from_slice(record);
    }
    payload
}

pub(super) fn push_frame(app: &mut App, epoch: NetworkSessionEpoch0104, frame: DecodedFrame) {
    app.world_mut()
        .resource_mut::<NetworkEntityLifecycleIngress0104>()
        .push_frame(epoch, frame);
}

#[test]
fn frame_decoder_covers_pc_and_npc_lifecycle_and_preserves_unknowns() {
    let pc = pc_appearance(11, [100, 200, 300]);
    let decoded = decode_entity_lifecycle_frame_0104(&frame(
        P_FE2CL_PC_NEW,
        PcNew0104 {
            appearance: pc.clone(),
        }
        .encode(),
    ))
    .unwrap();
    assert_eq!(
        decoded,
        Some(DecodedEntityLifecyclePacket0104::PcNew(PcNew0104 {
            appearance: pc,
        }))
    );

    let npc = npc_appearance(21, 4001, [400, 500, 600]);
    assert_eq!(
        decode_entity_lifecycle_frame_0104(&frame(
            P_FE2CL_NPC_ENTER,
            NpcEnter0104 { appearance: npc }.encode(),
        ))
        .unwrap(),
        Some(DecodedEntityLifecyclePacket0104::NpcEnter(NpcEnter0104 {
            appearance: npc
        }))
    );

    assert!(matches!(
        decode_entity_lifecycle_frame_0104(&frame(P_FE2CL_PC_EXIT, vec![0; 7])),
        Err(EntityLifecycleDecodeError0104::Fixed(
            PayloadError::WrongSize {
                expected: 8,
                actual: 7
            }
        ))
    ));
    assert_eq!(
        decode_entity_lifecycle_frame_0104(&frame(0x3100_0abc, vec![1, 2, 3])),
        Ok(None)
    );

    let transportation = TransportationAppearance0104 {
        transportation_kind: 3,
        id: 71,
        transportation_type: 9,
        position: [100, 200, 300],
    };
    assert_eq!(
        decode_entity_lifecycle_frame_0104(&frame(
            P_FE2CL_TRANSPORTATION_ENTER,
            transportation.encode(),
        ))
        .unwrap(),
        Some(DecodedEntityLifecyclePacket0104::TransportationUpsert(
            transportation
        ))
    );
    let shiny = ShinyAppearance0104 {
        shiny_id: 81,
        shiny_type: 2,
        map_number: 0,
        position: [400, 500, 600],
    };
    assert_eq!(
        decode_entity_lifecycle_frame_0104(&frame(P_FE2CL_SHINY_NEW, shiny.encode())).unwrap(),
        Some(DecodedEntityLifecyclePacket0104::ShinyUpsert(shiny))
    );
}

#[test]
fn frame_decoder_covers_remote_regen_and_sudden_dead_with_exact_0104_types() {
    let regen = PcRegen0104 {
        pc_id: 11,
        hp: 875,
        position: [700, 800, 900],
        angle: -135,
        condition_bit_flag: 0x1020_3040,
        pc_state: -2,
        special_state: 3,
        nano: Nano0104 {
            id: 37,
            skill_id: 42,
            stamina: 99,
        },
    };
    assert_eq!(
        decode_entity_lifecycle_frame_0104(&frame(P_FE2CL_PC_REGEN, regen.encode())).unwrap(),
        Some(DecodedEntityLifecyclePacket0104::PcRegen(regen))
    );

    let sudden_dead = PcSuddenDead0104 {
        pc_id: 11,
        sudden_dead_reason: 4,
        damage: 900,
        hp: 0,
    };
    assert_eq!(
        decode_entity_lifecycle_frame_0104(&frame(
            P_FE2CL_PC_SUDDEN_DEAD,
            sudden_dead.encode(),
        ))
        .unwrap(),
        Some(DecodedEntityLifecyclePacket0104::PcSuddenDead(sudden_dead))
    );
}

#[test]
fn remote_sudden_dead_updates_hp_and_stops_motion_without_inventing_wire_state() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    let mut initial = pc_appearance(11, [100, 200, 300]);
    initial.condition_bit_flag = 0x200;
    initial.pc_state = 7;
    initial.special_state = 8;
    initial.nano = Nano0104 {
        id: 12,
        skill_id: 13,
        stamina: 14,
    };
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_PC_NEW,
            PcNew0104 {
                appearance: initial.clone(),
            }
            .encode(),
        ),
    );
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_PC_MOVE, pc_move(11, [400, 500, 600]).encode()),
    );
    app.update();

    let entity = app
        .world()
        .resource::<RemotePcRegistry0104>()
        .get(11)
        .unwrap();
    let visible_pose = app.world().get::<Transform>(entity).unwrap().clone();
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_PC_SUDDEN_DEAD,
            PcSuddenDead0104 {
                pc_id: 11,
                sudden_dead_reason: 4,
                damage: 2_000,
                hp: 0,
            }
            .encode(),
        ),
    );
    app.update();

    let appearance = &app
        .world()
        .get::<NetworkPcAppearance0104>(entity)
        .unwrap()
        .0;
    assert_eq!(appearance.hp, 0);
    assert_eq!(appearance.condition_bit_flag, initial.condition_bit_flag);
    assert_eq!(appearance.pc_state, initial.pc_state);
    assert_eq!(appearance.special_state, initial.special_state);
    assert_eq!(appearance.nano, initial.nano);
    assert_eq!(app.world().get::<Transform>(entity).unwrap(), &visible_pose);
    let motion = app.world().get::<RemoteMotion>(entity).unwrap();
    assert_eq!(motion.target_position, visible_pose.translation);
    assert_eq!(motion.target_rotation, visible_pose.rotation);
    assert_eq!(motion.velocity, Vec3::ZERO);
    assert_eq!(motion.speed, 0.0);
    assert_eq!(motion.movement_key, 0);
    assert_eq!(
        app.world().get::<RemoteAnimation>(entity).unwrap().state,
        crate::remote::RemoteAnimationState::Idle
    );
}

#[test]
fn ordered_attack_skill_and_nano_authority_use_last_absolute_packet() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_PC_NEW,
            PcNew0104 {
                appearance: pc_appearance(11, [0, 0, 0]),
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
    let remote = app
        .world()
        .resource::<RemotePcRegistry0104>()
        .get(11)
        .unwrap();
    let npc = app
        .world()
        .resource::<NetworkNpcRegistry0104>()
        .get(21)
        .unwrap();

    let pc_attack = PcAttackNpcsSuccess0104 {
        battery_w: 90,
        results: vec![AttackResult0104 {
            entity_type: 2,
            id: 21,
            protected: 0,
            damage: 400,
            hp: 100,
            hit_flag: 1,
        }],
    };
    let mut npc_heal = skill_target_record(16, 2, 21);
    write_i32(&mut npc_heal, 8, 50);
    write_i32(&mut npc_heal, 12, 400);
    let npc_skill = frame(
        P_FE2CL_NPC_SKILL_HIT,
        npc_skill_hit_payload(21, 7, 2, &[npc_heal]),
    );

    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_PC_ATTACK_NPCS_SUCC, pc_attack.encode().unwrap()),
    );
    push_frame(&mut app, EPOCH_ONE, npc_skill.clone());
    app.update();
    assert_eq!(
        app.world()
            .get::<NetworkNpcAppearance0104>(npc)
            .unwrap()
            .0
            .hp,
        400
    );

    push_frame(&mut app, EPOCH_ONE, npc_skill);
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_PC_ATTACK_NPCS_SUCC, pc_attack.encode().unwrap()),
    );
    app.update();
    assert_eq!(
        app.world()
            .get::<NetworkNpcAppearance0104>(npc)
            .unwrap()
            .0
            .hp,
        100
    );

    let npc_attack = NpcAttackPcs0104 {
        npc_id: 21,
        results: vec![AttackResult0104 {
            entity_type: 1,
            id: 11,
            protected: 0,
            damage: 300,
            hp: 700,
            hit_flag: 1,
        }],
    };
    let mut remote_heal = skill_target_record(16, 1, 11);
    write_i32(&mut remote_heal, 8, 150);
    write_i32(&mut remote_heal, 12, 850);
    let nano = frame(
        P_FE2CL_NANO_SKILL_USE,
        nano_skill_payload(11, 5, 6, 80, false, 2, &[remote_heal]),
    );

    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_NPC_ATTACK_PCS, npc_attack.encode().unwrap()),
    );
    push_frame(&mut app, EPOCH_ONE, nano.clone());
    app.update();
    let remote_appearance = &app
        .world()
        .get::<NetworkPcAppearance0104>(remote)
        .unwrap()
        .0;
    assert_eq!(remote_appearance.hp, 850);
    assert_eq!(remote_appearance.nano.stamina, 80);

    push_frame(&mut app, EPOCH_ONE, nano);
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_NPC_ATTACK_PCS, npc_attack.encode().unwrap()),
    );
    app.update();
    assert_eq!(
        app.world()
            .get::<NetworkPcAppearance0104>(remote)
            .unwrap()
            .0
            .hp,
        700
    );

    let mut leech = skill_target_record(36, 2, 21);
    write_i32(&mut leech, 8, 50);
    write_i32(&mut leech, 12, 900);
    write_i32(&mut leech, 16, 2);
    write_i32(&mut leech, 20, 21);
    write_i32(&mut leech, 24, 0);
    write_i32(&mut leech, 28, 100);
    write_i32(&mut leech, 32, 300);
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_NANO_SKILL_USE,
            nano_skill_payload(11, 5, 6, 0, true, 30, &[leech]),
        ),
    );
    app.update();
    let remote_appearance = &app
        .world()
        .get::<NetworkPcAppearance0104>(remote)
        .unwrap()
        .0;
    assert_eq!(remote_appearance.hp, 900, "Leech HP belongs to the caster");
    let inactive_nano = Nano0104 {
        id: 0,
        skill_id: 0,
        stamina: 0,
    };
    assert_eq!(remote_appearance.nano, inactive_nano);
    assert_eq!(
        app.world().get::<PendingPcVisual0104>(remote).unwrap().nano,
        inactive_nano
    );
    assert_eq!(
        app.world()
            .get::<NetworkNpcAppearance0104>(npc)
            .unwrap()
            .0
            .hp,
        300
    );
}

#[test]
fn malformed_passthrough_and_ignored_frame_diagnostics_are_lossless_and_separate() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    let malformed = frame(P_FE2CL_PC_NEW, vec![0xaa; 231]);
    let unknown = frame(0x3100_0abc, vec![1, 2, 3, 4]);
    push_frame(&mut app, EPOCH_ONE, malformed.clone());
    push_frame(&mut app, EPOCH_ONE, unknown.clone());
    app.update();

    let malformed_diagnostics = app.world().resource::<MalformedLifecycleFrames0104>();
    assert_eq!(malformed_diagnostics.frames.len(), 1);
    assert_eq!(malformed_diagnostics.frames[0].frame, malformed);
    assert!(matches!(
        malformed_diagnostics.frames[0].error,
        EntityLifecycleDecodeError0104::Fixed(PayloadError::WrongSize {
            expected: 232,
            actual: 231
        })
    ));
    let passthrough = app.world().resource::<PassthroughLifecycleFrames0104>();
    assert_eq!(
        passthrough.frames,
        vec![PassthroughLifecycleFrame0104 {
            epoch: EPOCH_ONE,
            frame: unknown,
        }]
    );
    let bootstrap = app.world().resource::<LifecycleBootstrapDiagnostics0104>();
    assert!(bootstrap.passthrough.is_empty());
    assert!(
        app.world()
            .resource::<IgnoredLifecycleFrames0104>()
            .frames
            .is_empty()
    );
}
