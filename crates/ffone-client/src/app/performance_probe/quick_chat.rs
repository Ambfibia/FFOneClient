//! Bug 51: production Enter menu, popup stacking and real Bevy mouse focus.
use super::*;
use bevy::ui::UiStack;
use ffone_client::gameplay_ui::{
    GAMEPLAY_UI_CAMERA_ORDER, GameplayControllerMenuInput, QuickChatMenuMode,
};

#[derive(Resource, Default)]
struct Probe {
    frame: u32,
    clicked: Option<Entity>,
    complete: bool,
}

pub(super) fn install(app: &mut App) {
    app.init_resource::<Probe>()
        .add_systems(
            PreUpdate,
            pointer
                .after(bevy::input::InputSystems)
                .before(bevy::ui::UiSystems::Focus),
        )
        .add_systems(Update, discard_offline_events.before(poll_network));
}

fn discard_offline_events(bridge: Res<NetworkBridge>) {
    // This opt-in UI replay has no shard. Missing-session worker replies must
    // not reset Enter while its popup is being clicked.
    let _ = bridge.drain();
}

fn column(world: &mut World, level: i32) -> Entity {
    world
        .query::<(Entity, &GlobalZIndex, &ZIndex)>()
        .iter(world)
        .find(|(_, global, z)| global.0 == 0 && z.0 == GAMEPLAY_UI_CAMERA_ORDER as i32 + 11 + level)
        .expect("quick chat global layer")
        .0
}

fn row(world: &mut World, level: i32, index: usize) -> Entity {
    let root = column(world, level);
    world.get::<Children>(root).unwrap()[index]
}

fn click(world: &mut World, level: i32, index: usize) {
    let entity = row(world, level, index);
    assert_eq!(world.get::<Node>(entity).unwrap().display, Display::Flex);
    let point = world.get::<UiGlobalTransform>(entity).unwrap().translation;
    // Both rendering and focus consume this stack. The entire Mission root
    // (including its left-hand Enter buttons) must precede the popup row.
    let mission = world
        .query::<(Entity, &ZIndex)>()
        .iter(world)
        .find(|(_, z)| z.0 == GAMEPLAY_UI_CAMERA_ORDER as i32 + 10)
        .unwrap()
        .0;
    let stack = &world.resource::<UiStack>().uinodes;
    let position = |e| stack.iter().position(|n| *n == e).unwrap();
    assert!(position(entity) > position(mission));
    let mut window = world
        .query_filtered::<&mut Window, With<PrimaryWindow>>()
        .single_mut(world)
        .unwrap();
    window.focused = true;
    window.set_physical_cursor_position(Some(point.as_dvec2()));
    world
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    let mut probe = world.resource_mut::<Probe>();
    probe.clicked = Some(entity);
}

fn checked_press(world: &mut World) {
    let entity = world.resource::<Probe>().clicked.unwrap();
    assert_eq!(
        world.get::<Interaction>(entity),
        Some(&Interaction::Pressed),
        "popup row must receive the real mouse press over Enter"
    );
}

fn capture(world: &mut World, name: &str) {
    let path = world.resource::<Capture>().output.join(name);
    world
        .commands()
        .spawn(Screenshot::primary_window())
        .observe(bevy::render::view::screenshot::save_to_disk(path));
}

fn pointer(world: &mut World) {
    if world.resource::<Capture>().samples.is_empty() || world.resource::<Probe>().complete {
        return;
    }
    world.resource_mut::<Probe>().frame += 1;
    let frame = world.resource::<Probe>().frame;
    let custom_emote = env::var("FFONE_PERF_QUICK_CHAT_CUSTOM_CODE").ok().map(|code| {
        let code = code.parse::<usize>().expect("custom emote code 31..44");
        assert!((31..=44).contains(&code));
        code
    });
    match frame {
        1 | 60 => {
            let mut mission = world.resource_mut::<MissionUiModel>();
            mission.enabled = true;
            mission.nanocom_main_menu_visible = true;
            let mut model = world.resource_mut::<GameplayUiModel>();
            model.chat.active = true;
            model.chat.quick_menu.toggle(if frame == 1 {
                QuickChatMenuMode::Emotes
            } else {
                QuickChatMenuMode::MenuChat
            });
        }
        20 => capture(world, "emotes.png"),
        25 => click(world, 0, if custom_emote.is_some() { 18 } else { 16 }),
        26 => {
            checked_press(world);
            assert_eq!(
                world
                    .resource::<GameplayUiModel>()
                    .chat
                    .quick_menu
                    .open_level0,
                Some(if custom_emote.is_some() { 18 } else { 16 })
            );
        }
        40 => capture(world, "emotes-dance.png"),
        45 => click(world, 1, custom_emote.map_or(0, |code| code - 31)),
        46 | 86 => {
            checked_press(world);
            assert_eq!(
                world.resource::<GameplayUiModel>().chat.quick_menu.mode,
                QuickChatMenuMode::Closed
            );
        }
        50 | 90 => {
            for level in 0..3 {
                let root = column(world, level);
                assert_eq!(world.get::<Node>(root).unwrap().display, Display::None);
                for child in world.get::<Children>(root).unwrap() {
                    assert_eq!(world.get::<Node>(*child).unwrap().display, Display::None);
                    assert!(world.get::<ComputedNode>(*child).unwrap().is_empty());
                }
            }
            // Press again at the former row's location: it must no longer hit.
            world
                .resource_mut::<ButtonInput<MouseButton>>()
                .press(MouseButton::Left);
        }
        51 | 91 => {
            let entity = world.resource::<Probe>().clicked.unwrap();
            assert_ne!(
                world.get::<Interaction>(entity),
                Some(&Interaction::Pressed)
            );
        }
        80 => capture(world, "quick-chat.png"),
        85 => click(world, 0, 0),
        96 => world
            .resource_mut::<GameplayUiModel>()
            .chat
            .quick_menu
            .toggle(QuickChatMenuMode::Emotes),
        110 => world.resource_mut::<GameplayControllerMenuInput>().0 = true,
        112 => {
            assert!(!world.resource::<MissionUiModel>().nanocom_menu_presented());
            assert_eq!(
                world.resource::<GameplayUiModel>().chat.quick_menu.mode,
                QuickChatMenuMode::Closed
            );
        }
        125 => {
            capture(world, "closed.png");
            let path = world.resource::<Capture>().output.join("quick-chat.json");
            fs::write(path, "{\"emotePointer\":true,\"danceSubmenuPointer\":true,\"quickChatPointer\":true,\"hiddenRowsIgnorePointer\":true,\"enterClose\":true,\"networkTested\":false}").unwrap();
            world.resource_mut::<Probe>().complete = true;
            eprintln!("PASS Bug 51: Enter popup layers, leaf close and hidden-row hit testing");
        }
        _ => {}
    }
}

pub(super) fn assert_complete(world: &World) {
    if let Some(probe) = world.get_resource::<Probe>() {
        assert!(probe.complete, "Bug 51 acceptance incomplete");
    }
}
