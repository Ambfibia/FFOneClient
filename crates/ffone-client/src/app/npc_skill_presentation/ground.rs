use super::*;
use bevy::mesh::VertexAttributeValues;

/// Clip the actual collision triangles instead of resampling a coarse grid.
/// A grid bridges ridges and can bury even correctly sampled vertices. Keep
/// source creases and interpolate warning UVs in world X/Z on every surface.
pub(super) fn append_surface(
    mesh: &Mesh,
    global: &GlobalTransform,
    center: Vec3,
    scale: f32,
    height_band: Option<(f32, f32)>,
    vertices: &mut Vec<[f32; 3]>,
    uv: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
) {
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return;
    };
    let source: Vec<usize> = match mesh.indices() {
        Some(indices) => indices.iter().collect(),
        None => (0..positions.len()).collect(),
    };
    let half = scale * 0.5;
    let matrix = global.to_matrix();
    for triangle in source.chunks_exact(3) {
        let Some(points) = triangle
            .iter()
            .map(|&i| {
                positions
                    .get(i)
                    .map(|p| matrix.transform_point3(Vec3::from_array(*p)))
            })
            .collect::<Option<Vec<_>>>()
        else {
            continue;
        };
        let min = points.iter().copied().reduce(Vec3::min).unwrap();
        let max = points.iter().copied().reduce(Vec3::max).unwrap();
        if max.x < center.x - half
            || min.x > center.x + half
            || max.z < center.z - half
            || min.z > center.z + half
            || height_band.is_some_and(|(low, high)| max.y < low || min.y > high)
            || (points[1] - points[0]).cross(points[2] - points[0]).y.abs() < 1e-6
        {
            continue;
        }
        let mut polygon = points;
        for (axis, boundary, sign) in [
            (0, center.x - half, 1.0),
            (0, center.x + half, -1.0),
            (2, center.z - half, 1.0),
            (2, center.z + half, -1.0),
        ] {
            polygon = clip(polygon, axis, boundary, sign);
        }
        if let Some((low, high)) = height_band {
            polygon = clip(polygon, 1, low, 1.0);
            polygon = clip(polygon, 1, high, -1.0);
        }
        if polygon.len() < 3 {
            continue;
        }
        let base = vertices.len() as u32;
        for point in &polygon {
            vertices.push([point.x, point.y + 0.05, point.z]);
            uv.push([
                0.5 - (point.x - center.x) / scale,
                0.5 + (point.z - center.z) / scale,
            ]);
        }
        for i in 1..polygon.len() as u32 - 1 {
            indices.extend_from_slice(&[base, base + i, base + i + 1]);
        }
    }
}

fn clip(points: Vec<Vec3>, axis: usize, boundary: f32, sign: f32) -> Vec<Vec3> {
    let mut result = Vec::new();
    let Some(mut previous) = points.last().copied() else {
        return result;
    };
    for current in points {
        let a = (previous[axis] - boundary) * sign;
        let b = (current[axis] - boundary) * sign;
        if (a >= 0.0) != (b >= 0.0) {
            result.push(previous.lerp(current, a / (a - b)));
        }
        if b >= 0.0 {
            result.push(current);
        }
        previous = current;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uphill_ridge_keeps_source_triangles_above_ground_and_uvs_inside_warning() {
        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                [-4.0, 0.0, -4.0],
                [0.0, 6.0, -4.0],
                [4.0, 0.0, -4.0],
                [-4.0, 0.0, 4.0],
                [0.0, 6.0, 4.0],
                [4.0, 0.0, 4.0],
            ],
        )
        .with_inserted_indices(Indices::U32(vec![0, 3, 1, 1, 3, 4, 1, 4, 2, 2, 4, 5]));
        let (mut positions, mut uv, mut indices) = (Vec::new(), Vec::new(), Vec::new());
        append_surface(
            &mesh,
            &GlobalTransform::IDENTITY,
            Vec3::ZERO,
            6.0,
            None,
            &mut positions,
            &mut uv,
            &mut indices,
        );
        assert!(!indices.is_empty());
        assert!(positions.iter().any(|p| p[1] > 6.0));
        for p in positions {
            assert!((p[1] - (6.0 - p[0].abs() * 1.5) - 0.05).abs() < 1e-5);
        }
        assert!(uv.iter().flatten().all(|v| (0.0..=1.0).contains(v)));
    }
}
