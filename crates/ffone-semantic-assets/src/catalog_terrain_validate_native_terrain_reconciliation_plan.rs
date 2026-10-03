use super::*;

pub(super) fn insert_native_terrain_batch_file(
    files: &mut BTreeMap<String, NativeTerrainBatchFile>,
    source_root: &Path,
    destination_root: &str,
    relative_path: &str,
    kind: AssetKind,
    expected_blake3: &str,
) -> Result<()> {
    validate_relative_path(relative_path)?;
    if kind == AssetKind::Model || relative_path.to_ascii_lowercase().ends_with(".glb") {
        return Err(SemanticAssetError::InvalidPlan(format!(
            "native terrain batch forbids glTF payload {relative_path}"
        )));
    }
    let expected_blake3 = normalize_blake3(expected_blake3)?;
    let source = source_root.join(native_path(relative_path));
    let (_, actual) = hash_file(&source)?;
    if actual != expected_blake3 {
        return Err(SemanticAssetError::Verification(format!(
            "batch payload hash mismatch {}: expected {}, got {}",
            source.display(),
            expected_blake3,
            actual
        )));
    }
    let destination_path = format!("{destination_root}/{relative_path}");
    match files.get(&destination_path) {
        Some(existing) if existing.expected_blake3 == expected_blake3 && existing.kind == kind => {
            Ok(())
        }
        Some(_) => Err(SemanticAssetError::InvalidPlan(format!(
            "conflicting batch payload destination {destination_path}"
        ))),
        None => {
            files.insert(
                destination_path.clone(),
                NativeTerrainBatchFile {
                    source: NativeTerrainBatchFileSource::File(source),
                    destination_path,
                    kind,
                    expected_blake3,
                },
            );
            Ok(())
        }
    }
}

pub(super) fn validate_native_terrain_reconciliation_plan(asset_root: &Path, plan: &RoutePlan) -> Result<()> {
    if plan.schema != ROUTE_PLAN_SCHEMA {
        return Err(SemanticAssetError::InvalidPlan(format!(
            "expected schema {ROUTE_PLAN_SCHEMA}, got {}",
            plan.schema
        )));
    }
    let mut destinations = BTreeSet::new();
    for route in &plan.routes {
        validate_relative_path(&route.destination_path)?;
        validate_ownership(&route.ownership, &plan.source_build)?;
        let prefix = route.category.required_prefix().ok_or_else(|| {
            SemanticAssetError::InvalidPlan(format!(
                "route {} cannot publish to unresolved category",
                route.destination_path
            ))
        })?;
        if !route.destination_path.starts_with(prefix) {
            return Err(SemanticAssetError::InvalidPlan(format!(
                "{} must be under {prefix}",
                route.destination_path
            )));
        }
        if !destinations.insert(route.destination_path.as_str()) {
            return Err(SemanticAssetError::InvalidPlan(format!(
                "duplicate destination {}",
                route.destination_path
            )));
        }
        if route.kind == AssetKind::Model
            || route
                .destination_path
                .to_ascii_lowercase()
                .ends_with(".glb")
        {
            return Err(SemanticAssetError::InvalidPlan(format!(
                "native terrain reconciliation forbids model/glTF route {}",
                route.destination_path
            )));
        }
        if !matches!(route.content, RouteContent::GeneratedJson { .. }) {
            validate_relative_path(&route.source_path)?;
            let source = asset_root.join(native_path(&route.source_path));
            if !source.is_file() {
                return Err(SemanticAssetError::Verification(format!(
                    "native terrain source is not a file: {}",
                    route.source_path
                )));
            }
        }
    }
    Ok(())
}
