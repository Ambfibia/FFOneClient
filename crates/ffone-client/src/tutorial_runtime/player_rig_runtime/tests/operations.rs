use super::*;

#[test]
fn hit_stop_does_not_leave_the_replaced_attack_blended_into_later_poses() {
    use bevy::animation::{AnimatedBy, AnimationPlugin, AnimationTargetId, animated_field};
    use bevy::math::curve::{EaseFunction, EasingCurve};
    use bevy::time::TimeUpdateStrategy;

    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
        AnimationPlugin,
    ))
    .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        50,
    )));
    let bone = AnimationTargetId::from_name(&Name::new("Bip01 R UpperArm"));
    let attack_pose = Quat::from_rotation_x(1.0);
    let run_pose = Quat::from_rotation_z(0.5);
    let [attack, run] = [attack_pose, run_pose].map(|pose| {
        let mut clip = AnimationClip::default();
        clip.add_curve_to_target(
            bone,
            AnimatableCurve::new(
                animated_field!(Transform::rotation),
                EasingCurve::new(pose, pose, EaseFunction::Linear),
            ),
        );
        app.world_mut()
            .resource_mut::<Assets<AnimationClip>>()
            .add(clip)
    });
    let (graph, nodes) = AnimationGraph::from_clips([attack, run]);
    let graph = app
        .world_mut()
        .resource_mut::<Assets<AnimationGraph>>()
        .add(graph);
    let mut spawn = |release: bool| {
        let mut player = AnimationPlayer::default();
        let mut transitions = AnimationTransitions::new();
        transitions
            .play(&mut player, nodes[0], Duration::ZERO)
            .set_repeat(RepeatAnimation::Never);
        // `DelayCurrent` pauses the stationary shot; a direction key then
        // cross-fades the base to run before the hit-stop expires.
        player.animation_mut(nodes[0]).unwrap().pause();
        if release {
            release_paused_tutorial_main_animation(&mut player, &transitions, nodes[1]);
        }
        transitions
            .play(&mut player, nodes[1], Duration::from_millis(150))
            .repeat();
        let player = app
            .world_mut()
            .spawn((player, transitions, AnimationGraphHandle(graph.clone())))
            .id();
        let bone_entity = app
            .world_mut()
            .spawn((Transform::default(), (bone, AnimatedBy(player))))
            .id();
        (player, bone_entity)
    };
    let (released, released_bone) = spawn(true);
    let (stale, _) = spawn(false);
    for _ in 0..10 {
        app.update();
    }
    let world = app.world();
    assert!(
        world
            .get::<AnimationPlayer>(released)
            .unwrap()
            .animation(nodes[0])
            .is_none()
    );
    let pose = world.get::<Transform>(released_bone).unwrap().rotation;
    assert!(pose.angle_between(run_pose) < 1e-3, "{pose:?}");
    // Control: Bevy keeps a paused outgoing main animation active.
    assert!(
        world
            .get::<AnimationPlayer>(stale)
            .unwrap()
            .animation(nodes[0])
            .is_some()
    );
}

#[test]
fn hit_stop_expiry_resumes_the_current_base_and_upper_nodes() {
    let [captured, twin, upper] = [1, 2, 3].map(AnimationNodeIndex::new);
    let mut player = AnimationPlayer::default();
    for node in [captured, twin, upper] {
        player.start(node).pause();
    }
    // A composed-mask switch handed the paused base to its twin node.
    let delay = TutorialAnimationDelayPlayback {
        remaining_seconds: 0.0,
        base: true,
        upper: false,
    };
    resume_tutorial_delayed_playback(&mut player, delay, twin, Some(upper));
    assert!(!player.animation(twin).unwrap().is_paused());
    assert!(player.animation(upper).unwrap().is_paused());
    resume_tutorial_delayed_playback(
        &mut player,
        TutorialAnimationDelayPlayback {
            upper: true,
            ..delay
        },
        twin,
        Some(upper),
    );
    assert!(!player.animation(upper).unwrap().is_paused());
}

