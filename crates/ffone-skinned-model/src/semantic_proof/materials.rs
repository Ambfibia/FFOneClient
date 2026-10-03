use super::*;

/// Validates the render-facing GLB contract independently from the in-memory
/// publisher. This gate is also used by runtime model replacement so an
/// externally supplied GLB cannot bypass the winding/material checks.
pub fn validate_glb_render_contract(bytes: &[u8]) -> Result<()> {
    let glb = ParsedGlb::parse(bytes)?;
    let meshes = required_array(&glb.document, "meshes", "GLB meshes")?;
    if meshes.is_empty() {
        return Ok(());
    }
    let materials = required_array(&glb.document, "materials", "GLB materials")?;
    let accessors = Accessors::new(&glb)?;
    for (mesh_index, mesh) in meshes.iter().enumerate() {
        let primitives = required_array_value(mesh.get("primitives"), "mesh primitives")?;
        if primitives.is_empty() {
            return invalid(format!(
                "render contract mesh {mesh_index} has no primitives"
            ));
        }
        for (primitive_index, primitive) in primitives.iter().enumerate() {
            let context = format!("mesh {mesh_index} primitive {primitive_index}");
            if optional_u32(primitive.get("mode"))?.unwrap_or(4) != 4 {
                return invalid(format!("{context} is not a triangle primitive"));
            }
            let attributes = primitive
                .get("attributes")
                .and_then(Value::as_object)
                .ok_or_else(|| ModelError::Invalid(format!("{context} has no attributes")))?;
            let positions = vec3_f64(
                accessors.read_f32(
                    required_u32(attributes.get("POSITION"), "POSITION accessor")?,
                    "VEC3",
                )?,
                &format!("{context} positions"),
            )?;
            let indices = accessors.read_indices(required_u32(
                primitive.get("indices"),
                "primitive index accessor",
            )?)?;
            if positions.is_empty() || indices.is_empty() || indices.len() % 3 != 0 {
                return invalid(format!("{context} has invalid triangle geometry"));
            }
            if indices
                .iter()
                .any(|index| *index as usize >= positions.len())
            {
                return invalid(format!("{context} index is out of bounds"));
            }
            let normals = match attributes.get("NORMAL") {
                Some(accessor) => vec3_f64(
                    accessors.read_f32(value_u32(accessor, "NORMAL accessor")?, "VEC3")?,
                    &format!("{context} normals"),
                )?,
                None => Vec::new(),
            };
            if !normals.is_empty() && normals.len() != positions.len() {
                return invalid(format!("{context} normal count differs from positions"));
            }
            crate::validate::validate_primitive_normal_orientation(
                &context, &positions, &normals, &indices,
            )?;

            let material_index =
                required_u32(primitive.get("material"), "primitive material index")?;
            let material = indexed(materials, material_index, "primitive material")?;
            let material_name = required_string(material.get("name"), "material name")?;
            let slot = required_string(
                primitive.pointer("/extras/materialSlot"),
                "primitive material slot",
            )?;
            if slot != material_name {
                return invalid(format!(
                    "{context} material slot {slot:?} differs from material {material_name:?}"
                ));
            }
        }
    }
    Ok(())
}
