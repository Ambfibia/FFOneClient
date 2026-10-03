use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct NativePlayerTexture {
    pub path: String,
    pub contract: CharacterRuntimeTextureContract,
}

pub(super) fn validate_player_texture(texture: &NativePlayerTexture) -> Result<(), String> {
    validate_asset_path(&texture.path, ".png")?;
    let contract = &texture.contract;
    let source_mip_chain_is_consistent = if contract.source.mip_map {
        contract.source.source_mip_count > 1
    } else {
        contract.source.source_mip_count == 1
    };
    if contract.native_asset.path != texture.path
        || contract.true_name.is_empty()
        || contract.sampler.name != contract.true_name
        || contract.usage_color_space != TextureColorSpace::Srgb
        || contract.published_mip_policy != PublishedMipPolicy::BaseLevelOnly
        || !source_mip_chain_is_consistent
        || contract.source.image_count != 1
        || contract.source.texture_dimension != 2
        || contract.sampler.mip_map_bias != 0.0
        || contract.native_png_sha256.len() != 64
        || !contract
            .native_png_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(format!(
            "native player texture {:?} has an invalid exact runtime contract",
            texture.path
        ));
    }
    Ok(())
}
