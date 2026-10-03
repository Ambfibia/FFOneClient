use crate::remote::*;
use ffone_protocol::{PcJumpRequest0104, PcMoveRequest0104, PcStopRequest0104};

fn frame(packet_type: u32, payload: Vec<u8>) -> DecodedFrame {
    DecodedFrame {
        packet_type,
        flags: 0,
        checksum: 0,
        payload,
    }
}

fn assert_vec3_close(actual: Vec3, expected: Vec3) {
    assert!(
        actual.abs_diff_eq(expected, 0.000_01),
        "actual={actual:?}, expected={expected:?}"
    );
}

#[test]
fn converts_server_axes_and_scale_exactly() {
    assert_vec3_close(
        ProtocolPosition::new([100, 200, 300]).to_native(),
        Vec3::new(-1.0, 3.0, 2.0),
    );
    assert_vec3_close(
        ProtocolMoveVelocity::new([1.0, 2.0, 3.0]).to_native(),
        Vec3::new(-1.0, 3.0, 2.0),
    );
    assert_vec3_close(
        ProtocolScaledVelocity::new([100, 200, 300]).to_native(),
        Vec3::new(-1.0, 3.0, 2.0),
    );
}

#[test]
fn decodes_typed_move_and_applies_legacy_root_yaw() {
    let wire = PcMove0104 {
        movement: PcMoveRequest0104 {
            client_time: 11,
            position: [100, 200, 300],
            velocity: [1.0, 2.0, 3.0],
            angle: 45,
            key_value: 7,
            speed: 600,
        },
        pc_id: 42,
        server_time: 99,
    };
    let decoded = decode_remote_frame(&frame(P_FE2CL_PC_MOVE, wire.encode()))
        .expect("valid move")
        .expect("known packet");
    assert_eq!(decoded, DecodedRemotePacket::Move(wire));

    let (motion, animation) = RemoteMotion::from_packet(decoded);
    assert_vec3_close(motion.target_position, Vec3::new(-1.0, 3.0, 2.0));
    assert_vec3_close(motion.velocity, Vec3::new(-1.0, 3.0, 2.0));
    assert_eq!(motion.speed, 6.0);
    assert_eq!(motion.last_server_time, 99);
    assert_eq!(
        animation.state,
        RemoteAnimationState::Moving { direction_key: 7 }
    );
    assert!(
        motion
            .target_rotation
            .abs_diff_eq(ProtocolYawDegrees::new(45).native_root_rotation(), 0.000_01,)
    );
}

#[test]
fn malformed_and_unknown_frames_are_classified_without_panicking() {
    let malformed = frame(P_FE2CL_PC_MOVE, vec![0; PcMove0104::SIZE - 1]);
    assert!(decode_remote_frame(&malformed).is_err());

    let non_finite = PcMove0104 {
        movement: PcMoveRequest0104 {
            client_time: 0,
            position: [0; 3],
            velocity: [f32::NAN, 0.0, 0.0],
            angle: 0,
            key_value: 1,
            speed: 100,
        },
        pc_id: 1,
        server_time: 0,
    };
    assert!(decode_remote_frame(&frame(P_FE2CL_PC_MOVE, non_finite.encode())).is_err());
    assert_eq!(
        decode_remote_frame(&frame(0xdead_beef, vec![1, 2, 3])),
        Ok(None)
    );
}

#[test]
fn extrapolates_then_lerps_position_and_slerps_rotation() {
    let packet = DecodedRemotePacket::Move(PcMove0104 {
        movement: PcMoveRequest0104 {
            client_time: 0,
            position: [100, 0, 0],
            velocity: [1.0, 0.0, 0.0],
            angle: 0,
            key_value: 1,
            speed: 200,
        },
        pc_id: 1,
        server_time: 0,
    });
    let (mut motion, mut animation) = RemoteMotion::from_packet(packet);
    let mut transform = Transform::from_translation(motion.target_position);
    let initial_rotation_error = transform.rotation.angle_between(motion.target_rotation);

    step_remote_motion(&mut motion, &mut animation, &mut transform, 0.0625);

    assert_vec3_close(transform.translation, Vec3::new(-1.125, 0.0, 0.0));
    assert_vec3_close(motion.target_position, Vec3::new(-1.125, 0.0, 0.0));
    let remaining_rotation_error = transform.rotation.angle_between(motion.target_rotation);
    assert!((remaining_rotation_error - initial_rotation_error * 0.5).abs() < 0.000_1);
}

#[test]
fn correction_uses_delta_times_speed() {
    let mut motion = RemoteMotion {
        target_position: Vec3::new(10.0, 0.0, 0.0),
        speed: 2.0,
        ..default()
    };
    let mut animation = RemoteAnimation::default();
    let mut transform = Transform::default();

    step_remote_motion(&mut motion, &mut animation, &mut transform, 0.1);
    assert_vec3_close(transform.translation, Vec3::new(2.0, 0.0, 0.0));
}

