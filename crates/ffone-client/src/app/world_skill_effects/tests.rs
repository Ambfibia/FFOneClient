use super::*;

#[test]
fn status_buff_attachments_preserve_order_and_root_fallback() {
    assert_eq!(attachment_names(10), ["state"]);
    assert_eq!(attachment_names(13), ["Bip01 Footsteps"]);
    assert_eq!(
        attachment_names(19),
        ["center", "Bip01 spine", "Bip01 spine1"]
    );
    assert!(attachment_names(11).is_empty());
    assert_eq!(skill_buff(14), 13);
    assert_eq!(skill_buff(5), 8);
    assert_eq!(skill_buff(1), 0);
}

#[test]
fn corruption_packet_updates_hp_and_nano_and_emits_one_native_hit_projectile() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let assets = AssetLocator::open(&root).unwrap();
    let content = TutorialMissionContent::open(&assets).unwrap();
    let mut runtime = RuntimeStatus::default();
    runtime.player_id = Some(1);
    runtime.hp = Some(925);
    let mut buffs = SkillBuffUiModel::default();
    let mut payload = ffone_protocol::NpcSkillCorruptionHitPrefix0104 {
        npc_id: 75, skill_id: 118, style: 2, position: [0;3], target_count: 1,
    }.encode_prefix();
    let mut result = vec![0u8;40];
    for (offset, value) in [(0,1i32), (4,1), (12,196), (16,729)] {
        result[offset..offset+4].copy_from_slice(&value.to_le_bytes());
    }
    result[20] = 16;
    result[28..30].copy_from_slice(&2i16.to_le_bytes());
    result[30..32].copy_from_slice(&60i16.to_le_bytes());
    payload.extend(result);
    let projection = ffone_client::world_npc_skill_authority::decode_world_npc_skill_authority_0104(
        packet::P_FE2CL_NPC_SKILL_CORRUPTION_HIT, &payload).unwrap().unwrap();
    world_nano::apply_world_npc_skill_projection_to_local_status(&projection, &mut runtime, &mut buffs);
    assert_eq!(runtime.hp, Some(729));
    assert_eq!(runtime.nano_slots[0].nano_id, Some(2));
    assert_eq!(runtime.nano_slots[0].stamina, 60);
    assert!(runtime.nano_slots[0].active);
    let mut effects = TutorialEffectRuntime::with_library(TutorialEffectLibrary::load(&root).unwrap());
    effects.load_native_particle_catalog(&assets).unwrap();
    let mut app = App::new();
    app.insert_resource(State::new(ClientState::World)).insert_resource(runtime)
        .insert_resource(content).insert_resource(effects)
        .init_resource::<NetworkNanoEffectEvents0104>()
        .init_resource::<NetworkNpcResultEffectEvents0104>()
        .add_systems(Update, spawn_world_instant_skill_effects);
    app.world_mut().spawn((LocalPlayer, GlobalTransform::IDENTITY));
    app.world_mut().spawn((NetworkNpcAppearance0104(ffone_protocol::NpcAppearance0104 {
        npc_id:75,npc_type:75,hp:1000,condition_bit_flag:0,position:[0;3],angle:0,barker_type:0,
    }), GlobalTransform::from_translation(Vec3::X * 5.0)));
    app.world_mut().resource_mut::<NetworkNpcResultEffectEvents0104>().0.push_back(projection);
    for expected in [1, 0] {
        app.update();
        let mut effects = app.world_mut().resource_mut::<TutorialEffectRuntime>();
        effects.process_pending();
        assert_eq!(effects.drain_records().filter(|r| matches!(r.command,
            TutorialEffectRuntimeCommand::Projectile { bullet_type:169, .. })).count(), expected);
        let issues = effects.drain_issues().collect::<Vec<_>>();
        assert!(issues.is_empty(), "{issues:?}");
    }
}

#[test]
fn world_buff_effect_follows_authority_without_duplicates_and_cleans_up() {
    verify_persistent_effect(10);
}

#[test]
fn passive_speed_effect_plays_once_per_authoritative_activation() {
    verify_persistent_effect(1);
}

#[test]
fn root_buff_effect_attaches_to_unnamed_production_player() {
    verify_persistent_effect(2);
}

