use super::*;

pub(super) fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= EPSILON,
        "expected {expected}, got {actual}"
    );
}

pub(super) fn player(x: f32, y: f32, z: f32) -> WorldMapPlayer {
    WorldMapPlayer::new(WorldMapPoint::new(x, y, z), 37.0)
}

pub(super) fn open_other_local() -> WorldMapModel {
    let mut model = WorldMapModel::default();
    model
        .try_open(WorldMapOpenContext::gameplay(player(4096.0, 100.0, 4096.0)))
        .unwrap();
    model.pop_outbox();
    assert_eq!(
        model.select_local_view(WorldMapInputGates::default()),
        Ok(WorldMapInputResult::Changed)
    );
    for _ in 0..120 {
        model.advance_paint().unwrap();
    }
    model
}

#[test]
fn tutorial_lock_and_nonfinite_open_fail_without_mutation_or_outbox() {
    let mut model = WorldMapModel::default();
    let original_target = model.target_view();
    assert_eq!(
        model.try_open(WorldMapOpenContext {
            tutorial_locked: true,
            ..WorldMapOpenContext::gameplay(player(4096.0, 0.0, 4096.0))
        }),
        Err(WorldMapError::TutorialLocked)
    );
    assert_eq!(model.phase(), WorldMapPhase::Closed);
    assert_eq!(model.target_view(), original_target);
    assert_eq!(model.pop_outbox(), None);

    assert_eq!(
        model.try_open(WorldMapOpenContext::gameplay(player(f32::NAN, 0.0, 0.0,))),
        Err(WorldMapError::NonFiniteInput)
    );
    assert_eq!(model.phase(), WorldMapPhase::Closed);
    assert_eq!(model.pop_outbox(), None);
}

#[test]
fn escape_popup_and_key22_use_the_exact_close_gates() {
    let mut model = WorldMapModel::default();
    model
        .try_open(WorldMapOpenContext::gameplay(player(4096.0, 0.0, 4096.0)))
        .unwrap();
    model.pop_outbox();

    assert!(!model.try_close(
        WorldMapCloseInput::Escape,
        WorldMapInputGates {
            system_popup_open: false,
            escape_close_allowed: false,
        }
    ));
    assert!(!model.try_close(
        WorldMapCloseInput::MapKey22,
        WorldMapInputGates {
            system_popup_open: true,
            escape_close_allowed: true,
        }
    ));
    assert!(model.try_close(
        WorldMapCloseInput::Escape,
        WorldMapInputGates {
            system_popup_open: false,
            escape_close_allowed: true,
        }
    ));
    assert_eq!(model.pop_outbox(), Some(WorldMapOutboxEvent::ExitMode));

    model
        .try_open(WorldMapOpenContext::gameplay(player(4096.0, 0.0, 4096.0)))
        .unwrap();
    model.pop_outbox();
    assert!(model.try_close(
        WorldMapCloseInput::MapKey22,
        WorldMapInputGates {
            system_popup_open: false,
            escape_close_allowed: false,
        }
    ));
}

