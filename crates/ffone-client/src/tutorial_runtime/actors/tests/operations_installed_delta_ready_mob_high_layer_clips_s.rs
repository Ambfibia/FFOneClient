use super::*;

pub(super) fn spawn(id: i32, npc_type: i32, raw: [i32; 3], angle: Option<i16>) -> TutorialNpcSpawn {
    TutorialNpcSpawn::new(
        id,
        npc_type,
        LegacySpawnPosition::centiunits(raw[0], raw[1], raw[2]),
        angle,
    )
}

pub(super) fn production_content() -> TutorialMissionContent {
    let assets = crate::assets::AssetLocator::open(asset_root()).unwrap();
    TutorialMissionContent::open(&assets).unwrap()
}

pub(super) fn app() -> App {
    let content = production_content();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(content)
        .add_plugins(TutorialActorPlugin);
    app
}

pub(super) fn assert_vec3_close(actual: Vec3, expected: Vec3) {
    assert!(
        actual.abs_diff_eq(expected, 0.000_01),
        "actual {actual:?}, expected {expected:?}"
    );
}

#[test]
fn location_actor_2000_remains_a_hidden_proximity_root_and_is_not_targetable() {
    let mut app = app();
    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .spawn(spawn(2000, 2374, [69_700, 79_400, -8_200], None));

    app.update();

    let entity = app
        .world()
        .resource::<TutorialActorRegistry>()
        .entity(2000)
        .expect("location actor root");
    let actor = *app.world().get::<TutorialActor>(entity).unwrap();
    let transform = app.world().get::<Transform>(entity).unwrap();

    assert_eq!(actor.npc_type, 2374);
    assert_vec3_close(transform.translation, Vec3::new(-697.0, -82.0, 794.0));
    assert!(app.world().get::<NetworkNpcVisual0104>(entity).is_none());
    assert!(production_visual_catalog().get(2374).is_none());
    let content = production_content();
    assert!(
        tutorial_target_sample(
            &content,
            entity,
            &actor,
            transform,
            transform.translation,
            Vec3::NEG_Z,
            TutorialTargetingProfile::default(),
        )
        .is_none(),
        "legacy m_iNpcType 111 location actors are excluded from GetConeList"
    );
}

#[test]
fn rapid_combat_skip_warp_cancels_run_before_final_attack_stances() {
    let mut app = app();
    {
        let mut queue = app.world_mut().resource_mut::<TutorialActorCommandQueue>();
        queue.spawn(spawn(100, 2669, [0, 0, 0], None));
        queue.spawn(spawn(101, 2670, [100, 0, 0], None));
        queue.move_native(100, Vec3::new(20.0, 0.0, 0.0), 6.0);
        queue.move_native(101, Vec3::new(20.0, 0.0, 0.0), 6.0);
    }
    app.update();

    let numbuh_five = app
        .world()
        .resource::<TutorialActorRegistry>()
        .entity(100)
        .unwrap();
    let ben = app
        .world()
        .resource::<TutorialActorRegistry>()
        .entity(101)
        .unwrap();
    assert!(
        app.world()
            .get::<TutorialActorMotion>(numbuh_five)
            .is_some()
    );
    assert!(app.world().get::<TutorialActorMotion>(ben).is_some());

    // Exact EventCut order: WarpNpc followed by the two persistent combat
    // poses. A motion queued by the interrupted coroutine must not survive
    // that atomic command batch.
    {
        let mut queue = app.world_mut().resource_mut::<TutorialActorCommandQueue>();
        queue.warp_native(101, Vec3::new(5.0, 0.0, 6.0));
        queue.warp_native(100, Vec3::new(4.0, 0.0, 5.0));
        queue.play_pose(100, "melee1", false);
        queue.play_pose(101, "melee1event", false);
    }
    app.update();

    assert!(
        app.world()
            .get::<TutorialActorMotion>(numbuh_five)
            .is_none()
    );
    assert!(app.world().get::<TutorialActorMotion>(ben).is_none());
    assert_eq!(
        app.world()
            .get::<TutorialActorPose>(numbuh_five)
            .unwrap()
            .clip,
        Some("melee1")
    );
    assert_eq!(
        app.world().get::<TutorialActorPose>(ben).unwrap().clip,
        Some("melee1event")
    );
}

