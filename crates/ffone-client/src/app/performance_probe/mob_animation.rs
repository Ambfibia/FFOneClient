//! Opt-in offline combat replay through the real packet and rendering owners.
use super::*;
use bevy::{animation::graph::AnimationNodeType, gltf::Gltf};
use ffone_client::entity_lifecycle::{
    NetworkNpcAnimationLayers0104, NetworkNpcRegistry0104, NetworkSessionEpoch0104,
};
use ffone_client::network_world_runtime::{
    NetworkNpcVisual0104, NetworkNpcVisualMaterialReady0104,
};

const NPC: i32 = 1_904_461;

#[derive(Default, Resource)]
struct Replay {
    special: bool,
    idle: bool,
    frame: u32,
    records: Vec<serde_json::Value>,
}

pub(super) fn install(app: &mut App) {
    if env::var_os("FFONE_PERF_SPECIAL_SKILLS").is_some()
        || env::var_os("FFONE_PERF_NPC_IDLE").is_some()
    {
        app.add_systems(Update, (|mut resets: ResMut<Messages<NetworkSessionReset>>| {
            // The offline worker rejects periodic world traffic. Its synthetic
            // errors must not cancel the authoritative packet replay.
            resets.clear();
        }).after(poll_network).before(NetworkSessionLifecycleSet::Apply));
    }
    app.insert_resource(Replay { special: env::var_os("FFONE_PERF_SPECIAL_SKILLS").is_some(),
        idle: env::var_os("FFONE_PERF_NPC_IDLE").is_some(), ..default() })
        .add_systems(Update, drive.after(poll_network)
            .after(NetworkSessionLifecycleSet::Apply)
            .before(consume_network_entity_lifecycle_0104))
        .add_systems(Update, idle_camera.after(drive))
        .add_systems(Last, record.before(super::measure));
}

fn idle_camera(
    replay: Res<Replay>,
    registry: Res<NetworkNpcRegistry0104>,
    mut cameras: Query<&mut LegacyOrbitCamera>,
) {
    if !replay.idle { return; }
    let Some(root) = registry.get(NPC) else { return; };
    for mut camera in &mut cameras {
        // Frame the complete, accepted Bad Max model without changing scale.
        camera.target = root;
        camera.height = 6.0;
        camera.distance = 32.0;
        camera.minimum_distance = 32.0;
        camera.maximum_distance = 32.0;
        camera.yaw_degrees = 0.0;
        camera.pitch_degrees = 10.0;
    }
}

