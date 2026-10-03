use super::*;

pub(super) fn centered_and_clamped_view(
    zone: WorldMapZone,
    zoom: WorldMapZoom,
    center_x: f32,
    center_y: f32,
) -> WorldMapViewRect {
    let (width, height) = zoom.range(zone);
    let mut view = WorldMapViewRect::new(
        center_x - width * 0.5,
        center_y - height * 0.5,
        width,
        height,
    );
    if view.x < 0.0 {
        view.x = 0.0;
    }
    if view.y < 0.0 {
        view.y = 0.0;
    }
    if view.x + width > 1.0 {
        view.x = 1.0 - width;
    }
    if view.y + height > 1.0 {
        view.y = 1.0 - height;
    }
    if let Some(bounds) = zone.bounds() {
        if view.x < bounds.x {
            view.x = bounds.x;
        }
        if view.y < bounds.y {
            view.y = bounds.y;
        }
        if view.x + width > bounds.x + bounds.width {
            view.x = bounds.x + bounds.width - width;
        }
        if view.y + height > bounds.y + bounds.height {
            view.y = bounds.y + bounds.height - height;
        }
    }
    view
}

pub(super) fn clamp_after_legacy_pan(view: &mut WorldMapViewRect, zone: WorldMapZone) {
    if view.y <= 0.0 {
        view.y = 0.0;
    }
    if zone == WorldMapZone::Future {
        let bounds = FUTURE_ZONE_RECT.normalized();
        if view.y < bounds.y {
            view.y = bounds.y;
        }
    } else if zone == WorldMapZone::DarkLand {
        let bounds = DARKLANDS_ZONE_RECT.normalized();
        if view.y < bounds.y {
            view.y = bounds.y;
        }
    }
    if view.y + view.height > 1.0 {
        view.y = 1.0 - view.height;
    }
    if zone == WorldMapZone::Future {
        let bounds = FUTURE_ZONE_RECT.normalized();
        if view.y + view.height > bounds.y + bounds.height {
            view.y = bounds.y + bounds.height - view.height;
        }
    } else if zone == WorldMapZone::DarkLand {
        let bounds = DARKLANDS_ZONE_RECT.normalized();
        if view.y + view.height > bounds.y + bounds.height {
            view.y = bounds.y + bounds.height - view.height;
        }
    }
    if view.x <= 0.0 {
        view.x = 0.0;
    }
    if zone == WorldMapZone::Future {
        let bounds = FUTURE_ZONE_RECT.normalized();
        if view.x < bounds.x {
            view.x = bounds.x;
        }
    } else if zone == WorldMapZone::DarkLand {
        let bounds = DARKLANDS_ZONE_RECT.normalized();
        if view.x < bounds.x {
            view.x = bounds.x;
        }
    }
    if view.x + view.width > 1.0 {
        view.x = 1.0 - view.width;
    }
    if zone == WorldMapZone::Future {
        let bounds = FUTURE_ZONE_RECT.normalized();
        if view.x + view.width > bounds.x + bounds.width {
            view.x = bounds.x + bounds.width - view.width;
        }
    } else if zone == WorldMapZone::DarkLand {
        let bounds = DARKLANDS_ZONE_RECT.normalized();
        if view.x + view.width > bounds.x + bounds.width {
            view.x = bounds.x + bounds.width - view.width;
        }
    }
}

