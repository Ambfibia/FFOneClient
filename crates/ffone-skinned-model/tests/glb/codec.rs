use super::*;

#[test]
fn exact_ordered_material_payload_roundtrips_through_extras() {
    let model = materialized_rex();
    let document = glb_json(&encode_glb(&model).unwrap());
    let material: NativeMaterial =
        serde_json::from_value(document["materials"][0]["extras"]["ffone"].clone()).unwrap();
    assert_eq!(material, model.materials[0]);
    assert_eq!(material.name, "spwaneye");
    assert_eq!(material.colors[0].name, "_Emission");
    assert_eq!(material.colors[1].name, "_Color");
    assert_eq!(material.floats[0].name, "_Outline");
    assert_eq!(material.texture_bindings[0].slot, "_Detail");
    assert_eq!(material.texture_bindings[1].slot, "_MainTex");
    assert_eq!(material.texture_bindings[1].pivot, Some([0.5, 0.5]));

    let texture: NativeTexture =
        serde_json::from_value(document["images"][0]["extras"]["ffone"].clone()).unwrap();
    let sampler: NativeSampler =
        serde_json::from_value(document["samplers"][0]["extras"]["ffone"].clone()).unwrap();
    assert_eq!(texture, model.textures[0]);
    assert_eq!(sampler, model.samplers[0]);
    assert_eq!(
        document["images"][0]["extras"]["ffone"]["mipProvenance"]["publishedPixelTransform"],
        "vertical-flip-only-for-png-top-left-origin"
    );
    assert_eq!(
        document["materials"][0]["extras"]["ffone"]["textureBindings"][1]["mipProvenance"]["publishedPixelTransform"],
        "vertical-flip-only-for-png-top-left-origin"
    );
}
