use super::*;

impl NativeHeightmapCollider {
    #[cfg(test)]
    pub(crate) fn test_heightfield(width: usize, height: usize, heights: &[f32]) -> Self {
        assert_eq!(heights.len(), width * height);
        let positions = heights
            .iter()
            .enumerate()
            .map(|(i, &y)| [-((i % width) as f32), y, (i / width) as f32])
            .collect::<Vec<_>>();
        let mut indices = Vec::new();
        for row in 0..height - 1 {
            for column in 0..width - 1 {
                let i = (row * width + column) as u32;
                indices.extend([
                    i,
                    i + 1,
                    i + width as u32,
                    i + 1,
                    i + width as u32 + 1,
                    i + width as u32,
                ]);
            }
        }
        Self {
            source_mesh: Handle::default(),
            source_descriptor_path: String::new(),
            geometry: Arc::new(NativeTerrainGeometry {
                positions: positions.into(),
                indices: indices.into(),
                normals: vec![[0.0, 1.0, 0.0]; heights.len()].into(),
                uvs: vec![[0.0, 0.0]; heights.len()].into(),
            }),
            width,
            height,
            sample_spacing_x: 1.0,
            sample_spacing_z: 1.0,
            terrain_size_x: (width - 1) as f32,
            terrain_size_z: (height - 1) as f32,
            gameplay_attributes: None,
        }
    }

    /// Visits exact rendered triangles near a world-space swept capsule AABB.
    /// Only nearby grid cells are read; shifted vertices can belong to a
    /// neighboring cell, so retain one extra owner on each side.
    pub(crate) fn visit_collision_triangles(
        &self,
        global: &GlobalTransform,
        minimum: Vec3,
        maximum: Vec3,
        mut visit: impl FnMut([Vec3; 3]),
    ) {
        let world = global.to_matrix();
        let inverse = world.inverse();
        if self.width < 2
            || self.height < 2
            || !inverse.is_finite()
            || !minimum.is_finite()
            || !maximum.is_finite()
            || !self.sample_spacing_x.is_finite()
            || self.sample_spacing_x <= 0.0
            || !self.sample_spacing_z.is_finite()
            || self.sample_spacing_z <= 0.0
        {
            return;
        }
        let mut low = Vec3::splat(f32::INFINITY);
        let mut high = Vec3::splat(f32::NEG_INFINITY);
        for x in [minimum.x, maximum.x] {
            for y in [minimum.y, maximum.y] {
                for z in [minimum.z, maximum.z] {
                    let local = inverse.transform_point3(Vec3::new(x, y, z));
                    let grid = Vec3::new(
                        -local.x / self.sample_spacing_x,
                        local.y,
                        local.z / self.sample_spacing_z,
                    );
                    low = low.min(grid);
                    high = high.max(grid);
                }
            }
        }
        if high.x < -1.0 || high.z < -1.0 || low.x > self.width as f32 || low.z > self.height as f32
        {
            return;
        }
        let first_column = ((low.x - NATIVE_TERRAIN_SEGMENT_EPSILON).floor() - 1.0)
            .clamp(0.0, (self.width - 2) as f32) as usize;
        let last_column = ((high.x + NATIVE_TERRAIN_SEGMENT_EPSILON).floor() + 1.0)
            .clamp(0.0, (self.width - 2) as f32) as usize;
        let first_row = ((low.z - NATIVE_TERRAIN_SEGMENT_EPSILON).floor() - 1.0)
            .clamp(0.0, (self.height - 2) as f32) as usize;
        let last_row = ((high.z + NATIVE_TERRAIN_SEGMENT_EPSILON).floor() + 1.0)
            .clamp(0.0, (self.height - 2) as f32) as usize;
        for row in first_row..=last_row {
            for column in first_column..=last_column {
                let offset = (row * (self.width - 1) + column) * 6;
                for indices in self.geometry.indices[offset..offset + 6].chunks_exact(3) {
                    let vertices = [indices[0], indices[1], indices[2]].map(|index| {
                        world.transform_point3(Vec3::from_array(
                            self.geometry.positions[index as usize],
                        ))
                    });
                    let triangle_min = vertices[0].min(vertices[1]).min(vertices[2]);
                    let triangle_max = vertices[0].max(vertices[1]).max(vertices[2]);
                    if triangle_min.cmple(maximum).all() && triangle_max.cmpge(minimum).all() {
                        visit(vertices);
                    }
                }
            }
        }
    }

