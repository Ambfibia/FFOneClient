use super::*;

pub(super) fn clean_initial_trigger_server_id(
    kind: WorldTriggerKind,
    enabled: bool,
    serialized_server_id: i64,
    next_ring_server_id: &mut i64,
) -> i64 {
    if kind != WorldTriggerKind::Ring || !enabled {
        return serialized_server_id;
    }
    let server_id = *next_ring_server_id;
    *next_ring_server_id += 1;
    server_id
}

pub(super) const fn default_capsule_direction() -> i64 {
    1
}

pub(super) fn canonical_map_tile_id(tile_id: &str) -> String {
    tile_id
        .strip_prefix("tile_")
        .map(|coordinates| format!("map_{coordinates}"))
        .unwrap_or_else(|| tile_id.to_owned())
}

/// Convert an exported native world matrix into a Bevy transform.
///
/// The published matrix is already in native space with the `H=diag(-1,1,1)`
/// basis applied, so no further axis conversion happens here.
pub fn transform_from_world_matrix(matrix: &[[f64; 4]; 4]) -> Transform {
    Transform::from_matrix(mat4_from_world_matrix(matrix))
}

pub(super) fn mat4_from_world_matrix(matrix: &[[f64; 4]; 4]) -> Mat4 {
    Mat4::from_cols_array_2d(&[
        [
            matrix[0][0] as f32,
            matrix[1][0] as f32,
            matrix[2][0] as f32,
            matrix[3][0] as f32,
        ],
        [
            matrix[0][1] as f32,
            matrix[1][1] as f32,
            matrix[2][1] as f32,
            matrix[3][1] as f32,
        ],
        [
            matrix[0][2] as f32,
            matrix[1][2] as f32,
            matrix[2][2] as f32,
            matrix[3][2] as f32,
        ],
        [
            matrix[0][3] as f32,
            matrix[1][3] as f32,
            matrix[2][3] as f32,
            matrix[3][3] as f32,
        ],
    ])
}

pub(super) fn vec3(values: [f64; 3]) -> Vec3 {
    Vec3::new(values[0] as f32, values[1] as f32, values[2] as f32)
}

pub(super) fn placement(matrix: Option<&[[f64; 4]; 4]>) -> Transform {
    matrix.map(transform_from_world_matrix).unwrap_or_default()
}

pub(super) fn json_vector3(value: &serde_json::Value, field: &str) -> Result<Vec3, String> {
    let value = value
        .get(field)
        .ok_or_else(|| format!("particle element has no {field}"))?;
    let number = |component: &str| {
        value
            .get(component)
            .and_then(serde_json::Value::as_f64)
            .map(|value| value as f32)
            .filter(|value| value.is_finite())
            .ok_or_else(|| format!("particle element {field}.{component} is not finite"))
    };
    Ok(Vec3::new(number("x")?, number("y")?, number("z")?))
}

pub(super) fn json_quaternion(value: &serde_json::Value, field: &str) -> Result<Quat, String> {
    let value = value
        .get(field)
        .ok_or_else(|| format!("particle element has no {field}"))?;
    let number = |component: &str| {
        value
            .get(component)
            .and_then(serde_json::Value::as_f64)
            .map(|value| value as f32)
            .filter(|value| value.is_finite())
            .ok_or_else(|| format!("particle element {field}.{component} is not finite"))
    };
    let rotation = Quat::from_xyzw(number("x")?, number("y")?, number("z")?, number("w")?);
    if rotation.length_squared() <= f32::EPSILON {
        return Err("particle element rotation is zero length".to_owned());
    }
    Ok(rotation.normalize())
}

pub(super) fn enqueue_world_ep_effects(
    root: Entity,
    document: &NativeWorldBehaviourDocument,
    runtime: &mut TutorialEffectRuntime,
) -> usize {
    let mut queued = 0;
    for record in document
        .effect_emitters
        .iter()
        .filter(|record| record.enabled && record.controller == "EPElementController")
    {
        for element in &record.particle_elements {
            match world_ep_effect_command(root, record.world_matrix.as_ref(), element) {
                Ok(Some(command)) => {
                    runtime.enqueue_streamed_world_effect(command);
                    queued += 1;
                }
                Ok(None) => {}
                Err(error) => warn!(node = %record.node, "world EP effect rejected: {error}"),
            }
        }
    }
    queued
}

