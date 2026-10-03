use super::*;

#[test]
fn queued_lair_combat_pose_faces_dexter_toward_fusion_buttercup() {
    let mut app = app();
    let dexter_spawn = spawn(LAIR_DEXTER_ID, 2902, [56_200, 96_700, -13_300], None);
    {
        let mut queue = app.world_mut().resource_mut::<TutorialActorCommandQueue>();
        // `Infection_Event_B` completion and Chapter_04 fallthrough both
        // converge on this replacement during the same Update. Preserve
        // the second Spawn because its preceding Delete ends the first
        // queued lifetime.
        queue.delete(LAIR_DEXTER_ID);
        queue.spawn(dexter_spawn);
        queue.spawn(spawn(
            FUSION_BUTTERCUP_ID,
            2678,
            [56_300, 96_700, -13_300],
            Some(266),
        ));
        queue.face_actor(LAIR_DEXTER_ID, FUSION_BUTTERCUP_ID);
        queue.play_pose(LAIR_DEXTER_ID, "melee1event", false);
        queue.delete(LAIR_DEXTER_ID);
        queue.spawn(dexter_spawn);
        queue.face_actor(LAIR_DEXTER_ID, FUSION_BUTTERCUP_ID);
        queue.play_pose(LAIR_DEXTER_ID, "melee1event", false);
    }

    app.update();

    let registry = app.world().resource::<TutorialActorRegistry>();
    let dexter = registry.entity(LAIR_DEXTER_ID).unwrap();
    let fusion = registry.entity(FUSION_BUTTERCUP_ID).unwrap();
    let dexter_transform = app.world().get::<Transform>(dexter).unwrap();
    let fusion_transform = app.world().get::<Transform>(fusion).unwrap();
    let expected_facing = (fusion_transform.translation - dexter_transform.translation)
        .with_y(0.0)
        .normalize();

    assert!(dexter_transform.forward().as_vec3().dot(expected_facing) > 0.999);
    assert_eq!(
        app.world().get::<TutorialActorPose>(dexter).unwrap().clip,
        Some("melee1event")
    );
    assert!(!app.world().get::<TutorialActorPose>(dexter).unwrap().once);
}

#[test]
fn forced_update_buttercup_keeps_the_scripted_flyby_height_across_pose_changes() {
    let mut app = app();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
        0.1,
    )));
    {
        let mut queue = app.world_mut().resource_mut::<TutorialActorCommandQueue>();
        queue.spawn(spawn(202, 2665, [90_600, 74_700, -1_500], None));
        queue.play_pose(202, "run", false);
        queue.force_update(202);
    }
    app.update();

    let buttercup = app
        .world()
        .resource::<TutorialActorRegistry>()
        .entity(202)
        .expect("Buttercup tutorial actor");
    assert!(
        app.world()
            .get::<TutorialActorForceUpdate>(buttercup)
            .is_some(),
        "SetForceUpdate(true) must persist on NpcMoveController"
    );

    let scripted_height = 25.0;
    app.world_mut()
        .get_mut::<Transform>(buttercup)
        .unwrap()
        .translation
        .y = scripted_height;
    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .play_pose(202, "flying", false);
    app.update();

    assert_eq!(
        app.world()
            .get::<Transform>(buttercup)
            .unwrap()
            .translation
            .y,
        scripted_height,
        "NpcMoveController gravity must not override the coroutine flyby"
    );
    assert!(
        app.world()
            .get::<TutorialActorForceUpdate>(buttercup)
            .is_some(),
        "changing run to flying must not clear bForceUpdate"
    );
    assert!(
        !app.world()
            .get::<TutorialActorPose>(buttercup)
            .unwrap()
            .force_update,
        "the controller flag must remain independent from animation restart state"
    );
}

#[test]
fn pose_commands_advance_request_and_restart_serials_exactly() {
    let mut app = app();
    {
        let mut queue = app.world_mut().resource_mut::<TutorialActorCommandQueue>();
        queue.spawn(spawn(1005, 2671, [0, 0, 0], None));
        queue.play_pose(1005, "think", false);
    }
    app.update();
    let entity = app
        .world()
        .resource::<TutorialActorRegistry>()
        .entity(1005)
        .unwrap();
    let pose = *app.world().get::<TutorialActorPose>(entity).unwrap();
    assert_eq!((pose.request_serial, pose.restart_serial), (1, 1));

    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .play_pose(1005, "think", false);
    app.update();
    let pose = *app.world().get::<TutorialActorPose>(entity).unwrap();
    assert_eq!((pose.request_serial, pose.restart_serial), (2, 2));

    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .set_pose_state(1005, TutorialActorPoseState::Paused);
    app.update();
    let pose = *app.world().get::<TutorialActorPose>(entity).unwrap();
    assert_eq!((pose.request_serial, pose.restart_serial), (3, 2));
    assert_eq!(pose.state, TutorialActorPoseState::Paused);

    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .force_update(1005);
    app.update();
    let pose = *app.world().get::<TutorialActorPose>(entity).unwrap();
    assert_eq!((pose.request_serial, pose.restart_serial), (4, 4));
    assert!(pose.force_update);
    assert_eq!(pose.clip, Some("think"));
}

