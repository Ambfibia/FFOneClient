use super::*;

#[derive(Debug, Default, Resource)]
pub struct NetworkHealingTickEffects0104(pub VecDeque<ffone_protocol::TimeBuffHealTick0104>);

pub(super) fn apply_player_motion(
    world: &mut World,
    epoch: NetworkSessionEpoch0104,
    local_player_id: Option<i32>,
    packet: DecodedRemotePacket,
    frame: DecodedFrame,
) {
    let pc_id = packet.pc_id();
    if local_player_id == Some(pc_id) {
        push_ignored_frame(
            world,
            epoch,
            frame,
            IgnoredLifecycleFrameReason0104::LocalPlayer { pc_id },
        );
        return;
    }

    let Some(entity) = world.resource::<RemotePcRegistry0104>().get(pc_id) else {
        push_ignored_frame(
            world,
            epoch,
            frame,
            IgnoredLifecycleFrameReason0104::MissingAppearance {
                kind: LifecycleEntityKind0104::Player,
                id: pc_id,
            },
        );
        return;
    };
    if world.get_entity(entity).is_err() {
        world.resource_mut::<RemotePcRegistry0104>().remove(pc_id);
        push_ignored_frame(
            world,
            epoch,
            frame,
            IgnoredLifecycleFrameReason0104::MissingAppearance {
                kind: LifecycleEntityKind0104::Player,
                id: pc_id,
            },
        );
        return;
    }

    let mut motion = world
        .get::<RemoteMotion>(entity)
        .copied()
        .unwrap_or_default();
    let mut animation = world
        .get::<RemoteAnimation>(entity)
        .copied()
        .unwrap_or_default();
    motion.apply_packet(packet, &mut animation);
    world.entity_mut(entity).insert((motion, animation));
}

pub(super) fn apply_player_regen(
    world: &mut World,
    epoch: NetworkSessionEpoch0104,
    local_player_id: Option<i32>,
    packet: PcRegen0104,
    frame: DecodedFrame,
) {
    let Some(entity) = resolve_remote_player(world, epoch, local_player_id, packet.pc_id, frame)
    else {
        return;
    };
    let Some(mut appearance) = world.get::<NetworkPcAppearance0104>(entity).cloned() else {
        return;
    };

    appearance.0.hp = packet.hp;
    appearance.0.position = packet.position;
    appearance.0.angle = packet.angle;
    appearance.0.condition_bit_flag = packet.condition_bit_flag;
    appearance.0.pc_state = packet.pc_state;
    appearance.0.special_state = packet.special_state;
    appearance.0.nano = packet.nano;

    let position = ProtocolPosition::new(packet.position).to_native();
    let rotation = ProtocolYawDegrees::new(packet.angle).native_root_rotation();
    let mut motion = RemoteMotion::default();
    motion.target_position = position;
    motion.target_rotation = rotation;
    let pending_visual = world
        .get::<PendingPcVisual0104>(entity)
        .cloned()
        .map(|mut pending| {
            pending.nano = packet.nano;
            pending
        });

    let mut entity = world.entity_mut(entity);
    entity.insert((appearance, motion, RemoteAnimation::default()));
    if let Some(pending_visual) = pending_visual {
        entity.insert(pending_visual);
    }
    if let Some(mut transform) = entity.get_mut::<Transform>() {
        transform.translation = position;
        transform.rotation = rotation;
    }
}

pub(super) fn apply_player_sudden_dead(
    world: &mut World,
    epoch: NetworkSessionEpoch0104,
    local_player_id: Option<i32>,
    packet: PcSuddenDead0104,
    frame: DecodedFrame,
) {
    let Some(entity) = resolve_remote_player(world, epoch, local_player_id, packet.pc_id, frame)
    else {
        return;
    };
    let Some(mut appearance) = world.get::<NetworkPcAppearance0104>(entity).cloned() else {
        return;
    };
    appearance.0.hp = packet.hp;

    // The current remote runtime has no death-pose component. Stop all
    // interpolation at the authoritative visible pose so a dead PC cannot
    // continue sliding while preserving every unrelated appearance field.
    let transform = world.get::<Transform>(entity).cloned();
    let prior_motion = world.get::<RemoteMotion>(entity).copied();
    let mut motion = RemoteMotion::default();
    if let Some(transform) = transform {
        motion.target_position = transform.translation;
        motion.target_rotation = transform.rotation;
    } else if let Some(prior_motion) = prior_motion {
        motion.target_position = prior_motion.target_position;
        motion.target_rotation = prior_motion.target_rotation;
    }

    world
        .entity_mut(entity)
        .insert((appearance, motion, RemoteAnimation::default()));
}

