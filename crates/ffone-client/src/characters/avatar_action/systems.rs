use super::*;

pub(super) fn advance_attack_timers(
    delta_seconds: f32,
    context: &LegacyAvatarActionContext,
    state: &mut LegacyAvatarActionState,
    schedule: &mut LegacyFrameSchedule,
) {
    if state.seconds_since_attack.is_finite() {
        state.seconds_since_attack += delta_seconds;
    }
    if state.seconds_since_attack > 1.0 {
        state.attack_accumulated = (state.attack_accumulated - 3.0 * delta_seconds).max(0.0);
    }
    if let Some(mut remaining) = state.attack_delay_remaining {
        remaining -= delta_seconds;
        if remaining < 0.0 {
            state.attack_delay_remaining = None;
            if context.weapon_target_mode == LegacyWeaponTargetMode::Normal {
                schedule.visuals.push(LegacyVisualCommand::DelayCurrent {
                    seconds: LEGACY_ANIMATION_BLEND_SECONDS,
                });
            }
        } else {
            state.attack_delay_remaining = Some(remaining);
        }
    }
}

pub(super) fn update_legacy_avatar_locomotion(
    mut players: Query<(
        Entity,
        &LegacyPlayerController,
        &LegacyAvatarActionContext,
        Option<&LegacyAvatarPresentationContext>,
        &LegacyAvatarClipBindings,
        &mut LegacyAvatarActionState,
        Option<&LegacyWorldColliderPending>,
        Option<&LegacyAvatarEnvironmentState>,
    )>,
    mut visuals: ResMut<LegacyVisualRequestQueue>,
) {
    for (
        entity,
        controller,
        context,
        presentation,
        bindings,
        mut state,
        collision_pending,
        environment,
    ) in &mut players
    {
        let combat_ended = state.was_in_combat && !context.combat_condition;
        state.was_in_combat = context.combat_condition;
        if combat_ended {
            // A renderer completion can be lost while an attack is being
            // replaced or its rig is loading. The authoritative combat edge
            // is the final bound on both attack layers: keeping either one
            // here makes the next walk blend from a stale ready/upper pose.
            state.upper_action = None;
            state.base_action = None;
        }
        if context.dead {
            state.weapon_change_visual_active = false;
            state.upper_action = None;
            state.base_action = None;
            let entering_death = state.death_phase == LegacyAvatarDeathPhase::Alive;
            let requested = match state.death_phase {
                LegacyAvatarDeathPhase::Alive | LegacyAvatarDeathPhase::Dying => {
                    LegacyVisualClip::Die
                }
                LegacyAvatarDeathPhase::Dead => LegacyVisualClip::Death,
            };
            if entering_death {
                state.death_phase = LegacyAvatarDeathPhase::Dying;
            }
            if entering_death || !state.visual_initialized {
                visuals.pending.push_back(LegacyVisualRequest {
                    actor: entity,
                    // Clean `AvatarDead` cross-fades `die` in 0.15 seconds.
                    command: cross_fade(bindings, requested, LegacyAnimationLayer::FullBody, false),
                });
            }
            state.visual_initialized = true;
            state.was_grounded = controller.grounded;
            continue;
        }
        if state.death_phase != LegacyAvatarDeathPhase::Alive {
            state.death_phase = LegacyAvatarDeathPhase::Alive;
            state.visual_initialized = false;
            state.upper_action = None;
            state.base_action = None;
        }
        // Status and Nano Dash own the pose before normal jump/run classification.
        let ability_pose = if context.time_buff_condition == 5 {
            Some(LegacyLocomotionState::Stun)
        } else if controller.nano_dash_active() {
            Some(if controller.nano_dash_started_airborne() || environment.is_some_and(|e| e.in_water) {
                LegacyLocomotionState::DashAir
            } else { LegacyLocomotionState::Dash })
        } else { None };
        if let Some(next) = ability_pose {
            state.weapon_change_visual_active = false;
            state.base_action = None;
            state.upper_action = None;
            if !state.visual_initialized || state.locomotion != next {
                state.locomotion = next;
                let water = environment.is_some_and(|e| e.in_water);
                // Primary AvatarDash starts the weapon dash before blending
                // to the airborne somersault; swimming skips that first pose.
                if next == LegacyLocomotionState::DashAir && !water {
                    visuals.pending.push_back(LegacyVisualRequest { actor: entity,
                        command: cross_fade_with_duration(bindings, LegacyVisualClip::Dash, LegacyAnimationLayer::FullBody, false, 0.0) });
                }
                visuals.pending.push_back(LegacyVisualRequest { actor: entity,
                    command: cross_fade_with_duration(bindings, next.clip(), LegacyAnimationLayer::FullBody, false,
                        if next == LegacyLocomotionState::DashAir && !water { 0.1 } else { 0.0 }) });
                if next == LegacyLocomotionState::DashAir {
                    visuals.pending.push_back(LegacyVisualRequest { actor: entity,
                        command: cross_fade_with_duration(bindings,
                            if water { LegacyVisualClip::DashWaterUpper } else { LegacyVisualClip::DashUpper },
                            LegacyAnimationLayer::UpperBody, false, 0.0) });
                }
            }
            state.visual_initialized = true;
            state.was_grounded = controller.grounded;
            continue;
        }
        if let Some(next) = presentation
            .copied()
            .unwrap_or_default()
            .authoritative_locomotion_override()
        {
            // `AvatarSlope`, `AvatarZipline`, and `SetInvenMotion` own the
            // full-body clip while their source fact remains active. They must
            // preempt both a stationary base action and ordinary
            // ground/jump/swim classification, but they do not fabricate a
            // mounted state from the equipment slot.
            let interrupted_attack = state.base_action.is_some()
                || matches!(state.upper_action, Some(LegacyVisualClip::AttackUpper(_)));
            state.base_action = None;
            state.weapon_change_visual_active = false;
            if matches!(next, LegacyLocomotionState::RopeDown | LegacyLocomotionState::RopeDrop
                | LegacyLocomotionState::RopeLeft | LegacyLocomotionState::RopeRight
                | LegacyLocomotionState::RopeStand1 | LegacyLocomotionState::RopeStand2
                | LegacyLocomotionState::RopeTurn | LegacyLocomotionState::RopeUp)
            {
                if interrupted_attack {
                    state.attack_generation = state.attack_generation.wrapping_add(1);
                    visuals.cancel_attack_for(entity);
                }
                state.upper_action = None;
                state.attack_delay_remaining = None;
            }
            if !state.visual_initialized || next != state.locomotion {
                state.locomotion = next;
                visuals.pending.push_back(LegacyVisualRequest {
                    actor: entity,
                    command: cross_fade_with_duration(
                        bindings,
                        next.clip(),
                        LegacyAnimationLayer::FullBody,
                        false,
                        legacy_presentation_transition_seconds(next),
                    ),
                });
            }
            state.visual_initialized = true;
            state.was_grounded = controller.grounded;
            continue;
        }
        if collision_pending.is_some() {
            // The main runtime deliberately seeds `grounded=false` before the
            // first authored collision sample. That loading sentinel must not
            // enter JumpStart/Landing: the original visible avatar stays in
            // stand1 until its world placement is ready.
            if !state.visual_initialized || state.locomotion != LegacyLocomotionState::Stand {
                state.locomotion = LegacyLocomotionState::Stand;
                visuals.pending.push_back(LegacyVisualRequest {
                    actor: entity,
                    command: cross_fade(
                        bindings,
                        LegacyVisualClip::Stand1,
                        LegacyAnimationLayer::FullBody,
                        false,
                    ),
                });
            }
            state.visual_initialized = true;
            state.was_grounded = true;
            continue;
        }
        let direction_key = controller.current_direction_key();
        let in_water = !context.vehicle_mounted && environment.is_some_and(|environment| environment.in_water);
        if state.weapon_change_visual_active {
            if controller.grounded && direction_key == 0 && !in_water {
                state.base_action = None;
                if !state.visual_initialized || state.locomotion != LegacyLocomotionState::Ready {
                    state.locomotion = LegacyLocomotionState::Ready;
                    visuals.pending.push_back(LegacyVisualRequest {
                        actor: entity,
                        command: cross_fade(
                            bindings,
                            LegacyVisualClip::Ready,
                            LegacyAnimationLayer::FullBody,
                            false,
                        ),
                    });
                }
                state.visual_initialized = true;
                state.was_grounded = true;
                continue;
            }
            state.weapon_change_visual_active = false;
        }
        // A stationary shot owns the base pose until the clamp clip completes,
        // but `cnAvatarAnimation.AvatarMove` immediately cross-fades that base
        // to run/runback as soon as a direction is pressed. The upper attack
        // keeps playing, so the legs can move without delaying the shot.
        if state.base_action.is_some() && controller.grounded && !in_water && direction_key == 0 {
            state.visual_initialized = true;
            state.was_grounded = true;
            continue;
        }
        if state.base_action.is_some() {
            state.base_action = None;
        }
        let next = if in_water {
            water_locomotion(direction_key)
        } else if !state.visual_initialized {
            if controller.grounded {
                ground_locomotion(direction_key, context, state.upper_action)
            } else {
                LegacyLocomotionState::JumpStart
            }
        } else if !controller.grounded {
            if state.was_grounded
                || !matches!(
                    state.locomotion,
                    LegacyLocomotionState::JumpStart | LegacyLocomotionState::Jump
                )
            {
                LegacyLocomotionState::JumpStart
            } else {
                state.locomotion
            }
        } else if !state.was_grounded {
            if direction_key == 0 {
                LegacyLocomotionState::Landing
            } else {
                LegacyLocomotionState::LandingRun
            }
        } else {
            match state.locomotion {
                LegacyLocomotionState::Landing if direction_key != 0 => {
                    LegacyLocomotionState::LandingRun
                }
                LegacyLocomotionState::Landing | LegacyLocomotionState::LandingRun => {
                    state.locomotion
                }
                LegacyLocomotionState::JumpStart | LegacyLocomotionState::Jump => {
                    if direction_key == 0 {
                        LegacyLocomotionState::Landing
                    } else {
                        LegacyLocomotionState::LandingRun
                    }
                }
                LegacyLocomotionState::Ready if direction_key == 0 && context.combat_condition => {
                    // Keep the ready pose only while combat still owns it.
                    // Once the combat lease ends, resolve Stand in this same
                    // frame so a later movement input cannot blend from a
                    // stale combat pose.
                    LegacyLocomotionState::Ready
                }
                _ => ground_locomotion(direction_key, context, state.upper_action),
            }
        };

        if !state.visual_initialized || next != state.locomotion {
            state.locomotion = next;
            visuals.pending.push_back(LegacyVisualRequest {
                actor: entity,
                command: cross_fade(bindings, next.clip(), LegacyAnimationLayer::FullBody, false),
            });
        }
        state.visual_initialized = true;
        state.was_grounded = controller.grounded;
    }
}
