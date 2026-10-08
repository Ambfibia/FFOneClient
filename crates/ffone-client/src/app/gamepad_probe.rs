//! Opt-in Bug 14 acceptance in the production client; synthetic Bevy pad events.
//! Run with FFONE_PERF_OUTPUT and FFONE_PERF_GAMEPAD=1. Persistence is disabled
//! by the existing offline fixture. This does not replace physical hot-plug acceptance.
use super::*;
use bevy::input::gamepad::{Gamepad, GamepadButton, GamepadConnection, GamepadConnectionEvent};
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use ffone_client::option_ui::{OptionOpenAudioRoute, OptionTab, OptionTabButton};

mod screens;
mod services;

#[derive(Resource, Default)]
struct Probe {
    device: Option<Entity>,
    phase: u8,
    frames: u32,
    waiting_frames: u32,
    button: Option<GamepadButton>,
    stick: Vec2,
    first_focus: Option<Entity>,
    cycles: u8,
    value: usize,
    held_at: f64,
}

pub(super) fn install(app: &mut App) {
    if env::var_os("FFONE_PERF_GAMEPAD").is_none() {
        return;
    }
    assert!(
        performance_probe::requested(),
        "gamepad probe requires the isolated offline fixture"
    );
    app.init_resource::<Probe>()
        .add_systems(Update, discard_offline_network.before(poll_network))
        .add_systems(
            PreUpdate,
            inject.after(InputSystems).before(sample_gamepad_actions),
        )
        .add_systems(PostUpdate, (|world: &mut World| {
            if env::var_os("FFONE_PERF_GAMEPAD_SCENARIO").is_some() {
                services::drive(world);
            } else {
                drive(world);
            }
        }).after(bevy::ui::UiSystems::Layout));
}

fn discard_offline_network(bridge: Res<ffone_client::network::NetworkBridge>) {
    for event in bridge.drain() {
        assert!(
            matches!(event, ffone_client::network::NetworkEvent::Error(_)),
            "offline gamepad fixture received live event: {event:?}"
        );
    }
}

fn inject(probe: Res<Probe>, mut pads: Query<&mut Gamepad>, mut mission: ResMut<MissionUiModel>) {
    if probe.phase == 7 && env::var_os("FFONE_PERF_GAMEPAD_SCENARIO").is_none() {
        // Offline capture has no world-ready packet; present the open NanoCom
        // state at the input boundary to exercise Start while it owns UI.
        mission.enabled = true;
        mission.nanocom_main_menu_visible = true;
    }
    let Some(device) = probe.device else {
        return;
    };
    let Ok(mut pad) = pads.get_mut(device) else {
        return;
    };
    let held: Vec<_> = pad.get_pressed().copied().collect();
    for button in held {
        if Some(button) != probe.button {
            pad.digital_mut().release(button);
        }
    }
    if let Some(button) = probe.button {
        pad.digital_mut().press(button);
    }
    pad.analog_mut()
        .set(bevy::input::gamepad::GamepadAxis::LeftStickX, probe.stick.x);
    pad.analog_mut()
        .set(bevy::input::gamepad::GamepadAxis::LeftStickY, probe.stick.y);
}

fn connect(world: &mut World, device: Entity) {
    world.write_message(GamepadConnectionEvent::new(
        device,
        GamepadConnection::Connected {
            name: "Bug 14 synthetic acceptance".into(),
            vendor_id: None,
            product_id: None,
        },
    ));
}

fn open_options(world: &mut World, tab: OptionTab) {
    let settings = world.resource::<OptionProductionRuntime>();
    let (options, input) = (settings.options.clone(), settings.input.clone());
    world.resource_scope(|world, mut model: Mut<OptionUiModel>| {
        model.open(
            options,
            input,
            OptionOpenAudioRoute::default(),
            &mut world.resource_mut::<OptionUiOutbox>(),
        );
        model.select_tab(tab);
    });
}