#[test]
fn animation_player_restarts_same_node_and_resolves_once_vs_repeat() {
    let (_, node) = AnimationGraph::from_clip(Handle::<AnimationClip>::default());
    let mut player = AnimationPlayer::default();
    let mut transitions = AnimationTransitions::new();

    restart_tutorial_actor_animation(
        &mut player,
        &mut transitions,
        node,
        false,
        false,
        Duration::ZERO,
    );
    assert_eq!(
        player.animation(node).unwrap().repeat_mode(),
        RepeatAnimation::Forever
    );
    player.animation_mut(node).unwrap().set_seek_time(0.75);
    restart_tutorial_actor_animation(
        &mut player,
        &mut transitions,
        node,
        false,
        false,
        Duration::from_secs_f32(TUTORIAL_ACTOR_CROSS_FADE_SECONDS),
    );
    assert_eq!(player.animation(node).unwrap().seek_time(), 0.0);

    restart_tutorial_actor_animation(
        &mut player,
        &mut transitions,
        node,
        true,
        true,
        Duration::from_secs_f32(TUTORIAL_ACTOR_CROSS_FADE_SECONDS),
    );
    let active = player.animation(node).unwrap();
    assert_eq!(active.repeat_mode(), RepeatAnimation::Never);
    assert!(active.is_paused());
}

pub(super) fn gltf_with_named_animation(name: &str) -> Gltf {
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
    let animation = Handle::<AnimationClip>::default();
    gltf.animations.push(animation.clone());
    gltf.named_animations.insert(name.into(), animation);
    gltf
}

#[test]
fn named_animation_lookup_is_exact_and_missing_issue_is_deduplicated() {
    let gltf = gltf_with_named_animation("Think");
    assert!(exact_tutorial_actor_named_animation(&gltf, "Think").is_some());
    assert!(exact_tutorial_actor_named_animation(&gltf, "think").is_none());

    let mut history = TutorialActorAnimationIssueHistory::default();
    let mut issues = TutorialActorIssueQueue::default();
    report_tutorial_actor_animation_unavailable(&mut history, &mut issues, 1005, "think");
    report_tutorial_actor_animation_unavailable(&mut history, &mut issues, 1005, "think");
    assert_eq!(
        issues.take_all(),
        VecDeque::from([TutorialActorIssue::RigAnimationUnavailable {
            id: 1005,
            clip: "think",
        }])
    );
}

#[test]
fn one_prepared_graph_retains_every_named_clip_for_native_cross_fades() {
    let mut gltf = gltf_with_named_animation("run");
    let ready = Handle::<AnimationClip>::default();
    gltf.animations.push(ready.clone());
    gltf.named_animations.insert("ready".into(), ready);
    let melee = Handle::<AnimationClip>::default();
    gltf.animations.push(melee.clone());
    gltf.named_animations.insert("melee1".into(), melee);
    let mut graphs = Assets::<AnimationGraph>::default();

    let prepared = prepare_tutorial_actor_animation_graph(&gltf, &mut graphs);

    assert_eq!(graphs.len(), 1);
    assert!(prepared.nodes.contains_key("run"));
    assert!(prepared.nodes.contains_key("ready"));
    assert_ne!(prepared.nodes["run"], prepared.nodes["ready"]);
    assert!(prepared.nodes.contains_key("melee1"));
    assert!(prepared.additive_nodes.contains_key("melee1"));
    assert_ne!(prepared.nodes["melee1"], prepared.additive_nodes["melee1"]);
}

