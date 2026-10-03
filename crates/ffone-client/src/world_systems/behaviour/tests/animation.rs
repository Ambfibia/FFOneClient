use super::*;

#[test]
fn candy_cove_animation_bindings_keep_placements_near_their_authored_location() {
    let root_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game/map/tiles/map_08_06");
    let mut document: NativeWorldBehaviourDocument =
        serde_json::from_slice(&std::fs::read(root_path.join("behaviour.json")).unwrap())
            .unwrap();
    let scene: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root_path.join("scene.json")).unwrap()).unwrap();
    let visuals = scene["visuals"].as_array().unwrap();
    let octopus: Vec<_> = visuals
        .iter()
        .filter(|visual| {
            visual["name"]
                .as_str()
                .unwrap()
                .contains("ep_exsh_iumppad_octopus_01")
        })
        .collect();
    assert_eq!(octopus.len(), 2);
    for pad in octopus {
        assert!(
            !visuals.iter().any(|visual| {
                visual["name"]
                    .as_str()
                    .unwrap()
                    .contains("ep_h_object_jumppad01-standard_1")
                    && visual["transform"] == pad["transform"]
            }),
            "octopus jump pad must not intersect a standard metal shell"
        );
    }
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::transform::TransformPlugin));
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_millis(250),
    ));
    app.add_systems(
        Update,
        (update_world_animations, apply_world_animation_samples).chain(),
    );
    let root = app
        .world_mut()
        .spawn((Transform::IDENTITY, Visibility::Inherited))
        .id();
    let mut bindings = HashMap::new();
    let mut authored = HashMap::new();
    let mut placements = Vec::new();
    for visual in scene["visuals"].as_array().unwrap() {
        let trs: WorldAnimationTrs =
            serde_json::from_value(visual["transform"].clone()).unwrap();
        let transform = trs.transform();
        let entity = app
            .world_mut()
            .spawn((transform, ChildOf(root), Visibility::Inherited))
            .id();
        let id = legacy_static_model_id(visual["sourceModelPath"].as_str().unwrap()).unwrap();
        bindings.insert(id, vec![entity]);
        authored.insert(entity, transform.to_matrix());
        placements.push((
            entity,
            transform.translation,
            visual["name"].as_str().unwrap().to_owned(),
        ));
    }
    let clips = take_world_animation_clips(&mut document);
    let mut pending =
        PendingWorldBehaviourSpawn::new(Arc::new(document), clips, bindings, authored, 0, 0);
    while pending.phase != WorldBehaviourSpawnPhase::Complete {
        let mut queue = bevy::ecs::world::CommandQueue::default();
        streaming::materialize_world_behaviour_batch(
            &mut Commands::new(&mut queue, app.world()),
            root,
            &mut pending,
            WORLD_BEHAVIOUR_WORK_PER_ROOT_PER_FRAME,
        );
        queue.apply(app.world_mut());
        app.update();
    }
    for _ in 0..120 {
        app.update();
        let escaped: Vec<_> = placements
            .iter()
            .filter_map(|(entity, origin, name)| {
                let position = app
                    .world()
                    .get::<GlobalTransform>(*entity)
                    .unwrap()
                    .translation();
                (!position.is_finite() || position.distance(*origin) > 30.0)
                    .then(|| format!("{name}: {origin:?} -> {position:?}"))
            })
            .collect();
        assert!(escaped.is_empty(), "{}", escaped.join("\n"));
    }
}

