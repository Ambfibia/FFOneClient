//! One remaining service screen per full-client run. This is offline UI
//! acceptance; physical hot-plug and shard transactions need separate runs.
use super::*;
use ffone_client::email_ui::{EMAIL_UI_OPEN_SECONDS, EmailFolder};
use ffone_client::guide_ui::GuideMentor;
use ffone_client::launcher_ui::{LauncherTriggerSpec, LauncherUiDismissalSource, LauncherUiOutbox};
use ffone_client::mission_ui::NpcServiceUiEntry;
use ffone_client::transportation_ui::{
    TransportationOpenContext, TransportationPlayerSnapshot, TransportationPresentationAssetStatus,
    TransportationPresentationControl, TransportationPresentationControlNode, TransportationTarget,
    TransportationUnlocks, TransportationWorldPoint,
};
use ffone_client::ui::shared::controller::{ControllerUiDefault, ControllerUiInput};
use ffone_client::vendor_ui::{VendorItemPopupState, VendorTab0104, VendorUiElement};

fn focus(world: &World) -> Entity {
    world
        .resource::<super::super::gamepad_ui::PadUiFocus>()
        .entity
        .expect("service focus")
}

fn target(world: &mut World, predicate: impl Fn(&Node) -> bool) -> Entity {
    world
        .query_filtered::<(Entity, &Node, &ComputedNode), With<Button>>()
        .iter(world)
        .find(|(_, node, computed)| computed.size().min_element() > 0.0 && predicate(node))
        .expect("authored service control")
        .0
}

fn aim(world: &mut World, entity: Entity) {
    world
        .resource_mut::<super::super::gamepad_ui::PadUiFocus>()
        .entity = Some(entity);
}

fn move_mouse(world: &mut World, entity: Entity) {
    let position = world.get::<UiGlobalTransform>(entity).unwrap().translation;
    let mut windows = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
    let mut window = windows.single_mut(world).unwrap();
    window.set_cursor_position(Some(position));
}

fn vendor_element(world: &mut World, expected: VendorUiElement) -> Entity {
    world
        .query::<(Entity, &VendorUiElement)>()
        .iter(world)
        .find(|(_, element)| **element == expected)
        .expect("vendor element")
        .0
}

fn amount(world: &mut World) -> i32 {
    world
        .query::<(&Text, &Node, &ComputedNode)>()
        .iter(world)
        .find(|(_, node, computed)| {
            computed.size().min_element() > 0.0 && node.left == px(88) && node.top == px(163)
        })
        .expect("calculator amount")
        .0
        .0
        .parse()
        .expect("numeric quantity")
}

