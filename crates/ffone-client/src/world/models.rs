use super::*;

pub(super) const PRIMARY_DARKTREE_BRIDGE_15_MODEL_STEM: &str = "objects/nature/15_default_wd_dldt_darktree_01_14/models/\
     wd_dldt_darktree_bridge_01_15_default";

pub(super) const PRIMARY_DARKTREE_BRIDGE_16_MODEL_STEM: &str = "objects/nature/16_default_wd_dldt_darktree_01_15/models/\
     wd_dldt_darktree_bridge_01_16_default";

pub(super) const PRIMARY_ETC_TREE_07_FROND_MODEL_PATH: &str =
    "objects/nature/etc_tree_07_variant_0002/models/etc_tree_07_variant_0005/visual.glb";

pub(super) const PRIMARY_ETC_TREE_07_TRUNK_MODEL_PATH: &str =
    "objects/nature/etc_tree_10/models/etc_tree_07_variant_0004/visual.glb";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativeWorldModel {
    pub id: String,
    pub path: String,
    pub blake3: String,
    pub root_name: String,
}

pub(super) fn matches_primary_darktree_bridge_model_family(model_path: &str) -> bool {
    fn matches_stem(model_path: &str, stem: &str) -> bool {
        let Some(suffix) = model_path.strip_prefix(stem) else {
            return false;
        };
        if suffix == "/visual.glb" {
            return true;
        }
        suffix
            .strip_prefix("_variant_")
            .and_then(|suffix| suffix.strip_suffix("/visual.glb"))
            .is_some_and(|variant| {
                variant.len() == 4 && variant.bytes().all(|byte| byte.is_ascii_digit())
            })
    }

    matches_stem(model_path, PRIMARY_DARKTREE_BRIDGE_15_MODEL_STEM)
        || matches_stem(model_path, PRIMARY_DARKTREE_BRIDGE_16_MODEL_STEM)
}

/// Local-space render bounds shared by every placement of one mesh asset.
///
/// Bevy normally computes an AABB when a [`Mesh3d`] first appears. Static map
/// resources are heavily instanced, so letting that fallback run would scan
/// the same POSITION buffer once per placement. The direct-mesh loader
/// computes it once per asset and attaches the cached value immediately.
#[derive(Resource, Debug, Default)]
pub(super) struct NativeWorldMeshAabbCache {
    pub(super) entries: HashMap<AssetId<Mesh>, Aabb>,
}

pub(super) const LEGACY_ZERO_CONTRIBUTION_MODEL_ID: &str =
    "map-geometry-a088b9c58c7a3d5e0ba6829c216b8b23817fa2e68c774045a3d78c15622a893f";

pub(super) const LEGACY_ZERO_CONTRIBUTION_MODEL_PATH: &str = "objects/unclassified/no/models/no/visual.glb";

pub(super) const LEGACY_ZERO_CONTRIBUTION_MODEL_BLAKE3: &str =
    "fbd430c396babfe818319ef1c45d61c022d065b864200333bbfb3a51af5f89ee";

pub(super) const NATIVE_WORLD_WATER_MODEL_DIRECTORY: &str = "objects/unclassified/ffpoison/models/";

pub(super) fn is_canonical_native_world_water_model_path(path: &str, stem: &str) -> bool {
    path.strip_prefix(NATIVE_WORLD_WATER_MODEL_DIRECTORY)
        .and_then(|relative| relative.strip_suffix("/visual.glb"))
        .is_some_and(|model| {
            model == stem
                || model
                    .strip_prefix(&format!("{stem}_variant_"))
                    .is_some_and(|variant| {
                        variant.len() == 4 && variant.bytes().all(|byte| byte.is_ascii_digit())
                    })
        })
}

