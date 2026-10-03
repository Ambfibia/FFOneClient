//! Bug 002: two creations and menu returns through the production client.
use super::*;
use ffone_client::{
    character_creation_ui::CustomCharacterName,
    quit_menu_ui::{QuitMenuButton, QuitMenuButtonKind},
};

#[path = "equipment_session.rs"]
mod equipment;

#[derive(Clone, Copy, Debug)]
enum Step {
    Create,
    Name,
    Appearance,
    Tutorial,
    World,
    Menu,
    Returning,
    Capture,
}

#[derive(Resource)]
struct Probe {
    output: PathBuf,
    started: Instant,
    step: Step,
    completed: usize,
    last_status: String,
}

pub(super) fn install(app: &mut App, output: PathBuf) {
    assert_eq!(env::var("FFONE_ISOLATED_TEST_DATABASE").as_deref(), Ok("1"));
    fs::create_dir_all(&output).unwrap();
    equipment::install(app);
    app.insert_resource(Probe {
        output,
        started: Instant::now(),
        step: Step::Create,
        completed: 0,
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
    .add_systems(PreUpdate, click_menu.after(bevy::ui::UiSystems::Focus))
    .add_systems(Last, drive);
}

fn click_menu(
    mut probe: ResMut<Probe>,
    model: Res<QuitMenuUiModel>,
    mut buttons: Query<(&QuitMenuButton, &mut Interaction)>,
) {
    if !matches!(probe.step, Step::Menu) || !model.visible || !model.enabled {
        return;
    }
    for (button, mut interaction) in &mut buttons {
        if button.kind == QuitMenuButtonKind::ChangeCharacter {
            *interaction = Interaction::Pressed;
            probe.step = Step::Returning;
            break;
        }
    }
}

fn drive(world: &mut World) {
    let state = *world.resource::<State<ClientState>>().get();
    let loading = world.resource::<GameplayLoadingState>().visible;
    let status = world.resource::<RuntimeStatus>().message.clone();
    let (step, completed) = {
        let mut probe = world.resource_mut::<Probe>();
        assert!(
            probe.started.elapsed() < Duration::from_secs(600),
            "character session timed out: {status}"
        );
        let detail = format!("{:?} {state:?} loading={loading} {status}", probe.step);
        if detail != probe.last_status {
            println!("character-session {detail}");
            fs::write(probe.output.join("status.txt"), &detail).unwrap();
            probe.last_status = detail;
        }
        (probe.step, probe.completed)
    };
    assert!(
        !status.starts_with("Network error:") && !status.starts_with("Disconnected from"),
        "{status}"
    );
    match step {
        Step::Create if state == ClientState::CharacterSelect && !loading => {
            assert_eq!(
                world.resource::<RuntimeStatus>().roster.characters.len(),
                completed,
                "probe requires a fresh isolated account"
            );
            let slot = world
                .resource::<CharacterSelectionUiModel>()
                .first_creatable_protocol_slot()
                .unwrap();
            assert_eq!(slot, completed + 1);
            world
                .resource_mut::<CharacterSelectionUiOutbox>()
                .push(CharacterSelectionUiAction::CreateCharacter { slot });
            world.resource_mut::<Probe>().step = Step::Name;
        }
        Step::Name if state == ClientState::CharacterCreate && !loading => {
            world.resource_mut::<CharacterCreationUiOutbox>().push(
                CharacterCreationUiAction::SubmitCustomName(CustomCharacterName {
                    first: if completed == 0 { "Bugone" } else { "Bugtwo" }.into(),
                    last: "Session".into(),
                }),
            );
            world.resource_mut::<Probe>().step = Step::Appearance;
        }
        Step::Appearance
            if state == ClientState::CharacterCreate
                && !loading
                && world.resource::<CharacterCreationUiModel>().screen
                    == CharacterCreationScreen::Appearance
                && matches!(
                    world.resource::<NativePlayerPreviewModel>().status,
                    NativePlayerPreviewStatus::ReadyAnimated { .. }
                ) =>
        {
            let appearance = world
                .resource::<CharacterCreationUiModel>()
                .appearance
                .clone();
            world
                .resource_mut::<CharacterCreationUiOutbox>()
                .push(CharacterCreationUiAction::ConfirmAppearance(appearance));
            world.resource_mut::<Probe>().step = Step::Tutorial;
        }
        Step::Tutorial if state == ClientState::Tutorial && !loading => {
            // Test only the character-session boundary, not tutorial mission acceptance.
            let uid = world
                .resource::<RuntimeStatus>()
                .roster
                .selected_uid
                .unwrap();
            world
                .resource::<NetworkBridge>()
                .send(NetworkCommand::CompleteTutorial { pc_uid: uid })
                .unwrap();
            world.resource_mut::<Probe>().step = Step::World;
        }
        Step::World if state == ClientState::World && !loading => {
            if !equipment::prepare_return(world) {
                return;
            }
            world.resource_mut::<QuitMenuUiModel>().open();
            world.resource_mut::<Probe>().step = Step::Menu;
        }
        Step::Returning if state == ClientState::CharacterSelect && !loading => {
            equipment::verify_selection(world);
            let model = world.resource::<CharacterSelectionUiModel>();
            for slot in &model.slots[..completed + 1] {
                let character = slot.occupied().unwrap();
                assert!(
                    !character.location_label().contains("UNKNOWN"),
                    "{:?}",
                    character
                );
                let expected_location =
                    if world.contains_resource::<equipment::EquipmentCheck>() {
                        "CANDY COVE - THE FUTURE"
                    } else {
                        "SECTOR V - THE FUTURE"
                    };
                assert_eq!(character.location_label(), expected_location);
            }
            assert!(model.create.enabled());
            assert_eq!(model.first_creatable_protocol_slot(), Some(completed + 2));
            assert!(!world.resource::<QuitMenuUiModel>().visible);
            let output = world.resource::<Probe>().output.clone();
            world.spawn(Screenshot::primary_window()).observe(
                bevy::render::view::screenshot::save_to_disk(
                    output.join(format!("selection-{}.png", completed + 1)),
                ),
            );
            let mut probe = world.resource_mut::<Probe>();
            probe.completed += 1;
            probe.step = if probe.completed == 2 {
                Step::Capture
            } else {
                Step::Create
            };
        }
        Step::Capture => {
            let output = world.resource::<Probe>().output.clone();
            // Exit after the final screenshot is actually saved.
            world.spawn(Screenshot::primary_window())
                .observe(bevy::render::view::screenshot::save_to_disk(output.join("selection-final.png")))
                .observe(|_: On<ScreenshotCaptured>, probe: Res<Probe>, mut exit: MessageWriter<AppExit>| {
                    fs::write(probe.output.join("passed.txt"), "Two characters created, played and returned in one login session; roster and locations verified.\n").unwrap();
                    exit.write(AppExit::Success);
                });
            world.resource_mut::<Probe>().step = Step::World;
        }
        _ => {}
    }
}
