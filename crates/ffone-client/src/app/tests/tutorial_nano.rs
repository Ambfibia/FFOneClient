use super::*;
use ffone_client::tutorial_nano_gameplay::TutorialNanoGameplayIssueQueue;

fn nano_grant_app() -> App {
    let mut app = App::new();
    app.add_plugins(TutorialNanoGameplayPlugin);
    app
}

fn flush_nano_grant(app: &mut App, queue: TutorialNanoGameplayCommandQueue) {
    app.insert_resource(queue);
    ffone_client::tutorial_nano_gameplay::apply_tutorial_nano_gameplay_commands(app.world_mut());
}

#[test]
fn tutorial_nano_natural_and_skipped_creation_commit_both_grant_owners() {
    for completion in [SceneCompletion::Natural, SceneCompletion::Skipped] {
        for timeline_first in [false, true] {
            let mut app = nano_grant_app();
            let mut runtime = RuntimeStatus::default();
            let mut queue = TutorialNanoGameplayCommandQueue::default();
            let decision = ffone_client::tutorial_logic::evaluate_stage(
                TutorialStage::Infection(InfectionStage::NanoCreation),
                &TutorialObservation {
                    scene_completion: completion,
                    ..default()
                },
            )
            .unwrap();
            let grant = decision
                .intents
                .iter()
                .find_map(|intent| match intent {
                    TutorialIntent::EquipNano {
                        slot,
                        nano_id,
                        skill_id,
                    } => Some((*slot, *nano_id, *skill_id)),
                    _ => None,
                })
                .expect("scene completion must grant Buttercup");
            let timeline = tutorial_scene_choreography(TutorialScene::InfectionC).unwrap();
            let actions: Vec<_> = if completion == SceneCompletion::Skipped {
                timeline
                    .skip
                    .scene_specific
                    .iter()
                    .map(|event| event.action)
                    .collect()
            } else {
                timeline.actions.iter().map(|event| event.action).collect()
            };
            let action = actions
                .into_iter()
                .find_map(|action| match action {
                    ChoreographyAction::Nano(NanoAction::Equip {
                        slot,
                        nano_id,
                        skill_id,
                        stamina,
                    }) => Some((slot, nano_id, skill_id, stamina)),
                    _ => None,
                })
                .expect("timeline grant");
            for from_timeline in [timeline_first, !timeline_first] {
                let (slot, id, skill, stamina) = if from_timeline {
                    (action.0 as usize, action.1, action.2, action.3)
                } else {
                    (grant.0 as usize, grant.1 as i16, grant.2 as i16, 100)
                };
                assert!(equip_tutorial_nano(
                    &mut runtime,
                    &mut queue,
                    slot,
                    id,
                    skill,
                    stamina
                ));
            }
            flush_nano_grant(&mut app, queue);
            assert_eq!(runtime.nano_slots[0].nano_id, Some(1));
            assert_eq!(runtime.nano_slots[0].skill_id, 1);
            let nano = app.world().resource::<TutorialNanoGameplayState>();
            assert_eq!(
                nano.loadout(),
                Some(TutorialNanoGameplayLoadout {
                    nano_id: 1,
                    skill_id: 1
                })
            );
            assert_eq!(nano.stamina(), 100);
            assert!(nano.world_presentation().is_none());
            assert_eq!(
                decision.next_stage,
                TutorialStage::Infection(InfectionStage::ExitPrompt)
            );
        }
    }
}

#[test]
fn tutorial_nano_repairs_missing_slot_or_gameplay_loadout_at_every_post_grant_stage() {
    let stages = [
        TutorialStage::Infection(InfectionStage::ExitPrompt),
        TutorialStage::Infection(InfectionStage::WarpOut),
        TutorialStage::NanoPower(NanoPowerStage::AwaitCollapse),
        TutorialStage::NanoPower(NanoPowerStage::CollapseCutscene),
        TutorialStage::NanoPower(NanoPowerStage::CrossBridge),
        TutorialStage::NanoPower(NanoPowerStage::DemoMonsterCutscene),
        TutorialStage::NanoPower(NanoPowerStage::SummonNano),
        TutorialStage::NanoPower(NanoPowerStage::UseNanoPower),
        TutorialStage::NanoPower(NanoPowerStage::ReturnToNumbuhTwo),
        TutorialStage::NanoPower(NanoPowerStage::FinaleCutscene),
    ];
    for stage in stages {
        for slot_only in [false, true] {
            let mut app = nano_grant_app();
            let mut runtime = RuntimeStatus::default();
            if slot_only {
                runtime.nano_slots[0] = RuntimeNanoSlot {
                    nano_id: Some(1),
                    skill_id: 1,
                    stamina: 0,
                    active: true,
                };
            }
            let mut queue = TutorialNanoGameplayCommandQueue::default();
            ensure_tutorial_nano_loadout(
                stage,
                &mut runtime,
                app.world().resource::<TutorialNanoGameplayState>(),
                &mut queue,
            );
            flush_nano_grant(&mut app, queue);
            assert_eq!(
                runtime.nano_slots[0],
                RuntimeNanoSlot {
                    nano_id: Some(1),
                    skill_id: 1,
                    stamina: 100,
                    active: false
                },
                "{stage:?}"
            );
            runtime.nano_slots[0] = RuntimeNanoSlot::default();
            let mut queue = TutorialNanoGameplayCommandQueue::default();
            ensure_tutorial_nano_loadout(
                stage,
                &mut runtime,
                app.world().resource::<TutorialNanoGameplayState>(),
                &mut queue,
            );
            assert!(queue.is_empty());
            assert_eq!(runtime.nano_slots[0].nano_id, Some(1));
            assert_eq!(runtime.nano_slots[0].stamina, 100);
        }
    }
}

