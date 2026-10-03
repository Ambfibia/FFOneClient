#import bevy_pbr::{
    forward_io::Vertex,
    mesh_functions,
    mesh_view_bindings::{globals, view},
    pbr_functions,
    skinning,
    view_transformations::position_world_to_clip,
}
#ifdef DISTANCE_FOG
#import bevy_pbr::mesh_view_bindings::fog
#endif

struct LegacyModelMaterialUniform {
    base_color: vec4<f32>,
    tint_color: vec4<f32>,
    ambient_color: vec4<f32>,
    emission: vec4<f32>,
    rim_color: vec4<f32>,
    rim_effect: vec4<f32>,
    custom_effect: vec4<f32>,
    uv_scale_offset: vec4<f32>,
    uv_pivot_rotation: vec4<f32>,
    uv_animation: vec4<f32>,
    light_direction_family: vec4<f32>,
    alpha_effect: vec4<f32>,
    legacy_effect: vec4<f32>,
}

// Retrobution `mainData` RenderSettings path 24.
const LEGACY_GLOBAL_AMBIENT: f32 = 0.2;

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> material: LegacyModelMaterialUniform;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var base_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var base_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var toon_ramp: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var toon_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var bump_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var bump_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(7) var effect_map: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(8) var effect_sampler: sampler;

// Bevy's stock VertexOutput has no room for the three UV sets produced by the
// compiled FusionEffect vertex program. Keep the standard locations intact so
// skinning and visibility-range variants remain compatible, then append the
// exact legacy varyings at otherwise-unused locations.
struct LegacyVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
#ifdef VERTEX_UVS_A
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_UVS_B
    @location(3) uv_b: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(4) world_tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(5) color: vec4<f32>,
#endif
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    @location(6) @interpolate(flat) instance_index: u32,
#endif
#ifdef VISIBILITY_RANGE_DITHER
    @location(7) @interpolate(flat) visibility_range_dither: i32,
#endif
    @location(8) fusion_bump_uv: vec2<f32>,
    @location(9) fusion_shader_uv: vec2<f32>,
    @location(10) legacy_primary: vec4<f32>,
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

// Native comic art direction. Work in the same encoded palette as the model
// textures: broad, flat cel bands and cool colored shadows without print dots.
const COMIC_INK: vec3<f32> = vec3<f32>(0.075, 0.045, 0.20);

fn comic_surface(
    base: vec3<f32>,
    normal: vec3<f32>,
    light: vec3<f32>,
) -> vec3<f32> {
    let ndotl = dot(normal, light);
    let band_aa = max(fwidth(ndotl), 0.015);
    let shade = 1.0 - smoothstep(0.25 - band_aa, 0.25 + band_aa, ndotl);
    let deep = 1.0 - smoothstep(-0.50 - band_aa, -0.50 + band_aa, ndotl);
    let mid_color = base * vec3<f32>(0.94, 0.86, 1.0);
    let deep_color = base * vec3<f32>(0.72, 0.55, 0.88) + COMIC_INK * 0.12;
    let color = mix(base, mid_color, shade);
    return mix(color, deep_color, deep);
}

fn apply_legacy_fixed_function_fog(
    input_color: vec4<f32>,
    world_position: vec4<f32>,
    range_fade: f32,
) -> vec4<f32> {
    // Primary compiles the inherited fixed-function fog stage as
    // ARB_fog_exp2 and feeds it eye-space depth (not radial distance). The
    // camera's DistanceFog uniform carries the exact sampled world color and
    // `fogDepth * 0.005` density; recompute the legacy stage in encoded color
    // space before Bevy writes the linear render target.
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
    let view_position = view.view_from_world * world_position;
    let eye_depth = max(-view_position.z, 0.0);
    let density_depth = fog_density * eye_depth;
    let fogged = material.legacy_effect.w > 0.5 && fog_density > 0.0;
    let fog_amount = select(
        0.0,
        1.0 - exp(-(density_depth * density_depth)),
        fogged,
    );
    // The native runtime culls world objects far inside the exponential fog
    // curve, so Bevy's ordered visibility-range dither used to erode props out
    // of otherwise clear air and made that short draw distance obvious. Drive
    // the same object-range transition into the fog stage as well: an object
    // reaching the end of its range is fully hazed before its last dithered
    // texels disappear.
    let haze_amount = max(fog_amount, select(0.0, range_fade, fogged));
    let encoded_surface = linear_to_srgb(input_color.rgb);
    // ShaderLab additive passes are fogged toward black. Using the scene fog
    // color here adds that color under SrcAlpha/One blending; in infected-zone
    // ambience this stacked a red contribution over every blue EPbarrier wave.
    let encoded_fog = select(
        linear_to_srgb(fog_color),
        vec3<f32>(0.0),
        material.legacy_effect.w > 1.5,
    );
    return vec4<f32>(
        srgb_to_linear(mix(encoded_surface, encoded_fog, haze_amount)),
        input_color.a,
    );
}