#[test]
fn oversized_animation_records_are_materialized_across_multiple_batches() {
    let target = |index: usize| {
        serde_json::json!({
            "path": format!("target/{index}"),
            "node": format!("node-{index}"),
            "parentPath": (index == 0).then(|| {
                format!("target/{}", WORLD_ANIMATION_BINDINGS_PER_FRAME + 2)
            }),
            "animated": true,
            "baseLocalTrs": {
                "translation": [0.0, 0.0, 0.0],
                "rotation": [0.0, 0.0, 0.0, 1.0],
                "scale": [1.0, 1.0, 1.0]
            },
            "baseWorldMatrix": [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0]
            ]
        })
    };
    let mut document: NativeWorldBehaviourDocument =
        serde_json::from_value(serde_json::json!({
            "schema": NATIVE_WORLD_BEHAVIOUR_SCHEMA,
            "id": "oversized-animation",
            "scope": "world-map",
            "tile": [0, 0],
            "animations": [{
                "node": "animation",
                "enabled": true,
                "targets": (0..(WORLD_ANIMATION_BINDINGS_PER_FRAME + 3))
                    .map(target)
                    .collect::<Vec<_>>()
            }]
        }))
        .unwrap();
    let animation_clips = take_world_animation_clips(&mut document);
    let mut pending = PendingWorldBehaviourSpawn::new(
        Arc::new(document),
        animation_clips,
        HashMap::new(),
        HashMap::new(),
        0,
        0,
    );
    pending.phase = WorldBehaviourSpawnPhase::Animations;

    let mut world = World::new();
    let root = world.spawn_empty().id();
    let mut queue = bevy::ecs::world::CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        streaming::materialize_world_behaviour_batch(
            &mut commands,
            root,
            &mut pending,
            WORLD_ANIMATION_BINDINGS_PER_FRAME + 1,
        );
    }
    queue.apply(&mut world);

    assert_eq!(pending.next_record, 0);
    assert_eq!(
        pending
            .animation_record
            .as_ref()
            .expect("partial animation record")
            .next_target,
        WORLD_ANIMATION_BINDINGS_PER_FRAME
    );
    assert_eq!(
        world.query::<&WorldAnimationTarget>().iter(&world).count(),
        8
    );

    let mut queue = bevy::ecs::world::CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        streaming::materialize_world_behaviour_batch(
            &mut commands,
            root,
            &mut pending,
            WORLD_BEHAVIOUR_WORK_PER_ROOT_PER_FRAME,
        );
    }
    queue.apply(&mut world);

    assert_eq!(pending.phase, WorldBehaviourSpawnPhase::Complete);
    assert!(pending.animation_record.is_none());
    assert_eq!(
        world.query::<&WorldAnimationTarget>().iter(&world).count(),
        WORLD_ANIMATION_BINDINGS_PER_FRAME + 3
    );
    assert_eq!(
        world.query::<&WorldAnimationPlayer>().iter(&world).count(),
        1
    );
    let player = world
        .query::<&WorldAnimationPlayer>()
        .single(&world)
        .expect("one completed animation player");
    let child = player.targets["target/0"].entity;
    let parent =
        player.targets[&format!("target/{}", WORLD_ANIMATION_BINDINGS_PER_FRAME + 2)].entity;
    assert_eq!(world.get::<ChildOf>(child).unwrap().parent(), parent);
}

