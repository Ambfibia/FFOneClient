use super::*;

/// Select one bounded native materialization batch while preserving FIFO order
/// inside each priority class. A single oversized request is admitted when the
/// batch is otherwise empty so a complex exact effect cannot deadlock forever.
pub(super) fn take_native_spawn_batch(
    queue: &mut VecDeque<NativeSpawnRequest>,
    total_budget: usize,
    streamed_world_budget: usize,
) -> Vec<NativeSpawnRequest> {
    if total_budget == 0 {
        return Vec::new();
    }
    let mut gameplay = VecDeque::new();
    let mut streamed_world = VecDeque::new();
    for request in std::mem::take(queue) {
        if request.is_streamed_world() {
            streamed_world.push_back(request);
        } else {
            gameplay.push_back(request);
        }
    }

    let mut selected = Vec::new();
    let mut remaining_total = total_budget;
    while let Some(request) = gameplay.front() {
        let work = request.spawn_work();
        if work > remaining_total && !selected.is_empty() {
            break;
        }
        let request = gameplay
            .pop_front()
            .expect("a front gameplay spawn request must still exist");
        remaining_total = remaining_total.saturating_sub(work);
        selected.push(request);
        if remaining_total == 0 {
            break;
        }
    }

    // Do not use spare capacity for ambience while a gameplay request is
    // waiting; the latter must remain first on the next frame as well.
    if gameplay.is_empty() && remaining_total > 0 && streamed_world_budget > 0 {
        let mut remaining_streamed = streamed_world_budget.min(remaining_total);
        while let Some(request) = streamed_world.front() {
            let work = request.spawn_work();
            let empty_batch = selected.is_empty();
            if work > remaining_total || work > remaining_streamed {
                if !empty_batch {
                    break;
                }
                // Gameplay is absent, so one oversized ambient request owns
                // this frame. Do not admit more work after saturating both
                // budgets; this is the starvation escape hatch, not an
                // unbounded first request followed by ordinary ambience.
                let request = streamed_world
                    .pop_front()
                    .expect("a front streamed-world spawn request must still exist");
                selected.push(request);
                break;
            }
            let request = streamed_world
                .pop_front()
                .expect("a front streamed-world spawn request must still exist");
            remaining_total = remaining_total.saturating_sub(work);
            remaining_streamed = remaining_streamed.saturating_sub(work);
            selected.push(request);
            if remaining_total == 0 || remaining_streamed == 0 {
                break;
            }
        }
    }

    queue.extend(gameplay);
    queue.extend(streamed_world);
    selected
}

/// Remove requests that cannot possibly materialize before charging the
/// weighted frame budget. Stream owners enter terminal liveness as soon as a
/// tile starts unloading, not only after its multi-frame hierarchy teardown.
pub(super) fn purge_stale_native_spawn_requests(
    queue: &mut VecDeque<NativeSpawnRequest>,
    instance_is_active: impl Fn(u64) -> bool,
    stream_owner_is_live: impl Fn(Entity) -> bool,
) -> Vec<u64> {
    let mut stale_instances = Vec::new();
    queue.retain(|request| {
        let instance_id = request.instance_id();
        let live = instance_is_active(instance_id)
            && request
                .stream_owner()
                .is_none_or(|owner| stream_owner_is_live(owner));
        if !live {
            stale_instances.push(instance_id);
        }
        live
    });
    stale_instances.sort_unstable();
    stale_instances.dedup();
    stale_instances
}

pub(super) fn blocker(
    effect: &ValidatedEffect,
    reason: TutorialNativeClosureBlockerReason,
) -> NativeClosureBlocker {
    NativeClosureBlocker {
        asset: effect.closure.root_asset.clone(),
        path_id: effect.closure.root_path_id,
        object_type: "GameObject".to_owned(),
        reason,
    }
}

pub(super) fn node_blocker(
    object: &TutorialUnityObjectProof,
    reason: TutorialNativeClosureBlockerReason,
) -> NativeClosureBlocker {
    NativeClosureBlocker {
        asset: object.asset.clone(),
        path_id: object.path_id,
        object_type: object.object_type.clone(),
        reason,
    }
}

pub(super) fn invalid(detail: impl Into<String>) -> TutorialNativeClosureBlockerReason {
    TutorialNativeClosureBlockerReason::InvalidSerializedNode {
        detail: detail.into(),
    }
}

