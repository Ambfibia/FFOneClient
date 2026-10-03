//! Nano free-tuning production, preview animation and commit application.

use super::dexter_ship_drive::dexter_ship_descendant;
use super::local_inventory::LocalInventoryRuntime;
use super::nano_free_tuning::{
    NanoFreeTuningCursorSnapshot, NanoFreeTuningOpenTrigger, NanoFreeTuningPreview,
    NanoFreeTuningPreviewAnimation, NanoFreeTuningPreviewSoundCursor,
    NanoFreeTuningProductionInputs, NanoFreeTuningProductionRuntime, nano_free_tuning_audio_path,
    nano_free_tuning_creation_projectile_command,
};
use super::runtime_status::{RuntimeNanoSlot, RuntimeStatus, legacy_avatar_max_fusion_matter};
use super::tutorial_choreography::*;
use bevy::{
    animation::RepeatAnimation,
    gltf::Gltf,
    prelude::{AnimationTransitions, *},
    window::CursorGrabMode,
};
use ffone_client::{
    gameplay_audio::GameplayAudioRuntime,
    legacy_npc_nano_animation::LegacyNanoStandRandomStream,
    movement::advance_native_xorshift32,
    nano_free_tuning_runtime::NanoFreeTuningBank0104,
    nano_free_tuning_ui::{
        NanoFreeTuningCinematicIntent, NanoFreeTuningEffectIntent, NanoFreeTuningIntent,
        NanoFreeTuningModel, NanoFreeTuningPhase, NanoFreeTuningSoundIntent,
        NanoFreeTuningUiCommand, NanoFreeTuningUiCommandOutbox, NanoFreeTuningUiIntent,
        NanoFreeTuningWorldIntent, NanoTuneSuccess,
    },
    network::{NetworkBridge, NetworkCommand},
    tutorial_effects_runtime::{
        TutorialEffectPlacement, TutorialEffectRuntimeCommand, animation_event_crossings,
    },
};
use ffone_protocol::{
    ItemBase0104, NanoTuneRequest0104, NanoTuneSuccess0104, PcNanoCreateSuccess0104, WirePayload,
    packet,
};
use std::time::Duration;