#[test]
fn world_zoom_has_three_ticks_and_type4_is_a_separate_local_view() {
    let mut model = WorldMapModel::default();
    model
        .try_open(WorldMapOpenContext::gameplay(player(4096.0, 0.0, 4096.0)))
        .unwrap();
    let center = WORLD_MAP_CLICK_RECT.center();

    assert_eq!(
        model.zoom_wheel_at(
            center,
            WorldMapZoomDirection::In,
            WorldMapInputGates::default()
        ),
        Ok(WorldMapInputResult::Changed)
    );
    assert_eq!(model.zoom(), WorldMapZoom::Type2);
    assert_eq!(
        model.zoom_wheel_at(
            center,
            WorldMapZoomDirection::In,
            WorldMapInputGates::default()
        ),
        Ok(WorldMapInputResult::Changed)
    );
    assert_eq!(model.zoom(), WorldMapZoom::Type3);
    assert_eq!(
        model.zoom_wheel_at(
            center,
            WorldMapZoomDirection::In,
            WorldMapInputGates::default()
        ),
        Ok(WorldMapInputResult::Boundary)
    );
    assert_eq!(
        model.select_zoom_tick(0, WorldMapInputGates::default()),
        Ok(WorldMapInputResult::Changed)
    );
    assert_eq!(model.zoom(), WorldMapZoom::Type1);
    assert_eq!(
        model.select_local_view(WorldMapInputGates::default()),
        Ok(WorldMapInputResult::Changed)
    );
    assert_eq!(model.zoom(), WorldMapZoom::Type4);
    assert_eq!(
        model.select_zoom_tick(2, WorldMapInputGates::default()),
        Ok(WorldMapInputResult::Ignored)
    );
    assert_eq!(
        model.select_world_view(WorldMapInputGates::default()),
        Ok(WorldMapInputResult::Changed)
    );
    assert_eq!(model.zoom(), WorldMapZoom::Type1);

    model.try_close(WorldMapCloseInput::MapKey22, WorldMapInputGates::default());
    model
        .try_open(WorldMapOpenContext::gameplay(player(500.0, 0.0, 500.0)))
        .unwrap();
    assert_eq!(model.zone(), WorldMapZone::Tutorial);
    assert_eq!(model.zoom(), WorldMapZoom::Type4);
    assert_eq!(
        model.select_world_view(WorldMapInputGates::default()),
        Ok(WorldMapInputResult::Ignored)
    );
}

#[test]
fn drag_repeat_arrows_and_paint_use_legacy_rates_and_clamps() {
    let mut model = open_other_local();
    let start = model.target_view();
    assert_eq!(
        model.begin_drag(WORLD_MAP_CLICK_RECT.center(), WorldMapInputGates::default()),
        Ok(WorldMapInputResult::Changed)
    );
    assert_eq!(
        model.drag_by(10.0, -4.0, 0.5, WorldMapInputGates::default()),
        Ok(WorldMapInputResult::Changed)
    );
    let dragged = model.target_view();
    assert_close(
        dragged.x,
        start.x - 10.0 / WORLD_MAP_PICTURE_RECT.width * start.width,
    );
    assert_close(
        dragged.y,
        start.y - 4.0 / WORLD_MAP_PICTURE_RECT.height * start.height,
    );
    assert_eq!(
        model.end_drag(WorldMapInputGates::default()),
        WorldMapInputResult::Changed
    );

    let before_arrow = model.target_view();
    assert_eq!(
        model.repeat_pan(
            WorldMapPanDirection::Right,
            1.0,
            WorldMapInputGates::default()
        ),
        Ok(WorldMapInputResult::Changed)
    );
    assert_close(
        model.target_view().x,
        before_arrow.x + WORLD_MAP_TYPE4_RANGE / 8.0,
    );

    let display_before = model.display_view();
    let target = model.target_view();
    model.advance_paint().unwrap();
    assert_close(
        model.display_view().x,
        target.x * 0.2 + display_before.x * 0.8,
    );

    let unchanged = model.target_view();
    assert_eq!(
        model.drag_by(f32::NAN, 0.0, 0.016, WorldMapInputGates::default()),
        Err(WorldMapError::NonFiniteInput)
    );
    assert_eq!(model.target_view(), unchanged);
}

#[test]
fn present_npc_updates_are_transactional_and_duplicates_fail_closed() {
    let mut model = WorldMapModel::default();
    model.apply_present_npc_types(true, 10, &[7, 9]).unwrap();
    let before = model.present_npc_types().clone();
    assert_eq!(
        model.apply_present_npc_types(false, 11, &[12, 9]),
        Err(WorldMapError::DuplicatePresentNpcType(9))
    );
    assert_eq!(model.present_npc_types(), &before);
    assert_eq!(model.last_npc_type_sync_time(), 10);

    model.apply_present_npc_types(true, 20, &[12, 13]).unwrap();
    assert_eq!(model.present_npc_types(), &BTreeSet::from([12, 13]));
    assert_eq!(model.last_npc_type_sync_time(), 20);
}

