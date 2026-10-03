//! Opt-in live client interaction check for time-machine and service copy.
use super::*;
use ffone_client::guide_ui::{GuideMentor, GuideUiCommand, GuideUiPhase, apply_guide_ui_command};

#[derive(Default, Resource)]
struct Probe {
    frame: u32,
}

pub(super) fn install(app: &mut App) {
    app.init_resource::<Probe>()
        // This replay has no shard. Periodic movement/stop requests make the
        // real worker report a missing session and reset every modal. Consume
        // those offline-only events here; lifecycle behavior is tested separately.
        .add_systems(Update, discard_offline_events.before(poll_network))
        .add_systems(Last, drive.before(measure));
}

fn discard_offline_events(bridge: Res<NetworkBridge>) {
    let _ = bridge.drain();
}

fn command(world: &mut World, command: GuideUiCommand) {
    world.resource_scope(|world, mut model: Mut<GuideUiModel>| {
        world.resource_scope(|world, mut outbox: Mut<GuideUiOutbox>| {
            let mut audio = world.resource_mut::<GuideUiAudioOutbox>();
            assert!(apply_guide_ui_command(&mut model, &mut outbox, &mut audio, command), "command {command:?}; model={model:?}");
        });
    });
}

fn drive(world: &mut World) {
    if world.resource::<GameplayLoadingState>().visible {
        return;
    }
    if world.resource::<Capture>().transport_npc_type.is_some() {
        transport(world);
        return;
    }
    if world.resource::<Capture>().ready.is_none() || world.resource::<Capture>().samples.is_empty()
    {
        return;
    }
    let frame = {
        let mut probe = world.resource_mut::<Probe>();
        probe.frame += 1;
        probe.frame
    };
    match frame {
        5 => {
            // Metadata ordering is covered by the lifecycle regression. This
            // offline UI replay supplies the paid login context after world entry.
            world.resource_mut::<GuideProductionRuntime>().payment_flag = Some(1);
            let flag = world.resource::<GuideProductionRuntime>().payment_flag;
            assert_eq!(
                guide_service_entry(23, flag).unwrap().service,
                NpcServiceKind::PastWarp
            );
            world
                .resource_mut::<GuideUiModel>()
                .open_initial_selection();
        }
        45 => {
            eprintln!("guide replay state={:?}; hp={:?}; status={}", world.resource::<State<ClientState>>().get(), world.resource::<RuntimeStatus>().hp, world.resource::<RuntimeStatus>().message);
            assert_eq!(
                world.resource::<GuideUiModel>().phase,
                GuideUiPhase::WarpWarning
            );
            command(world, GuideUiCommand::AcceptWarpWarning);
        }
        65 => command(world, GuideUiCommand::SelectMentor(GuideMentor::Dexter)),
        85 => command(world, GuideUiCommand::OpenConfirmation),
        125 => command(world, GuideUiCommand::CancelConfirmation),
        145 => {
            command(
                world,
                GuideUiCommand::Dismiss(ffone_client::guide_ui::GuideUiDismissalSource::EscapeKey),
            );
            world.resource_scope(|world, mut runtime: Mut<NormalNpcWarpRuntime>| {
                world.resource_scope(|world, mut messages: Mut<SystemMessageUiModel>| {
                    runtime
                        .queue_system_message(
                            world.resource::<TutorialMissionContent>(),
                            &mut messages,
                            111,
                        )
                        .unwrap();
                });
            });
        }
        195 => {
            world
                .resource_mut::<Messages<AppExit>>()
                .write(AppExit::Success);
        }
        _ => {}
    }
    let name = match frame {
        30 => "warp-warning",
        75 => "mentor-selection",
        110 => "mentor-confirmation",
        175 => "level-gate",
        _ => return,
    };
    let output = world
        .resource::<Capture>()
        .output
        .join(format!("{name}.png"));
    world
        .spawn(Screenshot::primary_window())
        .observe(bevy::render::view::screenshot::save_to_disk(output));
    if frame == 175 {
        let request = world.resource::<SystemMessageUiModel>().current().unwrap();
        assert_eq!(
            request.localized.key,
            "content.tabledata.message.message.111.sz_string"
        );
        let resolved = world
            .resource::<Localization>()
            .text(world.resource::<Language>(), &request.localized);
        let strings: Vec<_> = world
            .query::<&Text>()
            .iter(world)
            .filter(|text| text.0 == resolved)
            .map(|text| text.0.clone())
            .collect();
        assert!(
            !strings.is_empty(),
            "level gate must render its semantic translation"
        );
        let language = &world.resource::<Language>().effective;
        if language == "ru" {
            assert!(strings.iter().any(|s| s.contains("уровня")));
        }
        fs::write(world.resource::<Capture>().output.join("service-dialogue-pass.txt"), format!("PASS locale={language}; paid service; warning; selection; confirmation/cancel; level gate={strings:?}")).unwrap();
    }
}

fn transport(world: &mut World) {
    if !world.resource::<Capture>().transport_opened
        || *world.resource::<TransportationPresentationAssetStatus>()
            != TransportationPresentationAssetStatus::Ready
    {
        return;
    }
    let frame = {
        let mut probe = world.resource_mut::<Probe>();
        probe.frame += 1;
        probe.frame
    };
    if frame == 30 {
        world
            .resource_mut::<TransportationModel>()
            .select_route(0)
            .unwrap();
    }
    if frame == 60 {
        let model = world.resource::<TransportationModel>();
        let route = &model.routes()[0];
        let name = LocalizedText::new(&route.name_localization_key, &route.name);
        let expected = world
            .resource::<Localization>()
            .text(world.resource::<Language>(), &name);
        let strings: Vec<_> = world
            .query::<&Text>()
            .iter(world)
            .filter(|text| text.0.contains(&expected))
            .map(|text| text.0.clone())
            .collect();
        assert!(
            strings.len() >= 2,
            "destination and selected caption must render {expected}"
        );
        let output = world.resource::<Capture>().output.clone();
        world.spawn(Screenshot::primary_window()).observe(
            bevy::render::view::screenshot::save_to_disk(output.join("transport-localized.png")),
        );
        fs::write(
            output.join("transport-pass.txt"),
            format!("PASS destination/caption={strings:?}"),
        )
        .unwrap();
    }
    if frame == 80 {
        world
            .resource_mut::<Messages<AppExit>>()
            .write(AppExit::Success);
    }
}
