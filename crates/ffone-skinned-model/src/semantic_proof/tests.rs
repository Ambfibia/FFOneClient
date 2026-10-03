use crate::{
    AnimationChannel, AnimationClip, AnimationEvent, AnimationMetadata, DuplicateTrsBinding,
    DuplicateTrsKeys, DuplicateTrsRelation, EmptyTrsBinding, EmptyTrsBindingKind,
    EmptyTrsSourceEncoding, ExactVec3Key, Interpolation, MODEL_SCHEMA, MaterialAlphaTestState,
    MaterialBlendFactor, MaterialBlendOperation, MaterialBlendState, MaterialCompareFunction,
    MaterialCullMode, MaterialOutlineState, MaterialPass, MaterialTextureBinding, ModelMesh,
    ModelNode, ModelPrimitive, ModelSkin, NativeMaterial, NativeModel, ShaderLabTextureDefault,
    ShaderLabTextureDefaultProperty, TextureColorSpace, TrackValues, encode_glb,
    exact_native_coordinate_contract,
};

use super::{prove_semantic_roundtrip, semantic_digests_from_glb};

fn fixture() -> NativeModel {
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
                translation: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0, 1.0, 1.0],
                mesh: Some(0),
                skin: Some(0),
            },
            ModelNode {
                name: "Bip01".into(),
                legacy_name: None,
                legacy_sibling_ordinal: None,
                parent: Some(0),
                translation: [0.0, 1.0, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0, 1.0, 1.0],
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
            inverse_bind_matrices: vec![[
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]],
        }],
        materials: Vec::new(),
        textures: Vec::new(),
        samplers: Vec::new(),
        animations: vec![
            AnimationClip {
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
                    interpolation: Interpolation::CubicSpline,
                    times: vec![0.0, 1.0],
                    values: TrackValues::Translation(vec![[0.0, 1.0, 0.0], [0.0, 1.5, 0.0]]),
                    in_tangents: Some(TrackValues::Translation(vec![[0.0; 3], [0.0; 3]])),
                    out_tangents: Some(TrackValues::Translation(vec![[0.0; 3], [0.0; 3]])),
                    tangent_modes: vec![0, 0],
                }],
                metadata: AnimationMetadata {
                    events: vec![AnimationEvent {
                        time: 0.25,
                        function_name: "Footstep".into(),
                        string_parameter: "left".into(),
                        float_parameter: 0.5,
                        float_parameter_provenance: None,
                        int_parameter: 1,
                        object_parameter: None,
                        object_parameter_provenance:
                            crate::AnimationEventObjectParameterProvenance::Missing {
                                interpretation:
                                    crate::MissingEventObjectParameterInterpretation::Missing,
                            },
                        message_options: 0,
                    }],
                    empty_trs_bindings: vec![EmptyTrsBinding {
                        kind: EmptyTrsBindingKind::Rotation,
                        target_node: 1,
                        target_path: "Bip01".into(),
                        source_index: 0,
                        source_encoding: EmptyTrsSourceEncoding::Plain,
                    }],
                    ..AnimationMetadata::default()
                },
            },
            AnimationClip {
                name: "nif-default".into(),
                duration: 0.0,
                declared_duration: None,
                keyed_duration: None,
                event_duration: None,
                sample_rate: None,
                wrap_mode: None,
                looped: false,
                channels: Vec::new(),
                metadata: AnimationMetadata::default(),
            },
        ],
    }
}

#[test]
fn proves_every_covered_section_from_redecoded_glb() {
    let model = fixture();
    let glb = encode_glb(&model).unwrap();
    let proof = prove_semantic_roundtrip(&model, &glb).unwrap();
    assert!(proof.matched);
    assert_eq!(proof.source, proof.emitted);
    assert_eq!(proof.emitted, semantic_digests_from_glb(&glb).unwrap());
    assert_eq!(proof.covered_scopes.len(), 6);
}

#[test]
fn rejects_same_count_geometry_value_change() {
    let model = fixture();
    let glb = encode_glb(&model).unwrap();
    let mut changed = model;
    changed.meshes[0].primitives[0].weights[0][0] = 0.75;
    changed.meshes[0].primitives[0].weights[0][1] = 0.25;
    let error = prove_semantic_roundtrip(&changed, &glb).unwrap_err();
    assert!(error.to_string().contains("mismatch in geometry"));
}

