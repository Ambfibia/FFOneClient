use super::*;

pub(super) fn cli_failure(error: &str) -> ExitCode {
    println!(
        "{}",
        json!({
            "status": "error",
            "stage": "cli",
            "error": error,
        })
    );
    eprintln!("{error}\n\n{HELP}");
    ExitCode::from(2)
}

pub(super) fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_owned()
    }
}

pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn set_once<T>(slot: &mut Option<T>, value: T, flag: &str) -> Result<(), String> {
    if slot.replace(value).is_some() {
        Err(format!("{flag} may only be supplied once"))
    } else {
        Ok(())
    }
}

pub(super) fn build_app(config: PreviewConfig, shared: SharedReport) -> App {
    let asset_file_path = config.asset_root.to_string_lossy().into_owned();
    let initial_camera_view = config.camera_view;
    let mut app = App::new();
    app.insert_resource(config)
        .insert_resource(shared)
        .insert_resource(RuntimeState::new(initial_camera_view))
        .insert_resource(ClearColor(Color::srgb(0.055, 0.075, 0.105)))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.82, 0.88, 1.0),
            brightness: 650.0,
            ..default()
        })
        .add_plugins(
            DefaultPlugins
                .set(bevy::log::LogPlugin {
                    filter: "wgpu=warn,naga=warn,bevy_render=info,info".to_owned(),
                    custom_layer: render_error_capture_layer,
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: asset_file_path,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne native logical-model GPU acceptance".to_owned(),
                        resolution: (960, 960).into(),
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(LegacyModelMaterialPlugin)
        .add_systems(Startup, setup_preview)
        .add_systems(
            Update,
            (
                sync_character_runtime_status,
                prepare_selected_animation,
                start_selected_animation,
                bind_preview_npc_texture_overrides,
                apply_preview_material_animation_curves,
                apply_preview_visibility_overrides,
                evaluate_acceptance_gate,
            )
                .chain(),
        );
    app
}

pub(super) fn surface_has_preview_named_ancestor(
    mut entity: Entity,
    target_name: &str,
    parents: &Query<&ChildOf>,
    names: &Query<&Name>,
) -> bool {
    loop {
        if names
            .get(entity)
            .is_ok_and(|name| name.as_str() == target_name)
        {
            return true;
        }
        let Ok(parent) = parents.get(entity) else {
            return false;
        };
        entity = parent.parent();
    }
}

pub(super) fn setup_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<PreviewConfig>,
) {
    let gltf: Handle<Gltf> = asset_server.load(config.model.clone());
    let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset(config.model.clone()));
    commands.insert_resource(ModelHandles { gltf });
    if let Some(character) = config.character.as_ref() {
        let gameplay_root = commands
            .spawn((
                Name::new("character runtime acceptance gameplay root"),
                Transform::IDENTITY,
                Visibility::Inherited,
            ))
            .id();
        let SpawnedLegacyCharacterScene {
            visual_container,
            scene: scene_entity,
        } = spawn_legacy_character_scene(
            &mut commands,
            gameplay_root,
            scene,
            character.true_root.clone(),
            character.policy(),
        );
        commands.insert_resource(CharacterSceneHandles {
            gameplay_root,
            visual_container,
            scene: scene_entity,
        });
    } else {
        commands
            .spawn(WorldAssetRoot(scene))
            .observe(mark_scene_ready);
    }

    commands.spawn((
        PreviewCamera,
        Camera3d::default(),
        Transform::from_xyz(2.0, 1.5, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.94, 0.84),
            illuminance: 11_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.75, -0.55, -0.15)),
    ));
}

pub(super) fn mark_scene_ready(_event: On<WorldInstanceReady>, mut state: ResMut<RuntimeState>) {
    state.scene_ready = true;
}

pub(super) fn character_scene_block_message(block: &LegacyCharacterSceneBlock) -> String {
    match block {
        LegacyCharacterSceneBlock::MissingNamedRoot { expected } => {
            format!("exact character root {expected:?} is missing after WorldInstanceReady")
        }
        LegacyCharacterSceneBlock::MultipleNamedRoots { expected, count } => format!(
            "exact character root {expected:?} is ambiguous after WorldInstanceReady: {count} matches"
        ),
        LegacyCharacterSceneBlock::InvalidSceneWrapper { expected, wrapper } => format!(
            "exact character root {expected:?} has a missing or non-identity scene wrapper {wrapper:?}"
        ),
        LegacyCharacterSceneBlock::InvalidNamedRoot { expected, error } => {
            format!(
                "exact character root {expected:?} is invalid after WorldInstanceReady: {error:?}"
            )
        }
    }
}

