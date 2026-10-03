use super::*;

pub(super) fn io_at(path: impl Into<PathBuf>, source: std::io::Error) -> SemanticAssetError {
    SemanticAssetError::Io {
        path: path.into(),
        source,
    }
}

/// Refuses legacy world publishers once the runtime-world registry has been created.
///
/// The guard treats any unreadable, malformed, or unexpected registry as an error too. Falling
/// back to the legacy publisher in that state could restore conversion-only catalogs and
/// provenance files into a migrated asset tree.
pub fn ensure_legacy_world_publication_allowed(asset_root: &Path, command: &str) -> Result<()> {
    let registry_path = asset_root.join("_runtime/world.json");
    let bytes = match fs::read(&registry_path) {
        Ok(bytes) => bytes,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(source) => return Err(io_at(&registry_path, source)),
    };
    let registry: Value =
        serde_json::from_slice(&bytes).map_err(|source| SemanticAssetError::Json {
            path: registry_path.clone(),
            source,
        })?;
    let schema = registry.get("schema").and_then(Value::as_str).ok_or_else(|| {
        SemanticAssetError::Verification(format!(
            "runtime world registry {} has no string schema; refusing legacy `{command}` publication; use the registry-aware world workflow in FusionForge",
            registry_path.display()
        ))
    })?;
    if schema != RUNTIME_WORLD_REGISTRY_SCHEMA {
        return Err(SemanticAssetError::Verification(format!(
            "runtime world registry {} has unsupported schema {schema:?}; refusing legacy `{command}` publication; use the registry-aware world workflow in FusionForge",
            registry_path.display()
        )));
    }

    Err(SemanticAssetError::InvalidPlan(format!(
        "legacy `{command}` publication is disabled because runtime world registry {} uses schema `{RUNTIME_WORLD_REGISTRY_SCHEMA}`; use the registry-aware world workflow in FusionForge",
        registry_path.display()
    )))
}