    /// Builds the runtime sampler from the same validated terrain used by rendering.
    #[must_use]
    pub fn from_terrain(terrain: &NativeTerrain, source_mesh: Handle<Mesh>) -> Self {
        Self {
            source_mesh,
            source_descriptor_path: terrain.descriptor_path().to_owned(),
            geometry: terrain.geometry().clone(),
            width: terrain.descriptor().dimensions.width as usize,
            height: terrain.descriptor().dimensions.height as usize,
            sample_spacing_x: terrain.descriptor().scale.sample_spacing_x as f32,
            sample_spacing_z: terrain.descriptor().scale.sample_spacing_z as f32,
            terrain_size_x: terrain.descriptor().scale.extent_x as f32,
            terrain_size_z: terrain.descriptor().scale.extent_z as f32,
            gameplay_attributes: terrain.gameplay_attributes.clone(),
        }
    }

    #[must_use]
    pub fn source_mesh(&self) -> &Handle<Mesh> {
        &self.source_mesh
    }

    #[must_use]
    pub fn source_descriptor_path(&self) -> &str {
        &self.source_descriptor_path
    }

    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.geometry.vertex_count()
    }

    #[must_use]
    pub fn index_count(&self) -> usize {
        self.geometry.index_count()
    }

    #[must_use]
    pub fn gameplay_attribute(
        &self,
        global: &GlobalTransform,
        world_x: f32,
        world_z: f32,
    ) -> NativeTerrainGameplayAttributeSample {
        let Some(attributes) = &self.gameplay_attributes else {
            return NativeTerrainGameplayAttributeSample::SidecarMissing;
        };
        if !world_x.is_finite() || !world_z.is_finite() {
            return NativeTerrainGameplayAttributeSample::OutsideTerrain;
        }
        let local_from_world = global.to_matrix().inverse();
        if !local_from_world.is_finite() {
            return NativeTerrainGameplayAttributeSample::OutsideTerrain;
        }
        let local = local_from_world.transform_point3(Vec3::new(world_x, 0.0, world_z));
        let spacing_x = self.sample_spacing_x;
        let spacing_z = self.sample_spacing_z;
        if spacing_x <= 0.0 || spacing_z <= 0.0 {
            return NativeTerrainGameplayAttributeSample::OutsideTerrain;
        }
        let column = (-local.x / spacing_x).floor() as i32;
        let row = (local.z / spacing_z).floor() as i32;
        if column < 0
            || row < 0
            || column >= attributes.width as i32
            || row >= attributes.height as i32
        {
            return NativeTerrainGameplayAttributeSample::OutsideTerrain;
        }
        let index = column as usize + row as usize * attributes.width as usize;
        NativeTerrainGameplayAttributeSample::Value(attributes.values[index])
    }

    /// Exact native replacement for the legacy `MapAttributeTable.GetHeight`
    /// path.
    ///
    /// Old Unity `Terrain.SampleHeight` subtracts the terrain world position,
    /// normalizes X/Z by `TerrainData.size`, and calls the four-neighbor
    /// interpolated-height query. `MapAttributeTable` then adds the owning
    /// transform's world Y exactly once. Native world X is reflected relative
    /// to Unity, while Z and Y retain their signs.
    #[must_use]
    pub fn legacy_interpolated_height(
        &self,
        global: &GlobalTransform,
        world_x: f32,
        world_z: f32,
    ) -> NativeTerrainLegacyInterpolatedHeightSample {
        use NativeTerrainLegacyInterpolatedHeightRejection as Rejection;
        use NativeTerrainLegacyInterpolatedHeightSample::{Height, Rejected};

        if !world_x.is_finite() || !world_z.is_finite() {
            return Rejected(Rejection::NonFiniteInput);
        }
        if self.width < 2
            || self.height < 2
            || self.geometry.positions.len() != self.width * self.height
            || !self.terrain_size_x.is_finite()
            || !self.terrain_size_z.is_finite()
            || self.terrain_size_x <= 0.0
            || self.terrain_size_z <= 0.0
        {
            return Rejected(Rejection::InvalidHeightfield);
        }

        let world_from_terrain = global.to_matrix();
        if !world_from_terrain.is_finite() {
            return Rejected(Rejection::NonFiniteTerrainTransform);
        }
        if !world_from_terrain.inverse().is_finite() {
            return Rejected(Rejection::SingularTerrainTransform);
        }
        let terrain_position = global.translation();
        if !terrain_position.is_finite() {
            return Rejected(Rejection::NonFiniteTerrainTransform);
        }

        // Unity X = -native X. Reflect both the query and the owning terrain
        // position before applying Terrain.SampleHeight's subtraction.
        let unity_world_x = -world_x;
        let unity_terrain_x = -terrain_position.x;
        let normalized_x =
            ((unity_world_x - unity_terrain_x) / self.terrain_size_x).clamp(0.0, 1.0);
        let normalized_z = ((world_z - terrain_position.z) / self.terrain_size_z).clamp(0.0, 1.0);

        let column_coordinate = normalized_x * (self.width - 1) as f32;
        let row_coordinate = normalized_z * (self.height - 1) as f32;
        let column = (column_coordinate.floor() as usize).min(self.width - 2);
        let row = (row_coordinate.floor() as usize).min(self.height - 2);
        let u = column_coordinate - column as f32;
        let v = row_coordinate - row as f32;
        let i00 = row * self.width + column;
        let h00 = self.geometry.positions[i00][1];
        let h10 = self.geometry.positions[i00 + 1][1];
        let h01 = self.geometry.positions[i00 + self.width][1];
        let h11 = self.geometry.positions[i00 + self.width + 1][1];
        if ![h00, h10, h01, h11].into_iter().all(f32::is_finite) {
            return Rejected(Rejection::InvalidHeightfield);
        }

        let near = h00 + (h10 - h00) * u;
        let far = h01 + (h11 - h01) * u;
        let local_height = near + (far - near) * v;
        let world_height = local_height + terrain_position.y;
        if world_height.is_finite() {
            Height(world_height)
        } else {
            Rejected(Rejection::NonFiniteTerrainTransform)
        }
    }

    /// Intersects a world-space segment with the exact two rendered triangles
    /// in every crossed heightfield cell. This is the native equivalent of the
    /// `TerrainCollider` part of `cnPlayerCamera`'s layer-masked raycast.
    #[must_use]
    pub(crate) fn segment_hit(
        &self,
        global: &GlobalTransform,
        start: Vec3,
        end: Vec3,
    ) -> Option<NativeTerrainSegmentHit> {
        if !start.is_finite()
            || !end.is_finite()
            || self.width < 2
            || self.height < 2
            || self.geometry.positions.len() != self.width * self.height
        {
            return None;
        }
        let world_from_local = global.to_matrix();
        let local_from_world = world_from_local.inverse();
        if !world_from_local.is_finite() || !local_from_world.is_finite() {
            return None;
        }
        let local_start = local_from_world.transform_point3(start);
        let local_end = local_from_world.transform_point3(end);
        if !local_start.is_finite() || !local_end.is_finite() {
            return None;
        }
        let spacing_x = self.sample_spacing_x;
        let spacing_z = self.sample_spacing_z;
        if !spacing_x.is_finite() || !spacing_z.is_finite() || spacing_x <= 0.0 || spacing_z <= 0.0
        {
            return None;
        }

        let start_column = -local_start.x / spacing_x;
        let end_column = -local_end.x / spacing_x;
        let start_row = local_start.z / spacing_z;
        let end_row = local_end.z / spacing_z;
        let minimum_column = start_column.min(end_column);
        let maximum_column = start_column.max(end_column);
        let minimum_row = start_row.min(end_row);
        let maximum_row = start_row.max(end_row);
        let terrain_maximum_column = (self.width - 1) as f32;
        let terrain_maximum_row = (self.height - 1) as f32;
        if maximum_column < -1.0 - NATIVE_TERRAIN_SEGMENT_EPSILON
            || minimum_column > terrain_maximum_column + 1.0 + NATIVE_TERRAIN_SEGMENT_EPSILON
            || maximum_row < -1.0 - NATIVE_TERRAIN_SEGMENT_EPSILON
            || minimum_row > terrain_maximum_row + 1.0 + NATIVE_TERRAIN_SEGMENT_EPSILON
        {
            return None;
        }

        // The gameplay camera ray is at most twelve source units long while
        // published terrain cells are four units wide. A clamped X/Z segment
        // rectangle therefore visits only a small constant number of cells;
        // extending by epsilon includes both owners of an exact grid edge.
        let maximum_cell_column = (self.width - 2) as isize;
        let maximum_cell_row = (self.height - 2) as isize;
        // Unity m_Shifts can move a vertex by one complete sample step. Include
        // the neighboring regular-grid owner on every side so shifted vertical
        // walls and collapsed edge triangles remain part of collision queries.
        let first_column = ((minimum_column - NATIVE_TERRAIN_SEGMENT_EPSILON).floor() as isize - 1)
            .clamp(0, maximum_cell_column) as usize;
        let last_column = ((maximum_column + NATIVE_TERRAIN_SEGMENT_EPSILON).floor() as isize + 1)
            .clamp(0, maximum_cell_column) as usize;
        let first_row = ((minimum_row - NATIVE_TERRAIN_SEGMENT_EPSILON).floor() as isize - 1)
            .clamp(0, maximum_cell_row) as usize;
        let last_row = ((maximum_row + NATIVE_TERRAIN_SEGMENT_EPSILON).floor() as isize + 1)
            .clamp(0, maximum_cell_row) as usize;

        let mut nearest: Option<NativeTerrainSegmentHit> = None;
        for row in first_row..=last_row {
            for column in first_column..=last_column {
                let triangle_offset = (row * (self.width - 1) + column) * 6;
                let triangles = self
                    .geometry
                    .indices
                    .get(triangle_offset..triangle_offset + 6)?;
                for triangle in triangles.chunks_exact(3) {
                    let a = world_from_local.transform_point3(Vec3::from_array(
                        self.geometry.positions[triangle[0] as usize],
                    ));
                    let b = world_from_local.transform_point3(Vec3::from_array(
                        self.geometry.positions[triangle[1] as usize],
                    ));
                    let c = world_from_local.transform_point3(Vec3::from_array(
                        self.geometry.positions[triangle[2] as usize],
                    ));
                    let Some(hit) = native_terrain_segment_triangle_hit(start, end, a, b, c) else {
                        continue;
                    };
                    if nearest.is_none_or(|current| hit.fraction < current.fraction) {
                        nearest = Some(hit);
                    }
                }
            }
        }
        nearest
    }

    #[must_use]
    pub(crate) fn ground_contact(
        &self,
        global: &GlobalTransform,
        world_x: f32,
        world_z: f32,
        minimum_y: f32,
        maximum_y: f32,
    ) -> Option<NativeTerrainSegmentHit> {
        if !world_x.is_finite()
            || !world_z.is_finite()
            || !minimum_y.is_finite()
            || !maximum_y.is_finite()
            || minimum_y > maximum_y
        {
            return None;
        }
        self.segment_hit(
            global,
            Vec3::new(world_x, maximum_y, world_z),
            Vec3::new(world_x, minimum_y, world_z),
        )
    }

    #[must_use]
    pub fn ground_height(
        &self,
        global: &GlobalTransform,
        world_x: f32,
        world_z: f32,
        minimum_y: f32,
        maximum_y: f32,
    ) -> Option<f32> {
        self.ground_contact(global, world_x, world_z, minimum_y, maximum_y)
            .map(|hit| hit.point.y)
    }
}

