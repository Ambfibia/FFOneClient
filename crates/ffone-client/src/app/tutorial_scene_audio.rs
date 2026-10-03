//! Tutorial cleanup, completion, scene timeline, voices and subtitles.

use super::runtime_status::RuntimeStatus;
use super::state::ClientState;
use super::tutorial_ambience::{
    TutorialCutsceneSfxAudio, TutorialLoopAudio, TutorialMusicAudio, TutorialSceneAudio,
    TutorialVoiceAudio,
};
use super::tutorial_session::{
    TutorialAuxiliaryPresentation, TutorialChoreographyExecution, TutorialMissionRuntime,
    TutorialSession,
};
use super::{LocalNetworkIdentity, LocalPlayer, WorldSliceEntity};
use bevy::{audio::Volume, ecs::system::SystemParam, prelude::*};
use ffone_client::{
    coordinates::unity_to_native_vector,
    gameplay_audio::{GameplaySfxAudio, RetrobutionAudioMix},
    localization::{Language, Localization, LocalizedVoice, localized_tutorial_scene_text},
    mission_ui::MissionUiModel,
    network::{NetworkBridge, NetworkCommand},
    semantic_audio::{NativeAudioAsset, NativeAudioCatalog, NativeAudioCategory},
    tutorial::TutorialScene,
    tutorial_choreography_runtime::{
        TutorialChoreographyIssueQueue, TutorialChoreographyPlayer,
        TutorialChoreographyPresentation,
    },
    tutorial_effects_runtime::TutorialEffectRuntime,
    tutorial_logic::{ClientPosition, TutorialLocaleBranch},
    tutorial_mission_content::TutorialMissionContent,
    tutorial_nano_presentation::TutorialNanoPresentationCommandQueue,
    tutorial_nanocom_message::{
        TutorialNanocomMessageContext, TutorialNanocomMessageQueue,
        TutorialNanocomMessageSoundQueue, TutorialNanocomMessageState,
    },
    tutorial_native_mechanics::TutorialNativeMechanics,
    tutorial_player_presentation::TutorialPlayerPresentationCommandQueue,
    tutorial_presenter::{TutorialAudioCueKind, tutorial_scene_presentation},
    tutorial_voice_subtitles::{
        TutorialVoiceSubtitleState, TutorialVoiceSubtitleUiContext, is_english_locale,
    },
};

pub(super) fn sync_tutorial_voice_subtitle_context(
    state: Res<State<ClientState>>,
    tutorial: Res<TutorialSession>,
    presentation: Res<TutorialChoreographyPresentation>,
    language: Res<Language>,
    mut context: ResMut<TutorialVoiceSubtitleUiContext>,
) {
    if *state.get() != ClientState::Tutorial {
        *context = TutorialVoiceSubtitleUiContext::default();
        context.set_locale(&language.effective);
        return;
    }
    context.event_scene = presentation.event_scene;
    context.scene =
        (tutorial.scene != TutorialScene::None).then_some(i32::from(tutorial.scene as u8));
    context.cinematic = presentation.cinematic;
    context.cinematic_alpha = presentation.cinematic_alpha;
    context.set_locale(&language.effective);
}

pub(super) const fn tutorial_exit_preserves_world_runtime(network_world_ready: bool) -> bool {
    network_world_ready
}

/// Tutorial effects are app-shared because ordinary-world NPC animation and
/// streamed map effects use the same validated runtime. A shard `WorldReady`
/// can create those owners before the queued Tutorial -> World transition is
/// applied, so that transition must not clear their pending/native instances.
pub(super) fn cleanup_tutorial_effect_runtime_unless_world_ready(
    mut runtime: ResMut<TutorialEffectRuntime>,
    network_players: Query<(), (With<LocalPlayer>, With<LocalNetworkIdentity>)>,
) {
    if !tutorial_exit_preserves_world_runtime(!network_players.is_empty()) {
        runtime.clear_scene_instances();
    }
}

