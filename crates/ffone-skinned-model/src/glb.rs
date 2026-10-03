use serde_json::{Map, Value, json};

use crate::{
    AnimationChannel, Interpolation, MaterialAlphaReference, MaterialAlphaTestState,
    MaterialCullMode, ModelError, NativeMaterial, NativeModel, NativeSampler, Result,
    SamplerMagFilter, SamplerMinFilter, SamplerWrapMode, TrackValues, validate,
};

const FLOAT: u32 = 5_126;
const UNSIGNED_SHORT: u32 = 5_123;
const UNSIGNED_INT: u32 = 5_125;
const ARRAY_BUFFER: u32 = 34_962;
const ELEMENT_ARRAY_BUFFER: u32 = 34_963;

/// Encodes a complete logical model into deterministic GLB 2.0 geometry with
/// semantic external PNG references.
pub fn encode_glb(model: &NativeModel) -> Result<Vec<u8>> {
    validate(model)?;
    let mut builder = BufferBuilder::default();

    let meshes = model
        .meshes
        .iter()
        .map(|mesh| {
            let primitives = mesh
                .primitives
                .iter()
                .map(|primitive| {
                    let positions = vec3_f32(&primitive.positions);
                    let (min, max) = position_bounds(&positions);
                    let position = builder.append_f32(
                        flatten3(&positions),
                        "VEC3",
                        Some(ARRAY_BUFFER),
                        Some((json!(min), json!(max))),
                    )?;
                    let mut attributes = Map::new();
                    attributes.insert("POSITION".into(), json!(position));
                    if !primitive.normals.is_empty() {
                        let normals = vec3_f32(&primitive.normals);
                        let accessor = builder.append_f32(
                            flatten3(&normals),
                            "VEC3",
                            Some(ARRAY_BUFFER),
                            None,
                        )?;
                        attributes.insert("NORMAL".into(), json!(accessor));
                    }
                    if !primitive.uvs.is_empty() {
                        let values = primitive
                            .uvs
                            .iter()
                            .flat_map(|value| [value[0] as f32, value[1] as f32])
                            .collect();
                        let accessor =
                            builder.append_f32(values, "VEC2", Some(ARRAY_BUFFER), None)?;
                        attributes.insert("TEXCOORD_0".into(), json!(accessor));
                    }
                    if !primitive.joints.is_empty() {
                        let joints = builder.append_joints(&primitive.joints)?;
                        let weights = builder.append_f32(
                            primitive
                                .weights
                                .iter()
                                .flat_map(|value| value.iter().map(|value| *value as f32))
                                .collect(),
                            "VEC4",
                            Some(ARRAY_BUFFER),
                            None,
                        )?;
                        attributes.insert("JOINTS_0".into(), json!(joints));
                        attributes.insert("WEIGHTS_0".into(), json!(weights));
                    }
                    let indices = builder.append_indices(&primitive.indices)?;
                    let mut encoded = json!({
                        "attributes": attributes,
                        "indices": indices,
                        "mode": 4,
                        "extras": {"materialSlot": primitive.material_slot},
                    });
                    if let Some(material) = primitive.material {
                        encoded
                            .as_object_mut()
                            .expect("primitive object")
                            .insert("material".into(), json!(material));
                    }
                    Ok(encoded)
                })
                .collect::<Result<Vec<_>>>()?;
            // Bevy's transparent phase does not preserve glTF node/primitive
            // insertion order. Publish the independently recovered legacy
            // renderer position rather than equating it with Mesh object order.
            Ok(json!({
                "name": mesh.name,
                "primitives": primitives,
                "extras": {
                    "ffone": {
                        "rendererIndex": mesh.renderer_order,
                    }
                }
            }))
        })
        .collect::<Result<Vec<_>>>()?;

    let mut children = vec![Vec::<u32>::new(); model.nodes.len()];
    for (index, node) in model.nodes.iter().enumerate() {
        if let Some(parent) = node.parent {
            children[parent as usize].push(index as u32);
        }
    }
    let nodes = model
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let mut value = json!({
                "name": node.name,
                "translation": to_vec3(node.translation),
                "rotation": to_vec4(node.rotation),
                "scale": to_vec3(node.scale),
            });
            let object = value.as_object_mut().expect("node JSON object");
            if !children[index].is_empty() {
                object.insert("children".into(), json!(children[index]));
            }
            if let Some(mesh) = node.mesh {
                object.insert("mesh".into(), json!(mesh));
            }
            if let Some(skin) = node.skin {
                object.insert("skin".into(), json!(skin));
            }
            if let (Some(legacy_name), Some(legacy_sibling_ordinal)) =
                (&node.legacy_name, node.legacy_sibling_ordinal)
            {
                object.insert(
                    "extras".into(),
                    json!({
                        "ffone": {
                            "legacyName": legacy_name,
                            "legacySiblingOrdinal": legacy_sibling_ordinal,
                        }
                    }),
                );
            }
            value
        })
        .collect::<Vec<_>>();

    let skins = model
        .skins
        .iter()
        .map(|skin| {
            // glTF MAT4 storage is column-major. Unity extraction must already
            // supply the converted inverse-bind matrices in native model space.
            let matrices = skin
                .inverse_bind_matrices
                .iter()
                .flat_map(|matrix| {
                    (0..4).flat_map(move |column| (0..4).map(move |row| matrix[row][column] as f32))
                })
                .collect();
            let accessor = builder.append_f32(matrices, "MAT4", None, None)?;
            Ok(json!({
                "name": skin.name,
                "skeleton": skin.skeleton_root,
                "joints": skin.joints,
                "inverseBindMatrices": accessor,
            }))
        })
        .collect::<Result<Vec<_>>>()?;

    let animations = model
        .animations
        .iter()
        .filter(|clip| !clip.channels.is_empty())
        .map(|clip| {
            let mut samplers = Vec::with_capacity(clip.channels.len());
            let mut channels = Vec::with_capacity(clip.channels.len());
            for channel in &clip.channels {
                let times = channel
                    .times
                    .iter()
                    .map(|value| *value as f32)
                    .collect::<Vec<_>>();
                let input = builder.append_f32(
                    times.clone(),
                    "SCALAR",
                    None,
                    Some((json!([times[0]]), json!([times[times.len() - 1]]))),
                )?;
                let (path, output_type, output_values) = encode_track(channel)?;
                let output = builder.append_f32(output_values, output_type, None, None)?;
                let sampler = samplers.len();
                let interpolation = match channel.interpolation {
                    Interpolation::Linear => "LINEAR",
                    Interpolation::Step => "STEP",
                    Interpolation::CubicSpline => "CUBICSPLINE",
                };
                samplers.push(json!({
                    "input": input,
                    "output": output,
                    "interpolation": interpolation,
                }));
                channels.push(json!({
                    "sampler": sampler,
                    "target": {"node": channel.target_node, "path": path},
                    "extras": {
                        "legacyTangentModes": channel.tangent_modes,
                        "sourceIndex": channel.source_index,
                        "sourceEncoding": channel.source_encoding,
                        "sourceKeyCount": channel.source_key_count,
                        "sourceKeyIndices": channel.source_key_indices,
                        "duplicateKeys": channel.duplicate_keys,
                    },
                }));
            }
            Ok(json!({
                "name": clip.name,
                "samplers": samplers,
                "channels": channels,
                "extras": {
                    "duration": clip.duration,
                    "declaredDuration": clip.declared_duration,
                    "keyedDuration": clip.keyed_duration,
                    "eventDuration": clip.event_duration,
                    "sampleRate": clip.sample_rate,
                    "wrapMode": clip.wrap_mode,
                    "loop": clip.looped,
                    "nonTrs": clip.metadata,
                },
            }))
        })
        .collect::<Result<Vec<_>>>()?;
    let metadata_only_animations = model
        .animations
        .iter()
        .filter(|clip| clip.channels.is_empty())
        .map(|clip| {
            json!({
                "name": clip.name,
                "duration": clip.duration,
                "declaredDuration": clip.declared_duration,
                "keyedDuration": clip.keyed_duration,
                "eventDuration": clip.event_duration,
                "sampleRate": clip.sample_rate,
                "wrapMode": clip.wrap_mode,
                "loop": clip.looped,
                "metadata": clip.metadata,
            })
        })
        .collect::<Vec<_>>();

    let samplers = model
        .samplers
        .iter()
        .map(encode_sampler)
        .collect::<Vec<_>>();
    let images = model
        .textures
        .iter()
        .map(|texture| {
            json!({
                "name": texture.source_name,
                "uri": texture.uri,
                "mimeType": "image/png",
                "extras": {"ffone": texture},
            })
        })
        .collect::<Vec<_>>();
    let textures = model
        .textures
        .iter()
        .enumerate()
        .map(|(index, texture)| {
            json!({
                "name": texture.source_name,
                "sampler": texture.sampler,
                "source": index,
                "extras": {"ffone": texture},
            })
        })
        .collect::<Vec<_>>();
    let mut uses_texture_transform = false;
    let materials = model
        .materials
        .iter()
        .map(|material| encode_material(material, &mut uses_texture_transform))
        .collect::<Vec<_>>();

    align(&mut builder.binary, 0);
    let mut root = json!({
        "asset": {"version": "2.0", "generator": "ffone-skinned-model"},
        "scene": 0,
        "scenes": [{"name": model.name, "nodes": model.roots}],
        "nodes": nodes,
        "meshes": meshes,
        "skins": skins,
        "buffers": [{"byteLength": builder.binary.len()}],
        "bufferViews": builder.views,
        "accessors": builder.accessors,
        "extras": {
            "schema": crate::MODEL_SCHEMA,
            "logicalModelName": model.name,
            "nativeCoordinateContract": model.native_coordinate_contract,
            "ffone": {
                "metadataOnlyAnimations": metadata_only_animations,
            },
        },
    });
    if !animations.is_empty() {
        root.as_object_mut()
            .expect("GLB root object")
            .insert("animations".into(), json!(animations));
    }
    let root_object = root.as_object_mut().expect("GLB root object");
    if !materials.is_empty() {
        root_object.insert("materials".into(), json!(materials));
    }
    if !images.is_empty() {
        root_object.insert("images".into(), json!(images));
        root_object.insert("textures".into(), json!(textures));
        root_object.insert("samplers".into(), json!(samplers));
    }
    if uses_texture_transform {
        root_object.insert("extensionsUsed".into(), json!(["KHR_texture_transform"]));
    }
    finish(root, builder.binary)
}