pub(super) fn world_scripted_effect_closure(
    document: &NativeWorldBehaviourDocument,
    record: &EffectEmitterRecord,
    closures: &HashMap<&str, &WorldEffectPrefabClosure>,
) -> Result<TutorialEffectClosureFile, String> {
    let mut objects = BTreeMap::<(String, i64), TutorialUnityObjectProof>::new();
    for closure_id in record.resolved_particle_prefabs.iter().flatten() {
        let closure = closures.get(closure_id.as_str()).ok_or_else(|| {
            format!("resolved prefab closure {closure_id:?} is absent from the tile")
        })?;
        for object in &closure.objects {
            let key = (object.asset.clone(), object.path_id);
            if let Some(previous) = objects.get(&key) {
                if previous != object {
                    return Err(format!(
                        "prefab closures disagree about exact object {}:{}",
                        key.0, key.1
                    ));
                }
            } else {
                objects.insert(key, object.clone());
            }
        }
    }

    let controller_asset = format!("world:{}:{}", document.id, record.node);
    let controller_value = serde_json::json!({
        "maxTimer": record.max_timer,
        "longestLifeTime": record.longest_life_time,
        "disableUpdate": if record.disable_update { 1 } else { 0 },
        "particles": record.particles,
    });
    let canonical = serde_json::to_vec(&controller_value)
        .map_err(|error| format!("could not canonicalize world effect controller: {error}"))?;
    objects.insert(
        (controller_asset.clone(), -1),
        TutorialUnityObjectProof {
            asset: controller_asset.clone(),
            path_id: -1,
            type_id: 114,
            class_id: 114,
            object_type: "MonoBehaviour".to_owned(),
            name: record
                .effect_name
                .clone()
                .unwrap_or_else(|| "EffectEmitterController".to_owned()),
            canonical_blake3: blake3::hash(&canonical).to_hex().to_string(),
            value: controller_value,
        },
    );
    Ok(TutorialEffectClosureFile {
        schema: TUTORIAL_EFFECT_CLOSURE_SCHEMA.to_owned(),
        effect_id: None,
        container_route: format!("world-behaviour:{}/{}", document.id, record.node),
        root_asset: controller_asset,
        root_path_id: -1,
        source_bundle_blake3: String::new(),
        source_dump_blake3: String::new(),
        source_assets: Vec::new(),
        objects: objects.into_values().collect(),
    })
}

pub(super) fn world_scripted_effect_placement(
    root: Entity,
    record: &EffectEmitterRecord,
) -> Result<(TutorialEffectPlacement, f32), String> {
    let matrix = record
        .world_matrix
        .as_ref()
        .ok_or_else(|| "effect controller has no exact world matrix".to_owned())?;
    let (scale, rotation, position) =
        mat4_from_world_matrix(matrix).to_scale_rotation_translation();
    if !position.is_finite() || !rotation.is_finite() || !scale.is_finite() {
        return Err("effect controller world transform is not finite".to_owned());
    }
    let scale = if record.conform_to_scale {
        let absolute = scale.abs();
        let minimum = absolute.min_element();
        let maximum = absolute.max_element();
        if minimum <= f32::EPSILON || maximum - minimum > maximum.max(1.0) * 0.000_1 {
            return Err(format!(
                "conformToScale controller has unsupported non-uniform scale {scale:?}"
            ));
        }
        (absolute.x + absolute.y + absolute.z) / 3.0
    } else {
        1.0
    };
    Ok((
        TutorialEffectPlacement::ExactEntityWorld {
            root_entity: root,
            position,
            rotation,
        },
        scale,
    ))
}

pub(super) fn reproducible_world_scripted_effect(record: &EffectEmitterRecord) -> bool {
    record.enabled
        && record.controller == "EffectEmitterController"
        && !record.particles.is_empty()
        && has_reproducible_world_particle_prefab(record)
}

pub(super) fn world_scripted_effect_count(document: &NativeWorldBehaviourDocument) -> usize {
    document
        .effect_emitters
        .iter()
        .filter(|record| reproducible_world_scripted_effect(record))
        .count()
}

pub(super) fn enqueue_world_scripted_effect_record(
    root: Entity,
    document: &NativeWorldBehaviourDocument,
    record: &EffectEmitterRecord,
    closures: &HashMap<&str, &WorldEffectPrefabClosure>,
    runtime: &mut TutorialEffectRuntime,
) -> bool {
    let result = world_scripted_effect_closure(document, record, closures).and_then(|closure| {
        let (placement, scale) = world_scripted_effect_placement(root, record)?;
        runtime.enqueue_world_serialized_effect(closure, placement, scale)
    });
    match result {
        Ok(_) => true,
        Err(error) => {
            warn!(node = %record.node, "world scripted effect rejected: {error}");
            false
        }
    }
}

