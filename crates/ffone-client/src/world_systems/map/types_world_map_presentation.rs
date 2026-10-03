use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldMapUiPoint {
    pub x: f32,
    pub y: f32,
}

impl WorldMapUiPoint {
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub(super) fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldMapUiRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl WorldMapUiRect {
    #[must_use]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    #[must_use]
    pub fn contains(self, point: WorldMapUiPoint) -> bool {
        point.x >= self.x
            && point.x < self.x + self.width
            && point.y >= self.y
            && point.y < self.y + self.height
    }

    #[must_use]
    pub fn center(self) -> WorldMapUiPoint {
        WorldMapUiPoint::new(self.x + self.width * 0.5, self.y + self.height * 0.5)
    }

    pub(super) fn is_valid(self) -> bool {
        self.x.is_finite()
            && self.y.is_finite()
            && self.width.is_finite()
            && self.height.is_finite()
            && self.width > 0.0
            && self.height > 0.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WorldMapFontRole {
    Chalet,
    Jeffe,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WorldMapTextAnchor {
    UpperLeft,
    UpperCenter,
    MiddleCenter,
}

/// Text-bearing styles reached by clean `WorldMapMode.OnGUI`.
///
/// The map skin contains unrelated serialized styles; keeping the role on
/// every native Text prevents them from leaking into this presentation.
#[derive(Clone, Copy, Component, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WorldMapTextStyle {
    ViewButton,
    CurrentLocationLabel,
    CurrentLocationValue,
    Tooltip,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldMapTextStyleSpec {
    pub source_style: &'static str,
    pub source_skin_path_id: i64,
    pub source_font_path_id: i64,
    pub font_role: WorldMapFontRole,
    pub font_path: &'static str,
    pub font_size: f32,
    pub line_height: f32,
    /// Unity `RectOffset` order: left, right, top, bottom.
    pub padding: [f32; 4],
    pub anchor: WorldMapTextAnchor,
    pub justify: Justify,
    pub linebreak: LineBreak,
    pub x_offset: f32,
    pub y_offset: f32,
}

impl WorldMapTextStyle {
    #[must_use]
    pub const fn spec(self) -> WorldMapTextStyleSpec {
        match self {
            Self::ViewButton => WorldMapTextStyleSpec {
                source_style: "view_but/view_off",
                source_skin_path_id: WORLD_MAP_SKIN_PATH_ID,
                source_font_path_id: WORLD_MAP_VIEW_FONT_PATH_ID,
                font_role: WorldMapFontRole::Chalet,
                font_path: WORLD_MAP_CHALET_FONT_PATH,
                font_size: WORLD_MAP_VIEW_FONT_SIZE,
                line_height: WORLD_MAP_VIEW_FONT_LINE_HEIGHT,
                padding: [WORLD_MAP_VIEW_TEXT_PADDING; 4],
                anchor: WorldMapTextAnchor::UpperCenter,
                justify: Justify::Center,
                linebreak: LineBreak::WordBoundary,
                x_offset: WORLD_MAP_VIEW_TEXT_OFFSET_X,
                y_offset: WORLD_MAP_VIEW_TEXT_OFFSET_Y,
            },
            Self::CurrentLocationLabel => WorldMapTextStyleSpec {
                source_style: "sfont",
                source_skin_path_id: WORLD_MAP_SKIN_PATH_ID,
                source_font_path_id: WORLD_MAP_LOCATION_LABEL_FONT_PATH_ID,
                font_role: WorldMapFontRole::Jeffe,
                font_path: WORLD_MAP_JEFFE_FONT_PATH,
                font_size: WORLD_MAP_LOCATION_LABEL_FONT_SIZE,
                line_height: WORLD_MAP_LOCATION_LABEL_FONT_LINE_HEIGHT,
                padding: [WORLD_MAP_LOCATION_LABEL_PADDING; 4],
                anchor: WorldMapTextAnchor::MiddleCenter,
                justify: Justify::Center,
                linebreak: LineBreak::WordBoundary,
                x_offset: WORLD_MAP_LOCATION_LABEL_TEXT_OFFSET_X,
                y_offset: WORLD_MAP_LOCATION_LABEL_TEXT_OFFSET_Y,
            },
            Self::CurrentLocationValue => WorldMapTextStyleSpec {
                source_style: "gray",
                source_skin_path_id: WORLD_MAP_SKIN_PATH_ID,
                source_font_path_id: WORLD_MAP_SMALL_CHALET_FONT_PATH_ID,
                font_role: WorldMapFontRole::Chalet,
                font_path: WORLD_MAP_CHALET_FONT_PATH,
                font_size: WORLD_MAP_SMALL_CHALET_FONT_SIZE,
                line_height: WORLD_MAP_SMALL_CHALET_FONT_LINE_HEIGHT,
                padding: [WORLD_MAP_LOCATION_VALUE_PADDING; 4],
                anchor: WorldMapTextAnchor::UpperLeft,
                justify: Justify::Left,
                linebreak: LineBreak::WordBoundary,
                x_offset: WORLD_MAP_LOCATION_VALUE_TEXT_OFFSET_X,
                y_offset: WORLD_MAP_SMALL_CHALET_TEXT_OFFSET_Y,
            },
            Self::Tooltip => WorldMapTextStyleSpec {
                source_style: "window",
                source_skin_path_id: WORLD_MAP_SKIN_PATH_ID,
                source_font_path_id: WORLD_MAP_SMALL_CHALET_FONT_PATH_ID,
                font_role: WorldMapFontRole::Chalet,
                font_path: WORLD_MAP_CHALET_FONT_PATH,
                font_size: WORLD_MAP_SMALL_CHALET_FONT_SIZE,
                line_height: WORLD_MAP_SMALL_CHALET_FONT_LINE_HEIGHT,
                padding: [0.0; 4],
                anchor: WorldMapTextAnchor::UpperCenter,
                justify: Justify::Center,
                linebreak: LineBreak::WordBoundary,
                x_offset: WORLD_MAP_TOOLTIP_TEXT_OFFSET_X,
                y_offset: WORLD_MAP_SMALL_CHALET_TEXT_OFFSET_Y,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldMapPoint {
    /// Legacy Unity world X.
    pub x: f32,
    /// Legacy Unity world Y.
    pub y: f32,
    /// Legacy Unity world Z.
    pub z: f32,
}

impl WorldMapPoint {
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub(super) fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    pub(super) fn normalized_xz(self) -> WorldMapNormalizedPoint {
        WorldMapNormalizedPoint {
            x: self.x / WORLD_MAP_EXTENT,
            y: self.z / WORLD_MAP_EXTENT,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct WorldMapNormalizedPoint {
    pub(super) x: f32,
    pub(super) y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldMapViewRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl WorldMapViewRect {
    pub(super) const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub(super) fn contains(self, point: WorldMapNormalizedPoint) -> bool {
        point.x >= self.x
            && point.x < self.x + self.width
            && point.y >= self.y
            && point.y < self.y + self.height
    }

    pub(super) fn is_valid(self) -> bool {
        self.x.is_finite()
            && self.y.is_finite()
            && self.width.is_finite()
            && self.height.is_finite()
            && self.width > 0.0
            && self.height > 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct WorldMapZoneRect {
    pub(super) x: f32,
    pub(super) z: f32,
    pub(super) width: f32,
    pub(super) depth: f32,
}

impl WorldMapZoneRect {
    pub(super) const fn new(x: f32, z: f32, width: f32, depth: f32) -> Self {
        Self { x, z, width, depth }
    }

    pub(super) fn contains(self, point: WorldMapPoint) -> bool {
        point.x >= self.x
            && point.x < self.x + self.width
            && point.z >= self.z
            && point.z < self.z + self.depth
    }

    pub(super) fn normalized(self) -> WorldMapViewRect {
        WorldMapViewRect::new(
            self.x / WORLD_MAP_EXTENT,
            self.z / WORLD_MAP_EXTENT,
            self.width / WORLD_MAP_EXTENT,
            self.depth / WORLD_MAP_EXTENT,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldMapZone {
    Future,
    DarkLand,
    Tutorial,
    Other,
}

impl WorldMapZone {
    #[must_use]
    pub fn for_point(point: WorldMapPoint) -> Option<Self> {
        if !point.is_finite() {
            return None;
        }
        if FUTURE_ZONE_RECT.contains(point) {
            Some(Self::Future)
        } else if DARKLANDS_ZONE_RECT.contains(point) {
            Some(Self::DarkLand)
        } else if TUTORIAL_ZONE_RECT.contains(point) {
            Some(Self::Tutorial)
        } else {
            Some(Self::Other)
        }
    }

    pub(super) fn bounds(self) -> Option<WorldMapViewRect> {
        match self {
            Self::Future => Some(FUTURE_ZONE_RECT.normalized()),
            Self::DarkLand => Some(DARKLANDS_ZONE_RECT.normalized()),
            Self::Tutorial => Some(TUTORIAL_ZONE_RECT.normalized()),
            Self::Other => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WorldMapZoom {
    Type1 = 0,
    Type2 = 1,
    Type3 = 2,
    Type4 = 3,
}

impl WorldMapZoom {
    #[must_use]
    pub const fn is_world_view(self) -> bool {
        !matches!(self, Self::Type4)
    }

    #[must_use]
    pub const fn tick_index(self) -> Option<usize> {
        match self {
            Self::Type1 => Some(0),
            Self::Type2 => Some(1),
            Self::Type3 => Some(2),
            Self::Type4 => None,
        }
    }

    pub(super) fn base_range(self) -> f32 {
        match self {
            Self::Type1 => 1.0,
            Self::Type2 => 0.5,
            Self::Type3 => 0.25,
            Self::Type4 => WORLD_MAP_TYPE4_RANGE,
        }
    }

    #[must_use]
    pub fn range(self, zone: WorldMapZone) -> (f32, f32) {
        let x = self.base_range();
        let y = if self == Self::Type1 && zone == WorldMapZone::Other {
            // InitMode restores the full-world Type1 Y range after Awake
            // multiplied the serialized array by the 950:624 screen ratio.
            1.0
        } else {
            x * WORLD_MAP_SCREEN_RATIO
        };
        (x, y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldMapPhase {
    Closed,
    Open,
    InstanceNoMap,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldMapPlayer {
    pub position: WorldMapPoint,
    pub yaw_degrees: f32,
}

impl WorldMapPlayer {
    #[must_use]
    pub const fn new(position: WorldMapPoint, yaw_degrees: f32) -> Self {
        Self {
            position,
            yaw_degrees,
        }
    }

    pub(super) fn is_finite(self) -> bool {
        self.position.is_finite() && self.yaw_degrees.is_finite()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldMapOpenContext {
    pub tutorial_locked: bool,
    pub tutorial_active: bool,
    pub player: Option<WorldMapPlayer>,
    pub instance_map: bool,
    pub episode_id: i32,
}

impl WorldMapOpenContext {
    #[must_use]
    pub const fn gameplay(player: WorldMapPlayer) -> Self {
        Self {
            tutorial_locked: false,
            tutorial_active: false,
            player: Some(player),
            instance_map: false,
            episode_id: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldMapOpenDisposition {
    Open,
    InstanceNoMap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldMapCloseInput {
    CloseButton,
    Escape,
    MapKey22,
    InstanceNoMapAcknowledged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WorldMapInputGates {
    pub system_popup_open: bool,
    /// Return value of clean Retrobution's event `(2, 24)` for Key4/Escape.
    pub escape_close_allowed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldMapZoomDirection {
    In,
    Out,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldMapPanDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldMapInputResult {
    Ignored,
    Boundary,
    Changed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldMapMissionAvailability {
    None,
    New,
    Advance,
    NewAndAdvance,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldMapNpcSource {
    pub npc_type: i32,
    pub position: WorldMapPoint,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorldMapMarkerKind {
    CustomWaypoint {
        color: u8,
        rotation_degrees: Option<f32>,
    },
    Player {
        yaw_degrees: f32,
    },
    Npc {
        npc_type: i32,
        map_icon: u8,
        display_name: String,
    },
    MissionNew {
        npc_type: i32,
        display_name: String,
    },
    MissionAdvance {
        npc_type: i32,
        display_name: String,
    },
    Waypoint,
    WaypointUp,
    WaypointDown,
    WaypointOffscreenArrow {
        rotation_degrees: f32,
    },
}

impl WorldMapMarkerKind {
    #[must_use]
    pub const fn legacy_icon_index(&self) -> u8 {
        match self {
            Self::CustomWaypoint {
                rotation_degrees, ..
            } => {
                if rotation_degrees.is_some() {
                    3
                } else {
                    2
                }
            }
            Self::Player { .. } => 0,
            Self::Npc { map_icon, .. } => *map_icon,
            Self::MissionNew { .. } => 15,
            Self::MissionAdvance { .. } => 16,
            Self::Waypoint => 2,
            Self::WaypointUp => 28,
            Self::WaypointDown => 29,
            Self::WaypointOffscreenArrow { .. } => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorldMapMarker {
    pub kind: WorldMapMarkerKind,
    pub rect: WorldMapUiRect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldMapPresentationControl {
    Filter(u8),
    Close,
    Help,
    Up,
    Down,
    Left,
    Right,
    ZoomIn,
    ZoomOut,
    ZoomTick(u8),
    LocalView,
    WorldView,
    ShowFilters,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldMapPresentationHover {
    pub control: Option<WorldMapPresentationControl>,
    pub marker_index: Option<usize>,
    /// Window-local legacy GUI point used for deterministic tooltip placement.
    pub pointer: WorldMapUiPoint,
}

impl Default for WorldMapPresentationHover {
    fn default() -> Self {
        Self {
            control: None,
            marker_index: None,
            pointer: WorldMapUiPoint::new(0.0, 0.0),
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct WorldMapPresentation {
    pub transport: std::collections::BTreeMap<i32, WorldMapTransportNode>,
    pub respawn_position: Option<WorldMapPoint>,
    pub model: WorldMapModel,
    /// A completed pure [`WorldMapModel::project_markers`] result.
    pub markers: Vec<WorldMapMarker>,
    pub current_location: String,
    pub hover: WorldMapPresentationHover,
    pub(super) ui_scale_override: Option<f32>,
}

impl Default for WorldMapPresentation {
    fn default() -> Self {
        Self {
            transport: Default::default(),
            respawn_position: None,
            model: WorldMapModel::default(),
            markers: Vec::new(),
            current_location: String::new(),
            hover: WorldMapPresentationHover::default(),
            ui_scale_override: None,
        }
    }
}

impl WorldMapPresentation {
    pub fn reset_session(&mut self) {
        let preferences = std::mem::take(&mut self.model.preferences);
        *self = Self::default();
        self.model.preferences = preferences;
    }
    #[must_use]
    pub fn new(
        model: WorldMapModel,
        markers: Vec<WorldMapMarker>,
        current_location: impl Into<String>,
    ) -> Self {
        Self {
            transport: Default::default(),
            respawn_position: None,
            model,
            markers,
            current_location: current_location.into(),
            hover: WorldMapPresentationHover::default(),
            ui_scale_override: None,
        }
    }

    pub fn set_ui_scale(&mut self, scale: f32) {
        self.ui_scale_override = Some(valid_world_map_ui_scale(scale));
    }

    pub fn clear_ui_scale_override(&mut self) {
        self.ui_scale_override = None;
    }

    #[must_use]
    pub const fn ui_scale_override(&self) -> Option<f32> {
        self.ui_scale_override
    }

    #[must_use]
    pub fn effective_ui_scale(&self, viewport_height: f32) -> f32 {
        self.ui_scale_override
            .unwrap_or_else(|| clean_world_map_ui_scale(viewport_height))
    }

    pub fn validate(&self) -> Result<(), WorldMapPresentationError> {
        if !self.hover.pointer.is_finite() {
            return Err(WorldMapPresentationError::NonFiniteHover);
        }
        if self
            .hover
            .marker_index
            .is_some_and(|index| index >= self.markers.len())
        {
            return Err(WorldMapPresentationError::InvalidHoverMarker);
        }
        match self.model.phase() {
            WorldMapPhase::Closed | WorldMapPhase::InstanceNoMap => return Ok(()),
            WorldMapPhase::Open => {}
        }
        if world_map_map_asset_path(self.model.zone(), self.model.zoom()).is_none()
            || !view_is_inside_texture(self.model.display_view())
            || !view_is_inside_texture(self.model.target_view())
        {
            return Err(WorldMapPresentationError::InvalidMapView);
        }
        if self.markers.iter().any(|marker| {
            !marker.rect.is_valid()
                || marker.rect.width != WORLD_MAP_MARKER_SIZE
                || marker.rect.height != WORLD_MAP_MARKER_SIZE
                || usize::from(marker.kind.legacy_icon_index()) >= WORLD_MAP_MARKER_PATHS.len()
        }) {
            return Err(WorldMapPresentationError::InvalidMarker);
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct WorldMapTransportNode {
    pub registered: bool,
    pub destinations: Vec<(WorldMapPoint, bool)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldMapScaledGroup {
    pub source: WorldMapUiRect,
    pub node_left: f32,
    pub node_top: f32,
    pub scale: f32,
    pub painted: WorldMapUiRect,
}

impl WorldMapScaledGroup {
    #[must_use]
    pub fn screen_to_local(self, point: WorldMapUiPoint) -> WorldMapUiPoint {
        WorldMapUiPoint::new(
            (point.x - self.painted.x) / self.scale,
            (point.y - self.painted.y) / self.scale,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldMapPresentationLayout {
    pub viewport: WorldMapUiRect,
    pub pivot: WorldMapUiPoint,
    pub scale: f32,
    pub backdrop: WorldMapScaledGroup,
    pub window: WorldMapScaledGroup,
}

#[derive(Resource, Clone)]
pub(super) struct WorldMapPresentationAssets {
    pub(super) images: BTreeMap<&'static str, Handle<Image>>,
    pub(super) chalet: Handle<Font>,
    pub(super) jeffe: Handle<Font>,
}

impl WorldMapPresentationAssets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        Self {
            images: world_map_presentation_asset_paths()
                .into_iter()
                .map(|path| (path, asset_server.load(path)))
                .collect(),
            chalet: asset_server.load(WORLD_MAP_CHALET_FONT_PATH),
            jeffe: asset_server.load(WORLD_MAP_JEFFE_FONT_PATH),
        }
    }

    pub(super) fn image(&self, path: &'static str) -> Handle<Image> {
        self.images.get(path).cloned().unwrap_or_default()
    }

    pub(super) fn failed_path(&self, asset_server: &AssetServer) -> Option<&'static str> {
        self.images
            .iter()
            .find_map(|(path, handle)| {
                matches!(asset_server.load_state(handle.id()), LoadState::Failed(_))
                    .then_some(*path)
            })
            .or_else(|| {
                matches!(
                    asset_server.load_state(self.chalet.id()),
                    LoadState::Failed(_)
                )
                .then_some(WORLD_MAP_CHALET_FONT_PATH)
            })
            .or_else(|| {
                matches!(
                    asset_server.load_state(self.jeffe.id()),
                    LoadState::Failed(_)
                )
                .then_some(WORLD_MAP_JEFFE_FONT_PATH)
            })
    }

    pub(super) fn all_loaded(&self, asset_server: &AssetServer) -> bool {
        self.images
            .values()
            .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
            && matches!(asset_server.load_state(self.chalet.id()), LoadState::Loaded)
            && matches!(asset_server.load_state(self.jeffe.id()), LoadState::Loaded)
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldMapPresentationRoot;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldMapPresentationWindow;
