use super::*;

#[test]
fn tutorial_voice_cancellation_includes_deferred_spawns_and_keeps_replacement() {
    use bevy::ecs::system::RunSystemOnce;
    let mut world = World::new();
    world.spawn(TutorialVoiceAudio);
    world
        .run_system_once(
            |mut commands: Commands, voices: Query<Entity, With<TutorialVoiceAudio>>| {
                commands.spawn((TutorialVoiceAudio, Name::new("queued obsolete voice")));
                stop_tutorial_voice(&mut commands, &voices);
                commands.spawn((TutorialVoiceAudio, Name::new("current voice")));
            },
        )
        .unwrap();
    let remaining: Vec<_> = world
        .query_filtered::<&Name, With<TutorialVoiceAudio>>()
        .iter(&world)
        .map(|name| name.as_str().to_owned())
        .collect();
    assert_eq!(remaining, ["current voice"]);
    world
        .run_system_once(
            |mut commands: Commands, voices: Query<Entity, With<TutorialVoiceAudio>>| {
                commands.spawn(TutorialVoiceAudio);
                stop_tutorial_voice(&mut commands, &voices);
            },
        )
        .unwrap();
    assert_eq!(
        world
            .query_filtered::<Entity, With<TutorialVoiceAudio>>()
            .iter(&world)
            .count(),
        0
    );
}

#[test]
fn stopping_a_tutorial_voice_keeps_npc_mode_until_explicit_exit_ui() {
    let mut runtime = TutorialMissionRuntime::default();
    let mut actor_commands = TutorialActorCommandQueue::default();
    let mut logic = TutorialLogicRuntime::default();
    logic.ui.npc_icon_mode_visible = true;

    assert!(apply_tutorial_dialogue_intent(
        TutorialIntent::StartDialogue(TutorialDialogue::TalkNumbuhTwoMission),
        &mut runtime,
        &mut actor_commands,
        &mut logic,
    ));
    assert!(apply_tutorial_dialogue_intent(
        TutorialIntent::StopDialogue(TutorialDialogue::TalkNumbuhTwoMission),
        &mut runtime,
        &mut actor_commands,
        &mut logic,
    ));
    assert!(runtime.dialogues.is_empty());
    assert!(runtime.auxiliary_dialogue.is_none());
    assert!(actor_commands.is_empty());
    assert!(logic.ui.npc_icon_mode_visible);

    assert!(apply_tutorial_dialogue_intent(
        TutorialIntent::ExitUi,
        &mut runtime,
        &mut actor_commands,
        &mut logic,
    ));
    assert!(!logic.ui.npc_icon_mode_visible);
    assert_eq!(
        actor_commands.take_all().into_iter().collect::<Vec<_>>(),
        vec![ffone_client::tutorial_actors::TutorialActorCommand::ClearInteractions]
    );
}

#[test]
fn every_tutorial_cutscene_effect_resolves_to_a_real_semantic_sfx_file() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = NativeAudioCatalog::open(&asset_root, false).unwrap();
    let mut effect_count = 0;

    for presentation in TUTORIAL_SCENE_PRESENTATIONS {
        for cue in presentation.audio.iter().filter(|cue| {
            matches!(
                cue.kind,
                TutorialAudioCueKind::OneShot | TutorialAudioCueKind::StartLoop
            )
        }) {
            let asset = tutorial_sound_asset(&catalog, cue).unwrap_or_else(|error| {
                panic!("{:?} cue {:?}: {error}", presentation.scene, cue.cue)
            });
            let path = tutorial_sound_path_for_locale(&catalog, asset, "en").unwrap_or_else(|| {
                panic!("{:?} cue {:?} has no en take", presentation.scene, cue.cue)
            });
            assert!(
                asset_root.join(&path).is_file(),
                "{:?} cue {:?} resolved to missing {path}",
                presentation.scene,
                cue.cue
            );
            effect_count += usize::from(!tutorial_audio_cue_is_music(cue.cue));
        }
    }

    assert!(
        effect_count >= 20,
        "the finite scenes must retain their recovered SFX timeline"
    );
}

#[test]
fn dexter_cutscene_completion_stops_every_scene_owned_audio_source() {
    fn stop_audio(
        mut commands: Commands,
        audio: Query<Entity, (With<DexterShipEntity>, With<AudioPlayer>)>,
    ) {
        stop_dexter_ship_audio(&mut commands, &audio);
    }

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, stop_audio);
    let voice = app
        .world_mut()
        .spawn((
            DexterShipEntity,
            AudioPlayer::new(Handle::<AudioSource>::default()),
        ))
        .id();
    let scene_visual = app.world_mut().spawn(DexterShipEntity).id();

    app.update();

    assert!(app.world().get_entity(voice).is_err());
    assert!(app.world().get_entity(scene_visual).is_ok());
}

#[test]
fn buttercup_animation_audio_resolves_through_catalog_for_both_voice_languages() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = NativeAudioCatalog::open(&asset_root, false).unwrap();
    for true_name in ["Nano Ability 06", "Stun_Root_GENERIC"] {
        let path = tutorial_nano_audio_path(
            &catalog,
            "en",
            true_name,
            TutorialNanoGameplayAudioCategory::Sfx,
        )
        .unwrap()
        .unwrap();
        assert!(
            asset_root.join(&path).is_file(),
            "catalog route for {true_name} is missing: {path}"
        );
    }
    for locale in ["en", "ru"] {
        for true_name in [
            "Btrcup_NanSummon01_01",
            "Btrcup_NanSummon01_02",
            "Btrcup_NanSummon01_03",
            "Btrcup_NanPwrMF01_01",
            "Btrcup_NanPwrMF01_02",
            "Btrcup_NanPwrMF01_03",
        ] {
            let path = tutorial_nano_audio_path(
                &catalog,
                locale,
                true_name,
                TutorialNanoGameplayAudioCategory::Voice,
            )
            .unwrap()
            .unwrap();
            assert!(
                asset_root.join(&path).is_file(),
                "{locale} catalog route for {true_name} is missing: {path}"
            );
        }
    }
}
