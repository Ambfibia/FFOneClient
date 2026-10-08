use super::*;

#[test]
fn academy_equipment_materials_load_complete_shared_texture_chains() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let avatar: Value = serde_json::from_slice(&std::fs::read(root.join("data/character_creation/avatar_items.json")).unwrap()).unwrap();
    let first = std::collections::BTreeMap::from([
        ("back",174), ("glasses",123), ("hat",368), ("shirt",668),
        ("pants",563), ("shoes",564), ("vehicle",137), ("weapon",768),
    ]);
    let mut paths = std::collections::BTreeSet::new();
    for item in avatar["items"].as_array().unwrap() {
        let Some(minimum) = first.get(item["category"].as_str().unwrap()) else {continue;};
        if item["itemNumber"].as_u64().unwrap() < *minimum {continue;}
        for gender in ["male", "female"] {
            for model in item[gender]["models"].as_array().into_iter().flatten() {
                paths.insert(model["nativeAsset"]["path"].as_str().unwrap().to_owned());
            }
        }
    }
    assert!(!paths.is_empty());
    for route in paths {
        let path=root.join(&route);
        let bytes=std::fs::read(&path).unwrap();
        let length=u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
        let model:Value=serde_json::from_slice(&bytes[20..20+length]).unwrap();
        for material in model["materials"].as_array().unwrap() {
            let pending=PendingLegacyModelMaterial::from_gltf_extras(material["name"].as_str(),&material["extras"].to_string())
                .unwrap_or_else(|error|panic!("{route}: {error}"));
            for binding in pending.texture_bindings {
                for level in binding.mip_levels.into_iter().flatten() {
                    assert!(path.parent().unwrap().join(&level.uri).is_file(), "{route}: missing {}",level.uri);
                }
            }
        }
    }
}

pub(super) fn production_model_pending(
    relative_glb_path: &str,
    exact_material_name: &str,
) -> PendingLegacyModelMaterial {
    let glb_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_glb_path);
    let bytes = std::fs::read(&glb_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", glb_path.display()));
    assert_eq!(&bytes[0..4], b"glTF");
    let json_length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    assert_eq!(&bytes[16..20], b"JSON");
    let document: Value = serde_json::from_slice(&bytes[20..20 + json_length]).unwrap();
    let source_material = document["materials"]
        .as_array()
        .unwrap()
        .iter()
        .find(|material| material["name"].as_str() == Some(exact_material_name))
        .unwrap_or_else(|| {
            panic!(
                "{} has no exact material {exact_material_name:?}",
                glb_path.display()
            )
        });
    let extras = serde_json::to_string(&source_material["extras"]).unwrap();
    PendingLegacyModelMaterial::from_gltf_extras(Some(exact_material_name), &extras).unwrap()
}

