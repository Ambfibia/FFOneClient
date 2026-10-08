//! Bug 19: live shard routes, table velocities and repeated world streaming.
use super::*;
use ffone_client::{
    entity_lifecycle::{NetworkNpc0104, NetworkNpcMotion0104},
    network_world_runtime::{NetworkHnpcVisual0104, NetworkNpcVisualMaterialReady0104},
    player_shared_rig::NativePlayerRigAnimationApplied,
};
use ffone_protocol::WirePayload;

const CASES: [(i32, f32, [i32; 3]); 3] = [
    (2895, 2.0, [219284, 138755, -5056]),
    (2596, 1.0, [239985, 332824, -5047]),
    (2918, 8.0, [368580, 347676, -5688]),
];

#[derive(Resource)]
struct Probe {
    output: PathBuf,
    started: Instant,
    entered: bool,
    stage: usize,
    warped: bool,
    warped_since: Instant,
    target: Option<Entity>,
    first_entities: Vec<Entity>,
    first_ids: Vec<i32>,
    first_position: Vec3,
    last_position: Vec3,
    sampling_since: Instant,
    samples: usize,
    records: Vec<serde_json::Value>,
    complete: Option<Instant>,
    reported: Instant,
}

pub(super) fn install(app: &mut App, output: PathBuf) {
    assert_eq!(env::var("FFONE_ISOLATED_TEST_DATABASE").as_deref(), Ok("1"));
    fs::create_dir_all(&output).unwrap();
    app.insert_resource(Probe {
        output,
        started: Instant::now(),
        entered: false,
        stage: 0,
        warped: false,
        warped_since: Instant::now(),
        target: None,
        first_entities: Vec::new(),
        first_ids: Vec::new(),
        first_position: Vec3::ZERO,
        last_position: Vec3::ZERO,
        sampling_since: Instant::now(),
        samples: 0,
        records: Vec::new(),
        complete: None,
        reported: Instant::now(),
    })
    .insert_resource(bevy::winit::WinitSettings::continuous())
    .add_systems(PostStartup, |bridge: Res<NetworkBridge>| {
        bridge
            .send(NetworkCommand::Login {
                login_address: env::var("FFONE_LOGIN_ADDRESS").unwrap(),
                username: env::var("FFONE_USERNAME").unwrap(),
                password: env::var("FFONE_PASSWORD").unwrap(),
            })
            .unwrap();
    })
    .add_systems(Last, drive);
}

fn capture(world: &mut World, suffix: &str) {
    let probe = world.resource::<Probe>();
    let path = probe
        .output
        .join(format!("stage-{}-{suffix}.png", probe.stage));
    world
        .spawn(Screenshot::primary_window())
        .observe(bevy::render::view::screenshot::save_to_disk(path));
}