pub fn enqueue_pending_world_scripted_effects(
    mut commands: Commands,
    mut effect_runtime: Option<ResMut<TutorialEffectRuntime>>,
    mut pending_roots: Query<
        (Entity, &mut PendingWorldScriptedEffects),
        Without<PendingNativeWorldSceneUnload>,
    >,
) {
    let Some(runtime) = effect_runtime.as_deref_mut() else {
        return;
    };
    for (root, mut pending) in &mut pending_roots {
        let document = Arc::clone(pending.document());
        let closures = document
            .effect_prefab_closures
            .iter()
            .map(|closure| (closure.id.as_str(), closure))
            .collect::<HashMap<_, _>>();
        let mut processed = 0;
        while pending.next_record < document.effect_emitters.len()
            && processed < WORLD_SCRIPTED_EFFECTS_PER_FRAME
        {
            let record_index = pending.next_record;
            pending.next_record += 1;
            let record = &document.effect_emitters[record_index];
            if !reproducible_world_scripted_effect(record) {
                continue;
            }
            // Keep the organizer-published nifObject visible. Although some
            // native particle closures also reference that model, ownership is
            // not yet proven strongly enough to hide the authored renderer:
            // doing so removes legitimate world geometry for several effects.
            // The effect itself is still enqueued; only destructive visual
            // ownership transfer is disabled.
            let _ =
                enqueue_world_scripted_effect_record(root, &document, record, &closures, runtime);
            processed += 1;
        }
        if pending.next_record == document.effect_emitters.len() {
            let owning_document = pending.take_document();
            drop(document);
            retire_world_behaviour_document(owning_document);
            commands
                .entity(root)
                .remove::<PendingWorldScriptedEffects>()
                .insert(NativeWorldBehaviourStatus::Ready);
        }
    }
}

pub(super) fn organized_world_source_node(name: &Name) -> Option<&str> {
    let (_, suffix) = name.as_str().rsplit_once('[')?;
    suffix
        .strip_suffix(" visual]")
        .or_else(|| suffix.strip_suffix(" collision]"))
}

pub(super) fn charge_world_behaviour_work(remaining: &mut usize, estimated: usize) {
    // A single indivisible source record is always allowed to make progress.
    // Oversized animation records consume the rest of this root's slice; the
    // next record then waits for a later frame.
    *remaining = remaining.saturating_sub(estimated.max(1));
}

pub(super) fn visibility_for(enabled: bool) -> Visibility {
    if enabled {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    }
}

pub(super) fn world_channel_sample_time(channel: &WorldAnimationChannel, elapsed: f64, clip_time: f64) -> f64 {
    let Some((&start, &end)) = channel.times.first().zip(channel.times.last()) else {
        return clip_time;
    };
    let period = end - start;
    if channel.post_infinity == 2 && elapsed > end && period > f64::EPSILON {
        start + (elapsed - start).rem_euclid(period)
    } else if channel.pre_infinity == 2 && elapsed < start && period > f64::EPSILON {
        start + (elapsed - start).rem_euclid(period)
    } else if channel.pre_infinity == 2 || channel.post_infinity == 2 {
        elapsed
    } else {
        clip_time
    }
}

#[cfg(test)]
pub(super) fn sample_cubic_vector(
    times: &[f64],
    values: &[Vec<f64>],
    in_tangents: &[Vec<f64>],
    out_tangents: &[Vec<f64>],
    time: f64,
) -> Option<Vec<f64>> {
    let first = values.first()?.clone();
    if times.len() != values.len() || times.is_empty() {
        return None;
    }
    if times.len() == 1 || time <= times[0] {
        return Some(first);
    }
    if time >= *times.last()? {
        return values.last().cloned();
    }
    let upper = times.partition_point(|sample| *sample <= time);
    let left = upper.saturating_sub(1);
    let right = upper.min(times.len() - 1);
    let t0 = times[left];
    let t1 = times[right];
    let duration = t1 - t0;
    if !duration.is_finite() || duration <= f64::EPSILON {
        return values.get(right).cloned();
    }
    let from = values.get(left)?;
    let to = values.get(right)?;
    if from.len() != to.len() {
        return None;
    }
    let u = ((time - t0) / duration).clamp(0.0, 1.0);
    let u2 = u * u;
    let u3 = u2 * u;
    let h00 = 2.0 * u3 - 3.0 * u2 + 1.0;
    let h10 = u3 - 2.0 * u2 + u;
    let h01 = -2.0 * u3 + 3.0 * u2;
    let h11 = u3 - u2;
    let input = in_tangents.get(right);
    let output = out_tangents.get(left);
    Some(
        (0..from.len())
            .map(|component| {
                let from_tangent = output
                    .and_then(|values| values.get(component))
                    .copied()
                    .unwrap_or(0.0);
                let to_tangent = input
                    .and_then(|values| values.get(component))
                    .copied()
                    .unwrap_or(0.0);
                h00 * from[component]
                    + h10 * duration * from_tangent
                    + h01 * to[component]
                    + h11 * duration * to_tangent
            })
            .collect(),
    )
}

