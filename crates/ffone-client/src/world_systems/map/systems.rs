use super::*;

pub const WORLD_MAP_ZOOM_TICK_RECTS: [WorldMapUiRect; 3] = [
    WorldMapUiRect::new(28.0, 110.0, 20.0, 7.0),
    WorldMapUiRect::new(28.0, 100.0, 20.0, 7.0),
    WorldMapUiRect::new(28.0, 90.0, 20.0, 7.0),
];

pub(super) fn sync_world_map_filter_visuals(
    presentation: Res<WorldMapPresentation>,
    assets: Res<WorldMapPresentationAssets>,
    mut images: ResMut<Assets<Image>>,
    mut variants: ResMut<crate::ui_icon_variants::UiIconVariants>,
    mut visuals: Query<(&WorldMapFilterVisual, &mut Node, &mut ImageNode)>,
) {
    if presentation.model.phase() != WorldMapPhase::Open || !presentation.model.show_filters() {
        return;
    }
    for (visual, mut node, mut image) in &mut visuals {
        let selected = presentation.model.filter_enabled(visual.index);
        let hovered = presentation.hover.control
            == Some(WorldMapPresentationControl::Filter(visual.index as u8));
        if visual.part == 0 {
            let alpha = if hovered {
                1.0
            } else if selected {
                0.333
            } else {
                0.1
            };
            if image.color.alpha() != alpha {
                image.color.set_alpha(alpha);
            }
        } else {
            let base =
                assets.image(WORLD_MAP_MARKER_PATHS[WORLD_MAP_FILTERS[visual.index].0 as usize]);
            let Some(size) = images.get(&base).map(|image| image.size_f32()) else {
                continue;
            };
            let size = if visual.part == 1 {
                size + Vec2::splat(2.0)
            } else {
                size
            };
            let rect =
                WorldMapUiRect::new((30.0 - size.x) * 0.5, (30.0 - size.y) * 0.5, size.x, size.y);
            let next = presentation_node(rect);
            if node.left != next.left
                || node.top != next.top
                || node.width != next.width
                || node.height != next.height
            {
                node.left = next.left;
                node.top = next.top;
                node.width = next.width;
                node.height = next.height;
            }
            if visual.part == 1 {
                let display = if selected || hovered {
                    Display::Flex
                } else {
                    Display::None
                };
                if node.display != display {
                    node.display = display;
                }
                if let Some(handle) = variants.image(
                    &base,
                    crate::ui_icon_variants::IconVariant::Outline,
                    &mut images,
                ) {
                    if image.image != handle {
                        image.image = handle;
                    }
                }
                let color = Color::srgb(48.0 / 255.0, 251.0 / 255.0, 251.0 / 255.0);
                if image.color != color {
                    image.color = color;
                }
            }
        }
    }
}

pub(super) fn apply_presentation_rect(node: &mut Node, rect: WorldMapUiRect) {
    node.left = px(rect.x);
    node.top = px(rect.y);
    node.width = px(rect.width);
    node.height = px(rect.height);
}

pub(super) fn apply_scaled_world_map_group(
    node: &mut Node,
    transform: &mut UiTransform,
    group: WorldMapScaledGroup,
) {
    node.left = px(group.node_left);
    node.top = px(group.node_top);
    node.width = px(group.source.width);
    node.height = px(group.source.height);
    transform.scale = Vec2::splat(group.scale);
}

pub(super) fn advance_world_map_scan_phases(
    animation: &mut WorldMapScanAnimation,
    map_height: f32,
    delta_seconds: f32,
) {
    if !map_height.is_finite()
        || map_height <= 0.0
        || !delta_seconds.is_finite()
        || delta_seconds <= 0.0
    {
        return;
    }
    animation.large_phase += delta_seconds * 0.15;
    if animation.large_phase > 1.0 + 64.0 / map_height {
        animation.large_phase = 0.0;
    }
    animation.small_phase += delta_seconds * 0.5;
    if animation.small_phase > 1.0 {
        animation.small_phase = 0.0;
    }
}

