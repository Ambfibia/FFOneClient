use super::gm_extended::{self, Action, Error};
use crate::app::gameplay_ui_actions::*;

pub(crate) use super::gm_state::GmRuntime;

pub(super) fn feedback(
    runtime: &mut RuntimeStatus,
    localization: &Localization,
    language: &Language,
    text: LocalizedText,
) {
    let line = localization.text(language, &text);
    runtime.message.clone_from(&line);
    push_world_chat_line(runtime, ChatLineUi::normal(line));
}

pub(super) fn handle(message: &str, c: &mut WorldGameplayActionContext<'_, '_>) -> bool {
    let Some(name) = message.split_whitespace().next() else {
        return false;
    };
    let item = matches!(name, "/item" | "/itemN" | "/itemQ");
    if !item && !gm_extended::NAMES.contains(&name) {
        return false;
    }
    let owner = c.runtime.player_id;
    if c.runtime.chat.gm.owner != owner {
        c.runtime.chat.gm = GmRuntime {
            owner,
            ..Default::default()
        };
    }
    let result: Result<LocalizedText, Error> = (|| {
        if item {
            // Validate syntax/access before accepting queue ownership. Slot selection
            // happens again at dispatch, after the preceding authoritative reply.
            gm_commands::request(
                message,
                c.runtime.user_level,
                c.runtime.player_id,
                |_, _| Some(0),
            )
            .map_err(|e| match e {
                gm_commands::Error::Denied => Error::Denied(50),
                gm_commands::Error::MissingPlayer => Error::PlayerMissing,
                _ => Error::Usage(gm_commands::usage(name).unwrap().to_owned()),
            })?;
            let owner = c.runtime.player_id;
            let gm = &mut c.runtime.chat.gm;
            if gm.owner != owner {
                *gm = GmRuntime {
                    owner,
                    ..Default::default()
                };
            }
            if gm.items.len() >= 32 || gm.stalled {
                return Ok(LocalizedText::new(
                    "ui.chat.command.gm.queue_blocked",
                    "The item queue is waiting for the server. Reconnect if the request timed out.",
                ));
            }
            gm.items.push_back(message.to_owned());
            return Ok(LocalizedText::new(
                "ui.chat.command.gm.queued",
                "Item request queued.",
            ));
        }
        let position = c
            .modal_owners
            .shell
            .players
            .single()
            .ok()
            .map(|(t, _, _)| ProtocolPosition::from_native(t.translation).raw())
            .unwrap_or([0; 3]);
        let position = if name == "/summonshiny" {
            c.runtime
                .chat
                .gm
                .shiny_jitter(position)
                .ok_or_else(|| Error::Usage("/summonshiny <type>".to_owned()))?
        } else {
            position
        };
        let selected = c
            .modal_owners
            .shell
            .avatar_actions
            .single()
            .ok()
            .and_then(|a| a.target_selection.focused_npc)
            .and_then(|t| c.npc_appearances.get(t.entity).ok())
            .filter(|(_, npc)| npc.0.hp > 0)
            .map(|(_, npc)| npc.0.npc_id);
        match gm_extended::parse(
            message,
            c.runtime.user_level,
            c.runtime.player_id,
            position,
            selected,
        )? {
            Action::Packet(request) => {
                c.bridge
                    .send(NetworkCommand::SendRegisteredGameplay0104(request))
                    .map_err(|_| Error::PlayerMissing)?;
                Ok(
                    LocalizedText::new("ui.chat.command.gm.requested", "Command sent: {command}")
                        .with_arg("command", message),
                )
            }
            Action::Local { name, value } => local(&name, value, c),
        }
    })();
    let text = match result {
        Ok(text) => text,
        Err(Error::Usage(usage)) => {
            LocalizedText::new("ui.chat.command.gm.usage", "Usage: {usage}")
                .with_arg("usage", usage)
        }
        Err(Error::Denied(level)) => LocalizedText::new(
            "ui.chat.command.gm.access",
            "This command requires accountLevel <= {level}.",
        )
        .with_arg("level", level.to_string()),
        Err(Error::PlayerMissing) => LocalizedText::new(
            "ui.chat.command.gm.player_missing",
            "The local player is not ready.",
        ),
        Err(Error::TargetMissing) => LocalizedText::new(
            "ui.chat.command.gm.target_missing",
            "Select a living NPC first.",
        ),
    };
    feedback(
        &mut c.runtime,
        &c.modal_owners.shell.localization,
        &c.modal_owners.shell.language,
        text,
    );
    true
}