pub(super) fn playback_fixture(
    state: TutorialActorPoseState,
    request_serial: u64,
    restart_serial: u64,
) -> TutorialActorPose {
    TutorialActorPose {
        clip: Some("think"),
        resolved_clip: Some("think"),
        once: false,
        role: LegacyNpcAnimationRole::Forced,
        blend: LegacyAnimationBlend::CrossFade300Ms,
        state,
        force_update: false,
        request_serial,
        restart_serial,
    }
}

pub(super) fn applied_fixture(
    state: TutorialActorPoseState,
    request_serial: u64,
    restart_serial: u64,
) -> TutorialActorAnimationPlayback {
    TutorialActorAnimationPlayback {
        actor_root: Entity::from_bits(1),
        request_serial,
        restart_serial,
        clip: Some("think"),
        resolved_clip: Some("think"),
        node: None,
        additive: false,
        once: false,
        state,
        force_update: false,
        terminally_unavailable: false,
    }
}

#[test]
fn playback_control_resolves_restart_pause_stop_and_non_destructive_holds() {
    let playing = playback_fixture(TutorialActorPoseState::Playing, 2, 2);
    let old = applied_fixture(TutorialActorPoseState::Playing, 1, 1);
    assert_eq!(
        tutorial_actor_animation_control(&playing, Some(&old), true, false),
        TutorialActorAnimationControl::Restart { paused: false }
    );
    let current = applied_fixture(TutorialActorPoseState::Playing, 2, 2);
    assert_eq!(
        tutorial_actor_animation_control(&playing, Some(&current), true, false),
        TutorialActorAnimationControl::Resume
    );

    let paused = playback_fixture(TutorialActorPoseState::Paused, 3, 2);
    assert_eq!(
        tutorial_actor_animation_control(&paused, Some(&current), true, false),
        TutorialActorAnimationControl::Pause
    );
    let forced_pause = playback_fixture(TutorialActorPoseState::Paused, 4, 4);
    assert_eq!(
        tutorial_actor_animation_control(&forced_pause, Some(&current), true, false),
        TutorialActorAnimationControl::Restart { paused: true }
    );
    let forced_stop = playback_fixture(TutorialActorPoseState::ForcedStop, 5, 4);
    assert_eq!(
        tutorial_actor_animation_control(&forced_stop, Some(&current), true, false),
        TutorialActorAnimationControl::Stop
    );
    let dead_message = playback_fixture(TutorialActorPoseState::Dead, 5, 4);
    assert_eq!(
        tutorial_actor_animation_control(&dead_message, Some(&current), true, false),
        TutorialActorAnimationControl::Hold
    );
    assert_eq!(
        tutorial_actor_animation_control(&playing, Some(&current), true, true),
        TutorialActorAnimationControl::Hold
    );
}

#[test]
fn combat_completion_keeps_the_last_base_node_until_ready_is_applied() {
    let mut app = app();
    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .spawn(spawn(1005, 2675, [0, 0, 0], None));
    app.update();
    let actor_root = app
        .world()
        .resource::<TutorialActorRegistry>()
        .entity(1005)
        .unwrap();
    let request_serial = 17;
    app.world_mut().entity_mut(actor_root).insert((
        TutorialActorPose {
            clip: Some("melee1"),
            resolved_clip: Some("melee1"),
            once: true,
            role: LegacyNpcAnimationRole::Melee,
            blend: LegacyAnimationBlend::CrossFade100Ms,
            state: TutorialActorPoseState::Playing,
            force_update: false,
            request_serial,
            restart_serial: request_serial,
        },
        TutorialActorCombatTransition {
            request_serial,
            completion: TutorialActorCombatCompletion::Ready,
        },
    ));
    let (_, node) = AnimationGraph::from_clip(Handle::<AnimationClip>::default());
    let mut player = AnimationPlayer::default();
    player.start(node).set_repeat(RepeatAnimation::Never);
    let player_entity = app
        .world_mut()
        .spawn((
            player,
            TutorialActorAnimationPlayback {
                actor_root,
                request_serial,
                restart_serial: request_serial,
                clip: Some("melee1"),
                resolved_clip: Some("melee1"),
                node: Some(node),
                additive: false,
                once: true,
                state: TutorialActorPoseState::Playing,
                force_update: false,
                // This makes the request terminal for the completion
                // owner without removing the valid node under test.
                terminally_unavailable: true,
            },
        ))
        .id();

    app.update();

    let pose = app.world().get::<TutorialActorPose>(actor_root).unwrap();
    assert_eq!(pose.role, LegacyNpcAnimationRole::Ready);
    assert!(
        app.world()
            .get::<AnimationPlayer>(player_entity)
            .unwrap()
            .animation(node)
            .is_some(),
        "NpcAnimation completion must not expose an empty skeleton frame"
    );
}

