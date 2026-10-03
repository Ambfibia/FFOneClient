use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in super::super) enum CleanRaceNpcRoute {
    Menu,
    AutoEnd,
    RecallNano,
    None,
}

#[must_use]
pub(in super::super) const fn clean_race_npc_route(
    service_category: i32,
    ring_race_active: bool,
    has_quest: bool,
) -> CleanRaceNpcRoute {
    match service_category {
        13 => CleanRaceNpcRoute::Menu,
        14 if ring_race_active && !has_quest => CleanRaceNpcRoute::AutoEnd,
        14 => CleanRaceNpcRoute::Menu,
        17 => CleanRaceNpcRoute::RecallNano,
        _ => CleanRaceNpcRoute::None,
    }
}

pub(in super::super) fn warp_away_xcom_index(
    content: &TutorialMissionContent,
    map_number: Option<i32>,
    position: [i32; 3],
) -> Option<i32> {
    match map_number? {
        // GetXCom returns its zero-initialized result even if no row matches.
        // A known map with no XCom must still be able to request regeneration.
        0 => Some(content.nearest_xcom_index(0, position).unwrap_or(0)),
        // Intentional native divergence from primary
        // `builds/retrobution-20260613/main.unity3d` CnGuiChat.GetXCom:
        // FFOne does not yet retain ZoneManager's instance ZoneNum, and must
        // not conflate it with iMapNum. Preserve GetXCom's zero-initialized
        // no-match result; OpenFusion authoritatively recomputes the closest
        // respawn and does not consume this request index.
        _ => Some(0),
    }
}
