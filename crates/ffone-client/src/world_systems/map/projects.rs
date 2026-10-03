use super::*;

pub(super) fn project_marker_rect(
    point: WorldMapNormalizedPoint,
    view: WorldMapViewRect,
    map_rect: WorldMapUiRect,
) -> WorldMapUiRect {
    let local_x = (point.x - view.x) / view.width;
    let local_y = (point.y - view.y) / view.height;
    WorldMapUiRect::new(
        map_rect.x + local_x * map_rect.width - WORLD_MAP_MARKER_SIZE * 0.5,
        map_rect.y + map_rect.height - local_y * map_rect.height - WORLD_MAP_MARKER_SIZE * 0.5,
        WORLD_MAP_MARKER_SIZE,
        WORLD_MAP_MARKER_SIZE,
    )
}