fn drive(world: &mut World) {
    if (world.resource::<Probe>().phase < 23
        && *world.resource::<State<ClientState>>().get() != ClientState::World)
        || world.resource::<GameplayLoadingState>().visible
        || !world
            .resource::<ffone_client::option_ui::OptionUiAssetGate>()
            .ready
    {
        return;
    }
    world.resource_scope(|world, mut probe: Mut<Probe>| {
        probe.frames += 1;
        assert!(probe.frames < 600, "Bug 14 production acceptance timed out at phase {}", probe.phase);
        if probe.phase == 0 {
            let device = world.spawn_empty().id();
            probe.device = Some(device);
            connect(world, device);
            open_options(world, OptionTab::Graphics);
            probe.phase = 1;
            probe.frames = 0;
            return;
        }
        let device = probe.device.unwrap();
        if probe.phase >= 12 {
            if screens::drive(world, &mut probe) { probe.frames = 0; }
            return;
        }
        if probe.frames < 3 { return; }
        match probe.phase {
            1 => {
                let focus = world.resource::<super::gamepad_ui::PadUiFocus>().entity;
                assert!(focus.is_some(), "visible Options must receive controller focus");
                let first_tab = world.query::<(Entity, &OptionTabButton)>().iter(world)
                    .find(|(_, tab)| tab.0 == OptionTab::Graphics).unwrap().0;
                world.resource_mut::<super::gamepad_ui::PadUiFocus>().entity = Some(first_tab);
                probe.first_focus = Some(first_tab);
                probe.button = Some(GamepadButton::DPadRight);
                probe.phase = 2;
            }
            2 => {
                let focus = world.resource::<super::gamepad_ui::PadUiFocus>().entity.unwrap();
                assert_ne!(Some(focus), probe.first_focus, "D-pad must move focus");
                let tab = world.get::<OptionTabButton>(focus).expect("first right step targets the next Options tab").0;
                assert_ne!(tab, OptionTab::Graphics);
                probe.button = Some(GamepadButton::South);
                probe.phase = 3;
            }
            3 => {
                assert_ne!(world.resource::<OptionUiModel>().selected_tab, OptionTab::Graphics, "A must activate the focused tab");
                probe.button = Some(GamepadButton::East);
                probe.phase = 4;
            }
            4 => {
                assert!(!world.resource::<OptionUiModel>().visible, "B must close Options");
                probe.button = Some(GamepadButton::Start);
                probe.phase = 5;
            }
            5 => {
                let chat = &world.resource::<GameplayUiModel>().chat;
                assert!(chat.active,
                    "Start must open the Enter reducer (chat visible={}, input_enabled={}, active={}, mission enabled={}, Start held={})",
                    chat.visible, chat.input_enabled, chat.active,
                    world.resource::<MissionUiModel>().enabled,
                    world.resource::<GamepadActionState>().buttons.pressed(GamepadButton::Start));
                probe.button = None;
                probe.phase = 6;
            }
            6 => {
                probe.button = Some(GamepadButton::Start);
                probe.phase = 7;
            }
            7 => {
                assert!(!world.resource::<GameplayUiModel>().chat.active,
                    "Start must close the Enter reducer while NanoCom owns UI");
                probe.button = None;
                world.write_message(GamepadConnectionEvent::new(device, GamepadConnection::Disconnected));
                probe.phase = 8;
            }
            8 => {
                assert!(!world.resource::<QuitMenuUiModel>().visible, "B closing Options must not reopen Quit on the next frame");
                assert!(world.get::<Gamepad>(device).is_none());
                assert!(!world.resource::<GamepadActionState>().connected());
                connect(world, device);
                probe.phase = 9;
            }
            9 => {
                assert!(world.get::<Gamepad>(device).is_some());
                assert!(world.resource::<GamepadActionState>().connected());
                probe.cycles += 1;
                if probe.cycles < 5 {
                    world.write_message(GamepadConnectionEvent::new(device, GamepadConnection::Disconnected));
                    probe.phase = 8;
                } else {
                    open_options(world, OptionTab::Controls);
                    probe.phase = 10;
                }
            }
            10 => {
                let first_tab = world.query::<(Entity, &OptionTabButton)>().iter(world)
                    .find(|(_, tab)| tab.0 == OptionTab::Graphics).unwrap().0;
                world.resource_mut::<super::gamepad_ui::PadUiFocus>().entity = Some(first_tab);
                probe.first_focus = Some(first_tab);
                probe.stick = Vec2::X;
                probe.phase = 11;
            }
            11 => {
                assert_ne!(world.resource::<super::gamepad_ui::PadUiFocus>().entity, probe.first_focus,
                    "left stick must move UI focus like D-pad");
                probe.stick = Vec2::ZERO;
                let path = PathBuf::from(env::var_os("FFONE_PERF_OUTPUT").unwrap()).join("bug-014-controls.png");
                if let Some(parent) = path.parent() { std::fs::create_dir_all(parent).unwrap(); }
                world.spawn(Screenshot::primary_window()).observe(move |event: On<ScreenshotCaptured>| {
                    let image = event.image.clone().try_into_dynamic().unwrap().to_rgba8();
                    image.save(&path).unwrap();
                    println!("BUG014 PASS: production D-pad and left-stick focus, A confirms tab, B closes Options, Start toggles Enter reducer under open NanoCom input gate, five Bevy disconnect/reconnect cycles; Controls screenshot {}", path.display());
                });
                probe.phase = 12;
            }
            _ => {}
        }
        probe.frames = 0;
    });
}
