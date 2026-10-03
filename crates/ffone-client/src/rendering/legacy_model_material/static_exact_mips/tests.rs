use crate::legacy_model_material::static_exact_mips::*;

#[test]
fn published_manifest_is_typed_and_bound_to_the_primary_container() {
    let manifest = LegacyStaticExactMipManifest::from_published().unwrap();
    assert_eq!(manifest.source, AssetPath::from(PUBLISHED_MANIFEST_PATH));
    assert_eq!(manifest.bindings.len(), 3);
    for (texture_id, binding) in &manifest.bindings {
        assert_eq!(binding.slot, "_MainTex");
        assert!(texture_id.ends_with(&binding.texture_index.unwrap().to_string()));
        assert!(
            binding
                .mip_levels
                .as_ref()
                .is_some_and(|levels| levels.len() > 1)
        );
    }
}
