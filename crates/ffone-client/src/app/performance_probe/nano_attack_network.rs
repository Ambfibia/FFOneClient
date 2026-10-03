//! T10: production renderer/input/packet application against an isolated RustyFusion.
//! Prepare the GM test character with equipped Nano 7/skill 19 and Nano 6/skill 21.
use super::*;
use bevy::{animation::graph::AnimationNodeType, gltf::Gltf};
use ffone_client::{
    avatar_action::{
        LegacyAvatarActionInput, LegacyAvatarActionSet, LegacyAvatarActionState,
        LegacyFocusedTarget, LegacyTargetKind,
    },
    entity_lifecycle::NetworkNpcAppearance0104,
    tutorial_nano_gameplay::TutorialNanoGameplayState,
    world_nano_cooldown::WorldNanoCooldownRuntime,
};
use ffone_protocol::{
    NanoActiveRequest0104, RegisteredGameplayRequest0104,
    wire_0104::{GmPcSpecialStateSwitchRequest0104, NpcSummonRequest0104, NpcUnsummonRequest0104},
};

#[derive(Resource)]
struct Probe {
    output: PathBuf,
    started: Instant,
    phase: u8,
    branch: usize,
    wait: Instant,
    fire: bool,
    target: Option<Entity>,
    ids: Vec<(Entity, i32)>,
    stamina_before: i16,
    records: Vec<serde_json::Value>,
    animations: BTreeSet<String>,
}

