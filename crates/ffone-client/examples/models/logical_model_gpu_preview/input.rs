use super::*;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum ParseOutcome {
    Help,
    Run(PreviewConfig),
}

pub(super) fn collect_rendered_world_bounds(
    query: &Query<
        (
            &Mesh3d,
            Option<&SkinnedMesh>,
            Option<&Aabb>,
            &GlobalTransform,
            Option<&LegacyMaterialApplied>,
            Option<&ExactMipChainApplied>,
            Option<&PendingLegacyModelMaterial>,
            Option<&LegacyMaterialMetadataError>,
            Option<&PreviewNpcTextureOverrideBound>,
        ),
        (With<Mesh3d>, Without<LegacyMaterialPassCompanion>),
    >,
    meshes: &Assets<Mesh>,
    inverse_bindposes: &Assets<SkinnedMeshInverseBindposes>,
    joint_transforms: &Query<&GlobalTransform>,
) -> Result<Option<Bounds3>, String> {
    let mut combined: Option<Bounds3> = None;
    for (mesh_handle, skin, aabb, transform, _, _, _, _, _) in query.iter() {
        if let Some(skin) = skin {
            let Some(mesh) = meshes.get(&mesh_handle.0) else {
                return Ok(None);
            };
            let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                Some(VertexAttributeValues::Float32x3(values)) => values,
                Some(_) => {
                    return Err(
                        "skinned POSITION attribute is not the required Float32x3".to_owned()
                    );
                }
                None => return Err("skinned mesh has no POSITION attribute".to_owned()),
            };
            let joint_indices = match mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX) {
                Some(VertexAttributeValues::Uint16x4(values)) => values,
                Some(_) => {
                    return Err(
                        "skinned JOINTS_0 attribute is not the required Uint16x4".to_owned()
                    );
                }
                None => return Err("skinned mesh has no JOINTS_0 attribute".to_owned()),
            };
            let joint_weights = match mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT) {
                Some(VertexAttributeValues::Float32x4(values)) => values,
                Some(_) => {
                    return Err(
                        "skinned WEIGHTS_0 attribute is not the required Float32x4".to_owned()
                    );
                }
                None => return Err("skinned mesh has no WEIGHTS_0 attribute".to_owned()),
            };
            if positions.len() != joint_indices.len() || positions.len() != joint_weights.len() {
                return Err("skinned POSITION/JOINTS_0/WEIGHTS_0 cardinality differs".to_owned());
            }
            let Some(bindposes) = inverse_bindposes.get(&skin.inverse_bindposes) else {
                return Ok(None);
            };
            if skin.joints.len() != bindposes.len() {
                return Err("skin joint/inverse-bind cardinality differs".to_owned());
            }
            let mut joint_matrices = Vec::with_capacity(skin.joints.len());
            for (&joint, inverse_bindpose) in skin.joints.iter().zip(bindposes.iter()) {
                let Ok(joint_transform) = joint_transforms.get(joint) else {
                    return Ok(None);
                };
                joint_matrices.push(joint_transform.to_matrix() * *inverse_bindpose);
            }
            for ((&position, &indices), &weights) in positions
                .iter()
                .zip(joint_indices.iter())
                .zip(joint_weights.iter())
            {
                let point = skinned_vertex_world(position, indices, weights, &joint_matrices)?;
                include_combined_point(&mut combined, point);
            }
        } else {
            let Some(aabb) = aabb else {
                return Ok(None);
            };
            let Some(mesh_bounds) = transformed_aabb(aabb, transform) else {
                return Err("rigid mesh AABB produced no finite world bounds".to_owned());
            };
            include_combined_point(&mut combined, mesh_bounds.min);
            include_combined_point(&mut combined, mesh_bounds.max);
        }
    }
    Ok(combined)
}
