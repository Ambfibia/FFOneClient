use super::*;

pub(super) const WORLD_ANIMATION_MATERIAL_BINDING_WORK_PER_PLAYER: usize = 16;

#[derive(Component, Debug, Default)]
pub(crate) struct WorldAnimationMaterialBindings {
    pub(super) ready: bool,
    pub(super) by_target: HashMap<String, Vec<WorldAnimationMaterialBinding>>,
    pub(super) initialized: bool,
    pub(super) target_paths: Vec<String>,
    pub(super) next_target: usize,
    pub(super) target_started: bool,
    pub(super) target_stack: Vec<Entity>,
    pub(super) target_handles: Vec<WorldAnimationMaterialBinding>,
}

#[derive(Clone, Debug)]
pub(super) struct WorldAnimationMaterialBinding {
    pub(super) renderer: Entity,
    pub(super) material: Handle<crate::legacy_model_material::LegacyModelMaterial>,
    pub(super) deferred_owner: Option<WorldAnimationMaterialOwner>,
}

// Preserve the original last binding when target subtrees/players overlap.
// An earlier deferred binding may own an asset without owning the rendered slot.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WorldAnimationMaterialOwner {
    pub(super) player: Entity,
    pub(super) target: usize,
    pub(super) ordinal: usize,
}

impl WorldAnimationMaterialBindings {
    pub(crate) fn is_ready(&self) -> bool {
        self.ready
    }
}

pub(super) fn compile_affine_world_material_curve(
    curve: &WorldAnimationFloatCurve,
    elapsed_seconds: f32,
) -> Option<WorldGpuUvCurve> {
    let sample_count = curve.times.len();
    if sample_count < 2
        || curve.values.len() != sample_count
        || curve.in_tangents.len() != sample_count
        || curve.out_tangents.len() != sample_count
        || !elapsed_seconds.is_finite()
        || !curve
            .times
            .iter()
            .chain(&curve.values)
            .chain(&curve.in_tangents)
            .chain(&curve.out_tangents)
            .all(|value| value.is_finite())
        || curve.times.windows(2).any(|times| times[1] < times[0])
    {
        return None;
    }
    let first_time = curve.times[0];
    let first_value = curve.values[0];
    let last_time = *curve.times.last()?;
    let last_value = *curve.values.last()?;
    let duration = last_time - first_time;
    if duration <= f64::EPSILON {
        return None;
    }
    let velocity = (last_value - first_value) / duration;
    if velocity.abs() <= 1.0e-7 {
        return None;
    }
    let value_scale = (last_value - first_value)
        .abs()
        .max(first_value.abs())
        .max(last_value.abs())
        .max(1.0);
    let value_tolerance = value_scale * 5.0e-4;
    let tangent_tolerance = velocity.abs().max(1.0) * 5.0e-4;
    for index in 0..sample_count {
        let expected = first_value + velocity * (curve.times[index] - first_time);
        if (curve.values[index] - expected).abs() > value_tolerance
            || (curve.in_tangents[index] - velocity).abs() > tangent_tolerance
            || (curve.out_tangents[index] - velocity).abs() > tangent_tolerance
        {
            return None;
        }
    }

    let value_at_bind = first_value + velocity * (f64::from(elapsed_seconds) - first_time);
    let value_at_bind = value_at_bind as f32;
    let velocity = velocity as f32;
    (value_at_bind.is_finite() && velocity.is_finite()).then_some(WorldGpuUvCurve {
        value_at_bind,
        velocity,
    })
}

