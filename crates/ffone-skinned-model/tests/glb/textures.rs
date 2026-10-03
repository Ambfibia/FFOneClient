use super::*;

#[test]
fn png_filename_is_minimally_derived_from_exact_source_name() {
    assert_eq!(
        minimal_windows_png_filename("spwaneye").unwrap(),
        "spwaneye.png"
    );
    assert_eq!(
        minimal_windows_png_filename("ToonRamp9.bmp").unwrap(),
        "ToonRamp9.png"
    );
    assert_eq!(
        minimal_windows_png_filename("fusionlight.DDS").unwrap(),
        "fusionlight.png"
    );
    assert_eq!(
        minimal_windows_png_filename("Fusion: glass.dds").unwrap(),
        "Fusion_ glass.png"
    );
    assert_eq!(minimal_windows_png_filename("CON.dds").unwrap(), "_CON.png");
    assert!(minimal_windows_png_filename("Texture2D#123").is_err());
    assert!(minimal_windows_png_filename("eye--b7f8cca850c45cf0").is_err());
}

#[test]
fn rejects_broken_material_texture_links_and_extra_payloads() {
    let mut model = materialized_rex();
    model.meshes[0].primitives[0].material = Some(1);
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("material index is out of bounds")
    );

    let mut model = materialized_rex();
    model.meshes[0].primitives[0].material_slot = Some("spawn_eye".into());
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("exact material m_Name")
    );

    let mut model = materialized_rex();
    model.materials[0].texture_bindings[1].texture = Some(1);
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("texture index is out of bounds")
    );

    let mut model = materialized_rex();
    model.textures[0].sampler = 1;
    model.materials[0].texture_bindings[1]
        .sampler
        .as_mut()
        .unwrap()
        .index = 1;
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("invalid texture sampler")
    );

    let mut model = materialized_rex();
    let mut extra = model.textures[0].clone();
    extra.source_name = "unused.dds".into();
    extra.uri = "textures/unused.png".into();
    for (level_index, level) in extra.mip_levels.iter_mut().enumerate() {
        level.uri = if level_index == 0 {
            "textures/unused.png".into()
        } else {
            format!("textures/unused.mips/mip-{level_index:02}.png")
        };
    }
    model.textures.push(extra);
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("unreferenced extra textures")
    );
}

#[test]
fn rejects_hash_pathid_unsafe_or_non_true_name_texture_uris() {
    for uri in [
        "textures/spwaneye--b7f8cca850c45cf0.png",
        "textures/PathID_123/spwaneye.png",
        "C:/textures/spwaneye.png",
        "../spwaneye.png",
        "textures/spawn_eye.png",
    ] {
        let mut model = materialized_rex();
        model.textures[0].uri = uri.into();
        assert!(validate(&model).is_err(), "URI {uri:?} must be rejected");
    }

    let mut model = materialized_rex();
    model.materials[0].name = "Material#123".into();
    model.meshes[0].primitives[0].material_slot = Some("Material#123".into());
    assert!(validate(&model).is_err());
}
