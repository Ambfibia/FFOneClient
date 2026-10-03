use super::*;

pub(super) fn nearest_view_target(
    feed: &LegacyAvatarTargetFeed,
    predicate: impl Fn(LegacyTargetKind) -> bool,
) -> Option<LegacyFocusedTarget> {
    feed.samples
        .iter()
        .copied()
        .filter(|sample| sample.in_view && predicate(sample.kind))
        .min_by(|left, right| left.distance.total_cmp(&right.distance))
        .map(Into::into)
}

pub(super) fn nano_kind_matches(target_type: i32, kind: LegacyTargetKind) -> bool {
    matches!(
        (target_type, kind),
        (1, LegacyTargetKind::Npc { team }) if team > 1
    ) || matches!((target_type, kind), (2, LegacyTargetKind::Player))
}

pub(super) fn nano_focus(
    policy: LegacyNanoTargetPolicy,
    focused_npc: Option<LegacyFocusedTarget>,
    focused_player: Option<LegacyFocusedTarget>,
) -> Option<LegacyFocusedTarget> {
    match policy.target_type {
        1 => focused_npc.filter(|target| nano_kind_matches(1, target.kind)),
        2 => focused_player.filter(|target| nano_kind_matches(2, target.kind)),
        _ => None,
    }
}

pub(super) fn nano_extent(sample: &LegacyTargetSample) -> f32 {
    sample.radius.max(sample.height * 0.5)
}

pub(super) fn nano_distance(left: [f32; 3], right: [f32; 3]) -> f32 {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| (left - right).powi(2))
        .sum::<f32>()
        .sqrt()
}

