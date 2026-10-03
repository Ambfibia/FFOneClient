//! Tutorial avatar actions, Nano gameplay events and actor events.

use super::dexter_ship_drive::dexter_ship_descendant;
use super::runtime_status::RuntimeStatus;
use super::state::ClientState;
use super::tutorial_choreography::*;
use super::tutorial_session::{
    TutorialMissionRuntime, TutorialProjectileRandomStream, TutorialSession,
};
use super::world_combat::tutorial_weapon_combat_profile;
use super::{LocalPlayer, WorldSliceEntity};
use bevy::{audio::Volume, prelude::*};
use ffone_client::{
    avatar_action::{
        LegacyAvatarActionIntent, LegacyAvatarActionQueue, LegacyAvatarActionState,
        LegacyVisualClip,
    },
    gameplay_audio::GameplayAudioRuntime,
    legacy_environment::LegacyAvatarEnvironmentState,
    localization::{LocalizedVoice, VoiceLanguage},
    semantic_audio::{NativeAudioCatalog, NativeAudioCategory},
    tutorial::{MissionStage, TutorialEvent, TutorialProgress, TutorialStage},
    tutorial_actors::{
        MISSION_TARGET_ID, TutorialActor, TutorialActorCommandQueue, TutorialActorEvent,
        TutorialActorEventQueue, TutorialActorIssueQueue, tutorial_virtual_server_damage,
    },
    tutorial_choreography_runtime::TutorialChoreographyIssueQueue,
    tutorial_effects_runtime::{
        TutorialEffectPlacement, TutorialEffectRuntime, TutorialEffectRuntimeCommand,
        TutorialProjectileMotion,
    },
    tutorial_mission_content::TutorialMissionContent,
    tutorial_nano_gameplay::{
        TutorialNanoGameplayAudioCategory, TutorialNanoGameplayCommandQueue,
        TutorialNanoGameplayEvent, TutorialNanoGameplayEventQueue, TutorialNanoGameplayIssue,
        TutorialNanoGameplayIssueQueue,
    },
    tutorial_player_presentation::PlayerWeaponAnimationCatalog,
    tutorial_player_rig_runtime::{
        TutorialPlayerRigIssueQueue, TutorialSelectedPlayerRig, TutorialSelectedPlayerRigActive,
    },
};

