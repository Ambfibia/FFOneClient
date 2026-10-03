use super::*;

/// Recomputes value-level digests solely from a GLB. The tree auditor uses this
/// at rest instead of trusting digests copied into the adjacent publish report.
pub fn semantic_digests_from_glb(glb: &[u8]) -> Result<SemanticDigests> {
    Ok(SemanticSections::from_glb(glb)?.digests())
}

pub(super) fn hierarchy_from_model(model: &NativeModel) -> Result<Vec<u8>> {
    let mut out = Canonical::default();
    out.tag("hierarchy-v2");
    let coordinates = serde_json::to_value(&model.native_coordinate_contract).map_err(|error| {
        ModelError::Invalid(format!(
            "cannot canonicalize native coordinate contract: {error}"
        ))
    })?;
    write_canonical_json(&mut out, &coordinates)?;
    out.string(&model.name);
    out.len(model.roots.len())?;
    for &root in &model.roots {
        out.u32(root);
    }
    out.len(model.nodes.len())?;
    for node in &model.nodes {
        out.string(&node.name);
        out.optional_u32(node.parent);
        for value in node.translation {
            out.f32(value as f32);
        }
        for value in node.rotation {
            out.f32(value as f32);
        }
        for value in node.scale {
            out.f32(value as f32);
        }
        out.optional_u32(node.mesh);
        out.optional_u32(node.skin);
    }
    Ok(out.0)
}

pub(super) fn geometry_from_model(model: &NativeModel) -> Result<Vec<u8>> {
    let mut out = Canonical::default();
    out.tag("geometry-v2");
    out.len(model.meshes.len())?;
    for mesh in &model.meshes {
        out.string(&mesh.name);
        out.u16(mesh.renderer_order);
        out.len(mesh.primitives.len())?;
        for primitive in &mesh.primitives {
            out.optional_u32(primitive.material);
            out.bool(primitive.material_slot.is_some());
            if let Some(slot) = &primitive.material_slot {
                out.string(slot);
            }
            write_f64_vectors_as_f32(&mut out, &primitive.positions)?;
            write_f64_vectors_as_f32(&mut out, &primitive.normals)?;
            write_f64_vectors_as_f32(&mut out, &primitive.uvs)?;
            out.len(primitive.indices.len())?;
            for &value in &primitive.indices {
                out.u32(value);
            }
            out.len(primitive.joints.len())?;
            for value in &primitive.joints {
                for &component in value {
                    out.u16(component);
                }
            }
            write_f64_vectors_as_f32(&mut out, &primitive.weights)?;
        }
    }
    Ok(out.0)
}

pub(super) fn skins_from_model(model: &NativeModel) -> Result<Vec<u8>> {
    let mut out = Canonical::default();
    out.tag("skins-v1");
    out.len(model.skins.len())?;
    for skin in &model.skins {
        out.string(&skin.name);
        out.u32(skin.skeleton_root);
        out.len(skin.joints.len())?;
        for &joint in &skin.joints {
            out.u32(joint);
        }
        out.len(skin.inverse_bind_matrices.len())?;
        for matrix in &skin.inverse_bind_matrices {
            for row in matrix {
                for &value in row {
                    out.f32(value as f32);
                }
            }
        }
    }
    Ok(out.0)
}

