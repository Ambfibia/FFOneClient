//! Minimap model, tile/marker/waypoint sampling and UI materials.

use super::hud::GameplayUiRect;
use crate::{
    tutorial_mission_content::GameplayNpcMinimapDefinition,
    world_map::{WORLD_MAP_MARKER_PATHS, WORLD_MAP_MARKER_SOURCE_DIMENSIONS},
};
use bevy::prelude::*;

pub const MINIMAP_GROUP_RECT: GameplayUiRect = GameplayUiRect::new(1093.0, -4.0, 187.0, 183.0);
// RenderMinimap keeps mapRect.y absolute while the other rectangles are
// inside NanoRect. Relative to NanoRect.y=-4, its exact top is 13-(-4)=17.
pub const MINIMAP_MAP_RECT: GameplayUiRect = GameplayUiRect::new(26.0, 17.0, 148.0, 148.0);
pub const MINIMAP_FRAME_RECT: GameplayUiRect = GameplayUiRect::new(7.0, -4.0, 187.0, 183.0);
pub const MINIMAP_FUSION_METER_RECT: GameplayUiRect = GameplayUiRect::new(12.0, 3.0, 176.0, 176.0);
pub const MINIMAP_NAME_RECT: GameplayUiRect = GameplayUiRect::new(42.0, 121.0, 120.0, 60.0);

pub const MINIMAP_WAYPOINT_IN_RANGE_PATH: &str = "ui/en/gameplay/minimap/map_icon_02.png";
pub const MINIMAP_WAYPOINT_OUT_OF_RANGE_PATH: &str = "ui/en/gameplay/minimap/map_icon_03.png";
pub const MINIMAP_WAYPOINT_ABOVE_PATH: &str = "ui/en/gameplay/minimap/plus_but_1.png";
pub const MINIMAP_WAYPOINT_BELOW_PATH: &str = "ui/en/gameplay/minimap/minus_but_1.png";
pub const MINIMAP_NEW_MISSION_PATH: &str = "ui/en/gameplay/minimap/map_icon_15.png";
pub const MINIMAP_ADVANCE_MISSION_PATH: &str = "ui/en/gameplay/minimap/map_icon_16.png";
pub const MINIMAP_GROUP_PATH: &str = "ui/en/world-map/markers/map_icon_17.png";
pub const MINIMAP_SHOW_NPC_PATH: &str = "ui/en/world-map/markers/map_icon_18.png";
pub const MINIMAP_FUSION_PATH: &str = "ui/en/world-map/markers/map_icon_19.png";
pub const MINIMAP_MOB_PATH: &str = "ui/en/world-map/markers/map_icon_20.png";
pub const MINIMAP_SHINY_PATH: &str = "ui/en/world-map/markers/map_icon_26.png";
pub(super) const MINIMAP_MARKER_CAPACITY: usize = 64;

pub(super) const CIRCULAR_MINIMAP_TILE_SHADER: &str = "shaders/circular_minimap_tile.wgsl";
pub(super) const FUSION_MATTER_METER_SHADER: &str = "shaders/fusion_matter_meter.wgsl";

#[derive(Clone, Debug, PartialEq)]
pub struct MinimapUi {
    /// World extent multiplier; smaller values zoom in. The initial view is 8.
    pub ratio: f32,
    pub map_name: String,
    pub fusion_matter: i32,
    pub max_fusion_matter: i32,
    /// One to four exact 512x512 legacy minimap tile samples.
    pub tiles: Vec<MinimapTileSample>,
    pub camera_heading_degrees: f32,
    pub avatar_heading_degrees: f32,
    pub waypoint: Option<MinimapWaypointSample>,
    pub custom_waypoints: Vec<(u8, MinimapWaypointSample)>,
    /// NPC, mission-replacement, group and shiny markers in the exact
    /// `RenderMinimap` draw-order family.
    pub markers: Vec<MinimapMarkerSample>,
    pub player_marker_alpha: f32,
}

impl Default for MinimapUi {
    fn default() -> Self {
        Self {
            ratio: 8.0,
            map_name: String::new(),
            fusion_matter: 0,
            max_fusion_matter: 1,
            tiles: Vec::new(),
            camera_heading_degrees: 0.0,
            avatar_heading_degrees: 0.0,
            waypoint: None,
            custom_waypoints: Vec::new(),
            markers: Vec::new(),
            player_marker_alpha: 1.0,
        }
    }
}

