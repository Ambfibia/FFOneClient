use super::*;

pub(super) fn identity() -> [[f64; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub(super) fn rex() -> NativeModel {
    NativeModel {
        schema: MODEL_SCHEMA.into(),
        name: "Rex".into(),
        native_coordinate_contract: exact_native_coordinate_contract(),
        roots: vec![0],
        nodes: vec![
            ModelNode {
                name: "Rex".into(),
                legacy_name: None,
                legacy_sibling_ordinal: None,
                parent: None,
                translation: [0.0; 3],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0; 3],
                mesh: Some(0),
                skin: Some(0),
            },
            ModelNode {
                name: "Bip01".into(),
                legacy_name: None,
                legacy_sibling_ordinal: None,
                parent: Some(0),
                translation: [0.0; 3],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0; 3],
                mesh: None,
                skin: None,
            },
        ],
        meshes: vec![ModelMesh {
            name: "Body".into(),
            renderer_order: 0,
            primitives: vec![ModelPrimitive {
                material: None,
                material_slot: Some("Body".into()),
                positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                normals: vec![[0.0, 0.0, 1.0]; 3],
                uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
                joints: vec![[0, 0, 0, 0]; 3],
                weights: vec![[1.0, 0.0, 0.0, 0.0]; 3],
                indices: vec![0, 1, 2],
            }],
        }],
        skins: vec![ModelSkin {
            name: "RexRig".into(),
            skeleton_root: 1,
            joints: vec![1],
            inverse_bind_matrices: vec![identity()],
        }],
        materials: Vec::new(),
        textures: Vec::new(),
        samplers: Vec::new(),
        animations: vec![AnimationClip {
            name: "run".into(),
            duration: 1.0,
            declared_duration: Some(1.0),
            keyed_duration: Some(1.0),
            event_duration: Some(0.25),
            sample_rate: Some(30.0),
            wrap_mode: Some(2),
            looped: true,
            channels: vec![AnimationChannel {
                target_node: 1,
                source_index: 0,
                source_encoding: EmptyTrsSourceEncoding::Plain,
                source_key_count: 2,
                source_key_indices: vec![0, 1],
                duplicate_keys: Vec::new(),
                interpolation: Interpolation::Linear,
                times: vec![0.0, 1.0],
                values: TrackValues::Rotation(vec![[0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 0.0, 1.0]]),
                in_tangents: None,
                out_tangents: None,
                tangent_modes: Vec::new(),
            }],
            metadata: AnimationMetadata {
                float_curves: vec![FloatCurve {
                    target_node: 1,
                    target_path: "Bip01".into(),
                    source_index: 0,
                    source_key_count: 2,
                    source_key_indices: vec![0, 1],
                    property: "m_Enabled".into(),
                    class_id: 1,
                    script: AnimationBindingPointer {
                        file_id: 0,
                        path_id: 0,
                    },
                    pre_infinity: 2,
                    post_infinity: 2,
                    interpolation: Interpolation::Linear,
                    times: vec![0.0, 1.0],
                    values: vec![1.0, 0.0],
                    in_tangents: None,
                    out_tangents: None,
                    tangent_modes: vec![0, 0],
                }],
                object_curves: vec![ObjectReferenceCurve {
                    target_node: 1,
                    property: "m_Material".into(),
                    keys: vec![ObjectReferenceKey {
                        time: 0.5,
                        value: Some(NativeAssetReference {
                            kind: "material".into(),
                            name: "Rex Body".into(),
                            uri: Some("materials/Rex Body.material.json".into()),
                        }),
                    }],
                }],
                events: vec![AnimationEvent {
                    time: 0.25,
                    function_name: "Footstep".into(),
                    string_parameter: "left".into(),
                    float_parameter: 0.5,
                    float_parameter_provenance: None,
                    int_parameter: 1,
                    object_parameter: None,
                    object_parameter_provenance:
                        ffone_skinned_model::AnimationEventObjectParameterProvenance::Missing {
                            interpretation: ffone_skinned_model::MissingEventObjectParameterInterpretation::Missing,
                        },
                    message_options: 0,
                }],
                empty_trs_bindings: Vec::new(),
                duplicate_trs_bindings: Vec::new(),
                time_recoveries: Vec::new(),
                curve_recoveries: Vec::new(),
                unsupported: Vec::new(),
            },
        }],
    }
}