pub(super) fn consume_nano_free_tuning_production(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    bridge: Res<NetworkBridge>,
    mut model: ResMut<NanoFreeTuningModel>,
    mut outbox: ResMut<NanoFreeTuningUiCommandOutbox>,
    mut inputs: NanoFreeTuningProductionInputs,
) {
    if inputs.resurrect_ui.visible {
        outbox.clear();
        return;
    }
    while let Some(command) = outbox.pop() {
        let result = match command {
            NanoFreeTuningUiCommand::SelectAndConfirm { power_index } => {
                model.click_power(power_index).map(|_| ())
            }
            NanoFreeTuningUiCommand::Cancel => model.cancel(),
        };
        if let Err(error) = result {
            inputs.runtime.message = format!("NanoFreeTuning input rejected: {error}");
        }
    }

    let preview_ready = inputs.production.preview_entity.is_some_and(|entity| {
        inputs.previews.get(entity).is_ok_and(|(_, _, _, preview)| {
            asset_server.is_loaded_with_dependencies(preview.gltf.id())
        })
    });
    let effect_ready = model.phase() != NanoFreeTuningPhase::EffectDelay
        || inputs.effects.is_native_preload_complete(
            ffone_client::nano_free_tuning_ui::NANO_FREE_TUNING_EFFECT_ID,
        );
    let assets_ready = preview_ready && effect_ready;
    if model.phase() != NanoFreeTuningPhase::Closed && !assets_ready {
        inputs.production.asset_wait_elapsed += time.delta_secs();
        if inputs.production.asset_wait_elapsed > 30.0 {
            inputs.runtime.message = "Nano acquisition assets did not finish loading".to_owned();
            let _ = model.cancel();
        }
    } else {
        inputs.production.asset_wait_elapsed = 0.0;
    }
    if assets_ready && model.phase() == NanoFreeTuningPhase::EffectDelay {
        let _ = model.mark_effect_ready();
    }
    if assets_ready
        && model.phase() != NanoFreeTuningPhase::Closed
        && let Err(error) = model.advance(time.delta_secs())
    {
        inputs.runtime.message = format!("NanoFreeTuning transition rejected: {error}");
    }

    let intents = model.drain_intents().collect::<Vec<_>>();
    for intent in intents {
        match intent {
            NanoFreeTuningIntent::Wire(wire) => {
                if wire.packet_id != packet::P_CL2FE_REQ_NANO_TUNE
                    || wire.payload_size != NanoTuneRequest0104::SIZE
                {
                    inputs.runtime.message = format!(
                        "NanoFreeTuning blocked an invalid wire intent id={} size={}",
                        wire.packet_id, wire.payload_size
                    );
                    let _ = model.cancel();
                    continue;
                }
                let request = NanoTuneRequest0104 {
                    nano_id: wire.body.nano_id,
                    tune_id: wire.body.tune_id,
                    needed_item_slots: wire.body.needed_item_slots,
                };
                if let Err(error) = bridge.send(NetworkCommand::TuneNano(request)) {
                    inputs.runtime.message = error;
                    let _ = model.cancel();
                }
            }
            NanoFreeTuningIntent::Sound(NanoFreeTuningSoundIntent::Play(true_name)) => {
                match nano_free_tuning_audio_path(&inputs.audio_catalog, true_name) {
                    Ok(path) => {
                        commands.spawn((
                            Name::new(format!("NanoFreeTuning audio {true_name}")),
                            ffone_client::audio_channel::GameplayAudioChannel::ui_sfx(),
                            AudioPlayer::new(asset_server.load(path)),
                            PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(0.7)),
                        ));
                    }
                    Err(error) => inputs.runtime.message = error,
                }
            }
            NanoFreeTuningIntent::Effect(NanoFreeTuningEffectIntent::Preload { effect_id }) => {
                inputs
                    .effects
                    .enqueue(TutorialEffectRuntimeCommand::Preload {
                        effect_id,
                        source_line: 300,
                    });
            }
            NanoFreeTuningIntent::Effect(NanoFreeTuningEffectIntent::Instantiate {
                effect_id,
                position,
                rotation,
            }) => {
                inputs.effects.enqueue(TutorialEffectRuntimeCommand::Add {
                    effect_id,
                    placement: TutorialEffectPlacement::World { position, rotation },
                    scale: 1.0,
                    tracked: false,
                    name: None,
                    destroy_after_seconds: None,
                    source_line: 124,
                });
            }
            NanoFreeTuningIntent::World(NanoFreeTuningWorldIntent::FacePlayerToward(target)) => {
                if let Ok((_entity, mut transform, _controller)) = inputs.players.single_mut()
                    && let Some(heading) = tutorial_player_heading_toward(&transform, target)
                {
                    transform.rotation = heading.native_root_rotation();
                }
            }
            NanoFreeTuningIntent::World(NanoFreeTuningWorldIntent::SpawnPreviewNano {
                nano_id,
                initial_position,
            }) => {
                if let Some(entity) = inputs.production.preview_entity
                    && let Ok((_entity, mut transform, mut visibility, preview)) =
                        inputs.previews.get_mut(entity)
                {
                    if preview.nano_id == nano_id {
                        transform.translation = initial_position;
                        *visibility = Visibility::Hidden;
                    } else {
                        inputs.runtime.message = format!(
                            "NanoFreeTuning preview identity mismatch: {} != {nano_id}",
                            preview.nano_id
                        );
                    }
                }
            }
            NanoFreeTuningIntent::World(NanoFreeTuningWorldIntent::SpawnCreationBullets {
                bullet_types,
                position,
            }) => {
                let source = inputs
                    .players
                    .single()
                    .map(|(_, transform, _)| transform.translation)
                    .unwrap_or(position);
                inputs
                    .effects
                    .enqueue(nano_free_tuning_creation_projectile_command(
                        bullet_types,
                        source,
                        position,
                        &mut inputs.projectile_random,
                    ));
            }
            NanoFreeTuningIntent::World(NanoFreeTuningWorldIntent::RevealPreviewNano {
                position,
                rotation,
            }) => {
                if let Some(entity) = inputs.production.preview_entity
                    && let Ok((_entity, mut transform, mut visibility, _)) =
                        inputs.previews.get_mut(entity)
                {
                    transform.translation = position;
                    transform.rotation = rotation;
                    // Clean moves the wrapper and calls `NanoAnimation.Call`
                    // in the same phase. Keep the native preview hidden until
                    // the call clip is actually bound so a slow GLB load can
                    // never flash the bind pose.
                    *visibility = Visibility::Hidden;
                }
            }
            NanoFreeTuningIntent::World(NanoFreeTuningWorldIntent::DestroyPreviewNano) => {
                if let Some(entity) = inputs.production.preview_entity.take() {
                    commands.entity(entity).despawn();
                }
            }
            NanoFreeTuningIntent::Cinematic(NanoFreeTuningCinematicIntent::LookAt(target)) => {
                if let Ok((_camera, mut transform)) = inputs.cameras.single_mut() {
                    transform.look_at(target, Vec3::Y);
                }
            }
            NanoFreeTuningIntent::Cinematic(
                NanoFreeTuningCinematicIntent::ConfigureCreationCamera {
                    rotation_x,
                    distance,
                    ..
                },
            ) => {
                if let Ok((mut camera, _)) = inputs.cameras.single_mut() {
                    camera.pitch_degrees = rotation_x;
                    camera.distance = distance;
                    if let Ok((_entity, _transform, controller)) = inputs.players.single() {
                        camera.yaw_degrees = controller.yaw_degrees;
                    }
                }
            }
            NanoFreeTuningIntent::Cinematic(NanoFreeTuningCinematicIntent::ApproachFrame {
                rotation_x_delta,
                distance_delta,
            }) => {
                if let Ok((mut camera, _)) = inputs.cameras.single_mut() {
                    camera.pitch_degrees -= rotation_x_delta.abs();
                    if camera.distance > 0.5 {
                        camera.distance = (camera.distance - distance_delta).max(0.5);
                    }
                }
            }
            NanoFreeTuningIntent::Cinematic(NanoFreeTuningCinematicIntent::SetSubTarget {
                distance,
                height,
                player_rotation,
                ..
            }) => {
                if let Some(preview) = inputs.production.preview_entity
                    && let Ok((mut camera, _)) = inputs.cameras.single_mut()
                {
                    camera.target = preview;
                    camera.height = height * 0.6;
                    camera.sub_target_forward = Some(player_rotation * Vec3::NEG_Z);
                    camera.minimum_distance = 0.1;
                    camera.distance = distance;
                }
            }
            NanoFreeTuningIntent::Cinematic(NanoFreeTuningCinematicIntent::CallPreviewNano) => {
                // Acquisition calls NanoAnimation.Call, not the world
                // NanoMoveControllerClone.Call wrapper with ES527..ES529.
                inputs.production.queue_animation("call");
            }
            NanoFreeTuningIntent::Cinematic(NanoFreeTuningCinematicIntent::SetStandMotion) => {
                let stand = inputs.stand_random.next_stand_clip();
                inputs.production.queue_animation(stand);
            }
            NanoFreeTuningIntent::Cinematic(NanoFreeTuningCinematicIntent::PlaySelectedSkill {
                power_index,
            }) => {
                if let Some(clip_name) = match power_index {
                    0 => Some("skill1"),
                    1 => Some("skill2"),
                    2 => Some("skill3"),
                    _ => None,
                } {
                    inputs.production.queue_animation(clip_name);
                }
            }
            NanoFreeTuningIntent::Cinematic(NanoFreeTuningCinematicIntent::PlayIdleHappy) => {
                inputs.production.queue_animation("happy");
            }
            NanoFreeTuningIntent::Cinematic(NanoFreeTuningCinematicIntent::HidePreviewNano) => {
                if let Some(entity) = inputs.production.preview_entity
                    && let Ok((_entity, _transform, mut visibility, _)) =
                        inputs.previews.get_mut(entity)
                {
                    *visibility = Visibility::Hidden;
                }
            }
            NanoFreeTuningIntent::Cinematic(NanoFreeTuningCinematicIntent::EndSubTarget) => {
                if let Some(snapshot) = inputs.production.camera_snapshot.take()
                    && let Ok((mut camera, _)) = inputs.cameras.single_mut()
                {
                    snapshot.restore(&mut camera);
                }
            }
            NanoFreeTuningIntent::Ui(NanoFreeTuningUiIntent::CaptureAndUnlockCursor) => {
                if let Ok(mut cursor) = inputs.cursors.single_mut() {
                    inputs.production.cursor_snapshot = Some(NanoFreeTuningCursorSnapshot {
                        grab_mode: cursor.grab_mode,
                        visible: cursor.visible,
                    });
                    cursor.grab_mode = CursorGrabMode::None;
                    cursor.visible = true;
                }
            }
            NanoFreeTuningIntent::Ui(NanoFreeTuningUiIntent::RestoreCapturedCursor) => {
                if let Some(snapshot) = inputs.production.cursor_snapshot.take()
                    && let Ok(mut cursor) = inputs.cursors.single_mut()
                {
                    cursor.grab_mode = snapshot.grab_mode;
                    cursor.visible = snapshot.visible;
                }
            }
            NanoFreeTuningIntent::Ui(
                NanoFreeTuningUiIntent::Show
                | NanoFreeTuningUiIntent::Hide
                | NanoFreeTuningUiIntent::SetEnabled(_),
            ) => {}
            NanoFreeTuningIntent::FirstUseCheck(condition) => {
                inputs.production.first_use_checks.push(condition);
            }
            NanoFreeTuningIntent::SetUpsellUpdate(enabled) => {
                inputs.production.upsell_update_enabled = enabled;
            }
            NanoFreeTuningIntent::DeleteEcomIcon(icon) => {
                inputs.production.visible_ecom_icons.remove(&icon);
            }
            NanoFreeTuningIntent::AuthoritativeCommit(success) => {
                let nano_id = success.nano_id;
                if let Err(error) = apply_nano_free_tuning_commit(
                    success,
                    &mut inputs.inventory,
                    &mut inputs.bank,
                    &mut inputs.runtime,
                ) {
                    inputs.runtime.message =
                        format!("NanoFreeTuning authoritative commit rejected: {error}");
                    let _ = model.cancel();
                } else if let Some(request) =
                    nano_acquisition_auto_equip_request(nano_id, &inputs.runtime.nano_slots)
                    && let Err(error) = bridge.send(NetworkCommand::EquipNano(request))
                {
                    inputs.runtime.message = error;
                }
            }
            NanoFreeTuningIntent::AuthoritativeFailure(failure) => {
                inputs.runtime.message = format!(
                    "OpenFusion rejected Nano tuning for PC {} with error {}",
                    failure.player_id, failure.error_code
                );
                let _ = model.cancel();
            }
            NanoFreeTuningIntent::RequestIdleRoll { exclusive_max } => {
                let sample =
                    advance_native_xorshift32(&mut inputs.production.idle_random_state) as i32;
                let roll = sample.rem_euclid(exclusive_max);
                if let Err(error) = model.apply_idle_roll(roll) {
                    inputs.runtime.message = format!("NanoFreeTuning idle roll rejected: {error}");
                }
            }
            NanoFreeTuningIntent::QueryAcquisitionContinuation => {
                if let Some(nano_id) = inputs.bank.first_untuned_index() {
                    let _ = model.cancel();
                    inputs.production.continuation_open_after_close =
                        Some(NanoFreeTuningOpenTrigger {
                            nano_id,
                            killed_fusion: false,
                        });
                } else if let Err(error) = model.resolve_acquisition_continuation(0) {
                    inputs.runtime.message =
                        format!("NanoFreeTuning continuation rejected: {error}");
                }
            }
            NanoFreeTuningIntent::ProtocolFault(fault) => {
                inputs.runtime.message = format!("NanoFreeTuning protocol fault: {fault:?}");
                let _ = model.cancel();
            }
            NanoFreeTuningIntent::ExitMode => {
                inputs.production.upsell_update_enabled = true;
            }
        }
    }
    if model.phase() == NanoFreeTuningPhase::Closed
        && inputs.production.preview_entity.is_none()
        && inputs.production.camera_snapshot.is_none()
        && inputs.production.cursor_snapshot.is_none()
        && let Some(next) = inputs.production.continuation_open_after_close.take()
    {
        inputs.production.pending_open = Some(next);
    }
}

