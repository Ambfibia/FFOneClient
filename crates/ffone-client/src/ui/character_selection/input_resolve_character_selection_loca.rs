use super::*;

/// Exact `CoordUtil.ServerToClient` X/Z point lookup against the ordered
/// `worldname` PathID 8 rectangles used by `ReceivePtDongName`.
pub fn resolve_character_selection_location(
    server_position: [i32; 3],
) -> Option<ResolvedCharacterSelectionLocation> {
    let unity_x = server_position[0] as f32 * 0.01;
    let unity_z = server_position[1] as f32 * 0.01;
    character_selection_world_names::LEGACY_WORLD_NAME_AREAS
        .iter()
        .copied()
        .find(|area| area.contains(unity_x, unity_z))
        .and_then(|area| {
            let background = match area.zone {
                "The Future" => CharacterLocationBackground::Future,
                "The Suburbs" => CharacterLocationBackground::Suburbs,
                "Downtown" => CharacterLocationBackground::Downtown,
                "The Wilds" => CharacterLocationBackground::Wilds,
                "The Darklands" => CharacterLocationBackground::Darklands,
                _ => return None,
            };
            Some(ResolvedCharacterSelectionLocation {
                district: area.district,
                zone: area.zone,
                background,
            })
        })
}
