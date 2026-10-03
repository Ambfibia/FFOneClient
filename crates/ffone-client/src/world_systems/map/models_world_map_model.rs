use super::*;

#[derive(Debug, Clone)]
pub struct WorldMapModel {
    pub(super) phase: WorldMapPhase,
    pub(super) zone: WorldMapZone,
    pub(super) zoom: WorldMapZoom,
    pub(super) target_view: WorldMapViewRect,
    pub(super) display_view: WorldMapViewRect,
    pub(super) player: Option<WorldMapPlayer>,
    pub(super) waypoint: Option<WorldMapPoint>,
    pub(super) tutorial_active: bool,
    pub(super) show_filters: bool,
    pub preferences: crate::map_preferences::MapPreferences,
    pub(super) dragging: bool,
    pub(super) present_npc_types: BTreeSet<i32>,
    pub(super) last_npc_type_sync_time: u64,
    pub(super) outbox: VecDeque<WorldMapOutboxEvent>,
}

impl Default for WorldMapModel {
    fn default() -> Self {
        // Awake starts with serialized zone Tutorial and enum default Type1,
        // then applies SetZoomRect before InitMode. Preserve that one-time
        // display origin because subsequent InitMode calls only retarget it.
        let zone = WorldMapZone::Tutorial;
        let zoom = WorldMapZoom::Type1;
        let awake_view = centered_and_clamped_view(zone, zoom, 0.5, 0.5);
        Self {
            phase: WorldMapPhase::Closed,
            zone,
            zoom,
            target_view: awake_view,
            display_view: awake_view,
            player: None,
            waypoint: None,
            tutorial_active: false,
            show_filters: true,
            preferences: crate::map_preferences::MapPreferences::default(),
            dragging: false,
            present_npc_types: BTreeSet::new(),
            last_npc_type_sync_time: 0,
            outbox: VecDeque::new(),
        }
    }
}
