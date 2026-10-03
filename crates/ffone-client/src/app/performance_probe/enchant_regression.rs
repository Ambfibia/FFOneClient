//! Real focus/input regression with offline authoritative Croc Pot session.
use super::*;
use ffone_client::enchant_ui::{
    EnchantAttachmentSlot0104, EnchantInventoryUiState0104, EnchantUiElement0104,
};
#[derive(Resource, Default)]
struct Probe {
    opened: bool,
    clicked: bool,
    checked: bool,
    frames: u32,
    step: u8,
}
pub(super) fn install(app: &mut App) {
    app.add_systems(
        Update,
        diagnose
            .after(EnchantUiSet0104::Interaction)
            .before(drive_enchant_production_0104),
    )
    .init_resource::<Probe>()
    .add_systems(
        Update,
        open.after(drive_enchant_production_0104)
            .before(sync_enchant_presentation_0104),
    )
    .add_systems(
        PreUpdate,
        pointer
            .after(bevy::input::InputSystems)
            .before(bevy::ui::UiSystems::Focus),
    );
}
fn open(world: &mut World) {
    if world.resource::<Probe>().clicked
        || world.resource::<Capture>().ready.is_none()
        || *world.resource::<State<ClientState>>().get() != ClientState::World
    {
        return;
    }
    if world.resource::<EnchantProductionRuntime0104>().is_active() {
        return;
    }
    // The offline worker can reset the shard session. A new session must start
    // a new gesture; carrying step 1 into it would test a stale drag snapshot.
    if world.resource::<Probe>().opened {
        world.resource_mut::<ButtonInput<MouseButton>>().reset_all();
        let mut probe = world.resource_mut::<Probe>();
        probe.step = 0;
        probe.frames = 0;
        eprintln!("Croc Pot fixture restarted after offline session reset");
    }
    let pc = world.resource::<RuntimeStatus>().player_id.unwrap();
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    let offset = ffone_protocol::PcLoadData0104::INVENTORY_OFFSET;
    load.as_bytes_mut()[offset + 2..offset + 4].copy_from_slice(&433i16.to_le_bytes());
    load.as_bytes_mut()[offset + 4..offset + 8].copy_from_slice(&1i32.to_le_bytes());
    world
        .resource_mut::<LocalInventoryRuntime>()
        .seed(pc, &load);
    let inventory = world
        .resource::<LocalInventoryRuntime>()
        .snapshot()
        .unwrap()
        .clone();
    let player = enchant_player_authority_0104(world.resource::<RuntimeStatus>()).unwrap();
    let output = world
        .resource_mut::<EnchantProductionRuntime0104>()
        .open(
            EnchantOpenContext0104::clean(9001, 6501, player, true),
            &inventory,
        )
        .unwrap();
    world
        .resource_mut::<EnchantProductionShell0104>()
        .push_output(output);
    world.resource_mut::<Probe>().opened = true;
}
fn pointer(world: &mut World) {
    if !world.resource::<Probe>().opened || world.resource::<Probe>().checked {
        return;
    }
    if world.resource::<Probe>().clicked {
        world
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        let status = &world.resource::<RuntimeStatus>().message;
        assert!(
            !world.resource::<EnchantProductionRuntime0104>().is_active(),
            "Croc Pot click did not close: {status}"
        );
        assert!(
            status.starts_with("EnchantMode closed for runtime NPC"),
            "unrelated reset: {status}"
        );
        world.resource_mut::<Probe>().checked = true;
        eprintln!("PASS full-client Croc Pot pointer close");
        return;
    }
    if !world.resource::<EnchantModeProjection0104>().visible {
        return;
    }
    world.resource_mut::<Probe>().frames += 1;
    let frames = world.resource::<Probe>().frames;
    if frames < 25 {
        return;
    }
    let projection = world.resource::<EnchantModeProjection0104>();
    let animation = world.resource::<EnchantInventoryUiState0104>();
    assert!(
        frames < 300,
        "Croc Pot controls stayed locked: {:?}; opening={:?}",
        projection.capabilities,
        animation
    );
    if !projection.capabilities.close_enabled || !animation.panel_controls_enabled() {
        return;
    }
    let step = world.resource::<Probe>().step;
    let element = match step {
        0 => EnchantUiElement0104::InventorySlotFrame(0),
        1 => EnchantUiElement0104::AttachmentFrame(EnchantAttachmentSlot0104::Target),
        2 => {
            assert!(
                world
                    .resource::<EnchantModeProjection0104>()
                    .selection
                    .is_attached(EnchantAttachmentSlot0104::Target),
                "drag release did not attach: {}",
                world.resource::<RuntimeStatus>().message
            );
            EnchantUiElement0104::Clear
        }
        3 => {
            world
                .resource_mut::<ButtonInput<MouseButton>>()
                .release(MouseButton::Left);
            world.resource_mut::<Probe>().step += 1;
            return;
        }
        _ => {
            assert!(
                !world
                    .resource::<EnchantModeProjection0104>()
                    .selection
                    .any_attached(),
                "Clear did not clear selection"
            );
            EnchantUiElement0104::Close
        }
    };
    let mut query = world.query::<(&EnchantUiElement0104, &UiGlobalTransform)>();
    let point = query
        .iter(world)
        .find(|(e, _)| **e == element)
        .unwrap()
        .1
        .translation;
    let mut windows = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
    let mut window = windows.single_mut(world).unwrap();
    window.focused = true;
    window.set_physical_cursor_position(Some(point.as_dvec2()));
    if step == 1 {
        world
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
    } else {
        world
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
    }
    world.resource_mut::<Probe>().step += 1;
    if step >= 4 {
        world.resource_mut::<Probe>().clicked = true;
    }
}
pub(super) fn assert_complete(world: &World) {
    if let Some(probe) = world.get_resource::<Probe>() {
        assert!(probe.checked, "Croc Pot regression incomplete");
    }
}

pub(super) fn owns_pointer(world: &World) -> bool {
    world.contains_resource::<Probe>()
}

fn diagnose(
    probe: Res<Probe>,
    projection: Res<EnchantModeProjection0104>,
    outbox: Res<EnchantUiOutbox0104>,
    controls: Query<(
        &ffone_client::enchant_ui::EnchantInteractiveControl0104,
        &Interaction,
    )>,
) {
    if probe.step > 0 && !probe.checked {
        eprintln!(
            "step={} capabilities={:?}, outbox={outbox:?}, controls={:?}",
            probe.step,
            projection.capabilities,
            controls
                .iter()
                .filter(|(_, i)| **i != Interaction::None)
                .collect::<Vec<_>>()
        );
    }
}
