use super::*;

#[must_use]
pub(in super::super) const fn race_service_allowed(service_category: i32, service: NpcServiceKind) -> bool {
    matches!(
        (service_category, service),
        (13, NpcServiceKind::Race)
            | (13, NpcServiceKind::RaceRank)
            | (14, NpcServiceKind::RaceRank)
    )
}

pub(in super::super) fn consume_race_network_frames(
    time: Res<Time>,
    mut inbox: ResMut<RaceNetworkFrameInbox>,
    mut model: ResMut<RaceModeModel>,
    mut production: ResMut<RaceProductionRuntime>,
    mut reward: ResMut<RaceRewardPresentation>,
    content: Res<TutorialMissionContent>,
    mut instance_audio: ResMut<RetrobutionInstanceAudioState>,
    mut status: ResMut<RuntimeStatus>,
) {
    while let Some(frame) = inbox.pop() {
        production.last_owned_packet = Some(frame.packet_type);
        match frame.packet_type {
            RACE_INSTANCE_MAP_INFO_PACKET_ID => {
                match decode_race_instance_map_info_0104(&frame.payload) {
                    Ok(info) => {
                        production.player.current_ep_id = info.ep_id;
                        production.course_bounds = Some(info.bounds);
                        production.race_ui_complete = false;
                        production.ring_ids.clear();
                        production.collected_rings.clear();
                        production.pending_rings.clear();
                        production.player.top_record = info.top_record;
                        instance_audio.active = true;
                    }
                    Err(error) => {
                        status.message = format!("Race authority rejected malformed {error}");
                    }
                }
            }
            RACE_GET_RING_SUCCESS_PACKET_ID => {
                match decode_race_ring_success_0104(&frame.payload) {
                    Ok((ring_local_id, ring_count)) => {
                        production.confirm_ring(ring_local_id, ring_count);
                    }
                    Err(error) => {
                        status.message = format!("Race authority rejected malformed {error}");
                    }
                }
            }
            RACE_GET_RING_FAILURE_PACKET_ID => {
                match decode_race_ring_failure_0104(&frame.payload) {
                    Ok(error_code) => {
                        production.pending_rings.clear();
                        status.message =
                            format!("RustyFusion rejected race-ring collection with {error_code}");
                    }
                    Err(error) => {
                        status.message = format!("Race authority rejected malformed {error}");
                    }
                }
            }
            _ => {
                let end_success = frame.packet_type == RACE_END_SUCCESS_PACKET_ID;
                match apply_correlated_race_reply_0104(
                    frame.packet_type,
                    &frame.payload,
                    time.elapsed_secs(),
                    &mut model,
                    &mut production,
                ) {
                    Ok(()) => {
                        if end_success {
                            let item = model.result().reward_item;
                            *reward = RaceRewardPresentation::default();
                            if item.was_granted() {
                                let base = ItemBase0104 {
                                    item_type: item.item_type,
                                    item_id: item.item_id,
                                    option: item.item_opt,
                                    time_limit: item.time_limit,
                                };
                                let detail = content.gameplay_user_equip_item_detail(item.item_type, item.item_id);
                                let name_key = content.gameplay_user_equip_item_text(item.item_type, item.item_id)
                                    .map(|(name, _)| name.key);
                                *reward = RaceRewardPresentation {
                                    item_type: item.item_type,
                                    item_id: item.item_id,
                                    item_opt: item.item_opt,
                                    name: detail.as_ref().map(|row| row.name.clone())
                                        .unwrap_or_else(|| format!("Item {}", item.item_id)),
                                    name_key,
                                    level: detail.as_ref().map_or(0, |row| row.level),
                                    icon_path: content.gameplay_item_display_icon(base).map(str::to_owned),
                                };
                            }
                        } else if frame.packet_type == RACE_START_SUCCESS_PACKET_ID {
                            *reward = RaceRewardPresentation::default();
                        }
                        status.fusion_matter = production.player.fusion_matter;
                        status.max_fusion_matter = legacy_avatar_max_fusion_matter(
                            status.player_level,
                            status.fusion_matter,
                        );
                    }
                    Err(error) => {
                        status.message = error;
                    }
                }
            }
        }
    }
}