pub(super) fn materialize_authored_tri_mesh_colliders(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    mut cache: ResMut<AuthoredColliderGeometryCache>,
    pending: Query<(Entity, &GlobalTransform, &PendingAuthoredTriMeshCollider)>,
) {
    cache
        .entries
        .retain(|key, _| meshes.get(key.source_mesh).is_some());
    let mut budget = AuthoredColliderCookBudget::default();
    let mut remaining_cached = AUTHORED_CACHED_COLLIDER_MATERIALIZATIONS_PER_FRAME;
    for (entity, global, pending) in &pending {
        let cache_key =
            (pending.perimeter_footprint.is_none()).then_some(AuthoredColliderGeometryCacheKey {
                source_mesh: pending.source_mesh.id(),
                cooking: pending.cooking,
                expected_vertex_count: pending.expected_vertex_count,
                expected_index_count: pending.expected_index_count,
            });
        if let Some(geometry) = cache_key.and_then(|key| cache.entries.get(&key)) {
            if remaining_cached == 0 {
                continue;
            }
            remaining_cached -= 1;
            finish_authored_collider_materialization(
                &mut commands,
                entity,
                global,
                authored_collider_from_cached_geometry(pending, geometry),
            );
            continue;
        }
        if budget.remaining_count == 0 {
            continue;
        }
        let Some(mesh) = meshes.get(&pending.source_mesh) else {
            continue;
        };
        let footprint_mesh = match pending.perimeter_footprint.as_ref() {
            Some(footprint) => {
                let Some(mesh) = meshes.get(&footprint.source_mesh) else {
                    continue;
                };
                Some(mesh)
            }
            None => None,
        };
        // Bevy may expand an indexed primitive while generating flat normals,
        // and the exact collider path then copies that expanded vertex array
        // and reconstructs its indices. Budget the payload that is actually
        // resident now, including a convex footprint, rather than the smaller
        // authored cardinality stored in metadata.
        let vertex_work = mesh.count_vertices().saturating_add(
            footprint_mesh
                .as_ref()
                .map_or(0, |mesh| mesh.count_vertices()),
        );
        let index_work = mesh.indices().map_or_else(
            || pending.expected_index_count.max(mesh.count_vertices()),
            |indices| indices.len(),
        );
        if !budget.admit(vertex_work, index_work) {
            continue;
        }
        match authored_collider_from_mesh(pending, mesh, footprint_mesh) {
            Ok(collider) => {
                if let Some(key) = cache_key {
                    cache.entries.insert(
                        key,
                        AuthoredColliderGeometry {
                            vertices: collider.vertices.clone(),
                            indices: collider.indices.clone(),
                            local_min: collider.local_min,
                            local_max: collider.local_max,
                        },
                    );
                }
                finish_authored_collider_materialization(&mut commands, entity, global, collider);
            }
            Err(error) => {
                commands
                    .entity(entity)
                    .remove::<PendingAuthoredTriMeshCollider>()
                    .insert(NativeWorldColliderStatus::Blocked(error));
            }
        }
    }
}

pub(super) fn validate_single_root_world_glb(
    bytes: &[u8],
    expected_root_name: &str,
    path: &str,
) -> Result<(), NativeWorldSceneError> {
    if bytes.len() < 20
        || &bytes[0..4] != b"glTF"
        || u32::from_le_bytes(bytes[4..8].try_into().unwrap()) != 2
        || u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize != bytes.len()
        || u32::from_le_bytes(bytes[16..20].try_into().unwrap()) != 0x4e4f_534a
    {
        return Err(NativeWorldSceneError::new(format!(
            "native world model {path:?} is not a complete GLB 2.0 file"
        )));
    }
    let json_length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    if 20 + json_length > bytes.len() {
        return Err(NativeWorldSceneError::new(format!(
            "native world model {path:?} has a truncated JSON chunk"
        )));
    }
    let document: serde_json::Value = serde_json::from_slice(&bytes[20..20 + json_length])
        .map_err(|error| {
            NativeWorldSceneError::new(format!(
                "native world model {path:?} has invalid GLB JSON: {error}"
            ))
        })?;
    let nodes = document["nodes"].as_array();
    let meshes = document["meshes"].as_array();
    let primitives = document["meshes"][0]["primitives"].as_array();
    let scene_roots = document["scenes"]
        .as_array()
        .and_then(|scenes| scenes.first())
        .and_then(|scene| scene["nodes"].as_array());
    if nodes.is_none_or(|nodes| nodes.len() != 1)
        || meshes.is_none_or(|meshes| meshes.len() != 1)
        || primitives.is_none_or(|primitives| primitives.len() != 1)
        || scene_roots.is_none_or(|roots| roots.as_slice() != [serde_json::json!(0)])
        || document["nodes"][0]["name"].as_str() != Some(expected_root_name)
        || document["meshes"][0]["name"].as_str() != Some(expected_root_name)
        || document["nodes"][0]["mesh"].as_u64() != Some(0)
        || document["nodes"][0].get("children").is_some()
        || document["nodes"][0].get("skin").is_some()
        || document["nodes"][0].get("matrix").is_some()
        || document["nodes"][0].get("translation").is_some()
        || document["nodes"][0].get("rotation").is_some()
        || document["nodes"][0].get("scale").is_some()
    {
        return Err(NativeWorldSceneError::new(format!(
            "native world model {path:?} does not have the exact identity single-primitive root {expected_root_name:?}"
        )));
    }
    Ok(())
}
