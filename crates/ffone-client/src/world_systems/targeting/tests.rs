use ffone_protocol::{
    FixedUtf16, ItemBase0104, Nano0104, NpcAppearance0104, PcAppearance0104, PcStyle0104,
};

use crate::world_targeting::*;

fn guide_definition() -> GameplayNpcUiDefinition {
    GameplayNpcUiDefinition {
        npc_type: 707,
        name: "Ben Guide Changer".to_owned(),
        greeting_string_id: 707,
        greeting: " ".to_owned(),
        barker: None,
        team: 1,
        npc_level: 1,
        npc_style: 0,
        attack_effect: 0,
        service_category: 18,
        service_number: None,
        npc_class: 18,
        ai_type: 0,
        mesh_id: None,
        table_scale: None,
        attack_range_server_units: None,
        move_voice_owner: String::new(),
        radius_server_units: 90,
        height_server_units: 190,
        sight_range_server_units: 600,
        max_hp: 439,
    }
}

fn appearance(hp: i32) -> NetworkNpcAppearance0104 {
    NetworkNpcAppearance0104(NpcAppearance0104 {
        npc_id: 70,
        npc_type: 707,
        hp,
        condition_bit_flag: 0,
        position: [0; 3],
        angle: 0,
        barker_type: 0,
    })
}

fn pc_appearance(hp: i32, special_state: i8) -> NetworkPcAppearance0104 {
    NetworkPcAppearance0104(PcAppearance0104 {
        id: 80,
        style: PcStyle0104 {
            pc_uid: 800,
            name_check: 1,
            first_name: FixedUtf16::from_str("Remote").unwrap(),
            last_name: FixedUtf16::from_str("Player").unwrap(),
            gender: 1,
            face_style: 1,
            hair_style: 1,
            hair_color: 1,
            skin_color: 1,
            eye_color: 1,
            height: 1,
            body: 1,
            class: 0,
        },
        condition_bit_flag: 0,
        pc_state: 1,
        special_state,
        level: 1,
        hp,
        map_number: 1,
        position: [0; 3],
        angle: 0,
        equipment: [ItemBase0104 {
            item_type: 0,
            item_id: 0,
            option: 0,
            time_limit: 0,
        }; 9],
        nano: Nano0104 {
            id: 0,
            skill_id: 0,
            stamina: 0,
        },
        render_type: 1,
    })
}

fn context() -> LegacyAvatarActionContext {
    LegacyAvatarActionContext {
        attack_half_angle_degrees: 90.0,
        attack_range: 2.0,
        ..default()
    }
}

#[test]
fn static_ai_zero_guide_npc_is_visible_and_talkable() {
    let entity = Entity::from_bits(7);
    let sample = world_npc_target_sample(
        entity,
        &appearance(439),
        &guide_definition(),
        &Transform::from_xyz(0.0, 0.0, -3.0),
        &Transform::IDENTITY,
        Vec3::NEG_Z,
        &context(),
        |_, _| false,
    )
    .unwrap();

    assert_eq!(sample.entity, entity);
    assert_eq!(sample.kind, LegacyTargetKind::Npc { team: 1 });
    assert!(sample.in_view);
    assert!(sample.talk_enabled);
    assert_eq!(guide_definition().ai_type, 0);
}

#[test]
fn normal_world_talk_gate_uses_exact_sight_range() {
    let definition = guide_definition();
    let inside = world_npc_target_sample(
        Entity::from_bits(1),
        &appearance(439),
        &definition,
        &Transform::from_xyz(0.0, 0.0, -5.9),
        &Transform::IDENTITY,
        Vec3::NEG_Z,
        &context(),
        |_, _| false,
    )
    .unwrap();
    let outside = world_npc_target_sample(
        Entity::from_bits(2),
        &appearance(439),
        &definition,
        &Transform::from_xyz(0.0, 0.0, -6.1),
        &Transform::IDENTITY,
        Vec3::NEG_Z,
        &context(),
        |_, _| false,
    )
    .unwrap();

    assert!(inside.talk_enabled);
    assert!(outside.in_view);
    assert!(!outside.talk_enabled);
}