#[test]
fn rejects_same_count_hierarchy_skin_and_tangent_changes() {
    let model = fixture();
    let glb = encode_glb(&model).unwrap();

    let mut hierarchy = model.clone();
    hierarchy.nodes[1].translation[1] = 2.0;
    assert!(
        prove_semantic_roundtrip(&hierarchy, &glb)
            .unwrap_err()
            .to_string()
            .contains("mismatch in hierarchy")
    );

    let mut skin = model.clone();
    skin.skins[0].inverse_bind_matrices[0][3][1] = 0.25;
    assert!(
        prove_semantic_roundtrip(&skin, &glb)
            .unwrap_err()
            .to_string()
            .contains("mismatch in skins")
    );

    let mut animation = model;
    let Some(TrackValues::Translation(tangents)) =
        animation.animations[0].channels[0].out_tangents.as_mut()
    else {
        unreachable!()
    };
    tangents[1][0] = 0.5;
    assert!(
        prove_semantic_roundtrip(&animation, &glb)
            .unwrap_err()
            .to_string()
            .contains("mismatch in standard animations")
    );

    let mut metadata = fixture();
    metadata.animations[0].metadata.events[0].float_parameter = 0.75;
    assert!(
        prove_semantic_roundtrip(&metadata, &glb)
            .unwrap_err()
            .to_string()
            .contains("mismatch in animation metadata")
    );
}

#[test]
fn empty_trs_identity_path_kind_index_and_encoding_are_digest_bound() {
    let model = fixture();
    let glb = encode_glb(&model).unwrap();

    let mut mutations = Vec::new();
    let mut path = model.clone();
    path.animations[0].metadata.empty_trs_bindings[0].target_path = "Rex/Bip01".into();
    mutations.push(path);
    let mut kind = model.clone();
    kind.animations[0].metadata.empty_trs_bindings[0].kind = EmptyTrsBindingKind::Scale;
    mutations.push(kind);
    let mut index = model.clone();
    index.animations[0].metadata.empty_trs_bindings[0].source_index = 1;
    mutations.push(index);
    let mut encoding = model;
    encoding.animations[0].metadata.empty_trs_bindings[0].source_encoding =
        EmptyTrsSourceEncoding::Compressed;
    mutations.push(encoding);

    for mutation in mutations {
        assert!(
            prove_semantic_roundtrip(&mutation, &glb)
                .unwrap_err()
                .to_string()
                .contains("mismatch in animation metadata")
        );
    }
}

fn material_with_optional_transform(
    pivot: Option<[f64; 2]>,
    rotation: Option<f64>,
) -> NativeMaterial {
    NativeMaterial {
        name: "legacy".into(),
        serialized_shader_name: "serialized-shader".into(),
        declared_shader_name: "declared-shader".into(),
        legacy_shader_name: "declared-shader".into(),
        render_queue: 2_000,
        colors: Vec::new(),
        floats: Vec::new(),
        shader_texture_defaults: vec![ShaderLabTextureDefaultProperty {
            slot: "_MainTex".into(),
            value: ShaderLabTextureDefault::BuiltinWhite,
        }],
        texture_bindings: vec![MaterialTextureBinding {
            slot: "_MainTex".into(),
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
            pivot,
            rotation,
            color_space: TextureColorSpace::Srgb,
        }],
        passes: vec![MaterialPass {
            name: None,
            blend: MaterialBlendState {
                enabled: false,
                source_color: MaterialBlendFactor::One,
                destination_color: MaterialBlendFactor::Zero,
                color_operation: MaterialBlendOperation::Add,
                source_alpha: MaterialBlendFactor::One,
                destination_alpha: MaterialBlendFactor::Zero,
                alpha_operation: MaterialBlendOperation::Add,
            },
            cull: MaterialCullMode::Back,
            z_write: true,
            z_test: MaterialCompareFunction::LessEqual,
            alpha_test: MaterialAlphaTestState::Disabled,
            color_mask: 0x0f,
            outline: MaterialOutlineState::Disabled,
        }],
        standard_texture_refs_are_loader_hints: true,
    }
}

