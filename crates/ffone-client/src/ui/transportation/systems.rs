use super::*;

#[allow(clippy::type_complexity)]
pub(super) fn sync_transportation_shell(
    model: Res<TransportationModel>,
    status: Res<TransportationPresentationAssetStatus>,
    assets: Res<TransportationPresentationAssets>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut elements: Query<
        (
            &mut Node,
            Option<&mut ImageNode>,
            Option<&TransportationPresentationRoot>,
            Option<&TransportationPresentationBackdrop>,
            Option<&TransportationPresentationWindow>,
            Option<&TransportationPresentationMap>,
            Option<&TransportationPresentationMonkey>,
            Option<&TransportationPresentationNpcCameraSlot>,
            Option<&TransportationPresentationTurboLayer>,
            Option<&TransportationPresentationScrollThumb>,
            Option<&TransportationPresentationScrollbar>,
        ),
        Or<(
            With<TransportationPresentationRoot>,
            With<TransportationPresentationBackdrop>,
            With<TransportationPresentationWindow>,
            With<TransportationPresentationMap>,
            With<TransportationPresentationMonkey>,
            With<TransportationPresentationNpcCameraSlot>,
            With<TransportationPresentationTurboLayer>,
            With<TransportationPresentationScrollThumb>,
            With<TransportationPresentationScrollbar>,
        )>,
    >,
) {
    let ready = matches!(*status, TransportationPresentationAssetStatus::Ready);
    let browsing = model.phase() == TransportationPhase::Browsing;
    let viewport = windows
        .single()
        .ok()
        .map(|window| TransportationUiRect::new(0.0, 0.0, window.width(), window.height()));
    for (
        mut node,
        image,
        root,
        backdrop,
        window,
        map,
        monkey,
        camera,
        turbo,
        scroll_thumb,
        scrollbar,
    ) in &mut elements
    {
        if root.is_some() {
            node.display = if ready && browsing {
                Display::Flex
            } else {
                Display::None
            };
            if let Some(viewport) = viewport {
                apply_transportation_rect(&mut node, viewport);
            }
            continue;
        }
        if backdrop.is_some() {
            if let Some(viewport) = viewport {
                apply_transportation_rect(
                    &mut node,
                    TransportationUiRect::new(
                        (viewport.width - TRANSPORTATION_BACKDROP_RECT.width) * 0.5,
                        (viewport.height - TRANSPORTATION_BACKDROP_RECT.height) * 0.5,
                        TRANSPORTATION_BACKDROP_RECT.width,
                        TRANSPORTATION_BACKDROP_RECT.height,
                    ),
                );
            }
            continue;
        }
        if window.is_some() {
            if let Some(viewport) = viewport {
                apply_transportation_rect(
                    &mut node,
                    TransportationUiRect::new(
                        (viewport.width - TRANSPORTATION_WINDOW_RECT.width) * 0.5,
                        (viewport.height - TRANSPORTATION_WINDOW_RECT.height) * 0.5,
                        TRANSPORTATION_WINDOW_RECT.width,
                        TRANSPORTATION_WINDOW_RECT.height,
                    ),
                );
            }
            continue;
        }
        if map.is_some() {
            if let Some(mut image) = image {
                image.image = assets.image(model.map().asset_path());
                image.color = Color::srgb(0.5, 0.5, 0.5);
                image.rect = transportation_source_rect(model.display_view());
            }
            continue;
        }
        if monkey.is_some() {
            node.display = if model.service() == TransportationService::Warp {
                Display::None
            } else {
                Display::Flex
            };
            continue;
        }
        if camera.is_some() {
            node.display = if model.service() == TransportationService::Warp {
                Display::Flex
            } else {
                Display::None
            };
            continue;
        }
        if turbo.is_some() {
            node.display = if model.service() == TransportationService::Wyvern {
                Display::Flex
            } else {
                Display::None
            };
            continue;
        }
        if scroll_thumb.is_some() {
            let maximum = model.maximum_scroll_y();
            node.display = if maximum > 0.0 {
                Display::Flex
            } else {
                Display::None
            };
            let ratio = if maximum > 0.0 {
                model.scroll_y() / maximum
            } else {
                0.0
            };
            let content_height = model.routes().len() as f32 * TRANSPORTATION_ROUTE_STRIDE
                + TRANSPORTATION_ROUTE_CONTENT_PADDING;
            let thumb_height =
                (TRANSPORTATION_SELECT_RECT.height / content_height * 421.0).clamp(15.0, 421.0);
            node.top = px(12.0 + ratio * (421.0 - thumb_height));
            node.height = px(thumb_height);
            continue;
        }
        if scrollbar.is_some() {
            node.display = if model.maximum_scroll_y() > 0.0 {
                Display::Flex
            } else {
                Display::None
            };
        }
    }
}

