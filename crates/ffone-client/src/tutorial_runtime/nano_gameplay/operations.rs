use super::*;

/// Exact Unity-local `(0.7, 0, 0)` Nano offset plus the 1.12 vertical follow
/// offset. `owner.rotation` is already the reflected native gameplay-root
/// rotation (including the model half-turn), so reflecting local X again
/// would put the Nano on the opposite side.
#[must_use]
pub fn tutorial_nano_follow_target(owner: &Transform) -> Vec3 {
    owner.translation + owner.rotation * Vec3::new(0.7, 0.0, 0.0) + Vec3::Y * 1.12
}

pub(super) fn equip_gameplay_nano(
    world: &mut World,
    nano_id: i16,
    skill_id: i16,
    stamina: i32,
    world_presentation: Option<WorldNanoGameplayPresentation>,
) {
    let loadout = TutorialNanoGameplayLoadout { nano_id, skill_id };
    if world.resource::<TutorialNanoGameplayState>().loadout == Some(loadout)
        && world
            .resource::<TutorialNanoGameplayState>()
            .world_presentation
            == world_presentation
    {
        // Ordinary-world stamina is authoritative and may be reconciled by a
        // periodic PC tick without changing the equipped Nano. Keep the
        // already-loaded presentation while accepting that fresh value.
        // Local tutorial equip is an idempotent grant: timeline completion
        // and skip recovery may repeat it after a summon or skill use.
        if world_presentation.is_some() {
            world.resource_mut::<TutorialNanoGameplayState>().stamina = stamina.max(0);
        }
        return;
    }
    if world
        .resource::<TutorialNanoGameplayState>()
        .entity
        .is_some()
    {
        dismiss_gameplay_nano(world, true, false);
    }
    // A failed material bind or an externally removed root can leave the
    // presentation absent while its old AnimationGraph is still retained.
    // Every genuine loadout/presentation change starts from a clean asset
    // contract, regardless of whether there is still an entity to dismiss.
    *world.resource_mut::<TutorialNanoGameplayAssets>() = TutorialNanoGameplayAssets::default();
    let mut state = world.resource_mut::<TutorialNanoGameplayState>();
    state.loadout = Some(loadout);
    state.world_presentation = world_presentation;
    state.stamina = stamina.max(0);
    // Cooldown channels belong to equipped slots in the source. This
    // native gameplay slice owns one exact loadout, so a genuinely new equip
    // creates a new channel; dismiss/summon deliberately leaves the existing
    // one.
    state.skill_cooldown_remaining_seconds = 0.0;
    state.mark_absent();
}