#[test]
fn combat_completion_removes_only_the_finished_native_additive_node() {
    let mut app = app();
    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .spawn(spawn(1006, 2676, [0, 0, 0], None));
    app.update();
    let actor_root = app
        .world()
        .resource::<TutorialActorRegistry>()
        .entity(1006)
        .unwrap();
    let request_serial = 23;
    app.world_mut().entity_mut(actor_root).insert((
        TutorialActorPose {
            clip: Some("wound"),
            resolved_clip: Some("wound"),
            once: true,
            role: LegacyNpcAnimationRole::Wound,
            blend: LegacyAnimationBlend::CrossFade100Ms,
            state: TutorialActorPoseState::Playing,
            force_update: false,
            request_serial,
            restart_serial: request_serial,
        },
        TutorialActorCombatTransition {
            request_serial,
            completion: TutorialActorCombatCompletion::Ready,
        },
    ));
    let (_, nodes) = AnimationGraph::from_clips([
        Handle::<AnimationClip>::default(),
        Handle::<AnimationClip>::default(),
    ]);
    let base_node = nodes[0];
    let wound_node = nodes[1];
    let mut player = AnimationPlayer::default();
    player.start(base_node).set_repeat(RepeatAnimation::Forever);
    player.start(wound_node).set_repeat(RepeatAnimation::Never);
    let player_entity = app
        .world_mut()
        .spawn((
            player,
            TutorialActorAnimationPlayback {
                actor_root,
                request_serial,
                restart_serial: request_serial,
                clip: Some("wound"),
                resolved_clip: Some("wound"),
                node: Some(wound_node),
                additive: true,
                once: true,
                state: TutorialActorPoseState::Playing,
                force_update: false,
                // Makes the completion deterministic without advancing a
                // real clip while retaining both live graph nodes.
                terminally_unavailable: true,
            },
        ))
        .id();

    app.update();

    let player = app.world().get::<AnimationPlayer>(player_entity).unwrap();
    assert!(player.animation(base_node).is_some());
    assert!(player.animation(wound_node).is_none());
    assert_eq!(
        app.world()
            .get::<TutorialActorPose>(actor_root)
            .unwrap()
            .role,
        LegacyNpcAnimationRole::Ready
    );
}

