use super::*;

/// Read and validate the behaviour document of one tile.
pub fn load_native_world_behaviours(
    asset_root: &Path,
    scope: NativeWorldScope,
    tile_id: &str,
) -> Result<NativeWorldBehaviourDocument, WorldBehaviourError> {
    let relative = behaviour_document_path(scope, tile_id);
    let path = asset_root.join(&relative);
    let bytes = std::fs::read(&path).map_err(|source| WorldBehaviourError::Io {
        path: path.display().to_string(),
        source,
    })?;
    parse_native_world_behaviours(&bytes, &relative, tile_id)
}

pub(super) fn parse_native_world_behaviours(
    bytes: &[u8],
    source_path: &str,
    tile_id: &str,
) -> Result<NativeWorldBehaviourDocument, WorldBehaviourError> {
    let relative = source_path;
    let canonical_tile_id = canonical_map_tile_id(tile_id);
    let document: NativeWorldBehaviourDocument =
        serde_json::from_slice(&bytes).map_err(|source| WorldBehaviourError::Json {
            path: relative.to_owned(),
            source,
        })?;
    if document.schema != NATIVE_WORLD_BEHAVIOUR_SCHEMA {
        return Err(WorldBehaviourError::Contract(format!(
            "{relative} declares schema {:?}, expected {NATIVE_WORLD_BEHAVIOUR_SCHEMA}",
            document.schema
        )));
    }
    if document.id != canonical_tile_id {
        return Err(WorldBehaviourError::Contract(format!(
            "{relative} declares tile {:?}, expected {canonical_tile_id:?}",
            document.id
        )));
    }
    if document.scope != "worldMap" {
        return Err(WorldBehaviourError::Contract(format!(
            "{relative} declares scope {:?}, expected {:?}",
            document.scope, "worldMap"
        )));
    }
    let mut closure_ids = std::collections::BTreeSet::new();
    for closure in &document.effect_prefab_closures {
        if closure.schema != WORLD_EFFECT_PREFAB_CLOSURE_SCHEMA
            || closure.id.is_empty()
            || closure.objects.is_empty()
        {
            return Err(WorldBehaviourError::Contract(format!(
                "{relative} contains invalid world effect prefab closure {:?}",
                closure.id
            )));
        }
        if !closure_ids.insert(closure.id.as_str()) {
            return Err(WorldBehaviourError::Contract(format!(
                "{relative} duplicates world effect prefab closure {:?}",
                closure.id
            )));
        }
        let root_asset = closure
            .root
            .get("asset")
            .and_then(serde_json::Value::as_str);
        let root_path_id = closure
            .root
            .get("pathId")
            .and_then(serde_json::Value::as_i64);
        if root_asset.is_none()
            || root_path_id.is_none()
            || !closure.objects.iter().any(|object| {
                Some(object.asset.as_str()) == root_asset
                    && Some(object.path_id) == root_path_id
                    && object.object_type == "GameObject"
            })
        {
            return Err(WorldBehaviourError::Contract(format!(
                "{relative} effect prefab closure {:?} has no exact GameObject root",
                closure.id
            )));
        }
    }
    for emitter in document
        .effect_emitters
        .iter()
        .filter(|emitter| emitter.controller == "EffectEmitterController")
    {
        if emitter.resolved_particle_prefabs.len() != emitter.particles.len() {
            return Err(WorldBehaviourError::Contract(format!(
                "{relative} EffectEmitterController {:?} has {} particles but {} prefab resolutions",
                emitter.node,
                emitter.particles.len(),
                emitter.resolved_particle_prefabs.len()
            )));
        }
        if let Some(missing) = emitter
            .resolved_particle_prefabs
            .iter()
            .flatten()
            .find(|id| !closure_ids.contains(id.as_str()))
        {
            return Err(WorldBehaviourError::Contract(format!(
                "{relative} EffectEmitterController {:?} references absent prefab closure {missing:?}",
                emitter.node
            )));
        }
    }
    Ok(document)
}

#[derive(Component)]
pub struct PendingWorldBehaviourLoad(pub(super) Option<Task<Result<LoadedNativeWorldBehaviour, String>>>);

pub(super) fn retire_world_behaviour_load_task(task: Task<Result<LoadedNativeWorldBehaviour, String>>) {
    IoTaskPool::get()
        .spawn(async move {
            // Cancellation returns a completed output when the producer won
            // the race. Keep that value, including its decoded graph, on this
            // worker until it is dropped here.
            drop(task.cancel().await);
        })
        .detach();
}
