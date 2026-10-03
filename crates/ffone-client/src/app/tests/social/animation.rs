use super::*;
use ffone_runtime_contracts::PlayerRigGender;

#[test]
fn standalone_emote_echo_queues_the_local_gender_clip_and_vehicle_blocks_it() {
    let mut app = App::new();
    let mut animations = bevy::ecs::system::SystemState::<
        Query<(&NetworkRemotePc0104, &mut RemoteAnimation)>,
    >::new(app.world_mut());
    let runtime = RuntimeStatus {
        core: RuntimePlayerStatus {
            player_id: Some(81),
            player_gender: Some(2),
            ..default()
        },
        ..default()
    };
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_AVATAR_EMOTES_CHAT,
        flags: 0,
        checksum: 0,
        payload: AvatarEmoteChat0104 {
            pc_id: 81,
            emote_code: 24,
        }
        .encode(),
    };
    let mut queue = TutorialPlayerPresentationCommandQueue::default();
    {
        let mut query = animations.get_mut(app.world_mut()).unwrap();
        assert_eq!(
            apply_avatar_emote_frame(
                &frame,
                &runtime,
                &mut query,
                &mut queue,
                &LocalVehiclePresentationRuntime::default(),
            ),
            Ok(true)
        );
    }
    assert_eq!(queue.len(), 1);
    let Some(TutorialPlayerPresentationCommand::Animation(request)) = queue.pop_front() else {
        panic!("local AvatarEmote echo must enqueue one animation");
    };
    assert_eq!(request.clip, TutorialPlayerClip::Beach3);
    assert_eq!(request.source_path_id, 34_552);
    assert_eq!(
        request.dispatch,
        TutorialPlayerAnimationDispatch::CrossFade(std::time::Duration::from_secs_f32(0.15))
    );

    let mut blocked_queue = TutorialPlayerPresentationCommandQueue::default();
    let mounted = LocalVehiclePresentationRuntime {
        family: LegacyVehiclePresentationFamily::Board,
        pending: None,
    };
    {
        let mut query = animations.get_mut(app.world_mut()).unwrap();
        assert_eq!(
            apply_avatar_emote_frame(&frame, &runtime, &mut query, &mut blocked_queue, &mounted,),
            Ok(true)
        );
    }
    assert!(blocked_queue.is_empty());
}

#[test]
fn retrobution_emote_echo_reaches_local_and_remote_players() {
    let mut app = App::new();
    let remote = app
        .world_mut()
        .spawn((
            NetworkRemotePc0104 { pc_id: 82 },
            RemoteAnimation::default(),
        ))
        .id();
    let mut animations = bevy::ecs::system::SystemState::<
        Query<(&NetworkRemotePc0104, &mut RemoteAnimation)>,
    >::new(app.world_mut());
    for (protocol_gender, gender) in [(1, PlayerRigGender::Male), (2, PlayerRigGender::Female)] {
        let runtime = RuntimeStatus {
            core: RuntimePlayerStatus {
                player_id: Some(81),
                player_gender: Some(protocol_gender),
                ..default()
            },
            ..default()
        };
        for code in 31..=44 {
            let clip = TutorialPlayerClip::from_avatar_emote_code(code).unwrap();
            let frame = |pc_id| DecodedFrame {
                packet_type: packet::P_FE2CL_REP_PC_AVATAR_EMOTES_CHAT,
                flags: 0,
                checksum: 0,
                payload: AvatarEmoteChat0104 {
                    pc_id,
                    emote_code: code,
                }
                .encode(),
            };
            let mut queue = TutorialPlayerPresentationCommandQueue::default();
            let mut query = animations.get_mut(app.world_mut()).unwrap();
            assert_eq!(
                apply_avatar_emote_frame(
                    &frame(81),
                    &runtime,
                    &mut query,
                    &mut queue,
                    &LocalVehiclePresentationRuntime::default(),
                ),
                Ok(true)
            );
            let Some(TutorialPlayerPresentationCommand::Animation(request)) = queue.pop_front()
            else {
                panic!("missing local Retrobution emote {code}");
            };
            assert_eq!(request.clip, clip);
            assert_eq!(request.source_path_id, clip.source_path_id(gender));
            assert_eq!(
                apply_avatar_emote_frame(
                    &frame(82),
                    &runtime,
                    &mut query,
                    &mut queue,
                    &LocalVehiclePresentationRuntime::default(),
                ),
                Ok(true)
            );
            drop(query);
            assert_eq!(
                app.world().get::<RemoteAnimation>(remote).unwrap().state,
                RemoteAnimationState::Emoting { clip }
            );
        }
    }
}
