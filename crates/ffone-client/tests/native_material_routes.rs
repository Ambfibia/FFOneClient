//! Exercise the actual runtime material decoder for domain-owned shared PNGs.
use ffone_client::legacy_model_material::PendingLegacyModelMaterial;
use std::{fs, io::Read, path::Path};

#[test]
fn domain_relocation_does_not_admit_arbitrary_parent_texture_routes() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game/characters/fusions/fusion_dexter/fusion_dexter.glb");
    let bytes = fs::read(path).unwrap();
    let length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let document: serde_json::Value = serde_json::from_slice(&bytes[20..20 + length]).unwrap();
    let material = document["materials"]
        .as_array()
        .unwrap()
        .iter()
        .find(|material| material["name"] == "spwaneye")
        .unwrap();
    for uri in [
        "../../../effects/shared/textures/../outside.png",
        "../../../audio/shared/textures/spwaneye.png",
        "../arbitrary/spwaneye.png",
    ] {
        let mut extras = material["extras"].clone();
        let binding = extras["ffone"]["textureBindings"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|binding| binding["slot"] == "_MainTex")
            .unwrap();
        binding["uri"] = uri.into();
        let error =
            PendingLegacyModelMaterial::from_gltf_extras(Some("spwaneye"), &extras.to_string())
                .unwrap_err();
        assert!(
            format!("{error:?}").contains("unsafe or non-semantic"),
            "{error:?}"
        );
    }
}

#[test]
fn production_shared_texture_materials_are_accepted_by_the_runtime() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let mut pending = vec![root];
    let mut checked = 0;
    let mut errors = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            if !path.extension().is_some_and(|extension| extension == "glb") {
                continue;
            }
            let mut file = fs::File::open(&path).unwrap();
            let mut header = [0; 20];
            file.read_exact(&mut header).unwrap();
            let mut json = vec![0; u32::from_le_bytes(header[12..16].try_into().unwrap()) as usize];
            file.read_exact(&mut json).unwrap();
            let document: serde_json::Value = serde_json::from_slice(&json).unwrap();
            let Some(materials) = document["materials"].as_array() else {
                continue;
            };
            for material in materials {
                let extras = material["extras"].to_string();
                if !extras.contains("/shared/textures/") && !extras.contains("/textures/shared/") {
                    continue;
                }
                checked += 1;
                if let Err(error) =
                    PendingLegacyModelMaterial::from_gltf_extras(material["name"].as_str(), &extras)
                {
                    errors.push(format!(
                        "{}: {}: {error:?}",
                        path.display(),
                        material["name"]
                    ));
                }
            }
        }
    }
    assert!(
        checked > 400,
        "production shared materials were not inspected"
    );
    assert!(
        errors.is_empty(),
        "runtime rejected shared textures:\n{}",
        errors.join("\n")
    );
    println!("Validated {checked} production materials with shared texture routes");
}
