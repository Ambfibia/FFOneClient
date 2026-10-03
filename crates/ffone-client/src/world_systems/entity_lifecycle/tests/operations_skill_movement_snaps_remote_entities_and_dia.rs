use super::*;

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(NetworkEntityLifecycle0104Plugin);
    app
}

pub(super) fn item(index: usize) -> ItemBase0104 {
    ItemBase0104 {
        item_type: index as i16,
        item_id: 1000 + index as i16,
        option: index as i32,
        time_limit: 2000 + index as i32,
    }
}

pub(super) fn pc_appearance(pc_id: i32, position: [i32; 3]) -> PcAppearance0104 {
    PcAppearance0104 {
        id: pc_id,
        style: PcStyle0104 {
            pc_uid: i64::from(pc_id) * 100,
            name_check: 1,
            first_name: FixedUtf16::from_str("Test").unwrap(),
            last_name: FixedUtf16::from_str("Player").unwrap(),
            gender: 1,
            face_style: 2,
            hair_style: 3,
            hair_color: 4,
            skin_color: 5,
            eye_color: 6,
            height: 7,
            body: 8,
            class: 9,
        },
        condition_bit_flag: 0,
        pc_state: 1,
        special_state: 0,
        level: 12,
        hp: 1000,
        map_number: 8,
        position,
        angle: 90,
        equipment: std::array::from_fn(item),
        nano: Nano0104 {
            id: 5,
            skill_id: 6,
            stamina: 150,
        },
        render_type: 1,
    }
}

pub(super) fn npc_appearance(npc_id: i32, npc_type: i32, position: [i32; 3]) -> NpcAppearance0104 {
    NpcAppearance0104 {
        npc_id,
        npc_type,
        hp: 500,
        condition_bit_flag: 0,
        position,
        angle: -45,
        barker_type: 3,
    }
}

pub(super) fn skill_target_record(size: usize, entity_type: i32, id: i32) -> Vec<u8> {
    let mut record = vec![0; size];
    record[0..4].copy_from_slice(&entity_type.to_le_bytes());
    record[4..8].copy_from_slice(&id.to_le_bytes());
    record
}

pub(super) fn pc_move(pc_id: i32, position: [i32; 3]) -> PcMove0104 {
    PcMove0104 {
        movement: PcMoveRequest0104 {
            client_time: 100,
            position,
            velocity: [1.0, 2.0, 3.0],
            angle: 45,
            key_value: 1,
            speed: 250,
        },
        pc_id,
        server_time: 200,
    }
}

pub(super) fn begin(app: &mut App, epoch: NetworkSessionEpoch0104) {
    app.world_mut()
        .resource_mut::<NetworkEntityLifecycleIngress0104>()
        .begin_session(epoch, LOCAL_PC_ID);
    app.update();
}

#[test]
fn npc_skill_presentation_keeps_ground_coordinates_and_style_from_validated_packets() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    upsert_npc(app.world_mut(), EPOCH_ONE, npc_appearance(99, 2, [0; 3]));
    let position = [125, -830, 470];
    let mut ready = npc_skill_payload(20, 99, 180);
    ready[6..8].copy_from_slice(&2i16.to_le_bytes());
    for (i, value) in position.iter().enumerate() {
        ready[8 + i * 4..12 + i * 4].copy_from_slice(&i32::to_le_bytes(*value));
    }
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(packet::P_FE2CL_NPC_SKILL_CORRUPTION_READY, ready),
    );
    let hit = ffone_protocol::NpcSkillHitPrefix0104 {
        npc_id: 99,
        skill_id: 176,
        pack_padding: [0; 2],
        position,
        skill_type: 1,
        target_count: 0,
    }
    .encode_prefix();
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(packet::P_FE2CL_NPC_SKILL_HIT, hit.clone()),
    );
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(packet::P_FE2CL_NPC_SKILL_HIT, hit[..20].to_vec()),
    );
    app.update();
    let events = &app.world().resource::<NetworkNpcSkillEffectEvents0104>().0;
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].position, Some(position));
    assert_eq!(events[0].style, Some(2));
    assert_eq!(events[1].position, Some(position));
    assert_eq!(events[1].style, None);
    begin(&mut app, EPOCH_TWO);
    assert!(
        app.world()
            .resource::<NetworkNpcSkillEffectEvents0104>()
            .0
            .is_empty()
    );
}

