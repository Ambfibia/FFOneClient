use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn play_network_npc_animation_0104(
    mut commands: Commands,
    gltfs: Option<Res<Assets<Gltf>>>,
    clips: Option<Res<Assets<AnimationClip>>>,
    graphs: Option<ResMut<Assets<AnimationGraph>>>,
    animation_assets: Option<ResMut<NetworkNpcAnimationAssets0104>>,
    mut shared_random: Option<ResMut<LegacyNanoStandRandomStream>>,
    mut local_random: Local<LegacyNanoStandRandomStream>,
    mut layer_owners: Query<&mut NetworkNpcAnimationLayers0104>,
    parents: Query<&ChildOf>,
    roots: Query<(
        Entity,
        &NetworkNpcVisual0104,
        &NetworkNpcAppearance0104,
        Option<&NetworkNpcMotion0104>,
        Option<&NetworkNpcCombatAnimation0104>,
        Option<&NetworkNpcReadyAnimation0104>,
    )>,
    mut players: Query<(
        Entity,
        &mut AnimationPlayer,
        Option<&mut AnimationTransitions>,
        Option<&NetworkNpcAnimationApplied0104>,
        Option<&mut NetworkNpcIdleEventCursor0104>,
        Option<&mut NetworkNpcHighAnimations0104>,
    )>,
) {
    let (Some(gltfs), Some(clips), Some(mut graphs), Some(mut animation_assets)) =
        (gltfs, clips, graphs, animation_assets)
    else {
        return;
    };
    // Unity's NPC stand and melee draws use the process-global stream.
    let random = match shared_random.as_deref_mut() {
        Some(random) => random,
        None => &mut *local_random,
    };
    for (entity, mut player, mut transitions, applied, mut idle_cursor, mut high_animations) in
        &mut players
    {
        let Some((root, visual, appearance, motion, latest_combat, ready)) =
            network_npc_ancestor(entity, &parents, &roots)
        else {
            continue;
        };
        let Some(gltf) = gltfs.get(&visual.gltf) else {
            continue;
        };
        let gltf_id = visual.gltf.id();
        if !animation_assets.graphs.contains_key(&gltf_id) {
            // Target coverage reads every clip's curves; retry until loaded.
            let Some(prepared) = prepare_network_npc_animation_graph_0104(
                gltf.named_animations
                    .iter()
                    .map(|(name, clip)| (&**name, clip)),
                &clips,
                &mut graphs,
            ) else {
                continue;
            };
            animation_assets.graphs.insert(gltf_id, prepared);
        }
        let prepared = animation_assets
            .graphs
            .get(&gltf_id)
            .expect("network NPC graph was inserted");
        let mut inserted_transitions = None;
        let transitions = match transitions.as_deref_mut() {
            Some(transitions) => transitions,
            None => inserted_transitions.insert(AnimationTransitions::new()),
        };

        let mut layers = layer_owners.get_mut(root).ok();
        let mut inserted_high = NetworkNpcHighAnimations0104::default();
        let high = high_animations.as_deref_mut().unwrap_or(&mut inserted_high);
        let high_completed = if let Some(layers) = layers.as_deref_mut() {
            apply_network_npc_high_layers_0104(
                root,
                visual,
                appearance.0.hp > 0,
                layers,
                high,
                &mut player,
                prepared,
                random,
            )
        } else {
            false
        };
        if high_completed {
            commands.entity(root).insert(NetworkNpcReadyAnimation0104);
            if latest_combat.is_some_and(|request| request.clip.high_layer().is_some()) {
                commands
                    .entity(root)
                    .remove::<NetworkNpcCombatAnimation0104>();
            }
        }
        if high_animations.is_none()
            && (inserted_high.reset_revision != 0
                || inserted_high.applied.iter().any(Option::is_some))
        {
            commands.entity(entity).insert(inserted_high);
        }
        let combat = layers
            .as_deref()
            .and_then(|layers| layers.low.as_ref())
            .or_else(|| layers.is_none().then_some(latest_combat).flatten());
        let stand_attack = layers.as_deref().is_some_and(|layers| layers.stand_attack);
        let motion = motion.filter(|_| !stand_attack && !high_completed);
        let desired = match (
            appearance.0.hp,
            combat,
            motion,
            ready.is_some() || high_completed,
        ) {
            (hp, _, _, _) if hp <= 0 => "death",
            (_, Some(combat), _, _) => combat.clip.name(),
            (_, None, Some(motion), _) if motion.move_style == 0 => "walk",
            (_, None, Some(_), _) => "run",
            (_, None, None, true) => "ready",
            (_, None, None, false)
                if stand_attack && applied.is_some_and(|applied| applied.clip == "run") =>
            {
                "ready"
            }
            (_, None, None, false) => NETWORK_NPC_IDLE_REQUEST_0104,
        };
        let combat_revision = combat.map(|combat| combat.revision);
        if desired == "ready" && stand_attack {
            commands.entity(root).insert(NetworkNpcReadyAnimation0104);
        }

        if let Some(applied) = applied.filter(|applied| {
            applied.root == root
                && applied.requested_clip == desired
                && applied.combat_revision == combat_revision
        }) {
            if combat.is_some_and(|combat| !combat.clip.repeats()) {
                if network_npc_one_shot_completed_0104(&player, applied, visual) {
                    // Low-layer skill/corruption completion selects stand;
                    // only the current high-layer owner enters AttackReady.
                    if let Some(layers) = layers.as_deref_mut() {
                        layers.low = None;
                    }
                    let mut owner = commands.entity(root);
                    owner.remove::<NetworkNpcReadyAnimation0104>();
                    if latest_combat
                        .is_some_and(|request| Some(request.revision) == combat_revision)
                    {
                        owner.remove::<NetworkNpcCombatAnimation0104>();
                    }
                }
            } else if desired == NETWORK_NPC_IDLE_REQUEST_0104 {
                if player.animation(applied.node).is_none() {
                    // A lost idle node must not hold the model's last pose.
                    commands
                        .entity(entity)
                        .remove::<NetworkNpcAnimationApplied0104>();
                } else {
                    let (end_crossed, observed) = network_npc_idle_end_crossed_0104(
                        &player,
                        applied,
                        idle_cursor.as_deref(),
                        // Ambient gestures must finish their full authored
                        // cycle. Combat end events still own combat recovery.
                        None,
                    );
                    let next = end_crossed
                        .then(|| {
                            network_npc_stand_clip_0104(
                                random.next_index(100),
                                &applied.clip,
                                |clip| prepared.nodes.contains_key(clip),
                            )
                        })
                        .filter(|next| *next != applied.clip)
                        .and_then(|next| Some((next, *prepared.nodes.get(next)?)));
                    if let Some((next, node)) = next {
                        cross_fade_network_npc_low_layer_0104(
                            &mut player,
                            transitions,
                            node,
                            network_npc_cross_fade_0104(next),
                            1.0,
                            true,
                        );
                        commands
                            .entity(entity)
                            .insert(NetworkNpcAnimationApplied0104 {
                                clip: next.to_owned(),
                                node,
                                ..applied.clone()
                            });
                        store_network_npc_idle_cursor_0104(
                            &mut commands,
                            entity,
                            idle_cursor.as_deref_mut(),
                            network_npc_idle_cursor_0104(&player, node),
                        );
                    } else if let Some(observed) = observed {
                        // CrossFade to the playing stand keeps its time: the
                        // clip plays its tail and wraps naturally.
                        store_network_npc_idle_cursor_0104(
                            &mut commands,
                            entity,
                            idle_cursor.as_deref_mut(),
                            observed,
                        );
                    }
                }
            }
            stop_finished_network_npc_additive_pairs_0104(&mut player, prepared, applied.node);
            if let Some(transitions) = inserted_transitions {
                commands.entity(entity).insert(transitions);
            }
            continue;
        }

        let has_clip = |clip: &str| prepared.nodes.contains_key(clip);
        let resolved = match desired {
            NETWORK_NPC_IDLE_REQUEST_0104 => Some(network_npc_stand_clip_0104(
                random.next_index(100),
                applied.map_or("", |applied| applied.clip.as_str()),
                has_clip,
            )),
            "melee1" => Some(network_npc_melee_clip_0104(random.next_index(2), has_clip)),
            _ => Some(desired),
        }
        .filter(|clip| has_clip(clip))
        .or_else(|| {
            combat
                .and_then(|combat| combat.clip.fallback_name())
                .filter(|fallback| has_clip(fallback))
        })
        .or_else(|| {
            (desired != "death"
                && combat.is_none_or(|combat| combat.clip.repeats())
                && has_clip("stand1"))
            .then_some("stand1")
        });
        let Some(resolved) = resolved else {
            // A missing one-shot cannot be allowed to pin the state machine.
            // Release the missing low-layer skill into stand. Persistent
            // preparation and terminal death remain pending.
            if combat.is_some_and(|combat| !combat.clip.repeats()) {
                if let Some(layers) = layers.as_deref_mut() {
                    layers.low = None;
                }
                commands
                    .entity(root)
                    .remove::<NetworkNpcCombatAnimation0104>()
                    .remove::<NetworkNpcReadyAnimation0104>();
                commands
                    .entity(entity)
                    .remove::<NetworkNpcAnimationApplied0104>();
            }
            if let Some(transitions) = inserted_transitions {
                commands.entity(entity).insert(transitions);
            }
            continue;
        };
        let additive_pair = network_npc_clip_is_additive_0104(resolved)
            .then(|| prepared.additive_nodes.get(resolved).copied())
            .flatten();
        let mut playback_revision = combat_revision;
        let node = if let Some(pair) = additive_pair {
            // The high layer is composed over the playing low layer. A model
            // hit before its first low-layer state enters AttackReady/stand1
            // through the transition owner so that state can fade out later.
            if !prepared
                .base_nodes
                .iter()
                .any(|node| player.animation(*node).is_some())
                && let Some(fallback) = prepared.fallback_base_node
            {
                cross_fade_network_npc_low_layer_0104(
                    &mut player,
                    transitions,
                    fallback,
                    Duration::ZERO,
                    1.0,
                    true,
                );
            }
            restart_network_npc_additive_pair_0104(&mut player, prepared, pair);
            pair.clip
        } else {
            let node = prepared.nodes[resolved];
            let speed = match resolved {
                "walk" => visual.walk_animation_speed,
                "run" => visual.run_animation_speed,
                _ => 1.0,
            };
            let repeat = appearance.0.hp > 0 && combat.is_none_or(|combat| combat.clip.repeats());
            if transitions.get_main_animation() == Some(node)
                && player.animation(node).is_some_and(|active| {
                    !active.is_finished()
                        && active.repeat_mode()
                            == if repeat {
                                RepeatAnimation::Forever
                            } else {
                                RepeatAnimation::Never
                            }
                })
            {
                playback_revision = applied.and_then(|applied| applied.playback_revision);
            }
            cross_fade_network_npc_low_layer_0104(
                &mut player,
                transitions,
                node,
                network_npc_cross_fade_0104(resolved),
                speed,
                repeat,
            );
            node
        };
        stop_finished_network_npc_additive_pairs_0104(&mut player, prepared, node);
        commands.entity(entity).insert((
            AnimationGraphHandle(prepared.graph.clone()),
            NetworkNpcAnimationApplied0104 {
                root,
                requested_clip: desired.to_owned(),
                clip: resolved.to_owned(),
                node,
                additive: additive_pair.is_some(),
                combat_revision,
                playback_revision,
            },
        ));
        if desired == NETWORK_NPC_IDLE_REQUEST_0104 {
            store_network_npc_idle_cursor_0104(
                &mut commands,
                entity,
                idle_cursor.as_deref_mut(),
                network_npc_idle_cursor_0104(&player, node),
            );
        }
        if let Some(transitions) = inserted_transitions {
            commands.entity(entity).insert(transitions);
        }
    }
}

