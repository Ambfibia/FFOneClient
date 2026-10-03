use super::*;

pub(super) fn validate_descriptor(descriptor: &NativeTerrainDescriptor) -> Result<(), NativeTerrainError> {
    if descriptor.schema != NATIVE_TERRAIN_SCHEMA {
        return Err(NativeTerrainError::new(format!(
            "unsupported native terrain schema {:?}; expected {:?}",
            descriptor.schema, NATIVE_TERRAIN_SCHEMA
        )));
    }
    validate_readable_name(&descriptor.true_name, "terrain trueName")?;
    if let Some(source) = &descriptor.source {
        validate_readable_name(&source.asset_name, "terrain source assetName")?;
        validate_prefixed_blake3(&source.bundle.blake3, "source bundle")?;
        if source.terrain_data_path_id <= 0 {
            return Err(NativeTerrainError::new(
                "terrainDataPathId must be a positive serialized-object id",
            ));
        }
    }

    let dimensions = &descriptor.dimensions;
    if dimensions.width < 2
        || dimensions.height < 2
        || dimensions.sample_count
            != (dimensions.width as usize)
                .checked_mul(dimensions.height as usize)
                .ok_or_else(|| NativeTerrainError::new("terrain dimensions overflow"))?
    {
        return Err(NativeTerrainError::new(
            "terrain dimensions and sampleCount are inconsistent",
        ));
    }
    let scale = &descriptor.scale;
    for (value, label) in [
        (scale.sample_spacing_x, "sampleSpacingX"),
        (scale.sample_spacing_z, "sampleSpacingZ"),
        (scale.height_scale, "heightScale"),
        (scale.extent_x, "extentX"),
        (scale.extent_z, "extentZ"),
    ] {
        if !value.is_finite() || value <= 0.0 {
            return Err(NativeTerrainError::new(format!(
                "terrain {label} must be finite and positive"
            )));
        }
    }
    if scale.height_normalization_denominator != NATIVE_TERRAIN_HEIGHT_DENOMINATOR
        || !approximately_equal(
            scale.extent_x,
            f64::from(dimensions.width - 1) * scale.sample_spacing_x,
        )
        || !approximately_equal(
            scale.extent_z,
            f64::from(dimensions.height - 1) * scale.sample_spacing_z,
        )
    {
        return Err(NativeTerrainError::new(
            "terrain scale does not exactly cover its sample grid or uses the wrong height denominator",
        ));
    }

    let heightmap = &descriptor.heightmap;
    validate_relative_path(&heightmap.path, "png")?;
    if heightmap.path != "heightmap.png"
        || heightmap.png_color_type != "Gray16"
        || heightmap.raw_encoding != "u16"
        || heightmap.png_sample_byte_order
            != "big-endian per PNG; image decoders return host-endian u16"
        || heightmap.raw_hash_byte_order != "little-endian"
        || heightmap.pixel_order != "row-major: pixelIndex = row * width + column"
        || heightmap.raw_min > heightmap.raw_max
    {
        return Err(NativeTerrainError::new(
            "terrain heightmap encoding/orientation metadata differs from the native contract",
        ));
    }
    for (hash, label) in [
        (&heightmap.source_order_raw_blake3, "source-order height"),
        (
            &heightmap.canonical_order_raw_blake3,
            "canonical-order height",
        ),
        (&heightmap.png_blake3, "heightmap PNG"),
    ] {
        validate_prefixed_blake3(hash, label)?;
    }
    validate_vertex_shifts(descriptor)?;
    if let Some(attributes) = &descriptor.gameplay_attributes {
        validate_relative_path(&attributes.path, "png")?;
        if attributes.path != "gameplay/attributes.png"
            || attributes.raw_path != "gameplay/attributes.bin"
            || attributes.width != dimensions.width
            || attributes.height != dimensions.height
            || attributes.sample_count != dimensions.sample_count
            || attributes.png_color_type != "Gray8"
            || attributes.raw_encoding != "u8"
            || attributes.pixel_order != "row-major: index = row * width + column"
        {
            return Err(NativeTerrainError::new(
                "gameplay attribute grid differs from the exact MapAttributeTable contract",
            ));
        }
        validate_relative_path(&attributes.raw_path, "bin")?;
        validate_prefixed_blake3(&attributes.raw_blake3, "gameplay attribute raw bytes")?;
        validate_prefixed_blake3(&attributes.png_blake3, "gameplay attribute PNG")?;
        if let Some(document) = &attributes.raw_parsed_document {
            validate_relative_path(&document.path, "json")?;
            if document.path != "gameplay/attributes.raw.json" {
                return Err(NativeTerrainError::new(
                    "gameplay attribute parsed document has a non-semantic path",
                ));
            }
            validate_prefixed_blake3(&document.blake3, "gameplay attribute parsed document")?;
        }
        if let Some(source) = &attributes.source {
            if source.pointer_path_id <= 0
                || source.resolved_path_id <= 0
                || source.script_true_name != "MapAttributeTable"
                || source.decompiled_evidence.index_formula != "attributes[x + z * heightmapWidth]"
            {
                return Err(NativeTerrainError::new(
                    "gameplay attribute source evidence is contradictory",
                ));
            }
            validate_sha256(
                &source.decompiled_evidence.sha256,
                "MapAttributeTable decompiled evidence",
            )?;
        }
    }

    let orientation = &descriptor.orientation;
    if orientation.source_height_layout != "x-major"
        || orientation.source_height_index != "x * height + z"
        || orientation.canonical_height_layout != "row-major-z"
        || orientation.canonical_height_index != "z * width + x"
        || orientation.canonical_columns != "-nativeX"
        || orientation.canonical_rows != "+nativeZ"
        || orientation.weight_map_transform != "flipY"
        || orientation.runtime_requires_unity_orientation_fixup
    {
        return Err(NativeTerrainError::new(
            "terrain orientation contract permits a transpose, flip, or runtime Unity fixup",
        ));
    }
    validate_geometry_contract(descriptor)?;
    if let Some(scene_instance) = &descriptor.scene_instance {
        validate_scene_instance_contract(scene_instance)?;
    }
    validate_splat_contract(descriptor)?;
    if let Some(environment) = &descriptor.environment {
        validate_environment_reference(environment)?;
    }
    Ok(())
}

