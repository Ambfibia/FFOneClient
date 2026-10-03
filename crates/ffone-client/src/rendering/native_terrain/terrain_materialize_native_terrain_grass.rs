use super::*;

pub(super) fn prepare_pending_native_terrain_grass(
    mut commands: Commands,
    mut pending: Query<(Entity, &mut PendingNativeTerrainGrassPreparation)>,
) {
    let running = pending
        .iter_mut()
        .filter(|(_, pending)| pending.task.is_some())
        .count();
    let mut slots = native_terrain_grass_preparation_slots(running);
    if slots > 0 {
        for (_, mut pending) in &mut pending {
            if pending.task.is_some() {
                continue;
            }
            let Some(terrain) = pending.terrain.take() else {
                continue;
            };
            pending.task = Some(
                AsyncComputeTaskPool::get()
                    .spawn(async move { prepare_native_terrain_grass(terrain) }),
            );
            slots -= 1;
            if slots == 0 {
                break;
            }
        }
    }

    for (entity, mut pending) in &mut pending {
        let Some(task) = pending.task.as_mut() else {
            continue;
        };
        let Some(layers) = block_on(future::poll_once(task)) else {
            continue;
        };
        let total_layers = layers.iter().map(|layer| layer.authored_layer_count).sum();
        let total_instances = layers.iter().map(|layer| layer.instance_count).sum();
        let mut entity_commands = commands.entity(entity);
        entity_commands.remove::<PendingNativeTerrainGrassPreparation>();
        if layers.is_empty() {
            entity_commands.insert(NativeTerrainDetailStatus::SourceContractPending);
        } else {
            entity_commands.insert(PendingNativeTerrainGrass {
                descriptor_path: pending.descriptor_path.clone(),
                layers: layers.into(),
                total_layers,
                total_instances,
                rendered_layers: 0,
                rendered_instances: 0,
            });
        }
    }
}

#[derive(Debug)]
pub(super) struct NativeTerrainGrassAdmissionBudget {
    pub(super) remaining_chunks: usize,
    pub(super) remaining_vertices: usize,
    pub(super) admitted_chunks: usize,
}

impl Default for NativeTerrainGrassAdmissionBudget {
    fn default() -> Self {
        Self {
            remaining_chunks: NATIVE_TERRAIN_GRASS_CHUNKS_PER_FRAME,
            remaining_vertices: NATIVE_TERRAIN_GRASS_VERTICES_PER_FRAME,
            admitted_chunks: 0,
        }
    }
}

impl NativeTerrainGrassAdmissionBudget {
    pub(super) fn admit(&mut self, vertices: usize) -> bool {
        if self.remaining_chunks == 0 || vertices > self.remaining_vertices {
            return false;
        }
        self.remaining_chunks -= 1;
        self.remaining_vertices -= vertices;
        self.admitted_chunks += 1;
        true
    }
}

pub(super) fn materialize_native_terrain_grass(
    mut commands: Commands,
    mut pending: Query<(Entity, &mut PendingNativeTerrainGrass)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut budget = NativeTerrainGrassAdmissionBudget::default();
    for (entity, mut pending) in &mut pending {
        loop {
            let Some(next_vertices) = pending
                .layers
                .front()
                .and_then(|layer| layer.chunks.front())
                .map(|chunk| chunk.mesh.count_vertices())
            else {
                break;
            };
            if !budget.admit(next_vertices) {
                break;
            }

            let descriptor_path = pending.descriptor_path.clone();
            let rendered_layers = pending.rendered_layers;
            let (chunk, material, chunk_index) = {
                let layer = pending
                    .layers
                    .front_mut()
                    .expect("a pending grass chunk was checked above");
                let material = layer
                    .material
                    .get_or_insert_with(|| {
                        let texture_path = native_terrain_detail_texture_asset_path(
                            &descriptor_path,
                            &layer.texture_path,
                        );
                        materials.add(StandardMaterial {
                            base_color_texture: Some(asset_server.load(texture_path)),
                            alpha_mode: AlphaMode::Mask(0.5),
                            cull_mode: None,
                            unlit: true,
                            // Ordinary legacy-world surfaces use the dedicated legacy material
                            // path and are not attenuated by Bevy's camera DistanceFog. Detail
                            // grass is authored as part of that same terrain presentation; leaving
                            // StandardMaterial fog enabled made it fade a second time and vanish
                            // much closer to the camera than every surrounding world object.
                            fog_enabled: false,
                            ..default()
                        })
                    })
                    .clone();
                let chunk_index = layer.rendered_chunks;
                layer.rendered_chunks += 1;
                let chunk = layer
                    .chunks
                    .pop_front()
                    .expect("a pending grass chunk was checked above");
                layer.instance_count = layer.instance_count.saturating_sub(chunk.instance_count);
                (chunk, material, chunk_index)
            };
            pending.rendered_instances += chunk.instance_count;
            let grass_mesh = meshes.add(chunk.mesh);
            let mut visibility_range = VisibilityRange::abrupt(0.0, chunk.visibility_end);
            // Measure from the transformed chunk AABB center, not from the
            // 512m terrain parent origin. The radius folded into
            // `visibility_end` keeps every blade whose point is within the
            // exact Unity detail distance visible at chunk boundaries.
            visibility_range.use_aabb = true;
            commands.spawn((
                Name::new(format!(
                    "terrain grass atlas {rendered_layers} chunk {chunk_index}",
                )),
                NativeWorldSceneEntity,
                ChildOf(entity),
                Transform::IDENTITY,
                Visibility::Inherited,
                visibility_range,
                Mesh3d(grass_mesh),
                MeshMaterial3d(material),
            ));

            if pending
                .layers
                .front()
                .is_some_and(|layer| layer.chunks.is_empty())
            {
                let completed = pending
                    .layers
                    .pop_front()
                    .expect("the completed grass layer was checked above");
                debug_assert_eq!(completed.instance_count, 0);
                pending.rendered_layers += completed.authored_layer_count;
            }
            commands
                .entity(entity)
                .insert(NativeTerrainDetailStatus::Loading {
                    rendered_layers: pending.rendered_layers,
                    total_layers: pending.total_layers,
                    rendered_instances: pending.rendered_instances,
                    total_instances: pending.total_instances,
                });
        }

        if pending.layers.is_empty() {
            commands
                .entity(entity)
                .remove::<PendingNativeTerrainGrass>()
                .insert(NativeTerrainDetailStatus::Ready {
                    rendered_layers: pending.rendered_layers,
                    rendered_instances: pending.rendered_instances,
                });
        }
        if budget.remaining_chunks == 0 || budget.remaining_vertices == 0 {
            break;
        }
    }
}

