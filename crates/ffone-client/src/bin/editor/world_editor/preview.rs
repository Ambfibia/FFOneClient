use super::*;
use ffone_client::{
    coordinates::ProtocolYawDegrees,
    native_terrain::{NativeTerrain, spawn_pending_native_heightmap},
};
use std::sync::{Arc, Mutex, mpsc};
#[path = "preview_terrain.rs"]
mod terrain;
#[path = "preview_collision.rs"]
mod collision;

#[derive(Default, Resource)]
pub(super) struct WorldPreview {
    root: Option<Entity>,
    pub(super) objects: BTreeMap<String, Entity>,
    signatures: BTreeMap<String, String>,
    pub(super) terrains: BTreeMap<String, Entity>,
    pub(super) terrain_data: BTreeMap<String, Arc<NativeTerrain>>,
    pub(super) terrain_applied: BTreeMap<String, u64>,
    terrain_jobs: Vec<TerrainJob>,
    terrain_failed: BTreeSet<String>,
    terrain_region: Option<[i32; 2]>,
    pending: bool,
    collision_material: Option<Handle<StandardMaterial>>,
    pub(super) retiring: Vec<Entity>,
}
impl WorldPreview {
    #[cfg(test)]
    pub(super) fn set_test_root(&mut self, root: Entity) {
        self.root = Some(root);
    }
    pub(super) fn reset(&mut self, commands: &mut Commands) {
        if let Some(root) = self.root.take() {
            commands.entity(root).insert(Visibility::Hidden);
            self.retiring.push(root);
        }
        let retiring = std::mem::take(&mut self.retiring);
        let collision_material = self.collision_material.take();
        *self = Self { retiring, collision_material, ..default() };
    }
    pub(super) fn loaded_count(&self) -> usize {
        self.terrains.len()
    }
}
struct TerrainJob {
    tile: String,
    transform: Transform,
    receiver: Mutex<mpsc::Receiver<Result<NativeTerrain, String>>>,
}
#[derive(Component)]
struct WorldRoot;
#[derive(Component)]
pub(super) struct SelectionMarker(bool);

