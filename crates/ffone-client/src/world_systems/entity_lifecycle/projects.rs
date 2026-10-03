use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NetworkSessionEpoch0104(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct NetworkSessionEntity0104 {
    pub epoch: NetworkSessionEpoch0104,
}

pub(super) fn begin_session(world: &mut World, epoch: NetworkSessionEpoch0104, local_player_id: i32) {
    clear_all_session_entities(world);
    *world.resource_mut::<ActiveNetworkEntitySession0104>() = ActiveNetworkEntitySession0104 {
        epoch: Some(epoch),
        local_player_id: Some(local_player_id),
    };
    world
        .resource_mut::<NetworkEntityLifecycleStats0104>()
        .sessions_started += 1;
}

pub(super) fn disconnect_session(world: &mut World, epoch: NetworkSessionEpoch0104) {
    let active = *world.resource::<ActiveNetworkEntitySession0104>();
    if active.epoch != Some(epoch) {
        return;
    }
    clear_all_session_entities(world);
    *world.resource_mut::<ActiveNetworkEntitySession0104>() =
        ActiveNetworkEntitySession0104::default();
}

pub(super) fn clear_all_session_entities(world: &mut World) {
    let entities = {
        let mut query = world.query_filtered::<Entity, With<NetworkSessionEntity0104>>();
        query.iter(world).collect::<Vec<_>>()
    };
    let had_session_state = !entities.is_empty()
        || !world.resource::<RemotePcRegistry0104>().is_empty()
        || !world.resource::<NetworkNpcRegistry0104>().is_empty()
        || !world
            .resource::<NetworkTransportationRegistry0104>()
            .is_empty()
        || !world.resource::<NetworkShinyRegistry0104>().is_empty();
    for entity in entities {
        let _ = world.despawn(entity);
    }
    world.resource_mut::<RemotePcRegistry0104>().clear();
    world.resource_mut::<NetworkNpcRegistry0104>().clear();
    world
        .resource_mut::<NetworkTransportationRegistry0104>()
        .clear();
    world.resource_mut::<NetworkShinyRegistry0104>().clear();
    world.resource_mut::<NetworkNpcResultEffectEvents0104>().0.clear();
    world.resource_mut::<NetworkHealingTickEffects0104>().0.clear();
    world
        .resource_mut::<NetworkNanoEffectEvents0104>()
        .0
        .clear();
    world
        .resource_mut::<NetworkNpcSkillEffectEvents0104>()
        .0
        .clear();
    world
        .resource_mut::<NetworkNpcAttackEventQueue0104>()
        .clear();
    world
        .resource_mut::<NetworkNpcBarkerEventQueue0104>()
        .clear();
    if had_session_state {
        world
            .resource_mut::<NetworkEntityLifecycleStats0104>()
            .sessions_cleared += 1;
    }
}