#[test]
fn installed_delta_ready_mob_high_layer_clips_start_at_additive_identity() {
    for relative_path in [
        "characters/mobs/mob_cerberus/mob_cerberus.glb",
        "characters/mobs/mob_oilmonster/mob_oilmonster.glb",
        "characters/mobs/mob_sneakyspawn/mob_sneakyspawn.glb",
    ] {
        let gltf = gltf::Gltf::open(asset_root().join(relative_path)).unwrap_or_else(|error| {
            panic!("shared {relative_path} must be installed: {error}")
        });
        let binary = gltf.blob.as_deref();
        let mut delta_clips = BTreeSet::new();

        for animation in gltf.animations() {
            let name = animation
                .name()
                .expect("published animation must retain its name");
            if !legacy_npc_clip_is_additive(name) {
                continue;
            }
            delta_clips.insert(name.to_owned());
            for channel in animation.channels() {
                let value_index = usize::from(matches!(
                    channel.sampler().interpolation(),
                    gltf::animation::Interpolation::CubicSpline
                ));
                let outputs = channel
                    .reader(|buffer| match buffer.source() {
                        gltf::buffer::Source::Bin => binary,
                        gltf::buffer::Source::Uri(_) => None,
                    })
                    .read_outputs()
                    .expect("published animation channel must have outputs");
                match outputs {
                    gltf::animation::util::ReadOutputs::Translations(mut values) => {
                        let value = values
                            .nth(value_index)
                            .expect("translation channel must have a first key value");
                        assert!(
                            value.iter().all(|component| component.abs() <= 1.0e-5),
                            "{relative_path} {name} translation starts at {value:?}, not additive zero"
                        );
                    }
                    gltf::animation::util::ReadOutputs::Rotations(values) => {
                        let value = values
                            .into_f32()
                            .nth(value_index)
                            .expect("rotation channel must have a first key value");
                        assert!(
                            value[0].abs() <= 1.0e-5
                                && value[1].abs() <= 1.0e-5
                                && value[2].abs() <= 1.0e-5
                                && (value[3] - 1.0).abs() <= 1.0e-5,
                            "{relative_path} {name} rotation starts at {value:?}, not additive identity"
                        );
                    }
                    gltf::animation::util::ReadOutputs::Scales(mut values) => {
                        let value = values
                            .nth(value_index)
                            .expect("scale channel must have a first key value");
                        assert!(
                            value.iter().all(|component| component.abs() <= 1.0e-5),
                            "{relative_path} {name} scale starts at {value:?}, not additive zero"
                        );
                    }
                    gltf::animation::util::ReadOutputs::MorphTargetWeights(_) => {
                        panic!(
                            "{relative_path} {name} unexpectedly contains morph-target animation"
                        )
                    }
                }
            }
        }

        assert_eq!(
            delta_clips,
            BTreeSet::from(["melee1".to_owned(), "melee2".to_owned(), "wound".to_owned()]),
            "{relative_path} must publish every MakeUperLayer combat clip as a Bevy-ready delta"
        );
    }
}

#[test]
fn converted_high_layer_clips_use_short_safe_native_cross_fades() {
    assert_eq!(
        tutorial_actor_native_combat_blend(LegacyNpcAnimationRole::Melee),
        LegacyAnimationBlend::CrossFade100Ms
    );
    assert_eq!(
        tutorial_actor_native_combat_blend(LegacyNpcAnimationRole::Wound),
        LegacyAnimationBlend::CrossFade100Ms
    );
    assert_eq!(
        tutorial_actor_native_combat_blend(LegacyNpcAnimationRole::Death),
        LegacyAnimationBlend::CrossFade300Ms
    );
}

#[test]
fn idle_semantic_resolver_matches_force_stand_motion_weights_and_repeat_rule() {
    for (roll, expected) in [
        (0, "stand1"),
        (39, "stand1"),
        (40, "stand2"),
        (69, "stand2"),
        (70, "stand3"),
        (89, "stand3"),
        (90, "stand4"),
        (99, "stand4"),
    ] {
        assert_eq!(tutorial_actor_idle_stand_for_roll(roll, None), expected);
    }
    assert_eq!(
        tutorial_actor_idle_stand_for_roll(40, Some("stand2")),
        "stand1"
    );
    assert_eq!(
        tutorial_actor_idle_stand_for_roll(70, Some("stand3")),
        "stand1"
    );
    assert_eq!(
        tutorial_actor_idle_stand_for_roll(90, Some("stand4")),
        "stand1"
    );
    assert_eq!(
        tutorial_actor_idle_stand_for_roll(0, Some("stand1")),
        "stand1"
    );
}

