use super::*;

/// Recover the per-instance model id retained by the original static-world
/// publication from an organizer-preserved source route such as
/// `models/world/maps/map_08_06/v-01384-11153-....glb`.
///
/// The readable map organizer deliberately deduplicates the runtime GLB table,
/// so its `map-geometry-*` id is not an instance identity.  Behaviour records
/// still own the original `static-map_08_06-01384` identity and must be joined
/// through this route instead of the deduplicated geometry id.
pub(super) fn legacy_static_model_id(source_model_path: &str) -> Option<String> {
    let normalized = source_model_path.replace('\\', "/");
    let mut components = normalized.split('/').collect::<Vec<_>>();
    let file_name = components.pop()?;
    let scope = components.pop()?;
    let mut file_parts = file_name.split('-');
    let role = file_parts.next()?;
    if !matches!(role, "v" | "c") {
        return None;
    }
    let ordinal = file_parts.next()?;
    if ordinal.len() != 5 || !ordinal.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    Some(format!("static-{scope}-{ordinal}"))
}

/// Move one published visual/collider below a scripted pivot without moving
/// it in world space. The previous implementation inserted only
/// `inverse(parent_world)`, which discarded the model's authored placement and
/// sent moving geometry toward the world origin. Keeping the original model
/// matrix is also what makes the visible mesh and its triangle collider remain
/// coincident while the parent platform or animation advances.
pub(super) fn reparent_authored_world_model(
    commands: &mut Commands,
    model: Entity,
    parent: Entity,
    parent_world: Mat4,
    authored_model_worlds: &HashMap<Entity, Mat4>,
) {
    let Some(model_world) = authored_model_worlds.get(&model).copied() else {
        return;
    };
    let local = authored_model_local_transform(parent_world, model_world);
    commands.entity(model).insert((ChildOf(parent), local));
}

pub(super) fn authored_model_local_transform(parent_world: Mat4, model_world: Mat4) -> Transform {
    Transform::from_matrix(parent_world.inverse() * model_world)
}

pub(super) fn entity_or_ancestor_is_model(
    mut entity: Entity,
    models: &[Entity],
    parents: &Query<&ChildOf>,
) -> bool {
    for _ in 0..16 {
        if models.contains(&entity) {
            return true;
        }
        let Ok(parent) = parents.get(entity) else {
            return false;
        };
        entity = parent.parent();
    }
    false
}