pub(super) fn validate_environment_reference(
    environment: &NativeTerrainEnvironmentReference,
) -> Result<(), NativeTerrainError> {
    if environment.schema != NATIVE_TERRAIN_ENVIRONMENT_SCHEMA
        || environment.path != "environment/environment.json"
        || !matches!(
            environment.status.as_str(),
            "complete" | "complete-with-placement-blocker" | "blocked"
        )
    {
        return Err(NativeTerrainError::new(
            "terrain environment reference differs from the native v1 contract",
        ));
    }
    validate_relative_path(&environment.path, "json")?;
    validate_prefixed_blake3(&environment.blake3, "terrain environment document")
}

pub(super) fn validate_environment(
    environment: &NativeTerrainEnvironment,
    reference: &NativeTerrainEnvironmentReference,
) -> Result<(), NativeTerrainError> {
    if environment.schema != NATIVE_TERRAIN_ENVIRONMENT_SCHEMA
        || environment.status != reference.status
        || !matches!(environment.scope.as_str(), "worldMap" | "tutorial")
        || environment.tile_id.len() != 5
        || environment.tile_id.as_bytes()[2] != b'_'
        || !environment
            .tile_id
            .bytes()
            .enumerate()
            .all(|(index, byte)| index == 2 || byte.is_ascii_digit())
    {
        return Err(NativeTerrainError::new(
            "terrain environment identity/status differs from its verified reference",
        ));
    }

    let ambience = &environment.ambience;
    if !matches!(
        ambience.status.as_str(),
        "exactSource" | "blockedCoordinateMismatch"
    ) || !ambience.fog_depth.is_finite()
        || ambience
            .fog_color
            .iter()
            .chain(&ambience.sky_color)
            .chain(&ambience.light_color)
            .any(|value| !value.is_finite())
    {
        return Err(NativeTerrainError::new(
            "terrain ambience contains an invalid source status or non-finite value",
        ));
    }

    let contract = &environment.runtime_ambience_contract;
    let application = &contract.default_ambience_application;
    let registration = &contract.registration;
    let sampling = &contract.sampling;
    let fallback = &sampling.zero_weight_fallback;
    if !approximately_equal(application.fog_alpha_after_scope_blend, 0.85)
        || application.fog_color != "sampledFogColor"
        || application.fog_density != "sampledFogDepth * 0.005"
        || application.fog_enabled != "sampledFogDepth > 0"
        || application.light_color != "sampledLightColor * 0.6 + white * 0.4"
        || application.source != "cnPlayerCamera.DefaultAmbience"
        || application.tutorial_fog_color != "sampledFogColor * 0.5 + white * 0.5"
        || application.tutorial_blend_applies_to_this_scope != (environment.scope == "tutorial")
        || registration.grid_index_formula != "x + 16 * y"
        || registration.grid_width != 16
        || registration.operation
            != "DongLoader.SetDongColors(x, y, fogDepth, fogColor, skyColor, lightColor)"
        || registration.source != "DongColorSetup.Start"
        || contract.runtime_policy
            != "evaluate continuously from player position; do not bake one interpolated value per tile"
        || sampling.fraction_remap != "clamp01((fraction - 0.25) * 2)"
        || sampling.local_coordinates.x != "playerPosition.x / 512 - 0.5"
        || sampling.local_coordinates.y != "playerPosition.z / 512 - 0.5"
        || sampling.missing_tile_policy != "a neighbor contributes only when HasDongColors is true"
        || sampling.neighbor_selection != "floor(localCoordinate) and +1 on each axis"
        || sampling.normalization
            != "divide accumulated fogDepth/fogColor/skyColor/lightColor by total contributing weight"
        || sampling.source != "DongLoader.GetDongColors"
        || sampling.weights
            != [
                "(1 - tx) * (1 - ty)",
                "tx * (1 - ty)",
                "(1 - tx) * ty",
                "tx * ty",
            ]
        || fallback.fog_depth != 0.0
        || fallback.fog_color != [0.0, 0.0, 0.0, 0.0]
        || fallback.sky_color != [1.0, 1.0, 1.0, 1.0]
        || fallback.light_color != [1.0, 1.0, 1.0, 1.0]
    {
        return Err(NativeTerrainError::new(
            "terrain environment runtime ambience formulas differ from the decompiled contract",
        ));
    }

    let detail = &environment.terrain_detail;
    let baseline = &detail.serialized_terrain_render_baseline;
    let fields = &baseline.serialized_fields;
    if detail.status != "exactSourceAndRuntimeBindings"
        || baseline.selection
            != "owner component whose m_TerrainData resolves to the exact routed TerrainData"
        || fields.cast_shadows != 1
        || fields.debug_draw_main_camera != 0
        || !fields.enabled
        || fields.heightmap_maximum_lod != 0
        || fields.render_mode != 1
        || !matches!(fields.use_lightmap, 0 | 1)
        || fields.tree_maximum_full_lod_count != 50
        || [
            fields.detail_object_distance,
            fields.heightmap_pixel_error,
            fields.splat_map_distance,
            fields.tree_billboard_distance,
            fields.tree_cross_fade_length,
            fields.tree_distance,
        ]
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0)
    {
        return Err(NativeTerrainError::new(
            "terrain serialized renderer baseline differs from the exact source contract",
        ));
    }
    Ok(())
}