pub(super) fn world_uv_curve_has_seamless_loop(curve: &WorldAnimationFloatCurve) -> bool {
    let (Some(first), Some(last)) = (curve.values.first(), curve.values.last()) else {
        return false;
    };
    let period = if curve.attribute == "_MainTex.rotation" {
        360.0
    } else {
        1.0
    };
    let turns = (last - first) / period;
    (turns - turns.round()).abs() <= 5.0e-4
}

pub(super) fn sample_cubic_scalar(curve: &WorldAnimationFloatCurve, time: f64) -> Option<f64> {
    let first = *curve.values.first()?;
    if curve.times.len() != curve.values.len() || curve.times.is_empty() {
        return None;
    }
    if curve.times.len() == 1 || time <= curve.times[0] {
        return Some(first);
    }
    if time >= *curve.times.last()? {
        return curve.values.last().copied();
    }
    let upper = curve.times.partition_point(|sample| *sample <= time);
    let left = upper.saturating_sub(1);
    let right = upper.min(curve.times.len() - 1);
    let duration = curve.times[right] - curve.times[left];
    if !duration.is_finite() || duration <= f64::EPSILON {
        return curve.values.get(right).copied();
    }
    let u = ((time - curve.times[left]) / duration).clamp(0.0, 1.0);
    let u2 = u * u;
    let u3 = u2 * u;
    let h00 = 2.0 * u3 - 3.0 * u2 + 1.0;
    let h10 = u3 - 2.0 * u2 + u;
    let h01 = -2.0 * u3 + 3.0 * u2;
    let h11 = u3 - u2;
    Some(
        h00 * curve.values[left]
            + h10 * duration * curve.out_tangents.get(left).copied().unwrap_or(0.0)
            + h01 * curve.values[right]
            + h11 * duration * curve.in_tangents.get(right).copied().unwrap_or(0.0),
    )
}

pub(super) fn legacy_platform_eased_progress(progress: f32) -> f32 {
    (progress - (std::f32::consts::TAU * progress).sin() * 0.15).clamp(0.0, 1.0)
}

pub(super) fn legacy_platform_source_position(
    motion: &WorldPlatformMotion,
    timer: f32,
    returning: bool,
) -> Vec3 {
    let eased = legacy_platform_eased_progress(timer);
    let delta = motion.to - motion.from;
    let length = delta.length();
    // EpPlatformTrigger.Start stores `(to - from) * 0.5`, rather than the
    // usual midpoint. The transform itself is already placed at `from`.
    let center = delta * 0.5;
    let angle = if returning {
        std::f32::consts::TAU - timer * std::f32::consts::PI + std::f32::consts::FRAC_PI_2
    } else {
        timer * std::f32::consts::PI + std::f32::consts::FRAC_PI_2
    };
    match motion.move_type {
        1 => center + Vec3::new(0.0, angle.sin(), angle.cos()) * (length * 0.5),
        2 => center + Vec3::new(angle.sin(), angle.cos(), 0.0) * (length * 0.5),
        3 => center + Vec3::new(angle.sin(), 0.0, angle.cos()) * (length * 0.5),
        8..=10 if motion.path_points.len() >= 2 => {
            let scaled = timer.clamp(0.0, 1.0) * (motion.path_points.len() - 1) as f32;
            let index = (scaled.floor() as usize).min(motion.path_points.len() - 2);
            motion.path_points[index].lerp(motion.path_points[index + 1], scaled.fract())
        }
        0 | 6 | 7 => delta * eased,
        _ => Vec3::ZERO,
    }
}

