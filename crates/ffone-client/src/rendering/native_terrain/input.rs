use super::*;

pub(super) fn load_environment(
    terrain_root: &Path,
    reference: &NativeTerrainEnvironmentReference,
    authoritative_blake3: Option<&str>,
) -> Result<NativeTerrainEnvironment, NativeTerrainError> {
    let path = join_relative(terrain_root, &reference.path);
    let bytes = read_file(&path, "native terrain environment")?;
    if let Some(authoritative_blake3) = authoritative_blake3 {
        validate_plain_blake3(authoritative_blake3, "authoritative terrain environment")?;
        verify_plain_hash(&bytes, authoritative_blake3, &path)?;
    } else {
        verify_prefixed_hash(&bytes, &reference.blake3, &path)?;
    }
    let environment: NativeTerrainEnvironment =
        serde_json::from_slice(&bytes).map_err(|error| {
            NativeTerrainError::new(format!(
                "invalid native terrain environment JSON {}: {error}",
                path.display()
            ))
        })?;
    validate_environment(&environment, reference)?;
    Ok(environment)
}

pub(super) fn load_gameplay_attributes(
    terrain_root: &Path,
    metadata: &NativeTerrainGameplayAttributes,
) -> Result<VerifiedGameplayAttributes, NativeTerrainError> {
    let raw_path = join_relative(terrain_root, &metadata.raw_path);
    let raw = read_file(&raw_path, "terrain gameplay attribute raw bytes")?;
    verify_prefixed_hash(&raw, &metadata.raw_blake3, &raw_path)?;
    if raw.len() != metadata.sample_count {
        return Err(NativeTerrainError::new(
            "raw gameplay attribute count differs from terrain.json",
        ));
    }
    let path = join_relative(terrain_root, &metadata.path);
    let png = read_file(&path, "terrain gameplay attributes")?;
    verify_prefixed_hash(&png, &metadata.png_blake3, &path)?;
    let decoded = decode_png(&png, &path)?;
    if decoded.color() != ColorType::L8
        || decoded.width() != metadata.width
        || decoded.height() != metadata.height
    {
        return Err(NativeTerrainError::new(format!(
            "{} must be exact {}x{} Gray8, got {}x{} {:?}",
            path.display(),
            metadata.width,
            metadata.height,
            decoded.width(),
            decoded.height(),
            decoded.color()
        )));
    }
    let values = decoded.into_luma8().into_raw();
    if values.len() != metadata.sample_count {
        return Err(NativeTerrainError::new(
            "decoded gameplay attribute count differs from terrain.json",
        ));
    }
    verify_prefixed_hash(&values, &metadata.raw_blake3, &path)?;
    if values != raw {
        return Err(NativeTerrainError::new(
            "Gray8 gameplay attributes differ from their exact raw sidecar",
        ));
    }
    Ok(VerifiedGameplayAttributes {
        width: metadata.width,
        height: metadata.height,
        values: values.into(),
    })
}

pub(super) fn load_grass_layers(
    terrain_root: &Path,
    descriptor_path: &str,
    descriptor: &NativeTerrainDescriptor,
) -> Result<Vec<NativeTerrainGrassLayer>, NativeTerrainError> {
    let Some(detail) = descriptor.detail_and_trees.as_ref() else {
        return Ok(Vec::new());
    };
    if detail.status != "serializedAssetClosureExported" {
        return Ok(Vec::new());
    }
    let Some(prototypes) = detail.prototypes.as_ref() else {
        return Ok(Vec::new());
    };
    let mut layers = Vec::new();
    for prototype in prototypes {
        let density = prototype
            .get("density")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| NativeTerrainError::new("terrain detail prototype has no density"))?;
        let density_total = density
            .get("densityTotal")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| NativeTerrainError::new("terrain detail density has no total"))?;
        if density_total == 0 {
            continue;
        }
        let width = density
            .get("width")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| NativeTerrainError::new("terrain detail density has no width"))?;
        let height = density
            .get("height")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| NativeTerrainError::new("terrain detail density has no height"))?;
        let relative = density
            .get("path")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| NativeTerrainError::new("terrain detail density has no path"))?;
        validate_relative_path(relative, "png")?;
        let path = join_relative(terrain_root, relative);
        let png = read_file(&path, "terrain detail density")?;
        if let Some(expected) = density.get("pngBlake3").and_then(serde_json::Value::as_str) {
            verify_prefixed_hash(&png, expected, &path)?;
        }
        let decoded = decode_png(&png, &path)?;
        if decoded.color() != ColorType::L8
            || decoded.width() != width
            || decoded.height() != height
        {
            return Err(NativeTerrainError::new(format!(
                "{} must be exact {width}x{height} Gray8 terrain detail density",
                path.display()
            )));
        }
        let values = decoded.into_luma8().into_raw();
        let actual_total = values.iter().map(|value| u64::from(*value)).sum::<u64>();
        if actual_total != density_total {
            return Err(NativeTerrainError::new(format!(
                "terrain detail density total differs from {descriptor_path}: expected {density_total}, got {actual_total}"
            )));
        }
        let texture_path = prototype
            .pointer("/prototypeTexture/path")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                NativeTerrainError::new("terrain detail prototype has no texture path")
            })?;
        validate_relative_path(texture_path, "png")?;
        let serialized = prototype
            .get("serialized")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| {
                NativeTerrainError::new("terrain detail prototype has no serialized fields")
            })?;
        let color = |name: &str| -> Result<[f32; 4], NativeTerrainError> {
            let value = serialized
                .get(name)
                .and_then(serde_json::Value::as_object)
                .ok_or_else(|| NativeTerrainError::new(format!("terrain detail has no {name}")))?;
            let channel = |name: &str| {
                value
                    .get(name)
                    .and_then(serde_json::Value::as_f64)
                    .map(|value| value as f32)
                    .filter(|value| value.is_finite())
                    .ok_or_else(|| NativeTerrainError::new("terrain detail color is invalid"))
            };
            Ok([channel("r")?, channel("g")?, channel("b")?, channel("a")?])
        };
        let scalar = |name: &str| -> Result<f32, NativeTerrainError> {
            serialized
                .get(name)
                .and_then(serde_json::Value::as_f64)
                .map(|value| value as f32)
                .filter(|value| value.is_finite() && *value >= 0.0)
                .ok_or_else(|| NativeTerrainError::new(format!("terrain detail {name} is invalid")))
        };
        layers.push(NativeTerrainGrassLayer {
            density_width: width,
            density_height: height,
            density: values.into(),
            texture_path: texture_path.to_owned(),
            healthy_color: color("healthyColor")?,
            dry_color: color("dryColor")?,
            minimum_width: scalar("minWidth")?,
            maximum_width: scalar("maxWidth")?,
            minimum_height: scalar("minHeight")?,
            maximum_height: scalar("maxHeight")?,
        });
    }
    Ok(layers)
}

pub(super) fn read_file(path: &Path, context: &str) -> Result<Vec<u8>, NativeTerrainError> {
    fs::read(path).map_err(|error| {
        NativeTerrainError::new(format!(
            "failed to read {context} {}: {error}",
            path.display()
        ))
    })
}