pub(super) fn summon_gameplay_nano(world: &mut World, owner: Entity) {
    if world.resource::<TutorialNanoGameplayState>().stamina <= 0 {
        return;
    }
    let (loadout, world_presentation, existing, existing_owner, status) = {
        let state = world.resource::<TutorialNanoGameplayState>();
        (
            state.loadout,
            state.world_presentation.clone(),
            state.entity,
            state.owner,
            state.status.clone(),
        )
    };
    // An unchanged blocked contract must remain fail-closed. Ordinary-world
    // synchronization can request a summon every frame while the failed root
    // is absent; retrying here would replay the summon effect and reload the
    // same broken asset indefinitely. A genuine equip changes the contract and
    // restores `Absent` before a new summon reaches this function.
    if matches!(&status, TutorialNanoGameplayStatus::Blocked(_)) {
        return;
    }
    let exact_loadout = TutorialNanoGameplayLoadout {
        nano_id: TUTORIAL_BUTTERCUP_NANO_ID,
        skill_id: TUTORIAL_BUTTERCUP_SKILL_ID,
    };
    if loadout != Some(exact_loadout) && world_presentation.is_none() {
        let fallback = loadout.unwrap_or(TutorialNanoGameplayLoadout {
            nano_id: 0,
            skill_id: 0,
        });
        world.resource_mut::<TutorialNanoGameplayIssueQueue>().push(
            TutorialNanoGameplayIssue::UnsupportedLoadout {
                nano_id: fallback.nano_id,
                skill_id: fallback.skill_id,
            },
        );
        return;
    }
    let Some(loadout) = loadout else {
        return;
    };
    let (model_path, style) = match world_presentation.as_ref() {
        Some(presentation) => {
            if presentation.skill_clip().is_none() {
                world.resource_mut::<TutorialNanoGameplayIssueQueue>().push(
                    TutorialNanoGameplayIssue::UnsupportedLoadout {
                        nano_id: loadout.nano_id,
                        skill_id: loadout.skill_id,
                    },
                );
                return;
            }
            (
                presentation.model_path.as_str(),
                i32::from(presentation.style),
            )
        }
        None => (TUTORIAL_NANO_MODEL_PATH, 1),
    };
    if existing.is_some()
        && existing_owner == Some(owner)
        && matches!(
            status,
            TutorialNanoGameplayStatus::Loading | TutorialNanoGameplayStatus::Ready
        )
    {
        return;
    }
    let Some(owner_transform) = world.get::<Transform>(owner).copied() else {
        world
            .resource_mut::<TutorialNanoGameplayIssueQueue>()
            .push(TutorialNanoGameplayIssue::MissingOwner { owner });
        return;
    };
    if existing.is_some() {
        dismiss_gameplay_nano(world, true, false);
    }

    let asset_server = world.resource::<AssetServer>().clone();
    let gltf = asset_server.load(model_path.to_owned());
    let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset(model_path.to_owned()));
    let generation = world
        .resource::<TutorialNanoGameplayState>()
        .generation
        .wrapping_add(1)
        .max(1);
    let now_seconds = world.get_resource::<Time>().map_or(0.0, Time::elapsed_secs);
    let movement = {
        let mut random = world.resource_mut::<LegacyNanoStandRandomStream>();
        TutorialNanoMovementPattern::new(&owner_transform, now_seconds, &mut random)
    };
    // `NanoMoveController.SetParent` repositions and rotates immediately; it
    // does not wait for the first `ForceUpdate` frame.
    let mut nano_transform = owner_transform;
    nano_transform.translation = tutorial_nano_follow_target(&owner_transform);
    let root = world
        .spawn((
            Name::new(format!("Nano {} gameplay Nano", loadout.nano_id)),
            TutorialGameplayNanoRoot { owner, generation },
            movement,
            TutorialNanoSkillAnimationEvents::default(),
            nano_transform,
            Visibility::Inherited,
        ))
        .id();
    world.spawn((
        Name::new(format!("Nano {} gameplay Scene0", loadout.nano_id)),
        TutorialGameplayNanoScene,
        WorldAssetRoot(scene.clone()),
        native_scene_container_transform(NativeSceneRole::CharacterGameplay),
        // `NanoMoveController.SetType` creates only the empty controller
        // container. Its asynchronous `SetupModel` attaches the renderable
        // model later, then `NanoAnimation.SetModel(...); Call()` starts the
        // first pose. Keeping Scene0 hidden until that animation player is
        // bound preserves the clean effect-first reveal even when Bevy has
        // already cached the GLB.
        Visibility::Hidden,
        ChildOf(root),
    ));
    {
        let mut assets = world.resource_mut::<TutorialNanoGameplayAssets>();
        assets.gltf = Some(gltf);
        assets.scene = Some(scene);
    }
    {
        let mut state = world.resource_mut::<TutorialNanoGameplayState>();
        state.entity = Some(root);
        state.owner = Some(owner);
        state.status = TutorialNanoGameplayStatus::Loading;
        state.generation = generation;
        state.activated_generation = None;
        state.asset_contract_ready = false;
        state.face_texture_bound = false;
        state.request_clip(CALL_CLIP);
    }
    // `EventNanoController.Call()` instantiates style + 527 at the Nano's
    // current world transform. Buttercup's exact style is 1, hence ES528.
    world.resource_mut::<TutorialNanoGameplayEventQueue>().push(
        TutorialNanoGameplayEvent::EffectRequested {
            effect_id: 527 + style,
            position: nano_transform.translation,
            rotation: nano_transform.rotation,
            source_line: 49,
        },
    );
}

