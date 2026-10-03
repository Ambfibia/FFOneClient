use super::*;

#[test]
fn organized_source_routes_restore_legacy_per_instance_model_ids() {
    assert_eq!(
        legacy_static_model_id("models/world/maps/map_08_06/v-01384-11153-3eb7984e8c48.glb")
            .as_deref(),
        Some("static-map_08_06-01384")
    );
    assert_eq!(
        legacy_static_model_id(
            "models/world/tutorial/tile_01_01/c-00413-6608-f652008795a6.glb"
        )
        .as_deref(),
        Some("static-tile_01_01-00413")
    );
    assert!(legacy_static_model_id("objects/shared/platform/visual.glb").is_none());
}

#[test]
fn scripted_reparent_preserves_the_authored_model_world_matrix() {
    let parent_world = Transform::from_xyz(-2845.0, -62.0, 6419.0)
        .with_rotation(Quat::from_rotation_y(1.1))
        .with_scale(Vec3::splat(1.1))
        .to_matrix();
    let model_world = Transform::from_xyz(-2843.2, -68.0, 6419.4)
        .with_rotation(Quat::from_rotation_x(-0.7))
        .with_scale(Vec3::splat(1.1))
        .to_matrix();

    let local = authored_model_local_transform(parent_world, model_world);
    let reconstructed = parent_world * local.to_matrix();
    assert!(
        (reconstructed - model_world)
            .to_cols_array()
            .iter()
            .all(|component| component.abs() < 1.0e-3)
    );
}

#[test]
fn primary_springcooler_nif_controller_repeats_its_exact_rotation_cycle() {
    // Primary proof:
    // Map_12_03.unity3d, BuildPlayer-Map_12_03#12932 / Animation#12933
    // -> Freearea_shared.resourceFile,
    // CustomAssetBundle-74b5d4666b65a47518510d30c66283ee#1162.
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let document =
        load_native_world_behaviours(&asset_root, NativeWorldScope::WorldMap, "map_12_03")
            .unwrap();
    let record = document
        .animations
        .iter()
        .find(|record| record.node == "BuildPlayer-Map_12_03#12932")
        .expect("published spring-cooler Animation owner");
    assert!(record.enabled);
    assert!(record.play_automatically);
    assert!(record.animate_only_if_visible);
    assert!(!record.animate_physics);
    assert_eq!(record.wrap_mode, 0);

    let clip = document
        .animation_clips
        .iter()
        .find(|clip| clip.id == "CustomAssetBundle-74b5d4666b65a47518510d30c66283ee#1162")
        .expect("published spring-cooler NIF clip");
    assert_eq!(clip.name, "nif-default");
    assert!(
        !clip.looped,
        "the generated Unity wrapper is not marked looped"
    );
    assert!((clip.duration - 3.33333158493042).abs() < 1.0e-9);
    assert_eq!(clip.channels.len(), 1);
    assert_eq!(clip.channels[0].target_path, "Object01");
    assert_eq!(clip.channels[0].property, "rotation");
    assert_eq!(clip.float_curves.len(), 3);
    assert!(world_animation_transform_repeats(
        Some(clip),
        record.wrap_mode
    ));

    let after_one_cycle =
        world_animation_sample_time(clip.duration + 0.43, clip.duration, true);
    assert!((after_one_cycle - 0.43).abs() < 1.0e-9);
    let channel = &clip.channels[0];
    let start = sample_world_animation_vector_fixed::<4>(channel, 0.0).unwrap();
    let advanced = sample_world_animation_vector_fixed::<4>(channel, after_one_cycle).unwrap();
    let start = Quat::from_xyzw(
        start[0] as f32,
        start[1] as f32,
        start[2] as f32,
        start[3] as f32,
    )
    .normalize();
    let advanced = Quat::from_xyzw(
        advanced[0] as f32,
        advanced[1] as f32,
        advanced[2] as f32,
        advanced[3] as f32,
    )
    .normalize();
    let (axis, angle) = (start.inverse() * advanced).to_axis_angle();
    assert!(
        axis.dot(Vec3::NEG_Z) > 0.99,
        "native rotation axis was {axis:?}"
    );
    assert!((angle.to_degrees() - 46.93).abs() < 0.2);
    assert!((360.0 / clip.duration - 108.0).abs() < 1.0e-4);
}