pub(super) fn apply_npc_motion(
    world: &mut World,
    epoch: NetworkSessionEpoch0104,
    packet: NpcMove0104,
    frame: DecodedFrame,
) {
    let Some(entity) = world
        .resource::<NetworkNpcRegistry0104>()
        .get(packet.npc_id)
    else {
        push_ignored_frame(
            world,
            epoch,
            frame,
            IgnoredLifecycleFrameReason0104::MissingAppearance {
                kind: LifecycleEntityKind0104::Npc,
                id: packet.npc_id,
            },
        );
        return;
    };
    if world.get_entity(entity).is_err() {
        world
            .resource_mut::<NetworkNpcRegistry0104>()
            .remove(packet.npc_id);
        push_ignored_frame(
            world,
            epoch,
            frame,
            IgnoredLifecycleFrameReason0104::MissingAppearance {
                kind: LifecycleEntityKind0104::Npc,
                id: packet.npc_id,
            },
        );
        return;
    }

    world
        .entity_mut(entity)
        .insert(NetworkNpcMotion0104 {
            destination: ProtocolPosition::new(packet.destination).to_native(),
            speed: protocol_distance_to_native(packet.speed),
            move_style: packet.move_style,
        })
        .remove::<NetworkNpcReadyAnimation0104>();
}

pub(super) fn apply_npc_attack_results(world: &mut World, results: Vec<AttackResult0104>) {
    for result in results {
        // This packet family exclusively contains NPC targets. OpenFusion's
        // PC_ATTACK_NPCS response leaves sAttackResult.eCT zero-initialized,
        // so filtering on eCT discards every real hit and leaves the HUD on
        // the spawn HP forever. The typed NPC registry is the discriminator.
        let Some(entity) = world.resource::<NetworkNpcRegistry0104>().get(result.id) else {
            continue;
        };
        let previous_hp;
        {
            let Some(mut appearance) = world.get_mut::<NetworkNpcAppearance0104>(entity) else {
                continue;
            };
            previous_hp = appearance.0.hp;
            appearance.0.hp = result.hp;
        }
        apply_network_npc_hp_animation(world, entity, result.id, previous_hp, result.hp);
        world
            .resource_mut::<NetworkEntityLifecycleStats0104>()
            .npc_damage_updates += 1;
    }
}

/// Route mixed PvP results the way `GameFrame` does: `eCT == 4` is an NPC,
/// anything else is a player. The pinned server leaves `eCT` zeroed in its
/// NPC-only replies, so an unknown type still tries the NPC registry first
/// before falling back to the remote player registry.
pub(super) fn apply_mixed_attack_results(world: &mut World, results: &[AttackResult0104]) {
    for result in results {
        let is_npc = match result.entity_type {
            NPC_ATTACK_ENTITY_TYPE => true,
            PC_ATTACK_ENTITY_TYPE => false,
            _ => world
                .resource::<NetworkNpcRegistry0104>()
                .get(result.id)
                .is_some(),
        };
        if is_npc {
            apply_npc_attack_results(world, vec![*result]);
        } else {
            apply_pc_attack_results(world, std::slice::from_ref(result));
        }
    }
}

pub(super) fn apply_pc_attack_results(world: &mut World, results: &[AttackResult0104]) {
    for result in results {
        // NPC_ATTACK_PCS is likewise a typed packet family and its clean
        // server response does not populate eCT. Local PC status remains
        // owned by the client runtime; only registered remote appearances are
        // mirrored here.
        let Some(entity) = world.resource::<RemotePcRegistry0104>().get(result.id) else {
            continue;
        };
        let Some(mut appearance) = world.get_mut::<NetworkPcAppearance0104>(entity) else {
            continue;
        };
        appearance.0.hp = result.hp;
    }
}

pub(super) fn apply_world_nano_authority(
    world: &mut World,
    epoch: NetworkSessionEpoch0104,
    local_player_id: Option<i32>,
    projection: &WorldNanoAuthoritativeProjection0104,
    frame: &DecodedFrame,
) {
    // `RuntimeStatus` in `main` is the sole owner of local HP, Nano slots,
    // batteries, conditions and cross-map controller warps. Lifecycle only
    // mirrors the caster prefix when that caster is a remote appearance.
    if local_player_id != Some(projection.caster.pc_id) {
        apply_remote_pc_authority(
            world,
            epoch,
            local_player_id,
            projection.caster.pc_id,
            RemotePcAuthorityUpdate0104 {
                absolute_hp: projection.caster.absolute_hp,
                absolute_nano_id: Some(projection.caster.nano_id),
                absolute_nano_skill_id: Some(projection.caster.skill_id),
                absolute_nano_stamina: Some(projection.caster.nano_stamina),
                nano_deactivated: Some(projection.caster.nano_deactivated),
                ..default()
            },
            frame,
        );
    }

    for target in &projection.targets {
        match target.target.kind {
            WorldNanoEntityKind0104::Player => apply_remote_pc_authority(
                world,
                epoch,
                local_player_id,
                target.target.id,
                RemotePcAuthorityUpdate0104 {
                    absolute_hp: target.absolute_hp,
                    absolute_condition_bit_flag: target.absolute_condition_bit_flag,
                    absolute_nano_stamina: target.absolute_nano_stamina,
                    nano_deactivated: target.nano_deactivated,
                    movement: target
                        .movement
                        .map(|movement| (movement.map_number, movement.position)),
                    ..default()
                },
                frame,
            ),
            WorldNanoEntityKind0104::Npc | WorldNanoEntityKind0104::Mob => {
                apply_npc_authority(
                    world,
                    epoch,
                    target.target.id,
                    NpcAuthorityUpdate0104 {
                        absolute_hp: target.absolute_hp,
                        absolute_condition_bit_flag: target.absolute_condition_bit_flag,
                        movement: target.movement.map(|movement| movement.position),
                    },
                    frame,
                );
            }
        }
    }
}

