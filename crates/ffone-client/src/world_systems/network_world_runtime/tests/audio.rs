use super::*;

#[test]
fn hnpc_emote_sound_tracks_applied_revision_distance_and_same_clip_restarts() {
    let mut app = App::new();
    app.init_resource::<GameplayAudioRuntime>();
    app.add_systems(Update, emit_network_hnpc_animation_sounds_0104);
    let node = AnimationNodeIndex::new(0);
    let root = app.world_mut().spawn_empty().id();
    let mut player = AnimationPlayer::default();
    player.start(node).set_seek_time(0.1);
    let player = app.world_mut().spawn(player).id();
    let rig = app
        .world_mut()
        .spawn((
            NetworkHnpcRig0104 {
                npc_root: root,
                npc_type: 2885,
                look: remote_pc_test_look(&[]),
                idle_clips: None,
                animation_ends: Arc::default(),
                animation_sounds: vec![
                    NetworkNpcAnimationSoundEvent0104 {
                        clip: "cheer".into(),
                        time: 0.25,
                        payload: "M_Avatar_Happy01.wav".into(),
                    },
                    NetworkNpcAnimationSoundEvent0104 {
                        clip: "cheer".into(),
                        time: 0.25,
                        payload: "M_Avatar_SFX_Dance2_01.wav".into(),
                    },
                ]
                .into(),
            },
            NativePlayerRigStand1Playback {
                animation_player: player,
                animation_graph: Handle::default(),
                animation_node: node,
            },
            NativePlayerRigAnimationApplied {
                clip: "cheer".into(),
                revision: 1,
                repeat: false,
            },
        ))
        .id();
    app.world_mut().entity_mut(root).insert((
        GlobalTransform::default(),
        NetworkHnpcAnimationState0104 {
            rig_root: rig,
            requested_clip: "cheer".into(),
            clip: "cheer".into(),
            combat_revision: None,
            rig_revision: 1,
            repeat: false,
            idle: true,
        },
    ));
    let listener = app
        .world_mut()
        .spawn((
            LegacyPlayerController::from_baseline_table(),
            GlobalTransform::default(),
        ))
        .id();
    app.update();
    assert!(
        app.world()
            .resource::<GameplayAudioRuntime>()
            .queued_animation_sound_names()
            .is_empty()
    );
    app.world_mut()
        .get_mut::<AnimationPlayer>(player)
        .unwrap()
        .animation_mut(node)
        .unwrap()
        .set_seek_time(0.3);
    app.update();
    assert_eq!(
        app.world()
            .resource::<GameplayAudioRuntime>()
            .queued_animation_sound_names()
            .len(),
        2
    );
    app.update();
    assert_eq!(
        app.world()
            .resource::<GameplayAudioRuntime>()
            .queued_animation_sound_names()
            .len(),
        2,
        "same pose frame must not duplicate sound"
    );
    app.world_mut()
        .get_mut::<NativePlayerRigAnimationApplied>(rig)
        .unwrap()
        .revision = 2;
    app.update();
    assert_eq!(
        app.world()
            .resource::<GameplayAudioRuntime>()
            .queued_animation_sound_names()
            .len(),
        4,
        "repeated identical emote owns new sounds"
    );
    app.world_mut()
        .entity_mut(listener)
        .insert(GlobalTransform::from_translation(Vec3::X * 11.0));
    app.world_mut()
        .get_mut::<NativePlayerRigAnimationApplied>(rig)
        .unwrap()
        .revision = 3;
    app.update();
    let names = app
        .world()
        .resource::<GameplayAudioRuntime>()
        .queued_animation_sound_names();
    assert_eq!(
        names.len(),
        5,
        "distant idle suppresses vocal cue but preserves Dance SFX"
    );
    assert_eq!(names[4].0, "M_Avatar_SFX_Dance2_01");
    app.world_mut()
        .entity_mut(listener)
        .insert(GlobalTransform::default());
    app.update();
    assert_eq!(
        app.world()
            .resource::<GameplayAudioRuntime>()
            .queued_animation_sound_names()
            .len(),
        5,
        "approaching must not replay muted past events"
    );
}
