use super::*;

/// WorldMapMode replaces the normal gameplay surface but remains below the
/// native Option/SystemMessage modal owners.
pub const WORLD_MAP_UI_Z_INDEX: i32 = 8_100;

pub const WORLD_MAP_ASSET_ROOT: &str = "ui/en/world-map";

pub const WORLD_MAP_BACKDROP_PATH: &str = "ui/en/world-map/backdrop.png";

pub const WORLD_MAP_NO_MAP_PATH: &str = "ui/en/world-map/no_map.png";

pub const WORLD_MAP_MAP_BACK_PATH: &str = "ui/en/world-map/skin/map_back.png";

pub const WORLD_MAP_BLACK_PATH: &str = "ui/en/world-map/skin/back.png";

pub const WORLD_MAP_LOCATION_BACK_PATH: &str = "ui/en/world-map/skin/gray.png";

pub const WORLD_MAP_TOOLTIP_BACK_PATH: &str = "ui/en/world-map/skin/buddy_select.png";

pub const WORLD_MAP_CLOSE_PATH: &str = "ui/en/world-map/controls/close.png";

pub const WORLD_MAP_CLOSE_HOVER_PATH: &str = "ui/en/world-map/controls/closeover.png";

pub const WORLD_MAP_HELP_PATH: &str = "ui/en/world-map/controls/NanoMachineHelpButton.png";

pub const WORLD_MAP_HELP_HOVER_PATH: &str =
    "ui/en/world-map/controls/NanoMachineHelpButtonOver.png";

pub const WORLD_MAP_UP_PATH: &str = "ui/en/world-map/controls/up_n.png";

pub const WORLD_MAP_UP_HOVER_PATH: &str = "ui/en/world-map/controls/up_o.png";

pub const WORLD_MAP_DOWN_PATH: &str = "ui/en/world-map/controls/dn_n.png";

pub const WORLD_MAP_DOWN_HOVER_PATH: &str = "ui/en/world-map/controls/dn_o.png";

pub const WORLD_MAP_LEFT_PATH: &str = "ui/en/world-map/controls/lt_n.png";

pub const WORLD_MAP_LEFT_HOVER_PATH: &str = "ui/en/world-map/controls/lt_o.png";

pub const WORLD_MAP_RIGHT_PATH: &str = "ui/en/world-map/controls/rt_n.png";

pub const WORLD_MAP_RIGHT_HOVER_PATH: &str = "ui/en/world-map/controls/rt_o.png";

pub const WORLD_MAP_ZOOM_IN_PATH: &str = "ui/en/world-map/controls/zoom_in_n.png";

pub const WORLD_MAP_ZOOM_IN_HOVER_PATH: &str = "ui/en/world-map/controls/zoom_in_over.png";

pub const WORLD_MAP_ZOOM_OUT_PATH: &str = "ui/en/world-map/controls/zoom_out_n.png";

pub const WORLD_MAP_ZOOM_OUT_HOVER_PATH: &str = "ui/en/world-map/controls/zoom_out_over.png";

pub const WORLD_MAP_ZOOM_TICK_PATH: &str = "ui/en/world-map/controls/zoom_n.png";

pub const WORLD_MAP_ZOOM_TICK_HOVER_PATH: &str = "ui/en/world-map/controls/zoom_over.png";

pub const WORLD_MAP_ZOOM_TICK_SELECTED_PATH: &str = "ui/en/world-map/controls/zoom_select.png";

pub const WORLD_MAP_ZOOM_BAR_PATH: &str = "ui/en/world-map/controls/zoom_bar.png";

pub const WORLD_MAP_VIEW_SELECTED_PATH: &str = "ui/en/world-map/controls/view_but.png";

pub const WORLD_MAP_VIEW_NORMAL_PATH: &str = "ui/en/world-map/controls/view_but_off.png";

pub const WORLD_MAP_LINE_PATH: &str = "ui/en/world-map/overlays/line_main.png";

pub const WORLD_MAP_LINE_EFFECT_LARGE_PATH: &str = "ui/en/world-map/overlays/line_main00.png";

pub const WORLD_MAP_LINE_EFFECT_SMALL_PATH: &str = "ui/en/world-map/overlays/line_main01.png";

pub const WORLD_MAP_CHALET_FONT_PATH: &str = "fonts/chaletbook-regular.ttf";

pub const WORLD_MAP_JEFFE_FONT_PATH: &str = "fonts/jeffe.otf";

/// Clean `FusionFallMapSkin` serialized in primary `main.unity3d`.
pub const WORLD_MAP_SKIN_PATH_ID: i64 = 1_373;

/// `view_but` / `view_off`: `ChaletBook-Regular Small 1`, path ID 1119.
pub const WORLD_MAP_VIEW_FONT_PATH_ID: i64 = 1_119;

/// `sfont`: `JEFFE___06`, path ID 1066. The approved native replacement
/// is calibrated against the clean uppercase-only 100 px label advance.
pub const WORLD_MAP_LOCATION_LABEL_FONT_PATH_ID: i64 = 1_066;