fn encode_material(material: &NativeMaterial, uses_texture_transform: &mut bool) -> Value {
    let base_color = material
        .colors
        .iter()
        .find(|property| property.name == "_Color")
        .map(|property| property.value.map(|value| value as f32))
        .unwrap_or([1.0; 4]);
    let mut pbr = json!({
        "baseColorFactor": base_color,
        "metallicFactor": 0.0,
        "roughnessFactor": 1.0,
    });
    if let Some(binding) = material.texture_bindings.iter().find(|binding| {
        !binding.ignored_stale_shader_binding
            && binding.slot == "_MainTex"
            && binding.texture.is_some()
    }) {
        // Unity 4 omitted pivot/rotation from ordinary m_TexEnvs. Keep the
        // exact Option values in extras, while standard glTF consumption uses
        // the legacy effective defaults only at this boundary.
        let effective_rotation = binding.rotation.unwrap_or(0.0);
        let mut texture_info = json!({
            "index": binding.texture.expect("filtered assigned texture"),
            "texCoord": 0,
            "extras": {
                "ffone": {
                    "slot": binding.slot,
                    "colorSpace": binding.color_space,
                    "pivot": binding.pivot,
                },
            },
        });
        if binding.scale != [1.0, 1.0] || binding.offset != [0.0, 0.0] || effective_rotation != 0.0
        {
            *uses_texture_transform = true;
            texture_info
                .as_object_mut()
                .expect("texture info object")
                .insert(
                    "extensions".into(),
                    json!({
                        "KHR_texture_transform": {
                            "offset": binding.offset.map(|value| value as f32),
                            "rotation": effective_rotation as f32,
                            "scale": binding.scale.map(|value| value as f32),
                        },
                    }),
                );
        }
        pbr.as_object_mut()
            .expect("PBR object")
            .insert("baseColorTexture".into(), texture_info);
    }

    let first_pass = material
        .passes
        .first()
        .expect("material was validated before encoding");
    let (alpha_mode, alpha_cutoff) = match &first_pass.alpha_test {
        MaterialAlphaTestState::Enabled { reference, .. } => {
            let cutoff = match reference {
                MaterialAlphaReference::Literal { value } => *value,
                MaterialAlphaReference::FloatProperty { resolved_value, .. } => *resolved_value,
            };
            ("MASK", Some(cutoff as f32))
        }
        MaterialAlphaTestState::Disabled
            if first_pass.blend.enabled || material.render_queue >= 3_000 =>
        {
            ("BLEND", None)
        }
        MaterialAlphaTestState::Disabled => ("OPAQUE", None),
    };
    let mut encoded = json!({
        "name": material.name,
        "pbrMetallicRoughness": pbr,
        "alphaMode": alpha_mode,
        "doubleSided": first_pass.cull == MaterialCullMode::Off,
        "extras": {"ffone": material},
    });
    let encoded_object = encoded.as_object_mut().expect("material object");
    if let Some(binding) = material.texture_bindings.iter().find(|binding| {
        !binding.ignored_stale_shader_binding
            && binding.slot == "_BumpMap"
            && binding.texture.is_some()
    }) {
        encoded_object.insert("normalTexture".into(), encode_loader_hint(binding));
    }
    if let Some(binding) = material.texture_bindings.iter().find(|binding| {
        !binding.ignored_stale_shader_binding
            && binding.slot == "_ShaderMap"
            && binding.texture.is_some()
    }) {
        encoded_object.insert("occlusionTexture".into(), encode_loader_hint(binding));
    }
    if let Some(cutoff) = alpha_cutoff {
        encoded_object.insert("alphaCutoff".into(), json!(cutoff));
    }
    encoded
}

