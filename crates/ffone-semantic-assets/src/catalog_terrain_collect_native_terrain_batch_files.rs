use super::*;

pub(super) fn collect_native_terrain_batch_files(
    plan_root: &Path,
    plan: &NativeTerrainPublicationPlan,
) -> Result<(BTreeMap<String, NativeTerrainBatchFile>, BTreeSet<String>)> {
    if plan.status.trim().is_empty() || plan.source_output_root.trim().is_empty() {
        return Err(SemanticAssetError::InvalidPlan(
            "publication plan status/sourceOutputRoot cannot be empty".to_owned(),
        ));
    }
    for blocker in &plan.blocked {
        if blocker.scope.trim().is_empty()
            || blocker.instance_id.trim().is_empty()
            || blocker.stage.trim().is_empty()
            || blocker.code.trim().is_empty()
            || blocker.message.trim().is_empty()
            || blocker.placement_status != "blocked"
        {
            return Err(SemanticAssetError::InvalidPlan(
                "publication blocker must be a complete typed blocked record".to_owned(),
            ));
        }
    }

    let mut files = BTreeMap::new();
    let mut destination_roots = BTreeSet::new();
    let mut catalog_entries = Vec::new();
    for entry in &plan.entries {
        if entry.instance_id.trim().is_empty() || entry.source_bundle.trim().is_empty() {
            return Err(SemanticAssetError::InvalidPlan(
                "publication entry instanceId/sourceBundle cannot be empty".to_owned(),
            ));
        }
        if !matches!(entry.placement_status.as_str(), "linked" | "blocked") {
            return Err(SemanticAssetError::InvalidPlan(format!(
                "{} has invalid placementStatus {}",
                entry.instance_id, entry.placement_status
            )));
        }
        validate_relative_path(&entry.source_root)?;
        let destination_root = entry
            .destination_root
            .strip_prefix("assets/game/")
            .ok_or_else(|| {
                SemanticAssetError::InvalidPlan(format!(
                    "{} destinationRoot must begin assets/game/",
                    entry.instance_id
                ))
            })?
            .to_owned();
        validate_relative_path(&destination_root)?;
        validate_batch_destination_root(&entry.scope, &destination_root)?;
        if !destination_roots.insert(destination_root.clone()) {
            return Err(SemanticAssetError::InvalidPlan(format!(
                "duplicate native terrain destinationRoot {destination_root}"
            )));
        }
        let source_root = plan_root.join(native_path(&entry.source_root));
        if !source_root.is_dir() {
            return Err(SemanticAssetError::Verification(format!(
                "{} sourceRoot is not a directory: {}",
                entry.instance_id,
                source_root.display()
            )));
        }

        validate_relative_path(&entry.terrain_document)?;
        if !entry.terrain_document.ends_with(".json") {
            return Err(SemanticAssetError::InvalidPlan(format!(
                "{} terrainDocument must be JSON",
                entry.instance_id
            )));
        }
        insert_native_terrain_batch_file(
            &mut files,
            &source_root,
            &destination_root,
            &entry.terrain_document,
            AssetKind::Data,
            &entry.terrain_document_blake3,
        )?;
        let terrain_path = source_root.join(native_path(&entry.terrain_document));
        let terrain = read_json(&terrain_path)?;
        let terrain_object = terrain.as_object().ok_or_else(|| {
            SemanticAssetError::Verification(format!(
                "{} terrainDocument is not an object",
                entry.instance_id
            ))
        })?;
        if required_str(terrain_object, "schema", &entry.terrain_document)?
            != "ffone.native-terrain.v1"
        {
            return Err(SemanticAssetError::Verification(format!(
                "{} terrainDocument schema is not ffone.native-terrain.v1",
                entry.instance_id
            )));
        }
        let heightmap = terrain_object
            .get("heightmap")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                SemanticAssetError::Verification(format!(
                    "{} terrainDocument has no heightmap",
                    entry.instance_id
                ))
            })?;
        let descriptor_heightmap_path = required_str(heightmap, "path", &entry.terrain_document)?;
        let descriptor_heightmap_hash =
            required_str(heightmap, "pngBlake3", &entry.terrain_document)?;
        if descriptor_heightmap_path != entry.heightmap_path
            || normalize_blake3(descriptor_heightmap_hash)?
                != normalize_blake3(&entry.heightmap_blake3)?
        {
            return Err(SemanticAssetError::Verification(format!(
                "{} publication entry disagrees with terrain heightmap",
                entry.instance_id
            )));
        }
        insert_native_terrain_batch_file(
            &mut files,
            &source_root,
            &destination_root,
            &entry.heightmap_path,
            AssetKind::Texture,
            &entry.heightmap_blake3,
        )?;

        let splat = terrain_object
            .get("splat")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                SemanticAssetError::Verification(format!(
                    "{} terrainDocument has no splat",
                    entry.instance_id
                ))
            })?;
        for weight in splat
            .get("weightMaps")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                SemanticAssetError::Verification(format!(
                    "{} terrainDocument has no weightMaps",
                    entry.instance_id
                ))
            })?
        {
            let weight = weight.as_object().ok_or_else(|| {
                SemanticAssetError::Verification("weight map is not an object".to_owned())
            })?;
            insert_native_terrain_batch_file(
                &mut files,
                &source_root,
                &destination_root,
                required_str(weight, "path", &entry.terrain_document)?,
                AssetKind::Texture,
                required_str(weight, "pngBlake3", &entry.terrain_document)?,
            )?;
        }
        for layer in splat
            .get("layers")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                SemanticAssetError::Verification(format!(
                    "{} terrainDocument has no layers",
                    entry.instance_id
                ))
            })?
        {
            let albedo = layer
                .get("albedo")
                .and_then(Value::as_object)
                .ok_or_else(|| {
                    SemanticAssetError::Verification("terrain layer has no albedo".to_owned())
                })?;
            insert_native_terrain_batch_file(
                &mut files,
                &source_root,
                &destination_root,
                required_str(albedo, "path", &entry.terrain_document)?,
                AssetKind::Texture,
                required_str(albedo, "pngBlake3", &entry.terrain_document)?,
            )?;
        }
        if let Some(attributes) = terrain_object
            .get("gameplayAttributes")
            .and_then(Value::as_object)
        {
            insert_native_terrain_batch_file(
                &mut files,
                &source_root,
                &destination_root,
                required_str(attributes, "path", &entry.terrain_document)?,
                AssetKind::Texture,
                required_str(attributes, "pngBlake3", &entry.terrain_document)?,
            )?;
            insert_native_terrain_batch_file(
                &mut files,
                &source_root,
                &destination_root,
                required_str(attributes, "rawPath", &entry.terrain_document)?,
                AssetKind::Data,
                required_str(attributes, "rawBlake3", &entry.terrain_document)?,
            )?;
            let parsed = attributes
                .get("rawParsedDocument")
                .and_then(Value::as_object)
                .ok_or_else(|| {
                    SemanticAssetError::Verification(
                        "gameplayAttributes has no rawParsedDocument".to_owned(),
                    )
                })?;
            insert_native_terrain_batch_file(
                &mut files,
                &source_root,
                &destination_root,
                required_str(parsed, "path", &entry.terrain_document)?,
                AssetKind::Data,
                required_str(parsed, "blake3", &entry.terrain_document)?,
            )?;
        }

        let environment = match (
            entry.environment_document.as_deref(),
            entry.environment_document_blake3.as_deref(),
            terrain_object.get("environment").and_then(Value::as_object),
        ) {
            (Some(path), Some(hash), Some(descriptor)) => {
                validate_relative_path(path)?;
                if !path.ends_with(".json") {
                    return Err(SemanticAssetError::InvalidPlan(format!(
                        "{} environmentDocument must be JSON",
                        entry.instance_id
                    )));
                }
                if required_str(descriptor, "schema", &entry.terrain_document)?
                    != "ffone.native-terrain-environment.v1"
                    || required_str(descriptor, "path", &entry.terrain_document)? != path
                    || normalize_blake3(required_str(
                        descriptor,
                        "blake3",
                        &entry.terrain_document,
                    )?)? != normalize_blake3(hash)?
                {
                    return Err(SemanticAssetError::Verification(format!(
                        "{} publication entry disagrees with terrain environment reference",
                        entry.instance_id
                    )));
                }
                insert_native_terrain_batch_file(
                    &mut files,
                    &source_root,
                    &destination_root,
                    path,
                    AssetKind::Data,
                    hash,
                )?;
                let document = read_json(&source_root.join(native_path(path)))?;
                let object = document.as_object().ok_or_else(|| {
                    SemanticAssetError::Verification(format!(
                        "{} environment document is not an object",
                        entry.instance_id
                    ))
                })?;
                if required_str(object, "schema", path)? != "ffone.native-terrain-environment.v1"
                    || required_str(object, "scope", path)? != entry.scope
                {
                    return Err(SemanticAssetError::Verification(format!(
                        "{} environment schema/scope is invalid",
                        entry.instance_id
                    )));
                }
                let (_, expected_y) =
                    parse_native_terrain_instance_coordinates(&entry.instance_id)?;
                let (expected_x, _) =
                    parse_native_terrain_instance_coordinates(&entry.instance_id)?;
                let expected_tile = format!("{expected_x:02}_{expected_y:02}");
                if required_str(object, "tileId", path)? != expected_tile {
                    return Err(SemanticAssetError::Verification(format!(
                        "{} environment tileId differs from {expected_tile}",
                        entry.instance_id
                    )));
                }
                let status = required_str(object, "status", path)?;
                if descriptor.get("status").and_then(Value::as_str) != Some(status) {
                    return Err(SemanticAssetError::Verification(format!(
                        "{} terrain/environment statuses disagree",
                        entry.instance_id
                    )));
                }
                Some((path.to_owned(), normalize_blake3(hash)?, document))
            }
            (None, None, None) => None,
            _ => {
                return Err(SemanticAssetError::InvalidPlan(format!(
                    "{} environment path/hash/terrain reference must be supplied together",
                    entry.instance_id
                )));
            }
        };

        let scene_instance = match (
            entry.scene_instance_document.as_deref(),
            entry.scene_instance_document_blake3.as_deref(),
        ) {
            (Some(path), Some(hash)) => {
                insert_native_terrain_batch_file(
                    &mut files,
                    &source_root,
                    &destination_root,
                    path,
                    AssetKind::Data,
                    hash,
                )?;
                Some((
                    path.to_owned(),
                    normalize_blake3(hash)?,
                    read_json(&source_root.join(native_path(path)))?,
                ))
            }
            (None, None) if entry.placement_status == "blocked" => None,
            (None, None) => {
                return Err(SemanticAssetError::InvalidPlan(format!(
                    "{} linked placement has no sceneInstanceDocument",
                    entry.instance_id
                )));
            }
            _ => {
                return Err(SemanticAssetError::InvalidPlan(format!(
                    "{} scene instance path/hash must be supplied together",
                    entry.instance_id
                )));
            }
        };
        for payload in &entry.payloads {
            if payload.role.trim().is_empty()
                || !matches!(payload.kind, AssetKind::Data | AssetKind::Texture)
            {
                return Err(SemanticAssetError::InvalidPlan(format!(
                    "{} payload must have a role and data/texture kind",
                    entry.instance_id
                )));
            }
            insert_native_terrain_batch_file(
                &mut files,
                &source_root,
                &destination_root,
                &payload.path,
                payload.kind,
                &payload.blake3,
            )?;
        }
        let generated = build_native_terrain_world_documents(
            entry,
            &destination_root,
            &terrain,
            scene_instance.as_ref(),
            environment.as_ref(),
        )?;
        for (path, kind, value) in generated.files {
            insert_generated_native_terrain_batch_file(&mut files, path, kind, &value)?;
        }
        catalog_entries.push(generated.catalog_entry);
    }
    let catalog = serde_json::json!({
        "schema": WORLD_CATALOG_SCHEMA,
        "sourceBuild": "retrobution-20260613",
        "status": "native-heightmap-batch-with-typed-placement-blockers",
        "terrainGlbAllowed": false,
        "entries": catalog_entries,
        "blocked": plan.blocked,
    });
    insert_generated_native_terrain_batch_file(
        &mut files,
        "world/catalog.json".to_owned(),
        AssetKind::Data,
        &catalog,
    )?;
    Ok((files, destination_roots))
}