pub(super) fn trigger_volume_contains_local_point(volume: &WorldTriggerVolume, point: Vec3) -> bool {
    let offset = point - volume.center;
    match volume.shape {
        WorldTriggerVolumeShape::Sphere => offset.length_squared() <= volume.radius.powi(2),
        WorldTriggerVolumeShape::Box => {
            let half = volume.size.abs() * 0.5;
            offset.abs().cmple(half).all()
        }
        WorldTriggerVolumeShape::Capsule => {
            let axis = match volume.direction {
                0 => Vec3::X,
                2 => Vec3::Z,
                _ => Vec3::Y,
            };
            let half_segment = (volume.height.abs() * 0.5 - volume.radius).max(0.0);
            let along = offset.dot(axis).clamp(-half_segment, half_segment);
            let closest = axis * along;
            (offset - closest).length_squared() <= volume.radius.powi(2)
        }
    }
}

/// Consume an armed jump pad only after authored ground collision identifies
/// that pad's renderer/collider hierarchy. This is the clean split between
/// `OnTriggerEnter` and `OnControllerColliderHit` and prevents mid-air sphere
/// entry from producing an early, visually incorrect bounce.
pub fn activate_world_jumppads(
    mut commands: Commands,
    time: Res<Time>,
    parents: Query<&ChildOf>,
    triggers: Query<&WorldTrigger>,
    mut players: Query<(
        Entity,
        &Transform,
        &mut crate::movement::LegacyPlayerController,
        Option<&crate::world::NativeWorldGroundSupport>,
        &mut WorldJumppadArmed,
    )>,
    mut gameplay: ResMut<WorldGameplayIntentQueue>,
    mut audio: Option<ResMut<GameplayAudioRuntime>>,
) {
    let delta = time.delta_secs().max(0.0);
    for (entity, transform, mut controller, support, mut armed) in &mut players {
        armed.remaining_seconds -= delta;
        if armed.remaining_seconds < 0.0 {
            commands.entity(entity).remove::<WorldJumppadArmed>();
            continue;
        }
        let Ok(trigger) = triggers.get(armed.trigger) else {
            commands.entity(entity).remove::<WorldJumppadArmed>();
            continue;
        };
        let touching_pad = support.is_some_and(|support| {
            entity_or_ancestor_is_model(support.collider, &trigger.model_entities, &parents)
        });
        if !touching_pad || !controller.launch_from_jumppad(armed.power) {
            continue;
        }
        let request = make_jumppad_request(
            transform.translation,
            controller.velocity,
            controller.yaw_degrees,
            controller.current_direction_key(),
            armed.power,
        );
        let _ = gameplay.push(packet::P_CL2FE_REQ_PC_JUMPPAD, &request);
        if let Some(audio) = audio.as_deref_mut() {
            audio.queue_player_jumppad(entity);
        }
        commands.entity(entity).remove::<WorldJumppadArmed>();
    }
}

pub(super) fn distance_to_segment(point: Vec3, start: Vec3, end: Vec3) -> f32 {
    let segment = end - start;
    let denominator = segment.length_squared();
    if denominator <= f32::EPSILON {
        return point.distance(start);
    }
    let progress = ((point - start).dot(segment) / denominator).clamp(0.0, 1.0);
    point.distance(start + segment * progress)
}

