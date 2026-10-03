use super::*;
use bevy::ecs::system::RunSystemOnce;

fn presentation_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .init_resource::<Assets<StandardMaterial>>()
        .init_resource::<NativeWorldStreamingStatus>();
    app
}

fn root(app: &mut App, tile: [i32; 2], scope: NativeWorldScope) -> Entity {
    app.world_mut()
        .spawn((
            NativeWorldSceneRoot {
                name: "presentation test".into(),
                tile,
                scope,
                selection_scope: scope,
            },
            NativeWorldBehaviourStatus::Ready,
            NativeWorldVisualPresentationStatus::Loading,
            NativeWorldPresentationStatus::Loading,
            Visibility::Hidden,
        ))
        .id()
}

#[test]
fn tile_waits_for_terrain_and_all_grass_then_reveals_without_delay() {
    let mut app = presentation_app();
    let root = root(&mut app, [0, 0], NativeWorldScope::WorldMap);
    let terrain = app
        .world_mut()
        .spawn((
            ChildOf(root),
            NativeWorldTerrainPresentation,
            NativeWorldColliderStatus::Loading,
        ))
        .id();
    app.world_mut()
        .run_system_once(reveal_native_world_scenes)
        .unwrap();
    assert_eq!(
        app.world().get::<Visibility>(root),
        Some(&Visibility::Hidden)
    );
    app.world_mut().entity_mut(terrain).insert((
        NativeWorldColliderStatus::Ready {
            vertex_count: 3,
            index_count: 3,
        },
        NativeTerrainDetailStatus::Loading {
            rendered_layers: 0,
            total_layers: 1,
            rendered_instances: 0,
            total_instances: 10,
        },
    ));
    app.world_mut()
        .run_system_once(reveal_native_world_scenes)
        .unwrap();
    assert_eq!(
        app.world().get::<NativeWorldPresentationStatus>(root),
        Some(&NativeWorldPresentationStatus::Loading)
    );
    app.world_mut()
        .entity_mut(terrain)
        .insert(NativeTerrainDetailStatus::Ready {
            rendered_layers: 1,
            rendered_instances: 10,
        });
    app.world_mut()
        .run_system_once(reveal_native_world_scenes)
        .unwrap();
    assert_eq!(
        app.world().get::<Visibility>(root),
        Some(&Visibility::Inherited)
    );
    assert_eq!(
        app.world().get::<NativeWorldPresentationStatus>(root),
        Some(&NativeWorldPresentationStatus::Ready)
    );
}

#[test]
fn grass_texture_is_required_even_after_detail_chunks_are_complete() {
    let mut app = presentation_app();
    let root = root(&mut app, [0, 0], NativeWorldScope::WorldMap);
    let terrain = app
        .world_mut()
        .spawn((
            ChildOf(root),
            NativeWorldTerrainPresentation,
            NativeWorldColliderStatus::Ready {
                vertex_count: 3,
                index_count: 3,
            },
            NativeTerrainDetailStatus::Ready {
                rendered_layers: 1,
                rendered_instances: 10,
            },
        ))
        .id();
    let texture = app
        .world()
        .resource::<AssetServer>()
        .load::<Image>("missing-bug33-detail.png");
    let material = app
        .world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color_texture: Some(texture),
            ..default()
        });
    let grass = app
        .world_mut()
        .spawn((ChildOf(terrain), MeshMaterial3d(material)))
        .id();
    app.world_mut()
        .run_system_once(reveal_native_world_scenes)
        .unwrap();
    assert_eq!(
        app.world().get::<Visibility>(root),
        Some(&Visibility::Hidden)
    );
    app.world_mut()
        .entity_mut(grass)
        .remove::<MeshMaterial3d<StandardMaterial>>();
    app.world_mut()
        .run_system_once(reveal_native_world_scenes)
        .unwrap();
    assert_eq!(
        app.world().get::<Visibility>(root),
        Some(&Visibility::Inherited)
    );
}