#[derive(SystemParam)]
pub(super) struct TutorialCleanupRuntime<'w> {
    pub(super) choreography_player: ResMut<'w, TutorialChoreographyPlayer>,
    pub(super) choreography_presentation: ResMut<'w, TutorialChoreographyPresentation>,
    pub(super) choreography_execution: ResMut<'w, TutorialChoreographyExecution>,
    pub(super) choreography_issues: ResMut<'w, TutorialChoreographyIssueQueue>,
    pub(super) mission_runtime: ResMut<'w, TutorialMissionRuntime>,
    pub(super) mission_model: ResMut<'w, MissionUiModel>,
    pub(super) auxiliary_presentation: ResMut<'w, TutorialAuxiliaryPresentation>,
    pub(super) native_mechanics: ResMut<'w, TutorialNativeMechanics>,
    pub(super) voice_subtitles: ResMut<'w, TutorialVoiceSubtitleState>,
    pub(super) voice_subtitle_context: ResMut<'w, TutorialVoiceSubtitleUiContext>,
    pub(super) nano_commands: ResMut<'w, TutorialNanoPresentationCommandQueue>,
    pub(super) player_commands: ResMut<'w, TutorialPlayerPresentationCommandQueue>,
    pub(super) nanocom_messages: ResMut<'w, TutorialNanocomMessageQueue>,
    pub(super) nanocom_state: ResMut<'w, TutorialNanocomMessageState>,
    pub(super) nanocom_context: ResMut<'w, TutorialNanocomMessageContext>,
    pub(super) nanocom_sounds: ResMut<'w, TutorialNanocomMessageSoundQueue>,
}

pub(super) fn cleanup_tutorial_runtime(
    mut runtime: TutorialCleanupRuntime,
    network_players: Query<(), (With<LocalPlayer>, With<LocalNetworkIdentity>)>,
) {
    let preserve_world_runtime = tutorial_exit_preserves_world_runtime(!network_players.is_empty());
    runtime.choreography_player.reset();
    runtime.choreography_presentation.reset();
    *runtime.choreography_execution = TutorialChoreographyExecution::default();
    runtime.choreography_issues.take_all();
    *runtime.mission_runtime = TutorialMissionRuntime::default();
    if !preserve_world_runtime {
        *runtime.mission_model = MissionUiModel::default();
    }
    *runtime.auxiliary_presentation = TutorialAuxiliaryPresentation::default();
    runtime.native_mechanics.reset();
    runtime.voice_subtitles.clear();
    *runtime.voice_subtitle_context = TutorialVoiceSubtitleUiContext::default();
    runtime.nano_commands.destroy();
    if !preserve_world_runtime {
        runtime.player_commands.clear();
    }
    runtime.nanocom_messages.clear();
    runtime.nanocom_state.clear();
    *runtime.nanocom_context = TutorialNanocomMessageContext::default();
    *runtime.nanocom_sounds = TutorialNanocomMessageSoundQueue::default();
}

pub(super) fn tutorial_client_position_to_native(position: ClientPosition) -> Vec3 {
    unity_to_native_vector(Vec3::new(
        position.x as f32 / 100.0,
        position.y as f32 / 100.0,
        position.z as f32 / 100.0,
    ))
}

pub(super) fn request_tutorial_completion(
    bridge: &NetworkBridge,
    tutorial: &mut TutorialSession,
    runtime: &mut RuntimeStatus,
    next_state: &mut NextState<ClientState>,
) {
    if tutorial.completion_requested {
        return;
    }
    let Some(pc_uid) = tutorial.character().map(|character| character.pc_uid) else {
        recover_tutorial_completion_to_character_selection(
            tutorial,
            runtime,
            next_state,
            "Tutorial completion lost the selected character; choose a character to retry",
        );
        return;
    };
    match bridge.send(NetworkCommand::CompleteTutorial { pc_uid }) {
        Ok(()) => {
            tutorial.completion_requested = true;
            runtime.message =
                "Saving tutorial completion and entering the OpenFusion world...".to_owned();
        }
        Err(error) => recover_tutorial_completion_to_character_selection(
            tutorial,
            runtime,
            next_state,
            format!(
                "Tutorial completion could not reach the network worker; choose a character to retry: {error}"
            ),
        ),
    }
}

pub(super) fn recover_tutorial_completion_to_character_selection(
    tutorial: &mut TutorialSession,
    runtime: &mut RuntimeStatus,
    next_state: &mut NextState<ClientState>,
    message: impl Into<String>,
) {
    tutorial.clear();
    runtime.clear_world();
    runtime.message = message.into();
    next_state.set(ClientState::CharacterSelect);
}