pub(super) fn scene(
    mut commands: Commands,
    state: Res<EditorState>,
    mut e: ResMut<WorldEditor>,
    mut preview: ResMut<WorldPreview>,
    server: Res<AssetServer>,
    catalog: Res<EditorCatalog>,
    mut rig: ResMut<NativePlayerRigAssetCache>,
    mut transforms: Query<&mut Transform, Without<PreviewRoot>>,
    mut visibility: Query<&mut Visibility>,
    model_roots: Query<Entity, With<PreviewRoot>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    selection: Query<(Entity, &SelectionMarker)>,
    mut last: Local<(u64, u64, Option<usize>, Option<bool>, [u32; 4])>,
) {
    let active = state.world_open == Some(true) && !e.map_picker;
    for root in &model_roots {
        if let Ok(mut v) = visibility.get_mut(root) {
            *v = if state.world_open.is_some() {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
        }
    }
    if let Some(root) = preview.root {
        if let Ok(mut v) = visibility.get_mut(root) {
            *v = if active {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
    if !active {
        return;
    }
    let next = (
        e.geometry_revision,
        e.revision,
        e.selected,
        state.world_open,
        [
            e.center.x.to_bits(),
            e.center.y.to_bits(),
            e.center.z.to_bits(),
            e.distance.to_bits(),
        ],
    );
    if *last == next && preview.terrain_jobs.is_empty() && !preview.pending {
        return;
    }
    *last = next;
    let root = if let Some(root) = preview.root {
        root
    } else {
        let root = commands
            .spawn((WorldRoot, Transform::IDENTITY, Visibility::Inherited))
            .id();
        preview.root = Some(root);
        root
    };
    let region: BTreeSet<_> = e.desired_tiles().into_iter().collect();
    let mut desired = BTreeSet::new();
    let mut spawn_budget = 128;
    preview.pending = false;
    for (i, p) in e.entities.iter().enumerate().filter(|(_, p)| {
        p.instance == e.instance
            && p.kind != 4
            && e.actor_enabled[p.kind]
            && atlas::tile_at(p.position).is_some_and(|t| region.contains(&t))
    }) {
        let key = p.key.clone();
        desired.insert(key.clone());
        let signature=format!("actor:{}:{}:{}",p.kind,p.type_id,e.object_npc_types.contains(&p.type_id));
        if preview.signatures.get(&key)!=Some(&signature){
            if let Some(entity)=preview.objects.remove(&key){commands.entity(entity).insert(Visibility::Hidden);preview.retiring.push(entity);}
        }
        preview.signatures.insert(key.clone(),signature);
        let transform = Transform {
            translation: p.position,
            rotation: ProtocolYawDegrees::new(p.angle as i32).native_root_rotation(),
            ..default()
        };
        if let Some(entity) = preview.objects.get(&key) {
            if let Ok(mut t) = transforms.get_mut(*entity) {
                if *t != transform {
                    *t = transform;
                }
            }
            continue;
        }
        if spawn_budget == 0 {
            preview.pending = true;
            continue;
        }
        spawn_budget -= 1;
        let entity = commands
            .spawn((
                Name::new(key.clone()),
                ChildOf(root),
                transform,
                Visibility::Inherited,
            ))
            .id();
        if p.kind == 3 {
            if let Some(path) = catalog.shiny_models.get(&p.type_id) {
                commands.entity(entity).insert(WorldAssetRoot(
                    server.load(GltfAssetLabel::Scene(0).from_asset(path.clone())),
                ));
            }
            preview.objects.insert(key, entity);
            continue;
        }
        let entry = catalog
            .entries
            .iter()
            .find(|entry| entry.kind == CatalogKind::Npc && entry.network_id == Some(p.type_id));
        if let Some(entry) = entry.filter(|e| e.hnpc_visual.is_some() || e.npc_visual.is_some()) {
            if let Some(definition) = &entry.hnpc_visual {
                if let Some(hnpc) = &catalog.hnpc {
                    match spawn_network_hnpc_visual_0104(
                        &mut commands,
                        &server,
                        &mut rig,
                        hnpc,
                        entity,
                        definition,
                        e.geometry_revision,
                    ) {
                        Ok(visual) => {
                            commands.entity(entity).insert(visual);
                        }
                        Err(error) => eprintln!("World entity {i}: {error}"),
                    }
                }
            } else if let Some(definition) = &entry.npc_visual {
                let visual = spawn_network_npc_visual_0104(
                    &mut commands,
                    &server,
                    entity,
                    definition,
                    key.clone(),
                );
                commands.entity(entity).insert(visual);
            }
        } else {
            // Location NPCs (XDT class >= 100) deliberately have no runtime
            // visual definition, even though their catalog entry exists.
            let mesh = meshes.add(Cuboid::new(2., 2., 2.));
            let material = materials.add(StandardMaterial {
                base_color: markers::placeholder_colour(e.object_npc_types.contains(&p.type_id)),
                unlit: true,
                ..default()
            });
            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_xyz(0., 1., 0.),
                ChildOf(entity),
            ));
        }
        preview.objects.insert(key, entity);
    }
    for tile in e.maps.values() {
        let document = &e.sources[tile.scene].draft;
        let scene_root: ffone_client::world::AuthoredWorldTransform =
            match serde_json::from_value(document["root"].clone()) {
                Ok(t) => t,
                Err(_) => continue,
            };
        let Ok(root_transform) = scene_root.try_to_bevy("editor map root") else {
            continue;
        };
        let models: BTreeMap<_, _> = document["models"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|m| Some((m["id"].as_str()?, m["path"].as_str()?)))
            .collect();
        for v in document["visuals"].as_array().into_iter().flatten() {
            if let Some(scene) = &tile.native_scene {
                if scene
                    .visuals
                    .iter()
                    .find(|visual| Some(visual.name.as_str()) == v["name"].as_str())
                    .is_some_and(|visual| !scene.visual_has_presentation(visual))
                {
                    continue;
                }
            }
            let Ok(t) = serde_json::from_value::<ffone_client::world::AuthoredWorldTransform>(
                v["transform"].clone(),
            ) else {
                continue;
            };
            let Ok(t) = t.try_to_bevy("editor world visual") else {
                continue;
            };
            let transform = root_transform.mul_transform(t);
            let Some(path) = v["model"].as_str().and_then(|id| models.get(id)) else {
                continue;
            };
            let key = format!(
                "{}:visual:{}",
                tile.id,
                v["name"].as_str().unwrap_or_default()
            );
            desired.insert(key.clone());
            let signature=format!("model:{}:{}",path,v["scene"].as_u64().unwrap_or(0));
            if preview.signatures.get(&key)!=Some(&signature){
                if let Some(entity)=preview.objects.remove(&key){commands.entity(entity).insert(Visibility::Hidden);preview.retiring.push(entity);}
            }
            preview.signatures.insert(key.clone(),signature);
            if let Some(entity) = preview.objects.get(&key) {
                if let Ok(mut t) = transforms.get_mut(*entity) {
                    if *t != transform {
                        *t = transform;
                    }
                }
            } else {
                if spawn_budget == 0 {
                    preview.pending = true;
                    continue;
                }
                spawn_budget -= 1;
                let entity = commands
                    .spawn((
                        WorldAssetRoot(
                            server.load(
                                GltfAssetLabel::Scene(v["scene"].as_u64().unwrap_or(0) as usize)
                                    .from_asset(path.to_string()),
                            ),
                        ),
                        transform,
                        Visibility::Inherited,
                        ChildOf(root),
                    ))
                    .id();
                preview.objects.insert(key, entity);
            }
        }
    }
    terrain::update(&mut commands, &mut e, &mut preview, root);
    collision::update(&mut commands,&e,&mut preview,root,&server,&mut materials,&mut transforms,&mut desired);
    let stale: Vec<_> = preview
        .objects
        .keys()
        .filter(|key| !desired.contains(*key))
        .cloned()
        .collect();
    for key in stale {
        preview.signatures.remove(&key);
        if let Some(entity) = preview.objects.remove(&key) {
            commands.entity(entity).insert(Visibility::Hidden);
            preview.retiring.push(entity);
        }
    }
    let selected = e.selected().filter(|p| p.instance == e.instance);
    for (entity, marker) in &selection {
        if let Ok(mut v) = visibility.get_mut(entity) {
            *v = if selected.is_some() {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
        if let Some(p) = selected
            && let Ok(mut t) = transforms.get_mut(entity)
        {
            let start = p.position + Vec3::Y * 0.15;
            let end = start
                + ProtocolYawDegrees::new(p.angle as i32).native_root_rotation() * Vec3::NEG_Z * 4.;
            *t = if marker.0 {
                Transform::from_translation((start + end) * 0.5).looking_at(end, Vec3::Y)
            } else {
                Transform::from_translation(start)
            };
        }
    }
    if !selection.is_empty() {
        return;
    }
    if let Some(p) = e.selected().filter(|p| p.instance == e.instance) {
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(1., 0.75, 0.15),
            unlit: true,
            ..default()
        });
        let forward = ProtocolYawDegrees::new(p.angle as i32).native_root_rotation() * Vec3::NEG_Z;
        let start = p.position + Vec3::Y * 0.15;
        commands.spawn((
            SelectionMarker(false),
            ChildOf(root),
            Mesh3d(meshes.add(Torus::new(1.5, 1.65))),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(start),
        ));
        let end = start + forward * 4.;
        commands.spawn((
            SelectionMarker(true),
            ChildOf(root),
            Mesh3d(meshes.add(Cuboid::new(0.12, 0.12, 4.))),
            MeshMaterial3d(material),
            Transform::from_translation((start + end) * 0.5).looking_at(end, Vec3::Y),
        ));
    }
}
// Removing a region can release thousands of instantiated scene descendants.
// Hide immediately, then retire leaves under the native client's frame budget.
pub(super) fn retire(world: &mut World) {
    world.resource_scope(|world, mut preview: Mut<WorldPreview>| {
        let started = Instant::now();
        let mut removed = 0;
        let mut visited = 0;
        while let Some(&entity) = preview.retiring.last() {
            if removed >= 256 || visited >= 4096 || started.elapsed().as_millis() >= 1 {
                break;
            }
            visited += 1;
            if !world.entities().contains(entity) {
                preview.retiring.pop();
                continue;
            }
            if let Some(child) = world
                .get::<Children>(entity)
                .and_then(|c| c.first())
                .copied()
            {
                preview.retiring.push(child);
            } else {
                world.despawn(entity);
                preview.retiring.pop();
                removed += 1;
            }
        }
    });
}
pub(super) fn camera(
    e: Res<WorldEditor>,
    state: Res<EditorState>,
    window: Single<&Window>,
    canvas: Query<(&ComputedNode, &UiGlobalTransform), With<Canvas>>,
    mut cameras: Query<(&mut Camera, &mut Transform, &mut Projection), With<PreviewCamera>>,
) {
    if state.world_open != Some(true) {
        return;
    }
    let canvas = canvas
        .iter()
        .find(|(node, _)| node.size().min_element() > 2.);
    for (mut camera, mut transform, mut projection) in &mut cameras {
        if let Projection::Perspective(p) = projection.as_mut() {
            p.far = 20000.;
        }
        if let Some((node, global)) = canvas {
            let size = node.size();
            let min = global.translation - size * 0.5;
            let mut viewport = Viewport {
                physical_position: UVec2::new(min.x.max(0.) as u32, min.y.max(0.) as u32),
                physical_size: UVec2::new(size.x.max(1.) as u32, size.y.max(1.) as u32),
                ..default()
            };
            viewport.clamp_to_size(UVec2::new(
                window.resolution.physical_width(),
                window.resolution.physical_height(),
            ));
            camera.viewport = Some(viewport);
        }
        let offset = Vec3::new(
            e.yaw.sin() * e.pitch.cos(),
            e.pitch.sin(),
            -e.yaw.cos() * e.pitch.cos(),
        ) * e.distance;
        *transform = Transform::from_translation(e.center + offset).looking_at(e.center, Vec3::Y);
    }
}
