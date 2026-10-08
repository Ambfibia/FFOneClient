//! Offline server-contract fixture in the full app, including real UI hit testing.
use super::*;
use ffone_client::vendor_ui::{VendorUiElement, VendorUiRoot};
use ffone_protocol::{
    ItemVendor0104, VENDOR_TABLE_ITEM_COUNT_0104, VendorPacket0104, VendorStartSuccess0104,
    VendorTableUpdateSuccess0104,
};

#[derive(Resource, Default)]
struct Probe {
    opened: bool,
    frames: u32,
    captured: bool,
    clicked: bool,
    checked: bool,
    geometry_checked: bool,
    opening_elapsed: f32,
}

pub(super) fn install(app: &mut App) {
    if env::var("FFONE_PERF_VENDOR_REGRESSION").as_deref() == Ok("manual") {
        app.add_systems(Update, (|bridge: Res<NetworkBridge>| { let _ = bridge.drain(); }).before(poll_network));
    }
    app.add_systems(
        PostUpdate,
        verify_geometry.after(bevy::ui::UiSystems::PostLayout),
    )
    .init_resource::<Probe>()
    .add_systems(
        Update,
        open.after(sync_vendor_ui_context)
            .before(VendorUiSet::Lifecycle),
    )
    .add_systems(
        PreUpdate,
        pointer
            .after(bevy::input::InputSystems)
            .before(bevy::ui::UiSystems::Focus),
    );
}

fn open(
    capture: Res<Capture>,
    mut probe: ResMut<Probe>,
    client: Res<State<ClientState>>,
    status: Res<RuntimeStatus>,
    mut inventory: ResMut<LocalInventoryRuntime>,
    mut production: ResMut<VendorProductionRuntime0104>,
    mut ui: ResMut<VendorUiState>,
    mut outbox: ResMut<VendorUiOutbox0104>,
    mut language: ResMut<ffone_client::localization::Language>,
) {
    if env::var("FFONE_PERF_VENDOR_LOCALE").as_deref() == Ok("ru") && language.effective != "ru" {
        language.requested = "ru".into();
        language.effective = "ru".into();
    }
    let gamepad = env::var("FFONE_PERF_GAMEPAD_SCENARIO").as_deref() == Ok("vendor");
    if probe.clicked || gamepad && probe.opened
        || !gamepad && capture.ready.is_none() || *client.get() != ClientState::World {
        return;
    }
    if probe.opened && production.modal_active() && state_is_open(&ui) {
        return;
    }
    let pc = status.player_id.expect("offline PC");
    inventory.seed(pc, &ffone_protocol::PcLoadData0104::zeroed());
    let source = ActiveVendorSourceNpc0104 {
        runtime_npc_id: 9001,
        table_npc_id: 650,
        ai_type: 1,
    };
    ui.begin_open(source.runtime_npc_id, source.table_npc_id, &mut outbox);
    // The offline worker has no shard and periodically emits session resets.
    // Reattach the fixture after that boundary, preserving elapsed animation
    // time rather than forcing Visible or bypassing the opening input gate.
    ui.opening_elapsed_seconds = probe.opening_elapsed;
    assert!(matches!(
        outbox.pop_front(),
        Some(VendorUiCommand0104::StartSession { .. })
    ));
    let VendorOutboundRequest0104::Start(request) = production.begin_start(source).unwrap() else {
        panic!("start")
    };
    // Exactly the echo and table-admission predicate in OpenFusion Vendors.cpp.
    let event = production
        .apply_packet(
            VendorPacket0104::StartSuccess(VendorStartSuccess0104 {
                npc_id: request.npc_id,
                vendor_id: request.vendor_id,
            }),
            inventory.snapshot_mut().unwrap(),
        )
        .unwrap();
    ui.accept_start_success();
    let VendorProductionEvent0104::StartAccepted { table_request } = event else {
        panic!("table")
    };
    assert_eq!(
        table_request.npc_id, table_request.vendor_id,
        "OpenFusion would silently discard this table request"
    );
    let mut items = [ItemVendor0104 {
        vendor_id: 650,
        buy_cost: 0.0,
        item: ItemBase0104 {
            item_type: 0,
            item_id: 0,
            option: 0,
            time_limit: 0,
        },
        sort_num: 0,
    }; VENDOR_TABLE_ITEM_COUNT_0104];
    // First two published listings of vendor 650, with their actual item types.
    let mut listings = vec![(0, 433), (4, 63)];
    if env::var("FFONE_PERF_GAMEPAD_SCENARIO").as_deref() == Ok("vendor") {
        listings.push((7, 1));
    }
    for (index, (kind, id)) in listings.into_iter().enumerate() {
        items[index].item = ItemBase0104 {
            item_type: kind,
            item_id: id,
            option: 1,
            time_limit: 0,
        };
    }
    assert_eq!(
        production
            .apply_packet(
                VendorPacket0104::TableSuccess(VendorTableUpdateSuccess0104 { items }),
                inventory.snapshot_mut().unwrap()
            )
            .unwrap(),
        VendorProductionEvent0104::TableAccepted
    );
    ui.accept_authoritative_table();
    probe.opened = true;
}

