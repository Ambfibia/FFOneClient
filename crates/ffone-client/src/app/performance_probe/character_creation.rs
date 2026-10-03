//! Offline regression through production schedules, asset loaders and render systems.
use super::*;
use bevy::ecs::system::RunSystemOnce;

#[derive(Resource)]
struct CreationProbe {
    output: PathBuf,
    started: Instant,
    stage: u8,
    last_status: String,
    pending_captures: usize,
}

pub(super) fn install(app: &mut App, output: PathBuf) {
    fs::create_dir_all(&output).unwrap();
    app.insert_resource(CreationProbe {
        output,
        started: Instant::now(),
        stage: 0,
        last_status: String::new(),
        pending_captures: 0,
    })
    .insert_resource(bevy::winit::WinitSettings::continuous())
    .add_systems(PostStartup, |mut next: ResMut<NextState<ClientState>>| {
        next.set(ClientState::CharacterSelect);
    })
    .add_systems(Last, drive);
}

fn drive(world: &mut World) {
    let state = *world.resource::<State<ClientState>>().get();
    let loading = world.resource::<GameplayLoadingState>();
    let creator = world.resource::<CharacterCreationUiModel>();
    let preview = world.resource::<NativePlayerPreviewModel>();
    let ready = !loading.visible;
    let preview_ready = matches!(
        preview.status,
        NativePlayerPreviewStatus::ReadyAnimated { .. }
    );
    let status = format!(
        "state={state:?} loading={:?}/{:?} progress={:.3} blocked={:?} creator={:?}/{:?} preview={:?} detail={:?}",
        loading.scope,
        loading.phase,
        loading.overall_progress,
        loading.blocked,
        creator.screen,
        creator.asset_status,
        preview.status,
        preview.loading_detail,
    );
    let stage = world.resource::<CreationProbe>().stage;
    {
        let mut probe = world.resource_mut::<CreationProbe>();
        if probe.last_status != status {
            println!("creation-probe stage={stage} {status}");
            probe.last_status = status.clone();
            fs::write(probe.output.join("status.txt"), &status).unwrap();
        }
        assert!(
            probe.started.elapsed() < Duration::from_secs(180),
            "creation probe timed out: {status}"
        );
    }
    match stage {
        0 if state == ClientState::CharacterSelect && ready => {
            let mut creator = world.resource_mut::<CharacterCreationUiModel>();
            *creator = CharacterCreationUiModel::default();
            creator.slot = Some(1);
            world
                .resource_mut::<NextState<ClientState>>()
                .set(ClientState::CharacterCreateIntro);
            world.resource_mut::<CreationProbe>().stage = 1;
        }
        1 if state == ClientState::CharacterCreate && ready => {
            capture(world, "name.png");
            world
                .run_system_once(
                    |mut session: ResMut<CharacterCreationSession>,
                     mut model: ResMut<CharacterCreationUiModel>,
                     mut loading: ResMut<GameplayLoadingState>,
                     mut runtime: ResMut<RuntimeStatus>| {
                        character_flow::accept_reserved_character_name(
                            CharacterNameSaveSuccess0104 {
                                pc_uid: 42,
                                slot: 1,
                                gender: 1,
                                first_name: ffone_protocol::FixedUtf16::from_str("Probe").unwrap(),
                                last_name: ffone_protocol::FixedUtf16::from_str("Creator").unwrap(),
                            },
                            &mut session,
                            &mut model,
                            &mut loading,
                            &mut runtime,
                        );
                    },
                )
                .unwrap();
            world.resource_mut::<CreationProbe>().stage = 2;
        }
        2 if ready && preview_ready => {
            capture(world, "appearance-random.png");
            let mut model = world.resource_mut::<CharacterCreationUiModel>();
            model.appearance = CharacterAppearance {
                gender: CharacterGender::Girl,
                skin_color: 36,
                hair_color: 54,
                eye_color: 10,
                ..default()
            };
            world
                .resource_mut::<GameplayLoadingState>()
                .begin(ResourceLoadingScope::CharacterCreation);
            world.resource_mut::<CreationProbe>().stage = 3;
        }
        3 if ready && preview_ready => {
            capture(world, "appearance-female-extended.png");
            world
                .resource_mut::<NextState<ClientState>>()
                .set(ClientState::CharacterSelect);
            world.resource_mut::<CreationProbe>().stage = 4;
        }
        4 if state == ClientState::CharacterSelect && ready => {
            // The same named, unfinished slot is resumed after its preview lease
            // has been released on exit. Reuse the production resume operation.
            let character = CharacterSummary {
                slot: 1,
                level: 1,
                pc_uid: 42,
                first_name: "Probe".into(),
                last_name: "Creator".into(),
                position: [0; 3],
                style: ffone_protocol::CharacterStyle0104 {
                    name_check: 0,
                    gender: 1,
                    face_style: 1,
                    hair_style: 1,
                    hair_color: 1,
                    skin_color: 1,
                    eye_color: 1,
                    height: 0,
                    body: 0,
                    class: 0,
                    appearance_flag: 0,
                    tutorial_flag: 0,
                    payzone_flag: 0,
                },
                equipment: [default(); ffone_protocol::CHARACTER_EQUIP_SLOT_COUNT_0104],
            };
            world
                .run_system_once(
                    move |mut session: ResMut<CharacterCreationSession>,
                          mut model: ResMut<CharacterCreationUiModel>| {
                        character_flow::resume_incomplete_character(
                            &character,
                            &mut session,
                            &mut model,
                        )
                        .unwrap();
                    },
                )
                .unwrap();
            world
                .resource_mut::<NextState<ClientState>>()
                .set(ClientState::CharacterCreate);
            world.resource_mut::<CreationProbe>().stage = 5;
        }
        5 if state == ClientState::CharacterCreate && ready && preview_ready => {
            capture(world, "appearance-resumed.png");
            world.resource_mut::<CreationProbe>().stage = 6;
        }
        6 if world.resource::<CreationProbe>().pending_captures == 0 => {
            fs::write(world.resource::<CreationProbe>().output.join("passed.txt"), "selection -> introduction -> name -> reserved identity -> animated appearance -> extended female palettes -> selection -> resumed appearance: passed\n").unwrap();
            world.write_message(AppExit::Success);
        }
        _ => {}
    }
}

fn capture(world: &mut World, name: &str) {
    let output = world.resource::<CreationProbe>().output.join(name);
    world.resource_mut::<CreationProbe>().pending_captures += 1;
    world
        .spawn(Screenshot::primary_window())
        .observe(bevy::render::view::screenshot::save_to_disk(output))
        .observe(
            |_: On<ScreenshotCaptured>, mut probe: ResMut<CreationProbe>| {
                probe.pending_captures -= 1;
            },
        );
}