#[test]
fn quest_object_sight_range_cannot_extend_server_interaction_distance() {
    let mut definition = guide_definition();
    // Mission 520 time points (tasks 598–600): class 100, sight range 1500.
    definition.npc_class = 100;
    definition.sight_range_server_units = 1500;
    for (position, expected) in [
        (Vec3::new(0.0, 0.0, -7.99), true),
        (Vec3::new(0.0, 0.0, -8.0), true),
        (Vec3::new(0.0, 0.0, -8.02), false),
        (Vec3::new(0.0, -1.5, -7.9), false),
        (Vec3::new(0.0, 0.0, -12.0), false),
    ] {
        let sample = world_npc_target_sample(
            Entity::from_bits(1),
            &appearance(439),
            &definition,
            &Transform::from_translation(position),
            &Transform::IDENTITY,
            Vec3::NEG_Z,
            &context(),
            |_, _| false,
        )
        .unwrap();
        assert!(
            sample.in_view,
            "the object remains selectable visually at {position}"
        );
        assert_eq!(sample.talk_enabled, expected, "NPC root {position}");
        use crate::avatar_action::{
            LegacyAvatarActionInput, LegacyAvatarActionIntent, LegacyAvatarActionState,
            LegacyAvatarClipBindings, schedule_legacy_action_frame, select_legacy_targets,
        };
        let selection = select_legacy_targets(
            &LegacyAvatarTargetFeed {
                source_connected: true,
                samples: vec![sample],
                trigger: None,
            },
            &context(),
        );
        let scheduled = schedule_legacy_action_frame(
            LegacyAvatarActionInput {
                primary_just_pressed: true,
                ..default()
            },
            &context(),
            &selection,
            &mut LegacyAvatarActionState::default(),
            &LegacyAvatarClipBindings::default(),
            1.0 / 60.0,
        );
        assert_eq!(
            scheduled
                .actions
                .contains(&LegacyAvatarActionIntent::TalkNpc(sample.entity)),
            expected
        );
    }
}

#[test]
fn interaction_range_rechecks_moving_roots_and_rejects_invalid_positions() {
    let npc = Vec3::new(100.0, 20.0, -100.0);
    assert!(world_npc_in_interaction_range(npc + Vec3::Z * 8.0, npc));
    assert!(!world_npc_in_interaction_range(npc + Vec3::Z * 8.02, npc));
    assert!(!world_npc_in_interaction_range(Vec3::NAN, npc));
    assert!(!world_npc_in_interaction_range(
        npc,
        Vec3::splat(f32::INFINITY)
    ));
}

#[test]
fn dead_and_non_cone_npc_classes_fail_closed() {
    let mut definition = guide_definition();
    assert!(
        world_npc_target_sample(
            Entity::from_bits(1),
            &appearance(0),
            &definition,
            &Transform::from_xyz(0.0, 0.0, -2.0),
            &Transform::IDENTITY,
            Vec3::NEG_Z,
            &context(),
            |_, _| false,
        )
        .is_none()
    );

    for excluded_class in [25, 111] {
        definition.npc_class = excluded_class;
        assert!(
            world_npc_target_sample(
                Entity::from_bits(1),
                &appearance(439),
                &definition,
                &Transform::from_xyz(0.0, 0.0, -2.0),
                &Transform::IDENTITY,
                Vec3::NEG_Z,
                &context(),
                |_, _| false,
            )
            .is_none()
        );
    }
}

#[test]
fn authored_world_blocker_removes_view_and_attack_cones() {
    let sample = world_npc_target_sample(
        Entity::from_bits(1),
        &appearance(439),
        &guide_definition(),
        &Transform::from_xyz(0.0, 0.0, -3.0),
        &Transform::IDENTITY,
        Vec3::NEG_Z,
        &context(),
        |_, _| true,
    )
    .unwrap();

    assert!(!sample.in_view);
    assert!(!sample.in_attack_cone);
    assert!(sample.in_attack_arc);
    // Talk range is distance-owned in the clean client; selection still
    // requires `in_view`, so a blocked NPC cannot receive TalkNpc.
    assert!(sample.talk_enabled);
}