pub(super) fn withdraw_gameplay_nano(world: &mut World) {
    let state = world.resource::<TutorialNanoGameplayState>();
    let Some(entity) = state.entity else { return };
    if state.animation.mode() == LegacyNanoAnimationMode::Withdraw {
        return;
    }
    if !world
        .resource::<TutorialNanoGameplayAssets>()
        .nodes
        .contains_key("withdraw")
    {
        // No invented animation or farewell for an unavailable/loading model.
        dismiss_gameplay_nano(world, true, false);
        return;
    }
    if let Some(transform) = world.get::<Transform>(entity).copied() {
        world.resource_mut::<TutorialNanoGameplayEventQueue>().push(
            TutorialNanoGameplayEvent::EffectRequested {
                effect_id: TUTORIAL_NANO_DISMISS_EFFECT_ID,
                position: transform.translation,
                rotation: transform.rotation,
                source_line: 161,
            },
        );
    }
    let mut state = world.resource_mut::<TutorialNanoGameplayState>();
    state.stamina = 0;
    state.animation.request(
        LegacyNanoAnimationMode::Withdraw,
        "withdraw",
        LegacyAnimationBlend::CrossFade100Ms,
    );
    state.applied_animation_request_serial = None;
}

pub(super) fn dismiss_gameplay_nano(world: &mut World, emit_event: bool, emit_hide_effect: bool) {
    let (entity, owner) = {
        let state = world.resource::<TutorialNanoGameplayState>();
        (state.entity, state.owner)
    };
    if let Some(entity) = entity {
        if emit_hide_effect && let Some(owner) = owner {
            let voice_owner = {
                let state = world.resource::<TutorialNanoGameplayState>();
                let path = state
                    .world_presentation
                    .as_ref()
                    .map_or(TUTORIAL_NANO_MODEL_PATH, |presentation| {
                        presentation.model_path.as_str()
                    });
                std::path::Path::new(path)
                    .parent()
                    .and_then(std::path::Path::file_name)
                    .and_then(|name| name.to_str())
                    .map(str::to_owned)
            };
            if let Some(voice_owner) = voice_owner
                && let Some(mut audio) = world.get_resource_mut::<GameplayAudioRuntime>()
            {
                audio.queue_nano_dismissal(entity, owner, &voice_owner);
            }
        }
        let effect_transform = emit_hide_effect
            .then(|| world.get::<Transform>(entity).copied())
            .flatten();
        let _ = world.despawn(entity);
        if emit_event {
            if let Some(transform) = effect_transform {
                world.resource_mut::<TutorialNanoGameplayEventQueue>().push(
                    TutorialNanoGameplayEvent::EffectRequested {
                        effect_id: TUTORIAL_NANO_DISMISS_EFFECT_ID,
                        position: transform.translation,
                        rotation: transform.rotation,
                        // NanoContainer.AddNano(-1)
                        source_line: 161,
                    },
                );
            }
            world
                .resource_mut::<TutorialNanoGameplayEventQueue>()
                .push(TutorialNanoGameplayEvent::Dismissed { owner, entity });
        }
    }
    world
        .resource_mut::<TutorialNanoGameplayState>()
        .mark_absent();
    let mut assets = world.resource_mut::<TutorialNanoGameplayAssets>();
    *assets = TutorialNanoGameplayAssets::default();
}