/// Resolve animated material handles once, after every GLB scene below the
/// owning tile has finished materializing. This replaces the old per-curve,
/// per-frame recursive hierarchy walk with direct ECS/asset bindings.
pub(super) fn prepare_world_animation_material_bindings(
    time: Res<Time>,
    mut commands: Commands,
    sharing: Option<Res<crate::legacy_model_material::StaticAssetSharing>>,
    model_sharing: Option<Res<crate::legacy_model_material::ModelMaterialSharing>>,
    static_sources: Query<(), With<crate::legacy_model_material::PendingLegacyStaticWorldMaterial>>,
    companions: Query<&crate::legacy_model_material::LegacyMaterialPassCompanion>,
    roots: Query<
        &crate::world::NativeWorldPresentationStatus,
        Without<crate::world::PendingNativeWorldSceneUnload>,
    >,
    visual_scene_readiness: Query<
        Option<&crate::world::NativeWorldVisualSceneReady>,
        With<crate::world::SpawnedNativeWorldVisual>,
    >,
    children: Query<&Children>,
    renderers: Query<(), With<Mesh3d>>,
    settled_materials: Query<
        (),
        Or<(
            With<crate::legacy_model_material::LegacyMaterialApplied>,
            With<crate::legacy_model_material::LegacyMaterialMetadataError>,
        )>,
    >,
    mut material_handles: Query<(
        Entity,
        &mut MeshMaterial3d<crate::legacy_model_material::LegacyModelMaterial>,
    )>,
    mut materials: ResMut<Assets<crate::legacy_model_material::LegacyModelMaterial>>,
    mut players: Query<(
        Entity,
        &ChildOf,
        &WorldAnimationPlayer,
        &mut WorldAnimationMaterialBindings,
        &mut WorldAnimationVisibilityBindings,
    )>,
) {
    let defer_shared = sharing.as_ref().is_none_or(|sharing| sharing.enabled)
        && model_sharing.as_ref().is_none_or(|sharing| sharing.0);
    let shader_time = time.elapsed_secs_wrapped();
    let shader_time_wrap_period = time.wrap_period().as_secs_f32();
    let mut remaining_global_work = WORLD_ANIMATION_MATERIAL_BINDING_WORK_PER_FRAME;
    for (player_entity, parent, player, mut bindings, mut visibility) in &mut players {
        if bindings.ready || roots.get(parent.parent()).is_err() {
            continue;
        }
        if remaining_global_work == 0 {
            break;
        }
        // Behaviour records are admitted before Bevy has necessarily
        // materialized the referenced glTF WorldAssetRoot descendants. Starting a
        // DFS earlier would observe an empty visual root and permanently mark
        // the animation bound with zero surfaces. WorldInstanceReady is the
        // earliest point at which every referenced visual hierarchy exists.
        if player
            .model_entities
            .iter()
            .any(|entity| matches!(visual_scene_readiness.get(*entity), Ok(None)))
        {
            continue;
        }
        let mut remaining_player_work =
            remaining_global_work.min(WORLD_ANIMATION_MATERIAL_BINDING_WORK_PER_PLAYER);

        if !visibility.initialized {
            visibility.pending_stack = player.model_entities.clone();
            visibility.initialized = true;
        }
        while remaining_player_work > 0
            && let Some(entity) = visibility.pending_stack.pop()
        {
            remaining_player_work -= 1;
            remaining_global_work -= 1;
            if renderers.get(entity).is_ok() {
                visibility.renderers.push(entity);
            }
            if let Ok(descendants) = children.get(entity) {
                visibility.pending_stack.extend(descendants.iter());
            }
        }
        if !visibility.pending_stack.is_empty() {
            continue;
        }
        if !visibility.ready {
            visibility.renderers.sort_unstable();
            visibility.renderers.dedup();
            visibility.ready = true;
        }

        if !bindings.initialized {
            bindings.target_paths = player
                .active_clip()
                .into_iter()
                .flat_map(|clip| &clip.float_curves)
                .filter(|curve| curve.class_id == 21)
                .map(|curve| curve.target_path.clone())
                .collect();
            bindings.target_paths.sort();
            bindings.target_paths.dedup();
            bindings.initialized = true;
        }
        while remaining_player_work > 0 && bindings.next_target < bindings.target_paths.len() {
            let target_path = bindings.target_paths[bindings.next_target].clone();
            if !bindings.target_started {
                bindings.target_stack = player
                    .targets
                    .get(&target_path)
                    .map(|binding| binding.model_entities.clone())
                    .unwrap_or_default();
                bindings.target_handles.clear();
                bindings.target_started = true;
            }
            let gpu_uv_animation = player.active_clip().and_then(|clip| {
                compile_world_gpu_uv_animation(
                    clip,
                    &target_path,
                    player.wrap_mode,
                    player.play_automatically,
                    player.elapsed_seconds,
                )
            });
            while remaining_player_work > 0
                && let Some(entity) = bindings.target_stack.pop()
            {
                remaining_player_work -= 1;
                remaining_global_work -= 1;
                if let Ok((_, mut handle)) = material_handles.get_mut(entity) {
                    let Some(source_material) = materials.get(&handle.0) else {
                        bindings.target_stack.push(entity);
                        remaining_player_work = 0;
                        break;
                    };
                    let source_entity = companions
                        .get(entity)
                        .map_or(entity, |pass| pass.source_mesh_entity);
                    // Defer only published static-world surfaces. Model/effect
                    // consumers with other appearance writers keep eager ownership.
                    let lazy = defer_shared
                        && gpu_uv_animation.is_none()
                        && source_material.is_shared_immutable()
                        && static_sources.contains(source_entity)
                        && visual_scene_readiness.contains(source_entity);
                    let owner = WorldAnimationMaterialOwner {
                        player: player_entity,
                        target: bindings.next_target,
                        ordinal: bindings.target_handles.len(),
                    };
                    if defer_shared {
                        commands.entity(entity).insert(owner);
                    }
                    let assigned = if lazy {
                        handle.0.clone()
                    } else {
                        let mut material = source_material.clone();
                        if let Some(gpu_uv_animation) = gpu_uv_animation {
                            gpu_uv_animation.apply(
                                &mut material,
                                shader_time,
                                shader_time_wrap_period,
                            );
                        }
                        let unique = materials.add(material);
                        handle.0 = unique.clone();
                        unique
                    };
                    bindings.target_handles.push(WorldAnimationMaterialBinding {
                        renderer: entity,
                        material: assigned,
                        deferred_owner: lazy.then_some(owner),
                    });
                } else if renderers.get(entity).is_ok() && settled_materials.get(entity).is_err() {
                    // The GLB renderer exists but the exact legacy material
                    // plugin has not settled it yet. Keep this cursor so the
                    // tile remains hidden rather than permanently losing the
                    // binding or revealing with a one-frame fallback.
                    bindings.target_stack.push(entity);
                    remaining_player_work = 0;
                    break;
                }
                if let Ok(descendants) = children.get(entity) {
                    bindings.target_stack.extend(descendants.iter());
                }
            }
            if !bindings.target_stack.is_empty() {
                break;
            }
            let handles = std::mem::take(&mut bindings.target_handles);
            bindings.by_target.insert(target_path, handles);
            bindings.next_target += 1;
            bindings.target_started = false;
        }
        if bindings.next_target == bindings.target_paths.len() {
            bindings.ready = true;
            bindings.target_paths.clear();
            bindings.target_stack.clear();
            bindings.target_handles.clear();
        }
    }
}

