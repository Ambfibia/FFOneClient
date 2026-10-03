use crate::app::gameplay_ui_actions::*;

pub(super) fn handle_warp(
    action: WarpAction,
    gameplay_modal: bool,
    context: &mut WorldGameplayActionContext<'_, '_>,
) {
    let WorldGameplayActionContext {
        gameplay_ui,
        buddy_ui,
        normal_warp,
        mission_ui,
        content,
        modal_owners,
        bridge,
        runtime,
        ..
    } = context;
    for action in std::iter::once(action) {
        match action {
            WarpAction::WarpAwayStarted => {
                if mission_ui.warp_away_countdown_seconds().is_none() {
                    runtime.message =
                        "Warp Away start rejected: no exact live countdown owner".to_owned();
                    continue;
                }
                if modal_owners.race.production.player.ring_race_active {
                    mission_ui.cancel_warp_away();
                    runtime.message = match open_race_fail_from_warp_away(
                        &mut modal_owners.race.mode,
                        &modal_owners.race.catalog,
                        &mut modal_owners.race.production,
                    ) {
                        Ok(()) => {
                            "Warp Away converted the active ring race to clean FailEcom".to_owned()
                        }
                        Err(error) => error,
                    };
                    continue;
                }
                if gameplay_modal
                    || buddy_ui.add_dialog_open()
                    || modal_owners.shell.system_messages.is_popup()
                    || modal_owners.shell.resurrect_ui.visible
                {
                    mission_ui.cancel_warp_away();
                    runtime.message =
                        "Warp Away start rejected while another gameplay modal owns input"
                            .to_owned();
                    continue;
                }
                let Ok((transform, _, pending_collider)) = modal_owners.shell.players.single_mut()
                else {
                    mission_ui.cancel_warp_away();
                    runtime.message =
                        "Warp Away start requires one authoritative local player".to_owned();
                    continue;
                };
                if pending_collider.is_some()
                    || warp_away_xcom_index(
                        &content,
                        runtime.map_number,
                        ProtocolPosition::from_native(transform.translation).raw(),
                    )
                    .is_none()
                {
                    mission_ui.cancel_warp_away();
                    runtime.message =
                        "Warp Away start rejected before an authoritative XCom target is ready"
                            .to_owned();
                    continue;
                }
                modal_owners.shell.system_messages.set_focus_out(true);
                gameplay_ui.chat.active = false;
                runtime.message = "Warp Away countdown started (20 seconds)".to_owned();
            }
            WarpAction::RequestWarpAway => {
                if !mission_ui.warp_away_request_pending() {
                    runtime.message =
                        "Warp Away request rejected: no expired live countdown".to_owned();
                    continue;
                }
                let request = modal_owners
                    .shell
                    .players
                    .single_mut()
                    .ok()
                    .and_then(|(transform, _, pending_collider)| {
                        pending_collider.is_none().then(|| {
                            warp_away_xcom_index(
                                &content,
                                runtime.map_number,
                                ProtocolPosition::from_native(transform.translation).raw(),
                            )
                        })?
                    })
                    .map(|index| PcRegenRequest0104 {
                        regen_type: 5,
                        e_il: 0,
                        index,
                    });
                let Some(request) = request else {
                    mission_ui.reject_warp_away_request();
                    modal_owners.shell.system_messages.set_focus_out(false);
                    runtime.message =
                        "Warp Away request rejected: the nearest proven XCom is unavailable"
                            .to_owned();
                    continue;
                };
                match bridge.send(NetworkCommand::Regen(request)) {
                    Ok(()) => {
                        if let Ok((_, mut controller, _)) = modal_owners.shell.players.single_mut()
                        {
                            // `CnGuiChat.SendWarpAway` disables movement-packet
                            // emission until the authoritative regen response.
                            controller.movement_enabled = false;
                        }
                        runtime.message = format!(
                            "Warp Away regeneration requested through XCom row {}",
                            request.index
                        );
                    }
                    Err(error) => {
                        mission_ui.reject_warp_away_request();
                        modal_owners.shell.system_messages.set_focus_out(false);
                        runtime.message = format!("Warp Away transport failed: {error}");
                    }
                }
            }
            WarpAction::NpcWarp {
                npc_id,
                npc_type,
                warp_id,
                required_task_id,
                target,
            } => {
                let identity = NormalNpcWarpIdentity {
                    npc_id,
                    npc_type,
                    warp_id,
                    required_task_id,
                    target,
                };
                if gameplay_modal {
                    identity.reject_ui(mission_ui);
                    runtime.message = format!(
                        "Normal NPC warp {warp_id} rejected while another gameplay modal is active"
                    );
                    continue;
                }
                if !validate_normal_npc_warp_ui_action(mission_ui, identity) {
                    runtime.message = format!(
                        "Normal NPC warp {warp_id} rejected: no exact pending live UI interaction"
                    );
                    continue;
                }

                let mut live_matches = normal_warp
                    .npc_sources
                    .iter()
                    .filter(|(_, appearance, _)| appearance.0.npc_id == npc_id);
                let Some((npc_entity, appearance, transform)) = live_matches.next() else {
                    identity.reject_ui(mission_ui);
                    runtime.message = format!(
                        "Normal NPC warp {warp_id} rejected: runtime NPC {npc_id} is no longer live"
                    );
                    continue;
                };
                if live_matches.next().is_some()
                    || appearance.0.npc_type != npc_type
                    || appearance.0.hp <= 0
                {
                    identity.reject_ui(mission_ui);
                    runtime.message = format!(
                        "Normal NPC warp {warp_id} rejected: runtime NPC {npc_id}/{npc_type} is ambiguous or no longer talkable"
                    );
                    continue;
                }
                let Some(warp) = content.normal_gameplay_warp_for_npc(npc_type) else {
                    identity.reject_ui(mission_ui);
                    runtime.message = format!(
                        "Normal NPC warp {warp_id} rejected: NPC table {npc_type} has no category-5 first-row warp"
                    );
                    continue;
                };
                let expected_required_task =
                    (warp.limit_task_id != 0).then_some(warp.limit_task_id);
                if warp.warp_id != warp_id
                    || warp.target != target
                    || expected_required_task != required_task_id
                {
                    identity.reject_ui(mission_ui);
                    runtime.message = format!(
                        "Normal NPC warp {warp_id} rejected: UI action no longer matches first serialized WarpTable row {}",
                        warp.warp_id
                    );
                    continue;
                }
                let Some(map_number) = runtime.map_number else {
                    identity.reject_ui(mission_ui);
                    runtime.message =
                        format!("Normal NPC warp {warp_id} rejected before authoritative map load");
                    continue;
                };
                let Some(local_pc_uid) = normal_warp.group_ui.local_pc_uid else {
                    identity.reject_ui(mission_ui);
                    runtime.message = format!(
                        "Normal NPC warp {warp_id} rejected before authoritative PC UID load"
                    );
                    continue;
                };

                let active_mission_ids = modal_owners
                    .mission
                    .world_mission
                    .active_tasks()
                    .iter()
                    .filter_map(|active| content.mission(active.task_id).ok())
                    .map(|definition| definition.provenance.mission_id)
                    .collect::<Vec<_>>();
                let limit_task_is_active = warp.limit_task_id != 0
                    && modal_owners
                        .mission
                        .world_mission
                        .active_tasks()
                        .iter()
                        .any(|active| active.task_id == warp.limit_task_id);
                let group_members = normal_warp
                    .group_ui
                    .pc_members
                    .iter()
                    .map(|member| GameplayWarpGroupMember {
                        pc_uid: member.pc_uid,
                        position: GameplayWarpServerPosition::new(
                            member.map_number,
                            member.position[0],
                            member.position[1],
                            member.position[2],
                        ),
                    })
                    .collect::<Vec<_>>();
                let npc_position = native_to_unity_vector(transform.translation());
                let eligibility = warp.resolve_normal_eligibility(
                    GameplayWarpEligibilityInput {
                        runtime_npc_id: npc_id,
                        cash: runtime.candy,
                        level: i32::from(runtime.player_level),
                        limit_task_is_active,
                        active_mission_ids: &active_mission_ids,
                        local_pc_uid,
                        npc_position: GameplayWarpWorldPosition::new(
                            map_number,
                            npc_position.x,
                            npc_position.y,
                            npc_position.z,
                        ),
                        group_members: &group_members,
                    },
                    |lookup| normal_npc_warp_item_slot(&normal_warp.inventory, lookup),
                );
                match eligibility {
                    Ok(GameplayWarpEligibility::Ready(request)) => {
                        let player_effect_transform = {
                            let mut players = modal_owners.shell.players.iter_mut();
                            let Some((transform, _, pending_collider)) = players.next() else {
                                identity.reject_ui(mission_ui);
                                runtime.message = format!(
                                    "Normal NPC warp {warp_id} rejected before local player spawn"
                                );
                                continue;
                            };
                            if pending_collider.is_some() || players.next().is_some() {
                                identity.reject_ui(mission_ui);
                                runtime.message = format!(
                                    "Normal NPC warp {warp_id} rejected for ambiguous or loading local player"
                                );
                                continue;
                            }
                            (transform.translation, transform.rotation)
                        };
                        let Some(definition) = content.gameplay_npc(npc_type) else {
                            identity.reject_ui(mission_ui);
                            runtime.message = format!(
                                "Normal NPC warp {warp_id} rejected: NPC table {npc_type} disappeared"
                            );
                            continue;
                        };

                        // Clean primary `NpcIconMode.WarpOK` plays the spatial
                        // Dexbot cue and WarpOK voice immediately, then its
                        // `LoadWarpEffect` coroutine preloads/instantiates
                        // ES394 at the local player's world transform.
                        modal_owners
                            .audio
                            .runtime
                            .queue_legacy_world_sound(npc_entity, NORMAL_NPC_WARP_SOUND_TRUE_NAME);
                        modal_owners.audio.runtime.queue_legacy_npc_voice(
                            npc_entity,
                            &definition.move_voice_owner,
                            LegacyNpcVoiceCue::WarpOk,
                        );
                        normal_warp
                            .effects
                            .enqueue(TutorialEffectRuntimeCommand::Preload {
                                effect_id: NORMAL_NPC_WARP_EFFECT_ID,
                                source_line: line!(),
                            });
                        normal_warp
                            .effects
                            .enqueue(TutorialEffectRuntimeCommand::Add {
                                effect_id: NORMAL_NPC_WARP_EFFECT_ID,
                                placement: TutorialEffectPlacement::World {
                                    position: player_effect_transform.0,
                                    rotation: player_effect_transform.1,
                                },
                                scale: 1.0,
                                tracked: false,
                                name: Some(NORMAL_NPC_WARP_PRESENTATION.to_owned()),
                                destroy_after_seconds: None,
                                source_line: line!(),
                            });
                        normal_warp.production.departure_clock = WarpDepartureClock::default();
                        normal_warp.production.pending = Some(PendingNormalNpcWarp {
                            identity,
                            request,
                            elapsed_seconds: 0.0,
                            sent: false,
                        });
                        runtime.message = format!(
                            "Normal NPC warp {warp_id} validated; waiting for departure effect completion"
                        );
                    }
                    Ok(GameplayWarpEligibility::Denied(denial)) => {
                        identity.reject_ui(mission_ui);
                        normal_warp.production.pending = None;
                        let message_id = denial.system_message_id();
                        if let Err(error) = normal_warp.production.queue_system_message(
                            &content,
                            &mut modal_owners.shell.system_messages,
                            message_id,
                        ) {
                            runtime.message = format!(
                                "Normal NPC warp {warp_id} denied by SystemMessage {message_id}, but presentation failed: {error}"
                            );
                        } else {
                            runtime.message = format!(
                                "Normal NPC warp {warp_id} denied by clean SystemMessage {message_id}"
                            );
                        }
                    }
                    Err(error) => {
                        identity.reject_ui(mission_ui);
                        normal_warp.production.pending = None;
                        runtime.message = format!("Normal NPC warp {warp_id} rejected: {error}");
                    }
                }
            }
        }
    }
}
