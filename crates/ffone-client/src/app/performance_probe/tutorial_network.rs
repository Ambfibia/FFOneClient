//! Full native tutorial entry against an explicitly isolated test server.
use super::*;

#[derive(Resource)]
struct Probe {
    output: PathBuf,
    started: Instant,
    entered: bool,
    captured: bool,
    ready_since: Option<Instant>,
    last_status: String,
}

pub(super) fn install(app: &mut App, output: PathBuf) {
    assert_eq!(env::var("FFONE_ISOLATED_TEST_DATABASE").as_deref(), Ok("1"));
    fs::create_dir_all(&output).unwrap();
    app.insert_resource(Probe {
        output,
        started: Instant::now(),
        entered: false,
        captured: false,
        ready_since: None,
        last_status: String::new(),
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
    let loading = world.resource::<GameplayLoadingState>();
    let status = format!(
        "{state:?} {loading:?} {}",
        world.resource::<RuntimeStatus>().message
    );
    let loading_visible = loading.visible;
    let expect_disconnect = env::var_os("FFONE_TUTORIAL_NETWORK_EXPECT_DISCONNECT").is_some();
    if world.resource::<Probe>().last_status != status {
        println!("tutorial-network {status}");
        let mut probe = world.resource_mut::<Probe>();
        fs::write(probe.output.join("status.txt"), &status).unwrap();
        probe.last_status = status.clone();
    }
    assert!(
        world.resource::<Probe>().started.elapsed() < Duration::from_secs(180),
        "tutorial network entry timed out: {status}"
    );
    if state == ClientState::CharacterSelect
        && !loading_visible
        && !world.resource::<Probe>().entered
    {
        let character = world
            .resource::<RuntimeStatus>()
            .roster
            .characters
            .first()
            .expect("create an unfinished character on the isolated server first")
            .clone();
        assert_eq!(character.style.tutorial_flag, 0);
        world.resource_mut::<Probe>().entered = true;
        world.resource_mut::<TutorialSession>().character = Some(character);
        world
            .resource_mut::<NextState<ClientState>>()
            .set(ClientState::TutorialIntro);
    }
    let disconnected = state == ClientState::Login
        && world.resource::<Probe>().entered
        && world
            .resource::<RuntimeStatus>()
            .message
            .starts_with("Disconnected from OpenFusion:");
    if expect_disconnect && disconnected {
        assert!(
            !loading_visible,
            "disconnect left the tutorial loading screen visible"
        );
    }
    if !expect_disconnect && state == ClientState::Tutorial && !loading_visible {
        let mut probe = world.resource_mut::<Probe>();
        let since = *probe.ready_since.get_or_insert_with(Instant::now);
        // Cover at least two default five-second keepalive intervals after
        // handoff, rather than accepting a single WorldReady frame.
        if since.elapsed() < Duration::from_secs(12) {
            return;
        }
    }
    if ((!expect_disconnect && state == ClientState::Tutorial && !loading_visible)
        || (expect_disconnect && disconnected))
        && !world.resource::<Probe>().captured
    {
        let output = world.resource::<Probe>().output.clone();
        world.resource_mut::<Probe>().captured = true;
        world.spawn(Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(output.join("frame.png")))
            .observe(|_: On<ScreenshotCaptured>, probe: Res<Probe>, mut exit: MessageWriter<AppExit>| {
                fs::write(probe.output.join("passed.txt"), "Expected full-network tutorial transition passed; loading overlay released.\n").unwrap();
                exit.write(AppExit::Success);
            });
    }
}