pub(super) fn drive(world: &mut World) {
    let scenario = env::var("FFONE_PERF_GAMEPAD_SCENARIO").unwrap();
    let expected = if scenario == "tutorial" {
        ClientState::Tutorial
    } else {
        ClientState::World
    };
    if *world.resource::<State<ClientState>>().get() != expected
        || world.resource::<GameplayLoadingState>().visible
        || scenario != "tutorial"
            && !world
                .resource::<ffone_client::option_ui::OptionUiAssetGate>()
                .ready
    {
        let waiting = {
            let mut probe = world.resource_mut::<Probe>();
            probe.waiting_frames += 1;
            probe.waiting_frames
        };
        if waiting == 600 {
            println!(
                "BUG014 {scenario} waiting: state {:?}, loading {:?}, option gate {:?}",
                world.resource::<State<ClientState>>().get(),
                world.resource::<GameplayLoadingState>(),
                world.resource::<ffone_client::option_ui::OptionUiAssetGate>()
            );
            screens::screenshot(world, "gamepad-waiting.png");
        }
        assert!(
            waiting < 3600,
            "Bug 14 {scenario} could not enter the ready client state"
        );
        return;
    }
    world.resource_scope(|world, mut probe: Mut<Probe>| {
        probe.frames += 1;
        assert!(
            probe.frames < 1200,
            "Bug 14 {scenario} timed out at {}",
            probe.phase
        );
        if probe.phase == 0 {
            let device = world.spawn_empty().id();
            probe.device = Some(device);
            connect(world, device);
            match scenario.as_str() {
                "vendor" => {
                    world.resource_mut::<RuntimeStatus>().candy = 100000;
                }
                "guide" => world
                    .resource_mut::<GuideUiModel>()
                    .open_change(GuideMentor::BenTennyson),
                "email" => {
                    let mut load = ffone_protocol::PcLoadData0104::zeroed();
                    let offset = ffone_protocol::PcLoadData0104::MENTOR_OFFSET;
                    load.as_bytes_mut()[offset..offset + 2].copy_from_slice(&5_i16.to_le_bytes());
                    world.resource_mut::<GuideRuntime>().load_pc_state(&load);
                    let player = world.resource::<RuntimeStatus>().player_id.unwrap();
                    world
                        .resource_mut::<LocalInventoryRuntime>()
                        .seed(player, &load);
                    world
                        .resource_mut::<GameplayUiOutbox>()
                        .push(GameplayUiAction::OpenEmailFromNanocom);
                }
                "transport" => {
                    world.resource_scope(|world, mut model: Mut<TransportationModel>| {
                        let position = TransportationWorldPoint::new(2246.94, 105.0, 2290.48);
                        model
                            .open(
                                world.resource::<TransportationCatalog>(),
                                TransportationOpenContext {
                                    player: TransportationPlayerSnapshot {
                                        position,
                                        taros: 10000,
                                        unlocks: TransportationUnlocks {
                                            warp_location_flags: u32::MAX,
                                            wyvern_location_flags: [u64::MAX; 2],
                                        },
                                        cursor_was_locked: false,
                                    },
                                    target: TransportationTarget::Npc {
                                        npc_instance_id: 26190,
                                        npc_table_id: 2619,
                                        npc_position: position,
                                        has_move_ok_voice: false,
                                    },
                                },
                            )
                            .unwrap();
                        while model.pop_outbox().is_some() {}
                    })
                }
                "dialogue" => open_dialogue(world),
                "tutorial" => {}
                "cannon" => world.resource_scope(|world, mut model: Mut<LauncherUiModel>| {
                    model
                        .open(
                            LauncherTriggerSpec {
                                trigger_position: Vec3::ZERO,
                                trigger_euler_degrees: Vec3::ZERO,
                                min_power: 10.0,
                                max_power: 100.0,
                                initial_rotation_degrees: Vec3::ZERO,
                                maximum_rotation_degrees: Vec3::splat(45.0),
                            },
                            Vec3::ZERO,
                            6.0,
                            &mut world.resource_mut::<LauncherUiOutbox>(),
                        )
                        .unwrap();
                }),
                other => panic!("unsupported gamepad service {other}"),
            }
            probe.phase = 1;
            probe.frames = 0;
            return;
        }
        let direction = probe.button.is_some_and(|button| {
            matches!(
                button,
                GamepadButton::DPadUp
                    | GamepadButton::DPadDown
                    | GamepadButton::DPadLeft
                    | GamepadButton::DPadRight
            )
        });
        let confirm = scenario == "dialogue" && probe.button == Some(GamepadButton::South);
        if probe.frames < 8 && !direction && !confirm {
            return;
        }
        let advanced = match scenario.as_str() {
            "vendor" => vendor(world, &mut probe),
            "guide" => guide(world, &mut probe),
            "email" => email(world, &mut probe),
            "transport" => transport(world, &mut probe),
            "dialogue" => dialogue(world, &mut probe),
            "tutorial" => tutorial(world, &mut probe),
            "cannon" => cannon(world, &mut probe),
            _ => unreachable!(),
        };
        if advanced {
            probe.phase += 1;
            probe.frames = 0;
        }
    });
}

fn open_dialogue(world: &mut World) {
    let npc = world
        .resource::<TutorialMissionContent>()
        .gameplay_npc(650)
        .unwrap();
    let interaction = NpcInteractionUi {
        npc_id: 9001,
        npc_type: 650,
        name: npc.name.clone(),
        available_missions: vec![],
        completed_missions: vec![],
        warp: None,
        services: vec![NpcServiceUiEntry::original(NpcServiceKind::Vendor)],
    };
    world
        .resource_mut::<MissionUiModel>()
        .show_npc_interaction(interaction);
}

