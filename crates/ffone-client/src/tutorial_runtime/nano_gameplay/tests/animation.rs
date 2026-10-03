use super::*;

#[test]
fn summon_stages_effect_before_model_and_activation_waits_for_a_real_animation_player() {
    let mut app = asset_app();
    let owner = app
        .world_mut()
        .spawn(Transform::from_xyz(1.0, 2.0, 3.0))
        .id();
    {
        let mut queue = app
            .world_mut()
            .resource_mut::<TutorialNanoGameplayCommandQueue>();
        queue.equip(
            TUTORIAL_BUTTERCUP_NANO_ID,
            TUTORIAL_BUTTERCUP_SKILL_ID,
            TUTORIAL_BUTTERCUP_INITIAL_STAMINA,
        );
        queue.summon(owner);
    }
    app.update();
    let root = app
        .world()
        .resource::<TutorialNanoGameplayState>()
        .entity()
        .unwrap();
    assert_eq!(
        app.world().get::<Visibility>(root),
        Some(&Visibility::Inherited)
    );
    assert!(matches!(
        app.world().resource::<TutorialNanoGameplayState>().status(),
        TutorialNanoGameplayStatus::Loading
    ));
    assert_eq!(
        app.world_mut()
            .resource_mut::<TutorialNanoGameplayEventQueue>()
            .take_all(),
        VecDeque::from([TutorialNanoGameplayEvent::EffectRequested {
            effect_id: TUTORIAL_BUTTERCUP_SUMMON_EFFECT_ID,
            position: Vec3::new(1.7, 3.12, 3.0),
            rotation: Quat::IDENTITY,
            source_line: 49,
        }])
    );
    assert!(app.world().get::<Transform>(root).is_some_and(|transform| {
        transform
            .translation
            .abs_diff_eq(Vec3::new(1.7, 3.12, 3.0), 0.000_01)
    }));
    let scene_transform = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<&Transform, With<TutorialGameplayNanoScene>>();
        *query.single(world).expect("one gameplay Nano scene")
    };
    assert!(scene_transform.rotation.abs_diff_eq(
        native_scene_container_transform(NativeSceneRole::CharacterGameplay).rotation,
        0.000_01,
    ));
    let scene_visibility = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<&Visibility, With<TutorialGameplayNanoScene>>();
        *query.single(world).expect("one gameplay Nano scene")
    };
    assert_eq!(scene_visibility, Visibility::Hidden);

    app.world_mut()
        .resource_mut::<TutorialNanoGameplayCommandQueue>()
        .summon(owner);
    app.update();
    let root_count = app
        .world_mut()
        .query::<&TutorialGameplayNanoRoot>()
        .iter(app.world())
        .count();
    assert_eq!(root_count, 1);
    assert_eq!(
        app.world().resource::<TutorialNanoGameplayState>().entity(),
        Some(root)
    );

    // Isolate the readiness gate from the installed Scene0: depending on
    // scheduling, the real asset can already prove its face texture during
    // this test. The following phase deliberately supplies clips and an
    // AnimationPlayer while withholding that one proof.
    let scene_entities = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<TutorialGameplayNanoScene>>();
        query.iter(world).collect::<Vec<_>>()
    };
    for entity in scene_entities {
        let _ = app.world_mut().despawn(entity);
    }
    {
        let mut state = app.world_mut().resource_mut::<TutorialNanoGameplayState>();
        state.status = TutorialNanoGameplayStatus::Loading;
        state.activated_generation = None;
        state.asset_contract_ready = false;
        state.face_texture_bound = false;
        state.request_clip(CALL_CLIP);
    }
    {
        let mut assets = app.world_mut().resource_mut::<TutorialNanoGameplayAssets>();
        assets.gltf = None;
        assets.scene = None;
        assets.graph = None;
        assets.nodes.clear();
    }
    let reveal_scene = app
        .world_mut()
        .spawn((TutorialGameplayNanoScene, Visibility::Hidden, ChildOf(root)))
        .id();
    let _ = app
        .world_mut()
        .resource_mut::<TutorialNanoGameplayEventQueue>()
        .take_all();

    let gltf = app
        .world_mut()
        .resource_mut::<Assets<Gltf>>()
        .add(gltf_with_gameplay_clips());
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayAssets>()
        .gltf = Some(gltf);
    app.world_mut()
        .spawn((AnimationPlayer::default(), ChildOf(root)));
    app.update();
    assert_eq!(
        app.world().get::<Visibility>(reveal_scene),
        Some(&Visibility::Inherited),
        "the renderable model becomes visible only when call playback is bound"
    );
    assert!(matches!(
        app.world().resource::<TutorialNanoGameplayState>().status(),
        TutorialNanoGameplayStatus::Loading
    ));
    assert!(
        app.world()
            .resource::<TutorialNanoGameplayEventQueue>()
            .is_empty(),
        "a real AnimationPlayer alone must not prove the exact face texture bind"
    );

    app.world_mut()
        .resource_mut::<TutorialNanoGameplayState>()
        .face_texture_bound = true;
    app.update();
    assert!(matches!(
        app.world().resource::<TutorialNanoGameplayState>().status(),
        TutorialNanoGameplayStatus::Ready
    ));
    assert!(matches!(
        app.world_mut()
            .resource_mut::<TutorialNanoGameplayEventQueue>()
            .take_all()
            .as_slices()
            .0,
        [TutorialNanoGameplayEvent::Activated { entity, .. }] if *entity == root
    ));
    app.update();
    assert!(
        app.world()
            .resource::<TutorialNanoGameplayEventQueue>()
            .is_empty(),
        "one proven generation must emit Activated exactly once"
    );
}