pub(super) fn materialized_rex() -> NativeModel {
    let mut model = rex();
    model.meshes[0].primitives[0].material = Some(0);
    model.meshes[0].primitives[0].material_slot = Some("spwaneye".into());
    let sampler = NativeSampler {
        name: "spwaneye".into(),
        mag_filter: SamplerMagFilter::Linear,
        min_filter: SamplerMinFilter::LinearMipmapLinear,
        wrap_s: SamplerWrapMode::Repeat,
        wrap_t: SamplerWrapMode::Repeat,
        legacy_filter_mode: 2,
        legacy_wrap_mode: 0,
        anisotropy_level: 1,
        mip_map_bias: -0.25,
    };
    let mip_provenance = TextureMipProvenance {
        source_texture_format: 10,
        source_texture_format_name: "DXT1".into(),
        source_mip_count: 7,
        source_chain_byte_length: 1_384,
        source_chain_sha256: "f".repeat(64),
        source_chain_complete: true,
        source_layout: TextureSourceMipLayout::LargestToSmallestContiguous,
        published_pixel_transform: PublishedPixelTransform::VerticalFlipOnlyForPngTopLeftOrigin,
        published_policy: PublishedMipPolicy::ExactSourceLevels,
    };
    let mip_layout = [
        (64, 32, 0, 1_024, 8_192),
        (32, 16, 1_024, 256, 2_048),
        (16, 8, 1_280, 64, 512),
        (8, 4, 1_344, 16, 128),
        (4, 2, 1_360, 8, 32),
        (2, 1, 1_368, 8, 8),
        (1, 1, 1_376, 8, 4),
    ];
    let mip_levels = mip_layout
        .into_iter()
        .enumerate()
        .map(
            |(level, (width, height, source_offset, source_length, rgba_length))| {
                NativeTextureMipLevel {
                    level: level as u32,
                    width,
                    height,
                    uri: if level == 0 {
                        "textures/spwaneye.png".into()
                    } else {
                        format!("textures/spwaneye.mips/mip-{level:02}.png")
                    },
                    source_byte_offset: source_offset,
                    source_byte_length: source_length,
                    source_byte_sha256: format!("{:064x}", level + 1),
                    decoded_rgba8_byte_length: rgba_length,
                    decoded_rgba8_sha256: format!("{:064x}", level + 16),
                    png_byte_length: 1,
                    png_sha256: format!("{:064x}", level + 32),
                }
            },
        )
        .collect::<Vec<_>>();
    model.materials.push(NativeMaterial {
        name: "spwaneye".into(),
        serialized_shader_name: "CartoonNetwork/Character/Toon".into(),
        declared_shader_name: "CartoonNetwork/Character/Toon".into(),
        legacy_shader_name: "CartoonNetwork/Character/Toon".into(),
        render_queue: 3_000,
        colors: vec![
            MaterialColorProperty {
                name: "_Emission".into(),
                value: [0.0, 0.1, 0.2, 1.0],
            },
            MaterialColorProperty {
                name: "_Color".into(),
                value: [0.25, 0.5, 0.75, 0.8],
            },
        ],
        floats: vec![
            MaterialFloatProperty {
                name: "_Outline".into(),
                value: 0.0125,
            },
            MaterialFloatProperty {
                name: "_Glossiness".into(),
                value: 0.4,
            },
            MaterialFloatProperty {
                name: "_Cutoff".into(),
                value: 0.9,
            },
        ],
        shader_texture_defaults: vec![
            ShaderLabTextureDefaultProperty {
                slot: "_Detail".into(),
                value: ShaderLabTextureDefault::Blank,
            },
            ShaderLabTextureDefaultProperty {
                slot: "_MainTex".into(),
                value: ShaderLabTextureDefault::BuiltinWhite,
            },
        ],
        texture_bindings: vec![
            MaterialTextureBinding {
                slot: "_Detail".into(),
                unassigned_stale_null: false,
                ignored_stale_shader_binding: false,
                dynamic_texture: None,
                texture: None,
                source_name: None,
                uri: None,
                sampler: None,
                mip_provenance: None,
                mip_levels: None,
                scale: [1.0, 1.0],
                offset: [0.0, 0.0],
                pivot: Some([0.0, 0.0]),
                rotation: Some(0.0),
                color_space: TextureColorSpace::Linear,
            },
            MaterialTextureBinding {
                slot: "_MainTex".into(),
                unassigned_stale_null: false,
                ignored_stale_shader_binding: false,
                dynamic_texture: None,
                texture: Some(0),
                source_name: Some("spwaneye.dds".into()),
                uri: Some("textures/spwaneye.png".into()),
                sampler: Some(MaterialTextureSamplerBinding {
                    index: 0,
                    descriptor: sampler.clone(),
                }),
                mip_provenance: Some(mip_provenance.clone()),
                mip_levels: Some(mip_levels.clone()),
                scale: [0.5, 0.75],
                offset: [0.25, 0.125],
                pivot: Some([0.5, 0.5]),
                rotation: Some(0.2),
                color_space: TextureColorSpace::Srgb,
            },
        ],
        passes: vec![MaterialPass {
            name: Some("FORWARD".into()),
            blend: MaterialBlendState {
                enabled: true,
                source_color: MaterialBlendFactor::SourceAlpha,
                destination_color: MaterialBlendFactor::OneMinusSourceAlpha,
                color_operation: MaterialBlendOperation::Add,
                source_alpha: MaterialBlendFactor::One,
                destination_alpha: MaterialBlendFactor::OneMinusSourceAlpha,
                alpha_operation: MaterialBlendOperation::Add,
            },
            cull: MaterialCullMode::Off,
            z_write: false,
            z_test: MaterialCompareFunction::LessEqual,
            alpha_test: MaterialAlphaTestState::Enabled {
                compare: MaterialCompareFunction::Greater,
                reference: MaterialAlphaReference::FloatProperty {
                    name: "_Cutoff".into(),
                    resolved_value: 0.9,
                },
            },
            color_mask: 0x0f,
            outline: MaterialOutlineState::WorldSpace {
                width: 0.0125,
                color: [0.0, 0.0, 0.0, 1.0],
            },
        }],
        standard_texture_refs_are_loader_hints: true,
    });
    model.textures.push(NativeTexture {
        source_name: "spwaneye.dds".into(),
        uri: "textures/spwaneye.png".into(),
        width: 64,
        height: 32,
        sampler: 0,
        mip_provenance,
        mip_levels,
    });
    model.samplers.push(sampler);
    model
}