#[test]
fn remote_regen_transactionally_refreshes_authoritative_pc_components_only() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_PC_NEW,
            PcNew0104 {
                appearance: pc_appearance(11, [100, 200, 300]),
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
    push_frame(&mut app, EPOCH_ONE, frame(P_FE2CL_PC_REGEN, regen.encode()));
    app.update();

    let entity = app
        .world()
        .resource::<RemotePcRegistry0104>()
        .get(11)
        .unwrap();
    let appearance = &app
        .world()
        .get::<NetworkPcAppearance0104>(entity)
        .unwrap()
        .0;
    assert_eq!(appearance.hp, regen.hp);
    assert_eq!(appearance.position, regen.position);
    assert_eq!(appearance.angle, regen.angle);
    assert_eq!(appearance.condition_bit_flag, regen.condition_bit_flag);
    assert_eq!(appearance.pc_state, regen.pc_state);
    assert_eq!(appearance.special_state, regen.special_state);
    assert_eq!(appearance.nano, regen.nano);
    assert_eq!(appearance.map_number, 8);
    assert_eq!(appearance.equipment[4], item(4));

    assert_eq!(
        app.world().get::<PendingPcVisual0104>(entity).unwrap().nano,
        regen.nano
    );
    let expected_position = ProtocolPosition::new(regen.position).to_native();
    let expected_rotation = ProtocolYawDegrees::new(regen.angle).native_root_rotation();
    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_eq!(transform.translation, expected_position);
    assert_eq!(transform.rotation, expected_rotation);
    let motion = app.world().get::<RemoteMotion>(entity).unwrap();
    assert_eq!(motion.target_position, expected_position);
    assert_eq!(motion.target_rotation, expected_rotation);
    assert_eq!(motion.velocity, Vec3::ZERO);
    assert_eq!(motion.speed, 0.0);
    assert_eq!(motion.movement_key, 0);
    assert_eq!(
        app.world().get::<RemoteAnimation>(entity).unwrap().state,
        crate::remote::RemoteAnimationState::Idle
    );
}

#[test]
fn malformed_remote_regen_family_is_transactional_and_local_ownership_is_ignored() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_PC_NEW,
            PcNew0104 {
                appearance: pc_appearance(11, [100, 200, 300]),
            }
            .encode(),
        ),
    );
    app.update();

    let entity = app
        .world()
        .resource::<RemotePcRegistry0104>()
        .get(11)
        .unwrap();
    let appearance_before = app
        .world()
        .get::<NetworkPcAppearance0104>(entity)
        .unwrap()
        .clone();
    let pending_before = app
        .world()
        .get::<PendingPcVisual0104>(entity)
        .unwrap()
        .clone();
    let motion_before = *app.world().get::<RemoteMotion>(entity).unwrap();
    let animation_before = *app.world().get::<RemoteAnimation>(entity).unwrap();
    let transform_before = app.world().get::<Transform>(entity).unwrap().clone();

    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_PC_REGEN, vec![0xaa; PcRegen0104::SIZE - 1]),
    );
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_PC_SUDDEN_DEAD,
            vec![0xbb; PcSuddenDead0104::SIZE + 1],
        ),
    );
    let local = PcRegen0104 {
        pc_id: LOCAL_PC_ID,
        hp: 1,
        position: [9, 8, 7],
        angle: 6,
        condition_bit_flag: 5,
        pc_state: 4,
        special_state: 3,
        nano: Nano0104 {
            id: 2,
            skill_id: 1,
            stamina: 0,
        },
    };
    let local_frame = frame(P_FE2CL_PC_REGEN, local.encode());
    push_frame(&mut app, EPOCH_ONE, local_frame.clone());
    app.update();

    assert_eq!(
        app.world().get::<NetworkPcAppearance0104>(entity).unwrap(),
        &appearance_before
    );
    assert_eq!(
        app.world().get::<PendingPcVisual0104>(entity).unwrap(),
        &pending_before
    );
    assert_eq!(
        *app.world().get::<RemoteMotion>(entity).unwrap(),
        motion_before
    );
    assert_eq!(
        *app.world().get::<RemoteAnimation>(entity).unwrap(),
        animation_before
    );
    assert_eq!(
        app.world().get::<Transform>(entity).unwrap(),
        &transform_before
    );

    let malformed = &app
        .world()
        .resource::<MalformedLifecycleFrames0104>()
        .frames;
    assert_eq!(malformed.len(), 2);
    assert!(matches!(
        malformed[0].error,
        EntityLifecycleDecodeError0104::Fixed(PayloadError::WrongSize {
            expected: PcRegen0104::SIZE,
            actual
        }) if actual == PcRegen0104::SIZE - 1
    ));
    assert!(matches!(
        malformed[1].error,
        EntityLifecycleDecodeError0104::Fixed(PayloadError::WrongSize {
            expected: PcSuddenDead0104::SIZE,
            actual
        }) if actual == PcSuddenDead0104::SIZE + 1
    ));
    assert_eq!(
        app.world()
            .resource::<IgnoredLifecycleFrames0104>()
            .frames
            .last(),
        Some(&IgnoredLifecycleFrame0104 {
            epoch: EPOCH_ONE,
            frame: local_frame,
            reason: IgnoredLifecycleFrameReason0104::LocalPlayer { pc_id: LOCAL_PC_ID },
        })
    );
    assert_eq!(
        app.world()
            .resource::<RemotePcRegistry0104>()
            .get(LOCAL_PC_ID),
        None
    );
}