#[test]
fn swampfire_summon_and_missing_idle_use_the_production_clip_contract() {
    let model_path = "characters/nanos/nano_swampfire/nano_swampfire.glb";
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/game")
            .join(model_path),
    )
    .unwrap();
    let source = gltf::Gltf::from_slice(&bytes).unwrap();
    let names = source
        .animations()
        .filter_map(|clip| clip.name())
        .collect::<Vec<_>>();
    assert!(names.contains(&"stand1"));
    assert!(!names.contains(&"stand2"));
    assert!(names.contains(&"stand3"));

    for skill_slot in 1..=3 {
        let mut app = asset_app();
        let owner = app.world_mut().spawn(Transform::IDENTITY).id();
        let root = activate_for_test(&mut app, owner);
        app.world_mut()
            .entity_mut(root)
            .insert(TutorialNanoMovementPattern::new(
                &Transform::IDENTITY,
                0.0,
                &mut LegacyNanoStandRandomStream::default(),
            ));
        {
            let mut state = app.world_mut().resource_mut::<TutorialNanoGameplayState>();
            state.status = TutorialNanoGameplayStatus::Loading;
            state.loadout = Some(TutorialNanoGameplayLoadout {
                nano_id: 30,
                skill_id: 1,
            });
            state.world_presentation = Some(WorldNanoGameplayPresentation {
                model_path: model_path.to_owned(),
                style: 1,
                skill_slot,
            });
            state.request_clip(CALL_CLIP);
        }
        let mut gltf = gltf_with_gameplay_clips();
        gltf.named_animations.clear();
        for name in &names {
            let mut clip = AnimationClip::default();
            clip.set_duration(10.0);
            let handle = app
                .world_mut()
                .resource_mut::<Assets<AnimationClip>>()
                .add(clip);
            gltf.named_animations.insert((*name).into(), handle);
        }
        let handle = app.world_mut().resource_mut::<Assets<Gltf>>().add(gltf);
        app.world_mut()
            .resource_mut::<TutorialNanoGameplayAssets>()
            .gltf = Some(handle);
        let scene = app
            .world_mut()
            .spawn((TutorialGameplayNanoScene, Visibility::Hidden, ChildOf(root)))
            .id();
        let player = app
            .world_mut()
            .spawn((AnimationPlayer::default(), ChildOf(root)))
            .id();
        app.update();
        assert!(
            app.world()
                .resource::<TutorialNanoGameplayState>()
                .asset_contract_ready,
            "{:?}",
            app.world().resource::<TutorialNanoGameplayState>().status()
        );
        assert_eq!(
            app.world().get::<Visibility>(scene),
            Some(&Visibility::Inherited)
        );
        assert_eq!(
            app.world()
                .get::<TutorialGameplayNanoAnimationPlayback>(player)
                .unwrap()
                .clip,
            "call"
        );

        for (requested, expected) in [("stand2", "stand1"), ("stand3", "stand3")] {
            app.world_mut()
                .resource_mut::<TutorialNanoGameplayState>()
                .animation
                .request(
                    LegacyNanoAnimationMode::Stand,
                    requested,
                    LegacyAnimationBlend::CrossFade100Ms,
                );
            app.update();
            let state = app.world().resource::<TutorialNanoGameplayState>();
            assert_eq!(state.animation.clip(), Some(expected));
            assert_eq!(
                app.world()
                    .get::<TutorialGameplayNanoAnimationPlayback>(player)
                    .unwrap()
                    .clip,
                expected
            );
        }
    }
}

#[test]
fn world_skill_prediction_only_starts_animation_without_changing_server_state() {
    let mut app = mechanics_app();
    let owner = app.world_mut().spawn(Transform::IDENTITY).id();
    activate_for_test(&mut app, owner);
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayCommandQueue>()
        .play_world_skill(owner);

    app.update();

    let state = app.world().resource::<TutorialNanoGameplayState>();
    assert_eq!(state.stamina(), TUTORIAL_BUTTERCUP_INITIAL_STAMINA);
    assert_eq!(state.skill_cooldown_remaining_fraction(), None);
    assert_eq!(state.skill_cooldown_remaining_seconds(), 0.0);
    assert_eq!(state.animation.mode(), LegacyNanoAnimationMode::Skill1);
    assert!(
        app.world()
            .resource::<TutorialNanoGameplayEventQueue>()
            .is_empty(),
        "ordinary-world prediction must not synthesize damage or protocol state"
    );
}

#[test]
fn ordinary_world_presentation_uses_its_own_skill_clip_without_owning_cooldown() {
    let mut app = mechanics_app();
    let owner = app.world_mut().spawn(Transform::IDENTITY).id();
    activate_for_test(&mut app, owner);
    {
        let mut state = app.world_mut().resource_mut::<TutorialNanoGameplayState>();
        state.loadout = Some(TutorialNanoGameplayLoadout {
            nano_id: 2,
            skill_id: 19,
        });
        state.world_presentation = Some(WorldNanoGameplayPresentation {
            model_path: "characters/nanos/nano_2.glb".to_owned(),
            style: 4,
            skill_slot: 3,
        });
    }
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayCommandQueue>()
        .play_world_skill(owner);

    app.update();

    let state = app.world().resource::<TutorialNanoGameplayState>();
    assert_eq!(state.skill_cooldown_remaining_seconds(), 0.0);
    assert_eq!(state.animation.mode(), LegacyNanoAnimationMode::Skill3);
    assert_eq!(state.animation.clip(), Some("skill3"));
    assert_eq!(state.stamina(), TUTORIAL_BUTTERCUP_INITIAL_STAMINA);
}