fn encode_loader_hint(binding: &crate::MaterialTextureBinding) -> Value {
    json!({
        "index": binding.texture.expect("filtered assigned loader-hint texture"),
        "texCoord": 0,
        "extras": {
            "ffone": {
                "slot": binding.slot,
                "colorSpace": binding.color_space,
                "standardTextureRefIsLoaderHint": true,
            },
        },
    })
}

fn encode_sampler(sampler: &NativeSampler) -> Value {
    json!({
        "name": sampler.name,
        "magFilter": match sampler.mag_filter {
            SamplerMagFilter::Nearest => 9_728,
            SamplerMagFilter::Linear => 9_729,
        },
        "minFilter": match sampler.min_filter {
            SamplerMinFilter::Nearest => 9_728,
            SamplerMinFilter::Linear => 9_729,
            SamplerMinFilter::NearestMipmapNearest => 9_984,
            SamplerMinFilter::LinearMipmapNearest => 9_985,
            SamplerMinFilter::NearestMipmapLinear => 9_986,
            SamplerMinFilter::LinearMipmapLinear => 9_987,
        },
        "wrapS": match sampler.wrap_s {
            SamplerWrapMode::ClampToEdge => 33_071,
            SamplerWrapMode::MirroredRepeat => 33_648,
            SamplerWrapMode::Repeat => 10_497,
        },
        "wrapT": match sampler.wrap_t {
            SamplerWrapMode::ClampToEdge => 33_071,
            SamplerWrapMode::MirroredRepeat => 33_648,
            SamplerWrapMode::Repeat => 10_497,
        },
        "extras": {"ffone": sampler},
    })
}

