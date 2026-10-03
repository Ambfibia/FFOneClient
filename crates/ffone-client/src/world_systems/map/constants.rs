use super::*;

pub const WORLD_MAP_ESCAPE_KEY_ID: u8 = 4;

pub const WORLD_MAP_TOGGLE_KEY_ID: u8 = 22;

pub const WORLD_MAP_HELP_PAGE_ID: u8 = 15;

pub const WORLD_MAP_NO_MAP_MESSAGE_ID: i32 = 174;

pub const WORLD_MAP_EXTENT: f32 = 8192.0;

pub const WORLD_MAP_MARKER_SIZE: f32 = 16.0;

pub const WORLD_MAP_SCREEN_RATIO: f32 = 0.656_842_1;

pub const WORLD_MAP_UI_SCALE_REFERENCE_HEIGHT: f32 = 768.0;

pub const WORLD_MAP_UI_SCALE_NUDGE: f32 = 1.05;

pub(super) const WORLD_MAP_TYPE4_RANGE: f32 = 0.115_966_8;

pub(super) const WORLD_MAP_PAINT_TARGET_WEIGHT: f32 = 0.2;

pub(super) const WORLD_MAP_PAINT_DISPLAY_WEIGHT: f32 = 0.8;

pub(super) const WORLD_MAP_ARROW_UNITS_PER_SECOND: f32 = 1024.0;

pub(super) const WORLD_MAP_LEGACY_DEGREES_DIVISOR: f32 = 3.14;

pub(super) const WORLD_MAP_VIEW_EDGE_EPSILON: f32 = 0.000_01;

pub const WORLD_MAP_BACKDROP_RECT: WorldMapUiRect = WorldMapUiRect::new(0.0, 0.0, 1920.0, 1440.0);

pub const WORLD_MAP_WINDOW_RECT: WorldMapUiRect = WorldMapUiRect::new(0.0, 0.0, 1036.0, 654.0);

pub const WORLD_MAP_CLICK_RECT: WorldMapUiRect = WorldMapUiRect::new(70.0, 60.0, 870.0, 540.0);

pub const WORLD_MAP_PICTURE_RECT: WorldMapUiRect = WorldMapUiRect::new(20.0, 14.0, 950.0, 624.0);

pub const WORLD_MAP_CLOSE_RECT: WorldMapUiRect = WorldMapUiRect::new(995.0, 13.0, 26.0, 26.0);

pub const WORLD_MAP_HELP_RECT: WorldMapUiRect = WorldMapUiRect::new(995.0, 608.0, 30.0, 30.0);

pub const WORLD_MAP_MISSION_FINDER_RECT: WorldMapUiRect =
    WorldMapUiRect::new(857.0, 18.0, 110.0, 18.0);

pub const WORLD_MAP_UP_RECT: WorldMapUiRect = WorldMapUiRect::new(478.0, 10.0, 40.0, 37.0);

pub const WORLD_MAP_DOWN_RECT: WorldMapUiRect = WorldMapUiRect::new(478.0, 600.0, 40.0, 37.0);

pub const WORLD_MAP_LEFT_RECT: WorldMapUiRect = WorldMapUiRect::new(17.0, 307.0, 37.0, 40.0);

pub const WORLD_MAP_RIGHT_RECT: WorldMapUiRect = WorldMapUiRect::new(938.0, 307.0, 37.0, 40.0);

pub const WORLD_MAP_LOCAL_VIEW_RECT: WorldMapUiRect = WorldMapUiRect::new(25.0, 18.0, 110.0, 18.0);

pub const WORLD_MAP_WORLD_VIEW_RECT: WorldMapUiRect = WorldMapUiRect::new(25.0, 38.0, 110.0, 18.0);

pub const WORLD_MAP_ZOOM_IN_RECT: WorldMapUiRect = WorldMapUiRect::new(25.0, 60.0, 26.0, 26.0);

pub const WORLD_MAP_ZOOM_BAR_RECT: WorldMapUiRect = WorldMapUiRect::new(35.0, 80.0, 8.0, 60.0);

