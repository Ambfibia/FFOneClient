//! Isolated shard acceptance using the production client and ordinary controls.
use super::*;
use ffone_client::entity_lifecycle::{
    NetworkNpcResultEffectEvents0104, NetworkNpcSkillEffectEvents0104,
};

#[derive(Resource)]
struct Probe {
    output: PathBuf,
    entered: bool,
    last: Instant,
    records: Vec<serde_json::Value>,
    phase: u8,
    wait: Instant,
    reactions: usize,
}

pub(super) fn install(app: &mut App, output: PathBuf) {
    assert_eq!(env::var("FFONE_ISOLATED_TEST_DATABASE").as_deref(), Ok("1"));
    fs::create_dir_all(&output).unwrap();
    app.insert_resource(Probe {
        output,
        entered: false,
        last: Instant::now(),
        records: Vec::new(),
        phase: 0,
        wait: Instant::now(),
        reactions: 0,
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
    .add_systems(
        Update,
        observe
            .after(consume_network_entity_lifecycle_0104)
            .before(npc_skill_presentation::present_npc_skills)
            .before(world_skill_effects::spawn_world_instant_skill_effects),
    )
    .add_systems(Last, drive);
}

fn observe(
    mut probe: ResMut<Probe>,
    casts: Res<NetworkNpcSkillEffectEvents0104>,
    results: Res<NetworkNpcResultEffectEvents0104>,
    bridge: Res<NetworkBridge>,
    mut runtime: ResMut<RuntimeStatus>,
) {
    for cast in &casts.0 {
        probe
            .records
            .push(serde_json::json!({"cast":format!("{:?}",cast.signal),
            "style":cast.style,"position":cast.position}));
        if cast.signal.kind == ffone_protocol::NpcSkillSignalKind0104::CorruptionReady
            && probe.reactions > 0
            && let Some(style) = cast.style
        {
            let desired = if probe.reactions == 1 {
                style as usize
            } else {
                [2, 0, 1][style as usize]
            };
            runtime.pending_nano_activation = Some(desired);
            bridge
                .send(NetworkCommand::ActivateNano(
                    ffone_protocol::NanoActiveRequest0104 {
                        nano_slot: desired as i16,
                    },
                ))
                .unwrap();
        }
    }
    for result in &results.0 {
        probe
            .records
            .push(serde_json::json!({"result":format!("{result:?}")}));
        if matches!(
            result.kind,
            ffone_client::world_npc_skill_authority::WorldNpcSkillCastKind0104::Corruption { .. }
        ) {
            probe.reactions += 1;
        }
    }
}

fn drive(world: &mut World) {
    let state = *world.resource::<State<ClientState>>().get();
    if state == ClientState::CharacterSelect
        && !world.resource::<GameplayLoadingState>().visible
        && !world.resource::<Probe>().entered
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
    }
    if state == ClientState::World && !world.resource::<GameplayLoadingState>().visible {
        let phase = world.resource::<Probe>().phase;
        let infected = world
            .resource::<SkillBuffUiModel>()
            .local_condition_bit_flag
            & (1 << 16)
            != 0;
        if phase == 0
            && infected
            && world
                .resource::<RuntimeStatus>()
                .hp
                .is_some_and(|hp| hp < world.resource::<RuntimeStatus>().max_hp)
        {
            use ffone_protocol::WirePayload;
            world
                .resource::<NetworkBridge>()
                .send(NetworkCommand::SendRegisteredGameplay0104(
                    ffone_protocol::RegisteredGameplayRequest0104::new(
                        packet::P_CL2FE_REQ_PC_GOTO,
                        ffone_protocol::wire_0104::PcGotoRequest0104 {
                            to_x: 632032,
                            to_y: 187177,
                            to_z: -5500,
                        }
                        .encode(),
                    )
                    .unwrap(),
                ))
                .unwrap();
            world.resource_mut::<Probe>().phase = 1;
            world.resource_mut::<Probe>().wait = Instant::now();
        } else if phase == 1
            && !infected
            && world.resource::<Probe>().wait.elapsed() > Duration::from_secs(4)
        {
            use ffone_protocol::WirePayload;
            world
                .resource_mut::<RuntimeStatus>()
                .pending_nano_activation = Some(0);
            world
                .resource::<NetworkBridge>()
                .send(NetworkCommand::ActivateNano(
                    ffone_protocol::NanoActiveRequest0104 { nano_slot: 0 },
                ))
                .unwrap();
            world
                .resource::<NetworkBridge>()
                .send(NetworkCommand::SendRegisteredGameplay0104(
                    ffone_protocol::RegisteredGameplayRequest0104::new(
                        0x13000045,
                        ffone_protocol::wire_0104::NpcSummonRequest0104 {
                            npc_type: 75,
                            npc_cnt: 1,
                        }
                        .encode(),
                    )
                    .unwrap(),
                ))
                .unwrap();
            world.resource_mut::<Probe>().phase = 2;
        } else if phase == 2 {
            let npc_id = world
                .query::<&NetworkNpcAppearance0104>()
                .iter(world)
                .find(|npc| npc.0.npc_type == 75)
                .map(|npc| npc.0.npc_id);
            if let Some(npc_id) = npc_id {
                world
                    .resource::<NetworkBridge>()
                    .send(NetworkCommand::AttackNpcs(
                        ffone_protocol::PcAttackNpcsRequest0104 {
                            npc_ids: vec![npc_id],
                        },
                    ))
                    .unwrap();
                world.resource_mut::<Probe>().phase = 3;
            }
        }
    }
    if world.resource::<Probe>().last.elapsed() < Duration::from_secs(1) {
        return;
    }
    let status = world.resource::<RuntimeStatus>();
    let record = serde_json::json!({"state":format!("{state:?}"),"hp":status.hp,
        "message":status.message,"nanoSlots":format!("{:?}",status.nano_slots),
        "condition":world.resource::<SkillBuffUiModel>().local_condition_bit_flag});
    let voices: Vec<_> = world
        .query::<&Name>()
        .iter(world)
        .filter(|name| name.as_str().contains("NanCorr") || name.as_str().contains("eruption"))
        .map(|name| name.as_str().to_owned())
        .collect();
    let mut probe = world.resource_mut::<Probe>();
    probe.last = Instant::now();
    probe
        .records
        .push(serde_json::json!({"status":record,"presentation":voices}));
    fs::write(
        probe.output.join("events.json"),
        serde_json::to_vec_pretty(&probe.records).unwrap(),
    )
    .unwrap();
}