pub(super) fn animations_from_model(model: &NativeModel) -> Result<Vec<u8>> {
    let clips = model
        .animations
        .iter()
        .filter(|clip| !clip.channels.is_empty())
        .collect::<Vec<_>>();
    let mut out = Canonical::default();
    out.tag("standard-animations-v1");
    out.len(clips.len())?;
    for clip in clips {
        out.string(&clip.name);
        out.f64(clip.duration);
        out.optional_f64(clip.declared_duration);
        out.optional_f64(clip.keyed_duration);
        out.optional_f64(clip.event_duration);
        out.optional_f64(clip.sample_rate);
        out.optional_i32(clip.wrap_mode);
        out.bool(clip.looped);
        out.len(clip.channels.len())?;
        for channel in &clip.channels {
            out.u32(channel.target_node);
            out.u32(channel.source_index);
            out.u8(source_encoding_tag(channel.source_encoding));
            out.u32(channel.source_key_count);
            out.len(channel.source_key_indices.len())?;
            for &source_key_index in &channel.source_key_indices {
                out.u32(source_key_index);
            }
            write_canonical_json(
                &mut out,
                &serde_json::to_value(&channel.duplicate_keys).map_err(|error| {
                    ModelError::Invalid(format!(
                        "cannot canonicalize source duplicate animation keys: {error}"
                    ))
                })?,
            )?;
            out.u8(interpolation_tag(channel.interpolation));
            out.len(channel.tangent_modes.len())?;
            for &mode in &channel.tangent_modes {
                out.i32(mode);
            }
            out.len(channel.times.len())?;
            for &time in &channel.times {
                out.f32(time as f32);
            }
            write_track(&mut out, &channel.values)?;
            write_optional_track(&mut out, channel.in_tangents.as_ref())?;
            write_optional_track(&mut out, channel.out_tangents.as_ref())?;
        }
    }
    Ok(out.0)
}

pub(super) fn materials_from_model(model: &NativeModel) -> Result<Vec<u8>> {
    let mut out = Canonical::default();
    out.tag("materials-v1");
    out.len(model.materials.len())?;
    for material in &model.materials {
        let exact = serde_json::to_value(material).map_err(|error| {
            ModelError::Invalid(format!("cannot canonicalize native material: {error}"))
        })?;
        write_canonical_json(&mut out, &exact)?;
    }
    Ok(out.0)
}

pub(super) struct ParsedGlb<'a> {
    pub(super) document: Value,
    pub(super) binary: &'a [u8],
}

impl<'a> ParsedGlb<'a> {
    pub(super) fn parse(bytes: &'a [u8]) -> Result<Self> {
        if bytes.len() < 20 || bytes.get(..4) != Some(b"glTF") {
            return invalid("semantic proof cannot parse invalid GLB header");
        }
        if read_u32(bytes, 4)? != 2 || usize_from_u32(read_u32(bytes, 8)?)? != bytes.len() {
            return invalid("semantic proof requires an exact GLB 2.0 byte length");
        }
        let json_len = usize_from_u32(read_u32(bytes, 12)?)?;
        if read_u32(bytes, 16)? != 0x4e4f_534a {
            return invalid("semantic proof GLB has no leading JSON chunk");
        }
        let json_start = 20usize;
        let json_end = checked_add(json_start, json_len, "GLB JSON end")?;
        let bin_header_end = checked_add(json_end, 8, "GLB BIN header")?;
        if bin_header_end > bytes.len() || read_u32(bytes, json_end + 4)? != 0x004e_4942 {
            return invalid("semantic proof GLB has no BIN chunk after JSON");
        }
        let bin_len = usize_from_u32(read_u32(bytes, json_end)?)?;
        let bin_end = checked_add(bin_header_end, bin_len, "GLB BIN end")?;
        if bin_end != bytes.len() {
            return invalid("semantic proof rejects trailing or truncated GLB chunks");
        }
        let document: Value =
            serde_json::from_slice(&bytes[json_start..json_end]).map_err(|error| {
                ModelError::Invalid(format!("semantic proof cannot parse GLB JSON: {error}"))
            })?;
        let binary = &bytes[bin_header_end..bin_end];
        let declared = document
            .pointer("/buffers/0/byteLength")
            .and_then(Value::as_u64)
            .and_then(|value| usize::try_from(value).ok());
        if declared != Some(binary.len()) {
            return invalid("semantic proof GLB buffer byteLength differs from BIN chunk");
        }
        Ok(Self { document, binary })
    }
}