#[allow(clippy::type_complexity)]
pub(super) fn sync_world_map_presentation(
    presentation: Res<WorldMapPresentation>,
    animation: Res<WorldMapScanAnimation>,
    status: Res<WorldMapPresentationAssetStatus>,
    assets: Res<WorldMapPresentationAssets>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut elements: Query<
        (
            &mut Node,
            Option<&mut UiTransform>,
            Option<&mut ImageNode>,
            Option<&WorldMapPresentationRoot>,
            Option<&WorldMapPresentationBackdrop>,
            Option<&WorldMapPresentationWindow>,
            Option<&WorldMapPresentationNormalLayer>,
            Option<&WorldMapPresentationNoMap>,
            Option<&WorldMapPresentationBlack>,
            Option<&WorldMapPresentationMap>,
            Option<&WorldMapPresentationLine>,
            Option<&WorldMapPresentationLineEffect>,
        ),
        Or<(
            With<WorldMapPresentationRoot>,
            With<WorldMapPresentationBackdrop>,
            With<WorldMapPresentationWindow>,
            With<WorldMapPresentationNormalLayer>,
            With<WorldMapPresentationNoMap>,
            With<WorldMapPresentationBlack>,
            With<WorldMapPresentationMap>,
            With<WorldMapPresentationLine>,
            With<WorldMapPresentationLineEffect>,
        )>,
    >,
) {
    let layout = windows.single().ok().and_then(|window| {
        world_map_presentation_layout_with_scale(
            window.width(),
            window.height(),
            presentation.effective_ui_scale(window.height()),
        )
    });
    let valid = presentation.validate().is_ok();
    let ready = matches!(*status, WorldMapPresentationAssetStatus::Ready);
    let visible = ready && valid && presentation.model.phase() != WorldMapPhase::Closed;
    let normal_visible = visible && presentation.model.phase() == WorldMapPhase::Open;
    let no_map_visible = visible && presentation.model.phase() == WorldMapPhase::InstanceNoMap;

    for (
        mut node,
        transform,
        image,
        root,
        backdrop,
        window,
        normal,
        no_map,
        black,
        map,
        line,
        line_effect,
    ) in &mut elements
    {
        if root.is_some() {
            node.display = if visible {
                Display::Flex
            } else {
                Display::None
            };
            if let Some(layout) = layout {
                apply_presentation_rect(&mut node, layout.viewport);
            }
            continue;
        }
        if backdrop.is_some() {
            if let Some(layout) = layout
                && let Some(mut transform) = transform
            {
                apply_scaled_world_map_group(&mut node, &mut transform, layout.backdrop);
            }
            continue;
        }
        if window.is_some() {
            if let Some(layout) = layout
                && let Some(mut transform) = transform
            {
                apply_scaled_world_map_group(&mut node, &mut transform, layout.window);
            }
            continue;
        }
        if normal.is_some() {
            node.display = if normal_visible {
                Display::Flex
            } else {
                Display::None
            };
            continue;
        }
        if no_map.is_some() {
            node.display = if no_map_visible {
                Display::Flex
            } else {
                Display::None
            };
            continue;
        }
        if black.is_some() {
            node.display = if normal_visible && presentation.model.zoom() == WorldMapZoom::Type1 {
                Display::Flex
            } else {
                Display::None
            };
            apply_presentation_rect(&mut node, presentation.model.map_draw_rect());
            continue;
        }
        if map.is_some() {
            node.display = if normal_visible {
                Display::Flex
            } else {
                Display::None
            };
            apply_presentation_rect(&mut node, presentation.model.map_draw_rect());
            if let Some(mut image) = image
                && let Some(path) =
                    world_map_map_asset_path(presentation.model.zone(), presentation.model.zoom())
            {
                image.image = assets.image(path);
                // Graphics.DrawTexture's legacy Internal-GUITexture shader
                // defines (0.5, 0.5, 0.5) as its neutral vertex color. Clean
                // WorldMapMode therefore passes Color.gray without dimming
                // the RGB output. Bevy's ordinary image shader uses white as
                // the equivalent neutral multiplier.
                image.color = Color::WHITE;
                image.rect = world_map_source_rect(
                    presentation.model.display_view(),
                    world_map_texture_dimensions(path),
                );
            }
            continue;
        }
        if line.is_some() {
            node.display = if normal_visible {
                Display::Flex
            } else {
                Display::None
            };
            apply_presentation_rect(&mut node, presentation.model.map_draw_rect());
            continue;
        }
        if let Some(effect) = line_effect {
            let map_rect = presentation.model.map_draw_rect();
            let (rect, source_rect) = world_map_scan_effect_rect(map_rect, effect.0, *animation);
            node.display = if normal_visible && rect.height > 0.0 {
                Display::Flex
            } else {
                Display::None
            };
            apply_presentation_rect(&mut node, rect);
            if let Some(mut image) = image {
                image.rect = source_rect;
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub(super) fn sync_world_map_controls(
    presentation: Res<WorldMapPresentation>,
    assets: Res<WorldMapPresentationAssets>,
    mut controls: Query<
        (
            &mut Node,
            Option<&mut ImageNode>,
            Option<&mut TextColor>,
            Option<&WorldMapPresentationControlNode>,
            Option<&WorldMapPresentationControlLabel>,
            Option<&WorldMapPresentationZoomBar>,
        ),
        Or<(
            With<WorldMapPresentationControlNode>,
            With<WorldMapPresentationControlLabel>,
            With<WorldMapPresentationZoomBar>,
        )>,
    >,
) {
    let open = presentation.model.phase() == WorldMapPhase::Open;
    for (mut node, image, text_color, control_node, label, zoom_bar) in &mut controls {
        if zoom_bar.is_some() {
            node.display = if open && presentation.model.zone() == WorldMapZone::Other {
                Display::Flex
            } else {
                Display::None
            };
            continue;
        }
        let control = control_node
            .map(|node| node.0)
            .or_else(|| label.map(|label| label.0));
        let Some(control) = control else {
            continue;
        };
        let is_zoom_control = matches!(
            control,
            WorldMapPresentationControl::ZoomIn
                | WorldMapPresentationControl::ZoomOut
                | WorldMapPresentationControl::ZoomTick(_)
        );
        let control_visible = open
            && (!is_zoom_control || presentation.model.zone() == WorldMapZone::Other)
            && (!matches!(control, WorldMapPresentationControl::Filter(_))
                || (presentation.model.show_filters()
                    && presentation.model.zoom() != WorldMapZoom::Type4));
        node.display = if control_visible {
            Display::Flex
        } else {
            Display::None
        };
        let enabled = world_map_control_enabled(&presentation.model, control);
        let hovered = enabled && presentation.hover.control == Some(control);
        let selected = world_map_control_selected(&presentation.model, control);
        if let Some(mut image) = image {
            image.image = assets.image(world_map_control_asset_path(control, hovered, selected));
        }
        if let Some(mut color) = text_color {
            color.0 = if selected || hovered {
                Color::WHITE
            } else {
                Color::srgb(0.0, 0.580_392_2, 0.580_392_2)
            };
        }
    }
}

#[allow(clippy::type_complexity)]
pub(super) fn sync_world_map_text(
    presentation: Res<WorldMapPresentation>,
    mut text_nodes: Query<
        (
            &mut Node,
            Option<&mut LocalizedText>,
            Option<&WorldMapPresentationCurrentValue>,
            Option<&WorldMapPresentationTooltip>,
            Option<&WorldMapPresentationTooltipText>,
        ),
        Or<(
            With<WorldMapPresentationCurrentValue>,
            With<WorldMapPresentationTooltip>,
            With<WorldMapPresentationTooltipText>,
        )>,
    >,
) {
    let tooltip = presentation
        .hover
        .marker_index
        .and_then(|index| presentation.markers.get(index))
        .and_then(|marker| match &marker.kind {
            WorldMapMarkerKind::Npc {
                npc_type,
                display_name,
                ..
            }
            | WorldMapMarkerKind::MissionNew {
                npc_type,
                display_name,
            }
            | WorldMapMarkerKind::MissionAdvance {
                npc_type,
                display_name,
            } => (!display_name.is_empty())
                .then(|| localized_tabledata_npc_name(*npc_type, display_name)),
            WorldMapMarkerKind::CustomWaypoint { color, .. } => Some(
                LocalizedText::new("ui.world_map.custom_waypoint", "Custom Waypoint #{number}")
                    .with_arg("number", (color + 1).to_string()),
            ),
            _ => None,
        });
    let tooltip_visible = presentation.model.phase() == WorldMapPhase::Open && tooltip.is_some();

    for (mut node, localized, current_value, tooltip_node, tooltip_text) in &mut text_nodes {
        if current_value.is_some() {
            if let Some(mut localized) = localized {
                update_world_map_location_text(&mut localized, &presentation.current_location);
            }
            continue;
        }
        if tooltip_node.is_some() {
            node.display = if tooltip_visible {
                Display::Flex
            } else {
                Display::None
            };
            node.left = px(presentation.hover.pointer.x);
            node.top = px((presentation.hover.pointer.y - 10.0).max(0.0));
            continue;
        }
        if tooltip_text.is_some()
            && let Some(mut localized) = localized
        {
            if let Some(next) = tooltip.as_ref() {
                if *localized != *next {
                    *localized = next.clone();
                }
            } else {
                update_world_map_passthrough_text(&mut localized, "");
            }
        }
    }
}

pub(super) fn update_world_map_passthrough_text(localized: &mut LocalizedText, value: &str) {
    let next = world_map_passthrough_text(value);
    if *localized != next {
        *localized = next;
    }
}

pub(super) fn update_world_map_location_text(localized: &mut LocalizedText, value: &str) {
    let next = localized_world_location_text(value);
    if *localized != next {
        *localized = next;
    }
}

pub(super) fn sync_world_map_markers(
    mut commands: Commands,
    time: Res<Time>,
    mut variants: ResMut<crate::ui_icon_variants::UiIconVariants>,
    mut images: ResMut<Assets<Image>>,
    presentation: Res<WorldMapPresentation>,
    status: Res<WorldMapPresentationAssetStatus>,
    assets: Res<WorldMapPresentationAssets>,
    marker_layer: Query<Entity, With<WorldMapPresentationMarkerLayer>>,
    existing: Query<Entity, Or<(With<WorldMapPresentationMarker>, With<WorldMapDecoration>)>>,
) {
    if !presentation.is_changed() && !status.is_changed() && !images.is_changed() {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    if !matches!(*status, WorldMapPresentationAssetStatus::Ready)
        || presentation.model.phase() != WorldMapPhase::Open
        || presentation.validate().is_err()
    {
        return;
    }
    let Ok(layer) = marker_layer.single() else {
        return;
    };
    for (index, marker) in presentation.markers.iter().enumerate() {
        let icon_index = usize::from(marker.kind.legacy_icon_index());
        let rotation_degrees = match marker.kind {
            WorldMapMarkerKind::Player { yaw_degrees }
            | WorldMapMarkerKind::CustomWaypoint {
                rotation_degrees: Some(yaw_degrees),
                ..
            }
            | WorldMapMarkerKind::WaypointOffscreenArrow {
                rotation_degrees: yaw_degrees,
            } => yaw_degrees,
            _ => 0.0,
        };
        let mut image = assets.image(WORLD_MAP_MARKER_PATHS[icon_index]);
        let variant = match marker.kind {
            WorldMapMarkerKind::CustomWaypoint { color, .. } => {
                Some(crate::ui_icon_variants::IconVariant::Hue(
                    crate::map_preferences::WAYPOINT_HUES[usize::from(color)],
                ))
            }
            WorldMapMarkerKind::Npc { npc_type, .. }
                if presentation
                    .transport
                    .get(&npc_type)
                    .is_some_and(|node| !node.registered) =>
            {
                Some(crate::ui_icon_variants::IconVariant::Grayscale)
            }
            _ => None,
        };
        if let Some(variant) = variant {
            image = variants
                .image(&image, variant, &mut images)
                .unwrap_or(image);
        }
        let hovered = presentation.hover.marker_index == Some(index)
            && matches!(
                marker.kind,
                WorldMapMarkerKind::Npc { .. }
                    | WorldMapMarkerKind::MissionNew { .. }
                    | WorldMapMarkerKind::MissionAdvance { .. }
            );
        let mut rect = marker.rect;
        if hovered {
            rect.x -= 2.0;
            rect.y -= 2.0;
            rect.width += 4.0;
            rect.height += 4.0;
        }
        if matches!(marker.kind, WorldMapMarkerKind::Npc { map_icon: 25, .. })
            && presentation.respawn_position.is_some_and(|position| {
                let expected = project_marker_rect(
                    position.normalized_xz(),
                    presentation.model.display_view(),
                    presentation.model.map_draw_rect(),
                );
                (expected.x - marker.rect.x).abs() < 0.01
                    && (expected.y - marker.rect.y).abs() < 0.01
            })
        {
            let center = marker.rect.center();
            commands.spawn((
                WorldMapDecoration,
                WorldMapSpin,
                ChildOf(layer),
                presentation_node(WorldMapUiRect::new(
                    center.x - 14.0,
                    center.y - 14.0,
                    28.0,
                    28.0,
                )),
                stretch_image(assets.image("ui/en/world-map/markers/map_icon_25_active.png")),
                UiTransform::from_rotation(Rot2::radians(
                    time.elapsed_secs() * std::f32::consts::TAU,
                )),
                ZIndex(1),
                Pickable::IGNORE,
            ));
        }
        if hovered
            && let WorldMapMarkerKind::Npc {
                npc_type, map_icon, ..
            } = marker.kind
            && let Some(transport) = presentation.transport.get(&npc_type)
        {
            let rgb = match map_icon {
                21 => [224.0 / 255.0, 196.0 / 255.0, 1.0],
                22 => [174.0 / 255.0, 1.0, 183.0 / 255.0],
                _ => [1.0, 219.0 / 255.0, 195.0 / 255.0],
            };
            let pulse = ((time.elapsed_secs() * 6.0).sin() * 0.5 + 0.75).clamp(0.0, 1.0);
            let available_color = Color::srgba(rgb[0], rgb[1], rgb[2], pulse);
            let unavailable_color = Color::srgba(1.0, 1.0, 1.0, 0.2);
            let start = marker.rect.center();
            for (destination, registered) in &transport.destinations {
                let view = presentation.model.target_view();
                let normalized = if crate::legacy_world_location::legacy_world_location_name(
                    destination.x,
                    destination.z,
                )
                .is_none()
                {
                    WorldMapNormalizedPoint {
                        x: view.x,
                        y: view.y + view.height,
                    }
                } else if WorldMapZone::for_point(*destination) != Some(presentation.model.zone()) {
                    WorldMapNormalizedPoint {
                        x: view.x + view.width,
                        y: view.y,
                    }
                } else {
                    destination.normalized_xz()
                };
                let target = project_marker_rect(
                    normalized,
                    presentation.model.display_view(),
                    presentation.model.map_draw_rect(),
                )
                .center();
                let available = transport.registered && *registered;
                let thickness = 2.0
                    + match presentation.model.zoom() {
                        WorldMapZoom::Type1 => 0.0,
                        WorldMapZoom::Type2 => 1.0,
                        WorldMapZoom::Type3 => 2.0,
                        WorldMapZoom::Type4 => 3.0,
                    };
                for (a, b) in world_map_route_segments(
                    Vec2::new(start.x, start.y),
                    Vec2::new(target.x, target.y),
                    !available,
                    presentation.model.map_draw_rect(),
                ) {
                    let delta = b - a;
                    let middle = (a + b) * 0.5;
                    commands.spawn((
                        WorldMapDecoration,
                        WorldMapPulse(available),
                        ChildOf(layer),
                        presentation_node(WorldMapUiRect::new(
                            middle.x - delta.length() * 0.5,
                            middle.y - thickness * 0.5,
                            delta.length(),
                            thickness,
                        )),
                        BackgroundColor(if available {
                            available_color
                        } else {
                            unavailable_color
                        }),
                        UiTransform::from_rotation(Rot2::radians(delta.y.atan2(delta.x))),
                        ZIndex(1),
                        Pickable::IGNORE,
                    ));
                }
            }
            if let Some(outline) = variants.image(
                &image,
                crate::ui_icon_variants::IconVariant::Outline,
                &mut images,
            ) {
                let mut outlined = stretch_image(outline);
                outlined.color = if transport.registered {
                    available_color
                } else {
                    unavailable_color
                };
                commands.spawn((
                    WorldMapDecoration,
                    WorldMapPulse(transport.registered),
                    ChildOf(layer),
                    presentation_node(rect),
                    outlined,
                    ZIndex(4),
                    Pickable::IGNORE,
                ));
                // The source expands the texture by one pixel on every side before
                // stretching that combined 18x18 image into the hovered 20x20 rect.
                let border = rect.width / 18.0;
                rect.x += border;
                rect.y += border;
                rect.width -= border * 2.0;
                rect.height -= border * 2.0;
            }
        }
        let entity = commands
            .spawn((
                WorldMapPresentationMarker(index),
                presentation_node(rect),
                stretch_image(image),
                UiTransform::from_rotation(Rot2::radians(rotation_degrees.to_radians())),
                Pickable::IGNORE,
                ZIndex(if hovered { 4 } else { 2 }),
            ))
            .id();
        commands.entity(layer).add_child(entity);
    }
}