pub(super) fn validate_geometry_contract(
    descriptor: &NativeTerrainDescriptor,
) -> Result<(), NativeTerrainError> {
    let geometry = &descriptor.native_geometry;
    let scale = &descriptor.scale;
    let heightmap = &descriptor.heightmap;
    let expected_min_y = f64::from(heightmap.raw_min)
        / f64::from(NATIVE_TERRAIN_HEIGHT_DENOMINATOR)
        * scale.height_scale;
    let expected_max_y = f64::from(heightmap.raw_max)
        / f64::from(NATIVE_TERRAIN_HEIGHT_DENOMINATOR)
        * scale.height_scale;
    let expected = [
        (
            geometry.local_origin,
            [0.0, 0.0, 0.0],
            "nativeGeometry.localOrigin",
        ),
        (
            geometry.column_step,
            [-scale.sample_spacing_x, 0.0, 0.0],
            "nativeGeometry.columnStep",
        ),
        (
            geometry.row_step,
            [0.0, 0.0, scale.sample_spacing_z],
            "nativeGeometry.rowStep",
        ),
        (
            geometry.height_axis,
            [0.0, 1.0, 0.0],
            "nativeGeometry.heightAxis",
        ),
        (
            geometry.local_bounds.min,
            [-scale.extent_x, expected_min_y, 0.0],
            "nativeGeometry.localBounds.min",
        ),
        (
            geometry.local_bounds.max,
            [0.0, expected_max_y, scale.extent_z],
            "nativeGeometry.localBounds.max",
        ),
    ];
    for (actual, wanted, label) in expected {
        if actual
            .iter()
            .zip(wanted)
            .any(|(actual, wanted)| !approximately_equal(*actual, wanted))
        {
            return Err(NativeTerrainError::new(format!(
                "{label} differs from the native heightmap geometry contract"
            )));
        }
    }
    if geometry.front_face != "counterClockwise"
        || geometry.cell_indices.i00 != "row * width + column"
        || geometry.cell_indices.i10 != "i00 + 1"
        || geometry.cell_indices.i01 != "i00 + width"
        || geometry.cell_indices.i11 != "i01 + 1"
        || geometry.cell_triangle_order != "[i00, i10, i01], [i10, i11, i01]"
        || geometry.geometric_front_normal != "+nativeY"
        || geometry.vertex_formula
            != "[-column * sampleSpacingX, rawU16 / 32767 * heightScale, row * sampleSpacingZ]"
    {
        return Err(NativeTerrainError::new(
            "native terrain vertex/index/front-face formula differs from the required Bevy contract",
        ));
    }
    Ok(())
}