pub(super) fn sync_transportation_controls(
    model: Res<TransportationModel>,
    assets: Res<TransportationPresentationAssets>,
    mut controls: Query<
        (
            &Interaction,
            &TransportationPresentationControlNode,
            Option<&mut ImageNode>,
        ),
        With<Button>,
    >,
    mut turbo_check: Query<
        &mut ImageNode,
        (
            With<TransportationPresentationTurboCheck>,
            Without<TransportationPresentationControlNode>,
        ),
    >,
    mut go_label: Query<
        &mut TextColor,
        (
            With<TransportationPresentationGoLabel>,
            Without<TransportationPresentationControlNode>,
        ),
    >,
) {
    let mut go_hovered = false;
    for (interaction, control, image) in &mut controls {
        if control.0 == TransportationPresentationControl::GoNow {
            go_hovered = matches!(*interaction, Interaction::Hovered | Interaction::Pressed);
        }
        let Some(mut image) = image else {
            continue;
        };
        image.image = match control.0 {
            TransportationPresentationControl::Close => {
                if matches!(*interaction, Interaction::Hovered | Interaction::Pressed) {
                    assets.image(WORLD_MAP_CLOSE_HOVER_PATH)
                } else {
                    assets.image(WORLD_MAP_CLOSE_PATH)
                }
            }
            TransportationPresentationControl::GoNow => {
                if matches!(*interaction, Interaction::Hovered | Interaction::Pressed) {
                    assets.image(TRANSPORTATION_BLUE_BUTTON_HOVER_PATH)
                } else {
                    assets.image(TRANSPORTATION_BLUE_BUTTON_PATH)
                }
            }
            TransportationPresentationControl::Turbo
            | TransportationPresentationControl::Route(_) => continue,
        };
    }
    if let Ok(mut check) = turbo_check.single_mut() {
        check.color = if model.turbo() {
            Color::WHITE
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.0)
        };
    }
    if let Ok(mut color) = go_label.single_mut() {
        color.0 = if go_hovered {
            Color::srgb(0.0, 0.278_431_4, 0.478_431_37)
        } else {
            Color::srgb(0.9, 0.9, 0.9)
        };
    }
}

