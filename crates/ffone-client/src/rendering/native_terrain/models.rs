use super::*;

pub(super) fn validate_vertex_shifts(descriptor: &NativeTerrainDescriptor) -> Result<(), NativeTerrainError> {
    let shifts = &descriptor.heightmap.vertex_shifts;
    let encoding = descriptor.native_geometry.vertex_shift_encoding.as_ref();
    if shifts.is_empty() && encoding.is_none() {
        return Ok(());
    }
    let encoding = encoding.ok_or_else(|| {
        NativeTerrainError::new(
            "terrain vertex shifts are present without their exact Unity encoding contract",
        )
    })?;
    if encoding.source_field != "m_Heightmap.m_Shifts"
        || encoding.source_coordinates != "column = x, row = y"
        || encoding.source_index != "y + x * height"
        || encoding.canonical_index != "row * width + column"
        || encoding.flag_bits.negative_source_x != 1
        || encoding.flag_bits.positive_source_x != 2
        || encoding.flag_bits.negative_source_z != 4
        || encoding.flag_bits.positive_source_z != 8
        || encoding.position_formula
            != "nativeX -= (positiveSourceX - negativeSourceX) * sampleSpacingX; nativeZ += (positiveSourceZ - negativeSourceZ) * sampleSpacingZ"
        || encoding.uv_formula
            != "u += (positiveSourceX - negativeSourceX) / (2 * (width - 1)); v -= (positiveSourceZ - negativeSourceZ) / (2 * (height - 1))"
    {
        return Err(NativeTerrainError::new(
            "terrain vertex-shift encoding differs from the Unity heightmap contract",
        ));
    }

    let mut seen = std::collections::HashSet::with_capacity(shifts.len());
    for (index, shift) in shifts.iter().enumerate() {
        if shift.flags == 0
            || shift.flags & !0b1111 != 0
            || shift.flags & 0b0011 == 0b0011
            || shift.flags & 0b1100 == 0b1100
        {
            return Err(NativeTerrainError::new(format!(
                "terrain vertexShifts[{index}] has invalid directional flags {:#06b}",
                shift.flags
            )));
        }
        if shift.column >= descriptor.dimensions.width || shift.row >= descriptor.dimensions.height
        {
            return Err(NativeTerrainError::new(format!(
                "terrain vertexShifts[{index}] is outside the heightmap"
            )));
        }
        if !seen.insert((shift.column, shift.row)) {
            return Err(NativeTerrainError::new(format!(
                "terrain vertexShifts[{index}] duplicates a heightmap coordinate"
            )));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn finish_grass_mesh_chunk(
    chunks: &mut Vec<PreparedNativeTerrainGrassChunk>,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    instance_count: usize,
    detail_distance: f32,
) {
    if instance_count == 0 {
        return;
    }
    debug_assert!(positions.len() <= NATIVE_TERRAIN_GRASS_VERTICES_PER_FRAME);
    let (chunk_minimum, chunk_maximum) = positions.iter().fold(
        (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
        |(minimum, maximum), position| {
            let position = Vec3::from_array(*position);
            (minimum.min(position), maximum.max(position))
        },
    );
    // VisibilityRange measures from this same position-derived AABB center.
    // Include its full 3D radius so terrain height variation cannot hide a
    // blade whose own source point is still within Unity's detail distance.
    let chunk_radius = (chunk_maximum - chunk_minimum).length() * 0.5;
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, std::mem::take(positions))
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, std::mem::take(normals))
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, std::mem::take(uvs))
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, std::mem::take(colors))
    .with_inserted_indices(Indices::U32(std::mem::take(indices)));
    chunks.push(PreparedNativeTerrainGrassChunk {
        mesh,
        instance_count,
        visibility_end: detail_distance + chunk_radius,
    });
}