pub(super) fn clamp_after_legacy_repeat(
    view: &mut WorldMapViewRect,
    zone: WorldMapZone,
    direction: WorldMapPanDirection,
) {
    match direction {
        WorldMapPanDirection::Down => {
            if view.y <= 0.0 {
                view.y = 0.0;
            }
            if zone == WorldMapZone::Future {
                let bounds = FUTURE_ZONE_RECT.normalized();
                if view.y < bounds.y {
                    view.y = bounds.y;
                }
            } else if zone == WorldMapZone::DarkLand {
                let bounds = DARKLANDS_ZONE_RECT.normalized();
                if view.y < bounds.y {
                    view.y = bounds.y;
                }
            }
        }
        WorldMapPanDirection::Up => {
            if view.y + view.height > 1.0 {
                view.y = 1.0 - view.height;
            }
            if zone == WorldMapZone::Future {
                let bounds = FUTURE_ZONE_RECT.normalized();
                if view.y + view.height > bounds.y + bounds.height {
                    view.y = bounds.y + bounds.height - view.height;
                }
            } else if zone == WorldMapZone::DarkLand {
                let bounds = DARKLANDS_ZONE_RECT.normalized();
                if view.y + view.height > bounds.y + bounds.height {
                    view.y = bounds.y + bounds.height - view.height;
                }
            }
        }
        WorldMapPanDirection::Left => {
            if view.x <= 0.0 {
                view.x = 0.0;
            }
            if zone == WorldMapZone::Future {
                let bounds = FUTURE_ZONE_RECT.normalized();
                if view.x < bounds.x {
                    view.x = bounds.x;
                }
            } else if zone == WorldMapZone::DarkLand {
                let bounds = DARKLANDS_ZONE_RECT.normalized();
                if view.x < bounds.x {
                    view.x = bounds.x;
                }
            }
        }
        WorldMapPanDirection::Right => {
            if view.x + view.width > 1.0 {
                view.x = 1.0 - view.width;
            }
            if zone == WorldMapZone::Future {
                let bounds = FUTURE_ZONE_RECT.normalized();
                if view.x + view.width > bounds.x + bounds.width {
                    view.x = bounds.x + bounds.width - view.width;
                }
            } else if zone == WorldMapZone::DarkLand {
                let bounds = DARKLANDS_ZONE_RECT.normalized();
                if view.x + view.width > bounds.x + bounds.width {
                    view.x = bounds.x + bounds.width - view.width;
                }
            }
        }
    }
}

pub(super) fn legacy_aspect_adjusted_rect(
    mut rect: WorldMapUiRect,
    texture_width: f32,
    texture_height: f32,
) -> WorldMapUiRect {
    if texture_width / rect.width > texture_height / rect.height {
        let extra = rect.height - texture_height * rect.width / texture_width;
        let leading = extra / 3.0;
        let trailing = extra - leading;
        rect.y += leading;
        rect.height -= trailing;
    } else {
        let extra = rect.width - texture_width * rect.height / texture_height;
        let leading = extra / 3.0;
        let trailing = extra - leading;
        rect.x += leading;
        rect.width -= trailing;
    }
    rect
}

pub(super) fn is_npc_outside_current_special_zone(
    position: WorldMapPoint,
    current_zone: WorldMapZone,
) -> bool {
    (FUTURE_ZONE_RECT.contains(position) && current_zone != WorldMapZone::Future)
        || (DARKLANDS_ZONE_RECT.contains(position) && current_zone != WorldMapZone::DarkLand)
        || (TUTORIAL_ZONE_RECT.contains(position) && current_zone != WorldMapZone::Tutorial)
}

pub const fn world_map_filter_rect(index: usize) -> WorldMapUiRect {
    WorldMapUiRect::new(553.0 + index as f32 * 32.0, 45.0, 30.0, 30.0)
}

