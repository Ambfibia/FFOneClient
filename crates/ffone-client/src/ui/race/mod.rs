//! Clean-Retrobution race result and race-rank parity boundary.
//!
//! `eGameMode::RaceMode` (16) and `eGameMode::RaceRankMode` (17) are
//! deliberately separate here. The former auto-sends start/cancel/end
//! requests and only paints the end result. The latter never sends the
//! declared rank packets: it browses four score tables through a legacy HTTP
//! POST. Protocol encoding, HTTP transport, live NPC-camera rendering, item
//! table presentation, and world-ring ownership remain typed external
//! boundaries.

use bevy::prelude::*;

pub mod mode;
pub mod rank;
pub mod hud;

pub const RACE_SOURCE_BUILD: &str = "retrobution-20260613";
pub const RACE_MAIN_ARCHIVE: &str = "main.unity3d";
pub const RACE_MAIN_ARCHIVE_BYTES: u64 = 7_000_415;
pub const RACE_MAIN_ARCHIVE_SHA256: &str =
    "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F";
pub const RACE_SHARED_ASSET: &str = "sharedassets0.assets";
pub const RACE_SHARED_ASSET_SHA256: &str =
    "B1431CC574D032827FE80F7923AECB73FAA1473835421802CD6D65646EE9DDA7";
pub const RACE_ASSEMBLY_SHA256: &str =
    "33D6F70216B1C7BA05BCC0F270FBA97E767B129159755AF4C8835922E60ACADB";
pub const RACE_TABLE_ARCHIVE: &str = "TableData.resourceFile";
pub const RACE_TABLE_ARCHIVE_BYTES: u64 = 784_963;
pub const RACE_TABLE_ARCHIVE_SHA256: &str =
    "6D4CEA151152E2AB75B7D16590BDA00FBA172600A163E0B3A5318564DF0D7E3B";

pub const RACE_JEFFE_FONT_PATH: &str = "fonts/jeffe.otf";
pub const RACE_CHALET_FONT_PATH: &str = "fonts/chaletbook-regular.ttf";
pub const RACE_JEFFE_FONT_SHA256: &str =
    "F8D41844AD2092D9998E51B8CBEF5B65B3CE6DB276C93949ECECAE227674C3E1";
pub const RACE_CHALET_FONT_SHA256: &str =
    "6383BD9F81E56D61139884D8E42CB7B2146A11DDE4EFDE55C8BFF1E4C2C0BBE8";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RaceUiRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl RaceUiRect {
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
    pub fn contains(self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x < self.x + self.width
            && point.y >= self.y
            && point.y < self.y + self.height
    }

    pub(crate) fn node(self) -> Node {
        Node {
            position_type: PositionType::Absolute,
            left: px(self.x),
            top: px(self.y),
            width: px(self.width),
            height: px(self.height),
            ..default()
        }
    }
}

pub(crate) fn color(value: [f32; 4]) -> Color {
    Color::srgba(value[0], value[1], value[2], value[3])
}
