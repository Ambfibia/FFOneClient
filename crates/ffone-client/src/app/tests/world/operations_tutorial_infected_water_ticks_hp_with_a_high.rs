use super::*;

#[test]
fn normal_world_nano_skill_success_reconciles_identity_stamina_and_deactivation() {
    let skill_frame = |pc_id, nano_id, stamina, deactivated| {
        let prefix = ffone_protocol::NanoSkillUseSuccessPrefix0104 {
            pc_id,
            bullet_id: 0,
            pack_padding: [0],
            skill_id: 8,
            arg1: 0,
            arg2: 0,
            arg3: 0,
            nano_deactivated: deactivated,
            nano_id,
            nano_stamina: stamina,
            skill_type: 8,
            target_count: 1,
        };
        let mut payload = prefix.encode_prefix();
        let mut result = vec![0; 32];
        result[0..4].copy_from_slice(&4_i32.to_le_bytes());
        result[4..8].copy_from_slice(&9001_i32.to_le_bytes());
        result[12..16].copy_from_slice(&25_i32.to_le_bytes());
        result[16..20].copy_from_slice(&975_i32.to_le_bytes());
        result[20..22].copy_from_slice(&55_i16.to_le_bytes());
        result[28..32].copy_from_slice(&0x400_i32.to_le_bytes());
        payload.extend_from_slice(&result);
        DecodedFrame {
            packet_type: packet::P_FE2CL_NANO_SKILL_USE_SUCC,
            flags: 0,
            checksum: 0,
            payload,
        }
    };
    let mut runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_id: Some(77),
            nano_slots: [
                RuntimeNanoSlot {
                    nano_id: Some(1),
                    skill_id: 1,
                    stamina: 80,
                    active: true,
                },
                RuntimeNanoSlot {
                    nano_id: Some(17),
                    skill_id: 21,
                    stamina: 90,
                    active: false,
                },
                RuntimeNanoSlot {
                    nano_id: Some(1),
                    skill_id: 1,
                    stamina: 80,
                    active: false,
                },
            ],
            pending_nano_activation: Some(1),
            ..default()
        },
        ..default()
    };
    let mut inbox = WorldNanoAuthorityInbox0104::default();
    let mut skill_buffs = SkillBuffUiModel::default();

    let frame = skill_frame(77, 1, 60, 1);
    assert_eq!(
        apply_world_nano_response_frame(&frame, &mut runtime, &mut skill_buffs, &mut inbox,),
        Ok(true)
    );
    assert_eq!(
        runtime.nano_slots.map(|slot| slot.stamina),
        [60, 90, 60],
        "a fully validated packet commits in network order"
    );
    assert_eq!(runtime.nano_slots.map(|slot| slot.stamina), [60, 90, 60]);
    assert_eq!(runtime.nano_slots.map(|slot| slot.skill_id), [8, 21, 8]);
    assert_eq!(runtime.nano_slots.map(|slot| slot.active), [false; 3]);
    assert_eq!(
        runtime.pending_nano_activation,
        Some(1),
        "an older skill deactivation must not cancel a newer activation request"
    );

    let committed = runtime.nano_slots;
    assert!(
        apply_world_nano_response_frame(
            &skill_frame(88, 1, 40, 0),
            &mut runtime,
            &mut skill_buffs,
            &mut inbox,
        )
        .is_err()
    );
    assert_eq!(runtime.nano_slots, committed);
    assert!(inbox.local_movements.is_empty());
    assert!(
        apply_world_nano_response_frame(
            &skill_frame(77, 36, 40, 0),
            &mut runtime,
            &mut skill_buffs,
            &mut inbox,
        )
        .is_err()
    );
    assert_eq!(runtime.nano_slots, committed);
    assert!(inbox.local_movements.is_empty());

    let mut malformed = skill_frame(77, 1, 20, 0);
    malformed.payload[32..36].copy_from_slice(&2_i32.to_le_bytes());
    assert!(
        apply_world_nano_response_frame(&malformed, &mut runtime, &mut skill_buffs, &mut inbox,)
            .is_err()
    );
    assert_eq!(runtime.nano_slots, committed);
    assert!(inbox.local_movements.is_empty());

    runtime.nano_slots[0].active = true;
    runtime.weapon_battery = 100;
    runtime.nano_battery = 200;
    let prefix = ffone_protocol::NanoSkillUseSuccessPrefix0104 {
        pc_id: 77,
        bullet_id: 0,
        pack_padding: [0],
        skill_id: 8,
        arg1: 0,
        arg2: 0,
        arg3: 0,
        nano_deactivated: 0,
        nano_id: 1,
        nano_stamina: 50,
        skill_type: 21,
        target_count: 1,
    };
    let mut payload = prefix.encode_prefix();
    let mut drain = vec![0; 40];
    drain[0..4].copy_from_slice(&1_i32.to_le_bytes());
    drain[4..8].copy_from_slice(&77_i32.to_le_bytes());
    drain[12..16].copy_from_slice(&10_i32.to_le_bytes());
    drain[16..20].copy_from_slice(&90_i32.to_le_bytes());
    drain[20..24].copy_from_slice(&20_i32.to_le_bytes());
    drain[24..28].copy_from_slice(&180_i32.to_le_bytes());
    drain[28..30].copy_from_slice(&45_i16.to_le_bytes());
    drain[32..36].copy_from_slice(&1_i32.to_le_bytes());
    drain[36..40].copy_from_slice(&0x1000_i32.to_le_bytes());
    payload.extend_from_slice(&drain);
    let drain_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_NANO_SKILL_USE_SUCC,
        flags: 0,
        checksum: 0,
        payload,
    };
    apply_world_nano_response_frame(&drain_frame, &mut runtime, &mut skill_buffs, &mut inbox)
        .unwrap();
    assert_eq!(runtime.weapon_battery, 90);
    assert_eq!(runtime.nano_battery, 180);
    assert_eq!(runtime.nano_slots[0].stamina, 45);
    assert_eq!(runtime.nano_slots.map(|slot| slot.active), [false; 3]);
    assert_eq!(skill_buffs.local_condition_bit_flag, 0x1000);
}