pub(super) fn validate_scene_instance_contract(
    contract: &NativeTerrainSceneInstanceContract,
) -> Result<(), NativeTerrainError> {
    if contract.status != "notExportedFromTerrainDataBundle"
        || !contract.terrain_data_local_geometry_only
        || !contract.required_for_world_placement
        || contract.required_source
            != "map-scene TerrainCollider/Terrain owner whose m_TerrainData resolves to source.terrainDataPathId"
        || contract.application_order
            != "terrainDataLocalVertex -> scene owner transform -> map tile root transform"
        || contract.height_formula != "sceneOwnerLocalTranslationY + rawU16 / 32767 * heightScale"
        || !contract.translation_is_not_baked_into_heightmap
    {
        return Err(NativeTerrainError::new(
            "terrain scene-instance contract does not keep owner translation separate from height samples",
        ));
    }
    Ok(())
}

pub(super) fn validate_pointer_source(
    source: &NativeTerrainPointerSource,
    context: &str,
) -> Result<(), NativeTerrainError> {
    if source.pointer_file_id < 0
        || source.pointer_path_id <= 0
        || source.resolved_path_id <= 0
        || source.resolved_asset_name.is_empty()
    {
        return Err(NativeTerrainError::new(format!(
            "{context} has invalid serialized pointer provenance"
        )));
    }
    Ok(())
}

pub(super) fn validate_plain_blake3(value: &str, context: &str) -> Result<(), NativeTerrainError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(NativeTerrainError::new(format!(
            "{context} has an invalid lowercase BLAKE3 digest"
        )));
    }
    Ok(())
}

pub(super) fn validate_prefixed_blake3(value: &str, context: &str) -> Result<(), NativeTerrainError> {
    let Some(digest) = value.strip_prefix("blake3:") else {
        return Err(NativeTerrainError::new(format!(
            "{context} hash has no blake3: prefix"
        )));
    };
    validate_plain_blake3(digest, context)
}

pub(super) fn validate_sha256(value: &str, context: &str) -> Result<(), NativeTerrainError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(NativeTerrainError::new(format!(
            "{context} has an invalid lowercase SHA-256 digest"
        )));
    }
    Ok(())
}

pub(super) fn validate_readable_name(value: &str, context: &str) -> Result<(), NativeTerrainError> {
    if value.is_empty()
        || value.len() > 512
        || value.chars().any(char::is_control)
        || value.contains('/')
        || value.contains('\\')
    {
        return Err(NativeTerrainError::new(format!(
            "{context} is empty, unsafe, or contains a control character"
        )));
    }
    Ok(())
}

pub(super) fn validate_relative_path(value: &str, extension: &str) -> Result<(), NativeTerrainError> {
    if value.is_empty() || value.contains('\\') {
        return Err(NativeTerrainError::new(format!(
            "unsafe native terrain path {value:?}"
        )));
    }
    let path = Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        || path.extension().and_then(|value| value.to_str()) != Some(extension)
    {
        return Err(NativeTerrainError::new(format!(
            "unsafe native terrain path {value:?}"
        )));
    }
    Ok(())
}