#[test]
fn terrain_without_authored_grass_does_not_block_the_location() {
    let mut app = presentation_app();
    let root = root(&mut app, [0, 0], NativeWorldScope::WorldMap);
    app.world_mut().spawn((
        ChildOf(root),
        NativeWorldTerrainPresentation,
        NativeWorldColliderStatus::Ready {
            vertex_count: 3,
            index_count: 3,
        },
        NativeTerrainDetailStatus::SourceContractPending,
    ));
    app.world_mut()
        .run_system_once(reveal_native_world_scenes)
        .unwrap();
    assert_eq!(
        app.world().get::<NativeWorldPresentationStatus>(root),
        Some(&NativeWorldPresentationStatus::Ready)
    );
}

#[test]
fn neighboring_water_waits_for_current_location_and_resets_on_warp() {
    let mut app = presentation_app();
    let catalog =
        load_native_world_scenes(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"))
            .unwrap();
    let position = Vec3::new(-1845.7359, -57.24495, 2720.1978);
    let scope = NativeWorldScope::WorldMap;
    let tiles = catalog.legacy_stream_target_tiles(scope, position);
    assert!(!tiles.is_empty());
    let mut waters = Vec::new();
    let mut supplement = None;
    for (scene, indices) in neighboring_water_scenes(&catalog, scope, position) {
        let owner = app
            .world_mut()
            .spawn(NativeWorldNeighborWaterRoot {
                tile: scene.tile,
                selection_scope: scope,
            })
            .id();
        for _ in indices {
            let entity = app
                .world_mut()
                .spawn((
                    ChildOf(owner),
                    NativeWorldNeighborWaterVisual,
                    NativeWorldVisualSceneReady,
                    LegacyMaterialApplied,
                    Visibility::Hidden,
                ))
                .id();
            if !tiles.contains(&scene.tile) {
                supplement = Some(entity);
            }
            waters.push(entity);
        }
    }
    assert!(!waters.is_empty(), "fixture must contain neighboring water");
    app.insert_resource(catalog)
        .init_resource::<NativeWorldLocationPresentation>();
    let player = app
        .world_mut()
        .spawn(Transform::from_translation(position))
        .id();
    app.world_mut().spawn(LegacyOrbitCamera::new(player));
    app.world_mut()
        .run_system_once(reveal_neighboring_water_surfaces)
        .unwrap();
    assert!(
        !app.world()
            .resource::<NativeWorldLocationPresentation>()
            .ready
    );
    for water in &waters {
        assert_eq!(
            app.world().get::<Visibility>(*water),
            Some(&Visibility::Hidden)
        );
    }
    for tile in tiles {
        let entity = root(&mut app, tile, scope);
        app.world_mut()
            .entity_mut(entity)
            .insert(NativeWorldVisualPresentationStatus::Ready);
    }
    app.world_mut()
        .run_system_once(reveal_neighboring_water_surfaces)
        .unwrap();
    assert!(
        app.world()
            .resource::<NativeWorldLocationPresentation>()
            .ready
    );
    {
        let entity =
            supplement.expect("City Hall fixture must include water outside full target tiles");
        assert_eq!(
            app.world().get::<Visibility>(entity),
            Some(&Visibility::Inherited)
        );
        app.world_mut()
            .entity_mut(entity)
            .remove::<LegacyMaterialApplied>();
        app.world_mut()
            .run_system_once(reveal_neighboring_water_surfaces)
            .unwrap();
        assert!(
            !app.world()
                .resource::<NativeWorldLocationPresentation>()
                .ready
        );
        assert_eq!(
            app.world().get::<Visibility>(entity),
            Some(&Visibility::Hidden)
        );
        app.world_mut()
            .entity_mut(entity)
            .insert(LegacyMaterialApplied);
        app.world_mut()
            .run_system_once(reveal_neighboring_water_surfaces)
            .unwrap();
        assert!(
            app.world()
                .resource::<NativeWorldLocationPresentation>()
                .ready
        );
    }
    // Water covered by a full tile belongs to that tile; supplements elsewhere
    // can reveal only once the complete target set is present.
    app.world_mut()
        .entity_mut(player)
        .insert(Transform::from_xyz(-2933.0, -50.7, 5467.0));
    app.world_mut()
        .run_system_once(reveal_neighboring_water_surfaces)
        .unwrap();
    assert!(
        !app.world()
            .resource::<NativeWorldLocationPresentation>()
            .ready
    );
    for water in waters {
        assert_eq!(
            app.world().get::<Visibility>(water),
            Some(&Visibility::Hidden)
        );
    }
}
