use super::*;

#[derive(Component)]
pub(super) struct DespawnBeforeResidency;

#[derive(Resource, Default)]
pub(super) struct ResidencyVisibilityChanges(pub(super) usize);