#[allow(clippy::too_many_arguments)]
pub(super) fn play_tutorial_scene_timeline(
    commands: &mut Commands,
    asset_server: &AssetServer,
    catalog: &NativeAudioCatalog,
    mix: &RetrobutionAudioMix,
    localization: &Localization,
    language: &Language,
    voice_locale: &str,
    scene: TutorialScene,
    elapsed: f32,
    cursor: &mut usize,
    runtime: &mut RuntimeStatus,
    tutorial_content: &TutorialMissionContent,
    voice_subtitles: &mut TutorialVoiceSubtitleState,
    mission_runtime: &mut TutorialMissionRuntime,
    now_seconds: f64,
    tutorial_voices: &Query<Entity, With<TutorialVoiceAudio>>,
    tutorial_loops: &Query<(Entity, &TutorialLoopAudio)>,
) {
    let Some(presentation) = tutorial_scene_presentation(scene) else {
        return;
    };
    // A long frame can cross both the StartLoop and StopLoops cues before
    // deferred Commands become visible to the query. Keep the entities spawned
    // in this invocation so the stop cue still has exact ownership.
    let mut loops_spawned_this_frame = Vec::new();
    while let Some(cue) = presentation.audio.get(*cursor)
        && elapsed >= cue.seconds
    {
        match cue.kind {
            TutorialAudioCueKind::Voice => {
                stop_tutorial_voice(commands, tutorial_voices);
                match tutorial_presenter_voice_path(catalog, voice_locale, cue) {
                    Ok(Some(path)) => {
                        let localized_voice = LocalizedVoice::by_true_name(
                            cue.semantic_path
                                .and_then(|semantic_path| catalog.by_path(semantic_path))
                                .map_or(cue.cue, |asset| asset.true_name.as_str()),
                        );
                        commands.spawn((
                            WorldSliceEntity,
                            TutorialVoiceAudio,
                            TutorialSceneAudio,
                            localized_voice,
                            AudioPlayer::new(asset_server.load(path)),
                            PlaybackSettings::DESPAWN
                                .with_volume(Volume::Linear(mix.voice.clamp(0.0, 1.0))),
                        ));
                    }
                    Ok(None) => {}
                    Err(error) => runtime.message = error,
                }
                start_tutorial_voice_subtitle(
                    voice_subtitles,
                    tutorial_content,
                    mission_runtime,
                    runtime,
                    localization,
                    language,
                    cue.cue,
                    now_seconds,
                );
            }
            TutorialAudioCueKind::OneShot => {
                let asset = match tutorial_sound_asset(catalog, cue) {
                    Ok(asset) => asset,
                    Err(_) => {
                        runtime.message = format!(
                            "Tutorial cutscene SFX {:?} is absent from the semantic catalog",
                            cue.cue
                        );
                        *cursor += 1;
                        continue;
                    }
                };
                let Some(path) = tutorial_sound_path_for_locale(catalog, asset, voice_locale)
                else {
                    // A performance with no take in this language stays silent.
                    *cursor += 1;
                    continue;
                };
                let music = tutorial_audio_cue_is_music(cue.cue);
                let performance = asset.category == NativeAudioCategory::Voice;
                let gain = if music {
                    mix.music
                } else if performance {
                    mix.voice
                } else {
                    mix.effects
                }
                .clamp(0.0, 1.0);
                let mut entity = commands.spawn((
                    Name::new(format!("Tutorial cutscene audio {}", cue.cue)),
                    WorldSliceEntity,
                    TutorialSceneAudio,
                    AudioPlayer::new(asset_server.load(path)),
                    PlaybackSettings::DESPAWN.with_volume(Volume::Linear(gain)),
                ));
                if music {
                    entity.insert(TutorialMusicAudio);
                } else {
                    entity.insert((TutorialCutsceneSfxAudio, GameplaySfxAudio));
                }
                if performance {
                    // Follows the voice-language switch like any other line.
                    entity.insert(LocalizedVoice::by_true_name(asset.true_name.clone()));
                }
            }
            TutorialAudioCueKind::StartLoop => {
                let already_playing = tutorial_loops
                    .iter()
                    .any(|(_, marker)| marker.cue == cue.cue)
                    || loops_spawned_this_frame
                        .iter()
                        .any(|(_, spawned_cue)| *spawned_cue == cue.cue);
                if already_playing {
                    *cursor += 1;
                    continue;
                }
                let Ok(path) = tutorial_sound_asset(catalog, cue).and_then(|asset| {
                    tutorial_sound_path_for_locale(catalog, asset, voice_locale)
                        .ok_or_else(|| format!("tutorial loop {:?} has no take", cue.cue))
                }) else {
                    runtime.message = format!(
                        "Tutorial cutscene loop {:?} is absent from the semantic catalog",
                        cue.cue
                    );
                    *cursor += 1;
                    continue;
                };
                let entity = commands
                    .spawn((
                        Name::new(format!("Tutorial cutscene loop {}", cue.cue)),
                        WorldSliceEntity,
                        TutorialSceneAudio,
                        TutorialCutsceneSfxAudio,
                        GameplaySfxAudio,
                        TutorialLoopAudio { cue: cue.cue },
                        AudioPlayer::new(asset_server.load(path)),
                        PlaybackSettings::LOOP
                            .with_volume(Volume::Linear(mix.effects.clamp(0.0, 1.0))),
                    ))
                    .id();
                loops_spawned_this_frame.push((entity, cue.cue));
            }
            TutorialAudioCueKind::StopLoops => {
                for (entity, _) in tutorial_loops {
                    commands.entity(entity).despawn();
                }
                for (entity, _) in loops_spawned_this_frame.drain(..) {
                    commands.entity(entity).despawn();
                }
            }
        }
        *cursor += 1;
    }
}

