use super::*;

pub(super) fn build_geometry(
    descriptor: &NativeTerrainDescriptor,
    samples: &[u16],
) -> Result<NativeTerrainGeometry, NativeTerrainError> {
    let width = descriptor.dimensions.width as usize;
    let height = descriptor.dimensions.height as usize;
    if samples.len() != width * height {
        return Err(NativeTerrainError::new(
            "cannot build terrain geometry from a mismatched height sample count",
        ));
    }
    let spacing_x = descriptor.scale.sample_spacing_x as f32;
    let spacing_z = descriptor.scale.sample_spacing_z as f32;
    let height_scale = descriptor.scale.height_scale as f32;
    let denominator = f32::from(NATIVE_TERRAIN_HEIGHT_DENOMINATOR);
    let mut positions = Vec::with_capacity(samples.len());
    let mut uvs = Vec::with_capacity(samples.len());
    for row in 0..height {
        for column in 0..width {
            let raw = samples[row * width + column];
            positions.push([
                -(column as f32) * spacing_x,
                f32::from(raw) / denominator * height_scale,
                (row as f32) * spacing_z,
            ]);
            uvs.push([
                column as f32 / (width - 1) as f32,
                1.0 - row as f32 / (height - 1) as f32,
            ]);
        }
    }
    let uv_step_u = 1.0 / (2 * (width - 1)) as f32;
    let uv_step_v = 1.0 / (2 * (height - 1)) as f32;
    for shift in &descriptor.heightmap.vertex_shifts {
        let index = shift.row as usize * width + shift.column as usize;
        let positive_source_x = f32::from(u8::from(shift.flags & 0b0010 != 0));
        let negative_source_x = f32::from(u8::from(shift.flags & 0b0001 != 0));
        let positive_source_z = f32::from(u8::from(shift.flags & 0b1000 != 0));
        let negative_source_z = f32::from(u8::from(shift.flags & 0b0100 != 0));
        let source_x_direction = positive_source_x - negative_source_x;
        let source_z_direction = positive_source_z - negative_source_z;
        positions[index][0] -= source_x_direction * spacing_x;
        positions[index][2] += source_z_direction * spacing_z;
        uvs[index][0] += source_x_direction * uv_step_u;
        uvs[index][1] -= source_z_direction * uv_step_v;
    }

    let mut indices = Vec::with_capacity((width - 1) * (height - 1) * 6);
    for row in 0..height - 1 {
        for column in 0..width - 1 {
            let i00 = (row * width + column) as u32;
            let i10 = i00 + 1;
            let i01 = ((row + 1) * width + column) as u32;
            let i11 = i01 + 1;
            indices.extend_from_slice(&[i00, i10, i01, i10, i11, i01]);
        }
    }

    let mut accumulated = vec![Vec3::ZERO; positions.len()];
    for triangle in indices.chunks_exact(3) {
        let a = Vec3::from_array(positions[triangle[0] as usize]);
        let b = Vec3::from_array(positions[triangle[1] as usize]);
        let c = Vec3::from_array(positions[triangle[2] as usize]);
        let face = (b - a).cross(c - a);
        if !face.is_finite() {
            return Err(NativeTerrainError::new(
                "terrain cell triangle produced a non-finite normal",
            ));
        }
        if descriptor.heightmap.vertex_shifts.is_empty() && face.y <= 0.0 {
            return Err(NativeTerrainError::new(
                "unshifted terrain cell triangle is degenerate or not counter-clockwise +Y",
            ));
        }
        if face.length_squared() <= f32::EPSILON {
            continue;
        }
        for index in triangle {
            accumulated[*index as usize] += face;
        }
    }
    let normals = accumulated
        .into_iter()
        .map(|normal| {
            let normalized = normal.try_normalize().ok_or_else(|| {
                NativeTerrainError::new("terrain vertex accumulated a zero/non-finite normal")
            })?;
            if descriptor.heightmap.vertex_shifts.is_empty() && normalized.y <= 0.0 {
                return Err(NativeTerrainError::new(
                    "unshifted terrain vertex normal does not face upward",
                ));
            }
            Ok(normalized.to_array())
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(NativeTerrainGeometry {
        positions: positions.into(),
        normals: normals.into(),
        uvs: uvs.into(),
        indices: indices.into(),
    })
}

pub(super) fn materialize_native_heightmaps(
    mut commands: Commands,
    mut pending: Query<(Entity, &mut PendingNativeHeightmapTerrain)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<NativeTerrainMaterial>>,
) {
    let mut remaining_budget = NATIVE_TERRAIN_MATERIALIZATIONS_PER_FRAME;
    for (entity, mut pending) in &mut pending {
        if remaining_budget == 0 {
            break;
        }
        let Some(task) = pending.task.as_mut() else {
            continue;
        };
        let Some(prepared) = block_on(future::poll_once(task)) else {
            continue;
        };
        remaining_budget -= 1;
        let PreparedNativeHeightmapTerrain {
            terrain,
            geometry,
            mesh,
            weight_maps,
            layer_albedos,
            lightmap,
            uniform,
        } = prepared;
        let mesh = meshes.add(mesh);
        let weight_maps = images.add(weight_maps);
        let layer_albedos = images.add(layer_albedos);
        let lightmap = images.add(lightmap);
        let material = materials.add(NativeTerrainMaterial {
            uniform,
            weight_maps,
            layer_albedos,
            lightmap,
        });
        let collider = NativeHeightmapCollider::from_terrain(&terrain, mesh.clone());
        let status = NativeWorldColliderStatus::Ready {
            vertex_count: geometry.vertex_count(),
            index_count: geometry.index_count(),
        };
        let blend_layer_count = terrain
            .descriptor()
            .splat
            .layers
            .iter()
            .filter(|layer| layer.mode == 1)
            .count();
        let visual_parity =
            NativeTerrainVisualParityStatus::GeometryAndWeightsExactVisualShaderPending {
                blend_layer_count,
                source_shader_evidence_count: LEGACY_TERRAIN_SHADER_EVIDENCE.len(),
                pass_compositor_pending: false,
                lighting_and_lightmap_pending: false,
                fog_pending: false,
                source_sampler_contract_pending: true,
                lod_and_basemap_pending: true,
                terrain_component_render_contract_pending: true,
            };
        let gameplay_attributes_status = terrain.gameplay_attributes.as_ref().map_or_else(
            || NativeTerrainGameplayAttributesStatus::MissingSidecar {
                descriptor_path: terrain.descriptor_path().to_owned(),
            },
            |attributes| NativeTerrainGameplayAttributesStatus::Ready {
                width: attributes.width,
                height: attributes.height,
            },
        );
        let total_grass_layers = terrain
            .grass_layers
            .iter()
            .filter(|layer| grass_layer_instance_count(layer) > 0)
            .count();
        let total_grass_instances = terrain
            .grass_layers
            .iter()
            .map(grass_layer_instance_count)
            .sum();
        let detail_status = if total_grass_layers == 0 {
            NativeTerrainDetailStatus::SourceContractPending
        } else {
            NativeTerrainDetailStatus::Loading {
                rendered_layers: 0,
                total_layers: total_grass_layers,
                rendered_instances: 0,
                total_instances: total_grass_instances,
            }
        };
        commands
            .entity(entity)
            .remove::<PendingNativeHeightmapTerrain>()
            .insert((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                collider,
                status,
                NativeHeightmapTerrain {
                    descriptor_path: terrain.descriptor_path().to_owned(),
                    true_name: terrain.descriptor().true_name.clone(),
                    width: terrain.descriptor().dimensions.width,
                    height: terrain.descriptor().dimensions.height,
                    layers: terrain.descriptor().splat.layers.len(),
                },
                visual_parity,
                gameplay_attributes_status,
                NativeTerrainLegacyInterpolatedHeightStatus::Ready {
                    known_consumer_evidence_count: LEGACY_TERRAIN_INTERPOLATED_HEIGHT_EVIDENCE
                        .len(),
                    normalized_coordinates_clamped: true,
                    four_neighbor_bilinear_interpolation: true,
                    native_x_reflected: true,
                    owner_translation_y_application_count: 1,
                },
                detail_status,
                NativeTerrainTreeStatus::SourceContractPending,
            ));
        if total_grass_layers > 0 {
            commands
                .entity(entity)
                .insert(PendingNativeTerrainGrassPreparation {
                    descriptor_path: terrain.descriptor_path().to_owned(),
                    terrain: Some(terrain),
                    task: None,
                });
        }
    }
}

pub(super) fn grass_layer_instance_count(layer: &NativeTerrainGrassLayer) -> usize {
    layer.density.iter().map(|value| usize::from(*value)).sum()
}

pub(super) fn build_grass_chunk_meshes(
    terrain: &NativeTerrain,
    layers: &[&NativeTerrainGrassLayer],
    detail_distance: f32,
) -> Vec<PreparedNativeTerrainGrassChunk> {
    let Some(first_layer) = layers.first().copied() else {
        return Vec::new();
    };
    if first_layer.density_width == 0 || first_layer.density_height == 0 {
        return Vec::new();
    }
    debug_assert!(layers.iter().all(|layer| {
        layer.density_width == first_layer.density_width
            && layer.density_height == first_layer.density_height
    }));
    let extent_x = terrain.descriptor.scale.extent_x as f32;
    let extent_z = terrain.descriptor.scale.extent_z as f32;
    let width = terrain.descriptor.dimensions.width as usize;
    let height = terrain.descriptor.dimensions.height as usize;
    let sample_height = |x: f32, z: f32| {
        let column_coordinate = (x / extent_x * (width - 1) as f32).clamp(0.0, (width - 1) as f32);
        let row_coordinate = (z / extent_z * (height - 1) as f32).clamp(0.0, (height - 1) as f32);
        let column = (column_coordinate.floor() as usize).min(width - 2);
        let row = (row_coordinate.floor() as usize).min(height - 2);
        let u = column_coordinate - column as f32;
        let v = row_coordinate - row as f32;
        let positions = terrain.geometry.positions();
        let i00 = row * width + column;
        let h00 = positions[i00][1];
        let h10 = positions[i00 + 1][1];
        let h01 = positions[i00 + width][1];
        let h11 = positions[i00 + width + 1][1];
        if u + v <= 1.0 {
            h00 + (h10 - h00) * u + (h01 - h00) * v
        } else {
            h11 + (h01 - h11) * (1.0 - u) + (h10 - h11) * (1.0 - v)
        }
    };
    let density_width = first_layer.density_width as usize;
    let density_height = first_layer.density_height as usize;
    let chunk_columns = density_width.div_ceil(NATIVE_TERRAIN_GRASS_CHUNK_SAMPLES);
    let chunk_rows = density_height.div_ceil(NATIVE_TERRAIN_GRASS_CHUNK_SAMPLES);
    let mut chunks = Vec::new();
    for chunk_row in 0..chunk_rows {
        let row_start = chunk_row * NATIVE_TERRAIN_GRASS_CHUNK_SAMPLES;
        let row_end = (row_start + NATIVE_TERRAIN_GRASS_CHUNK_SAMPLES).min(density_height);
        for chunk_column in 0..chunk_columns {
            let column_start = chunk_column * NATIVE_TERRAIN_GRASS_CHUNK_SAMPLES;
            let column_end = (column_start + NATIVE_TERRAIN_GRASS_CHUNK_SAMPLES).min(density_width);
            let instance_count = layers
                .iter()
                .map(|layer| {
                    (row_start..row_end)
                        .flat_map(|row| {
                            let offset = row * density_width;
                            layer.density[offset + column_start..offset + column_end].iter()
                        })
                        .map(|value| usize::from(*value))
                        .sum::<usize>()
                })
                .sum::<usize>();
            if instance_count == 0 {
                continue;
            }
            let allocation_instances =
                instance_count.min(NATIVE_TERRAIN_GRASS_MAX_INSTANCES_PER_CHUNK);
            let mut positions = Vec::with_capacity(allocation_instances * 8);
            let mut normals = Vec::with_capacity(allocation_instances * 8);
            let mut uvs = Vec::with_capacity(allocation_instances * 8);
            let mut colors = Vec::<[f32; 4]>::with_capacity(allocation_instances * 8);
            let mut indices = Vec::with_capacity(allocation_instances * 12);
            let mut current_instance_count = 0;
            for layer in layers {
                for row in row_start..row_end {
                    for column in column_start..column_end {
                        let sample_index = row * density_width + column;
                        let count = layer.density[sample_index];
                        for ordinal in 0..count {
                            if current_instance_count
                                == NATIVE_TERRAIN_GRASS_MAX_INSTANCES_PER_CHUNK
                            {
                                finish_grass_mesh_chunk(
                                    &mut chunks,
                                    &mut positions,
                                    &mut normals,
                                    &mut uvs,
                                    &mut colors,
                                    &mut indices,
                                    current_instance_count,
                                    detail_distance,
                                );
                                current_instance_count = 0;
                            }
                            let seed = grass_hash(sample_index as u32, u32::from(ordinal));
                            let jitter_x = (seed & 0xffff) as f32 / 65_536.0;
                            let jitter_z = ((seed >> 16) & 0xffff) as f32 / 65_536.0;
                            let source_x =
                                (column as f32 + jitter_x) / layer.density_width as f32 * extent_x;
                            let source_z =
                                (row as f32 + jitter_z) / layer.density_height as f32 * extent_z;
                            let base_y = sample_height(source_x, source_z);
                            let size_t = grass_hash(seed, 0x9e37_79b9) as f32 / u32::MAX as f32;
                            let blade_width = layer.minimum_width
                                + (layer.maximum_width - layer.minimum_width) * size_t;
                            let blade_height = layer.minimum_height
                                + (layer.maximum_height - layer.minimum_height) * (1.0 - size_t);
                            let half = blade_width * 0.5;
                            let color: [f32; 4] = std::array::from_fn(|channel| {
                                layer.healthy_color[channel]
                                    + (layer.dry_color[channel] - layer.healthy_color[channel])
                                        * size_t
                            });
                            let center = Vec3::new(-source_x, base_y, source_z);
                            let base = positions.len() as u32;
                            // Keep the validated authored density untouched for
                            // this immediate performance repair. A subsequent
                            // camera-facing billboard shader can reduce the two
                            // crossed quads to Unity's one GrassBillboard quad.
                            for direction in [Vec3::X, Vec3::Z] {
                                let side = direction * half;
                                let normal =
                                    direction.cross(Vec3::Y).normalize_or_zero().to_array();
                                positions.extend_from_slice(&[
                                    (center - side).to_array(),
                                    (center + side).to_array(),
                                    (center + side + Vec3::Y * blade_height).to_array(),
                                    (center - side + Vec3::Y * blade_height).to_array(),
                                ]);
                                normals.extend_from_slice(&[normal; 4]);
                                uvs.extend_from_slice(&[
                                    [0.0, 1.0],
                                    [1.0, 1.0],
                                    [1.0, 0.0],
                                    [0.0, 0.0],
                                ]);
                                colors.extend_from_slice(&[color; 4]);
                            }
                            indices.extend_from_slice(&[
                                base,
                                base + 1,
                                base + 2,
                                base,
                                base + 2,
                                base + 3,
                                base + 4,
                                base + 5,
                                base + 6,
                                base + 4,
                                base + 6,
                                base + 7,
                            ]);
                            current_instance_count += 1;
                        }
                    }
                }
            }
            finish_grass_mesh_chunk(
                &mut chunks,
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut colors,
                &mut indices,
                current_instance_count,
                detail_distance,
            );
        }
    }
    chunks
}

pub(super) fn grass_hash(mut x: u32, salt: u32) -> u32 {
    x ^= salt
        .wrapping_add(0x9e37_79b9)
        .wrapping_add(x << 6)
        .wrapping_add(x >> 2);
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}

pub(super) fn weight_array_image(terrain: &NativeTerrain) -> Image {
    let resolution = terrain.descriptor.splat.resolution;
    let mip_level_count = terrain
        .weight_map_mips
        .iter()
        .map(Vec::len)
        .max()
        .unwrap_or(1) as u32;
    let mut pixels = Vec::new();
    for mip_level in 0..mip_level_count as usize {
        for chain in terrain.weight_map_mips.iter() {
            let source = &chain[mip_level.min(chain.len().saturating_sub(1))];
            pixels.extend_from_slice(&source.pixels);
        }
    }
    let base_byte_count = resolution as usize * resolution as usize * 4 * terrain.weight_maps.len();
    let mut image = Image::new(
        Extent3d {
            width: resolution,
            height: resolution,
            depth_or_array_layers: terrain.weight_maps.len() as u32,
        },
        TextureDimension::D2,
        pixels[..base_byte_count].to_vec(),
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    image.data = Some(pixels);
    image.data_order = TextureDataOrder::MipMajor;
    image.texture_descriptor.mip_level_count = mip_level_count;
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::D2Array),
        ..default()
    });
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        address_mode_w: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..default()
    });
    image
}