pub(in super::super) fn consume_race_rank_ui_commands(
    catalog: Res<RaceRankCatalog>,
    mut model: ResMut<RaceRankModel>,
    mut commands: ResMut<RaceRankUiCommandOutbox>,
    mut status: ResMut<RuntimeStatus>,
) {
    while let Some(command) = commands.pop() {
        let result = match command {
            RaceRankUiCommand::SelectLocationSlot(slot) => {
                model.select_visible_slot(&catalog, slot).map(|_| ())
            }
            RaceRankUiCommand::PreviousPage => model.previous_page().map(|_| ()),
            RaceRankUiCommand::NextPage => model.next_page(catalog.locations().len()).map(|_| ()),
            RaceRankUiCommand::SelectPeriod(period) => model.select_period(period).map(|_| ()),
            RaceRankUiCommand::Close => {
                if model.close() {
                    Ok(())
                } else {
                    status.message = "RaceRankMode ignored a close while hidden".to_owned();
                    continue;
                }
            }
        };
        if let Err(error) = result {
            status.message = format!("RaceRankMode input {command:?} rejected: {error:?}");
        }
    }
}

pub(in super::super) fn set_race_cursor_locked(
    cursors: &mut Query<&mut CursorOptions, With<PrimaryWindow>>,
    locked: bool,
) {
    if let Ok(mut cursor) = cursors.single_mut() {
        cursor.grab_mode = if locked {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
        cursor.visible = !locked;
    }
}

pub(in super::super) fn consume_race_production_outputs(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut mode: ResMut<RaceModeModel>,
    mut rank: ResMut<RaceRankModel>,
    mut owners: RaceProductionOwners,
) {
    let effects_gain = option_channel_gain(owners.option_runtime.options.sound.effects);
    let voice_gain = option_channel_gain(owners.option_runtime.options.sound.voice);

    for _ in 0..owners.production.pending_ring_cues {
        commands.spawn((
            Name::new("Race ring pickup audio"),
            WorldSliceEntity,
            ffone_client::audio_channel::GameplayAudioChannel::ui_sfx(),
            AudioPlayer::new(asset_server.load(RACE_RING_AUDIO_PATH)),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.7 * effects_gain)),
        ));
    }
    owners.production.pending_ring_cues = 0;

    if let Some(PendingRaceNpcVoice::RaceFinished) = owners.production.pending_npc_voice.take()
        && let Err(error) = spawn_race_npc_voice(
            &mut commands,
            &asset_server,
            &mut owners.production,
            &owners.audio_catalog,
            &owners.voice_language,
            voice_gain,
            "RaceFinished",
        )
    {
        owners.status.message = error;
        owners.production.finished_voice_npc_id = None;
    }

    while let Some(output) = mode.pop_output() {
        match output {
            RaceModeOutput::Request(intent) => {
                let request = match intent.encode_registered() {
                    Ok(request) => request,
                    Err(error) => {
                        owners.status.message = format!("RaceMode request codec rejected: {error}");
                        abort_race_mode_transport(
                            &mut mode,
                            &mut owners.production,
                            &mut owners.cursors,
                        );
                        break;
                    }
                };
                match owners
                    .bridge
                    .send(NetworkCommand::SendRegisteredGameplay0104(request))
                {
                    Ok(()) => {
                        owners.status.message = format!(
                            "RaceMode {:?} request {} sent to OpenFusion",
                            intent.kind(),
                            intent.request_id()
                        );
                    }
                    Err(error) => {
                        owners.status.message =
                            format!("RaceMode request transport failed: {error}");
                        abort_race_mode_transport(
                            &mut mode,
                            &mut owners.production,
                            &mut owners.cursors,
                        );
                        break;
                    }
                }
            }
            RaceModeOutput::Effect(effect) => match effect {
                RaceModeEffect::SetCursorLocked(locked) => {
                    set_race_cursor_locked(&mut owners.cursors, locked);
                }
                RaceModeEffect::EndCameraSubTarget => {
                    if let Some(source) = owners.production.source_npc.as_mut() {
                        source.camera_subtarget_active = false;
                    }
                }
                RaceModeEffect::ExitMode => {
                    owners.production.finish_mode(RACE_MODE_GAME_MODE_ID);
                    owners.mission_ui.clear_npc_interaction_locally();
                }
                RaceModeEffect::ActivateRings => {
                    // Course selection belongs to GameFrameEpContainer. The
                    // published world has several overlapping course sets, so
                    // retain the exact request without revealing all rings.
                    owners.production.ring_activation_requested = Some(true);
                    owners.production.collected_rings.clear();
                    owners.production.pending_rings.clear();
                }
                RaceModeEffect::DeactivateRings => {
                    owners.production.ring_activation_requested = Some(false);
                }
                RaceModeEffect::PlaySound(sound) => {
                    let (path, clean_gain) = match sound {
                        RaceSound::RaceStart => (RACE_START_AUDIO_PATH, 0.7),
                        RaceSound::RaceFinish => (RACE_FINISH_AUDIO_PATH, 0.7),
                        RaceSound::Button => (
                            owners.production.next_button_sound_path(),
                            RACE_BUTTON_SOUND_GAIN,
                        ),
                    };
                    commands.spawn((
                        Name::new(format!("RaceMode audio {path}")),
                        WorldSliceEntity,
                        ffone_client::audio_channel::GameplayAudioChannel::ui_sfx(),
                        AudioPlayer::new(asset_server.load(path)),
                        PlaybackSettings::DESPAWN
                            .with_volume(Volume::Linear(clean_gain * effects_gain)),
                    ));
                }
                RaceModeEffect::PlayNpcRaceStartVoice { npc_instance_id } => {
                    let correlated = owners
                        .production
                        .source_npc
                        .as_ref()
                        .is_some_and(|source| source.runtime_npc_id == npc_instance_id);
                    if !correlated {
                        owners.status.message =
                            format!("RaceMode start voice rejected stale NPC {npc_instance_id}");
                    } else if let Err(error) = spawn_race_npc_voice(
                        &mut commands,
                        &asset_server,
                        &mut owners.production,
                        &owners.audio_catalog,
                        &owners.voice_language,
                        voice_gain,
                        "ClickStart",
                    ) {
                        owners.status.message = error;
                    }
                }
                RaceModeEffect::Ecom(operation) => match operation {
                    RaceEcomOperation::Delete(icon) => {
                        owners.production.visible_ecom_icons.remove(&icon);
                    }
                    RaceEcomOperation::Make(icon) => {
                        owners.production.visible_ecom_icons.insert(icon);
                    }
                },
                RaceModeEffect::SystemMessage { message_id, key: _ } => {
                    if let Err(error) = owners.production.queue_system_message(
                        &owners.content,
                        &mut owners.system_messages,
                        message_id,
                    ) {
                        owners.status.message = error;
                    }
                }
                RaceModeEffect::MessageBox {
                    message_id,
                    box_type,
                    copy,
                } => {
                    owners.production.last_message_box = Some((message_id, box_type, copy));
                    owners.status.message = format!(
                        "RaceMode retained clean MessageBox {message_id}/{box_type}; the distinct MessageBox owner is unavailable"
                    );
                }
                RaceModeEffect::ReceiveRewardItem {
                    inventory_location,
                    inventory_slot,
                    item,
                } => {
                    let reward = ItemReward0104 {
                        item: ItemBase0104 {
                            item_type: item.item_type,
                            item_id: item.item_id,
                            option: item.item_opt,
                            time_limit: item.time_limit,
                        },
                        inventory_location,
                        slot: inventory_slot,
                    };
                    if let Err(error) = owners
                        .inventory
                        .apply_single_race_reward_item_post_state(reward)
                    {
                        owners.status.message = format!("RaceMode reward rejected: {error}");
                    }
                }
                RaceModeEffect::RefreshInventory => {
                    // ReceiveRewardItem already committed the complete server
                    // post-state into the shared inventory owner.
                }
                RaceModeEffect::CheckFirstUseCondition(condition) => {
                    owners.production.first_use_checks.push(condition);
                }
            },
        }
    }

    while let Some(output) = rank.pop_output() {
        match output {
            RaceRankOutput::Http(_intent) => {
                debug_assert!(!RACE_RANK_HTTP_TRANSPORT_AVAILABLE);
                owners.production.rank_http_rejections =
                    owners.production.rank_http_rejections.saturating_add(1);
                owners.status.message = RACE_RANK_HTTP_GAP.to_owned();
                rank.close();
            }
            RaceRankOutput::Effect(effect) => match effect {
                RaceRankEffect::BindNpcCamera { .. } => {
                    // No production route is allowed to enter mode 17 while
                    // its clean HTTP owner is absent.
                    owners.status.message = RACE_RANK_HTTP_GAP.to_owned();
                    rank.close();
                }
                RaceRankEffect::ReleaseNpcCamera | RaceRankEffect::FreeLegacyAssets => {}
                RaceRankEffect::ExitMode => {
                    owners.production.finish_mode(RACE_RANK_GAME_MODE_ID);
                }
            },
        }
    }
}