#[test]
fn authoritative_world_teleport_rearms_readiness_and_deduplicates_goto_echo() {
    #[derive(Default, Resource)]
    struct Outcomes(Vec<bool>);

    fn apply_test_teleport(
        mut commands: Commands,
        mut loading: ResMut<GameplayLoadingState>,
        mut outcomes: ResMut<Outcomes>,
        mut players: Query<
            (
                Entity,
                &mut Transform,
                &mut LegacyPlayerController,
                &mut Visibility,
                &mut LegacyAvatarEnvironmentState,
            ),
            With<LocalPlayer>,
        >,
    ) {
        let (player, mut transform, mut controller, mut visibility, mut environment) =
            players.single_mut().unwrap();
        outcomes.0.push(apply_authoritative_world_teleport(
            &mut commands,
            &mut loading,
            player,
            &mut transform,
            &mut controller,
            &mut visibility,
            &mut environment,
            Vec3::new(12.0, 34.0, 56.0),
            Some(777),
        ));
    }

    let mut app = App::new();
    app.init_resource::<GameplayLoadingState>()
        .init_resource::<Outcomes>()
        .insert_resource(NativeWorldStreamingStatus {
            blocker: Some("stale source-tile failure".to_owned()),
            ..default()
        })
        .add_systems(Update, apply_test_teleport);
    let player = app
        .world_mut()
        .spawn((
            LocalPlayer,
            Transform::default(),
            LegacyPlayerController::from_baseline_table(),
            Visibility::Inherited,
            LegacyAvatarEnvironmentState {
                in_water: true,
                poisoned: true,
                ..default()
            },
        ))
        .id();

    app.update();
    assert_eq!(app.world().resource::<Outcomes>().0, vec![true]);
    assert!(
        app.world()
            .entity(player)
            .contains::<LegacyWorldColliderPending>()
    );
    assert_eq!(
        app.world()
            .entity(player)
            .get::<Transform>()
            .unwrap()
            .translation,
        Vec3::new(12.0, 34.0, 56.0)
    );
    let controller = app
        .world()
        .entity(player)
        .get::<LegacyPlayerController>()
        .unwrap();
    assert!(!controller.movement_enabled);
    assert_eq!(
        *app.world().entity(player).get::<Visibility>().unwrap(),
        Visibility::Hidden
    );
    let environment = app
        .world()
        .entity(player)
        .get::<LegacyAvatarEnvironmentState>()
        .unwrap();
    assert!(!environment.in_water);
    assert!(!environment.poisoned);
    assert_eq!(environment.last_observed_hp, Some(777));
    assert!(app.world().resource::<GameplayLoadingState>().visible);
    assert_eq!(
        app.world().resource::<GameplayLoadingState>().scope,
        Some(ResourceLoadingScope::World)
    );
    assert!(
        app.world()
            .resource::<NativeWorldStreamingStatus>()
            .blocker
            .is_none()
    );

    // OpenFusion's immediately-following GOTO_SUCC carries the same
    // coordinates; it reconciles components without restarting progress.
    app.update();
    assert_eq!(app.world().resource::<Outcomes>().0, vec![true, false]);
    assert!(
        app.world()
            .entity(player)
            .contains::<WarpLoadingAcknowledgment>()
    );
}

