//! Matched full-client face captures across the production equipment/vehicle path.
use super::*;
use ffone_client::tutorial_player_rig_runtime::{
    PersonalVehiclePresentation, TutorialPlayerWeaponAttachment,
};

const STAGES: [&str; 10] = [
    "initial",
    "weapon-328",
    "weapon-43",
    "unarmed",
    "board",
    "dismounted",
    "shirt",
    "shirt-hat",
    "hat-removed",
    "shirt-removed",
];

#[derive(Resource, Default)]
struct FaceProbe {
    stage: usize,
    angle: usize,
    frames: usize,
    initialized: bool,
    pending: bool,
}

pub(super) fn install(app: &mut App) {
    println!("BUG025 offline authority fixture: outgoing movement/gameplay disabled");
    app.add_systems(Update, discard_offline_network_events.before(poll_network));
    app.init_resource::<FaceProbe>()
        .add_systems(Last, drive.before(super::measure))
        .add_systems(
            Update,
            discard_offline_intents
                .after(LegacyMovementSet::Simulate)
                .after(NativeWorldSet::ResolveCollision)
                .before(flush_movement_intents)
                .before(flush_world_gameplay_intents),
        );
}

fn seed_equipment(world: &mut World, stage: usize) {
    let mut bytes = vec![0; ffone_protocol::PcLoadData0104::SIZE];
    // Match the initially spawned character's empty apparel slots. Positive
    // rows go through the same hidden-candidate refresh as real equip replies.
    for (slot, id) in [
        (1usize, if (6..=8).contains(&stage) { 1i16 } else { 0 }),
        (4, if stage == 7 { 1i16 } else { 0 }),
    ] {
        if id == 0 {
            continue;
        }
        let offset = ffone_protocol::PcLoadData0104::EQUIPMENT_OFFSET + slot * ItemBase0104::SIZE;
        bytes[offset..offset + 2].copy_from_slice(&(slot as i16).to_le_bytes());
        bytes[offset + 2..offset + 4].copy_from_slice(&id.to_le_bytes());
        bytes[offset + 4..offset + 8].copy_from_slice(&1_i32.to_le_bytes());
    }
    let weapon = match stage {
        1 => Some(
            ffone_client::tutorial_player_presentation::tutorial_weapon_request(true, 1)
                .unwrap()
                .item,
        ),
        2 => Some(
            ffone_client::tutorial_player_presentation::tutorial_weapon_request(false, 2)
                .unwrap()
                .item,
        ),
        _ => None,
    };
    if let Some(item) = weapon {
        let offset = ffone_protocol::PcLoadData0104::EQUIPMENT_OFFSET;
        bytes[offset..offset + 2].copy_from_slice(&item.item_type.to_le_bytes());
        bytes[offset + 2..offset + 4].copy_from_slice(&item.item_id.to_le_bytes());
        bytes[offset + 4..offset + 8].copy_from_slice(&item.option.to_le_bytes());
    }
    let offset = ffone_protocol::PcLoadData0104::EQUIPMENT_OFFSET + 8 * ItemBase0104::SIZE;
    bytes[offset..offset + 2].copy_from_slice(&10_i16.to_le_bytes());
    bytes[offset + 2..offset + 4].copy_from_slice(&1_i16.to_le_bytes());
    bytes[offset + 4..offset + 8].copy_from_slice(&1_i32.to_le_bytes());
    world
        .resource_mut::<LocalInventoryRuntime>()
        .seed(1, &ffone_protocol::PcLoadData0104::decode(&bytes).unwrap());
    world
        .resource_mut::<LocalVehiclePresentationRuntime>()
        .family = if stage == 4 {
        LegacyVehiclePresentationFamily::Board
    } else {
        LegacyVehiclePresentationFamily::None
    };
}