pub(super) fn collect_tutorial_avatar_actions(
    state: Res<State<ClientState>>,
    mut actions: ResMut<LegacyAvatarActionQueue>,
    mut tutorial: ResMut<TutorialSession>,
    runtime: Res<RuntimeStatus>,
    content: Res<TutorialMissionContent>,
    actors: Query<&TutorialActor>,
    transforms: Query<&GlobalTransform>,
    rigs: Query<&TutorialSelectedPlayerRig, With<TutorialSelectedPlayerRigActive>>,
    mut action_states: Query<
        (&LegacyAvatarActionState, &mut LegacyAvatarEnvironmentState),
        With<LocalPlayer>,
    >,
    parents: Query<&ChildOf>,
    named_transforms: Query<(Entity, &Name, &GlobalTransform)>,
    mut actor_commands: ResMut<TutorialActorCommandQueue>,
    mut gameplay_nano_commands: ResMut<TutorialNanoGameplayCommandQueue>,
    mut effect_runtime: ResMut<TutorialEffectRuntime>,
    mut gameplay_audio: ResMut<GameplayAudioRuntime>,
    weapon_catalog: Res<PlayerWeaponAnimationCatalog>,
) {
    if *state.get() != ClientState::Tutorial {
        return;
    }
    if tutorial.completion_requested {
        actions.take_all();
        return;
    }
    let pending = actions.take_all();
    let attacker_style = tutorial_active_nano_style(&runtime);
    for request in pending {
        match request.intent {
            LegacyAvatarActionIntent::PrimaryAttack { targets, .. } => {
                if let Ok((_, mut environment)) = action_states.get_mut(request.actor) {
                    environment.observe_local_combat();
                }
                let source_transform = transforms.get(request.actor).ok();
                let source_root = source_transform
                    .map(GlobalTransform::translation)
                    .unwrap_or(Vec3::ZERO);
                let weapon = tutorial_weapon_combat_profile(runtime.tutorial_weapon_id);
                if let Some(rig) = rigs.iter().find(|rig| rig.controller_root == request.actor) {
                    let item_id = rig.attached_weapon_item_id().or_else(|| {
                        runtime
                            .tutorial_weapon_id
                            .and_then(|item_id| i16::try_from(item_id).ok())
                    });
                    if let Some(item_id) = item_id {
                        gameplay_audio.queue_player_weapon_attack(
                            request.actor,
                            rig.gender,
                            item_id,
                            runtime.weapon_battery,
                            &weapon_catalog,
                        );
                    } else {
                        let full_body = action_states.get(request.actor).is_ok_and(|(state, _)| {
                            matches!(state.base_action(), Some(LegacyVisualClip::AttackFull(_)))
                        });
                        gameplay_audio.queue_player_unarmed_attack(
                            request.actor,
                            rig.gender,
                            full_body,
                        );
                    }
                }
                // CharacterController center is the exact fallback used by
                // GetConeList when no authored weapon FireLink is available.
                let source = weapon
                    .fire_link
                    .and_then(|fire_link| {
                        tutorial_weapon_fire_link_source(
                            request.actor,
                            fire_link,
                            &rigs,
                            &parents,
                            &named_transforms,
                        )
                    })
                    .unwrap_or(source_root + Vec3::Y * 0.8);
                if targets.is_empty()
                    && let (Some(bullet_type), Some(source_transform)) =
                        (weapon.bullet_type, source_transform)
                {
                    let mut forward = source_transform.rotation() * Vec3::NEG_Z;
                    forward.y = 0.0;
                    forward = forward.try_normalize().unwrap_or(Vec3::NEG_Z);
                    // cnAvatarAttack.MakeBullet(null) shoots to the equipped
                    // attack range and adds one Unity unit to target Y.
                    effect_runtime.enqueue(TutorialEffectRuntimeCommand::Projectile {
                        bullet_type,
                        source,
                        target: source_root + forward * weapon.range + Vec3::Y,
                        target_exists: false,
                        source_style: attacker_style,
                        target_style: -1,
                        motion: TutorialProjectileMotion::BulletMove,
                        source_line: line!(),
                    });
                }
                for target in targets {
                    let Ok(actor) = actors.get(target.entity) else {
                        continue;
                    };
                    if actor.team == 1 || !actor.is_alive() {
                        continue;
                    }
                    let Some(definition) = content.gameplay_npc(actor.npc_type) else {
                        continue;
                    };
                    if let Some(bullet_type) = weapon.bullet_type
                        && let Ok(target_transform) = transforms.get(target.entity)
                    {
                        effect_runtime.enqueue(TutorialEffectRuntimeCommand::Projectile {
                            bullet_type,
                            source,
                            target: target_transform.translation()
                                + Vec3::Y * (definition.height() * 0.5),
                            target_exists: true,
                            source_style: attacker_style,
                            target_style: definition.npc_style,
                            motion: TutorialProjectileMotion::BulletMove,
                            source_line: line!(),
                        });
                    }
                    let damage =
                        tutorial_virtual_server_damage(attacker_style, definition.npc_style, false);
                    if damage > 0 {
                        actor_commands.damage(actor.id, damage);
                    }
                }
            }
            LegacyAvatarActionIntent::TalkNpc(entity) => {
                if let Ok(actor) = actors.get(entity)
                    && actor.is_alive()
                {
                    actor_commands.set_interacting(actor.id, true);
                    if let Ok(player) = transforms.get(request.actor) {
                        // `NpcIconMode.InitMode` calls SetForceAngle(player)
                        // before binding its fixed sub-target camera.
                        actor_commands.face_native_position(actor.id, player.translation());
                    }
                }
            }
            LegacyAvatarActionIntent::WeaponCycle => tutorial
                .progress
                .receive_event(TutorialEvent::ChangeWeapon, 1),
            LegacyAvatarActionIntent::NanoSkill { targets } => {
                gameplay_nano_commands.use_skill(request.actor, targets);
            }
            _ => {}
        }
    }
}

