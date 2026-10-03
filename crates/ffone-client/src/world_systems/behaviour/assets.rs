use super::*;

pub fn behaviour_document_path(_scope: NativeWorldScope, tile_id: &str) -> String {
    format!(
        "map/tiles/{}/behaviour.json",
        canonical_map_tile_id(tile_id)
    )
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct NativeWorldObjectSourceRoute {
    pub(super) source_node: String,
    pub(super) source_geometry: HashMap<String, String>,
}

pub(super) fn read_hashed_world_behaviour_asset(
    path: &Path,
    _expected_blake3: &str,
    context: &str,
) -> Result<Vec<u8>, String> {
    std::fs::read(path)
        .map_err(|error| format!("failed to read {context} {}: {error}", path.display()))
}
