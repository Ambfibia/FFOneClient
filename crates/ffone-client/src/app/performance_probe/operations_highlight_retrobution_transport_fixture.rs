use super::*;

pub(super) fn highlight_retrobution_transport_fixture(mut presentation: ResMut<WorldMapPresentation>) {
    if let Some(index) = presentation.markers.iter().position(|marker| {
        matches!(
            marker.kind,
            ffone_client::world_map::WorldMapMarkerKind::Npc { map_icon: 22, .. }
        )
    }) {
        presentation.hover.marker_index = Some(index);
        presentation.hover.pointer = presentation.markers[index].rect.center();
        presentation.hover.control = None;
        if let ffone_client::world_map::WorldMapMarkerKind::Npc { npc_type, .. } =
            presentation.markers[index].kind
        {
            if let Some(transport) = presentation.transport.get_mut(&npc_type) {
                transport.registered = true;
                for (index, (_, registered)) in transport.destinations.iter_mut().enumerate() {
                    *registered = index % 2 == 0;
                }
            }
        }
    }
}