#[test]
fn tutorial_nano_is_not_granted_before_creation() {
    let mut runtime = RuntimeStatus::default();
    let mut queue = TutorialNanoGameplayCommandQueue::default();
    for stage in [
        TutorialStage::Movement(MovementStage::AwaitIntro),
        TutorialStage::Infection(InfectionStage::FightFusion),
        TutorialStage::Infection(InfectionStage::NanoCreation),
    ] {
        ensure_tutorial_nano_loadout(
            stage,
            &mut runtime,
            &TutorialNanoGameplayState::default(),
            &mut queue,
        );
        assert_eq!(runtime.nano_slots, [RuntimeNanoSlot::default(); 3]);
        assert!(queue.is_empty());
    }
}

#[test]
fn tutorial_nano_recovery_and_first_keypress_in_one_frame_equip_before_summon() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        TutorialNanoGameplayPlugin,
    ))
    .init_asset::<Gltf>()
    .init_asset::<WorldAsset>();
    let owner = app.world_mut().spawn(Transform::IDENTITY).id();
    let mut runtime = RuntimeStatus::default();
    let mut queue = TutorialNanoGameplayCommandQueue::default();
    ensure_tutorial_nano_loadout(
        TutorialStage::NanoPower(NanoPowerStage::SummonNano),
        &mut runtime,
        app.world().resource::<TutorialNanoGameplayState>(),
        &mut queue,
    );
    assert_eq!(
        tutorial_nano_shortcut_action(
            false,
            runtime.nano_slots[0].nano_id.is_some(),
            runtime.nano_slots[0].active,
            false
        ),
        Some(TutorialNanoShortcutAction::Summon)
    );
    queue.summon(owner);
    flush_nano_grant(&mut app, queue);
    let nano = app.world().resource::<TutorialNanoGameplayState>();
    assert_eq!(
        nano.loadout(),
        Some(TutorialNanoGameplayLoadout {
            nano_id: 1,
            skill_id: 1
        })
    );
    assert_eq!(nano.owner(), Some(owner));
    assert!(nano.entity().is_some());
    assert!(
        !nano.is_active(),
        "asynchronous model loading must not complete the summon lesson"
    );
    let mut tutorial = TutorialSession::default();
    let (chapter, step) = TutorialStage::NanoPower(NanoPowerStage::SummonNano).legacy();
    tutorial.init_chapter(chapter).unwrap();
    tutorial.init_step(step);
    reconcile_tutorial_nano_activation(&mut tutorial, nano);
    assert_eq!(tutorial.progress.event_value(TutorialEvent::NanoActive), 0);
    assert!(
        app.world_mut()
            .resource_mut::<TutorialNanoGameplayIssueQueue>()
            .take_all()
            .is_empty()
    );
}

#[test]
fn tutorial_nano_missing_or_dismissed_cannot_leave_a_stale_stun_prompt() {
    let mut tutorial = TutorialSession::default();
    let (chapter, step) = TutorialStage::NanoPower(NanoPowerStage::SummonNano).legacy();
    tutorial.init_chapter(chapter).unwrap();
    tutorial.init_step(step);
    tutorial
        .progress
        .receive_event(TutorialEvent::NanoActive, 1);
    reconcile_tutorial_nano_activation(&mut tutorial, &TutorialNanoGameplayState::default());
    assert_eq!(tutorial.progress.event_value(TutorialEvent::NanoActive), 0);
    assert_eq!(
        evaluate_progress(
            &tutorial.progress,
            &TutorialObservation::default().with_progress(&tutorial.progress)
        )
        .unwrap()
        .next_stage,
        TutorialStage::NanoPower(NanoPowerStage::SummonNano)
    );
    tutorial.init_step(NanoPowerStage::UseNanoPower as i16);
    reconcile_tutorial_nano_activation(&mut tutorial, &TutorialNanoGameplayState::default());
    assert_eq!(
        tutorial.progress.stage(),
        Some(TutorialStage::NanoPower(NanoPowerStage::SummonNano))
    );
    tutorial.init_step(NanoPowerStage::UseNanoPower as i16);
    tutorial.progress.receive_event(TutorialEvent::UseSkill, 1);
    reconcile_tutorial_nano_activation(&mut tutorial, &TutorialNanoGameplayState::default());
    assert_eq!(
        evaluate_progress(
            &tutorial.progress,
            &TutorialObservation::default().with_progress(&tutorial.progress)
        )
        .unwrap()
        .next_stage,
        TutorialStage::NanoPower(NanoPowerStage::ReturnToNumbuhTwo)
    );
}