pub const WORLD_MAP_ZOOM_OUT_RECT: WorldMapUiRect = WorldMapUiRect::new(25.0, 120.0, 26.0, 26.0);

pub const WORLD_MAP_CURRENT_LOCATION_LABEL_RECT: WorldMapUiRect =
    WorldMapUiRect::new(55.0, 616.0, 110.0, 20.0);

pub const WORLD_MAP_CURRENT_LOCATION_VALUE_RECT: WorldMapUiRect =
    WorldMapUiRect::new(170.0, 616.0, 270.0, 20.0);

/// Native Manrope replacement calibration for the clean 36/55/68 px
/// `MY VIEW`/`WORLD VIEW`/`SHOW FILTERS` advances.
pub const WORLD_MAP_VIEW_FONT_SIZE: f32 = 9.0;

pub const WORLD_MAP_VIEW_FONT_LINE_HEIGHT: f32 = 10.060_000_42;

pub const WORLD_MAP_VIEW_TEXT_OFFSET_X: f32 = 0.0;

pub const WORLD_MAP_VIEW_TEXT_OFFSET_Y: f32 = 5.0;

pub const WORLD_MAP_LOCATION_LABEL_FONT_SIZE: f32 = 8.4;

pub const WORLD_MAP_LOCATION_LABEL_FONT_LINE_HEIGHT: f32 = 8.225_999_83;

pub const WORLD_MAP_LOCATION_LABEL_TEXT_OFFSET_X: f32 = -1.0;

pub const WORLD_MAP_LOCATION_LABEL_TEXT_OFFSET_Y: f32 = 5.0;

/// Native Manrope replacement calibration for the clean 90 px
/// `Pokey Oaks North` advance.
pub const WORLD_MAP_SMALL_CHALET_FONT_SIZE: f32 = 11.0;

pub const WORLD_MAP_SMALL_CHALET_FONT_LINE_HEIGHT: f32 = 12.071_999_55;

pub const WORLD_MAP_LOCATION_VALUE_TEXT_OFFSET_X: f32 = 2.0;

pub const WORLD_MAP_TOOLTIP_TEXT_OFFSET_X: f32 = 0.0;

pub const WORLD_MAP_SMALL_CHALET_TEXT_OFFSET_Y: f32 = 4.0;

/// The four text-bearing map styles all serialize `m_ContentOffset.y = 0`.
/// Their different vertical placement comes from TextAnchor and padding, not
/// an invented global replacement-font shift.
pub const WORLD_MAP_TEXT_CONTENT_OFFSET_Y: f32 = 0.0;

pub const WORLD_MAP_VIEW_TEXT_PADDING: f32 = 2.0;

pub const WORLD_MAP_LOCATION_LABEL_PADDING: f32 = 2.0;

pub const WORLD_MAP_LOCATION_VALUE_PADDING: f32 = 2.0;

pub const WORLD_MAP_PAYZONE_PATHS: [&str; 4] = [
    "ui/en/world-map/maps/payzone_001.png",
    "ui/en/world-map/maps/payzone_002.png",
    "ui/en/world-map/maps/payzone_003.png",
    "ui/en/world-map/maps/payzone_004.png",
];

pub const WORLD_MAP_FREEZONE_PATHS: [&str; 2] = [
    "ui/en/world-map/maps/freezone_001.png",
    "ui/en/world-map/maps/freezone_002.png",
];

pub const WORLD_MAP_DARKLAND_PATHS: [&str; 2] = [
    "ui/en/world-map/maps/darkland_001.png",
    "ui/en/world-map/maps/darkland_002.png",
];

