//! Persistent Status.UpdateSkillBuff visuals, owned by live combatants.
use super::*;
use ffone_client::entity_lifecycle::{
    NetworkNanoEffectEvents0104, NetworkNpcResultEffectEvents0104,
};
use ffone_client::world_nano_authority::WorldNanoEntityKind0104;
use ffone_client::world_npc_skill_authority::{
    WorldNpcSkillCastKind0104, WorldNpcSkillSourceResult0104,
};

fn skill_buff(skill: i32) -> i32 {
    match skill {
        11 => 1,
        10 => 3,
        12 => 4,
        16 => 5,
        17 => 6,
        18 => 7,
        5 => 8,
        4 => 11,
        14 => 13,
        15 => 14,
        19 => 15,
        20 => 16,
        23 => 17,
        25 => 18,
        31 => 19,
        32 => 20,
        35 => 24,
        33 => 21,
        _ => 0,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_world_instant_skill_effects(
    state: Res<State<ClientState>>,
    runtime: Res<RuntimeStatus>,
    content: Res<TutorialMissionContent>,
    mut events: ResMut<NetworkNanoEffectEvents0104>,
    mut npc_events: ResMut<NetworkNpcResultEffectEvents0104>,
    local: Query<Entity, With<LocalPlayer>>,
    npcs: Query<(Entity, &NetworkNpcAppearance0104)>,
    pcs: Query<(Entity, &NetworkPcAppearance0104)>,
    names: Query<&Name>,
    presentation: (
        Query<&GlobalTransform>,
        Query<(
            Entity,
            &ffone_client::tutorial_nano_gameplay::TutorialGameplayNanoRoot,
        )>,
        Query<(Entity, &ffone_client::entity_lifecycle::NetworkShiny0104)>,
    ),
    children: Query<&Children>,
    scenes: Query<&LegacyCharacterSceneStatus>,
    rigs: Query<&TutorialSelectedPlayerRigStatus>,
    mut pending: Local<Vec<(Entity, i32, i32, f32)>>,
    mut effects: ResMut<TutorialEffectRuntime>,
) {
    if *state.get() != ClientState::World {
        events.0.clear();
        npc_events.0.clear();
        pending.clear();
        return;
    }
    let (poses, nanos, shinies) = presentation;
    let mut hits = Vec::new();
    for event in events.0.drain(..) {
        hits.extend(event.targets.into_iter().map(|target| {
            (
                event.caster.skill_id,
                event.skill_type,
                true,
                event.caster.pc_id,
                target.source_result,
                0,
            )
        }));
    }
    for event in npc_events.0.drain(..) {
        let (skill_type, projectile_offset) = match event.kind {
            WorldNpcSkillCastKind0104::Skill { skill_type } => (skill_type, 0),
            WorldNpcSkillCastKind0104::Corruption { style } => (22, i32::from(style)),
        };
        hits.extend(event.targets.into_iter().filter_map(
            |target| match target.source_result {
                WorldNpcSkillSourceResult0104::Skill(result) => Some((
                    event.caster.skill_id,
                    skill_type,
                    false,
                    event.caster.npc_id,
                    result,
                    projectile_offset,
                )),
                WorldNpcSkillSourceResult0104::Corruption(result) => Some((
                    event.caster.skill_id,
                    skill_type,
                    false,
                    event.caster.npc_id,
                    ffone_protocol::NanoSkillResult0104::Damage(
                        ffone_protocol::NanoSkillDamageResult0104 {
                            target: result.target,
                            protected: result.protected,
                            damage: result.damage,
                            hp: result.hp,
                        },
                    ),
                    projectile_offset,
                )),
            },
        ));
    }
    for (skill_id, skill_type, player_caster, caster_id, result, projectile_offset) in hits {
        let buff = skill_buff(skill_type);
        let protected = match result {
            ffone_protocol::NanoSkillResult0104::DamageDebuff(r) => r.protected,
            ffone_protocol::NanoSkillResult0104::Damage(r) => r.protected,
            ffone_protocol::NanoSkillResult0104::Buff(r) => r.protected,
            ffone_protocol::NanoSkillResult0104::BatteryDrain(r) => r.protected,
            _ => 0,
        };
        if protected > 0 {
            continue;
        }
        let target = result.target();
        let target_kind = match target.entity_type {
            1 => WorldNanoEntityKind0104::Player,
            2 => WorldNanoEntityKind0104::Npc,
            4 => WorldNanoEntityKind0104::Mob,
            _ => continue,
        };
        let owner = match target_kind {
            WorldNanoEntityKind0104::Player if runtime.player_id == Some(target.id) => {
                local.iter().next().map(|e| (e, 1.0))
            }
            WorldNanoEntityKind0104::Player => pcs
                .iter()
                .find(|(_, pc)| pc.0.id == target.id)
                .map(|(e, _)| (e, 1.0)),
            WorldNanoEntityKind0104::Npc | WorldNanoEntityKind0104::Mob => npcs
                .iter()
                .find(|(_, npc)| npc.0.npc_id == target.id)
                .map(|(e, npc)| {
                    (
                        e,
                        content
                            .gameplay_npc(npc.0.npc_type)
                            .map_or(1.0, |n| n.radius() * 2.0),
                    )
                }),
        };
        if let Some((entity, scale)) = owner {
            // An active condition edge is presented by sync_world_skill_effects.
            // Only independent instant results are emitted here, avoiding two
            // copies when PC_BUFF_UPDATE and SKILL_USE arrive together.
            let condition = match result {
                ffone_protocol::NanoSkillResult0104::Buff(r) => r.condition_bit_flag,
                ffone_protocol::NanoSkillResult0104::DamageDebuff(r) => r.condition_bit_flag,
                _ => 0,
            };
            if !(buff > 0 && condition as u32 & (1 << (buff - 1)) != 0)
                && let Some(row) = content
                    .gameplay_skill_buff(buff)
                    .filter(|r| r.instant_effect_id > 0)
            {
                pending.push((entity, buff, row.instant_effect_id, scale));
            }
            if let Some(skill) = content
                .gameplay_skill(skill_id)
                .filter(|r| r.target_effect > 0)
                && let Ok(target_pose) = poses.get(entity)
            {
                let projectile = skill.target_effect + projectile_offset;
                let caster = if player_caster && runtime.player_id == Some(caster_id) {
                    local.iter().next()
                } else if player_caster {
                    pcs.iter()
                        .find(|(_, p)| p.0.id == caster_id)
                        .map(|(e, _)| e)
                } else {
                    npcs.iter()
                        .find(|(_, n)| n.0.npc_id == caster_id)
                        .map(|(e, _)| e)
                };
                let caster = caster
                    .and_then(|e| {
                        nanos
                            .iter()
                            .find(|(_, n)| n.owner == e)
                            .map(|(n, _)| n)
                            .or(Some(e))
                    })
                    .or_else(|| {
                        (!player_caster)
                            .then(|| {
                                shinies
                                    .iter()
                                    .find(|(_, s)| s.shiny_id == caster_id)
                                    .map(|(e, _)| e)
                            })
                            .flatten()
                    });
                let (fire_link, success_link) = effects
                    .native_skill_projectile_links(projectile)
                    .or_else(|| {
                        effects.library().and_then(|l| {
                            validated_world_weapon_bullet_links(l, projectile)
                        })
                    })
                    .unwrap_or(("", ""));
                let named_position = |root, name: &str| {
                    if name.is_empty() || name == "\"" {
                        return None;
                    }
                    let mut matches = Vec::new();
                    tutorial_descendants_named(root, name, &names, &children, &mut matches);
                    if matches.len() != 1 {
                        return None;
                    }
                    poses.get(matches[0]).ok().map(|p| p.translation())
                };
                let center = |entity| {
                    npcs.get(entity)
                        .ok()
                        .and_then(|(_, n)| content.gameplay_npc(n.0.npc_type))
                        .map_or(0.8, |n| n.height_server_units as f32 * 0.005)
                };
                let target_position = named_position(entity, success_link)
                    .unwrap_or(target_pose.translation() + Vec3::Y * center(entity));
                let source = caster
                    .and_then(|e| {
                        named_position(e, fire_link).or_else(|| {
                            poses.get(e).ok().map(|p| {
                                p.translation()
                                    + if nanos.get(e).is_ok() {
                                        Vec3::ZERO
                                    } else {
                                        Vec3::Y * center(e)
                                    }
                            })
                        })
                    })
                    .unwrap_or(target_position);
                effects.enqueue(TutorialEffectRuntimeCommand::Projectile {
                    bullet_type: projectile,
                    source,
                    target: target_position,
                    target_exists: true,
                    source_style: -1,
                    target_style: -1,
                    motion: TutorialProjectileMotion::BulletMove,
                    source_line: 5574,
                });
            }
        }
    }
    pending.retain(|&(entity, buff, effect_id, scale)| {
        let Ok(pose) = poses.get(entity) else {
            return false;
        };
        let candidates = if buff == 11 {
            &["state"][..]
        } else {
            attachment_names(buff)
        };
        let mut attachment = None;
        for candidate in candidates {
            let mut matches = Vec::new();
            tutorial_descendants_named(entity, candidate, &names, &children, &mut matches);
            if matches.len() == 1 {
                attachment = Some((
                    candidate.to_string(),
                    Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                ));
                break;
            }
        }
        if attachment.is_none()
            && !(candidates.is_empty()
                || tutorial_actor_scene_is_terminal(entity, &scenes, &children)
                || children.iter_descendants(entity).any(|e| {
                    rigs.get(e)
                        .is_ok_and(|s| matches!(s, TutorialSelectedPlayerRigStatus::Ready))
                }))
        {
            return true;
        }
        let placement = if let Some((node_name, rotation)) = attachment {
            TutorialEffectPlacement::ExactEntityBone {
                root_entity: entity,
                node_name,
                spawn_world_rotation: rotation,
                local_translation_after_parenting: Vec3::ZERO,
                local_rotation_after_parenting: rotation,
            }
        } else {
            TutorialEffectPlacement::ExactEntityWorld {
                root_entity: entity,
                position: pose.translation(),
                rotation: pose.rotation(),
            }
        };
        effects.enqueue(TutorialEffectRuntimeCommand::Add {
            effect_id,
            placement,
            scale,
            tracked: false,
            name: None,
            destroy_after_seconds: None,
            source_line: 1905,
        });
        false
    });
}

fn attachment_names(buff: i32) -> &'static [&'static str] {
    match buff {
        10 | 12 | 14 | 15 | 16 => &["state"],
        13 => &["Bip01 Footsteps"],
        19 => &["center", "Bip01 spine", "Bip01 spine1"],
        _ => &[],
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn sync_world_skill_effects(
    state: Res<State<ClientState>>,
    runtime: Res<RuntimeStatus>,
    content: Res<TutorialMissionContent>,
    buffs: Res<SkillBuffUiModel>,
    local: Query<(Entity, &GlobalTransform), With<LocalPlayer>>,
    npcs: Query<(Entity, &NetworkNpcAppearance0104, &GlobalTransform)>,
    pcs: Query<(Entity, &NetworkPcAppearance0104, &GlobalTransform), Without<LocalPlayer>>,
    names: Query<&Name>,
    children: Query<&Children>,
    scenes: Query<&LegacyCharacterSceneStatus>,
    rigs: Query<&TutorialSelectedPlayerRigStatus>,
    mut active: Local<BTreeMap<(Entity, i32), String>>,
    mut effects: ResMut<TutorialEffectRuntime>,
) {
    let mut owners = Vec::new();
    if *state.get() == ClientState::World {
        if runtime.hp.is_some_and(|hp| hp > 0) {
            owners.extend(
                local
                    .iter()
                    .map(|(entity, pose)| (entity, buffs.local_condition_bit_flag, 1.0, *pose)),
            );
        }
        owners.extend(npcs.iter().filter_map(|(entity, appearance, pose)| {
            (appearance.0.hp > 0).then(|| {
                let scale = content
                    .gameplay_npc(appearance.0.npc_type)
                    .map_or(1.0, |npc| npc.radius() * 2.0);
                (entity, appearance.0.condition_bit_flag as u32, scale, *pose)
            })
        }));
        owners.extend(
            pcs.iter()
                .filter(|(_, pc, _)| pc.0.hp > 0)
                .map(|(entity, pc, pose)| (entity, pc.0.condition_bit_flag as u32, 1.0, *pose)),
        );
    }
    let desired: BTreeSet<_> = owners
        .iter()
        .flat_map(|(entity, mask, _, _)| {
            (1..26)
                .filter(move |buff| mask & (1 << (buff - 1)) != 0)
                .map(move |buff| (*entity, buff))
        })
        .collect();
    active.retain(|key, name| {
        if desired.contains(key) {
            return true;
        }
        effects.enqueue(TutorialEffectRuntimeCommand::DestroyNamed {
            name: name.clone(),
            source_line: 800,
        });
        false
    });
    for (entity, mask, scale, pose) in owners {
        for buff in (1..26).filter(|buff| mask & (1 << (buff - 1)) != 0) {
            let key = (entity, buff);
            let Some(definition) = content.gameplay_skill_buff(buff) else {
                continue;
            };
            let persistent = definition.effect_id > 0;
            let effect_id = if persistent {
                definition.effect_id
            } else {
                definition.instant_effect_id
            };
            if effect_id <= 0
                || active
                    .get(&key)
                    .is_some_and(|name| !persistent || effects.has_named_native_instance(name))
            {
                continue;
            }
            let mut attachment = None;
            for candidate in attachment_names(buff) {
                let mut matches = Vec::new();
                tutorial_descendants_named(entity, candidate, &names, &children, &mut matches);
                if matches.len() == 1 {
                    attachment = Some((
                        (*candidate).to_owned(),
                        Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                    ));
                    break;
                }
            }
            // Do not select the root fallback while the model is still loading.
            if attachment.is_none()
                && !(attachment_names(buff).is_empty()
                    || tutorial_actor_scene_is_terminal(entity, &scenes, &children)
                    || children.iter_descendants(entity).any(|child| {
                        rigs.get(child).is_ok_and(|status| {
                            matches!(status, TutorialSelectedPlayerRigStatus::Ready)
                        })
                    }))
            {
                continue;
            }
            let placement = if let Some((node_name, local_rotation)) = attachment {
                TutorialEffectPlacement::ExactEntityBone {
                    root_entity: entity,
                    node_name,
                    spawn_world_rotation: pose.rotation(),
                    local_translation_after_parenting: Vec3::ZERO,
                    local_rotation_after_parenting: local_rotation,
                }
            } else {
                TutorialEffectPlacement::ExactEntityWorld {
                    root_entity: entity,
                    position: pose.translation(),
                    rotation: pose.rotation(),
                }
            };
            let name = format!("world skill buff {entity:?}/{buff}");
            effects.enqueue(TutorialEffectRuntimeCommand::Add {
                effect_id,
                placement,
                scale,
                tracked: false,
                name: Some(name.clone()),
                destroy_after_seconds: None,
                source_line: 800,
            });
            active.insert(key, name);
        }
    }
}

#[cfg(test)]
mod tests;

/// Timed healing uses Status.iHealEffect independently of the Nano cast animation.
pub(super) fn spawn_healing_tick_effects(
    state: Res<State<ClientState>>,
    runtime: Res<RuntimeStatus>,
    content: Res<TutorialMissionContent>,
    mut ticks: ResMut<ffone_client::entity_lifecycle::NetworkHealingTickEffects0104>,
    local: Query<(Entity, &GlobalTransform), With<LocalPlayer>>,
    pcs: Query<(Entity, &NetworkPcAppearance0104, &GlobalTransform)>,
    npcs: Query<(Entity, &NetworkNpcAppearance0104, &GlobalTransform)>,
    mut effects: ResMut<TutorialEffectRuntime>,
) {
    for tick in ticks.0.drain(..) {
        if *state.get() != ClientState::World {
            continue;
        }
        let owner = match tick.character_type {
            1 if runtime.player_id == Some(tick.character_id) => local
                .iter()
                .next()
                .map(|(e, t)| (e, t.translation() + Vec3::Y * 0.8, 1.0)),
            1 => pcs
                .iter()
                .find(|(_, p, _)| p.0.id == tick.character_id)
                .map(|(e, _, t)| (e, t.translation() + Vec3::Y * 0.8, 1.0)),
            2 | 4 => npcs
                .iter()
                .find(|(_, p, _)| p.0.npc_id == tick.character_id)
                .and_then(|(e, p, t)| {
                    let n = content.gameplay_npc(p.0.npc_type)?;
                    Some((
                        e,
                        t.translation() + Vec3::Y * (n.height_server_units as f32 * 0.005),
                        n.radius() * 2.0,
                    ))
                }),
            _ => None,
        };
        if let Some((entity, position, scale)) = owner {
            effects.enqueue(TutorialEffectRuntimeCommand::Add {
                effect_id: 54,
                placement: TutorialEffectPlacement::ExactEntityWorld {
                    root_entity: entity,
                    position,
                    rotation: Quat::IDENTITY,
                },
                scale,
                tracked: false,
                name: None,
                destroy_after_seconds: None,
                source_line: 1235,
            });
        }
    }
}
