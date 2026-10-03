#import bevy_pbr::{
    forward_io::{Vertex, VertexOutput},
    mesh_functions,
    mesh_view_bindings::view,
    pbr_functions,
    skinning,
    view_transformations::position_world_to_clip,
}
#ifdef DISTANCE_FOG
#import bevy_pbr::mesh_view_bindings::fog
#endif

struct LegacyOutlineUniform {
    color: vec4<f32>,
    width_fat: vec4<f32>,
    fog_params: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> outline: LegacyOutlineUniform;

#ifdef VISIBILITY_RANGE_DITHER
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

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    var expanded_position = vertex.position;
#ifdef VERTEX_NORMALS
    expanded_position += vertex.normal * (outline.width_fat.x + outline.width_fat.y);
#endif

#ifdef SKINNED
    let world_from_local = skinning::skin_model(
        vertex.joint_indices,
        vertex.joint_weights,
        vertex.instance_index,
    );
#else
    let world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
#endif

    out.world_position = mesh_functions::mesh_position_local_to_world(
        world_from_local,
        vec4<f32>(expanded_position, 1.0),
    );
    out.position = position_world_to_clip(out.world_position.xyz);

#ifdef VERTEX_NORMALS
    // Retain the existing skinned hull/pass and its depth, but keep the ink
    // line readable without letting it balloon in close-up character views.
    // Fat-factor deformation belongs to the base shape, not the line width.
    let base_world = mesh_functions::mesh_position_local_to_world(
        world_from_local,
        vec4<f32>(vertex.position + vertex.normal * outline.width_fat.y, 1.0),
    );
    let base_clip = position_world_to_clip(base_world.xyz);
    if outline.width_fat.x > 0.0 && base_clip.w > 0.0 && out.position.w > 0.0 {
        let pixel_delta = (out.position.xy / out.position.w - base_clip.xy / base_clip.w)
            * view.viewport.zw * 0.5;
        let pixel_width = length(pixel_delta);
        // Do not manufacture a direction for normals facing the camera.
        let line_width = clamp(pixel_width, 1.1, 2.0);
        let ink_delta = pixel_delta * (line_width / max(pixel_width, 0.001));
        out.position = vec4<f32>(
            (base_clip.xy / base_clip.w + ink_delta * 2.0 / view.viewport.zw) * out.position.w,
            out.position.zw,
        );
    }
#endif

#ifdef VERTEX_NORMALS
#ifdef SKINNED
    out.world_normal = skinning::skin_normals(world_from_local, vertex.normal);
#else
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        vertex.instance_index,
    );
#endif
#endif
#ifdef VERTEX_UVS_A
    out.uv = vertex.uv;
#endif
#ifdef VERTEX_UVS_B
    out.uv_b = vertex.uv_b;
#endif
#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
        world_from_local,
        vertex.tangent,
        vertex.instance_index,
    );
#endif
#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    out.instance_index = vertex.instance_index;
#endif
#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither = native_world_object_range_dither(vertex.instance_index);
#endif
    return out;
}

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

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var range_fade = 0.0;
#ifdef VISIBILITY_RANGE_DITHER
    // Keep the expanded toon outline in exactly the same object-level fade as
    // its source surface. Fading only the base pass leaves dark silhouettes
    // hanging in the fog after the object itself has disappeared.
    range_fade = clamp(f32(mesh.visibility_range_dither) / 16.0, 0.0, 1.0);
    pbr_functions::visibility_range_dither(mesh.position, mesh.visibility_range_dither);
#endif
    // Bevy 0.19 binds the view `fog` uniform only for a camera that owns a
    // `DistanceFog`. The UI preview and portrait cameras render these same
    // legacy materials without one, so reading `fog` unconditionally dropped
    // the whole pipeline. Resolve the stage through neutral locals instead:
    // zero density reproduces exactly the unfogged view this code already saw.
#ifdef DISTANCE_FOG
    let fog_density = fog.be.x;
    let fog_color = fog.base_color.rgb;
#else
    let fog_density = 0.0;
    let fog_color = vec3<f32>(0.0);
#endif
    let view_position = view.view_from_world * mesh.world_position;
    let eye_depth = max(-view_position.z, 0.0);
    let density_depth = fog_density * eye_depth;
    let fogged = outline.fog_params.x > 0.5 && fog_density > 0.0;
    let fog_amount = select(
        0.0,
        1.0 - exp(-(density_depth * density_depth)),
        fogged,
    );
    // Ink is the highest-contrast thing a distant object owns, so it has to
    // reach the area haze on exactly the object-range curve the base surface
    // uses. Otherwise the silhouette stays crisp while the surface fades and
    // the short native draw distance reads as a hard cut.
    let haze_amount = max(fog_amount, select(0.0, range_fade, fogged));
    return vec4<f32>(
        srgb_to_linear(mix(
            // Indigo-black ink follows the comic shadow palette; authored
            // colored outlines retain their brighter channel values.
            max(linear_to_srgb(outline.color.rgb), vec3<f32>(0.075, 0.045, 0.20)),
            linear_to_srgb(fog_color),
            haze_amount,
        )),
        outline.color.a,
    );
}