pub(super) fn select_nano_targets(
    feed: &LegacyAvatarTargetFeed,
    policy: LegacyNanoTargetPolicy,
    focused_npc: Option<LegacyFocusedTarget>,
    focused_player: Option<LegacyFocusedTarget>,
) -> Vec<LegacyAttackTarget> {
    if policy.capacity == 0 || policy.target_type == 3 {
        return Vec::new();
    }
    let focus = nano_focus(policy, focused_npc, focused_player);
    let focus_sample = focus.and_then(|focus| {
        feed.samples
            .iter()
            .find(|sample| sample.entity == focus.entity)
    });
    let mut selected = match policy.effect_target {
        1 => focus_sample
            .filter(|sample| sample.distance <= policy.range + nano_extent(sample))
            .into_iter()
            .collect::<Vec<_>>(),
        3 => {
            let candidates = feed
                .samples
                .iter()
                .filter(|sample| nano_kind_matches(policy.target_type, sample.kind))
                .filter(|sample| sample.in_nano_arc)
                .filter(|sample| sample.distance <= policy.range + nano_extent(sample))
                .collect::<Vec<_>>();
            if focus_sample.is_some_and(|focus| {
                candidates
                    .iter()
                    .any(|candidate| candidate.entity == focus.entity)
            }) {
                candidates
            } else {
                Vec::new()
            }
        }
        5 => feed
            .samples
            .iter()
            .filter(|sample| nano_kind_matches(policy.target_type, sample.kind))
            .filter(|sample| sample.distance <= policy.area + nano_extent(sample))
            .collect::<Vec<_>>(),
        6 => focus_sample
            .filter(|focus| focus.distance <= policy.range + nano_extent(focus))
            .map(|focus| {
                feed.samples
                    .iter()
                    .filter(|sample| nano_kind_matches(policy.target_type, sample.kind))
                    .filter(|sample| {
                        nano_distance(sample.position, focus.position)
                            <= policy.area + nano_extent(sample)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    selected.sort_by(|left, right| left.distance.total_cmp(&right.distance));
    if let Some(focus) = focus_sample
        && let Some(index) = selected
            .iter()
            .position(|candidate| candidate.entity == focus.entity)
    {
        let focus = selected.remove(index);
        selected.insert(0, focus);
    }
    selected
        .into_iter()
        .take(policy.capacity)
        .map(|sample| LegacyAttackTarget {
            entity: sample.entity,
            kind: sample.kind,
            distance: sample.distance,
        })
        .collect()
}

/// Reproduces the post-cone-list target ordering in `cnAvatarAttack.Update`.
/// Tab is intentionally absent: legacy Key 11 is weapon change, not targeting.
#[must_use]
pub fn select_legacy_targets(
    feed: &LegacyAvatarTargetFeed,
    context: &LegacyAvatarActionContext,
) -> LegacyTargetSelection {
    let mut focused_npc =
        nearest_view_target(feed, |kind| matches!(kind, LegacyTargetKind::Npc { .. }));
    let mut focused_player = context
        .player_interaction_allowed
        .then(|| nearest_view_target(feed, |kind| matches!(kind, LegacyTargetKind::Player)))
        .flatten();
    let mut check_attack_target = true;

    if let Some(npc) = focused_npc
        && npc.kind.is_friendly_npc()
        && (!context.attack_players_enabled
            || focused_player.is_none()
            || focused_player.is_some_and(|player| npc.distance <= player.distance))
    {
        if context.combat_condition {
            focused_npc = None;
        } else {
            check_attack_target = false;
        }
    }

    let mut attack_targets = Vec::new();
    if check_attack_target && !context.attack_locked {
        attack_targets.extend(
            feed.samples
                .iter()
                .copied()
                .filter(|sample| sample.in_attack_cone)
                .filter(|sample| match sample.kind {
                    LegacyTargetKind::Npc { team } => team != 1,
                    LegacyTargetKind::Player => context.attack_players_enabled,
                })
                .map(|sample| LegacyAttackTarget {
                    entity: sample.entity,
                    kind: sample.kind,
                    distance: sample.distance,
                }),
        );
        attack_targets.sort_by(|left, right| left.distance.total_cmp(&right.distance));
        attack_targets.truncate(context.target_capacity);
    }

    if !context.attack_players_enabled && attack_targets.is_empty() {
        check_attack_target = false;
    }
    if context.combat_condition
        && let Some(first) = attack_targets.first().copied()
        && let Some(sample) = feed
            .samples
            .iter()
            .find(|sample| sample.entity == first.entity)
    {
        match first.kind {
            LegacyTargetKind::Npc { .. } => focused_npc = Some((*sample).into()),
            LegacyTargetKind::Player => focused_player = Some((*sample).into()),
        }
    }

    let mut nano_targets = if !context.nano_locked && context.nano_skill_usable {
        context
            .nano_target_policy
            .map(|policy| select_nano_targets(feed, policy, focused_npc, focused_player))
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    if context.nano_target_policy.is_none()
        && !context.nano_locked
        && context.nano_skill_usable
        && context.nano_target_range > 0.0
        && context.nano_target_capacity > 0
    {
        nano_targets.extend(
            feed.samples
                .iter()
                .copied()
                .filter(|sample| sample.in_attack_arc)
                .filter(|sample| sample.distance <= context.nano_target_range)
                .filter(|sample| matches!(sample.kind, LegacyTargetKind::Npc { team } if team > 1))
                .map(|sample| LegacyAttackTarget {
                    entity: sample.entity,
                    kind: sample.kind,
                    distance: sample.distance,
                }),
        );
        nano_targets.sort_by(|left, right| left.distance.total_cmp(&right.distance));
        // Buttercup is EffectTarget 3 / TargetType 1. Clean
        // `SearchNanoSkillTarget` first builds the hostile cone list, but then
        // clears the whole list unless the currently focused mob belongs to
        // it. `SearchList.Insert` moves that focus to index zero before the
        // target-count limit is applied.
        let focused_hostile = focused_npc
            .filter(|target| matches!(target.kind, LegacyTargetKind::Npc { team } if team > 1));
        if let Some(focused) = focused_hostile
            && let Some(index) = nano_targets
                .iter()
                .position(|target| target.entity == focused.entity)
        {
            let focused = nano_targets.remove(index);
            nano_targets.insert(0, focused);
            nano_targets.truncate(context.nano_target_capacity);
        } else {
            nano_targets.clear();
        }
    }

    let mut trigger = feed.trigger;
    if let Some(candidate) = trigger {
        if context.combat_condition {
            if focused_npc.is_some_and(|npc| npc.distance > candidate.radius) {
                focused_npc = None;
                check_attack_target = false;
            }
        } else {
            if focused_npc.is_some_and(|npc| npc.distance >= candidate.radius) {
                focused_npc = None;
            }
            if focused_player.is_some_and(|player| player.distance >= candidate.radius - 2.0) {
                focused_player = None;
            }
        }
        if focused_npc.is_some() || focused_player.is_some() {
            trigger = None;
        }
    }

    LegacyTargetSelection {
        source_connected: feed.source_connected,
        focused_npc,
        focused_player,
        trigger: trigger.map(|candidate| candidate.entity),
        check_attack_target,
        attack_targets,
        nano_targets,
    }
}

pub(super) fn cross_fade(
    bindings: &LegacyAvatarClipBindings,
    requested_clip: LegacyVisualClip,
    layer: LegacyAnimationLayer,
    queued_play_now: bool,
) -> LegacyVisualCommand {
    cross_fade_with_duration(
        bindings,
        requested_clip,
        layer,
        queued_play_now,
        LEGACY_ANIMATION_BLEND_SECONDS,
    )
}

#[must_use]
pub(super) const fn legacy_presentation_transition_seconds(state: LegacyLocomotionState) -> f32 {
    match state {
        // `AvatarSlope(1)` uses CrossFade("slide", .35f).
        LegacyLocomotionState::Slide => 0.35,
        // `SetInvenMotion` uses CrossFade(StrModifyForVehicle("inven"), .1f).
        LegacyLocomotionState::Inventory
        | LegacyLocomotionState::BoardInventory
        | LegacyLocomotionState::ScooterInventory => 0.1,
        // `AvatarZipline` and `AvatarRope` call Play, not CrossFade.
        LegacyLocomotionState::RopeDown
        | LegacyLocomotionState::RopeDrop
        | LegacyLocomotionState::RopeLeft
        | LegacyLocomotionState::RopeRight
        | LegacyLocomotionState::RopeStand1
        | LegacyLocomotionState::RopeStand2
        | LegacyLocomotionState::RopeTurn
        | LegacyLocomotionState::RopeUp => 0.0,
        _ => LEGACY_ANIMATION_BLEND_SECONDS,
    }
}

pub(super) fn cross_fade_with_duration(
    bindings: &LegacyAvatarClipBindings,
    requested_clip: LegacyVisualClip,
    layer: LegacyAnimationLayer,
    queued_play_now: bool,
    blend_seconds: f32,
) -> LegacyVisualCommand {
    LegacyVisualCommand::CrossFade {
        requested_clip,
        resolution: bindings.resolve(requested_clip),
        blend_seconds,
        layer,
        queued_play_now,
    }
}

pub(super) fn schedule_primary_attack(
    delta_seconds: f32,
    context: &LegacyAvatarActionContext,
    selection: &LegacyTargetSelection,
    state: &mut LegacyAvatarActionState,
    bindings: &LegacyAvatarClipBindings,
    schedule: &mut LegacyFrameSchedule,
) {
    state.attack_accumulated += delta_seconds;
    if state.attack_accumulated > LEGACY_ATTACK_ACCUMULATION_LIMIT
        || context.weapon_change_in_progress
        || context.special_state_four
        || context.attack_cooldown_blocked
        || state.seconds_since_attack < context.attack_cooldown_seconds
    {
        return;
    }
    state.seconds_since_attack = 0.0;

    schedule.force_camera_angle = true;
    state.attack_delay_remaining = None;
    if context.weapon_target_mode != LegacyWeaponTargetMode::Normal
        && !context.primary_weapon_shootable
    {
        return;
    }

    state.weapon_change_visual_active = false;
    state.attack_generation = state.attack_generation.wrapping_add(1);
    let targets = selection.attack_targets.clone();
    schedule
        .actions
        .push(LegacyAvatarActionIntent::PrimaryAttack {
            mode: context.weapon_target_mode,
            targets: targets.clone(),
        });

    let updates_sequence =
        context.weapon_target_mode == LegacyWeaponTargetMode::Normal && !targets.is_empty();
    if updates_sequence {
        state.attack_delay_remaining = Some(LEGACY_ATTACK_DELAY_SECONDS);
    } else {
        state.attack_sequence = 1;
    }

    let full_body = matches!(
        state.locomotion,
        LegacyLocomotionState::Stand | LegacyLocomotionState::Ready
    );
    let mut effective_sequence = state.attack_sequence;
    if full_body {
        let requested = LegacyVisualClip::AttackFull(effective_sequence);
        schedule.visuals.push(cross_fade(
            bindings,
            requested,
            LegacyAnimationLayer::FullBody,
            false,
        ));
        if !bindings.is_connected(requested) {
            effective_sequence = 1;
        }
        state.base_action = Some(requested);
    }

    let requested_upper = LegacyVisualClip::AttackUpper(effective_sequence);
    schedule.visuals.push(cross_fade(
        bindings,
        requested_upper,
        LegacyAnimationLayer::UpperBody,
        true,
    ));
    if !bindings.is_connected(requested_upper) {
        effective_sequence = 1;
    }
    state.upper_action = Some(requested_upper);
    state.attack_sequence = if updates_sequence {
        effective_sequence.saturating_add(1)
    } else {
        1
    };
}

pub(super) fn ground_locomotion(
    direction_key: u8,
    context: &LegacyAvatarActionContext,
    upper_action: Option<LegacyVisualClip>,
) -> LegacyLocomotionState {
    if direction_key == 0 {
        if (context.combat_condition && !context.vehicle_mounted && !context.tutorial_event)
            || matches!(upper_action, Some(LegacyVisualClip::AttackUpper(_)))
        {
            LegacyLocomotionState::Ready
        } else {
            LegacyLocomotionState::Stand
        }
    } else if (4..=6).contains(&direction_key) {
        LegacyLocomotionState::RunBack
    } else {
        LegacyLocomotionState::Run
    }
}

pub(super) fn water_locomotion(direction_key: u8) -> LegacyLocomotionState {
    match direction_key {
        0 => LegacyLocomotionState::SwimIdle,
        7 | 8 => LegacyLocomotionState::SwimLeft,
        2 | 3 => LegacyLocomotionState::SwimRight,
        4..=6 => LegacyLocomotionState::SwimBack,
        _ => LegacyLocomotionState::Swim,
    }
}

pub(super) fn process_legacy_visual_completions(
    mut completions: ResMut<LegacyVisualCompletionQueue>,
    mut players: Query<(
        &LegacyPlayerController,
        &LegacyAvatarActionContext,
        Option<&LegacyAvatarPresentationContext>,
        &LegacyAvatarClipBindings,
        &mut LegacyAvatarActionState,
    )>,
    mut visuals: ResMut<LegacyVisualRequestQueue>,
) {
    for completion in completions.take_all() {
        let Ok((controller, context, presentation, bindings, mut state)) =
            players.get_mut(completion.actor)
        else {
            continue;
        };
        let presentation_override = presentation
            .copied()
            .unwrap_or_default()
            .authoritative_locomotion_override();
        if completion.clip == LegacyVisualClip::Ready && state.weapon_change_visual_active {
            state.weapon_change_visual_active = false;
        }
        if context.dead
            && state.death_phase == LegacyAvatarDeathPhase::Dying
            && completion.clip == LegacyVisualClip::Die
        {
            // `cnAvatarAnimation.EndAnimation` maps `die` to the looping
            // `death` clip and uses its standard 0.3 second end blend.
            state.death_phase = LegacyAvatarDeathPhase::Dead;
            visuals.pending.push_back(LegacyVisualRequest {
                actor: completion.actor,
                command: cross_fade_with_duration(
                    bindings,
                    LegacyVisualClip::Death,
                    LegacyAnimationLayer::FullBody,
                    false,
                    LEGACY_END_ANIMATION_BLEND_SECONDS,
                ),
            });
            continue;
        }
        if state.upper_action == Some(completion.clip) {
            state.upper_action = None;
        }
        let next = if state.base_action == Some(completion.clip) {
            state.base_action = None;
            presentation_override.or_else(|| {
                matches!(completion.clip, LegacyVisualClip::AttackFull(_))
                    .then_some(LegacyLocomotionState::Ready)
            })
        } else if state.locomotion.clip() == completion.clip {
            presentation_override.or_else(|| match state.locomotion {
                LegacyLocomotionState::JumpStart if !controller.grounded => {
                    Some(LegacyLocomotionState::Jump)
                }
                LegacyLocomotionState::Landing | LegacyLocomotionState::LandingRun
                    if controller.grounded =>
                {
                    Some(ground_locomotion(
                        controller.current_direction_key(),
                        context,
                        state.upper_action,
                    ))
                }
                _ => None,
            })
        } else {
            None
        };
        if let Some(next) = next
            && next != state.locomotion
        {
            state.locomotion = next;
            visuals.pending.push_back(LegacyVisualRequest {
                actor: completion.actor,
                command: cross_fade_with_duration(
                    bindings,
                    next.clip(),
                    LegacyAnimationLayer::FullBody,
                    false,
                    LEGACY_END_ANIMATION_BLEND_SECONDS,
                ),
            });
        }
    }
}