#[test]
fn initially_submerged_water_surface_tracks_contact_without_inventing_poison() {
    let mut water_mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::MAIN_WORLD,
    );
    water_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [-2.0, 0.0, -2.0],
            [2.0, 0.0, -2.0],
            [2.0, 0.0, 2.0],
            [-2.0, 0.0, -2.0],
            [2.0, 0.0, 2.0],
            [-2.0, 0.0, 2.0],
        ],
    );

    let mut app = App::new();
    app.init_resource::<Assets<Mesh>>()
        .insert_resource(Time::<()>::default())
        .insert_resource(NetworkBridge::start())
        .init_resource::<TutorialSession>()
        .insert_resource(RuntimeStatus {
            core: RuntimePlayerStatus {
                hp: Some(1_000),
                max_hp: 1_000,
                ..default()
            },
            ..default()
        })
        .init_resource::<GameplayAudioRuntime>()
        .add_systems(Update, update_legacy_avatar_environment);
    let mesh = app
        .world_mut()
        .resource_mut::<Assets<Mesh>>()
        .add(water_mesh);
    let water = app
        .world_mut()
        .spawn((
            Mesh3d(mesh),
            Transform::IDENTITY,
            GlobalTransform::IDENTITY,
            LegacyWaterSurface { infected: true },
        ))
        .id();
    let player = app
        .world_mut()
        .spawn((
            LocalPlayer,
            Transform::from_xyz(0.0, -1.0, 0.0),
            LegacyPlayerController::from_baseline_table(),
            LegacyAvatarEnvironmentState::default(),
        ))
        .id();

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(0.016));
    app.update();
    let environment = app
        .world()
        .entity(player)
        .get::<LegacyAvatarEnvironmentState>()
        .unwrap();
    assert!(environment.in_water);
    assert!(environment.infected_water);
    assert!(!environment.poisoned);

    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .unwrap()
        .translation
        .y = 3.0;
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(
            LEGACY_ENVIRONMENT_TRANSITION_SECONDS + 0.01,
        ));
    app.update();
    let environment = app
        .world()
        .entity(player)
        .get::<LegacyAvatarEnvironmentState>()
        .unwrap();
    assert!(!environment.in_water);
    assert!(!environment.infected_water);
    assert!(!environment.poisoned);

    assert!(app.world().get_entity(water).is_ok());
}

