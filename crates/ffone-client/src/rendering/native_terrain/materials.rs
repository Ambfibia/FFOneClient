use super::*;

/// CPU reference for one realtime BlendPass group.
///
/// `Realtime-BlendPass` multiplies every sampled terrain RGB by the matching
/// RGBA control channel and adds the four results. The same weights sum into
/// pass alpha (all published tutorial albedos are opaque). This is distinct
/// from the sequential G/B/A lerp in `Lightmap-BlendPass`. A serialized
/// control-map quartet can end in zero-weight placeholder layers whose saved
/// mode is Splat, so the first layer selects the pass while all four channels
/// still participate in its shader.
pub(super) fn legacy_realtime_blend_group(layers: &[LegacyTerrainCompositorLayer]) -> (Vec3, f32) {
    debug_assert!(!layers.is_empty() && layers.len() <= 4);
    let mut source = Vec3::ZERO;
    let mut alpha = 0.0;
    for layer in layers {
        source += layer.texture_rgb * layer.weight;
        alpha += layer.weight;
    }
    (source, alpha)
}
