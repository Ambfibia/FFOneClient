use super::*;

#[test]
fn fusion_spawn_delta_combat_keeps_network_base_pose_active() {
    assert!(network_npc_animation_uses_delta_additive_layer_0104(
        "characters/mobs/mob_sneakyspawn/mob_sneakyspawn.glb",
        "melee1",
    ));
    assert!(!network_npc_animation_uses_delta_additive_layer_0104(
        "characters/mobs/mob_sneakyspawn/mob_sneakyspawn.glb",
        "ready",
    ));
    assert!(network_npc_animation_uses_delta_additive_layer_0104(
        "characters/mobs/mob_cerberus/mob_cerberus.glb",
        "melee1",
    ));

    let (_, nodes) = AnimationGraph::from_clips([
        Handle::<AnimationClip>::default(),
        Handle::<AnimationClip>::default(),
        Handle::<AnimationClip>::default(),
    ]);
    let pair = NetworkNpcAdditivePair0104 {
        reference: nodes[1],
        clip: nodes[2],
    };
    let prepared = NetworkNpcPreparedAnimationGraph0104 {
        graph: Handle::default(),
        nodes: HashMap::from([("ready".to_owned(), nodes[0])]),
        additive_nodes: HashMap::from([("melee1".to_owned(), pair)]),
        base_nodes: vec![nodes[0]],
        fallback_base_node: Some(nodes[0]),
    };
    let mut player = AnimationPlayer::default();
    let mut transitions = AnimationTransitions::new();
    // A mob hit before its first idle enters AttackReady through the
    // transition owner, then composes the high layer over it.
    cross_fade_network_npc_low_layer_0104(
        &mut player,
        &mut transitions,
        nodes[0],
        Duration::ZERO,
        1.0,
        true,
    );
    restart_network_npc_additive_pair_0104(&mut player, &prepared, pair);

    assert_eq!(transitions.get_main_animation(), Some(nodes[0]));
    assert_eq!(
        player.animation(nodes[0]).unwrap().repeat_mode(),
        RepeatAnimation::Forever
    );
    let clip = player.animation(pair.clip).unwrap();
    assert_eq!(clip.repeat_mode(), RepeatAnimation::Never);
    assert!(!clip.is_paused());
    let reference = player.animation(pair.reference).unwrap();
    assert!(reference.is_paused());
    assert_eq!(reference.seek_time(), 0.0);
}