/// Returns the exact clean `WorldMapMode.DoWindow` hit rectangle for a
/// presentation control. Keeping this mapping beside the renderer prevents
/// production input from drifting away from the serialized geometry.
#[must_use]
pub const fn world_map_control_rect(control: WorldMapPresentationControl) -> WorldMapUiRect {
    match control {
        WorldMapPresentationControl::Filter(index) => world_map_filter_rect(index as usize),
        WorldMapPresentationControl::Close => WORLD_MAP_CLOSE_RECT,
        WorldMapPresentationControl::Help => WORLD_MAP_HELP_RECT,
        WorldMapPresentationControl::Up => WORLD_MAP_UP_RECT,
        WorldMapPresentationControl::Down => WORLD_MAP_DOWN_RECT,
        WorldMapPresentationControl::Left => WORLD_MAP_LEFT_RECT,
        WorldMapPresentationControl::Right => WORLD_MAP_RIGHT_RECT,
        WorldMapPresentationControl::ZoomIn => WORLD_MAP_ZOOM_IN_RECT,
        WorldMapPresentationControl::ZoomOut => WORLD_MAP_ZOOM_OUT_RECT,
        WorldMapPresentationControl::ZoomTick(0) => WORLD_MAP_ZOOM_TICK_RECTS[0],
        WorldMapPresentationControl::ZoomTick(1) => WORLD_MAP_ZOOM_TICK_RECTS[1],
        WorldMapPresentationControl::ZoomTick(2) => WORLD_MAP_ZOOM_TICK_RECTS[2],
        WorldMapPresentationControl::ZoomTick(_) => WorldMapUiRect::new(0.0, 0.0, 0.0, 0.0),
        WorldMapPresentationControl::LocalView => WORLD_MAP_LOCAL_VIEW_RECT,
        WorldMapPresentationControl::WorldView => WORLD_MAP_WORLD_VIEW_RECT,
        WorldMapPresentationControl::ShowFilters => WORLD_MAP_MISSION_FINDER_RECT,
    }
}

/// Pure top-left-window-local hit test used by the production adapter.
///
/// The clean controls do not overlap, but the explicit order preserves the
/// same close/help/arrows/zoom/view/filter ordering used by `DoWindow`.
#[must_use]
pub fn world_map_control_at(point: WorldMapUiPoint) -> Option<WorldMapPresentationControl> {
    if !point.is_finite() {
        return None;
    }
    [
        WorldMapPresentationControl::Close,
        WorldMapPresentationControl::Help,
        WorldMapPresentationControl::Up,
        WorldMapPresentationControl::Down,
        WorldMapPresentationControl::Left,
        WorldMapPresentationControl::Right,
        WorldMapPresentationControl::ZoomIn,
        WorldMapPresentationControl::ZoomOut,
        WorldMapPresentationControl::ZoomTick(0),
        WorldMapPresentationControl::ZoomTick(1),
        WorldMapPresentationControl::ZoomTick(2),
        WorldMapPresentationControl::LocalView,
        WorldMapPresentationControl::WorldView,
        WorldMapPresentationControl::ShowFilters,
    ]
    .into_iter()
    .find(|control| world_map_control_rect(*control).contains(point))
}

/// Returns the top-most projected marker under a window-local point.
///
/// `WorldMapMode` assigns its tooltip while iterating the authored NPC array,
/// so the last overlapping marker wins. Reverse iteration reproduces that
/// visible ordering without coupling input to Bevy entities.
#[must_use]
pub fn world_map_marker_at(markers: &[WorldMapMarker], point: WorldMapUiPoint) -> Option<usize> {
    if !point.is_finite() {
        return None;
    }
    markers
        .iter()
        .rposition(|marker| marker.rect.contains(point))
}

/// Exact clean `cnGraphicOption.GetUiScale` result reached through
/// `FFGUIUtility.ScaleAroundPivot(ScreenPivot.Center)`.
#[must_use]
pub fn clean_world_map_ui_scale(viewport_height: f32) -> f32 {
    if !viewport_height.is_finite() || viewport_height <= 0.0 {
        return 1.0;
    }
    (viewport_height / WORLD_MAP_UI_SCALE_REFERENCE_HEIGHT * WORLD_MAP_UI_SCALE_NUDGE).max(1.0)
}

#[must_use]
pub fn world_map_presentation_layout(
    viewport_width: f32,
    viewport_height: f32,
) -> Option<WorldMapPresentationLayout> {
    world_map_presentation_layout_with_scale(
        viewport_width,
        viewport_height,
        clean_world_map_ui_scale(viewport_height),
    )
}