#[test]
fn nif_transform_continuation_rejects_open_and_authored_one_shot_clips() {
    let mut clip = WorldAnimationClip {
        id: "closed-nif-rotation".to_owned(),
        name: "nif-default".to_owned(),
        duration: 1.0,
        sample_rate: 30.0,
        looped: false,
        channels: vec![WorldAnimationChannel {
            target_path: "spinner".to_owned(),
            property: "rotation".to_owned(),
            pre_infinity: 1,
            post_infinity: 1,
            times: vec![0.0, 0.5, 1.0],
            values: vec![
                vec![0.0, 0.0, 0.0, 1.0],
                vec![0.0, 0.0, 1.0, 0.0],
                vec![0.0, 0.0, 0.0, -1.0],
            ],
            in_tangents: vec![vec![0.0; 4]; 3],
            out_tangents: vec![vec![0.0; 4]; 3],
        }],
        float_curves: Vec::new(),
        events: Vec::new(),
    };
    assert!(world_animation_transform_repeats(Some(&clip), 0));

    clip.name = "authored-one-shot".to_owned();
    assert!(!world_animation_transform_repeats(Some(&clip), 0));
    clip.name = "nif-default".to_owned();
    clip.channels[0].values[2] = vec![0.0, 0.0, 0.70710678, 0.70710678];
    assert!(!world_animation_transform_repeats(Some(&clip), 0));
    assert_eq!(world_animation_sample_time(1.25, 1.0, false), 1.0);
}

#[test]
fn nif_waterfall_and_lava_scrolls_compile_to_continuous_gpu_motion() {
    let clip = material_clip(
        "nif-default",
        false,
        vec![
            linear_material_curve("waterfall", "_MainTex.offset.y", 6.0, 1.0 / 6.0),
            linear_material_curve("lava", "_MainTex.offset.y", 66.666, 0.15),
        ],
    );

    let waterfall = compile_world_gpu_uv_animation(&clip, "waterfall", 0, true, 12.0).unwrap();
    let waterfall_y = waterfall.offset_y.unwrap();
    assert!((waterfall_y.velocity - 1.0 / 6.0).abs() < 1.0e-6);
    assert!((waterfall_y.value_at_bind - 2.0).abs() < 1.0e-5);
    assert!((native_world_texture_offset_y(waterfall_y.velocity) + 1.0 / 6.0).abs() < 1.0e-6);
    assert!((native_world_texture_offset_y(waterfall_y.value_at_bind) + 2.0).abs() < 1.0e-5);

    let lava = compile_world_gpu_uv_animation(&clip, "lava", 0, true, 20.0).unwrap();
    let lava_y = lava.offset_y.unwrap();
    assert!((lava_y.velocity - 0.15).abs() < 1.0e-6);
    assert!((lava_y.value_at_bind - 3.0).abs() < 1.0e-5);
}

#[test]
fn published_waterfall_and_lava_tiles_keep_gpu_compatible_nif_scrolls() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    for tile_id in ["map_02_12", "map_05_12"] {
        let document =
            load_native_world_behaviours(&asset_root, NativeWorldScope::WorldMap, tile_id)
                .unwrap();
        let compatible_targets = document
            .animation_clips
            .iter()
            .filter(|clip| clip.name == "nif-default")
            .map(|clip| {
                clip.float_curves
                    .iter()
                    .filter(|curve| curve.attribute.starts_with("_MainTex."))
                    .map(|curve| curve.target_path.as_str())
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .filter(|target| {
                        compile_world_gpu_uv_animation(clip, target, 0, true, 0.0).is_some()
                    })
                    .count()
            })
            .sum::<usize>();
        assert!(
            compatible_targets >= 4,
            "{tile_id} retained only {compatible_targets} GPU-compatible NIF scroll targets"
        );
    }
}