fn pointer(
    mut commands: Commands,
    mut probe: ResMut<Probe>,
    capture: Res<Capture>,
    state: Res<VendorUiState>,
    modal: Res<VendorModalState>,
    production: Res<VendorProductionRuntime0104>,
    status: Res<RuntimeStatus>,
    assets: Res<AssetServer>,
    roots: Query<&InheritedVisibility, With<VendorUiRoot>>,
    elements: Query<(&VendorUiElement, &UiGlobalTransform, Option<&ImageNode>)>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
) {
    if !probe.opened || probe.checked {
        return;
    }
    probe.opening_elapsed = probe.opening_elapsed.max(state.opening_elapsed_seconds);
    if env::var("FFONE_PERF_VENDOR_REGRESSION").as_deref() == Ok("manual") { return; }
    if probe.clicked {
        mouse.release(MouseButton::Left);
        keys.release(KeyCode::Escape);
        assert_eq!(
            state.phase,
            VendorLifecyclePhase::Hidden,
            "full app failed to close vendor"
        );
        assert!(
            !production.modal_active(),
            "closed shell retained its runtime modal"
        );
        assert_eq!(
            status.message, "VendorMode closed without a protocol packet",
            "an offline reset must not masquerade as a close click"
        );
        probe.checked = true;
        eprintln!(
            "PASS full-app vendor cold icons, authoritative table, close method={}",
            env::var("FFONE_PERF_VENDOR_REGRESSION").unwrap()
        );
        return;
    }
    if state.phase != VendorLifecyclePhase::Visible || !roots.iter().any(|v| v.get()) {
        return;
    }
    probe.frames += 1;
    let icons = elements
        .iter()
        .filter(|(e, _, _)| matches!(e, VendorUiElement::RowIcon(_)))
        .filter_map(|(_, _, image)| image);
    let loaded = icons
        .filter(|image| {
            assets
                .get_path(image.image.id())
                .is_some_and(|p| p.path().to_string_lossy().starts_with("icons/"))
                && matches!(
                    assets.load_state(image.image.id()),
                    bevy::asset::LoadState::Loaded
                )
        })
        .count();
    assert!(
        probe.frames < 600,
        "cold vendor item icons never finished loading"
    );
    if loaded < 2 || probe.frames < 20 || !probe.geometry_checked {
        return;
    }
    assert!(state.input_capabilities(*modal).close);
    if !probe.captured {
        let path = capture.output.join("vendor-open.png");
        commands.spawn(Screenshot::primary_window()).observe(
            move |event: On<ScreenshotCaptured>| {
                event
                    .image
                    .clone()
                    .try_into_dynamic()
                    .unwrap()
                    .save(&path)
                    .unwrap();
            },
        );
        probe.captured = true;
        return;
    }
    let mut window = windows.single_mut().unwrap();
    window.focused = true;
    if env::var("FFONE_PERF_VENDOR_REGRESSION").as_deref() == Ok("escape") {
        keys.press(KeyCode::Escape);
    } else {
        let point = elements
            .iter()
            .find(|(e, _, _)| **e == VendorUiElement::Close)
            .unwrap()
            .1
            .translation;
        window.set_physical_cursor_position(Some(point.as_dvec2()));
        mouse.press(MouseButton::Left);
    }
    probe.clicked = true;
}

