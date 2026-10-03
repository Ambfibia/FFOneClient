use super::*;

pub(super) fn advance_tutorial_player_animation_state(
    vehicle: Res<PersonalVehiclePresentation>,
    time: Res<Time>,
    mut queue: ResMut<TutorialPlayerPresentationCommandQueue>,
    mut completions: ResMut<LegacyVisualCompletionQueue>,
    weapon_catalog: Res<PlayerWeaponAnimationCatalog>,
    mut continuation: ResMut<PlayerEmoteContinuation>,
    mut audio: Option<ResMut<crate::gameplay_audio::GameplayAudioRuntime>>,
    nanos: Query<&crate::tutorial_nano_gameplay::TutorialGameplayNanoRoot>,
    mut rigs: Query<
        (
            &TutorialSelectedPlayerRig,
            &mut TutorialPlayerAnimationAdapter,
            &TutorialSelectedPlayerRigStatus,
        ),
        With<TutorialSelectedPlayerRigActive>,
    >,
    controllers: Query<(
        &LegacyPlayerController,
        Option<&LegacyWorldColliderPending>,
        &LegacyAvatarActionState,
    )>,
    mut players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    let Ok((selected, mut adapter, status)) = rigs.single_mut() else {
        return;
    };
    if !matches!(status, TutorialSelectedPlayerRigStatus::Ready) {
        return;
    }
    let Ok((controller, collision_pending, action_state)) =
        controllers.get(selected.controller_root)
    else {
        return;
    };
    let Ok((mut player, mut transitions)) = players.get_mut(adapter.animation_player) else {
        return;
    };
    adapter.damage.advance(
        &mut player,
        matches!(action_state.authoritative_visual_clip(), LegacyVisualClip::Die | LegacyVisualClip::Death),
    );
    // A fresh authoritative emote/forced pose will be applied below this
    // system. The outgoing clip must not append a renewal or idle return
    // behind it in the same FIFO batch.
    if queue.has_pending_direct_pose_override() {
        continuation
            .pending
            .retain(|(owner, _)| *owner != selected.controller_root);
        return;
    }
    let weapon_item_id = selected.attached_weapon_item_id;
    let weapon_profile =
        weapon_item_id.and_then(|item_id| weapon_catalog.profile_for_item(item_id));
    let rifle_equipped = weapon_profile == Some(PlayerWeaponAnimationProfile::Rifle);
    let direction_key = if collision_pending.is_some() {
        0
    } else {
        controller.current_direction_key()
    };
    update_tutorial_directional_turn(
        &mut adapter,
        &mut player,
        direction_key,
        rifle_equipped,
        vehicle.family == crate::avatar_action::LegacyVehiclePresentationFamily::Scooter,
        time.delta_secs(),
    );
    let finished_delay = if let Some(delay) = adapter.delay.as_mut() {
        delay.remaining_seconds -= time.delta_secs().max(0.0);
        delay.remaining_seconds <= 0.0
    } else {
        false
    };
    if finished_delay && let Some(delay) = adapter.delay.take() {
        let upper_node = adapter.upper_layer.map(|upper| upper.node);
        resume_tutorial_delayed_playback(&mut player, delay, adapter.active_node, upper_node);
    }
    let active_playback_alive = keep_tutorial_gameplay_playback_alive(
        &mut player,
        adapter.active_node,
        adapter.active_clip,
    );

    let finish_upper_fade_out = adapter.upper_layer.as_mut().is_some_and(|upper| {
        advance_tutorial_upper_layer_playback(upper, &mut player, time.delta_secs())
    });
    if finish_upper_fade_out {
        let pending = adapter.pending_upper_completion.take();
        if let Some(upper) = adapter.upper_layer.take() {
            player.stop(upper.node);
            if upper.base_masked || adapter.body_shape.upper_masked {
                switch_tutorial_composed_pose_mask(
                    &mut adapter,
                    &mut player,
                    &mut transitions,
                    false,
                );
            }
        }
        if let Some(pending) = pending {
            completions.push(LegacyVisualCompletion {
                actor: selected.controller_root,
                clip: pending.semantic_clip,
            });
        }
    }

    let active_clip_finished = adapter.active_clip.playback() == TutorialPlayerClipPlayback::Clamp
        && tutorial_clamp_finished_or_removed(
            &player,
            Some(adapter.active_node),
            selected.gender,
            adapter.active_clip,
        );
    let base_clip_finished = adapter.pending_base_completion.is_some_and(|pending| {
        pending.armed
            && adapter.active_clip == pending.animation_clip
            && tutorial_clamp_finished_or_removed(
                &player,
                Some(adapter.active_node),
                selected.gender,
                pending.animation_clip,
            )
    });
    if base_clip_finished {
        let pending = adapter
            .pending_base_completion
            .take()
            .expect("checked pending base completion");
        completions.push(LegacyVisualCompletion {
            actor: selected.controller_root,
            clip: pending.semantic_clip,
        });
    }
    let upper_clip_finished = adapter.pending_upper_completion.is_some_and(|pending| {
        pending.armed
            && adapter.upper_layer.is_some_and(|upper| !upper.fading_out)
            && tutorial_clamp_finished_or_removed(
                &player,
                adapter.upper_layer.map(|upper| upper.node),
                selected.gender,
                pending.animation_clip,
            )
    });
    if upper_clip_finished {
        let animation_clip = adapter
            .pending_upper_completion
            .expect("checked pending upper completion")
            .animation_clip;
        let upper = adapter
            .upper_layer
            .as_mut()
            .expect("checked active upper playback");
        upper.fading_out = true;
        upper.elapsed_seconds = 0.0;
        if upper_event_releases_immediately(animation_clip, action_state.locomotion) {
            // The clean upper event attempts to CrossFade to runposeupper.
            // That clip does not exist for the unarmed actor, and
            // CrossFadeQueued stops the previous upper state before lookup.
            upper.blend_seconds = 0.0;
        }
    }
    let authoritative_visual_clip = action_state.authoritative_visual_clip();
    let Some(authoritative_clip) =
        personal_vehicle::vehicle_clip(vehicle.family, authoritative_visual_clip).or_else(|| {
            legacy_visual_tutorial_clip(
                authoritative_visual_clip,
                weapon_profile,
                selected.tutorial_stand_semantics,
            )
        })
    else {
        // A base attack without its required weapon contract cannot ever
        // produce a renderer completion. Release the state explicitly rather
        // than panic or hold the last skeleton pose forever.
        if action_state.base_action().is_some() && !base_clip_finished {
            completions.push(LegacyVisualCompletion {
                actor: selected.controller_root,
                clip: authoritative_visual_clip,
            });
        }
        return;
    };
    // Ready is a looping combat pose in the asset contract. A confirmed
    // equipment transition plays one whole cycle before returning to idle;
    // wall-clock timers can expire before a delayed rig even starts playback.
    if action_state.weapon_change_visual_active()
        && authoritative_visual_clip == LegacyVisualClip::Ready
        && adapter.active_clip == authoritative_clip
        && queue.is_empty()
        && player.animation(adapter.active_node).is_some_and(|active| active.completions() > 0)
    {
        completions.push(LegacyVisualCompletion {
            actor: selected.controller_root,
            clip: LegacyVisualClip::Ready,
        });
    }
    // `cnAvatarAnimation.SetModel` installs stand1 before accepting any other
    // state and its Play/CrossFade helpers never clear the old base when a
    // destination is missing. Preserve that hard invariant even if a Bevy
    // transition node was externally stopped or expired: restore the one
    // centralized action state's base immediately, before unrelated FIFO
    // equipment/upper-layer commands can defer recovery for another frame.
    if !active_playback_alive && !base_clip_finished && !adapter.emote_state {
        let restored = adapter
            .nodes
            .get(&authoritative_clip)
            .copied()
            .map(|node| (authoritative_clip, node))
            .or_else(|| {
                adapter
                    .nodes
                    .get(&TutorialPlayerClip::Stand1)
                    .copied()
                    .map(|node| (TutorialPlayerClip::Stand1, node))
            });
        if let Some((restored_clip, node)) = restored {
            restart_missing_tutorial_base_animation(
                &mut player,
                &mut transitions,
                node,
                restored_clip,
            );
            adapter.active_node = node;
            adapter.active_clip = restored_clip;
            adapter.pending_base_completion = (restored_clip == authoritative_clip)
                .then(|| pending_legacy_visual_completion(authoritative_visual_clip, restored_clip))
                .flatten()
                .map(|mut pending| {
                    pending.armed = true;
                    pending
                });
        }
    }
    let full_body_preempts_upper = matches!(action_state.locomotion,
        crate::avatar_action::LegacyLocomotionState::RopeDown
            | crate::avatar_action::LegacyLocomotionState::RopeDrop
            | crate::avatar_action::LegacyLocomotionState::RopeLeft
            | crate::avatar_action::LegacyLocomotionState::RopeRight
            | crate::avatar_action::LegacyLocomotionState::RopeStand1
            | crate::avatar_action::LegacyLocomotionState::RopeStand2
            | crate::avatar_action::LegacyLocomotionState::RopeTurn
            | crate::avatar_action::LegacyLocomotionState::RopeUp)
        || matches!(action_state.authoritative_visual_clip(), LegacyVisualClip::Die | LegacyVisualClip::Death);
    if full_body_preempts_upper {
        queue.cancel_pending_runtime_attacks();
        queue.cancel_pending_upper_layers();
        adapter.pending_upper_completion = None;
        if let Some(upper) = adapter.upper_layer.take() {
            player.stop(upper.node);
            if upper.base_masked || adapter.body_shape.upper_masked {
                switch_tutorial_composed_pose_mask(
                    &mut adapter, &mut player, &mut transitions, false,
                );
            }
        }
    }
    if !queue.is_empty() {
        return;
    }
    // Requests can coincide with equipment or choreography commands. If an
    // earlier renderer batch ever lost/coalesced a base transition, converge
    // back to the one centralized action state instead of trusting the stale
    // adapter clip forever.
    if !adapter.emote_state && !base_clip_finished && adapter.active_clip != authoritative_clip {
        adapter.pending_base_completion =
            pending_legacy_visual_completion(authoritative_visual_clip, authoritative_clip);
        queue.push_animation(TutorialPlayerAnimationRequest::runtime_cross_fade(
            selected.gender,
            authoritative_clip,
        ));
        return;
    }
    if !adapter.emote_state && !upper_clip_finished && !finish_upper_fade_out {
        if let Some(semantic_clip) = action_state.upper_action {
            let Some(clip) = legacy_visual_tutorial_clip(
                semantic_clip,
                weapon_profile,
                selected.tutorial_stand_semantics,
            ) else {
                completions.push(LegacyVisualCompletion {
                    actor: selected.controller_root,
                    clip: semantic_clip,
                });
                return;
            };
            let expected_node = adapter.nodes.get(&clip).copied();
            let upper_is_live = adapter.upper_layer.is_some_and(|upper| {
                Some(upper.node) == expected_node && player.animation(upper.node).is_some()
            });
            if !upper_is_live {
                adapter.pending_upper_completion = Some(PendingLegacyVisualCompletion {
                    semantic_clip,
                    animation_clip: clip,
                    armed: false,
                });
                queue.push_animation(TutorialPlayerAnimationRequest::runtime_cross_fade(
                    selected.gender,
                    clip,
                ));
                return;
            }
        } else {
            // The action owner clears a stale attack on the combat-end edge.
            // Drop its renderer completion too: otherwise this pending token
            // keeps the upper pose alive over the first post-combat walk.
            adapter.pending_upper_completion = None;
            if let Some(upper) = adapter.upper_layer.take() {
                player.stop(upper.node);
                if upper.base_masked || adapter.body_shape.upper_masked {
                    switch_tutorial_composed_pose_mask(
                        &mut adapter,
                        &mut player,
                        &mut transitions,
                        false,
                    );
                }
            }
        }
    }
    // `AvatarStand` refuses to replace a direct AvatarEmote. Only an actual
    // directional move or jump calls EndEmote; the collider-loading sentinel
    // does not interrupt staying/standup.
    let emote_interrupt_requested = collision_pending.is_none()
        && tutorial_emote_interrupt_requested(
            controller.current_direction_key(),
            controller.jumping,
        );
    let startup_loading_barrier = tutorial_startup_emote_loading_barrier(
        selected.tutorial_stand_semantics,
        collision_pending.is_some(),
        adapter.active_clip,
    );
    if adapter.emote_state && !emote_interrupt_requested {
        if let Some(events) = emote_events(selected.gender, adapter.active_clip) {
            let elapsed = player
                .animation(adapter.active_node)
                .map_or(0.0, |active| active.seek_time());
            let nano_present = nanos
                .iter()
                .any(|nano| nano.owner == selected.controller_root);
            for sound in adapter.emote_cursor.sounds(&events, elapsed) {
                // The animation handler suppresses avatar vocals while a Nano
                // owns the emote voice; dance accompaniment still plays.
                if (!nano_present || sound.contains("_SFX_Dance"))
                    && let Some(audio) = audio.as_mut()
                {
                    audio.queue_legacy_character_animation_sound(selected.controller_root, sound);
                }
            }
        }
        if active_clip_finished && !adapter.emote_cursor.waiting_for_echo {
            if let Some(code) = continuation.next_code(adapter.active_clip) {
                continuation.pending.push((selected.controller_root, code));
                if (22..=24).contains(&code) {
                    // Beach queues the same pose locally while its renewal
                    // packet is in flight. Dance waits for the chosen echo.
                    transitions
                        .play(
                            &mut player,
                            adapter.active_node,
                            Duration::from_secs_f32(0.15),
                        )
                        .set_repeat(RepeatAnimation::Never)
                        .resume();
                    adapter.emote_cursor = PlayerEmoteCursor::default();
                }
                adapter.emote_cursor.waiting_for_echo = true;
            }
        }
        if adapter.emote_cursor.waiting_for_echo {
            return;
        }
    }
    if emote_interrupt_requested || !adapter.emote_state {
        continuation
            .pending
            .retain(|(owner, _)| *owner != selected.controller_root);
    }
    if tutorial_emote_holds_locomotion(
        adapter.emote_state,
        emote_interrupt_requested,
        active_clip_finished,
        startup_loading_barrier,
    ) {
        return;
    }
    if adapter.emote_state {
        adapter.emote_state = false;
        adapter.hand_attachment_hidden = false;
        let clip = authoritative_clip;
        // Ordinary locomotion requests which arrived while the direct emote
        // owned the avatar were intentionally drained above. Recreate both
        // halves of the authoritative return here: the visible clip request
        // and, for a clamp such as JumpEnd, its completion ownership. Without
        // the latter the landing reaches its terminal sample but can never
        // notify LegacyAvatarActionState to advance back to Stand.
        adapter.pending_base_completion =
            pending_legacy_visual_completion(authoritative_visual_clip, clip);
        if active_clip_finished && !emote_interrupt_requested {
            queue.push_animation(TutorialPlayerAnimationRequest::end_animation_cross_fade(
                selected.gender,
                clip,
            ));
        } else {
            queue.push_animation(TutorialPlayerAnimationRequest::runtime_cross_fade(
                selected.gender,
                clip,
            ));
        }
    }
}

