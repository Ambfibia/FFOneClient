//! Exact-data native renderer for tutorial effect nodes.

pub(super) mod native_catalog;
mod vehicle_trails;

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::{
    asset::{
        AssetPath, LoadState, RecursiveDependencyLoadState, RenderAssetUsages, embedded_asset,
        embedded_path,
    },
    audio::Volume,
    camera::visibility::NoFrustumCulling,
    gltf::{Gltf, GltfAssetLabel},
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    mesh::{Indices, MeshVertexBufferLayoutRef, PrimitiveTopology},
    pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin},
    prelude::*,
    reflect::TypePath,
    render::render_resource::{
        AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, ColorWrites,
        Extent3d, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
        TextureDimension, TextureFormat,
    },
    shader::ShaderRef,
    transform::TransformSystems,
};
use ffone_runtime_contracts::{
    TutorialEffectCatalogEntry, TutorialEffectClosureFile, TutorialUnityObjectProof,
};
use serde_json::Value as JsonValue;

use super::{
    TutorialEffectAttachmentProof, TutorialEffectPlacement, TutorialEffectRuntime,
    TutorialEffectRuntimeIssue, TutorialNativeClosureBlockerReason, TutorialOniMotionPhase,
    TutorialOniProjectileState, TutorialProjectileVisualReadiness, ValidatedEffect,
    process_tutorial_effect_runtime,
};
use crate::legacy_model_material::LegacyModelMaterial;
use crate::{
    avatar_action::{LegacyAvatarActionState, LegacyVisualClip},
    coordinates::unity_to_native_vector,
    gameplay_audio::{GameplaySfxAudio, LEGACY_SPATIAL_SCALE, RetrobutionAudioMix},
    movement::{LegacyOrbitCamera, advance_native_xorshift32},
    semantic_audio::{NativeAudioCatalog, NativeAudioCategory},
    tutorial_actors::TutorialActorRegistry,
    tutorial_player_rig_runtime::{TutorialPlayerWeaponAttachment, TutorialSelectedPlayerRig},
    world::LEGACY_WORLD_CAMERA_FAR_NATIVE,
};

#[cfg(test)]
#[path = "tutorial_native_effects/material_sharing_tests.rs"]
mod material_sharing_tests;

#[cfg(test)]
mod tests;

mod constants;
mod textures;
mod codec;
mod types;
mod materials;
mod animation;
mod commands;
mod operations_compile_projectile_plan;
mod operations_prepare_native_sword_trails;
mod containers;
mod assets;
mod models;
mod input;
mod state;
mod audio_native_impact_sfx;
mod output;
mod systems;
mod entities;

