use super::*;
use crate::app::nano_free_tuning::NanoFreeTuningPreview;
use crate::app::nano_free_tuning::NanoFreeTuningPreviewSoundCursor;
use crate::app::nano_free_tuning::nano_free_tuning_creation_projectile_command;
use crate::app::nano_free_tuning_production::nano_acquisition_auto_equip_request;
use ffone_client::character_scene::NativeSceneRole;
use ffone_client::character_scene::native_scene_container_transform;
use ffone_client::legacy_npc_nano_animation::LegacyNanoStandRandomStream;
use ffone_client::nano_free_tuning_runtime::project_nano_free_tuning_content;
use ffone_client::nano_free_tuning_ui::NanoFreeTuningOpenContext;
use ffone_client::nano_free_tuning_ui::NanoFreeTuningWorldSnapshot;
use ffone_client::network_world_runtime::NetworkNpcAnimationSoundEvent0104;

#[test]
fn acquisition_keeps_the_player_visible_while_hiding_only_the_hud() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let content = TutorialMissionContent::open(&AssetLocator::open(root).unwrap()).unwrap();
    let mut model = NanoFreeTuningModel::default();
    model
        .open(NanoFreeTuningOpenContext {
            player_id: 7,
            killed_fusion: true,
            content: project_nano_free_tuning_content(&content, 60).unwrap(),
            world: NanoFreeTuningWorldSnapshot::default(),
        })
        .unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<TutorialChoreographyPresentation>()
        .init_resource::<MissionUiModel>()
        .init_resource::<NanoFreeTuningProductionRuntime>()
        .insert_resource(model)
        .add_systems(Update, sync_tutorial_choreography_visibility);
    let player = app
        .world_mut()
        .spawn((LocalCharacterScene, Visibility::Inherited))
        .id();
    let hud = app
        .world_mut()
        .spawn((GameplayHud, Visibility::Inherited))
        .id();
    app.update();
    assert_eq!(
        app.world().get::<Visibility>(player),
        Some(&Visibility::Inherited)
    );
    assert_eq!(
        app.world().get::<Visibility>(hud),
        Some(&Visibility::Hidden)
    );
    app.world_mut()
        .resource_mut::<NanoFreeTuningModel>()
        .cancel()
        .unwrap();
    app.update();
    assert_eq!(
        app.world().get::<Visibility>(player),
        Some(&Visibility::Inherited)
    );
    assert_eq!(
        app.world().get::<Visibility>(hud),
        Some(&Visibility::Inherited)
    );
}