/// Consume the exact trigger chosen by `AvatarUseButton`. Launcher enters the
/// existing source-reconstructed aim/power UI; zipline hands transform
/// ownership to a deterministic scripted traversal.
pub fn process_world_trigger_uses(
    mut commands: Commands,
    mut uses: ResMut<WorldTriggerUseQueue>,
    mut triggers: Query<(
        &WorldTrigger,
        &GlobalTransform,
        Option<&mut WorldSwitchState>,
    )>,
    mut actors: Query<(&Transform, &mut crate::movement::LegacyPlayerController)>,
    cameras: Query<&LegacyOrbitCamera>,
    mut launcher: ResMut<LauncherUiModel>,
    mut launcher_outbox: ResMut<LauncherUiOutbox>,
    mut active_launcher: ResMut<ActiveWorldLauncher>,
) {
    for (actor, trigger_entity) in uses.take_all() {
        let Ok((trigger, trigger_global, mut switch_state)) = triggers.get_mut(trigger_entity)
        else {
            continue;
        };
        let Ok((actor_transform, mut controller)) = actors.get_mut(actor) else {
            continue;
        };
        match trigger.kind {
            WorldTriggerKind::Launcher => {
                let trigger_transform = trigger_global.compute_transform();
                let camera_height = cameras
                    .iter()
                    .find(|camera| camera.target == actor)
                    .map_or(7.0, |camera| camera.height);
                let spec = LauncherTriggerSpec {
                    trigger_position: trigger_global.translation(),
                    trigger_euler_degrees: native_rotation_to_unity_euler_degrees(
                        trigger_transform.rotation,
                    ),
                    min_power: trigger.min_power,
                    max_power: trigger.max_power,
                    initial_rotation_degrees: trigger.initial_rotate,
                    maximum_rotation_degrees: trigger.max_rotate,
                };
                let _ = launcher.open(
                    spec,
                    actor_transform.translation,
                    camera_height,
                    &mut launcher_outbox,
                );
                controller.begin_scripted_traversal();
                active_launcher.trigger = Some(trigger_entity);
            }
            WorldTriggerKind::Zipline => {
                let start = trigger_global.transform_point(trigger.from);
                let end = trigger_global.transform_point(trigger.to);
                let distance = start.distance(end);
                let speed = trigger.velocity.abs().max(0.01);
                if distance <= 0.01 || !distance.is_finite() {
                    continue;
                }
                controller.begin_scripted_traversal();
                commands.entity(actor).insert((
                    WorldZiplineTraversal {
                        start,
                        end,
                        speed,
                        travelled: 0.0,
                        packet_elapsed: f32::INFINITY,
                        // Matches the published player controller/bone span
                        // already used by the native rope path below.
                        hang_height: 2.2,
                    },
                    Transform::from_translation(start)
                        .with_rotation(actor_transform.rotation)
                        .with_scale(actor_transform.scale),
                ));
            }
            WorldTriggerKind::Rope => {
                let points = trigger
                    .path_points
                    .iter()
                    .map(|point| trigger_global.transform_point(*point))
                    .collect::<Vec<_>>();
                let total_length = polyline_length(&points);
                if points.len() < 2 || total_length <= 0.01 {
                    continue;
                }
                let distance = nearest_polyline_distance(&points, actor_transform.translation);
                controller.begin_scripted_traversal();
                commands.entity(actor).insert(WorldRopeTraversal {
                    points,
                    speed: trigger.speed.max(0.01),
                    distance,
                    total_length,
                    move_type: trigger.move_type,
                });
            }
            WorldTriggerKind::Switch => {
                // The clean switch has no registered OpenFusion request. Its
                // target relation is a local scripted visibility state.
                if let Some(state) = switch_state.as_deref_mut() {
                    state.on = !state.on;
                    let visibility = if state.on {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    };
                    for entity in &trigger.model_entities {
                        commands.entity(*entity).insert(visibility);
                    }
                }
            }
            _ => {}
        }
    }
}

pub(super) fn polyline_length(points: &[Vec3]) -> f32 {
    points
        .windows(2)
        .map(|segment| segment[0].distance(segment[1]))
        .sum()
}

pub(super) fn sample_polyline(points: &[Vec3], distance: f32) -> Option<(Vec3, Vec3)> {
    let mut remaining = distance.max(0.0);
    for segment in points.windows(2) {
        let delta = segment[1] - segment[0];
        let length = delta.length();
        if length <= f32::EPSILON {
            continue;
        }
        if remaining <= length {
            let direction = delta / length;
            return Some((segment[0] + direction * remaining, direction));
        }
        remaining -= length;
    }
    points
        .windows(2)
        .last()
        .map(|segment| (segment[1], (segment[1] - segment[0]).normalize_or_zero()))
}

pub(super) fn nearest_polyline_distance(points: &[Vec3], point: Vec3) -> f32 {
    let mut traversed = 0.0;
    let mut best = (f32::INFINITY, 0.0);
    for segment in points.windows(2) {
        let delta = segment[1] - segment[0];
        let length = delta.length();
        if length <= f32::EPSILON {
            continue;
        }
        let progress = ((point - segment[0]).dot(delta) / delta.length_squared()).clamp(0.0, 1.0);
        let projected = segment[0] + delta * progress;
        let squared = point.distance_squared(projected);
        if squared < best.0 {
            best = (squared, traversed + length * progress);
        }
        traversed += length;
    }
    best.1
}

pub(super) fn lerp_degrees(current: f32, target: f32, factor: f32) -> f32 {
    let delta = (target - current + 180.0).rem_euclid(360.0) - 180.0;
    current + delta * factor.clamp(0.0, 1.0)
}