pub(super) struct GeneratedNativeTerrainWorldDocuments {
    pub(super) files: Vec<(String, AssetKind, Value)>,
    pub(super) catalog_entry: Value,
}

pub(super) fn build_native_terrain_world_documents(
    entry: &NativeTerrainPublicationEntry,
    destination_root: &str,
    terrain: &Value,
    scene_instance: Option<&(String, String, Value)>,
    environment: Option<&(String, String, Value)>,
) -> Result<GeneratedNativeTerrainWorldDocuments> {
    let terrain_object = terrain.as_object().ok_or_else(|| {
        SemanticAssetError::Verification(format!(
            "{} terrain descriptor is not an object",
            entry.instance_id
        ))
    })?;
    let true_name = required_str(terrain_object, "trueName", &entry.terrain_document)?;
    let terrain_source = terrain_object
        .get("source")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            SemanticAssetError::Verification(format!(
                "{} terrain descriptor has no source",
                entry.instance_id
            ))
        })?;
    let terrain_data_path_id =
        required_i64(terrain_source, "terrainDataPathId", &entry.terrain_document)?;
    let source_asset = required_str(terrain_source, "assetName", &entry.terrain_document)?;
    let terrain_destination = format!("{destination_root}/{}", entry.terrain_document);
    let environment_reference = environment
        .map(|(relative, hash, document)| {
            let object = document.as_object().ok_or_else(|| {
                SemanticAssetError::Verification(format!(
                    "{} environment document is not an object",
                    entry.instance_id
                ))
            })?;
            Ok(serde_json::json!({
                "schema": required_str(object, "schema", relative)?,
                "status": required_str(object, "status", relative)?,
                "path": format!("{destination_root}/{relative}"),
                "blake3": hash,
            }))
        })
        .transpose()?;
    let (tile_x, tile_z) = parse_native_terrain_instance_coordinates(&entry.instance_id)?;
    let (scene_destination, provenance_destination) = match entry.scope.as_str() {
        "worldMap" => {
            let map_root = destination_root.strip_suffix("/terrain").ok_or_else(|| {
                SemanticAssetError::InvalidPlan(format!(
                    "{} worldMap destination does not end in /terrain",
                    entry.instance_id
                ))
            })?;
            (
                format!("{map_root}/scene.json"),
                format!("{map_root}/provenance.json"),
            )
        }
        "tutorial" => (
            format!("{destination_root}/scene.json"),
            format!("{destination_root}/provenance.json"),
        ),
        _ => {
            return Err(SemanticAssetError::InvalidPlan(format!(
                "unsupported native terrain scope {}",
                entry.scope
            )));
        }
    };
    let mut files = Vec::new();
    let mut scene_path = None;
    let mut scene_blake3 = None;
    let placement_evidence = if entry.placement_status == "linked" {
        let (scene_relative, scene_hash, scene_instance) = scene_instance.ok_or_else(|| {
            SemanticAssetError::InvalidPlan(format!(
                "{} linked placement has no scene instance",
                entry.instance_id
            ))
        })?;
        let scene_object = scene_instance.as_object().ok_or_else(|| {
            SemanticAssetError::Verification("scene-instance root is not an object".to_owned())
        })?;
        if required_str(scene_object, "schema", scene_relative)?
            != "ffone.native-terrain-scene-instance.v1"
        {
            return Err(SemanticAssetError::Verification(format!(
                "{} has the wrong scene-instance schema",
                entry.instance_id
            )));
        }
        let scene_tile = required_str(scene_object, "tileId", scene_relative)?;
        let expected_tile = format!("{tile_x:02}_{tile_z:02}");
        if scene_tile != expected_tile {
            return Err(SemanticAssetError::Verification(format!(
                "{} scene-instance tileId {scene_tile} differs from {expected_tile}",
                entry.instance_id
            )));
        }
        let scene_terrain = scene_object
            .get("terrainData")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                SemanticAssetError::Verification("scene-instance has no terrainData".to_owned())
            })?;
        if required_str(scene_terrain, "trueName", scene_relative)? != true_name
            || required_i64(scene_terrain, "sourcePathId", scene_relative)? != terrain_data_path_id
        {
            return Err(SemanticAssetError::Verification(format!(
                "{} scene-instance links a different TerrainData",
                entry.instance_id
            )));
        }
        let linkage = scene_object
            .get("linkage")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                SemanticAssetError::Verification("scene-instance has no linkage".to_owned())
            })?;
        let collider = linkage
            .get("terrainCollider")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                SemanticAssetError::Verification("scene-instance has no terrainCollider".to_owned())
            })?;
        let game_object = linkage
            .get("gameObject")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                SemanticAssetError::Verification("scene-instance has no gameObject".to_owned())
            })?;
        let transform = linkage
            .get("transform")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                SemanticAssetError::Verification("scene-instance has no transform".to_owned())
            })?;
        let terrain_component = linkage
            .get("terrainComponent")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                SemanticAssetError::Verification(
                    "scene-instance has no exact Terrain component".to_owned(),
                )
            })?;
        let root_chain = scene_object
            .get("rootChain")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                SemanticAssetError::Verification("scene-instance has no rootChain".to_owned())
            })?;
        let nodes = root_chain
            .get("nodes")
            .and_then(Value::as_array)
            .ok_or_else(|| SemanticAssetError::Verification("rootChain has no nodes".to_owned()))?;
        let canonical_nodes =
            serde_json::to_vec(nodes).map_err(|source| SemanticAssetError::Json {
                path: PathBuf::from(scene_relative),
                source,
            })?;
        let root_chain_hash = format!("blake3:{}", blake3::hash(&canonical_nodes).to_hex());
        if required_str(root_chain, "order", scene_relative)? != "immediateParentToRoot"
            || root_chain
                .get("includesOwnerTransform")
                .and_then(Value::as_bool)
                != Some(false)
            || required_str(root_chain, "canonicalJsonBlake3", scene_relative)? != root_chain_hash
            || required_str(scene_object, "rootChainBlake3", scene_relative)? != root_chain_hash
        {
            return Err(SemanticAssetError::Verification(format!(
                "{} rootChain order/hash contract is invalid",
                entry.instance_id
            )));
        }
        let map_root_id = required_i64(root_chain, "mapTileRootTransformPathId", scene_relative)?;
        if required_i64(scene_object, "mapTileRootTransformPathId", scene_relative)? != map_root_id
        {
            return Err(SemanticAssetError::Verification(format!(
                "{} root transform ids disagree",
                entry.instance_id
            )));
        }
        let order = root_chain
            .get("compositionOrderTransformPathIds")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                SemanticAssetError::Verification(
                    "rootChain has no composition order ids".to_owned(),
                )
            })?;
        let owner_parent_transform_path_id = transform
            .get("parentTransformPathId")
            .and_then(Value::as_i64);
        let root_terminal_is_valid = if nodes.is_empty() {
            owner_parent_transform_path_id.is_none()
                && map_root_id == required_i64(transform, "pathId", scene_relative)?
        } else {
            order.last().and_then(Value::as_i64) == Some(map_root_id)
                && order.first().and_then(Value::as_i64) == owner_parent_transform_path_id
        };
        if order.len() != nodes.len()
            || order.iter().zip(nodes).any(|(id, node)| {
                id.as_i64() != node.get("transformPathId").and_then(Value::as_i64)
            })
            || !root_terminal_is_valid
        {
            return Err(SemanticAssetError::Verification(format!(
                "{} rootChain node order is not owner-parent-to-root",
                entry.instance_id
            )));
        }
        let local_transform = scene_object.get("localTransform").cloned().ok_or_else(|| {
            SemanticAssetError::Verification("scene-instance has no localTransform".to_owned())
        })?;
        let map_scene = scene_object
            .get("mapScene")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                SemanticAssetError::Verification("scene-instance has no mapScene".to_owned())
            })?;
        let map_scene_path = required_str(map_scene, "path", scene_relative)?;
        let map_scene_hash = normalize_blake3(required_str(map_scene, "blake3", scene_relative)?)?;
        let game_object_name = required_str(game_object, "trueName", scene_relative)?;
        let display_name = native_terrain_display_name(game_object_name, true_name);
        let parent_transform_path_id = owner_parent_transform_path_id;
        let scene = serde_json::json!({
            "schema": NATIVE_WORLD_SCENE_SCHEMA,
            "name": entry.instance_id,
            "scope": entry.scope,
            "tile": [tile_x, tile_z],
            "coverage": "native-heightmap",
            "coordinateContract": {
                "schema": "ffone.native-coordinate-contract.v1",
                "space": "native",
                "basis": "H=diag(-1,1,1)",
                "unitScale": "1-unity-unit-equals-1-bevy-unit",
                "originPolicy": "source-trs-unchanged-no-auto-centering",
                "gameplayFacingRotationApplied": false
            },
            "provenance": {
                "sourceBuild": "retrobution-20260613",
                "sourceArchive": map_scene_path,
                "sourceArchiveBlake3": map_scene_hash,
                "sourceAsset": source_asset,
                "rootTransformPathId": map_root_id
            },
            "root": {
                "translation": [0.0, 0.0, 0.0],
                "rotation": [0.0, 0.0, 0.0, 1.0],
                "scale": [1.0, 1.0, 1.0]
            },
            "models": [],
            "visuals": [],
            "colliders": [],
            "nativeTerrain": {
                "name": display_name,
                "path": terrain_destination,
                "blake3": normalize_blake3(&entry.terrain_document_blake3)?,
                "trueName": true_name,
                "sourceGameObjectTrueName": game_object_name,
                "terrainDataPathId": terrain_data_path_id,
                "terrainColliderPathId": required_i64(collider, "pathId", scene_relative)?,
                "terrainGameObjectPathId": required_i64(game_object, "pathId", scene_relative)?,
                "terrainTransformPathId": required_i64(transform, "pathId", scene_relative)?,
                "parentTransformPathId": parent_transform_path_id,
                "terrainComponentPathId": required_i64(terrain_component, "pathId", scene_relative)?,
                "terrainRenderContract": terrain_component.get("renderContract").cloned().ok_or_else(|| {
                    SemanticAssetError::Verification("Terrain component has no renderContract".to_owned())
                })?,
                "environment": environment_reference,
                "sceneInstancePath": format!("{destination_root}/{scene_relative}"),
                "sceneInstanceBlake3": scene_hash,
                "transform": local_transform,
                "rootChain": Value::Object(root_chain.clone())
            }
        });
        let mut scene_bytes =
            serde_json::to_vec_pretty(&scene).map_err(|source| SemanticAssetError::Json {
                path: PathBuf::from(&scene_destination),
                source,
            })?;
        scene_bytes.push(b'\n');
        scene_blake3 = Some(blake3::hash(&scene_bytes).to_hex().to_string());
        files.push((scene_destination.clone(), AssetKind::Data, scene));
        scene_path = Some(scene_destination.clone());
        serde_json::json!({
            "status": "linked",
            "sceneInstance": format!("{destination_root}/{scene_relative}"),
            "sceneInstanceBlake3": scene_hash,
            "rootChainBlake3": root_chain_hash,
            "mapTileRootTransformPathId": map_root_id
        })
    } else {
        serde_json::json!({
            "status": "blocked",
            "reason": "scene owner/root placement is not proven; TerrainData payload remains published"
        })
    };
    let provenance = serde_json::json!({
        "schema": "ffone.semantic-native-terrain-provenance.v1",
        "sourceBuild": "retrobution-20260613",
        "scope": entry.scope,
        "instanceId": entry.instance_id,
        "trueTerrainDataName": true_name,
        "terrainDataPathId": terrain_data_path_id,
        "terrainDescriptor": terrain_destination,
        "terrainDescriptorBlake3": normalize_blake3(&entry.terrain_document_blake3)?,
        "environment": environment_reference,
        "placement": placement_evidence,
        "publicationPolicy": {
            "terrainRuntimeFormat": "Gray16 heightmap plus exact native sidecars",
            "terrainGlb": "forbidden",
            "autoCentered": false,
            "autoScaled": false,
            "unknownPlacementIsNotInvented": true
        }
    });
    files.push((provenance_destination.clone(), AssetKind::Data, provenance));
    Ok(GeneratedNativeTerrainWorldDocuments {
        files,
        catalog_entry: serde_json::json!({
            "scope": entry.scope,
            "instanceId": entry.instance_id,
            "tile": [tile_x, tile_z],
            "placementStatus": entry.placement_status,
            "scene": scene_path,
            "sceneBlake3": scene_blake3,
            "terrainDescriptor": terrain_destination,
            "terrainDescriptorBlake3": normalize_blake3(&entry.terrain_document_blake3)?,
            "environment": environment_reference,
            "provenance": provenance_destination
        }),
    })
}

