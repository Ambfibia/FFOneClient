//! Opt-in Bug 22 acceptance using production actor spawning, grounding and rendering.
use super::*;
use ffone_client::network_world_runtime::NetworkNpcVisualMaterialReady0104;
use ffone_client::tutorial_actors::{TutorialActorCommandQueue, TutorialActorRegistry};
use ffone_client::tutorial_logic::{LegacySpawnPosition, TutorialNpcSpawn};

const ACTORS: [(i32, i32); 2] = [(1_900_022, 2695), (1_900_023, 2674)];

#[derive(Resource, Default)]
struct Replay {
    round: usize,
    frame: u32,
    positions: Vec<Vec3>,
    samples: Vec<serde_json::Value>,
    complete: bool,
}

pub(super) fn install(app: &mut App) {
    app.init_resource::<Replay>()
        .add_systems(Last, record.before(super::measure));
}

fn record(world: &mut World) {
    // The normal tutorial starts a cinematic. Keep its production window
    // camera on the two fixture actors so the final capture verifies pixels.
    let positions = &world.resource::<Replay>().positions;
    if positions.len() == 2 {
        let center = (positions[0] + positions[1]) * 0.5 + Vec3::Y * 2.0;
        let pose = Transform::from_translation(center + Vec3::new(0.0, 6.0, 18.0))
            .looking_at(center, Vec3::Y);
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
    }
    if world.resource::<Capture>().samples.len() >= 590 {
        assert!(
            world.resource::<Replay>().complete,
            "Bug 22 actor replay did not finish"
        );
    }
    if world.resource::<Replay>().complete || world.resource::<Capture>().ready.is_none() {
        return;
    }
    if world.resource::<Replay>().frame == 0 {
        if world.resource::<Replay>().positions.is_empty() {
            let player = world
                .query_filtered::<&Transform, With<LocalPlayer>>()
                .single(world)
                .expect("fixture player")
                .translation;
            world.resource_mut::<Replay>().positions = vec![
                player + Vec3::new(5.0, 0.0, 4.0),
                player + Vec3::new(-5.0, 0.0, 4.0),
            ];
        }
        let positions = world.resource::<Replay>().positions.clone();
        let mut queue = world.resource_mut::<TutorialActorCommandQueue>();
        for ((id, npc_type), position) in ACTORS.into_iter().zip(positions) {
            let [x, y, z] = ProtocolPosition::from_native(position).raw();
            queue.spawn(TutorialNpcSpawn::new(
                id,
                npc_type,
                LegacySpawnPosition::centiunits(x, y, z),
                None,
            ));
        }
    }
    world.resource_mut::<Replay>().frame += 1;
    if world.resource::<Replay>().frame < 180 {
        return;
    }
    let mut sample = Vec::new();
    let positions = world.resource::<Replay>().positions.clone();
    for ((id, npc_type), initial) in ACTORS.into_iter().zip(positions) {
        let entity = world
            .resource::<TutorialActorRegistry>()
            .entity(id)
            .expect("live actor");
        if world
            .get::<NetworkNpcVisualMaterialReady0104>(entity)
            .is_none()
        {
            if world.resource::<Replay>().frame % 180 == 0 {
                let visual =
                    world.get::<ffone_client::network_world_runtime::NetworkNpcVisual0104>(entity);
                println!("Bug 22 pending NPC {id}: state={:?}, visual={}, scene={:?}, issue={:?}",
                    world.resource::<State<ClientState>>().get(), visual.is_some(),
                    visual.and_then(|v| world.get::<ffone_client::character_scene::LegacyCharacterSceneStatus>(v.scene)),
                    world.get::<ffone_client::network_world_runtime::NetworkNpcMaterialVisibilityBlocked0104>(entity));
            }
            return;
        }
        let position = world.get::<Transform>(entity).unwrap().translation;
        let terrain_y = world
            .query::<(
                &GlobalTransform,
                &ffone_client::native_terrain::NativeHeightmapCollider,
            )>()
            .iter(world)
            .filter_map(|(global, collider)| {
                collider.ground_height(global, position.x, position.z, -1_000_000.0, 1_000_000.0)
            })
            .max_by(f32::total_cmp)
            .expect("resident terrain under fixture actor");
        assert!(
            position.y >= terrain_y - 0.2,
            "NPC {id} fell below the local terrain: {position:?}"
        );
        sample.push(serde_json::json!({"id":id,"npcType":npc_type,
            "initial":initial.to_array(),"position":position.to_array(),
            "terrainY":terrain_y,"materialReady":true}));
    }
    let mut replay = world.resource_mut::<Replay>();
    replay.samples.push(serde_json::json!(sample));
    replay.round += 1;
    if replay.round == 1 {
        replay.frame = 0;
        let mut queue = world.resource_mut::<TutorialActorCommandQueue>();
        for (id, _) in ACTORS {
            queue.delete(id);
        }
    } else {
        replay.complete = true;
        let bytes = serde_json::to_vec_pretty(&replay.samples).unwrap();
        fs::write(
            world.resource::<Capture>().output.join("npc-floor.json"),
            bytes,
        )
        .unwrap();
        println!("Bug 22: portal and mob grounded and material-ready after both spawns");
    }
}
