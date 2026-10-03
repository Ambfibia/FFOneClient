use super::*;

pub(super) fn gltf_with_gameplay_clips() -> Gltf {
    let mut gltf = Gltf {
        scenes: Vec::new(),
        named_scenes: Default::default(),
        meshes: Vec::new(),
        named_meshes: Default::default(),
        materials: Vec::new(),
        named_materials: Default::default(),
        nodes: Vec::new(),
        named_nodes: Default::default(),
        skins: Vec::new(),
        named_skins: Default::default(),
        default_scene: None,
        animations: Vec::new(),
        named_animations: Default::default(),
        source: None,
    };
    for name in GAMEPLAY_REQUIRED_CLIPS {
        let animation = Handle::<AnimationClip>::default();
        gltf.animations.push(animation.clone());
        gltf.named_animations.insert(name.into(), animation);
    }
    gltf
}

#[test]
fn ordinary_world_summon_uses_the_selected_native_model_contract_and_style_effect() {
    let mut app = asset_app();
    let owner = app.world_mut().spawn(Transform::IDENTITY).id();
    let presentation = WorldNanoGameplayPresentation {
        model_path: "characters/nanos/nano_test/nano_test.glb".to_owned(),
        style: 4,
        skill_slot: 3,
    };
    {
        let mut queue = app
            .world_mut()
            .resource_mut::<TutorialNanoGameplayCommandQueue>();
        queue.equip_world(2, 19, 77, presentation.clone());
        queue.summon(owner);
    }

    app.update();

    let state = app.world().resource::<TutorialNanoGameplayState>();
    assert_eq!(
        state.loadout(),
        Some(TutorialNanoGameplayLoadout {
            nano_id: 2,
            skill_id: 19,
        })
    );
    assert_eq!(state.world_presentation(), Some(&presentation));
    assert!(state.entity().is_some());
    assert!(!state.requires_exact_face_texture());
    assert_eq!(
        app.world_mut()
            .resource_mut::<TutorialNanoGameplayEventQueue>()
            .take_all(),
        VecDeque::from([TutorialNanoGameplayEvent::EffectRequested {
            effect_id: 531,
            position: Vec3::new(0.7, 1.12, 0.0),
            rotation: Quat::IDENTITY,
            source_line: 49,
        }])
    );
}