#[test]
fn remote_pc_appearance_attachment_uses_exact_bone_map_or_blocks_hidden_rig() {
    let mut app = App::new();
    app.add_systems(Update, bind_network_pc_appearance_attachments_0104);

    let pc_root = app.world_mut().spawn_empty().id();
    let rig_root = app.world_mut().spawn_empty().id();
    let mut bone_entities = Vec::new();
    let mut attachments = Vec::new();
    for (bone_index, (kind, slot)) in [
        (NativePlayerPartKind::Hat, LegacyPlayerAttachmentSlot::Hat),
        (
            NativePlayerPartKind::Glasses,
            LegacyPlayerAttachmentSlot::Glasses,
        ),
        (NativePlayerPartKind::Back, LegacyPlayerAttachmentSlot::Back),
    ]
    .into_iter()
    .enumerate()
    {
        let socket_full_path = player_attachment_socket_full_path(
            ffone_runtime_contracts::PlayerRigGender::Male,
            slot,
        );
        let socket = app
            .world_mut()
            .spawn((Transform::from_scale(Vec3::splat(0.25)), ChildOf(rig_root)))
            .id();
        bone_entities.push(crate::player_shared_rig::NativePlayerRigBoneEntity {
            actor_bone_index: bone_index as u32,
            true_name: slot.socket_path().rsplit('/').next().unwrap().to_owned(),
            full_path: socket_full_path.clone(),
            entity: socket,
        });
        let attachment = app
            .world_mut()
            .spawn((
                Transform::default(),
                ChildOf(rig_root),
                NetworkPcAppearancePart0104 {
                    rig_root,
                    kind,
                    exact_route: format!("wear/{kind:?}"),
                    glb: format!("characters/test/{kind:?}.glb"),
                },
                PendingNetworkPcAppearanceAttachment0104 {
                    rig_root,
                    socket_full_path,
                    socket_local_scale_override: Some(Vec3::ONE),
                },
            ))
            .id();
        attachments.push((attachment, socket));
    }
    app.world_mut().entity_mut(rig_root).insert((
        NetworkPcRig0104 {
            pc_root,
            pc_id: 17,
            look: remote_pc_test_look(&[]),
        },
        NativePlayerRigBones::new(bone_entities),
        NativePlayerRigStatus::ReadyAnimated {
            actor_bones: 3,
            parts: 0,
            skin_palettes: 0,
            skinned_surfaces: 0,
        },
        NetworkPcRigAppearanceStatus0104::Loading,
    ));

    app.update();

    for (attachment, socket) in attachments {
        assert_eq!(
            app.world().get::<ChildOf>(attachment).map(ChildOf::parent),
            Some(socket)
        );
        assert!(
            app.world()
                .get::<PendingNetworkPcAppearanceAttachment0104>(attachment)
                .is_none()
        );
        assert_eq!(
            app.world().get::<Transform>(socket).unwrap().scale,
            Vec3::ONE
        );
    }
    assert_eq!(
        app.world()
            .get::<NetworkPcRigAppearanceStatus0104>(rig_root),
        Some(&NetworkPcRigAppearanceStatus0104::Loading)
    );

    let missing_pc_root = app.world_mut().spawn_empty().id();
    let missing_rig_root = app
        .world_mut()
        .spawn((
            NetworkPcRig0104 {
                pc_root: missing_pc_root,
                pc_id: 23,
                look: remote_pc_test_look(&[]),
            },
            NativePlayerRigBones::new(Vec::new()),
            NativePlayerRigStatus::ReadyAnimated {
                actor_bones: 0,
                parts: 0,
                skin_palettes: 0,
                skinned_surfaces: 0,
            },
            NetworkPcRigAppearanceStatus0104::Loading,
        ))
        .id();
    app.world_mut().spawn((
        Transform::default(),
        ChildOf(missing_rig_root),
        PendingNetworkPcAppearanceAttachment0104 {
            rig_root: missing_rig_root,
            socket_full_path: "m/missing".to_owned(),
            socket_local_scale_override: None,
        },
    ));

    app.update();

    assert!(matches!(
        app.world()
            .get::<NetworkPcRigAppearanceStatus0104>(missing_rig_root),
        Some(NetworkPcRigAppearanceStatus0104::Blocked(reason))
            if reason.contains("exact socket")
    ));
    assert!(matches!(
        app.world().get::<NetworkPcVisualIssue0104>(missing_pc_root),
        Some(issue) if issue.pc_id == 23 && issue.detail.contains("exact socket")
    ));
}

#[test]
fn hnpc_production_tables_retain_guard_and_crowd_animation_sets() {
    let locator = AssetLocator::open(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"),
    )
    .unwrap();
    let table: Value = locator.read_table_set().unwrap();
    let npc = table["tables"]
        .as_array()
        .unwrap()
        .iter()
        .find(|table| table["name"] == CONSOLIDATED_TABLE)
        .unwrap()
        .pointer("/value/m_pNpcTable")
        .unwrap();
    let meshes = npc["m_pNpcMeshData"].as_array().unwrap();
    let mut sets = std::collections::BTreeSet::new();
    for row in npc["m_pNpcData"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["m_iHNpc"].as_i64().unwrap_or_default() != 0)
    {
        if let Some(clips) = hnpc_idle_clips(row, meshes, "production HNPC").unwrap() {
            sets.insert(clips);
        }
    }
    for expected in [
        ["rifleguard", "rifleguard", "rifleguard"],
        ["swordguard", "swordguard", "swordguard"],
        ["talk", "talkexclamation", "talkquestion"],
        ["dance2", "dance4", "stand1"],
        ["applaud", "cheer", "stand1"],
    ] {
        assert!(
            sets.contains(&expected.map(str::to_owned)),
            "missing {expected:?}"
        );
    }
    for gender in ["male", "female"] {
        let path = format!("characters/player/{gender}/base/{gender}_skeleton.glb");
        let bytes = locator.read(&path).unwrap();
        let gltf = gltf::Gltf::from_slice(&bytes).unwrap();
        let names = gltf
            .animations()
            .filter_map(|animation| animation.name())
            .collect::<std::collections::BTreeSet<_>>();
        for clip in [
            "stand1",
            "stand2",
            "stand3",
            "stand4",
            "rifleguard",
            "swordguard",
            "talk",
            "talkexclamation",
            "talkquestion",
            "foldedarms",
            "report",
            "observe",
            "usestanding",
            "scratch",
            "dance2",
            "dance4",
            "applaud",
            "cheer",
        ] {
            assert!(
                names.contains(clip),
                "{gender} skeleton lacks ambient clip {clip}"
            );
        }
    }
}