#[test]
fn consecutive_acquisitions_bind_full_skill_duration_and_never_reveal_a_hidden_result() {
    use ffone_client::nano_free_tuning_ui::*;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(root.clone()).unwrap();
    let catalog = GameplayNanoPortraitCatalog::open(&locator).unwrap();
    let content = TutorialMissionContent::open(&locator).unwrap();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Gltf>()
        .init_asset::<AnimationClip>()
        .init_asset::<AnimationGraph>()
        .init_resource::<NanoFreeTuningProductionRuntime>()
        .init_resource::<NanoFreeTuningModel>()
        .init_resource::<RuntimeStatus>()
        .init_resource::<LegacyNanoStandRandomStream>()
        .add_systems(Update, sync_nano_free_tuning_preview_animation);

    let nano_ids = [42, 24, 51, 55]
        .into_iter()
        .chain(content.gameplay_nanos().map(|nano| nano.nano_id))
        .chain([42]);
    for nano_id in nano_ids {
        for power_index in 0..3 {
            let path = catalog.model_path(nano_id).unwrap();
            let bytes = std::fs::read(root.join(path)).unwrap();
            let source = gltf::Gltf::from_slice(&bytes).unwrap();
            let mut gltf = Gltf {
                scenes: Vec::new(),
                named_scenes: default(),
                meshes: Vec::new(),
                named_meshes: default(),
                materials: Vec::new(),
                named_materials: default(),
                nodes: Vec::new(),
                named_nodes: default(),
                skins: Vec::new(),
                named_skins: default(),
                default_scene: None,
                animations: Vec::new(),
                named_animations: default(),
                source: None,
            };
            for animation in source.animations() {
                let name = animation.name().unwrap();
                let duration = animation
                    .samplers()
                    .filter_map(|sampler| {
                        sampler
                            .input()
                            .max()
                            .and_then(|max| max[0].as_f64())
                            .map(|value| value as f32)
                    })
                    .fold(0.0_f32, f32::max);
                let mut clip = AnimationClip::default();
                clip.set_duration(duration);
                let handle = app
                    .world_mut()
                    .resource_mut::<Assets<AnimationClip>>()
                    .add(clip);
                gltf.named_animations.insert(name.into(), handle);
            }
            let handle = app.world_mut().resource_mut::<Assets<Gltf>>().add(gltf);
            let preview = app
                .world_mut()
                .spawn((
                    NanoFreeTuningPreview {
                        nano_id,
                        gltf: handle,
                        sound_events: Arc::from([]),
                    },
                    Visibility::Hidden,
                ))
                .id();
            let player = app
                .world_mut()
                .spawn((AnimationPlayer::default(), ChildOf(preview)))
                .id();
            let projected = project_nano_free_tuning_content(&content, nano_id).unwrap();
            let skill_id = projected.powers[power_index].skill_id;
            {
                let mut model = app.world_mut().resource_mut::<NanoFreeTuningModel>();
                model
                    .open(NanoFreeTuningOpenContext {
                        player_id: 7,
                        killed_fusion: false,
                        content: projected,
                        world: NanoFreeTuningWorldSnapshot::default(),
                    })
                    .unwrap();
                model
                    .advance(NANO_FREE_TUNING_CAMERA_SETTLE_SECONDS + 0.01)
                    .unwrap();
                assert!(
                    model.controls_enabled(),
                    "new award must wait for a new selection"
                );
            }
            {
                let mut production = app
                    .world_mut()
                    .resource_mut::<NanoFreeTuningProductionRuntime>();
                production.reset_transient();
                production.preview_entity = Some(preview);
                production.queue_animation("stand2");
            }
            app.update();
            assert_eq!(
                app.world().get::<Visibility>(preview),
                Some(&Visibility::Inherited)
            );
            if nano_id == 24 {
                assert_eq!(
                    app.world()
                        .get::<NanoFreeTuningPreviewAnimation>(player)
                        .unwrap()
                        .clip_name,
                    "stand1"
                );
            }
            {
                let mut model = app.world_mut().resource_mut::<NanoFreeTuningModel>();
                let token = model.click_power(power_index).unwrap();
                model
                    .apply_reply(NanoFreeTuningReplyEnvelope {
                        request_token: token,
                        packet_id: NANO_TUNE_SUCCESS_PACKET_ID,
                        payload_size: NANO_TUNE_SUCCESS_SIZE,
                        body: NanoFreeTuningReplyBody::Success(NanoTuneSuccess {
                            nano_id,
                            skill_id,
                            fusion_matter: 0,
                            item_slots: [0; NANO_TUNE_ITEM_SLOT_COUNT],
                            items: [NanoTuneItemBase::default(); NANO_TUNE_ITEM_SLOT_COUNT],
                        }),
                    })
                    .unwrap();
                model.advance(4.0).unwrap();
            }
            let requested = ["skill1", "skill2", "skill3"][power_index];
            app.world_mut()
                .resource_mut::<NanoFreeTuningProductionRuntime>()
                .queue_animation(requested);
            app.update();
            {
                let mut model = app.world_mut().resource_mut::<NanoFreeTuningModel>();
                model.advance(0.05).unwrap();
                assert_eq!(
                    model.phase(),
                    NanoFreeTuningPhase::ResultSkill,
                    "Nano {nano_id} skill {power_index} must not inherit the loading delay or disappear on a missing skill"
                );
                model.advance(10.0).unwrap();
                assert_eq!(model.phase(), NanoFreeTuningPhase::ResultHide);
            }
            app.world_mut()
                .entity_mut(preview)
                .insert(Visibility::Hidden);
            app.world_mut()
                .resource_mut::<NanoFreeTuningProductionRuntime>()
                .queue_animation("stand1");
            app.update();
            assert_eq!(
                app.world().get::<Visibility>(preview),
                Some(&Visibility::Hidden)
            );
            assert!(
                app.world()
                    .resource::<NanoFreeTuningProductionRuntime>()
                    .pending_animation
                    .is_none()
            );
            app.world_mut()
                .resource_mut::<NanoFreeTuningModel>()
                .cancel()
                .unwrap();
            app.world_mut().despawn(preview);
        }
    }
}

#[test]
fn acquisition_auto_equip_uses_first_free_slot_without_predicting_or_replacing() {
    let occupied = RuntimeNanoSlot {
        nano_id: Some(1),
        skill_id: 1,
        stamina: 150,
        active: true,
    };
    let mut slots = [
        occupied,
        RuntimeNanoSlot::default(),
        RuntimeNanoSlot::default(),
    ];
    let request = nano_acquisition_auto_equip_request(50, &slots).unwrap();
    assert_eq!(request.nano_id, 50);
    assert_eq!(request.nano_slot, 1);
    assert_eq!(slots[1].nano_id, None);
    slots[1].nano_id = Some(50);
    assert!(nano_acquisition_auto_equip_request(50, &slots).is_none());
    assert!(nano_acquisition_auto_equip_request(50, &[occupied; 3]).is_none());
}