pub(super) fn persist_success_outputs(
    config: &PreviewConfig,
    state: &RuntimeState,
    stats: SceneStats,
) -> Result<(), String> {
    let captured = state
        .captured_png
        .as_ref()
        .ok_or_else(|| "GPU success has no captured PNG bytes".to_owned())?;
    let Some(evidence_config) = config.evidence.as_ref() else {
        fs::write(&config.screenshot, &captured.bytes).map_err(|error| {
            format!(
                "cannot save diagnostic screenshot {}: {error}",
                config.screenshot.display()
            )
        })?;
        return Ok(());
    };

    let current_glb = fs::read(&evidence_config.model_path).map_err(|error| {
        format!(
            "cannot re-read evidence GLB {}: {error}",
            evidence_config.model_path.display()
        )
    })?;
    let current_length = u64::try_from(current_glb.len())
        .map_err(|_| "evidence GLB byte length exceeds u64".to_owned())?;
    let current_sha256 = sha256_hex(&current_glb);
    let current_facts = gpu_model_facts_from_glb(&current_glb)
        .map_err(|error| format!("cannot re-decode evidence GLB: {error}"))?;
    if current_length != evidence_config.glb_byte_length
        || current_sha256 != evidence_config.glb_sha256
        || current_facts != evidence_config.facts
    {
        return Err("candidate GLB changed while GPU evidence was running".to_owned());
    }

    let png_length = u64::try_from(captured.bytes.len())
        .map_err(|_| "GPU PNG byte length exceeds u64".to_owned())?;
    let evidence = LogicalModelGpuEvidence {
        schema: GPU_EVIDENCE_SCHEMA.to_owned(),
        status: AutomatedGpuStatus::Passed,
        render_profile: GPU_RENDER_PROFILE.to_owned(),
        visual_parity: VisualParityClaim::NotAsserted,
        model: GpuModelIdentity {
            relative_glb: config.model.clone(),
            true_name: evidence_config.facts.true_name.clone(),
            glb_byte_length: evidence_config.glb_byte_length,
            glb_sha256: evidence_config.glb_sha256.clone(),
        },
        animation: GpuAnimationEvidence {
            standard_clips_loaded: state.animations_loaded,
            selected_exact_name: config.animation.clone(),
            sample_normalized_ppm: config.animation.as_ref().map(|_| 500_000),
            animation_players: stats.animation_players,
            sampled_players: state.sampled_players,
        },
        runtime: GpuRuntimeEvidence {
            scene_ready: state.scene_ready,
            outline_mode: SourceOutlineMode::Source,
            mesh_parts: stats.meshes,
            skinned_mesh_parts: stats.skinned_meshes,
            skin_joint_references: stats.skin_joint_references,
            resolved_skin_joint_references: stats.resolved_skin_joint_references,
            inverse_bind_matrices: stats.inverse_bind_matrices,
            materials_applied: stats.materials_applied,
            legacy_pass_companions: stats.legacy_pass_companions,
            outline_pass_companions: stats.outline_pass_companions,
            assigned_texture_bindings: stats.assigned_texture_bindings,
            exact_mip_markers: stats.exact_mip_markers,
            exact_mip_chains: stats.exact_mip_chains,
            exact_mip_levels: stats.exact_mip_levels,
            material_errors: stats.material_errors,
            shader_errors: stats.shader_errors,
        },
        screenshot: GpuScreenshotEvidence {
            relative_png: evidence_config.relative_png.clone(),
            byte_length: png_length,
            sha256: sha256_hex(&captured.bytes),
            width: captured.width,
            height: captured.height,
            foreground_pixels: captured.foreground_pixels,
        },
    };
    let mut json = serde_json::to_vec_pretty(&evidence)
        .map_err(|error| format!("cannot serialize GPU evidence: {error}"))?;
    json.push(b'\n');

    atomic_write_new(&config.screenshot, &captured.bytes)?;
    if let Err(error) = atomic_write_new(&evidence_config.json_path, &json) {
        let cleanup = fs::remove_file(&config.screenshot).map_err(|cleanup_error| {
            format!(
                "{error}; additionally could not remove uncommitted PNG {}: {cleanup_error}",
                config.screenshot.display()
            )
        });
        return cleanup.and(Err(error));
    }
    if let Some(runtime_smoke_path) = config.runtime_smoke.as_deref() {
        let runtime_smoke = LogicalModelRuntimeSmoke {
            schema: RUNTIME_SMOKE_SCHEMA.to_owned(),
            status: AutomatedGpuStatus::Passed,
            render_profile: GPU_RENDER_PROFILE.to_owned(),
            model: evidence.model.clone(),
            gltf_loaded_with_dependencies: state.gltf_loaded_with_dependencies,
            exact_animation_names: state.exact_animation_names.clone(),
            selected_exact_name: config.animation.clone(),
            scene_ready: state.scene_ready,
            animation_players: stats.animation_players,
            animation_graph_handles: stats.animation_graph_handles,
            sampled_players: state.sampled_players,
            animation_evaluation_frames: state
                .animation_started_frame
                .map_or(0, |started| state.frames.saturating_sub(started)),
            frames: state.frames,
            material_errors: stats.material_errors,
            shader_errors: stats.shader_errors,
            gpu_evidence_relative_json: evidence_config.relative_json.clone(),
            gpu_evidence_sha256: sha256_hex(&json),
            screenshot_relative_png: evidence_config.relative_png.clone(),
            screenshot_sha256: sha256_hex(&captured.bytes),
        };
        let mut runtime_json = serde_json::to_vec_pretty(&runtime_smoke)
            .map_err(|error| format!("cannot serialize runtime-smoke evidence: {error}"))?;
        runtime_json.push(b'\n');
        if let Err(error) = atomic_write_new(runtime_smoke_path, &runtime_json) {
            let _ = fs::remove_file(&evidence_config.json_path);
            let _ = fs::remove_file(&config.screenshot);
            return Err(error);
        }
    }
    Ok(())
}

