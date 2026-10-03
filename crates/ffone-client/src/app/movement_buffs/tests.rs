use super::*;
use ffone_protocol::TimeBuff0104;

fn update(id: i32, kind: i32, value: i32) -> PcBuffUpdate0104 {
    PcBuffUpdate0104 {
        buff_id: id,
        update_kind: kind,
        buff_type: 0,
        time_buff: TimeBuff0104 { value, ..default() },
        condition_bit_flag: 0,
    }
}

#[test]
fn nano_rocket_all_production_nanos_launch_from_base_and_slot_power() {
    let assets = ffone_client::assets::AssetLocator::open(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"),
    )
    .unwrap();
    let content = TutorialMissionContent::open(&assets).unwrap();
    let mut skills = std::collections::BTreeSet::new();
    for nano in content.gameplay_nanos() {
        let Some(journal) = content.journal_nano(i32::from(nano.nano_id)) else {
            continue;
        };
        for tune in &journal.skills {
            let Some(skill) = i16::try_from(tune.skill_id)
                .ok()
                .and_then(|id| content.gameplay_skill(id))
            else {
                continue;
            };
            if skill.skill_type != 29 {
                continue;
            }
            skills.insert(skill.skill_id);
            assert!(ordinary_world_nano_skill_supported(skill));
            for slot in 0..3 {
                let mut buffs = MovementBuffs::default();
                buffs.apply(update(3, 1, 250));
                let base = MovementBuffBase {
                    run: 600,
                    jump: 568,
                };
                let mut controller = LegacyPlayerController::from_server_attributes(600, 818);
                assert!(launch_nano_rocket(
                    &mut controller,
                    Some(&base),
                    &buffs,
                    slot,
                    skill
                ));
                assert!((controller.velocity.y - 8.68).abs() < 0.0001);
                assert!(controller.jumping);
                buffs.apply(update(21 + slot as i32, 1, 3));
                assert!(launch_nano_rocket(
                    &mut controller,
                    Some(&base),
                    &buffs,
                    slot,
                    skill
                ));
                assert!((controller.velocity.y - 9.58).abs() < 0.0001);
                buffs.apply(update(21 + slot as i32, 2, 0));
                assert_eq!(buffs.rocket_jump_bonus(slot, skill), Some(300));
            }
        }
    }
    assert!(skills.contains(&36));
    assert_eq!(
        skills,
        std::collections::BTreeSet::from([36, 64, 79, 213, 222, 247, 250, 264, 268])
    );
}

#[test]
fn authoritative_stun_and_sleep_gate_controller_and_clear_on_removal() {
    let mut app=App::new();
    app.init_resource::<MovementBuffs>().init_resource::<SkillBuffUiModel>()
        .init_resource::<LocalVehiclePresentationRuntime>()
        .add_systems(Update,sync_movement_buffs);
    let player=app.world_mut().spawn((LocalPlayer,LegacyPlayerController::from_baseline_table())).id();
    for flags in [0x200,0x400,0x600,0,0x1000] {
        app.world_mut().resource_mut::<SkillBuffUiModel>().local_condition_bit_flag=flags;
        app.update();
        assert_eq!(app.world().get::<LegacyPlayerController>(player).unwrap().incapacitated,flags & 0x600 != 0);
    }
}

#[test]
fn snare_hit_condition_slows_before_buff_value_and_timeout_restores_speed() {
    let mut app = App::new();
    app.init_resource::<MovementBuffs>().init_resource::<SkillBuffUiModel>()
        .init_resource::<LocalVehiclePresentationRuntime>()
        .add_systems(Update, sync_movement_buffs);
    let player = app.world_mut().spawn((LocalPlayer,
        LegacyPlayerController::from_server_attributes(600, 700))).id();
    for (flags, speed) in [(0, 600), (0x80, 300), (0x80, 300), (0, 600)] {
        app.world_mut().resource_mut::<SkillBuffUiModel>().local_condition_bit_flag = flags;
        app.update();
        let controller = app.world().get::<LegacyPlayerController>(player).unwrap();
        assert_eq!(controller.run_speed_server_units, speed);
        assert!(!controller.incapacitated, "Snare slows; it does not stun");
    }
}