use constants::{
    RETROBUTION_SWORD_TRAIL_LENGTH, RETROBUTION_SWORD_TRAIL_MAX_SECONDS,
    RETROBUTION_SWORD_TRAIL_TINT, STREAMED_WORLD_EFFECT_FAR_MARGIN,
    MAX_STREAMED_WORLD_PARTICLES
};
use textures::{
    RETROBUTION_SWORD_TRAIL_TEXTURE, ExactTexture, material_and_texture, texture_format_layout,
    rgba_level_size, bc1_color_palette, bc3_alpha_palette, bc3_color_palette, flip_rgba_rows,
    configure_native_sword_trail_texture, texture_handle
};
use codec::{
    NATIVE_SPAWN_WORK_PER_FRAME, STREAMED_WORLD_NATIVE_SPAWN_WORK_PER_FRAME,
    STREAMED_WORLD_PARTICLE_DESPAWNS_PER_FRAME, decode_exact_texture
};
#[cfg(test)]
use codec::{decode_argb4444_level, decode_bc3_level, decode_bc1_level};
pub(super) use types::{
    NativeClosureBlocker, NativeCompileResult, NativeEffectPlan, NativeProjectilePlan,
    NativeLinearImpactPlan, NativeLinearProjectileMotion
};
use types::{
    EmitterPlan, EmitterGeneration, ScriptKey, CurveKey, TrailPlan,
    TutorialParticlePipelineKey, NativeVisualAssets, NativeProjectileVisualPrewarm,
    NativeProjectileVisualPrewarmSurface, NativeEffectPreloadCache,
    NativeParticleRandomStream, NativeRoot, NativeDetachedOwner, NativeEffectRoot,
    NativeEffectBillboard, NativeEmitter, NativeParticle, NativeProjectile,
    NativeProjectileMotion, NativeTrail, NativeSwordTrailBound, NativeSwordTrailEdge,
    NativeSwordTrail
};
use materials::{
    LegacyParticleRenderMode, MaterialAnimationPlan, MaterialCurvePlan, MaterialAnimatedProperty,
    ParticleBlendMode, compile_material_animation, particle_blend_mode, exact_shader_name,
    TutorialParticleUniform, TutorialParticleMaterial, ParticleMaterialCacheKey,
    NativeAnimatedParticleMaterial, mesh_effect_render_queue_sort_bias, apply_material_animation,
    material_surface_matches_node, shared_trail_material, particle_material_is_animated,
    shared_particle_material, particle_material_animation_sample
};
pub use materials::TutorialEffectMaterialAnimation;
pub(super) use materials::compile_material_animation_component;
use animation::{
    AnimationCurve, mesh_effect_repeats_standard_animation,
    projectile_mesh_repeats_standard_animation, PARTICLE_COLOR_ANIMATION_STEPS
};
pub(super) use commands::{NativePreloadRequest, NativeSpawnRequest};
use operations_compile_projectile_plan::{
    take_native_spawn_batch, purge_stale_native_spawn_requests, invalid, array, number, integer,
    boolean, vector2, named_color, argb4444_level_size, compressed_level_size,
    reconcile_named_native_root_liveness, streamed_effect_ownership, is_descendant_of,
    mark_native_effect_billboards, orient_native_effect_billboards
};
#[cfg(test)]
use operations_compile_projectile_plan::native_vector3;
pub(super) use operations_compile_projectile_plan::{
    compile_effect_plan, compile_world_emitter_plan, compile_projectile_plan
};
use operations_prepare_native_sword_trails::{
    prewarm_native_projectile_visuals, prepare_native_sword_trails, native_sword_attack_active,
    push_native_sword_trail_edge, particle_quad, legacy_emission_due,
    persistent_streamed_landmark_particle, streamed_particle_spawn_count,
    world_effect_is_within_camera_distance, native_emitted_particle_position, simulate_particles,
    cleanup_expired_particles, particle_billboard_rotation, particle_uv_scale_offset,
    particle_color, particle_size, warhead_sphere_hit, native_impact_playback_settings,
    push_interpolated_trail_points, cleanup_effect_roots, cleanup_effect_meshes,
    cleanup_stream_owned_native_effects
};
#[cfg(test)]
use operations_prepare_native_sword_trails::admit_particle_despawn;
use containers::{object, unity_vector_slerp, legacy_particle_initial_state_in_unity};
use assets::{path_id, game_object_path, unique_path_type};
use models::{
    exact_effect_mesh_scene, exact_projectile_mesh_scene, NativeMeshEffectPlayback,
    NativeMeshEffectLifetime, NativePreparedMeshEffectSurface,
    prepare_native_mesh_effect_materials, animate_native_mesh_effect_materials,
    play_native_mesh_effect_animations, native_sword_trail_mesh, trail_mesh
};
use input::{parse_colors, parse_curve, parse_curve_with_modes, collect_named};
use state::{
    NativePreloadHandleState, native_preload_handle_state, legacy_particle_initial_state,
    script_state
};
use audio_native_impact_sfx::NativeImpactSfx;
pub(super) use output::install;
use systems::{
    apply_native_requests, sync_native_preload_completion, update_native_sword_trails,
    update_emitters, update_animated_particle_materials, update_projectiles, update_trails
};
#[cfg(test)]
use systems::advance_native_sword_trail;
use entities::{
    spawn_effect, spawn_projectile, spawn_linear_projectile, spawn_world_impact,
    despawn_native_instance
};