pub(super) fn native_terrain_display_name<'a>(
    game_object_true_name: &'a str,
    terrain_true_name: &'a str,
) -> &'a str {
    if game_object_true_name.is_empty() {
        terrain_true_name
    } else {
        game_object_true_name
    }
}

pub(super) fn parse_native_terrain_instance_coordinates(instance_id: &str) -> Result<(i32, i32)> {
    let suffix = instance_id
        .strip_prefix("map_")
        .or_else(|| instance_id.strip_prefix("tile_"))
        .unwrap_or(instance_id);
    let (x, z) = suffix.split_once('_').ok_or_else(|| {
        SemanticAssetError::InvalidPlan(format!("invalid native terrain instanceId {instance_id}"))
    })?;
    if x.len() != 2
        || z.len() != 2
        || !x.bytes().all(|byte| byte.is_ascii_digit())
        || !z.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(SemanticAssetError::InvalidPlan(format!(
            "invalid native terrain instanceId {instance_id}"
        )));
    }
    Ok((
        x.parse().map_err(|_| {
            SemanticAssetError::InvalidPlan(format!("invalid tile X in {instance_id}"))
        })?,
        z.parse().map_err(|_| {
            SemanticAssetError::InvalidPlan(format!("invalid tile Z in {instance_id}"))
        })?,
    ))
}

pub(super) fn insert_generated_native_terrain_batch_file(
    files: &mut BTreeMap<String, NativeTerrainBatchFile>,
    destination_path: String,
    kind: AssetKind,
    value: &Value,
) -> Result<()> {
    validate_relative_path(&destination_path)?;
    let mut bytes =
        serde_json::to_vec_pretty(value).map_err(|source| SemanticAssetError::Json {
            path: PathBuf::from(&destination_path),
            source,
        })?;
    bytes.push(b'\n');
    let hash = blake3::hash(&bytes).to_hex().to_string();
    match files.get(&destination_path) {
        Some(existing) if existing.expected_blake3 == hash && existing.kind == kind => Ok(()),
        Some(_) => Err(SemanticAssetError::InvalidPlan(format!(
            "conflicting generated native terrain path {destination_path}"
        ))),
        None => {
            files.insert(
                destination_path.clone(),
                NativeTerrainBatchFile {
                    source: NativeTerrainBatchFileSource::Generated(bytes),
                    destination_path,
                    kind,
                    expected_blake3: hash,
                },
            );
            Ok(())
        }
    }
}