pub(in super::super) fn consume_race_system_message_outbox(
    mut system_outbox: ResMut<SystemMessageUiOutbox>,
    mut production: ResMut<RaceProductionRuntime>,
    mut status: ResMut<RuntimeStatus>,
) {
    if production.pending_system_messages.is_empty() {
        return;
    }
    let mut unrelated = Vec::new();
    for action in system_outbox.drain().collect::<Vec<_>>() {
        let SystemMessageUiAction::Chosen { request_id, .. } = action;
        let Some(message_id) = production.pending_system_messages.remove(&request_id) else {
            unrelated.push(action);
            continue;
        };
        status.message = format!("RaceMode SystemMessage {message_id} dismissed");
    }
    for action in unrelated {
        system_outbox.push(action);
    }
}

pub(in super::super) fn reset_race_shell(
    mode: &mut RaceModeModel,
    mode_commands: &mut RaceModeUiCommandOutbox,
    reward: &mut RaceRewardPresentation,
    rank: &mut RaceRankModel,
    rank_commands: &mut RaceRankUiCommandOutbox,
    production: &mut RaceProductionRuntime,
    frames: &mut RaceNetworkFrameInbox,
) {
    *mode = RaceModeModel::default();
    while mode_commands.pop().is_some() {}
    *reward = RaceRewardPresentation::default();
    *rank = RaceRankModel::default();
    while rank_commands.pop().is_some() {}
    production.reset();
    frames.clear();
}

pub(in super::super) fn open_race_fail_from_warp_away(
    model: &mut RaceModeModel,
    catalog: &RaceRankCatalog,
    production: &mut RaceProductionRuntime,
) -> Result<(), String> {
    // `CnGuiChat.WarpAway` first deletes game mode 16, then recreates it as
    // `FailEcom`. Preserve the live race-player state across that mode edge.
    production.finish_mode(RACE_MODE_GAME_MODE_ID);
    *model = RaceModeModel::default();
    let player = production.player;
    model
        .open(RaceModeOpenContext {
            ecom_type: RaceEcomType::Fail,
            npc: None,
            player,
            current_ep_instance_exists: catalog.location_by_ep(player.current_ep_id).is_some(),
        })
        .map_err(|error| format!("RaceMode Warp Away failure rejected: {error:?}"))?;
    if let Err(error) = production.begin_mode(RACE_MODE_GAME_MODE_ID, 0, 0, String::new()) {
        *model = RaceModeModel::default();
        return Err(format!(
            "RaceMode Warp Away failure lifecycle rejected: {error}"
        ));
    }
    Ok(())
}
