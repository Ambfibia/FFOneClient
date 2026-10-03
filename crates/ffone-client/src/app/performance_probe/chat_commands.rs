//! Bug 56: production chat Enter submission and localized server replies on GPU.
//! Offline presentation replay; authoritative rewards are covered by server tests.
use super::*;
use ffone_client::gameplay_ui::GameplayControllerMenuInput;

#[derive(Default, Resource)]
struct Probe {
    frame: u32,
    enter: bool,
}

pub(super) fn install(app: &mut App) {
    app.init_resource::<Probe>()
        .add_systems(Update, discard_offline_events.before(poll_network))
        .add_systems(Update, enter.before(ffone_client::gameplay_ui::GameplayUiSet::Input))
        .add_systems(Last, drive.before(measure));
}

fn enter(mut probe: ResMut<Probe>, mut input: ResMut<GameplayControllerMenuInput>) {
    if std::mem::take(&mut probe.enter) { input.0 = true; }
}

fn discard_offline_events(bridge: Res<NetworkBridge>) {
    let _ = bridge.drain();
}

fn receive(world: &mut World, message: &str) {
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_PC_MOTD_LOGIN,
        flags: 0,
        checksum: 0,
        payload: ffone_protocol::ServerMessage0104 {
            message_type: 0,
            message: FixedUtf16::from_str(message).unwrap(),
        }
        .encode(),
    };
    world.resource_scope(|world, mut runtime: Mut<RuntimeStatus>| {
        super::super::social_ingress::apply_server_message_frame_0104(
            &frame,
            &mut runtime,
            world.resource::<Localization>(),
            world.resource::<Language>(),
        )
        .unwrap();
    });
}

fn drive(world: &mut World) {
    if world.resource::<Capture>().samples.is_empty() {
        return;
    }
    world.resource_mut::<Probe>().frame += 1;
    let frame = world.resource::<Probe>().frame;
    match frame {
        1 => {
            let mut model = world.resource_mut::<GameplayUiModel>();
            model.chat.selected = ffone_client::gameplay_ui::ChatChannel::Buddy;
            model.chat.active = true;
            model.chat.input = "/help".into();
            world.resource_mut::<Probe>().enter = true;
        }
        5 => {
            assert_eq!(
                world.resource::<GameplayUiModel>().chat.selected,
                ffone_client::gameplay_ui::ChatChannel::All
            );
            assert!(
                world.resource::<GameplayUiModel>().chat.input.is_empty(),
                "Enter must consume /help"
            );
            for text in [
                "Available commands",
                "/help: Show this help message",
                "/redeem: Redeem a code item",
            ] {
                receive(world, text);
            }
        }
        25 | 65 => {
            let rows = world
                .resource::<RuntimeStatus>()
                .chat
                .world_chat_lines
                .clone();
            for row in &rows {
                assert!(
                    world
                        .query::<&Text>()
                        .iter(world)
                        .any(|text| text.0 == row.text),
                    "missing rendered chat row {}",
                    row.text
                );
            }
            let output = world.resource::<Capture>().output.clone();
            world.spawn(Screenshot::primary_window()).observe(
                bevy::render::view::screenshot::save_to_disk(
                    output.join(format!("chat-{frame}.png")),
                ),
            );
            fs::write(
                output.join(format!("chat-{frame}-pass.txt")),
                rows.iter()
                    .map(|row| row.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
            .unwrap();
        }
        35 => {
            world
                .resource_mut::<RuntimeStatus>()
                .chat
                .world_chat_lines
                .clear();
            world
                .resource_mut::<RuntimeStatus>()
                .chat
                .world_group_chat_lines
                .clear();
            let mut model = world.resource_mut::<GameplayUiModel>();
            model.chat.active = true;
            model.chat.input = "/redeem unknown".into();
            world.resource_mut::<Probe>().enter = true;
        }
        40 => {
            assert!(
                world.resource::<GameplayUiModel>().chat.input.is_empty(),
                "Enter must consume /redeem"
            );
            for text in [
                "/redeem: Unknown code",
                "You have redeemed code items",
                "/redeem: You have already redeemed this code item",
            ] {
                receive(world, text);
            }
        }
        85 => {
            world
                .resource_mut::<Messages<AppExit>>()
                .write(AppExit::Success);
        }
        _ => {}
    }
}
