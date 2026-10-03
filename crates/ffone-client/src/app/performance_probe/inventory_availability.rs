//! Bug 52: full-client UI binding with an offline authoritative inventory.
use super::*;
use ffone_client::user_equip_ui::{
    UserEquipDragState, UserEquipSlotEndpoint, UserEquipUiElement, UserEquipUiRoot,
};

#[derive(Resource, Default)]
struct Probe {
    frame: u32,
    stage: usize,
}

pub(super) fn install(app: &mut App) {
    app.init_resource::<Probe>()
        .add_systems(Update, discard_network.before(poll_network))
        .add_systems(Last, drive.before(measure));
}

fn discard_network(bridge: Res<ffone_client::network::NetworkBridge>) {
    for event in bridge.drain() {
        assert!(
            matches!(event, ffone_client::network::NetworkEvent::Error(_)),
            "offline inventory fixture received live event: {event:?}"
        );
    }
}

fn drive(world: &mut World) {
    let ready = {
        let mut capture = world.resource_mut::<Capture>();
        capture.samples.clear();
        capture.ready.is_some()
    };
    if !ready {
        return;
    }
    let (frame, stage) = {
        let mut probe = world.resource_mut::<Probe>();
        probe.frame += 1;
        (probe.frame, probe.stage)
    };
    if frame == 1 {
        let mut load = ffone_protocol::PcLoadData0104::zeroed();
        // Weapon 2 (Ben, level 24), neutral weapon 10 (level 23),
        // and weapon 2 combined with the neutral appearance.
        for (slot, id, option) in [(0usize, 2i16, 1i32), (1, 10, 1), (2, 2, (10 << 16) | 1)] {
            let offset =
                ffone_protocol::PcLoadData0104::INVENTORY_OFFSET + slot * ItemBase0104::SIZE;
            let bytes = load.as_bytes_mut();
            bytes[offset..offset + 2].copy_from_slice(&0i16.to_le_bytes());
            bytes[offset + 2..offset + 4].copy_from_slice(&id.to_le_bytes());
            bytes[offset + 4..offset + 8].copy_from_slice(&option.to_le_bytes());
        }
        world.resource_mut::<LocalInventoryRuntime>().seed(1, &load);
        world.resource_mut::<UserEquipUiState>().open_item_mode();
        world.resource_mut::<RuntimeStatus>().player_level = 36;
        world.resource_mut::<RuntimeStatus>().player_gender = Some(1);
        set_guide(world, 2);
    }
    // Allow opening and asset loads to complete before capturing each stage.
    if frame < 120 || frame % 45 != 0 {
        return;
    }
    if stage == 5 {
        println!(
            "BUG052 PASS: guide mismatch, guide change, level restriction, drag and cancellation; neutral and combined items"
        );
        world.resource_mut::<Capture>().captured = true;
        world.write_message(AppExit::Success);
        return;
    }
    let mut query = world.query::<(&UserEquipUiElement, &ImageNode, &Node, Option<&BackgroundColor>)>();
    let mut seen = [false; 3];
    let mut frames_seen = [false; 3];
    for (element, image, node, background) in query.iter(world) {
        if let UserEquipUiElement::InventorySlotFrame(slot @ 0..=2) = *element {
            let restricted = slot != 1 && stage != 1;
            assert_eq!(background.expect("slot background").0.to_srgba(),
                if restricted { Color::srgba(0.85, 0.02, 0.02, 0.8) } else { Color::NONE }.to_srgba(),
                "stage {stage}, frame {slot}");
            frames_seen[slot] = true;
        }
        if let UserEquipUiElement::InventorySlotIcon(slot @ 0..=2) = *element {
            assert_eq!(node.display, Display::Flex);
            let expected = Color::srgba(1.0, 1.0, 1.0,
                if stage == 3 && slot == 0 { 0.5 } else { 1.0 });
            assert_eq!(
                image.color.to_srgba(),
                expected.to_srgba(),
                "stage {stage}, slot {slot}"
            );
            seen[slot] = true;
        }
    }
    assert_eq!(seen, [true; 3]);
    assert_eq!(frames_seen, [true; 3]);
    let mut roots = world.query_filtered::<&InheritedVisibility, With<UserEquipUiRoot>>();
    assert!(roots.iter(world).any(|visibility| visibility.get()));
    let output = world
        .resource::<Capture>()
        .output
        .join(format!("inventory-stage-{stage}.png"));
    world
        .spawn(Screenshot::primary_window())
        .observe(bevy::render::view::screenshot::save_to_disk(output));
    println!("BUG052 stage {stage} PASS: inventory colors verified in full client");
    match stage {
        0 => set_guide(world, 4),
        1 => {
            world.resource_mut::<RuntimeStatus>().player_level = 23;
        }
        2 => world
            .resource_mut::<UserEquipDragState>()
            .begin(UserEquipSlotEndpoint::Inventory { slot_index: 0 }),
        3 => world.resource_mut::<UserEquipDragState>().cancel(),
        _ => {
            // Give the final screenshot time to finish saving.
            world.resource_mut::<Probe>().stage = 5;
            return;
        }
    }
    world.resource_mut::<Probe>().stage += 1;
}

fn set_guide(world: &mut World, mentor: i16) {
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    let offset = ffone_protocol::PcLoadData0104::MENTOR_OFFSET;
    load.as_bytes_mut()[offset..offset + 2].copy_from_slice(&mentor.to_le_bytes());
    world.resource_mut::<GuideRuntime>().load_pc_state(&load);
}
