use super::*;

#[test]
fn skill1_emits_its_exact_tagged_effect_sfx_and_randomized_voice_at_quarter_second() {
    let mut app = App::new();
    app.init_resource::<TutorialNanoGameplayState>()
        .init_resource::<TutorialNanoGameplayEventQueue>()
        .insert_resource(LegacyNanoStandRandomStream::with_seed(7))
        .add_systems(Update, emit_tutorial_nano_animation_events);
    let owner = app.world_mut().spawn(Transform::IDENTITY).id();
    let root = app
        .world_mut()
        .spawn((
            TutorialGameplayNanoRoot {
                owner,
                generation: 1,
            },
            TutorialNanoSkillAnimationEvents::default(),
            Transform::from_xyz(1.0, 2.0, 3.0),
        ))
        .id();
    let (_, nodes) = AnimationGraph::from_clips([Handle::<AnimationClip>::default()]);
    let node = nodes[0];
    let mut player = AnimationPlayer::default();
    player
        .play(node)
        .set_seek_time(TUTORIAL_BUTTERCUP_SKILL_EVENT_SECONDS);
    let player = app
        .world_mut()
        .spawn((
            player,
            TutorialGameplayNanoAnimationPlayback {
                generation: 1,
                request_serial: 1,
                clip: SKILL_CLIP,
                node,
            },
            ChildOf(root),
        ))
        .id();
    let mut animation = LegacyNanoAnimationMachine::default();
    animation.request(
        LegacyNanoAnimationMode::Skill1,
        SKILL_CLIP,
        LegacyAnimationBlend::CrossFade100Ms,
    );
    *app.world_mut().resource_mut::<TutorialNanoGameplayState>() = TutorialNanoGameplayState {
        loadout: Some(TutorialNanoGameplayLoadout {
            nano_id: TUTORIAL_BUTTERCUP_NANO_ID,
            skill_id: TUTORIAL_BUTTERCUP_SKILL_ID,
        }),
        entity: Some(root),
        owner: Some(owner),
        generation: 1,
        animation,
        ..default()
    };

    app.update();

    assert!(app.world().get_entity(player).is_ok());
    let emitted = app
        .world_mut()
        .resource_mut::<TutorialNanoGameplayEventQueue>()
        .take_all();
    assert_eq!(emitted.len(), 3);
    assert!(matches!(
        emitted[0],
        TutorialNanoGameplayEvent::TaggedEffectRequested {
            effect_id: TUTORIAL_BUTTERCUP_SKILL_EFFECT_ID,
            root: event_root,
            node_name: TUTORIAL_BUTTERCUP_SKILL_EFFECT_NODE,
            source_clip_path_id: 1050,
            source_event_seconds: TUTORIAL_BUTTERCUP_SKILL_EVENT_SECONDS,
        } if event_root == root
    ));
    assert!(matches!(
        emitted[1],
        TutorialNanoGameplayEvent::AudioRequested {
            true_name: TUTORIAL_BUTTERCUP_SKILL_SFX_TRUE_NAME,
            category: TutorialNanoGameplayAudioCategory::Sfx,
            position,
            source_clip_path_id: 1050,
            source_event_seconds: TUTORIAL_BUTTERCUP_SKILL_EVENT_SECONDS,
        } if position == Vec3::new(1.0, 2.0, 3.0)
    ));
    assert!(matches!(
        emitted[2],
        TutorialNanoGameplayEvent::AudioRequested {
            true_name,
            category: TutorialNanoGameplayAudioCategory::Voice,
            source_clip_path_id: 1050,
            source_event_seconds: TUTORIAL_BUTTERCUP_SKILL_EVENT_SECONDS,
            ..
        } if TUTORIAL_BUTTERCUP_SKILL_VOICE_TRUE_NAMES.contains(&true_name)
    ));
    app.update();
    assert!(
        app.world()
            .resource::<TutorialNanoGameplayEventQueue>()
            .is_empty()
    );
}