#[must_use]
pub fn world_map_presentation_layout_with_scale(
    viewport_width: f32,
    viewport_height: f32,
    ui_scale: f32,
) -> Option<WorldMapPresentationLayout> {
    if !viewport_width.is_finite()
        || !viewport_height.is_finite()
        || viewport_width <= 0.0
        || viewport_height <= 0.0
    {
        return None;
    }
    let width = legacy_world_map_screen_extent(viewport_width);
    let height = legacy_world_map_screen_extent(viewport_height);
    let viewport_width = width as f32;
    let viewport_height = height as f32;
    let pivot = WorldMapUiPoint::new(viewport_width * 0.5, viewport_height * 0.5);
    let scale = valid_world_map_ui_scale(ui_scale);
    let backdrop = WorldMapUiRect::new(
        ((width - WORLD_MAP_BACKDROP_RECT.width as i32) / 2) as f32,
        ((height - WORLD_MAP_BACKDROP_RECT.height as i32) / 2) as f32,
        WORLD_MAP_BACKDROP_RECT.width,
        WORLD_MAP_BACKDROP_RECT.height,
    );
    let window = WorldMapUiRect::new(
        (viewport_width - WORLD_MAP_WINDOW_RECT.width) * 0.5,
        (viewport_height - WORLD_MAP_WINDOW_RECT.height) * 0.5,
        WORLD_MAP_WINDOW_RECT.width,
        WORLD_MAP_WINDOW_RECT.height,
    );
    Some(WorldMapPresentationLayout {
        viewport: WorldMapUiRect::new(0.0, 0.0, viewport_width, viewport_height),
        pivot,
        scale,
        backdrop: scaled_world_map_group(backdrop, pivot, scale),
        window: scaled_world_map_group(window, pivot, scale),
    })
}

pub(super) fn legacy_world_map_screen_extent(value: f32) -> i32 {
    value.floor().min(i32::MAX as f32) as i32
}

pub(super) fn scaled_world_map_group(
    source: WorldMapUiRect,
    pivot: WorldMapUiPoint,
    scale: f32,
) -> WorldMapScaledGroup {
    let source_center = source.center();
    let painted_center = WorldMapUiPoint::new(
        pivot.x + (source_center.x - pivot.x) * scale,
        pivot.y + (source_center.y - pivot.y) * scale,
    );
    WorldMapScaledGroup {
        source,
        node_left: painted_center.x - source.width * 0.5,
        node_top: painted_center.y - source.height * 0.5,
        scale,
        painted: WorldMapUiRect::new(
            painted_center.x - source.width * scale * 0.5,
            painted_center.y - source.height * scale * 0.5,
            source.width * scale,
            source.height * scale,
        ),
    }
}

pub(super) fn animate_world_map_decorations(
    time: Res<Time>,
    mut visuals: Query<
        (
            Option<&WorldMapSpin>,
            Option<&WorldMapPulse>,
            Option<&mut UiTransform>,
            Option<&mut ImageNode>,
            Option<&mut BackgroundColor>,
        ),
        Or<(With<WorldMapSpin>, With<WorldMapPulse>)>,
    >,
    presentation: Res<WorldMapPresentation>,
) {
    if presentation.model.phase() != WorldMapPhase::Open {
        return;
    }
    let alpha = ((time.elapsed_secs() * 6.0).sin() * 0.5 + 0.75).clamp(0.0, 1.0);
    for (spin, pulse, transform, image, background) in &mut visuals {
        if spin.is_some()
            && let Some(mut transform) = transform
        {
            transform.rotation = Rot2::radians(time.elapsed_secs() * std::f32::consts::TAU);
        }
        if let Some(pulse) = pulse {
            let alpha = if pulse.0 { alpha } else { 0.2 };
            if let Some(mut image) = image {
                if image.color.alpha() != alpha {
                    image.color.set_alpha(alpha);
                }
            }
            if let Some(mut background) = background {
                if background.0.alpha() != alpha {
                    background.0.set_alpha(alpha);
                }
            }
        }
    }
}

pub(super) fn world_map_control_text(control: WorldMapPresentationControl) -> LocalizedText {
    match control {
        WorldMapPresentationControl::LocalView => {
            LocalizedText::new("ui.world_map.my_view", "MY VIEW")
        }
        WorldMapPresentationControl::WorldView => {
            LocalizedText::new("ui.world_map.world_view", "WORLD VIEW")
        }
        WorldMapPresentationControl::ShowFilters => {
            LocalizedText::new("ui.world_map.show_filters", "SHOW FILTERS")
        }
        _ => unreachable!("only text-bearing world-map controls use a text child"),
    }
}