/// A cutscene one-shot is a timeline decision, not a statement about the clip:
/// Fusion Buttercup's idle roar overlaps the line Dexter is speaking, yet it is
/// her performance and lives under `audio/voice`. Resolving returns the asset so
/// the caller can pick the bus and, for a performance, the listener's language.
pub(super) fn tutorial_sound_asset<'a>(
    catalog: &'a NativeAudioCatalog,
    cue: &ffone_client::tutorial_presenter::TutorialAudioCue,
) -> Result<&'a NativeAudioAsset, String> {
    // SFX no longer share one flat owner, so a cue's file is found by its Unity
    // true name rather than by rebuilding a path from the cue. A cue whose true
    // name is not unique still declares its exact route.
    let asset = match cue.semantic_path {
        Some(expected_path) => catalog.by_path(expected_path).ok_or_else(|| {
            format!(
                "semantic tutorial sound {:?} is missing at {expected_path:?}",
                cue.cue
            )
        })?,
        None => {
            let matches = catalog
                .by_true_name(cue.cue)
                .into_iter()
                .filter(|asset| {
                    matches!(
                        asset.category,
                        NativeAudioCategory::Sfx | NativeAudioCategory::Voice
                    )
                })
                .collect::<Vec<_>>();
            match matches.as_slice() {
                [asset] => *asset,
                [] => return Err(format!("semantic tutorial sound {:?} is missing", cue.cue)),
                _ => {
                    return Err(format!(
                        "semantic tutorial sound {:?} matches {} routes and needs an exact one",
                        cue.cue,
                        matches.len()
                    ));
                }
            }
        }
    };
    if !matches!(
        asset.category,
        NativeAudioCategory::Sfx | NativeAudioCategory::Voice
    ) || !asset.true_name.eq_ignore_ascii_case(cue.cue)
    {
        return Err(format!(
            "semantic tutorial sound {:?} resolved to {:?} ({:?})",
            cue.cue, asset.true_name, asset.category
        ));
    }
    Ok(asset)
}

/// The file for a cutscene one-shot in the listener's voice language. A
/// performance with no take for that language and no English fallback is
/// silent, exactly as a spoken line is.
pub(super) fn tutorial_sound_path_for_locale(
    catalog: &NativeAudioCatalog,
    asset: &NativeAudioAsset,
    voice_locale: &str,
) -> Option<String> {
    catalog
        .path_for_locale(asset, voice_locale)
        .map(str::to_owned)
}

pub(super) fn tutorial_audio_cue_is_music(cue: &str) -> bool {
    matches!(
        cue,
        "Flythru_Sting"
            | "FusionSpawns_Sting"
            | "Cyberus_Sting"
            | "PlanetFusion_Sting"
            | "InfectedZone_Sting"
            | "SCAMPER_Sting"
            | "TechWing_sting"
            | "DexCarrier_sting"
    )
}

pub(super) fn stop_tutorial_voice(
    commands: &mut Commands,
    _tutorial_voices: &Query<Entity, With<TutorialVoiceAudio>>,
) {
    // Resolve at command application time: a replacement/skip must also stop
    // voices spawned earlier in this frame, including still-loading clips.
    commands.queue(|world: &mut World| {
        let voices: Vec<_> = world
            .query_filtered::<Entity, With<TutorialVoiceAudio>>()
            .iter(world)
            .collect();
        for entity in voices {
            world.despawn(entity);
        }
    });
}