#[test]
fn vendor_filter_rejects_unmapped_ranger_sam_placements() {
    let mut model = open_other_local();
    model
        .select_world_view(WorldMapInputGates::default())
        .unwrap();
    model.preferences.enabled_icons = vec![4];
    model.apply_present_npc_types(true, 1, &[653]).unwrap();
    let catalog = TestCatalog {
        entries: BTreeMap::from([(
            653,
            catalog_entry("Ranger Sam", 4, WorldMapMissionAvailability::None),
        )]),
    };
    let document: serde_json::Value = serde_json::from_slice(
        &std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/game/data/missions/client-npc-waypoints.json"
        ))
        .unwrap(),
    )
    .unwrap();
    let sources: Vec<_> = document["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["npcType"].as_i64() == Some(653))
        .map(|row| {
            let position = &row["clientPosition"];
            WorldMapNpcSource {
                npc_type: 653,
                position: WorldMapPoint::new(
                    position[0].as_f64().unwrap() as f32,
                    position[1].as_f64().unwrap() as f32,
                    position[2].as_f64().unwrap() as f32,
                ),
            }
        })
        .collect();
    assert_eq!(sources.len(), 5);
    let markers = model.project_markers(&catalog, &sources).unwrap();
    assert_eq!(
        markers
            .iter()
            .filter(|marker| matches!(
                marker.kind,
                WorldMapMarkerKind::Npc { npc_type: 653, .. }
            ))
            .count(),
        1
    );
    for source in &sources[1..] {
        assert!(
            model
                .project_markers(&catalog, &[*source])
                .unwrap()
                .iter()
                .all(|marker| !matches!(marker.kind, WorldMapMarkerKind::Npc { .. }))
        );
    }
}

#[test]
fn local_map_pan_does_not_admit_distant_npcs() {
    let mut model = open_other_local();
    model.apply_present_npc_types(true, 1, &[10]).unwrap();
    let catalog = TestCatalog {
        entries: BTreeMap::from([(
            10,
            catalog_entry("Vendor", 4, WorldMapMissionAvailability::None),
        )]),
    };
    let source = [WorldMapNpcSource {
        npc_type: 10,
        position: WorldMapPoint::new(4800.0, 0.0, 4200.0),
    }];
    model.target_view = centered_and_clamped_view(
        model.zone,
        model.zoom,
        4800.0 / WORLD_MAP_EXTENT,
        4200.0 / WORLD_MAP_EXTENT,
    );
    model.display_view = model.target_view;
    assert!(
        model
            .target_view
            .contains(source[0].position.normalized_xz())
    );
    assert!(model.project_markers(&catalog, &source).unwrap().is_empty());
    model.set_player(player(4800.0, 100.0, 4200.0)).unwrap();
    assert!(matches!(
        model.project_markers(&catalog, &source).unwrap()[0].kind,
        WorldMapMarkerKind::Npc { npc_type: 10, .. }
    ));
}

#[test]
fn waypoint_vertical_variants_and_tutorial_npc_suppression_are_exact() {
    let catalog = TestCatalog::default();
    for (height, expected_icon) in [(106.0, 28), (94.0, 29), (105.0, 2), (95.0, 2)] {
        let mut model = open_other_local();
        model
            .set_waypoint(Some(WorldMapPoint::new(4096.0, height, 4096.0)))
            .unwrap();
        let markers = model.project_markers(&catalog, &[]).unwrap();
        assert_eq!(markers[0].kind.legacy_icon_index(), expected_icon);
    }

    let mut model = WorldMapModel::default();
    model
        .try_open(WorldMapOpenContext {
            tutorial_active: true,
            ..WorldMapOpenContext::gameplay(player(500.0, 0.0, 500.0))
        })
        .unwrap();
    model.apply_present_npc_types(true, 1, &[10]).unwrap();
    let catalog = TestCatalog {
        entries: BTreeMap::from([(
            10,
            catalog_entry("Tutorial NPC", 4, WorldMapMissionAvailability::New),
        )]),
    };
    let markers = model
        .project_markers(
            &catalog,
            &[WorldMapNpcSource {
                npc_type: 10,
                position: WorldMapPoint::new(500.0, 0.0, 500.0),
            }],
        )
        .unwrap();
    assert_eq!(markers.len(), 1);
    assert!(matches!(markers[0].kind, WorldMapMarkerKind::Player { .. }));
}