pub(super) fn tutorial_weapon_fire_link_source(
    actor: Entity,
    fire_link: &str,
    rigs: &Query<&TutorialSelectedPlayerRig, With<TutorialSelectedPlayerRigActive>>,
    parents: &Query<&ChildOf>,
    named_transforms: &Query<(Entity, &Name, &GlobalTransform)>,
) -> Option<Vec3> {
    let weapon_root = rigs
        .iter()
        .find(|rig| rig.controller_root == actor)?
        .attached_weapon()?;
    named_transforms
        .iter()
        .find(|(entity, name, _)| {
            name.as_str() == fire_link
                && (*entity == weapon_root || dexter_ship_descendant(*entity, weapon_root, parents))
        })
        .map(|(_, _, transform)| transform.translation())
}

pub(super) fn collect_tutorial_nano_gameplay_events(
    mut commands: Commands,
    client_state: Res<State<ClientState>>,
    asset_server: Res<AssetServer>,
    audio_catalog: Res<NativeAudioCatalog>,
    voice_language: Res<VoiceLanguage>,
    mut events: ResMut<TutorialNanoGameplayEventQueue>,
    mut issues: ResMut<TutorialNanoGameplayIssueQueue>,
    mut effects: ResMut<TutorialEffectRuntime>,
    mut tutorial: ResMut<TutorialSession>,
    mut runtime: ResMut<RuntimeStatus>,
) {
    let tutorial_active = *client_state.get() == ClientState::Tutorial;
    for event in events.take_all() {
        match event {
            TutorialNanoGameplayEvent::EffectRequested {
                effect_id,
                position,
                rotation,
                source_line,
            } => {
                effects.enqueue(TutorialEffectRuntimeCommand::Add {
                    effect_id,
                    placement: TutorialEffectPlacement::World { position, rotation },
                    scale: 1.0,
                    tracked: false,
                    name: None,
                    destroy_after_seconds: None,
                    source_line,
                });
            }
            TutorialNanoGameplayEvent::TaggedEffectRequested {
                effect_id,
                root,
                node_name,
                source_clip_path_id,
                source_event_seconds,
            } => {
                let legacy_tag_rotation = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
                effects.enqueue(TutorialEffectRuntimeCommand::Add {
                    effect_id,
                    placement: TutorialEffectPlacement::ExactEntityBone {
                        root_entity: root,
                        node_name: node_name.to_owned(),
                        spawn_world_rotation: legacy_tag_rotation,
                        local_translation_after_parenting: Vec3::ZERO,
                        local_rotation_after_parenting: legacy_tag_rotation,
                    },
                    scale: 1.0,
                    tracked: false,
                    name: Some(format!(
                        "Nano AnimationClip#{source_clip_path_id} tag_p at {source_event_seconds:.2}s"
                    )),
                    destroy_after_seconds: None,
                    // `AnimationEventHandler.tag_p` in the clean managed DLL.
                    source_line: 281,
                });
            }
            TutorialNanoGameplayEvent::Activated {
                nano_id,
                skill_id,
                stamina,
                ..
            } => {
                // In World this event only confirms that the local model
                // finished loading. Nano slot ownership remains server-owned.
                if !tutorial_active {
                    continue;
                }
                let Ok(stamina) = i16::try_from(stamina) else {
                    runtime.message =
                        "Gameplay Nano activation rejected: stamina does not fit protocol i16"
                            .into();
                    continue;
                };
                for slot in &mut runtime.nano_slots {
                    slot.active = false;
                }
                if let Some(slot) = runtime
                    .nano_slots
                    .iter_mut()
                    .find(|slot| slot.nano_id == Some(nano_id) && slot.skill_id == skill_id)
                {
                    slot.active = true;
                    slot.stamina = stamina;
                    tutorial
                        .progress
                        .receive_event(TutorialEvent::NanoActive, 1);
                } else {
                    runtime.message = format!(
                        "Gameplay Nano activation rejected: loadout ({nano_id}, {skill_id}) is not equipped"
                    );
                }
            }
            TutorialNanoGameplayEvent::Dismissed { .. } => {
                if tutorial_active {
                    for slot in &mut runtime.nano_slots {
                        slot.active = false;
                    }
                }
            }
            TutorialNanoGameplayEvent::DamageNpc { .. } => {
                if tutorial_active {
                    tutorial.progress.receive_event(TutorialEvent::DamageNpc, 1);
                }
            }
            TutorialNanoGameplayEvent::UseSkill {
                skill_id, stamina, ..
            } => {
                if !tutorial_active {
                    continue;
                }
                let Ok(stamina) = i16::try_from(stamina) else {
                    runtime.message =
                        "Gameplay Nano skill result rejected: stamina does not fit protocol i16"
                            .into();
                    continue;
                };
                if let Some(slot) = runtime
                    .nano_slots
                    .iter_mut()
                    .find(|slot| slot.active && slot.skill_id == skill_id)
                {
                    slot.stamina = stamina;
                }
                tutorial.progress.receive_event(TutorialEvent::UseSkill, 1);
            }
            TutorialNanoGameplayEvent::AudioRequested {
                true_name,
                category,
                position,
                source_clip_path_id,
                source_event_seconds,
            } => match tutorial_nano_audio_path(
                &audio_catalog,
                &voice_language.effective,
                true_name,
                category,
            ) {
                Ok(Some(path)) => {
                    let mut entity = commands.spawn((
                        Name::new(format!("Buttercup Nano animation audio {true_name}")),
                        WorldSliceEntity,
                        Transform::from_translation(position),
                        AudioPlayer::new(asset_server.load(path)),
                        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(1.0)),
                    ));
                    if category == TutorialNanoGameplayAudioCategory::Voice {
                        entity.insert(LocalizedVoice::by_true_name(true_name));
                    }
                }
                Ok(None) => {}
                Err(error) => {
                    runtime.message = format!(
                        "Buttercup Nano audio from AnimationClip pathId {source_clip_path_id} at {source_event_seconds:.2}s failed: {error}"
                    );
                }
            },
        }
    }
    for issue in issues.take_all() {
        runtime.message = match issue {
            TutorialNanoGameplayIssue::UnsupportedLoadout { nano_id, skill_id } => {
                format!("Gameplay Nano blocked: unsupported loadout ({nano_id}, {skill_id})")
            }
            TutorialNanoGameplayIssue::MissingOwner { owner } => {
                format!("Gameplay Nano blocked: owner {owner:?} is unavailable")
            }
            TutorialNanoGameplayIssue::AssetBlocked(error) => {
                format!("Gameplay Nano asset blocked: {error}")
            }
        };
    }
}

