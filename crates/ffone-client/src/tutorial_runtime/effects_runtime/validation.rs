use super::*;

#[derive(Clone, Debug)]
pub struct TutorialEffectLibraryError(pub String);

impl fmt::Display for TutorialEffectLibraryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for TutorialEffectLibraryError {}

pub(super) fn validate_catalog_headers(
    effects: &TutorialEffectCatalog,
    projectiles: &TutorialProjectileCatalog,
) -> Result<(), TutorialEffectLibraryError> {
    if effects.schema != TUTORIAL_EFFECT_CATALOG_SCHEMA
        || projectiles.schema != TUTORIAL_PROJECTILE_CATALOG_SCHEMA
        || effects.source_build != RETROBUTION_TUTORIAL_BUILD_ID
        || projectiles.source_build != RETROBUTION_TUTORIAL_BUILD_ID
        || effects.renderer_status != EFFECT_RENDERER_STATUS
        || projectiles.renderer_status != PROJECTILE_RENDERER_STATUS
        || effects.source_bundle != projectiles.source_bundle
        || effects.source_dump != projectiles.source_dump
        || effects.source_assets != projectiles.source_assets
        || projectiles.bullet_table_route != "bullettable.asset"
    {
        return fail("tutorial effect/projectile catalog identity mismatch".to_owned());
    }
    let actual_assets = effects
        .source_assets
        .iter()
        .map(|proof| proof.asset.as_str())
        .collect::<Vec<_>>();
    let expected_assets = [
        PRIMARY_EFFECTS_ASSET,
        EFFECTS_DEPENDENCY_B4,
        EFFECTS_DEPENDENCY_BD5,
    ];
    if actual_assets != expected_assets
        || effects.source_assets[0].serialized_asset != effects.source_bundle
        || effects.source_assets[0].object_dump != effects.source_dump
    {
        return fail(format!(
            "tutorial effect source dependency identity mismatch: actual={actual_assets:?}, expected={expected_assets:?}"
        ));
    }
    for (actual, (asset, serialized_bytes, serialized_hash, dump_bytes, dump_hash)) in effects
        .source_assets
        .iter()
        .zip(RETROBUTION_TUTORIAL_SOURCE_ASSET_PROOFS)
    {
        if actual.asset != asset
            || actual.serialized_asset.logical_name != format!("{asset}/serialized-asset")
            || actual.serialized_asset.bytes != serialized_bytes
            || actual.serialized_asset.blake3 != serialized_hash
            || actual.object_dump.logical_name
                != format!("{asset}/fusionforge-dump-object-all.json")
            || actual.object_dump.bytes != dump_bytes
            || actual.object_dump.blake3 != dump_hash
        {
            return fail(format!(
                "tutorial source proof for {:?} differs from exact Retrotribution evidence",
                actual.asset
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_effect_closure(
    entry: &TutorialEffectCatalogEntry,
    closure: &TutorialEffectClosureFile,
    source_bundle: &TutorialSourceFileProof,
    source_dump: &TutorialSourceFileProof,
    source_assets: &[TutorialSourceAssetProof],
) -> Result<(), TutorialEffectLibraryError> {
    if closure.schema != TUTORIAL_EFFECT_CLOSURE_SCHEMA
        || closure.effect_id != Some(entry.effect_id)
        || closure.container_route != entry.container_route
        || closure.root_asset != entry.root_asset
        || closure.root_path_id != entry.root_path_id
        || closure.source_bundle_blake3 != source_bundle.blake3
        || closure.source_dump_blake3 != source_dump.blake3
        || closure.source_assets != source_assets
        || closure.objects.len() as u64 != entry.object_count
    {
        return fail(format!(
            "effect {} closure disagrees with exact catalog evidence",
            entry.effect_id
        ));
    }
    validate_object_proofs(closure)
}

pub(super) fn validate_object_proofs(
    closure: &TutorialEffectClosureFile,
) -> Result<(), TutorialEffectLibraryError> {
    let mut ids = BTreeSet::new();
    for object in &closure.objects {
        if !ids.insert((object.asset.clone(), object.path_id)) {
            return fail(format!(
                "closure {:?} contains duplicate object {}#{}",
                closure.container_route, object.asset, object.path_id
            ));
        }
        let canonical = canonical_json(&object.value);
        let bytes = serde_json::to_vec(&canonical).map_err(|error| {
            TutorialEffectLibraryError(format!(
                "could not canonicalize {}#{}: {error}",
                object.asset, object.path_id
            ))
        })?;
        if blake3_hash(&bytes) != object.canonical_blake3 {
            return fail(format!(
                "closure {:?} object pathId {} hash mismatch",
                closure.container_route, object.path_id
            ));
        }
    }
    if !ids.contains(&(closure.root_asset.clone(), closure.root_path_id)) {
        return fail(format!(
            "closure {:?} omits root pathId {}",
            closure.container_route, closure.root_path_id
        ));
    }
    Ok(())
}

pub(super) fn validate_exact_bullet_parameters(
    parameters: &TutorialBulletParameters,
    bullet_type: i32,
) -> Result<(), TutorialEffectLibraryError> {
    let expected = match bullet_type {
        5 => TutorialBulletParameters {
            cancel_script: 0,
            fire_script: -1,
            particle_script: 100,
            success_script: 757,
            cancel_model_scale: 0.0,
            curve_height: 0.0,
            fire_model_scale: 1.0,
            bullet_model_scale: 0.0,
            success_model_scale: 1.5,
            hide_time_seconds: 0.4000000059604645,
            maximum_time_seconds: 0.0,
            fire_link: "\"".to_owned(),
            success_link: "center".to_owned(),
            success_sound: "MeleeMedTarget-03".to_owned(),
        },
        13 => TutorialBulletParameters {
            cancel_script: 0,
            fire_script: 809,
            particle_script: 808,
            success_script: 4,
            cancel_model_scale: 0.0,
            curve_height: 0.0,
            fire_model_scale: 1.0,
            bullet_model_scale: 1.0,
            success_model_scale: 1.0,
            hide_time_seconds: 0.0,
            maximum_time_seconds: 0.30000001192092896,
            fire_link: "Gtag01".to_owned(),
            success_link: "center".to_owned(),
            success_sound: "LaserHvyTarget-05".to_owned(),
        },
        66 => TutorialBulletParameters {
            cancel_script: 0,
            fire_script: -1,
            particle_script: 17,
            success_script: 653,
            cancel_model_scale: 0.0,
            curve_height: 0.0,
            fire_model_scale: 1.0,
            bullet_model_scale: 1.0,
            success_model_scale: 3.0,
            hide_time_seconds: 0.30000001192092896,
            maximum_time_seconds: 0.5,
            fire_link: "tag01".to_owned(),
            success_link: "Bip01 center".to_owned(),
            success_sound: "......".to_owned(),
        },
        76 | 77 => TutorialBulletParameters {
            cancel_script: 0,
            fire_script: 0,
            particle_script: if bullet_type == 76 { 15 } else { 391 },
            success_script: 0,
            cancel_model_scale: 0.0,
            curve_height: 0.0,
            fire_model_scale: 0.0,
            bullet_model_scale: 1.0,
            success_model_scale: 0.0,
            hide_time_seconds: 0.0,
            maximum_time_seconds: 2.0,
            fire_link: "\"".to_owned(),
            success_link: "\"".to_owned(),
            success_sound: "\"".to_owned(),
        },
        106 => TutorialBulletParameters {
            cancel_script: 0,
            fire_script: -1,
            particle_script: 0,
            success_script: 653,
            cancel_model_scale: 0.0,
            curve_height: 0.0,
            fire_model_scale: 1.0,
            bullet_model_scale: 0.0,
            success_model_scale: 1.5,
            hide_time_seconds: 0.30000001192092896,
            maximum_time_seconds: 0.30000001192092896,
            fire_link: "\"".to_owned(),
            success_link: "Bip01 Spine1".to_owned(),
            success_sound: "\"".to_owned(),
        },
        113 => TutorialBulletParameters {
            cancel_script: 0,
            fire_script: 812,
            particle_script: 718,
            success_script: 723,
            cancel_model_scale: 0.0,
            curve_height: 0.0,
            fire_model_scale: 1.0,
            bullet_model_scale: 0.800000011920929,
            success_model_scale: 1.399999976158142,
            hide_time_seconds: 0.30000001192092896,
            maximum_time_seconds: 0.5,
            fire_link: "Gtag01".to_owned(),
            success_link: "center".to_owned(),
            success_sound: "SonicTarget-02".to_owned(),
        },
        145 => TutorialBulletParameters {
            cancel_script: 0,
            fire_script: -1,
            particle_script: 100,
            success_script: 772,
            cancel_model_scale: 0.0,
            curve_height: 0.0,
            fire_model_scale: 1.0,
            bullet_model_scale: 0.0,
            success_model_scale: 2.4000000953674316,
            hide_time_seconds: 0.4000000059604645,
            maximum_time_seconds: 0.0,
            fire_link: "\"".to_owned(),
            success_link: "center".to_owned(),
            success_sound: "MeleeMedTarget-01".to_owned(),
        },
        151 => TutorialBulletParameters {
            cancel_script: 0,
            fire_script: 813,
            particle_script: 795,
            success_script: 778,
            cancel_model_scale: 0.0,
            curve_height: 0.0,
            fire_model_scale: 1.0,
            bullet_model_scale: 0.800000011920929,
            success_model_scale: 1.899999976158142,
            hide_time_seconds: 0.30000001192092896,
            maximum_time_seconds: 0.5,
            fire_link: "Gtag01".to_owned(),
            success_link: "center".to_owned(),
            success_sound: "SonicTarget-02".to_owned(),
        },
        152 => TutorialBulletParameters {
            cancel_script: 0,
            fire_script: 817,
            particle_script: 789,
            success_script: 774,
            cancel_model_scale: 0.0,
            curve_height: 0.0,
            fire_model_scale: 2.0,
            bullet_model_scale: 1.0,
            success_model_scale: 2.0,
            hide_time_seconds: 0.0,
            maximum_time_seconds: 0.30000001192092896,
            fire_link: "Gtag01".to_owned(),
            success_link: "center".to_owned(),
            success_sound: "LaserLtTarget-04".to_owned(),
        },
        _ => {
            if ffone_runtime_contracts::retrobution_bullet_row_proof(bullet_type).is_some() {
                // The expanded clean weapon set is pinned by the canonical
                // serialized-row proof during library admission. The original
                // ten rows retain typed assertions as an extra regression
                // guard for the first projectile slice.
                return Ok(());
            }
            return fail(format!("unsupported tutorial bullet type {bullet_type}"));
        }
    };
    if parameters != &expected {
        return fail(format!(
            "bullet {bullet_type} differs from the exact Retrotribution parameters"
        ));
    }
    Ok(())
}

pub(super) fn reject_symlink(path: &Path, label: &str) -> Result<(), TutorialEffectLibraryError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        TutorialEffectLibraryError(format!("could not inspect {label}: {error}"))
    })?;
    if metadata.file_type().is_symlink() {
        return fail(format!("{label} must not be a symlink"));
    }
    Ok(())
}
