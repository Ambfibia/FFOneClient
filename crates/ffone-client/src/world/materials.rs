use super::*;

/// Permanent fail-closed presentation state for one authored visual whose
/// exact legacy material could not be reconstructed.
///
/// The tile may finish loading so unrelated objects remain playable, but no
/// range/residency update may ever expose this visual's glTF fallback shader.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct NativeWorldMaterialPresentationFailed;

pub(super) fn validate_render_contract(
    contract: &NativeTerrainRenderContract,
) -> Result<(), NativeWorldSceneError> {
    let fields = &contract.serialized_fields;
    if contract.selection
        != "owner component whose m_TerrainData resolves to the exact routed TerrainData"
        || ![
            fields.m_heightmap_pixel_error,
            fields.m_splat_map_distance,
            fields.m_detail_object_distance,
            fields.m_tree_billboard_distance,
            fields.m_tree_cross_fade_length,
            fields.m_tree_distance,
        ]
        .iter()
        .all(|value| value.is_finite())
        || fields
            .m_basemap_distance
            .is_some_and(|value| !value.is_finite())
    {
        return Err(NativeWorldSceneError::new(
            "Terrain component render contract is incomplete/non-finite",
        ));
    }
    Ok(())
}