#[allow(clippy::type_complexity)]
pub(super) fn sync_transportation_text(
    localization: Option<Res<Localization>>,
    language: Option<Res<Language>>,
    model: Res<TransportationModel>,
    copy: Res<TransportationUiCopy>,
    mut labels: Query<
        (
            &mut LocalizedText,
            Option<&TransportationPresentationTitle>,
            Option<&TransportationPresentationSubtitle>,
            Option<&TransportationPresentationWhereTo>,
            Option<&TransportationPresentationGoLabel>,
            Option<&TransportationPresentationTurboLabel>,
        ),
        Or<(
            With<TransportationPresentationTitle>,
            With<TransportationPresentationSubtitle>,
            With<TransportationPresentationWhereTo>,
            With<TransportationPresentationGoLabel>,
            With<TransportationPresentationTurboLabel>,
        )>,
    >,
) {
    if !model.is_changed() && !copy.is_changed()
        && !language.as_ref().is_some_and(|v| v.is_changed())
        && !localization.as_ref().is_some_and(|v| v.is_changed()) {
        return;
    }
    for (mut localized, title, subtitle, where_to, go, turbo) in &mut labels {
        *localized = if title.is_some() {
            LocalizedText::new("ui.transportation.title", copy.title.clone())
        } else if subtitle.is_some() {
            model
                .selected_route()
                .and_then(|route_index| model.routes().get(route_index))
                .map_or_else(transportation_empty_subtitle_text, |route| {
                    match (localization.as_deref(), language.as_deref()) {
                        (Some(localization), Some(language)) => transportation_subtitle_text(
                            &localization.text(language, &transportation_route_name_text(route)),
                            &localization.text(language, &transportation_route_region_text(&route.region)),
                        ),
                        _ => transportation_subtitle_text(&route.name, &route.region),
                    }
                })
        } else if where_to.is_some() {
            LocalizedText::new("ui.transportation.where_to", copy.where_to.clone())
        } else if go.is_some() {
            LocalizedText::new("ui.transportation.go_now", copy.go_now.clone())
        } else if turbo.is_some() {
            LocalizedText::new("ui.transportation.turbo_travel", copy.turbo_travel.clone())
        } else {
            continue;
        };
    }
}

pub(super) fn sync_transportation_routes(
    mut commands: Commands,
    model: Res<TransportationModel>,
    copy: Res<TransportationUiCopy>,
    status: Res<TransportationPresentationAssetStatus>,
    assets: Res<TransportationPresentationAssets>,
    mut route_layer: Query<(Entity, &mut Node), With<TransportationPresentationRouteLayer>>,
    existing: Query<Entity, With<TransportationPresentationDynamicRoute>>,
) {
    if !model.is_changed() && !copy.is_changed() && !status.is_changed() {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    if model.phase() != TransportationPhase::Browsing
        || !matches!(*status, TransportationPresentationAssetStatus::Ready)
    {
        return;
    }
    let Ok((layer, mut layer_node)) = route_layer.single_mut() else {
        return;
    };
    layer_node.top = px(-model.scroll_y());
    layer_node.height = px(model.routes().len() as f32 * TRANSPORTATION_ROUTE_STRIDE
        + TRANSPORTATION_ROUTE_CONTENT_PADDING);
    for (route_index, route) in model.routes().iter().enumerate() {
        spawn_transportation_route(
            &mut commands,
            layer,
            route_index,
            route,
            &model,
            &copy,
            &assets,
        );
    }
}

pub(super) fn sync_transportation_markers(
    mut commands: Commands,
    model: Res<TransportationModel>,
    status: Res<TransportationPresentationAssetStatus>,
    assets: Res<TransportationPresentationAssets>,
    marker_layer: Query<Entity, With<TransportationPresentationMarkerLayer>>,
    existing: Query<Entity, With<TransportationPresentationDynamicMarker>>,
) {
    if !model.is_changed() && !status.is_changed() {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    if model.phase() != TransportationPhase::Browsing
        || !matches!(*status, TransportationPresentationAssetStatus::Ready)
    {
        return;
    }
    let Ok(layer) = marker_layer.single() else {
        return;
    };
    for marker in model.marker_projections() {
        let local = TransportationUiRect::new(
            marker.rect.x - TRANSPORTATION_MAP_RECT.x,
            marker.rect.y - TRANSPORTATION_MAP_RECT.y,
            marker.rect.width,
            marker.rect.height,
        );
        let entity = commands
            .spawn((
                TransportationPresentationDynamicMarker,
                TransportationPresentationMarker(marker.kind),
                transportation_node(local),
                transportation_stretch_image(assets.image(marker.asset_path)),
                Pickable::IGNORE,
                ZIndex(if marker.kind == TransportationMarkerKind::Selected {
                    3
                } else {
                    2
                }),
            ))
            .id();
        commands.entity(layer).add_child(entity);
    }
}