#[test]
fn gameplay_weapon_materials_bind_production_item_textures_once_without_mutating_source() {
    use crate::legacy_model_material::LegacyModelTextures;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let data = crate::character_creation_data::CharacterCreationData::open(&root).unwrap();
    for gender in [PlayerRigGender::Male, PlayerRigGender::Female] {
        for item_id in [43, 197, 328] {
            let part = data
                .resolve_weapon_attachment(item_id as u32, gender)
                .unwrap();
            let expected = part.primary_texture.as_ref().unwrap().path.clone();
            assert!(root.join(&expected).is_file());
            let bytes = std::fs::read(root.join(&part.glb)).unwrap();
            let len = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
            let document: serde_json::Value =
                serde_json::from_slice(&bytes[20..20 + len]).unwrap();
            let source = document["materials"]
                .as_array()
                .unwrap()
                .iter()
                .find(|material| {
                    material["name"]
                        .as_str()
                        .is_some_and(|name| name.contains("main"))
                })
                .unwrap();
            let pending = PendingLegacyModelMaterial::from_gltf_extras(
                source["name"].as_str(),
                &source["extras"].to_string(),
            )
            .unwrap();
            let material = pending
                .params
                .render_plan()
                .passes
                .iter()
                .find_map(|pass| {
                    pending
                        .params
                        .material_for_pass(*pass, &LegacyModelTextures::default())
                })
                .unwrap();
            let baseline = material.clone();
            let mut app = App::new();
            app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()))
                .init_asset::<Image>()
                .init_asset::<LegacyModelMaterial>()
                .init_resource::<TutorialPlayerRigIssueQueue>()
                .add_systems(Update, bind_tutorial_player_weapon_materials);
            let source_handle = app
                .world_mut()
                .resource_mut::<Assets<LegacyModelMaterial>>()
                .add(material);
            let rig_root = app.world_mut().spawn_empty().id();
            let attachment = app
                .world_mut()
                .spawn((
                    TutorialPlayerWeaponAttachment {
                        rig_root,
                        item_id,
                        exact_route: part.exact_route.clone(),
                        socket_full_path: String::new(),
                    },
                    TutorialPlayerWeaponLook(NativePlayerLook {
                        identity: "weapon binding regression".into(),
                        gender,
                        parts: vec![part],
                        skin_texture: None,
                        skin_color: LinearRgba::WHITE,
                        hair_color: LinearRgba::WHITE,
                        weapon_animation_profile: None,
                        height_selector: 0,
                        body_selector: 0,
                    }),
                ))
                .id();
            // The scene and its material metadata can arrive after equip.
            app.update();
            let surface = app
                .world_mut()
                .spawn((
                    ChildOf(attachment),
                    pending,
                    MeshMaterial3d(source_handle.clone()),
                ))
                .id();
            app.update();
            let assigned = app
                .world()
                .get::<MeshMaterial3d<LegacyModelMaterial>>(surface)
                .unwrap()
                .0
                .clone();
            assert_ne!(assigned, source_handle);
            let assets = app.world().resource::<Assets<LegacyModelMaterial>>();
            assert_eq!(assets.get(&source_handle).unwrap(), &baseline);
            let texture = assets
                .get(&assigned)
                .unwrap()
                .base_texture
                .as_ref()
                .unwrap();
            assert_eq!(texture.path().unwrap().path().to_str().unwrap(), expected);
            assert!(
                app.world()
                    .resource::<TutorialPlayerRigIssueQueue>()
                    .pending
                    .is_empty()
            );
            app.update();
            assert_eq!(
                app.world()
                    .get::<MeshMaterial3d<LegacyModelMaterial>>(surface)
                    .unwrap()
                    .0,
                assigned
            );
        }
    }
}