fn rotate_about_pivot(uv: vec2<f32>, pivot: vec2<f32>, degrees: f32) -> vec2<f32> {
    let angle = radians(degrees);
    let sine = sin(angle);
    let cosine = cos(angle);
    let centered = uv - pivot;
    return vec2<f32>(
        centered.x * cosine - centered.y * sine,
        centered.x * sine + centered.y * cosine,
    ) + pivot;
}

#ifdef VISIBILITY_RANGE_DITHER
// Native static-world ranges use one packed logical-object center for every
// renderer in a prefab. Do not call Bevy's per-Mesh3d LOD-table helper here:
// these GLBs retain baked vertices/zero origins and their individual AABBs can
// be far apart even though they form one object.
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
fn vertex(vertex: Vertex) -> LegacyVertexOutput {
    var out: LegacyVertexOutput;
    var expanded_position = vertex.position;

    // Both toon BASE programs displace before the skin/world transform. The
    // OUTLINE pass adds `_Outline` separately in legacy_model_outline.wgsl.
#ifdef VERTEX_NORMALS
    let family = material.light_direction_family.w;
    if (family > 0.5 && family < 1.5) || (family > 2.5 && family < 3.5) {
        expanded_position += vertex.normal * material.legacy_effect.x;
    }
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

    var world_normal = vec3<f32>(0.0, 1.0, 0.0);
#ifdef VERTEX_NORMALS
#ifdef SKINNED
    world_normal = skinning::skin_normals(world_from_local, vertex.normal);
#else
    world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        vertex.instance_index,
    );
#endif
#endif
    world_normal = normalize(world_normal);
    out.world_normal = world_normal;

    var transformed_uv = vec2<f32>(0.0, 0.0);
#ifdef VERTEX_UVS_A
    var animated_offset = material.uv_scale_offset.zw;
    var animated_rotation = material.uv_pivot_rotation.z;
#ifdef LEGACY_GPU_UV_ANIMATION
    var animation_time = globals.time - material.uv_animation.w;
    if animation_time < 0.0 {
        animation_time += material.uv_pivot_rotation.w;
    }
    animated_offset += material.uv_animation.xy * animation_time;
    animated_rotation += material.uv_animation.z * animation_time;