pub(super) fn use_gameplay_nano_skill(world: &mut World, owner: Entity, targets: Vec<LegacyAttackTarget>) {
    let allowed = {
        let state = world.resource::<TutorialNanoGameplayState>();
        state.can_use_skill() && state.owner == Some(owner)
    };
    if !allowed {
        return;
    }
    let Some(owner_position) = world.get::<Transform>(owner).map(|value| value.translation) else {
        return;
    };

    let mut accepted = Vec::new();
    let mut seen = HashSet::new();
    for target in targets {
        if !matches!(target.kind, LegacyTargetKind::Npc { team } if team > 1)
            || !seen.insert(target.entity)
        {
            continue;
        }
        let Some(actor) = world.get::<TutorialActor>(target.entity).copied() else {
            continue;
        };
        let Some(target_position) = world
            .get::<Transform>(target.entity)
            .map(|value| value.translation)
        else {
            continue;
        };
        if actor.id != DEMO_MONSTER_ID
            || actor.team <= 1
            || !actor.is_alive()
            || owner_position.distance(target_position) > TUTORIAL_BUTTERCUP_SKILL_RANGE
        {
            continue;
        }
        accepted.push((target.entity, actor.id));
        if accepted.len() == TUTORIAL_BUTTERCUP_SKILL_TARGET_CAPACITY {
            break;
        }
    }
    if accepted.is_empty() {
        return;
    }

    let (skill_id, stamina) = {
        let mut state = world.resource_mut::<TutorialNanoGameplayState>();
        state.stamina = (state.stamina - TUTORIAL_BUTTERCUP_SKILL_DRAIN)
            .max(TUTORIAL_BUTTERCUP_SKILL_STAMINA_FLOOR);
        // `GameCondition.IsCallCoolTime` starts the local slot timer as soon
        // as the accepted request is made; the success packet does not own it.
        state.skill_cooldown_remaining_seconds = TUTORIAL_BUTTERCUP_SKILL_COOLDOWN_SECONDS;
        state.request_clip(SKILL_CLIP);
        (
            state
                .loadout
                .expect("active gameplay Nano retains a loadout")
                .skill_id,
            state.stamina,
        )
    };
    for (target, actor_id) in &accepted {
        let current = world
            .get::<TutorialNanoSkillCondition>(*target)
            .map_or(0, |condition| condition.0);
        world.entity_mut(*target).insert(TutorialNanoSkillCondition(
            current | TUTORIAL_BUTTERCUP_SKILL_CONDITION,
        ));
        // `cnVirtualServer` raises tutorial event 6 while building each target
        // result, before it sends the packet and raises event 4.
        world.resource_mut::<TutorialNanoGameplayEventQueue>().push(
            TutorialNanoGameplayEvent::DamageNpc {
                target: *target,
                actor_id: *actor_id,
                condition: TUTORIAL_BUTTERCUP_SKILL_CONDITION,
            },
        );
    }
    world.resource_mut::<TutorialNanoGameplayEventQueue>().push(
        TutorialNanoGameplayEvent::UseSkill {
            owner,
            skill_id,
            stamina,
        },
    );
}

/// Starts the recovered equipped-Nano skill presentation for an ordinary
/// shard-world request. Damage, status conditions, stamina, and the per-slot
/// cooldown stay with their ordinary-world owners; this graph only predicts
/// the animation that corresponds to the XDT skill slot.
pub(super) fn play_world_nano_skill(world: &mut World, owner: Entity) {
    let skill_clip = {
        let state = world.resource::<TutorialNanoGameplayState>();
        if state.owner != Some(owner) || state.loadout.is_none() || state.stamina <= 0 {
            return;
        }
        let skill_clip = state
            .world_presentation
            .as_ref()
            .and_then(WorldNanoGameplayPresentation::skill_clip)
            .unwrap_or(SKILL_CLIP);
        skill_clip
    };
    let mut state = world.resource_mut::<TutorialNanoGameplayState>();
    if state.is_active() {
        state.request_clip(skill_clip);
    } else if matches!(state.status, TutorialNanoGameplayStatus::Loading) {
        let skill_mode = match skill_clip {
            "skill1" => LegacyNanoAnimationMode::Skill1,
            "skill2" => LegacyNanoAnimationMode::Skill2,
            "skill3" => LegacyNanoAnimationMode::Skill3,
            _ => return,
        };
        if state.animation.mode() == LegacyNanoAnimationMode::Call {
            // `NanoAnimation.EndAnimation` consumes this after the summon call
            // finishes, preserving both the call and the accepted skill.
            state.animation.set_passive_skill(Some(skill_mode));
        } else {
            // Loading normally owns Call already, but retaining the desired
            // clip directly also keeps presentation deterministic if a scene
            // rebuild has not yet restored that request.
            state.request_clip(skill_clip);
        }
    }
}