pub(super) fn accessor_f32(bytes: &[u8], document: &Value, accessor: usize) -> Vec<f32> {
    let json_length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let binary_header = 20 + json_length;
    assert_eq!(
        u32::from_le_bytes(
            bytes[binary_header + 4..binary_header + 8]
                .try_into()
                .unwrap()
        ),
        0x004e_4942
    );
    let accessor = &document["accessors"][accessor];
    assert_eq!(accessor["componentType"], 5_126);
    let view = accessor["bufferView"].as_u64().unwrap() as usize;
    let view = &document["bufferViews"][view];
    let offset = view["byteOffset"].as_u64().unwrap_or(0) as usize
        + accessor["byteOffset"].as_u64().unwrap_or(0) as usize;
    let components = match accessor["type"].as_str().unwrap() {
        "SCALAR" => 1,
        "VEC2" => 2,
        "VEC3" => 3,
        "VEC4" => 4,
        "MAT4" => 16,
        other => panic!("unexpected accessor type {other}"),
    };
    let count = accessor["count"].as_u64().unwrap() as usize * components;
    let binary = &bytes[binary_header + 8..];
    binary[offset..offset + count * 4]
        .chunks_exact(4)
        .map(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()))
        .collect()
}

#[test]
fn rejects_lost_or_corrupt_skin_data() {
    let mut model = rex();
    model.meshes[0].primitives[0].weights[0] = [0.2, 0.2, 0.2, 0.2];
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("not normalized")
    );
    let mut model = rex();
    model.skins[0].inverse_bind_matrices.clear();
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("inverse-bind")
    );
    let mut model = rex();
    model.meshes[0].primitives[0].joints[0][0] = 1;
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("exceeds palette")
    );
}

#[test]
fn allows_exact_negative_preroll_keys_in_float_curves() {
    let mut model = rex();
    model.animations[0].metadata.float_curves[0].times = vec![-1.0 / 30.0, 1.0];
    validate(&model).unwrap();
}