fn drive(world: &mut World) {
    // Keep the ordinary benchmark from capturing/exiting during the angle sweep.
    let ready = {
        let mut capture = world.resource_mut::<Capture>();
        capture.samples.clear();
        capture.ready.is_some()
    };
    if !ready {
        return;
    }
    world.resource_scope(|world, mut probe: Mut<FaceProbe>| {
        if probe.pending {
            return;
        }
        if probe.stage == STAGES.len() {
            println!(
                "BUG025 FACE SWEEP PASS: 10 states x 12 matched angles; offline authority fixture"
            );
            world.write_message(AppExit::Success);
            return;
        }
        if !probe.initialized {
            println!(
                "BUG025 seed stage={} state={:?}",
                probe.stage,
                world.resource::<State<ClientState>>().get()
            );
            seed_equipment(world, probe.stage);
            println!(
                "BUG025 seeded inventory={:?}",
                world
                    .resource::<LocalInventoryRuntime>()
                    .snapshot()
                    .map(|s| s.equipment()[0])
            );
            probe.initialized = true;
            probe.frames = 0;
        }
        for mut camera in world.query::<&mut LegacyOrbitCamera>().iter_mut(world) {
            camera.yaw_degrees = probe.angle as f32 * 30.0;
            camera.pitch_degrees = 5.0;
            camera.distance = 4.0;
        }
        probe.frames += 1;
        if probe.frames < 90 {
            return;
        }
        assert_eq!(
            *world.resource::<State<ClientState>>().get(),
            ClientState::World,
            "fixture left world"
        );
        let projection = world.resource::<WorldPlayerEquipmentProjection>();
        assert!(
            projection.rejected_apparel.is_none(),
            "apparel rejected at {}",
            STAGES[probe.stage]
        );
        if projection.candidate.is_some() {
            return;
        }
        let weapon_ids: Vec<_> = world
            .query::<&TutorialPlayerWeaponAttachment>()
            .iter(world)
            .map(|w| w.item_id)
            .collect();
        let expected_weapon = match probe.stage {
            1 => Some(328),
            2 => Some(43),
            _ => None,
        };
        assert_eq!(
            weapon_ids.first().copied(),
            expected_weapon,
            "weapon stage {} projection={:?} inventory={:?}",
            probe.stage,
            world.resource::<WorldPlayerEquipmentProjection>(),
            world
                .resource::<LocalInventoryRuntime>()
                .snapshot()
                .map(|s| s.equipment()[0])
        );
        let vehicle = world.resource::<PersonalVehiclePresentation>();
        assert_eq!(
            vehicle.item_id.is_some(),
            probe.stage == 4,
            "vehicle stage {}",
            probe.stage
        );
        let clip = world
            .query::<&TutorialPlayerAnimationApplied>()
            .iter(world)
            .next()
            .expect("player pose")
            .clip;
        assert_eq!(
            clip.name().starts_with("board_"),
            probe.stage == 4,
            "pose stage {}: {}",
            probe.stage,
            clip.name()
        );
        let output = world.resource::<Capture>().output.join(format!(
            "{}-{:03}.png",
            STAGES[probe.stage],
            probe.angle * 30
        ));
        println!("BUG025 capture {} clip={}", output.display(), clip.name());
        probe.pending = true;
        world.spawn(Screenshot::primary_window()).observe(
            move |event: On<ScreenshotCaptured>, mut probe: ResMut<FaceProbe>| {
                event
                    .image
                    .clone()
                    .try_into_dynamic()
                    .unwrap()
                    .save(&output)
                    .unwrap();
                probe.pending = false;
                probe.angle += 1;
                probe.frames = 0;
                if probe.angle == 12 {
                    probe.angle = 0;
                    probe.stage += 1;
                    probe.initialized = false;
                }
            },
        );
    });
}

// The fixture has no shard. Sending idle movement would emit an Error boundary
// every frame and clear the authoritative inventory before its projection.
fn discard_offline_intents(
    mut movement: ResMut<ffone_client::movement::MovementIntentQueue>,
    mut gameplay: ResMut<ffone_client::world_behaviour::WorldGameplayIntentQueue>,
) {
    movement.clear();
    let _ = gameplay.take_all();
}

// Offline fixtures do not own a shard; unsolicited worker errors must not
// execute a live session teardown against their synthetic inventory.
fn discard_offline_network_events(
    bridge: Res<ffone_client::network::NetworkBridge>,
    mut logged: Local<bool>,
) {
    for event in bridge.drain() {
        match event {
            ffone_client::network::NetworkEvent::Error(error) => {
                if !*logged {
                    println!("BUG025 offline worker error ignored: {error}");
                    *logged = true;
                }
            }
            other => panic!("offline face fixture received a live network event: {other:?}"),
        }
    }
}