fn local(
    name: &str,
    value: Option<i32>,
    c: &mut WorldGameplayActionContext<'_, '_>,
) -> Result<LocalizedText, Error> {
    match name {
        "/rule" => {
            c.modal_owners
                .modes
                .rule_runtime
                .open(
                    &mut c.modal_owners.modes.rule_model,
                    ffone_client::rule_runtime::RuleOpenRequest::GmChatCommand {
                        user_level: i32::from(c.runtime.user_level),
                        table_index: value.unwrap(),
                    },
                )
                .map_err(|_| Error::Usage("/rule <page-index>".to_owned()))?;
        }
        "/equipitem" => {
            let slot = usize::try_from(value.unwrap())
                .map_err(|_| Error::Usage("/equipitem <inventory-slot>".to_owned()))?;
            let item = c
                .normal_warp
                .inventory
                .snapshot()
                .and_then(|s| s.inventory().get(slot))
                .filter(|i| i.item_id > 0 && (0..=6).contains(&i.item_type))
                .ok_or_else(|| {
                    Error::Usage("/equipitem <occupied equipment-item inventory-slot>".to_owned())
                })?;
            let packet = ffone_protocol::RegisteredGameplayRequest0104::new(
                packet::P_CL2FE_REQ_ITEM_MOVE,
                ffone_protocol::ItemMoveRequest0104 {
                    from_location: 1,
                    from_slot_num: slot as i32,
                    to_location: 0,
                    to_slot_num: i32::from(item.item_type),
                }
                .encode(),
            )
            .unwrap();
            c.bridge
                .send(NetworkCommand::SendRegisteredGameplay0104(packet))
                .map_err(|_| Error::PlayerMissing)?;
        }
        "/nanoArr" => {
            let max = value.unwrap();
            if !(1..=36).contains(&max) || max <= i32::from(c.runtime.player_level) {
                return Err(Error::Usage(
                    "/nanoArr <level greater than current, up to 36>".to_owned(),
                ));
            }
            // Server-owned level and Nano changes; each normal reply uses the
            // same acquisition and tuning consumers as individual commands.
            c.bridge
                .send(NetworkCommand::SendFreeChat(FreeChatRequest0104 {
                    message: FixedUtf16::from_str(&format!("/level {max}")).unwrap(),
                    emote_code: 0,
                }))
                .map_err(|_| Error::PlayerMissing)?;
            for id in (c.runtime.player_level as i16 + 1)..=max as i16 {
                if c.content.gameplay_nano(id).is_some() {
                    let request = ffone_protocol::RegisteredGameplayRequest0104::new(
                        packet::P_CL2FE_REQ_PC_GIVE_NANO,
                        ffone_protocol::wire_0104::PcGiveNanoRequest0104 { nano_id: id }.encode(),
                    )
                    .unwrap();
                    c.bridge
                        .send(NetworkCommand::SendRegisteredGameplay0104(request))
                        .map_err(|_| Error::PlayerMissing)?;
                    if let Some(skill) = c
                        .content
                        .journal_nano(i32::from(id))
                        .and_then(|n| n.skills.first())
                    {
                        let request = ffone_protocol::RegisteredGameplayRequest0104::new(
                            0x13000048,
                            ffone_protocol::wire_0104::PcGiveNanoSkillRequest0104 {
                                nano_id: id,
                                nano_skill_id: skill.skill_id as i16,
                            }
                            .encode(),
                        )
                        .unwrap();
                        c.bridge
                            .send(NetworkCommand::SendRegisteredGameplay0104(request))
                            .map_err(|_| Error::PlayerMissing)?;
                    }
                }
            }
        }
        "/qinven" => {
            let rows = c
                .normal_warp
                .inventory
                .quest_inventory
                .as_ref()
                .ok_or(Error::PlayerMissing)?
                .iter()
                .enumerate()
                .filter(|(_, i)| i.item_id > 0)
                .map(|(slot, i)| format!("{slot}: {} × {}", i.item_id, i.option))
                .collect::<Vec<_>>()
                .join("; ");
            return Ok(LocalizedText::new(
                "ui.chat.command.gm.quest_inventory",
                "Quest inventory: {items}",
            )
            .with_arg("items", rows));
        }
        "/tasklog" => {
            return Ok(
                LocalizedText::new("ui.chat.command.gm.tasks", "Active task IDs: {tasks}")
                    .with_arg(
                        "tasks",
                        c.modal_owners
                            .mission
                            .world_mission
                            .active_task_ids()
                            .iter()
                            .map(i32::to_string)
                            .collect::<Vec<_>>()
                            .join(", "),
                    ),
            );
        }
        "/viewloc" | "/viweloc" => c.runtime.chat.gm.view_location ^= true,
        "/viewnetinfo" => {
            c.runtime.chat.gm.view_network = value.is_some() || !c.runtime.chat.gm.view_network;
            if value.is_some() {
                c.runtime.chat.gm.record_history ^= true;
            }
        }
        "/viewid" => c.runtime.chat.gm.view_ids ^= true,
        "/viewcol" => c.runtime.chat.gm.view_collision ^= true,
        "/hideui" => c.runtime.chat.gm.hide_ui ^= true,
        "/Store" => {
            c.runtime.chat.gm.store_open = Some(value.unwrap_or_default());
        }
        _ => return Err(Error::Usage(name.to_owned())),
    }
    Ok(
        LocalizedText::new("ui.chat.command.gm.requested", "Command sent: {command}")
            .with_arg("command", name),
    )
}