#[test]
fn presentation_geometry_routes_all_verified_views_and_crops_top_left_pixels() {
    let layout = world_map_presentation_layout(1_264.0, 681.0).unwrap();
    assert_eq!(
        layout.viewport,
        WorldMapUiRect::new(0.0, 0.0, 1_264.0, 681.0)
    );
    assert_eq!(
        layout.window.source,
        WorldMapUiRect::new(114.0, 13.5, 1_036.0, 654.0)
    );
    assert_eq!(
        layout.backdrop.source,
        WorldMapUiRect::new(-328.0, -379.0, 1_920.0, 1_440.0)
    );
    assert_eq!(layout.scale, 1.0);
    assert_eq!(layout.window.painted, layout.window.source);
    assert_eq!(layout.backdrop.painted, layout.backdrop.source);
    assert!(world_map_presentation_layout(f32::NAN, 681.0).is_none());
    assert!(world_map_presentation_layout(0.0, 681.0).is_none());

    let hd = world_map_presentation_layout(1_920.0, 1_080.0).unwrap();
    assert_close(hd.scale, 1.476_562_5);
    assert_eq!(
        hd.window.source,
        WorldMapUiRect::new(442.0, 213.0, 1_036.0, 654.0)
    );
    assert_close(hd.window.painted.x, 195.140_63);
    assert_close(hd.window.painted.y, 57.164_063);
    assert_close(
        hd.window.painted.width,
        WORLD_MAP_WINDOW_RECT.width * hd.scale,
    );
    assert_close(
        hd.window.painted.height,
        WORLD_MAP_WINDOW_RECT.height * hd.scale,
    );
    assert_eq!(
        hd.backdrop.source,
        WorldMapUiRect::new(0.0, -180.0, 1_920.0, 1_440.0)
    );
    assert_close(
        hd.backdrop.painted.x,
        hd.pivot.x - WORLD_MAP_BACKDROP_RECT.width * hd.scale * 0.5,
    );
    assert_close(
        hd.backdrop.painted.y,
        hd.pivot.y - WORLD_MAP_BACKDROP_RECT.height * hd.scale * 0.5,
    );
    assert_close(
        hd.backdrop.painted.width,
        WORLD_MAP_BACKDROP_RECT.width * hd.scale,
    );
    assert_close(
        hd.backdrop.painted.height,
        WORLD_MAP_BACKDROP_RECT.height * hd.scale,
    );
    let local = WorldMapUiPoint::new(25.0, 18.0);
    let screen = WorldMapUiPoint::new(
        hd.window.painted.x + local.x * hd.scale,
        hd.window.painted.y + local.y * hd.scale,
    );
    let round_trip = hd.window.screen_to_local(screen);
    assert_close(round_trip.x, local.x);
    assert_close(round_trip.y, local.y);

    let disabled = world_map_presentation_layout_with_scale(1_920.0, 1_080.0, 1.0).unwrap();
    assert_eq!(disabled.scale, 1.0);
    assert_eq!(disabled.window.painted, disabled.window.source);

    for (zone, zoom, path) in [
        (
            WorldMapZone::Other,
            WorldMapZoom::Type1,
            WORLD_MAP_PAYZONE_PATHS[0],
        ),
        (
            WorldMapZone::Other,
            WorldMapZoom::Type2,
            WORLD_MAP_PAYZONE_PATHS[1],
        ),
        (
            WorldMapZone::Other,
            WorldMapZoom::Type3,
            WORLD_MAP_PAYZONE_PATHS[2],
        ),
        (
            WorldMapZone::Other,
            WorldMapZoom::Type4,
            WORLD_MAP_PAYZONE_PATHS[3],
        ),
        (
            WorldMapZone::Future,
            WorldMapZoom::Type3,
            WORLD_MAP_FREEZONE_PATHS[0],
        ),
        (
            WorldMapZone::Future,
            WorldMapZoom::Type4,
            WORLD_MAP_FREEZONE_PATHS[1],
        ),
        (
            WorldMapZone::DarkLand,
            WorldMapZoom::Type3,
            WORLD_MAP_DARKLAND_PATHS[0],
        ),
        (
            WorldMapZone::DarkLand,
            WorldMapZoom::Type4,
            WORLD_MAP_DARKLAND_PATHS[1],
        ),
        (
            WorldMapZone::Tutorial,
            WorldMapZoom::Type4,
            WORLD_MAP_FREEZONE_PATHS[1],
        ),
    ] {
        assert_eq!(world_map_map_asset_path(zone, zoom), Some(path));
    }
    assert_eq!(
        world_map_map_asset_path(WorldMapZone::Tutorial, WorldMapZoom::Type3),
        None
    );

    let rect = world_map_source_rect(
        WorldMapViewRect::new(0.25, 0.125, 0.5, 0.25),
        Some((2_048, 2_048)),
    )
    .unwrap();
    assert_eq!(rect.min, Vec2::new(512.0, 1_280.0));
    assert_eq!(rect.max, Vec2::new(1_536.0, 1_792.0));
    assert!(world_map_source_rect(WorldMapViewRect::new(0.0, 0.0, 1.0, 1.0), None).is_none());
}