fn dialogue(world: &mut World, probe: &mut Probe) -> bool {
    match probe.phase {
        1 => {
            assert!(
                world.get::<ControllerUiDefault>(focus(world)).is_some(),
                "first NPC option, not close"
            );
            screens::screenshot(world, "gamepad-npc-first-option.png");
            probe.button = Some(GamepadButton::East);
        }
        2 => {
            assert!(!world.resource::<MissionUiModel>().npc_icon_mode_visible);
            probe.button = None;
            open_dialogue(world);
        }
        3 => {
            probe.first_focus = Some(focus(world));
            assert!(world.get::<ControllerUiDefault>(focus(world)).is_some());
            probe.button = Some(GamepadButton::South);
        }
        4 => {
            assert_eq!(
                world.resource::<ControllerUiInput>().confirmed,
                probe.first_focus
            );
            finish(world, "DIALOGUE");
        }
        _ => unreachable!(),
    }
    true
}

fn tutorial(world: &mut World, probe: &mut Probe) -> bool {
    match probe.phase {
        1 => {
            probe.button = Some(GamepadButton::South);
        }
        2 => {
            assert!(
                world
                    .resource::<GamepadActionState>()
                    .held(LegacyOptionAction::Jump)
            );
            assert!(
                world
                    .resource::<super::super::gamepad_ui::PadUiFocus>()
                    .entity
                    .is_none()
            );
            screens::screenshot(world, "gamepad-tutorial-input.png");
            probe.button = Some(GamepadButton::West);
            probe.stick.y = 1.0;
        }
        3 => {
            let pad = world.resource::<GamepadActionState>();
            assert!(pad.held(LegacyOptionAction::Nano1));
            assert!(pad.value(LegacyOptionAction::Up) > 0.9);
            assert!(
                world
                    .resource::<super::super::gamepad_ui::PadUiFocus>()
                    .entity
                    .is_none()
            );
            finish(world, "TUTORIAL INPUT OWNERSHIP");
        }
        _ => unreachable!(),
    }
    true
}

fn cannon(world: &mut World, probe: &mut Probe) -> bool {
    match probe.phase {
        1 => {
            probe.value = world.resource::<LauncherUiModel>().current_power as usize;
            probe.button = Some(GamepadButton::South);
            probe.held_at = world.resource::<Time>().elapsed_secs_f64();
        }
        2 => {
            let model = world.resource::<LauncherUiModel>();
            assert!(model.charging && model.current_power > probe.value as f32);
            if world.resource::<Time>().elapsed_secs_f64() - probe.held_at < 0.6 {
                return false;
            }
            screens::screenshot(world, "gamepad-cannon-charge.png");
            probe.button = None;
        }
        3 => {
            assert_eq!(
                world.resource::<LauncherUiModel>().dismissal,
                Some(LauncherUiDismissalSource::Fired)
            );
            finish(world, "CANNON CHARGE/RELEASE");
        }
        _ => unreachable!(),
    }
    true
}

fn finish(world: &mut World, scenario: &str) {
    println!("BUG014 {scenario} PASS (offline full client; synthetic pad)");
    fs::write(
        PathBuf::from(env::var_os("FFONE_PERF_OUTPUT").unwrap()).join("gamepad-result.txt"),
        format!("BUG014 {scenario} PASS (offline full client; synthetic pad)\n"),
    )
    .unwrap();
    world.write_message(AppExit::Success);
}

