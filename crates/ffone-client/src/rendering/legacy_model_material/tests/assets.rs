use super::*;

#[test]
fn static_world_adapter_uses_asset_baked_winding_without_normal_inversion() {
    let extras = serde_json::json!({
        "ffoneSourceMaterialId": "CustomAssetBundle-example:42",
        "shaderName": "normal",
        "colors": [],
        "floats": [],
        "exactTextureSlots": []
    })
    .to_string();
    let pending =
        PendingLegacyStaticWorldMaterial::from_gltf_extras(Some("wall"), &extras).unwrap();
    let material = pending
        .params
        .material_for_pass(pending.render_passes[0], &LegacyModelTextures::default())
        .unwrap();
    assert_eq!(material.render_mode.cull, LegacyCullMode::Back);
    assert!(pending.params.fixed_function_fog);
    assert_eq!(material.uniform.legacy_effect.w, 1.0);
}
