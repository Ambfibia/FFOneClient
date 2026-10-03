use super::*;

pub(super) fn validate_relative_glb(value: &str) -> Result<(), String> {
    let path = Path::new(value);
    if value.is_empty() || path.is_absolute() {
        return Err("--model must be a non-empty relative GLB path".to_owned());
    }
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err("--model may not escape --asset-root".to_owned());
    }
    let is_glb = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("glb"));
    if !is_glb {
        return Err("--model must point to a native .glb file".to_owned());
    }
    Ok(())
}

#[derive(Resource)]
pub(super) struct ModelHandles {
    pub(super) gltf: Handle<Gltf>,
}

pub(super) fn skinned_vertex_world(
    position: [f32; 3],
    joint_indices: [u16; 4],
    joint_weights: [f32; 4],
    joint_matrices: &[Mat4],
) -> Result<Vec3, String> {
    let local = Vec4::new(position[0], position[1], position[2], 1.0);
    let mut world = Vec4::ZERO;
    let mut positive_weight = false;
    for influence in 0..4 {
        let weight = joint_weights[influence];
        if !weight.is_finite() || weight < 0.0 {
            return Err("skinned vertex has a non-finite or negative weight".to_owned());
        }
        if weight == 0.0 {
            continue;
        }
        positive_weight = true;
        let matrix = joint_matrices
            .get(joint_indices[influence] as usize)
            .ok_or_else(|| "skinned vertex joint index is out of bounds".to_owned())?;
        world += (*matrix * local) * weight;
    }
    if !positive_weight {
        return Err("skinned vertex has no positive joint weight".to_owned());
    }
    let point = world.truncate();
    if !point.is_finite() || !world.w.is_finite() || world.w <= 0.0 {
        return Err("skinned vertex produced a non-finite homogeneous position".to_owned());
    }
    Ok(point)
}