#[test]
fn scan_effects_preserve_clean_rates_and_edge_source_cropping() {
    let mut live = WorldMapScanAnimation::default();
    advance_world_map_scan_phases(&mut live, WORLD_MAP_PICTURE_RECT.height, 2.0);
    let (large_phase, small_phase) = live.phases();
    assert_close(large_phase, 0.3);
    assert_close(small_phase, 1.0);
    advance_world_map_scan_phases(&mut live, WORLD_MAP_PICTURE_RECT.height, 0.1);
    let (large_phase, small_phase) = live.phases();
    assert_close(large_phase, 0.315);
    assert_close(small_phase, 0.0);

    let animation = WorldMapScanAnimation::frozen(0.49, 0.43);
    let (large, large_source) =
        world_map_scan_effect_rect(WORLD_MAP_PICTURE_RECT, 0, animation);
    assert_close(large.x, WORLD_MAP_PICTURE_RECT.x);
    assert_close(large.y, WORLD_MAP_PICTURE_RECT.y + 241.76);
    assert_close(large.width, WORLD_MAP_PICTURE_RECT.width);
    assert_close(large.height, 64.0);
    let large_source = large_source.unwrap();
    assert_close(large_source.min.x, 0.0);
    assert_close(large_source.min.y, 0.0);
    assert_close(large_source.max.x, 16.0);
    assert_close(large_source.max.y, 64.0);

    let (small, small_source) =
        world_map_scan_effect_rect(WORLD_MAP_PICTURE_RECT, 1, animation);
    assert_close(small.y, WORLD_MAP_PICTURE_RECT.y + 268.32);
    assert_close(small.height, 4.0);
    assert!(small_source.is_none());

    let (entering, entering_source) = world_map_scan_effect_rect(
        WORLD_MAP_PICTURE_RECT,
        0,
        WorldMapScanAnimation::frozen(0.05, 0.0),
    );
    assert_close(entering.y, WORLD_MAP_PICTURE_RECT.y);
    assert_close(entering.height, 31.2);
    let entering_source = entering_source.unwrap();
    assert_close(entering_source.min.x, 0.0);
    assert_close(entering_source.min.y, 32.8);
    assert_close(entering_source.max.x, 16.0);
    assert_close(entering_source.max.y, 64.0);

    let (leaving, leaving_source) = world_map_scan_effect_rect(
        WORLD_MAP_PICTURE_RECT,
        0,
        WorldMapScanAnimation::frozen(1.05, 0.0),
    );
    assert_close(leaving.y, WORLD_MAP_PICTURE_RECT.y + 591.2);
    assert_close(leaving.height, 32.8);
    let leaving_source = leaving_source.unwrap();
    assert_close(leaving_source.min.x, 0.0);
    assert_close(leaving_source.min.y, 0.0);
    assert_close(leaving_source.max.x, 16.0);
    assert_close(leaving_source.max.y, 32.8);
}