#[test]
fn animation_clip_compilation_is_budgeted_without_retaining_the_tile_document() {
    let channel_count = WORLD_ANIMATION_BINDINGS_PER_FRAME * 2;
    let channels = (0..channel_count)
        .map(|_| {
            serde_json::json!({
                "targetPath": "target/0",
                "property": "translation",
                "times": [0.0],
                "values": [[0.0, 0.0, 0.0]],
                "inTangents": [[0.0, 0.0, 0.0]],
                "outTangents": [[0.0, 0.0, 0.0]]
            })
        })
        .collect::<Vec<_>>();
    let mut document: NativeWorldBehaviourDocument =
        serde_json::from_value(serde_json::json!({
            "schema": NATIVE_WORLD_BEHAVIOUR_SCHEMA,
            "id": "budgeted-animation-clip",
            "scope": "world-map",
            "tile": [0, 0],
            "animations": [{
                "node": "animation",
                "enabled": true,
                "defaultClipId": "clip",
                "targets": [{
                    "path": "target/0",
                    "node": "target",
                    "animated": true,
                    "baseLocalTrs": {
                        "translation": [0.0, 0.0, 0.0],
                        "rotation": [0.0, 0.0, 0.0, 1.0],
                        "scale": [1.0, 1.0, 1.0]
                    },
                    "baseWorldMatrix": [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0]
                    ]
                }]
            }],
            "animationClips": [{
                "id": "clip",
                "name": "clip",
                "duration": 1.0,
                "sampleRate": 30.0,
                "looped": true,
                "channels": channels
            }]
        }))
        .unwrap();
    let animation_clips = take_world_animation_clips(&mut document);
    let document = Arc::new(document);
    let mut pending = PendingWorldBehaviourSpawn::new(
        Arc::clone(&document),
        animation_clips,
        HashMap::new(),
        HashMap::new(),
        0,
        0,
    );
    let shared_clip = Arc::as_ptr(&pending.animation_clips[0]);
    pending.phase = WorldBehaviourSpawnPhase::Animations;

    let mut world = World::new();
    let root = world.spawn_empty().id();
    let mut queue = bevy::ecs::world::CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        streaming::materialize_world_behaviour_batch(&mut commands, root, &mut pending, 3);
    }
    queue.apply(&mut world);

    assert!(pending.animation_record.is_some());
    assert_eq!(
        world.query::<&WorldAnimationPlayer>().iter(&world).count(),
        0
    );

    let mut queue = bevy::ecs::world::CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        streaming::materialize_world_behaviour_batch(
            &mut commands,
            root,
            &mut pending,
            WORLD_BEHAVIOUR_WORK_PER_ROOT_PER_FRAME,
        );
    }
    queue.apply(&mut world);

    let player = world
        .query::<&WorldAnimationPlayer>()
        .single(&world)
        .expect("one completed animation player");
    assert_eq!(
        Arc::strong_count(&document),
        2,
        "animation players must not retain the full decoded tile document"
    );
    assert_eq!(
        player.active_clip().unwrap() as *const WorldAnimationClip,
        shared_clip
    );
    let tracks = world
        .query::<&WorldAnimationCompiledTracks>()
        .single(&world)
        .expect("compiled animation tracks");
    assert_eq!(tracks.0.len(), 1);
    assert_eq!(tracks.0[0].translation, Some(channel_count - 1));

    let owning_document = pending.take_document();
    assert!(pending.document.is_none());
    drop(document);
    assert_eq!(Arc::strong_count(&owning_document), 1);

    let mut scripted = PendingWorldScriptedEffects {
        document: Some(owning_document),
        next_record: 0,
    };
    let owning_document = scripted.take_document();
    assert!(scripted.document.is_none());
    assert_eq!(Arc::strong_count(&owning_document), 1);
}

#[test]
fn animation_curve_sampling_uses_legacy_cubic_tangents() {
    let scalar = WorldAnimationFloatCurve {
        target_path: "animated".to_owned(),
        class_id: 21,
        attribute: "_Emission.r".to_owned(),
        pre_infinity: 2,
        post_infinity: 2,
        times: vec![0.0, 1.0],
        values: vec![0.0, 0.0],
        in_tangents: vec![0.0, 0.0],
        out_tangents: vec![2.0, 0.0],
    };
    assert!((sample_cubic_scalar(&scalar, 0.5).unwrap() - 0.25).abs() < f64::EPSILON);

    let vector = sample_cubic_vector(
        &[0.0, 1.0],
        &[vec![0.0, 1.0], vec![0.0, 3.0]],
        &[vec![0.0, 0.0], vec![0.0, 0.0]],
        &[vec![2.0, 0.0], vec![0.0, 0.0]],
        0.5,
    )
    .unwrap();
    assert!((vector[0] - 0.25).abs() < f64::EPSILON);
    assert!((vector[1] - 2.0).abs() < f64::EPSILON);

    let channel = WorldAnimationChannel {
        target_path: "animated".to_owned(),
        property: "translation".to_owned(),
        pre_infinity: 2,
        post_infinity: 2,
        times: vec![0.0, 1.0],
        values: vec![vec![0.0, 1.0, 2.0], vec![0.0, 3.0, 4.0]],
        in_tangents: vec![vec![0.0; 3], vec![0.0; 3]],
        out_tangents: vec![vec![2.0, 0.0, 0.0], vec![0.0; 3]],
    };
    let fixed = sample_world_animation_vector_fixed::<3>(&channel, 0.5).unwrap();
    assert!((fixed[0] - 0.25).abs() < f64::EPSILON);
    assert!((fixed[1] - 2.0).abs() < f64::EPSILON);
    assert!((fixed[2] - 3.0).abs() < f64::EPSILON);
}
