#import bevy_pbr::{
    forward_io::Vertex,
    mesh_functions,
    mesh_view_bindings::{globals, view},
    pbr_functions,
    view_transformations::position_world_to_clip,
}

struct LegacyWaterMaterialUniform {
    color: vec4<f32>,
    wave_speed: vec4<f32>,
    refr_color: vec4<f32>,
    horizon_color: vec4<f32>,
    foam_color: vec4<f32>,
    water_params: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> material: LegacyWaterMaterialUniform;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var reflective_gradient: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var reflective_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var bump_map: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var bump_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var fresnel_map: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var fresnel_sampler: sampler;

struct LegacyWaterVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) wave_uv_a: vec2<f32>,
    @location(3) wave_uv_b: vec2<f32>,
#ifdef VISIBILITY_RANGE_DITHER
    @location(4) @interpolate(flat) visibility_range_dither: i32,
#endif
}

#ifdef VISIBILITY_RANGE_DITHER
// Keep ffWater/ffPoison on the same logical-object range contract as the
// ordinary legacy passes. The packed MeshTag is shared by every renderer in a
// source prefab, so disconnected water pieces do not pop independently.
fn native_world_object_range_dither(instance_index: u32) -> i32 {
    let tag = mesh_functions::get_tag(instance_index);
    let end_code = tag & 0x0fu;
    let y_code = (tag >> 4u) & 0x03ffu;
    let z_code = (tag >> 14u) & 0x01ffu;
    let x_code = tag >> 23u;
    let center = vec3<f32>(
        -8192.0 + (f32(x_code) + 0.5) * 16.0,
        -18000.0 + (f32(y_code) + 0.5) * 20.0,
        (f32(z_code) + 0.5) * 16.0,
    );
    let end = 75.0 + f32(end_code) / 15.0 * 265.0;
    let distance = length(view.world_position.xyz - center);
    return i32(round(clamp((distance - (end - 32.0)) / 32.0, 0.0, 1.0) * 16.0));
}
#endif

fn linear_to_srgb(rgb: vec3<f32>) -> vec3<f32> {
    let low = rgb * 12.92;
    let high = 1.055 * pow(max(rgb, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(low, high, rgb > vec3<f32>(0.0031308));
}

fn srgb_to_linear(rgb: vec3<f32>) -> vec3<f32> {
    let low = rgb / 12.92;
    let high = pow((max(rgb, vec3<f32>(0.0)) + 0.055) / 1.055, vec3<f32>(2.4));
    return select(low, high, rgb > vec3<f32>(0.04045));
}

@vertex
fn vertex(vertex: Vertex) -> LegacyWaterVertexOutput {
    var out: LegacyWaterVertexOutput;
    let world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
    let world_position = mesh_functions::mesh_position_local_to_world(
        world_from_local,
        vec4<f32>(vertex.position, 1.0),
    );
    out.position = position_world_to_clip(world_position.xyz);
    out.world_position = world_position.xyz;

    var world_normal = vec3<f32>(0.0, 1.0, 0.0);
#ifdef VERTEX_NORMALS
    world_normal = normalize(mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        vertex.instance_index,
    ));
#endif
    out.world_normal = world_normal;
#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither = native_world_object_range_dither(vertex.instance_index);
#endif

    // Decompiled ffWater reflective vertex program. Unity `_Time.x` is
    // elapsed seconds / 20; the two bump reads use different XY/XZ swizzles.
    let legacy_time_x = globals.time / 20.0;
    let wave_scale = material.water_params.x;
    let phase = material.wave_speed * legacy_time_x
        + vec4<f32>(
            vertex.position.x,
            vertex.position.z,
            vertex.position.x,
            vertex.position.z,
        );
    let scaled_phase = phase * wave_scale;
    out.wave_uv_a = scaled_phase.xy * vec2<f32>(0.4, 0.45);
    out.wave_uv_b = vec2<f32>(scaled_phase.w, scaled_phase.z);
    return out;
}

@fragment
fn fragment(mesh: LegacyWaterVertexOutput) -> @location(0) vec4<f32> {
#ifdef VISIBILITY_RANGE_DITHER
    pbr_functions::visibility_range_dither(mesh.position, mesh.visibility_range_dither);
#endif
    // Exact reflective pass: two scrolling bump samples form the view-facing
    // lookup coordinate for `_ReflectiveColor`. `_Fresnel` is retained in the
    // material contract for the other source tiers but is not sampled by this
    // audited high-end pass.
    let bump_a = textureSample(bump_map, bump_sampler, mesh.wave_uv_a).rgb;
    let bump_b = textureSample(bump_map, bump_sampler, mesh.wave_uv_b).rgb;
    let summed_bump = bump_a + bump_b - vec3<f32>(1.0);
    let view_to_camera = normalize(view.world_position - mesh.world_position);
    let legacy_view = vec3<f32>(
        view_to_camera.x,
        view_to_camera.z,
        view_to_camera.y,
    );
    let reflection_coordinate = dot(legacy_view, summed_bump);
    let reflection = textureSample(
        reflective_gradient,
        reflective_sampler,
        vec2<f32>(reflection_coordinate, 0.5),
    );
    // The source D3D9 pass performs this combiner in encoded RGB. The
    // reflective gradient is a Bevy sRGB texture, so re-enter that domain for
    // the exact arithmetic and convert the framebuffer result back to linear.
    let encoded_reflection = linear_to_srgb(reflection.rgb);
    let horizon_mix = mix(
        encoded_reflection,
        material.horizon_color.rgb,
        reflection.a,
    );
    let foam_mix = mix(material.foam_color.rgb, horizon_mix, mesh.world_normal.y);
    let alpha = reflection.a * material.horizon_color.a * 2.0;
    return vec4<f32>(
        srgb_to_linear(foam_mix * material.color.rgb),
        alpha * material.color.a,
    );
}