#[test]
fn ordinary_world_nano_replays_its_own_glb_voice_event_once() {
    let mut app = App::new();
    app.init_resource::<TutorialNanoGameplayState>()
        .init_resource::<TutorialNanoGameplayAssets>()
        .init_resource::<GameplayAudioRuntime>()
        .init_resource::<LegacyNanoStandRandomStream>()
        .add_systems(Update, emit_world_nano_animation_sounds);
    let owner = app.world_mut().spawn(Transform::IDENTITY).id();
    let root = app
        .world_mut()
        .spawn(TutorialGameplayNanoRoot {
            owner,
            generation: 1,
        })
        .id();
    let (_, nodes) = AnimationGraph::from_clips([Handle::<AnimationClip>::default()]);
    let node = nodes[0];
    let mut player = AnimationPlayer::default();
    player.play(node).set_seek_time(0.1);
    app.world_mut().spawn((
        player,
        TutorialGameplayNanoAnimationPlayback {
            generation: 1,
            request_serial: 1,
            clip: CALL_CLIP,
            node,
        },
        ChildOf(root),
    ));
    *app.world_mut().resource_mut::<TutorialNanoGameplayState>() = TutorialNanoGameplayState {
        loadout: Some(TutorialNanoGameplayLoadout {
            nano_id: 2,
            skill_id: 4,
        }),
        world_presentation: Some(WorldNanoGameplayPresentation {
            model_path: "characters/nanos/nano_bloo/nano_bloo.glb".to_owned(),
            style: 1,
            skill_slot: 1,
        }),
        entity: Some(root),
        owner: Some(owner),
        generation: 1,
        ..default()
    };
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayAssets>()
        .sound_events = Some(
        vec![NetworkNpcAnimationSoundEvent0104 {
            clip: CALL_CLIP.to_owned(),
            time: 0.1,
            payload: "Bloo_NanSummon0(RAND:1-3).wav".to_owned(),
        }]
        .into(),
    );

    app.update();
    let queued = app
        .world()
        .resource::<GameplayAudioRuntime>()
        .queued_animation_sound_names();
    assert_eq!(queued.len(), 1);
    assert!(queued[0].0.starts_with("Bloo_NanSummon0"));
    assert!(queued[0].1);

    app.update();
    assert_eq!(
        app.world()
            .resource::<GameplayAudioRuntime>()
            .queued_animation_sound_names()
            .len(),
        1,
        "the animation cursor must not replay a crossed voice event every frame"
    );
}

#[test]
fn call_emits_exact_summon_sfx_and_randomized_voice_at_quarter_second() {
    let mut app = App::new();
    app.init_resource::<TutorialNanoGameplayState>()
        .init_resource::<TutorialNanoGameplayEventQueue>()
        .insert_resource(LegacyNanoStandRandomStream::with_seed(9))
        .add_systems(Update, emit_tutorial_nano_animation_events);
    let owner = app.world_mut().spawn(Transform::IDENTITY).id();
    let root = app
        .world_mut()
        .spawn((
            TutorialGameplayNanoRoot {
                owner,
                generation: 1,
            },
            TutorialNanoSkillAnimationEvents::default(),
            Transform::from_xyz(1.0, 2.0, 3.0),
        ))
        .id();
    let (_, nodes) = AnimationGraph::from_clips([Handle::<AnimationClip>::default()]);
    let node = nodes[0];
    let mut player = AnimationPlayer::default();
    player
        .play(node)
        .set_seek_time(TUTORIAL_BUTTERCUP_SUMMON_EVENT_SECONDS);
    app.world_mut().spawn((
        player,
        TutorialGameplayNanoAnimationPlayback {
            generation: 1,
            request_serial: 1,
            clip: CALL_CLIP,
            node,
        },
        ChildOf(root),
    ));
    let mut animation = LegacyNanoAnimationMachine::default();
    animation.request(
        LegacyNanoAnimationMode::Call,
        CALL_CLIP,
        LegacyAnimationBlend::CrossFade100Ms,
    );
    *app.world_mut().resource_mut::<TutorialNanoGameplayState>() = TutorialNanoGameplayState {
        loadout: Some(TutorialNanoGameplayLoadout {
            nano_id: TUTORIAL_BUTTERCUP_NANO_ID,
            skill_id: TUTORIAL_BUTTERCUP_SKILL_ID,
        }),
        entity: Some(root),
        owner: Some(owner),
        generation: 1,
        animation,
        ..default()
    };

    app.update();

    let emitted = app
        .world_mut()
        .resource_mut::<TutorialNanoGameplayEventQueue>()
        .take_all();
    assert_eq!(emitted.len(), 2);
    assert!(matches!(
        emitted[0],
        TutorialNanoGameplayEvent::AudioRequested {
            true_name,
            category: TutorialNanoGameplayAudioCategory::Voice,
            source_clip_path_id: 1049,
            source_event_seconds: TUTORIAL_BUTTERCUP_SUMMON_EVENT_SECONDS,
            ..
        } if TUTORIAL_BUTTERCUP_SUMMON_VOICE_TRUE_NAMES.contains(&true_name)
    ));
    assert!(matches!(
        emitted[1],
        TutorialNanoGameplayEvent::AudioRequested {
            true_name: TUTORIAL_BUTTERCUP_SUMMON_SFX_TRUE_NAME,
            category: TutorialNanoGameplayAudioCategory::Sfx,
            source_clip_path_id: 1049,
            source_event_seconds: TUTORIAL_BUTTERCUP_SUMMON_EVENT_SECONDS,
            ..
        }
    ));
}
