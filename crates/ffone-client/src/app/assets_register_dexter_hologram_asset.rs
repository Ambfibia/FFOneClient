use super::*;

pub(super) const CHARACTER_ASSET_SOURCE: &str = "character";

pub(super) const DEFAULT_CHARACTER_ASSET_ROOT: &str = "assets/game";

pub(super) fn dexter_hologram_asset_path() -> PathBuf {
    embedded_path!("dexter_hologram.wgsl")
}

pub(super) fn register_dexter_hologram_asset(app: &mut App) {
    let shader_path = dexter_hologram_asset_path();
    let watched_path = bevy::asset::io::embedded::watched_path(file!(), "../dexter_hologram.wgsl");
    app.world_mut()
        .resource_mut::<bevy::asset::io::embedded::EmbeddedAssetRegistry>()
        .insert_asset(
            watched_path,
            &shader_path,
            include_bytes!("../dexter_hologram.wgsl"),
        );
}
