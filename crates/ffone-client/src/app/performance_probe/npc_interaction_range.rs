//! Bug 47: production world talk/quest routing with offline authority.
use super::*;
use ffone_client::{
    avatar_action::{LegacyAvatarActionContext, LegacyAvatarActionInput, LegacyAvatarTargetFeed},
    entity_lifecycle::{NetworkNpcRegistry0104, NetworkSessionEpoch0104},
    world_targeting::world_npc_target_sample,
};

const NPC_ID: i32 = 1_900_047;

#[derive(Default, Resource)]
struct Replay {
    frame: u32,
}

pub(super) fn install(app: &mut App) {
    app.init_resource::<Replay>()
        .add_systems(Update, discard_offline_events.before(poll_network))
        .add_systems(
            Update,
            click
                .after(produce_world_avatar_target_feed)
                .after(read_configured_avatar_action_input)
                .before(LegacyAvatarActionSet::Resolve),
        )
        .add_systems(Last, drive.before(measure));
}

fn discard_offline_events(bridge: Res<NetworkBridge>) {
    let _ = bridge.drain();
}

fn click(
    replay: Res<Replay>,
    content: Res<TutorialMissionContent>,
    registry: Res<NetworkNpcRegistry0104>,
    mut input: ResMut<LegacyAvatarActionInput>,
    mut players: Query<
        (
            &Transform,
            &LegacyAvatarActionContext,
            &mut LegacyAvatarTargetFeed,
        ),
        With<LocalPlayer>,
    >,
    npcs: Query<(&Transform, &NetworkNpcAppearance0104), Without<LocalPlayer>>,
) {
    if !matches!(replay.frame, 40 | 120) {
        return;
    }
    let entity = registry.get(NPC_ID).expect("fixture NPC");
    let (npc_transform, appearance) = npcs.get(entity).unwrap();
    let (player, context, mut feed) = players.single_mut().unwrap();
    let sample = world_npc_target_sample(
        entity,
        appearance,
        content.gameplay_npc(1088).unwrap(),
        npc_transform,
        player,
        Vec3::NEG_Z,
        context,
        |_, _| false,
    )
    .unwrap();
    assert_eq!(sample.talk_enabled, replay.frame == 120);
    feed.source_connected = true;
    feed.samples = vec![sample];
    feed.trigger = None;
    input.primary_just_pressed = true;
}

fn drive(world: &mut World) {
    if world.resource::<Capture>().samples.is_empty()
        || world.resource::<GameplayLoadingState>().visible
    {
        return;
    }
    let frame = {
        let mut replay = world.resource_mut::<Replay>();
        replay.frame += 1;
        replay.frame
    };
    match frame {
        1 => {
            let load =
                ffone_protocol::PcLoadData0104::decode(&vec![
                    0;
                    ffone_protocol::PcLoadData0104::SIZE
                ])
                .unwrap();
            world.resource_mut::<LocalInventoryRuntime>().seed(1, &load);
            let player = world
                .query_filtered::<&Transform, With<LocalPlayer>>()
                .single(world)
                .unwrap()
                .translation;
            let packet = ffone_protocol::NpcEnter0104 {
                appearance: ffone_protocol::NpcAppearance0104 {
                    npc_id: NPC_ID,
                    npc_type: 1088,
                    hp: 1000,
                    condition_bit_flag: 0,
                    position: ProtocolPosition::from_native(player + Vec3::NEG_Z * 12.0).raw(),
                    angle: 0,
                    barker_type: 0,
                },
            };
            let epoch = NetworkSessionEpoch0104(1);
            let mut ingress = world.resource_mut::<NetworkEntityLifecycleIngress0104>();
            ingress.begin_session(epoch, 1);
            ingress.push_frame(
                epoch,
                DecodedFrame {
                    packet_type: packet::P_FE2CL_NPC_ENTER,
                    flags: 0,
                    checksum: 0,
                    payload: packet.encode(),
                },
            );
            world.resource_scope(|world, mut runtime: Mut<WorldMissionRuntime>| {
                runtime
                    .apply_event(
                        &WorldMissionServerEvent0104::TaskStartSuccess(
                            ffone_protocol::PcTaskStartSuccess0104 {
                                task_id: 598,
                                remaining_time: 0,
                            },
                        ),
                        world.resource::<TutorialMissionContent>(),
                    )
                    .unwrap();
            });
        }
        42 => assert!(
            world.resource::<MissionUiModel>().npc_interaction.is_none(),
            "far click opened NPC UI"
        ),
        100 | 180 => {
            let player = world
                .query_filtered::<&Transform, With<LocalPlayer>>()
                .single(world)
                .unwrap()
                .translation;
            let entity = world
                .resource::<NetworkNpcRegistry0104>()
                .get(NPC_ID)
                .unwrap();
            let distance = if frame == 100 { 5.0 } else { 12.0 };
            let transform = Transform::from_translation(player + Vec3::NEG_Z * distance);
            world
                .entity_mut(entity)
                .insert((transform, GlobalTransform::from(transform)));
            if frame == 180 {
                world.resource_scope(|world, mut model: Mut<MissionUiModel>| {
                    assert!(
                        model.select_npc_mission(0, &mut world.resource_mut::<GameplayUiOutbox>())
                    );
                });
            }
        }
        122 => {
            let model = world.resource::<MissionUiModel>();
            let npc = model
                .npc_interaction
                .as_ref()
                .expect("near click must open NPC UI");
            assert_eq!(npc.npc_id, NPC_ID);
            assert!(
                npc.completed_missions
                    .iter()
                    .any(|mission| mission.task_id == 598)
            );
        }
        150 => {
            let output = world.resource::<Capture>().output.join("near-dialogue.png");
            world
                .spawn(Screenshot::primary_window())
                .observe(bevy::render::view::screenshot::save_to_disk(output));
        }
        182 => {
            let model = world.resource::<MissionUiModel>();
            assert!(
                model.npc_interaction.is_none() && model.pending.is_none(),
                "stale quest dialog remained open"
            );
            assert!(
                world
                    .resource::<RuntimeStatus>()
                    .message
                    .contains("out-of-range NPC")
            );
            eprintln!(
                "Bug 47 PASS: task 598 / NPC 1088; far click blocked, near dialog opened, stale confirmation closed (offline authority)"
            );
        }
        220 => {
            world
                .resource_mut::<Messages<AppExit>>()
                .write(AppExit::Success);
        }
        _ => {}
    }
}
