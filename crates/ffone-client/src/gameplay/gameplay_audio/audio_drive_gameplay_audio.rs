use super::*;

pub(super) fn drive_gameplay_audio(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    audio_sources: Res<Assets<AudioSource>>,
    (catalog, voice_language): (Res<NativeAudioCatalog>, Res<VoiceLanguage>),
    mix: Res<RetrobutionAudioMix>,
    tutorial_presentation: Option<Res<TutorialChoreographyPresentation>>,
    rigs: Query<&TutorialSelectedPlayerRig>,
    user_equip_ui: Option<Res<UserEquipUiState>>,
    local_players: Query<&GlobalTransform, With<LegacyPlayerController>>,
    spatial_listeners: Query<&GlobalTransform, With<SpatialListener>>,
    player_actions: Query<&LegacyAvatarActionState>,
    vehicle_contexts: Query<&crate::avatar_action::LegacyAvatarPresentationContext>,
    transforms: Query<&GlobalTransform>,
    active_sounds: Query<(Entity, Option<&SpatialAudioSink>), With<GameplaySfxAudio>>,
    mut runtime: ResMut<GameplayAudioRuntime>,
) {
    let delta_seconds = time.delta_secs().max(0.0);
    let mut seen_players = HashSet::new();
    let mut locomotion_events = Vec::new();
    let mut inventory_audio_starts = Vec::new();
    for rig in &rigs {
        let Ok(action) = player_actions.get(rig.controller_root) else {
            continue;
        };
        seen_players.insert(rig.controller_root);
        runtime.observe_player_death_pose(
            rig.controller_root,
            rig.gender,
            action.authoritative_visual_clip(),
        );
        let mounted = vehicle_contexts.get(rig.controller_root).is_ok_and(|p| {
            p.mounted_vehicle != crate::avatar_action::LegacyVehiclePresentationFamily::None
        });
        if mounted {
            runtime.player_locomotion.remove(&rig.controller_root);
        } else {
            let cursor = runtime.player_locomotion.get_mut(&rig.controller_root);
            if let Some(cursor) = cursor
                && cursor.locomotion == action.locomotion
            {
                cursor.elapsed_seconds += delta_seconds;
                if let Some(next_event) = cursor.next_loop_event_seconds
                    && cursor.elapsed_seconds >= next_event
                {
                    let duration = player_locomotion_loop_duration(rig.gender, action.locomotion)
                        .expect("loop audio cursor requires a duration");
                    let crossings =
                        ((cursor.elapsed_seconds - next_event) / duration).floor() as u32 + 1;
                    cursor.next_loop_event_seconds = Some(next_event + duration * crossings as f32);
                    if let Some(cue) = player_locomotion_cue(rig.gender, action.locomotion) {
                        locomotion_events.extend(std::iter::repeat_n(
                            (rig.controller_root, 0.0, cue),
                            crossings as usize,
                        ));
                    }
                }
            } else {
                let loop_duration = player_locomotion_loop_duration(rig.gender, action.locomotion);
                runtime.player_locomotion.insert(
                    rig.controller_root,
                    PlayerLocomotionAudioCursor {
                        locomotion: action.locomotion,
                        elapsed_seconds: 0.0,
                        next_loop_event_seconds: loop_duration.map(|duration| duration + 0.25),
                    },
                );
                if let Some(cue) = player_locomotion_cue(rig.gender, action.locomotion) {
                    locomotion_events.push((rig.controller_root, 0.25, cue));
                }
            }
        }
        let inventory_open = runtime.service_inventory_audio_active || user_equip_ui
            .as_deref()
            .is_some_and(UserEquipUiState::is_active);
        if inventory_open {
            if !runtime
                .player_inventory_audio
                .contains_key(&rig.controller_root)
            {
                inventory_audio_starts.push(rig.controller_root);
            }
        } else if let Some(audio) = runtime.player_inventory_audio.remove(&rig.controller_root) {
            commands.entity(audio).despawn();
        }
    }
    runtime
        .player_locomotion
        .retain(|entity, _| seen_players.contains(entity));
    runtime
        .player_death_poses
        .retain(|entity| seen_players.contains(entity));
    let stale_inventory_audio = runtime
        .player_inventory_audio
        .keys()
        .copied()
        .filter(|entity| !seen_players.contains(entity))
        .collect::<Vec<_>>();
    for player in stale_inventory_audio {
        if let Some(audio) = runtime.player_inventory_audio.remove(&player) {
            commands.entity(audio).despawn();
        }
    }
    for (player, delay, cue) in locomotion_events {
        runtime.schedule(player, delay, cue);
    }

    for player in inventory_audio_starts {
        let candidates = catalog
            .by_true_name("InvenLooping")
            .into_iter()
            .filter(|audio| audio.category == NativeAudioCategory::Sfx)
            .collect::<Vec<_>>();
        let [audio] = candidates.as_slice() else {
            warn!(
                "UserEquip InvenLooping resolved to {} native SFX assets",
                candidates.len()
            );
            continue;
        };
        let entity = commands
            .spawn((
                Name::new("UserEquip InvenLooping audio"),
                GameplayAudioChannel::new(NativeAudioCategory::Sfx, 0.7),
                GameplaySfxAudio,
                AudioPlayer::new(asset_server.load(audio.path.clone())),
                PlaybackSettings::LOOP
                    .with_volume(Volume::Linear(0.7 * mix.effects.clamp(0.0, 1.0))),
            ))
            .id();
        runtime.player_inventory_audio.insert(player, entity);
    }

    let mut scheduled = std::mem::take(&mut runtime.scheduled);
    let mut retained = Vec::with_capacity(scheduled.len());
    let mut due = Vec::new();
    runtime
        .active_npc_voices
        .retain(|_, source| active_sounds.contains(*source));
    for (anchor, request) in std::mem::take(&mut runtime.npc_voice_changes) {
        runtime.pending_npc_voices.remove(&anchor);
        if let Some(true_name) = request {
            due.push((
                anchor,
                SelectedSound {
                    true_name,
                    voice_route: true,
                    npc_dialogue: true,
                    random_voice: true,
                },
                true,
            ));
        } else if let Some(previous) = runtime.active_npc_voices.remove(&anchor) {
            commands.entity(previous).despawn();
        }
    }
    runtime.closing_npc_dialogues.clear();
    runtime
        .pending_npc_voices
        .retain(|anchor, _| transforms.contains(*anchor));
    due.extend(
        runtime
            .pending_npc_voices
            .iter()
            .map(|(anchor, (selected, _))| (*anchor, selected.clone(), true)),
    );
    for mut sound in scheduled.drain(..) {
        if transforms.get(sound.anchor).is_err() {
            continue;
        }
        sound.remaining_seconds -= delta_seconds;
        if sound.remaining_seconds > 0.0 {
            retained.push(sound);
            continue;
        }
        due.push((
            sound.anchor,
            sound.cue.choose(&mut runtime.random),
            sound.priority,
        ));
    }
    runtime.scheduled = retained;

    let mut scheduled_named = std::mem::take(&mut runtime.scheduled_named);
    let mut retained_named = Vec::with_capacity(scheduled_named.len());
    for mut sound in scheduled_named.drain(..) {
        if transforms.get(sound.anchor).is_err() {
            continue;
        }
        sound.remaining_seconds -= delta_seconds;
        if sound.remaining_seconds > 0.0 {
            retained_named.push(sound);
            continue;
        }
        due.push((
            sound.anchor,
            SelectedSound {
                true_name: sound.true_name,
                voice_route: false,
                npc_dialogue: false,
                random_voice: false,
            },
            false,
        ));
    }
    runtime.scheduled_named = retained_named;

    due.extend(
        std::mem::take(&mut runtime.animation_sounds)
            .into_iter()
            .filter(|sound| transforms.get(sound.anchor).is_ok())
            .map(|sound| {
                (
                    sound.anchor,
                    SelectedSound {
                        true_name: sound.true_name,
                        voice_route: sound.voice_route,
                        npc_dialogue: false,
                        random_voice: sound.random_voice,
                    },
                    false,
                )
            }),
    );

    let player_listener = rigs
        .iter()
        .find_map(|rig| transforms.get(rig.controller_root).ok())
        .map(GlobalTransform::translation)
        .or_else(|| {
            local_players
                .iter()
                .next()
                .map(GlobalTransform::translation)
        });
    let fallback_listener = spatial_listeners
        .iter()
        .next()
        .map(GlobalTransform::translation);
    let spatial_audio_target = tutorial_presentation
        .as_deref()
        .map_or(SpatialAudioTarget::Player, |presentation| {
            presentation.spatial_audio_target
        });
    let mut active_sound_count = active_sounds.iter().count();
    for selected in std::mem::take(&mut runtime.ui_sounds) {
        if active_sound_count > 12 {
            continue;
        }
        let candidates = catalog
            .by_true_name(&selected.true_name)
            .into_iter()
            .filter(|audio| audio.category == NativeAudioCategory::Sfx)
            .collect::<Vec<_>>();
        let [audio] = candidates.as_slice() else {
            warn!(
                "UI audio {:?} resolved to {} native SFX assets",
                selected.true_name,
                candidates.len()
            );
            continue;
        };
        commands.spawn((
            Name::new(format!("Retrobution UI SFX {}", selected.true_name)),
            GameplayAudioChannel::new(NativeAudioCategory::Sfx, selected.clean_gain),
            GameplaySfxAudio,
            AudioPlayer::new(asset_server.load(audio.path.clone())),
            PlaybackSettings { speed: selected.pitch, ..PlaybackSettings::DESPAWN }.with_volume(Volume::Linear(
                selected.clean_gain * mix.effects.clamp(0.0, 1.0),
            )),
        ));
        active_sound_count += 1;
    }
    for (anchor, mut selected, priority) in due {
        let audio = {
            let localized_take = selected
                .random_voice
                .then(|| {
                    catalog.choose_voice_family(
                        &selected.true_name,
                        &voice_language.effective,
                        runtime.random.next_u32(),
                    )
                })
                .flatten();
            let mut candidates = localized_take
                .map_or_else(|| catalog.by_true_name(&selected.true_name), |a| vec![a]);
            if candidates.is_empty() {
                candidates = semantic_numeric_family(&catalog, &selected.true_name);
            }
            let preferred_category = if selected.voice_route {
                NativeAudioCategory::Voice
            } else {
                NativeAudioCategory::Sfx
            };
            let category_matches = candidates
                .iter()
                .copied()
                .filter(|audio| audio.category == preferred_category)
                .collect::<Vec<_>>();
            let preferred = if preferred_category == NativeAudioCategory::Sfx {
                let primary_runtime_routes = category_matches
                    .iter()
                    .copied()
                    .filter(|audio| {
                        !audio.logical_key.starts_with("sfx/character_creation/")
                            && !audio.logical_key.contains("/zone_local/")
                            && !audio.logical_key.contains("/tutorial_audio/")
                    })
                    .collect::<Vec<_>>();
                if primary_runtime_routes.is_empty() {
                    category_matches
                } else {
                    primary_runtime_routes
                }
            } else {
                category_matches
            };
            let fallback_sfx =
                if preferred_category == NativeAudioCategory::Voice && preferred.is_empty() {
                    let category_matches = candidates
                        .iter()
                        .copied()
                        .filter(|audio| audio.category == NativeAudioCategory::Sfx)
                        .collect::<Vec<_>>();
                    let primary_runtime_routes = category_matches
                        .iter()
                        .copied()
                        .filter(|audio| {
                            !audio.logical_key.starts_with("sfx/character_creation/")
                                && !audio.logical_key.contains("/zone_local/")
                                && !audio.logical_key.contains("/tutorial_audio/")
                        })
                        .collect::<Vec<_>>();
                    if primary_runtime_routes.is_empty() {
                        category_matches
                    } else {
                        primary_runtime_routes
                    }
                } else {
                    Vec::new()
                };
            match (
                preferred.as_slice(),
                fallback_sfx.as_slice(),
                candidates.as_slice(),
            ) {
                ([audio], _, _) => *audio,
                ([audio, ..], _, _) => {
                    // A primary route plus explicit alternate take shares the
                    // Unity true name. Catalog order keeps the primary take
                    // first; zone-local and tutorial duplicates are filtered
                    // above for this general world route.
                    *audio
                }
                ([], [audio, ..], _) => *audio,
                ([], [], [audio]) => *audio,
                _ => {
                    warn!(
                        "Gameplay audio {:?} resolved to {} native assets and no deterministic character-audio route",
                        selected.true_name,
                        candidates.len()
                    );
                    continue;
                }
            }
        };
        // Freeze the chosen localized take before a pending NPC load is retried.
        if selected.random_voice {
            selected.true_name.clone_from(&audio.true_name);
            selected.random_voice = false;
        }
        let Ok(transform) = transforms.get(anchor) else {
            continue;
        };
        // Clean SoundUtil permits one more source while the count equals its
        // nominal maximum (`> 12`, not `>= 12`).
        if !priority && active_sound_count > 12 {
            continue;
        }
        let (listener, maximum_distance) = legacy_spatial_sound_gate(
            spatial_audio_target,
            player_listener,
            fallback_listener,
            active_sound_count,
        );
        if !priority
            && listener.is_some_and(|listener| {
                transform.translation().distance(listener) >= maximum_distance
            })
        {
            continue;
        }
        if audio.category == NativeAudioCategory::Voice && audio.owner.starts_with("nano_") {
            runtime.nano.queue(anchor, audio.logical_key.clone());
            continue;
        }
        let path = if audio.category == NativeAudioCategory::Voice {
            let Some(path) = catalog.path_for_locale(audio, &voice_language.effective) else {
                runtime.pending_npc_voices.remove(&anchor);
                continue;
            };
            path
        } else {
            audio.path.as_str()
        };
        let gain = if audio.category == NativeAudioCategory::Voice {
            mix.voice
        } else {
            mix.effects
        }
        .clamp(0.0, 1.0);
        let handle = asset_server.load(path.to_owned());
        if selected.npc_dialogue {
            // PlayVO waits for its clip before replacing the NPC's existing
            // source. A missing/failed farewell must not cut off the old line.
            if !audio_sources.contains(handle.id()) {
                if matches!(
                    asset_server.load_state(handle.id()),
                    bevy::asset::LoadState::Failed(_)
                ) {
                    runtime.pending_npc_voices.remove(&anchor);
                } else {
                    runtime
                        .pending_npc_voices
                        .insert(anchor, (selected, handle));
                }
                continue;
            }
            runtime.pending_npc_voices.remove(&anchor);
            if let Some(previous) = runtime.active_npc_voices.remove(&anchor) {
                commands.entity(previous).despawn();
            }
        }
        let mut spawned = commands.spawn((
            Name::new(format!("Gameplay character audio {}", audio.true_name)),
            GameplayAudioChannel::new(audio.category, 1.0),
            GameplaySfxAudio,
            ChildOf(anchor),
            Transform::IDENTITY,
            GlobalTransform::from_translation(transform.translation()),
            AudioPlayer::new(handle),
            PlaybackSettings::DESPAWN
                .with_volume(Volume::Linear(gain))
                .with_spatial(true)
                .with_spatial_scale(LEGACY_SPATIAL_SCALE),
        ));
        if audio.category == NativeAudioCategory::Voice {
            spawned.insert(LocalizedVoice::by_true_name(audio.true_name.clone()));
        }
        if selected.npc_dialogue {
            runtime.active_npc_voices.insert(anchor, spawned.id());
        }
        active_sound_count += 1;
    }
    let (listener, maximum_distance) = legacy_spatial_sound_gate(
        spatial_audio_target,
        player_listener,
        fallback_listener,
        active_sound_count,
    );
    runtime.nano.drive(
        &mut commands,
        &asset_server,
        &audio_sources,
        &catalog,
        &voice_language,
        &mix,
        &transforms,
        &active_sounds,
        listener,
        maximum_distance,
        active_sound_count,
    );
}

pub(super) fn legacy_spatial_sound_gate(
    target: SpatialAudioTarget,
    player_position: Option<Vec3>,
    camera_position: Option<Vec3>,
    active_sound_count: usize,
) -> (Option<Vec3>, f32) {
    match target {
        // SoundUtil.SetPlayer(player): IsPlayable uses the player and the
        // tighter 18/16 Unity-unit limits.
        SpatialAudioTarget::Player => (
            player_position.or(camera_position),
            if active_sound_count < 6 { 18.0 } else { 16.0 },
        ),
        // SoundUtil.SetPlayer(null): GListenerPosition/IsPlayable fall back
        // to Camera.main and widen the limits to 40/30. Tutorial choreography
        // already preserves every exact SetPlayer edge.
        SpatialAudioTarget::None => (
            camera_position,
            if active_sound_count < 6 { 40.0 } else { 30.0 },
        ),
    }
}