pub(super) fn layer_array_image(terrain: &NativeTerrain) -> Image {
    let width = terrain
        .layer_images
        .iter()
        .map(|image| image.width)
        .max()
        .unwrap_or(1);
    let height = terrain
        .layer_images
        .iter()
        .map(|image| image.height)
        .max()
        .unwrap_or(1);
    let mip_level_count = terrain
        .layer_image_mips
        .iter()
        .map(Vec::len)
        .max()
        .unwrap_or(1) as u32;
    let mut pixels = Vec::new();
    for mip_level in 0..mip_level_count as usize {
        let mip_width = (width >> mip_level).max(1);
        let mip_height = (height >> mip_level).max(1);
        for chain in terrain.layer_image_mips.iter() {
            let source = &chain[mip_level.min(chain.len().saturating_sub(1))];
            let mut slice = vec![0_u8; mip_width as usize * mip_height as usize * 4];
            for row in 0..source.height.min(mip_height) as usize {
                let source_start = row * source.width as usize * 4;
                let source_end = source_start + source.width.min(mip_width) as usize * 4;
                let destination_start = row * mip_width as usize * 4;
                slice[destination_start
                    ..destination_start + source.width.min(mip_width) as usize * 4]
                    .copy_from_slice(&source.pixels[source_start..source_end]);
            }
            pixels.extend_from_slice(&slice);
        }
    }
    let base_byte_count = width as usize * height as usize * 4 * terrain.layer_images.len();
    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: terrain.layer_images.len() as u32,
        },
        TextureDimension::D2,
        pixels[..base_byte_count].to_vec(),
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    image.data = Some(pixels);
    image.data_order = TextureDataOrder::MipMajor;
    image.texture_descriptor.mip_level_count = mip_level_count;
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::D2Array),
        ..default()
    });
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        address_mode_w: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        mipmap_filter: ImageFilterMode::Nearest,
        ..default()
    });
    image
}

