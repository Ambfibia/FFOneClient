use super::*;

pub(super) const WORLD_EFFECT_PREFAB_CLOSURE_SCHEMA: &str = "ffone.world-effect-prefab-closure.v1";

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldEffectPrefabClosure {
    pub schema: String,
    pub id: String,
    pub root: serde_json::Value,
    pub objects: Vec<TutorialUnityObjectProof>,
}

#[derive(Debug, Default)]
pub(super) struct NativeWorldObjectSourceRoutes {
    pub(super) by_node_and_geometry: HashMap<(String, String), String>,
}

pub(super) fn load_native_world_object_source_routes(
    asset_root: &Path,
    tile_id: &str,
) -> Result<NativeWorldObjectSourceRoutes, String> {
    let relative = format!("map/tiles/{}/objects.json", canonical_map_tile_id(tile_id));
    let path = asset_root.join(&relative);
    let bytes =
        std::fs::read(&path).map_err(|error| format!("failed to read {relative}: {error}"))?;
    parse_native_world_object_source_routes(&bytes, &relative)
}

pub(super) fn parse_native_world_object_source_routes(
    bytes: &[u8],
    relative: &str,
) -> Result<NativeWorldObjectSourceRoutes, String> {
    let document: NativeWorldObjectsDocument =
        serde_json::from_slice(bytes).map_err(|error| format!("invalid {relative}: {error}"))?;
    let mut routes = NativeWorldObjectSourceRoutes::default();
    for object in document.objects {
        for (geometry, source_path) in object.source_geometry {
            let key = (object.source_node.clone(), geometry);
            if let Some(previous) = routes.by_node_and_geometry.insert(key, source_path.clone())
                && previous != source_path
            {
                return Err(format!(
                    "{relative} maps one source node/geometry pair to contradictory routes"
                ));
            }
        }
    }
    Ok(routes)
}

pub(super) fn unity_vec3(values: [f64; 3]) -> Vec3 {
    unity_to_native_vector(vec3(values))
}

pub(super) fn has_reproducible_world_particle_prefab(record: &EffectEmitterRecord) -> bool {
    // Clean EffectEmitterController.Start removes an EffectKey when its
    // particlePrefab is null. The independently instantiated nifObject is
    // already materialized by the static-world publication, so an all-null
    // particle array is exact no-particle behaviour rather than a missing
    // serialized closure.
    record.resolved_particle_prefabs.iter().any(Option::is_some)
}

pub(super) fn native_rotation_to_unity_euler_degrees(rotation: Quat) -> Vec3 {
    let reflection = Mat3::from_cols(Vec3::NEG_X, Vec3::Y, Vec3::Z);
    let unity = reflection * Mat3::from_quat(rotation) * reflection;
    // Unity applies Z, then X, then Y. XYZ folds headings beyond 90 degrees
    // into pitch/roll; taking only that Y aimed opposite-facing cannons alike.
    let (y, x, z) = Quat::from_mat3(&unity).to_euler(EulerRot::YXZ);
    Vec3::new(x.to_degrees(), y.to_degrees(), z.to_degrees())
}
