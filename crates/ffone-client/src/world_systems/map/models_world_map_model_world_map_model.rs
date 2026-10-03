use super::*;

impl WorldMapModel {
    pub fn place_custom_waypoint(
        &mut self,
        pointer: WorldMapUiPoint,
        gates: WorldMapInputGates,
    ) -> bool {
        if self.phase != WorldMapPhase::Open
            || gates.system_popup_open
            || !pointer.is_finite()
            || !self.map_draw_rect().contains(pointer)
        {
            return false;
        }
        let rect = self.map_draw_rect();
        let x = (self.target_view.x + (pointer.x - rect.x) / rect.width * self.target_view.width)
            * WORLD_MAP_EXTENT;
        let z = (self.target_view.y
            + (1.0 - (pointer.y - rect.y) / rect.height) * self.target_view.height)
            * WORLD_MAP_EXTENT;
        self.preferences.add_waypoint(x, z).is_some()
    }

    pub fn remove_custom_waypoint(&mut self, color: u8, gates: WorldMapInputGates) -> bool {
        if self.phase != WorldMapPhase::Open || gates.system_popup_open {
            return false;
        }
        let old = self.preferences.waypoints.len();
        self.preferences
            .waypoints
            .retain(|point| point.color != color);
        old != self.preferences.waypoints.len()
    }
    pub fn filter_enabled(&self, index: usize) -> bool {
        WORLD_MAP_FILTERS
            .get(index)
            .is_some_and(|(icon, _)| self.preferences.enabled_icons.contains(icon))
    }

    pub fn toggle_filter(
        &mut self,
        index: usize,
        gates: WorldMapInputGates,
    ) -> WorldMapInputResult {
        if self.phase != WorldMapPhase::Open
            || gates.system_popup_open
            || !self.show_filters
            || self.zoom == WorldMapZoom::Type4
        {
            return WorldMapInputResult::Ignored;
        }
        let Some((icon, group)) = WORLD_MAP_FILTERS.get(index) else {
            return WorldMapInputResult::Ignored;
        };
        let enabled = self.preferences.enabled_icons.contains(icon);
        for icon in *group {
            if enabled {
                self.preferences
                    .enabled_icons
                    .retain(|candidate| candidate != icon);
            } else {
                if !self.preferences.enabled_icons.contains(icon) {
                    self.preferences.enabled_icons.push(*icon);
                }
            }
        }
        WorldMapInputResult::Changed
    }

    pub fn control_at(&self, point: WorldMapUiPoint) -> Option<WorldMapPresentationControl> {
        if self.show_filters && self.zoom != WorldMapZoom::Type4 {
            for index in 0..WORLD_MAP_FILTERS.len() {
                if world_map_filter_rect(index).contains(point) {
                    return Some(WorldMapPresentationControl::Filter(index as u8));
                }
            }
        }
        world_map_control_at(point)
    }
    #[must_use]
    pub fn phase(&self) -> WorldMapPhase {
        self.phase
    }

    #[must_use]
    pub fn zone(&self) -> WorldMapZone {
        self.zone
    }

    #[must_use]
    pub fn zoom(&self) -> WorldMapZoom {
        self.zoom
    }

    #[must_use]
    pub fn target_view(&self) -> WorldMapViewRect {
        self.target_view
    }

    #[must_use]
    pub fn display_view(&self) -> WorldMapViewRect {
        self.display_view
    }

    #[must_use]
    pub fn show_filters(&self) -> bool {
        self.show_filters
    }

    #[must_use]
    pub fn is_dragging(&self) -> bool {
        self.dragging
    }

    #[must_use]
    pub fn last_npc_type_sync_time(&self) -> u64 {
        self.last_npc_type_sync_time
    }

    #[must_use]
    pub fn present_npc_types(&self) -> &BTreeSet<i32> {
        &self.present_npc_types
    }

    pub fn pop_outbox(&mut self) -> Option<WorldMapOutboxEvent> {
        self.outbox.pop_front()
    }