/// `gray` and `window`: `ChaletBook-Regular Small`, path ID 1018.
pub const WORLD_MAP_SMALL_CHALET_FONT_PATH_ID: i64 = 1_018;

pub const WORLD_MAP_MARKER_SOURCE_PATH_IDS: [i64; 34] = [
    173, 224, 353, 311, 557, 139, 383, 496, 56, 450, 150, 486, 507, 64, 460, 388, 7, 263, 431, 608,
    439, 273, 679, 468, 676, 359, 249, 100, 184, 45, 171, 347, 394, 651,
];

pub const WORLD_MAP_SEMANTIC_ASSET_FILES: usize = 79;

pub const WORLD_MAP_SEMANTIC_ASSET_BYTES: u64 = 10043301;

/// SHA-256 over each sorted UTF-8 relative path, a NUL byte, its little-endian
/// `u64` length, and its exact converted PNG bytes.
pub const WORLD_MAP_SEMANTIC_ASSET_SET_SHA256: &str =
    "ec84e7e7b96dae387633e8e54a002cb8b38184856ddf95caeeb0c118c90e1db4";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldMapAssetSource {
    MainSharedAssets0,
    TutorialBundle,
}

impl WorldMapAssetSource {
    #[must_use]
    pub const fn archive_relative_path(self) -> &'static str {
        match self {
            Self::MainSharedAssets0 => "main.unity3d",
            Self::TutorialBundle => "Tutorial.resourceFile",
        }
    }

    #[must_use]
    pub const fn archive_bytes(self) -> u64 {
        match self {
            Self::MainSharedAssets0 => RETROBUTION_WORLD_MAP_MAIN_ARCHIVE_BYTES,
            Self::TutorialBundle => RETROBUTION_WORLD_MAP_TUTORIAL_ARCHIVE_BYTES,
        }
    }

    #[must_use]
    pub const fn archive_sha256(self) -> &'static str {
        match self {
            Self::MainSharedAssets0 => RETROBUTION_WORLD_MAP_MAIN_ARCHIVE_SHA256,
            Self::TutorialBundle => RETROBUTION_WORLD_MAP_TUTORIAL_ARCHIVE_SHA256,
        }
    }

    #[must_use]
    pub const fn serialized_asset(self) -> &'static str {
        match self {
            Self::MainSharedAssets0 => "sharedassets0.assets",
            Self::TutorialBundle => "CustomAssetBundle-b4f543c102ded400fbc6f1da25d9679a",
        }
    }
}

