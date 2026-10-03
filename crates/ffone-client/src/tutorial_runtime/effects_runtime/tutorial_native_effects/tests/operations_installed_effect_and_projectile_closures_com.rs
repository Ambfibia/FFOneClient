use super::*;

#[test]
fn trail_keeps_degenerate_tail_and_plus_plus_last_edge() {
    let mesh = trail_mesh(
        &[Vec3::ZERO, Vec3::X, Vec3::new(2.0, 0.0, 0.0)],
        Vec3::Y,
        0.0,
        1.0,
    );
    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() {
        bevy::mesh::VertexAttributeValues::Float32x3(values) => values,
        _ => panic!("unexpected position format"),
    };
    assert_eq!(positions[4], positions[5]);
    let uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap() {
        bevy::mesh::VertexAttributeValues::Float32x2(values) => values,
        _ => panic!("unexpected UV format"),
    };
    assert!((uvs[0][0] - 2.0 / 3.0).abs() < 1.0e-6);
    assert_eq!(uvs[0][1], 0.0);
    assert!((uvs[1][0] - 2.0 / 3.0).abs() < 1.0e-6);
    assert_eq!(uvs[1][1], 1.0);
    assert_eq!(uvs[4], [0.0, 0.0]);
    assert_eq!(uvs[5], [0.0, 1.0]);
    let indices = mesh.indices().unwrap();
    assert_eq!(indices.len(), 18);
    assert_eq!(
        indices.iter().skip(12).collect::<Vec<_>>(),
        vec![0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn trail_inserts_each_retrobution_ftimes_substep() {
    let mut points = [Vec3::ZERO; 8];
    push_interpolated_trail_points(&mut points, Vec3::X * 4.0, 0.25);
    assert_eq!(
        &points[4..],
        &[Vec3::X, Vec3::X * 2.0, Vec3::X * 3.0, Vec3::X * 4.0]
    );
}

#[test]
fn installed_effect_and_projectile_closures_compile_to_native_plans() {
    let asset_root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let library = TutorialEffectLibrary::load(asset_root).unwrap();

    let buttercup_handoff = compile_effect_plan(372, library.effects.get(&372).unwrap());
    assert_eq!(
        buttercup_handoff
            .plan
            .as_ref()
            .unwrap_or_else(|| {
                panic!("effect 372 blockers: {:#?}", buttercup_handoff.blockers)
            })
            .rendered_nodes,
        5
    );
    assert!(
        buttercup_handoff.blockers.is_empty(),
        "effect 372 blockers: {:#?}",
        buttercup_handoff.blockers
    );

    let effect = compile_effect_plan(750, library.effects.get(&750).unwrap());
    assert_eq!(
        effect
            .plan
            .as_ref()
            .unwrap_or_else(|| panic!("effect 750 blockers: {:#?}", effect.blockers))
            .rendered_nodes,
        2
    );
    assert!(effect.blockers.is_empty());

    for effect_id in 527..=529 {
        let nano_summon =
            compile_effect_plan(effect_id, library.effects.get(&effect_id).unwrap());
        let nano_summon_plan = nano_summon.plan.as_ref().unwrap_or_else(|| {
            panic!("effect {effect_id} blockers: {:#?}", nano_summon.blockers)
        });
        assert!(
            nano_summon_plan.rendered_nodes >= 2,
            "effect {effect_id} must retain its multi-card call effect"
        );
        assert!(
            nano_summon_plan
                .emitters
                .iter()
                .all(|emitter| emitter.legacy_gamma_accumulation_gain == 2.0)
        );
        assert!(
            nano_summon.blockers.is_empty(),
            "effect {effect_id} blockers: {:#?}",
            nano_summon.blockers
        );
    }

    for effect_id in [668, 865, 866] {
        let mission_indicator =
            compile_effect_plan(effect_id, library.effects.get(&effect_id).unwrap());
        assert!(
            mission_indicator.plan.is_some(),
            "mission indicator {effect_id} blockers: {:#?}",
            mission_indicator.blockers
        );
        assert!(
            mission_indicator.blockers.iter().all(|blocker| matches!(
                blocker.reason,
                TutorialNativeClosureBlockerReason::LegacyMeshRenderer
                    | TutorialNativeClosureBlockerReason::LegacyAnimationRuntimeUnavailable
            )),
            "mission indicator {effect_id} blockers: {:#?}",
            mission_indicator.blockers
        );
    }
    let waypoint = compile_effect_plan(668, library.effects.get(&668).unwrap());
    let waypoint_plan = waypoint
        .plan
        .as_ref()
        .unwrap_or_else(|| panic!("effect 668 blockers: {:#?}", waypoint.blockers));
    assert_eq!(waypoint_plan.emitters.len(), 5);
    let ground_projection = waypoint_plan
        .emitters
        .iter()
        .find(|emitter| emitter.source_path_id == 16234)
        .expect("ES668 waypoint ground-projection emitter");
    assert_eq!(
        ground_projection.render_mode,
        LegacyParticleRenderMode::HorizontalBillboard
    );
    assert_eq!(ground_projection.blend_mode, ParticleBlendMode::SrcAlphaOne);
    assert_eq!(ground_projection.legacy_gamma_accumulation_gain, 2.0);
    assert_eq!(ground_projection.texture.width, 128);
    assert_eq!(ground_projection.texture.height, 128);

    let explosion = compile_effect_plan(767, library.effects.get(&767).unwrap());
    let explosion_plan = explosion
        .plan
        .as_ref()
        .unwrap_or_else(|| panic!("effect 767 blockers: {:#?}", explosion.blockers));
    assert_eq!(explosion_plan.rendered_nodes, 1);
    assert!(explosion.blockers.is_empty());
    assert_eq!(
        explosion_plan.emitters[0].legacy_gamma_accumulation_gain,
        1.0
    );
    assert_eq!(explosion_plan.emitters[0].uv_tiles, UVec2::new(4, 2));
    assert_eq!(explosion_plan.emitters[0].uv_cycles, 1.0);
    assert_eq!(
        particle_uv_scale_offset(&explosion_plan.emitters[0], 0.0),
        Vec4::new(0.25, 0.5, 0.0, 0.0)
    );
    assert_eq!(
        particle_uv_scale_offset(&explosion_plan.emitters[0], 0.5),
        Vec4::new(0.25, 0.5, 0.0, 0.5)
    );
    for normalized in [0.0, 0.5, 1.0] {
        let size = particle_size(&explosion_plan.emitters[0], normalized, 1.0);
        assert_eq!(
            size.x, size.y,
            "ES767's width-only curve must remain square at t={normalized}"
        );
    }

    for (effect_id, expected_scene) in [
        (425, "map/shared/effects/models/es740/m_inven.glb"),
        (668, "map/shared/effects/models/es668/patrol_elena7.glb"),
        (705, "map/shared/effects/models/es705/nano_get.glb"),
        (734, "map/shared/effects/models/es734/T_pistol.glb"),
        (736, "map/shared/effects/models/es736/t_bubbles.glb"),
        (
            739,
            "map/shared/effects/models/es739/Tutorial_dexterhologram.glb",
        ),
        (740, "map/shared/effects/models/es740/m_inven.glb"),
        (
            741,
            "map/shared/effects/models/es741/Tutorial_dexterhologram_1.glb",
        ),
        (
            751,
            "map/shared/effects/models/es751/npc_samuraijack_event_eff.glb",
        ),
        (
            833,
            "map/shared/effects/models/es833/vehicle_board_inven0.glb",
        ),
        (
            834,
            "map/shared/effects/models/es834/m_vehicle_scooter_inven.glb",
        ),
    ] {
        let mesh_effect =
            compile_effect_plan(effect_id, library.effects.get(&effect_id).unwrap());
        let mesh_plan = mesh_effect.plan.as_ref().unwrap_or_else(|| {
            panic!("effect {effect_id} blockers: {:#?}", mesh_effect.blockers)
        });
        if effect_id == 668 || effect_id == 739 {
            assert!(
                mesh_plan.rendered_nodes > 1,
                "ES{effect_id} must keep its exact auxiliary particles beside the mesh"
            );
        } else {
            assert_eq!(mesh_plan.rendered_nodes, 1);
        }
        assert_eq!(mesh_plan.mesh_scene, Some(expected_scene));
        assert!(mesh_effect.blockers.is_empty());
        if effect_id == 734 {
            let material_animation = mesh_plan
                .material_animation
                .as_ref()
                .expect("effect 734 exact material animation");
            assert_eq!(material_animation.curves.len(), 20);
            assert!((material_animation.duration - 0.300_000_01).abs() < 1e-5);
        } else if effect_id == 736 {
            let material_animation = mesh_plan
                .material_animation
                .as_ref()
                .expect("effect 736 exact material animation");
            assert_eq!(material_animation.curves.len(), 42);
            assert!((material_animation.duration - 6.999_995).abs() < 1e-5);
        } else if effect_id == 739 {
            let material_animation = mesh_plan
                .material_animation
                .as_ref()
                .expect("effect 739 exact material animation");
            assert_eq!(mesh_plan.rendered_nodes, 5);
            assert_eq!(material_animation.curves.len(), 14);
            assert!((material_animation.duration - 3.333_331_6).abs() < 1e-5);
            assert!(material_animation.curves.iter().any(|curve| matches!(
                curve.property,
                MaterialAnimatedProperty::UvRotationDegrees
            )));
        } else if effect_id == 425 || effect_id == 740 {
            let material_animation = mesh_plan
                .material_animation
                .as_ref()
                .expect("inventory computer exact material animation");
            assert_eq!(material_animation.curves.len(), 16);
            assert!((material_animation.duration - 3.333_331_6).abs() < 1e-5);
            assert!(material_animation.curves.iter().any(|curve| matches!(
                curve.property,
                MaterialAnimatedProperty::UvRotationDegrees
            )));
        } else if effect_id == 741 {
            let material_animation = mesh_plan
                .material_animation
                .as_ref()
                .expect("effect 741 exact material animation");
            assert_eq!(material_animation.curves.len(), 14);
            assert!((material_animation.duration - 3.333_331_6).abs() < 1e-5);
        } else if effect_id == 705 {
            let animation = mesh_plan
                .material_animation
                .as_ref()
                .expect("Nano acquisition material animation");
            assert!(!animation.curves.is_empty());
            assert!(animation.duration > 0.0);
        } else if effect_id == 751 {
            let material_animation = mesh_plan
                .material_animation
                .as_ref()
                .expect("effect 751 exact material animation");
            assert_eq!(material_animation.curves.len(), 21);
            assert!((material_animation.duration - 1.466_666_6).abs() < 1e-5);
        } else if effect_id == 833 {
            let material_animation = mesh_plan
                .material_animation
                .as_ref()
                .expect("board inventory computer exact material animation");
            assert_eq!(material_animation.curves.len(), 9);
            assert!((material_animation.duration - 3.333_331_6).abs() < 1e-5);
        } else if effect_id == 834 {
            let material_animation = mesh_plan
                .material_animation
                .as_ref()
                .expect("scooter inventory computer exact material animation");
            assert_eq!(material_animation.curves.len(), 16);
            assert!((material_animation.duration - 3.333_331_6).abs() < 1e-5);
        } else {
            assert!(mesh_plan.material_animation.is_none());
        }
    }

    for effect_id in [15, 391] {
        let projectile = compile_projectile_plan(
            effect_id,
            library.projectile_effects.get(&effect_id).unwrap(),
        );
        let plan = projectile.plan.as_ref().unwrap_or_else(|| {
            panic!(
                "projectile {effect_id} blockers: {:#?}",
                projectile.blockers
            )
        });
        assert_eq!(plan.rendered_nodes, 2);
        assert!(
            projectile.blockers.is_empty(),
            "projectile {effect_id} retained blockers: {:#?}",
            projectile.blockers
        );
    }
}

#[test]
fn every_tutorial_effect_has_a_native_plan() {
    let asset_root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let library = TutorialEffectLibrary::load(asset_root).unwrap();
    let mut missing = Vec::new();
    for effect_id in ffone_runtime_contracts::RETROBUTION_TUTORIAL_EFFECT_IDS {
        let compiled = compile_effect_plan(effect_id, library.effects.get(&effect_id).unwrap());
        if compiled.plan.is_none() {
            missing.push((effect_id, compiled.blockers));
        }
    }
    assert!(
        missing.is_empty(),
        "every Retrobution tutorial effect must now have a native plan: {missing:#?}"
    );
}

#[test]
fn npc_warp_effect_has_a_complete_native_plan() {
    let asset_root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let library = TutorialEffectLibrary::load(asset_root).unwrap();
    for effect_id in ffone_runtime_contracts::RETROBUTION_NPC_WARP_EFFECT_IDS {
        let compiled = compile_effect_plan(effect_id, library.effects.get(&effect_id).unwrap());
        let rendered_nodes = compiled.plan.as_ref().map_or(0, |plan| plan.rendered_nodes);
        assert!(
            rendered_nodes > 0 && compiled.blockers.is_empty(),
            "clean NPC Warp effect {effect_id} must render completely: {:#?}",
            compiled.blockers
        );
    }
}

#[test]
fn fusion_actor_effect_native_coverage_is_explicit() {
    let asset_root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let library = TutorialEffectLibrary::load(asset_root).unwrap();
    let mut ready = Vec::new();
    let mut blocked = Vec::new();
    for effect_id in ffone_runtime_contracts::RETROBUTION_FUSION_ACTOR_EFFECT_IDS {
        let compiled = compile_effect_plan(effect_id, library.effects.get(&effect_id).unwrap());
        let rendered_nodes = compiled.plan.as_ref().map_or(0, |plan| plan.rendered_nodes);
        if rendered_nodes > 0 && compiled.blockers.is_empty() {
            ready.push(effect_id);
        } else {
            assert!(
                !compiled.blockers.is_empty(),
                "Fusion effect {effect_id} has neither a complete native plan nor an exact blocker"
            );
            blocked.push(effect_id);
        }
    }
    println!("Fusion native effects ready={ready:?}; blocked={blocked:?}");
    assert_eq!(ready.len() + blocked.len(), 95);
}

#[test]
fn non_fusion_character_actor_effect_native_coverage_is_explicit() {
    let asset_root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let library = TutorialEffectLibrary::load(asset_root).unwrap();
    let mut ready = Vec::new();
    let mut blocked = Vec::new();
    for effect_id in ffone_runtime_contracts::RETROBUTION_CHARACTER_ACTOR_EFFECT_IDS {
        let compiled = compile_effect_plan(effect_id, library.effects.get(&effect_id).unwrap());
        let rendered_nodes = compiled.plan.as_ref().map_or(0, |plan| plan.rendered_nodes);
        if rendered_nodes > 0 && compiled.blockers.is_empty() {
            ready.push(effect_id);
        } else {
            assert!(
                !compiled.blockers.is_empty(),
                "character actor effect {effect_id} has neither a complete native plan nor an exact blocker"
            );
            blocked.push(effect_id);
        }
    }
    println!("non-Fusion character effects ready={ready:?}; blocked={blocked:?}");
    assert_eq!(ready.len() + blocked.len(), 26);
    for effect_id in [425, 833, 834] {
        assert!(
            ready.contains(&effect_id),
            "inventory computer effect {effect_id} must have a complete native plan"
        );
    }
}

#[test]
fn every_world_ep_effect_has_a_complete_native_plan() {
    let asset_root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let library = TutorialEffectLibrary::load(asset_root).unwrap();
    let mut missing = Vec::new();
    for effect_id in ffone_runtime_contracts::RETROBUTION_WORLD_EP_EFFECT_IDS {
        let compiled = compile_effect_plan(effect_id, library.effects.get(&effect_id).unwrap());
        let rendered_nodes = compiled.plan.as_ref().map_or(0, |plan| plan.rendered_nodes);
        println!(
            "world EP effect {effect_id}: rendered_nodes={rendered_nodes}, blockers={:#?}",
            compiled.blockers
        );
        if rendered_nodes == 0 || !compiled.blockers.is_empty() {
            missing.push((effect_id, compiled.blockers));
        }
    }
    assert!(
        missing.is_empty(),
        "every Retrobution world EP effect must have a complete native plan: {missing:#?}"
    );
}
