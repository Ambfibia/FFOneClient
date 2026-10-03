use super::*;

pub fn consume_network_entity_lifecycle_0104(world: &mut World) {
    let events = {
        let mut ingress = world.resource_mut::<NetworkEntityLifecycleIngress0104>();
        ingress.events.drain(..).collect::<Vec<_>>()
    };

    for event in events {
        match event {
            NetworkEntityLifecycleIngressEvent0104::BeginSession {
                epoch,
                local_player_id,
            } => begin_session(world, epoch, local_player_id),
            NetworkEntityLifecycleIngressEvent0104::Bootstrap { epoch, entities } => {
                consume_bootstrap(world, epoch, entities);
            }
            NetworkEntityLifecycleIngressEvent0104::Frame { epoch, frame } => {
                consume_live_frame(world, epoch, frame);
            }
            NetworkEntityLifecycleIngressEvent0104::Disconnect { epoch } => {
                disconnect_session(world, epoch);
            }
        }
    }
}

pub(super) fn consume_bootstrap(
    world: &mut World,
    epoch: NetworkSessionEpoch0104,
    entities: InitialAroundPacket0104,
) {
    let active = *world.resource::<ActiveNetworkEntitySession0104>();
    if active.epoch != Some(epoch) {
        world
            .resource_mut::<LifecycleBootstrapDiagnostics0104>()
            .stale
            .push(IgnoredLifecycleBootstrap0104 {
                epoch,
                active_epoch: active.epoch,
                entities,
            });
        return;
    }

    world
        .resource_mut::<NetworkEntityLifecycleStats0104>()
        .bootstrap_packets += 1;
    match entities {
        InitialAroundPacket0104::Players(players) => {
            for appearance in players {
                upsert_player(world, epoch, active.local_player_id, appearance);
            }
        }
        InitialAroundPacket0104::Npcs(npcs) => {
            for appearance in npcs {
                upsert_npc(world, epoch, appearance);
            }
        }
        InitialAroundPacket0104::Transportation(entries) => {
            for appearance in entries {
                upsert_transportation(world, epoch, appearance);
            }
        }
        InitialAroundPacket0104::Shinies(entries) => {
            for appearance in entries {
                upsert_shiny(world, epoch, appearance);
            }
        }
    }
}

pub(super) fn upsert_player(
    world: &mut World,
    epoch: NetworkSessionEpoch0104,
    local_player_id: Option<i32>,
    appearance: PcAppearance0104,
) {
    if local_player_id == Some(appearance.id) {
        world
            .resource_mut::<NetworkEntityLifecycleStats0104>()
            .local_player_appearances_ignored += 1;
        return;
    }

    let pc_id = appearance.id;
    let position = ProtocolPosition::new(appearance.position).to_native();
    let rotation = ProtocolYawDegrees::new(appearance.angle).native_root_rotation();
    let mut motion = RemoteMotion::default();
    motion.target_position = position;
    motion.target_rotation = rotation;
    let components = (
        Name::new(format!("network player {pc_id}")),
        NetworkRemotePc0104 { pc_id },
        RemotePlayer::new(pc_id),
        NetworkPcAppearance0104(appearance.clone()),
        PendingPcVisual0104::from(&appearance),
        NetworkSessionEntity0104 { epoch },
        motion,
        RemoteAnimation::default(),
        Transform {
            translation: position,
            rotation,
            ..default()
        },
        Visibility::Inherited,
    );

    let existing = world.resource::<RemotePcRegistry0104>().get(pc_id);
    let entity = match existing.filter(|entity| world.get_entity(*entity).is_ok()) {
        Some(entity) => {
            world.entity_mut(entity).insert(components);
            entity
        }
        None => world.spawn((components, RemotePcVisibility0104::default())).id(),
    };
    world
        .resource_mut::<RemotePcRegistry0104>()
        .insert(pc_id, entity);
    world
        .resource_mut::<NetworkEntityLifecycleStats0104>()
        .player_upserts += 1;
}

pub(super) fn upsert_npc(world: &mut World, epoch: NetworkSessionEpoch0104, appearance: NpcAppearance0104) {
    let npc_id = appearance.npc_id;
    let position = ProtocolPosition::new(appearance.position).to_native();
    let rotation = ProtocolYawDegrees::new(appearance.angle).native_root_rotation();
    let components = (
        Name::new(format!(
            "network NPC {} type {}",
            npc_id, appearance.npc_type
        )),
        NetworkNpc0104 {
            npc_id,
            npc_type: appearance.npc_type,
        },
        NetworkNpcAppearance0104(appearance),
        PendingNpcVisual0104::from(appearance),
        NetworkSessionEntity0104 { epoch },
        Transform {
            translation: position,
            rotation,
            ..default()
        },
        Visibility::Inherited,
    );

    let existing = world.resource::<NetworkNpcRegistry0104>().get(npc_id);
    let entity = match existing.filter(|entity| world.get_entity(*entity).is_ok()) {
        Some(entity) => {
            world.entity_mut(entity).insert(components);
            world
                .entity_mut(entity)
                .remove::<NetworkNpcMotion0104>()
                .remove::<NetworkNpcCombatAnimation0104>()
                .remove::<NetworkNpcReadyAnimation0104>()
                .remove::<NetworkNpcSkillPhase0104>();
            if let Some(mut layers) = world.get_mut::<NetworkNpcAnimationLayers0104>(entity) {
                layers.reset();
            }
            entity
        }
        None => world.spawn(components).id(),
    };
    world
        .resource_mut::<NetworkNpcRegistry0104>()
        .insert(npc_id, entity);
    world
        .resource_mut::<NetworkEntityLifecycleStats0104>()
        .npc_upserts += 1;
}

