use super::*;

#[test]
fn additive_and_glass_states_match_the_audit() {
    let additive_pass =
        LegacyModelRenderPlan::for_shader(LegacyShaderKind::AdditiveTwoSided).passes[0];
    let additive = additive_pass.render_mode;
    assert_eq!(additive.blend, LegacyBlendMode::OneOne);
    assert_eq!(additive.cull, LegacyCullMode::Off);
    assert!(!additive.depth_write);
    assert!(additive.alpha_cutout);
    assert_eq!(
        additive_pass.alpha_test,
        LegacyAlphaTest::GreaterMaterialCutoff
    );
    let es740_test =
        LegacyModelRenderPlan::for_shader(LegacyShaderKind::AdditiveTestTwoSidedQueue3011)
            .passes[0];
    assert_eq!(es740_test.render_mode.source_queue, 3011);
    assert_eq!(
        es740_test.alpha_test,
        LegacyAlphaTest::GreaterMaterialCutoff
    );
    let es740_depth =
        LegacyModelRenderPlan::for_shader(LegacyShaderKind::AdditiveOneOneTwoSidedDepthWrite)
            .passes[0]
            .render_mode;
    assert_eq!(es740_depth.cull, LegacyCullMode::Off);
    assert!(es740_depth.depth_write);
    assert_eq!(es740_depth.source_queue, 3000);
    let es740_backface =
        LegacyModelRenderPlan::for_shader(LegacyShaderKind::AdditiveOneOneBackface).passes[0]
            .render_mode;
    assert_eq!(es740_backface.cull, LegacyCullMode::Back);
    assert!(!es740_backface.depth_write);
    assert_eq!(es740_backface.source_queue, 3011);
    let nanomachine_additive =
        LegacyModelRenderPlan::for_shader(LegacyShaderKind::AdditiveOneOneBackfaceDepthWrite)
            .passes[0]
            .render_mode;
    assert_eq!(nanomachine_additive.cull, LegacyCullMode::Back);
    assert!(nanomachine_additive.depth_write);
    assert_eq!(nanomachine_additive.source_queue, 3000);
    let glass = LegacyModelRenderPlan::for_shader(LegacyShaderKind::TransparentNormal).passes
        [0]
    .render_mode;
    assert_eq!(glass.blend, LegacyBlendMode::SrcAlphaOneMinusSrcAlpha);
    assert_eq!(glass.cull, LegacyCullMode::Back);
    assert!(!glass.depth_write);

    // `ETC_domeglass_04.source.json` preserves the exact original
    // ShaderLab state: Queue Transparent+10, Blend SrcAlpha
    // OneMinusSrcAlpha, ColorMask RGB, ZWrite Off, Cull Off.
    let dome_glass =
        LegacyModelRenderPlan::for_shader(LegacyShaderKind::TransparentNormalCullOff).passes[0]
            .render_mode;
    assert_eq!(dome_glass.blend, LegacyBlendMode::SrcAlphaOneMinusSrcAlpha);
    assert_eq!(dome_glass.cull, LegacyCullMode::Off);
    assert!(!dome_glass.depth_write);
    assert_eq!(dome_glass.color_write, LegacyColorWriteMask::Rgb);
    assert_eq!(dome_glass.source_queue, 3010);
}