fn vendor(world: &mut World, probe: &mut Probe) -> bool {
    if probe.phase == 1 && world.resource::<VendorUiState>().phase != VendorLifecyclePhase::Visible
    {
        return false;
    }
    match probe.phase {
        1 => {
            assert_eq!(focus(world), vendor_element(world, VendorUiElement::Row(0)));
            probe.button = Some(GamepadButton::RightTrigger);
        }
        2 => {
            assert_eq!(
                world.resource::<VendorUiState>().tab,
                VendorTab0104::Buyback
            );
            probe.button = Some(GamepadButton::LeftTrigger);
        }
        3 => {
            assert_eq!(world.resource::<VendorUiState>().tab, VendorTab0104::Buy);
            probe.button = None;
            let row = vendor_element(world, VendorUiElement::Row(0));
            aim(world, row);
        }
        4 => {
            probe.button = Some(GamepadButton::DPadDown);
        }
        5 => {
            assert_eq!(focus(world), vendor_element(world, VendorUiElement::Row(1)));
            probe.button = None;
        }
        6 => {
            probe.button = Some(GamepadButton::DPadDown);
        }
        7 => {
            assert_eq!(focus(world), vendor_element(world, VendorUiElement::Row(2)));
            probe.button = Some(GamepadButton::South);
        }
        8 => {
            assert!(world.resource::<VendorItemPopupState>().is_open());
            probe.button = None;
        }
        9 => {
            let entity = focus(world);
            let node = world.get::<Node>(entity).unwrap();
            assert_eq!(
                (node.left, node.top),
                (px(88), px(191)),
                "first calculator digit"
            );
            screens::screenshot(world, "gamepad-vendor-calculator.png");
            let close = vendor_element(world, VendorUiElement::Close);
            move_mouse(world, close);
            probe.button = Some(GamepadButton::South);
            probe.held_at = world.resource::<Time>().elapsed_secs_f64();
        }
        10 => {
            assert!(amount(world) >= 1, "mouse movement must not block pad A");
            if world.resource::<Time>().elapsed_secs_f64() - probe.held_at < 0.6 {
                return false;
            }
            assert!(amount(world) > 1, "holding A must repeat quantity digits");
            probe.button = None;
        }
        11 => {
            probe.button = Some(GamepadButton::East);
        }
        12 => {
            assert!(!world.resource::<VendorItemPopupState>().is_open());
            assert_eq!(
                world.resource::<VendorUiState>().phase,
                VendorLifecyclePhase::Visible,
                "B closes the calculator first"
            );
            probe.button = None;
        }
        13 => {
            probe.button = Some(GamepadButton::East);
        }
        14 => {
            assert_eq!(
                world.resource::<VendorUiState>().phase,
                VendorLifecyclePhase::Hidden
            );
            finish(world, "VENDOR");
        }
        _ => unreachable!(),
    }
    true
}

fn guide(world: &mut World, probe: &mut Probe) -> bool {
    match probe.phase {
        1 => {
            assert!(world.get::<ControllerUiDefault>(focus(world)).is_some());
            probe.button = Some(GamepadButton::DPadRight);
        }
        2 => {
            probe.button = Some(GamepadButton::South);
        }
        3 => {
            assert_eq!(
                world.resource::<GuideUiModel>().selected,
                Some(GuideMentor::Dexter)
            );
            screens::screenshot(world, "gamepad-guide-selection.png");
            probe.button = None;
            let primary = target(world, |node| node.left == px(840) && node.top == px(605));
            aim(world, primary);
            move_mouse(world, primary);
        }
        4 => {
            probe.button = Some(GamepadButton::South);
        }
        5 => {
            assert!(world.resource::<GuideUiModel>().confirmation_open);
            probe.button = None;
        }
        6 => {
            assert!(world.get::<ControllerUiDefault>(focus(world)).is_some());
            probe.button = Some(GamepadButton::DPadUp);
            probe.first_focus = Some(focus(world));
        }
        7 => {
            assert_eq!(
                Some(focus(world)),
                probe.first_focus,
                "confirmation captures focus"
            );
            screens::screenshot(world, "gamepad-guide-confirmation.png");
            probe.button = Some(GamepadButton::East);
        }
        8 => {
            assert!(!world.resource::<GuideUiModel>().confirmation_open);
            assert!(!world.resource::<GuideUiModel>().visible);
            finish(world, "GUIDE");
        }
        _ => unreachable!(),
    }
    true
}

