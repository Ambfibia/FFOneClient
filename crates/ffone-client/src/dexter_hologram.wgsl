#import bevy_ui::ui_vertex_output::UiVertexOutput

struct DexterHologramUniform {
    // rgb = Graphics.DrawTexture color; a = original HologramColor alpha.
    color: vec4<f32>,
    // xy = original _MainTex offset; zw are reserved.
    offset: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> material: DexterHologramUniform;
@group(1) @binding(1)
var scene_texture: texture_2d<f32>;
@group(1) @binding(2)
var scene_sampler: sampler;
@group(1) @binding(3)
var decal_texture: texture_2d<f32>;
@group(1) @binding(4)
var decal_sampler: sampler;

fn linear_to_srgb(value: vec3<f32>) -> vec3<f32> {
    let low = value * 12.92;
    let high = 1.055 * pow(value, vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055);
    return select(low, high, value > vec3<f32>(0.0031308));
}

fn srgb_to_linear(value: vec3<f32>) -> vec3<f32> {
    let low = value / 12.92;
    let high = pow((value + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return select(low, high, value > vec3<f32>(0.04045));
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    // Exact fixed-function Hologram pass from sharedassets0 shader 728:
    // SetTexture _MainTex { Combine texture }
    // SetTexture _DecalTex { Combine texture * previous }
    let scene = textureSample(scene_texture, scene_sampler, in.uv + material.offset.xy);
    let scanline = textureSample(decal_texture, decal_sampler, in.uv);
    // Both sRGB textures arrive linearized by wgpu, while the legacy
    // fixed-function combiner multiplied their encoded samples. Recreate that
    // operation and convert the finished RGB back for Bevy's sRGB target.
    let encoded = linear_to_srgb(scene.rgb) * linear_to_srgb(scanline.rgb) * material.color.rgb;
    // ES740 is authored entirely from RGB-only additive passes (`ColorMask
    // RGB`). It contributes to the hologram color wherever it overlaps the
    // actors, but it must not expand the render target's alpha silhouette.
    // Unity's Graphics.DrawTexture blend therefore keeps scene.a as coverage;
    // promoting RGB-only pixels to alpha exposes the computer's rectangular
    // support meshes instead of the character-shaped reference hologram.
    return vec4<f32>(
        srgb_to_linear(encoded),
        scene.a * scanline.a * material.color.a,
    );
}