fn state_is_open(state: &VendorUiState) -> bool {
    state.phase != VendorLifecyclePhase::Hidden
}

pub(super) fn assert_complete(world: &World) {
    if env::var("FFONE_PERF_VENDOR_REGRESSION").as_deref() == Ok("manual") { return; }
    if let Some(probe) = world.get_resource::<Probe>() {
        assert!(
            probe.checked,
            "full-app vendor input regression did not complete"
        );
    }
}

fn verify_geometry(
    mut probe: ResMut<Probe>,
    state: Res<VendorUiState>,
    texts: Query<(
        &VendorUiElement,
        &ComputedNode,
        &UiGlobalTransform,
        &ChildOf,
        &Text,
        &ffone_client::localization::LocalizedText,
    )>,
    nodes: Query<(&ComputedNode, &UiGlobalTransform)>,
    images: Query<(&VendorUiElement, &ImageNode)>,
) {
    if probe.geometry_checked || probe.frames < 15 || state.phase != VendorLifecyclePhase::Visible {
        return;
    }
    let mut equipment = 0;
    let mut digits = 0;
    let mut translated_names = 0;
    for (element, computed, transform, parent, text, localized) in &texts {
        if matches!(
            element,
            VendorUiElement::RowName(0)
                | VendorUiElement::RowName(1)
                | VendorUiElement::VendorService
        ) && env::var("FFONE_PERF_VENDOR_LOCALE").as_deref() == Ok("ru")
        {
            assert!(
                localized.key.starts_with("tabledata.npc.service.")
                    || localized.key.starts_with("content.tabledata."),
                "unkeyed vendor copy: {}",
                localized.key
            );
            assert!(
                text.0
                    .chars()
                    .any(|c| ('\u{0400}'..='\u{052f}').contains(&c)),
                "untranslated vendor copy: {}",
                text.0
            );
            translated_names += 1;
        }
        let right = match element {
            VendorUiElement::EquipmentSlotLabel(_) => {
                equipment += 1;
                true
            }
            VendorUiElement::TarosDigit(_) => {
                digits += 1;
                false
            }
            _ => continue,
        };
        let (container, position) = nodes.get(parent.parent()).unwrap();
        assert!(
            computed.size().y > 0.0 && computed.size().y < container.size().y,
            "text must have an intrinsic line box: {element:?}"
        );
        // Taffy rounds the parent-relative location independently from the
        // cumulative-edge-rounded sizes. At fractional DPI, subtracting only
        // the rounded widths invents up to two pixels of alignment error.
        let free = container.unrounded_size() - computed.unrounded_size();
        let location = Vec2::new(if right { free.x } else { free.x * 0.5 }, free.y * 0.5).round();
        let center = location + (computed.size() - container.size()) * 0.5;
        let expected = position.transform_point2(center);
        assert!(
            (transform.translation - expected).abs().max_element() <= 1.0,
            "wrong computed alignment {element:?}: {:?} != {expected:?}",
            transform.translation
        );
    }
    assert_eq!(
        (equipment, digits),
        (
            ffone_client::user_equip_ui::USER_EQUIP_EQUIPMENT_STRIP_ORDER.len(),
            9
        )
    );
    if env::var("FFONE_PERF_VENDOR_LOCALE").as_deref() == Ok("ru") {
        assert_eq!(translated_names, 3);
    }
    for (element, image) in &images {
        if matches!(
            element,
            VendorUiElement::TableShadow | VendorUiElement::InventoryShadow
        ) {
            assert_eq!(image.color, Color::BLACK);
        }
    }
    probe.geometry_checked = true;
    eprintln!(
        "PASS full-app vendor equipment/Taros computed alignment and localized service/item copy"
    );
}
