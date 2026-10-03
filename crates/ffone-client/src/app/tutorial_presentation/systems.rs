use super::*;

pub(super) fn sync_world_location_notice(
    state: Res<State<ClientState>>,
    model: Res<GameplayUiModel>,
    loading: Res<GameplayLoadingState>,
    mut previous: Local<Option<(u64, String)>>,
    mut notice: ResMut<ffone_client::gameplay_ui::CombatModeNotice>,
    mut random: ResMut<ffone_client::legacy_npc_nano_animation::LegacyNanoStandRandomStream>,
) {
    if *state.get() != ClientState::World || !model.visible {
        *previous = None;
        return;
    }
    // Keep the new location pending until the loading screen has gone away.
    if loading.visible {
        return;
    }
    let name = &model.minimap.map_name;
    if name.is_empty() {
        return;
    }
    if previous
        .as_ref()
        .is_some_and(|(owner, location)| *owner == model.player.owner && location == name)
    {
        return;
    }
    notice.show_location(name, &mut random);
    *previous = Some((model.player.owner, name.clone()));
}

pub(super) fn sync_gameplay_hud(
    time: Res<Time>,
    runtime: Res<RuntimeStatus>,
    option: GameplayHudOptionInputs,
    missions: GameplayHudMissionInputs,
    nano_portrait_catalog: Res<GameplayNanoPortraitCatalog>,
    state: Res<State<ClientState>>,
    gameplay_nano: Res<TutorialNanoGameplayState>,
    world_nano_cooldowns: Res<WorldNanoCooldownRuntime>,
    players: Query<(&Transform, &LegacyPlayerController), With<LocalPlayer>>,
    actor_registry: Res<TutorialActorRegistry>,
    actors: Query<(&TutorialActor, &GlobalTransform)>,
    network_npcs: Query<(&NetworkNpcAppearance0104, &GlobalTransform)>,
    cameras: Query<&LegacyOrbitCamera>,
    mut nano_wheel: ResMut<NanoWheelTransientUi>,
    mut model: ResMut<GameplayUiModel>,
) {
    let mission_runtime = &missions.tutorial;
    let tutorial_content = &missions.content;
    let mut next_nano_wheel = nano_wheel.clone();
    project_nano_skill_cooldown(
        *state.get(),
        &runtime.nano_slots,
        gameplay_nano.loadout(),
        gameplay_nano.skill_cooldown_remaining_fraction(),
        std::array::from_fn(|index| {
            world_nano_cooldowns.remaining_fraction(index, runtime.nano_slots[index].skill_id)
        }),
        &mut next_nano_wheel,
    );
    nano_wheel.set_if_neq(next_nano_wheel);
    let mut next = model.clone();
    next.npc_names_visible = option.runtime.options.display.npc_name;
    next.balloon_chat_visible = option.runtime.options.display.balloon;
    next.ui_scale = option.windows.iter().next().map_or(1.0, |window| {
        option.runtime.effective_ui_scale(window.height())
    });
    next.visible = matches!(state.get(), ClientState::Tutorial | ClientState::World)
        && runtime.player_id.is_some();
    if !next.visible {
        next.current_objective = CurrentObjectiveUi::default();
        if *model != next {
            *model = next;
        }
        return;
    }

    next.player = PlayerStatusUi {
        owner: runtime.player_id.unwrap_or_default() as u64,
        name: runtime.player_name.clone(),
        level: runtime.player_level,
        hp: runtime.hp.unwrap_or_default(),
        max_hp: runtime.max_hp,
        free_chat: runtime.free_chat,
        allow_player_interaction: runtime.allow_player_interaction,
    };
    // Instances lie outside every WorldNameScript rectangle; like the clean
    // WorldDataContainer, keep this avatar's last named location there.
    // `runtime.map_name` is only the fallback before any named rectangle.
    let same_avatar = model.player.owner == next.player.owner;
    let last_location = std::mem::replace(&mut next.minimap.map_name, runtime.map_name.clone());
    next.minimap.fusion_matter = if *state.get() == ClientState::Tutorial {
        mission_runtime.fusion_matter
    } else {
        runtime.fusion_matter
    };
    next.minimap.max_fusion_matter = if *state.get() == ClientState::Tutorial {
        // `cnOwnAvatarStatus.UpdateAttrib` forces the level-one grow row while
        // the tutorial flag is set, irrespective of the selected character.
        legacy_avatar_max_fusion_matter(1, next.minimap.fusion_matter)
    } else {
        runtime.max_fusion_matter
    };
    let elapsed_seconds = time.elapsed_secs_f64() as f32;
    next.minimap.player_marker_alpha =
        minimap_marker_alpha(mission_runtime.my_point_event, elapsed_seconds);
    if let Ok((transform, controller)) = players.single() {
        // ProtocolPosition reflects Unity +X into native Bevy -X. The old
        // minimap samples the original Unity X/Z plane.
        let unity_x = -transform.translation.x;
        let unity_z = transform.translation.z;
        next.minimap.tiles = minimap_tiles(unity_x, unity_z, next.minimap.ratio);
        let last_location = if same_avatar {
            last_location.as_str()
        } else {
            ""
        };
        if let Some(location) = legacy_world_location_name_or_last(last_location, unity_x, unity_z)
        {
            next.minimap.map_name = location.to_owned();
        }
        next.minimap.avatar_heading_degrees = controller.yaw_degrees;
        let player_position = Vec3::new(
            -transform.translation.x,
            transform.translation.y,
            transform.translation.z,
        );
        next.minimap.waypoint = if !option.runtime.options.display.waypoint {
            None
        } else if *state.get() == ClientState::Tutorial {
            mission_runtime
                .waypoint_actor_id
                .and_then(|actor_id| actor_registry.entity(actor_id))
                .and_then(|entity| actors.get(entity).ok())
                .and_then(|(_, target_transform)| {
                    let target = target_transform.translation();
                    minimap_waypoint(
                        player_position,
                        Vec3::new(-target.x, target.y, target.z),
                        next.minimap.ratio,
                        mission_runtime.waypoint_event,
                        elapsed_seconds,
                    )
                })
        } else {
            missions.waypoint.native_target.and_then(|native_target| {
                minimap_waypoint(
                    player_position,
                    native_to_unity_vector(native_target),
                    next.minimap.ratio,
                    false,
                    elapsed_seconds,
                )
            })
        };
        next.minimap.custom_waypoints = missions
            .world_map
            .model
            .preferences
            .waypoints
            .iter()
            .filter_map(|point| {
                minimap_waypoint(
                    player_position,
                    Vec3::new(point.x, player_position.y, point.z),
                    next.minimap.ratio,
                    false,
                    elapsed_seconds,
                )
                .map(|sample| (point.color, sample))
            })
            .collect();
        next.minimap.markers = if *state.get() == ClientState::Tutorial {
            actors
                .iter()
                .filter(|(actor, _)| actor.is_alive())
                .filter_map(|(actor, transform)| {
                    // Quest 15/16 replaces, rather than overlays, the actor's
                    // regular 18/19/20 marker in `RenderMinimap`.
                    let icon = tutorial_minimap_marker_icon(
                        actor.npc_type,
                        tutorial_npc_mission_symbol(actor, &mission_runtime),
                    )?;
                    let target = transform.translation();
                    minimap_marker(
                        player_position,
                        Vec3::new(-target.x, target.y, target.z),
                        next.minimap.ratio,
                        icon,
                    )
                })
                .collect()
        } else {
            let view_mobs = missions.skill_buffs.reveals_mobs();
            let owned_nanos = missions
                .nano_bank
                .entries()
                .iter()
                .filter_map(|nano| (nano.id > 0).then_some(i32::from(nano.id)))
                .collect::<BTreeSet<_>>();
            let guide = missions
                .guide
                .authoritative()
                .map_or(0, |state| i32::from(state.raw_mentor()));
            let quest_inventory = missions
                .inventory
                .quest_inventory
                .as_ref()
                .map(|inventory| inventory.as_slice())
                .unwrap_or(&[]);
            let live = network_npcs.iter().map(|(appearance, transform)| (
                appearance.0.npc_id, appearance.0.npc_type, native_to_unity_vector(transform.translation())
            )).collect::<Vec<_>>();
            let live_ids: BTreeSet<_> = live.iter().map(|row|row.0).collect();
            let known = missions.world_map_production.server_npcs.entries.as_ref().into_iter()
                .flat_map(|entries|entries.values()).filter(|entry|!live_ids.contains(&entry.id))
                .map(|entry|(entry.id, entry.npc_type, Vec3::new(entry.position[0] as f32/100., entry.position[2] as f32/100., entry.position[1] as f32/100.)));
            live.into_iter().chain(known)
                .filter_map(|(_, npc_type, target)| {
                    let delta=Vec2::new(target.x-player_position.x,target.z-player_position.z);
                    if delta.length_squared()>=(next.minimap.ratio*16.).powi(2) {return None;}
                    let definition =
                        tutorial_content.gameplay_npc_minimap(npc_type)?;
                    let (new_available, advance_available) =
                        missions.world.npc_has_available_or_completable_mission(
                            npc_type,
                            i32::from(runtime.player_level),
                            guide,
                            &owned_nanos,
                            quest_inventory,
                            tutorial_content,
                        );
                    let style = normal_world_minimap_marker_style(
                        definition,
                        view_mobs,
                        advance_available,
                        new_available,
                    )?;
                    minimap_marker_sized(
                        player_position,
                        target,
                        next.minimap.ratio,
                        style.icon,
                        style.source_dimensions,
                    )
                })
                .collect()
        };
        if *state.get() == ClientState::World
            && missions.skill_buffs.reveals_shinies()
        {
            next.minimap.markers.extend(missions.shinies.iter().filter_map(|transform| {
                minimap_marker(player_position, native_to_unity_vector(transform.translation()),
                    next.minimap.ratio, MinimapMarkerIcon::Shiny)
            }));
        }
    } else {
        next.minimap.tiles.clear();
        next.minimap.avatar_heading_degrees = 0.0;
        next.minimap.waypoint = None;
        next.minimap.custom_waypoints.clear();
        next.minimap.markers.clear();
    }
    next.minimap.camera_heading_degrees = cameras
        .iter()
        .next()
        .map_or(0.0, |camera| camera.yaw_degrees);
    next.current_objective = if *state.get() == ClientState::Tutorial {
        tutorial_current_objective_ui(
            &mission_runtime,
            &tutorial_content,
            option.runtime.options.display.current_objective,
        )
    } else if option.runtime.options.display.current_objective {
        missions
            .inventory
            .quest_inventory
            .as_ref()
            .and_then(|quest_inventory| {
                missions
                    .world
                    .current_objective(quest_inventory, tutorial_content)
                    .ok()
                    .flatten()
            })
            .map_or_else(CurrentObjectiveUi::default, |objective| {
                CurrentObjectiveUi {
                    visible: true,
                    task_id: Some(objective.task_id),
                    title: objective.title,
                    body: objective.objective,
                    remaining_time_seconds: objective.remaining_time_seconds,
                    enemies: objective
                        .enemies
                        .into_iter()
                        .map(|row| CurrentObjectiveProgressUi {
                            content_id: row.content_id,
                            name: row.name,
                            complete: row.complete,
                            needed: row.needed,
                        })
                        .collect(),
                    quest_items: objective
                        .quest_items
                        .into_iter()
                        .map(|row| CurrentObjectiveProgressUi {
                            content_id: row.content_id,
                            name: row.name,
                            complete: row.complete,
                            needed: row.needed,
                        })
                        .collect(),
                }
            })
    } else {
        CurrentObjectiveUi::default()
    };

    next.chat.visible = true;
    next.chat.text_colors = option.runtime.options.text_colors;
    let world_chat = *state.get() == ClientState::World;
    next.chat.group_available = world_chat && runtime.chat.world_group_available;
    next.chat.alerts = if world_chat {
        runtime.chat.world_chat_alerts
    } else {
        [false; 3]
    };
    if !next.chat.group_available {
        next.chat.alerts[ChatChannel::Group.index()] = false;
    }
    if !next.chat.group_available && next.chat.selected == ChatChannel::Group {
        next.chat.selected = ChatChannel::All;
    }
    // The shard parses slash commands before enforcing FreeChat mute. Keep
    // the input available in-world; SendChat still drops non-command text
    // while `runtime.free_chat` is false.
    next.chat.input_enabled = world_chat;
    if world_chat {
        next.chat.lines.clone_from(match next.chat.selected {
            ChatChannel::All => &runtime.chat.world_chat_lines,
            ChatChannel::Group => &runtime.chat.world_group_chat_lines,
            ChatChannel::Buddy => &runtime.chat.world_buddy_chat_lines,
        });
    } else {
        next.chat.lines.clone_from(&mission_runtime.chat_lines);
    }
    if !next.chat.input_enabled {
        next.chat.active = false;
        next.chat.input.clear();
    }
    next.nanos = runtime.nano_slots.map(|slot| {
        let nano = slot
            .nano_id
            .and_then(|nano_id| tutorial_content.gameplay_nano(nano_id));
        let skill = slot
            .nano_id
            .and_then(|_| tutorial_content.gameplay_skill(slot.skill_id));
        NanoSlotUi {
            nano_id: slot.nano_id,
            name: nano.map_or_else(String::new, |definition| {
                missions.localization.text(
                    &missions.language,
                    &LocalizedText::new(
                        format!("content.nano.{}.name", definition.nano_id),
                        definition.name.clone(),
                    ),
                )
            }),
            skill_id: slot.nano_id.map(|_| slot.skill_id),
            style: nano.map(|definition| definition.style),
            model_path: slot
                .nano_id
                .and_then(|nano_id| nano_portrait_catalog.model_path(nano_id).map(str::to_owned)),
            nano_icon_number: nano.map(|definition| definition.icon_number),
            skill_icon_number: skill.map(|definition| definition.icon_number),
            active_skill: skill.is_some_and(|definition| definition.active),
            stamina_fraction: nano.map_or(1.0, |definition| {
                (f32::from(slot.stamina) / f32::from(definition.max_stamina)).clamp(0.0, 1.0)
            }),
            active: slot.active,
        }
    });
    next.nano_battery = runtime.nano_battery;
    next.weapon_battery = runtime.weapon_battery;

    if *model != next {
        *model = next;
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn sync_tutorial_overlay(
    runtime: Res<RuntimeStatus>,
    option: GameplayHudOptionInputs,
    tutorial: Res<TutorialSession>,
    mission_runtime: Res<TutorialMissionRuntime>,
    auxiliary_presentation: Res<TutorialAuxiliaryPresentation>,
    tutorial_content: Res<TutorialMissionContent>,
    state: Res<State<ClientState>>,
    localization: Res<Localization>,
    language: Res<Language>,
    mut model: ResMut<TutorialOverlayUiModel>,
) {
    let mut next = model.clone();
    next.ui_scale = option.windows.iter().next().map_or(1.0, |window| {
        option.runtime.effective_ui_scale(window.height())
    });
    next.visible = *state.get() == ClientState::Tutorial
        && runtime.player_id.is_some()
        && !tutorial.completion_requested;
    if !next.visible {
        // Fail closed on lifecycle, not on content. This prevents an old
        // auxiliary subtitle (notably BasicArrowKey's W hint) from reviving
        // after the tutorial has already switched to World/character select.
        next.tutorial = TutorialUi::default();
        if *model != next {
            *model = next;
        }
        return;
    }

    let viewport = option
        .windows
        .iter()
        .next()
        .map(|window| {
            (
                window.width().round() as i32,
                window.height().round() as i32,
            )
        })
        .unwrap_or((1280, 720));
    let tutorial_stage = tutorial.progress.stage();
    let mut tutorial_ui = tutorial_stage.map_or_else(TutorialUi::default, |stage| {
        tutorial_ui_for_stage(
            stage,
            tutorial.step_elapsed,
            viewport.0,
            viewport.1,
            &localization,
            &language,
        )
    });
    if tutorial_auxiliary_presentation_matches(
        &auxiliary_presentation,
        &mission_runtime,
        tutorial_stage,
    ) {
        if auxiliary_presentation.primary_subtitle_touched {
            tutorial_ui.instruction = auxiliary_presentation.primary_subtitle.and_then(|text| {
                resolve_tutorial_auxiliary_text(text, &tutorial_content, &localization, &language)
            });
        }
        if auxiliary_presentation.secondary_subtitle_touched {
            tutorial_ui.secondary_instruction =
                auxiliary_presentation.secondary_subtitle.and_then(|text| {
                    resolve_tutorial_auxiliary_text(
                        text,
                        &tutorial_content,
                        &localization,
                        &language,
                    )
                });
        }
        if auxiliary_presentation.picture_touched {
            tutorial_ui.illustration =
                auxiliary_presentation
                    .picture
                    .and_then(|(resource, position, pivot)| {
                        tutorial_illustration_cue(resource, position, pivot, viewport.0, viewport.1)
                    });
        }
        if auxiliary_presentation.cursor_touched {
            tutorial_ui.arrow =
                auxiliary_presentation
                    .cursor
                    .and_then(|(resource, position, pivot)| {
                        tutorial_arrow_cue(resource, position, pivot, viewport.0, viewport.1)
                    });
        }
    }
    if tutorial_stage == Some(TutorialStage::Movement(MovementStage::MoveForward)) {
        super::operations::bind_forward_key(&mut tutorial_ui.instruction, &option.runtime.input);
        super::operations::bind_forward_key(
            &mut tutorial_ui.secondary_instruction,
            &option.runtime.input,
        );
        if super::operations::forward_key_binding(&option.runtime.input)
            != Some(LegacyPhysicalKey::W)
        {
            // The recovered movement illustration draws W into the bitmap.
            tutorial_ui.illustration = None;
        }
    }
    next.tutorial = tutorial_ui;
    if *model != next {
        *model = next;
    }
}