#endif
    transformed_uv = vertex.uv * material.uv_scale_offset.xy + animated_offset;
    transformed_uv = rotate_about_pivot(
        transformed_uv,
        material.uv_pivot_rotation.xy,
        animated_rotation,
    );
    out.uv = transformed_uv;
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

    // Unity's `_Time.x` is elapsed seconds / 20. The compiled shader scrolls
    // MainTex and BumpMap in opposite Y directions. ShaderMap is driven by the
    // same scroll plus the clip transform of vec4(normal, 1), not by a rotated
    // pivot or a sampled distortion vector.
    out.fusion_bump_uv = transformed_uv;
    out.fusion_shader_uv = transformed_uv;
    if material.light_direction_family.w > 1.5 && material.light_direction_family.w < 2.5 {
        let legacy_time_x = globals.time / 20.0;
        let scroll = material.legacy_effect.y * legacy_time_x;
#ifdef VERTEX_UVS_A
        out.uv = vec2<f32>(transformed_uv.x, transformed_uv.y + scroll);
#endif
        out.fusion_bump_uv = vec2<f32>(transformed_uv.x, transformed_uv.y - scroll);
        let transformed_normal = view.clip_from_world * vec4<f32>(world_normal, 1.0);
        out.fusion_shader_uv = vec2<f32>(scroll + transformed_normal.x, transformed_normal.y);
    }

    // `primary` from Lighting On / the Fusion vertex program is vertex-lit:
    // diffuse * max(N.L, 0) + emission. Family 0 and FusionEffect consume it
    // through the fixed-function path. Toon consumes the compiled ramp
    // coordinate instead.
    let light_direction = normalize(-material.light_direction_family.xyz);
    let ndotl = dot(world_normal, light_direction);
    let diffuse_light = max(ndotl, 0.0);
    var primary_alpha = material.base_color.a;
    if material.light_direction_family.w > 1.5 && material.light_direction_family.w < 2.5 {
        // Exact D3D9 FusionEffect vertex program:
        //   mov r0.xyz, _Emission
        //   mov r0.w, _Color        // scalar source selects _Color.r
        //   mad oD0, NdotL, _Color, r0
        primary_alpha = material.base_color.r + diffuse_light * material.base_color.a;
    }
    var primary_rgb = material.base_color.rgb * diffuse_light + material.emission.rgb;
    if (material.light_direction_family.w > 2.5 && material.light_direction_family.w < 3.5)
        || (material.light_direction_family.w > 3.5 && material.light_direction_family.w < 4.5)
        || (material.light_direction_family.w > 5.5 && material.light_direction_family.w < 6.5)
    {
        // Fixed-function Material Ambient is modulated by the global
        // RenderSettings ambient light. Adding `_AmbColor` directly made
        // additive ES740 materials five times too bright.
        primary_rgb += material.ambient_color.rgb * LEGACY_GLOBAL_AMBIENT;
    }
    out.legacy_primary = vec4<f32>(primary_rgb, primary_alpha);
    if material.light_direction_family.w > 8.5 && material.light_direction_family.w < 9.5 {
        // ColorMaterial AmbientAndDiffuse replaces the two material colors
        // with the vertex color; emission remains an independent term.
        var ambient_diffuse = vec4<f32>(1.0);
#ifdef VERTEX_COLORS
        ambient_diffuse = vertex.color;
#endif
        out.legacy_primary = vec4<f32>(
            ambient_diffuse.rgb * (diffuse_light + LEGACY_GLOBAL_AMBIENT) + material.emission.rgb,
            ambient_diffuse.a,
        );
    }

    if material.light_direction_family.w > 4.5 && material.light_direction_family.w < 5.5 {
        // The legacy particle family is explicitly `Lighting Off` and binds
        // the optional mesh color stream as fixed-function `primary`. Unity
        // supplies white when the mesh has no color stream.
        var particle_vertex_color = vec4<f32>(1.0);
#ifdef VERTEX_COLORS
        particle_vertex_color = vertex.color;
#endif
        out.legacy_primary = material.tint_color * particle_vertex_color;
    }

    if material.light_direction_family.w > 7.5 && material.light_direction_family.w < 8.5 {
        // The ES865/ES866 hologram program is explicitly Lighting Off.
        var hologram_vertex_color = vec4<f32>(1.0);
#ifdef VERTEX_COLORS
        hologram_vertex_color = vertex.color;
#endif
        out.legacy_primary = hologram_vertex_color;
    }

    if material.light_direction_family.w > 0.5 && material.light_direction_family.w < 1.5 {
        let view_direction = normalize(view.world_position - out.world_position.xyz);
        let ndotv = dot(world_normal, view_direction);
        let view_against_light = dot(view_direction, -light_direction);
        let shaped = clamp(ndotl + (1.0 - ndotv) * view_against_light, 0.0, 1.0);
        let ramp_coordinate = clamp(ndotl + shaped, 0.01, 0.99);
        out.legacy_primary = vec4<f32>(
            ramp_coordinate,
            ndotl,
            0.0,
            material.base_color.a,
        );
    }

    if material.light_direction_family.w > 6.5 && material.light_direction_family.w < 7.5 {
        // Exact Fusion Matter vertex color:
        // _Color * (1 + (saturate(N.L) - 1) * _ShadowStrength) + 0.3.
        let shadowed = 1.0 + (max(ndotl, 0.0) - 1.0) * material.rim_effect.w;
        out.legacy_primary = vec4<f32>(
            material.base_color.rgb * shadowed + vec3<f32>(0.3),
            1.0,
        );
    }

    return out;
}