#[test]
fn production_hit_testing_reuses_exact_control_rects_and_topmost_marker_order() {
    assert_eq!(
        world_map_control_at(WORLD_MAP_CLOSE_RECT.center()),
        Some(WorldMapPresentationControl::Close)
    );
    assert_eq!(
        world_map_control_at(WORLD_MAP_ZOOM_TICK_RECTS[2].center()),
        Some(WorldMapPresentationControl::ZoomTick(2))
    );
    assert_eq!(
        world_map_control_at(WORLD_MAP_MISSION_FINDER_RECT.center()),
        Some(WorldMapPresentationControl::ShowFilters)
    );
    assert_eq!(
        world_map_control_at(WorldMapUiPoint::new(f32::NAN, 0.0)),
        None
    );
    assert_eq!(
        world_map_control_rect(WorldMapPresentationControl::ZoomTick(9)),
        WorldMapUiRect::new(0.0, 0.0, 0.0, 0.0)
    );

    let overlapping = vec![
        WorldMapMarker {
            kind: WorldMapMarkerKind::Npc {
                npc_type: 1,
                map_icon: 1,
                display_name: "first".to_owned(),
            },
            rect: WorldMapUiRect::new(20.0, 20.0, 16.0, 16.0),
        },
        WorldMapMarker {
            kind: WorldMapMarkerKind::Npc {
                npc_type: 2,
                map_icon: 2,
                display_name: "last".to_owned(),
            },
            rect: WorldMapUiRect::new(20.0, 20.0, 16.0, 16.0),
        },
    ];
    assert_eq!(
        world_map_marker_at(&overlapping, WorldMapUiPoint::new(28.0, 28.0)),
        Some(1)
    );
    assert_eq!(
        world_map_marker_at(&overlapping, WorldMapUiPoint::new(100.0, 100.0)),
        None
    );
}

#[test]
fn presentation_control_assets_are_deterministic_and_type4_disables_zoom_ticks() {
    let mut model = WorldMapModel::default();
    model
        .try_open(WorldMapOpenContext::gameplay(player(
            4_096.0, 100.0, 4_096.0,
        )))
        .unwrap();
    assert!(world_map_control_selected(
        &model,
        WorldMapPresentationControl::ZoomTick(0)
    ));
    assert!(world_map_control_selected(
        &model,
        WorldMapPresentationControl::WorldView
    ));
    assert!(world_map_control_selected(
        &model,
        WorldMapPresentationControl::ShowFilters
    ));
    assert_eq!(
        world_map_control_asset_path(WorldMapPresentationControl::Help, true, false),
        WORLD_MAP_HELP_HOVER_PATH
    );
    assert_eq!(
        world_map_control_asset_path(WorldMapPresentationControl::ZoomTick(0), true, true),
        WORLD_MAP_ZOOM_TICK_SELECTED_PATH
    );
    model
        .select_local_view(WorldMapInputGates::default())
        .unwrap();
    assert!(world_map_control_selected(
        &model,
        WorldMapPresentationControl::LocalView
    ));
    assert!(!world_map_control_enabled(
        &model,
        WorldMapPresentationControl::ZoomIn
    ));
    assert!(world_map_control_enabled(
        &model,
        WorldMapPresentationControl::WorldView
    ));
}

#[test]
fn presentation_plugin_registers_preload_tree_and_stays_fail_closed() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game")
        .canonicalize()
        .unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .add_plugins(WorldMapPresentationPlugin);
    app.init_asset::<Image>().init_asset::<Font>();
    app.update();

    assert!(!matches!(
        *app.world().resource::<WorldMapPresentationAssetStatus>(),
        WorldMapPresentationAssetStatus::Ready
    ));
    let assets = app.world().resource::<WorldMapPresentationAssets>();
    assert_eq!(assets.images.len(), WORLD_MAP_SEMANTIC_ASSET_FILES);
    let mut roots = app
        .world_mut()
        .query_filtered::<(&Node, &Pickable, &GlobalZIndex), With<WorldMapPresentationRoot>>();
    let (root, pickable, z_index) = roots.single(app.world()).unwrap();
    assert_eq!(root.display, Display::None);
    assert!(pickable.should_block_lower);
    assert!(!pickable.is_hoverable);
    assert_eq!(*z_index, GlobalZIndex(WORLD_MAP_UI_Z_INDEX));
}