/// Exact non-HNPC `NpcAnimation.SetStandMotion` table: stand1/2/3/4 at
/// 40/30/20/10 percent. An alternate that repeats the current low clip, or is
/// absent from the model, becomes stand1.
#[must_use]
pub(super) fn network_npc_stand_clip_0104(
    roll: usize,
    current: &str,
    has_clip: impl Fn(&str) -> bool,
) -> &'static str {
    let clip = match roll {
        0..40 => "stand1",
        40..70 => "stand2",
        70..90 => "stand3",
        _ => "stand4",
    };
    if clip == current || !has_clip(clip) {
        "stand1"
    } else {
        clip
    }
}

/// `NpcAnimation.AttackMotion` draws `Random.Range(1, 3)` and falls back to
/// melee1 when the drawn clip is absent.
#[must_use]
pub(super) fn network_npc_melee_clip_0104(roll: usize, has_clip: impl Fn(&str) -> bool) -> &'static str {
    if roll == 1 && has_clip("melee2") {
        "melee2"
    } else {
        "melee1"
    }
}

pub(super) fn prepare_network_npc_animation_graph_0104<'a>(
    named_animations: impl IntoIterator<Item = (&'a str, &'a Handle<AnimationClip>)>,
    clips: &Assets<AnimationClip>,
    graphs: &mut Assets<AnimationGraph>,
) -> Option<NetworkNpcPreparedAnimationGraph0104> {
    let mut named_animations = named_animations
        .into_iter()
        .map(|(name, handle)| Some((name, handle, clips.get(handle)?)))
        .collect::<Option<Vec<_>>>()?;
    named_animations.sort_unstable_by(|(left, ..), (right, ..)| left.cmp(right));
    let uncovered = network_npc_uncovered_high_layer_targets_0104(&named_animations);
    let high_layer_mask: u64 = if uncovered.is_empty() {
        0
    } else {
        1 << NETWORK_NPC_UNCOVERED_HIGH_LAYER_GROUP_0104
    };
    let mut graph = AnimationGraph::new();
    for target in uncovered {
        graph.add_target_to_mask_group(target, NETWORK_NPC_UNCOVERED_HIGH_LAYER_GROUP_0104);
    }
    let composition = graph.add_additive_blend(1.0, graph.root);
    let base = graph.add_blend(1.0, composition);
    let mut nodes = HashMap::new();
    let mut additive_nodes = HashMap::new();
    let mut base_nodes = Vec::new();
    for (name, handle, _) in named_animations {
        let node = graph.add_clip(handle.clone(), 1.0, base);
        nodes.insert(name.to_owned(), node);
        if network_npc_clip_is_additive_0104(name) {
            // Created first so the Add node applies it before the clip.
            let reference =
                graph.add_clip_with_mask(handle.clone(), high_layer_mask, -1.0, composition);
            let clip = graph.add_clip_with_mask(handle.clone(), high_layer_mask, 1.0, composition);
            additive_nodes.insert(
                name.to_owned(),
                NetworkNpcAdditivePair0104 { reference, clip },
            );
        } else {
            base_nodes.push(node);
        }
    }
    let fallback_base_node = nodes.get("ready").or_else(|| nodes.get("stand1")).copied();
    Some(NetworkNpcPreparedAnimationGraph0104 {
        graph: graphs.add(graph),
        nodes,
        additive_nodes,
        base_nodes,
        fallback_base_node,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Component)]
pub(super) struct NetworkNpcAnimationEffectCursor0104 {
    pub(super) combat_revision: u64,
    pub(super) node: AnimationNodeIndex,
    pub(super) seek_time: f32,
    pub(super) completions: u32,
}

#[derive(Default, Component)]
pub(super) struct NetworkNpcAnimationEffectCursors0104(pub(super) [Option<NetworkNpcAnimationEffectCursor0104>; 3]);

pub(super) fn emit_network_npc_animation_effects_0104(
    mut commands: Commands,
    parents: Query<&ChildOf>,
    roots: Query<(Entity, &NetworkNpc0104, &NetworkNpcVisual0104)>,
    players: Query<(
        Entity,
        &AnimationPlayer,
        &NetworkNpcAnimationApplied0104,
        Option<&NetworkNpcHighAnimations0104>,
        Option<&NetworkNpcAnimationEffectCursors0104>,
    )>,
    effects: Option<ResMut<TutorialEffectRuntime>>,
) {
    let Some(mut effects) = effects else {
        return;
    };
    for (player_entity, player, low, high, cursors) in &players {
        let mut next_cursors = NetworkNpcAnimationEffectCursors0104::default();
        for (index, applied) in network_npc_playbacks_0104(low, high) {
            let cursor = cursors.and_then(|cursors| cursors.0[index].as_ref());
            let Some(combat_revision) = applied.playback_revision else {
                continue;
            };
            let Some(active) = player.animation(applied.node) else {
                continue;
            };
            let Some((root, npc, visual)) =
                network_npc_effect_ancestor(player_entity, &parents, &roots)
            else {
                continue;
            };
            if root != applied.root {
                continue;
            }
            let same_playback = cursor.is_some_and(|cursor| {
                cursor.combat_revision == combat_revision
                    && cursor.node == applied.node
                    && active.completions() >= cursor.completions
            });
            let (previous_seek, previous_completions) = if same_playback {
                let cursor = cursor.expect("matching network effect cursor must exist");
                (cursor.seek_time, cursor.completions)
            } else {
                (0.0, 0)
            };
            for event in visual
                .animation_effect_events
                .iter()
                .filter(|event| event.clip == applied.clip)
            {
                let crossings = animation_event_crossings(
                    previous_seek,
                    previous_completions,
                    active.seek_time(),
                    active.completions(),
                    active.repeat_mode(),
                    event.time,
                );
                for _ in 0..crossings {
                    let local_rotation_after_parenting = if event.node_name.is_some() {
                        Quat::from_rotation_x(90_f32.to_radians())
                    } else {
                        Quat::IDENTITY
                    };
                    effects.enqueue(TutorialEffectRuntimeCommand::Add {
                        effect_id: event.effect_id,
                        placement: TutorialEffectPlacement::ExactEntityBone {
                            root_entity: root,
                            node_name: event
                                .node_name
                                .clone()
                                .unwrap_or_else(|| visual.logical_name.clone()),
                            spawn_world_rotation: Quat::IDENTITY,
                            local_translation_after_parenting: Vec3::ZERO,
                            local_rotation_after_parenting,
                        },
                        scale: 1.0,
                        tracked: false,
                        name: Some(format!(
                            "AnimationEvent network NPC {} ES{}",
                            npc.npc_id, event.effect_id
                        )),
                        destroy_after_seconds: None,
                        source_line: line!(),
                    });
                }
            }
            next_cursors.0[index] = Some(NetworkNpcAnimationEffectCursor0104 {
                combat_revision,
                node: applied.node,
                seek_time: active.seek_time(),
                completions: active.completions(),
            });
        }
        commands.entity(player_entity).insert(next_cursors);
    }
}

pub(super) fn emit_network_hnpc_animation_sounds_0104(
    mut commands: Commands,
    rigs: Query<(
        Entity,
        &NetworkHnpcRig0104,
        &NativePlayerRigStand1Playback,
        &NativePlayerRigAnimationApplied,
        Option<&NetworkHnpcAnimationSoundCursor0104>,
    )>,
    roots: Query<(&GlobalTransform, &NetworkHnpcAnimationState0104)>,
    local_players: Query<&GlobalTransform, With<LegacyPlayerController>>,
    players: Query<&AnimationPlayer>,
    runtime: Option<ResMut<GameplayAudioRuntime>>,
) {
    let Some(mut runtime) = runtime else {
        return;
    };
    let listener = local_players
        .iter()
        .next()
        .map(GlobalTransform::translation);
    for (entity, rig, playback, applied, cursor) in &rigs {
        let Ok((transform, state)) = roots.get(rig.npc_root) else {
            continue;
        };
        if state.rig_root != entity {
            continue;
        }
        let Ok(player) = players.get(playback.animation_player) else {
            continue;
        };
        let Some(active) = player.animation(playback.animation_node) else {
            continue;
        };
        let same = cursor.is_some_and(|cursor| {
            cursor.revision == applied.revision && active.completions() >= cursor.completions
        });
        let (previous_seek, previous_completions) = if same {
            let cursor = cursor.unwrap();
            (cursor.seek_time, cursor.completions)
        } else {
            (-f32::EPSILON, 0)
        };
        let distant_idle = state.idle
            && listener.is_some_and(|listener| transform.translation().distance(listener) > 10.0);
        for event in rig
            .animation_sounds
            .iter()
            .filter(|event| event.clip == applied.clip)
        {
            // IsStand applies to the ambient state, including emotes. Dance SFX
            // bypass its ten-unit gate and retain the shared SoundUtil range.
            if distant_idle && !event.payload.contains("_SFX_Dance") {
                continue;
            }
            let count = animation_event_crossings(
                previous_seek,
                previous_completions,
                active.seek_time(),
                active.completions(),
                active.repeat_mode(),
                event.time,
            );
            for _ in 0..count {
                runtime.queue_legacy_character_animation_sound(rig.npc_root, &event.payload);
                debug!(
                    npc_type = rig.npc_type,
                    clip = applied.clip,
                    sound = event.payload,
                    seek = active.seek_time(),
                    completions = active.completions(),
                    rig_revision = applied.revision,
                    "HNPC animation sound"
                );
            }
        }
        if !same
            || cursor.is_none_or(|cursor| {
                cursor.seek_time != active.seek_time() || cursor.completions != active.completions()
            })
        {
            commands
                .entity(entity)
                .insert(NetworkHnpcAnimationSoundCursor0104 {
                    revision: applied.revision,
                    seek_time: active.seek_time(),
                    completions: active.completions(),
                });
        }
    }
}

/// Replays authored sound events for ordinary world NPCs, mobs and fusions.
pub(super) fn emit_network_npc_animation_sounds_0104(
    mut commands: Commands,
    parents: Query<&ChildOf>,
    roots: Query<(
        Entity,
        &NetworkNpc0104,
        &NetworkNpcVisual0104,
        &GlobalTransform,
    )>,
    local_players: Query<&GlobalTransform, With<LegacyPlayerController>>,
    players: Query<(
        Entity,
        &AnimationPlayer,
        &NetworkNpcAnimationApplied0104,
        Option<&NetworkNpcHighAnimations0104>,
        Option<&NetworkNpcAnimationSoundCursors0104>,
    )>,
    runtime: Option<ResMut<GameplayAudioRuntime>>,
) {
    let Some(mut runtime) = runtime else {
        return;
    };
    let listener = local_players
        .iter()
        .next()
        .map(GlobalTransform::translation);
    for (player_entity, player, low, high, cursors) in &players {
        let mut next_cursors = NetworkNpcAnimationSoundCursors0104::default();
        for (index, applied) in network_npc_playbacks_0104(low, high) {
            let cursor = cursors.and_then(|cursors| cursors.0[index].as_ref());
            let Some(active) = player.animation(applied.node) else {
                continue;
            };
            let Some((root, _npc, visual, root_transform)) =
                network_npc_sound_ancestor(player_entity, &parents, &roots)
            else {
                continue;
            };
            if root != applied.root {
                continue;
            }
            let same_playback = cursor.is_some_and(|cursor| {
                cursor.root == root
                    && cursor.clip == applied.clip
                    && cursor.combat_revision == applied.playback_revision
                    && cursor.node == applied.node
                    && active.completions() >= cursor.completions
            });
            let (previous_seek, previous_completions) = if same_playback {
                let cursor = cursor.expect("matching network sound cursor must exist");
                (cursor.seek_time, cursor.completions)
            } else {
                (0.0, 0)
            };
            // NpcAnimation.IsStand suppresses ordinary stand sounds beyond ten
            // Unity units. Other clips proceed to SoundUtil's shared 18/16-unit
            // admission rule in `drive_gameplay_audio`.
            let muted_stand = low.clip.starts_with("stand")
                && high.is_none_or(|high| {
                    high.applied.iter().flatten().all(|applied| {
                        player
                            .animation(applied.node)
                            .is_none_or(|active| active.is_finished())
                    })
                })
                && listener
                    .is_some_and(|listener| root_transform.translation().distance(listener) > 10.0);
            if !muted_stand {
                for event in visual
                    .animation_sound_events
                    .iter()
                    .filter(|event| event.clip == applied.clip)
                {
                    let crossings = animation_event_crossings(
                        previous_seek,
                        previous_completions,
                        active.seek_time(),
                        active.completions(),
                        active.repeat_mode(),
                        event.time,
                    );
                    for _ in 0..crossings {
                        runtime.queue_legacy_character_animation_sound(root, &event.payload);
                    }
                }
            }
            next_cursors.0[index] = Some(NetworkNpcAnimationSoundCursor0104 {
                root,
                clip: applied.clip.clone(),
                combat_revision: applied.playback_revision,
                node: applied.node,
                seek_time: active.seek_time(),
                completions: active.completions(),
            });
        }
        commands.entity(player_entity).insert(next_cursors);
    }
}

pub(super) fn network_pc_vehicle_animation_clip(
    state: RemoteAnimationState,
    equip_type: Option<i32>,
) -> Option<(&'static str, bool)> {
    let family = match equip_type? {
        1 => "board",
        2 | 3 => "scooter",
        _ => return None,
    };
    let suffix = match state {
        RemoteAnimationState::Moving { direction_key } if (4..=6).contains(&direction_key) => {
            "runback"
        }
        RemoteAnimationState::Moving { .. } => "run",
        RemoteAnimationState::Jumping { .. } => "jump",
        _ => "stand1",
    };
    let clip = TutorialPlayerClip::from_exact_name(&format!("{family}_{suffix}"))?;
    Some((clip.name(), true))
}
