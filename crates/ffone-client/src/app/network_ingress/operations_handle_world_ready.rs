use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_world_ready(
    commands: &mut Commands,
    config: &ClientConfig,
    bridge: &NetworkBridge,
    runtime: &mut RuntimeStatus,
    tutorial_session: &mut TutorialSession,
    next_state: &mut NextState<ClientState>,
    lifecycle_ingress: &mut NetworkEntityLifecycleIngress0104,
    lifecycle_session: &mut NetworkLifecycleSession,
    asset_server: &AssetServer,
    world_catalog: &NativeWorldCatalog,
    character_data: &LoadedCharacterCreationData,
    rig_catalog: &NativePlayerRigCatalog,
    world_spawn: &mut WorldSpawnRuntime<'_, '_>,
    world_slice: &Query<Entity, With<WorldSliceEntity>>,
    world: WorldReady,
) {
    world_spawn.session.world_combat.clear();
    let Some(selected_character_index) = runtime
        .roster
        .characters
        .iter()
        .position(|character| character.pc_uid == world.pc_uid)
    else {
        let failure = format!(
            "Native player blocked: WorldReady UID {} has no login character summary",
            world.pc_uid
        );
        rollback_failed_world_ready_spawn(
            commands,
            lifecycle_ingress,
            lifecycle_session,
            runtime,
            &mut world_spawn.inventory.inventory,
            &mut world_spawn.session.effect_runtime,
            &mut world_spawn.session.player_commands,
            &mut world_spawn.session.rig_assets,
            &mut world_spawn.session.gameplay_loading,
            ResourceLoadingScope::World,
        );
        let credentials = world_spawn.session.active_credentials.credentials.clone();
        runtime.message = recover_failed_world_ready_connection(
            &bridge,
            &config,
            credentials.as_ref(),
            next_state,
            &mut world_spawn.session.gameplay_loading,
            failure,
        );
        return;
    };
    let mut selected_character = runtime.roster.characters[selected_character_index].clone();
    selected_character.position = world.position;
    // OpenFusion deliberately omits PCStyle2 from PC_ENTER_SUCC;
    // the legacy client keeps these flags from CHAR_INFO. The
    // network worker also updates this retained flag after the
    // fire-and-forget SAVE_CHAR_TUTOR packet, before CHAR_SELECT.
    selected_character.style = world.login_style;
    // The shard PC load owns current equipment after entry. Feed
    // that authoritative state to both look assembly and the
    // initial rigid hand attachment.
    selected_character.equipment =
        world
            .load
            .equipment()
            .map(|item| ffone_protocol::EquippedItem0104 {
                item_type: item.item_type,
                item_id: item.item_id,
                option: item.option,
                time_limit: item.time_limit,
            });
    let world_scope = match native_world_scope(world.login_style.tutorial_flag) {
        Ok(scope) => scope,
        Err(error) => {
            let failure = format!("Native world blocked: {error}");
            rollback_failed_world_ready_spawn(
                commands,
                lifecycle_ingress,
                lifecycle_session,
                runtime,
                &mut world_spawn.inventory.inventory,
                &mut world_spawn.session.effect_runtime,
                &mut world_spawn.session.player_commands,
                &mut world_spawn.session.rig_assets,
                &mut world_spawn.session.gameplay_loading,
                ResourceLoadingScope::World,
            );
            let credentials = world_spawn.session.active_credentials.credentials.clone();
            runtime.message = recover_failed_world_ready_connection(
                &bridge,
                &config,
                credentials.as_ref(),
                next_state,
                &mut world_spawn.session.gameplay_loading,
                failure,
            );
            return;
        }
    };
    let mut nano_bank = NanoFreeTuningBank0104::default();
    if let Err(error) =
        nano_bank.seed_with_bootstrap(&world.load, world.player_id, &world.bootstrap)
    {
        rollback_failed_world_ready_spawn(
            commands,
            lifecycle_ingress,
            lifecycle_session,
            runtime,
            &mut world_spawn.inventory.inventory,
            &mut world_spawn.session.effect_runtime,
            &mut world_spawn.session.player_commands,
            &mut world_spawn.session.rig_assets,
            &mut world_spawn.session.gameplay_loading,
            ResourceLoadingScope::World,
        );
        let credentials = world_spawn.session.active_credentials.credentials.clone();
        runtime.message = recover_failed_world_ready_connection(
            &bridge,
            &config,
            credentials.as_ref(),
            next_state,
            &mut world_spawn.session.gameplay_loading,
            format!("Native world blocked: {error}"),
        );
        return;
    }
    info!(
        nano_bank_size = nano_bank.entries().len(),
        "Nano book bootstrap accepted"
    );
    info!(
        "WorldReady accepted: uid={} player={} tutorial_flag={} scope={world_scope:?} position={:?} angle={}",
        world.pc_uid, world.player_id, world.login_style.tutorial_flag, world.position, world.angle
    );
    // Scene/map/rig assembly is the only fallible part of entry.
    // Keep every authoritative model and lifecycle packet staged
    // until it succeeds so a bad native asset cannot publish a
    // half-entered shard to the rest of the app.
    if let Err(error) = spawn_native_world_slice(
        commands,
        &asset_server,
        &world_catalog,
        &mut world_spawn.session.rig_assets,
        &rig_catalog,
        &world_spawn.session.weapon_animation_catalog,
        &character_data.0,
        &selected_character,
        world.pc_uid,
        world.player_id,
        world_scope,
        world.position,
        world.angle,
        true,
        &mut world_spawn.session.gameplay_loading,
        Some(&mut world_spawn.session.player_commands),
    ) {
        rollback_failed_world_ready_spawn(
            commands,
            lifecycle_ingress,
            lifecycle_session,
            runtime,
            &mut world_spawn.inventory.inventory,
            &mut world_spawn.session.effect_runtime,
            &mut world_spawn.session.player_commands,
            &mut world_spawn.session.rig_assets,
            &mut world_spawn.session.gameplay_loading,
            match world_scope {
                NativeWorldScope::Tutorial => ResourceLoadingScope::Tutorial,
                NativeWorldScope::WorldMap => ResourceLoadingScope::World,
            },
        );
        let failure = format!("Native world assembly failed: {error}");
        let credentials = world_spawn.session.active_credentials.credentials.clone();
        runtime.message = recover_failed_world_ready_connection(
            &bridge,
            &config,
            credentials.as_ref(),
            next_state,
            &mut world_spawn.session.gameplay_loading,
            failure,
        );
        return;
    }
    info!(
        "Native world slice assembled: uid={} scope={world_scope:?} position={:?}",
        world.pc_uid, world.position
    );
    runtime.roster.characters[selected_character_index] = selected_character.clone();
    runtime.roster.pending_character_entry_uid = None;
    // WorldReady is the ownership boundary for both tutorial and ordinary
    // gameplay. The tutorial flag selects authored map scope, but the shard
    // owns the player, NPC lifecycle, mechanics, and authoritative state in
    // both cases.
    world_spawn.session.effect_runtime.clear_scene_instances();
    if let Some(epoch) = lifecycle_session.take() {
        lifecycle_ingress.disconnect(epoch);
    }
    let epoch = lifecycle_session.begin();
    lifecycle_ingress.begin_session(epoch, world.player_id);
    world_spawn.world_modes.reward_notices.clear();
    reset_world_map_shell(
        &mut world_spawn.world_modes.world_map,
        &mut world_spawn.world_modes.world_map_production,
        &mut world_spawn.world_modes.world_map_clock,
    );
    reset_transportation_shell(
        &mut world_spawn.world_modes.transportation_model,
        &mut world_spawn.world_modes.transportation_outbox,
        &mut world_spawn.world_modes.transportation_production,
    );
    reset_race_shell(
        &mut world_spawn.world_modes.race_mode,
        &mut world_spawn.world_modes.race_mode_commands,
        &mut world_spawn.world_modes.race_reward,
        &mut world_spawn.world_modes.race_rank,
        &mut world_spawn.world_modes.race_rank_commands,
        &mut world_spawn.world_modes.race_production,
        &mut world_spawn.world_modes.race_frames,
    );
    reset_combi_shell_0104(
        commands,
        &mut world_spawn.world_modes.combi_runtime,
        &mut world_spawn.world_modes.combi_state,
        &mut world_spawn.world_modes.combi_projection,
        &mut world_spawn.world_modes.combi_outbox,
        &mut world_spawn.world_modes.combi_shell,
        &mut world_spawn.world_modes.combi_frames,
        &mut world_spawn.npc_modes.system_messages,
    );
    reset_enchant_shell_0104(
        &mut world_spawn.world_modes.enchant_runtime,
        &mut world_spawn.world_modes.enchant_projection,
        &mut world_spawn.world_modes.enchant_outbox,
        &mut world_spawn.world_modes.enchant_shell,
        &mut world_spawn.world_modes.enchant_frames,
        &mut world_spawn.npc_modes.system_messages,
    );
    world_spawn
        .world_modes
        .world_map_clock
        .observe(world.server_time);
    let style = world.load.style();
    let first_name = style.first_name.to_string_lossy();
    let last_name = style.last_name.to_string_lossy();
    runtime.player_name = if style.name_check != 0 {
        format!("{first_name} {last_name}").trim().to_owned()
    } else {
        format!("Player {}", world.pc_uid)
    };
    world_spawn
        .social
        .group_runtime
        .begin_session(world.player_id, world.pc_uid);
    world_spawn.social.group_integration.reset();
    *world_spawn.social.group_ui = GroupUiModel {
        local_pc_uid: Some(world.pc_uid),
        ..default()
    };
    world_spawn.social.pc2pc_offers.reset();
    world_spawn.social.pc2pc_ui.reset();
    world_spawn
        .social
        .pc2pc_prompt
        .reset(&mut world_spawn.npc_modes.system_messages);
    *world_spawn.social.buddy_ui = BuddyUiModel::default();
    world_spawn.social.buddy_runtime.reset();
    world_spawn.social.nanocom_messages.clear();
    world_spawn.social.nanocom_outbox.clear();
    reset_nano_free_tuning_shell(
        commands,
        &mut world_spawn.inventory.nano_free_tuning_model,
        &mut world_spawn.inventory.nano_free_tuning_outbox,
        &mut world_spawn.inventory.nano_free_tuning_bank,
        &mut world_spawn.inventory.nano_free_tuning_production,
        true,
    );
    *world_spawn.inventory.nano_free_tuning_bank = nano_bank;
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
    *world_spawn.inventory.quick_slot_ui = QuickSlotUiModel::default();
    world_spawn
        .inventory
        .inventory
        .seed(world.player_id, &world.load);
    *world_spawn.inventory.skill_buff_ui = SkillBuffUiModel::default();
    *world_spawn.inventory.movement_buffs = movement_buffs::MovementBuffs::default();
    runtime.nano_recall = default();
    reset_resurrect_shell(
        &mut world_spawn.inventory.resurrect_ui,
        &mut world_spawn.inventory.resurrect_context,
        &mut world_spawn.inventory.resurrect_outbox,
    );
    reset_guide_shell(
        &mut world_spawn.npc_modes.guide_ui,
        &mut world_spawn.npc_modes.guide_outbox,
        &mut world_spawn.npc_modes.guide_audio,
        &mut world_spawn.npc_modes.guide_runtime,
        &mut world_spawn.npc_modes.guide_production,
        false,
    );
    world_spawn.npc_modes.normal_npc_warp.reset();
    reset_bank_shell(
        &mut world_spawn.npc_modes.bank_state,
        &mut world_spawn.npc_modes.bank_modal,
        &mut world_spawn.npc_modes.bank_projection,
        &mut world_spawn.npc_modes.bank_outbox,
        &mut world_spawn.npc_modes.bank_production,
        &mut world_spawn.npc_modes.bank_system_runtime,
    );
    reset_vendor_shell(
        &mut world_spawn.npc_modes.vendor_state,
        &mut world_spawn.npc_modes.vendor_modal,
        &mut world_spawn.npc_modes.vendor_projection,
        &mut world_spawn.npc_modes.vendor_outbox,
        &mut world_spawn.npc_modes.vendor_production,
        &mut world_spawn.npc_modes.vendor_system_runtime,
        false,
    );
    reset_upsell_shell(
        &mut world_spawn.npc_modes.upsell_ui,
        &mut world_spawn.npc_modes.upsell_outbox,
        &mut world_spawn.npc_modes.upsell_audio,
    );
    world_spawn
        .npc_modes
        .guide_runtime
        .load_pc_state(&world.load);
    reset_quit_menu_shell(
        &mut world_spawn.npc_modes.quit_menu,
        &mut world_spawn.npc_modes.quit_outbox,
        &mut world_spawn.npc_modes.quit_runtime,
    );
    world_spawn
        .inventory
        .skill_buff_ui
        .seed_from_pc_load(world.player_id, &world.load);
    world_spawn.npc_modes.system_messages.clear();
    *world_spawn.world_modes.mission_ui = MissionUiModel::default();
    if let Err(error) = world_spawn
        .world_modes
        .world_mission
        .seed(&world.load, &world_spawn.world_modes.tutorial_content)
    {
        runtime.message = format!("World mission PC load rejected: {error}");
        world_spawn.world_modes.world_mission.clear();
    }
    world_spawn.world_modes.mission_ui.enabled = true;
    runtime.player_level = world.load.level().max(0) as u16;
    runtime.user_level = world.load.user_level();
    runtime.player_id = Some(world.player_id);
    runtime.player_gender = Some(i32::from(style.gender));
    runtime.map_number = Some(world.map_number);
    let native_position = ProtocolPosition::new(world.position).to_native();
    runtime.map_name = legacy_world_location_name(-native_position.x, native_position.z)
        .map(str::to_owned)
        .or_else(|| {
            world_catalog
                .select_in_scope(world_scope, native_position)
                .map(|scene| scene.name.clone())
        })
        .unwrap_or_else(|| format!("Map {}", world.map_number));
    runtime.fusion_matter = world.load.fusion_matter();
    runtime.max_fusion_matter =
        legacy_avatar_max_fusion_matter(runtime.player_level, runtime.fusion_matter);
    runtime.candy = world.load.candy();
    runtime.transportation_unlocks = ffone_client::transportation_ui::TransportationUnlocks {
        warp_location_flags: world.load.warp_location_flag(),
        wyvern_location_flags: world.load.wyvern_location_flags(),
    };
    runtime.hp = Some(world.hp);
    runtime.max_hp = legacy_avatar_max_hp(runtime.player_level, style.class, world.hp);
    runtime.special_state = world.load.special_state();
    runtime.free_chat = runtime.special_state & 64 == 0;
    runtime.allow_player_interaction = true;
    runtime.tutorial_mouse_right = world_scope == NativeWorldScope::Tutorial;
    runtime.nano_slots = resolve_runtime_nano_slots(
        &world.load,
        world_spawn.inventory.nano_free_tuning_bank.entries(),
    );
    runtime.pending_nano_activation = None;
    runtime.nano_battery = world.load.nano_battery();
    runtime.weapon_battery = world.load.weapon_battery();
    runtime.resurrection_item_slot =
        world_spawn
            .inventory
            .inventory
            .snapshot()
            .and_then(|inventory| {
                resolve_runtime_resurrection_item_slot(
                    inventory,
                    &world_spawn.world_modes.tutorial_content,
                )
            });
    if world_scope == NativeWorldScope::WorldMap {
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
    runtime.diagnostics.bootstrap_packets = world.bootstrap.packets.len();
    runtime.diagnostics.bootstrap_decode_errors = world.bootstrap.decode_errors().count();
    runtime.message = "OpenFusion world ready; native movement enabled".to_owned();
    for packet in world.bootstrap.packets {
        match packet {
            WorldBootstrapPacket::InitialEntities { entities, .. } => {
                lifecycle_ingress.push_bootstrap(epoch, entities);
            }
            WorldBootstrapPacket::MalformedInitialEntities { frame, .. } => {
                lifecycle_ingress.push_frame(epoch, frame);
            }
            WorldBootstrapPacket::Passthrough(frame) => {
                match runtime.nano_recall.apply(&frame) {
                    Ok(Some(message)) =>
                        nano_recall::notice(&mut world_spawn.social.nanocom_messages, message),
                    Ok(None) => {}
                    Err(error) => {
                        runtime.message = format!("Recall bootstrap packet rejected: {error}");
                        continue;
                    }
                }
                if world_spawn.social.email_inbox.push(frame.clone()) {
                    continue;
                }
                if world_spawn
                    .world_modes
                    .combi_frames
                    .push_if_owned(&world_spawn.world_modes.combi_runtime, frame.clone())
                {
                    continue;
                }
                if world_spawn
                    .world_modes
                    .enchant_frames
                    .push_if_owned(&world_spawn.world_modes.enchant_runtime, frame.clone())
                {
                    continue;
                }
                if world_spawn
                    .world_modes
                    .race_frames
                    .push_if_owned(frame.clone())
                {
                    continue;
                }

                match apply_world_mission_network_frame(
                    &frame,
                    &bridge,
                    &world_spawn.world_modes.tutorial_content,
                    &world_spawn.queries.mission_npcs,
                    &mut world_spawn.inventory.inventory,
                    Some(&mut world_spawn.world_modes.reward_notices),
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
                    Ok(true) => continue,
                    Ok(false) => {}
                    Err(error) => {
                        runtime.message = format!("World mission ignored {error}");
                        continue;
                    }
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
                    Ok(true) => continue,
                    Ok(false) => {}
                    Err(error) => {
                        runtime.message = error;
                        continue;
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
                    Ok(true) => continue,
                    Ok(false) => {}
                    Err(error) => {
                        runtime.message = error;
                        continue;
                    }
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
                if let Err(error) =
                    apply_quick_slot_frame(&frame, &mut world_spawn.inventory.quick_slot_ui)
                {
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
                lifecycle_ingress.push_frame(epoch, frame);
            }
        }
    }
    for entity in world_slice.iter() {
        commands.entity(entity).despawn();
    }
    match world_scope {
        NativeWorldScope::Tutorial => {
            tutorial_session.character = Some(selected_character);
            runtime.message = "Tutorial sequence entering shard-owned gameplay world".to_owned();
        }
        NativeWorldScope::WorldMap => {
            tutorial_session.clear();
            runtime.message = "OpenFusion world ready; loading authored world collider".to_owned();
        }
    }
    next_state.set(client_state_for_world_scope(world_scope));
}
