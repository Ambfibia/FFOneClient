#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct LegacyGlowSettings {
    filter_color: vec4<f32>,
    glow_tint: vec4<f32>,
    // x = glowIntensity (1.8), y = blurSpread (1.0).
    parameters: vec4<f32>,
}

@group(0) @binding(0)
var source_texture: texture_2d<f32>;
@group(0) @binding(1)
var source_sampler: sampler;
@group(0) @binding(2)
var<uniform> settings: LegacyGlowSettings;

fn linear_to_srgb(value: vec3<f32>) -> vec3<f32> {
    let low = value * 12.92;
    let high = 1.055 * pow(max(value, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(low, high, value > vec3<f32>(0.0031308));
}

fn srgb_to_linear(value: vec3<f32>) -> vec3<f32> {
    let low = value / 12.92;
    let high = pow((value + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return select(low, high, value > vec3<f32>(0.04045));
}

fn sample_encoded(texture: texture_2d<f32>, texture_sampler: sampler, uv: vec2<f32>) -> vec4<f32> {
    let sampled = textureSample(texture, texture_sampler, uv);
    return vec4<f32>(linear_to_srgb(sampled.rgb), sampled.a);
}

@fragment
fn downsample(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let dimensions = vec2<f32>(textureDimensions(source_texture));
    let texel = 1.0 / dimensions;
    let average = (
        sample_encoded(source_texture, source_sampler, in.uv + vec2<f32>(-texel.x, -texel.y))
        + sample_encoded(source_texture, source_sampler, in.uv + vec2<f32>(texel.x, -texel.y))
        + sample_encoded(source_texture, source_sampler, in.uv + vec2<f32>(texel.x, texel.y))
        + sample_encoded(source_texture, source_sampler, in.uv + vec2<f32>(-texel.x, texel.y))
    ) * 0.25;
    // Exact Hidden/Glow Downsample program:
    // average four samples, tint RGB, multiply by one minus averaged alpha.
    let glow = average.rgb * settings.glow_tint.rgb * (1.0 - average.a);
    return vec4<f32>(srgb_to_linear(glow), 0.0);
}

fn cone_tap(in: FullscreenVertexOutput, iteration: f32) -> vec4<f32> {
    let dimensions = vec2<f32>(textureDimensions(source_texture));
    let offset = (0.5 + iteration * settings.parameters.y) / dimensions;
    let samples = array<vec3<f32>, 4>(
        sample_encoded(source_texture, source_sampler, in.uv + vec2<f32>(-offset.x, -offset.y)).rgb,
        sample_encoded(source_texture, source_sampler, in.uv + vec2<f32>(offset.x, -offset.y)).rgb,
        sample_encoded(source_texture, source_sampler, in.uv + vec2<f32>(offset.x, offset.y)).rgb,
        sample_encoded(source_texture, source_sampler, in.uv + vec2<f32>(-offset.x, offset.y)).rgb,
    );
    // Native correction: a blur must preserve a constant field. The source
    // combiner's three unweighted additions amplified it by 3.45^4 and
    // erased glass, summon silhouettes and foliage under white halos.
    // Apply intensity once at composition, independently of blur iterations.
    let encoded = (samples[0] + samples[1] + samples[2] + samples[3]) * 0.25;
    return vec4<f32>(srgb_to_linear(encoded), 0.0);
}

@fragment
fn blur_0(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    return cone_tap(in, 0.0);
}

@fragment
fn blur_1(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    return cone_tap(in, 1.0);
}

@fragment
fn blur_2(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    return cone_tap(in, 2.0);
}

@fragment
fn blur_3(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    return cone_tap(in, 3.0);
}

@group(0) @binding(3)
var glow_texture: texture_2d<f32>;
@group(0) @binding(4)
var glow_sampler: sampler;

@fragment
fn composite(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let source = sample_encoded(source_texture, source_sampler, in.uv);
    let glow = sample_encoded(glow_texture, glow_sampler, in.uv).rgb;
    // FilterCompose followed by GlowCompose (Blend One One, DOUBLE).
    let encoded = source.rgb * settings.filter_color.rgb
        + glow * 2.0 * max(settings.parameters.x, 0.0);
    return vec4<f32>(srgb_to_linear(encoded), source.a);
}