#[test]
fn effect_emitter_nif_models_remain_independently_published_static_world_geometry() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game/map/tiles/map_06_03/behaviour.json");
    let document: NativeWorldBehaviourDocument =
        serde_json::from_slice(&std::fs::read(root).unwrap()).unwrap();
    let record = document
        .effect_emitters
        .iter()
        .find(|record| record.effect_name.as_deref() == Some("neededjump_nano"))
        .expect("published needed-jump marker");
    assert!(!reproducible_world_scripted_effect(record));
    assert!(!record.models.is_empty());
    assert!(record.nif_object.is_some());
}

#[test]
fn scripted_particle_prefabs_identify_the_runtime_owned_nif_expansion() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game/map/tiles/map_08_07/behaviour.json");
    let document: NativeWorldBehaviourDocument =
        serde_json::from_slice(&std::fs::read(root).unwrap()).unwrap();
    let x_com = document
        .effect_emitters
        .iter()
        .find(|record| record.effect_name.as_deref() == Some("x_com_ma"))
        .expect("Peach Creek x_com_ma emitter");
    assert_eq!(x_com.controller, "EffectEmitterController");
    assert!(!x_com.models.is_empty());

    assert!(reproducible_world_scripted_effect(x_com));
    assert!(x_com.nif_object.is_some());
    assert!(x_com.resolved_particle_prefabs.iter().any(Option::is_some));
}