#[test]
fn movement_reclaims_locomotion_and_run_arrival_enters_attack_ready() {
    let mut app = app();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
        0.25,
    )));
    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .spawn(spawn(100, 2669, [0, 0, 0], None));
    app.update();
    let entity = app
        .world()
        .resource::<TutorialActorRegistry>()
        .entity(100)
        .unwrap();
    // Keep this state-machine test independent from the no-terrain gravity
    // fallback; grounding itself has a dedicated parity test above.
    app.world_mut().get_mut::<TutorialActor>(entity).unwrap().hp = 0;
    let target = app.world().get::<Transform>(entity).unwrap().translation + Vec3::X * 5.0;

    {
        let mut queue = app.world_mut().resource_mut::<TutorialActorCommandQueue>();
        queue.move_native(100, target, 7.0);
    }
    app.update();
    assert_eq!(
        app.world().get::<TutorialActorPose>(entity).unwrap().role,
        LegacyNpcAnimationRole::Locomotion
    );

    // A high/combat completion enters ready first. While MoveNpc is still
    // active, the next FixedUpdate-equivalent must immediately reclaim run.
    let pose = *app.world().get::<TutorialActorPose>(entity).unwrap();
    app.world_mut()
        .entity_mut(entity)
        .insert(TutorialActorPose {
            clip: Some("ready"),
            resolved_clip: Some("ready"),
            role: LegacyNpcAnimationRole::Ready,
            blend: LegacyAnimationBlend::CrossFade200Ms,
            request_serial: pose.request_serial.wrapping_add(1),
            restart_serial: pose.restart_serial.wrapping_add(1),
            ..pose
        });
    app.update();
    let pose = app.world().get::<TutorialActorPose>(entity).unwrap();
    assert_eq!(pose.clip, Some("run"));
    assert_eq!(pose.role, LegacyNpcAnimationRole::Locomotion);

    app.update();
    let pose = app.world().get::<TutorialActorPose>(entity).unwrap();
    assert_eq!(pose.clip, Some("ready"));
    assert_eq!(pose.role, LegacyNpcAnimationRole::Ready);
    assert_eq!(pose.blend, LegacyAnimationBlend::CrossFade200Ms);
    assert!(app.world().get::<TutorialActorMotion>(entity).is_none());
}

#[test]
fn fusion_spawn_walk_ignores_target_height_and_finishes_in_a_stand() {
    let mut app = app();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
        0.25,
    )));
    app.insert_resource(TutorialActorStandRandomStream::with_seed(0x1234_5678));
    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .spawn(spawn(1004, 2897, [0, 0, 0], None));
    app.update();

    let entity = app
        .world()
        .resource::<TutorialActorRegistry>()
        .entity(1004)
        .unwrap();
    // Keep this locomotion contract independent from terrain fixtures.
    app.world_mut().get_mut::<TutorialActor>(entity).unwrap().hp = 0;
    let start = app.world().get::<Transform>(entity).unwrap().translation;
    let target = Vec3::new(start.x + 1.0, start.y - 100.0, start.z);
    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .move_native(1004, target, 3.0);

    app.update();
    let moving = app.world().get::<TutorialActorPose>(entity).unwrap();
    assert_eq!(moving.clip, Some("walk"));
    assert_eq!(moving.role, LegacyNpcAnimationRole::Locomotion);
    assert_eq!(
        app.world().get::<Transform>(entity).unwrap().translation.y,
        start.y,
        "MoveNpc must not consume the server target height"
    );

    app.update();
    let arrived = app.world().get::<TutorialActorPose>(entity).unwrap();
    assert_eq!(arrived.clip, Some("idle"));
    assert_eq!(arrived.role, LegacyNpcAnimationRole::Stand);
    assert!(TUTORIAL_ACTOR_STAND_CLIPS.contains(&arrived.resolved_clip.unwrap()));
    assert!(app.world().get::<TutorialActorMotion>(entity).is_none());
    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_eq!(transform.translation.x, target.x);
    assert_eq!(transform.translation.z, target.z);
    assert_eq!(transform.translation.y, start.y);
}

pub(super) fn gather_spawn_types(action: NpcAction, types: &mut BTreeMap<i32, BTreeSet<i32>>) {
    let mut add = |spawn: NpcSpawn| {
        types.entry(spawn.id).or_default().insert(spawn.npc_type);
    };
    match action {
        NpcAction::Spawn(spawn) => add(spawn),
        NpcAction::SpawnBatch(spawns) => {
            for spawn in spawns {
                add(*spawn);
            }
        }
        _ => {}
    }
}

