use super::*;

#[derive(Clone, Copy)]
pub(super) enum PartTextureRule {
    Static,
    Face { eye_color: u8 },
    Hair,
}

pub(super) fn compatible_shared_texture_contract(
    left: &CharacterRuntimeTextureContract,
    right: &CharacterRuntimeTextureContract,
) -> bool {
    left.native_asset == right.native_asset
        && left.native_png_sha256 == right.native_png_sha256
        && left.usage_color_space == right.usage_color_space
        && left.usage_color_space_source == right.usage_color_space_source
        && left.sampler.mag_filter == right.sampler.mag_filter
        && left.sampler.min_filter == right.sampler.min_filter
        && left.sampler.wrap_s == right.sampler.wrap_s
        && left.sampler.wrap_t == right.sampler.wrap_t
        && left.sampler.legacy_filter_mode == right.sampler.legacy_filter_mode
        && left.sampler.legacy_wrap_mode == right.sampler.legacy_wrap_mode
        && left.sampler.anisotropy_level == right.sampler.anisotropy_level
        && left.sampler.mip_map_bias == right.sampler.mip_map_bias
        && left.published_mip_policy == right.published_mip_policy
}

pub(super) fn validate_runtime_texture_contract(
    locator: &AssetLocator,
    contract: &CharacterRuntimeTextureContract,
) -> CharacterCreationDataResult<()> {
    let path = normalize_relative_path(&contract.native_asset.path)?;
    if Path::new(&path)
        .extension()
        .and_then(|value| value.to_str())
        != Some("png")
    {
        return invalid(format!(
            "runtime texture contract route {path} is not a PNG"
        ));
    }
    let bytes = locator
        .read_verified(
            &path,
            Some(contract.native_asset.bytes),
            &contract.native_asset.blake3,
        )
        .map_err(CharacterCreationDataError::Invalid)?;
    let blake3 = blake3::hash(&bytes).to_hex().to_string();
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    if blake3 != contract.native_asset.blake3 || sha256 != contract.native_png_sha256 {
        return invalid(format!(
            "runtime texture contract route {path} failed byte/hash verification"
        ));
    }
    Ok(())
}
