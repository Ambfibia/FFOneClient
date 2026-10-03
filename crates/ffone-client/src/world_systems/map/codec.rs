use super::*;

pub const WORLD_MAP_FRAME_RECT: WorldMapUiRect = WorldMapUiRect::new(17.0, 10.0, 960.0, 630.0);

pub const WORLD_MAP_FRAME_PATH: &str = "ui/en/world-map/frame.png";

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WorldMapPresentationFrame;