pub(super) fn sample_world_player_material_curve(
    player: &WorldAnimationPlayer,
    curve: &WorldAnimationFloatCurve,
) -> Option<f64> {
    let clip = player.active_clip()?;
    sample_world_clip_material_curve(
        clip,
        player.wrap_mode,
        player.transform_repeats,
        player.elapsed_seconds,
        curve,
    )
}

pub(super) fn sample_world_clip_material_curve(
    clip: &WorldAnimationClip,
    wrap_mode: i64,
    inferred_transform_repeat: bool,
    elapsed_seconds: f32,
    curve: &WorldAnimationFloatCurve,
) -> Option<f64> {
    let elapsed = f64::from(elapsed_seconds);
    let sample_time = if curve.pre_infinity == 2 || curve.post_infinity == 2 {
        // A source Cycle curve owns its own key-range period. Blackhole uses
        // this contract and must see raw elapsed time rather than a clip-level
        // endpoint clamp.
        elapsed
    } else {
        // Other curves inherit the AnimationClip/Animation wrap contract.
        // Thousands of authored pulsing materials are looped at clip level
        // even though their individual curves use Constant infinity.
        world_animation_sample_time(
            elapsed,
            clip.duration,
            clip.looped || wrap_mode == 2 || inferred_transform_repeat || clip.name == "nif-default",
        )
    };
    sample_world_material_curve(curve, sample_time)
}