#[test]
fn emote_interruption_requires_an_accepted_attack() {
    use crate::avatar_action::*;
    for blocked in [false, true] {
        let frame = schedule_legacy_action_frame(
            LegacyAvatarActionInput {
                primary_held: true,
                ..default()
            },
            &LegacyAvatarActionContext {
                attack_cooldown_blocked: blocked,
                ..default()
            },
            &LegacyTargetSelection::default(),
            &mut LegacyAvatarActionState::default(),
            &LegacyAvatarClipBindings::default(),
            1.0 / 60.0,
        );
        assert_eq!(
            frame.visuals.iter().any(attack_interrupts_player_emote),
            !blocked
        );
    }
}

#[test]
fn traversal_models_keep_identity_roots_and_gender_specific_sockets() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    for path in [SKYWAY_MODEL_PATH, ZIPLINE_MODEL_PATH] {
        let model = gltf::Gltf::open(root.join(path)).unwrap();
        let nodes: Vec<_> = model.default_scene().unwrap().nodes().collect();
        assert_eq!(nodes.len(), 1);
        let (translation, rotation, scale) = nodes[0].transform().decomposed();
        assert_eq!(translation, [0.0; 3]);
        assert_eq!(rotation, [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(scale, [1.0; 3]);
    }
    let catalog = NativePlayerRigCatalog::open(root).unwrap();
    for gender in [PlayerRigGender::Male, PlayerRigGender::Female] {
        let contract = catalog.gender(gender).unwrap();
        // Production paths must remain distinct: the trolley follows the
        // right hand; the monkey follows the dedicated pelvis socket.
        assert_ne!(
            skyway_socket_full_path(gender),
            player_attachment_socket_full_path(gender, LegacyPlayerAttachmentSlot::Zipline)
        );
        for clip in ["mount1", "mount2", "ropedown"] {
            assert!(contract.clips.iter().any(|entry| entry.name == clip));
        }
    }
}

#[test]
fn production_contract_resolves_required_death_and_unarmed_attacks_for_both_genders() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = NativePlayerRigCatalog::open(root).unwrap();
    for gender in [PlayerRigGender::Male, PlayerRigGender::Female] {
        let gender_contract = catalog.gender(gender).unwrap();
        let primary_tutorial_sources = match gender {
            PlayerRigGender::Male => [
                ("stand1", 237_023),
                ("staying", 34_269),
                ("standup", 34_404),
            ],
            PlayerRigGender::Female => [
                ("stand1", 237_027),
                ("staying", 34_194),
                ("standup", 34_533),
            ],
        };
        for (name, source_path_id) in primary_tutorial_sources {
            let clip = gender_contract
                .clips
                .iter()
                .find(|clip| clip.name == name)
                .unwrap_or_else(|| panic!("missing production {gender:?} {name}"));
            assert_eq!(clip.source_path_id, source_path_id);
        }
        let capabilities =
            TutorialPlayerRigCapabilities::from_contract(gender, &gender_contract.clips)
                .unwrap();
        let indices = resolve_required_tutorial_player_clips(&capabilities, gender).unwrap();
        assert!(indices.contains_key(&TutorialPlayerClip::Die));
        assert!(indices.contains_key(&TutorialPlayerClip::Death));
        assert!(indices.contains_key(&TutorialPlayerClip::Attack1));
        assert!(indices.contains_key(&TutorialPlayerClip::Attack1Upper));
        assert_ne!(
            indices[&TutorialPlayerClip::Die],
            indices[&TutorialPlayerClip::Death]
        );
        assert_ne!(
            indices[&TutorialPlayerClip::Attack1],
            indices[&TutorialPlayerClip::Attack1Upper]
        );
    }
}

#[test]
fn body_shape_layers_use_actor_skin_combiner_sampling_direction() {
    assert_eq!(legacy_body_shape_normalized_times(0, 0), (1.0, 0.0));
    assert_eq!(legacy_body_shape_normalized_times(2, 1), (0.5, 0.5));
    assert_eq!(legacy_body_shape_normalized_times(4, 2), (0.0, 1.0));
    assert_eq!(legacy_body_shape_normalized_times(-1, 9), (1.0, 1.0));
}