#[test]
fn upgraded_mobs_use_delta_additive_nodes_without_stopping_their_base_pose() {
    assert!(network_npc_animation_uses_delta_additive_layer_0104(
        "characters/mobs/mob_oilmonster/mob_oilmonster.glb",
        "melee1",
    ));
    assert!(network_npc_animation_uses_delta_additive_layer_0104(
        "characters/mobs/mob_sneakyspawn/mob_sneakyspawn.glb",
        "wound",
    ));
    assert!(network_npc_animation_uses_delta_additive_layer_0104(
        "characters/mobs/mob_cerberus/mob_cerberus.glb",
        "melee1",
    ));
    assert!(!network_npc_animation_uses_delta_additive_layer_0104(
        "characters/mobs/mob_sneakyspawn/mob_sneakyspawn.glb",
        "ready",
    ));
    assert!(legacy_npc_clip_is_additive("wound"));
    assert!(legacy_npc_clip_is_additive("melee1"));
    assert!(!legacy_npc_clip_is_additive("woundupper"));
    assert!(!legacy_npc_clip_is_additive("melee1upper"));
    assert!(!legacy_npc_clip_is_additive("melee1event"));

    let (_, nodes) = AnimationGraph::from_clips([
        Handle::<AnimationClip>::default(),
        Handle::<AnimationClip>::default(),
    ]);
    let mut player = AnimationPlayer::default();
    player
        .start(nodes[0])
        .set_repeat(RepeatAnimation::Forever)
        .resume();
    restart_tutorial_actor_additive_animation(&mut player, nodes[1], false);

    assert_eq!(
        player.animation(nodes[0]).unwrap().repeat_mode(),
        RepeatAnimation::Forever
    );
    assert_eq!(
        player.animation(nodes[1]).unwrap().repeat_mode(),
        RepeatAnimation::Never
    );
}

#[test]
fn additive_combat_pose_starts_ready_when_no_base_animation_is_active() {
    let (_, nodes) = AnimationGraph::from_clips([
        Handle::<AnimationClip>::default(),
        Handle::<AnimationClip>::default(),
    ]);
    let mut player = AnimationPlayer::default();

    ensure_tutorial_actor_additive_base_animation(
        &mut player,
        &[nodes[0]],
        Some(nodes[0]),
        false,
    );
    restart_tutorial_actor_additive_animation(&mut player, nodes[1], false);

    assert_eq!(
        player.animation(nodes[0]).unwrap().repeat_mode(),
        RepeatAnimation::Forever
    );
    assert_eq!(
        player.animation(nodes[1]).unwrap().repeat_mode(),
        RepeatAnimation::Never
    );
}

#[test]
fn move_clip_matches_original_move_npc_style_threshold() {
    assert_eq!(tutorial_actor_move_clip(3.0), "walk");
    assert_eq!(tutorial_actor_move_clip(4.0), "walk");
    assert_eq!(tutorial_actor_move_clip(4.01), "run");
    assert_eq!(tutorial_actor_move_clip(7.0), "run");
}

#[test]
fn force_stop_cancels_motion_without_turning_run_into_a_paused_pose() {
    let mut app = app();
    {
        let mut queue = app.world_mut().resource_mut::<TutorialActorCommandQueue>();
        queue.spawn(spawn(100, 2669, [0, 0, 0], None));
        queue.move_native(100, Vec3::X * 10.0, 7.0);
    }
    app.update();
    let entity = app
        .world()
        .resource::<TutorialActorRegistry>()
        .entity(100)
        .unwrap();
    assert!(app.world().get::<TutorialActorMotion>(entity).is_some());
    assert_eq!(
        app.world().get::<TutorialActorPose>(entity).unwrap().clip,
        Some("run")
    );

    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .stop_motion(100);
    app.update();

    assert!(app.world().get::<TutorialActorMotion>(entity).is_none());
    let pose = app.world().get::<TutorialActorPose>(entity).unwrap();
    assert_eq!(pose.clip, Some("run"));
    assert_eq!(pose.state, TutorialActorPoseState::Playing);
}

#[test]
fn forced_spawn_pose_suppresses_the_default_set_model_stand_draw() {
    let mut app = app();
    app.insert_resource(TutorialActorStandRandomStream::with_seed(0x1234_5678));
    {
        let mut queue = app.world_mut().resource_mut::<TutorialActorCommandQueue>();
        queue.spawn(spawn(101, 2670, [0, 0, 0], None));
        queue.play_pose(101, "run", false);
    }
    app.update();

    let entity = app
        .world()
        .resource::<TutorialActorRegistry>()
        .entity(101)
        .unwrap();
    let pose = *app.world().get::<TutorialActorPose>(entity).unwrap();
    assert_eq!(pose.resolved_clip, Some("run"));
    assert_eq!(
        app.world()
            .resource::<TutorialActorStandRandomStream>()
            .draw_count(),
        0
    );
}

