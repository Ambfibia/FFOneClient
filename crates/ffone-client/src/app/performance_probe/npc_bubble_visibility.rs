//! Bug 54 replay in the production window, against a loaded authored world wall.
use super::*;
use ffone_client::localization::LocalizedText;
use ffone_client::movement::LegacyMovementSet;
use ffone_client::world::{
    AuthoredColliderWorldBounds, AuthoredTriMeshCollider,
    authored_collider_blocks_segment_with_bounds,
};

const LINE: &str = "Bug 54 visibility";

#[derive(Default, Resource)]
struct Replay {
    frame: u32,
    fixture: Option<(Entity, Vec3, Vec3, Vec3, Vec3)>, // speaker, root, anchor, wall axis, blocked camera
    samples: Vec<serde_json::Value>,
}

pub(super) fn install(app: &mut App) {
    app.init_resource::<Replay>()
        .add_systems(Update, discard_offline_events.before(poll_network))
        .add_systems(
            Update,
            pose.after(LegacyMovementSet::CameraPose)
                .after(ffone_client::network_world_runtime::advance_network_npc_motion_0104)
                .before(ffone_client::gameplay_ui::GameplayUiSet::NpcSpeech),
        )
        .add_systems(Last, record.before(measure));
}

fn discard_offline_events(bridge: Res<NetworkBridge>) {
    let _ = bridge.drain();
}

fn blocked(world: &mut World, start: Vec3, end: Vec3) -> bool {
    world
        .query::<(
            &GlobalTransform,
            &AuthoredTriMeshCollider,
            &AuthoredColliderWorldBounds,
        )>()
        .iter(world)
        .any(|(global, collider, bounds)| {
            authored_collider_blocks_segment_with_bounds(collider, global, bounds, start, end)
        })
}

fn pose(world: &mut World) {
    if world.resource::<Replay>().fixture.is_none() && world.resource::<Capture>().ready.is_none() {
        return;
    }
    if world.resource::<Replay>().fixture.is_none() {
        let owner = world
            .query::<(Entity, &NetworkNpcAppearance0104)>()
            .iter(world)
            .find(|(_, npc)| npc.0.npc_type == 2555)
            .map(|(e, _)| e)
            .expect("Computress 2555");
        let height = world
            .resource::<TutorialMissionContent>()
            .gameplay_npc(2555)
            .unwrap()
            .height_server_units as f32
            * ffone_client::movement::SERVER_TO_CLIENT_SCALE
            * 0.8;
        let mut candidates: Vec<_> = world
            .query::<(
                &GlobalTransform,
                &AuthoredTriMeshCollider,
                &AuthoredColliderWorldBounds,
            )>()
            .iter(world)
            .filter_map(|(global, collider, bounds)| {
                let (min, max) = bounds.debug_bounds();
                let size = max - min;
                let axis = if size.x < size.z { Vec3::X } else { Vec3::Z };
                let width = size.dot(axis);
                let span = size.x.max(size.z);
                if collider.is_trigger() || size.y < 3.0 || size.y > 15.0 || width > 20.0
                    || span < 6.0 || size.y > span * 0.8
                {
                    return None;
                }
                let center = (min + max) * 0.5;
                let anchor = center + axis * (width * 0.5 + 2.0);
                let camera = anchor - axis * (width + 4.0);
                authored_collider_blocks_segment_with_bounds(
                    collider, global, bounds, camera, anchor,
                )
                .then(|| {
                    (
                        anchor,
                        axis,
                        collider.source_model_path().to_owned(),
                        min,
                        max,
                        camera,
                    )
                })
            })
            .collect();
        let origin = world.resource::<Capture>().position;
        candidates.sort_by(|a, b| {
            a.0.distance_squared(origin)
                .total_cmp(&b.0.distance_squared(origin))
        });
        let (anchor, axis, source, min, max, blocked_camera) = candidates
            .into_iter()
            .find(|(anchor, axis, _, _, _, _)| !blocked(world, *anchor + *axis * 4.0, *anchor))
            .expect("loaded wall with clear speaker side");
        let root = anchor - Vec3::Y * height;
        world.resource_mut::<Replay>().fixture = Some((owner, root, anchor, axis, blocked_camera));
        let output = world.resource::<Capture>().output.join("wall.json");
        fs::write(
            output,
            serde_json::to_vec_pretty(&serde_json::json!({
                "npcType":2555, "source":source, "bounds":[min.to_array(),max.to_array()],
                "speakerRoot":root.to_array(), "anchor":anchor.to_array(), "authority":"offline"
            }))
            .unwrap(),
        )
        .unwrap();
        world
            .resource_mut::<NpcBarkerBubbleRuntime>()
            .request_quest_dialogue(
                owner,
                2555,
                LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", LINE),
            );
    }
    let (owner, root, anchor, axis, blocked_camera) = world.resource::<Replay>().fixture.unwrap();
    world.resource_mut::<Replay>().frame += 1;
    let stage = (world.resource::<Replay>().frame - 1) / 30;
    let distance = match stage {
        0 | 4 => 40.0,
        6 | 8 => 30.5,
        7 => 31.5,
        _ => 4.0,
    };
    let player = world
        .query_filtered::<Entity, With<LocalPlayer>>()
        .single(world)
        .unwrap();
    let player_pose = Transform::from_translation(root + axis * distance);
    world.resource_mut::<Capture>().position = player_pose.translation;
    world
        .entity_mut(player)
        .insert((player_pose, GlobalTransform::from(player_pose)));
    let npc_pose = Transform::from_translation(root);
    world
        .entity_mut(owner)
        .insert((npc_pose, GlobalTransform::from(npc_pose)));
    let camera_position = if stage == 2 {
        blocked_camera
    } else {
        anchor + axis * 4.0
    };
    let camera_pose = Transform::from_translation(camera_position).looking_at(anchor, Vec3::Y);
    let camera = world
        .query::<(Entity, &LegacyOrbitCamera)>()
        .iter(world)
        .find(|(_, orbit)| orbit.target == player)
        .map(|(e, _)| e)
        .unwrap();
    world
        .entity_mut(camera)
        .insert((camera_pose, GlobalTransform::from(camera_pose)));
}