impl MinimapUi {
    /// Exact `cnGUINanocom.fDegree / 360` fill used by the circular FM meter.
    #[must_use]
    pub fn fusion_matter_fraction(&self) -> f32 {
        if self.max_fusion_matter <= 0 {
            0.0
        } else {
            (self.fusion_matter.max(0) as f32 / self.max_fusion_matter as f32).clamp(0.0, 1.0)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MinimapTileSample {
    /// Exact suffix from `all_minimap_01` through `all_minimap_16`.
    pub tile_number: u8,
    /// PNG top-left pixel rectangle inside the selected 512x512 tile.
    pub source: GameplayUiRect,
    /// Destination rectangle in the 148x148 minimap viewport.
    pub destination: GameplayUiRect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MinimapWaypointIcon {
    InRange,
    OutOfRange,
    Above,
    Below,
}

impl MinimapWaypointIcon {
    pub const ALL: [Self; 4] = [Self::InRange, Self::OutOfRange, Self::Above, Self::Below];

    pub const fn asset_path(self) -> &'static str {
        match self {
            Self::InRange => MINIMAP_WAYPOINT_IN_RANGE_PATH,
            Self::OutOfRange => MINIMAP_WAYPOINT_OUT_OF_RANGE_PATH,
            Self::Above => MINIMAP_WAYPOINT_ABOVE_PATH,
            Self::Below => MINIMAP_WAYPOINT_BELOW_PATH,
        }
    }

    pub(super) const fn index(self) -> usize {
        match self {
            Self::InRange => 0,
            Self::OutOfRange => 1,
            Self::Above => 2,
            Self::Below => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MinimapWaypointSample {
    pub icon: MinimapWaypointIcon,
    pub left: f32,
    pub top: f32,
    pub rotation_degrees: f32,
    pub alpha: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MinimapMarkerIcon {
    New,
    Advance,
    Group,
    ShowNpc,
    Fusion,
    Mob,
    Shiny,
    /// Exact positive `NpcTableElement.m_iMapIcon` route. The private
    /// newtype guarantees that the published 0..33 marker catalog owns the
    /// referenced image and source dimensions.
    TableData(MinimapTableMarkerIcon),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MinimapTableMarkerIcon(pub(super) u8);

impl MinimapTableMarkerIcon {
    /// Resolves an index used for clean pre-draw rectangle sizing. Index zero
    /// is valid here because mission icons 15/16 can replace an NPC whose
    /// regular icon is zero after the clean code has already sized its rect.
    #[must_use]
    pub fn from_table_data(map_icon: i32) -> Option<Self> {
        let index = usize::try_from(map_icon).ok()?;
        WORLD_MAP_MARKER_PATHS.get(index)?;
        WORLD_MAP_MARKER_SOURCE_DIMENSIONS.get(index)?;
        Some(Self(u8::try_from(index).ok()?))
    }

    #[must_use]
    pub const fn index(self) -> u8 {
        self.0
    }

    #[must_use]
    pub fn asset_path(self) -> &'static str {
        WORLD_MAP_MARKER_PATHS[usize::from(self.0)]
    }

    #[must_use]
    pub fn dimensions(self) -> Vec2 {
        let (width, height) = WORLD_MAP_MARKER_SOURCE_DIMENSIONS[usize::from(self.0)];
        Vec2::new(width as f32, height as f32)
    }
}

impl MinimapMarkerIcon {
    pub const ALL: [Self; 7] = [
        Self::New,
        Self::Advance,
        Self::Group,
        Self::ShowNpc,
        Self::Fusion,
        Self::Mob,
        Self::Shiny,
    ];

    pub fn asset_path(self) -> &'static str {
        match self {
            Self::New => MINIMAP_NEW_MISSION_PATH,
            Self::Advance => MINIMAP_ADVANCE_MISSION_PATH,
            Self::Group => MINIMAP_GROUP_PATH,
            Self::ShowNpc => MINIMAP_SHOW_NPC_PATH,
            Self::Fusion => MINIMAP_FUSION_PATH,
            Self::Mob => MINIMAP_MOB_PATH,
            Self::Shiny => MINIMAP_SHINY_PATH,
            Self::TableData(icon) => icon.asset_path(),
        }
    }

    pub(super) const fn fixed_index(self) -> Option<usize> {
        match self {
            Self::New => Some(0),
            Self::Advance => Some(1),
            Self::Group => Some(2),
            Self::ShowNpc => Some(3),
            Self::Fusion => Some(4),
            Self::Mob => Some(5),
            Self::Shiny => Some(6),
            Self::TableData(_) => None,
        }
    }

    #[must_use]
    pub fn dimensions(self) -> Vec2 {
        match self {
            Self::TableData(icon) => icon.dimensions(),
            _ => Vec2::splat(16.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NormalWorldMinimapMarkerStyle {
    pub icon: MinimapMarkerIcon,
    /// Clean sizes `npcRect` from the row's regular `m_iMapIcon` before a
    /// mission icon replaces the drawn texture.
    pub source_dimensions: Vec2,
}

/// Exact immutable half of `cnGUINanocom.RenderMinimap`'s ordinary-NPC
/// branch. `view_mobs` is an explicit mission-owned snapshot; callers must not
/// infer it from hostility, team, HP, or icon number. Advance mission icon 16
/// wins over new-mission icon 15 when both predicates are true.
#[must_use]
pub fn normal_world_minimap_marker_style(
    definition: GameplayNpcMinimapDefinition,
    view_mobs: bool,
    advance_available: bool,
    new_available: bool,
) -> Option<NormalWorldMinimapMarkerStyle> {
    if definition.npc_class == 0 && definition.sound != 2 && !view_mobs {
        return None;
    }
    let table_icon = MinimapTableMarkerIcon::from_table_data(definition.map_icon)?;
    let icon = if advance_available {
        MinimapMarkerIcon::Advance
    } else if new_available {
        MinimapMarkerIcon::New
    } else if definition.map_icon > 0 {
        MinimapMarkerIcon::TableData(table_icon)
    } else {
        return None;
    };
    Some(NormalWorldMinimapMarkerStyle {
        icon,
        source_dimensions: table_icon.dimensions(),
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MinimapMarkerSample {
    pub icon: MinimapMarkerIcon,
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

/// Exact in-range NPC projection used before `mapIcons[15]`/`mapIcons[16]`
/// replace the NPC's regular minimap texture.
#[must_use]
pub fn minimap_marker(
    player_unity: Vec3,
    target_unity: Vec3,
    ratio: f32,
    icon: MinimapMarkerIcon,
) -> Option<MinimapMarkerSample> {
    minimap_marker_sized(player_unity, target_unity, ratio, icon, icon.dimensions())
}

/// Exact marker projection with an explicit clean pre-replacement rectangle
/// size. `RenderMinimap` sizes from `m_iMapIcon` before mission icons 15/16
/// replace the drawn texture, so normal-world mission projection must retain
/// that source icon's dimensions.
#[must_use]
pub fn minimap_marker_sized(
    player_unity: Vec3,
    target_unity: Vec3,
    ratio: f32,
    icon: MinimapMarkerIcon,
    dimensions: Vec2,
) -> Option<MinimapMarkerSample> {
    const WORLD_SIZE: f32 = 8192.0;
    const VIEW_PIXELS: f32 = 148.0;
    const VIEW_CENTER: f32 = VIEW_PIXELS * 0.5;

    if !player_unity.is_finite()
        || !target_unity.is_finite()
        || !ratio.is_finite()
        || ratio <= 0.0
        || !dimensions.is_finite()
        || dimensions.x <= 0.0
        || dimensions.y <= 0.0
    {
        return None;
    }

    let view_world_fraction = ratio / 256.0;
    let delta = Vec2::new(
        target_unity.x / WORLD_SIZE - player_unity.x / WORLD_SIZE,
        target_unity.z / WORLD_SIZE - player_unity.z / WORLD_SIZE,
    );
    if delta.length() >= view_world_fraction * 0.5 {
        return None;
    }

    Some(MinimapMarkerSample {
        icon,
        left: VIEW_CENTER + delta.x / view_world_fraction * VIEW_PIXELS - dimensions.x * 0.5,
        top: VIEW_CENTER - delta.y / view_world_fraction * VIEW_PIXELS - dimensions.y * 0.5,
        width: dimensions.x,
        height: dimensions.y,
    })
}

/// Exact `Time.time` pulse used independently by `waypoint_event` and `my_point_event`.
pub fn minimap_marker_alpha(event: bool, elapsed_seconds: f32) -> f32 {
    if !event {
        return 1.0;
    }
    if !elapsed_seconds.is_finite() {
        return 0.0;
    }
    (elapsed_seconds * std::f32::consts::PI).sin().abs()
}

/// Reproduces the four-icon `cnGUINanocom` waypoint branch in the 148x148 viewport.
pub fn minimap_waypoint(
    player_unity: Vec3,
    target_unity: Vec3,
    ratio: f32,
    waypoint_event: bool,
    elapsed_seconds: f32,
) -> Option<MinimapWaypointSample> {
    const WORLD_SIZE: f32 = 8192.0;
    const VIEW_PIXELS: f32 = 148.0;
    const VIEW_CENTER: f32 = VIEW_PIXELS * 0.5;
    const MARKER_HALF: f32 = 8.5;

    if !player_unity.is_finite() || !target_unity.is_finite() || !ratio.is_finite() || ratio <= 0.0
    {
        return None;
    }

    let num = ratio / 256.0;
    let mut delta = Vec2::new(
        target_unity.x / WORLD_SIZE - player_unity.x / WORLD_SIZE,
        target_unity.z / WORLD_SIZE - player_unity.z / WORLD_SIZE,
    );
    let magnitude = delta.length();
    let radius = num * 0.5;
    let out_of_range = magnitude >= radius;
    if out_of_range && magnitude > 0.0 {
        delta *= radius / magnitude;
    }

    let relative_screen_x = delta.x / num * VIEW_PIXELS;
    let relative_screen_y = -delta.y / num * VIEW_PIXELS;
    let icon = if out_of_range {
        MinimapWaypointIcon::OutOfRange
    } else if target_unity.y > player_unity.y + 5.0 {
        MinimapWaypointIcon::Above
    } else if target_unity.y < player_unity.y - 5.0 {
        MinimapWaypointIcon::Below
    } else {
        MinimapWaypointIcon::InRange
    };
    let rotation_degrees = if out_of_range {
        relative_screen_y.atan2(relative_screen_x) * 180.0 / 3.14 + 90.0
    } else {
        0.0
    };

    Some(MinimapWaypointSample {
        icon,
        left: VIEW_CENTER + relative_screen_x - MARKER_HALF,
        top: VIEW_CENTER + relative_screen_y - MARKER_HALF,
        rotation_degrees,
        alpha: minimap_marker_alpha(waypoint_event, elapsed_seconds),
    })
}

/// Reproduces `cnGUINanocom.RenderMaps` tile routing for the 8192x8192 legacy world map.
pub fn minimap_tiles(unity_x: f32, unity_z: f32, ratio: f32) -> Vec<MinimapTileSample> {
    const WORLD_SIZE: f32 = 8192.0;
    const GRID: i32 = 4;
    const TILE_PIXELS: f32 = 512.0;
    const VIEW_PIXELS: f32 = 148.0;

    if !unity_x.is_finite() || !unity_z.is_finite() || !ratio.is_finite() {
        return Vec::new();
    }
    let ratio = ratio.clamp(4.0, 16.0);
    let center_x = (unity_x / WORLD_SIZE).clamp(0.0, 1.0);
    let center_z = (unity_z / WORLD_SIZE).clamp(0.0, 1.0);
    // `cnGUINanocom.RenderMinimap`: view width is
    // `0.00390625 * fMiniMapRatio` (`ratio / 256`), centered on the player.
    // The legacy plus button decreases the ratio and therefore zooms in.
    let half = ratio / 512.0;
    let left = (center_x - half).clamp(0.0, 1.0);
    let right = (center_x + half).clamp(0.0, 1.0);
    let bottom = (center_z - half).clamp(0.0, 1.0);
    let top = (center_z + half).clamp(0.0, 1.0);
    let width = (right - left).max(f32::EPSILON);
    let height = (top - bottom).max(f32::EPSILON);
    let max_x = (right - f32::EPSILON).max(left);
    let max_z = (top - f32::EPSILON).max(bottom);
    let first_col = ((left * GRID as f32).floor() as i32).clamp(0, GRID - 1);
    let last_col = ((max_x * GRID as f32).floor() as i32).clamp(0, GRID - 1);
    let first_row = ((bottom * GRID as f32).floor() as i32).clamp(0, GRID - 1);
    let last_row = ((max_z * GRID as f32).floor() as i32).clamp(0, GRID - 1);
    let mut samples = Vec::with_capacity(4);

    for row_from_bottom in first_row..=last_row {
        for col in first_col..=last_col {
            let tile_left = col as f32 / GRID as f32;
            let tile_right = (col + 1) as f32 / GRID as f32;
            let tile_bottom = row_from_bottom as f32 / GRID as f32;
            let tile_top = (row_from_bottom + 1) as f32 / GRID as f32;
            let ix0 = left.max(tile_left);
            let ix1 = right.min(tile_right);
            let iz0 = bottom.max(tile_bottom);
            let iz1 = top.min(tile_top);
            if ix1 <= ix0 || iz1 <= iz0 {
                continue;
            }
            let local_x0 = ix0 * GRID as f32 - col as f32;
            let local_x1 = ix1 * GRID as f32 - col as f32;
            let local_z0 = iz0 * GRID as f32 - row_from_bottom as f32;
            let local_z1 = iz1 * GRID as f32 - row_from_bottom as f32;
            samples.push(MinimapTileSample {
                tile_number: ((GRID - 1 - row_from_bottom) * GRID + col + 1) as u8,
                source: GameplayUiRect::new(
                    local_x0 * TILE_PIXELS,
                    (1.0 - local_z1) * TILE_PIXELS,
                    (local_x1 - local_x0) * TILE_PIXELS,
                    (local_z1 - local_z0) * TILE_PIXELS,
                ),
                destination: GameplayUiRect::new(
                    (ix0 - left) / width * VIEW_PIXELS,
                    (top - iz1) / height * VIEW_PIXELS,
                    (ix1 - ix0) / width * VIEW_PIXELS,
                    (iz1 - iz0) / height * VIEW_PIXELS,
                ),
            });
        }
    }
    samples
}