#[test]
fn out_of_range_npc_does_not_scan_static_world_line_of_sight() {
    let raycasts = std::cell::Cell::new(0_usize);
    let sample = world_npc_target_sample(
        Entity::from_bits(2),
        &appearance(439),
        &guide_definition(),
        &Transform::from_xyz(0.0, 0.0, -1_000.0),
        &Transform::IDENTITY,
        Vec3::NEG_Z,
        &context(),
        |_, _| {
            raycasts.set(raycasts.get() + 1);
            false
        },
    )
    .unwrap();

    assert!(!sample.in_view);
    assert!(!sample.in_attack_cone);
    assert_eq!(raycasts.get(), 0);
}

#[test]
fn remote_pc_view_uses_camera_forward_and_the_strict_user_container_limit() {
    let view_limit = AUTHORED_CHARACTER_CONTROLLER_RADIUS
        .max(AUTHORED_CHARACTER_CONTROLLER_HEIGHT * 0.5)
        + LEGACY_WORLD_VIEW_DISTANCE * LEGACY_WORLD_PC_VIEW_DISTANCE_FACTOR;
    let avatar = Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2));
    let inside = world_pc_target_sample(
        Entity::from_bits(80),
        &pc_appearance(1_000, 0),
        &Transform::from_xyz(0.0, 0.0, -view_limit + 0.001),
        &avatar,
        Vec3::NEG_Z,
        &context(),
    )
    .unwrap();
    let boundary = world_pc_target_sample(
        Entity::from_bits(81),
        &pc_appearance(1_000, 0),
        &Transform::from_xyz(0.0, 0.0, -view_limit),
        &avatar,
        Vec3::NEG_Z,
        &context(),
    )
    .unwrap();

    assert!(inside.in_view);
    assert!(
        !boundary.in_view,
        "UserContainer uses a strict distance test"
    );
    assert_eq!(inside.kind, LegacyTargetKind::Player);
    assert!(inside.talk_enabled);
}

#[test]
fn remote_pc_vertical_distance_is_halved_after_the_strict_capsule_clamp() {
    let sample = world_pc_target_sample(
        Entity::from_bits(80),
        &pc_appearance(1_000, 0),
        // Candidate top equals the source eye. The source's strict clamp
        // falls through to bottom (-0.8), producing -1.6 before halving.
        &Transform::from_xyz(0.0, -0.8, 0.0),
        &Transform::IDENTITY,
        Vec3::NEG_Z,
        &context(),
    )
    .unwrap();
    assert!((sample.distance - 0.8).abs() < 0.000_01);
}

#[test]
fn remote_pc_dead_invisible_and_invulnerable_states_fail_closed() {
    for (hp, special_state) in [(0, 0), (1_000, 2), (1_000, 4), (1_000, 6)] {
        assert!(
            world_pc_target_sample(
                Entity::from_bits(80),
                &pc_appearance(hp, special_state),
                &Transform::from_xyz(0.0, 0.0, -2.0),
                &Transform::IDENTITY,
                Vec3::NEG_Z,
                &context(),
            )
            .is_none()
        );
    }
}

#[test]
fn remote_pc_sample_reaches_the_shared_focused_player_selection() {
    let sample = world_pc_target_sample(
        Entity::from_bits(80),
        &pc_appearance(1_000, 0),
        &Transform::from_xyz(0.0, 0.0, -2.0),
        &Transform::IDENTITY,
        Vec3::NEG_Z,
        &context(),
    )
    .unwrap();
    let feed = LegacyAvatarTargetFeed {
        source_connected: true,
        samples: vec![sample],
        trigger: None,
    };
    let selection = crate::avatar_action::select_legacy_targets(&feed, &context());
    assert_eq!(
        selection.focused_player.map(|target| target.entity),
        Some(sample.entity)
    );
}