#[test]
fn upper_attack_layer_fades_in_and_out_without_a_mask_switch() {
    let (_, nodes) = AnimationGraph::from_clips([Handle::<AnimationClip>::default()]);
    let mut player = AnimationPlayer::default();
    player.start(nodes[0]).set_weight(0.0);
    let mut upper = TutorialUpperLayerPlayback {
        node: nodes[0],
        elapsed_seconds: 0.0,
        blend_seconds: 0.15,
        base_masked: false,
        fading_out: false,
    };

    assert!(!advance_tutorial_upper_layer_playback(
        &mut upper,
        &mut player,
        0.075,
    ));
    let middle_weight = player.animation(nodes[0]).unwrap().weight();
    assert!((middle_weight - 1.0).abs() <= 0.000_01);
    assert!(!advance_tutorial_upper_layer_playback(
        &mut upper,
        &mut player,
        0.075,
    ));
    assert!(player.animation(nodes[0]).unwrap().weight() > 100_000.0);

    upper.fading_out = true;
    upper.elapsed_seconds = 0.0;
    assert!(!advance_tutorial_upper_layer_playback(
        &mut upper,
        &mut player,
        0.075,
    ));
    assert!((player.animation(nodes[0]).unwrap().weight() - 1.0).abs() <= 0.000_01);
    assert!(advance_tutorial_upper_layer_playback(
        &mut upper,
        &mut player,
        0.075,
    ));
    assert_eq!(player.animation(nodes[0]).unwrap().weight(), 0.0);
}

#[test]
fn primary_clips_use_end_events_instead_of_trailing_recovery_keys() {
    assert_eq!(
        tutorial_player_clip_end_event_seconds(
            PlayerRigGender::Male,
            TutorialPlayerClip::Attack1,
        ),
        Some(0.75)
    );
    assert_eq!(
        tutorial_player_clip_end_event_seconds(
            PlayerRigGender::Female,
            TutorialPlayerClip::Attack1Upper,
        ),
        Some(0.75)
    );
    assert_eq!(
        tutorial_player_clip_end_event_seconds(
            PlayerRigGender::Female,
            TutorialPlayerClip::Standup,
        ),
        Some(2.083_333_5)
    );
    assert_eq!(
        tutorial_player_clip_end_event_seconds(
            PlayerRigGender::Male,
            TutorialPlayerClip::JumpEnd,
        ),
        Some(0.183_333_34)
    );
    assert_eq!(
        tutorial_player_clip_end_event_seconds(
            PlayerRigGender::Female,
            TutorialPlayerClip::JumpEnd,
        ),
        Some(0.216_666_67)
    );
    assert_eq!(
        tutorial_player_clip_end_event_seconds(
            PlayerRigGender::Female,
            TutorialPlayerClip::RocketJumpEnd,
        ),
        Some(0.183_333_34)
    );
    assert_eq!(
        tutorial_player_clip_end_event_seconds(
            PlayerRigGender::Male,
            TutorialPlayerClip::RifleJumpLandRun,
        ),
        Some(0.183_333_34)
    );
    assert_eq!(
        tutorial_player_clip_end_event_seconds(
            PlayerRigGender::Male,
            TutorialPlayerClip::RifleAttack1Upper,
        ),
        None
    );
    assert!(upper_event_releases_immediately(
        TutorialPlayerClip::Attack1Upper,
        LegacyLocomotionState::Run,
    ));
    assert!(!upper_event_releases_immediately(
        TutorialPlayerClip::Attack1Upper,
        LegacyLocomotionState::Stand,
    ));
    assert!(!upper_event_releases_immediately(
        TutorialPlayerClip::RifleAttack1Upper,
        LegacyLocomotionState::Run,
    ));
    assert_eq!(
        tutorial_upper_pose_source(TutorialPlayerClip::Attack1Upper),
        TutorialPlayerClip::Attack1
    );
    for weapon_upper in [
        TutorialPlayerClip::StickAttack1Upper,
        TutorialPlayerClip::PistolAttack1Upper,
        TutorialPlayerClip::RifleAttack1Upper,
        TutorialPlayerClip::BombAttack1Upper,
        TutorialPlayerClip::RocketAttack1Upper,
    ] {
        assert_eq!(tutorial_upper_pose_source(weapon_upper), weapon_upper);
    }

    let upper = AnimationTargetId::from_name(&Name::new("upper"));
    let pelvis = AnimationTargetId::from_name(&Name::new("pelvis"));
    let leg = AnimationTargetId::from_name(&Name::new("leg"));
    let upper_targets = BTreeSet::from([upper]);
    let attack_body_chain = BTreeSet::from([pelvis]);
    assert!(!tutorial_unarmed_attack_masks_target(
        &upper,
        &upper_targets,
        &attack_body_chain,
    ));
    assert!(!tutorial_unarmed_attack_masks_target(
        &pelvis,
        &upper_targets,
        &attack_body_chain,
    ));
    assert!(tutorial_unarmed_attack_masks_target(
        &leg,
        &upper_targets,
        &attack_body_chain,
    ));

    let mut graph = AnimationGraph::new();
    let node = add_tutorial_upper_clip(
        &mut graph,
        Handle::<AnimationClip>::default(),
        UNARMED_ATTACK_LOWER_BODY_MASK,
    );
    assert!(graph.graph.find_edge(graph.root, node).is_some());
    assert_eq!(
        graph.get(node).expect("upper clip graph node").mask,
        UNARMED_ATTACK_LOWER_BODY_MASK
    );
}

