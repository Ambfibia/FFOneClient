use super::*;

#[test]
fn face_binding_error_despawns_loading_root_without_a_false_dismissal() {
    let mut app = asset_app();
    let owner = app.world_mut().spawn(Transform::IDENTITY).id();
    let root = app
        .world_mut()
        .spawn((
            TutorialGameplayNanoRoot {
                owner,
                generation: 1,
            },
            Transform::IDENTITY,
        ))
        .id();
    let params = LegacyModelMaterialParams::for_shader(LegacyShaderKind::SkinnedToon);
    let pass = params.render_plan().passes[0];
    let material = params
        .material_for_pass(pass, &LegacyModelTextures::default())
        .expect("surface pass produces a gameplay material");
    let material = app
        .world_mut()
        .resource_mut::<Assets<LegacyModelMaterial>>()
        .add(material);
    let surface = app
        .world_mut()
        .spawn((
            MeshMaterial3d(material),
            PendingLegacyModelMaterial {
                true_name: TUTORIAL_NANO_FACE_MATERIAL_NAME.into(),
                serialized_shader_name: LegacyShaderKind::SkinnedToon.exact_name().into(),
                declared_shader_name: LegacyShaderKind::SkinnedToon.exact_name().into(),
                params,
                shader_texture_defaults: Vec::new(),
                texture_bindings: Vec::new(),
                source_render_queue: 2_900,
                source_passes: Vec::new(),
                source_pass_count: 0,
            },
            ChildOf(root),
        ))
        .id();
    *app.world_mut().resource_mut::<TutorialNanoGameplayState>() = TutorialNanoGameplayState {
        loadout: Some(TutorialNanoGameplayLoadout {
            nano_id: TUTORIAL_BUTTERCUP_NANO_ID,
            skill_id: TUTORIAL_BUTTERCUP_SKILL_ID,
        }),
        entity: Some(root),
        owner: Some(owner),
        stamina: TUTORIAL_BUTTERCUP_INITIAL_STAMINA,
        status: TutorialNanoGameplayStatus::Loading,
        generation: 1,
        asset_contract_ready: true,
        ..default()
    };

    app.update();

    let state = app.world().resource::<TutorialNanoGameplayState>();
    assert!(matches!(
        state.status(),
        TutorialNanoGameplayStatus::Blocked(_)
    ));
    assert_eq!(state.entity(), None);
    assert_eq!(state.owner(), None);
    assert!(app.world().get_entity(root).is_err());
    assert!(app.world().get_entity(surface).is_err());
    assert!(
        app.world()
            .resource::<TutorialNanoGameplayEventQueue>()
            .is_empty(),
        "a loading Nano was never active and must not emit Dismissed"
    );
    assert!(matches!(
        app.world_mut()
            .resource_mut::<TutorialNanoGameplayIssueQueue>()
            .take_all()
            .as_slices()
            .0,
        [TutorialNanoGameplayIssue::AssetBlocked(error)]
            if error.contains("has no typed _MainTex binding")
    ));

    app.world_mut()
        .resource_mut::<TutorialNanoGameplayCommandQueue>()
        .summon(owner);
    app.update();

    assert!(matches!(
        app.world().resource::<TutorialNanoGameplayState>().status(),
        TutorialNanoGameplayStatus::Blocked(_)
    ));
    assert_eq!(
        app.world().resource::<TutorialNanoGameplayState>().entity(),
        None
    );
    assert!(
        app.world()
            .resource::<TutorialNanoGameplayEventQueue>()
            .is_empty(),
        "an unchanged blocked contract must not replay its summon effect"
    );
    assert!(
        app.world()
            .resource::<TutorialNanoGameplayIssueQueue>()
            .is_empty(),
        "an unchanged blocked contract must not repeat the same issue"
    );
    let roots = app
        .world_mut()
        .query::<&TutorialGameplayNanoRoot>()
        .iter(app.world())
        .count();
    assert_eq!(roots, 0);
}
