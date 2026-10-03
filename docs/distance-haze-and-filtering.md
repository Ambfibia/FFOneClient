# Distance haze and texture filtering

Two owner-requested quality changes to the world renderer. Both keep the exact legacy data as
their source: the haze color is still the sampled `DongColorSetup` area fog, and the filtering
change never alters a serialized filter mode, wrap mode, mip chain, or pass order.

## Distance haze

The native world camera has `far = 340` (`EXTENDED_WORLD_CAMERA_FAR_NATIVE`) and culls each
world object at its own visibility range, which starts at `NATIVE_WORLD_MIN_OBJECT_RANGE = 75`.
The exact `cnPlayerCamera.DefaultAmbience` density, `fogDepth * 0.005`, is thinner than that
budget: a dong with `fogDepth 0.7` still shows 76% of a surface at the far clip, so terrain cut
against the sky and props dithered out of visibly clear air.

Two additions close that, in `terrain_ambience.rs` and the two model shaders:

- `horizon_haze_density(far)` is the smallest exponential-squared density that leaves only
  `HORIZON_HAZE_RESIDUAL` (2%) of a surface visible at `far`. `distance_haze_density` takes the
  larger of that and the exact `DefaultAmbience` value, so a dong that already fogs out sooner is
  untouched. Only the density moves; the sampled RGB is passed through unchanged, which is what
  keeps the Future green, Downtown blue, and the infected zones red.
- `legacy_model_base.wgsl` and `legacy_model_outline.wgsl` publish the visibility-range dither
  level as a continuous `range_fade` and take `max(fog_amount, range_fade)` for the fog stage. An
  object at the end of its range is fully hazed before its last dithered texels disappear, and its
  ink line follows the same curve instead of staying crisp around a fading surface.

`apply_legacy_world_ambience` reads `far` from the camera's own `Projection`, so a scene with a
longer far clip keeps the exact legacy density.

Materials outside the fixed-function fog allow-list (`legacy_shader_uses_fixed_function_fog`) are
still not hazed; those programs declare `Fog { Mode Off }` in the source ShaderLab. `ffWater`
likewise has no fog stage.

## Anisotropic filtering

`ANISOTROPIC FILTERING` in `OptionMode` was persisted but reached nothing. Two separate paths
needed it:

- **Terrain.** The splat compositor fetches every layer with `textureLoad` so each source keeps
  its own dimensions inside the losslessly padded texture array, which a WebGPU sampler
  `anisotropy_clamp` can never reach. `native_terrain.wgsl` now performs the construction itself:
  the mip level follows the footprint's short axis while up to `render_quality.x` bilinear taps
  walk its long axis. `NATIVE_TERRAIN_ISOTROPIC_TAPS` (1) reproduces the exact legacy
  `max(|ddx|, |ddy|)` single-mip footprint; `NATIVE_TERRAIN_ANISOTROPIC_TAPS` (4) is the enabled
  state. Weight maps keep the legacy footprint.
- **Legacy model textures.** Every published sampler serializes `anisotropyLevel: 1`, so world
  geometry was point-sampled along its long axis. `exact_sampler_descriptor` now raises only
  `anisotropy_clamp`, and only when all three filters are linear (WebGPU rejects it otherwise).
  Bevy bakes the sampler into the `Image` asset, so `sync_legacy_texture_anisotropy` also
  re-stamps resident textures when the option is toggled; `validate_sampler_descriptor` accepts
  either side of a live toggle while keeping filters and wrap modes exact.

## GPU verification

`world_performance_probe` reproduces the gameplay ambience path, including the terrain uniform, so
its capture is a picture of the production renderer. `FFONE_PROBE_ANISOTROPIC=0` reproduces the
disabled option so a capture pair isolates terrain filtering.

```powershell
cargo run -p ffone-client --features diagnostics --example world_performance_probe -- baseline target/performance/haze
```

Keep comparison images and reports under ignored `target/performance`. The WGSL files are embedded
in the executable, so rebuild the client to pick up edits.
