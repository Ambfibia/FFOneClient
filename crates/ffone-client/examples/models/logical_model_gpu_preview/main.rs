//! GPU acceptance harness for one natively published logical FusionFall model.
//!
//! This executable reads only the published GLB/PNG assets below `--asset-root`.
//! It is intentionally separate from the game runtime and never opens Unity
//! bundles or an `.ffclient` project.

use std::{
    env, fs,
    fs::OpenOptions,
    io::Write,
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Component, Path, PathBuf},
    process::{self, ExitCode},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use bevy::{
    asset::{AssetPlugin, LoadState, RecursiveDependencyLoadState},
    camera::primitives::Aabb,
    gltf::{Gltf, GltfAssetLabel},
    log::{
        BoxedLayer, Level,
        tracing::{self, Subscriber},
        tracing_subscriber::Layer,
    },
    mesh::{
        VertexAttributeValues,
        skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
    },
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    window::PresentMode,
    world_serialization::WorldInstanceReady,
};
use ffone_client::legacy_model_material::{
    ExactMipChainApplied, LegacyGltfTextureBinding, LegacyMaterialApplied,
    LegacyMaterialMetadataError, LegacyMaterialPassCompanion, LegacyModelMaterial,
    LegacyModelMaterialPlugin, LegacyNpcTextureRole, LegacyOutlineSource, LegacyPassKind,
    PendingLegacyModelMaterial, legacy_npc_texture_role, load_legacy_main_texture_replacement,
    load_legacy_main_texture_replacement_with_sampler,
};
use ffone_client::{
    character_scene::{
        LegacyCharacterSceneBlock, LegacyCharacterSceneStatus, SpawnedLegacyCharacterScene,
        spawn_legacy_character_scene,
    },
    coordinates::LegacyCharacterRootPolicy,
    legacy_material_animation::{
        LegacyMaterialAnimationClip, apply_legacy_material_float,
        parse_legacy_material_animation_clips, sample_legacy_float_curve,
    },
};
use ffone_skinned_model::{
    AutomatedGpuStatus, GPU_EVIDENCE_SCHEMA, GPU_RENDER_PROFILE, GpuAnimationEvidence,
    GpuModelIdentity, GpuRuntimeEvidence, GpuScreenshotEvidence, LogicalModelGpuEvidence,
    LogicalModelGpuFacts, LogicalModelRuntimeSmoke, NativeSampler, PublishedMipPolicy,
    RUNTIME_SMOKE_SCHEMA, SourceOutlineMode, VisualParityClaim, gpu_evidence_relative_paths,
    gpu_model_facts_from_glb,
};
use image::{ColorType, ImageEncoder, codecs::png::PngEncoder};
use serde_json::json;
use sha2::{Digest, Sha256};
fn main() -> ExitCode {
    let parsed = match PreviewConfig::parse(std::env::args().skip(1)) {
        Ok(ParseOutcome::Help) => {
            println!("{HELP}");
            return ExitCode::SUCCESS;
        }
        Ok(ParseOutcome::Run(config)) => config,
        Err(error) => return cli_failure(&error),
    };
    let config = match parsed.validate_and_resolve() {
        Ok(config) => config,
        Err(error) => return cli_failure(&error),
    };

    let shared = SharedReport::new(&config);
    let mut app = build_app(config.clone(), shared.clone());
    let run_result = catch_unwind(AssertUnwindSafe(|| app.run()));

    let app_exit_ok = match run_result {
        Ok(app_exit) => app_exit.is_success(),
        Err(payload) => {
            shared.fail(format!("Bevy/render panic: {}", panic_message(payload)));
            false
        }
    };
    if shared.is_pending() {
        let reason = if app_exit_ok {
            "preview window closed before the acceptance gate completed"
        } else {
            "Bevy exited before the acceptance gate completed"
        };
        shared.fail(reason.to_owned());
    }

    if let Some(report_path) = config.report.as_deref() {
        let report_json = shared.to_json(&config);
        if let Err(error) = write_json_report(report_path, &report_json) {
            shared.fail(error);
        }
    }
    println!("{}", shared.to_json(&config));
    if app_exit_ok && shared.is_success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests;

mod constants;
mod validation;
mod output;
mod operations;
mod types_preview_config;
mod types_shared_report;
mod state;
mod input;
mod assets;
mod models;
mod textures;
mod materials;
mod systems;
mod animation;
mod codec;

use constants::{
    HELP, DEFAULT_MAX_FRAMES, DEFAULT_TIMEOUT_SECS, PIPELINE_WARMUP_FRAMES,
    SCREENSHOT_COLOR_TOLERANCE
};
use validation::{POST_CAPTURE_ERROR_GRACE_FRAMES, validate_json_path};
use output::{write_json_report, save_preview_screenshot, atomic_write_new};
use operations::{
    cli_failure, panic_message, sha256_hex, set_once, build_app,
    surface_has_preview_named_ancestor, setup_preview, mark_scene_ready,
    character_scene_block_message, persist_success_outputs, pixel_differs_from,
    screenshot_foreground, gate_limit, readiness_reason, transformed_aabb,
    include_combined_point
};
use types_preview_config::{
    PreviewConfig, CharacterKind, EvidenceConfig, CharacterSceneHandles, PreviewCamera,
    PreviewCameraView, BlankCameraRetry, SceneStats, Bounds3
};
use types_shared_report::{TransformSnapshot, PreviewReport, SharedReport};
use state::{
    CharacterRuntimeConfig, PreviewOutlineMode, sync_character_runtime_status,
    is_exact_character_runtime_chain, RuntimeState, fail_runtime, succeed_runtime,
    CharacterRuntimeReport
};
use input::{ParseOutcome, collect_rendered_world_bounds};
use assets::slash_path;
use models::{validate_relative_glb, ModelHandles, skinned_vertex_world};
use textures::{
    validate_relative_png_asset, validate_png_path, PreviewNpcTextureSlot,
    PreviewNpcTextureOverrideBound, preview_npc_texture_role,
    bind_preview_npc_texture_overrides, CapturedPng, expected_exact_mip_marker,
    linear_rgba_to_srgb8, texture_override_status
};
use materials::{
    PreviewMaterialAnimationBase, apply_preview_material_animation_curves,
    CapturedRenderErrors, RenderErrorCaptureLayer, RenderLogVisitor, is_render_error,
    render_error_capture_layer
};
use systems::{apply_preview_visibility_overrides, evaluate_acceptance_gate};
use animation::{PreparedAnimation, prepare_selected_animation, start_selected_animation};
use codec::CameraFrame;
