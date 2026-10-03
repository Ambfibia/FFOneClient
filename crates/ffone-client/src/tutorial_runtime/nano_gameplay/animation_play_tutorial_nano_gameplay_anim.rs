use super::*;

pub(super) const CALL_CLIP: &str = "call";

pub(super) const SKILL_CLIP: &str = "skill1";

pub(super) const CHEESE_SKILL1_ASSET_CLIP: &str = "Skill1";

/// Resolves the one clean-primary spelling divergence without making clip
/// lookup generally case-insensitive. Primary `Nano.resourceFile`
/// (`40_659_362` bytes, SHA-256
/// `f0601478456f803ce6a624e328685caa4f121b444dc8cb0909a6097e5e67895f`),
/// serialized asset `CustomAssetBundle-d44c8d83281084c7c860065e437b031e`,
/// exact route `nano/nano_cheese.kfm`, owns `AnimationClip` pathId `1118` as
/// `Skill1`. The checked `ffone.logical-model-source.v1` export and native GLB
/// retain that capitalization. Missing skill clips never authorize borrowing
/// a different skill; they also must not prevent the model's call from loading.
pub(super) fn gameplay_nano_asset_clip_name(
    nano_id: Option<i16>,
    model_path: Option<&str>,
    logical_clip: &'static str,
) -> &'static str {
    if nano_id == Some(CHEESE_NANO_ID)
        && model_path == Some(CHEESE_NANO_MODEL_PATH)
        && logical_clip == SKILL_CLIP
    {
        CHEESE_SKILL1_ASSET_CLIP
    } else if model_path == Some("characters/nanos/nano_ghostfreak/nano_ghostfreak.glb")
        && logical_clip == "stand1"
    {
        "Stand1"
    } else {
        logical_clip
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Component)]
pub(super) struct TutorialNanoSkillAnimationEvents {
    pub(super) fired_request_serial: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub(super) struct TutorialGameplayNanoAnimationPlayback {
    pub(super) generation: u64,
    pub(super) request_serial: u64,
    pub(super) clip: &'static str,
    pub(super) node: AnimationNodeIndex,
}

/// Replays the sound AnimationEvents embedded in every ordinary-world Nano
/// GLB. Buttercup retains its separately proven tutorial event contract; all
/// other equipped Nanos use their own native model metadata and semantic
/// catalog names instead of borrowing Buttercup's fixed cue list.
pub(super) fn emit_world_nano_animation_sounds(
    mut commands: Commands,
    state: Res<TutorialNanoGameplayState>,
    mut stand_random: ResMut<LegacyNanoStandRandomStream>,
    assets: Res<TutorialNanoGameplayAssets>,
    parents: Query<&ChildOf>,
    players: Query<(
        Entity,
        &AnimationPlayer,
        &TutorialGameplayNanoAnimationPlayback,
        Option<&WorldNanoAnimationSoundCursor>,
    )>,
    runtime: Option<ResMut<GameplayAudioRuntime>>,
) {
    let (Some(root), Some(loadout), Some(presentation), Some(sound_events), Some(mut runtime)) = (
        state.entity,
        state.loadout,
        state.world_presentation.as_ref(),
        assets.sound_events.as_deref(),
        runtime,
    ) else {
        return;
    };
    for (entity, player, applied, cursor) in &players {
        // The tutorial's fixed Buttercup call/skill1 producer owns these two
        // clips. Its other world clips still need their model sound events.
        if loadout.nano_id == TUTORIAL_BUTTERCUP_NANO_ID
            && matches!(applied.clip, CALL_CLIP | SKILL_CLIP)
        {
            continue;
        }
        if applied.generation != state.generation || !is_descendant_of(entity, root, &parents) {
            continue;
        }
        let Some(active) = player.animation(applied.node) else {
            continue;
        };
        let same_playback = cursor.is_some_and(|cursor| {
            cursor.generation == applied.generation
                && cursor.request_serial == applied.request_serial
                && cursor.clip == applied.clip
                && cursor.node == applied.node
                && active.completions() >= cursor.completions
        });
        let (previous_seek, previous_completions) = if same_playback {
            let cursor = cursor.expect("matching world Nano sound cursor must exist");
            (cursor.seek_time, cursor.completions)
        } else {
            (0.0, 0)
        };
        let asset_clip = gameplay_nano_asset_clip_name(
            Some(loadout.nano_id),
            Some(&presentation.model_path),
            applied.clip,
        );
        for event in sound_events.iter().filter(|event| event.clip == asset_clip) {
            let crossings = animation_event_crossings(
                previous_seek,
                previous_completions,
                active.seek_time(),
                active.completions(),
                active.repeat_mode(),
                event.time,
            );
            for _ in 0..crossings {
                if state
                    .animation
                    .allows_sound(&event.payload, &mut stand_random)
                {
                    runtime.queue_legacy_nano_animation_sound(root, &event.payload);
                }
            }
        }
        commands
            .entity(entity)
            .insert(WorldNanoAnimationSoundCursor {
                generation: applied.generation,
                request_serial: applied.request_serial,
                clip: applied.clip,
                node: applied.node,
                seek_time: active.seek_time(),
                completions: active.completions(),
            });
    }
}

pub(super) fn play_tutorial_nano_gameplay_animation(
    mut commands: Commands,
    content: Option<Res<crate::tutorial_mission_content::TutorialMissionContent>>,
    assets: Res<TutorialNanoGameplayAssets>,
    mut state: ResMut<TutorialNanoGameplayState>,
    mut stand_random: ResMut<LegacyNanoStandRandomStream>,
    parents: Query<&ChildOf>,
    mut scenes: Query<
        (Entity, &mut Visibility),
        (With<TutorialGameplayNanoScene>, Without<AnimationPlayer>),
    >,
    mut players: Query<(
        Entity,
        &mut AnimationPlayer,
        Option<&mut AnimationTransitions>,
        Option<&TutorialGameplayNanoAnimationPlayback>,
    )>,
    mut events: ResMut<TutorialNanoGameplayEventQueue>,
) {
    if !state.asset_contract_ready {
        return;
    }
    let (Some(root), Some(owner), Some(loadout), Some(graph)) = (
        state.entity,
        state.owner,
        state.loadout,
        assets.graph.clone(),
    ) else {
        return;
    };
    let generation = state.generation;
    let current_serial = state.animation.request_serial();
    let mut matched_request = false;
    let mut request_finished = true;
    for (entity, player, _, applied) in &mut players {
        if !is_descendant_of(entity, root, &parents) {
            continue;
        }
        let Some(applied) = applied.filter(|applied| {
            applied.generation == generation && applied.request_serial == current_serial
        }) else {
            continue;
        };
        matched_request = true;
        request_finished &= player
            .animation(applied.node)
            .is_some_and(|animation| animation.is_finished());
    }
    if matched_request && request_finished {
        let maximum_stamina = content
            .as_deref()
            .and_then(|content| content.gameplay_nano(loadout.nano_id))
            .map_or(TUTORIAL_BUTTERCUP_INITIAL_STAMINA, |nano| {
                i32::from(nano.max_stamina)
            });
        let low_stamina = state.stamina * 5 < maximum_stamina;
        state.animation.set_low_stamina(low_stamina);
        match state.animation.complete(&mut stand_random) {
            LegacyNanoCompletion::Despawn => {
                commands.entity(root).despawn();
                commands.insert_resource(TutorialNanoGameplayAssets::default());
                events.push(TutorialNanoGameplayEvent::Dismissed {
                    owner: Some(owner),
                    entity: root,
                });
                state.mark_absent();
                return;
            }
            LegacyNanoCompletion::Continue | LegacyNanoCompletion::SkillSpecialFinished => {
                state.applied_animation_request_serial = None;
            }
        }
    }

    // NanoAnimation.SetStandMotion keeps the random draw, then uses stand1
    // when that model does not own the selected idle variant. Resolve the
    // request itself so playback and AnimationEvent audio use the same clip.
    if state.world_presentation.is_some()
        && state.animation.mode() == LegacyNanoAnimationMode::Stand
        && state
            .animation
            .clip()
            .is_some_and(|clip| !assets.nodes.contains_key(clip))
    {
        state.animation.request(
            LegacyNanoAnimationMode::Stand,
            "stand1",
            LegacyAnimationBlend::CrossFade100Ms,
        );
    }

    let Some((clip, node)) = state
        .animation
        .clip()
        .and_then(|clip| assets.nodes.get_key_value(clip))
        .map(|(clip, node)| (*clip, *node))
    else {
        return;
    };
    let request_serial = state.animation.request_serial();
    let transition_duration = state.animation.blend().duration();
    let mut found_player = false;
    for (entity, mut player, transitions, applied) in &mut players {
        if !is_descendant_of(entity, root, &parents) {
            continue;
        }
        found_player = true;
        if applied.is_some_and(|applied| {
            applied.generation == generation && applied.request_serial == request_serial
        }) {
            continue;
        }

        if let Some(mut transitions) = transitions {
            transitions
                .play(&mut player, node, transition_duration)
                .set_repeat(RepeatAnimation::Never)
                .resume();
            commands.entity(entity).insert((
                AnimationGraphHandle(graph.clone()),
                TutorialGameplayNanoAnimationPlayback {
                    generation,
                    request_serial,
                    clip,
                    node,
                },
            ));
        } else {
            let mut transitions = AnimationTransitions::new();
            transitions
                // Unity's `NanoAnimation.Call()` always uses
                // `Animation.CrossFade("call", 0.1f)`, including the first
                // state after the asynchronously attached model appears.
                .play(&mut player, node, transition_duration)
                .set_repeat(RepeatAnimation::Never)
                .resume();
            commands.entity(entity).insert((
                AnimationGraphHandle(graph.clone()),
                transitions,
                TutorialGameplayNanoAnimationPlayback {
                    generation,
                    request_serial,
                    clip,
                    node,
                },
            ));
        }
    }
    if found_player {
        for (scene, mut visibility) in &mut scenes {
            if is_descendant_of(scene, root, &parents) {
                *visibility = Visibility::Inherited;
            }
        }
        state.applied_animation_request_serial = Some(request_serial);
        if (!state.requires_exact_face_texture() || state.face_texture_bound)
            && state.activated_generation != Some(generation)
        {
            state.activated_generation = Some(generation);
            state.status = TutorialNanoGameplayStatus::Ready;
            events.push(TutorialNanoGameplayEvent::Activated {
                owner,
                entity: root,
                nano_id: loadout.nano_id,
                skill_id: loadout.skill_id,
                stamina: state.stamina,
            });
        }
    }
}

pub(super) fn emit_tutorial_nano_animation_events(
    state: Res<TutorialNanoGameplayState>,
    mut random: ResMut<LegacyNanoStandRandomStream>,
    parents: Query<&ChildOf>,
    players: Query<(
        Entity,
        &AnimationPlayer,
        &TutorialGameplayNanoAnimationPlayback,
    )>,
    mut roots: Query<(&Transform, &mut TutorialNanoSkillAnimationEvents)>,
    mut events: ResMut<TutorialNanoGameplayEventQueue>,
) {
    let (Some(root), Some(loadout), Some(clip)) =
        (state.entity, state.loadout, state.animation.clip())
    else {
        return;
    };
    // Only Buttercup's recovered AnimationClip event ownership has exact
    // native SFX/voice/effect evidence. Other Nanos still animate, but never
    // borrow Buttercup's presentation events.
    if loadout.nano_id != TUTORIAL_BUTTERCUP_NANO_ID
        || (clip != CALL_CLIP
            && !(loadout.skill_id == TUTORIAL_BUTTERCUP_SKILL_ID && clip == SKILL_CLIP))
    {
        return;
    }
    let event_seconds = match clip {
        CALL_CLIP => TUTORIAL_BUTTERCUP_SUMMON_EVENT_SECONDS,
        SKILL_CLIP => TUTORIAL_BUTTERCUP_SKILL_EVENT_SECONDS,
        _ => unreachable!("Nano animation event filter accepts only call and skill1"),
    };
    let request_serial = state.animation.request_serial();
    let reached_event = players.iter().any(|(entity, player, applied)| {
        is_descendant_of(entity, root, &parents)
            && applied.generation == state.generation
            && applied.request_serial == request_serial
            && applied.clip == clip
            && player
                .animation(applied.node)
                .is_some_and(|animation| animation.seek_time() >= event_seconds)
    });
    if !reached_event {
        return;
    }
    let Ok((transform, mut fired)) = roots.get_mut(root) else {
        return;
    };
    if fired.fired_request_serial == Some(request_serial) {
        return;
    }
    fired.fired_request_serial = Some(request_serial);
    let (source_clip_path_id, sfx, voice) = if clip == CALL_CLIP {
        (
            1049,
            TUTORIAL_BUTTERCUP_SUMMON_SFX_TRUE_NAME,
            TUTORIAL_BUTTERCUP_SUMMON_VOICE_TRUE_NAMES
                [random.next_index(TUTORIAL_BUTTERCUP_SUMMON_VOICE_TRUE_NAMES.len())],
        )
    } else {
        events.push(TutorialNanoGameplayEvent::TaggedEffectRequested {
            effect_id: TUTORIAL_BUTTERCUP_SKILL_EFFECT_ID,
            root,
            node_name: TUTORIAL_BUTTERCUP_SKILL_EFFECT_NODE,
            source_clip_path_id: 1050,
            source_event_seconds: TUTORIAL_BUTTERCUP_SKILL_EVENT_SECONDS,
        });
        (
            1050,
            TUTORIAL_BUTTERCUP_SKILL_SFX_TRUE_NAME,
            TUTORIAL_BUTTERCUP_SKILL_VOICE_TRUE_NAMES
                [random.next_index(TUTORIAL_BUTTERCUP_SKILL_VOICE_TRUE_NAMES.len())],
        )
    };
    let audio_events = if clip == CALL_CLIP {
        // Serialized call pathId 1049 lists the voice event before
        // `Nano Ability 06` at the same 0.25-second timestamp.
        [
            (voice, TutorialNanoGameplayAudioCategory::Voice),
            (sfx, TutorialNanoGameplayAudioCategory::Sfx),
        ]
    } else {
        [
            (sfx, TutorialNanoGameplayAudioCategory::Sfx),
            (voice, TutorialNanoGameplayAudioCategory::Voice),
        ]
    };
    for (true_name, category) in audio_events {
        events.push(TutorialNanoGameplayEvent::AudioRequested {
            true_name,
            category,
            position: transform.translation,
            source_clip_path_id,
            source_event_seconds: event_seconds,
        });
    }
}