#[test]
fn idle_native_clip_resolution_is_exact_then_falls_back_to_stand1() {
    let exact = gltf_with_named_animation("stand4");
    let pose = TutorialActorPose {
        clip: Some("idle"),
        resolved_clip: Some("stand4"),
        ..default()
    };
    assert_eq!(
        exact_tutorial_actor_pose_animation(&exact, &pose).map(|(_, name)| name),
        Some("stand4")
    );

    let fallback = gltf_with_named_animation("stand1");
    assert_eq!(
        exact_tutorial_actor_pose_animation(&fallback, &pose).map(|(_, name)| name),
        Some("stand1")
    );
    let wrong_case = gltf_with_named_animation("Stand1");
    assert!(exact_tutorial_actor_pose_animation(&wrong_case, &pose).is_none());
}

pub(super) fn add_animation_requirement(
    id: i32,
    clip: &'static str,
    origin: &str,
    live: &BTreeMap<i32, i32>,
    all_types: &BTreeMap<i32, BTreeSet<i32>>,
    catalog: &NetworkNpcVisualCatalog0104,
    requirements: &mut BTreeSet<(String, &'static str)>,
    blockers: &mut BTreeSet<String>,
) {
    if id == -1 {
        return;
    }
    let npc_type = match resolve_requirement_type(id, live, all_types) {
        Ok(npc_type) => npc_type,
        Err(error) => {
            blockers.insert(format!("{origin}: {error}, clip {clip:?}"));
            return;
        }
    };
    let Some(definition) = catalog.get(npc_type) else {
        blockers.insert(format!(
            "{origin}: actor id {id} npc type {npc_type} has no shared visual, clip {clip:?}"
        ));
        return;
    };
    requirements.insert((definition.glb.clone(), clip));
}

pub(super) fn choreography_animation_requirements() -> (BTreeSet<(String, &'static str)>, BTreeSet<String>)
{
    let catalog = production_visual_catalog();
    let mut all_types = BTreeMap::<i32, BTreeSet<i32>>::new();
    all_types
        .entry(TUTORIAL_INITIALIZATION.initial_npc.runtime_id)
        .or_default()
        .insert(TUTORIAL_INITIALIZATION.initial_npc.npc_type);
    // These actors are emitted by tutorial_logic immediately before their
    // scene, rather than by a choreography NpcSpawn action.
    all_types.entry(5100).or_default().insert(2671);
    all_types.entry(5101).or_default().insert(2800);
    for scene in TUTORIAL_SCENE_CHOREOGRAPHIES {
        for action in scene.actions {
            if let ChoreographyAction::Npc(action) = action.action {
                gather_spawn_types(action, &mut all_types);
            }
        }
        for action in scene
            .skip
            .scene_specific
            .iter()
            .chain(scene.skip.common_cleanup.iter())
        {
            if let ChoreographyAction::Npc(action) = action.action {
                gather_spawn_types(action, &mut all_types);
            }
        }
    }

    let mut live = BTreeMap::from([(
        TUTORIAL_INITIALIZATION.initial_npc.runtime_id,
        TUTORIAL_INITIALIZATION.initial_npc.npc_type,
    )]);
    let mut requirements = BTreeSet::new();
    let mut blockers = BTreeSet::new();
    for scene in TUTORIAL_SCENE_CHOREOGRAPHIES {
        match scene.scene {
            // tutorial_logic spawns LAIR_DEXTER_ID as 2673 before it emits
            // StartScene(InfectionB); the later 2902 replacement happens
            // only after that scene completes.
            TutorialScene::InfectionB => {
                live.insert(3000, 2673);
            }
            // tutorial_logic emits both actors immediately before
            // StartScene(NanoPowerA), and NanoPowerB retains them.
            TutorialScene::NanoPowerA | TutorialScene::NanoPowerB => {
                live.insert(5100, 2671);
                live.insert(5101, 2800);
            }
            _ => {}
        }
        for timed in scene.actions {
            if let ChoreographyAction::Npc(action) = timed.action {
                audit_npc_action(
                    action,
                    &format!("{:?}:{}", scene.scene, timed.source_line),
                    &mut live,
                    &all_types,
                    &catalog,
                    &mut requirements,
                    &mut blockers,
                );
            }
        }

        let mut skip_live = live.clone();
        for sourced in scene
            .skip
            .scene_specific
            .iter()
            .chain(scene.skip.common_cleanup.iter())
        {
            if let ChoreographyAction::Npc(action) = sourced.action {
                audit_npc_action(
                    action,
                    &format!("{:?}:skip:{}", scene.scene, sourced.source_line),
                    &mut skip_live,
                    &all_types,
                    &catalog,
                    &mut requirements,
                    &mut blockers,
                );
            }
        }
    }

    for definition in TUTORIAL_AUXILIARY_DEFINITIONS {
        let mut auxiliary_live = live.clone();
        // MinimapEvent is started while tutorial_logic's Numbuh Two (2671)
        // owns runtime id 1005. Reusing the final main-scene map here used
        // to misattribute its exact `think` request to later Cerberus 2675.
        if definition.sequence == TutorialAuxiliarySequence::MinimapEvent {
            auxiliary_live.insert(1005, 2671);
        }
        for timed in definition.actions {
            if let TutorialAuxiliaryAction::SetNpcAnimation {
                runtime_id,
                animation,
            } = timed.action
            {
                add_animation_requirement(
                    runtime_id,
                    animation,
                    &format!("{:?}:{}", definition.sequence, timed.at_seconds),
                    &auxiliary_live,
                    &all_types,
                    &catalog,
                    &mut requirements,
                    &mut blockers,
                );
            }
        }
        if let Some(repeating) = definition.repeating {
            for action in repeating.actions {
                if let TutorialAuxiliaryAction::SetNpcAnimation {
                    runtime_id,
                    animation,
                } = *action
                {
                    add_animation_requirement(
                        runtime_id,
                        animation,
                        &format!("{:?}:repeat", definition.sequence),
                        &auxiliary_live,
                        &all_types,
                        &catalog,
                        &mut requirements,
                        &mut blockers,
                    );
                }
            }
        }
    }
    (requirements, blockers)
}

pub(super) fn glb_animation_names(path: &Path) -> Result<BTreeSet<String>, String> {
    let mut file = File::open(path).map_err(|error| format!("{path:?}: {error}"))?;
    let mut header = [0_u8; 20];
    file.read_exact(&mut header)
        .map_err(|error| format!("{path:?}: short GLB header: {error}"))?;
    if &header[0..4] != b"glTF" {
        return Err(format!("{path:?}: invalid GLB magic"));
    }
    let json_length = u32::from_le_bytes(header[12..16].try_into().unwrap()) as usize;
    let chunk_type = u32::from_le_bytes(header[16..20].try_into().unwrap());
    if chunk_type != 0x4e4f_534a {
        return Err(format!("{path:?}: first GLB chunk is not JSON"));
    }
    let mut json = vec![0_u8; json_length];
    file.read_exact(&mut json)
        .map_err(|error| format!("{path:?}: short GLB JSON chunk: {error}"))?;
    let document: serde_json::Value =
        serde_json::from_slice(&json).map_err(|error| format!("{path:?}: {error}"))?;
    Ok(document
        .get("animations")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|animation| {
            animation
                .get("name")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .collect())
}

#[test]
fn every_reachable_published_actor_clip_is_exactly_named_or_explicitly_blocked() {
    // Every animation reachable from the tutorial choreography now has an
    // exact, source-named clip in its published GLB. Keep this explicit
    // empty matrix so a future asset regression still fails closed here.
    const SOURCE_PROVEN_PUBLISH_BLOCKERS: &[(&str, &str)] = &[];
    let (requirements, blockers) = choreography_animation_requirements();
    let mut load_errors = Vec::new();
    let mut missing = BTreeSet::new();
    let mut names_by_path = BTreeMap::new();
    for (path, clip) in &requirements {
        let names = names_by_path.entry(path.clone()).or_insert_with(|| {
            glb_animation_names(&asset_root().join(path)).unwrap_or_else(|error| {
                load_errors.push(error);
                BTreeSet::new()
            })
        });
        let resolved = if *clip == "idle" {
            TUTORIAL_ACTOR_STAND_CLIPS
                .iter()
                .all(|candidate| names.contains(*candidate) || names.contains("stand1"))
        } else {
            names.contains(*clip)
        };
        if !resolved {
            missing.insert((path.clone(), *clip));
        }
    }
    assert!(
        blockers.is_empty() && load_errors.is_empty(),
        "unresolved choreography mappings:\n{}\nGLB read failures:\n{}",
        blockers.into_iter().collect::<Vec<_>>().join("\n"),
        load_errors.join("\n")
    );
    assert_eq!(
        missing,
        SOURCE_PROVEN_PUBLISH_BLOCKERS
            .iter()
            .map(|(path, clip)| ((*path).to_owned(), *clip))
            .collect(),
        "published exact-clip blocker matrix changed; investigate source animation mapping"
    );
    assert!(
        !requirements.is_empty(),
        "tutorial choreography must retain actor animation requirements"
    );
}