fn encode_track(channel: &AnimationChannel) -> Result<(&'static str, &'static str, Vec<f32>)> {
    match (&channel.values, channel.interpolation) {
        (TrackValues::Translation(values), Interpolation::Linear | Interpolation::Step) => {
            Ok(("translation", "VEC3", flatten3(&vec3_f32(values))))
        }
        (TrackValues::Rotation(values), Interpolation::Linear | Interpolation::Step) => {
            Ok(("rotation", "VEC4", flatten4(&vec4_f32(values))))
        }
        (TrackValues::Scale(values), Interpolation::Linear | Interpolation::Step) => {
            Ok(("scale", "VEC3", flatten3(&vec3_f32(values))))
        }
        (TrackValues::Translation(values), Interpolation::CubicSpline) => {
            let (Some(TrackValues::Translation(input)), Some(TrackValues::Translation(output))) =
                (&channel.in_tangents, &channel.out_tangents)
            else {
                return Err(ModelError::Invalid(
                    "validated translation tangents changed before GLB encoding".into(),
                ));
            };
            Ok(("translation", "VEC3", interleave3(input, values, output)))
        }
        (TrackValues::Rotation(values), Interpolation::CubicSpline) => {
            let (Some(TrackValues::Rotation(input)), Some(TrackValues::Rotation(output))) =
                (&channel.in_tangents, &channel.out_tangents)
            else {
                return Err(ModelError::Invalid(
                    "validated rotation tangents changed before GLB encoding".into(),
                ));
            };
            Ok(("rotation", "VEC4", interleave4(input, values, output)))
        }
        (TrackValues::Scale(values), Interpolation::CubicSpline) => {
            let (Some(TrackValues::Scale(input)), Some(TrackValues::Scale(output))) =
                (&channel.in_tangents, &channel.out_tangents)
            else {
                return Err(ModelError::Invalid(
                    "validated scale tangents changed before GLB encoding".into(),
                ));
            };
            Ok(("scale", "VEC3", interleave3(input, values, output)))
        }
    }
}