pub(super) fn hierarchy_from_glb(glb: &ParsedGlb<'_>) -> Result<Vec<u8>> {
    let nodes = required_array(&glb.document, "nodes", "GLB nodes")?;
    let scene_index = optional_u32(glb.document.get("scene"))?.unwrap_or(0);
    let scenes = required_array(&glb.document, "scenes", "GLB scenes")?;
    let scene = indexed(scenes, scene_index, "selected GLB scene")?;
    let logical_name = required_string(
        glb.document.pointer("/extras/logicalModelName"),
        "logical model name",
    )?;
    if required_string(scene.get("name"), "scene name")? != logical_name {
        return invalid("semantic proof scene name differs from exact logical model name");
    }
    let roots = required_array_value(scene.get("nodes"), "scene root nodes")?;
    let mut parents = vec![None; nodes.len()];
    for (parent, node) in nodes.iter().enumerate() {
        for child in optional_array(node.get("children"), "node children")? {
            let child = value_u32(child, "child node index")?;
            let child_usize = usize_from_u32(child)?;
            if child_usize >= nodes.len() || parents[child_usize].replace(parent as u32).is_some() {
                return invalid("semantic proof found invalid or multiply-parented GLB node");
            }
        }
    }

    let mut out = Canonical::default();
    out.tag("hierarchy-v2");
    write_canonical_json(
        &mut out,
        glb.document
            .pointer("/extras/nativeCoordinateContract")
            .ok_or_else(|| {
                ModelError::Invalid("semantic proof GLB has no coordinate contract".into())
            })?,
    )?;
    out.string(logical_name);
    out.len(roots.len())?;
    for root in roots {
        out.u32(value_u32(root, "scene root node")?);
    }
    out.len(nodes.len())?;
    for (index, node) in nodes.iter().enumerate() {
        out.string(required_string(node.get("name"), "node name")?);
        out.optional_u32(parents[index]);
        write_json_f32_array::<3>(&mut out, node.get("translation"), "node translation")?;
        write_json_f32_array::<4>(&mut out, node.get("rotation"), "node rotation")?;
        write_json_f32_array::<3>(&mut out, node.get("scale"), "node scale")?;
        out.optional_u32(optional_u32(node.get("mesh"))?);
        out.optional_u32(optional_u32(node.get("skin"))?);
    }
    Ok(out.0)
}

pub(super) fn geometry_from_glb(glb: &ParsedGlb<'_>) -> Result<Vec<u8>> {
    let meshes = required_array(&glb.document, "meshes", "GLB meshes")?;
    let accessors = Accessors::new(glb)?;
    let mut out = Canonical::default();
    out.tag("geometry-v2");
    out.len(meshes.len())?;
    for mesh in meshes {
        out.string(required_string(mesh.get("name"), "mesh name")?);
        let renderer_order = required_u32(
            mesh.pointer("/extras/ffone/rendererIndex"),
            "mesh legacy renderer order",
        )?;
        out.u16(
            u16::try_from(renderer_order).map_err(|_| {
                ModelError::Invalid("mesh legacy renderer order exceeds u16".into())
            })?,
        );
        let primitives = required_array_value(mesh.get("primitives"), "mesh primitives")?;
        out.len(primitives.len())?;
        for primitive in primitives {
            out.optional_u32(optional_u32(primitive.get("material"))?);
            let material_slot = primitive.pointer("/extras/materialSlot");
            out.bool(!matches!(material_slot, None | Some(Value::Null)));
            if !matches!(material_slot, None | Some(Value::Null)) {
                out.string(required_string(material_slot, "primitive material slot")?);
            }
            let attributes = primitive
                .get("attributes")
                .and_then(Value::as_object)
                .ok_or_else(|| {
                    ModelError::Invalid("semantic proof primitive has no attributes".into())
                })?;
            write_accessor_f32(&mut out, &accessors, attributes.get("POSITION"), "VEC3")?;
            write_optional_accessor_f32(&mut out, &accessors, attributes.get("NORMAL"), "VEC3")?;
            write_optional_accessor_f32(
                &mut out,
                &accessors,
                attributes.get("TEXCOORD_0"),
                "VEC2",
            )?;
            write_indices(&mut out, &accessors, primitive.get("indices"))?;
            write_optional_joints(&mut out, &accessors, attributes.get("JOINTS_0"))?;
            write_optional_accessor_f32(&mut out, &accessors, attributes.get("WEIGHTS_0"), "VEC4")?;
            if attributes.contains_key("JOINTS_0") != attributes.contains_key("WEIGHTS_0") {
                return invalid("semantic proof requires JOINTS_0 and WEIGHTS_0 together");
            }
        }
    }
    Ok(out.0)
}