pub(super) fn sync_nano_free_tuning_preview_animation(
    mut commands: Commands,
    gltfs: Res<Assets<Gltf>>,
    clips: Res<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    parents: Query<&ChildOf>,
    mut previews: Query<(Entity, &NanoFreeTuningPreview, &mut Visibility)>,
    mut players: Query<(
        Entity,
        &mut AnimationPlayer,
        Option<&mut AnimationTransitions>,
        Option<&NanoFreeTuningPreviewAnimation>,
    )>,
    mut model: ResMut<NanoFreeTuningModel>,
    mut production: ResMut<NanoFreeTuningProductionRuntime>,
    mut stand_random: ResMut<LegacyNanoStandRandomStream>,
    mut runtime: ResMut<RuntimeStatus>,
) {
    // Hide/close owns the end of this preview. A delayed call completion or
    // an older queued idle must not reveal it again during the next award.
    if matches!(
        model.phase(),
        NanoFreeTuningPhase::Closed | NanoFreeTuningPhase::ResultHide
    ) {
        production.pending_animation = None;
        production.queued_animations.clear();
        return;
    }
    let Some(preview_entity) = production.preview_entity else {
        production.pending_animation = None;
        production.queued_animations.clear();
        return;
    };
    let Ok((root_entity, preview, _)) = previews.get_mut(preview_entity) else {
        return;
    };
    let nano_id = preview.nano_id;
    let gltf_handle = preview.gltf.clone();

    // Clean `NanoAnimation.EndAnimation` returns the creation `call` clip to
    // a stand pose as soon as the clip finishes.
    if production.pending_animation.is_none() {
        let mut matched_call = false;
        let mut call_finished = true;
        for (entity, player, _, previous) in &mut players {
            if !dexter_ship_descendant(entity, root_entity, &parents) {
                continue;
            }
            let Some(previous) = previous.filter(|previous| previous.clip_name == "call") else {
                continue;
            };
            matched_call = true;
            call_finished &= player
                .animation(previous.node)
                .is_some_and(|animation| animation.is_finished());
        }
        if matched_call && call_finished {
            production.queue_animation(stand_random.next_stand_clip());
        }
    }

    let Some(requested_clip) = production.pending_animation else {
        return;
    };
    let Some(gltf) = gltfs.get(&gltf_handle) else {
        return;
    };
    let result_animation =
        model.phase() == NanoFreeTuningPhase::ResultSkill && requested_clip.starts_with("skill");
    let requested_available = gltf
        .named_animations
        .keys()
        .any(|name| name.eq_ignore_ascii_case(requested_clip));
    // Missing random idle uses the source stand1 fallback. Some installed
    // models have no skill clip at all: retain a neutral idle for one full
    // cycle during the result instead of hiding immediately or borrowing a
    // different ability. This recovery does not change the server's skill.
    let clip_name = if !requested_available
        && (matches!(requested_clip, "stand2" | "stand3") || result_animation)
    {
        "stand1"
    } else {
        requested_clip
    };

    if production.animation_graph.is_none() {
        let mut graph = AnimationGraph::new();
        let root = graph.root;
        let mut named = gltf.named_animations.iter().collect::<Vec<_>>();
        named.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
        production.animation_nodes = named
            .into_iter()
            .map(|(name, clip)| {
                let node = graph.add_clip(clip.clone(), 1.0, root);
                (name.to_string(), node)
            })
            .collect();
        production.animation_graph = Some(graphs.add(graph));
    }

    let Some(node) = production
        .animation_nodes
        .get(clip_name)
        .copied()
        .or_else(|| {
            production
                .animation_nodes
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(clip_name))
                .map(|(_, node)| *node)
        })
    else {
        runtime.message = format!(
            "NanoFreeTuning native Nano {nano_id} is missing clean animation {clip_name:?}"
        );
        if model.phase() == NanoFreeTuningPhase::ResultSkill && clip_name.starts_with("skill") {
            let _ = model.set_result_animation_duration(0.0);
        }
        production.complete_pending_animation();
        return;
    };
    let Some(clip_handle) = gltf.named_animations.get(clip_name).cloned().or_else(|| {
        gltf.named_animations
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(clip_name))
            .map(|(_, handle)| handle.clone())
    }) else {
        runtime.message = format!(
            "NanoFreeTuning native Nano {nano_id} is missing clean animation {clip_name:?}"
        );
        if model.phase() == NanoFreeTuningPhase::ResultSkill && clip_name.starts_with("skill") {
            let _ = model.set_result_animation_duration(0.0);
        }
        production.complete_pending_animation();
        return;
    };
    let Some(clip) = clips.get(&clip_handle) else {
        return;
    };
    let duration = clip.duration();
    let graph = production
        .animation_graph
        .clone()
        .expect("NanoFreeTuning graph was built with its node map");
    let mut applied = false;
    for (entity, mut player, transitions, previous) in &mut players {
        if !dexter_ship_descendant(entity, root_entity, &parents) {
            continue;
        }
        if previous.is_some_and(|previous| previous.node == node)
            && clip_name.starts_with("stand")
            && !result_animation
        {
            applied = true;
            continue;
        }
        let repeat = if clip_name.starts_with("stand") && !result_animation {
            RepeatAnimation::Forever
        } else {
            RepeatAnimation::Never
        };
        if let Some(mut transitions) = transitions {
            transitions
                .play(&mut player, node, Duration::from_millis(100))
                .set_repeat(repeat)
                .resume();
            commands.entity(entity).insert((
                AnimationGraphHandle(graph.clone()),
                NanoFreeTuningPreviewAnimation { clip_name, node },
            ));
        } else {
            let mut transitions = AnimationTransitions::new();
            transitions
                .play(&mut player, node, Duration::ZERO)
                .set_repeat(repeat)
                .resume();
            commands.entity(entity).insert((
                AnimationGraphHandle(graph.clone()),
                transitions,
                NanoFreeTuningPreviewAnimation { clip_name, node },
            ));
        }
        applied = true;
    }
    if !applied {
        return;
    }
    if result_animation && let Err(error) = model.set_result_animation_duration(duration) {
        runtime.message = format!("NanoFreeTuning animation duration rejected: {error}");
    }
    if (clip_name == "call" || clip_name.starts_with("stand"))
        && let Ok((_entity, _preview, mut visibility)) = previews.get_mut(preview_entity)
    {
        *visibility = Visibility::Inherited;
    }
    production.complete_pending_animation();
}