@fragment
fn fragment(mesh: LegacyVertexOutput) -> @location(0) vec4<f32> {
    var range_fade = 0.0;
#ifdef VISIBILITY_RANGE_DITHER
    // The vertex path already publishes Bevy's object-range transition, but
    // the legacy fragment program used to ignore it and therefore popped at
    // `end_margin.end`. Apply the same ordered discard as Bevy's PBR path so
    // complete legacy objects dissolve smoothly into the distance fog. The
    // matching continuous level also drives the fog stage below; the dither
    // alone erodes an object without ever tinting it toward the area haze.
    range_fade = clamp(f32(mesh.visibility_range_dither) / 16.0, 0.0, 1.0);
    pbr_functions::visibility_range_dither(mesh.position, mesh.visibility_range_dither);
#endif
#ifdef VERTEX_UVS_A
    let main_uv = mesh.uv;
#else
    let main_uv = vec2<f32>(0.0, 0.0);
#endif
    let base_sample = textureSample(base_texture, base_sampler, main_uv);
    let family = material.light_direction_family.w;
    var surface = vec4<f32>(0.0, 0.0, 0.0, base_sample.a);

    if family > 0.5 && family < 1.5 {
        // Shared native character style for players, NPCs, Nanos and toon
        // attachments. Preserve texture/tint and the authored alpha contract;
        // replace the old RGB ramp with comic lighting from skinned normals.
        let encoded_base = clamp(
            linear_to_srgb(base_sample.rgb) * material.base_color.rgb * 2.0,
            vec3<f32>(0.0),
            vec3<f32>(1.0),
        );
        let ramp = textureSample(
            toon_ramp,
            toon_sampler,
            vec2<f32>(mesh.legacy_primary.x, 0.5),
        );
        let normal = normalize(mesh.world_normal);
        let light_direction = normalize(-material.light_direction_family.xyz);
        var encoded_surface = comic_surface(
            encoded_base, normal, light_direction,
        ) * material.custom_effect.x;
        let view_direction = normalize(view.world_position - mesh.world_position.xyz);
        let rim = pow(
            1.0 - clamp(dot(normal, view_direction), 0.0, 1.0),
            material.rim_effect.x,
        );
        if material.rim_effect.z > 0.5 && material.rim_effect.z < 1.5 {
            // A restrained, hard rim lets area lighting color the lit edge
            // without washing the texture and colored shadows into white.
            let lit_gate = smoothstep(0.20, 0.30, dot(normal, light_direction));
            let rim_aa = max(fwidth(rim), 0.02);
            let rim_band = smoothstep(0.55 - rim_aa, 0.55 + rim_aa, rim);
            encoded_surface += material.rim_color.rgb * rim_band
                * min(material.rim_effect.y, 1.0) * lit_gate * 0.08;
        } else if material.rim_effect.z > 1.5 && material.rim_effect.z < 2.5 {
            // Explicit special-rim programs retain their serialized color and
            // properties instead of following the area sky.
            encoded_surface += material.rim_color.rgb * rim * material.rim_effect.y;
        }
        surface = vec4<f32>(
            srgb_to_linear(clamp(encoded_surface, vec3<f32>(0.0), vec3<f32>(1.0))),
            clamp(
                base_sample.a
                    * material.base_color.a
                    * ramp.a
                    * 2.0
                    * material.custom_effect.y,
                0.0,
                1.0,
            ),
        );
    } else if family > 1.5 && family < 2.5 {
        // FusionEffect's three fixed-function stages. BumpMap and ShaderMap are
        // additive color stages; neither one is a normal-map distortion or a
        // multiplicative light map. Unity's legacy D3D9 gamma pipeline performed
        // these combiners on encoded RGB values. Re-enter that domain for the
        // fixed-function arithmetic, then return linear RGB to Bevy.
        var fusion_rgb = clamp(
            linear_to_srgb(base_sample.rgb) * mesh.legacy_primary.rgb * 2.0,
            vec3<f32>(0.0),
            vec3<f32>(1.0),
        );
        let bump = textureSample(bump_texture, bump_sampler, mesh.fusion_bump_uv);
        fusion_rgb = clamp(fusion_rgb + bump.rgb, vec3<f32>(0.0), vec3<f32>(1.0));
        let shader_map = textureSample(effect_map, effect_sampler, mesh.fusion_shader_uv);
        surface = vec4<f32>(
            srgb_to_linear(
                clamp(fusion_rgb + shader_map.rgb, vec3<f32>(0.0), vec3<f32>(1.0)),
            ),
            clamp(base_sample.a * mesh.legacy_primary.a * 2.0, 0.0, 1.0),
        );
    } else if family > 6.5 && family < 7.5 {
        // Retrobution's replacement Fusion Matter packs Fusion, bubbles and
        // electricity into MainTex R/G/B and evaluates three independently
        // scrolling samples.
        // The compiled program uses Unity `_Time.x` (elapsed seconds / 20),
        // not `_Time.y`/raw elapsed seconds.
        let legacy_time_x = globals.time / 20.0;
        let fusion_sample = textureSample(
            base_texture,
            base_sampler,
            main_uv + vec2<f32>(0.0, legacy_time_x * 0.25),
        );
        let bubbles_sample = textureSample(
            base_texture,
            base_sampler,
            main_uv + vec2<f32>(0.0, legacy_time_x * -0.15),
        );
        let electricity_uv = main_uv
            + vec2<f32>(0.0, legacy_time_x * -0.35)
            + linear_to_srgb(fusion_sample.rgb).xy * 0.65;
        let electricity_sample = textureSample(
            base_texture,
            base_sampler,
            electricity_uv,
        );
        let masks_fusion = linear_to_srgb(fusion_sample.rgb);
        let masks_bubbles = linear_to_srgb(bubbles_sample.rgb);
        let masks_electricity = linear_to_srgb(electricity_sample.rgb);

        var encoded_matter = masks_fusion.r * material.tint_color.rgb
            + masks_bubbles.g * material.ambient_color.rgb * 0.5;
        // The compiled Retrobution program derives a scalar from the
        // normalized normal's vertical component, then uses (.6,.45,.24) as
        // the highlight *color*. Treating that color as a light direction
        // produced broad, camera-inconsistent bright/dark polygons that made
        // sound replacement meshes look as if their normals were broken.
        let view_normal = normalize(
            (view.view_from_world * vec4<f32>(normalize(mesh.world_normal), 0.0)).xyz,
        );
        let highlight = clamp(
            view_normal.y * 2.0 - 0.6,
            0.0,
            1.0,
        );
        encoded_matter += highlight * vec3<f32>(0.6, 0.45, 0.24);
        encoded_matter *= mesh.legacy_primary.rgb * 1.25;

        let view_direction = normalize(view.world_position - mesh.world_position.xyz);
        let rim = pow(
            1.0 - clamp(dot(normalize(mesh.world_normal), view_direction), 0.0, 1.0),
            2.0,
        );
        let encoded_electricity =
            masks_electricity.b * material.emission.rgb
            + rim * material.rim_color.a * material.rim_color.rgb;
        surface = vec4<f32>(
            srgb_to_linear(clamp(
                encoded_matter + encoded_electricity,
                vec3<f32>(0.0),
                vec3<f32>(1.0),
            )),
            material.base_color.a,
        );
    } else if family > 7.5 && family < 8.5 {
        // Retrobution ES865/ES866 `HologramSolid_Additive` uses one packed
        // mask. BASE interpolates ColorA/ColorB from alpha. OVERLAYS samples
        // the scrolling R/G/B channels and adds their authored colors.
        let mask_alpha = base_sample.a;
        let encoded_base = mask_alpha * material.base_color.rgb
            + (1.0 - mask_alpha) * material.tint_color.rgb;
        if material.custom_effect.z < 0.5 {
            surface = vec4<f32>(
                srgb_to_linear(clamp(encoded_base, vec3<f32>(0.0), vec3<f32>(1.0))),
                clamp(
                    material.custom_effect.x
                        * material.custom_effect.y
                        * mesh.legacy_primary.a,
                    0.0,
                    1.0,
                ),
            );
        } else {
            let legacy_time_y = globals.time;
            let red_mask = textureSample(
                base_texture,
                base_sampler,
                vec2<f32>(main_uv.x * 1.25 + legacy_time_y * 0.2, main_uv.y),
            ).r;
            let green_mask = textureSample(
                base_texture,
                base_sampler,
                main_uv + vec2<f32>(0.35, 0.7) * legacy_time_y,
            ).g;
            let blue_mask = textureSample(
                base_texture,
                base_sampler,
                vec2<f32>(main_uv.x * 0.65 + legacy_time_y * 0.1, main_uv.y),
            ).b;
            let overlay_base = vec3<f32>(1.0)
                + mask_alpha * (material.base_color.rgb - vec3<f32>(1.0));
            let encoded_overlay = overlay_base * (
                material.ambient_color.rgb * red_mask * material.rim_effect.x
                + material.emission.rgb * green_mask * material.rim_effect.y
                + material.rim_color.rgb * blue_mask * material.rim_effect.w
            );
            surface = vec4<f32>(
                srgb_to_linear(clamp(encoded_overlay, vec3<f32>(0.0), vec3<f32>(1.0))),
                clamp(
                    ((mask_alpha - 1.0) * material.custom_effect.w + 1.0)
                        * material.custom_effect.x
                        * mesh.legacy_primary.a,
                    0.0,
                    1.0,
                ),
            );
        }
    } else if family > 4.5 && family < 5.5 {
        // Dual-texture-card particle path:
        //   constantColor(_TintColor) * primary(vertex color)
        //   texture * previous DOUBLE
        // The published optional texture resolves to ShaderLab's white
        // default when `_MainTex` is unassigned.
        surface = vec4<f32>(
            srgb_to_linear(
                clamp(
                    linear_to_srgb(base_sample.rgb) * mesh.legacy_primary.rgb * 2.0,
                    vec3<f32>(0.0),
                    vec3<f32>(1.0),
                ),
            ),
            clamp(base_sample.a * mesh.legacy_primary.a * 2.0, 0.0, 1.0),
        );
    } else {
        var fixed_function_primary = mesh.legacy_primary.rgb;
        var fixed_function_rgb_scale = 2.0;
        var fixed_function_alpha = base_sample.a;
        if material.light_direction_family.w > 8.5 && material.light_direction_family.w < 9.5 {
            fixed_function_alpha *= mesh.legacy_primary.a;
        }
        if material.legacy_effect.z > 0.5 {
            // `normal_glow_*` does not use `_BumpMap` as a normal map. Its
            // first fixed-function stage is:
            //   constantColor (0.5, 0.5, 0.5)
            //   combine constant lerp(texture) previous
            // so the texture alpha blends vertex lighting toward neutral
            // gray before MainTex is multiplied by that result.
            let glow_mask = textureSample(bump_texture, bump_sampler, main_uv);
            fixed_function_primary = mix(
                fixed_function_primary,
                vec3<f32>(0.5),
                glow_mask.a,
            );
            // Its following source stage is exactly
            // `Combine texture * previous, texture * primary`: unlike the
            // ordinary `normal` family it has no `DOUBLE` modifier, and its
            // explicit alpha term retains the primary/material alpha.
            fixed_function_rgb_scale = 1.0;
            fixed_function_alpha = base_sample.a * mesh.legacy_primary.a;
        }
        // D3D9 performed the fixed-function combiners on encoded texture RGB.
        // Bevy samples sRGB PNGs as linear, so re-enter that domain for the
        // legacy multiply and convert the final framebuffer value once.
        surface = vec4<f32>(
            srgb_to_linear(
                clamp(
                    linear_to_srgb(base_sample.rgb)
                        * fixed_function_primary
                        * fixed_function_rgb_scale,
                    vec3<f32>(0.0),
                    vec3<f32>(1.0),
                ),
            ),
            fixed_function_alpha,
        );
    }

    if material.alpha_effect.w > 0.5 {
        // Explicit native cel extension. Preserve the source alpha, blend,
        // depth and ordering contracts while adopting the character palette.
        let encoded_base = clamp(linear_to_srgb(base_sample.rgb) * material.base_color.rgb * 2.0,
            vec3<f32>(0.0), vec3<f32>(1.0));
        surface = vec4<f32>(srgb_to_linear(comic_surface(encoded_base,
            normalize(mesh.world_normal), normalize(-material.light_direction_family.xyz))), surface.a);
        if material.alpha_effect.w > 1.5 {
            // Inset eyes need ink on their visible surface: an inverted hull
            // is hidden by the surrounding head. Skinning already transformed
            // these normals. Derivative AA keeps the border stable at distance.
            let to_camera = normalize(view.world_position - mesh.world_position.xyz);
            let facing = dot(normalize(mesh.world_normal), to_camera);
            let aa = max(fwidth(facing), 0.015);
            let white = smoothstep(0.48 - aa, 0.48 + aa, facing);
            surface = vec4<f32>(surface.rgb * white, surface.a);
        }
    }
    if material.alpha_effect.y > 0.5 {
        if material.alpha_effect.z > 0.5 {
            // ShaderLab `Greater`: equality fails (not GEqual).
            if surface.a <= material.alpha_effect.x {
                discard;
            }
        } else if surface.a < material.alpha_effect.x {
            // ShaderLab `GEqual`: equality passes.
            discard;
        }
    }

    return apply_legacy_fixed_function_fog(surface, mesh.world_position, range_fade);
}
