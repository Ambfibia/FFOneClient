use super::*;

pub const WORLD_MAP_MODE_ID: u8 = 15;

pub(super) fn world_map_control_selected(model: &WorldMapModel, control: WorldMapPresentationControl) -> bool {
    match control {
        WorldMapPresentationControl::Filter(index) => model.filter_enabled(usize::from(index)),
        WorldMapPresentationControl::ZoomTick(index) => {
            model.zoom().tick_index() == Some(usize::from(index))
        }
        WorldMapPresentationControl::LocalView => model.zoom() == WorldMapZoom::Type4,
        WorldMapPresentationControl::WorldView => model.zoom() != WorldMapZoom::Type4,
        WorldMapPresentationControl::ShowFilters => model.show_filters(),
        _ => false,
    }
}