#[test]
fn allows_exact_zero_key_float_binding_metadata() {
    let mut model = rex();
    let curve = &mut model.animations[0].metadata.float_curves[0];
    curve.source_key_count = 0;
    curve.source_key_indices.clear();
    curve.interpolation = Interpolation::CubicSpline;
    curve.times.clear();
    curve.values.clear();
    curve.in_tangents = Some(Vec::new());
    curve.out_tangents = Some(Vec::new());
    curve.tangent_modes.clear();

    validate(&model).expect("zero-key float binding is exact metadata");
    let document = glb_json(&encode_glb(&model).unwrap());
    let preserved = &document["animations"][0]["extras"]["nonTrs"]["floatCurves"][0];
    assert_eq!(preserved["sourceKeyCount"], json!(0));
    assert_eq!(preserved["times"], json!([]));
    assert_eq!(preserved["values"], json!([]));
}

#[test]
fn requires_one_exactly_named_logical_root() {
    let mut model = rex();
    model.nodes.push(ModelNode {
        name: "Other".into(),
        legacy_name: None,
        legacy_sibling_ordinal: None,
        parent: None,
        translation: [0.0; 3],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0; 3],
        mesh: None,
        skin: None,
    });
    model.roots.push(2);
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("exactly one root")
    );

    let mut model = rex();
    model.nodes[0].name = "NotRex".into();
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("exact logical model name")
    );
}

#[test]
fn cubic_spline_interleaves_tangents_values_and_preserves_modes() {
    let mut model = rex();
    let channel = &mut model.animations[0].channels[0];
    channel.interpolation = Interpolation::CubicSpline;
    channel.values = TrackValues::Translation(vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
    channel.in_tangents = Some(TrackValues::Translation(vec![
        [0.1, 0.2, 0.3],
        [0.4, 0.5, 0.6],
    ]));
    channel.out_tangents = Some(TrackValues::Translation(vec![
        [0.7, 0.8, 0.9],
        [1.0, 1.1, 1.2],
    ]));
    channel.tangent_modes = vec![21, 22];

    let bytes = encode_glb(&model).unwrap();
    let document = glb_json(&bytes);
    let sampler = &document["animations"][0]["samplers"][0];
    assert_eq!(sampler["interpolation"], "CUBICSPLINE");
    let output = sampler["output"].as_u64().unwrap() as usize;
    assert_eq!(document["accessors"][output]["count"], 6);
    assert_eq!(
        document["animations"][0]["channels"][0]["extras"]["legacyTangentModes"],
        json!([21, 22])
    );
    assert_eq!(
        accessor_f32(&bytes, &document, output),
        vec![
            0.1, 0.2, 0.3, 1.0, 2.0, 3.0, 0.7, 0.8, 0.9, 0.4, 0.5, 0.6, 4.0, 5.0, 6.0, 1.0, 1.1,
            1.2,
        ]
    );
}

#[test]
fn rejects_missing_mismatched_or_unrepresentable_cubic_tangents() {
    let mut model = rex();
    let channel = &mut model.animations[0].channels[0];
    channel.interpolation = Interpolation::CubicSpline;
    channel.tangent_modes = vec![0, 0];
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("tangents are missing")
    );

    let mut model = rex();
    let channel = &mut model.animations[0].channels[0];
    channel.interpolation = Interpolation::CubicSpline;
    channel.in_tangents = Some(TrackValues::Rotation(vec![[0.0; 4]; 2]));
    channel.out_tangents = Some(TrackValues::Scale(vec![[0.0; 3]; 2]));
    channel.tangent_modes = vec![0, 0];
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("type mismatch")
    );

    let mut model = rex();
    let channel = &mut model.animations[0].channels[0];
    channel.interpolation = Interpolation::CubicSpline;
    channel.in_tangents = Some(TrackValues::Rotation(vec![[f64::MAX, 0.0, 0.0, 0.0]; 2]));
    channel.out_tangents = Some(TrackValues::Rotation(vec![[0.0; 4]; 2]));
    channel.tangent_modes = vec![0, 0];
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("not representable as f32")
    );
}

