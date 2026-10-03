use super::*;

pub(super) fn resolve_remote_player(
    world: &mut World,
    epoch: NetworkSessionEpoch0104,
    local_player_id: Option<i32>,
    pc_id: i32,
    frame: DecodedFrame,
) -> Option<Entity> {
    if local_player_id == Some(pc_id) {
        push_ignored_frame(
            world,
            epoch,
            frame,
            IgnoredLifecycleFrameReason0104::LocalPlayer { pc_id },
        );
        return None;
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
        return None;
    };
    if world.get_entity(entity).is_err() || world.get::<NetworkPcAppearance0104>(entity).is_none() {
        if world.get_entity(entity).is_err() {
            world.resource_mut::<RemotePcRegistry0104>().remove(pc_id);
        }
        push_ignored_frame(
            world,
            epoch,
            frame,
            IgnoredLifecycleFrameReason0104::MissingAppearance {
                kind: LifecycleEntityKind0104::Player,
                id: pc_id,
            },
        );
        return None;
    }

    Some(entity)
}