#[test]
fn hnpc_idle_preserves_same_clip_tail_and_yields_to_motion_and_death() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), bevy::animation::AnimationPlugin));
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0)));
    let mut clip = AnimationClip::default();
    clip.set_duration(5.3333335);
    let clip = app.world_mut().resource_mut::<Assets<AnimationClip>>().add(clip);
    let (graph, nodes) = AnimationGraph::from_clip(clip);
    let graph = app.world_mut().resource_mut::<Assets<AnimationGraph>>().add(graph);
    app.init_resource::<NetworkHnpcAnimationRevision0104>();
    app.add_systems(Update, sync_network_hnpc_animation_0104);
    let root = app.world_mut().spawn_empty().id();
    let node = nodes;
    let mut animation_player = AnimationPlayer::default();
    animation_player.start(node).repeat().set_seek_time(5.0);
    let player = app.world_mut().spawn((animation_player, AnimationGraphHandle(graph))).id();
    let mut look = remote_pc_test_look(&[]);
    look.weapon_animation_profile = Some(PlayerWeaponAnimationProfile::Rifle);
    let rig = app
        .world_mut()
        .spawn((
            NetworkHnpcRig0104 {
                npc_root: root,
                npc_type: 1,
                look,
                animation_ends: Arc::new(BTreeMap::from([("rifleguard".into(), 5.0833335)])),
                animation_sounds: Arc::from([]),
                idle_clips: Some([
                    "rifleguard".to_owned(),
                    "rifleguard".to_owned(),
                    "rifleguard".to_owned(),
                ]),
            },
            NativePlayerRigStand1Playback {
                animation_player: player,
                animation_graph: Handle::default(),
                animation_node: node,
            },
        ))
        .id();
    app.world_mut().entity_mut(root).insert((
        NetworkHnpcVisual0104 {
            npc_type: 1,
            appearance_index: 1,
            rig_root: rig,
            generation: 1,
            walk_animation_speed: 1.0,
            run_animation_speed: 1.0,
        },
        NetworkNpcAppearance0104(ffone_protocol::NpcAppearance0104 {
            npc_id: 1,
            npc_type: 1,
            hp: 100,
            condition_bit_flag: 0,
            position: [0; 3],
            angle: 0,
            barker_type: 0,
        }),
    ));
    app.update();
    let first = app
        .world()
        .get::<NativePlayerRigAnimationRequest>(rig)
        .unwrap()
        .clone();
    assert_eq!(first.clip, "rifleguard");
    assert!(first.repeat);
    app.update();
    assert_eq!(
        app.world().get::<NativePlayerRigAnimationRequest>(rig),
        Some(&first),
        "loading must not reroll or keep replacing the pending request"
    );
    app.world_mut()
        .entity_mut(rig)
        .insert(NativePlayerRigAnimationApplied {
            clip: first.clip.clone(),
            revision: first.revision,
            repeat: true,
        });
    app.update();
    assert_eq!(
        app.world().get::<NativePlayerRigAnimationRequest>(rig),
        Some(&first)
    );
    app.world_mut()
        .get_mut::<AnimationPlayer>(player)
        .unwrap()
        .animation_mut(node)
        .unwrap()
        .set_seek_time(5.1);
    app.update();
    assert_eq!(
        app.world().get::<NativePlayerRigAnimationRequest>(rig),
        Some(&first),
        "selecting the same clip at end must preserve its remaining quarter-second"
    );
    // Any second selection would now visibly change the request. The end
    // event must be consumed, not retriggered on every frame of its tail.
    app.world_mut()
        .get_mut::<NetworkHnpcRig0104>(rig)
        .unwrap()
        .idle_clips = Some(["cheer".into(), "cheer".into(), "cheer".into()]);
    app.world_mut()
        .get_mut::<AnimationPlayer>(player)
        .unwrap()
        .animation_mut(node)
        .unwrap()
        .set_seek_time(5.2);
    app.update();
    assert_eq!(
        app.world().get::<NativePlayerRigAnimationRequest>(rig),
        Some(&first)
    );
    // Seeking backwards does not complete a loop; crossing the early end
    // event again must still preserve the gesture's final poses.
    app.world_mut()
        .get_mut::<AnimationPlayer>(player)
        .unwrap()
        .animation_mut(node)
        .unwrap()
        .set_seek_time(0.1);
    app.update();
    assert_eq!(
        app.world().get::<NativePlayerRigAnimationRequest>(rig),
        Some(&first)
    );
    app.world_mut()
        .get_mut::<AnimationPlayer>(player)
        .unwrap()
        .animation_mut(node)
        .unwrap()
        .set_seek_time(5.1);
    app.update();
    assert_eq!(
        app.world()
            .get::<NativePlayerRigAnimationRequest>(rig)
            .unwrap()
            .clip,
        "rifleguard"
    );
    for _ in 0..20 { app.update(); }
    assert_eq!(
        app.world().get::<NativePlayerRigAnimationRequest>(rig).unwrap().clip,
        "cheer"
    );
    app.world_mut()
        .get_mut::<NetworkHnpcRig0104>(rig)
        .unwrap()
        .idle_clips = Some([
        "rifleguard".into(),
        "rifleguard".into(),
        "rifleguard".into(),
    ]);
    app.world_mut()
        .entity_mut(root)
        .insert(NetworkNpcMotion0104 {
            destination: Vec3::X,
            speed: 1.0,
            move_style: 0,
        });
    app.update();
    let moving = app
        .world()
        .get::<NativePlayerRigAnimationRequest>(rig)
        .unwrap();
    assert_eq!(moving.clip, "walk");
    assert!(moving.repeat);
    app.world_mut()
        .entity_mut(root)
        .remove::<NetworkNpcMotion0104>();
    app.update();
    assert_eq!(
        app.world()
            .get::<NativePlayerRigAnimationRequest>(rig)
            .unwrap()
            .clip,
        "rifleguard"
    );
    app.world_mut()
        .get_mut::<NetworkNpcAppearance0104>(root)
        .unwrap()
        .0
        .hp = 0;
    app.update();
    let dead = app
        .world()
        .get::<NativePlayerRigAnimationRequest>(rig)
        .unwrap();
    assert_eq!(dead.clip, "death");
    assert!(!dead.repeat);
}

#[test]
fn npc_stand_and_melee_draws_follow_the_primary_npc_animation_tables() {
    let all = |_: &str| true;
    for (roll, current, expected) in [
        (0, "", "stand1"),
        (39, "stand1", "stand1"),
        (40, "", "stand2"),
        (69, "stand2", "stand1"),
        (70, "stand2", "stand3"),
        (89, "stand3", "stand1"),
        (90, "ready", "stand4"),
        (99, "stand4", "stand1"),
    ] {
        assert_eq!(
            network_npc_stand_clip_0104(roll, current, all),
            expected,
            "roll {roll}"
        );
    }
    assert_eq!(
        network_npc_stand_clip_0104(95, "", |clip| clip != "stand4"),
        "stand1"
    );
    assert_eq!(network_npc_melee_clip_0104(0, all), "melee1");
    assert_eq!(network_npc_melee_clip_0104(1, all), "melee2");
    assert_eq!(
        network_npc_melee_clip_0104(1, |clip| clip != "melee2"),
        "melee1"
    );
    assert_eq!(
        network_npc_cross_fade_0104("stand3"),
        Duration::from_millis(300)
    );
    assert_eq!(
        network_npc_cross_fade_0104("ready"),
        Duration::from_millis(200)
    );
}