pub(super) fn apply_world_npc_skill_authority(
    world: &mut World,
    epoch: NetworkSessionEpoch0104,
    local_player_id: Option<i32>,
    projection: &WorldNpcSkillAuthoritativeProjection0104,
    frame: &DecodedFrame,
) {
    // Resolve the caster even when this family has no scalar caster post-state.
    // That preserves the lifecycle missing/offscreen diagnostic for hit frames
    // whose animation root has already left the streamed slice.
    apply_npc_authority(
        world,
        epoch,
        projection.caster.npc_id,
        NpcAuthorityUpdate0104 {
            absolute_hp: projection.caster.absolute_hp,
            ..default()
        },
        frame,
    );

    for target in &projection.targets {
        match target.target.kind {
            WorldNpcSkillEntityKind0104::Player => apply_remote_pc_authority(
                world,
                epoch,
                local_player_id,
                target.target.id,
                RemotePcAuthorityUpdate0104 {
                    absolute_hp: target.absolute_hp,
                    absolute_condition_bit_flag: target.absolute_condition_bit_flag,
                    absolute_nano_id: target.absolute_nano_id,
                    absolute_nano_stamina: target.absolute_nano_stamina,
                    nano_deactivated: target.nano_deactivated,
                    movement: target
                        .movement
                        .map(|movement| (movement.map_number, movement.position)),
                    ..default()
                },
                frame,
            ),
            WorldNpcSkillEntityKind0104::Npc | WorldNpcSkillEntityKind0104::Mob => {
                apply_npc_authority(
                    world,
                    epoch,
                    target.target.id,
                    NpcAuthorityUpdate0104 {
                        absolute_hp: target.absolute_hp,
                        absolute_condition_bit_flag: target.absolute_condition_bit_flag,
                        movement: target.movement.map(|movement| movement.position),
                    },
                    frame,
                );
            }
        }
    }
}

pub(super) fn apply_remote_pc_authority(
    world: &mut World,
    epoch: NetworkSessionEpoch0104,
    local_player_id: Option<i32>,
    pc_id: i32,
    update: RemotePcAuthorityUpdate0104,
    frame: &DecodedFrame,
) {
    let Some(entity) = resolve_remote_player(world, epoch, local_player_id, pc_id, frame.clone())
    else {
        return;
    };
    let Some(mut appearance) = world.get::<NetworkPcAppearance0104>(entity).cloned() else {
        return;
    };

    if let Some(hp) = update.absolute_hp {
        appearance.0.hp = hp;
    }
    if let Some(condition_bit_flag) = update.absolute_condition_bit_flag {
        appearance.0.condition_bit_flag = condition_bit_flag;
    }

    let previous_nano = appearance.0.nano;
    if update.nano_deactivated == Some(true) || update.absolute_nano_id == Some(0) {
        appearance.0.nano = Nano0104 {
            id: 0,
            skill_id: 0,
            stamina: 0,
        };
    } else {
        if let Some(nano_id) = update.absolute_nano_id {
            if appearance.0.nano.id != nano_id && update.absolute_nano_skill_id.is_none() {
                // Corruption names the active Nano but carries no skill ID.
                // Never retain a different Nano's skill as false authority.
                appearance.0.nano.skill_id = 0;
            }
            appearance.0.nano.id = nano_id;
        }
        if let Some(skill_id) = update.absolute_nano_skill_id {
            appearance.0.nano.skill_id = skill_id;
        }
        if let Some(stamina) = update.absolute_nano_stamina {
            appearance.0.nano.stamina = stamina;
        }
    }

    if let Some((map_number, position)) = update.movement {
        appearance.0.map_number = map_number;
        appearance.0.position = position;
    }
    let updated_nano = appearance.0.nano;
    world.entity_mut(entity).insert(appearance);

    if updated_nano != previous_nano
        && let Some(mut pending_visual) = world.get::<PendingPcVisual0104>(entity).cloned()
    {
        pending_visual.nano = updated_nano;
        world.entity_mut(entity).insert(pending_visual);
    }

    if let Some((_, position)) = update.movement {
        let position = ProtocolPosition::new(position).to_native();
        let rotation = if let Some(mut transform) = world.get_mut::<Transform>(entity) {
            transform.translation = position;
            transform.rotation
        } else {
            Quat::IDENTITY
        };
        let mut motion = world
            .get::<RemoteMotion>(entity)
            .copied()
            .unwrap_or_default();
        motion.target_position = position;
        motion.target_rotation = rotation;
        motion.velocity = Vec3::ZERO;
        motion.speed = 0.0;
        motion.seconds_without_movement_packet = 0.0;
        motion.movement_key = 0;
        world.entity_mut(entity).insert(motion);
    }
}