fn interleave3(input: &[[f64; 3]], values: &[[f64; 3]], output: &[[f64; 3]]) -> Vec<f32> {
    input
        .iter()
        .zip(values)
        .zip(output)
        .flat_map(|((input, value), output)| {
            input
                .iter()
                .chain(value)
                .chain(output)
                .map(|component| *component as f32)
        })
        .collect()
}

fn interleave4(input: &[[f64; 4]], values: &[[f64; 4]], output: &[[f64; 4]]) -> Vec<f32> {
    input
        .iter()
        .zip(values)
        .zip(output)
        .flat_map(|((input, value), output)| {
            input
                .iter()
                .chain(value)
                .chain(output)
                .map(|component| *component as f32)
        })
        .collect()
}

#[derive(Default)]
struct BufferBuilder {
    binary: Vec<u8>,
    views: Vec<Value>,
    accessors: Vec<Value>,
}

impl BufferBuilder {
    fn append_f32(
        &mut self,
        values: Vec<f32>,
        kind: &'static str,
        target: Option<u32>,
        bounds: Option<(Value, Value)>,
    ) -> Result<u32> {
        align(&mut self.binary, 0);
        let offset = self.binary.len();
        for value in &values {
            self.binary.extend_from_slice(&value.to_le_bytes());
        }
        let components = match kind {
            "SCALAR" => 1,
            "VEC2" => 2,
            "VEC3" => 3,
            "VEC4" => 4,
            "MAT4" => 16,
            _ => unreachable!(),
        };
        let view_index = self.views.len() as u32;
        let mut view = json!({
            "buffer": 0,
            "byteOffset": offset,
            "byteLength": self.binary.len() - offset,
        });
        if let Some(target) = target {
            view.as_object_mut()
                .expect("buffer view object")
                .insert("target".into(), json!(target));
        }
        self.views.push(view);
        let mut accessor = json!({
            "bufferView": view_index,
            "componentType": FLOAT,
            "count": values.len() / components,
            "type": kind,
        });
        if let Some((min, max)) = bounds {
            let object = accessor.as_object_mut().expect("accessor object");
            object.insert("min".into(), min);
            object.insert("max".into(), max);
        }
        let index = self.accessors.len() as u32;
        self.accessors.push(accessor);
        Ok(index)
    }

    fn append_joints(&mut self, values: &[[u16; 4]]) -> Result<u32> {
        align(&mut self.binary, 0);
        let offset = self.binary.len();
        for value in values {
            for component in value {
                self.binary.extend_from_slice(&component.to_le_bytes());
            }
        }
        let view = self.push_view(offset, Some(ARRAY_BUFFER));
        let index = self.accessors.len() as u32;
        self.accessors.push(json!({
            "bufferView": view,
            "componentType": UNSIGNED_SHORT,
            "count": values.len(),
            "type": "VEC4",
        }));
        Ok(index)
    }