#[test]
fn fifo_commands_use_xdt_hp_and_emit_first_hit_then_single_death() {
    let mut app = app();
    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .spawn(spawn(MISSION_TARGET_ID, 2676, [0, 0, 0], None));
    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .damage(MISSION_TARGET_ID, 200);
    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .damage(MISSION_TARGET_ID, 1_100);
    app.world_mut()
        .resource_mut::<TutorialActorCommandQueue>()
        .damage(MISSION_TARGET_ID, 1);
    app.update();

    let entity = app
        .world()
        .resource::<TutorialActorRegistry>()
        .entity(MISSION_TARGET_ID)
        .unwrap();
    let actor = app.world().get::<TutorialActor>(entity).unwrap();
    assert_eq!((actor.max_hp, actor.hp, actor.team), (1300, 0, 2));
    assert!(actor.damaged);

    let events = app
        .world_mut()
        .resource_mut::<TutorialActorEventQueue>()
        .take_all();
    assert_eq!(
        events,
        VecDeque::from([
            TutorialActorEvent::Damaged {
                id: MISSION_TARGET_ID,
                entity,
                amount: 200,
                remaining_hp: 1100,
                first_hit: true,
            },
            TutorialActorEvent::Damaged {
                id: MISSION_TARGET_ID,
                entity,
                amount: 1100,
                remaining_hp: 0,
                first_hit: false,
            },
            TutorialActorEvent::Dead {
                id: MISSION_TARGET_ID,
                entity,
            },
        ])
    );
}

#[test]
fn observation_builder_maps_all_named_ids_and_preserves_damaged_override() {
    let observation = build_tutorial_npc_observation(
        Vec3::ZERO,
        [
            TutorialActorObservationSample {
                id: NUMBUH_TWO_ID,
                position: Vec3::new(3.0, 4.0, 0.0),
                damaged: true,
                interacting: false,
            },
            TutorialActorObservationSample {
                id: FUSION_PORTAL_ID,
                position: Vec3::new(0.0, 0.0, 12.0),
                damaged: false,
                interacting: true,
            },
            TutorialActorObservationSample {
                id: DEMO_MONSTER_ID,
                position: Vec3::new(0.0, 0.0, 7.0),
                damaged: false,
                interacting: false,
            },
        ],
    );
    assert_eq!(observation.numbuh_two.distance, Some(5.0));
    assert!(observation.numbuh_two.damaged);
    assert!(observation.numbuh_two.within_or_damaged(0.1));
    assert!(observation.fusion_portal.interacting);
    assert_eq!(observation.demo_monster.distance, Some(7.0));
    assert_eq!(observation.buttercup, ObservedNpc::default());
}

#[test]
fn target_feed_uses_original_level_one_cones_and_tutorial_talk_range() {
    let profile = TutorialTargetingProfile::default();
    let content = production_content();
    let hostile = TutorialActor {
        id: 1,
        npc_type: 2674,
        team: 2,
        hp: 400,
        max_hp: 400,
        damaged: false,
        interacting: false,
        invulnerable: false,
    };
    let sample = tutorial_target_sample(
        &content,
        Entity::from_bits(1),
        &hostile,
        &Transform::from_xyz(0.0, 0.0, -2.8),
        Vec3::ZERO,
        Vec3::NEG_Z,
        profile,
    )
    .unwrap();
    assert!(sample.in_view);
    assert!(sample.in_attack_arc);
    assert!(sample.in_attack_cone, "2.0 range plus 0.9 body extent");
    assert!(!sample.talk_enabled);

    let friendly = TutorialActor {
        id: NUMBUH_TWO_ID,
        npc_type: 2671,
        team: 1,
        hp: 361,
        max_hp: 361,
        damaged: false,
        interacting: false,
        invulnerable: false,
    };
    let sample = tutorial_target_sample(
        &content,
        Entity::from_bits(2),
        &friendly,
        &Transform::from_xyz(0.0, 0.0, -5.99),
        Vec3::ZERO,
        Vec3::NEG_Z,
        profile,
    )
    .unwrap();
    assert!(sample.in_view);
    assert!(sample.in_attack_arc);
    assert!(sample.talk_enabled);

    let cutscene_only = TutorialActor {
        id: 100,
        npc_type: 2669,
        team: 1,
        hp: 361,
        max_hp: 361,
        damaged: false,
        interacting: false,
        invulnerable: false,
    };
    assert!(
        tutorial_target_sample(
            &content,
            Entity::from_bits(3),
            &cutscene_only,
            &Transform::from_xyz(0.0, 0.0, -1.0),
            Vec3::ZERO,
            Vec3::NEG_Z,
            profile,
        )
        .is_none(),
        "legacy m_iNpcType 25 actors are excluded from NpcContainer.GetConeList"
    );
}