fn verify_persistent_effect(buff: i32) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let assets = AssetLocator::open(&root).unwrap();
    let content = TutorialMissionContent::open(&assets).unwrap();
    assert!(content.gameplay_skill_buff(10).unwrap().effect_id > 0);
    let mut app = App::new();
    let mut runtime = RuntimeStatus::default();
    runtime.hp = Some(500);
    app.insert_resource(State::new(ClientState::World))
        .insert_resource(runtime)
        .insert_resource(content)
        .insert_resource(SkillBuffUiModel::default())
        .insert_resource(TutorialEffectRuntime::with_library(
            TutorialEffectLibrary::load(&root).unwrap(),
        ))
        .add_systems(Update, sync_world_skill_effects);
    app.world_mut()
        .resource_mut::<TutorialEffectRuntime>()
        .load_native_particle_catalog(&AssetLocator::open(&root).unwrap())
        .unwrap();
    let player = app
        .world_mut()
        .spawn((LocalPlayer, GlobalTransform::IDENTITY))
        .id();
    let bone = app
        .world_mut()
        .spawn((Name::new("state"), GlobalTransform::IDENTITY))
        .id();
    app.world_mut().entity_mut(player).add_child(bone);
    let name = format!("world skill buff {player:?}/{buff}");
    app.world_mut()
        .resource_mut::<SkillBuffUiModel>()
        .local_condition_bit_flag = 1 << (buff - 1);
    for _ in 0..3 {
        app.update();
        let mut effects = app.world_mut().resource_mut::<TutorialEffectRuntime>();
        effects.process_pending();
        assert!(effects.has_named_native_instance(&name));
        assert_eq!(effects.active_native_instance_count(), 1);
    }
    app.world_mut()
        .resource_mut::<SkillBuffUiModel>()
        .local_condition_bit_flag = 0;
    app.update();
    app.world_mut()
        .resource_mut::<TutorialEffectRuntime>()
        .process_pending();
    assert!(
        !app.world()
            .resource::<TutorialEffectRuntime>()
            .has_named_native_instance(&name)
    );
    app.world_mut()
        .resource_mut::<SkillBuffUiModel>()
        .local_condition_bit_flag = 1 << (buff - 1);
    app.update();
    app.world_mut()
        .resource_mut::<TutorialEffectRuntime>()
        .process_pending();
    app.world_mut().entity_mut(player).despawn();
    app.update();
    app.world_mut()
        .resource_mut::<TutorialEffectRuntime>()
        .process_pending();
    assert_eq!(
        app.world()
            .resource::<TutorialEffectRuntime>()
            .active_native_instance_count(),
        0
    );
}

#[test]
fn instant_nano_result_waits_for_attachment_and_is_consumed_once() {
    use ffone_client::world_nano_authority::*;
    use ffone_protocol::{
        NanoSkillBuffResult0104, NanoSkillResult0104, NanoSkillTarget0104,
        NanoSkillUseDelivery0104,
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let content = TutorialMissionContent::open(&AssetLocator::open(&root).unwrap()).unwrap();
    let mut runtime = RuntimeStatus::default();
    runtime.player_id = Some(1);
    let mut app = App::new();
    app.insert_resource(State::new(ClientState::World))
        .insert_resource(runtime)
        .insert_resource(content)
        .init_resource::<NetworkNanoEffectEvents0104>()
        .init_resource::<NetworkNpcResultEffectEvents0104>()
        .insert_resource(TutorialEffectRuntime::with_library(
            TutorialEffectLibrary::load(&root).unwrap(),
        ))
        .add_systems(Update, spawn_world_instant_skill_effects);
    app.world_mut()
        .resource_mut::<TutorialEffectRuntime>()
        .load_native_particle_catalog(&AssetLocator::open(&root).unwrap())
        .unwrap();
    let player = app
        .world_mut()
        .spawn((LocalPlayer, GlobalTransform::IDENTITY))
        .id();
    let target = WorldNanoTargetPostState0104 {
        target: WorldNanoEntity0104 {
            kind: WorldNanoEntityKind0104::Player,
            id: 1,
        },
        absolute_hp: None,
        absolute_condition_bit_flag: Some(1 << 12),
        absolute_weapon_battery: None,
        absolute_nano_battery: None,
        absolute_nano_stamina: None,
        nano_deactivated: None,
        movement: None,
        source_result: NanoSkillResult0104::Buff(NanoSkillBuffResult0104 {
            target: NanoSkillTarget0104 {
                entity_type: 1,
                id: 1,
            },
            protected: 0,
            condition_bit_flag: 0,
        }),
    };
    app.world_mut()
        .resource_mut::<NetworkNanoEffectEvents0104>()
        .0
        .push_back(WorldNanoAuthoritativeProjection0104 {
            delivery: NanoSkillUseDelivery0104::LocalSuccess,
            skill_type: 14,
            caster: WorldNanoCasterPostState0104 {
                pc_id: 1,
                nano_id: 1,
                skill_id: 1,
                nano_stamina: 100,
                nano_deactivated: false,
                absolute_hp: None,
            },
            targets: vec![target],
        });
    app.update();
    app.world_mut()
        .resource_mut::<TutorialEffectRuntime>()
        .process_pending();
    assert_eq!(
        app.world()
            .resource::<TutorialEffectRuntime>()
            .active_native_instance_count(),
        0
    );
    let bone = app
        .world_mut()
        .spawn((Name::new("Bip01 Footsteps"), GlobalTransform::IDENTITY))
        .id();
    app.world_mut().entity_mut(player).add_child(bone);
    for _ in 0..3 {
        app.update();
        app.world_mut()
            .resource_mut::<TutorialEffectRuntime>()
            .process_pending();
        assert_eq!(
            app.world()
                .resource::<TutorialEffectRuntime>()
                .active_native_instance_count(),
            1
        );
    }
}