/// The unarmed upper event keeps its source timing/audio identity, but the
/// visible running swing is projected from the full stationary punch so both
/// states use the same arm motion. Weapon attacks retain their primary upper
/// clips.
pub(super) const fn tutorial_upper_pose_source(clip: TutorialPlayerClip) -> TutorialPlayerClip {
    match clip {
        TutorialPlayerClip::Attack1Upper => TutorialPlayerClip::Attack1,
        _ => clip,
    }
}

pub(super) fn add_tutorial_upper_clip(
    graph: &mut AnimationGraph,
    clip: Handle<AnimationClip>,
    mask: u64,
) -> AnimationNodeIndex {
    // This must stay directly below the root. A fixed-weight intermediate
    // Blend normalizes the ActiveAnimation weight internally and turns every
    // moving upper attack into a permanent 50/50 mix with locomotion.
    graph.add_clip_with_mask(clip, mask, 1.0, graph.root)
}

pub(super) fn apply_resolved_tutorial_animation(
    player: &mut AnimationPlayer,
    transitions: &mut AnimationTransitions,
    node: AnimationNodeIndex,
    resolved: ContractResolvedTutorialPlayerAnimation,
) {
    let duration = tutorial_animation_transition_duration(resolved.request.dispatch);
    let repeat = match resolved.playback {
        TutorialPlayerClipPlayback::Loop => RepeatAnimation::Forever,
        TutorialPlayerClipPlayback::Clamp => RepeatAnimation::Never,
    };
    // Keep the source dispatch intact even when the previous clamp just
    // completed. `cnAvatarAnimation.EndAnimation` deliberately calls
    // `CrossFade(next, 0.3f)` from that terminal state; replacing it with an
    // immediate switch creates the tutorial pose pop this adapter is meant to
    // avoid. Bevy keeps a finished, unpaused ActiveAnimation available as the
    // outgoing transition source, matching the legacy ownership contract.
    release_paused_tutorial_main_animation(player, transitions, node);
    transitions
        .play(player, node, duration)
        .set_repeat(repeat)
        .resume();
}