#[must_use]
pub fn world_map_presentation_asset_paths() -> Vec<&'static str> {
    WORLD_MAP_TEXTURE_PROOFS
        .iter()
        .map(|proof| proof.asset_path)
        .chain(WORLD_MAP_MARKER_PATHS)
        .chain([
            "ui/en/world-map/markers/map_icon_25_active.png",
            "ui/en/world-map/filters/filter-arrow.png",
            "ui/en/world-map/markers/neon-shine-background.png",
            "ui/en/character/creation/layout/CCLeftIn3BG.png",
        ])
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldMapNpcCatalogEntry {
    pub display_name: String,
    pub map_icon: i32,
    pub mission: WorldMapMissionAvailability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldMapCatalogLookup<T> {
    Missing,
    Unique(T),
    Ambiguous,
}

/// Typed adapter boundary for TableData/mission/native-world catalog data.
///
/// Adapters must report duplicate or contradictory records as `Ambiguous`.
/// Projection then fails closed instead of choosing an arbitrary row.
pub trait WorldMapCatalog {
    fn npc(&self, npc_type: i32) -> WorldMapCatalogLookup<WorldMapNpcCatalogEntry>;
}

#[must_use]
pub const fn world_map_map_asset_path(
    zone: WorldMapZone,
    zoom: WorldMapZoom,
) -> Option<&'static str> {
    match (zone, zoom) {
        (WorldMapZone::Other, WorldMapZoom::Type1) => Some(WORLD_MAP_PAYZONE_PATHS[0]),
        (WorldMapZone::Other, WorldMapZoom::Type2) => Some(WORLD_MAP_PAYZONE_PATHS[1]),
        (WorldMapZone::Other, WorldMapZoom::Type3) => Some(WORLD_MAP_PAYZONE_PATHS[2]),
        (WorldMapZone::Other, WorldMapZoom::Type4) => Some(WORLD_MAP_PAYZONE_PATHS[3]),
        (WorldMapZone::Future, WorldMapZoom::Type3) => Some(WORLD_MAP_FREEZONE_PATHS[0]),
        (WorldMapZone::Future, WorldMapZoom::Type4)
        | (WorldMapZone::Tutorial, WorldMapZoom::Type4) => Some(WORLD_MAP_FREEZONE_PATHS[1]),
        (WorldMapZone::DarkLand, WorldMapZoom::Type3) => Some(WORLD_MAP_DARKLAND_PATHS[0]),
        (WorldMapZone::DarkLand, WorldMapZoom::Type4) => Some(WORLD_MAP_DARKLAND_PATHS[1]),
        _ => None,
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub enum WorldMapPresentationAssetStatus {
    #[default]
    Loading,
    Ready,
    Failed {
        asset_path: &'static str,
    },
}

pub(super) fn update_world_map_asset_status(
    asset_server: Res<AssetServer>,
    assets: Res<WorldMapPresentationAssets>,
    mut status: ResMut<WorldMapPresentationAssetStatus>,
) {
    let next = if let Some(asset_path) = assets.failed_path(&asset_server) {
        WorldMapPresentationAssetStatus::Failed { asset_path }
    } else if assets.all_loaded(&asset_server) {
        WorldMapPresentationAssetStatus::Ready
    } else {
        WorldMapPresentationAssetStatus::Loading
    };
    if *status != next {
        *status = next;
    }
}

pub(super) fn world_map_control_asset_path(
    control: WorldMapPresentationControl,
    hovered: bool,
    selected: bool,
) -> &'static str {
    match control {
        WorldMapPresentationControl::Filter(_) => "ui/en/character/creation/layout/CCLeftIn3BG.png",
        WorldMapPresentationControl::Close if hovered => WORLD_MAP_CLOSE_HOVER_PATH,
        WorldMapPresentationControl::Close => WORLD_MAP_CLOSE_PATH,
        WorldMapPresentationControl::Help if hovered => WORLD_MAP_HELP_HOVER_PATH,
        WorldMapPresentationControl::Help => WORLD_MAP_HELP_PATH,
        WorldMapPresentationControl::Up if hovered => WORLD_MAP_UP_HOVER_PATH,
        WorldMapPresentationControl::Up => WORLD_MAP_UP_PATH,
        WorldMapPresentationControl::Down if hovered => WORLD_MAP_DOWN_HOVER_PATH,
        WorldMapPresentationControl::Down => WORLD_MAP_DOWN_PATH,
        WorldMapPresentationControl::Left if hovered => WORLD_MAP_LEFT_HOVER_PATH,
        WorldMapPresentationControl::Left => WORLD_MAP_LEFT_PATH,
        WorldMapPresentationControl::Right if hovered => WORLD_MAP_RIGHT_HOVER_PATH,
        WorldMapPresentationControl::Right => WORLD_MAP_RIGHT_PATH,
        WorldMapPresentationControl::ZoomIn if hovered => WORLD_MAP_ZOOM_IN_HOVER_PATH,
        WorldMapPresentationControl::ZoomIn => WORLD_MAP_ZOOM_IN_PATH,
        WorldMapPresentationControl::ZoomOut if hovered => WORLD_MAP_ZOOM_OUT_HOVER_PATH,
        WorldMapPresentationControl::ZoomOut => WORLD_MAP_ZOOM_OUT_PATH,
        WorldMapPresentationControl::ZoomTick(_) if selected => WORLD_MAP_ZOOM_TICK_SELECTED_PATH,
        WorldMapPresentationControl::ZoomTick(_) if hovered => WORLD_MAP_ZOOM_TICK_HOVER_PATH,
        WorldMapPresentationControl::ZoomTick(_) => WORLD_MAP_ZOOM_TICK_PATH,
        WorldMapPresentationControl::LocalView
        | WorldMapPresentationControl::WorldView
        | WorldMapPresentationControl::ShowFilters
            if selected || hovered =>
        {
            WORLD_MAP_VIEW_SELECTED_PATH
        }
        WorldMapPresentationControl::LocalView
        | WorldMapPresentationControl::WorldView
        | WorldMapPresentationControl::ShowFilters => WORLD_MAP_VIEW_NORMAL_PATH,
    }
}

/// Quadratic route arc with the original five-pixel segmentation and alternate
/// segment dashes. Clip each segment to the map before emitting UI geometry.
pub fn world_map_route_segments(
    a: Vec2,
    b: Vec2,
    dashed: bool,
    clip: WorldMapUiRect,
) -> Vec<(Vec2, Vec2)> {
    let delta = b - a;
    let distance = delta.length();
    if !distance.is_finite() || distance <= f32::EPSILON {
        return Vec::new();
    }
    let direction = delta / distance;
    let control = (a + b) * 0.5 + Vec2::new(-direction.y, direction.x) * distance * 0.2;
    let count = ((distance / 5.0) as usize).max(1);
    let mut previous = a;
    let mut result = Vec::new();
    for index in 1..=count {
        let t = index as f32 / count as f32;
        let next = (1.0 - t).powi(2) * a + 2.0 * (1.0 - t) * t * control + t * t * b;
        if !dashed || index % 2 == 0 {
            let d = next - previous;
            let mut low = 0.0_f32;
            let mut high = 1.0_f32;
            let mut visible = true;
            for (p, q) in [
                (-d.x, previous.x - clip.x),
                (d.x, clip.x + clip.width - previous.x),
                (-d.y, previous.y - clip.y),
                (d.y, clip.y + clip.height - previous.y),
            ] {
                if p == 0.0 {
                    if q < 0.0 {
                        visible = false;
                        break;
                    }
                } else if p < 0.0 {
                    low = low.max(q / p);
                } else {
                    high = high.min(q / p);
                }
            }
            if visible && low <= high {
                result.push((previous + low * d, previous + high * d));
            }
        }
        previous = next;
    }
    result
}