#[test]
fn effect_streaming_keeps_authored_nif_models_after_native_enqueue() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game/map/tiles");

    let x_com_document: Arc<NativeWorldBehaviourDocument> = Arc::new(
        serde_json::from_slice(
            &std::fs::read(asset_root.join("map_08_07/behaviour.json")).unwrap(),
        )
        .unwrap(),
    );
    let x_com_index = x_com_document
        .effect_emitters
        .iter()
        .position(|record| record.effect_name.as_deref() == Some("x_com_ma"))
        .expect("Peach Creek x_com_ma emitter");
    let x_com = &x_com_document.effect_emitters[x_com_index];
    let x_com_model = x_com.models.first().expect("x_com nif model").clone();
    let mut x_com_world = World::new();
    let x_com_root = x_com_world.spawn_empty().id();
    let x_com_static_visual = x_com_world.spawn(Visibility::Inherited).id();
    let mut x_com_pending = PendingWorldBehaviourSpawn::new(
        Arc::clone(&x_com_document),
        Vec::new(),
        HashMap::from([(x_com_model, vec![x_com_static_visual])]),
        HashMap::new(),
        0,
        1,
    );
    x_com_pending.phase = WorldBehaviourSpawnPhase::EffectEmitters;
    x_com_pending.next_record = x_com_index;
    let mut queue = bevy::ecs::world::CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &x_com_world);
        streaming::materialize_world_behaviour_batch(
            &mut commands,
            x_com_root,
            &mut x_com_pending,
            1,
        );
    }
    queue.apply(&mut x_com_world);

    assert!(
        x_com_world
            .get::<RuntimeManagedNativeWorldVisual>(x_com_static_visual)
            .is_none(),
        "ownership must not transfer before the runtime accepts the particle closure"
    );
    assert_eq!(
        x_com_world.get::<Visibility>(x_com_static_visual),
        Some(&Visibility::Inherited)
    );
    let emitter = x_com_world
        .query::<&WorldEffectEmitter>()
        .single(&x_com_world)
        .expect("the scripted emitter remains materialized");
    assert_eq!(emitter.model_entities, vec![x_com_static_visual]);

    x_com_world.insert_resource(TutorialEffectRuntime::default());
    x_com_world
        .entity_mut(x_com_root)
        .insert(PendingWorldScriptedEffects {
            document: Some(Arc::clone(&x_com_document)),
            next_record: x_com_index,
        });
    let mut enqueue_schedule = Schedule::default();
    enqueue_schedule.add_systems(enqueue_pending_world_scripted_effects);
    enqueue_schedule.run(&mut x_com_world);

    assert!(
        x_com_world
            .get::<RuntimeManagedNativeWorldVisual>(x_com_static_visual)
            .is_none(),
        "native enqueue must not claim ownership of authored static geometry"
    );
    assert_eq!(
        x_com_world.get::<Visibility>(x_com_static_visual),
        Some(&Visibility::Inherited),
        "successful native enqueue must preserve the authored nifObject"
    );

    let neededjump_document: Arc<NativeWorldBehaviourDocument> = Arc::new(
        serde_json::from_slice(
            &std::fs::read(asset_root.join("map_06_03/behaviour.json")).unwrap(),
        )
        .unwrap(),
    );
    let neededjump_index = neededjump_document
        .effect_emitters
        .iter()
        .position(|record| record.effect_name.as_deref() == Some("neededjump_nano"))
        .expect("published needed-jump marker");
    let neededjump = &neededjump_document.effect_emitters[neededjump_index];
    let neededjump_model = neededjump
        .models
        .first()
        .expect("needed-jump nif model")
        .clone();
    let mut neededjump_world = World::new();
    let neededjump_root = neededjump_world.spawn_empty().id();
    let authored_rotation = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
    let neededjump_static_visual = neededjump_world
        .spawn((Visibility::Inherited, Transform::from_rotation(authored_rotation)))
        .id();
    let mut neededjump_pending = PendingWorldBehaviourSpawn::new(
        Arc::clone(&neededjump_document),
        Vec::new(),
        HashMap::from([(neededjump_model, vec![neededjump_static_visual])]),
        HashMap::new(),
        0,
        0,
    );
    neededjump_pending.phase = WorldBehaviourSpawnPhase::EffectEmitters;
    neededjump_pending.next_record = neededjump_index;
    let mut queue = bevy::ecs::world::CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &neededjump_world);
        streaming::materialize_world_behaviour_batch(
            &mut commands,
            neededjump_root,
            &mut neededjump_pending,
            1,
        );
    }
    queue.apply(&mut neededjump_world);

    assert!(
        neededjump_world
            .get::<RuntimeManagedNativeWorldVisual>(neededjump_static_visual)
            .is_none(),
        "an all-null particle controller relies on its static nifObject publication"
    );
    assert_eq!(
        neededjump_world.get::<Visibility>(neededjump_static_visual),
        Some(&Visibility::Inherited)
    );
    assert_eq!(
        neededjump_world
            .query::<&WorldEffectEmitter>()
            .single(&neededjump_world)
            .expect("legacy all-null emitter remains represented")
            .model_entities,
        vec![neededjump_static_visual]
    );
    assert!(
        neededjump_world
            .get::<WorldFloatingIconSpin>(neededjump_static_visual)
            .is_some(),
        "the published wing mesh needs its own visual motion"
    );
    assert!(
        neededjump_world
            .get::<crate::world::NativeWorldDynamicObjectRange>(neededjump_static_visual)
            .is_some(),
        "a spinning wing cannot keep the static placement's culling range"
    );
    let mut clock = Time::<()>::default();
    clock.advance_by(std::time::Duration::from_secs(1));
    neededjump_world.insert_resource(clock);
    let mut spin_schedule = Schedule::default();
    spin_schedule.add_systems(update_world_floating_icons);
    spin_schedule.run(&mut neededjump_world);
    let rotation = neededjump_world
        .get::<Transform>(neededjump_static_visual)
        .unwrap()
        .rotation;
    let expected = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2) * authored_rotation;
    assert!(rotation.dot(expected).abs() > 0.999);
    assert_eq!(
        neededjump_world.get::<Visibility>(neededjump_static_visual),
        Some(&Visibility::Inherited)
    );
}