pub(super) fn tutorial_nano_audio_path(
    catalog: &NativeAudioCatalog,
    voice_locale: &str,
    true_name: &str,
    category: TutorialNanoGameplayAudioCategory,
) -> Result<Option<String>, String> {
    let expected = match category {
        TutorialNanoGameplayAudioCategory::Sfx => NativeAudioCategory::Sfx,
        TutorialNanoGameplayAudioCategory::Voice => NativeAudioCategory::Voice,
    };
    let candidates = catalog
        .by_true_name(true_name)
        .into_iter()
        .filter(|asset| asset.category == expected)
        .collect::<Vec<_>>();
    let [asset] = candidates.as_slice() else {
        return Err(format!(
            "true name {true_name:?} resolved to {} {:?} assets; expected exactly one",
            candidates.len(),
            expected
        ));
    };
    Ok(if category == TutorialNanoGameplayAudioCategory::Voice {
        catalog
            .path_for_locale(asset, voice_locale)
            .map(str::to_owned)
    } else {
        Some(asset.path.clone())
    })
}

pub(super) fn report_tutorial_runtime_issues(
    mut runtime: ResMut<RuntimeStatus>,
    mut choreography: ResMut<TutorialChoreographyIssueQueue>,
    mut effects: ResMut<TutorialEffectRuntime>,
    mut actors: ResMut<TutorialActorIssueQueue>,
    mut player_rig: ResMut<TutorialPlayerRigIssueQueue>,
) {
    const MAX_VISIBLE_ISSUES: usize = 4;
    const MAX_RETAINED_ISSUES: usize = 256;

    let mut messages = Vec::new();
    messages.extend(
        choreography
            .take_all()
            .into_iter()
            .map(|issue| format!("choreography={issue:?}")),
    );
    messages.extend(
        effects
            .drain_issues()
            .map(|issue| format!("effect={issue:?}")),
    );
    messages.extend(
        actors
            .take_all()
            .into_iter()
            .map(|issue| format!("actor={issue:?}")),
    );
    messages.extend(
        player_rig
            .take_all()
            .into_iter()
            .map(|issue| format!("player-rig={issue:?}")),
    );

    if messages.is_empty() {
        return;
    }
    for message in &messages {
        warn!("Tutorial native issue: {message}");
        runtime
            .diagnostics
            .tutorial_issue_history
            .push_back(message.clone());
        while runtime.diagnostics.tutorial_issue_history.len() > MAX_RETAINED_ISSUES {
            runtime.diagnostics.tutorial_issue_history.pop_front();
        }
    }
    let hidden = messages.len().saturating_sub(MAX_VISIBLE_ISSUES);
    messages.truncate(MAX_VISIBLE_ISSUES);
    runtime.message = if hidden == 0 {
        format!("Tutorial native issue: {}", messages.join(" | "))
    } else {
        format!(
            "Tutorial native issue: {} | +{hidden} more",
            messages.join(" | ")
        )
    };
}

