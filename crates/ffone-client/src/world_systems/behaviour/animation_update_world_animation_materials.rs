use super::*;

impl WorldGpuUvAnimation {
    pub(super) fn is_active(self) -> bool {
        self.offset_x.is_some() || self.offset_y.is_some() || self.rotation.is_some()
    }

    pub(super) fn apply(
        self,
        material: &mut crate::legacy_model_material::LegacyModelMaterial,
        shader_time: f32,
        shader_time_wrap_period: f32,
    ) {
        let mut velocity = Vec3::ZERO;
        if let Some(curve) = self.offset_x {
            material.uniform.uv_scale_offset.z = curve.value_at_bind;
            velocity.x = curve.velocity;
        }
        if let Some(curve) = self.offset_y {
            // Native model textures are published with the source Unity rows
            // flipped into the canonical image orientation. A positive Unity
            // V offset therefore moves in the negative native V direction.
            // Applying the source sign here made waterfalls run uphill even
            // though their exact serialized curves were retained.
            material.uniform.uv_scale_offset.w = native_world_texture_offset_y(curve.value_at_bind);
            velocity.y = native_world_texture_offset_y(curve.velocity);
        }
        if let Some(curve) = self.rotation {
            material.uniform.uv_pivot_rotation.z = curve.value_at_bind;
            velocity.z = curve.velocity;
        }
        material.uniform.uv_animation = velocity.extend(shader_time);
        material.uniform.uv_pivot_rotation.w = shader_time_wrap_period;
        material.gpu_uv_animation = true;
    }
}

/// Compile only affine source texture controllers. `nif-default` is the
/// publisher's explicit identity for Gamebryo/NIF texture controllers; those
/// controllers continue past their recovered key range even though Unity's
/// generated AnimationClip wrapper is serialized as non-looping. Other clips
/// retain their authored one-shot/cubic behavior unless an explicit loop is
/// seamless under repeat texture addressing.
pub(super) fn compile_world_gpu_uv_animation(
    clip: &WorldAnimationClip,
    target_path: &str,
    wrap_mode: i64,
    play_automatically: bool,
    elapsed_seconds: f32,
) -> Option<WorldGpuUvAnimation> {
    if !play_automatically {
        return None;
    }
    let nif_texture_controller = clip.name == "nif-default";
    let explicitly_looped = clip.looped || wrap_mode == 2;
    if !nif_texture_controller && !explicitly_looped {
        return None;
    }

    let mut animation = WorldGpuUvAnimation::default();
    for curve in clip
        .float_curves
        .iter()
        .filter(|curve| curve.class_id == 21 && curve.target_path == target_path)
    {
        let Some(affine) = compile_affine_world_material_curve(curve, elapsed_seconds) else {
            continue;
        };
        if !nif_texture_controller && !world_uv_curve_has_seamless_loop(curve) {
            continue;
        }
        match curve.attribute.as_str() {
            "_MainTex.offset.x" => animation.offset_x = Some(affine),
            "_MainTex.offset.y" => animation.offset_y = Some(affine),
            "_MainTex.rotation" => animation.rotation = Some(affine),
            _ => {}
        }
    }
    animation.is_active().then_some(animation)
}