#[test]
fn tutorial_infected_water_ticks_hp_with_a_higher_resident_neighbor() {
    use ffone_client::world::NativeWorldScene;

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = load_native_world_scenes(&root).unwrap();
    let scene: &NativeWorldScene = catalog
        .select_in_scope(NativeWorldScope::Tutorial, Vec3::new(-256.0, 0.0, 768.0))
        .unwrap();
    let terrain_instance = scene.native_terrain.as_ref().unwrap();
    let terrain = catalog.load_terrain(terrain_instance).unwrap();
    let collider = NativeHeightmapCollider::from_terrain(&terrain, Handle::default());
    let mut terrain_global = GlobalTransform::from(scene.root.try_to_bevy("root").unwrap());
    if let Some(chain) = &terrain_instance.root_chain {
        for node in chain.nodes.iter().rev() {
            terrain_global = terrain_global
                * node
                    .native_local_transform
                    .try_to_bevy("terrain parent")
                    .unwrap();
        }
    }
    terrain_global = terrain_global * terrain_instance.transform.try_to_bevy("terrain").unwrap();
    let visual = scene
        .visuals
        .iter()
        .find(|visual| {
            scene
                .models
                .iter()
                .find(|model| model.id == visual.model)
                .is_some_and(|model| model.root_name == "ffPoison")
        })
        .unwrap();
    let water_global = GlobalTransform::from(scene.root.try_to_bevy("root").unwrap())
        * visual.transform.try_to_bevy("water").unwrap();
    let glb = gltf::Gltf::from_slice(
        &std::fs::read(
            root.join(
                &scene
                    .models
                    .iter()
                    .find(|model| model.id == visual.model)
                    .unwrap()
                    .path,
            ),
        )
        .unwrap(),
    )
    .unwrap();
    let blob = glb.blob.as_deref().unwrap();
    let primitive = glb.meshes().next().unwrap().primitives().next().unwrap();
    let reader = primitive.reader(|_| Some(blob));
    let positions = reader.read_positions().unwrap().collect::<Vec<_>>();
    let indices = reader
        .read_indices()
        .unwrap()
        .into_u32()
        .collect::<Vec<_>>();
    let point = indices
        .chunks_exact(3)
        .find_map(|triangle| {
            let local = triangle
                .iter()
                .map(|&index| Vec3::from_array(positions[index as usize]))
                .sum::<Vec3>()
                / 3.0;
            let world = water_global.to_matrix().transform_point3(local);
            let NativeTerrainLegacyInterpolatedHeightSample::Height(ground) =
                collider.legacy_interpolated_height(&terrain_global, world.x, world.z)
            else {
                return None;
            };
            (native_dong_key(world.x, world.z) == Some(1)
                && world.y > ground + 1.0
                && collider.gameplay_attribute(&terrain_global, world.x, world.z)
                    == NativeTerrainGameplayAttributeSample::Value(0x0c))
            .then_some(world + Vec3::Y * 0.01)
        })
        .expect("real tutorial water above poisoned terrain in dong 00_01");
    let mut water_mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::MAIN_WORLD,
    );
    water_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    water_mesh.insert_indices(bevy::mesh::Indices::U32(indices));

    // The former maximum-height scan selected this unrelated tile because
    // SampleHeight clamps the query, although its attribute query is outside.
    let neighbor_global = GlobalTransform::from_translation(
        terrain_global.translation() + Vec3::new(-512.0, 1_000.0, 0.0),
    );
    assert_eq!(
        collider.gameplay_attribute(&neighbor_global, point.x, point.z),
        NativeTerrainGameplayAttributeSample::OutsideTerrain
    );
    assert!(
        matches!(collider.legacy_interpolated_height(&neighbor_global, point.x, point.z),
        NativeTerrainLegacyInterpolatedHeightSample::Height(height) if height > point.y)
    );

    for (neighbor_first, protocol_gender, locale) in [
        (false, 1, "en"),
        (true, 1, "ru"),
        (false, 2, "ru"),
        (true, 2, "en"),
    ] {
        let mut app = App::new();
        let mut character = entry_test_character(77, 1, 1);
        character.style.gender = protocol_gender;
        app.add_plugins((
            bevy::app::TaskPoolPlugin::default(),
            TransformPlugin,
            bevy::asset::AssetPlugin {
                file_path: root.to_string_lossy().into_owned(),
                ..default()
            },
            GameplayAudioPlugin,
        ))
        .init_asset::<AudioSource>()
        .insert_resource(NativeAudioCatalog::open(&root, false).unwrap())
        .insert_resource(VoiceLanguage {
            requested: locale.into(),
            effective: locale.into(),
        })
        .init_resource::<Assets<Mesh>>()
        .insert_resource(Time::<()>::default())
        .insert_resource(NetworkBridge::start())
        .insert_resource(TutorialSession {
            character: Some(character),
            ..default()
        })
        .insert_resource(RuntimeStatus {
            core: RuntimePlayerStatus {
                hp: Some(1_000),
                max_hp: 1_000,
                ..default()
            },
            ..default()
        })
        .add_systems(
            Update,
            update_legacy_avatar_environment.before(GameplayAudioSet::Drive),
        );
        let globals = if neighbor_first {
            [neighbor_global, terrain_global]
        } else {
            [terrain_global, neighbor_global]
        };
        for global in globals {
            app.world_mut()
                .spawn((global.compute_transform(), global, collider.clone()));
        }
        let mesh = app
            .world_mut()
            .resource_mut::<Assets<Mesh>>()
            .add(water_mesh.clone());
        app.world_mut().spawn((
            Mesh3d(mesh),
            water_global.compute_transform(),
            water_global,
            LegacyWaterSurface { infected: true },
        ));
        let player = app
            .world_mut()
            .spawn((
                LocalPlayer,
                LocalNetworkIdentity {
                    pc_uid: 77,
                    player_id: 77,
                },
                Transform::from_translation(point),
                LegacyPlayerController::from_baseline_table(),
                LegacyAvatarEnvironmentState::default(),
            ))
            .id();
        for frame in 1..=16 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(250));
            app.update();
            let environment = app
                .world()
                .get::<LegacyAvatarEnvironmentState>(player)
                .unwrap();
            assert!(
                environment.in_water && environment.infected_water && environment.poisoned,
                "frame {frame}: {environment:?}"
            );
            assert_eq!(environment.terrain_attribute, Some(0x0c));
            assert_eq!(
                app.world().resource::<RuntimeStatus>().hp,
                Some(1_000 - (frame / 8) * 150)
            );
            assert_local_infection_audio(&mut app, frame % 8 == 0, protocol_gender, locale);
        }
        app.world_mut()
            .get_mut::<Transform>(player)
            .unwrap()
            .translation
            .y += 10.0;
        for _ in 0..8 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(250));
            app.update();
            assert_local_infection_audio(&mut app, false, protocol_gender, locale);
        }
        assert!(
            !app.world()
                .get::<LegacyAvatarEnvironmentState>(player)
                .unwrap()
                .poisoned
        );
        assert_eq!(
            app.world().resource::<RuntimeStatus>().hp,
            Some(700),
            "no infection ticks after exit"
        );
        // The lethal tick sounds once; a corpse must not keep producing hurt sounds.
        app.world_mut()
            .get_mut::<Transform>(player)
            .unwrap()
            .translation = point;
        app.world_mut().resource_mut::<RuntimeStatus>().hp = Some(100);
        for frame in 1..=16 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(250));
            app.update();
            assert_local_infection_audio(&mut app, frame == 8, protocol_gender, locale);
        }
        assert_eq!(app.world().resource::<RuntimeStatus>().hp, Some(0));

        // A network-owned player waits for the shard packet, never local duplicate audio.
        app.world_mut()
            .resource_mut::<TutorialSession>()
            .completion_requested = true;
        app.world_mut().resource_mut::<RuntimeStatus>().hp = Some(1_000);
        for _ in 0..8 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(250));
            app.update();
            assert_local_infection_audio(&mut app, false, protocol_gender, locale);
        }
        assert_eq!(app.world().resource::<RuntimeStatus>().hp, Some(1_000));
    }
}