#[test]
fn tutorial_nano_shard_mutations_are_routed_away_from_the_local_grant() {
    for packet_type in [
        packet::P_FE2CL_REP_NANO_ACTIVE_SUCC,
        packet::P_FE2CL_NANO_SKILL_USE_SUCC,
        packet::P_FE2CL_NANO_SKILL_USE,
        packet::P_FE2CL_REP_NANO_BOOK_SUBSET,
        packet::P_FE2CL_REP_NANO_EQUIP_SUCC,
        packet::P_FE2CL_REP_NANO_UNEQUIP_SUCC,
        packet::P_FE2CL_REP_NANO_TUNE_SUCC,
        packet::P_FE2CL_REP_NANO_TUNE_FAIL,
        packet::P_FE2CL_REP_PC_NANO_CREATE_SUCC,
        packet::P_FE2CL_REP_PC_NANO_CREATE_FAIL,
        packet::P_FE2CL_NPC_SKILL_HIT,
        packet::P_FE2CL_NPC_SKILL_CORRUPTION_HIT,
        packet::P_FE2CL_REP_PC_REGEN_SUCC,
        packet::P_FE2CL_PC_SUDDEN_DEAD,
    ] {
        assert!(network_ingress::tutorial_owns_local_gameplay_frame(
            true,
            packet_type
        ));
        assert!(!network_ingress::tutorial_owns_local_gameplay_frame(
            false,
            packet_type
        ));
    }
    assert!(!network_ingress::tutorial_owns_local_gameplay_frame(
        true,
        packet::P_FE2CL_REP_PC_TICK
    ));
}

#[test]
fn tutorial_nano_stale_activation_does_not_erase_a_simultaneous_death_or_timer() {
    let mut tutorial = TutorialSession::default();
    tutorial
        .init_chapter(
            TutorialStage::NanoPower(NanoPowerStage::SummonNano)
                .legacy()
                .0,
        )
        .unwrap();
    tutorial.init_step(NanoPowerStage::SummonNano as i16);
    tutorial
        .progress
        .receive_event(TutorialEvent::NanoActive, 1);
    tutorial.progress.receive_event(TutorialEvent::DeadNpc, 1);
    tutorial.progress.set_timer_seconds(7);
    reconcile_tutorial_nano_activation(&mut tutorial, &TutorialNanoGameplayState::default());
    assert_eq!(tutorial.progress.event_value(TutorialEvent::NanoActive), 0);
    assert_eq!(tutorial.progress.event_value(TutorialEvent::DeadNpc), 1);
    assert_eq!(tutorial.progress.timer_seconds(), 7);
}

#[test]
fn infection_reveal_sets_new_nano_to_camera_yaw_plus_half_turn() {
    let infection = tutorial_scene_choreography(TutorialScene::InfectionC).unwrap();

    assert!(infection.actions.iter().any(|action| {
        action.at_seconds == 3.5
            && action.source_line == 4065
            && action.action
                == ChoreographyAction::Npc(NpcAction::SetRotation {
                    id: -1,
                    rotation: RotationExpr::EntityYaw {
                        entity: EntityRef::Camera,
                        pitch: 0.0,
                        yaw_offset: 180.0,
                        roll: 0.0,
                    },
                })
    }));
}

#[test]
fn infection_uses_the_exact_event_nano_animation_methods() {
    let infection = tutorial_scene_choreography(TutorialScene::InfectionC).unwrap();
    let action_at = |seconds, expected| {
        infection
            .actions
            .iter()
            .any(|action| action.at_seconds == seconds && action.action == expected)
    };

    assert!(action_at(
        3.5,
        ChoreographyAction::Nano(NanoAction::Emote("call2"))
    ));
    assert!(action_at(4.3, ChoreographyAction::Nano(NanoAction::Happy)));
    assert!(action_at(8.9, ChoreographyAction::Nano(NanoAction::Call)));
    assert!(action_at(
        15.9,
        ChoreographyAction::Nano(NanoAction::Emote("happy"))
    ));
}

#[test]
fn new_nano_camera_formulas_use_rendered_model_rotation() {
    let model_rotation = Quat::from_rotation_y(0.73 + std::f32::consts::PI);
    let root_rotation = choreography_character_root_rotation(model_rotation);
    let root = Transform::from_xyz(4.0, 5.0, 6.0).with_rotation(root_rotation);

    let choreography = tutorial_nano_choreography_transform(root);
    let rendered_forward = root_rotation * native_model_forward_child_rotation() * Vec3::Z;
    let camera_offset = choreography.rotation * Vec3::Z * 1.5;

    assert!((choreography.rotation * Vec3::Z).abs_diff_eq(model_rotation * Vec3::Z, 1.0e-6));
    assert!(rendered_forward.abs_diff_eq(model_rotation * Vec3::Z, 1.0e-6));
    assert!(camera_offset.abs_diff_eq(rendered_forward * 1.5, 1.0e-6));
}
