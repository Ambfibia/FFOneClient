use super::*;

pub(super) fn despawn_player(world: &mut World, local_player_id: Option<i32>, pc_id: i32) {
    if local_player_id == Some(pc_id) {
        return;
    }
    let Some(entity) = world.resource_mut::<RemotePcRegistry0104>().remove(pc_id) else {
        return;
    };
    if world
        .get::<RemotePcVisibility0104>(entity)
        .is_some_and(|fade| fade.alpha > 0.0)
    {
        let alpha = world
            .get::<RemotePcVisibility0104>(entity)
            .map_or(1.0, |fade| fade.alpha);
        world
            .entity_mut(entity)
            .remove::<(
                NetworkRemotePc0104,
                NetworkPcAppearance0104,
                RemoteMotion,
                RemoteAnimation,
                PendingPcVisual0104,
            )>()
            .insert(RemotePcVisibility0104 {
                alpha,
                retiring: true,
            });
    } else {
        let _ = world.despawn(entity);
    }
    world
        .resource_mut::<NetworkEntityLifecycleStats0104>()
        .player_despawns += 1;
}

pub(super) fn despawn_npc(world: &mut World, npc_id: i32) {
    let Some(entity) = world
        .resource_mut::<NetworkNpcRegistry0104>()
        .remove(npc_id)
    else {
        return;
    };
    let _ = world.despawn(entity);
    world
        .resource_mut::<NetworkEntityLifecycleStats0104>()
        .npc_despawns += 1;
}

pub(super) fn despawn_transportation(world: &mut World, transportation_kind: i32, id: i32) {
    let Some(entity) = world
        .resource_mut::<NetworkTransportationRegistry0104>()
        .remove(transportation_kind, id)
    else {
        return;
    };
    let _ = world.despawn(entity);
    world
        .resource_mut::<NetworkEntityLifecycleStats0104>()
        .transportation_despawns += 1;
}

pub fn despawn_shiny(world: &mut World, shiny_id: i32) {
    let Some(entity) = world
        .resource_mut::<NetworkShinyRegistry0104>()
        .remove(shiny_id)
    else {
        return;
    };
    let _ = world.despawn(entity);
    world
        .resource_mut::<NetworkEntityLifecycleStats0104>()
        .shiny_despawns += 1;
}
