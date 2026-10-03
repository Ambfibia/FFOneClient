#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

struct NativeTerrainMaterialUniform {
    // xy = exact layer repetitions across the 512x512 TerrainData extent.
    // z = serialized legacy terrain layer mode, retained for native parity.
    layer_tile_scale_mode: array<vec4<f32>, 17>,
    // xy = source dimensions; z = last mip level; w = serialized mip bias.
    layer_source_size: array<vec4<f32>, 17>,
    // x = Unity filter mode; y = last mip; z = mip bias; w = base resolution.
    weight_source_sampler: array<vec4<f32>, 5>,
    // x = layer count; y = exact Terrain.m_RenderMode:
    // 0 Vertexlit, 1 Lightmap, 2 Realtime; z = m_SplatMapDistance;
    // w = isolated GPU diagnostic view (zero in gameplay).
    metadata: vec4<f32>,
    // rgb = exact DefaultAmbience light tint.
    ambience_light: vec4<f32>,
    // rgb = exact applied fog color; a = exponential density (zero disables).
    ambience_fog: vec4<f32>,
    // x = maximum anisotropic taps for the splat sampler. One tap reproduces
    // the exact legacy single-mip footprint; yzw are reserved.
    render_quality: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> material: NativeTerrainMaterialUniform;
@group(#{MATERIAL_BIND_GROUP}) @binding(1)
var weight_maps: texture_2d_array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2)
var weight_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3)
var layer_albedos: texture_2d_array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4)
var layer_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(5)
var terrain_lightmap: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(6)
var terrain_lightmap_sampler: sampler;

const LEGACY_TERRAIN_VERTEX_AMBIENT: f32 = 0.2;

fn splat_channel(value: vec4<f32>, channel: u32) -> f32 {
    if channel == 0u {
        return value.r;
    }
    if channel == 1u {
        return value.g;
    }
    if channel == 2u {
        return value.b;
    }
    return value.a;
}

fn wrapped_index(value: i32, size: i32) -> i32 {
    return ((value % size) + size) % size;
}

fn legacy_lod(
    uv: vec2<f32>,
    base_size: vec2<f32>,
    last_mip: f32,
    mip_bias: f32,
) -> f32 {
    let dx = dpdx(uv) * base_size;
    let dy = dpdy(uv) * base_size;
    let footprint = max(length(dx), length(dy));
    return clamp(log2(max(footprint, 1.0)) + mip_bias, 0.0, last_mip);
}

