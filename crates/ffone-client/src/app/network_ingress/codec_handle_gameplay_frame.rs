use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_gameplay_frame(
    commands: &mut Commands,
    bridge: &NetworkBridge,
    runtime: &mut RuntimeStatus,
    next_state: &mut NextState<ClientState>,
    lifecycle_ingress: &mut NetworkEntityLifecycleIngress0104,
    lifecycle_session: &mut NetworkLifecycleSession,
    asset_server: &AssetServer,
    world_spawn: &mut WorldSpawnRuntime<'_, '_>,
    local_tutorial_hp_authority: bool,
    frame: DecodedFrame,
) {
    runtime.diagnostics.gameplay_frames = runtime.diagnostics.gameplay_frames.saturating_add(1);
    if frame.packet_type == ffone_protocol::npc_map::NPC_MAP_SNAPSHOT {
        match world_spawn.world_modes.world_map_production.server_npcs.receive(&frame.payload) {
            Ok(true) => world_spawn.world_modes.world_map_production.npc_sources_dirty = true,
            Ok(false) => {},
            Err(error) => runtime.message = format!("NPC map snapshot rejected: {error}"),
        }
        return;
    }
    if matches!(frame.packet_type, 0x31000138 | 0x31000139) {
        if world_spawn.npc_modes.barber.0.len() < 8 {
            world_spawn.npc_modes.barber.0.push_back(frame);
        }
        return;
    }
    if frame.packet_type == packet::P_FE2CL_REP_SHINY_PICKUP_SUCC {
        match ffone_protocol::wire_0104::ShinyPickupSuccess0104::decode(&frame.payload) {
            Ok(reply) => {
                let modes = &mut world_spawn.world_modes;
                // `cnDisplayMapName.ShinyIcon` always queues the icon; its
                // `AddEventString(5, ...)` chat line honours `bEventMessage1`.
                if let Some(message) = modes.reward_notices.receive_shiny(
                    &reply,
                    &modes.tutorial_content,
                    &modes.localization,
                    &modes.language,
                ) && modes.option_runtime.options.display.npc_messages_in_chat
                {
                    let text = modes.localization.text(&modes.language, &message);
                    push_world_chat_line(runtime, ChatLineUi::receive(text).with_localized(message));
                }
            }
            Err(error) => warn!("Invalid Coco pickup reply: {error}"),
        }
        return;
    }

    if frame.packet_type == 0x3100_007a {
        if let Ok(state) = ffone_protocol::wire_0104::PcStateChange0104::decode(&frame.payload) {
            if runtime.player_id == Some(state.pc_id) && matches!(state.state, 0..=3) {
                world_spawn.inventory.local_vehicle.family = LegacyVehiclePresentationFamily::None;
                world_spawn.inventory.local_vehicle.pending = None;
            }
        }
    }
    match runtime.nano_recall.apply(&frame) {
        Ok(Some(message)) => nano_recall::notice(&mut world_spawn.social.nanocom_messages, message),
        Ok(None) => {}
        Err(error) => {
            runtime.message = format!("Recall packet rejected: {error}");
            return;
        }
    }

    if (0x31000107..=0x31000115).contains(&frame.packet_type) {
        if runtime.chat.gm.store_frames.len()<32 {runtime.chat.gm.store_frames.push_back(frame);}
        return;
    }
    if runtime.chat.gm.record_history {
        runtime.chat.gm.history.push_back((frame.packet_type, frame.payload.len()));
        if runtime.chat.gm.history.len() > 128 { runtime.chat.gm.history.pop_front(); }
    }
    if tutorial_owns_local_gameplay_frame(local_tutorial_hp_authority, frame.packet_type) {
        return;
    }
    if matches!(
        frame.packet_type,
        packet::P_FE2CL_PC_VEHICLE_ON_SUCC
            | packet::P_FE2CL_PC_VEHICLE_ON_FAIL
            | packet::P_FE2CL_PC_VEHICLE_OFF_SUCC
            | packet::P_FE2CL_PC_VEHICLE_OFF_FAIL
    ) {
        let expected_mount = matches!(
            frame.packet_type,
            packet::P_FE2CL_PC_VEHICLE_ON_SUCC | packet::P_FE2CL_PC_VEHICLE_ON_FAIL
        );
        vehicle_transition::finish_pending(
            &mut world_spawn.inventory.local_vehicle.pending,
            expected_mount,
        );
        match frame.packet_type {
            packet::P_FE2CL_PC_VEHICLE_ON_SUCC => {
                let item = world_spawn.inventory.inventory.snapshot().map(|snapshot| {
                    snapshot.equipment()[ffone_protocol::CharacterEquipSlot0104::Vehicle as usize]
                });
                let equip_type = item.and_then(|item| {
                    world_spawn
                        .world_modes
                        .tutorial_content
                        .gameplay_vehicle_equip_type(item.item_id)
                });
                let family = match equip_type
                    .and_then(LegacyVehiclePresentationFamily::from_legacy_equip_type)
                {
                    Some(LegacyVehiclePresentationFamily::Board) => {
                        LegacyVehiclePresentationFamily::Board
                    }
                    Some(LegacyVehiclePresentationFamily::Scooter) => {
                        LegacyVehiclePresentationFamily::Scooter
                    }
                    other => {
                        runtime.message = format!(
                            "Vehicle mount reply has unsupported clean m_iEquipType {other:?}"
                        );
                        return;
                    }
                };
                world_spawn.inventory.local_vehicle.family = family;
                if runtime.nano_slots.iter().any(|slot| slot.active) {
                    if let Err(error) = bridge.send(NetworkCommand::ActivateNano(ffone_protocol::NanoActiveRequest0104 { nano_slot: -1 })) {
                        runtime.message = format!("Vehicle Nano dismissal failed: {error}");
                    }
                }
                if let Ok((player, transform, ..)) = world_spawn.queries.local_player.single() {
                    world_spawn
                        .session
                        .gameplay_audio
                        .queue_player_vehicle_mode_edge(player, true);
                    world_spawn.session.effect_runtime.enqueue(TutorialEffectRuntimeCommand::Add {
                        effect_id: 832, placement: TutorialEffectPlacement::World { position: transform.translation, rotation: transform.rotation },
                        scale: 1.0, tracked: false, name: None, destroy_after_seconds: None, source_line: line!(),
                    });
                }
                return;
            }
            packet::P_FE2CL_PC_VEHICLE_OFF_SUCC => {
                world_spawn.inventory.local_vehicle.family = LegacyVehiclePresentationFamily::None;
                if let Ok((player, transform, ..)) = world_spawn.queries.local_player.single() {
                    world_spawn
                        .session
                        .gameplay_audio
                        .queue_player_vehicle_mode_edge(player, false);
                    world_spawn.session.effect_runtime.enqueue(TutorialEffectRuntimeCommand::Add {
                        effect_id: 832, placement: TutorialEffectPlacement::World { position: transform.translation, rotation: transform.rotation },
                        scale: 1.0, tracked: false, name: None, destroy_after_seconds: None, source_line: line!(),
                    });
                }
                return;
            }
            packet::P_FE2CL_PC_VEHICLE_ON_FAIL | packet::P_FE2CL_PC_VEHICLE_OFF_FAIL => {
                let error_code = frame
                    .payload
                    .get(..4)
                    .map(|bytes| i32::from_le_bytes(bytes.try_into().unwrap()))
                    .unwrap_or_default();
                runtime.message =
                    format!("OpenFusion rejected vehicle transition with error {error_code}");
                return;
            }
            _ => unreachable!(),
        }
    }
    match apply_world_weapon_warhead_frame(
        &frame,
        runtime,
        &world_spawn.session.weapon_animation_catalog,
        &mut world_spawn.session.effect_runtime,
        &mut world_spawn.queries,
    ) {
        Ok(true) => {
            world_spawn.session.world_combat.observe();
            return;
        }
        Ok(false) => {}
        Err(error) => {
            runtime.message = format!("Weapon warhead ignored: {error}");
            return;
        }
    }
    if runtime
        .player_id
        .is_some_and(|player_id| frame_observes_local_combat(&frame, player_id))
    {
        world_spawn.session.world_combat.observe();
    }
    match ffone_client::pc2pc_ui::decode_pc2pc_session_frame_0104(&frame) {
        Ok(Some(outcome)) => {
            let success_sound = matches!(&outcome,
                ffone_client::pc2pc_ui::Pc2pcServerOutcome0104::ConfirmSuccess { .. }
                | ffone_client::pc2pc_ui::Pc2pcServerOutcome0104::RegisterItemSuccess { .. }
                | ffone_client::pc2pc_ui::Pc2pcServerOutcome0104::RegisterCashSuccess { .. });
            let model = &mut world_spawn.social.pc2pc_ui;
            let mut next = (**model).clone();
            match next.apply_server_outcome(outcome, Default::default(), &*world_spawn.world_modes.tutorial_content, &ffone_client::pc2pc_ui::Pc2pcFailClosedEquipEligibility) {
                Ok(receipt) => {
                    if let Some(receipt) = receipt {
                        world_spawn.inventory.inventory.snapshot = next.completed_inventory();
                        runtime.candy = receipt.wallet_taros;
                    }
                    **model = next;
                    if success_sound {
                        world_spawn.session.gameplay_audio.queue_gameplay_ui_sound("Action_Sucess");
                    }
                }
                Err(error) => runtime.message = format!("Trade reply rejected: {error}"),
            }
            return;
        }
        Err(error) => { runtime.message = error; return; }
        Ok(None) => {}
    }
    if let Some(local_pc_id) = runtime.player_id {
        match world_spawn
            .social
            .pc2pc_offers
            .apply_frame(local_pc_id, &frame)
        {
            Ok(true) => {
                if let Some(identity) = world_spawn.social.pc2pc_offers.take_accepted() {
                    world_spawn
                        .social
                        .pc2pc_prompt
                        .dismiss(&mut world_spawn.npc_modes.system_messages);
                    if let Err(error) = open_world_pc2pc_accepted_session(
                        identity,
                        &runtime,
                        &world_spawn.inventory.inventory,
                        &world_spawn.world_modes.tutorial_content,
                        &world_spawn.queries.remote_appearances,
                        &mut world_spawn.social.pc2pc_ui,
                    ) {
                        runtime.message = format!("Pc2pcMode open failed: {error}");
                        world_spawn.social.pc2pc_ui.reset();
                    }
                } else if let Some(Pc2pcPendingOffer0104::Incoming(identity)) =
                    world_spawn.social.pc2pc_offers.pending()
                {
                    let display_name = world_spawn
                        .queries
                        .remote_appearances
                        .iter()
                        .find(|appearance| appearance.0.id == identity.remote_pc_id)
                        .map(world_remote_pc_display_name)
                        .unwrap_or_else(|| format!("Player {}", identity.remote_pc_id));
                    let text = world_spawn.world_modes.localization.text(
                        &world_spawn.world_modes.language,
                        &LocalizedText::new(
                            "ui.pc2pc.offer.incoming",
                            "{player} wants to trade with you.",
                        )
                        .with_arg("player", display_name),
                    );
                    world_spawn.social.pc2pc_prompt.open(
                        identity,
                        text,
                        &mut world_spawn.npc_modes.system_messages,
                    );
                } else {
                    world_spawn
                        .social
                        .pc2pc_prompt
                        .dismiss(&mut world_spawn.npc_modes.system_messages);
                }
                return;
            }
            Ok(false) => {}
            Err(error) => {
                runtime.message = format!("Pc2pcMode protocol fault: {error}");
                return;
            }
        }
    }
    if world_spawn.social.email_inbox.push(frame.clone()) {
        return;
    }
    if world_spawn
        .world_modes
        .combi_frames
        .push_if_owned(&world_spawn.world_modes.combi_runtime, frame.clone())
    {
        return;
    }
    if world_spawn
        .world_modes
        .enchant_frames
        .push_if_owned(&world_spawn.world_modes.enchant_runtime, frame.clone())
    {
        return;
    }
    if world_spawn
        .world_modes
        .race_frames
        .push_if_owned(frame.clone())
    {
        return;
    }

    match apply_world_mission_network_frame(
        &frame,
        &bridge,
        &world_spawn.world_modes.tutorial_content,
        &world_spawn.queries.mission_npcs,
        &mut world_spawn.inventory.inventory,
        (!local_tutorial_hp_authority).then_some(&mut world_spawn.world_modes.reward_notices),
        world_spawn
            .world_modes
            .option_runtime
            .options
            .display
            .npc_messages_in_chat,
        &mut world_spawn.world_modes.world_mission,
        &mut world_spawn.world_modes.mission_ui,
        &mut world_spawn.social.nanocom_messages,
        &mut world_spawn.session.gameplay_audio,
        runtime,
    ) {
        Ok(true) => return,
        Ok(false) => {}
        Err(error) => {
            runtime.message = format!("World mission ignored {error}");
            return;
        }
    }
    if gameplay_ui_actions::gm_runtime::reply(&frame, runtime, &world_spawn.world_modes.localization, &world_spawn.world_modes.language) {
        if frame.packet_type == 0x310000de && let Ok(reply) = ffone_protocol::wire_0104::PcMissionCompleteSuccess0104::decode(&frame.payload) {
            if let Err(error) = world_spawn.world_modes.world_mission.accept_gm_mission_completion(reply.mission_num) {
                runtime.message = error;
            }
        }
        return;
    }
    if let Ok(Some(reply)) = decode_gm_set_value_reply_0104(frame.packet_type, &frame.payload) {
        if runtime.player_id == Some(reply.pc_id) && matches!(reply.value_type, 1..=5 | 7) {
            match reply.value_type {
                1 => runtime.hp = Some(reply.value),
                2 => runtime.weapon_battery = reply.value,
                3 => runtime.nano_battery = reply.value,
                4 => {
                    runtime.fusion_matter = reply.value;
                    runtime.max_fusion_matter =
                        legacy_avatar_max_fusion_matter(runtime.player_level, reply.value);
                }
                5 => runtime.candy = reply.value,
                7 => {
                    if let Ok((entity, _, mut controller, _, _)) =
                        world_spawn.queries.local_player.single_mut()
                    {
                        movement_buffs::apply_gm_jump(
                            &mut controller,
                            world_spawn
                                .queries
                                .movement_bases
                                .get_mut(entity)
                                .ok()
                                .as_deref_mut(),
                            reply.value,
                        );
                    }
                }
                _ => unreachable!(),
            }
            return;
        }
    }
    match decode_local_gm_speed_reply_0104(&frame, runtime.player_id) {
        Ok(Some(speed)) => {
            let Ok((entity, _, mut controller, _, _)) =
                world_spawn.queries.local_player.single_mut()
            else {
                runtime.message = world_spawn.world_modes.localization.text(
                    &world_spawn.world_modes.language,
                    &LocalizedText::new(
                        "ui.chat.command.speed.controller_missing",
                        "Speed reply could not be applied: the player controller is unavailable.",
                    ),
                );
                return;
            };
            movement_buffs::apply_gm_speed(
                &mut controller,
                world_spawn
                    .queries
                    .movement_bases
                    .get_mut(entity)
                    .ok()
                    .as_deref_mut(),
                speed,
            );
            let feedback = world_spawn.world_modes.localization.text(
                &world_spawn.world_modes.language,
                &LocalizedText::new("ui.chat.command.speed.applied", "Speed changed to {speed}.")
                    .with_arg("speed", speed.to_string()),
            );
            runtime.message.clone_from(&feedback);
            push_world_chat_line(runtime, ChatLineUi::normal(feedback));
            return;
        }
        Ok(None) => {}
        Err(error) => {
            runtime.message = error;
            return;
        }
    }
    match apply_music_frame_0104(&frame, &mut world_spawn.session.music_requests) {
        Ok(true) => return,
        Ok(false) => {}
        Err(error) => {
            runtime.message = error;
            return;
        }
    }
    if let Ok(Some(message))=decode_server_message_0104(frame.packet_type,&frame.payload) {
        let text=message.message.to_string_lossy();
        if let Some(id)=text.strip_prefix("Quest ").and_then(|s|s.strip_suffix(" removed from completed missions.")).and_then(|s|s.parse::<i32>().ok()) {
            if let Err(error)=world_spawn.world_modes.world_mission.accept_gm_mission_reset(id) {runtime.message=error;}
        }
    }
    match apply_server_message_frame_0104(
        &frame,
        runtime,
        &world_spawn.world_modes.localization,
        &world_spawn.world_modes.language,
    ) {
        Ok(true) => return,
        Ok(false) => {}
        Err(error) => {
            runtime.message = error;
            return;
        }
    }
    if let Ok(Some(packet)) = decode_remote_frame(&frame) {
        world_spawn
            .world_modes
            .world_map_clock
            .observe(packet.server_time());
    }
    let hp_before_npc_skill = runtime.hp;
    match apply_world_npc_skill_response_frame(
        &frame,
        runtime,
        &mut world_spawn.inventory.skill_buff_ui,
        &mut world_spawn.world_modes.world_nano_authority,
    ) {
        Ok(true) => {
            if let Some(current_hp) = runtime.hp
                && let Some(edge) =
                    local_player_damage_audio_edge(hp_before_npc_skill, current_hp, false)
                && let Ok((player, ..)) = world_spawn.queries.local_player.single_mut()
            {
                queue_local_player_damage_audio(
                    &runtime,
                    player,
                    edge,
                    &mut world_spawn.session.gameplay_audio,
                );
            }
            if let Some(epoch) = lifecycle_session.active {
                // The lifecycle consumer applies the same fully
                // validated frame to remote entities and preserves
                // the clean NPC hit/corruption animation phase.
                lifecycle_ingress.push_frame(epoch, frame);
            }
            return;
        }
        Ok(false) => {}
        Err(error) => {
            runtime.message = format!("NPC skill reply rejected: {error}");
            return;
        }
    }
    match apply_world_nano_response_frame(
        &frame,
        runtime,
        &mut world_spawn.inventory.skill_buff_ui,
        &mut world_spawn.world_modes.world_nano_authority,
    ) {
        Ok(true) => {
            if matches!(
                frame.packet_type,
                packet::P_FE2CL_NANO_SKILL_USE_SUCC | packet::P_FE2CL_NANO_SKILL_USE
            ) && let Some(epoch) = lifecycle_session.active
            {
                // Remote PC/NPC projection shares the ordered
                // lifecycle stream with ordinary combat packets.
                lifecycle_ingress.push_frame(epoch, frame);
            }
            return;
        }
        Ok(false) => {}
        Err(error) => {
            runtime.message = format!("Nano gameplay reply rejected: {error}");
            return;
        }
    }
    if let Some(pending) = world_spawn
        .inventory
        .nano_free_tuning_model
        .pending_request()
        && let Some(correlated) =
            NetworkEvent::Frame(frame.clone()).correlate_nano_tune_0104(NanoTunePending0104 {
                request_token: pending.request_token,
                player_id: runtime.player_id.unwrap_or_default(),
                nano_id: pending.nano_id,
                skill_id: pending.skill_id,
            })
    {
        match correlated {
            Ok(reply) => {
                let envelope = project_correlated_nano_tune_reply(reply);
                if let Err(error) = world_spawn
                    .inventory
                    .nano_free_tuning_model
                    .apply_reply(envelope)
                {
                    runtime.message = format!("NanoFreeTuning reply rejected: {error}");
                } else {
                    runtime.message =
                        "OpenFusion Nano tune reply correlated; applying authoritative post-state"
                            .to_owned();
                }
            }
            Err(error) => {
                let fault = project_nano_tune_correlation_fault(error);
                let _ = world_spawn
                    .inventory
                    .nano_free_tuning_model
                    .reject_protocol_fault(fault);
                runtime.message = format!("NanoFreeTuning transport rejected: {fault:?}");
            }
        }
        return;
    }
    if matches!(
        frame.packet_type,
        packet::P_FE2CL_REP_NANO_TUNE_SUCC | packet::P_FE2CL_REP_NANO_TUNE_FAIL
    ) {
        runtime.message =
            "NanoFreeTuning ignored a tune reply without its local pending request".to_owned();
        return;
    }
    if frame.packet_type == packet::P_FE2CL_REP_NANO_BOOK_SUBSET {
        let result = ffone_protocol::wire_0104::NanoBookSubsetReply0104::decode(&frame.payload)
            .map_err(|error| error.to_string())
            .and_then(|page| {
                world_spawn
                    .inventory
                    .nano_free_tuning_bank
                    .apply_book_subset(&page, runtime.player_id.unwrap_or_default())
                    .map_err(|error| error.to_string())
            });
        if let Err(error) = result {
            runtime.message = format!("Nano book rejected: {error}");
            return;
        }
        for slot in &mut runtime.nano_slots {
            if let Some(id) = slot.nano_id
                && let Some(nano) = world_spawn
                    .inventory
                    .nano_free_tuning_bank
                    .entries()
                    .get(id as usize)
                    .filter(|nano| nano.id == id)
            {
                slot.skill_id = nano.skill_id;
                slot.stamina = nano.stamina;
            }
        }
        if !world_spawn
            .inventory
            .nano_free_tuning_production
            .modal_active(&world_spawn.inventory.nano_free_tuning_model)
        {
            world_spawn
                .inventory
                .nano_free_tuning_production
                .pending_open = world_spawn
                .inventory
                .nano_free_tuning_bank
                .first_untuned_index()
                .map(|nano_id| NanoFreeTuningOpenTrigger {
                    nano_id,
                    killed_fusion: false,
                });
        }
        return;
    }
    if frame.packet_type == packet::P_FE2CL_REP_NANO_UNEQUIP_SUCC {
        match ffone_protocol::wire_0104::NanoUnequipSuccess0104::decode(&frame.payload) {
            Ok(reply) => {
                if let Some(slot) = usize::try_from(reply.nano_slot_num)
                    .ok()
                    .and_then(|index| runtime.nano_slots.get_mut(index))
                {
                    if let Some(nano_id) = slot.nano_id
                        && world_spawn
                            .inventory
                            .user_equip_state
                            .acknowledge_nano_station_request(UserEquipNanoStationAction::Unequip {
                                nano_id,
                                slot: reply.nano_slot_num as usize,
                            })
                    {
                        world_spawn.inventory.nano_viewer.close();
                    }
                    *slot = RuntimeNanoSlot::default();
                    if reply.nano_deactive != 0 {
                        for equipped in &mut runtime.nano_slots {
                            equipped.active = false;
                        }
                    }
                }
            }
            Err(error) => runtime.message = format!("Nano unequip reply rejected: {error}"),
        }
        return;
    }
    if frame.packet_type == packet::P_FE2CL_REP_NANO_EQUIP_SUCC {
        match ffone_protocol::wire_0104::NanoEquipSuccess0104::decode(&frame.payload) {
            Ok(reply) => {
                let was_active = runtime
                    .nano_slots
                    .iter()
                    .any(|slot| slot.nano_id == Some(reply.nano_id) && slot.active);
                if let (Some(slot), Some(nano)) = (
                    usize::try_from(reply.nano_slot_num)
                        .ok()
                        .and_then(|index| runtime.nano_slots.get_mut(index)),
                    usize::try_from(reply.nano_id).ok().and_then(|index| {
                        world_spawn
                            .inventory
                            .nano_free_tuning_bank
                            .entries()
                            .get(index)
                    }),
                ) && nano.id == reply.nano_id
                    && nano.skill_id > 0
                {
                    if world_spawn
                        .inventory
                        .user_equip_state
                        .acknowledge_nano_station_request(UserEquipNanoStationAction::Equip {
                            nano_id: reply.nano_id,
                            slot: reply.nano_slot_num as usize,
                        })
                    {
                        world_spawn.inventory.nano_viewer.close();
                    }
                    *slot = RuntimeNanoSlot {
                        nano_id: Some(nano.id),
                        skill_id: nano.skill_id,
                        stamina: nano.stamina,
                        active: was_active && reply.nano_deactive == 0,
                    };
                    if reply.nano_deactive != 0 {
                        for equipped in &mut runtime.nano_slots {
                            equipped.active = false;
                        }
                    }
                }
            }
            Err(error) => runtime.message = format!("Nano equip reply rejected: {error}"),
        }
        return;
    }
    match decode_pc_nano_create_packet_0104(frame.packet_type, &frame.payload) {
        Ok(Some(PcNanoCreatePacket0104::Success(success))) => {
            let previous_level = runtime.player_level;
            if let Err(error) = apply_nano_create_commit(
                success,
                &mut world_spawn.inventory.inventory,
                &mut world_spawn.inventory.nano_free_tuning_bank,
                runtime,
            ) {
                runtime.message = format!("NanoFreeTuning Nano-create reply rejected: {error}");
                return;
            }
            if !local_tutorial_hp_authority {
                guide_nanocom::enqueue_level_up(
                    &world_spawn.world_modes.tutorial_content,
                    &world_spawn.npc_modes.guide_runtime,
                    previous_level,
                    runtime.player_level,
                    &mut world_spawn.social.nanocom_messages,
                );
            }
            world_spawn
                .inventory
                .nano_free_tuning_production
                .pending_open = world_spawn
                .inventory
                .nano_free_tuning_bank
                .first_untuned_index()
                .map(|nano_id| NanoFreeTuningOpenTrigger {
                    nano_id,
                    killed_fusion: true,
                });
            // Acquisition is presented by the Nano reveal/tuning UI. Keep the
            // numeric event in diagnostics instead of publishing it to chat.
            runtime.message = format!("Nano acquisition applied: {}", success.nano.id);
            return;
        }
        Ok(Some(PcNanoCreatePacket0104::Failure(failure))) => {
            if runtime.player_id == Some(failure.pc_id) {
                let message = super::super::chat_commands::nano_create_failure_message(failure.error_code);
                let modes = &world_spawn.world_modes;
                runtime.message = modes.localization.text(&modes.language, &message);
                push_world_chat_line(runtime, ChatLineUi::receive(runtime.message.clone()).with_localized(message));
            } else {
                runtime.message = format!(
                    "Ignored Nano-create failure for PC {}; local PC is {:?}",
                    failure.pc_id, runtime.player_id
                );
            }
            return;
        }
        Ok(None) => {}
        Err(error) => {
            runtime.message =
                format!("NanoFreeTuning malformed Nano-create reply rejected: {error}");
            return;
        }
    }
    match PresentNpcTypesGameplayFrame0104::decode(frame.clone()) {
        PresentNpcTypesGameplayFrame0104::Decoded {
            packet: PresentNpcTypesPacket0104::Reply(reply),
            ..
        } => {
            if let Err(error) = apply_world_map_present_reply(
                &mut world_spawn.world_modes.world_map,
                *world_spawn.world_modes.world_map_clock,
                &reply,
            ) {
                runtime.message = format!("WorldMap present-NPC-types reply rejected: {error}");
            }
            return;
        }
        PresentNpcTypesGameplayFrame0104::Decoded {
            packet: PresentNpcTypesPacket0104::Request(_),
            ..
        } => {
            runtime.message =
                "WorldMap ignored a client-direction present-NPC-types request from the shard"
                    .to_owned();
            return;
        }
        PresentNpcTypesGameplayFrame0104::Malformed { error, .. } => {
            runtime.message =
                format!("WorldMap malformed present-NPC-types reply rejected: {error:?}");
            return;
        }
        PresentNpcTypesGameplayFrame0104::Passthrough(_) => {}
    }
    match apply_user_equip_frame(
        &frame,
        &mut world_spawn.inventory.inventory,
        &world_spawn.world_modes.tutorial_content,
        &mut world_spawn.inventory.user_equip_production,
        &world_spawn.npc_modes.bank_production,
        &world_spawn.npc_modes.vendor_production,
        runtime,
    ) {
        Ok(true) => return,
        Ok(false) => {}
        Err(error) => {
            runtime.message = error;
            return;
        }
    }
    match apply_vendor_frame(
        &frame,
        &bridge,
        &world_spawn.world_modes.tutorial_content,
        &mut world_spawn.inventory.inventory,
        &mut world_spawn.npc_modes.vendor_state,
        &mut world_spawn.npc_modes.vendor_production,
        &mut world_spawn.npc_modes.vendor_system_runtime,
        &mut world_spawn.npc_modes.system_messages,
        runtime,
        &mut world_spawn.session.gameplay_audio,
    ) {
        Ok(true) => return,
        Ok(false) => {}
        Err(error) => {
            runtime.message = error;
            return;
        }
    }
    let quit_reply = match decode_quit_menu_exit_reply(&frame) {
        Ok(reply) => reply,
        Err(error) => {
            runtime.message = format!("QuitMenu ignored {error}");
            return;
        }
    };
    if let Some(reply) = quit_reply {
        let reply_pc_id = match reply {
            QuitMenuExitReply::Failure { pc_id, .. } | QuitMenuExitReply::Success { pc_id, .. } => {
                pc_id
            }
        };
        if runtime.player_id != Some(reply_pc_id) {
            runtime.message = format!(
                "QuitMenu ignored PC-exit reply for {reply_pc_id}; local runtime PC is {:?}",
                runtime.player_id
            );
            return;
        }
        world_spawn.npc_modes.quit_menu.set_enabled(true);
        match reply {
            QuitMenuExitReply::Failure { error_code, .. } => {
                let destination = world_spawn.npc_modes.quit_runtime.finish();
                runtime.message = format!(
                    "OpenFusion rejected {:?} PC-exit request with error {error_code}",
                    destination
                );
                let _ = bridge.send(NetworkCommand::Disconnect);
                next_state.set(ClientState::Login);
            }
            QuitMenuExitReply::Success { exit_code: 1, .. } => {
                let Some(destination) = world_spawn.npc_modes.quit_runtime.finish() else {
                    runtime.message =
                        "OpenFusion confirmed a PC exit without a pending QuitMenu destination"
                            .to_owned();
                    return;
                };
                if let Err(error) = complete_quit_menu_destination(
                    destination,
                    &bridge,
                    &mut world_spawn.session.active_credentials,
                    runtime,
                    next_state,
                    &mut world_spawn.session.app_exit,
                ) {
                    runtime.message = format!("QuitMenu destination transition failed: {error}");
                }
            }
            QuitMenuExitReply::Success { exit_code, .. } => {
                world_spawn.npc_modes.quit_runtime.finish();
                let message = clean_pc_exit_code_message(exit_code)
                    .unwrap_or("Unknown server disconnection.");
                runtime.message = format!("OpenFusion PC exit code {exit_code}: {message}");
                // Clean displays the corresponding EndGame popup
                // before its external callback. Until that callback
                // surface is native, leave the dead shard safely and
                // return to Login instead of pretending the world is
                // still interactive.
                if let Err(error) = bridge.send(NetworkCommand::Disconnect) {
                    runtime.message =
                        format!("OpenFusion PC exit code {exit_code}: {message} ({error})");
                }
                next_state.set(ClientState::Login);
            }
        }
        return;
    }
    let mentor_reply = match apply_guide_mentor_reply_transactional(
        &frame,
        &mut world_spawn.npc_modes.guide_runtime,
        &mut world_spawn.npc_modes.guide_ui,
        &mut world_spawn.npc_modes.guide_outbox,
    ) {
        Ok(reply) => reply,
        Err(error) => {
            runtime.message = format!("Guide mode ignored {error}");
            return;
        }
    };
    if let Some(reply) = mentor_reply {
        match reply {
            GuideMentorReplyOutcome::Success {
                mentor,
                raw_mentor_count,
                fusion_matter,
                intent,
            } => {
                runtime.fusion_matter = fusion_matter;
                runtime.max_fusion_matter =
                    legacy_avatar_max_fusion_matter(runtime.player_level, fusion_matter);
                if matches!(intent, GuidePostChangeIntent::RefreshGuideMissions) {
                    enqueue_login_guide_nanocom_0104(
                        &world_spawn.social.email_catalog,
                        &world_spawn.world_modes.tutorial_content,
                        &world_spawn.world_modes.world_mission,
                        &world_spawn.npc_modes.guide_runtime,
                        &world_spawn.inventory.nano_free_tuning_bank,
                        &world_spawn.inventory.inventory,
                        i32::from(runtime.player_level),
                        &mut world_spawn.social.nanocom_messages,
                    );
                }
                runtime.message = format!(
                    "OpenFusion accepted Guide {} (raw count {}, follow-up {:?})",
                    mentor.name(),
                    raw_mentor_count,
                    intent
                );
            }
            GuideMentorReplyOutcome::Failure { mentor, error_code } => {
                runtime.message = format!(
                    "OpenFusion rejected Guide {} with error {error_code}",
                    mentor.name()
                );
            }
        }
        return;
    }
    if matches!(
        frame.packet_type,
        TRANSPORTATION_REGISTRATION_SUCCESS_PACKET_ID
            | TRANSPORTATION_REGISTRATION_FAILURE_PACKET_ID
    ) {
        match decode_transportation_registration_reply_0104(frame.packet_type, &frame.payload) {
            Ok(TransportationRegistrationReply0104::Success {
                transportation_type,
                location_id,
                unlocks,
            }) => {
                let Some(registration) = world_spawn
                    .world_modes
                    .transportation_production
                    .take_registration_reply(transportation_type, location_id)
                else
                {
                    runtime.message = format!(
                        "Ignored uncorrelated transportation registration success for type {transportation_type}, location {location_id}"
                    );
                    return;
                };
                let newly_registered =
                    registration.became_registered(runtime.transportation_unlocks, unlocks);
                runtime.transportation_unlocks = unlocks;
                // Clean `NpcIconMode.ReceivePacket` turns this reply into its
                // letterbox `TopString`; the next `InitMode` clears it.
                if newly_registered {
                    world_spawn
                        .world_modes
                        .mission_ui
                        .show_transportation_registration_notice(transportation_type);
                }
                world_spawn
                    .world_modes
                    .transportation_production
                    .last_handled_packet = Some(frame.packet_type);
                runtime.message = format!(
                    "Transportation location {location_id} registered for type {transportation_type}; unlock flags synchronized"
                );
            }
            Ok(TransportationRegistrationReply0104::Failure {
                transportation_type,
                location_id,
                error_code,
            }) => {
                if world_spawn
                    .world_modes
                    .transportation_production
                    .take_registration_reply(transportation_type, location_id)
                    .is_none()
                {
                    runtime.message = format!(
                        "Ignored uncorrelated transportation registration failure for type {transportation_type}, location {location_id}"
                    );
                    return;
                }
                world_spawn
                    .world_modes
                    .transportation_production
                    .last_handled_packet = Some(frame.packet_type);
                runtime.message = format!(
                    "Transportation location {location_id} registration for type {transportation_type} failed with error {error_code}"
                );
            }
            Err(error) => {
                runtime.message =
                    format!("Malformed transportation registration reply rejected: {error:?}");
            }
        }
        return;
    }
    if matches!(
        frame.packet_type,
        TRANSPORTATION_SUCCESS_PACKET_ID | TRANSPORTATION_FAILURE_PACKET_ID
    ) {
        if world_spawn.world_modes.transportation_model.phase()
            != TransportationPhase::AwaitingServer
        {
            runtime.message = format!(
                "Ignored uncorrelated transportation reply 0x{:08x}",
                frame.packet_type
            );
            return;
        }
        match decode_transportation_warp_reply_0104(frame.packet_type, &frame.payload) {
            Ok(TransportationWarpReply0104::Failure {
                transportation_id,
                error_code,
            }) => {
                if world_spawn
                    .world_modes
                    .transportation_production
                    .take_warp_failure(transportation_id)
                    .is_none()
                {
                    runtime.message = format!(
                        "Ignored transportation failure for unowned transportation {transportation_id}"
                    );
                    return;
                }
                world_spawn
                    .world_modes
                    .transportation_model
                    .receive_failure(error_code);
                runtime.message = format!(
                    "OpenFusion rejected transportation {transportation_id} with error {error_code}"
                );
            }
            Ok(TransportationWarpReply0104::Success {
                transportation_type,
                position: server_position,
                candy,
            }) => {
                let Some(lease) = world_spawn
                    .world_modes
                    .transportation_production
                    .take_warp_success(transportation_type)
                else {
                    runtime.message = format!(
                        "Ignored transportation success for unowned type {transportation_type}"
                    );
                    return;
                };
                runtime.candy = candy;
                let position = ProtocolPosition::new(server_position).to_native();
                let projection_result = match lease.transportation_type {
                    1 => {
                        runtime.map_name = legacy_world_location_name_or_last(
                            &runtime.map_name,
                            -position.x,
                            position.z,
                        )
                        .map(str::to_owned)
                        .unwrap_or_else(|| {
                            runtime.map_number.map_or_else(
                                || "unknown".to_owned(),
                                |map_number| format!("Map {map_number}"),
                            )
                        });
                        match world_spawn.queries.local_player.single_mut() {
                            Ok((
                                player,
                                mut transform,
                                mut controller,
                                mut visibility,
                                mut environment,
                            )) => Ok(Some(apply_authoritative_world_teleport(
                                commands,
                                &mut world_spawn.session.gameplay_loading,
                                player,
                                &mut transform,
                                &mut controller,
                                &mut visibility,
                                &mut environment,
                                position,
                                runtime.hp,
                            ))),
                            Err(error) => Err(format!(
                                "transportation warp has no unique local player: {error}"
                            )),
                        }
                    }
                    2 => {
                        world_spawn
                            .world_modes
                            .transportation_production
                            .skyway_active = true;
                        world_spawn
                            .world_modes
                            .transportation_production
                            .skyway_motion = Some(SkywayTraversalMotion::new(position));
                        world_spawn
                            .world_modes
                            .transportation_production
                            .movement_packet_emission = false;
                        match world_spawn.queries.local_player.single_mut() {
                            Ok((_, mut transform, mut controller, _, _)) => {
                                transform.translation = position;
                                controller.apply_authoritative_teleport(position);
                                controller.movement_enabled = false;
                                Ok(None)
                            }
                            Err(error) => Err(format!(
                                "transportation skyway has no unique local player: {error}"
                            )),
                        }
                    }
                    _ => unreachable!("owned NPC transportation is type 1 or 2"),
                };
                world_spawn
                    .world_modes
                    .transportation_model
                    .receive_success();
                world_spawn
                    .world_modes
                    .transportation_production
                    .window_fade_in_requested = false;
                runtime.message = match projection_result {
                    Ok(Some(true)) => format!(
                        "Transportation type {transportation_type} accepted at ({}, {}, {}); Candy synchronized and destination collision is loading",
                        server_position[0], server_position[1], server_position[2]
                    ),
                    Ok(Some(false)) => format!(
                        "Transportation type {transportation_type} accepted at ({}, {}, {}); Candy synchronized",
                        server_position[0], server_position[1], server_position[2]
                    ),
                    Ok(None) => format!(
                        "Transportation type 2 accepted; Candy synchronized and authoritative Skyway movement owns the avatar"
                    ),
                    Err(error) => format!(
                        "Transportation state synchronized, but authoritative world projection was blocked: {error}"
                    ),
                };
            }
            Err(error) => {
                runtime.message =
                    format!("Malformed transportation warp reply rejected: {error:?}");
            }
        }
        return;
    }
    match decode_transportation_skyway_frame_0104(frame.packet_type, &frame.payload) {
        Ok(Some(TransportationSkywayFrame0104::Move {
            pc_id,
            position: server_position,
            speed,
        })) if runtime.player_id == Some(pc_id) => {
            if !world_spawn
                .world_modes
                .transportation_production
                .skyway_active
            {
                runtime.message =
                    format!("Ignored uncorrelated local Skyway movement for PC {pc_id}");
                return;
            }
            if speed < 0 {
                runtime.message =
                    format!("Rejected local Skyway movement with negative speed {speed}");
                return;
            }
            let position = ProtocolPosition::new(server_position).to_native();
            let projection = match world_spawn.queries.local_player.single_mut() {
                Ok((_, mut transform, mut controller, _, _)) => {
                    let motion = world_spawn
                        .world_modes
                        .transportation_production
                        .skyway_motion
                        .get_or_insert_with(|| SkywayTraversalMotion::new(transform.translation));
                    motion.retarget(&mut transform.translation, position);
                    controller.movement_enabled = false;
                    Ok(())
                }
                Err(error) => Err(format!(
                    "Skyway movement has no unique local player: {error}"
                )),
            };
            if projection.is_ok() {
                runtime.map_name =
                    legacy_world_location_name_or_last(&runtime.map_name, -position.x, position.z)
                        .map(str::to_owned)
                        .unwrap_or_else(|| {
                            runtime.map_number.map_or_else(
                                || "unknown".to_owned(),
                                |map_number| format!("Map {map_number}"),
                            )
                        });
                runtime.message = format!(
                    "Authoritative Skyway movement applied for PC {pc_id} at speed {speed}"
                );
            } else if let Err(error) = projection {
                runtime.message = error;
            }
            return;
        }
        Ok(Some(TransportationSkywayFrame0104::Dismount { pc_id, riding_type }))
            if runtime.player_id == Some(pc_id) =>
        {
            if !world_spawn
                .world_modes
                .transportation_production
                .skyway_active
                || riding_type != 0
            {
                runtime.message = format!(
                    "Ignored uncorrelated local Skyway dismount for PC {pc_id}, type {riding_type}"
                );
                return;
            }
            world_spawn
                .world_modes
                .transportation_production
                .skyway_active = false;
            world_spawn
                .world_modes
                .transportation_production
                .movement_packet_emission = true;
            let final_target = world_spawn
                .world_modes
                .transportation_production
                .skyway_motion
                .take()
                .map(|motion| motion.target);
            match world_spawn.queries.local_player.single_mut() {
                Ok((_, mut transform, mut controller, _, _)) => {
                    if let Some(target) = final_target
                        && transform.translation.distance(target) > 5.0
                    {
                        transform.translation = target;
                    }
                    controller.apply_authoritative_teleport(transform.translation);
                    controller.movement_enabled = true;
                    runtime.message = format!("Authoritative Skyway ride completed for PC {pc_id}");
                }
                Err(error) => {
                    runtime.message =
                        format!("Skyway dismount has no unique local player: {error}");
                }
            }
            return;
        }
        Ok(Some(_)) | Ok(None) => {}
        Err(error) => {
            runtime.message = format!("Malformed transportation Skyway frame rejected: {error}");
            return;
        }
    }
    let warp_reply = match decode_authoritative_warp_reply_0104(&frame) {
        Ok(reply) => reply,
        Err(error) => {
            runtime.message = format!("Authoritative warp ignored {error}");
            return;
        }
    };
    if let Some(reply) = warp_reply {
        let (server_position, npc_success) = match reply {
            AuthoritativeWarpReply0104::NpcFailure(failure) => {
                let pending_normal = world_spawn.npc_modes.normal_npc_warp.take_correlated_sent();
                if let Some(pending) = pending_normal {
                    if let Ok((_, _, mut controller, _, _)) =
                        world_spawn.queries.local_player.single_mut()
                    {
                        controller.movement_enabled = true;
                    }
                    pending
                        .identity
                        .reject_ui(&mut world_spawn.world_modes.mission_ui);
                    if let Err(error) = world_spawn.npc_modes.normal_npc_warp.queue_system_message(
                        &world_spawn.world_modes.tutorial_content,
                        &mut world_spawn.npc_modes.system_messages,
                        NORMAL_NPC_WARP_FAILURE_MESSAGE_ID,
                    ) {
                        runtime.message = format!(
                            "OpenFusion rejected normal NPC warp {} with error {}; SystemMessage 173 failed: {error}",
                            pending.identity.warp_id, failure.error_code
                        );
                    } else {
                        runtime.message = format!(
                            "OpenFusion rejected normal NPC warp {} with error {}; clean SystemMessage 173 opened",
                            pending.identity.warp_id, failure.error_code
                        );
                    }
                } else if world_spawn
                    .npc_modes
                    .guide_production
                    .pending_warp
                    .is_some_and(|pending| pending.sent)
                {
                    world_spawn.npc_modes.guide_production.pending_warp = None;
                    world_spawn.npc_modes.guide_production.push_system_message(
                        &mut world_spawn.npc_modes.system_messages,
                        GUIDE_WARP_FAILURE_MESSAGE_ID,
                        format!("Warp Fail (Code:{})", failure.error_code),
                    );
                    runtime.message = format!(
                        "OpenFusion rejected Guide warp with error {}",
                        failure.error_code
                    );
                } else {
                    runtime.message = format!(
                        "OpenFusion rejected an uncorrelated NPC warp with error {}",
                        failure.error_code
                    );
                }
                return;
            }
            AuthoritativeWarpReply0104::NpcSuccess(success) => {
                world_spawn.session.instance_audio.active = false;
                runtime.nano_recall.leave_instance();
                (success.position, Some(success))
            }
            AuthoritativeWarpReply0104::GotoSuccess(success) => (success.position, None),
        };

        // A warp can change instance even when the public map number is unchanged.
        // Clear old markers now and fetch the new server placements after its rate limit.
        world_spawn.world_modes.world_map.reset_session();
        let map = &mut world_spawn.world_modes.world_map_production;
        map.server_npcs = default();
        map.observed_npcs.clear();
        map.combined_npc_sources.clear();
        map.npc_sources_dirty = true;
        map.snapshot_retry_seconds = 1.1;

        let pending_guide = world_spawn
            .npc_modes
            .guide_production
            .pending_warp
            .filter(|pending| pending.sent);
        let pending_normal = world_spawn.npc_modes.normal_npc_warp.correlated_sent();
        if let Some(pending) = pending_guide {
            runtime.map_number = Some(pending.map_number);
        }
        if let Some(pending) = pending_normal {
            runtime.map_number = Some(pending.identity.target.map_id);
        }
        let position = ProtocolPosition::new(server_position).to_native();
        runtime.map_name =
            legacy_world_location_name_or_last(&runtime.map_name, -position.x, position.z)
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    runtime.map_number.map_or_else(
                        || "unknown".to_owned(),
                        |map_number| format!("Map {map_number}"),
                    )
                });
        if let Some(success) = npc_success {
            runtime.candy = success.candy;
        }

        let teleport_result = match world_spawn.queries.local_player.single_mut() {
            Ok((player, mut transform, mut controller, mut visibility, mut environment)) => {
                Ok(apply_authoritative_world_teleport(
                    commands,
                    &mut world_spawn.session.gameplay_loading,
                    player,
                    &mut transform,
                    &mut controller,
                    &mut visibility,
                    &mut environment,
                    position,
                    runtime.hp,
                ))
            }
            Err(error) => Err(format!(
                "authoritative warp has no unique local player: {error}"
            )),
        };

        if pending_guide.is_some() {
            world_spawn.npc_modes.guide_production.pending_warp = None;
        }
        if let Some(pending) = pending_normal {
            let cleared = world_spawn.npc_modes.normal_npc_warp.take_correlated_sent();
            debug_assert_eq!(cleared, Some(pending));
            pending
                .identity
                .confirm_ui(&mut world_spawn.world_modes.mission_ui);
        }
        let buddy_message = match complete_pending_buddy_warp_after_authoritative_teleport(
            &bridge,
            &mut world_spawn.social.buddy_runtime,
            &mut world_spawn.social.group_runtime,
            &mut world_spawn.npc_modes.system_messages,
        ) {
            Ok(message) => message,
            Err(error) => {
                runtime.message = error;
                return;
            }
        };

        runtime.message = match teleport_result {
            Err(error) => format!("Authoritative warp blocked: {error}"),
            Ok(_) if pending_normal.is_some() => {
                let pending = pending_normal.expect("checked above");
                if server_position
                    != [
                        pending.identity.target.x,
                        pending.identity.target.y,
                        pending.identity.target.z,
                    ]
                {
                    format!(
                        "Normal NPC warp {} accepted authoritative position {server_position:?} differing from validated row {:?}",
                        pending.identity.warp_id,
                        [
                            pending.identity.target.x,
                            pending.identity.target.y,
                            pending.identity.target.z,
                        ]
                    )
                } else if let Some(success) = npc_success {
                    format!(
                        "Normal NPC warp {} accepted; Candy synchronized (eIL {}, slot {})",
                        pending.identity.warp_id, success.e_il, success.item_slot_num
                    )
                } else {
                    format!(
                        "Normal NPC warp {} accepted by authoritative GOTO success",
                        pending.identity.warp_id
                    )
                }
            }
            Ok(_) if pending_guide.is_some() => {
                let pending = pending_guide.expect("checked above");
                if server_position != pending.target {
                    format!(
                        "Guide warp accepted an authoritative target {server_position:?} differing from validated row {:?}",
                        pending.target
                    )
                } else if let Some(success) = npc_success {
                    if success.e_il == 4 {
                        format!(
                            "Guide warp accepted at ({}, {}, {}); Candy synchronized",
                            server_position[0], server_position[1], server_position[2]
                        )
                    } else {
                        format!(
                            "Guide warp position/Candy applied, but inventory mutation eIL {} slot {} remains unavailable",
                            success.e_il, success.item_slot_num
                        )
                    }
                } else {
                    format!(
                        "Guide warp accepted at ({}, {}, {})",
                        server_position[0], server_position[1], server_position[2]
                    )
                }
            }
            Ok(_) if buddy_message.is_some() => buddy_message.expect("checked above"),
            Ok(true) => format!(
                "Authoritative server warp accepted at ({}, {}, {}); destination collision is loading",
                server_position[0], server_position[1], server_position[2]
            ),
            // OpenFusion emits NPC_WARP_SUCC followed by an exact
            // duplicate GOTO_SUCC. Preserve the correlated message
            // from the first packet instead of replacing it.
            Ok(false) => runtime.message.clone(),
        };
        return;
    }
    let local_resurrect = match decode_local_resurrect_packet_0104(&frame, runtime.player_id) {
        Ok(packet) => packet,
        Err(error) => {
            runtime.message = format!("ResurrectMode ignored {error}");
            return;
        }
    };
    match local_resurrect {
        Some(LocalResurrectPacket0104::Success(success)) => {
            world_spawn
                .session
                .gameplay_audio
                .queue_gameplay_ui_sound("Resurrectm_Warp");
            if world_spawn.world_modes.mission_ui.complete_warp_away() {
                world_spawn.npc_modes.system_messages.set_focus_out(false);
            }
            let previous_map = runtime.map_number;
            let position = apply_pc_regen_success_to_runtime(success, runtime);
            world_spawn.world_modes.world_map.reset_session();
            let map = &mut world_spawn.world_modes.world_map_production;
            map.server_npcs = default();
            map.observed_npcs.clear();
            map.combined_npc_sources.clear();
            map.npc_sources_dirty = true;
            map.snapshot_retry_seconds = 1.1;
            runtime.map_name =
                legacy_world_location_name_or_last(&runtime.map_name, -position.x, position.z)
                    .map(str::to_owned)
                    .unwrap_or_else(|| format!("Map {}", success.regen_data.map_number));
            let teleport_result = match world_spawn.queries.local_player.single_mut() {
                Ok((player, mut transform, mut controller, mut visibility, mut environment)) => {
                    Ok(apply_authoritative_world_teleport(
                        commands,
                        &mut world_spawn.session.gameplay_loading,
                        player,
                        &mut transform,
                        &mut controller,
                        &mut visibility,
                        &mut environment,
                        position,
                        runtime.hp,
                    ))
                }
                Err(error) => Err(format!("regeneration has no unique local player: {error}")),
            };
            let closed = world_spawn
                .inventory
                .resurrect_ui
                .accept_regen_success(&mut world_spawn.inventory.resurrect_outbox);
            runtime.message = match teleport_result {
                Err(error) => format!("OpenFusion regeneration blocked: {error}"),
                Ok(true)
                    if previous_map != Some(success.regen_data.map_number)
                        || success.move_location != 0 =>
                {
                    format!(
                        "OpenFusion regeneration accepted at map {} ({}, {}, {}); destination collision is loading",
                        success.regen_data.map_number,
                        success.regen_data.position[0],
                        success.regen_data.position[1],
                        success.regen_data.position[2]
                    )
                }
                Ok(true) if closed => {
                    "OpenFusion regeneration accepted; destination collision is loading"
                        .to_owned()
                }
                Ok(true) => {
                    "OpenFusion regeneration state applied outside ResurrectMode; destination collision is loading"
                        .to_owned()
                }
                Ok(false) if closed => {
                    "OpenFusion regeneration accepted; ResurrectMode closed".to_owned()
                }
                Ok(false) => {
                    "OpenFusion regeneration state applied outside ResurrectMode"
                        .to_owned()
                }
            };
            return;
        }
        Some(LocalResurrectPacket0104::SuddenDead { hp }) => {
            runtime.hp = Some(hp);
            if hp <= 0 {
                if world_spawn.world_modes.mission_ui.cancel_warp_away() {
                    world_spawn.npc_modes.system_messages.set_focus_out(false);
                }
                let entry_element = if world_spawn
                    .inventory
                    .nano_free_tuning_production
                    .modal_active(&world_spawn.inventory.nano_free_tuning_model)
                {
                    22
                } else {
                    0
                };
                world_spawn
                    .inventory
                    .resurrect_ui
                    .enter_from_authoritative_death(
                        entry_element,
                        &mut world_spawn.inventory.resurrect_outbox,
                    );
            }
            return;
        }
        None => {}
    }
    if frame.packet_type == packet::P_FE2CL_REP_CHARGE_NANO_STAMINA {
        if !local_tutorial_hp_authority {
            if let Err(error) = apply_nano_charge_frame(
                &frame,
                runtime,
                &mut world_spawn.inventory.nano_free_tuning_bank,
                &mut world_spawn.session.gameplay_audio,
            ) {
                runtime.message = error;
            }
        }
        return;
    }
    let hp_before_runtime_frame = runtime.hp;
    let infection_tick = (!local_tutorial_hp_authority
        && frame.packet_type == packet::P_FE2CL_CHAR_TIME_BUFF_TIME_TICK)
        .then(|| TimeBuffDotDamageTick0104::decode(&frame.payload).ok())
        .flatten()
        .filter(|tick| {
            tick.character_type == 1
                && runtime.player_id == Some(tick.character_id)
                && tick.time_buff_id == 17
        });
    let npc_attack_audio_result = (!local_tutorial_hp_authority)
        .then(|| {
            runtime
                .player_id
                .and_then(|player_id| local_player_npc_attack_result_from_frame(&frame, player_id))
        })
        .flatten();
    let previous_level = runtime.player_level;
    let updated_hp =
        apply_runtime_frame_with_authority(&frame, runtime, !local_tutorial_hp_authority);
    if !local_tutorial_hp_authority {
        guide_nanocom::enqueue_level_up(
            &world_spawn.world_modes.tutorial_content,
            &world_spawn.npc_modes.guide_runtime,
            previous_level,
            runtime.player_level,
            &mut world_spawn.social.nanocom_messages,
        );
    }
    if let Some(tick) = infection_tick
        && let Ok((player, transform, ..)) = world_spawn.queries.local_player.single_mut()
    {
        world_spawn
            .session
            .effect_runtime
            .enqueue(TutorialEffectRuntimeCommand::Add {
                effect_id: LOCAL_INFECTION_DAMAGE_EFFECT_ID,
                placement: TutorialEffectPlacement::World {
                    // Clean `CharacterController.center` is (0, 0.8, 0).
                    position: transform.translation + Vec3::Y * 0.8,
                    rotation: Quat::IDENTITY,
                },
                scale: 1.0,
                tracked: false,
                name: None,
                destroy_after_seconds: None,
                source_line: 0,
            });
        if tick.protected {
            world_spawn
                .session
                .effect_runtime
                .enqueue(TutorialEffectRuntimeCommand::Add {
                    effect_id: LOCAL_INFECTION_PROTECTION_EFFECT_ID,
                    placement: TutorialEffectPlacement::World {
                        position: transform.translation,
                        rotation: Quat::IDENTITY,
                    },
                    scale: 1.0,
                    tracked: false,
                    name: None,
                    destroy_after_seconds: None,
                    source_line: 0,
                });
        }
        if let Some(protocol_gender) = runtime
            .player_gender
            .and_then(|gender| i8::try_from(gender).ok())
            && let Ok(gender) = tutorial_player_gender_from_protocol(protocol_gender)
        {
            world_spawn
                .session
                .gameplay_audio
                .queue_player_infection_damage(player, gender);
        }
    }
    if let Some(result) = npc_attack_audio_result
        && let Some(edge) = local_player_damage_audio_edge(
            hp_before_runtime_frame,
            result.hp,
            result.hit_flag & 2 != 0,
        )
        && let Ok((player, ..)) = world_spawn.queries.local_player.single_mut()
    {
        queue_local_player_damage_audio(
            &runtime,
            player,
            edge,
            &mut world_spawn.session.gameplay_audio,
        );
    }
    if updated_hp.is_some_and(|hp| hp <= 0) {
        if world_spawn.world_modes.mission_ui.cancel_warp_away() {
            world_spawn.npc_modes.system_messages.set_focus_out(false);
        }
        let entry_element = if world_spawn
            .inventory
            .nano_free_tuning_production
            .modal_active(&world_spawn.inventory.nano_free_tuning_model)
        {
            22
        } else {
            0
        };
        world_spawn
            .inventory
            .resurrect_ui
            .enter_from_authoritative_death(
                entry_element,
                &mut world_spawn.inventory.resurrect_outbox,
            );
    }
    apply_group_and_freechat_frame(
        &frame,
        commands,
        asset_server,
        bridge,
        &world_spawn.world_modes.tutorial_content,
        &mut world_spawn.social,
        &mut world_spawn.npc_modes.system_messages,
        &world_spawn.queries.remote_appearances,
        &world_spawn.queries.remote_players,
        &world_spawn.queries.local_identities,
        &mut world_spawn.queries.remote_animations,
        &mut world_spawn.session.player_commands,
        &world_spawn.inventory.local_vehicle,
        &world_spawn.world_modes.option_runtime,
        runtime,
    );
    let bank_owned = match apply_bank_network_frame(
        &frame,
        commands,
        &asset_server,
        &bridge,
        &world_spawn.world_modes.tutorial_content,
        &mut world_spawn.inventory.inventory,
        &mut world_spawn.npc_modes.bank_state,
        &mut world_spawn.npc_modes.bank_modal,
        &mut world_spawn.npc_modes.bank_projection,
        &mut world_spawn.npc_modes.bank_outbox,
        &mut world_spawn.npc_modes.bank_production,
        &mut world_spawn.npc_modes.bank_system_runtime,
        &mut world_spawn.npc_modes.system_messages,
        runtime,
    ) {
        Ok(owned) => owned,
        Err(error) => {
            runtime.message = error;
            true
        }
    };
    if !bank_owned
        && let Err(error) = apply_inventory_frame(
            &frame,
            &mut world_spawn.inventory.inventory,
            &world_spawn.world_modes.tutorial_content,
            runtime,
        )
    {
        runtime.message = format!("Inventory ignored {error}");
    }
    if let Err(error) = apply_quick_slot_frame(&frame, &mut world_spawn.inventory.quick_slot_ui) {
        runtime.message = format!("QuickSlot ignored {error}");
    }
    if let Err(error) = apply_skill_buff_frame(
        &frame,
        &mut world_spawn.inventory.skill_buff_ui,
        &mut world_spawn.inventory.movement_buffs,
    ) {
        runtime.message = format!("Skill buff HUD ignored {error}");
    }
    if let Err(error) = apply_special_state_frame(&frame, runtime) {
        runtime.message = format!("Special state ignored {error}");
    }
    apply_buddy_chat_frame(
        &frame,
        bridge,
        &mut world_spawn.social,
        &mut world_spawn.npc_modes.system_messages,
        &world_spawn.queries.remote_appearances,
        &world_spawn.world_modes.gameplay_ui,
        runtime,
    );
    if let Some(epoch) = lifecycle_session.active {
        lifecycle_ingress.push_frame(epoch, frame);
    } else {
        runtime.message = "Protocol order error: live frame arrived before WorldReady".to_owned();
    }
}