#[test]
fn exact_source_mips_validate_hashes_and_assemble_contiguously() {
    let source_base_pixels = (1_u8..=16).collect::<Vec<_>>();
    let published_base_pixels = vec![9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8];
    let lower_pixels = vec![2_u8; 4];
    let base_png = b"published-base-png".to_vec();
    let lower_png = b"published-lower-png".to_vec();
    let source_level_zero = b"source-level-zero";
    let source_level_one = b"source-one";
    let mut source_chain = source_level_zero.to_vec();
    source_chain.extend_from_slice(source_level_one);
    let uri = "npc_dexter.textures/npc_dexter_glass.png";
    let lower_uri = "npc_dexter.textures/npc_dexter_glass.mips/mip-01.png";
    let mut value = test_assigned_binding("_MainTex", 0, "npc_dexter_glass.dds", uri, "srgb");
    value["mipProvenance"] = serde_json::json!({
        "sourceTextureFormat": 12,
        "sourceTextureFormatName": "DXT5",
        "sourceMipCount": 2,
        "sourceChainByteLength": source_chain.len(),
        "sourceChainSha256": sha256_hex(&source_chain),
        "sourceChainComplete": true,
        "sourceLayout": "largestToSmallestContiguous",
        "publishedPixelTransform": "vertical-flip-only-for-png-top-left-origin",
        "publishedPolicy": "exactSourceLevels"
    });
    value["mipLevels"] = serde_json::json!([
        {
            "level": 0,
            "width": 2,
            "height": 2,
            "uri": uri,
            "sourceByteOffset": 0,
            "sourceByteLength": source_level_zero.len(),
            "sourceByteSha256": sha256_hex(source_level_zero),
            "decodedRgba8ByteLength": source_base_pixels.len(),
            "decodedRgba8Sha256": sha256_hex(&source_base_pixels),
            "pngByteLength": base_png.len(),
            "pngSha256": sha256_hex(&base_png)
        },
        {
            "level": 1,
            "width": 1,
            "height": 1,
            "uri": lower_uri,
            "sourceByteOffset": source_level_zero.len(),
            "sourceByteLength": source_level_one.len(),
            "sourceByteSha256": sha256_hex(source_level_one),
            "decodedRgba8ByteLength": lower_pixels.len(),
            "decodedRgba8Sha256": sha256_hex(&lower_pixels),
            "pngByteLength": lower_png.len(),
            "pngSha256": sha256_hex(&lower_png)
        }
    ]);
    let typed = serde_json::from_value::<MaterialTextureBinding>(value.clone()).unwrap();
    validate_runtime_texture_binding(&typed).unwrap();
    let binding = runtime_binding_from_typed(typed);
    let sampler =
        exact_sampler_descriptor(&binding.slot, &binding.sampler.as_ref().unwrap().descriptor)
            .unwrap();
    let make_image = |width, height, data: Vec<u8>| {
        let mut image = Image::default();
        image.texture_descriptor.size.width = width;
        image.texture_descriptor.size.height = height;
        image.texture_descriptor.format = TextureFormat::Rgba8UnormSrgb;
        // Simulate glTF winning the URI-keyed load race with its generic
        // trilinear sampler. Exact mip staging must accept these pixels;
        // the separately assembled render Image must not inherit it.
        let mut staging_sampler = sampler.clone();
        staging_sampler.mipmap_filter = ImageFilterMode::Linear;
        image.sampler = ImageSampler::Descriptor(staging_sampler);
        image.data = Some(data);
        image
    };
    let base_image = make_image(2, 2, published_base_pixels.clone());
    let lower_image = make_image(1, 1, lower_pixels.clone());
    let metadata = binding.mip_levels.as_ref().unwrap();
    validate_loaded_mip_level(
        &binding,
        &metadata[0],
        &base_image,
        &ExactMipPngBytes {
            bytes: base_png.clone(),
        },
    )
    .unwrap();
    validate_loaded_mip_level(
        &binding,
        &metadata[1],
        &lower_image,
        &ExactMipPngBytes {
            bytes: lower_png.clone(),
        },
    )
    .unwrap();
    let assembled = assemble_exact_mip_image(&binding, &[base_image, lower_image]).unwrap();
    let mut expected = published_base_pixels;
    expected.extend_from_slice(&lower_pixels);
    assert_eq!(assembled.texture_descriptor.mip_level_count, 2);
    assert_eq!(assembled.texture_descriptor.size.width, 2);
    assert_eq!(assembled.texture_descriptor.size.height, 2);
    assert_eq!(assembled.data.as_deref(), Some(expected.as_slice()));
    assert!(
        assembled
            .asset_usage
            .contains(RenderAssetUsages::RENDER_WORLD),
        "the assembled image must be uploadable to WebGPU"
    );
    let ImageSampler::Descriptor(assembled_sampler) = &assembled.sampler else {
        panic!("assembled exact mip chain must carry a descriptor sampler");
    };
    validate_sampler_descriptor(
        &binding.slot,
        &binding.sampler.as_ref().unwrap().descriptor,
        assembled_sampler,
    )
    .unwrap();

    let error = validate_loaded_mip_level(
        &binding,
        &metadata[1],
        &make_image(1, 1, lower_pixels),
        &ExactMipPngBytes {
            bytes: b"tampered-lower-png".to_vec(),
        },
    )
    .unwrap_err();
    assert!(error.contains("published PNG length/hash mismatch"));

    value["mipLevels"][1]["uri"] = serde_json::json!("../mip-01.png");
    let invalid = serde_json::from_value::<MaterialTextureBinding>(value).unwrap();
    assert!(
        validate_runtime_texture_binding(&invalid)
            .unwrap_err()
            .0
            .contains("level 1 fields are invalid")
    );
}