    pub fn drain_outbox(&mut self) -> impl Iterator<Item = WorldMapOutboxEvent> + '_ {
        self.outbox.drain(..)
    }

    pub fn try_open(
        &mut self,
        context: WorldMapOpenContext,
    ) -> Result<WorldMapOpenDisposition, WorldMapError> {
        if self.phase != WorldMapPhase::Closed {
            return Err(WorldMapError::AlreadyOpen);
        }
        if context.tutorial_locked {
            return Err(WorldMapError::TutorialLocked);
        }
        if context.player.is_some_and(|player| !player.is_finite()) {
            return Err(WorldMapError::NonFiniteInput);
        }

        let (zone, zoom, target_view) = if let Some(player) = context.player {
            let zone =
                WorldMapZone::for_point(player.position).ok_or(WorldMapError::NonFiniteInput)?;
            let zoom = match zone {
                WorldMapZone::Future | WorldMapZone::DarkLand => WorldMapZoom::Type3,
                WorldMapZone::Tutorial => WorldMapZoom::Type4,
                WorldMapZone::Other => WorldMapZoom::Type1,
            };
            let normalized = player.position.normalized_xz();
            (
                zone,
                zoom,
                centered_and_clamped_view(zone, zoom, normalized.x, normalized.y),
            )
        } else {
            // InitMode sets Other, then returns before changing the existing
            // zoom/target state when UserContainer has no player.
            (WorldMapZone::Other, self.zoom, self.target_view)
        };
        if !target_view.is_valid() {
            return Err(WorldMapError::InvalidViewState);
        }

        let phase = if context.instance_map && context.episode_id <= 0 {
            WorldMapPhase::InstanceNoMap
        } else {
            WorldMapPhase::Open
        };
        self.zone = zone;
        self.zoom = zoom;
        self.target_view = target_view;
        self.player = context.player;
        self.tutorial_active = context.tutorial_active;
        self.dragging = false;
        self.phase = phase;
        self.outbox
            .push_back(WorldMapOutboxEvent::RequestPresentNpcTypes {
                last_sync_time: self.last_npc_type_sync_time,
            });

        Ok(match phase {
            WorldMapPhase::Open => WorldMapOpenDisposition::Open,
            WorldMapPhase::InstanceNoMap => WorldMapOpenDisposition::InstanceNoMap,
            WorldMapPhase::Closed => unreachable!("phase was selected above"),
        })
    }

    pub fn try_close(&mut self, input: WorldMapCloseInput, gates: WorldMapInputGates) -> bool {
        if self.phase == WorldMapPhase::Closed {
            return false;
        }
        let allowed = match input {
            WorldMapCloseInput::InstanceNoMapAcknowledged => {
                self.phase == WorldMapPhase::InstanceNoMap
            }
            WorldMapCloseInput::CloseButton | WorldMapCloseInput::MapKey22 => {
                self.phase == WorldMapPhase::Open && !gates.system_popup_open
            }
            WorldMapCloseInput::Escape => {
                self.phase == WorldMapPhase::Open
                    && !gates.system_popup_open
                    && gates.escape_close_allowed
            }
        };
        if !allowed {
            return false;
        }
        self.phase = WorldMapPhase::Closed;
        self.dragging = false;
        self.outbox.push_back(WorldMapOutboxEvent::ExitMode);
        true
    }

    pub fn set_player(&mut self, player: WorldMapPlayer) -> Result<(), WorldMapError> {
        if !player.is_finite() {
            return Err(WorldMapError::NonFiniteInput);
        }
        self.player = Some(player);
        Ok(())
    }

    pub fn set_waypoint(&mut self, waypoint: Option<WorldMapPoint>) -> Result<(), WorldMapError> {
        if waypoint.is_some_and(|point| !point.is_finite()) {
            return Err(WorldMapError::NonFiniteInput);
        }
        self.waypoint = waypoint;
        Ok(())
    }

    pub fn toggle_filters(&mut self, gates: WorldMapInputGates) -> WorldMapInputResult {
        if self.phase != WorldMapPhase::Open || gates.system_popup_open {
            return WorldMapInputResult::Ignored;
        }
        self.show_filters = !self.show_filters;
        WorldMapInputResult::Changed
    }

    #[must_use]
    pub fn map_draw_rect(&self) -> WorldMapUiRect {
        if self.zoom != WorldMapZoom::Type1 {
            return WORLD_MAP_PICTURE_RECT;
        }
        legacy_aspect_adjusted_rect(
            WORLD_MAP_PICTURE_RECT,
            WORLD_MAP_TYPE1_TEXTURE_WIDTH,
            WORLD_MAP_TYPE1_TEXTURE_HEIGHT,
        )
    }

    pub fn zoom_wheel_at(
        &mut self,
        window_point: WorldMapUiPoint,
        direction: WorldMapZoomDirection,
        gates: WorldMapInputGates,
    ) -> Result<WorldMapInputResult, WorldMapError> {
        if !window_point.is_finite() {
            return Err(WorldMapError::NonFiniteInput);
        }
        if self.phase != WorldMapPhase::Open
            || gates.system_popup_open
            || self.zone != WorldMapZone::Other
            || !WORLD_MAP_CLICK_RECT.contains(window_point)
        {
            return Ok(WorldMapInputResult::Ignored);
        }
        let map_rect = self.map_draw_rect();
        let focal_x = (window_point.x - map_rect.x) / map_rect.width;
        let focal_y = (window_point.y - map_rect.y) / map_rect.height;
        self.zoom_from_focal(direction, focal_x, focal_y)
    }

    pub fn zoom_button(
        &mut self,
        direction: WorldMapZoomDirection,
        gates: WorldMapInputGates,
    ) -> Result<WorldMapInputResult, WorldMapError> {
        if self.phase != WorldMapPhase::Open
            || gates.system_popup_open
            || self.zone != WorldMapZone::Other
            || self.zoom == WorldMapZoom::Type4
        {
            return Ok(WorldMapInputResult::Ignored);
        }
        self.zoom_from_focal(direction, 0.5, 0.5)
    }

    pub fn select_zoom_tick(
        &mut self,
        tick_index: usize,
        gates: WorldMapInputGates,
    ) -> Result<WorldMapInputResult, WorldMapError> {
        if self.phase != WorldMapPhase::Open
            || gates.system_popup_open
            || self.zone != WorldMapZone::Other
            || self.zoom == WorldMapZoom::Type4
        {
            return Ok(WorldMapInputResult::Ignored);
        }
        let next = match tick_index {
            0 => WorldMapZoom::Type1,
            1 => WorldMapZoom::Type2,
            2 => WorldMapZoom::Type3,
            _ => return Ok(WorldMapInputResult::Ignored),
        };
        if next == self.zoom {
            return Ok(WorldMapInputResult::Boundary);
        }
        let center = self.target_center()?;
        self.zoom = next;
        self.target_view = centered_and_clamped_view(self.zone, self.zoom, center.x, center.y);
        Ok(WorldMapInputResult::Changed)
    }

    pub fn select_local_view(
        &mut self,
        gates: WorldMapInputGates,
    ) -> Result<WorldMapInputResult, WorldMapError> {
        if self.phase != WorldMapPhase::Open || gates.system_popup_open {
            return Ok(WorldMapInputResult::Ignored);
        }
        if self.zoom == WorldMapZoom::Type4 {
            return Ok(WorldMapInputResult::Boundary);
        }
        let center = self
            .player
            .map(|player| player.position.normalized_xz())
            .unwrap_or(WorldMapNormalizedPoint { x: 0.0, y: 0.0 });
        self.zoom = WorldMapZoom::Type4;
        self.target_view = centered_and_clamped_view(self.zone, self.zoom, center.x, center.y);
        Ok(WorldMapInputResult::Changed)
    }

    pub fn select_world_view(
        &mut self,
        gates: WorldMapInputGates,
    ) -> Result<WorldMapInputResult, WorldMapError> {
        if self.phase != WorldMapPhase::Open || gates.system_popup_open {
            return Ok(WorldMapInputResult::Ignored);
        }
        if self.zoom != WorldMapZoom::Type4 {
            return Ok(WorldMapInputResult::Boundary);
        }
        if self.zone == WorldMapZone::Tutorial {
            return Ok(WorldMapInputResult::Ignored);
        }
        let center = self
            .player
            .map(|player| player.position.normalized_xz())
            .unwrap_or(WorldMapNormalizedPoint { x: 0.0, y: 0.0 });
        self.zoom = if self.zone == WorldMapZone::Other {
            WorldMapZoom::Type1
        } else {
            WorldMapZoom::Type3
        };
        self.target_view = centered_and_clamped_view(self.zone, self.zoom, center.x, center.y);
        Ok(WorldMapInputResult::Changed)
    }

    pub fn begin_drag(
        &mut self,
        window_point: WorldMapUiPoint,
        gates: WorldMapInputGates,
    ) -> Result<WorldMapInputResult, WorldMapError> {
        if !window_point.is_finite() {
            return Err(WorldMapError::NonFiniteInput);
        }
        if self.phase != WorldMapPhase::Open
            || gates.system_popup_open
            || self.zoom == WorldMapZoom::Type1
            || !WORLD_MAP_CLICK_RECT.contains(window_point)
        {
            return Ok(WorldMapInputResult::Ignored);
        }
        if self.dragging {
            return Ok(WorldMapInputResult::Boundary);
        }
        self.dragging = true;
        Ok(WorldMapInputResult::Changed)
    }

    pub fn drag_by(
        &mut self,
        delta_x: f32,
        delta_y: f32,
        delta_seconds: f32,
        gates: WorldMapInputGates,
    ) -> Result<WorldMapInputResult, WorldMapError> {
        validate_motion(delta_x, delta_y, delta_seconds)?;
        if self.phase != WorldMapPhase::Open || gates.system_popup_open || !self.dragging {
            return Ok(WorldMapInputResult::Ignored);
        }
        let (range_x, range_y) = self.zoom.range(self.zone);
        let mut next = self.target_view;
        next.x -= delta_x / WORLD_MAP_PICTURE_RECT.width * range_x;
        next.y += delta_y / WORLD_MAP_PICTURE_RECT.height * range_y;
        clamp_after_legacy_pan(&mut next, self.zone);
        if !next.is_valid() {
            return Err(WorldMapError::InvalidViewState);
        }
        let changed = next != self.target_view;
        self.target_view = next;
        Ok(if changed {
            WorldMapInputResult::Changed
        } else {
            WorldMapInputResult::Boundary
        })
    }

    pub fn end_drag(&mut self, gates: WorldMapInputGates) -> WorldMapInputResult {
        if self.phase != WorldMapPhase::Open || gates.system_popup_open || !self.dragging {
            return WorldMapInputResult::Ignored;
        }
        self.dragging = false;
        WorldMapInputResult::Changed
    }

    pub fn repeat_pan(
        &mut self,
        direction: WorldMapPanDirection,
        delta_seconds: f32,
        gates: WorldMapInputGates,
    ) -> Result<WorldMapInputResult, WorldMapError> {
        if !delta_seconds.is_finite() || delta_seconds < 0.0 {
            return Err(WorldMapError::InvalidDeltaTime);
        }
        if self.phase != WorldMapPhase::Open || gates.system_popup_open {
            return Ok(WorldMapInputResult::Ignored);
        }
        let (range_x, range_y) = self.zoom.range(self.zone);
        let mut next = self.target_view;
        match direction {
            WorldMapPanDirection::Up => {
                next.y +=
                    range_y / WORLD_MAP_EXTENT * WORLD_MAP_ARROW_UNITS_PER_SECOND * delta_seconds;
            }
            WorldMapPanDirection::Down => {
                next.y -=
                    range_y / WORLD_MAP_EXTENT * WORLD_MAP_ARROW_UNITS_PER_SECOND * delta_seconds;
            }
            WorldMapPanDirection::Left => {
                next.x -=
                    range_x / WORLD_MAP_EXTENT * WORLD_MAP_ARROW_UNITS_PER_SECOND * delta_seconds;
            }
            WorldMapPanDirection::Right => {
                next.x +=
                    range_x / WORLD_MAP_EXTENT * WORLD_MAP_ARROW_UNITS_PER_SECOND * delta_seconds;
            }
        }
        clamp_after_legacy_repeat(&mut next, self.zone, direction);
        if !next.is_valid() {
            return Err(WorldMapError::InvalidViewState);
        }
        let changed = next != self.target_view;
        self.target_view = next;
        Ok(if changed {
            WorldMapInputResult::Changed
        } else {
            WorldMapInputResult::Boundary
        })
    }

    /// Applies the clean repaint interpolation: target * 0.2 + display * 0.8.
    pub fn advance_paint(&mut self) -> Result<WorldMapInputResult, WorldMapError> {
        if self.phase != WorldMapPhase::Open {
            return Ok(WorldMapInputResult::Ignored);
        }
        if !self.target_view.is_valid() || !self.display_view.is_valid() {
            return Err(WorldMapError::InvalidViewState);
        }
        let previous = self.display_view;
        self.display_view.x = self.target_view.x * WORLD_MAP_PAINT_TARGET_WEIGHT
            + self.display_view.x * WORLD_MAP_PAINT_DISPLAY_WEIGHT;
        self.display_view.y = self.target_view.y * WORLD_MAP_PAINT_TARGET_WEIGHT
            + self.display_view.y * WORLD_MAP_PAINT_DISPLAY_WEIGHT;
        self.display_view.width = self.target_view.width * WORLD_MAP_PAINT_TARGET_WEIGHT
            + self.display_view.width * WORLD_MAP_PAINT_DISPLAY_WEIGHT;
        self.display_view.height = self.target_view.height * WORLD_MAP_PAINT_TARGET_WEIGHT
            + self.display_view.height * WORLD_MAP_PAINT_DISPLAY_WEIGHT;
        Ok(if self.display_view == previous {
            WorldMapInputResult::Boundary
        } else {
            WorldMapInputResult::Changed
        })
    }

    /// Applies a decoded `P_FE2CL_REP_PRESENT_NPC_TYPES` update transactionally.
    pub fn apply_present_npc_types(
        &mut self,
        clear: bool,
        server_time: u64,
        npc_types: &[i32],
    ) -> Result<(), WorldMapError> {
        let mut next = if clear {
            BTreeSet::new()
        } else {
            self.present_npc_types.clone()
        };
        for &npc_type in npc_types {
            if !next.insert(npc_type) {
                return Err(WorldMapError::DuplicatePresentNpcType(npc_type));
            }
        }
        self.present_npc_types = next;
        self.last_npc_type_sync_time = server_time;
        Ok(())
    }

    /// Projects markers in clean draw order: authored NPC array, waypoint,
    /// then player. No click/warp/waypoint-placement behavior exists here.
    pub fn project_markers<C: WorldMapCatalog>(
        &self,
        catalog: &C,
        npcs: &[WorldMapNpcSource],
    ) -> Result<Vec<WorldMapMarker>, WorldMapError> {
        if self.phase != WorldMapPhase::Open {
            return Err(WorldMapError::ModeNotInteractive);
        }
        if !self.target_view.is_valid() || !self.display_view.is_valid() {
            return Err(WorldMapError::InvalidViewState);
        }
        if self.player.is_some_and(|player| !player.is_finite())
            || self.waypoint.is_some_and(|point| !point.is_finite())
            || npcs.iter().any(|npc| !npc.position.is_finite())
        {
            return Err(WorldMapError::NonFiniteInput);
        }

        let map_rect = self.map_draw_rect();
        if !map_rect.is_valid() {
            return Err(WorldMapError::InvalidViewState);
        }
        let mut markers = Vec::new();
        if !self.tutorial_active {
            for npc in npcs {
                if FUSION_LAIR_RECT.contains(npc.position)
                    || is_npc_outside_current_special_zone(npc.position, self.zone)
                {
                    continue;
                }
                let normalized = npc.position.normalized_xz();
                if !self.target_view.contains(normalized)
                    || !self.present_npc_types.contains(&npc.npc_type)
                {
                    continue;
                }
                // Local view remains tied to the player's 950 x 624 world-unit
                // neighborhood even when the map is panned away from them.
                if self.zoom == WorldMapZoom::Type4
                    && !self.player.is_some_and(|player| {
                        WorldMapZoneRect::new(
                            player.position.x - WORLD_MAP_PICTURE_RECT.width * 0.5,
                            player.position.z - WORLD_MAP_PICTURE_RECT.height * 0.5,
                            WORLD_MAP_PICTURE_RECT.width,
                            WORLD_MAP_PICTURE_RECT.height,
                        )
                        .contains(npc.position)
                    })
                {
                    continue;
                }
                // Presence is per NPC type, so it also admits unused placements
                // of a valid vendor. Only named world locations may show them.
                if !crate::legacy_world_location::legacy_world_location_name(
                    npc.position.x,
                    npc.position.z,
                )
                .is_some_and(|name| !name.contains("unknown"))
                {
                    continue;
                }
                let entry = match catalog.npc(npc.npc_type) {
                    WorldMapCatalogLookup::Missing => continue,
                    WorldMapCatalogLookup::Unique(entry) => entry,
                    WorldMapCatalogLookup::Ambiguous => {
                        return Err(WorldMapError::AmbiguousNpcCatalog(npc.npc_type));
                    }
                };
                let kind = match entry.mission {
                    WorldMapMissionAvailability::Advance
                    | WorldMapMissionAvailability::NewAndAdvance => {
                        Some(WorldMapMarkerKind::MissionAdvance {
                            npc_type: npc.npc_type,
                            display_name: entry.display_name,
                        })
                    }
                    WorldMapMissionAvailability::New => Some(WorldMapMarkerKind::MissionNew {
                        npc_type: npc.npc_type,
                        display_name: entry.display_name,
                    }),
                    _ if entry.map_icon > 0 => {
                        let map_icon = u8::try_from(entry.map_icon)
                            .ok()
                            .filter(|icon| *icon <= 34)
                            .ok_or(WorldMapError::InvalidNpcMapIcon {
                                npc_type: npc.npc_type,
                                map_icon: entry.map_icon,
                            })?;
                        Some(WorldMapMarkerKind::Npc {
                            npc_type: npc.npc_type,
                            map_icon,
                            display_name: entry.display_name,
                        })
                    }
                    _ => None,
                };
                if let Some(kind) = kind {
                    if self.zoom != WorldMapZoom::Type4
                        && !self
                            .preferences
                            .enabled_icons
                            .contains(&kind.legacy_icon_index())
                    {
                        continue;
                    }
                    markers.push(WorldMapMarker {
                        kind,
                        rect: project_marker_rect(normalized, self.display_view, map_rect),
                    });
                }
            }
        }

        if let Some(waypoint) = self.waypoint
            && WorldMapZone::for_point(waypoint) == Some(self.zone)
        {
            let player = self.player.ok_or(WorldMapError::MissingPlayerForWaypoint)?;
            let original = waypoint.normalized_xz();
            let mut clipped = original;
            let mut offscreen = false;
            if clipped.x < self.display_view.x {
                clipped.x = self.display_view.x;
                offscreen = true;
            }
            if clipped.y < self.display_view.y {
                clipped.y = self.display_view.y;
                offscreen = true;
            }
            if clipped.x > self.display_view.x + self.display_view.width {
                clipped.x = self.display_view.x + self.display_view.width;
                offscreen = true;
            }
            if clipped.y > self.display_view.y + self.display_view.height {
                clipped.y = self.display_view.y + self.display_view.height;
                offscreen = true;
            }
            let rect = project_marker_rect(clipped, self.display_view, map_rect);
            let kind = if offscreen {
                let center = rect.center();
                let map_center = map_rect.center();
                let dx = (center.x - map_center.x) / map_rect.width;
                let dy = (center.y - map_center.y) / map_rect.height;
                WorldMapMarkerKind::WaypointOffscreenArrow {
                    rotation_degrees: dy.atan2(dx) * 180.0 / WORLD_MAP_LEGACY_DEGREES_DIVISOR
                        + 90.0,
                }
            } else if waypoint.y > player.position.y + 5.0 {
                WorldMapMarkerKind::WaypointUp
            } else if waypoint.y < player.position.y - 5.0 {
                WorldMapMarkerKind::WaypointDown
            } else {
                WorldMapMarkerKind::Waypoint
            };
            markers.push(WorldMapMarker { kind, rect });
        }

        for point in &self.preferences.waypoints {
            let point_position = WorldMapPoint::new(point.x, 0.0, point.z);
            let normalized = point_position.normalized_xz();
            let clipped = WorldMapNormalizedPoint {
                x: normalized.x.clamp(
                    self.display_view.x,
                    self.display_view.x + self.display_view.width,
                ),
                y: normalized.y.clamp(
                    self.display_view.y,
                    self.display_view.y + self.display_view.height,
                ),
            };
            let rect = project_marker_rect(clipped, self.display_view, map_rect);
            let rotation_degrees = if clipped.x != normalized.x || clipped.y != normalized.y {
                let center = rect.center();
                let map_center = map_rect.center();
                Some(
                    ((center.y - map_center.y) / map_rect.height)
                        .atan2((center.x - map_center.x) / map_rect.width)
                        * 180.0
                        / WORLD_MAP_LEGACY_DEGREES_DIVISOR
                        + 90.0,
                )
            } else {
                None
            };
            markers.push(WorldMapMarker {
                kind: WorldMapMarkerKind::CustomWaypoint {
                    color: point.color,
                    rotation_degrees,
                },
                rect,
            });
        }
        if let Some(player) = self.player {
            let normalized = player.position.normalized_xz();
            if self.target_view.contains(normalized) {
                markers.push(WorldMapMarker {
                    kind: WorldMapMarkerKind::Player {
                        yaw_degrees: player.yaw_degrees,
                    },
                    rect: project_marker_rect(normalized, self.display_view, map_rect),
                });
            }
        }
        Ok(markers)
    }

    pub(super) fn zoom_from_focal(
        &mut self,
        direction: WorldMapZoomDirection,
        focal_x: f32,
        focal_y: f32,
    ) -> Result<WorldMapInputResult, WorldMapError> {
        if !focal_x.is_finite() || !focal_y.is_finite() {
            return Err(WorldMapError::NonFiniteInput);
        }
        let next = match (direction, self.zoom) {
            (WorldMapZoomDirection::In, WorldMapZoom::Type1) => WorldMapZoom::Type2,
            (WorldMapZoomDirection::In, WorldMapZoom::Type2) => WorldMapZoom::Type3,
            (WorldMapZoomDirection::Out, WorldMapZoom::Type3) => WorldMapZoom::Type2,
            (WorldMapZoomDirection::Out, WorldMapZoom::Type2) => WorldMapZoom::Type1,
            _ => return Ok(WorldMapInputResult::Boundary),
        };
        let old = self.target_view;
        let center_x = focal_x * old.width + old.x;
        let center_y = focal_y * old.height + old.y;
        let next_view = centered_and_clamped_view(self.zone, next, center_x, center_y);
        if !next_view.is_valid() {
            return Err(WorldMapError::InvalidViewState);
        }
        self.zoom = next;
        self.target_view = next_view;
        Ok(WorldMapInputResult::Changed)
    }

    pub(super) fn target_center(&self) -> Result<WorldMapNormalizedPoint, WorldMapError> {
        if !self.target_view.is_valid() {
            return Err(WorldMapError::InvalidViewState);
        }
        Ok(WorldMapNormalizedPoint {
            x: self.target_view.x + self.target_view.width * 0.5,
            y: self.target_view.y + self.target_view.height * 0.5,
        })
    }
}