pub(crate) fn pump(
    mut runtime: ResMut<RuntimeStatus>,
    inventory: Res<LocalInventoryRuntime>,
    bridge: Res<NetworkBridge>,
    time: Res<Time>,
    localization: Res<Localization>,
    language: Res<Language>,
) {
    let owner = runtime.player_id;
    if runtime.chat.gm.owner.is_some() && runtime.chat.gm.owner != owner {
        runtime.chat.gm = GmRuntime::default();
    }
    if let Some((key, fallback)) = runtime.chat.gm.notice.take() {
        feedback(
            &mut runtime,
            &localization,
            &language,
            LocalizedText::new(key, fallback),
        );
    }
    if runtime.chat.gm.pending.is_some() {
        runtime.chat.gm.age += time.delta_secs();
        if runtime.chat.gm.age > 15.0 && !runtime.chat.gm.stalled {
            runtime.chat.gm.stalled = true;
            feedback(
                &mut runtime,
                &localization,
                &language,
                LocalizedText::new(
                    "ui.chat.command.gm.queue_blocked",
                    "The item queue is waiting for the server. Reconnect if the request timed out.",
                ),
            );
        }
        return;
    }
    let Some(message) = runtime.chat.gm.next_item() else {
        return;
    };
    let request = gm_commands::request(&message, runtime.user_level, owner, |quest, id| {
        if quest {
            let slots = inventory.quest_inventory.as_ref()?;
            slots
                .iter()
                .position(|i| i.item_id == id)
                .or_else(|| slots.iter().position(|i| i.option == 0))
        } else {
            inventory
                .snapshot()?
                .inventory()
                .iter()
                .position(|i| i.item_id <= 0)
        }
    });
    match request {
        Ok(request) => {
            let pending =
                ffone_protocol::wire_0104::PcGiveItemRequest0104::decode(request.payload())
                    .unwrap();
            match bridge.send(NetworkCommand::SendRegisteredGameplay0104(request)) {
                Ok(()) => {
                    runtime.chat.gm.start_item(pending);
                }
                Err(error) => feedback(
                    &mut runtime,
                    &localization,
                    &language,
                    LocalizedText::new(
                        "ui.chat.command.gm.failed",
                        "Could not send command: {error}",
                    )
                    .with_arg("error", error),
                ),
            }
        }
        Err(error) => feedback(
            &mut runtime,
            &localization,
            &language,
            match error {
                gm_commands::Error::Full => LocalizedText::new(
                    "ui.chat.command.gm.inventory_full",
                    "No free inventory slot is available.",
                ),
                gm_commands::Error::Denied => LocalizedText::new(
                    "ui.chat.command.gm.denied",
                    "This command requires GM access (accountLevel <= 50).",
                ),
                gm_commands::Error::MissingPlayer => LocalizedText::new(
                    "ui.chat.command.gm.player_missing",
                    "The local player is not ready.",
                ),
                gm_commands::Error::Usage => {
                    LocalizedText::new("ui.chat.command.gm.usage", "Usage: {usage}").with_arg(
                        "usage",
                        gm_commands::usage(message.split_whitespace().next().unwrap_or_default())
                            .unwrap_or("/item <type> <id> <count>"),
                    )
                }
            },
        ),
    }
}

pub(crate) fn item_reply(frame: &DecodedFrame, runtime: &mut RuntimeStatus) {
    runtime.chat.gm.accept_item_reply(frame);
}