/// Build an idempotent publication plan from the canonical native heightmap
/// tree. Terrain glTF is intentionally not a supported route.
pub fn build_retro_world_plan(asset_root: &Path) -> Result<RoutePlan> {
    let source_build = "retrobution-20260613";
    let mut routes = Vec::new();
    let mut map_catalog = Vec::new();

    for (legacy_tile, slug) in [("Map_08_06", "map_08_06"), ("Map_12_03", "map_12_03")] {
        let scene_path = format!("world/maps/{slug}/scene.json");
        let scene_disk_path = asset_root.join(native_path(&scene_path));
        let scene = read_json(&scene_disk_path)?;
        let provenance = scene
            .get("provenance")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                SemanticAssetError::Verification(format!("{scene_path} has no provenance object"))
            })?;
        let terrain_instance = scene
            .get("nativeTerrain")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                SemanticAssetError::Verification(format!(
                    "{scene_path} has no nativeTerrain object"
                ))
            })?;
        if scene
            .get("models")
            .and_then(Value::as_array)
            .is_none_or(|models| !models.is_empty())
            || scene
                .get("visuals")
                .and_then(Value::as_array)
                .is_none_or(|visuals| !visuals.is_empty())
            || scene
                .get("colliders")
                .and_then(Value::as_array)
                .is_none_or(|colliders| !colliders.is_empty())
        {
            return Err(SemanticAssetError::Verification(format!(
                "{scene_path} still contains a model/visual/triangle-mesh terrain route"
            )));
        }
        let terrain_path = required_str(terrain_instance, "path", &scene_path)?.to_owned();
        if !terrain_path.starts_with(&format!("world/maps/{slug}/terrain/"))
            || terrain_path.to_ascii_lowercase().ends_with(".glb")
        {
            return Err(SemanticAssetError::Verification(format!(
                "{scene_path} native terrain path is not canonical JSON: {terrain_path}"
            )));
        }
        let terrain_disk_path = asset_root.join(native_path(&terrain_path));
        let terrain = read_json(&terrain_disk_path)?;
        let terrain_object = terrain.as_object().ok_or_else(|| {
            SemanticAssetError::Verification(format!("{terrain_path} is not a JSON object"))
        })?;
        let descriptor_hash = hash_file(&terrain_disk_path)?.1;
        if required_str(terrain_instance, "blake3", &scene_path)? != descriptor_hash {
            return Err(SemanticAssetError::Verification(format!(
                "{scene_path} nativeTerrain hash does not match {terrain_path}"
            )));
        }
        let true_name = required_str(terrain_object, "trueName", &terrain_path)?.to_owned();
        if required_str(terrain_instance, "trueName", &scene_path)? != true_name {
            return Err(SemanticAssetError::Verification(format!(
                "{scene_path} and {terrain_path} disagree on trueName"
            )));
        }
        let terrain_source = terrain_object
            .get("source")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                SemanticAssetError::Verification(format!("{terrain_path} has no source"))
            })?;
        let source_asset = required_str(terrain_source, "assetName", &terrain_path)?.to_owned();
        let source_archive = required_str(provenance, "sourceArchive", &scene_path)?.to_owned();
        let root_transform_id = required_i64(provenance, "rootTransformPathId", &scene_path)?;
        let terrain_data_id = required_i64(terrain_source, "terrainDataPathId", &terrain_path)?;
        let mut object_ids = BTreeMap::from([
            ("rootTransformPathId".to_owned(), root_transform_id),
            ("terrainDataPathId".to_owned(), terrain_data_id),
        ]);
        for key in [
            "terrainColliderPathId",
            "terrainGameObjectPathId",
            "terrainTransformPathId",
            "parentRootTransformPathId",
        ] {
            object_ids.insert(
                key.to_owned(),
                required_i64(terrain_instance, key, &scene_path)?,
            );
        }
        let ownership = OwnershipProof {
            true_legacy_name: true_name.clone(),
            authority: format!(
                "{scene_path} nativeTerrain hierarchy + {terrain_path} TerrainData pathID {terrain_data_id}"
            ),
            source_build: source_build.to_owned(),
            source_archive,
            source_asset,
            source_object_ids: object_ids,
        };

        routes.push(SemanticRoute {
            source_path: scene_path.clone(),
            destination_path: scene_path.clone(),
            kind: AssetKind::Data,
            category: SemanticCategory::World,
            ownership: ownership.clone(),
            duplicate_group: None,
            variant_reason: None,
            content: RouteContent::CopyExact,
        });

        let terrain_root = terrain_path
            .strip_suffix("terrain.json")
            .ok_or_else(|| {
                SemanticAssetError::Verification(format!(
                    "{terrain_path} does not end in terrain.json"
                ))
            })?
            .to_owned();
        let mut payloads = BTreeMap::from([(terrain_path.clone(), AssetKind::Data)]);
        let heightmap = terrain_object
            .get("heightmap")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                SemanticAssetError::Verification(format!("{terrain_path} has no heightmap"))
            })?;
        let heightmap_relative = required_str(heightmap, "path", &terrain_path)?;
        let heightmap_path = format!("{terrain_root}{heightmap_relative}");
        verify_prefixed_blake3(
            &asset_root.join(native_path(&heightmap_path)),
            required_str(heightmap, "pngBlake3", &terrain_path)?,
            &heightmap_path,
        )?;
        payloads.insert(heightmap_path, AssetKind::Texture);
        let splat = terrain_object
            .get("splat")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                SemanticAssetError::Verification(format!("{terrain_path} has no splat"))
            })?;
        let weight_maps = splat
            .get("weightMaps")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                SemanticAssetError::Verification(format!("{terrain_path} has no weightMaps"))
            })?;
        for weight in weight_maps {
            let weight = weight.as_object().ok_or_else(|| {
                SemanticAssetError::Verification("weight map is not an object".to_owned())
            })?;
            let weight_path = format!(
                "{terrain_root}{}",
                required_str(weight, "path", &terrain_path)?
            );
            verify_prefixed_blake3(
                &asset_root.join(native_path(&weight_path)),
                required_str(weight, "pngBlake3", &terrain_path)?,
                &weight_path,
            )?;
            payloads.insert(weight_path, AssetKind::Texture);
        }
        let layers = splat
            .get("layers")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                SemanticAssetError::Verification(format!("{terrain_path} has no layers"))
            })?;
        for layer in layers {
            let albedo = layer
                .get("albedo")
                .and_then(Value::as_object)
                .ok_or_else(|| {
                    SemanticAssetError::Verification("terrain layer has no albedo".to_owned())
                })?;
            let albedo_path = format!(
                "{terrain_root}{}",
                required_str(albedo, "path", &terrain_path)?
            );
            verify_prefixed_blake3(
                &asset_root.join(native_path(&albedo_path)),
                required_str(albedo, "pngBlake3", &terrain_path)?,
                &albedo_path,
            )?;
            payloads.insert(albedo_path, AssetKind::Texture);
        }
        for (payload, kind) in &payloads {
            if payload.to_ascii_lowercase().ends_with(".glb") {
                return Err(SemanticAssetError::Verification(format!(
                    "native terrain plan attempted to retain glTF payload {payload}"
                )));
            }
            routes.push(SemanticRoute {
                source_path: payload.clone(),
                destination_path: payload.clone(),
                kind: *kind,
                category: SemanticCategory::World,
                ownership: ownership.clone(),
                duplicate_group: None,
                variant_reason: None,
                content: RouteContent::CopyExact,
            });
        }

        let coordinate_contract = scene.get("coordinateContract").cloned().ok_or_else(|| {
            SemanticAssetError::Verification(format!("{scene_path} has no coordinateContract"))
        })?;
        let root = scene
            .get("root")
            .cloned()
            .ok_or_else(|| SemanticAssetError::Verification(format!("{scene_path} has no root")))?;
        let source_scene_hash = hash_file(&scene_disk_path)?.1;
        let provenance_destination = format!("world/maps/{slug}/provenance.json");
        let provenance_json = serde_json::json!({
            "schema": "ffone.semantic-world-heightmap-provenance.v1",
            "sourceBuild": source_build,
            "legacyTileName": legacy_tile,
            "trueTerrainDataName": true_name,
            "scene": scene_path,
            "sceneBlake3": source_scene_hash,
            "terrainDescriptor": terrain_path,
            "terrainDescriptorBlake3": descriptor_hash,
            "heightmap": heightmap,
            "nativeTerrainInstance": Value::Object(terrain_instance.clone()),
            "coordinateContract": coordinate_contract,
            "rootTransform": root,
            "serializedObjectProvenance": {
                "rootTransformPathId": root_transform_id,
                "terrainDataPathId": terrain_data_id,
                "terrainColliderPathId": required_i64(terrain_instance, "terrainColliderPathId", &scene_path)?,
                "terrainGameObjectPathId": required_i64(terrain_instance, "terrainGameObjectPathId", &scene_path)?,
                "terrainTransformPathId": required_i64(terrain_instance, "terrainTransformPathId", &scene_path)?
            },
            "publicationPolicy": {
                "terrainRuntimeFormat": "Gray16 heightmap + linear RGBA8 weights + semantic sRGB PNG layers",
                "terrainGlb": "forbidden",
                "sourceAliasesRetainedInRuntimeAssets": false,
                "heightmapOwnerTranslationBaked": false,
                "autoCentered": false,
                "autoScaled": false
            }
        });
        routes.push(SemanticRoute {
            source_path: format!("semantic-generated/{provenance_destination}"),
            destination_path: provenance_destination.clone(),
            kind: AssetKind::Data,
            category: SemanticCategory::World,
            ownership: ownership.clone(),
            duplicate_group: None,
            variant_reason: None,
            content: RouteContent::GeneratedJson {
                value: provenance_json,
            },
        });

        map_catalog.push(serde_json::json!({
            "id": slug,
            "legacyTileName": legacy_tile,
            "trueTerrainDataName": true_name,
            "scene": scene_path,
            "terrainDescriptor": terrain_path,
            "provenance": provenance_destination,
            "coordinateContract": coordinate_contract,
            "rootTransform": root,
            "nativeTerrainInstance": Value::Object(terrain_instance.clone()),
            "sceneBlake3": source_scene_hash,
            "terrainDescriptorBlake3": descriptor_hash,
            "payloadFiles": payloads.keys().collect::<Vec<_>>()
        }));
    }

    let catalog_path = "world/catalog.json";
    routes.push(SemanticRoute {
        source_path: format!("semantic-generated/{catalog_path}"),
        destination_path: catalog_path.to_owned(),
        kind: AssetKind::Data,
        category: SemanticCategory::World,
        ownership: OwnershipProof {
            true_legacy_name: "FusionFall native heightmap world catalog".to_owned(),
            authority:
                "canonical world/maps/map_08_06 and map_12_03 scene + TerrainData provenance"
                    .to_owned(),
            source_build: source_build.to_owned(),
            source_archive: "Map_08_06 + Map_12_03 + DongResources".to_owned(),
            source_asset: "TerrainData_08_06 + TerrainData_12_03".to_owned(),
            source_object_ids: BTreeMap::new(),
        },
        duplicate_group: None,
        variant_reason: None,
        content: RouteContent::GeneratedJson {
            value: serde_json::json!({
                "schema": WORLD_CATALOG_SCHEMA,
                "sourceBuild": source_build,
                "status": "ownership-proven-native-heightmap-terrain",
                "sourceAliasesRetainedInRuntimeAssets": false,
                "runtimeMigrationPending": false,
                "terrainGlbAllowed": false,
                "maps": map_catalog
            }),
        },
    });

    Ok(RoutePlan {
        schema: ROUTE_PLAN_SCHEMA.to_owned(),
        source_build: source_build.to_owned(),
        policy: "native heightmap publication; Gray16 heights; linear RGBA weights; semantic PNG layers; no terrain glTF or legacy runtime alias".to_owned(),
        routes,
    })
}