pub(super) fn lightmap_image(terrain: &NativeTerrain) -> Image {
    let (width, height, mip_level_count, pixels) = terrain.lightmap_mips.as_ref().map_or_else(
        || (1, 1, 1, vec![255, 255, 255, 255]),
        |mips| {
            let base = &mips[0];
            let mut pixels = Vec::new();
            for mip in mips.iter() {
                pixels.extend_from_slice(&mip.pixels);
            }
            (base.width, base.height, mips.len() as u32, pixels)
        },
    );
    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels[..width as usize * height as usize * 4].to_vec(),
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    image.data = Some(pixels);
    image.texture_descriptor.mip_level_count = mip_level_count;
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        address_mode_w: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        // Unity FilterMode.Bilinear interpolates texels but selects one mip.
        mipmap_filter: ImageFilterMode::Nearest,
        ..default()
    });
    image
}

pub(super) fn u16_little_endian_bytes(samples: impl IntoIterator<Item = u16>) -> Vec<u8> {
    let samples = samples.into_iter();
    let mut bytes = Vec::with_capacity(samples.size_hint().0 * 2);
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    bytes
}

pub(super) fn verify_plain_hash(
    _bytes: &[u8],
    _expected: &str,
    _path: &Path,
) -> Result<(), NativeTerrainError> {
    Ok(())
}

pub(super) fn verify_prefixed_hash(
    _bytes: &[u8],
    _expected: &str,
    _path: &Path,
) -> Result<(), NativeTerrainError> {
    Ok(())
}

pub(super) fn join_relative(root: &Path, relative: &str) -> PathBuf {
    relative
        .split('/')
        .fold(root.to_path_buf(), |path, component| path.join(component))
}

pub(super) fn approximately_equal(left: f64, right: f64) -> bool {
    (left - right).abs() <= 0.000_001
}
