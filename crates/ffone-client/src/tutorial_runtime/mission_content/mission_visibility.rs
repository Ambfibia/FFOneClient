//! Authored mission secrecy affects markers, independently of task availability.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MissionMarkerVisibility {
    #[default]
    Ordinary,
    SemiSecret,
    Exploration,
    Secret,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MissionMarkerSurface {
    WorldMap,
    Minimap,
    Overhead,
}

impl MissionMarkerVisibility {
    pub const fn from_code(code: i32) -> Option<Self> {
        match code {
            0 => Some(Self::Ordinary),
            1 => Some(Self::SemiSecret),
            2 => Some(Self::Exploration),
            3 => Some(Self::Secret),
            _ => None,
        }
    }
    pub const fn visible_on(self, surface: MissionMarkerSurface) -> bool {
        match self {
            Self::Ordinary => true,
            Self::SemiSecret => matches!(surface, MissionMarkerSurface::Minimap),
            Self::Exploration => matches!(surface, MissionMarkerSurface::Overhead),
            Self::Secret => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn four_secrecy_modes_control_each_surface_independently() {
        for (code, expected) in [
            (0, [true, true, true]),
            (1, [false, true, false]),
            (2, [false, false, true]),
            (3, [false, false, false]),
        ] {
            let policy = MissionMarkerVisibility::from_code(code).unwrap();
            let actual = [
                MissionMarkerSurface::WorldMap,
                MissionMarkerSurface::Minimap,
                MissionMarkerSurface::Overhead,
            ]
            .map(|surface| policy.visible_on(surface));
            assert_eq!(actual, expected);
        }
        assert_eq!(MissionMarkerVisibility::from_code(4), None);
    }
    #[test]
    fn secrecy_filters_runtime_markers_without_hiding_interactions() {
        use crate::{
            assets::AssetLocator,
            tutorial_mission_content::TutorialMissionContent,
            world_mission_runtime::{WorldMissionRuntime, WorldMissionServerEvent0104},
        };
        use ffone_protocol::PcTaskStartSuccess0104;
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
        let mut content = TutorialMissionContent::open(&AssetLocator::open(root).unwrap()).unwrap();
        for definition in content.missions.values_mut() {
            definition.provenance.marker_visibility = MissionMarkerVisibility::Secret;
        }
        let task = 2255;
        let npc = content
            .mission(task)
            .unwrap()
            .provenance
            .terminator_npc_type;
        let mut runtime = WorldMissionRuntime::default();
        let owned = std::collections::BTreeSet::new();
        let start_npc = content.mission(task).unwrap().provenance.start_npc_type;
        for code in 0..=3 {
            let policy = MissionMarkerVisibility::from_code(code).unwrap();
            content
                .missions
                .get_mut(&task)
                .unwrap()
                .provenance
                .marker_visibility = policy;
            assert!(
                runtime
                    .npc_has_available_or_completable_mission(
                        start_npc,
                        100,
                        0,
                        &owned,
                        &[],
                        &content
                    )
                    .0
            );
            for surface in [
                MissionMarkerSurface::WorldMap,
                MissionMarkerSurface::Minimap,
                MissionMarkerSurface::Overhead,
            ] {
                assert_eq!(
                    runtime
                        .npc_mission_markers_on(start_npc, surface, 100, 0, &owned, &[], &content)
                        .0,
                    policy.visible_on(surface)
                );
            }
            assert_eq!(
                runtime
                    .mission_availability_by_npc(100, 0, &owned, &[], &content)
                    .get(&start_npc)
                    .copied()
                    .unwrap_or_default()
                    .0,
                policy.visible_on(MissionMarkerSurface::WorldMap)
            );
        }
        runtime
            .apply_event(
                &WorldMissionServerEvent0104::TaskStartSuccess(PcTaskStartSuccess0104 {
                    task_id: task,
                    remaining_time: 0,
                }),
                &content,
            )
            .unwrap();
        for code in 0..=3 {
            let policy = MissionMarkerVisibility::from_code(code).unwrap();
            content
                .missions
                .get_mut(&task)
                .unwrap()
                .provenance
                .marker_visibility = policy;
            assert!(
                runtime
                    .npc_has_available_or_completable_mission(npc, 100, 0, &owned, &[], &content)
                    .1
            );
            for surface in [
                MissionMarkerSurface::WorldMap,
                MissionMarkerSurface::Minimap,
                MissionMarkerSurface::Overhead,
            ] {
                assert_eq!(
                    runtime
                        .npc_mission_markers_on(npc, surface, 100, 0, &owned, &[], &content)
                        .1,
                    policy.visible_on(surface)
                );
            }
            assert_eq!(
                runtime
                    .mission_availability_by_npc(100, 0, &owned, &[], &content)
                    .get(&npc)
                    .copied()
                    .unwrap_or_default()
                    .1,
                policy.visible_on(MissionMarkerSurface::WorldMap)
            );
        }
    }
}