pub(super) fn upsert_transportation(
    world: &mut World,
    epoch: NetworkSessionEpoch0104,
    appearance: TransportationAppearance0104,
) {
    let id = appearance.id;
    let components = (
        Name::new(format!(
            "network transportation {} type {}",
            id, appearance.transportation_type
        )),
        NetworkTransportation0104 {
            transportation_kind: appearance.transportation_kind,
            id,
            transportation_type: appearance.transportation_type,
        },
        NetworkTransportationAppearance0104(appearance),
        NetworkSessionEntity0104 { epoch },
        Transform::from_translation(ProtocolPosition::new(appearance.position).to_native()),
        Visibility::Inherited,
    );
    let existing = world
        .resource::<NetworkTransportationRegistry0104>()
        .get(appearance.transportation_kind, id);
    let entity = match existing.filter(|entity| world.get_entity(*entity).is_ok()) {
        Some(entity) => {
            world.entity_mut(entity).insert(components);
            world
                .entity_mut(entity)
                .remove::<NetworkTransportationMotion0104>();
            entity
        }
        None => world.spawn(components).id(),
    };
    world
        .resource_mut::<NetworkTransportationRegistry0104>()
        .insert(appearance.transportation_kind, id, entity);
    world
        .resource_mut::<NetworkEntityLifecycleStats0104>()
        .transportation_upserts += 1;
}

pub(super) fn upsert_shiny(
    world: &mut World,
    epoch: NetworkSessionEpoch0104,
    appearance: ShinyAppearance0104,
) {
    let shiny_id = appearance.shiny_id;
    let components = (
        Name::new(format!(
            "network shiny {} type {}",
            shiny_id, appearance.shiny_type
        )),
        NetworkShiny0104 {
            shiny_id,
            shiny_type: appearance.shiny_type,
            map_number: appearance.map_number,
        },
        NetworkShinyAppearance0104(appearance),
        NetworkSessionEntity0104 { epoch },
        Transform::from_translation(ProtocolPosition::new(appearance.position).to_native()),
        Visibility::Inherited,
    );
    let existing = world.resource::<NetworkShinyRegistry0104>().get(shiny_id);
    let entity = match existing.filter(|entity| world.get_entity(*entity).is_ok()) {
        Some(entity) => {
            world.entity_mut(entity).insert(components);
            entity
        }
        None => world.spawn(components).id(),
    };
    world
        .resource_mut::<NetworkShinyRegistry0104>()
        .insert(shiny_id, entity);
    world
        .resource_mut::<NetworkEntityLifecycleStats0104>()
        .shiny_upserts += 1;
}

pub(super) fn face_network_npc_toward_npc(world: &mut World, npc_id: i32, target_id: i32) {
    let registry = world.resource::<NetworkNpcRegistry0104>();
    let (Some(attacker), Some(target)) = (registry.get(npc_id), registry.get(target_id)) else {
        return;
    };
    let Some(position) = world
        .get::<Transform>(target)
        .map(|transform| transform.translation)
    else {
        return;
    };
    let Some(mut transform) = world.get_mut::<Transform>(attacker) else {
        return;
    };
    let direction = position - transform.translation;
    let horizontal = Vec3::new(direction.x, 0.0, direction.z);
    if horizontal.length_squared() > f32::EPSILON {
        transform.rotation = Transform::IDENTITY
            .looking_to(horizontal.normalize(), Vec3::Y)
            .rotation;
    }
    // A companion's melee packet ends the last follow segment. Otherwise that
    // segment could turn the actor away from its target on the following frame.
    world.entity_mut(attacker).remove::<NetworkNpcMotion0104>();
}

pub(super) fn face_network_npc_toward_pc(
    world: &mut World,
    npc_id: i32,
    pc_id: i32,
    local_player_id: Option<i32>,
) {
    let Some(npc_entity) = world.resource::<NetworkNpcRegistry0104>().get(npc_id) else {
        return;
    };
    let remote_entity = world.resource::<RemotePcRegistry0104>().get(pc_id);
    let mut target_position = remote_entity
        .and_then(|entity| world.get::<Transform>(entity))
        .map(|transform| transform.translation);
    if target_position.is_none() && local_player_id == Some(pc_id) {
        let mut players = world.query_filtered::<&Transform, With<LegacyPlayerController>>();
        target_position = players
            .iter(world)
            .next()
            .map(|transform| transform.translation);
    }
    let Some(target_position) = target_position else {
        return;
    };
    let Some(mut transform) = world.get_mut::<Transform>(npc_entity) else {
        return;
    };
    let direction = target_position - transform.translation;
    let horizontal = Vec3::new(direction.x, 0.0, direction.z);
    if horizontal.length_squared() > f32::EPSILON {
        transform.rotation = Transform::IDENTITY
            .looking_to(horizontal.normalize(), Vec3::Y)
            .rotation;
    }
}
