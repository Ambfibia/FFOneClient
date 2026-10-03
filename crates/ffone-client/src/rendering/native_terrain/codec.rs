use super::*;

pub(super) const NATIVE_TERRAIN_MATERIALIZATIONS_PER_FRAME: usize = 1;

pub(super) const NATIVE_TERRAIN_GRASS_CHUNKS_PER_FRAME: usize = 64;

pub(super) const NATIVE_TERRAIN_GRASS_VERTICES_PER_FRAME: usize = 65_536;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct VerifiedRgbaPayloadCacheKey {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) rgba_blake3: String,
}

pub(super) fn decode_png(bytes: &[u8], path: &Path) -> Result<DynamicImage, NativeTerrainError> {
    image::load_from_memory_with_format(bytes, ImageFormat::Png).map_err(|error| {
        NativeTerrainError::new(format!("failed to decode PNG {}: {error}", path.display()))
    })
}