#[must_use]
pub(super) fn tutorial_animation_transition_duration(dispatch: TutorialPlayerAnimationDispatch) -> Duration {
    match dispatch {
        TutorialPlayerAnimationDispatch::PlayImmediate => Duration::ZERO,
        TutorialPlayerAnimationDispatch::CrossFade(duration) => duration,
    }
}

pub(super) fn restart_missing_tutorial_base_animation(
    player: &mut AnimationPlayer,
    transitions: &mut AnimationTransitions,
    node: AnimationNodeIndex,
    clip: TutorialPlayerClip,
) {
    release_paused_tutorial_main_animation(player, transitions, node);
    transitions
        .play(player, node, Duration::ZERO)
        .set_repeat(match clip.playback() {
            TutorialPlayerClipPlayback::Loop => RepeatAnimation::Forever,
            TutorialPlayerClipPlayback::Clamp => RepeatAnimation::Never,
        })
        .set_speed(1.0)
        .resume();
}

/// Bevy's `AnimationTransitions::play` does not fade out a *paused* main
/// animation: the outgoing node stays active at its old weight and is blended
/// into every later pose. A hit-stop pauses the attack base, so moving during
/// it froze the avatar in a partial attack. Resume it so the transition owns
/// its fade-out.
pub(super) fn release_paused_tutorial_main_animation(
    player: &mut AnimationPlayer,
    transitions: &AnimationTransitions,
    next: AnimationNodeIndex,
) {
    if let Some(main) = transitions.get_main_animation()
        && main != next
        && let Some(active) = player.animation_mut(main)
        && active.is_paused()
    {
        active.resume();
    }
}
