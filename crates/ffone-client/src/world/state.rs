use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RuntimeMapTile {
    pub(super) schema: String,
    pub(super) id: String,
    pub(super) grid: [i32; 2],
    pub(super) terrain: MapCatalogArtifact,
    pub(super) scene: MapCatalogArtifact,
    pub(super) behaviour: MapCatalogArtifact,
    pub(super) objects: MapCatalogArtifact,
    pub(super) files: Vec<MapCatalogArtifact>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeWorldPresentationStatus {
    Loading,
    Ready,
}

/// Readiness of render presentation, tracked independently from collision.
///
/// Exact authored triangle colliders are intentionally cooked over multiple
/// frames. Keeping this state separate lets a complete, fully-materialized
/// tile become visible while collision continues to finish; gameplay systems
/// still use [`NativeWorldPresentationStatus`] as the stricter all-ready gate.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeWorldVisualPresentationStatus {
    Loading,
    Ready,
}

#[derive(Component)]
pub(super) struct NativeWorldTerrainPresentation;

/// Readiness of the scripted half of a streamed world tile.
///
/// Static GLB scenes and colliders can finish before the sibling behaviour
/// document has been decoded and bound.  Revealing at that earlier point makes
/// moving platforms look complete while their renderer/collision ownership is
/// still absent, which is both visually distracting and gameplay-breaking.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum NativeWorldBehaviourStatus {
    Loading,
    Ready,
    Blocked(String),
}

/// The renderer belongs to a legacy runtime controller rather than to the
/// always-visible static map. Its controller decides when presentation may be
/// materialized; the generic tile reveal pass must keep it hidden.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeManagedNativeWorldVisual;

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct NativeWorldStreamingStatus {
    pub blocker: Option<String>,
    pub resident_tiles: usize,
    pub target_tiles: usize,
    pub loading_colliders: usize,
    pub resident_budget_exceeded: bool,
}

/// Presentation at the camera target, including water outside physical tiles.
/// Recomputed from the current scope/position, so a warp cannot reuse readiness.
#[derive(Resource, Debug, Default)]
pub struct NativeWorldLocationPresentation {
    pub scope: Option<NativeWorldScope>,
    pub position: Option<Vec3>,
    pub ready: bool,
}

pub(super) fn verify_runtime_world_reference(
    asset_root: &Path,
    reference: &RuntimeWorldAssetReference,
    context: &str,
) -> Result<(), NativeWorldSceneError> {
    read_runtime_world_reference(asset_root, reference, context).map(|_| ())
}
