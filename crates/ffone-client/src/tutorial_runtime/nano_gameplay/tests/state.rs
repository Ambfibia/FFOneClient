use super::*;

#[test]
fn genuine_equip_clears_stale_assets_and_blocked_state_without_a_live_entity() {
    let mut app = mechanics_app();
    let previous = WorldNanoGameplayPresentation {
        model_path: "characters/nanos/nano_previous.glb".to_owned(),
        style: 1,
        skill_slot: 1,
    };
    *app.world_mut().resource_mut::<TutorialNanoGameplayState>() = TutorialNanoGameplayState {
        loadout: Some(TutorialNanoGameplayLoadout {
            nano_id: 2,
            skill_id: 7,
        }),
        world_presentation: Some(previous),
        stamina: 80,
        status: TutorialNanoGameplayStatus::Blocked("old asset contract".to_owned()),
        ..default()
    };
    let (_, nodes) = AnimationGraph::from_clips([Handle::<AnimationClip>::default()]);
    {
        let mut assets = app.world_mut().resource_mut::<TutorialNanoGameplayAssets>();
        assets.gltf = Some(Handle::<Gltf>::default());
        assets.scene = Some(Handle::<WorldAsset>::default());
        assets.graph = Some(Handle::<AnimationGraph>::default());
        assets.nodes.insert(SKILL_CLIP, nodes[0]);
    }
    let replacement = WorldNanoGameplayPresentation {
        model_path: "characters/nanos/nano_replacement.glb".to_owned(),
        style: 2,
        skill_slot: 3,
    };
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayCommandQueue>()
        .equip_world(3, 19, 90, replacement.clone());

    app.update();

    let state = app.world().resource::<TutorialNanoGameplayState>();
    assert_eq!(
        state.loadout(),
        Some(TutorialNanoGameplayLoadout {
            nano_id: 3,
            skill_id: 19,
        })
    );
    assert_eq!(state.world_presentation(), Some(&replacement));
    assert!(matches!(state.status(), TutorialNanoGameplayStatus::Absent));
    assert_eq!(state.entity(), None);
    let assets = app.world().resource::<TutorialNanoGameplayAssets>();
    assert!(assets.gltf.is_none());
    assert!(assets.scene.is_none());
    assert!(assets.graph.is_none());
    assert!(assets.nodes.is_empty());
}

#[test]
fn world_skill_prediction_waits_for_call_then_plays_the_selected_skill() {
    let mut app = mechanics_app();
    let owner = app.world_mut().spawn(Transform::IDENTITY).id();
    activate_for_test(&mut app, owner);
    {
        let mut state = app.world_mut().resource_mut::<TutorialNanoGameplayState>();
        state.status = TutorialNanoGameplayStatus::Loading;
        state.world_presentation = Some(WorldNanoGameplayPresentation {
            model_path: "characters/nanos/nano_loading.glb".to_owned(),
            style: 2,
            skill_slot: 3,
        });
        state.request_clip(CALL_CLIP);
    }
    app.world_mut()
        .resource_mut::<TutorialNanoGameplayCommandQueue>()
        .play_world_skill(owner);

    app.update();

    let state = app.world().resource::<TutorialNanoGameplayState>();
    assert_eq!(state.skill_cooldown_remaining_fraction(), None);
    assert_eq!(state.skill_cooldown_remaining_seconds(), 0.0);
    assert_eq!(state.animation.mode(), LegacyNanoAnimationMode::Call);
    assert_eq!(state.animation.clip(), Some(CALL_CLIP));

    let mut random = LegacyNanoStandRandomStream::with_seed(11);
    let completion = app
        .world_mut()
        .resource_mut::<TutorialNanoGameplayState>()
        .animation
        .complete(&mut random);
    assert_eq!(completion, LegacyNanoCompletion::Continue);
    let state = app.world().resource::<TutorialNanoGameplayState>();
    assert_eq!(state.animation.mode(), LegacyNanoAnimationMode::Skill3);
    assert_eq!(state.animation.clip(), Some("skill3"));
}
