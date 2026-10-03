use super::*;

impl NativeTerrainGeometry {
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    #[must_use]
    pub fn index_count(&self) -> usize {
        self.indices.len()
    }

    #[must_use]
    pub fn positions(&self) -> &[[f32; 3]] {
        &self.positions
    }

    #[must_use]
    pub fn normals(&self) -> &[[f32; 3]] {
        &self.normals
    }

    #[must_use]
    pub fn uvs(&self) -> &[[f32; 2]] {
        &self.uvs
    }

    #[must_use]
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    pub(super) fn to_mesh(&self) -> Mesh {
        let vertex_lighting = self
            .normals
            .iter()
            .map(|normal| legacy_terrain_vertex_light(Vec3::from_array(*normal)).to_array())
            .collect::<Vec<_>>();
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions.to_vec())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals.to_vec())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs.to_vec())
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vertex_lighting)
        .with_inserted_indices(Indices::U32(self.indices.to_vec()))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeTerrain {
    pub(super) descriptor_path: String,
    pub(super) descriptor_blake3: String,
    pub(super) descriptor: NativeTerrainDescriptor,
    pub(super) height_samples: Arc<[u16]>,
    pub(super) weight_maps: Arc<[VerifiedRgbaImage]>,
    pub(super) weight_map_mips: Arc<[Vec<VerifiedRgbaImage>]>,
    pub(super) layer_images: Arc<[VerifiedRgbaImage]>,
    pub(super) layer_image_mips: Arc<[Vec<VerifiedRgbaImage>]>,
    pub(super) lightmap: Option<Arc<VerifiedRgbaImage>>,
    pub(super) lightmap_mips: Option<Arc<[VerifiedRgbaImage]>>,
    pub(super) gameplay_attributes: Option<Arc<VerifiedGameplayAttributes>>,
    pub(super) environment: Option<Arc<NativeTerrainEnvironment>>,
    pub(super) geometry: Arc<NativeTerrainGeometry>,
    pub(super) grass_layers: Arc<[NativeTerrainGrassLayer]>,
}

impl NativeTerrain {
    pub fn open(
        asset_root: impl AsRef<Path>,
        descriptor_path: &str,
        expected_descriptor_blake3: &str,
    ) -> Result<Self, NativeTerrainError> {
        Self::open_with_authoritative_environment(
            asset_root,
            descriptor_path,
            expected_descriptor_blake3,
            None,
        )
    }