fn email(world: &mut World, probe: &mut Probe) -> bool {
    if probe.phase == 1 {
        if !world.resource::<EmailUiModel>().visible {
            return false;
        }
        {
            let mut model = world.resource_mut::<EmailUiModel>();
            model.opening_elapsed_seconds = EMAIL_UI_OPEN_SECONDS;
            model.right_opening_elapsed_seconds = EMAIL_UI_OPEN_SECONDS;
        }
        if world
            .resource::<super::super::gamepad_ui::PadUiFocus>()
            .entity
            .is_none()
        {
            return false;
        }
    }
    match probe.phase {
        1 => {
            probe.button = Some(GamepadButton::RightTrigger);
        }
        2 => {
            assert_eq!(world.resource::<EmailUiModel>().folder, EmailFolder::Player);
            cancel_offline_email_request(world);
            move_mouse(world, focus(world));
            probe.button = Some(GamepadButton::LeftTrigger);
        }
        3 => {
            assert_eq!(world.resource::<EmailUiModel>().folder, EmailFolder::Guide);
            cancel_offline_email_request(world);
            screens::screenshot(world, "gamepad-email-guide-tab.png");
            probe.button = Some(GamepadButton::East);
        }
        4 => {
            assert!(!world.resource::<EmailUiModel>().visible);
            finish(world, "EMAIL");
        }
        _ => unreachable!(),
    }
    true
}

fn cancel_offline_email_request(world: &mut World) {
    // No shard is connected in this fixture. Exercise the socket-failure path;
    // do not fabricate a successful mail transaction or bypass the close guard.
    if world
        .resource::<EmailProductionRuntime0104>()
        .pending_request()
        .is_some()
    {
        world.resource_scope(|world, mut runtime: Mut<EmailProductionRuntime0104>| {
            world.resource_scope(|world, mut model: Mut<EmailUiModel>| {
                runtime
                    .cancel_pending_transport(
                        &mut model,
                        &mut world.resource_mut::<EmailNetworkRuntime0104>(),
                    )
                    .unwrap();
            });
        });
    }
}

fn transport(world: &mut World, probe: &mut Probe) -> bool {
    if probe.phase == 1 {
        if world.resource::<TransportationModel>().phase() != TransportationPhase::Browsing
            || !matches!(
                *world.resource::<TransportationPresentationAssetStatus>(),
                TransportationPresentationAssetStatus::Ready
            )
            || !world
                .query_filtered::<&ComputedNode, With<ControllerUiDefault>>()
                .iter(world)
                .any(|node| node.size().min_element() > 0.0)
        {
            return false;
        }
        // Layout just materialized these rows in PostUpdate. Navigation runs
        // in PreUpdate and must read that layout once before asserting focus.
        if probe.value == 0 {
            probe.value = 1;
            let focused = focus(world);
            println!(
                "transport initial focus {focused:?}, node {:?}, control {:?}",
                world.get::<Node>(focused),
                world.get::<TransportationPresentationControlNode>(focused)
            );
            for (entity, control, computed, transform, clip) in world
                .query::<(
                    Entity,
                    &TransportationPresentationControlNode,
                    &ComputedNode,
                    &UiGlobalTransform,
                    Option<&bevy::ui::CalculatedClip>,
                )>()
                .iter(world)
            {
                println!(
                    "transport {entity:?} {:?}: size {:?}, center {:?}, clip {:?}",
                    control.0,
                    computed.size(),
                    transform.translation,
                    clip
                );
            }
            screens::screenshot(world, "gamepad-transport-first-focus.png");
            return false;
        }
        if probe.value < 8 {
            probe.value += 1;
            return false;
        }
    }
    match probe.phase {
        1 => {
            assert!(world.get::<ControllerUiDefault>(focus(world)).is_some());
            probe.button = Some(GamepadButton::South);
        }
        2 => {
            assert!(
                world
                    .resource::<TransportationModel>()
                    .selected_route()
                    .is_some()
            );
            probe.button = None;
            let turbo = world
                .query::<(Entity, &TransportationPresentationControlNode)>()
                .iter(world)
                .find(|(_, control)| control.0 == TransportationPresentationControl::Turbo)
                .unwrap()
                .0;
            aim(world, turbo);
            move_mouse(world, turbo);
        }
        3 => {
            probe.button = Some(GamepadButton::South);
        }
        4 => {
            assert!(world.resource::<TransportationModel>().turbo());
            screens::screenshot(world, "gamepad-transport-turbo.png");
            probe.button = Some(GamepadButton::East);
        }
        5 => {
            assert_eq!(
                world.resource::<TransportationModel>().phase(),
                TransportationPhase::Hidden
            );
            finish(world, "TRANSPORT");
        }
        _ => unreachable!(),
    }
    true
}
