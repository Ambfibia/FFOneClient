use super::*;

pub(super) fn glb_json(bytes: &[u8]) -> Value {
    assert_eq!(&bytes[0..4], b"glTF");
    assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 2);
    assert_eq!(
        u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize,
        bytes.len()
    );
    let length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    serde_json::from_slice(&bytes[20..20 + length]).unwrap()
}

#[test]
fn rejects_values_that_overflow_glb_f32() {
    let mut model = rex();
    model.meshes[0].primitives[0].positions[0][0] = f64::MAX;
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("not representable as f32")
    );
}

#[test]
fn ordinary_gltf_parser_sees_native_materials_images_and_true_names() {
    let model = materialized_rex();
    let first = encode_glb(&model).unwrap();
    let second = encode_glb(&model).unwrap();
    assert_eq!(first, second, "material GLB output must be deterministic");

    let parsed = gltf::Gltf::from_slice(&first).expect("ordinary glTF parser must accept output");
    assert_eq!(parsed.document.materials().count(), 1);
    assert_eq!(parsed.document.images().count(), 1);
    assert_eq!(parsed.document.textures().count(), 1);
    assert_eq!(parsed.document.samplers().count(), 1);
    let material = parsed.document.materials().next().unwrap();
    assert_eq!(material.name(), Some("spwaneye"));
    let pbr = material.pbr_metallic_roughness();
    assert_eq!(pbr.base_color_factor(), [0.25, 0.5, 0.75, 0.8]);
    assert_eq!(pbr.metallic_factor(), 0.0);
    assert_eq!(pbr.roughness_factor(), 1.0);
    let texture_info = pbr.base_color_texture().unwrap();
    let transform = texture_info.texture_transform().unwrap();
    assert_eq!(transform.scale(), [0.5, 0.75]);
    assert_eq!(transform.offset(), [0.25, 0.125]);
    assert!((transform.rotation() - 0.2).abs() < 1.0e-6);
    let image = parsed.document.images().next().unwrap();
    assert_eq!(image.name(), Some("spwaneye.dds"));
    match image.source() {
        gltf::image::Source::Uri { uri, mime_type } => {
            assert_eq!(uri, "textures/spwaneye.png");
            assert_eq!(mime_type, Some("image/png"));
        }
        gltf::image::Source::View { .. } => panic!("PNG must stay externally viewable"),
    }

    let document = glb_json(&first);
    assert_eq!(document["meshes"][0]["primitives"][0]["material"], 0);
    assert_eq!(
        document["meshes"][0]["primitives"][0]["extras"]["materialSlot"],
        "spwaneye"
    );
    assert_eq!(document["extensionsUsed"], json!(["KHR_texture_transform"]));
    assert_eq!(document["samplers"][0]["magFilter"], 9_729);
    assert_eq!(document["samplers"][0]["minFilter"], 9_987);
    assert_eq!(document["materials"][0]["alphaMode"], "MASK");
    assert!((document["materials"][0]["alphaCutoff"].as_f64().unwrap() - 0.9).abs() < 1.0e-6);
    assert_eq!(
        document["materials"][0]["extras"]["ffone"]["passes"][0]["alphaTest"]["compare"],
        "greater"
    );
}