#[test]
fn typed_bootstrap_spawns_remote_appearance_only_and_excludes_local_player() {
    let mut app = test_app();
    app.world_mut()
        .resource_mut::<NetworkEntityLifecycleIngress0104>()
        .begin_session(EPOCH_ONE, LOCAL_PC_ID);
    app.world_mut()
        .resource_mut::<NetworkEntityLifecycleIngress0104>()
        .push_bootstrap(
            EPOCH_ONE,
            InitialAroundPacket0104::Players(vec![
                pc_appearance(LOCAL_PC_ID, [0, 0, 0]),
                pc_appearance(11, [100, 200, 300]),
            ]),
        );
    app.world_mut()
        .resource_mut::<NetworkEntityLifecycleIngress0104>()
        .push_bootstrap(
            EPOCH_ONE,
            InitialAroundPacket0104::Npcs(vec![npc_appearance(21, 4001, [400, 500, 600])]),
        );
    app.update();

    let player_entity = {
        let registry = app.world().resource::<RemotePcRegistry0104>();
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.get(LOCAL_PC_ID), None);
        registry.get(11).unwrap()
    };
    let npc_entity = {
        let registry = app.world().resource::<NetworkNpcRegistry0104>();
        assert_eq!(registry.len(), 1);
        registry.get(21).unwrap()
    };

    assert_eq!(
        app.world()
            .get::<NetworkSessionEntity0104>(player_entity)
            .unwrap()
            .epoch,
        EPOCH_ONE
    );
    assert_eq!(
        app.world()
            .get::<PendingPcVisual0104>(player_entity)
            .unwrap()
            .equipment[4]
            .item_id,
        1004
    );
    assert_eq!(
        app.world()
            .get::<Transform>(player_entity)
            .unwrap()
            .translation,
        Vec3::new(-1.0, 3.0, 2.0)
    );
    assert!(app.world().get::<Mesh3d>(player_entity).is_none());
    assert!(app.world().get::<WorldAssetRoot>(player_entity).is_none());
    assert_eq!(
        app.world()
            .get::<PendingNpcVisual0104>(npc_entity)
            .unwrap()
            .npc_type,
        4001
    );
    assert!(app.world().get::<Mesh3d>(npc_entity).is_none());
    assert!(app.world().get::<WorldAssetRoot>(npc_entity).is_none());
    assert_eq!(
        app.world()
            .resource::<NetworkEntityLifecycleStats0104>()
            .local_player_appearances_ignored,
        1
    );
}

#[test]
fn movement_without_appearance_never_creates_an_invisible_player() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    let movement = frame(P_FE2CL_PC_MOVE, pc_move(11, [100, 200, 300]).encode());
    push_frame(&mut app, EPOCH_ONE, movement.clone());
    app.update();

    assert!(app.world().resource::<RemotePcRegistry0104>().is_empty());
    let mut query = app.world_mut().query::<&NetworkRemotePc0104>();
    assert_eq!(query.iter(app.world()).count(), 0);
    assert_eq!(
        app.world().resource::<IgnoredLifecycleFrames0104>().frames,
        vec![IgnoredLifecycleFrame0104 {
            epoch: EPOCH_ONE,
            frame: movement,
            reason: IgnoredLifecycleFrameReason0104::MissingAppearance {
                kind: LifecycleEntityKind0104::Player,
                id: 11,
            },
        }]
    );
}