fn srgb_to_linear(value: vec3<f32>) -> vec3<f32> {
    let low = value / 12.92;
    let high = pow((value + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return select(low, high, value > vec3<f32>(0.04045));
}

fn load_layer_texel(
    layer: u32,
    coordinate: vec2<i32>,
    size: vec2<i32>,
    mip: u32,
) -> vec4<f32> {
    let wrapped = vec2<i32>(
        wrapped_index(coordinate.x, size.x),
        wrapped_index(coordinate.y, size.y),
    );
    return textureLoad(layer_albedos, wrapped, i32(layer), i32(mip));
}

// Unity FilterMode.Bilinear: bilinear texels within the nearest mip level.
// Manual addressing preserves every source's own dimensions inside the
// losslessly padded texture array.
fn sample_layer_bilinear_mip(
    layer: u32,
    uv: vec2<f32>,
    mip: u32,
) -> vec4<f32> {
    let base_size = vec2<u32>(material.layer_source_size[layer].xy);
    let size = vec2<i32>(max(base_size >> vec2<u32>(mip), vec2<u32>(1u)));
    let texel = fract(uv) * vec2<f32>(size) - vec2<f32>(0.5);
    let base = vec2<i32>(floor(texel));
    let fraction = fract(texel);
    let c00 = load_layer_texel(layer, base, size, mip);
    let c10 = load_layer_texel(layer, base + vec2<i32>(1, 0), size, mip);
    let c01 = load_layer_texel(layer, base + vec2<i32>(0, 1), size, mip);
    let c11 = load_layer_texel(layer, base + vec2<i32>(1, 1), size, mip);
    return mix(mix(c00, c10, fraction.x), mix(c01, c11, fraction.x), fraction.y);
}

fn sample_layer_bilinear(layer: u32, uv: vec2<f32>) -> vec4<f32> {
    let source = material.layer_source_size[layer];
    let lod = legacy_lod(uv, source.xy, source.z, source.w);
    return sample_layer_bilinear_mip(layer, uv, u32(floor(lod + 0.5)));
}

fn load_weight_texel(
    map_index: u32,
    coordinate: vec2<i32>,
    size: vec2<i32>,
    mip: u32,
) -> vec4<f32> {
    let clamped = clamp(coordinate, vec2<i32>(0), size - vec2<i32>(1));
    return textureLoad(weight_maps, clamped, i32(map_index), i32(mip));
}

fn sample_weight_point(map_index: u32, uv: vec2<f32>, mip: u32) -> vec4<f32> {
    let base_resolution = u32(material.weight_source_sampler[map_index].w);
    let size = vec2<i32>(i32(max(base_resolution >> mip, 1u)));
    let coordinate = vec2<i32>(floor(uv * vec2<f32>(size)));
    return load_weight_texel(map_index, coordinate, size, mip);
}

fn sample_weight_bilinear(map_index: u32, uv: vec2<f32>, mip: u32) -> vec4<f32> {
    let base_resolution = u32(material.weight_source_sampler[map_index].w);
    let size = vec2<i32>(i32(max(base_resolution >> mip, 1u)));
    let texel = uv * vec2<f32>(size) - vec2<f32>(0.5);
    let base = vec2<i32>(floor(texel));
    let fraction = fract(texel);
    let c00 = load_weight_texel(map_index, base, size, mip);
    let c10 = load_weight_texel(map_index, base + vec2<i32>(1, 0), size, mip);
    let c01 = load_weight_texel(map_index, base + vec2<i32>(0, 1), size, mip);
    let c11 = load_weight_texel(map_index, base + vec2<i32>(1, 1), size, mip);
    return mix(mix(c00, c10, fraction.x), mix(c01, c11, fraction.x), fraction.y);
}

fn sample_weight_map(map_index: u32, uv: vec2<f32>) -> vec4<f32> {
    let source = material.weight_source_sampler[map_index];
    let base_size = vec2<f32>(source.w);
    let lod = legacy_lod(uv, base_size, source.y, source.z);
    if source.x < 0.5 {
        return sample_weight_point(map_index, uv, u32(floor(lod + 0.5)));
    }
    if source.x < 1.5 {
        return sample_weight_bilinear(map_index, uv, u32(floor(lod + 0.5)));
    }
    let low = u32(floor(lod));
    let high = min(low + 1u, u32(source.y));
    return mix(
        sample_weight_bilinear(map_index, uv, low),
        sample_weight_bilinear(map_index, uv, high),
        fract(lod),
    );
}

fn layer_weight(layer: u32, terrain_uv: vec2<f32>) -> f32 {
    let map_index = layer / 4u;
    let channel_index = layer % 4u;
    let weights = sample_weight_map(map_index, terrain_uv);
    return splat_channel(weights, channel_index);
}

// Derivatives must be evaluated before branching on a sampled control weight.
// Packing the tiled UV and selected mip together lets zero-weight layers skip
// their texture loads without putting dpdx/dpdy in non-uniform control flow.
// A hardware anisotropic sampler can never reach the splat array: every layer
// fetch is a manual `textureLoad` so each source keeps its own dimensions
// inside the losslessly padded texture array. Reproduce the same construction
// here. The mip level follows the footprint's short axis while up to
// `render_quality.x` bilinear taps walk its long axis, which is what removes
// the grazing-angle blur and shimmer from the ground. One tap collapses to the
// exact legacy `max(|ddx|, |ddy|)` single-mip footprint.
struct LayerFootprint {
    uv: vec2<f32>,
    // UV distance between anisotropic taps along the footprint's long axis.
    tap_step: vec2<f32>,
    mip: u32,
    taps: u32,
}

fn layer_sample_coordinates(layer: u32, terrain_uv: vec2<f32>) -> LayerFootprint {
    let tiled_uv = terrain_uv * material.layer_tile_scale_mode[layer].xy;
    let source = material.layer_source_size[layer];
    let ddx = dpdx(tiled_uv);
    let ddy = dpdy(tiled_uv);
    let x_length = length(ddx * source.xy);
    let y_length = length(ddy * source.xy);
    let long_axis = select(ddy, ddx, x_length >= y_length);
    let long_length = max(x_length, y_length);
    let short_length = min(x_length, y_length);
    let max_taps = max(material.render_quality.x, 1.0);
    let ratio = clamp(long_length / max(short_length, 0.000001), 1.0, max_taps);
    let taps = u32(clamp(round(ratio), 1.0, max_taps));
    let lod = clamp(
        log2(max(long_length / f32(taps), 1.0)) + source.w,
        0.0,
        source.z,
    );
    return LayerFootprint(
        tiled_uv,
        long_axis / f32(taps),
        u32(floor(lod + 0.5)),
        taps,
    );
}

fn layer_rgb(layer: u32, sample_coordinates: LayerFootprint) -> vec3<f32> {
    var accumulated = vec3<f32>(0.0);
    let first = -0.5 * (f32(sample_coordinates.taps) - 1.0);
    for (var tap = 0u; tap < sample_coordinates.taps; tap += 1u) {
        accumulated += sample_layer_bilinear_mip(
            layer,
            sample_coordinates.uv + sample_coordinates.tap_step * (first + f32(tap)),
            sample_coordinates.mip,
        ).rgb;
    }
    return accumulated / f32(sample_coordinates.taps);
}

fn layer_mode(layer: u32) -> u32 {
    return u32(material.layer_tile_scale_mode[layer].z);
}

fn terrain_render_mode() -> u32 {
    return u32(material.metadata.y + 0.5);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
#ifdef VERTEX_UVS_A
    let terrain_uv = mesh.uv;
#else
    let terrain_uv = vec2<f32>(0.0, 0.0);
#endif
    let debug_view = u32(material.metadata.w + 0.5);
    if debug_view >= 10u && debug_view < 15u {
        let weight = sample_weight_map(debug_view - 10u, terrain_uv);
        // RGBA control maps need all four channels visible in diagnostics.
        // A grayscale alpha inset would complicate deterministic crops, so
        // tint alpha magenta while preserving the original RGB channels.
        let diagnostic_color = min(
            weight.rgb + vec3<f32>(weight.a, 0.0, weight.a),
            vec3<f32>(1.0),
        );
        return vec4<f32>(srgb_to_linear(diagnostic_color), 1.0);
    }
    if debug_view == 2u {
        var total_weight = 0.0;
        let diagnostic_layer_count = u32(material.metadata.x);
        for (var diagnostic_group = 0u;
            diagnostic_group < diagnostic_layer_count;
            diagnostic_group += 4u
        ) {
            let weights = sample_weight_map(diagnostic_group / 4u, terrain_uv);
            let group_end = min(diagnostic_group + 4u, diagnostic_layer_count);
            for (var diagnostic_layer = diagnostic_group;
                diagnostic_layer < group_end;
                diagnostic_layer += 1u
            ) {
                total_weight += splat_channel(weights, diagnostic_layer - diagnostic_group);
            }
        }
        let intensity = clamp(total_weight / 4.0, 0.0, 1.0);
        return vec4<f32>(vec3<f32>(intensity), 1.0);
    }
    if debug_view >= 20u && debug_view < 37u {
        let diagnostic_layer = debug_view - 20u;
        let intensity = layer_weight(diagnostic_layer, terrain_uv);
        return vec4<f32>(vec3<f32>(intensity), 1.0);
    }

    // Exact single-draw reduction of the legacy multipass compositor.
    // Queue order: FirstPass Geometry-100; AddPass Geometry-99;
    // BlendPass Geometry-98. Passes are selected per serialized control-map
    // quartet: one _Control texture and its original _Splat0..3 must remain
    // together. Some quartets end in zero-weight Splat placeholder layers, so
    // the first layer selects the pass mode for the complete quartet.
    //
    // Splat groups are direct weighted RGB sums; AddPass uses Blend One One.
    // Primary Blend groups sum every texture multiplied by its matching
    // R/G/B/A control channel, then apply SrcAlpha OneMinusSrcAlpha using the
    // same channel sum as alpha.
    // Texture alpha is not part of either RGB equation. The original D3D9
    // fixed-function path performs the complete splat/lightmap/fog chain in
    // gamma-encoded texture space; conversion happens only once at the Bevy
    // sRGB framebuffer boundary.
    var framebuffer = vec3<f32>(0.0);
    let layer_count = u32(material.metadata.x);

    // FirstPass followed by Blend One One AddPass draws. The D3D9 normalized
    // render target saturates every completed pass before the next draw reads
    // the destination.
    var has_first_pass = false;
    for (var group_start = 0u; group_start < layer_count; group_start += 4u) {
        if layer_mode(group_start) == 0u {
            var splat_source = vec3<f32>(0.0);
            let group_end = min(group_start + 4u, layer_count);
            // A control map belongs to the whole four-layer pass. Sampling it
            // once here matches Unity's pass binding and avoids repeating the
            // same manual bilinear/trilinear textureLoad sequence for every
            // R/G/B/A channel.
            let weights = sample_weight_map(group_start / 4u, terrain_uv);
            for (var layer = group_start; layer < group_end; layer += 1u) {
                let weight = splat_channel(weights, layer - group_start);
                let sample_coordinates = layer_sample_coordinates(layer, terrain_uv);
                if weight > 0.0 {
                    splat_source += layer_rgb(layer, sample_coordinates) * weight;
                }
            }
            let pass_color = clamp(splat_source, vec3<f32>(0.0), vec3<f32>(1.0));
            if has_first_pass {
                framebuffer = clamp(
                    framebuffer + pass_color,
                    vec3<f32>(0.0),
                    vec3<f32>(1.0),
                );
            } else {
                framebuffer = pass_color;
                has_first_pass = true;
            }
        }
    }
    if debug_view == 3u {
        return vec4<f32>(srgb_to_linear(clamp(framebuffer, vec3<f32>(0.0), vec3<f32>(1.0))), 1.0);
    }

    // BlendPass draws execute after all Splat groups, but retain their original
    // control-map quartet boundaries.
    for (var group_start = 0u; group_start < layer_count; group_start += 4u) {
        if layer_mode(group_start) == 1u {
            var blend_source = vec3<f32>(0.0);
            var blend_alpha = 0.0;
            let group_end = min(group_start + 4u, layer_count);
            let weights = sample_weight_map(group_start / 4u, terrain_uv);
            for (var layer = group_start; layer < group_end; layer += 1u) {
                let weight = splat_channel(weights, layer - group_start);
                let sample_coordinates = layer_sample_coordinates(layer, terrain_uv);
                // The active Realtime-BlendPass is a direct RGBA-weighted sum.
                // Lightmap-BlendPass instead starts at unweighted Splat0 and
                // sequentially lerps G/B/A. Shader-family selection comes
                // from m_RenderMode, not the separate legacy m_UseLightmap
                // field retained in the serialized Terrain component.
                if layer == group_start || weight > 0.0 {
                    let source = layer_rgb(layer, sample_coordinates);
                    if terrain_render_mode() == 1u {
                        blend_source = select(
                            mix(blend_source, source, weight),
                            source,
                            layer == group_start,
                        );
                    } else {
                        blend_source += source * weight;
                    }
                }
                blend_alpha += weight;
            }
            blend_source = clamp(
                blend_source,
                vec3<f32>(0.0),
                vec3<f32>(1.0),
            );
            blend_alpha = clamp(blend_alpha, 0.0, 1.0);
            framebuffer = clamp(
                blend_source * blend_alpha + framebuffer * (1.0 - blend_alpha),
                vec3<f32>(0.0),
                vec3<f32>(1.0),
            );
        }
    }
    if debug_view == 1u {
        return vec4<f32>(srgb_to_linear(clamp(framebuffer, vec3<f32>(0.0), vec3<f32>(1.0))), 1.0);
    }

    let view_position = view.view_from_world * mesh.world_position;
    let eye_depth = max(-view_position.z, 0.0);
    let lightmap = textureSample(
        terrain_lightmap,
        terrain_lightmap_sampler,
        terrain_uv,
    );

    // SplatDatabase::GetMaterial receives Terrain.m_RenderMode directly.
    // Retrobution's Unity 2.5.5 native table maps 0 to Vertexlit, 1 to
    // Lightmap and 2 to Realtime. Every published source terrain saves mode 1
    // while m_UseLightmap remains zero, so treating that separate field as the
    // shader-family selector incorrectly chose Realtime and produced per-quad
    // lighting differences.
    if terrain_render_mode() == 1u {
        framebuffer *= lightmap.rgb;
    } else if terrain_render_mode() == 2u {
        let fade_start = material.metadata.z * 0.8;
        let fade_width = max(material.metadata.z - fade_start, 0.0001);
        let realtime_fade = clamp((eye_depth - fade_start) / fade_width, 0.0, 1.0);
        // The active Retrobution terrain base pass receives the continuously
        // sampled DongColor light after cnPlayerCamera.DefaultAmbience. Using
        // heightfield vertex normals here introduced four-unit dark quads
        // that do not exist in the reference renderer. Close terrain keeps
        // the uniform ambience tint; only the far edge fades to the baked
        // terrain lightmap.
        let vertex_pass = mix(
            material.ambience_light.rgb,
            lightmap.rgb,
            realtime_fade,
        );
        framebuffer = clamp(
            framebuffer * vertex_pass,
            vec3<f32>(0.0),
            vec3<f32>(1.0),
        );
    } else {
        framebuffer *= vec3<f32>(LEGACY_TERRAIN_VERTEX_AMBIENT);
    }

    // Fog is the global fixed-function stage and therefore remains applied
    // after the terrain surface/light chain.
    // The original shader receives the fixed-function eye-space fog
    // coordinate and compiles ARB_fog_exp2. It is view depth, not radial
    // camera distance: factor = exp(-(density * depth)^2).
    let density_depth = material.ambience_fog.a * eye_depth;
    let fog_amount = select(
        0.0,
        1.0 - exp(-(density_depth * density_depth)),
        material.ambience_fog.a > 0.0,
    );
    framebuffer = mix(framebuffer, material.ambience_fog.rgb, fog_amount);

    return vec4<f32>(srgb_to_linear(framebuffer), 1.0);
}