pub(super) fn pixel_differs_from(pixel: &[u8], color: [u8; 3], tolerance: i16) -> bool {
    (0..3).any(|channel| (i16::from(pixel[channel]) - i16::from(color[channel])).abs() > tolerance)
}

/// Rejects screenshots that contain only the solid clear color or only the
/// source outline shell. ECS readiness is not sufficient evidence that both
/// independently-specialized GPU pipelines drew the model: the simpler
/// outline pipeline can become ready before the toon surface pipeline, and an
/// unoccluded back-face shell fills the complete silhouette.
pub(super) fn screenshot_foreground(
    rgb: &[u8],
    width: u32,
    height: u32,
    outline_colors: &[[u8; 3]],
    minimum_foreground_override: Option<u64>,
) -> Result<(u64, f64), String> {
    let pixel_count = u64::from(width) * u64::from(height);
    let expected_bytes = usize::try_from(pixel_count)
        .ok()
        .and_then(|pixels| pixels.checked_mul(3))
        .ok_or_else(|| "GPU screenshot dimensions overflow host memory".to_owned())?;
    if width < 2 || height < 2 || rgb.len() != expected_bytes {
        return Err(format!(
            "GPU screenshot has invalid RGB layout: {width}x{height}, {} bytes",
            rgb.len()
        ));
    }

    let width = usize::try_from(width).map_err(|_| "screenshot width is not addressable")?;
    let height = usize::try_from(height).map_err(|_| "screenshot height is not addressable")?;
    let corners = [
        0,
        (width - 1) * 3,
        (height - 1) * width * 3,
        (height * width - 1) * 3,
    ];
    let mut background = [0_u32; 3];
    for offset in corners {
        for channel in 0..3 {
            background[channel] += u32::from(rgb[offset + channel]);
        }
    }
    let background = background.map(|value| (value / 4) as i16);
    let is_foreground = |pixel: &[u8]| {
        (0..3).any(|channel| (i16::from(pixel[channel]) - background[channel]).abs() > 8)
    };
    let foreground_pixels = rgb
        .chunks_exact(3)
        .filter(|pixel| is_foreground(pixel))
        .count() as u64;
    let minimum_foreground =
        minimum_foreground_override.unwrap_or_else(|| (pixel_count / 2_000).max(128));
    if foreground_pixels < minimum_foreground {
        return Err(format!(
            "GPU screenshot contains only the clear background: {foreground_pixels}/{pixel_count} foreground pixels (minimum {minimum_foreground})"
        ));
    }
    if !outline_colors.is_empty() {
        let surface_pixels = rgb
            .chunks_exact(3)
            .filter(|pixel| is_foreground(pixel))
            .filter(|pixel| {
                outline_colors
                    .iter()
                    .all(|outline| pixel_differs_from(pixel, *outline, SCREENSHOT_COLOR_TOLERANCE))
            })
            .count() as u64;
        let minimum_surface = (foreground_pixels / 100).max(128);
        if surface_pixels < minimum_surface {
            return Err(format!(
                "GPU screenshot contains only the source outline pass: {surface_pixels}/{foreground_pixels} foreground pixels differ from every outline color (minimum {minimum_surface})"
            ));
        }
    }
    Ok((
        foreground_pixels,
        foreground_pixels as f64 / pixel_count as f64,
    ))
}

