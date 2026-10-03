use super::*;

#[test]
fn native_nano_outlines_preserve_surface_passes_and_titan_shell() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    for nano in ["mordecai", "titan", "vankleiss"] {
        let bytes = std::fs::read(root.join(format!(
            "characters/nanos/nano_{nano}/nano_{nano}.glb"
        )))
        .unwrap();
        let length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
        let model: Value = serde_json::from_slice(&bytes[20..20 + length]).unwrap();
        for material in model["materials"].as_array().unwrap() {
            let name = material["name"].as_str().unwrap();
            let mut extras = material["extras"].clone();
            let pending = PendingLegacyModelMaterial::from_gltf_extras(
                Some(name), &extras.to_string(),
            ).unwrap();
            assert!(pending.params.native_cel_shading, "{name}");
            let plan = pending.params.render_plan();
            let original = LegacyModelRenderPlan::for_shader(pending.params.shader);
            assert_eq!(plan.passes[0], original.passes[0], "surface changed: {name}");
            let shell = name.contains("shell");
            assert_eq!(pending.params.native_outline, !shell);
            let eye_rim = name == "nano_mordecai-eyes";
            assert_eq!(pending.params.native_ink_rim, eye_rim);
            let surface = pending.params.material_for_pass(plan.passes[0], &LegacyModelTextures::default()).unwrap();
            assert_eq!(surface.uniform.alpha_effect.w, if eye_rim { 2.0 } else { 1.0 });
            assert_eq!(plan.passes.len(), if shell { 1 } else { 2 });
            if shell {
                assert_eq!(material["alphaMode"], "BLEND");
                assert!(!plan.passes[0].render_mode.depth_write);
                assert_eq!(plan.passes[0].render_mode.blend, LegacyBlendMode::SrcAlphaOneMinusSrcAlpha);
            } else {
                let ink = pending.params.outline_material().unwrap();
                assert_eq!(ink.uniform.color, LinearRgba::BLACK);
                assert_eq!(ink.uniform.width_fat.x, 0.005);
                assert_eq!(ink.render_mode.cull, LegacyCullMode::Front);
            }
            extras["ffone"]["nativeOutline"] = Value::String("yes".into());
            assert!(PendingLegacyModelMaterial::from_gltf_extras(Some(name), &extras.to_string()).is_err());
            extras["ffone"]["nativeOutline"] = Value::Bool(true);
            extras["ffone"]["nativeInkRim"] = Value::String("yes".into());
            assert!(PendingLegacyModelMaterial::from_gltf_extras(Some(name), &extras.to_string()).is_err());
            extras["ffone"].as_object_mut().unwrap().remove("nativeInkRim");
            extras["ffone"]["nativeOutline"] = Value::Bool(true);
            extras["ffone"].as_object_mut().unwrap().remove("nativeSurfaceStyle");
            assert!(PendingLegacyModelMaterial::from_gltf_extras(Some(name), &extras.to_string()).is_err());
        }
    }
}

#[test]
fn mordecai_ink_rim_is_scoped_to_both_skinned_eyes() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game/characters/nanos/nano_mordecai/nano_mordecai.glb");
    let bytes = std::fs::read(path).unwrap();
    let length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let model: Value = serde_json::from_slice(&bytes[20..20 + length]).unwrap();
    let bin = &bytes[28 + length..];
    let primitives = model["meshes"][0]["primitives"].as_array().unwrap();
    assert_eq!(primitives.len(), 2);
    assert_eq!(primitives[0]["attributes"], primitives[1]["attributes"]);
    assert_eq!(primitives[0]["material"], 0);
    assert_eq!(primitives[1]["material"], 1);
    let read_indices = |accessor: usize| {
        let a = &model["accessors"][accessor];
        assert_eq!(a["componentType"], 5123);
        let view = &model["bufferViews"][a["bufferView"].as_u64().unwrap() as usize];
        let offset = view["byteOffset"].as_u64().unwrap() as usize;
        let count = a["count"].as_u64().unwrap() as usize;
        bin[offset..offset + count * 2].chunks_exact(2)
            .map(|b| u16::from_le_bytes(b.try_into().unwrap())).collect::<Vec<_>>()
    };
    let body = read_indices(primitives[0]["indices"].as_u64().unwrap() as usize);
    let eyes = read_indices(primitives[1]["indices"].as_u64().unwrap() as usize);
    assert_eq!(eyes.len(), 120 * 3);
    // Exact accepted eye islands; no head, beak, pupil texture or body triangles.
    assert!(eyes.iter().all(|i| (297..334).contains(i) || (453..490).contains(i)));
    assert!(eyes.iter().any(|i| (297..334).contains(i)));
    assert!(eyes.iter().any(|i| (453..490).contains(i)));
    assert!(body.iter().all(|i| !eyes.contains(i)));
    let mut original = read_indices(5).chunks_exact(3).map(|t| t.to_vec()).collect::<Vec<_>>();
    let mut split = body.chunks_exact(3).chain(eyes.chunks_exact(3))
        .map(|t| t.to_vec()).collect::<Vec<_>>();
    original.sort();
    split.sort();
    assert_eq!(split, original, "splitting must preserve every triangle and winding");
    assert_eq!(model["materials"][0]["extras"]["ffone"]["nativeInkRim"], Value::Null);
    assert_eq!(model["materials"][1]["extras"]["ffone"]["nativeInkRim"], true);
    assert_eq!(model["materials"][0]["extras"]["ffone"]["textureBindings"],
        model["materials"][1]["extras"]["ffone"]["textureBindings"]);
}