pub(super) fn native_terrain_segment_triangle_hit(
    start: Vec3,
    end: Vec3,
    a: Vec3,
    b: Vec3,
    c: Vec3,
) -> Option<NativeTerrainSegmentHit> {
    if !start.is_finite() || !end.is_finite() || !a.is_finite() || !b.is_finite() || !c.is_finite()
    {
        return None;
    }
    let direction = end - start;
    let edge_ab = b - a;
    let edge_ac = c - a;
    let cross = direction.cross(edge_ac);
    let determinant = edge_ab.dot(cross);
    if determinant.abs() <= NATIVE_TERRAIN_SEGMENT_EPSILON {
        return None;
    }
    let inverse = determinant.recip();
    let from_a = start - a;
    let u = from_a.dot(cross) * inverse;
    if !(-NATIVE_TERRAIN_SEGMENT_EPSILON..=1.0 + NATIVE_TERRAIN_SEGMENT_EPSILON).contains(&u) {
        return None;
    }
    let q = from_a.cross(edge_ab);
    let v = direction.dot(q) * inverse;
    if v < -NATIVE_TERRAIN_SEGMENT_EPSILON || u + v > 1.0 + NATIVE_TERRAIN_SEGMENT_EPSILON {
        return None;
    }
    let fraction = edge_ac.dot(q) * inverse;
    if !(NATIVE_TERRAIN_SEGMENT_EPSILON..=1.0).contains(&fraction) {
        return None;
    }
    let normal = edge_ab.cross(edge_ac).normalize_or_zero();
    (normal != Vec3::ZERO).then_some(NativeTerrainSegmentHit {
        fraction,
        point: start + direction * fraction,
        normal,
    })
}