fn record(world: &mut World) {
    let frame = world.resource::<Replay>().frame;
    if frame == 310 {
        world
            .resource_mut::<Messages<AppExit>>()
            .write(AppExit::Success);
        return;
    }
    if frame == 0 || (frame != 1 && frame % 30 != 20) {
        return;
    }
    let stage = (frame - 1) / 30;
    let names = [
        "initial-far",
        "near",
        "wall",
        "wall-restored",
        "far",
        "near-restored",
        "distance-band",
        "outside",
        "band-hidden",
        "returned",
    ];
    let expected = !matches!(stage, 0 | 2 | 4 | 7 | 8);
    let bubble = world
        .query::<(&LocalizedText, &ChildOf)>()
        .iter(world)
        .find(|(text, parent)| {
            text.args.get("text").is_some_and(|value| value == LINE)
                && world.get::<Node>(parent.parent()).is_some_and(|node| {
                    node.position_type == PositionType::Absolute
                        && node.min_width == px(ffone_client::gameplay_ui::NPC_BARKER_MIN_WIDTH)
                })
        })
        .map(|(_, parent)| parent.parent())
        .expect("rendered speech text");
    let displayed = world.get::<Node>(bubble).unwrap().display == Display::Flex;
    assert_eq!(displayed, expected, "Bug 54 {}", names[stage as usize]);
    world
        .resource_mut::<Replay>()
        .samples
        .push(serde_json::json!({
            "stage":names[stage as usize], "displayed":displayed, "frame":frame
        }));
    let output = world
        .resource::<Capture>()
        .output
        .join(format!("{}.png", names[stage as usize]));
    world
        .spawn(Screenshot::primary_window())
        .observe(bevy::render::view::screenshot::save_to_disk(output));
    if stage == 9 {
        let output = world
            .resource::<Capture>()
            .output
            .join("visibility-pass.json");
        fs::write(
            output,
            serde_json::to_vec_pretty(&world.resource::<Replay>().samples).unwrap(),
        )
        .unwrap();
    }
}
