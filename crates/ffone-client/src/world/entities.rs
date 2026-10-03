use super::*;

/// Attaches one proven, non-rendering GLB primitive as an authored triangle
/// collider below an existing gameplay entity.
///
/// This is also used by scripted legacy props whose `MeshCollider` lived in
/// their KFM prefab rather than in a streamed world scene.
#[allow(clippy::too_many_arguments)]
pub fn spawn_pending_authored_model_collider(
    commands: &mut Commands,
    asset_server: &AssetServer,
    parent: Entity,
    name: impl Into<String>,
    model_path: impl Into<String>,
    mesh: usize,
    primitive: usize,
    transform: Transform,
    expected_vertex_count: usize,
    expected_index_count: usize,
) -> Entity {
    spawn_pending_authored_model_collider_with_cooking(
        commands,
        asset_server,
        parent,
        name,
        model_path,
        mesh,
        primitive,
        transform,
        expected_vertex_count,
        expected_index_count,
        AuthoredColliderCooking::PublishedGltfTriangleMesh,
    )
}

/// Attaches a continuous perimeter shell whose height comes from one proven
/// collider primitive while its XY boundary follows a second, visible
/// footprint primitive. The entity transform may rotate that local Z-up prism
/// into the runtime's Y-up world. Its lower edge is extended downward by the
/// collider's source Z span so the blocker remains closed across uneven terrain.
///
/// This is intentionally separate from ordinary authored triangle meshes:
/// concave world geometry must retain its exact triangles.
#[allow(clippy::too_many_arguments)]
pub fn spawn_pending_authored_model_perimeter_collider(
    commands: &mut Commands,
    asset_server: &AssetServer,
    parent: Entity,
    name: impl Into<String>,
    model_path: impl Into<String>,
    mesh: usize,
    primitive: usize,
    transform: Transform,
    expected_vertex_count: usize,
    expected_index_count: usize,
    footprint_mesh: usize,
    footprint_primitive: usize,
    footprint_local_scale: Vec2,
    expected_footprint_vertex_count: usize,
    expected_footprint_index_count: usize,
) -> Entity {
    let model_path = model_path.into();
    let source_mesh = asset_server
        .load(GltfAssetLabel::Primitive { mesh, primitive }.from_asset(model_path.clone()));
    let footprint_source_mesh = asset_server.load(
        GltfAssetLabel::Primitive {
            mesh: footprint_mesh,
            primitive: footprint_primitive,
        }
        .from_asset(model_path.clone()),
    );
    commands
        .spawn((
            Name::new(name.into()),
            ChildOf(parent),
            transform,
            PendingAuthoredTriMeshCollider {
                source_mesh,
                source_model_path: model_path.clone(),
                is_trigger: false,
                expected_vertex_count,
                expected_index_count,
                perimeter_footprint: Some(PendingAuthoredPerimeterFootprint {
                    source_mesh: footprint_source_mesh,
                    local_scale: footprint_local_scale,
                    expected_vertex_count: expected_footprint_vertex_count,
                    expected_index_count: expected_footprint_index_count,
                }),
                cooking: AuthoredColliderCooking::ConvexXyPrism,
            },
            SpawnedNativeWorldCollider {
                source_model_path: model_path.clone(),
                model_path,
            },
            NativeWorldColliderStatus::Loading,
            Visibility::Hidden,
        ))
        .id()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_pending_authored_model_collider_with_cooking(
    commands: &mut Commands,
    asset_server: &AssetServer,
    parent: Entity,
    name: impl Into<String>,
    model_path: impl Into<String>,
    mesh: usize,
    primitive: usize,
    transform: Transform,
    expected_vertex_count: usize,
    expected_index_count: usize,
    cooking: AuthoredColliderCooking,
) -> Entity {
    let model_path = model_path.into();
    let source_mesh = asset_server
        .load(GltfAssetLabel::Primitive { mesh, primitive }.from_asset(model_path.clone()));
    commands
        .spawn((
            Name::new(name.into()),
            ChildOf(parent),
            transform,
            PendingAuthoredTriMeshCollider {
                source_mesh,
                source_model_path: model_path.clone(),
                is_trigger: false,
                expected_vertex_count,
                expected_index_count,
                perimeter_footprint: None,
                cooking,
            },
            RuntimeAttachedAuthoredModelCollider,
            SpawnedNativeWorldCollider {
                source_model_path: model_path.clone(),
                model_path,
            },
            NativeWorldColliderStatus::Loading,
            Visibility::Hidden,
        ))
        .id()
}

pub(super) fn spawn_native_world_scene_root(
    commands: &mut Commands,
    scene: &NativeWorldScene,
    selection_scope: NativeWorldScope,
) -> Result<Entity, NativeWorldSceneError> {
    Ok(commands
        .spawn((
            Name::new(scene.name.clone()),
            NativeWorldSceneEntity,
            NativeWorldSceneRoot {
                name: scene.name.clone(),
                tile: scene.tile,
                scope: scene.scope.unwrap_or(NativeWorldScope::WorldMap),
                selection_scope,
            },
            NativeWorldVisualPresentationStatus::Loading,
            NativeWorldPresentationStatus::Loading,
            NativeWorldBehaviourStatus::Loading,
            NativeWorldObjectRangeGroups::from_scene(scene),
            scene.root.try_to_bevy("world root")?,
            // Terrain and GLB objects share the complete tile admission gate.
            Visibility::Hidden,
        ))
        .id())
}

pub(super) fn spawn_native_world_visual(
    commands: &mut Commands,
    asset_server: &AssetServer,
    scene: &NativeWorldScene,
    root: Entity,
    visual_index: usize,
    visual: &NativeWorldVisual,
) -> Result<Entity, NativeWorldSceneError> {
    let model = scene.model(&visual.model).ok_or_else(|| {
        NativeWorldSceneError::new(format!(
            "visual {:?} references missing model {:?}",
            visual.name, visual.model
        ))
    })?;
    let transform = visual
        .transform
        .try_to_bevy(&format!("visual {:?}", visual.name))?;
    if is_legacy_non_presenting_visual(scene, visual) {
        // The frozen primary set contains two proven non-presenting families:
        // alpha collision sheets and one additive-black/no-texture material.
        // The former are physics helpers; the latter contributes exact zero in
        // Unity. Loading either through a fallback shader produces the giant
        // foliage/black sheets reported in dense tiles. Retain metadata and
        // sibling colliders, but never request render geometry for them.
        return Ok(commands
            .spawn((
                Name::new(visual.name.clone()),
                NativeWorldSceneEntity,
                SpawnedNativeWorldVisual {
                    model_path: model.path.clone(),
                    source_model_path: visual.source_model_path.clone(),
                    scene: visual.scene,
                },
                NativeWorldVisualSceneReady,
                ChildOf(root),
                transform,
                Visibility::Hidden,
            ))
            .id());
    }
    if visual.scene != 0 {
        return Err(NativeWorldSceneError::new(format!(
            "visual {:?} uses scene {} but native static geometry requires scene 0",
            visual.name, visual.scene
        )));
    }
    let gltf = asset_server.load(model.path.clone());
    let mesh = asset_server.load(GltfAssetLabel::Mesh(0).from_asset(model.path.clone()));
    let mut entity = commands.spawn((
        Name::new(visual.name.clone()),
        NativeWorldSceneEntity,
        SpawnedNativeWorldVisual {
            model_path: model.path.clone(),
            source_model_path: visual.source_model_path.clone(),
            scene: visual.scene,
        },
        NativeWorldObjectRangeMember {
            scene_root: root,
            group: native_world_object_range_group_key(visual_index, &visual.name),
        },
        ChildOf(root),
        PendingNativeWorldVisualAsset { gltf, mesh },
        transform,
        Visibility::Hidden,
    ));
    if matches_primary_audited_smallstuff_presentation_identity(visual.legacy_layer, &model.path) {
        entity.insert(NativeWorldPrimaryFarPresentationGuard);
    }
    if let Some(water) = native_world_water_surface(scene, visual) {
        entity.insert(water);
    }
    Ok(entity.id())
}

pub(super) fn spawn_native_world_collider(
    commands: &mut Commands,
    asset_server: &AssetServer,
    scene: &NativeWorldScene,
    root: Entity,
    collider: &NativeWorldCollider,
) -> Result<Entity, NativeWorldSceneError> {
    let model = scene.model(&collider.model).ok_or_else(|| {
        NativeWorldSceneError::new(format!(
            "collider {:?} references missing model {:?}",
            collider.name, collider.model
        ))
    })?;
    let source_mesh = asset_server.load(
        GltfAssetLabel::Primitive {
            mesh: collider.mesh,
            primitive: collider.primitive,
        }
        .from_asset(model.path.clone()),
    );
    Ok(commands
        .spawn((
            Name::new(collider.name.clone()),
            NativeWorldSceneEntity,
            ChildOf(root),
            collider
                .transform
                .try_to_bevy(&format!("collider {:?}", collider.name))?,
            PendingAuthoredTriMeshCollider {
                source_mesh,
                source_model_path: model.path.clone(),
                is_trigger: collider.is_trigger,
                expected_vertex_count: collider.expected_vertex_count,
                expected_index_count: collider.expected_index_count,
                perimeter_footprint: None,
                cooking: native_world_collider_cooking(&model.root_name),
            },
            SpawnedNativeWorldCollider {
                model_path: model.path.clone(),
                source_model_path: collider.source_model_path.clone(),
            },
            NativeWorldColliderStatus::Loading,
            // This entity owns only the primitive asset used to cook physics;
            // it is never part of the visual presentation. Keep the entire
            // hierarchy hidden defensively so behaviour reparenting or a
            // future loader change cannot expose collision proxies.
            Visibility::Hidden,
        ))
        .id())
}

pub(super) fn spawn_native_world_terrain(
    commands: &mut Commands,
    scene: &NativeWorldScene,
    root: Entity,
    terrain: Arc<NativeTerrain>,
) -> Result<Entity, NativeWorldSceneError> {
    let instance = scene.native_terrain.as_ref().ok_or_else(|| {
        NativeWorldSceneError::new("streamed terrain payload has no scene terrain owner")
    })?;
    let mut terrain_parent = root;
    if let Some(root_chain) = &instance.root_chain {
        for node in root_chain.nodes.iter().rev() {
            let node_name = if node.game_object.true_name.is_empty() {
                format!("Transform_{}", node.transform_path_id)
            } else {
                node.game_object.true_name.clone()
            };
            terrain_parent = commands
                .spawn((
                    Name::new(node_name),
                    NativeWorldSceneEntity,
                    ChildOf(terrain_parent),
                    node.native_local_transform
                        .try_to_bevy("native terrain rootChain node")?,
                    Visibility::Inherited,
                ))
                .id();
        }
    }
    let entity = spawn_pending_native_heightmap(
        commands,
        terrain_parent,
        instance.name.clone(),
        instance
            .transform
            .try_to_bevy(&format!("native terrain {:?}", instance.name))?,
        terrain,
    );
    commands.entity(entity).insert(NativeWorldTerrainPresentation);
    Ok(entity)
}

/// Spawns only native Bevy data: GLB scenes plus collider primitives decoded
/// from those same GLBs. The root and child transforms are applied exactly as
/// authored in the JSON graph.
pub fn spawn_native_world_scene(
    commands: &mut Commands,
    asset_server: &AssetServer,
    catalog: &NativeWorldCatalog,
    scene: &NativeWorldScene,
) -> Result<SpawnedNativeWorldScene, NativeWorldSceneError> {
    // Published runtime scenes intentionally omit conversion-only provenance
    // and scene-instance documents. Their complete structural contract still
    // has to be validated before any entities are spawned.
    scene.validate_with_policy(false)?;
    let authored_scope = scene.scope.unwrap_or(NativeWorldScope::WorldMap);
    let root = spawn_native_world_scene_root(commands, scene, authored_scope)?;

    let mut visuals =
        Vec::with_capacity(scene.visuals.len() + usize::from(scene.native_terrain.is_some()));
    for (visual_index, visual) in scene.visuals.iter().enumerate() {
        visuals.push(spawn_native_world_visual(
            commands,
            asset_server,
            scene,
            root,
            visual_index,
            visual,
        )?);
    }

    let mut colliders =
        Vec::with_capacity(scene.colliders.len() + usize::from(scene.native_terrain.is_some()));
    for collider in &scene.colliders {
        colliders.push(spawn_native_world_collider(
            commands,
            asset_server,
            scene,
            root,
            collider,
        )?);
    }
    if let Some(instance) = &scene.native_terrain {
        let terrain = instance
            .loaded
            .clone()
            .map_or_else(|| catalog.load_terrain(instance), Ok)?;
        let entity = spawn_native_world_terrain(commands, scene, root, terrain)?;
        visuals.push(entity);
        colliders.push(entity);
    }

    Ok(SpawnedNativeWorldScene {
        root,
        visuals,
        colliders,
    })
}