#[test]
fn emote_return_into_landing_preserves_completion_ownership() {
    let pending = pending_legacy_visual_completion(
        LegacyVisualClip::JumpEnd,
        TutorialPlayerClip::JumpEnd,
    )
    .expect("clamped landing must retain a renderer completion");
    assert_eq!(pending.semantic_clip, LegacyVisualClip::JumpEnd);
    assert_eq!(pending.animation_clip, TutorialPlayerClip::JumpEnd);
    assert!(!pending.armed);
    assert!(
        pending_legacy_visual_completion(LegacyVisualClip::Stand1, TutorialPlayerClip::Stand1,)
            .is_none(),
        "looping locomotion must not manufacture a completion"
    );
}

#[test]
fn gameplay_playback_watchdog_repairs_loops_and_completes_removed_clamps() {
    let (_, nodes) = AnimationGraph::from_clips([Handle::<AnimationClip>::default()]);
    let mut player = AnimationPlayer::default();
    player
        .start(nodes[0])
        .set_repeat(RepeatAnimation::Never)
        .set_speed(0.0)
        .pause();

    assert!(keep_tutorial_gameplay_playback_alive(
        &mut player,
        nodes[0],
        TutorialPlayerClip::Run,
    ));
    let active = player.animation(nodes[0]).unwrap();
    assert!(!active.is_paused());
    assert_eq!(active.speed(), 1.0);
    assert_eq!(active.repeat_mode(), RepeatAnimation::Forever);

    player.stop(nodes[0]);
    assert!(!keep_tutorial_gameplay_playback_alive(
        &mut player,
        nodes[0],
        TutorialPlayerClip::Run,
    ));
    assert!(tutorial_clamp_finished_or_removed(
        &player,
        Some(nodes[0]),
        PlayerRigGender::Male,
        TutorialPlayerClip::StickAttack1,
    ));

    player
        .start(nodes[0])
        .set_repeat(RepeatAnimation::Never)
        .resume();
    assert!(!tutorial_clamp_finished_or_removed(
        &player,
        Some(nodes[0]),
        PlayerRigGender::Male,
        TutorialPlayerClip::StickAttack1,
    ));
    player.animation_mut(nodes[0]).unwrap().set_seek_time(0.19);
    assert!(tutorial_clamp_finished_or_removed(
        &player,
        Some(nodes[0]),
        PlayerRigGender::Male,
        TutorialPlayerClip::JumpEnd,
    ));
    assert!(!tutorial_clamp_finished_or_removed(
        &player,
        Some(nodes[0]),
        PlayerRigGender::Female,
        TutorialPlayerClip::JumpEnd,
    ));
    player.stop(nodes[0]);
    assert!(tutorial_clamp_finished_or_removed(
        &player,
        Some(nodes[0]),
        PlayerRigGender::Male,
        TutorialPlayerClip::StickAttack1,
    ));
}

