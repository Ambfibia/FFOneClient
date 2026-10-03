use super::*;

pub(super) fn apply_preview_visibility_overrides(
    config: Res<PreviewConfig>,
    mut passes: Query<
        (&LegacyMaterialPassCompanion, &mut Visibility),
        Without<PendingLegacyModelMaterial>,
    >,
    mut surfaces: Query<
        (&PendingLegacyModelMaterial, &mut Visibility),
        Without<LegacyMaterialPassCompanion>,
    >,
    source_materials: Query<&PendingLegacyModelMaterial>,
) {
    if let Some(only_material) = config.only_material.as_deref() {
        for (pending, mut visibility) in &mut surfaces {
            let expected = if pending.true_name == only_material {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            if *visibility != expected {
                *visibility = expected;
            }
        }
    }
    for (pass, mut visibility) in &mut passes {
        let hidden_by_material_filter = config.only_material.as_deref().is_some_and(|only| {
            source_materials
                .get(pass.source_mesh_entity)
                .is_ok_and(|pending| pending.true_name != only)
        });
        let hidden_outline =
            config.outline == PreviewOutlineMode::Hidden && pass.pass == LegacyPassKind::Outline;
        let expected = if hidden_by_material_filter || hidden_outline {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if *visibility != expected {
            *visibility = expected;
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn evaluate_acceptance_gate(
    mut commands: Commands,
    config: Res<PreviewConfig>,
    handles: Option<Res<ModelHandles>>,
    asset_server: Res<AssetServer>,
    mut state: ResMut<RuntimeState>,
    mesh_state: Query<
        (
            &Mesh3d,
            Option<&SkinnedMesh>,
            Option<&Aabb>,
            &GlobalTransform,
            Option<&LegacyMaterialApplied>,
            Option<&ExactMipChainApplied>,
            Option<&PendingLegacyModelMaterial>,
            Option<&LegacyMaterialMetadataError>,
            Option<&PreviewNpcTextureOverrideBound>,
        ),
        (With<Mesh3d>, Without<LegacyMaterialPassCompanion>),
    >,
    players: Query<Option<&AnimationGraphHandle>, With<AnimationPlayer>>,
    meshes: Res<Assets<Mesh>>,
    joint_entities: Query<&GlobalTransform>,
    inverse_bindposes: Res<Assets<SkinnedMeshInverseBindposes>>,
    pass_companions: Query<&LegacyMaterialPassCompanion>,
    metadata_errors: Query<&LegacyMaterialMetadataError>,
    mut camera: Query<(&mut Transform, &mut Projection), With<PreviewCamera>>,
    render_errors: Res<CapturedRenderErrors>,
    shared: Res<SharedReport>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if state.terminal {
        return;
    }
    state.frames += 1;
    let elapsed = state.started.elapsed();

    let captured_render_errors = render_errors.messages();
    if !captured_render_errors.is_empty() {
        fail_runtime(
            &mut state,
            &shared,
            &mut app_exit,
            format!(
                "GPU shader/render error: {}",
                captured_render_errors.join(" | ")
            ),
        );
        return;
    }

    if let Some(handles) = handles.as_deref() {
        if let LoadState::Failed(error) = asset_server.load_state(handles.gltf.id()) {
            fail_runtime(
                &mut state,
                &shared,
                &mut app_exit,
                format!("GLB load failed: {error}"),
            );
            return;
        }
        if let Some(RecursiveDependencyLoadState::Failed(error)) =
            asset_server.get_recursive_dependency_load_state(handles.gltf.id())
        {
            fail_runtime(
                &mut state,
                &shared,
                &mut app_exit,
                format!("GLB dependency load failed: {error}"),
            );
            return;
        }
        state.gltf_loaded_with_dependencies =
            asset_server.is_loaded_with_dependencies(handles.gltf.id());
    }

    let mut stats = SceneStats {
        animation_players: players.iter().count() as u64,
        animation_graph_handles: players.iter().filter(|handle| handle.is_some()).count() as u64,
        shader_errors: captured_render_errors.len() as u64,
        ..default()
    };
    let mut main_texture_overrides = 0_u64;
    let mut sub_texture_overrides = 0_u64;
    let mut texture_overrides_loaded = true;
    for companion in &pass_companions {
        stats.legacy_pass_companions = stats.legacy_pass_companions.saturating_add(1);
        if companion.pass == LegacyPassKind::Outline {
            stats.outline_pass_companions = stats.outline_pass_companions.saturating_add(1);
        }
    }
    let mut mip_proof_errors = Vec::new();
    for (_, skin, _, _, applied, mip_marker, pending, error, texture_override) in &mesh_state {
        stats.meshes += 1;
        stats.skinned_meshes += u64::from(skin.is_some());
        stats.materials_applied += u64::from(applied.is_some());
        if let Some(skin) = skin {
            stats.skin_joint_references = stats
                .skin_joint_references
                .saturating_add(skin.joints.len() as u64);
            stats.resolved_skin_joint_references =
                stats.resolved_skin_joint_references.saturating_add(
                    skin.joints
                        .iter()
                        .filter(|joint| joint_entities.get(**joint).is_ok())
                        .count() as u64,
                );
            stats.inverse_bind_matrices = stats.inverse_bind_matrices.saturating_add(
                inverse_bindposes
                    .get(&skin.inverse_bindposes)
                    .map_or(0, |values| values.len() as u64),
            );
        }
        if let Some(marker) = mip_marker {
            stats.exact_mip_markers += 1;
            stats.assigned_texture_bindings += u64::from(marker.assigned_texture_bindings);
            stats.exact_mip_chains += u64::from(marker.mip_chains_applied);
            stats.exact_mip_levels += u64::from(marker.mip_levels);
        }
        match (applied, mip_marker, pending) {
            (Some(_), Some(actual), Some(pending)) => {
                match expected_exact_mip_marker(&pending.texture_bindings) {
                    Ok(expected) if *actual == expected => {}
                    Ok(expected) => mip_proof_errors.push(format!(
                        "exact mip marker {actual:?} contradicts typed material expectation {expected:?}"
                    )),
                    Err(error) => mip_proof_errors.push(error),
                }
            }
            (Some(_), None, _) => mip_proof_errors
                .push("LegacyMaterialApplied entity has no ExactMipChainApplied proof".to_owned()),
            // Static-world compatibility materials are discovered through
            // PendingLegacyStaticWorldMaterial rather than the model-pending
            // component queried by this generic preview. A non-empty exact
            // marker is already emitted only after the published manifest,
            // hashes, mip layout and assembled sampler all validate.
            (Some(_), Some(actual), None)
                if actual.assigned_texture_bindings > 0
                    && actual.mip_chains_applied > 0
                    && actual.mip_levels > 0 => {}
            (Some(_), Some(_), None) => mip_proof_errors.push(
                "LegacyMaterialApplied entity lost PendingLegacyModelMaterial evidence".to_owned(),
            ),
            (None, Some(_), _) => mip_proof_errors
                .push("ExactMipChainApplied exists before its legacy material".to_owned()),
            (None, None, _) => {}
        }
        stats.material_errors += u64::from(error.is_some());
        if let Some(bound) = texture_override {
            match bound.slot {
                PreviewNpcTextureSlot::Main => main_texture_overrides += 1,
                PreviewNpcTextureSlot::Sub => sub_texture_overrides += 1,
            }
            texture_overrides_loaded &=
                asset_server.is_loaded_with_dependencies(bound.texture.id());
        }
    }
    let metadata_messages = metadata_errors
        .iter()
        .map(|error| error.0.clone())
        .collect::<Vec<_>>();
    if !metadata_messages.is_empty() {
        fail_runtime(
            &mut state,
            &shared,
            &mut app_exit,
            format!(
                "exact legacy material metadata error: {}",
                metadata_messages.join(" | ")
            ),
        );
        return;
    }
    if !mip_proof_errors.is_empty() {
        fail_runtime(
            &mut state,
            &shared,
            &mut app_exit,
            format!(
                "exact native mip proof error: {}",
                mip_proof_errors.join(" | ")
            ),
        );
        return;
    }

    let bounds = match collect_rendered_world_bounds(
        &mesh_state,
        &meshes,
        &inverse_bindposes,
        &joint_entities,
    ) {
        Ok(bounds) => bounds,
        Err(error) => {
            fail_runtime(
                &mut state,
                &shared,
                &mut app_exit,
                format!("exact rendered model bounds error: {error}"),
            );
            return;
        }
    };
    shared.update(|report| {
        report.frames = state.frames;
        report.elapsed_seconds = elapsed.as_secs_f64();
        report.scene_ready = state.scene_ready;
        report.gltf_loaded_with_dependencies = state.gltf_loaded_with_dependencies;
        report.meshes = stats.meshes;
        report.skinned_meshes = stats.skinned_meshes;
        report.animation_players = stats.animation_players;
        report.animation_graph_handles = stats.animation_graph_handles;
        report.sampled_players = state.sampled_players;
        report.animation_evaluation_frames = state
            .animation_started_frame
            .map_or(0, |started| state.frames.saturating_sub(started));
        report.materials_applied = stats.materials_applied;
        report.exact_mip_markers = stats.exact_mip_markers;
        report.assigned_texture_bindings = stats.assigned_texture_bindings;
        report.exact_mip_chains = stats.exact_mip_chains;
        report.exact_mip_levels = stats.exact_mip_levels;
        report.material_errors = stats.material_errors;
        report.main_texture_overrides = main_texture_overrides;
        report.sub_texture_overrides = sub_texture_overrides;
        report.texture_overrides_loaded = texture_overrides_loaded;
        report.bounds = bounds;
        report.camera_view = state.camera_view;
        report.blank_capture_attempts = state.blank_capture_attempts;
        report.screenshot_saved = state.screenshot_saved_frame.is_some();
        report.shader_errors = captured_render_errors.len();
    });

    let animation_evaluation_frames = state
        .animation_started_frame
        .map_or(0, |started| state.frames.saturating_sub(started));
    // NpcMoveController.SetupNPC only replaces materials whose names contain
    // the requested `main`/`sub` token. A table texture with no matching
    // material is therefore an intentional Unity no-op, not a broken model.
    let common_ready = state.scene_ready
        && state.gltf_loaded_with_dependencies
        && state.gltf_inspected
        && state.animation_resolved
        && stats.meshes > 0
        && stats.material_errors == 0
        && texture_overrides_loaded
        && bounds.is_some();
    let profile_ready = if let Some(evidence) = config.evidence.as_ref() {
        let expected = &evidence.facts;
        let animation_ready = if expected.standard_animation_names.is_empty() {
            config.animation.is_none()
                && !state.animation_started
                && state.sampled_players == 0
                && stats.animation_players == 0
        } else {
            config.animation.is_some()
                && state.animation_started
                && stats.animation_players > 0
                && stats.animation_graph_handles == stats.animation_players
                && state.sampled_players == stats.animation_players
                && animation_evaluation_frames > 0
        };
        animation_ready
            && state.animations_loaded == expected.standard_animation_names.len() as u64
            && stats.meshes == expected.mesh_parts
            && stats.skinned_meshes == expected.skinned_mesh_parts
            && stats.skin_joint_references == expected.skin_joint_references
            && stats.resolved_skin_joint_references == expected.skin_joint_references
            && stats.inverse_bind_matrices == expected.inverse_bind_matrices
            && stats.materials_applied == expected.materials_applied
            && stats.legacy_pass_companions == expected.legacy_pass_companions
            && stats.outline_pass_companions == expected.outline_pass_companions
            && stats.assigned_texture_bindings == expected.assigned_texture_bindings
            && stats.exact_mip_markers == expected.exact_mip_markers
            && stats.exact_mip_chains == expected.exact_mip_chains
            && stats.exact_mip_levels == expected.exact_mip_levels
    } else {
        (config.animation.is_none()
            || (state.animation_started
                && stats.animation_players > 0
                && stats.animation_graph_handles == stats.animation_players
                && animation_evaluation_frames > 0))
            && stats.materials_applied == stats.meshes
            && stats.exact_mip_markers == stats.materials_applied
    };
    let ready = common_ready && profile_ready;

    if ready {
        let current_frame = state.frames;
        let ready_since = *state.ready_since_frame.get_or_insert(current_frame);
        // AnimationGraphHandle and the fixed seek are inserted through
        // deferred commands. Wait one complete frame so animation evaluation
        // and transform propagation have reached the joint GlobalTransforms
        // used by both the GPU skinning pass and the CPU framing proof.
        if !state.camera_framed && current_frame > ready_since {
            let bounds = bounds.expect("ready requires finite bounds");
            let Some(frame) = CameraFrame::for_bounds(bounds, state.camera_view) else {
                fail_runtime(
                    &mut state,
                    &shared,
                    &mut app_exit,
                    "model bounds cannot produce a finite camera frame".to_owned(),
                );
                return;
            };
            let Ok((mut transform, mut projection)) = camera.single_mut() else {
                fail_runtime(
                    &mut state,
                    &shared,
                    &mut app_exit,
                    "preview must have exactly one 3D camera".to_owned(),
                );
                return;
            };
            *transform = Transform::from_translation(frame.eye).looking_at(frame.target, Vec3::Y);
            if let Projection::Perspective(perspective) = projection.as_mut() {
                perspective.near = frame.near;
                perspective.far = frame.far;
            }
            state.camera_framed = true;
        }

        if !state.capture_issued
            && state.frames.saturating_sub(ready_since) >= PIPELINE_WARMUP_FRAMES
        {
            state.capture_issued = true;
            shared.update(|report| report.capture_issued = true);
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_preview_screenshot);
        }

        if let Some(saved_frame) = state.screenshot_saved_frame
            && state.frames.saturating_sub(saved_frame) >= POST_CAPTURE_ERROR_GRACE_FRAMES
        {
            if let Err(error) = persist_success_outputs(&config, &state, stats) {
                fail_runtime(&mut state, &shared, &mut app_exit, error);
                return;
            }
            succeed_runtime(&mut state, &shared, &mut app_exit);
            return;
        }
    } else {
        state.ready_since_frame = None;
    }

    if let Some(limit) = gate_limit(state.frames, elapsed, config.max_frames, config.timeout) {
        let reason = readiness_reason(&state, stats, bounds, &config);
        fail_runtime(
            &mut state,
            &shared,
            &mut app_exit,
            format!("{limit}: {reason}"),
        );
    }
}