fn drive(world: &mut World) {
    if world.resource::<Probe>().reported.elapsed() > Duration::from_secs(3) {
        let state = *world.resource::<State<ClientState>>().get();
        let loading = world.resource::<GameplayLoadingState>();
        let stage = world.resource::<Probe>().stage;
        println!(
            "civilian-routes status stage={stage} state={state:?} loading={:?}/{:?} visible={} blocked={:?}",
            loading.scope, loading.phase, loading.visible, loading.blocked
        );
        let ty = CASES[stage % CASES.len()].0;
        for (npc, motion, issue, ready, hnpc) in world
            .query::<(
                &NetworkNpc0104,
                Option<&NetworkNpcMotion0104>,
                Option<&ffone_client::network_world_runtime::NetworkNpcVisualIssue0104>,
                Option<&NetworkNpcVisualMaterialReady0104>,
                Option<&NetworkHnpcVisual0104>,
            )>()
            .iter(world)
            .filter(|(npc, _, _, _, _)| npc.npc_type == ty)
        {
            println!(
                "civilian-routes NPC={} motion={motion:?} issue={issue:?} ready={}",
                npc.npc_id,
                ready.is_some()
            );
            if let Some(hnpc) = hnpc {
                println!(
                    "civilian-routes HNPC rig={:?} visibility={:?} applied={:?} issue={:?}",
                    hnpc.rig_root,
                    world.get::<Visibility>(hnpc.rig_root),
                    world.get::<NativePlayerRigAnimationApplied>(hnpc.rig_root),
                    world.get::<ffone_client::player_shared_rig::NativePlayerRigAnimationIssue>(
                        hnpc.rig_root
                    )
                );
            }
        }
        world.resource_mut::<Probe>().reported = Instant::now();
    }
    let probe = world.resource::<Probe>();
    if let Some(done) = probe.complete {
        if done.elapsed() > Duration::from_secs(2) {
            world.write_message(AppExit::Success);
        }
        return;
    }
    assert!(
        probe.started.elapsed() < Duration::from_secs(600),
        "civilian route acceptance timed out"
    );
    let state = *world.resource::<State<ClientState>>().get();
    if state == ClientState::CharacterSelect
        && !world.resource::<GameplayLoadingState>().visible
        && !probe.entered
    {
        let uid = world.resource::<RuntimeStatus>().roster.characters[0].pc_uid;
        world
            .resource_mut::<RuntimeStatus>()
            .roster
            .pending_character_entry_uid = Some(uid);
        world
            .resource::<NetworkBridge>()
            .send(NetworkCommand::SelectCharacter {
                pc_uid: uid,
                location: CharacterEntryLocation0104::Saved,
            })
            .unwrap();
        world
            .resource_mut::<GameplayLoadingState>()
            .begin(ResourceLoadingScope::World);
        world.resource_mut::<Probe>().entered = true;
        return;
    }
    if state != ClientState::World || world.resource::<GameplayLoadingState>().visible {
        return;
    }
    let stage = world.resource::<Probe>().stage;
    let (ty, expected_speed, pos) = CASES[stage % CASES.len()];
    if !world.resource::<Probe>().warped {
        world
            .resource::<NetworkBridge>()
            .send(NetworkCommand::SendRegisteredGameplay0104(
                ffone_protocol::RegisteredGameplayRequest0104::new(
                    packet::P_CL2FE_REQ_PC_GOTO,
                    ffone_protocol::wire_0104::PcGotoRequest0104 {
                        to_x: pos[0],
                        to_y: pos[1],
                        to_z: pos[2],
                    }
                    .encode(),
                )
                .unwrap(),
            ))
            .unwrap();
        world.resource_mut::<Probe>().warped = true;
        world.resource_mut::<Probe>().warped_since = Instant::now();
        println!("civilian-routes stage={stage} type={ty} warp={pos:?}");
        return;
    }
    let target = world.resource::<Probe>().target;
    let expected_id =
        (stage >= CASES.len()).then(|| world.resource::<Probe>().first_ids[stage % CASES.len()]);
    let player_position = world
        .query_filtered::<&Transform, With<LocalPlayer>>()
        .single(world)
        .expect("live player")
        .translation;
    // Wait for the goto response and the new streamed world before binding an
    // entity. The old scene can still be visible in the frame after sending.
    let observer = Vec3::new(-pos[0] as f32 * 0.01, 0.0, pos[1] as f32 * 0.01);
    if world.resource::<Probe>().warped_since.elapsed() < Duration::from_secs(1)
        || ((player_position - observer) * Vec3::new(1.0, 0.0, 1.0)).length() > 2.0
    {
        return;
    }
    let candidate = world
        .query::<(
            Entity,
            &NetworkNpc0104,
            &Transform,
            &NetworkNpcMotion0104,
            Option<&NetworkNpcVisualMaterialReady0104>,
            Option<&NetworkHnpcVisual0104>,
        )>()
        .iter(world)
        .filter(|(entity, npc, _, _, ready, hnpc)| {
            npc.npc_type == ty
                && target.is_none_or(|t| t == *entity)
                && expected_id.is_none_or(|id| id == npc.npc_id)
                && (ready.is_some()
                    || hnpc.is_some_and(|hnpc| {
                        world
                            .get::<Visibility>(hnpc.rig_root)
                            .is_some_and(|v| *v != Visibility::Hidden)
                            && world
                                .get::<NativePlayerRigAnimationApplied>(hnpc.rig_root)
                                .is_some_and(|applied| applied.clip == "walk" && applied.repeat)
                    }))
        })
        .min_by(|a, b| {
            a.2.translation
                .distance_squared(player_position)
                .total_cmp(&b.2.translation.distance_squared(player_position))
        })
        .map(|(entity, npc, transform, motion, _, hnpc)| {
            if let Some(hnpc) = hnpc {
                let applied = world
                    .get::<NativePlayerRigAnimationApplied>(hnpc.rig_root)
                    .unwrap();
                assert_eq!(applied.clip, "walk", "the rendered HNPC must play Walk");
                assert!(applied.repeat);
            }
            (
                entity,
                npc.npc_id,
                transform.translation,
                motion.speed,
                motion.move_style,
            )
        });
    let Some((entity, npc_id, position, speed, style)) = candidate else {
        return;
    };
    assert_eq!(style, 0, "type {ty} must use Walk");
    assert!(
        (speed - expected_speed).abs() < 0.001,
        "type {ty}: speed={speed}, expected={expected_speed}"
    );
    if target.is_none() {
        let mut probe = world.resource_mut::<Probe>();
        if stage < CASES.len() {
            probe.first_entities.push(entity);
            probe.first_ids.push(npc_id);
        } else {
            assert_ne!(
                entity,
                probe.first_entities[stage % CASES.len()],
                "NPC must respawn after streaming out"
            );
        }
        probe.target = Some(entity);
        probe.first_position = position;
        probe.last_position = position;
        probe.sampling_since = Instant::now();
        probe.samples = 0;
        println!("civilian-routes stage={stage} NPC={npc_id} speed={speed} style={style}");
        capture(world, "start");
    }
    let center = position + Vec3::Y * 1.5;
    let pose =
        Transform::from_translation(center + Vec3::new(6.0, 4.0, 10.0)).looking_at(center, Vec3::Y);
    let cameras: Vec<_> = world
        .query_filtered::<(Entity, &Camera, &bevy::camera::RenderTarget), With<Camera3d>>()
        .iter(world)
        .filter(|(_, camera, target)| {
            camera.is_active && matches!(target, bevy::camera::RenderTarget::Window(_))
        })
        .map(|(entity, _, _)| entity)
        .collect();
    for camera in cameras {
        world
            .entity_mut(camera)
            .insert((pose, GlobalTransform::from(pose)));
    }
    let delta = world.resource::<Time>().delta_secs();
    let mut probe = world.resource_mut::<Probe>();
    let horizontal = (position - probe.last_position) * Vec3::new(1.0, 0.0, 1.0);
    assert!(
        horizontal.length() <= speed * delta + 0.02,
        "client exceeded the packet velocity"
    );
    probe.last_position = position;
    probe.samples += 1;
    if probe.sampling_since.elapsed() < Duration::from_secs(3) {
        return;
    }
    let distance = ((position - probe.first_position) * Vec3::new(1.0, 0.0, 1.0)).length();
    assert!(
        distance > 0.2 && probe.samples > 10,
        "type {ty} did not move in the real client"
    );
    let samples = probe.samples;
    probe
        .records
        .push(serde_json::json!({"stage":stage,"type":ty,"npcId":npc_id,
        "entity":format!("{entity:?}"),"speed":speed,"moveStyle":style,
        "samples":samples,"horizontalDisplacement":distance,"reentry":stage >= CASES.len()}));
    fs::write(
        probe.output.join("acceptance.json"),
        serde_json::to_vec_pretty(&probe.records).unwrap(),
    )
    .unwrap();
    capture(world, "end");
    let mut probe = world.resource_mut::<Probe>();
    probe.stage += 1;
    probe.warped = false;
    probe.target = None;
    if probe.stage == CASES.len() * 2 {
        probe.complete = Some(Instant::now());
        println!("civilian-routes passed all six live streaming stages");
    }
}
