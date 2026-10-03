use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeWorldSceneError {
    pub(super) message: String,
}

impl NativeWorldSceneError {
    pub(super) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for NativeWorldSceneError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for NativeWorldSceneError {}

pub(super) fn validate_name(value: &str, context: &str) -> Result<(), NativeWorldSceneError> {
    if value.is_empty() || value.len() > 1_024 || value.chars().any(char::is_control) {
        return Err(NativeWorldSceneError::new(format!(
            "{context} is empty, too long, or contains a control character"
        )));
    }
    Ok(())
}

pub(super) fn validate_plain_or_prefixed_blake3(
    value: &str,
    context: &str,
) -> Result<(), NativeWorldSceneError> {
    let digest = value.strip_prefix("blake3:").unwrap_or(value);
    validate_blake3(digest, context)
}

pub(super) fn validate_serialized_transform(
    transform: &NativeWorldSerializedTransform,
    context: &str,
) -> Result<(), NativeWorldSceneError> {
    if transform.translation.iter().any(|value| !value.is_finite())
        || transform.rotation.iter().any(|value| !value.is_finite())
        || transform
            .scale
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
    {
        return Err(NativeWorldSceneError::new(format!(
            "{context} contains a non-finite or non-positive serialized TRS"
        )));
    }
    let rotation_length_squared = transform
        .rotation
        .iter()
        .map(|value| value * value)
        .sum::<f64>();
    if (rotation_length_squared - 1.0).abs() > f64::from(UNIT_QUATERNION_TOLERANCE) {
        return Err(NativeWorldSceneError::new(format!(
            "{context} contains a non-unit serialized quaternion"
        )));
    }
    Ok(())
}

pub(super) fn validate_root_chain(
    root_chain: &NativeWorldRootChain,
    owner_parent_transform_path_id: Option<i64>,
    expected_root_transform_path_id: i64,
) -> Result<(), NativeWorldSceneError> {
    if root_chain.order != "immediateParentToRoot"
        || root_chain.includes_owner_transform
        || root_chain.nodes.len() > 256
        || root_chain.composition_order_transform_path_ids.len() != root_chain.nodes.len()
        || root_chain.map_tile_root_transform_path_id != expected_root_transform_path_id
    {
        return Err(NativeWorldSceneError::new(
            "native terrain rootChain header/order is contradictory",
        ));
    }
    if root_chain
        .composition_order_transform_path_ids
        .iter()
        .zip(&root_chain.nodes)
        .any(|(path_id, node)| *path_id != node.transform_path_id)
    {
        return Err(NativeWorldSceneError::new(
            "native terrain rootChain composition ids do not match nodes",
        ));
    }
    match root_chain.nodes.first() {
        Some(first) if owner_parent_transform_path_id != Some(first.transform_path_id) => {
            return Err(NativeWorldSceneError::new(
                "rootChain first node is not the owner's exact parent",
            ));
        }
        None if owner_parent_transform_path_id.is_some() => {
            return Err(NativeWorldSceneError::new(
                "rootChain is empty despite a non-null owner parent",
            ));
        }
        _ => {}
    }
    for (index, node) in root_chain.nodes.iter().enumerate() {
        if node.asset_name.is_empty()
            || node.transform_path_id <= 0
            || node.game_object.asset_name.is_empty()
            || node.game_object.path_id <= 0
        {
            return Err(NativeWorldSceneError::new(
                "rootChain contains an invalid serialized identity",
            ));
        }
        node.native_local_transform
            .try_to_bevy("rootChain nativeLocalTransform")?;
        validate_serialized_transform(
            &node.unity_local_transform,
            "rootChain unityLocalTransform",
        )?;
        let expected_parent = root_chain
            .nodes
            .get(index + 1)
            .map(|parent| parent.transform_path_id);
        if node.parent_pointer.as_ref().map(|pointer| pointer.path_id) != expected_parent {
            return Err(NativeWorldSceneError::new(
                "rootChain parentPointer does not match the next owner-to-root node",
            ));
        }
    }
    if root_chain
        .nodes
        .last()
        .is_some_and(|node| node.transform_path_id != root_chain.map_tile_root_transform_path_id)
    {
        return Err(NativeWorldSceneError::new(
            "rootChain terminal node is not mapTileRootTransformPathId",
        ));
    }
    let value = serde_json::to_value(&root_chain.nodes).map_err(|error| {
        NativeWorldSceneError::new(format!("could not canonicalize rootChain nodes: {error}"))
    })?;
    let bytes = serde_json::to_vec(&value).map_err(|error| {
        NativeWorldSceneError::new(format!("could not hash rootChain nodes: {error}"))
    })?;
    let actual = format!("blake3:{}", blake3::hash(&bytes).to_hex());
    if root_chain.canonical_json_blake3 != actual {
        return Err(NativeWorldSceneError::new(
            "rootChain canonicalJsonBlake3 differs from its exact typed nodes",
        ));
    }
    Ok(())
}

pub(super) fn validate_scene_instance(
    instance: &NativeWorldTerrainInstance,
    document: &NativeTerrainSceneInstance,
) -> Result<(), NativeWorldSceneError> {
    if document.schema != "ffone.native-terrain-scene-instance.v1"
        || document.coordinate_contract.space != "native"
        || document.coordinate_contract.basis != WORLD_BASIS
        || document.coordinate_contract.unit_scale != WORLD_UNIT_SCALE
        || !document.terrain_data_local_geometry_only
        || !document.transform_is_not_baked_into_heightmap
        || document.application_order
            != "terrainDataLocalVertex -> owner localTransform -> each rootChain parent in owner-to-root order"
        || document.terrain_data.true_name != instance.true_name
        || Some(document.terrain_data.source_path_id) != instance.terrain_data_path_id
        || document.linkage.terrain_collider.path_id != instance.terrain_collider_path_id
        || document.linkage.game_object.path_id != instance.terrain_game_object_path_id
        || document.linkage.transform.path_id != instance.terrain_transform_path_id
        || document.linkage.transform.parent_transform_path_id != instance.parent_transform_path_id
        || Some(document.linkage.terrain_component.path_id) != instance.terrain_component_path_id
        || instance.terrain_render_contract.as_ref()
            != Some(&document.linkage.terrain_component.render_contract)
        || document.local_transform != instance.transform
        || instance.root_chain.as_ref() != Some(&document.root_chain)
        || document.map_tile_root_transform_path_id
            != document.root_chain.map_tile_root_transform_path_id
        || document.root_chain_blake3 != document.root_chain.canonical_json_blake3
        || instance
            .source_game_object_true_name
            .as_deref()
            .is_some_and(|name| name != document.linkage.game_object.true_name)
    {
        return Err(NativeWorldSceneError::new(
            "native world scene and exact scene-instance metadata disagree",
        ));
    }
    match (&instance.environment, &document.environment) {
        (None, None) => {}
        (Some(instance_environment), Some(document_environment)) => {
            validate_plain_or_prefixed_blake3(
                &document_environment.blake3,
                "scene-instance environment",
            )?;
            let instance_hash = instance_environment
                .blake3
                .strip_prefix("blake3:")
                .unwrap_or(&instance_environment.blake3);
            let document_hash = document_environment
                .blake3
                .strip_prefix("blake3:")
                .unwrap_or(&document_environment.blake3);
            if document_environment.schema != NATIVE_TERRAIN_ENVIRONMENT_SCHEMA
                || document_environment.status != "complete"
                || document_environment.path != "environment/environment.json"
                || instance_environment.schema != document_environment.schema
                || instance_environment.status != document_environment.status
                || !instance_environment
                    .path
                    .ends_with(&format!("/{}", document_environment.path))
                || instance_hash != document_hash
            {
                return Err(NativeWorldSceneError::new(
                    "scene-instance environment reference disagrees with its world scene",
                ));
            }
        }
        _ => {
            return Err(NativeWorldSceneError::new(
                "scene-instance and world scene must either both reference environment data or both omit it",
            ));
        }
    }
    let terrain_pointer = &document.linkage.terrain_data_pointer;
    if terrain_pointer.resolved_path_id != document.terrain_data.source_path_id
        || document
            .linkage
            .terrain_component
            .terrain_data_pointer
            .file_id
            != terrain_pointer.file_id
        || document
            .linkage
            .terrain_component
            .terrain_data_pointer
            .path_id
            != terrain_pointer.path_id
        || document
            .linkage
            .terrain_collider
            .contract
            .terrain_data_pointer
            .file_id
            != terrain_pointer.file_id
        || document
            .linkage
            .terrain_collider
            .contract
            .terrain_data_pointer
            .path_id
            != terrain_pointer.path_id
    {
        return Err(NativeWorldSceneError::new(
            "scene-instance TerrainData pointers do not resolve to one identity",
        ));
    }
    validate_plain_or_prefixed_blake3(&document.map_scene.blake3, "map scene")?;
    validate_root_chain(
        &document.root_chain,
        document.linkage.transform.parent_transform_path_id,
        document.map_tile_root_transform_path_id,
    )?;
    validate_render_contract(&document.linkage.terrain_component.render_contract)?;
    let mut component_ids = HashSet::new();
    for component in &document.owner_components {
        if component.asset_name.is_empty()
            || component.path_id <= 0
            || component.raw_data.byte_length == 0
            || !component_ids.insert(component.path_id)
        {
            return Err(NativeWorldSceneError::new(
                "scene-instance ownerComponents contain invalid/duplicate identities",
            ));
        }
        validate_relative_asset_path(&component.parsed_data.path, "json")?;
        validate_relative_asset_path(&component.raw_data.path, "bin")?;
        validate_plain_or_prefixed_blake3(&component.parsed_data.blake3, "component parsedData")?;
        validate_plain_or_prefixed_blake3(&component.raw_data.blake3, "component rawData")?;
    }
    Ok(())
}

pub(super) fn validate_catalog_entry(
    entry: &NativeWorldCatalogEntry,
    require_provenance: bool,
) -> Result<(), NativeWorldSceneError> {
    if entry.instance_id.trim().is_empty()
        || !matches!(entry.placement_status.as_str(), "linked" | "blocked")
    {
        return Err(NativeWorldSceneError::new(
            "catalog entry identity/placement status is invalid",
        ));
    }
    validate_relative_asset_path(&entry.terrain_descriptor, "json")?;
    if require_provenance {
        validate_relative_asset_path(&entry.provenance, "json")?;
    } else if !entry.provenance.is_empty() {
        return Err(NativeWorldSceneError::new(
            "runtime world registry must not retain conversion provenance",
        ));
    }
    validate_blake3(&entry.terrain_descriptor_blake3, &entry.terrain_descriptor)?;
    if !entry
        .terrain_descriptor
        .starts_with(entry.scope.canonical_scene_prefix())
        || entry
            .terrain_descriptor
            .to_ascii_lowercase()
            .contains("terrain.glb")
    {
        return Err(NativeWorldSceneError::new(
            "catalog terrain path is outside its canonical scope",
        ));
    }
    if let Some(scene) = &entry.scene {
        validate_relative_asset_path(scene, "json")?;
        if !scene.starts_with(entry.scope.canonical_scene_prefix()) {
            return Err(NativeWorldSceneError::new(
                "catalog scene path is outside its canonical scope",
            ));
        }
    }
    if let Some(hash) = &entry.scene_blake3 {
        validate_blake3(hash, "catalog scene")?;
    }
    if let Some(environment) = &entry.environment {
        validate_relative_asset_path(&environment.path, "json")?;
        validate_blake3(&environment.blake3, "catalog environment")?;
        let descriptor_parent = entry
            .terrain_descriptor
            .rsplit_once('/')
            .map(|(parent, _)| parent)
            .ok_or_else(|| {
                NativeWorldSceneError::new("catalog terrain descriptor has no parent path")
            })?;
        let expected_environment = format!("{descriptor_parent}/environment/environment.json");
        if environment.schema != NATIVE_TERRAIN_ENVIRONMENT_SCHEMA
            || environment.path != expected_environment
            || !matches!(
                environment.status.as_str(),
                "complete" | "complete-with-placement-blocker" | "blocked"
            )
        {
            return Err(NativeWorldSceneError::new(
                "catalog environment reference differs from the native terrain environment contract",
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_runtime_world_reference(
    reference: &RuntimeWorldAssetReference,
    context: &str,
) -> Result<(), NativeWorldSceneError> {
    validate_relative_asset_path(&reference.path, "json")?;
    validate_blake3(&reference.blake3, context)
}

pub(super) fn validate_runtime_world_registry_entry(
    entry: &RuntimeWorldRegistryEntry,
) -> Result<(), NativeWorldSceneError> {
    validate_name(&entry.id, "runtime world id")?;
    let unified_id = format!("map_{:02}_{:02}", entry.tile[0], entry.tile[1]);
    let unified_scene = format!("map/tiles/{}/scene.json", entry.id);
    let unified_terrain = format!("map/tiles/{}/terrain/terrain.json", entry.id);
    let unified_environment = format!(
        "map/tiles/{}/terrain/environment/environment.json",
        entry.id
    );
    let unified = entry.scope == NativeWorldScope::WorldMap
        && entry.id == unified_id
        && entry.scene.path == unified_scene
        && entry.terrain.path == unified_terrain
        && entry
            .environment
            .as_ref()
            .is_none_or(|reference| reference.path == unified_environment);
    let (id_prefix, scene, terrain, environment) = match entry.scope {
        NativeWorldScope::WorldMap => (
            "map",
            format!("world/maps/{}/scene.json", entry.id),
            format!("world/maps/{}/terrain/terrain.json", entry.id),
            format!(
                "world/maps/{}/terrain/environment/environment.json",
                entry.id
            ),
        ),
        NativeWorldScope::Tutorial => (
            "tile",
            format!("world/tutorial/terrain/tiles/{}/scene.json", entry.id),
            format!("world/tutorial/terrain/tiles/{}/terrain.json", entry.id),
            format!(
                "world/tutorial/terrain/tiles/{}/environment/environment.json",
                entry.id
            ),
        ),
    };
    let expected_id = format!("{id_prefix}_{:02}_{:02}", entry.tile[0], entry.tile[1]);
    let legacy = entry.id == expected_id
        && entry.scene.path == scene
        && entry.terrain.path == terrain
        && entry
            .environment
            .as_ref()
            .is_none_or(|reference| reference.path == environment);
    if !unified && !legacy {
        return Err(NativeWorldSceneError::new(format!(
            "runtime world entry {:?} has a scope/tile/path identity mismatch",
            entry.id
        )));
    }
    if unified {
        for (reference, expected, context) in [
            (
                entry.behaviour.as_ref(),
                format!("map/tiles/{}/behaviour.json", entry.id),
                "runtime world behaviour",
            ),
            (
                entry.objects.as_ref(),
                format!("map/tiles/{}/objects.json", entry.id),
                "runtime world object routes",
            ),
        ] {
            if reference.is_some_and(|reference| reference.path != expected) {
                return Err(NativeWorldSceneError::new(format!(
                    "{context} path disagrees with the unified tile identity"
                )));
            }
        }
    }
    for (reference, context) in [
        (&entry.scene, "runtime world scene"),
        (&entry.terrain, "runtime terrain descriptor"),
    ] {
        validate_relative_asset_path(&reference.path, "json")?;
        validate_blake3(&reference.blake3, context)?;
    }
    if let Some(reference) = &entry.environment {
        validate_relative_asset_path(&reference.path, "json")?;
        validate_blake3(&reference.blake3, "runtime terrain environment")?;
    }
    if let Some(reference) = &entry.behaviour {
        validate_runtime_world_reference(reference, "runtime world behaviour")?;
    }
    if let Some(reference) = &entry.objects {
        validate_runtime_world_reference(reference, "runtime world object routes")?;
    }
    Ok(())
}

pub(super) fn validate_runtime_scene_link(
    entry: &RuntimeWorldRegistryEntry,
    scene: &NativeWorldScene,
) -> Result<(), NativeWorldSceneError> {
    let terrain_matches = scene.native_terrain.as_ref().is_some_and(|terrain| {
        terrain.path == entry.terrain.path
            && terrain.blake3 == entry.terrain.blake3
            && match (&terrain.environment, &entry.environment) {
                (None, None) => true,
                (Some(scene), Some(registry)) => {
                    scene.path == registry.path && scene.blake3 == registry.blake3
                }
                _ => false,
            }
    });
    if scene.schema != NATIVE_WORLD_SCENE_SCHEMA
        || scene.scope != Some(entry.scope)
        || scene.tile != entry.tile
        || scene.name != entry.id
        || !terrain_matches
    {
        return Err(NativeWorldSceneError::new(
            "runtime world registry entry disagrees with its native scene",
        ));
    }
    Ok(())
}

pub(super) fn validate_catalog_scene_link(
    entry: &NativeWorldCatalogEntry,
    scene: &NativeWorldScene,
) -> Result<(), NativeWorldSceneError> {
    if scene.schema != NATIVE_WORLD_SCENE_SCHEMA
        || scene.scope != Some(entry.scope)
        || scene.tile != entry.tile
        || scene.name != entry.instance_id
        || scene.native_terrain.as_ref().is_none_or(|terrain| {
            terrain.path != entry.terrain_descriptor
                || terrain.blake3 != entry.terrain_descriptor_blake3
                || terrain.environment != entry.environment
        })
    {
        return Err(NativeWorldSceneError::new(
            "v2 catalog entry disagrees with its typed native world scene",
        ));
    }
    Ok(())
}

pub(super) fn validate_blake3(value: &str, path: &str) -> Result<(), NativeWorldSceneError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(NativeWorldSceneError::new(format!(
            "model {path:?} has an invalid lowercase BLAKE3 digest"
        )));
    }
    Ok(())
}

pub(super) fn validate_relative_asset_path(value: &str, extension: &str) -> Result<(), NativeWorldSceneError> {
    if value.is_empty() || value.contains('\\') {
        return Err(NativeWorldSceneError::new(format!(
            "unsafe native asset path {value:?}"
        )));
    }
    let path = Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        || path.extension().and_then(|value| value.to_str()) != Some(extension)
    {
        return Err(NativeWorldSceneError::new(format!(
            "unsafe native asset path {value:?}"
        )));
    }
    Ok(())
}
