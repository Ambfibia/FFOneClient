use super::*;

pub(super) fn despawn_before_residency(
    mut commands: Commands,
    targets: Query<Entity, With<DespawnBeforeResidency>>,
) {
    for entity in &targets {
        commands.entity(entity).despawn();
    }
}