fn drive(
    mut commands: Commands,
    mut capture: ResMut<Capture>,
    mut replay: ResMut<Replay>,
    players: Query<(Entity, &Transform), With<LocalPlayer>>,
    registry: Res<NetworkNpcRegistry0104>,
    ready: Query<(), With<NetworkNpcVisualMaterialReady0104>>,
    mut ingress: ResMut<NetworkEntityLifecycleIngress0104>,
    mut runtime: ResMut<RuntimeStatus>,
    mut buffs: ResMut<SkillBuffUiModel>,
    mut movement_buffs: ResMut<movement_buffs::MovementBuffs>,
    mut nano_inbox: ResMut<WorldNanoAuthorityInbox0104>,
) {
    if capture.ready.is_none() || capture.samples.is_empty() {
        return;
    }
    let Ok((player_entity, player)) = players.single() else { return };
    let epoch = NetworkSessionEpoch0104(capture.entry as u64);
    if replay.frame == 0 {
        ingress.begin_session(epoch, 1);
        if replay.special {
            // Packet post-state owns HP in this replay, just as in a shard
            // session. An offline avatar otherwise regenerates HP locally.
            commands.entity(player_entity).insert(LocalNetworkIdentity {
                pc_uid: 1,
                player_id: 1,
            });
        }
    }
    if replay.frame == 1 && registry.get(NPC).is_none_or(|npc| !ready.contains(npc)) {
        return;
    }
    if replay.frame == 1 {
        // Give the replay its full capture window after asynchronous model
        // loading, independent of disk and GPU upload latency.
        capture.samples.clear();
    }
    replay.frame += 1;
    let frame = replay.frame;
    let special = replay.special;
    if special { buffs.local_character_id = runtime.player_id; }
    let mut send = |packet_type, payload| {
        let frame = DecodedFrame { packet_type, flags: 0, checksum: 0, payload };
        if special {
            apply_world_npc_skill_response_frame(&frame, &mut runtime, &mut buffs, &mut nano_inbox)
                .expect("special skill local post-state");
            if packet_type == packet::P_FE2CL_NPC_SKILL_HIT && frame.payload[4..6] == 171i16.to_le_bytes() {
                assert_eq!(buffs.local_condition_bit_flag & 0x80, 0x80, "Snare packet post-state");
            }
            apply_world_nano_response_frame(&frame, &mut runtime, &mut buffs, &mut nano_inbox)
                .expect("reflected Nano local post-state");
            apply_skill_buff_frame(&frame, &mut buffs, &mut movement_buffs)
                .expect("skill buff packet post-state");
            if packet_type == packet::P_FE2CL_NPC_SKILL_CORRUPTION_HIT {
                assert_eq!(runtime.hp, Some(729));
                assert!(runtime.nano_slots[0].active);
                assert_eq!(runtime.nano_slots[0].nano_id, Some(2));
                assert!(matches!(runtime.nano_slots[0].stamina, 60 | 105));
            }
        }
        ingress.push_frame(
            epoch,
            frame,
        )
    };
    if frame == 1 {
        send(
            packet::P_FE2CL_NPC_ENTER,
            ffone_protocol::NpcEnter0104 {
                appearance: ffone_protocol::NpcAppearance0104 {
                    npc_id: NPC,
                    npc_type: if special { 7 } else { 461 },
                    hp: 1000,
                    condition_bit_flag: 0,
                    position: ProtocolPosition::from_native(
                        player.translation + if special || replay.idle { Vec3::new(0.0,0.0,8.0) } else { Vec3::new(40.0, 0.0, 40.0) },
                    )
                    .raw(),
                    angle: 225,
                    barker_type: 0,
                },
            }
            .encode(),
        );
    }
    if replay.idle {
        // Bug 44: several complete ambient cycles, then both packet styles.
        if matches!(frame, 360 | 480) {
            send(packet::P_FE2CL_NPC_MOVE, ffone_protocol::NpcMove0104 {
                npc_id: NPC,
                destination: ProtocolPosition::from_native(player.translation + Vec3::new(
                    if frame == 360 { 16.0 } else { -16.0 }, 0.0, 8.0)).raw(),
                speed: if frame == 360 { 300 } else { 500 },
                move_style: if frame == 360 { 0 } else { 1 },
            }.encode());
        }
        if matches!(frame, 90 | 180 | 270 | 400 | 520) {
            commands.spawn(Screenshot::primary_window()).observe(
                bevy::render::view::screenshot::save_to_disk(
                    capture.output.join(format!("npc-idle-{frame}.png"))));
        }
        return;
    }
    if special {
        let position=ProtocolPosition::from_native(player.translation).raw();
        if matches!(frame,30|150|270) {
            let mut payload=vec![0u8;20];
            payload[..4].copy_from_slice(&NPC.to_le_bytes());
            payload[4..6].copy_from_slice(&118i16.to_le_bytes());
            payload[6..8].copy_from_slice(&(((frame-30)/120) as i16).to_le_bytes());
            for (i,v) in position.iter().enumerate() { payload[8+i*4..12+i*4].copy_from_slice(&v.to_le_bytes()); }
            send(packet::P_FE2CL_NPC_SKILL_CORRUPTION_READY,payload);
        }
        if matches!(frame, 100 | 220 | 340) {
            let index = (frame - 100) / 120;
            if index == 2 {
                let mut reflected = ffone_protocol::NanoSkillUseSuccessPrefix0104 {
                    pc_id: 1, bullet_id: 0, pack_padding: [0], skill_id: 1,
                    arg1: 0, arg2: 0, arg3: 0, nano_deactivated: 0,
                    nano_id: 2, nano_stamina: 105, skill_type: 1, target_count: 1,
                }.encode_prefix();
                for value in [4i32, NPC, 0, 200, 800] { reflected.extend(value.to_le_bytes()); }
                send(packet::P_FE2CL_NANO_SKILL_USE_SUCC, reflected);
            }
            let mut payload = ffone_protocol::NpcSkillCorruptionHitPrefix0104 {
                npc_id: NPC, skill_id: 118, style: index as i16,
                position, target_count: 1,
            }.encode_prefix();
            let mut result = vec![0u8; 40];
            for (offset, value) in [(0, 1i32), (4, 1), (12, if index == 0 {196} else {0}),
                (16, 729), (32, 0)] {
                result[offset..offset+4].copy_from_slice(&value.to_le_bytes());
            }
            result[20] = [16, 8, 4][index as usize];
            result[22..24].copy_from_slice(&0i16.to_le_bytes());
            result[28..30].copy_from_slice(&2i16.to_le_bytes());
            result[30..32].copy_from_slice(&([60i16, 60, 105][index as usize]).to_le_bytes());
            payload.extend(result);
            send(packet::P_FE2CL_NPC_SKILL_CORRUPTION_HIT, payload);
        }
        if frame==390 {
            let mut payload=vec![0u8;20];
            payload[..4].copy_from_slice(&NPC.to_le_bytes());
            payload[4..6].copy_from_slice(&178i16.to_le_bytes());
            for (i,v) in position.iter().enumerate() { payload[8+i*4..12+i*4].copy_from_slice(&v.to_le_bytes()); }
            send(packet::P_FE2CL_NPC_SKILL_READY,payload);
        }
        if frame == 490 {
            let mut payload = vec![0u8;20];
            payload[..4].copy_from_slice(&NPC.to_le_bytes());
            payload[4..6].copy_from_slice(&178i16.to_le_bytes());
            for (i, v) in position.iter().enumerate() {
                payload[8+i*4..12+i*4].copy_from_slice(&v.to_le_bytes());
            }
            send(packet::P_FE2CL_NPC_SKILL_READY, payload);
        }
        if frame == 495 { send(packet::P_FE2CL_NPC_SKILL_CANCEL, NPC.to_le_bytes().to_vec()); }
        if frame==460 {
            let mut payload = ffone_protocol::NpcSkillHitPrefix0104 {
                npc_id:NPC,skill_id:178,pack_padding:[0;2],position,skill_type:1,target_count:1,
            }.encode_prefix();
            for value in [1i32, 1, 0, 50, 679] { payload.extend(value.to_le_bytes()); }
            send(packet::P_FE2CL_NPC_SKILL_HIT, payload);
        }
        if frame == 360 {
            let mut payload = ffone_protocol::NpcSkillHitPrefix0104 {
                npc_id: NPC, skill_id: 171, pack_padding: [0;2], position,
                skill_type: 5, target_count: 1,
            }.encode_prefix();
            for value in [1i32, 1, 0, 5, 729, 105, 0, 0x80] { payload.extend(value.to_le_bytes()); }
            send(packet::P_FE2CL_NPC_SKILL_HIT, payload);
        }
        if frame == 380 {
            let mut payload = ffone_protocol::NanoSkillUseSuccessPrefix0104 {
                pc_id: 1, bullet_id: 0, pack_padding: [0], skill_id: 171,
                arg1: 0, arg2: 0, arg3: 0, nano_deactivated: 0,
                nano_id: 2, nano_stamina: 105, skill_type: 5, target_count: 1,
            }.encode_prefix();
            for value in [4i32, NPC, 0, 0, 800, 105, 0, 0x80] { payload.extend(value.to_le_bytes()); }
            send(packet::P_FE2CL_NANO_SKILL_USE_SUCC, payload);
        }
        if frame == 470 {
            send(packet::P_FE2CL_CHAR_TIME_BUFF_TIME_OUT, ffone_protocol::CharTimeBuffTimeout0104 {
                character_type: 4, character_id: NPC, condition_bit_flag: 0,
            }.encode());
        }
        if frame == 440 {
            send(packet::P_FE2CL_CHAR_TIME_BUFF_TIME_OUT, ffone_protocol::CharTimeBuffTimeout0104 {
                character_type: 1, character_id: 1, condition_bit_flag: 0,
            }.encode());
        }
        drop(send);
        if frame == 360 { assert_eq!(buffs.local_condition_bit_flag & 0x80, 0x80, "Snare drive end"); }
        if matches!(frame,45|110|165|230|285|350|375|385|410|462|475) {
            commands.spawn(Screenshot::primary_window()).observe(bevy::render::view::screenshot::save_to_disk(
                capture.output.join(format!("special-{frame}.png"))));
        }
        return;
    }
    if matches!(frame, 60 | 180 | 200 | 340) {
        send(
            packet::P_FE2CL_NPC_ATTACK_PCS,
            ffone_protocol::NpcAttackPcs0104 {
                npc_id: NPC,
                results: vec![],
            }
            .encode()
            .unwrap(),
        );
    }
    if matches!(frame, 72 | 78 | 84 | 90 | 190 | 310 | 360) {
        send(
            packet::P_FE2CL_PC_ATTACK_NPCS_SUCC,
            ffone_protocol::PcAttackNpcsSuccess0104 {
                battery_w: 0,
                results: vec![ffone_protocol::AttackResult0104 {
                    entity_type: 4,
                    id: NPC,
                    protected: 0,
                    damage: 10,
                    hp: if frame == 360 { 0 } else { 1000 - frame as i32 },
                    hit_flag: 1,
                }],
            }
            .encode()
            .unwrap(),
        );
    }
    if frame == 300 {
        // Empty authoritative RETROROCKET_SELF result body; the fixed skill
        // identity exercises Bad Max's table-selected skill1 request, including
        // recovery if the native model has no authored skill1 clip.
        send(
            packet::P_FE2CL_NPC_SKILL_HIT,
            ffone_protocol::NpcSkillHitPrefix0104 {
                npc_id: NPC,
                skill_id: 114,
                pack_padding: [0; 2],
                position: [0; 3],
                skill_type: 29,
                target_count: 0,
            }
            .encode_prefix(),
        );
    }
    if matches!(frame, 65 | 95 | 205 | 305 | 335 | 370 | 550) {
        commands.spawn(Screenshot::primary_window()).observe(
            bevy::render::view::screenshot::save_to_disk(
                capture.output.join(format!("mob-{frame}.png")),
            ),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn record(
    capture: Res<Capture>,
    mut replay: ResMut<Replay>,
    registry: Res<NetworkNpcRegistry0104>,
    owners: Query<&NetworkNpcAnimationLayers0104>,
    visuals: Query<&NetworkNpcVisual0104>,
    parents: Query<&ChildOf>,
    players: Query<(Entity, &AnimationPlayer, &AnimationGraphHandle)>,
    graphs: Res<Assets<AnimationGraph>>,
    gltfs: Res<Assets<Gltf>>,
    assets: Res<AssetServer>,
    runtime: Res<RuntimeStatus>,
    buffs: Res<SkillBuffUiModel>,
    controllers: Query<&LegacyPlayerController, With<LocalPlayer>>,
    marks: Query<&npc_skill_presentation::EruptionMark>,
    appearances: Query<&NetworkNpcAppearance0104>,
) {
    if replay.frame == 0 || replay.frame > 560 || replay.frame % 5 != 0 {
        return;
    }
    let Some(root) = registry.get(NPC) else {
        return;
    };
    let Ok(visual) = visuals.get(root) else {
        return;
    };
    let gltf = gltfs.iter().find_map(|(id, gltf)| {
        assets
            .get_path(id)
            .filter(|path| path.path() == std::path::Path::new(&visual.glb_path))
            .map(|_| gltf)
    });
    let Some(gltf) = gltf else { return };
    let mut active = Vec::new();
    for (entity, player, handle) in &players {
        let mut ancestor = entity;
        while ancestor != root {
            let Ok(parent) = parents.get(ancestor) else {
                break;
            };
            ancestor = parent.parent();
        }
        if ancestor != root {
            continue;
        }
        let Some(graph) = graphs.get(&handle.0) else {
            continue;
        };
        for (node, animation) in player.playing_animations() {
            if animation.is_paused() {
                continue;
            }
            let Some(weight) = graph.graph.node_weight(*node) else {
                continue;
            };
            let AnimationNodeType::Clip(clip) = &weight.node_type else {
                continue;
            };
            let name = gltf
                .named_animations
                .iter()
                .find(|(_, handle)| *handle == clip)
                .map(|(name, _)| name.as_ref());
            active.push(serde_json::json!({"clip":name,"seek":animation.seek_time(),
                "weight":animation.weight(),"finished":animation.is_finished(),"completions":animation.completions()}));
        }
    }
    let layers = owners.get(root).ok();
    let frame = replay.frame;
    if replay.special && matches!(frame, 110 | 230 | 350) {
        assert_eq!(runtime.hp, Some(729), "RPS HP remains server-owned between packets");
        assert_eq!(runtime.nano_slots[0].stamina, if frame == 350 { 105 } else { 60 });
    }
    if replay.special && frame == 350 {
        assert_eq!(appearances.get(root).unwrap().0.hp, 800, "reflected Nano damage reaches the target HUD authority");
    }
    if replay.special && matches!(frame, 385 | 475) {
        assert_eq!(appearances.get(root).unwrap().0.condition_bit_flag & 0x80,
            if frame == 385 { 0x80 } else { 0 }, "Nano Snare target post-state and timeout");
    }
    if replay.special && frame == 500 { assert!(marks.is_empty(), "CANCEL must clear the eruption warning"); }
    if replay.special && frame == 450 {
        assert_eq!(buffs.local_condition_bit_flag & 0x80, 0);
        assert_eq!(controllers.single().unwrap().run_speed_server_units, 600);
    }
    if replay.special && matches!(frame, 375 | 465) {
        if frame == 375 {
            assert_eq!(buffs.local_condition_bit_flag & 0x80, 0x80);
            assert_eq!(controllers.single().unwrap().run_speed_server_units, 300);
        } else {
            assert_eq!(runtime.hp, Some(679));
            assert_eq!(buffs.local_condition_bit_flag & 0x80, 0);
            assert!(marks.is_empty(), "HIT must clear the eruption warning");
        }
    }
    replay.records.push(
        serde_json::json!({"frame":frame,"model":visual.glb_path,"active":active,
        "highOwner":layers.and_then(|layers|layers.high_owner),
        "standAttack":layers.is_some_and(|layers|layers.stand_attack),
        "hp":runtime.hp,"nanoStamina":runtime.nano_slots[0].stamina,
        "snare":buffs.local_condition_bit_flag & 0x80 != 0,
        "speed":controllers.iter().next().map(|c|c.run_speed_server_units),
        "eruptionMarks":marks.iter().count(),
        "npcHp":appearances.get(root).ok().map(|npc|npc.0.hp),
        "npcSnare":appearances.get(root).is_ok_and(|npc|npc.0.condition_bit_flag & 0x80 != 0)}),
    );
    if frame == 560 {
        fs::write(
            capture.output.join("mob-animation-replay.json"),
            serde_json::to_vec_pretty(&replay.records).unwrap(),
        )
        .unwrap();
    }
}