    /// Opens terrain with a scene/registry-owned environment acceptance hash.
    ///
    /// The migrated world-v1 registry is the runtime authority for exact
    /// scene, terrain and environment bytes. Older terrain descriptors retain
    /// their pre-migration environment digest as provenance, so the world
    /// loader supplies the already-validated registry digest here instead of
    /// silently weakening byte verification.
    pub(crate) fn open_with_authoritative_environment(
        asset_root: impl AsRef<Path>,
        descriptor_path: &str,
        expected_descriptor_blake3: &str,
        authoritative_environment_blake3: Option<&str>,
    ) -> Result<Self, NativeTerrainError> {
        validate_relative_path(descriptor_path, "json")?;
        let canonical_map = descriptor_path.starts_with("map/tiles/map_")
            && descriptor_path.ends_with("/terrain/terrain.json");
        let canonical_world_map = descriptor_path.starts_with("world/maps/")
            && descriptor_path.ends_with("/terrain/terrain.json");
        let canonical_tutorial = descriptor_path.starts_with("world/tutorial/terrain/tiles/")
            && descriptor_path.ends_with("/terrain.json");
        if !canonical_map && !canonical_world_map && !canonical_tutorial {
            return Err(NativeTerrainError::new(
                "native terrain descriptor must use a canonical unified map path",
            ));
        }
        validate_plain_blake3(expected_descriptor_blake3, "terrain descriptor")?;
        let asset_root = asset_root.as_ref();
        let descriptor_file = join_relative(asset_root, descriptor_path);
        let descriptor_bytes = read_file(&descriptor_file, "native terrain descriptor")?;
        verify_plain_hash(
            &descriptor_bytes,
            expected_descriptor_blake3,
            &descriptor_file,
        )?;
        let descriptor: NativeTerrainDescriptor = serde_json::from_slice(&descriptor_bytes)
            .map_err(|error| {
                NativeTerrainError::new(format!(
                    "invalid native terrain JSON {}: {error}",
                    descriptor_file.display()
                ))
            })?;
        validate_descriptor(&descriptor)?;
        if authoritative_environment_blake3.is_some() && descriptor.environment.is_none() {
            return Err(NativeTerrainError::new(
                "authoritative environment hash has no terrain environment reference",
            ));
        }
        let terrain_root = descriptor_file.parent().ok_or_else(|| {
            NativeTerrainError::new("native terrain descriptor has no parent directory")
        })?;
        let environment = descriptor
            .environment
            .as_ref()
            .map(|reference| {
                load_environment(terrain_root, reference, authoritative_environment_blake3)
            })
            .transpose()?
            .map(Arc::new);
        let height_samples = load_heightmap(terrain_root, &descriptor)?;
        let weight_images = descriptor
            .splat
            .weight_maps
            .iter()
            .map(|weight| {
                let image = load_rgba_image(
                    terrain_root,
                    &weight.path,
                    weight.width,
                    weight.height,
                    &weight.png_blake3,
                    &weight.canonical_rgba_blake3,
                    "terrain weight map",
                )?;
                let mips =
                    load_texture_mip_files(terrain_root, &weight.mips, "terrain weight map mip")?;
                Ok((image, mips))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let (weight_maps, weight_map_mips): (Vec<_>, Vec<_>) = weight_images.into_iter().unzip();
        let layer_texture_images = descriptor
            .splat
            .layers
            .iter()
            .map(|layer| {
                let layer_root = if layer.albedo.path.starts_with("map/shared/") {
                    asset_root
                } else {
                    terrain_root
                };
                let image = load_rgba_image(
                    layer_root,
                    &layer.albedo.path,
                    layer.albedo.width,
                    layer.albedo.height,
                    &layer.albedo.png_blake3,
                    &layer.albedo.canonical_rgba_blake3,
                    "terrain layer albedo",
                )?;
                let mips =
                    load_texture_mip_files(layer_root, &layer.albedo.mips, "terrain layer mip")?;
                Ok((image, mips))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let (layer_images, layer_image_mips): (Vec<_>, Vec<_>) =
            layer_texture_images.into_iter().unzip();
        let lightmap_images = descriptor
            .lightmap
            .as_ref()
            .filter(|lightmap| lightmap.status == "exported")
            .map(|lightmap| {
                let path = lightmap.path.as_deref().ok_or_else(|| {
                    NativeTerrainError::new("exported terrain lightmap has no path")
                })?;
                let width = lightmap.width.ok_or_else(|| {
                    NativeTerrainError::new("exported terrain lightmap has no width")
                })?;
                let height = lightmap.height.ok_or_else(|| {
                    NativeTerrainError::new("exported terrain lightmap has no height")
                })?;
                let png_blake3 = lightmap.png_blake3.as_deref().ok_or_else(|| {
                    NativeTerrainError::new("exported terrain lightmap has no PNG hash")
                })?;
                let canonical_rgba_blake3 =
                    lightmap.canonical_rgba_blake3.as_deref().ok_or_else(|| {
                        NativeTerrainError::new(
                            "exported terrain lightmap has no canonical RGBA hash",
                        )
                    })?;
                let image = load_rgba_image(
                    terrain_root,
                    path,
                    width,
                    height,
                    png_blake3,
                    canonical_rgba_blake3,
                    "terrain lightmap",
                )?;
                let mips = load_texture_mip_files(
                    terrain_root,
                    lightmap.mips.as_deref().unwrap_or(&[]),
                    "terrain lightmap mip",
                )?;
                Ok((image, mips))
            })
            .transpose()?;
        let (lightmap, lightmap_mips) = lightmap_images.map_or((None, None), |(image, mips)| {
            (Some(Arc::new(image)), Some(Arc::from(mips)))
        });
        let gameplay_attributes = descriptor
            .gameplay_attributes
            .as_ref()
            .map(|metadata| load_gameplay_attributes(terrain_root, metadata))
            .transpose()?
            .map(Arc::new);
        let geometry = build_geometry(&descriptor, &height_samples)?;
        let grass_layers = load_grass_layers(terrain_root, descriptor_path, &descriptor)?;
        Ok(Self {
            descriptor_path: descriptor_path.to_owned(),
            descriptor_blake3: expected_descriptor_blake3.to_owned(),
            descriptor,
            height_samples: height_samples.into(),
            weight_maps: weight_maps.into(),
            weight_map_mips: weight_map_mips.into(),
            layer_images: layer_images.into(),
            layer_image_mips: layer_image_mips.into(),
            lightmap,
            lightmap_mips,
            gameplay_attributes,
            environment,
            geometry: Arc::new(geometry),
            grass_layers: grass_layers.into(),
        })
    }

    #[must_use]
    pub fn descriptor(&self) -> &NativeTerrainDescriptor {
        &self.descriptor
    }

    #[must_use]
    pub fn descriptor_path(&self) -> &str {
        &self.descriptor_path
    }

    #[must_use]
    pub fn descriptor_blake3(&self) -> &str {
        &self.descriptor_blake3
    }

    #[must_use]
    pub fn height_samples(&self) -> &[u16] {
        &self.height_samples
    }

    #[must_use]
    pub fn geometry(&self) -> &Arc<NativeTerrainGeometry> {
        &self.geometry
    }

    #[must_use]
    pub fn gameplay_attributes_available(&self) -> bool {
        self.gameplay_attributes.is_some()
    }

    #[must_use]
    pub fn environment(&self) -> Option<&NativeTerrainEnvironment> {
        self.environment.as_deref()
    }
}

pub(super) fn validate_splat_contract(descriptor: &NativeTerrainDescriptor) -> Result<(), NativeTerrainError> {
    let splat = &descriptor.splat;
    if splat.resolution == 0
        || splat.base_map_resolution.is_some_and(|value| value == 0)
        || splat.layers.is_empty()
        || splat.layers.len() > NATIVE_TERRAIN_MAX_LAYER_COUNT
        || splat.weight_maps.is_empty()
        || splat.weight_maps.len() > NATIVE_TERRAIN_MAX_WEIGHT_MAP_COUNT
        || splat.weight_maps.len() != splat.layers.len().div_ceil(4)
    {
        return Err(NativeTerrainError::new(format!(
            "native terrain requires 1..={NATIVE_TERRAIN_MAX_LAYER_COUNT} layers and exactly ceil(layerCount/4) weight maps (maximum {NATIVE_TERRAIN_MAX_WEIGHT_MAP_COUNT})"
        )));
    }
    for (index, weight) in splat.weight_maps.iter().enumerate() {
        if weight.index != index
            || weight.width != splat.resolution
            || weight.height != splat.resolution
            || weight.color_space != "linear"
            || weight.orientation.source_decoded_rows != "unityTextureRows"
            || weight.orientation.canonical_transform != "flipY"
            || weight.orientation.canonical_uv.u != "+heightColumn (-nativeX)"
            || weight.orientation.canonical_uv.v != "+heightRow (+nativeZ)"
        {
            return Err(NativeTerrainError::new(format!(
                "weight map {index} differs from the canonical linear RGBA splat contract"
            )));
        }
        validate_relative_path(&weight.path, "png")?;
        if weight.path != format!("weights/weights_{index:02}.png") {
            return Err(NativeTerrainError::new(format!(
                "weight map {index} has a non-semantic path {:?}",
                weight.path
            )));
        }
        if let Some(source) = &weight.source {
            validate_pointer_source(source, &format!("weight map {index}"))?;
        }
        validate_prefixed_blake3(&weight.canonical_rgba_blake3, "weight RGBA")?;
        validate_prefixed_blake3(&weight.png_blake3, "weight PNG")?;
        validate_texture_contract_metadata(
            weight.sampler.as_ref(),
            &weight.mips,
            weight.width,
            weight.height,
            "linear",
            &format!("weight map {index}"),
        )?;
    }
    let channels = ["r", "g", "b", "a"];
    for (index, layer) in splat.layers.iter().enumerate() {
        let expected_map = index / 4;
        let expected_channel = index % 4;
        if layer.index != index
            || layer.weight.map_index != expected_map
            || layer.weight.channel_index != expected_channel
            || layer.weight.channel != channels[expected_channel]
            || layer.weight.map_path != splat.weight_maps[expected_map].path
            || !layer.tile_size.x.is_finite()
            || !layer.tile_size.y.is_finite()
            || layer.tile_size.x <= 0.0
            || layer.tile_size.y <= 0.0
            || !matches!(layer.mode, 0 | 1)
        {
            return Err(NativeTerrainError::new(format!(
                "terrain layer {index} has an invalid weight route, mode, or tile size"
            )));
        }
        validate_readable_name(
            &layer.true_texture_name,
            &format!("terrain layer {index} trueTextureName"),
        )?;
        validate_relative_path(&layer.albedo.path, "png")?;
        let expected_path = format!("layers/{}/albedo.png", layer.true_texture_name);
        let shared_path = layer.albedo.path.starts_with("map/shared/terrain/layers/")
            && layer.albedo.path.ends_with("/mips/mip_00.png");
        if (layer.albedo.path != expected_path && !shared_path)
            || layer.albedo.width == 0
            || layer.albedo.height == 0
            || layer.albedo.color_space != "srgb"
            || layer.albedo.orientation.source_decoded_rows != "unityTextureRows"
            || layer.albedo.orientation.canonical_transform != "flipY"
        {
            return Err(NativeTerrainError::new(format!(
                "terrain layer {index} has a non-semantic albedo path or wrong color/orientation metadata"
            )));
        }
        if let Some(source) = &layer.albedo.source {
            validate_pointer_source(source, &format!("terrain layer {index}"))?;
        }
        validate_prefixed_blake3(&layer.albedo.canonical_rgba_blake3, "layer RGBA")?;
        validate_prefixed_blake3(&layer.albedo.png_blake3, "layer PNG")?;
        validate_texture_contract_metadata(
            layer.albedo.sampler.as_ref(),
            &layer.albedo.mips,
            layer.albedo.width,
            layer.albedo.height,
            "srgb",
            &format!("terrain layer {index}"),
        )?;
        match (&layer.mode_source, &layer.mode_evidence) {
            (None, None) => {}
            (Some(source), Some(evidence))
                if source == "serializedField"
                    && evidence
                        .get("fieldPresence")
                        .and_then(serde_json::Value::as_str)
                        == Some("present") => {}
            (Some(source), Some(evidence))
                if source == "implicitSchemaDefault"
                    && layer.mode == 0
                    && evidence
                        .get("fieldPresence")
                        .and_then(serde_json::Value::as_str)
                        == Some("absent")
                    && evidence
                        .get("resolvedValue")
                        .and_then(serde_json::Value::as_i64)
                        == Some(0) => {}
            _ => {
                return Err(NativeTerrainError::new(format!(
                    "terrain layer {index} has contradictory mode provenance"
                )));
            }
        }
    }
    Ok(())
}

pub(super) fn load_heightmap(
    terrain_root: &Path,
    descriptor: &NativeTerrainDescriptor,
) -> Result<Vec<u16>, NativeTerrainError> {
    let metadata = &descriptor.heightmap;
    let path = join_relative(terrain_root, &metadata.path);
    let png = read_file(&path, "heightmap PNG")?;
    verify_prefixed_hash(&png, &metadata.png_blake3, &path)?;
    let decoded = decode_png(&png, &path)?;
    if decoded.color() != ColorType::L16
        || decoded.width() != descriptor.dimensions.width
        || decoded.height() != descriptor.dimensions.height
    {
        return Err(NativeTerrainError::new(format!(
            "{} must be exact {}x{} Gray16, got {}x{} {:?}",
            path.display(),
            descriptor.dimensions.width,
            descriptor.dimensions.height,
            decoded.width(),
            decoded.height(),
            decoded.color()
        )));
    }
    let samples = decoded.into_luma16().into_raw();
    if samples.len() != descriptor.dimensions.sample_count {
        return Err(NativeTerrainError::new(
            "decoded heightmap sample count differs from terrain.json",
        ));
    }
    let raw_min = samples.iter().copied().min().unwrap_or_default();
    let raw_max = samples.iter().copied().max().unwrap_or_default();
    if raw_min != metadata.raw_min || raw_max != metadata.raw_max {
        return Err(NativeTerrainError::new(format!(
            "heightmap raw range mismatch: expected {}..={}, got {raw_min}..={raw_max}",
            metadata.raw_min, metadata.raw_max
        )));
    }
    let canonical_bytes = u16_little_endian_bytes(samples.iter().copied());
    verify_prefixed_hash(
        &canonical_bytes,
        &metadata.canonical_order_raw_blake3,
        &path,
    )?;

    let width = descriptor.dimensions.width as usize;
    let height = descriptor.dimensions.height as usize;
    let mut source_order = Vec::with_capacity(samples.len());
    for column in 0..width {
        for row in 0..height {
            source_order.push(samples[row * width + column]);
        }
    }
    let source_bytes = u16_little_endian_bytes(source_order);
    verify_prefixed_hash(&source_bytes, &metadata.source_order_raw_blake3, &path)?;
    Ok(samples)
}

/// Maximum splat taps walked along the texture footprint's long axis while the
/// `ANISOTROPIC FILTERING` graphics option is enabled. The terrain composites
/// every layer through manual `textureLoad` fetches, so a WebGPU sampler
/// `anisotropy_clamp` can never reach it; the shader performs the same
/// construction itself.
pub const NATIVE_TERRAIN_ANISOTROPIC_TAPS: f32 = 4.0;

/// One tap is the exact legacy `max(|ddx|, |ddy|)` single-mip footprint, which
/// is what the option's disabled state must reproduce.
pub const NATIVE_TERRAIN_ISOTROPIC_TAPS: f32 = 1.0;

#[derive(Debug, Clone, Copy, ShaderType)]
pub struct NativeTerrainMaterialUniform {
    /// xy: repetitions across the full terrain extent; z: source mode.
    pub layer_tile_scale_mode: [Vec4; NATIVE_TERRAIN_MAX_LAYER_COUNT],
    /// xy: exact source width/height; z: last mip level; w: mip bias.
    pub layer_source_size: [Vec4; NATIVE_TERRAIN_MAX_LAYER_COUNT],
    /// x: filter mode (0 point, 1 bilinear, 2 trilinear); y: last mip level;
    /// z: mip bias; w: base resolution.
    pub weight_source_sampler: [Vec4; NATIVE_TERRAIN_MAX_WEIGHT_MAP_COUNT],
    /// x: active layer count; y: exact legacy `m_RenderMode`;
    /// z: `m_SplatMapDistance`.
    pub metadata: Vec4,
    /// rgb: exact `cnPlayerCamera.DefaultAmbience` light tint; a is reserved.
    pub ambience_light: Vec4,
    /// rgb: exact applied fog color; a: exponential fog density (zero disables fog).
    pub ambience_fog: Vec4,
    /// x: maximum anisotropic taps for the manual splat sampler; yzw reserved.
    pub render_quality: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[type_path = "ffone_client::native_terrain"]
pub struct NativeTerrainMaterial {
    #[uniform(0)]
    pub uniform: NativeTerrainMaterialUniform,
    #[texture(1, dimension = "2d_array")]
    #[sampler(2)]
    pub weight_maps: Handle<Image>,
    #[texture(3, dimension = "2d_array")]
    #[sampler(4)]
    pub layer_albedos: Handle<Image>,
    #[texture(5)]
    #[sampler(6)]
    pub lightmap: Handle<Image>,
}

impl Material for NativeTerrainMaterial {
    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn fragment_shader() -> ShaderRef {
        AssetPath::from_path_buf(embedded_path!("native_terrain.wgsl"))
            .with_source("embedded")
            .into()
    }
}

#[derive(Component)]
pub struct PendingNativeHeightmapTerrain {
    pub(super) terrain: Option<Arc<NativeTerrain>>,
    pub(super) task: Option<Task<PreparedNativeHeightmapTerrain>>,
}

pub(super) struct PreparedNativeHeightmapTerrain {
    pub(super) terrain: Arc<NativeTerrain>,
    pub(super) geometry: Arc<NativeTerrainGeometry>,
    pub(super) mesh: Mesh,
    pub(super) weight_maps: Image,
    pub(super) layer_albedos: Image,
    pub(super) lightmap: Image,
    pub(super) uniform: NativeTerrainMaterialUniform,
}

pub(super) struct PreparedNativeTerrainGrassLayer {
    pub(super) chunks: VecDeque<PreparedNativeTerrainGrassChunk>,
    pub(super) texture_path: String,
    pub(super) authored_layer_count: usize,
    pub(super) instance_count: usize,
    pub(super) rendered_chunks: usize,
    pub(super) material: Option<Handle<StandardMaterial>>,
}

pub(super) struct PreparedNativeTerrainGrassChunk {
    pub(super) mesh: Mesh,
    pub(super) instance_count: usize,
    pub(super) visibility_end: f32,
}

/// Deferred detail preparation begins only after the base heightfield,
/// weights and material are ready for GPU admission. This keeps tens of
/// thousands of authored blades from delaying seam coverage while preserving
/// every density instance and its exact generated geometry.
#[derive(Component)]
pub(super) struct PendingNativeTerrainGrassPreparation {
    pub(super) descriptor_path: String,
    pub(super) terrain: Option<Arc<NativeTerrain>>,
    pub(super) task: Option<Task<Vec<PreparedNativeTerrainGrassLayer>>>,
}

#[derive(Component)]
pub(super) struct PendingNativeTerrainGrass {
    pub(super) descriptor_path: String,
    pub(super) layers: VecDeque<PreparedNativeTerrainGrassLayer>,
    pub(super) total_layers: usize,
    pub(super) total_instances: usize,
    pub(super) rendered_layers: usize,
    pub(super) rendered_instances: usize,
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct NativeHeightmapTerrain {
    pub descriptor_path: String,
    pub true_name: String,
    pub width: u32,
    pub height: u32,
    pub layers: usize,
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum NativeTerrainGameplayAttributesStatus {
    Ready { width: u32, height: u32 },
    MissingSidecar { descriptor_path: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTerrainGameplayAttributeSample {
    Value(u8),
    OutsideTerrain,
    SidecarMissing,
}

/// Native implementation status for the old client's
/// `Terrain.SampleHeight`/`TerrainData.GetInterpolatedHeight` path.
///
/// This remains separate from [`NativeHeightmapCollider::ground_height`]:
/// Unity's interpolated-height query blends four neighboring height samples,
/// while the visible/collision mesh evaluates one of the two rendered
/// triangles in a cell.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum NativeTerrainLegacyInterpolatedHeightStatus {
    Ready {
        known_consumer_evidence_count: usize,
        normalized_coordinates_clamped: bool,
        four_neighbor_bilinear_interpolation: bool,
        native_x_reflected: bool,
        owner_translation_y_application_count: u8,
    },
}

/// Typed result from the legacy Unity interpolated-height query.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NativeTerrainLegacyInterpolatedHeightSample {
    Height(f32),
    Rejected(NativeTerrainLegacyInterpolatedHeightRejection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTerrainLegacyInterpolatedHeightRejection {
    NonFiniteInput,
    NonFiniteTerrainTransform,
    SingularTerrainTransform,
    InvalidHeightfield,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegacyTerrainInterpolatedHeightEvidence {
    pub project_relative_path: &'static str,
    pub call: &'static str,
}

/// Known runtime consumers plus the decompiled Unity/MapAttribute call path
/// whose coordinate and owner-Y behavior this native implementation preserves.
pub const LEGACY_TERRAIN_INTERPOLATED_HEIGHT_EVIDENCE: [LegacyTerrainInterpolatedHeightEvidence;
    5] = [
    LegacyTerrainInterpolatedHeightEvidence {
        project_relative_path: "work/ilspy-6877-csharp/cnOwnAvatarStatus.cs",
        call: "MapAttributeTable.GetHeight -> Terrain.SampleHeight",
    },
    LegacyTerrainInterpolatedHeightEvidence {
        project_relative_path: "work/ilspy-6877-csharp/CnEquip.cs",
        call: "MapAttributeTable.GetHeight -> Terrain.SampleHeight",
    },
    LegacyTerrainInterpolatedHeightEvidence {
        project_relative_path: "work/ilspy-6877-csharp/NpcMoveController.cs",
        call: "MapAttributeTable.GetHeight -> Terrain.SampleHeight",
    },
    LegacyTerrainInterpolatedHeightEvidence {
        project_relative_path: "work/ilspy-b8c3-firstpass/MapAttributeTable.cs",
        call: "Terrain.SampleHeight(pos) + transform.position.y",
    },
    LegacyTerrainInterpolatedHeightEvidence {
        project_relative_path: "work/map-zero-quaternion-investigation/unityengine-decompiled/UnityEngine/Terrain.cs",
        call: "subtract Terrain.GetPosition, normalize by TerrainData.size, then GetInterpolatedHeight",
    },
];

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum NativeTerrainDetailStatus {
    SourceContractPending,
    Loading {
        rendered_layers: usize,
        total_layers: usize,
        rendered_instances: usize,
        total_instances: usize,
    },
    Ready {
        rendered_layers: usize,
        rendered_instances: usize,
    },
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum NativeTerrainTreeStatus {
    SourceContractPending,
}

/// Exact O(1) collision sampler for the same regular-grid vertices and
/// triangle diagonal submitted to the GPU.
#[derive(Component, Debug, Clone)]
pub struct NativeHeightmapCollider {
    pub(super) source_mesh: Handle<Mesh>,
    pub(super) source_descriptor_path: String,
    pub(super) geometry: Arc<NativeTerrainGeometry>,
    pub(super) width: usize,
    pub(super) height: usize,
    pub(super) sample_spacing_x: f32,
    pub(super) sample_spacing_z: f32,
    pub(super) terrain_size_x: f32,
    pub(super) terrain_size_z: f32,
    pub(super) gameplay_attributes: Option<Arc<VerifiedGameplayAttributes>>,
}

/// Exact hit returned by the regular-grid TerrainCollider replacement.
/// Fraction is measured along the caller's complete world-space segment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct NativeTerrainSegmentHit {
    pub fraction: f32,
    pub point: Vec3,
    pub normal: Vec3,
}

pub(super) const NATIVE_TERRAIN_SEGMENT_EPSILON: f32 = 0.000_01;