pub(super) fn spawn_tutorial_voice(
    commands: &mut Commands,
    asset_server: &AssetServer,
    catalog: &NativeAudioCatalog,
    voice_locale: &str,
    cue: &str,
) -> Result<(), String> {
    let Some(path) = tutorial_voice_path(catalog, voice_locale, cue)? else {
        return Ok(());
    };
    commands.spawn((
        WorldSliceEntity,
        TutorialVoiceAudio,
        LocalizedVoice::by_true_name(cue),
        AudioPlayer::new(asset_server.load(path)),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(1.0)),
    ));
    Ok(())
}

pub(super) fn start_tutorial_voice_subtitle(
    state: &mut TutorialVoiceSubtitleState,
    content: &TutorialMissionContent,
    mission_runtime: &mut TutorialMissionRuntime,
    runtime: &mut RuntimeStatus,
    localization: &Localization,
    language: &Language,
    cue: &str,
    now_seconds: f64,
) {
    let locale = if is_english_locale(&language.effective) {
        TutorialLocaleBranch::OriginalEnglish
    } else {
        TutorialLocaleBranch::Localized
    };
    match state.start_from_cue_with(cue, locale, now_seconds, |event, line| {
        let fallback = content.scene_text(event, line).ok()?;
        Some(localization.text(
            language,
            &localized_tutorial_scene_text(event, line, fallback),
        ))
    }) {
        Ok(Some(active)) => {
            mission_runtime.push_voice_chat_line(active.resolved.full_text.clone());
        }
        Ok(None) => {}
        Err(error) => {
            runtime.message = format!("Tutorial subtitle error for {cue:?}: {error}");
        }
    }
}

pub(super) fn tutorial_voice_path(
    catalog: &NativeAudioCatalog,
    language: &str,
    cue: &str,
) -> Result<Option<String>, String> {
    let candidates = catalog
        .by_true_name(cue)
        .into_iter()
        .filter(|asset| asset.category == NativeAudioCategory::Voice)
        .collect::<Vec<_>>();
    let [asset] = candidates.as_slice() else {
        return Err(format!(
            "Tutorial voice {cue:?} resolved to {} semantic voice assets; expected exactly one",
            candidates.len()
        ));
    };
    Ok(catalog.path_for_locale(asset, language).map(str::to_owned))
}

pub(super) fn tutorial_presenter_voice_path(
    catalog: &NativeAudioCatalog,
    language: &str,
    cue: &ffone_client::tutorial_presenter::TutorialAudioCue,
) -> Result<Option<String>, String> {
    let Some(semantic_path) = cue.semantic_path else {
        return tutorial_voice_path(catalog, language, cue.cue);
    };
    let Some(asset) = catalog.by_path(semantic_path) else {
        return Err(format!(
            "Tutorial voice {:?} is missing its proven semantic asset {semantic_path:?}",
            cue.cue
        ));
    };
    if asset.category != NativeAudioCategory::Voice {
        return Err(format!(
            "Tutorial voice {:?} resolved to non-voice semantic asset {semantic_path:?}",
            cue.cue
        ));
    }
    Ok(catalog.path_for_locale(asset, language).map(str::to_owned))
}

pub(super) fn tutorial_target_in_center(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    unity_target: Vec3,
    reference_width: f32,
    reference_height: f32,
) -> bool {
    let Some(viewport_size) = camera.logical_viewport_size() else {
        return false;
    };
    let Ok(point) =
        camera.world_to_viewport(camera_transform, unity_to_native_vector(unity_target))
    else {
        return false;
    };
    let width = reference_width.min(viewport_size.x);
    let height = if reference_height.is_finite() {
        reference_height.min(viewport_size.y)
    } else {
        viewport_size.y
    };
    let half = Vec2::new(width, height) * 0.5;
    let center = viewport_size * 0.5;
    point.x >= center.x - half.x
        && point.x <= center.x + half.x
        && point.y >= center.y - half.y
        && point.y <= center.y + half.y
}

pub(super) fn horizontal_distance(left: Vec3, right: Vec3) -> f32 {
    Vec2::new(left.x - right.x, left.z - right.z).length()
}

pub(super) fn shortest_angle_delta(left: f32, right: f32) -> f32 {
    ((left - right + 180.0).rem_euclid(360.0) - 180.0).abs()
}
