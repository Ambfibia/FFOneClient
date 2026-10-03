//! Real client ChangeCharacter cycles against an isolated release server.
use super::*;
use ffone_client::quit_menu_ui::{QuitMenuButton, QuitMenuButtonKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Step {
    Select,
    Entering,
    OpeningMenu,
    Clicking,
    AwaitingRoster,
    Capturing,
}

#[derive(Resource)]
struct Probe {
    output: PathBuf,
    started: Instant,
    step: Step,
    cycles: usize,
    selected_uid: Option<i64>,
}

pub(super) fn install(app: &mut App, output: PathBuf) {
    assert_eq!(env::var("FFONE_ISOLATED_TEST_DATABASE").as_deref(), Ok("1"));
    fs::create_dir_all(&output).unwrap();
    app.insert_resource(Probe {
        output,
        started: Instant::now(),
        step: Step::Select,
        cycles: 0,
        selected_uid: None,
    })
    .insert_resource(bevy::winit::WinitSettings::continuous())
    .add_systems(PostStartup, |bridge: Res<NetworkBridge>| {
        bridge
            .send(NetworkCommand::Login {
                login_address: env::var("FFONE_LOGIN_ADDRESS").unwrap(),
                username: env::var("FFONE_USERNAME").unwrap(),
                password: env::var("FFONE_PASSWORD").unwrap(),
            })
            .unwrap();
    })
    .add_systems(Last, drive);
}

fn drive(world: &mut World) {
    let state = *world.resource::<State<ClientState>>().get();
    let loading = world.resource::<GameplayLoadingState>().visible;
    let status = world.resource::<RuntimeStatus>().message.clone();
    let (step, cycles, selected_uid, elapsed) = {
        let probe = world.resource::<Probe>();
        (
            probe.step,
            probe.cycles,
            probe.selected_uid,
            probe.started.elapsed(),
        )
    };
    assert!(
        elapsed < Duration::from_secs(240),
        "ChangeCharacter probe timed out after {cycles} cycles: {state:?}, {step:?}, loading={loading}, {status}"
    );
    assert!(
        step == Step::Select || state != ClientState::Login,
        "ChangeCharacter unexpectedly returned to Login: {status}"
    );
    assert!(
        !status.starts_with("Network error:") && !status.starts_with("Disconnected from"),
        "ChangeCharacter network failure: {status}"
    );
    match step {
        Step::Select if state == ClientState::CharacterSelect && !loading => {
            assert!(!world.resource::<QuitMenuUiModel>().visible);
            assert!(world.resource::<RuntimeStatus>().player_id.is_none());
            let roster = &world.resource::<RuntimeStatus>().roster.characters;
            let character = selected_uid
                .and_then(|uid| roster.iter().find(|character| character.pc_uid == uid))
                .or_else(|| roster.first())
                .expect("isolated test account needs a character");
            let uid = character.pc_uid;
            world
                .resource::<NetworkBridge>()
                .send(NetworkCommand::SelectCharacter {
                    pc_uid: uid,
                    location: CharacterEntryLocation0104::Saved,
                })
                .unwrap();
            let mut probe = world.resource_mut::<Probe>();
            probe.selected_uid = Some(uid);
            probe.step = Step::Entering;
        }
        Step::Entering if state == ClientState::World && !loading => {
            world.resource_mut::<QuitMenuUiModel>().open();
            world.resource_mut::<Probe>().step = Step::OpeningMenu;
        }
        Step::OpeningMenu if world.resource::<QuitMenuUiModel>().visible => {
            let mut query = world.query::<(&QuitMenuButton, &mut Interaction)>();
            let mut clicked = false;
            for (button, mut interaction) in query.iter_mut(world) {
                if button.kind == QuitMenuButtonKind::ChangeCharacter {
                    *interaction = Interaction::Pressed;
                    clicked = true;
                    break;
                }
            }
            if clicked {
                world.resource_mut::<Probe>().step = Step::Clicking;
            }
        }
        Step::Clicking if world.resource::<QuitMenuRuntime>().is_waiting_for_server() => {
            world.resource_mut::<Probe>().step = Step::AwaitingRoster;
        }
        Step::AwaitingRoster if state == ClientState::CharacterSelect && !loading => {
            assert!(!world.resource::<QuitMenuUiModel>().visible);
            assert!(!world.resource::<QuitMenuRuntime>().is_waiting_for_server());
            assert!(world.resource::<RuntimeStatus>().player_id.is_none());
            let remaining_world_entities = world
                .query_filtered::<Entity, With<WorldSliceEntity>>()
                .iter(world)
                .count();
            assert_eq!(
                remaining_world_entities, 0,
                "old world entities survived exit"
            );
            let completed = cycles + 1;
            println!("ChangeCharacter cycle {completed} returned to selection");
            world.resource_mut::<Probe>().cycles = completed;
            if completed == 3 {
                world.resource_mut::<Probe>().step = Step::Capturing;
                let output = world.resource::<Probe>().output.clone();
                world.spawn(Screenshot::primary_window())
                    .observe(bevy::render::view::screenshot::save_to_disk(output.join("selection.png")))
                    .observe(|_: On<ScreenshotCaptured>, probe: Res<Probe>, mut exit: MessageWriter<AppExit>| {
                        fs::write(probe.output.join("passed.txt"), "Three ChangeCharacter cycles returned to an unloaded character selection.\n").unwrap();
                        exit.write(AppExit::Success);
                    });
            } else {
                world.resource_mut::<Probe>().step = Step::Select;
            }
        }
        _ => {}
    }
}