/// Replays the exact `AnimationEventHandler.sound` payloads embedded in the
/// acquired Nano's own GLB. `_Nan` events take the localized voice route in
/// `GameplayAudioRuntime`; companion call SFX remain ordinary effects audio.
pub(super) fn emit_nano_free_tuning_preview_animation_sounds(
    mut commands: Commands,
    production: Res<NanoFreeTuningProductionRuntime>,
    parents: Query<&ChildOf>,
    previews: Query<&NanoFreeTuningPreview>,
    players: Query<(
        Entity,
        &AnimationPlayer,
        &NanoFreeTuningPreviewAnimation,
        Option<&NanoFreeTuningPreviewSoundCursor>,
    )>,
    mut audio: ResMut<GameplayAudioRuntime>,
) {
    let Some(root) = production.preview_entity else {
        return;
    };
    let Ok(preview) = previews.get(root) else {
        return;
    };
    for (entity, player, applied, cursor) in &players {
        if !dexter_ship_descendant(entity, root, &parents) {
            continue;
        }
        let Some(active) = player.animation(applied.node) else {
            continue;
        };
        let same_playback = cursor.is_some_and(|cursor| {
            cursor.clip_name == applied.clip_name
                && cursor.node == applied.node
                && (active.completions() > cursor.completions
                    || (active.completions() == cursor.completions
                        && active.seek_time() >= cursor.seek_time))
        });
        let (previous_seek, previous_completions) = if same_playback {
            let cursor = cursor.expect("matching NanoFreeTuning audio cursor exists");
            (cursor.seek_time, cursor.completions)
        } else {
            (0.0, 0)
        };
        for event in preview
            .sound_events
            .iter()
            .filter(|event| event.clip.eq_ignore_ascii_case(applied.clip_name))
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
                audio.queue_legacy_nano_animation_sound(root, &event.payload);
            }
        }
        commands
            .entity(entity)
            .insert(NanoFreeTuningPreviewSoundCursor {
                clip_name: applied.clip_name,
                node: applied.node,
                seek_time: active.seek_time(),
                completions: active.completions(),
            });
    }
}