pub(super) fn install(app: &mut App, output: PathBuf) {
    assert_eq!(env::var("FFONE_ISOLATED_TEST_DATABASE").as_deref(), Ok("1"));
    fs::create_dir_all(&output).unwrap();
    app.insert_resource(Probe {
        output,
        started: Instant::now(),
        wait: Instant::now(),
        phase: 0,
        branch: 0,
        fire: false,
        target: None,
        ids: Vec::new(),
        stamina_before: 0,
        records: Vec::new(),
        animations: BTreeSet::new(),
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
        inject_input
            .after(LegacyAvatarActionSet::ReadInput)
                .after(super::super::hotkeys::read_configured_avatar_action_input)
            .before(LegacyAvatarActionSet::Resolve),
    )
    .add_systems(
        Update,
        focus
            .after(LegacyAvatarActionSet::Resolve)
            .before(super::super::world_npc_interaction::collect_world_npc_interactions),
    )
    .add_systems(Last, drive);
}

fn inject_input(probe: Res<Probe>, mut input: ResMut<LegacyAvatarActionInput>) {
    input.nano_just_pressed = probe.fire;
}

// Select a real server-spawned actor deterministically, leaving geometry, target
// planning, input gates, dispatch, cooldown and damage application to production.
fn focus(probe: Res<Probe>, mut actors: Query<&mut LegacyAvatarActionState, With<LocalPlayer>>) {
    if probe.fire {
        for mut state in &mut actors {
            state.target_selection.focused_npc = probe.target.map(|entity| LegacyFocusedTarget {
                entity,
                kind: LegacyTargetKind::Npc { team: 2 },
                distance: 0.0,
                talk_enabled: false,
            });
        }
    }
}

fn drive(world: &mut World) {
    let mut p = world.remove_resource::<Probe>().unwrap();
    assert!(
        p.started.elapsed() < Duration::from_secs(180),
        "Nano network probe timed out in phase {}: {}",
        p.phase,
        world.resource::<RuntimeStatus>().message
    );
    let state = *world.resource::<State<ClientState>>().get();
    let loading = world.resource::<GameplayLoadingState>().visible;
    let old_phase = p.phase;
    if p.phase == 0 && state == ClientState::CharacterSelect && !loading {
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
        p.phase = 1;
    }
    if state == ClientState::World && !loading {
        match p.phase {
            1 => {
                if p.branch == 0 {
                    world
                        .resource::<NetworkBridge>()
                        .send(NetworkCommand::SendRegisteredGameplay0104(
                            RegisteredGameplayRequest0104::new(
                                0x1300_006a,
                                GmPcSpecialStateSwitchRequest0104 {
                                    pc_id: world.resource::<RuntimeStatus>().player_id.unwrap(),
                                    special_state_flag: 4,
                                }
                                .encode(),
                            )
                            .unwrap(),
                        ))
                        .unwrap();
                }
                let slots = &world.resource::<RuntimeStatus>().nano_slots;
                assert_eq!((slots[0].nano_id, slots[0].skill_id), (Some(7), 19));
                assert_eq!((slots[1].nano_id, slots[1].skill_id), (Some(6), 21));
                world
                    .resource_mut::<RuntimeStatus>()
                    .pending_nano_activation = Some(p.branch);
                world
                    .resource::<NetworkBridge>()
                    .send(NetworkCommand::ActivateNano(NanoActiveRequest0104 {
                        nano_slot: p.branch as i16,
                    }))
                    .unwrap();
                p.phase = 2;
                p.wait = Instant::now();
            }
            2 if world.resource::<RuntimeStatus>().nano_slots[p.branch].active
                && p.wait.elapsed() > Duration::from_secs(4) =>
            {
                world
                    .resource::<NetworkBridge>()
                    .send(NetworkCommand::SendRegisteredGameplay0104(
                        RegisteredGameplayRequest0104::new(
                            0x1300_0045,
                            NpcSummonRequest0104 {
                                npc_type: 59,
                                npc_cnt: if p.branch == 0 { 1 } else { 3 },
                            }
                            .encode(),
                        )
                        .unwrap(),
                    ))
                    .unwrap();
                p.phase = 3;
                p.wait = Instant::now();
            }
            3 if p.wait.elapsed() > Duration::from_millis(500) => {
                let needed = if p.branch == 0 { 1 } else { 3 };
                let mut candidates = world
                    .query::<(Entity, &NetworkNpcAppearance0104)>()
                    .iter(world)
                    .filter(|(_, a)| a.0.npc_type == 59 && a.0.hp == 350)
                    .map(|(e, a)| (e, a.0.npc_id))
                    .collect::<Vec<_>>();
                candidates.sort_by_key(|(_, id)| *id);
                if candidates.len() >= needed {
                    p.ids = candidates.into_iter().rev().take(needed).collect();
                    p.target = Some(p.ids[0].0);
                    p.stamina_before =
                        world.resource::<RuntimeStatus>().nano_slots[p.branch].stamina;
                    p.fire = true;
                    p.phase = 4;
                    p.wait = Instant::now();
                }
            }
            4 => {
                p.fire = false;
                let hp = p
                    .ids
                    .iter()
                    .map(|(e, _)| world.get::<NetworkNpcAppearance0104>(*e).unwrap().0.hp)
                    .collect::<Vec<_>>();
                let expected = if p.branch == 0 { 150 } else { 220 };
                if hp.iter().all(|v| *v == expected) {
                    let slot = world.resource::<RuntimeStatus>().nano_slots[p.branch];
                    let cost = if p.branch == 0 { 26 } else { 39 };
                    assert!((cost..=cost + 1).contains(&(p.stamina_before - slot.stamina)));
                    assert!(
                        world
                            .resource::<WorldNanoCooldownRuntime>()
                            .is_active(p.branch, slot.skill_id)
                    );
                    println!(
                        "T10 native branch={} server HP={hp:?} stamina={} -> {}",
                        p.branch, p.stamina_before, slot.stamina
                    );
                    p.records.push(serde_json::json!({"branch":p.branch,"targets":p.ids.iter().map(|(_,id)|id).collect::<Vec<_>>(),"hp":hp,"staminaBefore":p.stamina_before,"staminaAfter":slot.stamina}));
                    world.spawn(Screenshot::primary_window()).observe(
                        bevy::render::view::screenshot::save_to_disk(
                            p.output.join(format!("attack-{}.png", p.branch)),
                        ),
                    );
                    p.phase = 5;
                    p.wait = Instant::now();
                }
            }
            5 if p.wait.elapsed() > Duration::from_secs(16) => {
                let slot = world.resource::<RuntimeStatus>().nano_slots[p.branch];
                assert!(
                    !world
                        .resource::<WorldNanoCooldownRuntime>()
                        .is_active(p.branch, slot.skill_id)
                );
                assert!(
                    p.animations.iter().any(|name| name.contains("skill")),
                    "Nano never played a skill animation: {:?}",
                    p.animations
                );
                p.records.push(serde_json::json!({"branch":p.branch,"playedAnimations":p.animations,"cooldownReleased":true}));
                p.animations.clear();
                if p.branch == 0 {
                    for (_, id) in &p.ids {
                        world
                            .resource::<NetworkBridge>()
                            .send(NetworkCommand::SendRegisteredGameplay0104(
                                RegisteredGameplayRequest0104::new(
                                    0x1300_0046,
                                    NpcUnsummonRequest0104 { npc_id: *id }.encode(),
                                )
                                .unwrap(),
                            ))
                            .unwrap();
                    }
                    p.branch = 1;
                    p.phase = 1;
                } else {
                    fs::write(
                        p.output.join("passed.json"),
                        serde_json::to_vec_pretty(&p.records).unwrap(),
                    )
                    .unwrap();
                    world.write_message(AppExit::Success);
                    p.phase = 6;
                }
            }
            _ => {}
        }
    }
    if matches!(p.phase, 4 | 5) {
        record_animations(world, &mut p.animations);
    }
    if p.phase != old_phase {
        println!(
            "T10 phase={} branch={} {}",
            p.phase,
            p.branch,
            world.resource::<RuntimeStatus>().message
        );
    }
    world.insert_resource(p);
}

fn record_animations(world: &mut World, seen: &mut BTreeSet<String>) {
    let Some(root) = world.resource::<TutorialNanoGameplayState>().entity() else {
        return;
    };
    let mut query = world.query::<(Entity, &AnimationPlayer, &AnimationGraphHandle)>();
    let graphs = world.resource::<Assets<AnimationGraph>>();
    let gltfs = world.resource::<Assets<Gltf>>();
    for (entity, player, graph) in query.iter(world) {
        let mut ancestor = entity;
        while ancestor != root {
            let Some(parent) = world.get::<ChildOf>(ancestor) else {
                break;
            };
            ancestor = parent.parent();
        }
        if ancestor != root {
            continue;
        }
        let Some(graph) = graphs.get(&graph.0) else {
            continue;
        };
        for (node, animation) in player.playing_animations() {
            if animation.is_paused() || animation.is_finished() {
                continue;
            }
            let Some(weight) = graph.graph.node_weight(*node) else {
                continue;
            };
            let AnimationNodeType::Clip(clip) = &weight.node_type else {
                continue;
            };
            for (_, gltf) in gltfs.iter() {
                for (name, handle) in &gltf.named_animations {
                    if handle == clip {
                        seen.insert(name.to_string());
                    }
                }
            }
        }
    }
}