#[test]
fn creation_animation_keeps_hidden_stand_ahead_of_call() {
    let mut runtime = NanoFreeTuningProductionRuntime::default();

    runtime.queue_animation("stand2");
    runtime.queue_animation("call");

    assert_eq!(runtime.pending_animation, Some("stand2"));
    assert_eq!(runtime.queued_animations, VecDeque::from(["call"]));
    runtime.complete_pending_animation();
    assert_eq!(runtime.pending_animation, Some("call"));
}

#[test]
fn creation_preview_basis_faces_back_toward_the_player() {
    let player_rotation = Quat::from_rotation_y(0.73);
    let wrapper_rotation = player_rotation * Quat::from_rotation_y(std::f32::consts::PI);
    let visual_rotation =
        native_scene_container_transform(NativeSceneRole::CharacterGameplay).rotation;

    let rendered_forward = wrapper_rotation * visual_rotation * Vec3::Z;
    let from_preview_toward_player = player_rotation * Vec3::Z;
    assert!(rendered_forward.abs_diff_eq(from_preview_toward_player, 1.0e-6));
    assert!(rendered_forward.dot(player_rotation * Vec3::NEG_Z) < -0.999_999);
}

#[test]
fn creation_projectiles_take_the_operational_sampled_reverse_path() {
    let mut random = TutorialProjectileRandomStream::with_seed(0x1234_5678);
    let command = nano_free_tuning_creation_projectile_command(
        [76, 77],
        Vec3::new(1.0, 2.0, 3.0),
        Vec3::new(4.0, 5.0, 6.0),
        &mut random,
    );

    assert!(matches!(
        command,
        TutorialEffectRuntimeCommand::ProjectilePairSampled {
            types: [76, 77],
            source,
            target,
            oni: true,
            priority: 0,
            reverse: true,
            sampled_initial_velocity,
            source_line: 161,
        } if source == Vec3::new(1.0, 2.0, 3.0)
            && target == Vec3::new(4.0, 5.0, 6.0)
            && sampled_initial_velocity.iter().all(|velocity| velocity.is_finite())
    ));
}

#[test]
fn acquisition_call_consumes_its_embedded_voice_and_sfx_events_once() {
    let mut app = App::new();
    app.init_resource::<NanoFreeTuningProductionRuntime>()
        .init_resource::<GameplayAudioRuntime>()
        .add_systems(Update, emit_nano_free_tuning_preview_animation_sounds);

    let root = app
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            NanoFreeTuningPreview {
                nano_id: 2,
                gltf: Handle::default(),
                sound_events: Arc::from([
                    NetworkNpcAnimationSoundEvent0104 {
                        clip: "call".to_owned(),
                        time: 0.25,
                        payload: "Bloo_NanSummon0(RAND:1-3).wav".to_owned(),
                    },
                    NetworkNpcAnimationSoundEvent0104 {
                        clip: "call".to_owned(),
                        time: 0.25,
                        payload: "Nano Ability 06.wav".to_owned(),
                    },
                ]),
            },
        ))
        .id();
    app.world_mut()
        .resource_mut::<NanoFreeTuningProductionRuntime>()
        .preview_entity = Some(root);

    let (_, nodes) = AnimationGraph::from_clips([Handle::<AnimationClip>::default()]);
    let node = nodes[0];
    let mut player = AnimationPlayer::default();
    player.play(node).set_seek_time(0.3);
    let player_entity = app
        .world_mut()
        .spawn((
            player,
            NanoFreeTuningPreviewAnimation {
                clip_name: "call",
                node,
            },
            ChildOf(root),
        ))
        .id();

    app.update();
    let first_cursor = app
        .world()
        .get::<NanoFreeTuningPreviewSoundCursor>(player_entity)
        .expect("call animation must consume both crossed embedded sound events");
    assert_eq!(first_cursor.clip_name, "call");
    assert_eq!(first_cursor.node, node);
    assert_eq!(first_cursor.seek_time, 0.3);
    assert_eq!(first_cursor.completions, 0);
    let first_timing = (first_cursor.seek_time, first_cursor.completions);

    app.update();
    let second_cursor = app
        .world()
        .get::<NanoFreeTuningPreviewSoundCursor>(player_entity)
        .unwrap();
    assert_eq!(
        (second_cursor.seek_time, second_cursor.completions),
        first_timing
    );
}