    fn append_indices(&mut self, values: &[u32]) -> Result<u32> {
        align(&mut self.binary, 0);
        let offset = self.binary.len();
        let max = values.iter().copied().max().unwrap_or(0);
        let component_type = if u16::try_from(max).is_ok() {
            for value in values {
                self.binary
                    .extend_from_slice(&(*value as u16).to_le_bytes());
            }
            UNSIGNED_SHORT
        } else {
            for value in values {
                self.binary.extend_from_slice(&value.to_le_bytes());
            }
            UNSIGNED_INT
        };
        let view = self.push_view(offset, Some(ELEMENT_ARRAY_BUFFER));
        let index = self.accessors.len() as u32;
        self.accessors.push(json!({
            "bufferView": view,
            "componentType": component_type,
            "count": values.len(),
            "type": "SCALAR",
            "min": [0],
            "max": [max],
        }));
        Ok(index)
    }

    fn push_view(&mut self, offset: usize, target: Option<u32>) -> u32 {
        let mut value = json!({
            "buffer": 0,
            "byteOffset": offset,
            "byteLength": self.binary.len() - offset,
        });
        if let Some(target) = target {
            value
                .as_object_mut()
                .expect("buffer view object")
                .insert("target".into(), json!(target));
        }
        let index = self.views.len() as u32;
        self.views.push(value);
        index
    }
}

fn finish(root: Value, mut binary: Vec<u8>) -> Result<Vec<u8>> {
    align(&mut binary, 0);
    let mut document = serde_json::to_vec(&root)
        .map_err(|error| ModelError::Invalid(format!("cannot serialize GLB JSON: {error}")))?;
    align(&mut document, b' ');
    let total = 12usize
        .checked_add(8)
        .and_then(|value| value.checked_add(document.len()))
        .and_then(|value| value.checked_add(8))
        .and_then(|value| value.checked_add(binary.len()))
        .ok_or(ModelError::Overflow("total file size"))?;
    let total = u32::try_from(total).map_err(|_| ModelError::Overflow("total file size"))?;
    let mut glb = Vec::with_capacity(total as usize);
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2u32.to_le_bytes());
    glb.extend_from_slice(&total.to_le_bytes());
    glb.extend_from_slice(&(document.len() as u32).to_le_bytes());
    glb.extend_from_slice(&0x4e4f_534au32.to_le_bytes());
    glb.extend_from_slice(&document);
    glb.extend_from_slice(&(binary.len() as u32).to_le_bytes());
    glb.extend_from_slice(&0x004e_4942u32.to_le_bytes());
    glb.extend_from_slice(&binary);
    Ok(glb)
}

fn align(bytes: &mut Vec<u8>, padding: u8) {
    while bytes.len() % 4 != 0 {
        bytes.push(padding);
    }
}

fn to_vec3(value: [f64; 3]) -> [f32; 3] {
    [value[0] as f32, value[1] as f32, value[2] as f32]
}

fn to_vec4(value: [f64; 4]) -> [f32; 4] {
    [
        value[0] as f32,
        value[1] as f32,
        value[2] as f32,
        value[3] as f32,
    ]
}

fn vec3_f32(values: &[[f64; 3]]) -> Vec<[f32; 3]> {
    values.iter().copied().map(to_vec3).collect()
}

fn vec4_f32(values: &[[f64; 4]]) -> Vec<[f32; 4]> {
    values.iter().copied().map(to_vec4).collect()
}

fn flatten3(values: &[[f32; 3]]) -> Vec<f32> {
    values.iter().flatten().copied().collect()
}

fn flatten4(values: &[[f32; 4]]) -> Vec<f32> {
    values.iter().flatten().copied().collect()
}

fn position_bounds(values: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for value in values {
        for component in 0..3 {
            min[component] = min[component].min(value[component]);
            max[component] = max[component].max(value[component]);
        }
    }
    (min, max)
}
