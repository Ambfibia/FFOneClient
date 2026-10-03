#import bevy_pbr::{
    forward_io::Vertex,
    mesh_functions,
    mesh_view_bindings::view,
    view_transformations::position_world_to_clip,
}
#ifdef DISTANCE_FOG
#import bevy_pbr::mesh_view_bindings::fog
#endif

struct TutorialParticleUniform {
    tint: vec4<f32>,
    uv_scale_offset: vec4<f32>,
    legacy_gamma_accumulation_gain: f32,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: TutorialParticleUniform;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var main_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var main_sampler: sampler;

struct TutorialParticleVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) world_position: vec4<f32>,
}

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

@vertex
fn vertex(vertex: Vertex) -> TutorialParticleVertexOutput {
    var out: TutorialParticleVertexOutput;
    let world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
    let world_position = mesh_functions::mesh_position_local_to_world(
        world_from_local,
        vec4<f32>(vertex.position, 1.0),
    );
    out.position = position_world_to_clip(world_position.xyz);
    out.world_position = world_position;
#ifdef VERTEX_UVS_A
    out.uv = vertex.uv;
#else
    out.uv = vec2<f32>(0.0, 0.0);
#endif
#ifdef VERTEX_COLORS
    out.color = vertex.color;
#else
    out.color = vec4<f32>(1.0);
#endif
    return out;
}

@fragment
fn fragment(mesh: TutorialParticleVertexOutput) -> @location(0) vec4<f32> {
    // Unity fixed-function particle shader: tint * primary, then
    // texture * previous DOUBLE; AlphaTest Greater .01. Unity 2.5's D3D9
    // combiner evaluated those stages on gamma-encoded values. The PNG is an
    // sRGB texture in Bevy, so explicitly re-enter that domain before doing
    // the legacy arithmetic and return linear RGB to the sRGB attachment.
    let uv = mesh.uv * material.uv_scale_offset.xy
        + material.uv_scale_offset.zw;
    let sample = textureSample(main_texture, main_sampler, uv);
    let encoded_rgb = clamp(
        linear_to_srgb(sample.rgb) * material.tint.rgb * mesh.color.rgb * 2.0,
        vec3<f32>(0.0),
        vec3<f32>(1.0),
    );
    let alpha = clamp(sample.a * material.tint.a * mesh.color.a * 2.0, 0.0, 1.0);
    if alpha <= 0.01 {
        discard;
    }
    // Every published clean-primary particle program in this pipeline
    // inherits the fixed-function `Fog { Color (0,0,0,0) }` stage. Unity
    // compiles that stage as ARB_fog_exp2 over eye-space depth. This is
    // particularly important for the authored Past-world `galadriel` cards:
    // their conforming scale and width curve can produce a 173 m quad, which
    // must fade toward black instead of entering the frustum as an opaque
    // rock/leaf-shaped sheet.
    // Bevy 0.19 binds the view `fog` uniform only for a camera that owns a
    // `DistanceFog`. Preview and portrait cameras render these same effects
    // without one, so reading `fog` unconditionally dropped the whole
    // pipeline. Zero density reproduces exactly the unfogged view this code
    // already saw.
#ifdef DISTANCE_FOG
    let fog_density = fog.be.x;
#else
    let fog_density = 0.0;
#endif
    let view_position = view.view_from_world * mesh.world_position;
    let eye_depth = max(-view_position.z, 0.0);
    let density_depth = fog_density * eye_depth;
    let fog_amount = select(
        0.0,
        1.0 - exp(-(density_depth * density_depth)),
        fog_density > 0.0,
    );
    let fogged_encoded_rgb = mix(encoded_rgb, vec3<f32>(0.0), fog_amount);
    // D3D9 Retrobution accumulated legacy particle cards in gamma color
    // space. The per-effect gain compensates for Bevy's linear sRGB-target
    // blending where several authored cards deliberately overlap.
    let accumulated_rgb = min(
        srgb_to_linear(fogged_encoded_rgb) * material.legacy_gamma_accumulation_gain,
        vec3<f32>(1.0),
    );
    return vec4<f32>(accumulated_rgb, alpha);
}
