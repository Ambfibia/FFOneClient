use super::*;

pub fn validate(model: &NativeModel) -> Result<()> {
    if model.schema != crate::MODEL_SCHEMA {
        return invalid(format!("schema must be {}", crate::MODEL_SCHEMA));
    }
    validate_logical_name(&model.name)?;
    if model.native_coordinate_contract != crate::exact_native_coordinate_contract() {
        return invalid(
            "native coordinate contract is missing/contradictory or permits recenter/rescale",
        );
    }
    validate_nodes(model)?;
    validate_skins(model)?;
    validate_materials_and_textures(model)?;
    validate_meshes(model)?;
    validate_animations(model)?;
    Ok(())
}

pub(super) fn validate_nodes(model: &NativeModel) -> Result<()> {
    if model.nodes.is_empty() {
        return invalid("model must contain nodes");
    }
    for (index, node) in model.nodes.iter().enumerate() {
        validate_true_asset_name("node", &node.name)?;
        match (&node.legacy_name, node.legacy_sibling_ordinal) {
            (None, None) => {}
            (Some(legacy_name), Some(ordinal)) if ordinal > 0 => {
                validate_true_asset_name("legacy node", legacy_name)?;
                if node.name != format!("{legacy_name}@#{ordinal}") {
                    return invalid(format!(
                        "node {index} duplicate-sibling name does not match exact legacy provenance"
                    ));
                }
            }
            _ => {
                return invalid(format!(
                    "node {index} has incomplete duplicate-sibling provenance"
                ));
            }
        }
        if node
            .parent
            .is_some_and(|value| value as usize >= model.nodes.len())
        {
            return invalid(format!("node {index} parent is out of bounds"));
        }
        if node
            .mesh
            .is_some_and(|value| value as usize >= model.meshes.len())
        {
            return invalid(format!("node {index} mesh is out of bounds"));
        }
        if node
            .skin
            .is_some_and(|value| value as usize >= model.skins.len())
        {
            return invalid(format!("node {index} skin is out of bounds"));
        }
        if node.skin.is_some() && node.mesh.is_none() {
            return invalid(format!("node {index} has a skin but no mesh"));
        }
        finite(
            "node TRS",
            node.translation
                .iter()
                .chain(&node.rotation)
                .chain(&node.scale),
        )?;
        normalized_quat(&node.rotation, "node rotation")?;
    }
    for index in 0..model.nodes.len() {
        let mut seen = BTreeSet::new();
        let mut cursor = Some(index as u32);
        while let Some(current) = cursor {
            if !seen.insert(current) {
                return invalid(format!("node hierarchy contains a cycle at {current}"));
            }
            cursor = model.nodes[current as usize].parent;
        }
    }
    let expected = model
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| node.parent.is_none().then_some(index as u32))
        .collect::<BTreeSet<_>>();
    let actual = model.roots.iter().copied().collect::<BTreeSet<_>>();
    if actual != expected || actual.len() != model.roots.len() || model.roots.len() != 1 {
        return invalid("model must have exactly one root listing the only parentless node");
    }
    let root = model.roots[0] as usize;
    if model.nodes[root].name != model.name {
        return invalid("root node name must equal the exact logical model name");
    }
    Ok(())
}

pub(super) fn validate_skins(model: &NativeModel) -> Result<()> {
    for (skin_index, skin) in model.skins.iter().enumerate() {
        validate_true_asset_name("skin", &skin.name)?;
        if skin.skeleton_root as usize >= model.nodes.len() || skin.joints.is_empty() {
            return invalid(format!(
                "skin {skin_index} has an invalid skeleton root or no joints"
            ));
        }
        // Unity 2.x renderer m_BindPose may be empty. The authoritative Mesh
        // m_BindPose must still match renderer m_Bones exactly at this boundary.
        if skin.inverse_bind_matrices.len() != skin.joints.len() {
            return invalid(format!(
                "skin {skin_index} inverse-bind count differs from joints"
            ));
        }
        let mut joints = BTreeSet::new();
        for joint in &skin.joints {
            if *joint as usize >= model.nodes.len() || !joints.insert(*joint) {
                return invalid(format!(
                    "skin {skin_index} has invalid or duplicate joint {joint}"
                ));
            }
        }
        for matrix in &skin.inverse_bind_matrices {
            finite("inverse bind matrix", matrix.iter().flatten())?;
        }
    }
    Ok(())
}