#[test]
fn server_and_skill_ready_barkers_preserve_network_fifo_order() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_REP_BARKER,
            NpcBarker0104 {
                npc_id: 21,
                mission_string_id: 3412,
            }
            .encode(),
        ),
    );
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(P_FE2CL_NPC_SKILL_READY, npc_skill_payload(20, 21, 8)),
    );
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_NPC_SKILL_CORRUPTION_READY,
            npc_skill_payload(20, 21, 9),
        ),
    );
    app.update();

    assert_eq!(
        app.world_mut()
            .resource_mut::<NetworkNpcBarkerEventQueue0104>()
            .take_all(),
        VecDeque::from([
            NetworkNpcBarkerEvent0104::Mission(NpcBarker0104 {
                npc_id: 21,
                mission_string_id: 3412,
            }),
            NetworkNpcBarkerEvent0104::SkillReady(NpcSkillSignal0104 {
                npc_id: 21,
                skill_id: Some(8),
                kind: NpcSkillSignalKind0104::Ready,
            }),
            NetworkNpcBarkerEvent0104::SkillReady(NpcSkillSignal0104 {
                npc_id: 21,
                skill_id: Some(9),
                kind: NpcSkillSignalKind0104::CorruptionReady,
            }),
        ])
    );
    assert!(
        app.world()
            .resource::<PassthroughLifecycleFrames0104>()
            .frames
            .is_empty(),
        "recognized Barker packets must no longer disappear into passthrough"
    );
}

#[test]
fn buff_timeout_clears_npc_and_remote_masks_without_changing_hp_or_position() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
    let mut pc = pc_appearance(11, [10, 20, 30]);
    let mut npc = npc_appearance(21, 4001, [40, 50, 60]);
    pc.condition_bit_flag = 512;
    npc.condition_bit_flag = 512;
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_PC_NEW,
            PcNew0104 {
                appearance: pc.clone(),
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
                appearance: npc.clone(),
            }
            .encode(),
        ),
    );
    app.update();
    for (character_type, character_id) in [(1, 11), (2, 21), (4, 21)] {
        push_frame(
            &mut app,
            EPOCH_ONE,
            frame(
                ffone_protocol::packet::P_FE2CL_CHAR_TIME_BUFF_TIME_OUT,
                CharTimeBuffTimeout0104 {
                    character_type,
                    character_id,
                    condition_bit_flag: 0,
                }
                .encode(),
            ),
        );
    }
    app.update();
    let pc_entity = app
        .world()
        .resource::<RemotePcRegistry0104>()
        .get(11)
        .unwrap();
    let npc_entity = app
        .world()
        .resource::<NetworkNpcRegistry0104>()
        .get(21)
        .unwrap();
    pc.condition_bit_flag = 0;
    npc.condition_bit_flag = 0;
    assert_eq!(
        app.world()
            .get::<NetworkPcAppearance0104>(pc_entity)
            .unwrap()
            .0,
        pc
    );
    assert_eq!(
        app.world()
            .get::<NetworkNpcAppearance0104>(npc_entity)
            .unwrap()
            .0,
        npc
    );
}

#[test]
fn local_nano_success_is_typed_but_only_remote_targets_are_lifecycle_owned() {
    let mut app = test_app();
    begin(&mut app, EPOCH_ONE);
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

    let mut damage = skill_target_record(20, 2, 21);
    write_i32(&mut damage, 8, 0);
    write_i32(&mut damage, 12, 155);
    write_i32(&mut damage, 16, 345);
    let local_success = frame(
        P_FE2CL_NANO_SKILL_USE_SUCC,
        nano_skill_payload(LOCAL_PC_ID, 5, 6, 80, false, 1, &[damage]),
    );
    assert!(matches!(
        decode_entity_lifecycle_frame_0104(&local_success),
        Ok(Some(DecodedEntityLifecyclePacket0104::NanoSkillAuthority(
            WorldNanoAuthoritativeProjection0104 {
                delivery: ffone_protocol::NanoSkillUseDelivery0104::LocalSuccess,
                ..
            }
        )))
    ));

    push_frame(&mut app, EPOCH_ONE, local_success);
    app.update();
    assert_eq!(
        app.world()
            .get::<NetworkNpcAppearance0104>(npc)
            .unwrap()
            .0
            .hp,
        345
    );
    assert!(
        app.world()
            .resource::<RemotePcRegistry0104>()
            .get(LOCAL_PC_ID)
            .is_none(),
        "lifecycle must never create or mutate a local-PC appearance"
    );
}