#[test]
fn missing_player_base_is_reinstalled_atomically_as_transition_main() {
    let (_, nodes) = AnimationGraph::from_clips([Handle::<AnimationClip>::default()]);
    let mut player = AnimationPlayer::default();
    let mut transitions = AnimationTransitions::new();

    restart_missing_tutorial_base_animation(
        &mut player,
        &mut transitions,
        nodes[0],
        TutorialPlayerClip::Run,
    );

    assert_eq!(transitions.get_main_animation(), Some(nodes[0]));
    let active = player
        .animation(nodes[0])
        .expect("base recovery must install a node in the same call");
    assert_eq!(active.repeat_mode(), RepeatAnimation::Forever);
    assert!(!active.is_paused());
    assert_eq!(active.speed(), 1.0);
}

#[test]
fn completed_startup_staying_holds_until_the_loading_barrier_opens() {
    assert!(tutorial_startup_emote_loading_barrier(
        true,
        true,
        TutorialPlayerClip::Staying,
    ));
    assert!(tutorial_emote_holds_locomotion(true, false, true, true));
    assert!(!tutorial_startup_emote_loading_barrier(
        true,
        false,
        TutorialPlayerClip::Staying,
    ));
    assert!(!tutorial_startup_emote_loading_barrier(
        false,
        true,
        TutorialPlayerClip::Staying,
    ));
    assert!(!tutorial_startup_emote_loading_barrier(
        true,
        true,
        TutorialPlayerClip::Standup,
    ));
}

#[test]
fn ready_candidate_cannot_replace_its_active_fallback_outside_owner_commit() {
    let mut app = App::new();
    app.add_systems(Update, swap_tutorial_player_fallback_on_ready);
    let active = app.world_mut().spawn(Visibility::Inherited).id();
    let candidate = app
        .world_mut()
        .spawn((
            TutorialSelectedPlayerRigStatus::Ready,
            Visibility::Hidden,
            TutorialSelectedPlayerRigCandidate { fallback: active },
            TutorialPlayerFallbackVisual {
                entity: Some(active),
            },
        ))
        .id();

    app.update();

    assert!(app.world().get_entity(active).is_ok());
    assert_eq!(
        app.world().get::<Visibility>(candidate),
        Some(&Visibility::Hidden)
    );
    assert!(
        app.world()
            .get::<TutorialPlayerFallbackReplaced>(candidate)
            .is_none()
    );
}

#[test]
fn visible_cosmetics_use_exact_attachment_slots_and_never_duplicate_the_hand() {
    assert_eq!(
        tutorial_player_appearance_attachment_slot(NativePlayerPartKind::Hat),
        Some(LegacyPlayerAttachmentSlot::Hat)
    );
    assert_eq!(
        tutorial_player_appearance_attachment_slot(NativePlayerPartKind::Glasses),
        Some(LegacyPlayerAttachmentSlot::Glasses)
    );
    assert_eq!(
        tutorial_player_appearance_attachment_slot(NativePlayerPartKind::Back),
        Some(LegacyPlayerAttachmentSlot::Back)
    );
    assert_eq!(
        tutorial_player_appearance_attachment_slot(NativePlayerPartKind::Weapon),
        None,
        "the authoritative Hand queue remains the only weapon owner"
    );
}