pub(super) fn validate_meshes(model: &NativeModel) -> Result<()> {
    let mut used_materials = BTreeSet::new();
    let mut renderer_orders = BTreeSet::new();
    for (mesh_index, mesh) in model.meshes.iter().enumerate() {
        validate_true_asset_name("mesh", &mesh.name)?;
        if !renderer_orders.insert(mesh.renderer_order) {
            return invalid(format!(
                "mesh {mesh_index} duplicates legacy renderer order {}",
                mesh.renderer_order
            ));
        }
        if mesh.primitives.is_empty() {
            return invalid(format!("mesh {mesh_index} has no primitives"));
        }
        let palettes = model
            .nodes
            .iter()
            .filter(|node| node.mesh == Some(mesh_index as u32))
            .filter_map(|node| {
                node.skin
                    .map(|skin| model.skins[skin as usize].joints.len())
            })
            .collect::<Vec<_>>();
        for (primitive_index, primitive) in mesh.primitives.iter().enumerate() {
            let vertex_count = primitive.positions.len();
            let context = format!("mesh {mesh_index} primitive {primitive_index}");
            match primitive.material {
                Some(material) => {
                    let Some(source) = model.materials.get(material as usize) else {
                        return invalid(format!("{context} material index is out of bounds"));
                    };
                    if primitive.material_slot.as_deref() != Some(source.name.as_str()) {
                        return invalid(format!(
                            "{context} material slot does not equal the exact material m_Name"
                        ));
                    }
                    used_materials.insert(material);
                }
                None => {
                    if let Some(slot) = &primitive.material_slot {
                        validate_true_asset_name("material slot", slot)?;
                        if !model.materials.is_empty() {
                            return invalid(format!(
                                "{context} has a named material slot but no material index"
                            ));
                        }
                    }
                }
            }
            if vertex_count == 0 || primitive.indices.is_empty() || primitive.indices.len() % 3 != 0
            {
                return invalid(format!("{context} has invalid triangle geometry"));
            }
            if (!primitive.normals.is_empty() && primitive.normals.len() != vertex_count)
                || (!primitive.uvs.is_empty() && primitive.uvs.len() != vertex_count)
            {
                return invalid(format!("{context} attribute count mismatch"));
            }
            finite("positions", primitive.positions.iter().flatten())?;
            finite("normals", primitive.normals.iter().flatten())?;
            finite("UVs", primitive.uvs.iter().flatten())?;
            if primitive
                .indices
                .iter()
                .any(|index| *index as usize >= vertex_count)
            {
                return invalid(format!("{context} index out of bounds"));
            }
            validate_primitive_normal_orientation(
                &context,
                &primitive.positions,
                &primitive.normals,
                &primitive.indices,
            )?;
            if primitive.joints.is_empty() != primitive.weights.is_empty()
                || (!primitive.joints.is_empty()
                    && (primitive.joints.len() != vertex_count
                        || primitive.weights.len() != vertex_count))
            {
                return invalid(format!("{context} skin attribute count mismatch"));
            }
            if !primitive.joints.is_empty() {
                if palettes.is_empty() {
                    return invalid(format!(
                        "{context} has weighted vertices but no owning skin node"
                    ));
                }
                for (vertex, (joints, weights)) in
                    primitive.joints.iter().zip(&primitive.weights).enumerate()
                {
                    finite("weights", weights)?;
                    if weights.iter().any(|value| *value < 0.0)
                        || (weights.iter().sum::<f64>() - 1.0).abs() > 1.0e-4
                    {
                        return invalid(format!(
                            "{context} vertex {vertex} weights are not normalized"
                        ));
                    }
                    if palettes
                        .iter()
                        .any(|palette| joints.iter().any(|joint| *joint as usize >= *palette))
                    {
                        return invalid(format!(
                            "{context} vertex {vertex} joint index exceeds palette"
                        ));
                    }
                }
            }
        }
    }
    if renderer_orders
        .iter()
        .copied()
        .ne((0..model.meshes.len()).filter_map(|index| u16::try_from(index).ok()))
    {
        return invalid("mesh renderer orders must be exact contiguous values starting at zero");
    }
    if used_materials.len() != model.materials.len() {
        return invalid("material table contains unreferenced extra materials");
    }

    Ok(())
}

pub(crate) fn validate_primitive_normal_orientation(
    context: &str,
    positions: &[[f64; 3]],
    normals: &[[f64; 3]],
    indices: &[u32],
) -> Result<()> {
    if normals.is_empty() {
        return Ok(());
    }
    for (index, normal) in normals.iter().enumerate() {
        let length_squared = normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2];
        if !length_squared.is_finite() || length_squared <= 1.0e-12 {
            return invalid(format!("{context} normal {index} is zero or non-finite"));
        }
    }

    let mut opposed = 0_u64;
    for triangle in indices.chunks_exact(3) {
        if triangle_winding_opposes_normals(positions, normals, triangle)? == Some(true) {
            opposed += 1;
        }
    }
    if opposed > 0 {
        return invalid(format!(
            "{context} has {opposed} triangle(s) whose winding opposes authored normals"
        ));
    }
    Ok(())
}