pub(super) fn tutorial_active_nano_style(runtime: &RuntimeStatus) -> i32 {
    runtime
        .nano_slots
        .iter()
        .find(|slot| slot.active)
        .and_then(|slot| slot.nano_id)
        .map(|nano_id| match nano_id {
            // NanoTableElement 1 (Buttercup), used by the tutorial.
            1 => 1,
            _ => -1,
        })
        .unwrap_or(-1)
}

pub(super) fn active_world_nano_style(runtime: &RuntimeStatus, content: &TutorialMissionContent) -> i32 {
    runtime
        .nano_slots
        .iter()
        .find(|slot| slot.active)
        .and_then(|slot| slot.nano_id)
        .and_then(|nano_id| content.gameplay_nano(nano_id))
        .map_or(-1, |nano| i32::from(nano.style))
}

pub(super) fn apply_tutorial_player_damage(current_hp: i32, damage: i32) -> i32 {
    (current_hp - damage.max(0)).max(100)
}

pub(super) fn apply_tutorial_actor_event(
    event: TutorialActorEvent,
    progress: &mut TutorialProgress,
    player_hp: &mut Option<i32>,
    mission_reward_advanced: bool,
) {
    match event {
        TutorialActorEvent::Damaged { .. } => {
            progress.receive_event(TutorialEvent::DamageNpc, 1);
        }
        TutorialActorEvent::Dead { id, .. } => {
            let mission_objective_was_active = matches!(
                progress.stage(),
                Some(TutorialStage::Mission(MissionStage::ObjectiveCombat))
            );
            progress.receive_event(TutorialEvent::DeadNpc, 1);
            // The original reward packet advances the accepted task, which in
            // turn re-emits tutorial event 28. Preserve that mission state
            // change without advancing the chapter directly.
            if mission_objective_was_active && id == MISSION_TARGET_ID && mission_reward_advanced {
                progress.receive_event(TutorialEvent::TaskStart, 1);
            }
        }
        TutorialActorEvent::AttackedPlayer { damage, .. } => {
            let current = player_hp.unwrap_or(1_000);
            *player_hp = Some(apply_tutorial_player_damage(current, damage));
            progress.receive_event(TutorialEvent::DamageUser, 1);
        }
        TutorialActorEvent::Interaction { .. }
        | TutorialActorEvent::PlayerKillDeathPresentation { .. } => {}
    }
}