#[test]
fn linear_extra_textures_are_explicit_bevy_loader_hints() {
    let mut model = materialized_rex();
    let clone_texture = |name: &str, uri: &str, sampler_index: u32| {
        let mut texture = model.textures[0].clone();
        texture.source_name = name.into();
        texture.uri = uri.into();
        texture.sampler = sampler_index;
        let (parent, filename) = uri.rsplit_once('/').unwrap();
        let stem = filename.strip_suffix(".png").unwrap();
        for level in &mut texture.mip_levels {
            level.uri = if level.level == 0 {
                uri.into()
            } else {
                format!("{parent}/{stem}.mips/mip-{:02}.png", level.level)
            };
        }
        let mut sampler = model.samplers[0].clone();
        sampler.name = name.into();
        (texture, sampler)
    };
    let (bump, bump_sampler) = clone_texture("bubble2.dds", "textures/bubble2.png", 1);
    let (shader_map, shader_sampler) =
        clone_texture("fusionlight.DDS", "textures/fusionlight.png", 2);
    let binding = |slot: &str,
                   texture_index: u32,
                   texture: &NativeTexture,
                   sampler: &NativeSampler| MaterialTextureBinding {
        slot: slot.into(),
        unassigned_stale_null: false,
        ignored_stale_shader_binding: false,
        dynamic_texture: None,
        texture: Some(texture_index),
        source_name: Some(texture.source_name.clone()),
        uri: Some(texture.uri.clone()),
        sampler: Some(MaterialTextureSamplerBinding {
            index: texture.sampler,
            descriptor: sampler.clone(),
        }),
        mip_provenance: Some(texture.mip_provenance.clone()),
        mip_levels: Some(texture.mip_levels.clone()),
        scale: [1.0, 1.0],
        offset: [0.0, 0.0],
        pivot: Some([0.0, 0.0]),
        rotation: Some(0.0),
        color_space: TextureColorSpace::Linear,
    };
    model.materials[0].shader_texture_defaults.extend([
        ShaderLabTextureDefaultProperty {
            slot: "_BumpMap".into(),
            value: ShaderLabTextureDefault::BuiltinBump,
        },
        ShaderLabTextureDefaultProperty {
            slot: "_ShaderMap".into(),
            value: ShaderLabTextureDefault::BuiltinWhite,
        },
    ]);
    model.materials[0]
        .texture_bindings
        .push(binding("_BumpMap", 1, &bump, &bump_sampler));
    model.materials[0].texture_bindings.push(binding(
        "_ShaderMap",
        2,
        &shader_map,
        &shader_sampler,
    ));
    model.textures.extend([bump, shader_map]);
    model.samplers.extend([bump_sampler, shader_sampler]);

    let glb = encode_glb(&model).unwrap();
    let parsed = gltf::Gltf::from_slice(&glb).expect("loader-hint GLB remains standard glTF");
    let material = parsed.document.materials().next().unwrap();
    assert_eq!(material.normal_texture().unwrap().texture().index(), 1);
    assert_eq!(material.occlusion_texture().unwrap().texture().index(), 2);

    let document = glb_json(&glb);
    let material = &document["materials"][0];
    assert_eq!(material["normalTexture"]["index"], 1);
    assert_eq!(
        material["normalTexture"]["extras"]["ffone"]["slot"],
        "_BumpMap"
    );
    assert_eq!(
        material["normalTexture"]["extras"]["ffone"]["colorSpace"],
        "linear"
    );
    assert_eq!(
        material["normalTexture"]["extras"]["ffone"]["standardTextureRefIsLoaderHint"],
        true
    );
    assert_eq!(material["occlusionTexture"]["index"], 2);
    assert_eq!(
        material["occlusionTexture"]["extras"]["ffone"]["slot"],
        "_ShaderMap"
    );
    assert_eq!(
        material["occlusionTexture"]["extras"]["ffone"]["colorSpace"],
        "linear"
    );
    assert_eq!(
        material["extras"]["ffone"]["standardTextureRefsAreLoaderHints"],
        true
    );
}

#[test]
fn preserves_an_explicit_null_renderer_slot_alongside_real_materials() {
    let mut model = materialized_rex();
    let mut explicit_null = model.meshes[0].primitives[0].clone();
    explicit_null.material = None;
    explicit_null.material_slot = None;
    model.meshes[0].primitives.push(explicit_null);

    validate(&model).expect("an authoritative renderer null slot is valid");
    let glb = encode_glb(&model).expect("the null slot must remain representable in GLB");
    let document = glb_json(&glb);
    let primitives = document["meshes"][0]["primitives"].as_array().unwrap();
    assert_eq!(primitives[0]["material"], 0);
    assert!(primitives[1].get("material").is_none());
    assert!(primitives[1]["extras"]["materialSlot"].is_null());

    let mut unresolved_named_slot = materialized_rex();
    unresolved_named_slot.meshes[0].primitives[0].material = None;
    assert!(
        validate(&unresolved_named_slot)
            .unwrap_err()
            .to_string()
            .contains("named material slot but no material index")
    );
}
