use super::*;

pub(super) const WORLD_MAP_TYPE1_TEXTURE_WIDTH: f32 = 1024.0;

pub(super) const WORLD_MAP_TYPE1_TEXTURE_HEIGHT: f32 = 1024.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldMapTextureProof {
    pub asset_path: &'static str,
    pub source: WorldMapAssetSource,
    pub source_path_id: i64,
    pub width: u32,
    pub height: u32,
}

pub(super) const fn texture_proof(
    asset_path: &'static str,
    source: WorldMapAssetSource,
    source_path_id: i64,
    width: u32,
    height: u32,
) -> WorldMapTextureProof {
    WorldMapTextureProof {
        asset_path,
        source,
        source_path_id,
        width,
        height,
    }
}

pub const WORLD_MAP_TEXTURE_PROOFS: [WorldMapTextureProof; 40] = [
    texture_proof(
        WORLD_MAP_BACKDROP_PATH,
        WorldMapAssetSource::TutorialBundle,
        330,
        1920,
        1440,
    ),
    texture_proof(
        WORLD_MAP_FRAME_PATH,
        WorldMapAssetSource::TutorialBundle,
        336,
        959,
        632,
    ),
    texture_proof(
        WORLD_MAP_NO_MAP_PATH,
        WorldMapAssetSource::TutorialBundle,
        291,
        958,
        631,
    ),
    texture_proof(
        WORLD_MAP_PAYZONE_PATHS[0],
        WorldMapAssetSource::TutorialBundle,
        27,
        1024,
        1024,
    ),
    texture_proof(
        WORLD_MAP_PAYZONE_PATHS[1],
        WorldMapAssetSource::TutorialBundle,
        243,
        1024,
        1024,
    ),
    texture_proof(
        WORLD_MAP_PAYZONE_PATHS[2],
        WorldMapAssetSource::TutorialBundle,
        15,
        2048,
        2048,
    ),
    texture_proof(
        WORLD_MAP_PAYZONE_PATHS[3],
        WorldMapAssetSource::TutorialBundle,
        270,
        2048,
        2048,
    ),
    texture_proof(
        WORLD_MAP_FREEZONE_PATHS[0],
        WorldMapAssetSource::TutorialBundle,
        296,
        2048,
        2048,
    ),
    texture_proof(
        WORLD_MAP_FREEZONE_PATHS[1],
        WorldMapAssetSource::TutorialBundle,
        20,
        2048,
        2048,
    ),
    texture_proof(
        WORLD_MAP_DARKLAND_PATHS[0],
        WorldMapAssetSource::TutorialBundle,
        335,
        2048,
        2048,
    ),
    texture_proof(
        WORLD_MAP_DARKLAND_PATHS[1],
        WorldMapAssetSource::TutorialBundle,
        176,
        2048,
        2048,
    ),
    texture_proof(
        WORLD_MAP_MAP_BACK_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        429,
        40,
        653,
    ),
    texture_proof(
        WORLD_MAP_BLACK_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        270,
        8,
        8,
    ),
    texture_proof(
        WORLD_MAP_LOCATION_BACK_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        665,
        23,
        19,
    ),
    texture_proof(
        WORLD_MAP_TOOLTIP_BACK_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        391,
        38,
        17,
    ),
    texture_proof(
        WORLD_MAP_CLOSE_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        105,
        32,
        33,
    ),
    texture_proof(
        WORLD_MAP_CLOSE_HOVER_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        157,
        32,
        33,
    ),
    texture_proof(
        WORLD_MAP_HELP_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        245,
        32,
        32,
    ),
    texture_proof(
        WORLD_MAP_HELP_HOVER_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        83,
        32,
        32,
    ),
    texture_proof(
        WORLD_MAP_UP_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        2,
        40,
        37,
    ),
    texture_proof(
        WORLD_MAP_UP_HOVER_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        498,
        40,
        37,
    ),
    texture_proof(
        WORLD_MAP_DOWN_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        487,
        40,
        37,
    ),
    texture_proof(
        WORLD_MAP_DOWN_HOVER_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        180,
        40,
        37,
    ),
    texture_proof(
        WORLD_MAP_LEFT_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        395,
        37,
        40,
    ),
    texture_proof(
        WORLD_MAP_LEFT_HOVER_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        446,
        37,
        40,
    ),
    texture_proof(
        WORLD_MAP_RIGHT_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        404,
        38,
        41,
    ),
    texture_proof(
        WORLD_MAP_RIGHT_HOVER_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        331,
        38,
        41,
    ),
    texture_proof(
        WORLD_MAP_ZOOM_IN_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        250,
        26,
        26,
    ),
    texture_proof(
        WORLD_MAP_ZOOM_IN_HOVER_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        59,
        26,
        26,
    ),
    texture_proof(
        WORLD_MAP_ZOOM_OUT_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        219,
        26,
        26,
    ),
    texture_proof(
        WORLD_MAP_ZOOM_OUT_HOVER_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        444,
        26,
        26,
    ),
    texture_proof(
        WORLD_MAP_ZOOM_TICK_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        511,
        20,
        7,
    ),
    texture_proof(
        WORLD_MAP_ZOOM_TICK_HOVER_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        95,
        20,
        7,
    ),
    texture_proof(
        WORLD_MAP_ZOOM_TICK_SELECTED_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        277,
        20,
        7,
    ),
    texture_proof(
        WORLD_MAP_ZOOM_BAR_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        314,
        7,
        11,
    ),
    texture_proof(
        WORLD_MAP_VIEW_SELECTED_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        58,
        20,
        18,
    ),
    texture_proof(
        WORLD_MAP_VIEW_NORMAL_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        149,
        20,
        18,
    ),
    texture_proof(
        WORLD_MAP_LINE_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        348,
        16,
        512,
    ),
    texture_proof(
        WORLD_MAP_LINE_EFFECT_LARGE_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        604,
        16,
        64,
    ),
    texture_proof(
        WORLD_MAP_LINE_EFFECT_SMALL_PATH,
        WorldMapAssetSource::MainSharedAssets0,
        203,
        16,
        4,
    ),
];

#[must_use]
pub fn world_map_texture_dimensions(path: &str) -> Option<(u32, u32)> {
    match path {
        "ui/en/character/creation/layout/CCLeftIn3BG.png" => return Some((30, 35)),
        "ui/en/world-map/filters/filter-arrow.png" => return Some((13, 16)),
        "ui/en/world-map/markers/map_icon_25_active.png" => return Some((28, 28)),
        "ui/en/world-map/markers/neon-shine-background.png" => return Some((25, 25)),
        _ => {}
    }

    WORLD_MAP_TEXTURE_PROOFS
        .iter()
        .find(|proof| proof.asset_path == path)
        .map(|proof| (proof.width, proof.height))
        .or_else(|| {
            WORLD_MAP_MARKER_PATHS
                .iter()
                .position(|candidate| *candidate == path)
                .map(|index| {
                    WORLD_MAP_MARKER_SOURCE_DIMENSIONS
                        .get(index)
                        .copied()
                        .unwrap_or((16, 16))
                })
        })
}

pub(super) fn view_is_inside_texture(view: WorldMapViewRect) -> bool {
    view.is_valid()
        && view.x >= -WORLD_MAP_VIEW_EDGE_EPSILON
        && view.y >= -WORLD_MAP_VIEW_EDGE_EPSILON
        && view.x + view.width <= 1.0 + WORLD_MAP_VIEW_EDGE_EPSILON
        && view.y + view.height <= 1.0 + WORLD_MAP_VIEW_EDGE_EPSILON
}