pub(super) fn sample_world_material_curve(curve: &WorldAnimationFloatCurve, time: f64) -> Option<f64> {
    let sample_time = if curve.times.is_empty() {
        return None;
    } else if curve.pre_infinity == 2 || curve.post_infinity == 2 {
        let first = curve.times[0];
        let last = *curve.times.last()?;
        let duration = last - first;
        if duration > f64::EPSILON {
            first + (time - first).rem_euclid(duration)
        } else {
            time
        }
    } else {
        time
    };
    sample_cubic_scalar(curve, sample_time)
}

pub(super) fn world_material_curve_is_gpu_driven(
    material: &crate::legacy_model_material::LegacyModelMaterial,
    attribute: &str,
) -> bool {
    material.gpu_uv_animation
        && match attribute {
            "_MainTex.offset.x" => material.uniform.uv_animation.x != 0.0,
            "_MainTex.offset.y" => material.uniform.uv_animation.y != 0.0,
            "_MainTex.rotation" => material.uniform.uv_animation.z != 0.0,
            _ => false,
        }
}

pub(super) fn apply_world_material_float_curve(
    material: &mut crate::legacy_model_material::LegacyModelMaterial,
    attribute: &str,
    value: f32,
) -> bool {
    if !value.is_finite() {
        return false;
    }
    // CPU-sampled/non-affine material curves use the same canonical texture
    // orientation as the GPU fast path above.
    let value = if attribute == "_MainTex.offset.y" {
        native_world_texture_offset_y(value)
    } else {
        value
    };
    if world_material_curve_is_gpu_driven(material, attribute) {
        return false;
    }
    let target = match attribute {
        "_Emission.r" => &mut material.uniform.emission.red,
        "_Emission.g" => &mut material.uniform.emission.green,
        "_Emission.b" => &mut material.uniform.emission.blue,
        "_Emission.a" => &mut material.uniform.emission.alpha,
        "_Color.r" => &mut material.uniform.base_color.red,
        "_Color.g" => &mut material.uniform.base_color.green,
        "_Color.b" => &mut material.uniform.base_color.blue,
        "_Color.a" => &mut material.uniform.base_color.alpha,
        "_AmbColor.r" => &mut material.uniform.ambient_color.red,
        "_AmbColor.g" => &mut material.uniform.ambient_color.green,
        "_AmbColor.b" => &mut material.uniform.ambient_color.blue,
        "_AmbColor.a" => &mut material.uniform.ambient_color.alpha,
        "_Cutoff" => &mut material.uniform.alpha_effect.x,
        "_MainTex.scale.x" => &mut material.uniform.uv_scale_offset.x,
        "_MainTex.scale.y" => &mut material.uniform.uv_scale_offset.y,
        "_MainTex.offset.x" => &mut material.uniform.uv_scale_offset.z,
        "_MainTex.offset.y" => &mut material.uniform.uv_scale_offset.w,
        "_MainTex.rotation" => &mut material.uniform.uv_pivot_rotation.z,
        _ => return false,
    };
    if target.to_bits() == value.to_bits() {
        return false;
    }
    *target = value;
    true
}