pub(super) fn apply_npc_authority(
    world: &mut World,
    epoch: NetworkSessionEpoch0104,
    npc_id: i32,
    update: NpcAuthorityUpdate0104,
    frame: &DecodedFrame,
) {
    let Some(entity) = world.resource::<NetworkNpcRegistry0104>().get(npc_id) else {
        push_ignored_frame(
            world,
            epoch,
            frame.clone(),
            IgnoredLifecycleFrameReason0104::MissingAppearance {
                kind: LifecycleEntityKind0104::Npc,
                id: npc_id,
            },
        );
        return;
    };
    if world.get_entity(entity).is_err() || world.get::<NetworkNpcAppearance0104>(entity).is_none()
    {
        if world.get_entity(entity).is_err() {
            world
                .resource_mut::<NetworkNpcRegistry0104>()
                .remove(npc_id);
        }
        push_ignored_frame(
            world,
            epoch,
            frame.clone(),
            IgnoredLifecycleFrameReason0104::MissingAppearance {
                kind: LifecycleEntityKind0104::Npc,
                id: npc_id,
            },
        );
        return;
    }
    if update.absolute_hp.is_none()
        && update.absolute_condition_bit_flag.is_none()
        && update.movement.is_none()
    {
        return;
    }

    let mut appearance = *world
        .get::<NetworkNpcAppearance0104>(entity)
        .expect("resolved NPC appearance must remain present");
    let previous_hp = appearance.0.hp;
    if let Some(hp) = update.absolute_hp {
        appearance.0.hp = hp;
    }
    if let Some(condition_bit_flag) = update.absolute_condition_bit_flag {
        appearance.0.condition_bit_flag = condition_bit_flag;
    }
    if let Some(position) = update.movement {
        appearance.0.position = position;
    }
    world.entity_mut(entity).insert(appearance);

    if let Some(hp) = update.absolute_hp {
        apply_network_npc_hp_animation(world, entity, npc_id, previous_hp, hp);
    }
    if let Some(position) = update.movement {
        let position = ProtocolPosition::new(position).to_native();
        if let Some(mut transform) = world.get_mut::<Transform>(entity) {
            transform.translation = position;
        }
        world.entity_mut(entity).remove::<NetworkNpcMotion0104>();
    }
    if update.absolute_hp.is_some() {
        world
            .resource_mut::<NetworkEntityLifecycleStats0104>()
            .npc_damage_updates += 1;
    }
}

pub(super) fn apply_transportation_motion(
    world: &mut World,
    epoch: NetworkSessionEpoch0104,
    packet: TransportationMove0104,
    frame: DecodedFrame,
) {
    let Some(entity) = world
        .resource::<NetworkTransportationRegistry0104>()
        .get(packet.transportation_kind, packet.id)
    else {
        push_ignored_frame(
            world,
            epoch,
            frame,
            IgnoredLifecycleFrameReason0104::MissingAppearance {
                kind: LifecycleEntityKind0104::Transportation,
                id: packet.id,
            },
        );
        return;
    };
    if world.get_entity(entity).is_err() {
        world
            .resource_mut::<NetworkTransportationRegistry0104>()
            .remove(packet.transportation_kind, packet.id);
        push_ignored_frame(
            world,
            epoch,
            frame,
            IgnoredLifecycleFrameReason0104::MissingAppearance {
                kind: LifecycleEntityKind0104::Transportation,
                id: packet.id,
            },
        );
        return;
    }

    world
        .entity_mut(entity)
        .insert(NetworkTransportationMotion0104 {
            destination: ProtocolPosition::new(packet.destination).to_native(),
            speed: protocol_distance_to_native(packet.speed),
            move_style: packet.move_style,
        });
}