#[test]
fn every_production_dash_has_a_local_controller_activation() {
    let locator=ffone_client::assets::AssetLocator::open(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game")).unwrap();
    let content=TutorialMissionContent::open(&locator).unwrap();
    let mut ids=BTreeSet::new();
    for nano in content.gameplay_nanos() {
        let Some(nano)=content.journal_nano(i32::from(nano.nano_id)) else { continue; };
        for tune in &nano.skills {
            let Some(skill)=i16::try_from(tune.skill_id).ok().and_then(|id|content.gameplay_skill(id)) else {continue;};
            if skill.skill_type != 40 {continue;}
            ids.insert(skill.skill_id);
            for slot in 0..3 {
                let mut controller=LegacyPlayerController::from_baseline_table();
                assert!(ordinary_world_nano_skill_supported(skill));
                assert!(launch_nano_rocket(&mut controller,None,&MovementBuffs::default(),slot,skill));
            }
        }
    }
    assert!(!ids.is_empty());
}

#[test]
fn gm_speed_survives_projection_and_buff_removal() {
    for capture_base_first in [false, true] {
        let mut app = App::new();
        app.init_resource::<MovementBuffs>()
            .init_resource::<LocalVehiclePresentationRuntime>()
            .add_systems(Update, sync_movement_buffs);
        let player = app
            .world_mut()
            .spawn((
                LocalPlayer,
                LegacyPlayerController::from_server_attributes(600, 568),
            ))
            .id();
        if capture_base_first {
            app.world_mut()
                .resource_mut::<MovementBuffs>()
                .apply(update(1, 1, 200));
            app.update();
        }
        let frame = DecodedFrame {
            packet_type: packet::P_FE2CL_GM_REP_PC_SET_VALUE,
            flags: 0,
            checksum: 0,
            payload: ffone_protocol::GmSetValueReply0104 {
                pc_id: 81,
                value_type: GM_SET_VALUE_SPEED_0104,
                value: 1200,
            }
            .encode(),
        };
        let speed = decode_local_gm_speed_reply_0104(&frame, Some(81))
            .unwrap()
            .unwrap();
        let mut query = app
            .world_mut()
            .query::<(&mut LegacyPlayerController, Option<&mut MovementBuffBase>)>();
        let (mut controller, mut base) = query.get_mut(app.world_mut(), player).unwrap();
        apply_gm_speed(&mut controller, base.as_deref_mut(), speed);
        app.world_mut()
            .resource_mut::<MovementBuffs>()
            .apply(update(1, 1, 200));
        app.world_mut()
            .resource_mut::<MovementBuffs>()
            .apply(update(8, 1, 100));
        for _ in 0..60 {
            app.update();
        }
        assert_eq!(
            app.world()
                .get::<LegacyPlayerController>(player)
                .unwrap()
                .run_speed_server_units,
            1300
        );
        for id in [1, 8] {
            app.world_mut()
                .resource_mut::<MovementBuffs>()
                .apply(update(id, 2, 0));
        }
        app.update();
        let controller = app.world().get::<LegacyPlayerController>(player).unwrap();
        assert_eq!(controller.run_speed_server_units, 1200);
        assert_eq!(controller.jump_height_server_units, 568);
    }
}

#[test]
fn server_buff_values_replace_and_remove_without_stacking() {
    let mut buffs = MovementBuffs::default();
    buffs.apply(update(1, 1, 200));
    buffs.apply(update(1, 1, 200));
    buffs.apply(update(3, 1, 250));
    buffs.apply(update(8, 1, 100));
    assert_eq!(buffs.attributes(600, 568, false), (700, 818));
    assert_eq!(buffs.attributes(600, 568, true), (700, 568));
    buffs.apply(update(1, 3, 300));
    assert_eq!(buffs.attributes(600, 568, false), (800, 818));
    buffs.apply(update(8, 3, 5000));
    assert_eq!(buffs.attributes(600, 568, false).0, 0);
    for id in [1, 3, 8] {
        buffs.apply(update(id, 2, 999));
    }
    assert_eq!(buffs.attributes(600, 568, false), (600, 568));
}

#[test]
fn buffs_change_sixty_hz_displacement_and_jump_launch_velocity() {
    use bevy::time::TimeUpdateStrategy;
    use ffone_client::movement::simulate_legacy_players;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ))
        .init_resource::<MovementBuffs>()
        .init_resource::<LocalVehiclePresentationRuntime>()
        .init_resource::<MovementIntentQueue>()
        .insert_resource(LegacyInputState {
            local_axis: Vec2::Y,
            ..default()
        })
        .add_systems(
            Update,
            (sync_movement_buffs, simulate_legacy_players).chain(),
        );
    app.update();
    let player = app
        .world_mut()
        .spawn((
            LocalPlayer,
            Transform::default(),
            LegacyPlayerController::from_baseline_table().with_placeholder_ground_plane(0.0),
        ))
        .id();
    app.world_mut()
        .resource_mut::<MovementBuffs>()
        .apply(update(1, 1, 200));
    for _ in 0..60 {
        app.update();
    }
    // Server heading zero faces native +Z (the gameplay root is reflected).
    assert!((app.world().get::<Transform>(player).unwrap().translation.z - 8.0).abs() < 0.001);
    app.world_mut()
        .resource_mut::<MovementBuffs>()
        .apply(update(3, 1, 250));
    app.world_mut()
        .resource_mut::<LegacyInputState>()
        .jump_just_pressed = true;
    app.update();
    let controller = app.world().get::<LegacyPlayerController>(player).unwrap();
    assert!((controller.velocity.y - 8.18).abs() < 1e-5);
    assert!(controller.jumping);
}