/// Geometry, source weights, tilings, texture bytes and the source-backed
/// FirstPass/AddPass/BlendPass compositor are exact. Lighting, fog, source
/// sampler/LOD behavior and the remaining Terrain component render fields stay
/// explicitly pending instead of being replaced by preview heuristics.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum NativeTerrainVisualParityStatus {
    GeometryAndWeightsExactVisualShaderPending {
        blend_layer_count: usize,
        source_shader_evidence_count: usize,
        pass_compositor_pending: bool,
        lighting_and_lightmap_pending: bool,
        fog_pending: bool,
        source_sampler_contract_pending: bool,
        lod_and_basemap_pending: bool,
        terrain_component_render_contract_pending: bool,
    },
}

/// Serialized `UnityEngine.TerrainTextureMode` values used by the legacy
/// Terrain renderer. Keeping this typed prevents Blend layers from silently
/// falling back to the weighted Splat formula.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i64)]
pub enum LegacyTerrainTextureMode {
    Splat = 0,
    Blend = 1,
}

/// Native shader-family index consumed by Unity 2.5.5
/// `SplatDatabase::GetMaterial`.
///
/// Retrobution serializes `m_RenderMode = 1` and `m_UseLightmap = 0` on every
/// published Terrain component. The latter is not the shader-family selector.
/// The player binary dispatches index 0 to `Vertexlit-*`, 1 to `Lightmap-*`
/// and 2 to `Realtime-*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i64)]
pub enum LegacyTerrainRenderMode {
    VertexLit = 0,
    Lightmap = 1,
    Realtime = 2,
}

