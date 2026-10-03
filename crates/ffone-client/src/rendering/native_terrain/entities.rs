use super::*;

pub fn spawn_pending_native_heightmap(
    commands: &mut Commands,
    parent: Entity,
    name: impl Into<String>,
    transform: Transform,
    terrain: Arc<NativeTerrain>,
) -> Entity {
    // Preparation starts through a bounded scheduler below. Spawning every
    // neighboring tile directly on AsyncComputeTaskPool let grass generation
    // occupy all workers at once and contend with the render schedule.
    commands
        .spawn((
            Name::new(name.into()),
            NativeWorldSceneEntity,
            ChildOf(parent),
            transform,
            Visibility::Inherited,
            NativeWorldColliderStatus::Loading,
            PendingNativeHeightmapTerrain {
                terrain: Some(terrain),
                task: None,
            },
        ))
        .id()
}