pub(crate) fn reply(
    frame: &DecodedFrame,
    runtime: &mut RuntimeStatus,
    localization: &Localization,
    language: &Language,
) -> bool {
    use ffone_protocol::wire_0104::*;
    let result: Result<LocalizedText, String> = (|| {
        Ok(match frame.packet_type {
            0x310000c7 => {
                let p = GmPcLocationReply0104::decode(&frame.payload).map_err(|e| e.to_string())?;
                LocalizedText::new(
                    "ui.chat.command.gm.location",
                    "{player}: PC {id}, UID {uid}, map {map}, position {position}",
                )
                .with_arg(
                    "player",
                    format!(
                        "{} {}",
                        p.target_pc_first_name.to_string_lossy(),
                        p.target_pc_last_name.to_string_lossy()
                    ),
                )
                .with_arg("id", p.target_pc_id.to_string())
                .with_arg("uid", p.target_pc_uid.to_string())
                .with_arg("map", p.map_num.to_string())
                .with_arg("position", format!("{}, {}, {}", p.x, p.y, p.z))
            }
            0x310000c8 => {
                let p = GmPcAnnounceReply0104::decode(&frame.payload).map_err(|e| e.to_string())?;
                runtime.chat.gm.announcement = Some((
                    p.announce_msg.to_string_lossy(),
                    p.during_time.clamp(1, 3600) as f32,
                ));
                LocalizedText::new("ui.chat.command.gm.announcement", "Announcement: {message}")
                    .with_arg("message", p.announce_msg.to_string_lossy())
            }
            0x3100012c => {
                let p =
                    GmRewardRateSuccess0104::decode(&frame.payload).map_err(|e| e.to_string())?;
                LocalizedText::new(
                    "ui.chat.command.gm.rates",
                    "Taros: {taros}; Fusion Matter: {fm}",
                )
                .with_arg("taros", format!("{:?}", p.af_reward_rate_taros))
                .with_arg("fm", format!("{:?}", p.af_reward_rate_fusion_matter))
            }
            0x310000de => {
                let p = PcMissionCompleteSuccess0104::decode(&frame.payload)
                    .map_err(|e| e.to_string())?;
                LocalizedText::new(
                    "ui.chat.command.gm.mission_completed",
                    "Mission completed: {id}",
                )
                .with_arg("id", p.mission_num.to_string())
            }
            0x310000fb => {
                let p = PcChannelNumReply0104::decode(&frame.payload).map_err(|e| e.to_string())?;
                LocalizedText::new("ui.chat.command.gm.channel", "Current channel: {channel}")
                    .with_arg("channel", p.channel_num.to_string())
            }
            0x310000fc => {
                PcWarpChannelFailure0104::decode(&frame.payload).map_err(|e| e.to_string())?;
                LocalizedText::new(
                    "ui.chat.command.gm.channel_unavailable",
                    "The requested channel or shard is unavailable.",
                )
            }
            0x310000fd => {
                PcWarpChannelSuccess0104::decode(&frame.payload).map_err(|e| e.to_string())?;
                LocalizedText::new(
                    "ui.chat.command.gm.channel_current",
                    "You are already connected to this channel or shard.",
                )
            }
            0x310000fa => {
                let header =
                    ChannelInfoReply0104::decode(frame.payload.get(..8).ok_or("channel header")?)
                        .map_err(|e| e.to_string())?;
                if header.channel_cnt < 0
                    || frame.payload.len() != 8 + header.channel_cnt as usize * 8
                {
                    return Err("invalid channel count".to_owned());
                }
                let mut rows = Vec::new();
                for bytes in frame.payload[8..].chunks_exact(8) {
                    let p = ChannelInfo0104::decode(bytes).map_err(|e| e.to_string())?;
                    rows.push(format!("{}: {}", p.channel_num, p.current_user_cnt));
                }
                LocalizedText::new(
                    "ui.chat.command.gm.channels",
                    "Channels (number: players): {channels}",
                )
                .with_arg("channels", rows.join("; "))
            }
            _ => return Err(String::new()),
        })
    })();
    match result {
        Ok(text) => {
            feedback(runtime, localization, language, text);
            true
        }
        Err(error) if !error.is_empty() => {
            runtime.message = error;
            true
        }
        _ => false,
    }
}

#[derive(Component)]
pub(crate) struct GmDebugOverlay;

