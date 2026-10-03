use super::*;

#[test]
fn actor_effect_events_match_primary_animation_clip_metadata() {
    let event = |npc_type: i32, clip: &str, effect_id: i32| {
        RETROBUTION_ACTOR_EFFECT_EVENTS
            .iter()
            .find(|event| {
                event.npc_type == npc_type && event.clip == clip && event.effect_id == effect_id
            })
            .copied()
            .expect("primary AnimationClip effect event")
    };

    let numbuh_one = event(2667, "melee1", 734);
    assert_eq!(numbuh_one.source_clip_path_id, 892);
    assert!((numbuh_one.event_seconds - 0.180_000_01).abs() < 1.0e-7);
    assert_eq!(numbuh_one.node_name, Some("tag01"));

    let dexter = event(2668, "melee1", 734);
    assert_eq!(dexter.source_clip_path_id, 1_018);
    assert!((dexter.event_seconds - 0.264).abs() < 1.0e-7);
    assert_eq!(dexter.node_name, Some("tag01"));

    let numbuh_five = event(2669, "melee1", 734);
    assert_eq!(numbuh_five.source_clip_path_id, 1_117);
    assert!((numbuh_five.event_seconds - 0.233_333).abs() < 1.0e-7);
    assert_eq!(numbuh_five.node_name, Some("tag01"));

    let jack = RETROBUTION_ACTOR_EFFECT_EVENTS
        .iter()
        .filter(|event| event.npc_type == 2666 && event.clip == "melee1event")
        .collect::<Vec<_>>();
    assert_eq!(jack.len(), 3);
    assert_eq!(jack[0].source_clip_path_id, 1_103);
    assert_eq!(jack[0].effect_id, 751);
    assert_eq!(jack[0].node_name, None);
    assert_eq!(jack[1].node_name, Some("Bip01 R Hand"));
    assert_eq!(jack[2].node_name, Some("Bip01 L Hand"));

    let fusion_spawn = RETROBUTION_ACTOR_DEATH_PRESENTATION_EVENTS
        .iter()
        .find(|event| event.npc_type == 2674)
        .expect("Fusion Spawn DeadMotion presentation");
    assert_eq!(fusion_spawn.source_clip_path_id, 1_126);
    assert_eq!(fusion_spawn.event_seconds, 0.6);
    assert_eq!(fusion_spawn.effect_id, 372);
    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets")
        .join("game");
    let assets = crate::assets::AssetLocator::open(asset_root).unwrap();
    let content =
        crate::tutorial_mission_content::TutorialMissionContent::open(&assets).unwrap();
    let particle_scale = content.gameplay_npc(2674).unwrap().radius() * 2.0;
    assert!(
        (particle_scale - 1.2).abs() < 1.0e-6,
        "DeadMotion.SetParticleScale uses the NPC controller radius"
    );

    assert_ne!(
        retrobution_actor_current_effect_name(204, 734),
        retrobution_actor_current_effect_name(205, 734),
        "each actor owns an independent currentEffect slot"
    );
    assert_ne!(
        retrobution_actor_current_effect_name(203, 38),
        retrobution_actor_current_effect_name(203, 751),
        "different prefabs do not destroy AnimationEventHandler.currentEffect"
    );
}

#[test]
fn actor_effect_event_crossings_follow_the_real_repeating_clip_clock() {
    let event = 0.18;
    assert_eq!(
        animation_event_crossings(0.10, 0, 0.17, 0, RepeatAnimation::Forever, event),
        0
    );
    assert_eq!(
        animation_event_crossings(0.17, 0, 0.19, 0, RepeatAnimation::Forever, event),
        1
    );
    assert_eq!(
        animation_event_crossings(0.90, 0, 0.05, 1, RepeatAnimation::Forever, event),
        0
    );
    assert_eq!(
        animation_event_crossings(0.10, 0, 0.25, 1, RepeatAnimation::Forever, event),
        2
    );
    assert_eq!(
        animation_event_crossings(0.10, 2, 0.25, 5, RepeatAnimation::Forever, event),
        4
    );
    assert_eq!(
        animation_event_crossings(0.10, 0, 1.16, 1, RepeatAnimation::Never, event),
        1
    );
}

#[test]
fn exact_bone_attachment_is_preserved_in_the_command_record() {
    let mut runtime = TutorialEffectRuntime::with_library(library());
    runtime.enqueue(TutorialEffectRuntimeCommand::Add {
        effect_id: 742,
        placement: TutorialEffectPlacement::ExactBone {
            actor_id: 100,
            node_name: "Bip01 L Hand".to_owned(),
            spawn_world_rotation: Quat::from_rotation_x(90_f32.to_radians()),
            local_translation_after_parenting: Vec3::ZERO,
            local_rotation_after_parenting: Quat::from_rotation_x(90_f32.to_radians()),
        },
        scale: 1.0,
        tracked: false,
        name: Some("Numbuh Five handoff particle 742".to_owned()),
        destroy_after_seconds: Some(1.3),
        source_line: 3071,
    });
    runtime.process_pending();
    let record = runtime.drain_records().next().unwrap();
    assert!(matches!(
        record.command,
        TutorialEffectRuntimeCommand::Add {
            placement: TutorialEffectPlacement::ExactBone {
                actor_id: 100,
                ref node_name,
                ..
            },
            ..
        } if node_name == "Bip01 L Hand"
    ));
}

#[test]
fn exact_entity_bone_attachment_is_preserved_in_the_command_record() {
    let root_entity = Entity::from_bits(42);
    let legacy_tag_rotation = Quat::from_rotation_x(90_f32.to_radians());
    let mut runtime = TutorialEffectRuntime::with_library(library());
    runtime.enqueue(TutorialEffectRuntimeCommand::Add {
        effect_id: 742,
        placement: TutorialEffectPlacement::ExactEntityBone {
            root_entity,
            node_name: "Bip01 Footsteps".to_owned(),
            spawn_world_rotation: legacy_tag_rotation,
            local_translation_after_parenting: Vec3::ZERO,
            local_rotation_after_parenting: legacy_tag_rotation,
        },
        scale: 1.0,
        tracked: false,
        name: Some("Buttercup skill particle 60".to_owned()),
        destroy_after_seconds: None,
        source_line: 281,
    });

    runtime.process_pending();

    let record = runtime.drain_records().next().unwrap();
    assert!(matches!(
        record.command,
        TutorialEffectRuntimeCommand::Add {
            placement: TutorialEffectPlacement::ExactEntityBone {
                root_entity: event_root,
                ref node_name,
                ..
            },
            ..
        } if event_root == root_entity && node_name == "Bip01 Footsteps"
    ));
}