pub(super) fn world_map_passthrough_text(value: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", value)
}

pub(super) fn presentation_node(rect: WorldMapUiRect) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(rect.x),
        top: px(rect.y),
        width: px(rect.width),
        height: px(rect.height),
        ..default()
    }
}

pub(super) fn presentation_text_node(
    rect: WorldMapUiRect,
    padding: f32,
    justify_content: JustifyContent,
    align_items: AlignItems,
) -> Node {
    Node {
        justify_content,
        align_items,
        padding: UiRect::all(px(padding)),
        overflow: Overflow::clip(),
        ..presentation_node(rect)
    }
}

pub(super) fn valid_world_map_scan_phase(phase: f32) -> f32 {
    if phase.is_finite() {
        phase.max(0.0)
    } else {
        0.0
    }
}

pub(super) fn world_map_scan_effect_rect(
    map_rect: WorldMapUiRect,
    effect_index: u8,
    animation: WorldMapScanAnimation,
) -> (WorldMapUiRect, Option<BevyRect>) {
    if effect_index != 0 {
        let top = (animation.small_phase * map_rect.height).min(map_rect.height - 4.0);
        return (
            WorldMapUiRect::new(map_rect.x, map_rect.y + top, map_rect.width, 4.0),
            None,
        );
    }

    let texture_height = 64.0;
    let mut top = animation.large_phase * map_rect.height - texture_height;
    let mut source_top = 0.0;
    let mut height = texture_height;
    if top < 0.0 {
        source_top = -top;
        height += top;
        top = 0.0;
    }
    if top > map_rect.height - height {
        height = map_rect.height - top;
    }
    let height = height.max(0.0);
    (
        WorldMapUiRect::new(map_rect.x, map_rect.y + top, map_rect.width, height),
        Some(BevyRect {
            min: Vec2::new(0.0, source_top),
            max: Vec2::new(16.0, source_top + height),
        }),
    )
}

pub(super) fn world_map_source_rect(
    view: WorldMapViewRect,
    dimensions: Option<(u32, u32)>,
) -> Option<BevyRect> {
    let (width, height) = dimensions?;
    let width = width as f32;
    let height = height as f32;
    Some(BevyRect {
        min: Vec2::new(
            (view.x * width).clamp(0.0, width),
            ((1.0 - view.y - view.height) * height).clamp(0.0, height),
        ),
        max: Vec2::new(
            ((view.x + view.width) * width).clamp(0.0, width),
            ((1.0 - view.y) * height).clamp(0.0, height),
        ),
    })
}

pub(super) fn world_map_control_enabled(model: &WorldMapModel, control: WorldMapPresentationControl) -> bool {
    if model.phase() != WorldMapPhase::Open {
        return false;
    }
    match control {
        WorldMapPresentationControl::Filter(_) => {
            model.show_filters() && model.zoom() != WorldMapZoom::Type4
        }
        WorldMapPresentationControl::ZoomIn
        | WorldMapPresentationControl::ZoomOut
        | WorldMapPresentationControl::ZoomTick(_) => {
            model.zone() == WorldMapZone::Other && model.zoom() != WorldMapZoom::Type4
        }
        WorldMapPresentationControl::LocalView => model.zoom() != WorldMapZoom::Type4,
        WorldMapPresentationControl::WorldView => {
            model.zoom() == WorldMapZoom::Type4 && model.zone() != WorldMapZone::Tutorial
        }
        WorldMapPresentationControl::Close
        | WorldMapPresentationControl::Help
        | WorldMapPresentationControl::Up
        | WorldMapPresentationControl::Down
        | WorldMapPresentationControl::Left
        | WorldMapPresentationControl::Right
        | WorldMapPresentationControl::ShowFilters => true,
    }
}