#[test]
fn decoded_buff_packet_reaches_controller_and_recall_restores_attributes() {
    let mut app = App::new();
    app.init_resource::<MovementBuffs>()
        .init_resource::<LocalVehiclePresentationRuntime>()
        .add_systems(Update, sync_movement_buffs);
    let player = app
        .world_mut()
        .spawn((
            LocalPlayer,
            LegacyPlayerController::from_server_attributes(650, 580),
        ))
        .id();
    let mut icons = SkillBuffUiModel::default();
    for (id, value) in [(1, 200), (3, 250)] {
        apply_skill_buff_frame(
            &DecodedFrame {
                packet_type: packet::P_FE2CL_PC_BUFF_UPDATE,
                flags: 0,
                checksum: 0,
                payload: update(id, 1, value).encode(),
            },
            &mut icons,
            &mut app.world_mut().resource_mut::<MovementBuffs>(),
        )
        .unwrap();
    }
    for _ in 0..60 {
        app.update();
    }
    let controller = app.world().get::<LegacyPlayerController>(player).unwrap();
    assert_eq!(
        (
            controller.run_speed_server_units,
            controller.jump_height_server_units
        ),
        (850, 830)
    );
    app.world_mut()
        .resource_mut::<LocalVehiclePresentationRuntime>()
        .family = LegacyVehiclePresentationFamily::Board;
    app.update();
    assert_eq!(
        app.world()
            .get::<LegacyPlayerController>(player)
            .unwrap()
            .jump_height_server_units,
        580
    );
    app.world_mut()
        .resource_mut::<LocalVehiclePresentationRuntime>()
        .family = LegacyVehiclePresentationFamily::None;
    for id in [1, 3] {
        app.world_mut()
            .resource_mut::<MovementBuffs>()
            .apply(update(id, 2, 0));
    }
    app.update();
    let controller = app.world().get::<LegacyPlayerController>(player).unwrap();
    assert_eq!(
        (
            controller.run_speed_server_units,
            controller.jump_height_server_units
        ),
        (650, 580)
    );
}
