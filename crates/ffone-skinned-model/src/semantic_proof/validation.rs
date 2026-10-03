use super::*;

/// Retrobution's updated Fusion Eyes are a single exact primary-source
/// Texture2D/material contract. Models without the optional eye material pass;
/// models that declare it must carry the new high-resolution payload metadata.
pub fn validate_retrobution_fusion_eye_contract(bytes: &[u8]) -> Result<()> {
    const EYE_SHADER: &str = "normal_blendOneOneTest_cullOff";
    const EYE_SHADER_SHA256: &str =
        "cb4b27442bbd396742347760f7610d17d767e2d4fa226ec8cd3ec863cdebe0ed";
    const EYE_PNG_SHA256: &str = "b895006b292f14784a902625761cf75b642c2103d3d845605f09bc47851de214";

    let glb = ParsedGlb::parse(bytes)?;
    let materials = required_array(&glb.document, "materials", "GLB materials")?;
    let eyes = materials
        .iter()
        .filter(|material| material.get("name").and_then(Value::as_str) == Some("spwaneye"))
        .collect::<Vec<_>>();
    if eyes.is_empty() {
        return Ok(());
    }
    let [eye] = eyes.as_slice() else {
        return invalid("Retrobution Fusion model must not duplicate spwaneye material");
    };
    if eye.get("doubleSided").and_then(Value::as_bool) != Some(true)
        || eye.get("alphaMode").and_then(Value::as_str) != Some("MASK")
        || eye.get("alphaCutoff").and_then(Value::as_f64) != Some(0.0)
    {
        return invalid("Retrobution spwaneye has incorrect glTF render state");
    }
    let ffone = eye
        .pointer("/extras/ffone")
        .ok_or_else(|| ModelError::Invalid("Retrobution spwaneye has no extras.ffone".into()))?;
    for field in [
        "serializedShaderName",
        "declaredShaderName",
        "legacyShaderName",
    ] {
        if ffone.get(field).and_then(Value::as_str) != Some(EYE_SHADER) {
            return invalid(format!("Retrobution spwaneye has incorrect {field}"));
        }
    }
    let bindings = required_array_value(ffone.get("textureBindings"), "spwaneye bindings")?;
    let main = bindings
        .iter()
        .filter(|binding| binding.get("slot").and_then(Value::as_str) == Some("_MainTex"))
        .collect::<Vec<_>>();
    let [main] = main.as_slice() else {
        return invalid("Retrobution spwaneye requires exactly one _MainTex binding");
    };
    if main.get("sourceName").and_then(Value::as_str) != Some("spwaneye.dds")
        || main.get("colorSpace").and_then(Value::as_str) != Some("srgb")
        || main
            .pointer("/mipProvenance/publishedPolicy")
            .and_then(Value::as_str)
            != Some("baseLevelOnly")
        || main
            .pointer("/mipProvenance/sourceTextureFormatName")
            .and_then(Value::as_str)
            != Some("RGBA32")
        || main
            .pointer("/mipProvenance/sourceMipCount")
            .and_then(Value::as_u64)
            != Some(1)
        || main
            .pointer("/sampler/descriptor/magFilter")
            .and_then(Value::as_str)
            != Some("linear")
        || main
            .pointer("/sampler/descriptor/minFilter")
            .and_then(Value::as_str)
            != Some("linear")
        || main
            .pointer("/sampler/descriptor/wrapS")
            .and_then(Value::as_str)
            != Some("repeat")
        || main
            .pointer("/sampler/descriptor/wrapT")
            .and_then(Value::as_str)
            != Some("repeat")
    {
        return invalid("Retrobution spwaneye texture/sampler contract is incorrect");
    }
    let levels = required_array_value(main.get("mipLevels"), "spwaneye mip levels")?;
    let [level] = levels.as_slice() else {
        return invalid("Retrobution spwaneye must publish exactly one source mip");
    };
    if level.get("width").and_then(Value::as_u64) != Some(128)
        || level.get("height").and_then(Value::as_u64) != Some(512)
        || level.get("pngSha256").and_then(Value::as_str) != Some(EYE_PNG_SHA256)
    {
        return invalid("Retrobution spwaneye is not the accepted 128x512 primary texture");
    }
    let material_report = ffone
        .get("shaderSha256")
        .and_then(Value::as_str)
        .unwrap_or(EYE_SHADER_SHA256);
    if material_report != EYE_SHADER_SHA256 {
        return invalid("Retrobution spwaneye shader hash is incorrect");
    }
    Ok(())
}