pub const WORLD_MAP_MARKER_PATHS: [&str; 35] = [
    "ui/en/world-map/markers/map_icon_00.png",
    "ui/en/world-map/markers/map_icon_01.png",
    "ui/en/world-map/markers/map_icon_02.png",
    "ui/en/world-map/markers/map_icon_03.png",
    "ui/en/world-map/markers/map_icon_04.png",
    "ui/en/world-map/markers/map_icon_05.png",
    "ui/en/world-map/markers/map_icon_06.png",
    "ui/en/world-map/markers/map_icon_07.png",
    "ui/en/world-map/markers/map_icon_08.png",
    "ui/en/world-map/markers/map_icon_09.png",
    "ui/en/world-map/markers/map_icon_10.png",
    "ui/en/world-map/markers/map_icon_11.png",
    "ui/en/world-map/markers/map_icon_12.png",
    "ui/en/world-map/markers/map_icon_13.png",
    "ui/en/world-map/markers/map_icon_14.png",
    "ui/en/world-map/markers/map_icon_15.png",
    "ui/en/world-map/markers/map_icon_16.png",
    "ui/en/world-map/markers/map_icon_17.png",
    "ui/en/world-map/markers/map_icon_18.png",
    "ui/en/world-map/markers/map_icon_19.png",
    "ui/en/world-map/markers/map_icon_20.png",
    "ui/en/world-map/markers/map_icon_21.png",
    "ui/en/world-map/markers/map_icon_22.png",
    "ui/en/world-map/markers/map_icon_23.png",
    "ui/en/world-map/markers/map_icon_24.png",
    "ui/en/world-map/markers/map_icon_25.png",
    "ui/en/world-map/markers/map_icon_26.png",
    "ui/en/world-map/markers/map_icon_27.png",
    "ui/en/world-map/markers/plus_but_1.png",
    "ui/en/world-map/markers/minus_but_1.png",
    "ui/en/world-map/markers/map_icon_30.png",
    "ui/en/world-map/markers/map_icon_31.png",
    "ui/en/world-map/markers/map_icon_32.png",
    "ui/en/world-map/markers/map_icon_33.png",
    "ui/en/world-map/markers/map_icon_woosh.png",
];

pub const WORLD_MAP_MARKER_SOURCE_DIMENSIONS: [(u32, u32); 35] = [
    (18, 18),
    (32, 32),
    (17, 17),
    (17, 17),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (17, 17),
    (17, 17),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
];

pub const RETROBUTION_WORLD_MAP_MAIN_ARCHIVE_SHA256: &str =
    "59788201962b6a1737b114486c361fe74eef69f507d1d125ca3171377eec602f";

pub const RETROBUTION_WORLD_MAP_TUTORIAL_ARCHIVE_SHA256: &str =
    "49a684ff4236848d0b882a5d725ffbe99d5cfb5d2090dc8705350e97dd3fd024";

pub const RETROBUTION_WORLD_MAP_MAIN_ARCHIVE_BYTES: u64 = 7_000_415;

pub const RETROBUTION_WORLD_MAP_TUTORIAL_ARCHIVE_BYTES: u64 = 27_003_826;

pub const WORLD_MAP_CHALET_FONT_SHA256: &str =
    "6383bd9f81e56d61139884d8e42cb7b2146a11dde4efde55c8bff1e4c2c0bbe8";

pub const WORLD_MAP_JEFFE_FONT_SHA256: &str =
    "f8d41844ad2092d9998e51b8cbef5b65b3ce6db276c93949ececae227674c3e1";

pub(super) const FUTURE_ZONE_RECT: WorldMapZoneRect = WorldMapZoneRect::new(5632.0, 512.0, 2048.0, 2048.0);

pub(super) const DARKLANDS_ZONE_RECT: WorldMapZoneRect = WorldMapZoneRect::new(1024.0, 5120.0, 2560.0, 2048.0);

pub(super) const TUTORIAL_ZONE_RECT: WorldMapZoneRect = WorldMapZoneRect::new(0.0, 0.0, 1536.0, 1536.0);

pub(super) const FUSION_LAIR_RECT: WorldMapZoneRect = WorldMapZoneRect::new(6144.0, 6144.0, 2048.0, 2048.0);

pub const WORLD_MAP_FILTERS: [(u8, &[u8]); 12] = [
    (15, &[15, 16]),
    (10, &[4, 5, 6, 7, 8, 9, 10, 31, 33]),
    (13, &[13]),
    (12, &[12]),
    (32, &[32]),
    (22, &[22]),
    (21, &[21]),
    (34, &[34]),
    (24, &[24]),
    (11, &[11]),
    (30, &[30]),
    (25, &[25]),
];