#[test]
fn local_tutorial_combat_refreshes_the_five_second_regeneration_timeout() {
    let mut app = App::new();
    app.init_resource::<GameplayAudioRuntime>()
        .init_resource::<Assets<Mesh>>()
        .insert_resource(Time::<()>::default())
        .insert_resource(NetworkBridge::start())
        .insert_resource(TutorialSession {
            character: Some(entry_test_character(77, 1, 1)),
            ..default()
        })
        .insert_resource(RuntimeStatus {
            core: RuntimePlayerStatus {
                hp: Some(500),
                max_hp: 1_000,
                ..default()
            },
            ..default()
        })
        .add_systems(Update, update_legacy_avatar_environment);
    let mut environment = LegacyAvatarEnvironmentState {
        local_regen_elapsed_seconds: LEGACY_LOCAL_REGEN_TICK_SECONDS,
        ..default()
    };
    environment.observe_local_combat();
    let player = app
        .world_mut()
        .spawn((
            LocalPlayer,
            LocalNetworkIdentity {
                pc_uid: 77,
                player_id: 77,
            },
            Transform::default(),
            LegacyPlayerController::from_baseline_table(),
            environment,
        ))
        .id();

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(
            ffone_client::legacy_environment::LEGACY_COMBAT_TIMEOUT_SECONDS - 0.01,
        ));
    app.update();
    assert_eq!(app.world().resource::<RuntimeStatus>().hp, Some(500));
    assert_eq!(
        app.world()
            .entity(player)
            .get::<LegacyAvatarEnvironmentState>()
            .unwrap()
            .local_regen_elapsed_seconds,
        LEGACY_LOCAL_REGEN_TICK_SECONDS
    );

    app.world_mut()
        .entity_mut(player)
        .get_mut::<LegacyAvatarEnvironmentState>()
        .unwrap()
        .observe_local_combat();
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(
            ffone_client::legacy_environment::LEGACY_COMBAT_TIMEOUT_SECONDS - 0.01,
        ));
    app.update();
    assert_eq!(app.world().resource::<RuntimeStatus>().hp, Some(500));

    // The frame crossing the strict `> 5` source boundary is still combat;
    // regeneration becomes eligible on the following update.
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(0.02));
    app.update();
    assert_eq!(app.world().resource::<RuntimeStatus>().hp, Some(500));
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(0.01));
    app.update();
    assert_eq!(app.world().resource::<RuntimeStatus>().hp, Some(700));
}