pub(super) fn normalize_blake3(value: &str) -> Result<String> {
    let value = value.strip_prefix("blake3:").unwrap_or(value);
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(SemanticAssetError::InvalidPlan(format!(
            "invalid lowercase BLAKE3 hash {value:?}"
        )));
    }
    Ok(value.to_owned())
}

pub(super) fn restore_quarantined(paths: &[(PathBuf, PathBuf)]) {
    for (original, quarantine) in paths.iter().rev() {
        if !quarantine.exists() {
            continue;
        }
        if let Some(parent) = original.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::rename(quarantine, original);
    }
}

pub(super) fn rollback_replaced(paths: &[(PathBuf, Option<PathBuf>)]) {
    for (destination, backup) in paths.iter().rev() {
        let _ = fs::remove_file(destination);
        if let Some(backup) = backup {
            if let Some(parent) = destination.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::rename(backup, destination);
        }
    }
}

pub(super) fn remove_empty_legacy_world_directories(asset_root: &Path) {
    for relative in [
        "world/maps/map_08_06/terrain/model",
        "world/maps/map_12_03/terrain/model",
        "models/world/Map_08_06",
        "models/world/Map_12_03",
        "worlds",
    ] {
        let _ = fs::remove_dir(asset_root.join(native_path(relative)));
    }
}