pub(super) fn array<'a>(value: &'a JsonValue, key: &str) -> Result<&'a Vec<JsonValue>, String> {
    value
        .get(key)
        .and_then(JsonValue::as_array)
        .ok_or_else(|| format!("{key} is absent or is not an array"))
}

pub(super) fn number(value: &JsonValue, key: &str) -> Result<f32, String> {
    let result = value
        .get(key)
        .and_then(JsonValue::as_f64)
        .map(|value| value as f32)
        .ok_or_else(|| format!("{key} is absent or is not numeric"))?;
    result
        .is_finite()
        .then_some(result)
        .ok_or_else(|| format!("{key} is non-finite"))
}

pub(super) fn positive_number(value: &JsonValue, key: &str) -> Result<f32, String> {
    let result = number(value, key)?;
    (result > 0.0)
        .then_some(result)
        .ok_or_else(|| format!("{key} is not positive"))
}

pub(super) fn nonnegative_number(value: &JsonValue, key: &str) -> Result<f32, String> {
    let result = number(value, key)?;
    (result >= 0.0)
        .then_some(result)
        .ok_or_else(|| format!("{key} is negative"))
}

pub(super) fn integer(value: &JsonValue, key: &str) -> Result<i64, String> {
    value
        .get(key)
        .and_then(JsonValue::as_i64)
        .ok_or_else(|| format!("{key} is absent or is not an integer"))
}

pub(super) fn boolean(value: &JsonValue, key: &str) -> Result<bool, String> {
    value
        .get(key)
        .and_then(JsonValue::as_bool)
        .ok_or_else(|| format!("{key} is absent or is not a boolean"))
}

pub(super) fn vector2(value: &JsonValue, key: &str) -> Result<Vec2, String> {
    let value = object(value, key)?;
    Ok(Vec2::new(number(value, "x")?, number(value, "y")?))
}

pub(super) fn vector3(value: &JsonValue, key: &str) -> Result<Vec3, String> {
    let value = object(value, key)?;
    Ok(Vec3::new(
        number(value, "x")?,
        number(value, "y")?,
        number(value, "z")?,
    ))
}

pub(super) fn native_vector3(value: &JsonValue, key: &str) -> Result<Vec3, String> {
    vector3(value, key).map(unity_to_native_vector)
}

pub(super) fn color(value: &JsonValue) -> Result<Vec4, String> {
    Ok(Vec4::new(
        number(value, "r")?,
        number(value, "g")?,
        number(value, "b")?,
        number(value, "a")?,
    ))
}

pub(super) fn unique_component<'a>(
    closure: &'a TutorialEffectClosureFile,
    go: i64,
    object_type: &str,
) -> Result<&'a TutorialUnityObjectProof, String> {
    let matches = closure
        .objects
        .iter()
        .filter(|object| {
            object.object_type == object_type && game_object_path(&object.value) == Some(go)
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [object] => Ok(*object),
        [] => Err(format!("GameObject {go} has no {object_type}")),
        _ => Err(format!("GameObject {go} has multiple {object_type} nodes")),
    }
}