#[test]
fn mesh_extras_preserve_exact_legacy_renderer_order() {
    assert_eq!(
        renderer_index_from_mesh_extras(r#"{"ffone":{"rendererIndex":2}}"#),
        Ok(Some(2))
    );
    assert_eq!(
        renderer_index_from_mesh_extras(r#"{"unrelated":true}"#),
        Ok(None)
    );
    assert!(renderer_index_from_mesh_extras(r#"{"ffone":{"rendererIndex":65536}}"#).is_err());
    assert!(renderer_index_from_mesh_extras(r#"{"ffone":{}}"#).is_err());
}

#[test]
fn relocated_chef_materials_reuse_base_mandroid_textures() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let path = root.join("characters/npcs/npc_mandroid_chef/npc_mandroid_chef.glb");
    let bytes = std::fs::read(&path).unwrap();
    let length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let model: Value = serde_json::from_slice(&bytes[20..20 + length]).unwrap();
    let mut reused_body = false;
    for material in model["materials"].as_array().unwrap() {
        let pending = PendingLegacyModelMaterial::from_gltf_extras(
            material["name"].as_str(),
            &serde_json::to_string(&material["extras"]).unwrap(),
        )
        .expect("relocated Chef material must pass the runtime URI contract");
        for binding in &pending.texture_bindings {
            if let Some(uri) = &binding.uri {
                assert!(path.parent().unwrap().join(uri).is_file(), "{uri}");
                reused_body |= uri == "../npc_mandroid1/npc_mandroid1.textures/npc_mandroid1.png";
            }
        }
    }
    assert!(reused_body, "Chef must share the original Mandroid body texture");
}

#[test]
fn production_fusion_kimchi_uses_original_atlas_and_complete_kimchi_mesh() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let bytes =
        std::fs::read(root.join("characters/fusions/fusion_kimchi/fusion_kimchi.glb")).unwrap();
    let length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let model: Value = serde_json::from_slice(&bytes[20..20 + length]).unwrap();
    // Fusion Kimchi uses its own original atlas on the complete Kimchi mesh.
    // The rejected donor eye cards and projected spwaneye material must not return.
    assert_eq!(model["meshes"].as_array().unwrap().len(), 4);
    assert!(
        model["meshes"]
            .as_array()
            .unwrap()
            .iter()
            .all(|m| m["primitives"].as_array().unwrap().len() == 1)
    );
    let body = &model["meshes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == "Editable Mesh1")
        .unwrap()["primitives"][0];
    assert_eq!(
        model["accessors"][body["indices"].as_u64().unwrap() as usize]["count"],
        660 * 3
    );
    assert_eq!(
        model["accessors"][body["attributes"]["POSITION"].as_u64().unwrap() as usize]["count"],
        550
    );
    assert_eq!(model["skins"][0]["joints"].as_array().unwrap().len(), 13);
    assert!(
        !model["materials"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["name"] == "spwaneye")
    );
    let tables: Value = crate::xdt::from_slice(
        &std::fs::read(root.join("data/tables/xdt.json")).unwrap(),
    )
    .unwrap();
    let npc = &tables["tables"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["value"].get("m_pNpcTable").is_some())
        .unwrap()["value"]["m_pNpcTable"];
    let mesh = npc["m_pNpcData"][3460]["m_iMesh"].as_u64().unwrap() as usize;
    assert_eq!(
        npc["m_pNpcMeshData"][mesh]["m_pstrMTextureString"],
        "fusion_kimchi"
    );
    let textures: Value = serde_json::from_slice(
        &std::fs::read(root.join("data/tables/npc_texture_overrides.json")).unwrap(),
    )
    .unwrap();
    let texture = textures["textures"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["trueName"] == "fusion_kimchi")
        .unwrap();
    assert_eq!(
        texture["path"],
        "characters/fusions/fusion_kimchi/textures/fusion_kimchi.png"
    );
    assert_eq!(
        texture["sha256"],
        "2617fe0aba23496809ae24f98ac21b17c38ccb5fab09d92c18583e4b1f6f4d22"
    );
    assert_eq!(texture["sampler"]["minFilter"], "linear");
    assert_eq!(texture["sampler"]["wrapS"], "repeat");
}

#[test]
fn exact_gltf_ffone_extras_become_typed_runtime_params() {
    let mut main = test_assigned_binding(
        "_MainTex",
        0,
        "fusion_dexter.dds",
        "fusion_dexter.textures/fusion_dexter.png",
        "srgb",
    );
    main["scale"] = serde_json::json!([2.0, 3.0]);
    main["offset"] = serde_json::json!([0.25, -0.5]);
    main["pivot"] = Value::Null;
    main["rotation"] = Value::Null;
    let extras = serde_json::json!({
        "ffone": {
            "name": "fusion_dexter-main-link_a.dds",
            "serializedShaderName": "Skin_FusionEffect_blendSrcalphaInvsrcalpha",
            "declaredShaderName": "Skin_FusionEffect_blendSrcalphaInvsrcalpha",
            "legacyShaderName": "Skin_FusionEffect_blendSrcalphaInvsrcalpha",
            "renderQueue": 2900,
            "standardTextureRefsAreLoaderHints": true,
            "colors": [
                {"name":"_Color","value":[0.25,0.5,0.75,1.0]},
                {"name":"_Emission","value":[0.1,0.2,0.3,0.0]}
            ],
            "floats": [
                {"name":"_Speed","value":7.0},
                {"name":"_Outline","value":0.005}
            ],
            "shaderTextureDefaults": [
                {"slot":"_MainTex","value":"builtinWhite"},
                {"slot":"_BumpMap","value":"builtinBump"},
                {"slot":"_ShaderMap","value":"builtinWhite"},
                {"slot":"_SpecMap","value":"blank"}
            ],
            "textureBindings": [
                main,
                test_assigned_binding("_BumpMap", 1, "bubble2.dds", "fusion_dexter.textures/bubble2.png", "linear"),
                test_assigned_binding("_ShaderMap", 2, "fusionlight.DDS", "fusion_dexter.textures/fusionlight.png", "linear"),
                test_null_binding("_SpecMap", "linear")
            ],
            "passes": [{
                "name":"BASE",
                "blend": {
                    "enabled":true,
                    "sourceColor":"sourceAlpha",
                    "destinationColor":"oneMinusSourceAlpha",
                    "colorOperation":"add",
                    "sourceAlpha":"sourceAlpha",
                    "destinationAlpha":"oneMinusSourceAlpha",
                    "alphaOperation":"add"
                },
                "cull":"back",
                "zWrite":true,
                "zTest":"lessEqual",
                "alphaTest":{"mode":"disabled"},
                "colorMask":7,
                "outline":{"mode":"disabled"}
            }]
        }
    })
    .to_string();
    let pending = PendingLegacyModelMaterial::from_gltf_extras(None, &extras).unwrap();
    assert_eq!(pending.true_name, "fusion_dexter-main-link_a.dds");
    assert_eq!(
        pending.serialized_shader_name,
        "Skin_FusionEffect_blendSrcalphaInvsrcalpha"
    );
    assert_eq!(
        pending.declared_shader_name,
        "Skin_FusionEffect_blendSrcalphaInvsrcalpha"
    );
    assert_eq!(pending.params.shader, LegacyShaderKind::FusionEffect);
    assert_eq!(
        pending.params.base_color,
        LinearRgba::new(0.25, 0.5, 0.75, 1.0)
    );
    assert_eq!(pending.params.uv_scale, Vec2::new(2.0, 3.0));
    assert_eq!(pending.params.uv_offset, Vec2::new(0.25, -0.5));
    assert_eq!(pending.params.fusion_speed, 7.0);
    assert_eq!(pending.shader_texture_defaults.len(), 4);
    assert_eq!(pending.shader_texture_defaults[0].slot, "_MainTex");
    assert_eq!(
        pending.shader_texture_defaults[3].value,
        ShaderLabTextureDefault::Blank
    );
    assert_eq!(pending.texture_bindings.len(), 4);
    assert_eq!(pending.texture_bindings[0].pivot, None);
    assert_eq!(pending.texture_bindings[0].rotation, None);
    assert_eq!(pending.texture_bindings[0].effective_pivot(), Vec2::ZERO);
    assert_eq!(pending.texture_bindings[0].effective_rotation(), 0.0);
    assert_eq!(pending.texture_bindings[1].pivot, Some(Vec2::splat(0.5)));
    assert_eq!(pending.texture_bindings[1].rotation, Some(0.0));
    assert_eq!(pending.source_render_queue, 2900);
    assert_eq!(pending.source_passes.len(), 1);
    assert_eq!(pending.source_pass_count, 1);
}