pub(super) fn nano_acquisition_auto_equip_request(
    nano_id: i16,
    slots: &[RuntimeNanoSlot; 3],
) -> Option<ffone_protocol::NanoEquipRequest0104> {
    if slots.iter().any(|slot| slot.nano_id == Some(nano_id)) {
        return None;
    }
    slots
        .iter()
        .position(|slot| slot.nano_id.is_none())
        .map(|slot| ffone_protocol::NanoEquipRequest0104 {
            nano_id,
            nano_slot: slot as i16,
        })
}

pub(super) fn apply_nano_free_tuning_commit(
    success: NanoTuneSuccess,
    inventory: &mut LocalInventoryRuntime,
    bank: &mut NanoFreeTuningBank0104,
    runtime: &mut RuntimeStatus,
) -> Result<(), String> {
    let packet = NanoTuneSuccess0104 {
        nano_id: success.nano_id,
        skill_id: success.skill_id,
        fusion_matter: success.fusion_matter,
        item_slots: success.item_slots,
        items: success.items.map(|item| ItemBase0104 {
            item_type: item.item_type,
            item_id: item.item_id,
            option: item.option,
            time_limit: item.time_limit,
        }),
    };
    let mut next_inventory = inventory
        .snapshot()
        .cloned()
        .ok_or_else(|| "local inventory is not seeded".to_owned())?;
    let mut next_bank = bank.clone();
    next_inventory
        .apply_nano_tune_success(packet)
        .map_err(|error| error.to_string())?;
    next_bank
        .apply_tune_success(success.nano_id, success.skill_id)
        .map_err(|error| error.to_string())?;

    *inventory
        .snapshot_mut()
        .expect("validated inventory snapshot remains present") = next_inventory;
    *bank = next_bank;
    runtime.fusion_matter = success.fusion_matter;
    runtime.max_fusion_matter =
        legacy_avatar_max_fusion_matter(runtime.player_level, success.fusion_matter);
    for slot in &mut runtime.nano_slots {
        if slot.nano_id == Some(success.nano_id) {
            slot.skill_id = success.skill_id;
        }
    }
    runtime.message = format!(
        "Nano {} tuned to skill {}; authoritative inventory/Fusion Matter committed",
        success.nano_id, success.skill_id
    );
    Ok(())
}

pub(super) fn apply_nano_create_commit(
    packet: PcNanoCreateSuccess0104,
    inventory: &mut LocalInventoryRuntime,
    bank: &mut NanoFreeTuningBank0104,
    runtime: &mut RuntimeStatus,
) -> Result<(), String> {
    let mut next_inventory = inventory.clone();
    let mut next_bank = bank.clone();
    next_inventory.apply_nano_create_quest_post_state(packet)?;
    next_bank
        .apply_create_success(packet)
        .map_err(|error| error.to_string())?;

    *inventory = next_inventory;
    *bank = next_bank;
    runtime.fusion_matter = packet.fusion_matter;
    if packet.player_level > 0 {
        runtime.player_level = packet.player_level as u16;
    }
    runtime.max_fusion_matter =
        legacy_avatar_max_fusion_matter(runtime.player_level, packet.fusion_matter);
    Ok(())
}