pub(super) fn collect_tutorial_actor_events(
    mut events: ResMut<TutorialActorEventQueue>,
    mut tutorial: ResMut<TutorialSession>,
    mut runtime: ResMut<RuntimeStatus>,
    content: Res<TutorialMissionContent>,
    mut mission_runtime: ResMut<TutorialMissionRuntime>,
    actors: Query<(&TutorialActor, &GlobalTransform)>,
    rigs: Query<&TutorialSelectedPlayerRig, With<TutorialSelectedPlayerRigActive>>,
    local_players: Query<(Entity, &GlobalTransform), With<LocalPlayer>>,
    parents: Query<&ChildOf>,
    named_transforms: Query<(Entity, &Name, &GlobalTransform)>,
    mut effect_runtime: ResMut<TutorialEffectRuntime>,
    mut projectile_random: ResMut<TutorialProjectileRandomStream>,
    mut gameplay_audio: ResMut<GameplayAudioRuntime>,
    mut player_environments: Query<&mut LegacyAvatarEnvironmentState, With<LocalPlayer>>,
) {
    for event in events.take_all() {
        if matches!(
            event,
            TutorialActorEvent::AttackedPlayer { .. } | TutorialActorEvent::Damaged { .. }
        ) && let Ok(mut environment) = player_environments.single_mut()
        {
            environment.observe_local_combat();
        }
        if matches!(event, TutorialActorEvent::Dead { .. }) {
            // Retrobution cnVirtualServer.SendReward adds 30 FM for every
            // lethal weapon/Nano result, even before a mission is accepted.
            // Dead is emitted once on the HP edge; scripted death poses and
            // the delayed loot presentation must not grant another reward.
            mission_runtime.fusion_matter = mission_runtime.fusion_matter.saturating_add(30);
        }
        if let TutorialActorEvent::Damaged { entity, .. } = event
            && let Ok((actor, _)) = actors.get(entity)
        {
            gameplay_audio.queue_tutorial_actor_damage(entity, actor.npc_type);
        }
        if let TutorialActorEvent::AttackedPlayer { .. } = event
            && let Ok((player_entity, _)) = local_players.single()
            && let Some(rig) = rigs.iter().find(|rig| rig.controller_root == player_entity)
        {
            gameplay_audio.queue_player_hurt(player_entity, rig.gender);
        }
        if let TutorialActorEvent::AttackedPlayer { entity, .. } = event
            && let Ok((actor, actor_transform)) = actors.get(entity)
            && let Some(definition) = content.gameplay_npc(actor.npc_type)
            && definition.attack_effect > 0
            && let Ok((player_entity, player_transform)) = local_players.single()
        {
            let source = if definition.attack_effect == 66 {
                tutorial_named_descendant_position(entity, "tag01", &parents, &named_transforms)
                    .unwrap_or_else(|| {
                        actor_transform.translation() + Vec3::Y * (definition.height() * 0.5)
                    })
            } else {
                actor_transform.translation() + Vec3::Y * (definition.height() * 0.5)
            };
            let target = tutorial_named_descendant_position(
                player_entity,
                "Bip01 Spine1",
                &parents,
                &named_transforms,
            )
            .unwrap_or_else(|| player_transform.translation() + Vec3::Y * 0.8);
            effect_runtime.enqueue(TutorialEffectRuntimeCommand::Projectile {
                bullet_type: definition.attack_effect,
                source,
                target,
                target_exists: true,
                source_style: definition.npc_style,
                target_style: tutorial_active_nano_style(&runtime),
                motion: TutorialProjectileMotion::BulletMove,
                source_line: line!(),
            });
        }
        if let TutorialActorEvent::PlayerKillDeathPresentation { entity, .. } = event
            && let Ok((_, actor_transform)) = actors.get(entity)
            && let Ok((_, player_transform)) = local_players.single()
        {
            // DeadMotion.cs lines 103/110: player kills generate the normal
            // (non-reversed) Oni Candy/Blob pair after ES372 is started.
            effect_runtime.enqueue(sampled_oni_projectile_pair_command(
                [76, 77],
                actor_transform.translation(),
                player_transform.translation(),
                false,
                103,
                &mut projectile_random,
            ));
        }
        let mission_reward_advanced = matches!(
            event,
            TutorialActorEvent::Dead {
                id: MISSION_TARGET_ID,
                ..
            }
        ) && matches!(
            tutorial.progress.stage(),
            Some(TutorialStage::Mission(MissionStage::ObjectiveCombat))
        );
        let mission_reward_advanced = if mission_reward_advanced {
            match mission_runtime.complete_task(&content, 2248) {
                Ok(mutation) => {
                    mutation.state_changed
                        && mutation.outgoing_task_id == Some(2249)
                        && mutation.outgoing_started
                }
                Err(error) => {
                    runtime.message = format!("Tutorial mission reward rejected: {error}");
                    false
                }
            }
        } else {
            false
        };
        apply_tutorial_actor_event(
            event,
            &mut tutorial.progress,
            &mut runtime.hp,
            mission_reward_advanced,
        );
    }
}

pub(super) fn tutorial_named_descendant_position(
    root: Entity,
    exact_name: &str,
    parents: &Query<&ChildOf>,
    named_transforms: &Query<(Entity, &Name, &GlobalTransform)>,
) -> Option<Vec3> {
    named_transforms
        .iter()
        .find(|(entity, name, _)| {
            name.as_str().eq_ignore_ascii_case(exact_name)
                && (*entity == root || dexter_ship_descendant(*entity, root, parents))
        })
        .map(|(_, _, transform)| transform.translation())
}