pub(super) fn skins_from_glb(glb: &ParsedGlb<'_>) -> Result<Vec<u8>> {
    let skins = optional_root_array(&glb.document, "skins")?;
    let accessors = Accessors::new(glb)?;
    let mut out = Canonical::default();
    out.tag("skins-v1");
    out.len(skins.len())?;
    for skin in skins {
        out.string(required_string(skin.get("name"), "skin name")?);
        out.u32(required_u32(skin.get("skeleton"), "skin skeleton")?);
        let joints = required_array_value(skin.get("joints"), "skin joints")?;
        out.len(joints.len())?;
        for joint in joints {
            out.u32(value_u32(joint, "skin joint")?);
        }
        let matrices = accessors.read_f32(
            required_u32(skin.get("inverseBindMatrices"), "inverse bind accessor")?,
            "MAT4",
        )?;
        out.len(matrices.len())?;
        for matrix in matrices {
            // Accessor MAT4 values are column-major; canonical matrices retain
            // the NativeModel row/column indexing used before GLB encoding.
            for row in 0..4 {
                for column in 0..4 {
                    out.f32(matrix[column * 4 + row]);
                }
            }
        }
    }
    Ok(out.0)
}

pub(super) fn animations_from_glb(glb: &ParsedGlb<'_>) -> Result<Vec<u8>> {
    let animations = optional_root_array(&glb.document, "animations")?;
    let accessors = Accessors::new(glb)?;
    let mut out = Canonical::default();
    out.tag("standard-animations-v1");
    out.len(animations.len())?;
    for animation in animations {
        out.string(required_string(animation.get("name"), "animation name")?);
        let extras = animation
            .get("extras")
            .ok_or_else(|| ModelError::Invalid("semantic proof animation has no extras".into()))?;
        out.f64(required_f64(extras.get("duration"), "animation duration")?);
        out.optional_f64(optional_f64(extras.get("declaredDuration"))?);
        out.optional_f64(optional_f64(extras.get("keyedDuration"))?);
        out.optional_f64(optional_f64(extras.get("eventDuration"))?);
        out.optional_f64(optional_f64(extras.get("sampleRate"))?);
        out.optional_i32(optional_i32(extras.get("wrapMode"))?);
        out.bool(required_bool(extras.get("loop"), "animation loop")?);
        let channels = required_array_value(animation.get("channels"), "animation channels")?;
        let samplers = required_array_value(animation.get("samplers"), "animation samplers")?;
        if samplers.len() != channels.len() {
            return invalid("semantic proof requires one ordered sampler per animation channel");
        }
        out.len(channels.len())?;
        for (channel_index, channel) in channels.iter().enumerate() {
            if required_u32(channel.get("sampler"), "animation channel sampler")?
                != channel_index as u32
            {
                return invalid("semantic proof animation sampler order is not canonical");
            }
            let target = channel.get("target").ok_or_else(|| {
                ModelError::Invalid("semantic proof channel has no target".into())
            })?;
            out.u32(required_u32(target.get("node"), "animation target node")?);
            out.u32(required_u32(
                channel.pointer("/extras/sourceIndex"),
                "animation source index",
            )?);
            let source_encoding: crate::EmptyTrsSourceEncoding = serde_json::from_value(
                channel
                    .pointer("/extras/sourceEncoding")
                    .cloned()
                    .ok_or_else(|| {
                        ModelError::Invalid("semantic proof channel has no sourceEncoding".into())
                    })?,
            )
            .map_err(|error| {
                ModelError::Invalid(format!(
                    "semantic proof channel sourceEncoding is invalid: {error}"
                ))
            })?;
            out.u8(source_encoding_tag(source_encoding));
            out.u32(required_u32(
                channel.pointer("/extras/sourceKeyCount"),
                "animation source key count",
            )?);
            let source_key_indices = required_array_value(
                channel.pointer("/extras/sourceKeyIndices"),
                "animation source key indices",
            )?;
            out.len(source_key_indices.len())?;
            for source_key_index in source_key_indices {
                out.u32(value_u32(source_key_index, "animation source key index")?);
            }
            write_canonical_json(
                &mut out,
                channel.pointer("/extras/duplicateKeys").ok_or_else(|| {
                    ModelError::Invalid(
                        "semantic proof channel has no typed duplicateKeys metadata".into(),
                    )
                })?,
            )?;
            let sampler = &samplers[channel_index];
            let interpolation =
                match required_string(sampler.get("interpolation"), "animation interpolation")? {
                    "LINEAR" => Interpolation::Linear,
                    "STEP" => Interpolation::Step,
                    "CUBICSPLINE" => Interpolation::CubicSpline,
                    value => return invalid(format!("unsupported GLB interpolation {value:?}")),
                };
            out.u8(interpolation_tag(interpolation));
            let tangent_modes = required_array_value(
                channel.pointer("/extras/legacyTangentModes"),
                "legacy tangent modes",
            )?;
            out.len(tangent_modes.len())?;
            for mode in tangent_modes {
                out.i32(value_i32(mode, "legacy tangent mode")?);
            }
            let times = accessors.read_f32(
                required_u32(sampler.get("input"), "animation input accessor")?,
                "SCALAR",
            )?;
            out.len(times.len())?;
            for time in &times {
                out.f32(time[0]);
            }
            let (kind, gltf_type) =
                match required_string(target.get("path"), "animation target path")? {
                    "translation" => (0, "VEC3"),
                    "rotation" => (1, "VEC4"),
                    "scale" => (2, "VEC3"),
                    value => return invalid(format!("unsupported GLB animation path {value:?}")),
                };
            let output = accessors.read_f32(
                required_u32(sampler.get("output"), "animation output accessor")?,
                gltf_type,
            )?;
            if interpolation == Interpolation::CubicSpline {
                if output.len() != times.len().saturating_mul(3) {
                    return invalid("CUBICSPLINE output does not contain in/value/out triples");
                }
                write_track_rows(
                    &mut out,
                    kind,
                    output.iter().skip(1).step_by(3),
                    times.len(),
                )?;
                out.bool(true);
                write_track_rows(&mut out, kind, output.iter().step_by(3), times.len())?;
                out.bool(true);
                write_track_rows(
                    &mut out,
                    kind,
                    output.iter().skip(2).step_by(3),
                    times.len(),
                )?;
            } else {
                if output.len() != times.len() {
                    return invalid("animation output count differs from input time count");
                }
                write_track_rows(&mut out, kind, output.iter(), output.len())?;
                out.bool(false);
                out.bool(false);
            }
        }
    }
    Ok(out.0)
}

pub(super) fn materials_from_glb(glb: &ParsedGlb<'_>) -> Result<Vec<u8>> {
    let materials = optional_root_array(&glb.document, "materials")?;
    let mut out = Canonical::default();
    out.tag("materials-v1");
    out.len(materials.len())?;
    for material in materials {
        let exact = material.pointer("/extras/ffone").ok_or_else(|| {
            ModelError::Invalid("semantic proof material has no exact ffone extras".into())
        })?;
        write_canonical_json(&mut out, exact)?;
    }
    Ok(out.0)
}