/// Opt-in diagnostics retain the normal gameplay model and input ownership.
/// The HUD can therefore be restored by entering /hideui a second time.
pub(crate) fn presentation(
    mut commands: Commands,
    mut runtime: ResMut<RuntimeStatus>,
    time: Res<Time>,
    assets: Res<AssetServer>,
    localization: Res<Localization>,
    language: Res<Language>,
    mut refresh: Local<f32>,
    mut overlays: Query<(Entity, &mut LocalizedText), With<GmDebugOverlay>>,
    players: Query<
        (&Transform, &ffone_client::movement::LegacyPlayerController),
        With<LocalPlayer>,
    >,
    npcs: Query<&NetworkNpcAppearance0104>,
    colliders: Query<&ffone_client::world::AuthoredColliderWorldBounds>,
    mut gizmos: Gizmos,
) {
    let active = runtime.player_id.is_some() && runtime.user_level <= 50;
    if let Some((_, remaining)) = &mut runtime.chat.gm.announcement {
        *remaining -= time.delta_secs();
        if *remaining <= 0.0 {
            runtime.chat.gm.announcement = None;
        }
    }
    let gm = &runtime.chat.gm;
    let debug = active && (gm.view_location || gm.view_network || gm.view_ids || gm.view_collision);
    if !debug && gm.announcement.is_none() {
        for (entity, _) in &overlays {
            commands.entity(entity).despawn();
        }
        return;
    }
    let position = players
        .single()
        .ok()
        .map(|(t, _)| t.translation)
        .unwrap_or_default();
    if active && gm.view_collision {
        for collider in &colliders {
            let (min, max) = collider.debug_bounds();
            let center = (min + max) * 0.5;
            if center.distance_squared(position) < 100.0 * 100.0 {
                gizmos.cube(
                    Transform::from_translation(center).with_scale(max - min),
                    Color::srgb(0.2, 1.0, 0.2),
                );
            }
        }
    }
    *refresh -= time.delta_secs();
    if *refresh > 0.0 && !overlays.is_empty() {
        return;
    }
    *refresh = 0.2;
    let mut lines = Vec::new();
    if let Some((message, _)) = &gm.announcement {
        lines.push(message.clone());
    }
    if active && gm.view_location {
        let p = ProtocolPosition::from_native(position).raw();
        if let Ok((_, controller)) = players.single() {
            let angle =
                ffone_client::coordinates::LegacyUnityHeadingDegrees::new(controller.yaw_degrees)
                    .to_protocol()
                    .degrees();
            lines.push(
                localization.text(
                    &language,
                    &LocalizedText::new(
                        "ui.chat.command.gm.view_location",
                        "map={map} XYZ={x},{y},{z} iAngle={angle}° (player)",
                    )
                    .with_arg("map", runtime.map_number.unwrap_or_default().to_string())
                    .with_arg("x", p[0].to_string())
                    .with_arg("y", p[1].to_string())
                    .with_arg("z", p[2].to_string())
                    .with_arg("angle", angle.to_string()),
                ),
            );
        }
    }
    if active && gm.view_network {
        lines.push(format!(
            "PC={} RX={} bootstrap={} errors={}",
            runtime.player_id.unwrap_or_default(),
            runtime.diagnostics.gameplay_frames,
            runtime.diagnostics.bootstrap_packets,
            runtime.diagnostics.bootstrap_decode_errors
        ));
    }
    if active && gm.view_network && gm.record_history {
        lines.push(
            gm.history
                .iter()
                .rev()
                .take(8)
                .map(|(id, size)| format!("{id:08x}:{size}"))
                .collect::<Vec<_>>()
                .join("  "),
        );
    }
    if active && gm.view_ids {
        let ids = npcs
            .iter()
            .take(20)
            .map(|npc| format!("{}({})", npc.0.npc_id, npc.0.npc_type))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("NPC ID(type): {ids}"));
    }
    if active && gm.view_collision {
        lines.push(format!("colliders={}", colliders.iter().count()));
    }
    let text = LocalizedText::new(
        "ui.chat.command.gm.diagnostics",
        "GM diagnostics / announcement\n{details}",
    )
    .with_arg("details", lines.join("\n"));
    if let Ok((_, mut current)) = overlays.single_mut() {
        current.set_if_neq(text);
    } else {
        commands.spawn((
            GmDebugOverlay,
            Node {
                position_type: PositionType::Absolute,
                top: px(12.0),
                left: px(220.0),
                max_width: px(800.0),
                ..default()
            },
            Text::default(),
            text,
            TextFont {
                font: (assets.load("fonts/chaletbook-regular.ttf")).into(),
                font_size: (16.0).into(),
                ..default()
            },
            TextColor(Color::WHITE),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            GlobalZIndex(2000),
        ));
    }
}