#[test]
fn movement_times_out_only_after_point_seven_seconds() {
    let mut motion = RemoteMotion {
        velocity: Vec3::X,
        speed: 1.0,
        movement_key: 1,
        seconds_without_movement_packet: 0.69,
        ..default()
    };
    let mut animation = RemoteAnimation {
        state: RemoteAnimationState::Moving { direction_key: 1 },
    };
    let mut transform = Transform::default();

    step_remote_motion(&mut motion, &mut animation, &mut transform, 0.01);
    assert!(matches!(
        animation.state,
        RemoteAnimationState::Moving { .. }
    ));
    step_remote_motion(&mut motion, &mut animation, &mut transform, 0.001);
    assert_eq!(animation.state, RemoteAnimationState::Idle);
    assert_eq!(motion.movement_key, 0);
    assert_eq!(motion.velocity.x, 0.0);
    assert_eq!(motion.velocity.z, 0.0);
}

#[test]
fn movement_and_jump_packets_interrupt_server_confirmed_emotes() {
    let mut motion = RemoteMotion::default();
    let mut animation = RemoteAnimation {
        state: RemoteAnimationState::Emoting {
            clip: TutorialPlayerClip::Dance4,
        },
    };
    motion.apply_packet(
        DecodedRemotePacket::Move(PcMove0104 {
            movement: PcMoveRequest0104 {
                client_time: 1,
                position: [0; 3],
                velocity: [1.0, 0.0, 0.0],
                angle: 0,
                key_value: 1,
                speed: 100,
            },
            pc_id: 4,
            server_time: 2,
        }),
        &mut animation,
    );
    assert_eq!(
        animation.state,
        RemoteAnimationState::Moving { direction_key: 1 }
    );

    animation.state = RemoteAnimationState::Emoting {
        clip: TutorialPlayerClip::Dance4,
    };
    motion.apply_packet(
        DecodedRemotePacket::Jump(PcJump0104 {
            movement: PcJumpRequest0104 {
                client_time: 3,
                position: [0; 3],
                velocity: [0; 3],
                angle: 0,
                key_value: 1,
                speed: 100,
            },
            pc_id: 4,
            server_time: 4,
        }),
        &mut animation,
    );
    assert_eq!(
        animation.state,
        RemoteAnimationState::Jumping {
            direction_key: 1,
            double_jump: None,
        }
    );
}
#[test]
fn stop_and_jump_match_typed_packet_semantics() {
    let stop = DecodedRemotePacket::Stop(PcStop0104 {
        movement: PcStopRequest0104 {
            client_time: 0,
            position: [100, 200, 300],
        },
        pc_id: 4,
        server_time: 5,
    });
    let (stop_motion, stop_animation) = RemoteMotion::from_packet(stop);
    assert_eq!(stop_animation.state, RemoteAnimationState::Idle);
    assert_eq!(stop_motion.velocity, Vec3::ZERO);
    assert_eq!(stop_motion.speed, 0.0);

    let jump = DecodedRemotePacket::Jump(PcJump0104 {
        movement: PcJumpRequest0104 {
            client_time: 0,
            position: [0; 3],
            velocity: [100, 200, 300],
            angle: 90,
            key_value: 51,
            speed: 400,
        },
        pc_id: 4,
        server_time: 6,
    });
    let (jump_motion, jump_animation) = RemoteMotion::from_packet(jump);
    assert_vec3_close(jump_motion.velocity, Vec3::new(-1.0, 3.0, 2.0));
    assert_eq!(jump_motion.movement_key, 1);
    assert_eq!(
        jump_animation.state,
        RemoteAnimationState::Jumping {
            direction_key: 1,
            double_jump: Some(1),
        }
    );
}

#[test]
fn interpolation_only_changes_animation_on_lag_stop() {
    fn collect(changed: Query<Entity, Changed<RemoteAnimation>>, mut counts: ResMut<Counts>) {
        counts.0.push(changed.iter().count());
    }
    #[derive(Resource, Default)]
    struct Counts(Vec<usize>);
    let mut app = App::new();
    app.insert_resource(Time::<()>::default()).init_resource::<Counts>();
    app.add_systems(Update, (interpolate_remote_players, collect).chain());
    let entity = app.world_mut().spawn((
        RemoteMotion { movement_key: 1, speed: 1.0, ..default() },
        RemoteAnimation { state: RemoteAnimationState::Moving { direction_key: 1 } },
        Transform::default(),
    )).id();
    app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(100));
    app.update();
    app.update();
    assert_eq!(app.world().resource::<Counts>().0, vec![1, 0]);
    app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs(1));
    app.update();
    assert_eq!(app.world().get::<RemoteAnimation>(entity).unwrap().state, RemoteAnimationState::Idle);
    assert_eq!(app.world().resource::<Counts>().0, vec![1, 0, 1]);
}