pub(super) fn gate_limit(
    frames: u64,
    elapsed: Duration,
    max_frames: u64,
    timeout: Duration,
) -> Option<&'static str> {
    if elapsed >= timeout {
        Some("wall-clock timeout")
    } else if frames >= max_frames {
        Some("frame limit reached")
    } else {
        None
    }
}

pub(super) fn readiness_reason(
    state: &RuntimeState,
    stats: SceneStats,
    bounds: Option<Bounds3>,
    config: &PreviewConfig,
) -> String {
    let mut missing = Vec::new();
    if !state.scene_ready {
        missing.push("Scene0 not spawned".to_owned());
    }
    if !state.gltf_inspected {
        missing.push("GLB metadata not loaded".to_owned());
    }
    if stats.meshes == 0 {
        missing.push("no Mesh3d".to_owned());
    }
    if config.animation.is_some() && stats.animation_players == 0 {
        missing.push("no AnimationPlayer".to_owned());
    }
    if config.animation.is_some()
        && stats.animation_players > 0
        && stats.animation_graph_handles != stats.animation_players
    {
        missing.push(format!(
            "animation graphs {}/{} players",
            stats.animation_graph_handles, stats.animation_players
        ));
    }
    if config.evidence.is_some()
        && config.animation.is_some()
        && stats.animation_players > 0
        && state.sampled_players != stats.animation_players
    {
        missing.push(format!(
            "sampled animation players {}/{}",
            state.sampled_players, stats.animation_players
        ));
    }
    if stats.materials_applied != stats.meshes {
        missing.push(format!(
            "exact materials {}/{} applied",
            stats.materials_applied, stats.meshes
        ));
    }
    if stats.exact_mip_markers != stats.materials_applied {
        missing.push(format!(
            "exact mip proofs {}/{} applied materials",
            stats.exact_mip_markers, stats.materials_applied
        ));
    }
    if bounds.is_none() {
        missing.push("no finite scene bounds".to_owned());
    }
    if config.animation.is_some() && !state.animation_started {
        missing.push("selected animation not started".to_owned());
    }
    if state.capture_issued && state.screenshot_saved_frame.is_none() {
        missing.push("GPU screenshot still pending".to_owned());
    }
    if state.blank_capture_attempts > 0 && state.screenshot_saved_frame.is_none() {
        missing.push(format!(
            "{} clear-only GPU captures while pipelines warmed up",
            state.blank_capture_attempts
        ));
    }
    if missing.is_empty() {
        "GPU pipeline warmup or post-capture error grace did not finish".to_owned()
    } else {
        missing.join(", ")
    }
}

pub(super) fn transformed_aabb(aabb: &Aabb, transform: &GlobalTransform) -> Option<Bounds3> {
    let center = Vec3::new(aabb.center.x, aabb.center.y, aabb.center.z);
    let half = Vec3::new(
        aabb.half_extents.x,
        aabb.half_extents.y,
        aabb.half_extents.z,
    );
    let mut bounds = None;
    for x in [-1.0, 1.0] {
        for y in [-1.0, 1.0] {
            for z in [-1.0, 1.0] {
                let local = center + half * Vec3::new(x, y, z);
                let point = transform.transform_point(local);
                if let Some(bounds) = bounds.as_mut() {
                    Bounds3::include_point(bounds, point);
                } else {
                    bounds = Bounds3::from_point(point);
                }
            }
        }
    }
    bounds
}

pub(super) fn include_combined_point(combined: &mut Option<Bounds3>, point: Vec3) {
    if let Some(bounds) = combined.as_mut() {
        bounds.include_point(point);
    } else {
        *combined = Bounds3::from_point(point);
    }
}
