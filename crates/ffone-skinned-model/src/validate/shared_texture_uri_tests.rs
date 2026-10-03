use super::*;
#[test]
fn shared_texture_paths_keep_semantic_sources_and_safe_structure() {
    assert!(validate_texture_uri("ToonRamp", "../../shared/textures/toonramp_variant_03.png", false).is_ok());
    assert!(validate_texture_uri("Skin", "../../shared/textures/skin-deadbeef1234.png", false).is_ok());
    assert!(validate_texture_uri("Skin", "../../shared/textures/../escape.png", false).is_err());
    assert!(validate_texture_uri("Skin", "../../shared/textures/skin.ogg", false).is_err());
    assert!(validate_texture_uri("Skin", "https://example.test/skin.png", false).is_err());
    assert!(validate_texture_uri("Skin", "model.textures/Wrong.png", false).is_err());
}