pub(super) fn native_terrain_detail_texture_asset_path(descriptor_path: &str, texture_path: &str) -> String {
    if texture_path.starts_with("map/shared/") {
        return texture_path.to_owned();
    }
    Path::new(descriptor_path)
        .parent()
        .expect("validated terrain descriptor has a parent")
        .join(texture_path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub(super) fn terrain_material_uniform(terrain: &NativeTerrain) -> NativeTerrainMaterialUniform {
    let descriptor = terrain.descriptor();
    let (render_mode, splat_map_distance) =
        terrain
            .environment()
            .map_or((LegacyTerrainRenderMode::Lightmap, 150.0), |environment| {
                let fields = &environment
                    .terrain_detail
                    .serialized_terrain_render_baseline
                    .serialized_fields;
                (
                    LegacyTerrainRenderMode::try_from(fields.render_mode)
                        .expect("validated native terrain render mode"),
                    fields.splat_map_distance as f32,
                )
            });
    let mut layer_tile_scale_mode = [Vec4::ZERO; NATIVE_TERRAIN_MAX_LAYER_COUNT];
    let mut layer_source_size = [Vec4::ZERO; NATIVE_TERRAIN_MAX_LAYER_COUNT];
    for ((destination, source_size), layer) in layer_tile_scale_mode
        .iter_mut()
        .zip(layer_source_size.iter_mut())
        .zip(&descriptor.splat.layers)
    {
        *destination = Vec4::new(
            (descriptor.scale.extent_x / layer.tile_size.x) as f32,
            (descriptor.scale.extent_z / layer.tile_size.y) as f32,
            layer.mode as f32,
            0.0,
        );
        *source_size = Vec4::new(
            layer.albedo.width as f32,
            layer.albedo.height as f32,
            layer.albedo.mips.len().saturating_sub(1) as f32,
            layer
                .albedo
                .sampler
                .as_ref()
                .map_or(0.0, |sampler| sampler.mip_bias as f32),
        );
    }
    let mut weight_source_sampler = [Vec4::ZERO; NATIVE_TERRAIN_MAX_WEIGHT_MAP_COUNT];
    for (destination, weight) in weight_source_sampler
        .iter_mut()
        .zip(&descriptor.splat.weight_maps)
    {
        let sampler = weight.sampler.as_ref();
        *destination = Vec4::new(
            sampler.map_or(1.0, |sampler| sampler.filter_mode.serialized_value as f32),
            weight.mips.len().saturating_sub(1) as f32,
            sampler.map_or(0.0, |sampler| sampler.mip_bias as f32),
            weight.width as f32,
        );
    }
    NativeTerrainMaterialUniform {
        layer_tile_scale_mode,
        layer_source_size,
        weight_source_sampler,
        metadata: Vec4::new(
            descriptor.splat.layers.len() as f32,
            render_mode as i64 as f32,
            splat_map_distance,
            0.0,
        ),
        ambience_light: Vec4::ONE,
        ambience_fog: Vec4::ZERO,
        render_quality: Vec4::new(NATIVE_TERRAIN_ANISOTROPIC_TAPS, 0.0, 0.0, 0.0),
    }
}
