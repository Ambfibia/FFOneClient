use super::*;

/// Classifies one native-space triangle against its authored vertex normals.
/// Exact Unity dumps can contain mixed serialized winding, so publication uses
/// this decision per triangle instead of applying one global index swap.
pub fn triangle_winding_opposes_normals(
    positions: &[[f64; 3]],
    normals: &[[f64; 3]],
    triangle: &[u32],
) -> Result<Option<bool>> {
    if normals.is_empty() {
        return Ok(None);
    }
    if triangle.len() != 3 {
        return invalid("triangle winding audit requires exactly three indices");
    }
    let [ia, ib, ic] = [
        triangle[0] as usize,
        triangle[1] as usize,
        triangle[2] as usize,
    ];
    let a = *positions
        .get(ia)
        .ok_or_else(|| ModelError::Invalid(format!("triangle index {ia} is out of bounds")))?;
    let b = *positions
        .get(ib)
        .ok_or_else(|| ModelError::Invalid(format!("triangle index {ib} is out of bounds")))?;
    let c = *positions
        .get(ic)
        .ok_or_else(|| ModelError::Invalid(format!("triangle index {ic} is out of bounds")))?;
    let na = *normals
        .get(ia)
        .ok_or_else(|| ModelError::Invalid(format!("normal index {ia} is out of bounds")))?;
    let nb = *normals
        .get(ib)
        .ok_or_else(|| ModelError::Invalid(format!("normal index {ib} is out of bounds")))?;
    let nc = *normals
        .get(ic)
        .ok_or_else(|| ModelError::Invalid(format!("normal index {ic} is out of bounds")))?;

    let edge_ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let edge_ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let face = [
        edge_ab[1] * edge_ac[2] - edge_ab[2] * edge_ac[1],
        edge_ab[2] * edge_ac[0] - edge_ab[0] * edge_ac[2],
        edge_ab[0] * edge_ac[1] - edge_ab[1] * edge_ac[0],
    ];
    let face_length_squared = face[0] * face[0] + face[1] * face[1] + face[2] * face[2];
    if face_length_squared <= 1.0e-20 {
        return Ok(None);
    }
    let normal_sum = [
        na[0] + nb[0] + nc[0],
        na[1] + nb[1] + nc[1],
        na[2] + nb[2] + nc[2],
    ];
    let agreement = face[0] * normal_sum[0] + face[1] * normal_sum[1] + face[2] * normal_sum[2];
    let threshold = face_length_squared.sqrt() * 1.0e-8;
    if agreement > threshold {
        Ok(Some(false))
    } else if agreement < -threshold {
        Ok(Some(true))
    } else {
        Ok(None)
    }
}