pub(super) fn json_array_len(value: &Value, key: &str) -> u64 {
    value
        .get(key)
        .and_then(Value::as_array)
        .map_or(0, |values| values.len() as u64)
}

pub(super) fn hash_file(path: &Path) -> Result<(u64, String)> {
    let mut file = File::open(path).map_err(|source| io_at(path, source))?;
    let mut hasher = blake3::Hasher::new();
    let mut bytes = 0u64;
    let mut buffer = [0u8; 128 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|source| io_at(path, source))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        bytes += read as u64;
    }
    Ok((bytes, hasher.finalize().to_hex().to_string()))
}

pub(super) fn verify_prefixed_blake3(path: &Path, expected: &str, context: &str) -> Result<()> {
    let expected = expected.strip_prefix("blake3:").ok_or_else(|| {
        SemanticAssetError::Verification(format!(
            "{context} expected hash must use the blake3: prefix"
        ))
    })?;
    let (_, actual) = hash_file(path)?;
    if actual != expected {
        return Err(SemanticAssetError::Verification(format!(
            "{context} PNG hash mismatch: expected blake3:{expected}, got blake3:{actual}"
        )));
    }
    Ok(())
}

pub(super) fn rewrite_json_strings(value: &mut Value, replacements: &BTreeMap<String, String>) {
    match value {
        Value::String(text) => {
            if let Some(replacement) = replacements.get(text) {
                *text = replacement.clone();
            }
        }
        Value::Array(values) => {
            for value in values {
                rewrite_json_strings(value, replacements);
            }
        }
        Value::Object(object) => {
            for value in object.values_mut() {
                rewrite_json_strings(value, replacements);
            }
        }
        _ => {}
    }
}

pub(super) fn rollback_created(paths: &[PathBuf]) {
    for path in paths.iter().rev() {
        let _ = fs::remove_file(path);
    }
}

pub(super) fn unique_stamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos())
}

pub(super) fn has_content_hash_filename(path: &str) -> bool {
    let file = path.rsplit('/').next().unwrap_or(path);
    let stem = file.rsplit_once('.').map_or(file, |(stem, _)| stem);
    let Some((_, suffix)) = stem.rsplit_once("--") else {
        return false;
    };
    suffix.len() >= 16 && suffix.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub(super) fn remove_content_hash(path: &str) -> String {
    let (prefix, file) = path
        .rsplit_once('/')
        .map_or(("", path), |(prefix, file)| (prefix, file));
    let (stem, extension) = file
        .rsplit_once('.')
        .map_or((file, ""), |(stem, extension)| (stem, extension));
    let readable_stem = stem.rsplit_once("--").map_or(stem, |(name, _)| name);
    let file = if extension.is_empty() {
        readable_stem.to_owned()
    } else {
        format!("{readable_stem}.{extension}")
    };
    if prefix.is_empty() {
        file
    } else {
        format!("{prefix}/{file}")
    }
}

pub(super) fn contains_variant_directory(path: &str) -> bool {
    let segments: Vec<_> = path.split('/').collect();
    segments.windows(2).any(|pair| {
        pair[0] == "variants"
            && pair[1].strip_prefix("variant_").is_some_and(|number| {
                number.len() == 2 && number.bytes().all(|b| b.is_ascii_digit())
            })
    })
}

pub(super) fn required_str<'a>(object: &'a Map<String, Value>, key: &str, context: &str) -> Result<&'a str> {
    object.get(key).and_then(Value::as_str).ok_or_else(|| {
        SemanticAssetError::Verification(format!("{context} field {key} is not a string"))
    })
}

pub(super) fn required_i64(object: &Map<String, Value>, key: &str, context: &str) -> Result<i64> {
    object.get(key).and_then(Value::as_i64).ok_or_else(|| {
        SemanticAssetError::Verification(format!("{context} field {key} is not an integer"))
    })
}