pub(super) fn follow_tutorial_gameplay_nano(
    time: Res<Time>,
    mut commands: Commands,
    mut state: ResMut<TutorialNanoGameplayState>,
    mut random: ResMut<LegacyNanoStandRandomStream>,
    mut roots: Query<(
        &TutorialGameplayNanoRoot,
        &mut TutorialNanoMovementPattern,
        &mut Transform,
    )>,
    owners: Query<&Transform, Without<TutorialGameplayNanoRoot>>,
    mut events: ResMut<TutorialNanoGameplayEventQueue>,
) {
    let Some(entity) = state.entity else {
        return;
    };
    let Ok((root, mut movement, mut transform)) = roots.get_mut(entity) else {
        state.mark_absent();
        return;
    };
    let Ok(owner) = owners.get(root.owner) else {
        commands.entity(entity).despawn();
        events.push(TutorialNanoGameplayEvent::Dismissed {
            owner: Some(root.owner),
            entity,
        });
        state.mark_absent();
        return;
    };
    let mode = state.animation.mode();
    let skill_frozen = matches!(
        mode,
        LegacyNanoAnimationMode::Skill1
            | LegacyNanoAnimationMode::Skill2
            | LegacyNanoAnimationMode::Skill3
    );
    let movement_frozen = skill_frozen
        || matches!(
            mode,
            LegacyNanoAnimationMode::Call
                | LegacyNanoAnimationMode::Discharge
                | LegacyNanoAnimationMode::Casting
        );
    let delta_seconds = time.delta_secs();
    let now_seconds = time.elapsed_secs();
    if skill_frozen {
        // ForceUpdate snaps facing only for modes 11..13, not Call/Casting.
        transform.rotation = owner.rotation;
    } else if !movement_frozen {
        if now_seconds - movement.pattern_started_seconds > TUTORIAL_NANO_PATTERN_SECONDS {
            if random.next_index(3) == 0 {
                movement.randomize(now_seconds, &mut random);
            } else {
                movement.pattern_started_seconds = now_seconds;
            }
        }
        movement.lateral_offset = advance_tutorial_nano_orbit(
            movement.lateral_offset,
            movement.orbit,
            movement.orbit_speed_radians,
            delta_seconds,
        );
        let turn_delta = movement.turn_speed_radians * delta_seconds;
        match movement.turn {
            TutorialNanoTurnPattern::Clockwise => {
                transform.rotation *= Quat::from_rotation_y(-turn_delta);
            }
            TutorialNanoTurnPattern::CounterClockwise => {
                transform.rotation *= Quat::from_rotation_y(turn_delta);
            }
            TutorialNanoTurnPattern::None => {}
        }
    }
    let target =
        owner.translation + movement.lateral_offset + Vec3::Y * (1.12 + movement.pulse_offset());
    transform.translation =
        advance_tutorial_nano_follow(transform.translation, target, delta_seconds);
}

/// Exact state-transition cleanup hook. It affects only the gameplay Nano and
/// never the cutscene-owned `newNano` presentation root.
pub fn cleanup_tutorial_nano_gameplay(world: &mut World) {
    let entity = world
        .get_resource::<TutorialNanoGameplayState>()
        .and_then(TutorialNanoGameplayState::entity);
    if let Some(entity) = entity {
        let _ = world.despawn(entity);
    }
    if let Some(mut state) = world.get_resource_mut::<TutorialNanoGameplayState>() {
        *state = TutorialNanoGameplayState::default();
    }
    if let Some(mut queue) = world.get_resource_mut::<TutorialNanoGameplayCommandQueue>() {
        *queue = TutorialNanoGameplayCommandQueue::default();
    }
    if let Some(mut events) = world.get_resource_mut::<TutorialNanoGameplayEventQueue>() {
        *events = TutorialNanoGameplayEventQueue::default();
    }
    if let Some(mut issues) = world.get_resource_mut::<TutorialNanoGameplayIssueQueue>() {
        *issues = TutorialNanoGameplayIssueQueue::default();
    }
    if let Some(mut assets) = world.get_resource_mut::<TutorialNanoGameplayAssets>() {
        *assets = TutorialNanoGameplayAssets::default();
    }
}