#[test]
fn material_transform_absence_and_explicit_zero_have_distinct_semantic_proofs() {
    let mut absent = fixture();
    absent.materials = vec![material_with_optional_transform(None, None)];
    absent.meshes[0].primitives[0].material = Some(0);
    absent.meshes[0].primitives[0].material_slot = Some("legacy".into());
    let absent_glb = encode_glb(&absent).unwrap();
    let absent_proof = prove_semantic_roundtrip(&absent, &absent_glb).unwrap();

    let mut explicit_zero = absent.clone();
    explicit_zero.materials[0].texture_bindings[0].pivot = Some([0.0, 0.0]);
    explicit_zero.materials[0].texture_bindings[0].rotation = Some(0.0);
    let explicit_glb = encode_glb(&explicit_zero).unwrap();
    let explicit_proof = prove_semantic_roundtrip(&explicit_zero, &explicit_glb).unwrap();

    assert_ne!(
        absent_proof.source.materials_sha256,
        explicit_proof.source.materials_sha256
    );
    assert!(
        prove_semantic_roundtrip(&explicit_zero, &absent_glb)
            .unwrap_err()
            .to_string()
            .contains("mismatch in materials")
    );
}

#[test]
fn shaderlab_white_and_blank_texture_defaults_are_digest_bound() {
    let mut white = fixture();
    white.materials = vec![material_with_optional_transform(None, None)];
    white.meshes[0].primitives[0].material = Some(0);
    white.meshes[0].primitives[0].material_slot = Some("legacy".into());
    let white_glb = encode_glb(&white).unwrap();
    let white_proof = prove_semantic_roundtrip(&white, &white_glb).unwrap();

    let mut blank = white.clone();
    blank.materials[0].shader_texture_defaults[0].value = ShaderLabTextureDefault::Blank;
    let blank_glb = encode_glb(&blank).unwrap();
    let blank_proof = prove_semantic_roundtrip(&blank, &blank_glb).unwrap();
    assert_ne!(
        white_proof.source.materials_sha256,
        blank_proof.source.materials_sha256
    );
    assert!(
        prove_semantic_roundtrip(&blank, &white_glb)
            .unwrap_err()
            .to_string()
            .contains("mismatch in materials")
    );
}

#[test]
fn animation_metadata_json_roundtrip_preserves_holonano_tangent_ulp() {
    let mut model = fixture();
    let tangent = f64::from_bits(13_703_890_709_155_676_159);
    model.animations[0].channels[0].in_tangents = Some(TrackValues::Translation(vec![
        [tangent, 0.0, 0.0],
        [0.0; 3],
    ]));
    model.animations[0].channels[0].out_tangents = Some(TrackValues::Translation(vec![
        [tangent, 0.0, 0.0],
        [0.0; 3],
    ]));
    model.animations[0].metadata.duplicate_trs_bindings = vec![DuplicateTrsBinding {
        kind: EmptyTrsBindingKind::Translation,
        target_node: 1,
        target_path: "Bip01".into(),
        source_index: 1,
        source_encoding: EmptyTrsSourceEncoding::Plain,
        relation: DuplicateTrsRelation::Identical,
        canonical_track_index: 0,
        canonical_source_index: 0,
        canonical_source_encoding: EmptyTrsSourceEncoding::Plain,
        source_key_count: 2,
        duplicate_keys: Vec::new(),
        keys: DuplicateTrsKeys::Vec3(vec![
            ExactVec3Key {
                source_key_index: 0,
                time: 0.0,
                value: [0.0, 1.0, 0.0],
                in_tangent: Some([tangent, 0.0, 0.0]),
                out_tangent: Some([tangent, 0.0, 0.0]),
                tangent_mode: Some(0),
            },
            ExactVec3Key {
                source_key_index: 1,
                time: 1.0,
                value: [0.0, 1.5, 0.0],
                in_tangent: Some([0.0; 3]),
                out_tangent: Some([0.0; 3]),
                tangent_mode: Some(0),
            },
        ]),
        resolution_proof: None,
    }];
    let glb = encode_glb(&model).unwrap();
    prove_semantic_roundtrip(&model, &glb).unwrap();
}
