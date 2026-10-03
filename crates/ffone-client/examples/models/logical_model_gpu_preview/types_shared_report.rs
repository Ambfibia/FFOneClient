use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct TransformSnapshot {
    pub(super) translation: Vec3,
    pub(super) rotation: Quat,
    pub(super) scale: Vec3,
}

impl TransformSnapshot {
    pub(super) fn from_local(transform: Transform) -> Self {
        Self {
            translation: transform.translation,
            rotation: transform.rotation,
            scale: transform.scale,
        }
    }

    pub(super) fn from_global(transform: &GlobalTransform) -> Option<Self> {
        let (scale, rotation, translation) = transform.to_scale_rotation_translation();
        (translation.is_finite()
            && rotation.is_finite()
            && rotation.length_squared() > f32::EPSILON
            && scale.is_finite())
        .then_some(Self {
            translation,
            rotation,
            scale,
        })
    }

    pub(super) fn as_json(self) -> serde_json::Value {
        json!({
            "translation": [self.translation.x, self.translation.y, self.translation.z],
            "rotationXyzw": [self.rotation.x, self.rotation.y, self.rotation.z, self.rotation.w],
            "scale": [self.scale.x, self.scale.y, self.scale.z],
        })
    }
}

#[derive(Debug, Clone)]
pub(super) struct PreviewReport {
    pub(super) status: &'static str,
    pub(super) error: Option<String>,
    pub(super) frames: u64,
    pub(super) elapsed_seconds: f64,
    pub(super) scene_ready: bool,
    pub(super) gltf_loaded_with_dependencies: bool,
    pub(super) meshes: u64,
    pub(super) skinned_meshes: u64,
    pub(super) animation_players: u64,
    pub(super) animation_graph_handles: u64,
    pub(super) sampled_players: u64,
    pub(super) animation_evaluation_frames: u64,
    pub(super) animations: usize,
    pub(super) exact_animation_names: Vec<String>,
    pub(super) selected_animation: Option<String>,
    pub(super) materials_applied: u64,
    pub(super) exact_mip_markers: u64,
    pub(super) assigned_texture_bindings: u64,
    pub(super) exact_mip_chains: u64,
    pub(super) exact_mip_levels: u64,
    pub(super) material_errors: u64,
    pub(super) main_texture_overrides: u64,
    pub(super) sub_texture_overrides: u64,
    pub(super) texture_overrides_loaded: bool,
    pub(super) shader_errors: usize,
    pub(super) bounds: Option<Bounds3>,
    pub(super) camera_view: PreviewCameraView,
    pub(super) capture_issued: bool,
    pub(super) blank_capture_attempts: u32,
    pub(super) screenshot_saved: bool,
    pub(super) foreground_pixels: u64,
    pub(super) foreground_coverage: f64,
    pub(super) character_runtime: Option<CharacterRuntimeReport>,
}

impl Default for PreviewReport {
    fn default() -> Self {
        Self {
            status: "pending",
            error: None,
            frames: 0,
            elapsed_seconds: 0.0,
            scene_ready: false,
            gltf_loaded_with_dependencies: false,
            meshes: 0,
            skinned_meshes: 0,
            animation_players: 0,
            animation_graph_handles: 0,
            sampled_players: 0,
            animation_evaluation_frames: 0,
            animations: 0,
            exact_animation_names: Vec::new(),
            selected_animation: None,
            materials_applied: 0,
            exact_mip_markers: 0,
            assigned_texture_bindings: 0,
            exact_mip_chains: 0,
            exact_mip_levels: 0,
            material_errors: 0,
            main_texture_overrides: 0,
            sub_texture_overrides: 0,
            texture_overrides_loaded: true,
            shader_errors: 0,
            bounds: None,
            camera_view: PreviewCameraView::Primary,
            capture_issued: false,
            blank_capture_attempts: 0,
            screenshot_saved: false,
            foreground_pixels: 0,
            foreground_coverage: 0.0,
            character_runtime: None,
        }
    }
}

#[derive(Clone, Resource)]
pub(super) struct SharedReport(pub(super) Arc<Mutex<PreviewReport>>);

impl SharedReport {
    pub(super) fn new(_config: &PreviewConfig) -> Self {
        Self(Arc::new(Mutex::new(PreviewReport::default())))
    }

    pub(super) fn update(&self, update: impl FnOnce(&mut PreviewReport)) {
        let mut report = self
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        update(&mut report);
    }

    pub(super) fn fail(&self, error: String) {
        self.update(|report| {
            if report.status != "success" {
                report.status = "error";
                report.error = Some(error);
            }
        });
    }

    pub(super) fn succeed(&self) {
        self.update(|report| {
            report.status = "success";
            report.error = None;
        });
    }

    pub(super) fn is_pending(&self) -> bool {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .status
            == "pending"
    }

    pub(super) fn is_success(&self) -> bool {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .status
            == "success"
    }

    pub(super) fn to_json(&self, config: &PreviewConfig) -> serde_json::Value {
        let report = self
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        json!({
            "schema": RUNTIME_SMOKE_SCHEMA,
            "status": report.status,
            "model": config.model,
            "screenshot": config.screenshot,
            "report": config.report,
            "frames": report.frames,
            "elapsedSeconds": (report.elapsed_seconds * 1000.0).round() / 1000.0,
            "sceneReady": report.scene_ready,
            "gltfLoadedWithDependencies": report.gltf_loaded_with_dependencies,
            "meshes": report.meshes,
            "skinnedMeshes": report.skinned_meshes,
            "animationPlayers": report.animation_players,
            "animationGraphHandles": report.animation_graph_handles,
            "sampledPlayers": report.sampled_players,
            "sampleMidpoint": config.sample_midpoint || config.evidence.is_some(),
            "animationEvaluationFrames": report.animation_evaluation_frames,
            "animations": report.animations,
            "exactAnimationNames": report.exact_animation_names,
            "selectedAnimation": report.selected_animation,
            "materialsApplied": report.materials_applied,
            "exactMipMarkers": report.exact_mip_markers,
            "assignedTextureBindings": report.assigned_texture_bindings,
            "exactMipChains": report.exact_mip_chains,
            "exactMipLevels": report.exact_mip_levels,
            "materialErrors": report.material_errors,
            "xdtTextureOverrides": {
                "main": config.main_texture,
                "sub": config.sub_texture,
                "mainMaterial": config.main_material,
                "subMaterial": config.sub_material,
                "mainSampler": config.main_sampler,
                "subSampler": config.sub_sampler,
                "mainMaterials": report.main_texture_overrides,
                "subMaterials": report.sub_texture_overrides,
                "mainStatus": texture_override_status(
                    config.main_texture.as_deref(),
                    report.main_texture_overrides,
                ),
                "subStatus": texture_override_status(
                    config.sub_texture.as_deref(),
                    report.sub_texture_overrides,
                ),
                "loaded": report.texture_overrides_loaded,
            },
            "shaderErrors": report.shader_errors,
            "bounds": report.bounds.map(Bounds3::as_json),
            "cameraView": report.camera_view.as_str(),
            "blankCameraRetry": config.blank_camera_retry.as_str(),
            "captureIssued": report.capture_issued,
            "blankCaptureAttempts": report.blank_capture_attempts,
            "screenshotSaved": report.screenshot_saved,
            "foregroundPixels": report.foreground_pixels,
            "foregroundCoverage": report.foreground_coverage,
            "characterRuntime": report.character_runtime.as_ref().map(CharacterRuntimeReport::as_json),
            "error": report.error,
        })
    }
}