pub(in super::super) fn compile_effect_plan(
    effect_id: i32,
    effect: &ValidatedEffect,
) -> NativeCompileResult<NativeEffectPlan> {
    let closure = &effect.closure;
    let mesh_scene = exact_effect_mesh_scene(effect_id);
    let material_animation = match compile_material_animation(effect_id, closure) {
        Ok(animation) => animation,
        Err(reason) => {
            return NativeCompileResult {
                plan: None,
                blockers: vec![blocker(effect, reason)],
            };
        }
    };
    let mut blockers = closure
        .objects
        .iter()
        .filter_map(|object| match object.object_type.as_str() {
            "MeshRenderer" if mesh_scene.is_none() => Some(node_blocker(
                object,
                TutorialNativeClosureBlockerReason::LegacyMeshRenderer,
            )),
            "Animation" | "AnimationClip" if mesh_scene.is_none() => Some(node_blocker(
                object,
                TutorialNativeClosureBlockerReason::LegacyAnimationRuntimeUnavailable,
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    let Some(controller) = closure.objects.iter().find(|object| {
        object.object_type == "MonoBehaviour"
            && object.value.get("particles").is_some()
            && object.value.get("maxTimer").is_some()
    }) else {
        if mesh_scene.is_some() {
            return NativeCompileResult {
                plan: Some(NativeEffectPlan {
                    rendered_nodes: 1,
                    emitters: Vec::new(),
                    mesh_scene,
                    material_animation,
                    // EPElementController owns these persistent world-map
                    // instances. The ES mesh prefab has no internal
                    // EffectEmitterController lifetime.
                    maximum_timer: -1.0,
                    longest_lifetime: 0.0,
                    disable_update: true,
                }),
                blockers,
            };
        }
        blockers.push(blocker(
            effect,
            invalid("EffectEmitterController is absent from the exact closure"),
        ));
        return NativeCompileResult {
            plan: None,
            blockers,
        };
    };
    let common = (|| {
        Ok::<_, String>((
            number(&controller.value, "maxTimer")?,
            number(&controller.value, "longestLifeTime")?,
            integer(&controller.value, "disableUpdate")? != 0,
        ))
    })();
    let (maximum_timer, longest_lifetime, disable_update) = match common {
        Ok(common) => common,
        Err(detail) => {
            blockers.push(node_blocker(controller, invalid(detail)));
            return NativeCompileResult {
                plan: None,
                blockers,
            };
        }
    };
    let mut emitters = Vec::new();
    for source in closure.objects.iter().filter(|object| {
        object.object_type == "MonoBehaviour" && object.value.get("numberPerGeneration").is_some()
    }) {
        match compile_emitter(effect_id, closure, controller, source) {
            Ok(plan) => emitters.push(plan),
            Err(reason) => blockers.push(node_blocker(source, reason)),
        }
    }
    if emitters.is_empty() && mesh_scene.is_none() {
        if blockers.is_empty() {
            blockers.push(blocker(
                effect,
                invalid("closure contains no reproducible ParticleEmitterController node"),
            ));
        }
        return NativeCompileResult {
            plan: None,
            blockers,
        };
    }
    NativeCompileResult {
        plan: Some(NativeEffectPlan {
            rendered_nodes: emitters.len() + usize::from(mesh_scene.is_some()),
            emitters,
            mesh_scene,
            material_animation,
            maximum_timer,
            longest_lifetime,
            disable_update,
        }),
        blockers,
    }
}

/// Compile one exact world-map `EffectEmitterController` closure through the
/// same Unity-particle implementation used by the tutorial catalog. World
/// closures are embedded in the streamed tile document instead of owning a
/// catalog row, so the entry below is deliberately metadata-only.
pub(in super::super) fn compile_world_emitter_plan(
    closure: TutorialEffectClosureFile,
) -> NativeCompileResult<NativeEffectPlan> {
    let object_types = closure
        .objects
        .iter()
        .map(|object| object.object_type.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let effect = ValidatedEffect {
        entry: TutorialEffectCatalogEntry {
            effect_id: i32::MIN,
            container_route: closure.container_route.clone(),
            root_asset: closure.root_asset.clone(),
            root_path_id: closure.root_path_id,
            closure_path: String::new(),
            closure_bytes: 0,
            closure_blake3: String::new(),
            object_count: closure.objects.len() as u64,
            object_types,
            component_types: Vec::new(),
        },
        closure,
    };
    compile_effect_plan(i32::MIN, &effect)
}

pub(in super::super) fn compile_projectile_plan(
    effect_id: i32,
    effect: &ValidatedEffect,
) -> NativeCompileResult<NativeProjectilePlan> {
    let closure = &effect.closure;
    let mesh_scene = exact_projectile_mesh_scene(effect_id);
    let mut blockers = Vec::new();
    let material_animation = match compile_material_animation(effect_id, closure) {
        Ok(animation) => animation,
        Err(reason) => {
            return NativeCompileResult {
                plan: None,
                blockers: vec![blocker(effect, reason)],
            };
        }
    };
    let trail_controller = closure.objects.iter().find(|object| {
        object.object_type == "MonoBehaviour"
            && object.value.get("trailLength").is_some()
            && object.value.get("fHeight").is_some()
    });
    let particle_controller = closure.objects.iter().find(|object| {
        object.object_type == "MonoBehaviour"
            && object.value.get("numberPerGeneration").is_some()
            && object.value.get("m_iGenType").and_then(JsonValue::as_i64) == Some(12)
    });
    if trail_controller.is_some() != particle_controller.is_some()
        || trail_controller.is_none() && mesh_scene.is_none()
    {
        blockers.push(blocker(
            effect,
            TutorialNativeClosureBlockerReason::MissingProjectileTrail,
        ));
        return NativeCompileResult {
            plan: None,
            blockers,
        };
    }
    let trail_game_object =
        trail_controller.and_then(|controller| game_object_path(&controller.value));
    for object in &closure.objects {
        match object.object_type.as_str() {
            "MeshRenderer"
                if game_object_path(&object.value) != trail_game_object && mesh_scene.is_none() =>
            {
                blockers.push(node_blocker(
                    object,
                    TutorialNativeClosureBlockerReason::LegacyMeshRenderer,
                ));
            }
            "Animation" | "AnimationClip" if mesh_scene.is_none() => blockers.push(node_blocker(
                object,
                TutorialNativeClosureBlockerReason::LegacyAnimationRuntimeUnavailable,
            )),
            _ => {}
        }
    }
    let trail =
        trail_controller
            .zip(particle_controller)
            .map(|(trail_controller, particle_controller)| {
                (|| {
                    let go = path_id(&trail_controller.value, "m_GameObject")?;
                    if path_id(&particle_controller.value, "m_GameObject")? != go
                        || vector3(&particle_controller.value, "position")? != Vec3::ZERO
                        || vector3(&particle_controller.value, "initVelocity")? != Vec3::ZERO
                        || number(&particle_controller.value, "randomPosition")? != 0.0
                    {
                        return Err("projectile TRAIL node is not deterministic".to_owned());
                    }
                    let renderer = unique_component(closure, go, "MeshRenderer")?;
                    let (blend_mode, material_tint, texture) =
                        material_and_texture(closure, renderer)
                            .map_err(|reason| format!("{reason:?}"))?;
                    let length =
                        nonnegative_number(&particle_controller.value, "numberPerGeneration")?
                            .ceil() as usize;
                    if length < 2 {
                        return Err("projectile trail has fewer than two exact points".to_owned());
                    }
                    Ok(TrailPlan {
                        source_asset: trail_controller.asset.clone(),
                        source_path_id: trail_controller.path_id,
                        length,
                        // EffectEmitterController.SetParticleScale overrides fHeight=1
                        // with ParticleEmitterController.initialSize.
                        height: positive_number(&particle_controller.value, "initialSize")?,
                        interpolation_step: positive_number(&trail_controller.value, "fTimes")?,
                        blend_mode,
                        material_tint,
                        texture,
                    })
                })()
                .map_err(|detail| (trail_controller, detail))
            });
    match trail.transpose() {
        Ok(trail) => NativeCompileResult {
            plan: Some(NativeProjectilePlan {
                rendered_nodes: usize::from(trail.is_some()) + usize::from(mesh_scene.is_some()),
                trail,
                mesh_scene,
                material_animation,
            }),
            blockers,
        },
        Err((trail_controller, detail)) => {
            blockers.push(node_blocker(trail_controller, invalid(detail)));
            NativeCompileResult {
                plan: None,
                blockers,
            }
        }
    }
}

pub(super) fn compile_emitter(
    effect_id: i32,
    closure: &TutorialEffectClosureFile,
    effect_controller: &TutorialUnityObjectProof,
    source: &TutorialUnityObjectProof,
) -> Result<EmitterPlan, TutorialNativeClosureBlockerReason> {
    let go = path_id(&source.value, "m_GameObject").map_err(invalid)?;
    let generation =
        EmitterGeneration::try_from(integer(&source.value, "m_iGenType").map_err(invalid)?)
            .map_err(invalid)?;
    let renderer = unique_component(closure, go, "ParticleRenderer").map_err(invalid)?;
    let animator = unique_component(closure, go, "ParticleAnimator").map_err(invalid)?;
    let uv = object(&renderer.value, "UV Animation").map_err(invalid)?;
    let x_tiles = integer(uv, "x Tile").map_err(invalid)?;
    let y_tiles = integer(uv, "y Tile").map_err(invalid)?;
    let x_tiles = u32::try_from(x_tiles)
        .ok()
        .filter(|tiles| *tiles > 0)
        .ok_or_else(|| invalid("UV Animation.x Tile is not positive"))?;
    let y_tiles = u32::try_from(y_tiles)
        .ok()
        .filter(|tiles| *tiles > 0)
        .ok_or_else(|| invalid("UV Animation.y Tile is not positive"))?;
    let uv_cycles = positive_number(uv, "cycles").map_err(invalid)?;
    if number(&animator.value, "damping").map_err(invalid)? != 1.0 {
        return Err(
            TutorialNativeClosureBlockerReason::UnsupportedParticleAnimator {
                field: "damping".to_owned(),
            },
        );
    }
    if number(&animator.value, "sizeGrow").map_err(invalid)? != 0.0 {
        return Err(
            TutorialNativeClosureBlockerReason::UnsupportedParticleAnimator {
                field: "sizeGrow".to_owned(),
            },
        );
    }
    for field in ["rndForce", "localRotationAxis", "worldRotationAxis"] {
        if vector3(&animator.value, field).map_err(invalid)? != Vec3::ZERO {
            return Err(
                TutorialNativeClosureBlockerReason::UnsupportedParticleAnimator {
                    field: field.to_owned(),
                },
            );
        }
    }
    let (script_keys, initial_translation, initial_emit) =
        script_keys_for(effect_controller, go).map_err(invalid)?;
    let (blend_mode, material_tint, texture) = material_and_texture(closure, renderer)?;
    Ok(EmitterPlan {
        source_asset: source.asset.clone(),
        source_path_id: source.path_id,
        generation,
        random_position: number(&source.value, "randomPosition").map_err(invalid)?,
        random_angle: number(&source.value, "randomAngle").map_err(invalid)?,
        random_velocity: number(&source.value, "randomVelocity").map_err(invalid)?,
        initial_velocity: vector3(&source.value, "initVelocity").map_err(invalid)?,
        plane: vector3(&source.value, "plane").map_err(invalid)?,
        initial_translation,
        script_keys,
        initial_emit,
        generations_per_second: positive_number(&source.value, "generationsPerSecond")
            .map_err(invalid)?,
        number_per_generation: nonnegative_number(&source.value, "numberPerGeneration")
            .map_err(invalid)?,
        lifetime: positive_number(&source.value, "lifeTime").map_err(invalid)?,
        initial_size: positive_number(&source.value, "initialSize").map_err(invalid)?,
        force: native_vector3(&animator.value, "force").map_err(invalid)?,
        colors: parse_colors(&animator.value).map_err(invalid)?,
        animate_color: boolean(&animator.value, "Does Animate Color?").map_err(invalid)?,
        uv_tiles: UVec2::new(x_tiles, y_tiles),
        uv_cycles,
        width_curve: parse_curve(&renderer.value, "m_WidthCurve").map_err(invalid)?,
        height_curve: parse_curve(&renderer.value, "m_HeightCurve").map_err(invalid)?,
        rotation_curve: parse_curve(&renderer.value, "m_RotationCurve").map_err(invalid)?,
        render_mode: LegacyParticleRenderMode::try_from(
            integer(&renderer.value, "m_StretchParticles").map_err(invalid)?,
        )
        .map_err(invalid)?,
        blend_mode,
        material_tint,
        // Retrobution's D3D9 gamma-color-space backbuffer accumulated these
        // overlapping additive cards before display conversion. Bevy blends
        // an sRGB target in linear space, which makes ES668's rings and beam,
        // and the ES527-529 Nano call columns, much too faint.
        // A conservative source lift reproduces the legacy mid-tone
        // accumulation without enabling the whole-frame GlowEffect.
        legacy_gamma_accumulation_gain: if effect_id == 668 || (527..=529).contains(&effect_id) {
            2.0
        } else {
            1.0
        },
        texture,
    })
}

pub(super) fn script_keys_for(
    controller: &TutorialUnityObjectProof,
    go: i64,
) -> Result<(Vec<ScriptKey>, Vec3, bool), String> {
    let entry = array(&controller.value, "particles")?
        .iter()
        .find(|entry| path_id(entry, "particlePrefab").ok() == Some(go))
        .ok_or_else(|| format!("EffectEmitterController has no prefab {go}"))?;
    let keys = array(entry, "scriptKeys")?
        .iter()
        .map(|key| {
            Ok(ScriptKey {
                time: number(key, "time")?,
                // EffectEmitterController assigns this serialized Unity local
                // position directly to the instantiated emitter transform.
                translate: native_vector3(key, "translate")?,
                emit: integer(key, "genType")? != 0,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let first = *keys
        .first()
        .ok_or_else(|| format!("particle prefab {go} has no script keys"))?;
    if keys.windows(2).any(|pair| pair[0].time > pair[1].time) {
        return Err(format!("particle prefab {go} has unordered script keys"));
    }
    Ok((keys, first.translate, first.emit))
}

pub(super) fn named_color(properties: &JsonValue, name: &str) -> Result<Vec4, String> {
    for pair in array(properties, "m_Colors")? {
        let pair = pair
            .as_array()
            .ok_or_else(|| "m_Colors entry is not a pair".to_owned())?;
        if pair.len() == 2 && pair[0].get("name").and_then(JsonValue::as_str) == Some(name) {
            return color(&pair[1]);
        }
    }
    Err(format!("m_Colors has no exact {name} entry"))
}

pub(super) fn argb4444_level_size(width: u32, height: u32) -> Result<usize, String> {
    if width == 0 || height == 0 {
        return Err("ARGB4444 texture level has a zero dimension".to_owned());
    }
    usize::try_from(width)
        .ok()
        .and_then(|width| {
            usize::try_from(height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|pixels| pixels.checked_mul(2))
        .ok_or_else(|| "ARGB4444 texture level byte count overflows".to_owned())
}

pub(super) fn compressed_level_size(block_bytes: usize, width: u32, height: u32) -> Result<usize, String> {
    if width == 0 || height == 0 {
        return Err("compressed texture level has a zero dimension".to_owned());
    }
    let blocks_x = usize::try_from(width.div_ceil(4))
        .map_err(|_| "compressed texture block width does not fit usize".to_owned())?;
    let blocks_y = usize::try_from(height.div_ceil(4))
        .map_err(|_| "compressed texture block height does not fit usize".to_owned())?;
    blocks_x
        .checked_mul(blocks_y)
        .and_then(|blocks| blocks.checked_mul(block_bytes))
        .ok_or_else(|| "compressed texture level byte count overflows".to_owned())
}

pub(super) fn reconcile_named_native_root_liveness(
    mut runtime: ResMut<TutorialEffectRuntime>,
    roots: Query<(Entity, &NativeRoot)>,
) {
    let live_roots = roots
        .iter()
        .map(|(entity, root)| (root.instance_id, entity))
        .collect::<BTreeMap<_, _>>();
    runtime.reconcile_named_native_roots(&live_roots);
}

pub(super) fn streamed_effect_ownership(
    streamed_world: bool,
    placement: &TutorialEffectPlacement,
) -> (bool, Option<Entity>) {
    (
        streamed_world,
        streamed_world.then_some(placement.stream_owner()).flatten(),
    )
}

pub(super) fn is_descendant_of(entity: Entity, ancestor: Entity, parents: &Query<&ChildOf>) -> bool {
    let mut current = entity;
    while let Ok(parent) = parents.get(current) {
        current = parent.parent();
        if current == ancestor {
            return true;
        }
    }
    false
}

pub(super) fn mark_native_effect_billboards(
    mut commands: Commands,
    names: Query<(Entity, &Name), Added<Name>>,
) {
    for (entity, name) in &names {
        if name.as_str().ends_with("BillboardCamera") {
            commands.entity(entity).insert(NativeEffectBillboard);
        }
    }
}

pub(super) fn orient_native_effect_billboards(
    cameras: Query<&GlobalTransform, (With<Camera3d>, With<LegacyOrbitCamera>)>,
    parents: Query<&GlobalTransform, Without<NativeEffectBillboard>>,
    mut billboards: Query<
        (&GlobalTransform, &ChildOf, &mut Transform),
        With<NativeEffectBillboard>,
    >,
) {
    let Some(camera) = cameras.iter().next() else {
        return;
    };
    let camera_position = camera.translation();
    for (global, parent, mut transform) in &mut billboards {
        let direction = (camera_position - global.translation()).normalize_or_zero();
        if direction == Vec3::ZERO {
            continue;
        }
        let Ok(parent_global) = parents.get(parent.parent()) else {
            continue;
        };
        let global_rotation = Quat::from_rotation_arc(Vec3::Z, direction);
        let desired_rotation = parent_global.rotation().inverse() * global_rotation;
        if transform.rotation != desired_rotation {
            transform.rotation = desired_rotation;
        }
    }
}