#[test]
fn skill_movement_snaps_remote_entities_and_diagnoses_local_or_offscreen_targets() {
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
    for npc_id in [21, 22] {
        push_frame(
            &mut app,
            EPOCH_ONE,
            frame(
                P_FE2CL_NPC_ENTER,
                NpcEnter0104 {
                    appearance: npc_appearance(npc_id, 4000 + npc_id, [0, 0, 0]),
                }
                .encode(),
            ),
        );
    }
    app.update();
    let remote = app
        .world()
        .resource::<RemotePcRegistry0104>()
        .get(12)
        .unwrap();
    let moved_npc = app
        .world()
        .resource::<NetworkNpcRegistry0104>()
        .get(22)
        .unwrap();
    app.world_mut()
        .entity_mut(moved_npc)
        .insert(NetworkNpcMotion0104 {
            destination: Vec3::X,
            speed: 1.0,
            move_style: 1,
        });

    let movement_record = |id, map_number, position: [i32; 3]| {
        let mut record = skill_target_record(24, 1, id);
        write_i32(&mut record, 8, map_number);
        write_i32(&mut record, 12, position[0]);
        write_i32(&mut record, 16, position[1]);
        write_i32(&mut record, 20, position[2]);
        record
    };
    let remote_move = movement_record(12, 9, [100, 200, 300]);
    let local_move = movement_record(LOCAL_PC_ID, 9, [400, 500, 600]);
    let missing_move = movement_record(999, 9, [700, 800, 900]);
    let nano_frame = frame(
        P_FE2CL_NANO_SKILL_USE,
        nano_skill_payload(
            11,
            5,
            6,
            75,
            false,
            28,
            &[remote_move, local_move, missing_move],
        ),
    );
    push_frame(&mut app, EPOCH_ONE, nano_frame.clone());

    let mut npc_move = skill_target_record(24, 2, 22);
    write_i32(&mut npc_move, 8, 9);
    write_i32(&mut npc_move, 12, -100);
    write_i32(&mut npc_move, 16, 250);
    write_i32(&mut npc_move, 20, 350);
    push_frame(
        &mut app,
        EPOCH_ONE,
        frame(
            P_FE2CL_NPC_SKILL_HIT,
            npc_skill_hit_payload(21, 7, 27, &[npc_move]),
        ),
    );
    app.update();

    let remote_appearance = &app
        .world()
        .get::<NetworkPcAppearance0104>(remote)
        .unwrap()
        .0;
    assert_eq!(remote_appearance.map_number, 9);
    assert_eq!(remote_appearance.position, [100, 200, 300]);
    let expected_remote_position = ProtocolPosition::new([100, 200, 300]).to_native();
    assert_eq!(
        app.world().get::<Transform>(remote).unwrap().translation,
        expected_remote_position
    );
    let motion = app.world().get::<RemoteMotion>(remote).unwrap();
    assert_eq!(motion.target_position, expected_remote_position);
    assert_eq!(motion.velocity, Vec3::ZERO);
    assert_eq!(motion.speed, 0.0);

    assert_eq!(
        app.world()
            .get::<NetworkNpcAppearance0104>(moved_npc)
            .unwrap()
            .0
            .position,
        [-100, 250, 350]
    );
    assert_eq!(
        app.world().get::<Transform>(moved_npc).unwrap().translation,
        ProtocolPosition::new([-100, 250, 350]).to_native()
    );
    assert!(app.world().get::<NetworkNpcMotion0104>(moved_npc).is_none());

    let ignored = &app.world().resource::<IgnoredLifecycleFrames0104>().frames;
    assert!(ignored.contains(&IgnoredLifecycleFrame0104 {
        epoch: EPOCH_ONE,
        frame: nano_frame.clone(),
        reason: IgnoredLifecycleFrameReason0104::LocalPlayer { pc_id: LOCAL_PC_ID },
    }));
    assert!(ignored.contains(&IgnoredLifecycleFrame0104 {
        epoch: EPOCH_ONE,
        frame: nano_frame,
        reason: IgnoredLifecycleFrameReason0104::MissingAppearance {
            kind: LifecycleEntityKind0104::Player,
            id: 999,
        },
    }));
}