/// Apply legacy Material float curves (notably glow/emission pulses) through
/// the direct handles compiled by `prepare_world_animation_material_bindings`.
pub(super) fn update_world_animation_materials(
    mut players: Query<(
        &WorldAnimationPlayer,
        &mut WorldAnimationMaterialBindings,
        &WorldAnimationVisibilityBindings,
    )>,
    mut render_materials: Query<(
        &mut MeshMaterial3d<crate::legacy_model_material::LegacyModelMaterial>,
        Option<&WorldAnimationMaterialOwner>,
    )>,
    view_visibility: Query<&ViewVisibility, With<Mesh3d>>,
    mut materials: ResMut<Assets<crate::legacy_model_material::LegacyModelMaterial>>,
    mut uniform_updates: Option<ResMut<crate::legacy_model_material::NativeMaterialUniformUpdates>>,
) {
    let mut changed_materials = HashSet::new();
    let mut pipeline_changes = HashSet::new();
    for (player, mut bindings, visibility) in &mut players {
        if !player.play_automatically || !bindings.ready {
            continue;
        }
        // Material animation has no simulation-side owner. Skip every
        // off-screen renderer, not just clips carrying Unity's optional
        // animateOnlyIfVisible flag; elapsed_seconds still advances in the TRS
        // player, so re-entry samples the correct current clip time.
        if visibility.ready
            && !visibility.renderers.is_empty()
            && !visibility.renderers.iter().any(|renderer| {
                view_visibility
                    .get(*renderer)
                    .is_ok_and(|visibility| visibility.get())
            })
        {
            continue;
        }
        let Some(clip) = player.active_clip() else {
            continue;
        };
        for curve in &clip.float_curves {
            // classId 21 is Unity Material. Other component/property curves
            // remain published in the clip but require their own typed
            // runtime owner rather than being guessed here.
            if curve.class_id != 21 {
                continue;
            }
            let Some(handles) = bindings.by_target.get_mut(&curve.target_path) else {
                continue;
            };
            // Sample at most once, only if a visible material consumes the CPU
            // value. Analytic GPU UV tracks otherwise repeated spline sampling
            // and discarded the result on every frame.
            let mut sampled = None;
            for binding in handles {
                // One Animation player can own many repeated surfaces spread
                // across a large world object. A visible renderer used to wake
                // every material below that player, including off-screen
                // instances, and Bevy then rebuilt thousands of GPU bind groups
                // each frame. Sample only the renderer that consumes this
                // instance-local material; re-entry still sees the exact current
                // clip time because the player clock keeps advancing above.
                if !view_visibility
                    .get(binding.renderer)
                    .is_ok_and(|visibility| visibility.get())
                {
                    continue;
                }
                let Some(material) = materials.get(&binding.material) else {
                    continue;
                };
                if world_material_curve_is_gpu_driven(material, &curve.attribute) {
                    continue;
                }
                // Keep the original raw clock/infinity sampling contract.
                let Some(value) = *sampled
                    .get_or_insert_with(|| sample_world_player_material_curve(player, curve))
                else {
                    continue;
                };
                let changed = if let Some(owner) = binding.deferred_owner {
                    let mut private = material.clone();
                    if !apply_world_material_float_curve(
                        &mut private,
                        &curve.attribute,
                        value as f32,
                    ) {
                        continue;
                    }
                    let previous = binding.material.clone();
                    binding.material = materials.add(private);
                    binding.deferred_owner = None;
                    if let Ok((mut rendered, current_owner)) =
                        render_materials.get_mut(binding.renderer)
                        && current_owner == Some(&owner)
                        && rendered.0 == previous
                    {
                        rendered.0 = binding.material.clone();
                    }
                    true
                } else {
                    let material = materials.get_mut_untracked(&binding.material).unwrap();
                    apply_world_material_float_curve(material, &curve.attribute, value as f32)
                };
                if changed {
                    changed_materials.insert(binding.material.id());
                    // AlphaMode::Mask caches the cutoff in PreparedMaterial.
                    if curve.attribute == "_Cutoff" {
                        pipeline_changes.insert(binding.material.id());
                    }
                }
            }
        }
    }
    // Numeric animation writes the persistent GPU uniform, without rebuilding
    // texture bindings. Pipeline-affecting edits and headless apps keep normal
    // asset events; the curves, visibility policy and current time are unchanged.
    for material_id in changed_materials {
        if pipeline_changes.contains(&material_id)
            || !uniform_updates
                .as_mut()
                .is_some_and(|updates| updates.enqueue(material_id))
        {
            // Bevy 0.19 tracks mutable dereferences, not get_mut calls.
            // Publish the preceding untracked edits when a full rebuild is required.
            let _ = materials
                .get_mut(material_id)
                .map(|material| material.into_inner());
        }
    }
}

pub(super) fn take_world_animation_clips(
    document: &mut NativeWorldBehaviourDocument,
) -> Vec<Arc<WorldAnimationClip>> {
    std::mem::take(&mut document.animation_clips)
        .into_iter()
        .map(Arc::new)
        .collect()
}
