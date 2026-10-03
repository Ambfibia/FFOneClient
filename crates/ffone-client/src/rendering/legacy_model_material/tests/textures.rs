use super::*;

#[test]
fn static_texture_alpha_scan_distinguishes_opaque_from_cutout() {
    let opaque = Image::new_fill(
        Extent3d {
            width: 2,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[10, 20, 30, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    let cutout = Image::new_fill(
        Extent3d {
            width: 2,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[10, 20, 30, 127],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    assert_eq!(image_is_fully_opaque(&opaque), Some(true));
    assert_eq!(image_is_fully_opaque(&cutout), Some(false));
}

pub(super) fn test_runtime_texture_contract(
    true_name: &str,
    source_texture_format: i32,
    source_chain_sha256: &str,
) -> CharacterRuntimeTextureContract {
    serde_json::from_value(serde_json::json!({
        "trueName": true_name,
        "nativeAsset": {
            "path": format!("textures/{true_name}.png"),
            "bytes": 70,
            "blake3": "0000000000000000000000000000000000000000000000000000000000000000"
        },
        "nativePngSha256":
            "0000000000000000000000000000000000000000000000000000000000000000",
        "source": {
            "asset": "CharTexture.resourceFile/test",
            "containerRoute": format!("texture/{true_name}.dds"),
            "pathId": 1,
            "width": 1,
            "height": 1,
            "textureFormat": source_texture_format,
            "textureFormatName": "test",
            "completeImageSize": 4,
            "sourceChainSha256": source_chain_sha256,
            "mipMap": false,
            "sourceMipCount": 1,
            "imageCount": 1,
            "textureDimension": 2
        },
        "usageColorSpace": "srgb",
        "usageColorSpaceSource": "test ActorSkinCombiner assignment",
        "sampler": {
            "name": true_name,
            "magFilter": "linear",
            "minFilter": "linear",
            "wrapS": "repeat",
            "wrapT": "repeat",
            "legacyFilterMode": 1,
            "legacyWrapMode": 0,
            "anisotropyLevel": 1,
            "mipMapBias": 0.0
        },
        "publishedMipPolicy": "baseLevelOnly"
    }))
    .unwrap()
}

#[test]
fn actor_skin_dynamic_replacement_does_not_claim_authored_texture_chain_identity() {
    let authored = serde_json::from_value::<MaterialTextureBinding>(test_assigned_binding(
        "_MainTex",
        0,
        "m_shirt_coolshirt",
        "m_shirt_coolshirt.textures/m_shirt_coolshirt.png",
        "srgb",
    ))
    .unwrap();
    let binding = runtime_binding_from_typed(authored);
    let skin = test_runtime_texture_contract(
        "m_skin",
        10,
        "1111111111111111111111111111111111111111111111111111111111111111",
    );

    validate_runtime_replacement_contract(&binding, &skin)
        .expect("ActorSkinCombiner may replace an authored shirt map with secondary skin");
}

#[test]
fn actor_skin_same_source_replacement_still_rejects_mip_provenance_contradiction() {
    let authored = serde_json::from_value::<MaterialTextureBinding>(test_assigned_binding(
        "_MainTex",
        0,
        "m_skin",
        "skin.textures/m_skin.png",
        "srgb",
    ))
    .unwrap();
    let binding = runtime_binding_from_typed(authored);
    let contradictory = test_runtime_texture_contract(
        "m_skin",
        10,
        "0000000000000000000000000000000000000000000000000000000000000000",
    );

    assert!(
        validate_runtime_replacement_contract(&binding, &contradictory)
            .unwrap_err()
            .contains("mip provenance contradicts")
    );
}

#[test]
fn actor_skin_same_name_different_source_chain_uses_runtime_texture_authority() {
    let authored = serde_json::from_value::<MaterialTextureBinding>(test_assigned_binding(
        "_MainTex",
        0,
        "spawn11_green",
        "back_fish.textures/back_fish.png",
        "srgb",
    ))
    .unwrap();
    let binding = runtime_binding_from_typed(authored);
    let replacement = test_runtime_texture_contract(
        "spawn11_green",
        10,
        "1111111111111111111111111111111111111111111111111111111111111111",
    );

    validate_runtime_replacement_contract(&binding, &replacement).expect(
        "equal Texture2D names from different source chains must not conflate their samplers",
    );
}

#[test]
fn declared_null_npc_main_texture_retains_its_runtime_assignment_slot() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game/characters/npcs/npc_fusion_johnnytest/fusion_johnnytest.glb");
    let bytes = std::fs::read(path).unwrap();
    let length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let document: Value = serde_json::from_slice(&bytes[20..20 + length]).unwrap();
    let source = &document["materials"][0];
    let pending = PendingLegacyModelMaterial::from_gltf_extras(
        source["name"].as_str(),
        &serde_json::to_string(&source["extras"]).unwrap(),
    )
    .unwrap();
    let binding = pending
        .texture_bindings
        .iter()
        .find(|b| b.slot == "_MainTex")
        .expect("declared null _MainTex must accept the XDT texture");
    assert!(binding.sampler.is_none());
    assert_eq!(binding.color_space, TextureColorSpace::Srgb);
}

#[test]
fn external_texture_uri_allows_only_canonical_published_parent_routes() {
    let binding = serde_json::from_value::<MaterialTextureBinding>(test_assigned_binding(
        "_MainTex",
        0,
        "npc_dexter.dds",
        "npc_dexter.textures/npc_dexter.png",
        "srgb",
    ))
    .unwrap();
    validate_runtime_texture_binding(&binding).unwrap();

    let collision_safe =
        serde_json::from_value::<MaterialTextureBinding>(test_assigned_binding(
            "_MainTex",
            0,
            "npc_dexter.dds",
            "npc_dexter.textures/npc_dexter.dds.png",
            "srgb",
        ))
        .unwrap();
    validate_runtime_texture_binding(&collision_safe).unwrap();

    let shared_rendering =
        serde_json::from_value::<MaterialTextureBinding>(test_assigned_binding(
            "_ShaderMap",
            0,
            "ToonRamp9.bmp",
            "../../../../../rendering/textures/toonramp9_bmp.png",
            "linear",
        ))
        .unwrap();
    validate_runtime_texture_binding(&shared_rendering).unwrap();

    let shared_set = serde_json::from_value::<MaterialTextureBinding>(test_assigned_binding(
        "_MainTex",
        0,
        "f_shoes_kivaboots",
        "../../textures/shoes_kivaboots.png",
        "srgb",
    ))
    .unwrap();
    validate_runtime_texture_binding(&shared_set).unwrap();

    for uri in [
        "../../../effects/shared/textures/npc_dexter_variant_02.png",
        "../../shared/textures/npc_dexter_variant_02.png",
        "../nanomachine/textures/shared/npc_dexter_variant_02.png",
        "../../../../../characters/npcs/nanomachine/textures/shared/npc_dexter_variant_02.png",
    ] {
        let shared = serde_json::from_value::<MaterialTextureBinding>(test_assigned_binding(
            "_MainTex",
            0,
            "npc_dexter.dds",
            uri,
            "srgb",
        ))
        .unwrap();
        validate_runtime_texture_binding(&shared).unwrap();
    }
    for uri in [
        "../../../effects/shared/textures/../outside.png",
        "../../../audio/shared/textures/npc_dexter.png",
        "../arbitrary/npc_dexter.png",
    ] {
        assert!(!is_safe_relative_png_uri(uri), "accepted {uri}");
    }

    let mut escaped = binding;
    escaped.uri = Some("../npc_dexter.png".into());
    assert!(
        validate_runtime_texture_binding(&escaped)
            .unwrap_err()
            .0
            .contains("unsafe or non-semantic")
    );
}

#[test]
fn nonzero_legacy_mip_bias_is_fail_closed() {
    let value = test_assigned_binding(
        "_MainTex",
        0,
        "npc_dexter.dds",
        "npc_dexter.textures/npc_dexter.png",
        "srgb",
    );
    let typed = serde_json::from_value::<MaterialTextureBinding>(value).unwrap();
    let binding = LegacyGltfTextureBinding {
        slot: typed.slot,
        texture_index: typed.texture.map(|value| value as usize),
        source_name: typed.source_name,
        uri: typed.uri,
        sampler: typed.sampler.map(|mut sampler| {
            sampler.descriptor.mip_map_bias = 0.25;
            sampler
        }),
        mip_provenance: typed.mip_provenance,
        mip_levels: typed.mip_levels,
        color_space: typed.color_space,
        scale: Vec2::ONE,
        offset: Vec2::ZERO,
        pivot: Some(Vec2::splat(0.5)),
        rotation: Some(0.0),
    };
    let error = validate_loaded_image(&binding, &Image::default()).unwrap_err();
    assert!(error.contains("nonzero legacy mipMapBias"));
}