#[test]
fn empty_nano_slots_use_clean_row_zero_for_target_stims_but_unknown_ids_fail_closed() {
    let content = runtime_test_mission_content();
    assert_eq!(
        skill_buff_nano_styles([RuntimeNanoSlot::default(); 3], &content),
        [Some(0), Some(0), Some(0)]
    );
    let unresolved = RuntimeNanoSlot {
        nano_id: Some(i16::MAX),
        ..default()
    };
    assert_eq!(
        skill_buff_nano_styles(
            [
                unresolved,
                RuntimeNanoSlot::default(),
                RuntimeNanoSlot::default(),
            ],
            &content,
        ),
        [None, Some(0), Some(0)]
    );
}

#[test]
fn mission_objective_binds_combat_input_after_restoring_the_npc_filter() {
    let mut native = TutorialNativeMechanics::default();
    native.sync_stable_stage(TutorialStage::Mission(MissionStage::Approach));
    let approach_locks = *native.live_input_locks();

    apply_tutorial_input_stage_transition(
        &mut native,
        TutorialStage::Mission(MissionStage::TargetAndTalk),
        &[TutorialIntent::PushInputFilter],
    );
    assert_eq!(native.saved_input_locks(), &approach_locks);

    apply_tutorial_input_stage_transition(
        &mut native,
        TutorialStage::Mission(MissionStage::ObjectiveCombat),
        &[TutorialIntent::PopInputFilter],
    );

    assert_eq!(
        native.bound_stage(),
        Some(TutorialStage::Mission(MissionStage::ObjectiveCombat))
    );
    for input in [
        TutorialInputLock::Front,
        TutorialInputLock::Attack,
        TutorialInputLock::Menu,
        TutorialInputLock::ModeChange,
    ] {
        assert!(!native.is_locked(input), "{input:?} must be available");
    }
}