impl TryFrom<i64> for LegacyTerrainRenderMode {
    type Error = NativeTerrainError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::VertexLit),
            1 => Ok(Self::Lightmap),
            2 => Ok(Self::Realtime),
            other => Err(NativeTerrainError::new(format!(
                "unsupported legacy terrain m_RenderMode {other}"
            ))),
        }
    }
}

/// One already-sampled layer input to the pure legacy pass compositor.
///
/// Only RGB is represented deliberately: the proven FirstPass, AddPass and
/// primary BlendPass programs never multiply their terrain RGB by the sampled
/// splat texture alpha.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LegacyTerrainCompositorLayer {
    pub mode: LegacyTerrainTextureMode,
    pub weight: f32,
    pub texture_rgb: Vec3,
}

/// Pure CPU reference equivalent of the proven legacy terrain pass blend.
///
/// Shader queue tags impose a stable partition: FirstPass `Geometry-100` and
/// AddPass `Geometry-99` submit every Splat control-map group before BlendPass
/// `Geometry-98`, while relative serialized order remains unchanged within
/// each pass type. Crucially, each serialized quartet remains one `_Control`
/// plus `_Splat0..3` draw; layers from different control maps are never packed
/// together. The legacy normalized render target saturates each completed
/// pass before the next fixed-function blend reads it.
#[must_use]
pub fn legacy_terrain_pass_composite(layers: &[LegacyTerrainCompositorLayer]) -> Vec3 {
    let saturate = |value: Vec3| value.clamp(Vec3::ZERO, Vec3::ONE);
    let mut framebuffer = Vec3::ZERO;
    let mut has_first_pass = false;

    for group in layers
        .chunks(4)
        .filter(|group| group[0].mode == LegacyTerrainTextureMode::Splat)
    {
        let source = saturate(group.iter().fold(Vec3::ZERO, |sum, layer| {
            sum + layer.texture_rgb * layer.weight
        }));
        if has_first_pass {
            framebuffer = saturate(framebuffer + source);
        } else {
            framebuffer = source;
            has_first_pass = true;
        }
    }

    for group in layers
        .chunks(4)
        .filter(|group| group[0].mode == LegacyTerrainTextureMode::Blend)
    {
        let (source, alpha) = legacy_realtime_blend_group(group);
        let source = saturate(source);
        let alpha = alpha.clamp(0.0, 1.0);
        framebuffer = source * alpha + framebuffer * (1.0 - alpha);
        framebuffer = saturate(framebuffer);
    }
    framebuffer
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegacyTerrainShaderEvidence {
    pub project_relative_path: &'static str,
    pub sha256: &'static str,
}

/// Exact decompiled shader evidence used to replace the current one-pass preview
/// compositor. These hashes make any later source-evidence drift fail visibly.
pub const LEGACY_TERRAIN_SHADER_EVIDENCE: [LegacyTerrainShaderEvidence; 12] = [
    LegacyTerrainShaderEvidence {
        project_relative_path: "work/main-shaders/AddPassLightmap.cg",
        sha256: "1c79af76cbd230a147a53b8647b2c0a8ea8c14f1897ae69774d696392a3ce7c9",
    },
    LegacyTerrainShaderEvidence {
        project_relative_path: "work/main-shaders/AddPassRealtime.cg",
        sha256: "0c23563f154e76de3bd57b3dde5fb82751ee059ff906ea40930808473ed4f772",
    },
    LegacyTerrainShaderEvidence {
        project_relative_path: "work/main-shaders/AddPassVertexLit.cg",
        sha256: "f7b1bed7318b8d0ed97fd4d683689e64c4e325346a31185676d55f5c23e07827",
    },
    LegacyTerrainShaderEvidence {
        project_relative_path: "work/main-shaders/BlendPassLightmap.cg",
        sha256: "e6692aa7cadfb127dc924842830265b98647218bc13c43b7a75065e2b1091319",
    },
    LegacyTerrainShaderEvidence {
        project_relative_path: "work/main-shaders/BlendPassRealtime.cg",
        sha256: "d023cce686fc8b2a19226660ec6002f48da42b2be09affd7093754ac98a1276e",
    },
    LegacyTerrainShaderEvidence {
        project_relative_path: "work/main-shaders/BlendPassVertexLit.cg",
        sha256: "9419afd183273e33b822617daea0aed135d0978bc1b8d80c8cac70c26794629d",
    },
    LegacyTerrainShaderEvidence {
        project_relative_path: "work/main-shaders/FirstPassLightmap.cg",
        sha256: "cd2452d914c37431c9cbf824aca196d3b63392e0c448420a201bd7d973970501",
    },
    LegacyTerrainShaderEvidence {
        project_relative_path: "work/main-shaders/FirstPassRealtime.cg",
        sha256: "3175114474d1142411117a35653d6556af1074d660c1bc6b48be8c9b9f08668f",
    },
    LegacyTerrainShaderEvidence {
        project_relative_path: "work/main-shaders/FirstPassVertexLit.cg",
        sha256: "5e359cb70cd9f7e52255120716587a275a2bee836b9c482dab0f8cc3c98d8227",
    },
    LegacyTerrainShaderEvidence {
        project_relative_path: "work/main-shaders/OldBlendPassLightmap.cg",
        sha256: "5a26b7c6b7320d9a14922bfa089d4564cd35e87fcc1edd80b5cbc3f220e75f99",
    },
    LegacyTerrainShaderEvidence {
        project_relative_path: "work/main-shaders/OldBlendPassRealtime.cg",
        sha256: "a1a674ca88a3f20e95d1912456671a80b7055472a3bcc8d2624b4da553f62e9f",
    },
    LegacyTerrainShaderEvidence {
        project_relative_path: "work/main-shaders/OldBlendPassVertexLit.cg",
        sha256: "1087fd58c2e48c9feb0deb3dfda79628af8c7bd2b6703fe758d5c072a7e29ad2",
    },
];

pub(super) fn prepare_native_heightmap_terrain(terrain: Arc<NativeTerrain>) -> PreparedNativeHeightmapTerrain {
    let geometry = terrain.geometry().clone();
    let mesh = geometry.to_mesh();
    let weight_maps = weight_array_image(&terrain);
    let layer_albedos = layer_array_image(&terrain);
    let lightmap = lightmap_image(&terrain);
    let uniform = terrain_material_uniform(&terrain);
    PreparedNativeHeightmapTerrain {
        terrain,
        geometry,
        mesh,
        weight_maps,
        layer_albedos,
        lightmap,
        uniform,
    }
}

pub(super) fn prepare_native_terrain_grass(
    terrain: Arc<NativeTerrain>,
) -> Vec<PreparedNativeTerrainGrassLayer> {
    // Unity detail prototypes that share an atlas are submitted together.
    // Preserve source order within each group so the generated blade positions
    // and colors remain deterministic, but do not create a separate material
    // and draw family for byte-identical prototype textures.
    let mut texture_groups: Vec<(String, Vec<&NativeTerrainGrassLayer>)> = Vec::new();
    for layer in terrain.grass_layers.iter() {
        if let Some((_, layers)) = texture_groups
            .iter_mut()
            .find(|(texture_path, _)| texture_path == &layer.texture_path)
        {
            layers.push(layer);
        } else {
            texture_groups.push((layer.texture_path.clone(), vec![layer]));
        }
    }
    let detail_distance = terrain
        .environment()
        .map(|environment| {
            environment
                .terrain_detail
                .serialized_terrain_render_baseline
                .serialized_fields
                .detail_object_distance as f32
        })
        .unwrap_or(80.0);
    texture_groups
        .into_iter()
        .filter_map(|(texture_path, layers)| {
            let authored_layer_count = layers.len();
            let chunks = build_grass_chunk_meshes(&terrain, &layers, detail_distance);
            if chunks.is_empty() {
                return None;
            }
            let instance_count = chunks.iter().map(|chunk| chunk.instance_count).sum();
            Some(PreparedNativeTerrainGrassLayer {
                chunks: chunks.into(),
                texture_path,
                authored_layer_count,
                instance_count,
                rendered_chunks: 0,
                material: None,
            })
        })
        .collect()
}

pub(super) fn native_heightmap_preparation_slots(running: usize) -> usize {
    NATIVE_TERRAIN_MAX_IN_FLIGHT_PREPARATIONS.saturating_sub(running)
}

pub(super) fn native_terrain_grass_preparation_slots(running: usize) -> usize {
    NATIVE_TERRAIN_MAX_IN_FLIGHT_GRASS_PREPARATIONS.saturating_sub(running)
}

pub(super) fn start_pending_native_heightmap_preparations(
    mut pending: Query<&mut PendingNativeHeightmapTerrain>,
) {
    let running = pending
        .iter_mut()
        .filter(|pending| pending.task.is_some())
        .count();
    let mut slots = native_heightmap_preparation_slots(running);
    if slots == 0 {
        return;
    }
    for mut pending in &mut pending {
        if pending.task.is_some() {
            continue;
        }
        let Some(terrain) = pending.terrain.take() else {
            continue;
        };
        pending.task = Some(
            AsyncComputeTaskPool::get()
                .spawn(async move { prepare_native_heightmap_terrain(terrain) }),
        );
        slots -= 1;
        if slots == 0 {
            break;
        }
    }
}

pub(crate) fn install_native_terrain(app: &mut App) {
    embedded_asset!(app, "native_terrain.wgsl");
    app.add_plugins(MaterialPlugin::<NativeTerrainMaterial>::default())
        .add_systems(
            Update,
            (
                start_pending_native_heightmap_preparations,
                materialize_native_heightmaps,
                prepare_pending_native_terrain_grass,
                materialize_native_terrain_grass,
            )
                .chain(),
        );
}
