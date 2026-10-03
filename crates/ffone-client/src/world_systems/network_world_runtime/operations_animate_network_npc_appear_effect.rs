use super::*;

/// Applies the NPC alpha branch of Retrobution's shared `AppearEffect` to any
/// NPC visual, regardless of whether the owning root came from the live server
/// or tutorial choreography.
///
/// The source effect starts at alpha zero, reaches 0.5 over two seconds, then
/// restores alpha to 1.0. Retrobution writes both `_Color.a` and
/// `_OutlineColor.a`, so the native effect must keep the fill and outline passes
/// synchronized. Native sampling waits until every legacy surface has completed
/// the common XDT texture binding so the two systems never clone the same
/// material concurrently.
pub(super) fn animate_network_npc_appear_effect_0104(
    time: Option<Res<Time>>,
    mut commands: Commands,
    children: Query<&Children>,
    mut appearances: Query<(Entity, &mut NetworkNpcAppearEffect0104), With<NetworkNpcVisual0104>>,
    materials: Option<ResMut<Assets<LegacyModelMaterial>>>,
    outline_materials: Option<ResMut<Assets<LegacyOutlineMaterial>>>,
    mut surfaces: ParamSet<(
        Query<
            Option<&NetworkNpcTextureVariantBound0104>,
            With<MeshMaterial3d<LegacyModelMaterial>>,
        >,
        Query<(
            Entity,
            &mut MeshMaterial3d<LegacyModelMaterial>,
            Option<&NetworkNpcAppearMaterialBound0104>,
        )>,
    )>,
    mut outline_surfaces: Query<(
        Entity,
        &mut MeshMaterial3d<LegacyOutlineMaterial>,
        Option<&NetworkNpcAppearOutlineBound0104>,
    )>,
) {
    let (Some(time), Some(mut materials)) = (time, materials) else {
        return;
    };
    let mut outline_materials = outline_materials;
    let mut ready = Vec::new();
    let mut completed_roots = Vec::new();
    for (root, mut appearance) in &mut appearances {
        let mut stack = vec![root];
        let mut root_surfaces = Vec::new();
        let mut root_outline_surfaces = Vec::new();
        let mut has_unbound_surface = false;
        while let Some(entity) = stack.pop() {
            if let Ok(texture_bound) = surfaces.p0().get(entity) {
                root_surfaces.push(entity);
                has_unbound_surface |= texture_bound.is_none();
            }
            if outline_surfaces.get(entity).is_ok() {
                root_outline_surfaces.push(entity);
            }
            if let Ok(descendants) = children.get(entity) {
                stack.extend(descendants.iter());
            }
        }
        if root_surfaces.is_empty() || has_unbound_surface {
            continue;
        }
        appearance.elapsed_seconds =
            (appearance.elapsed_seconds + time.delta_secs().max(0.0)).min(2.0);
        ready.push((
            root,
            network_npc_appear_alpha_0104(appearance.elapsed_seconds),
            root_surfaces,
            root_outline_surfaces,
        ));
        if appearance.elapsed_seconds >= 2.0 {
            completed_roots.push(root);
        }
    }

    let mut mutable_surfaces = surfaces.p1();
    for (_, alpha, root_surfaces, root_outline_surfaces) in ready {
        for entity in root_surfaces {
            let Ok((_, mut material_handle, bound)) = mutable_surfaces.get_mut(entity) else {
                continue;
            };
            if bound.is_some() {
                if let Some(mut material) = materials.get_mut(&material_handle.0) {
                    material.uniform.base_color.alpha = alpha;
                }
                continue;
            }
            let Some(mut material) = materials.get(&material_handle.0).cloned() else {
                continue;
            };
            material.uniform.base_color.alpha = alpha;
            material_handle.0 = materials.add(material);
            commands
                .entity(entity)
                .insert(NetworkNpcAppearMaterialBound0104);
        }

        let Some(outline_materials) = outline_materials.as_deref_mut() else {
            continue;
        };
        for entity in root_outline_surfaces {
            let Ok((_, mut material_handle, bound)) = outline_surfaces.get_mut(entity) else {
                continue;
            };
            if bound.is_some() {
                if let Some(mut material) = outline_materials.get_mut(&material_handle.0) {
                    material.uniform.color.alpha = alpha.min(1.0);
                }
                continue;
            }
            let Some(mut material) = outline_materials.get(&material_handle.0).cloned() else {
                continue;
            };
            material.uniform.color.alpha = alpha.min(1.0);
            material_handle.0 = outline_materials.add(material);
            commands
                .entity(entity)
                .insert(NetworkNpcAppearOutlineBound0104);
        }
    }

    for root in completed_roots {
        commands.entity(root).remove::<NetworkNpcAppearEffect0104>();
    }
}

#[must_use]
pub(super) fn network_npc_appear_alpha_0104(elapsed_seconds: f32) -> f32 {
    if elapsed_seconds >= 2.0 {
        1.0
    } else {
        (elapsed_seconds.max(0.0) * 0.25).min(0.5)
    }
}

pub(super) fn network_npc_visual_ancestor<'a>(
    mut entity: Entity,
    parents: &Query<&ChildOf>,
    roots: &'a NpcTextureRoots0104,
) -> Option<(
    Entity,
    i32,
    Option<&'a NetworkNpcTextureOverride0104>,
    Option<&'a NetworkNpcTextureOverride0104>,
)> {
    loop {
        if let Ok((root, visual, cinematic)) = roots.get(entity) {
            if let Some(visual) = visual {
                return Some((
                    root,
                    visual.npc_type,
                    visual.main_texture.as_ref(),
                    visual.sub_texture.as_ref(),
                ));
            }
            if let Some(cinematic) = cinematic {
                return Some((
                    root,
                    cinematic.npc_type,
                    cinematic.main_texture.as_ref(),
                    cinematic.sub_texture.as_ref(),
                ));
            }
        }
        entity = parents.get(entity).ok()?.parent();
    }
}

/// `NpcAnimation` fades: SetStandMotion uses 0.3 s, AttackReady and the
/// spell/skill states its 0.2 s `fadeTime`, and walk/run/death the 0.3 s
/// default of the one-argument `Animation.CrossFade`.
#[must_use]
pub(super) fn network_npc_cross_fade_0104(clip: &str) -> Duration {
    if clip.starts_with("stand") || matches!(clip, "walk" | "run" | "death") {
        Duration::from_millis(300)
    } else {
        Duration::from_millis(200)
    }
}

/// Unity's `CrossFade` towards the state that already owns the low layer only
/// retargets its weight: a looping clip keeps its time and events. Every
/// other destination starts at its first key; the first state of a model has
/// nothing to fade from.
pub(super) fn cross_fade_network_npc_low_layer_0104(
    player: &mut AnimationPlayer,
    transitions: &mut AnimationTransitions,
    node: AnimationNodeIndex,
    fade: Duration,
    speed: f32,
    repeat: bool,
) {
    let main = transitions
        .get_main_animation()
        .filter(|main| player.animation(*main).is_some());
    if main == Some(node)
        && let Some(active) = player.animation_mut(node)
        && !active.is_finished()
        && active.repeat_mode()
            == if repeat {
                RepeatAnimation::Forever
            } else {
                RepeatAnimation::Never
            }
    {
        active.set_speed(speed);
        return;
    }
    let fade = if main.is_some() { fade } else { Duration::ZERO };
    transitions
        .play(player, node, fade)
        .set_speed(speed)
        .set_repeat(if repeat {
            RepeatAnimation::Forever
        } else {
            RepeatAnimation::Never
        })
        .resume();
}

/// `NpcAnimation.EndAnimation` continues a one-shot at its authored `end`
/// event rather than the last key. A clip without that event, or a node that
/// was stopped or expired, completes as soon as it no longer plays.
pub(super) fn network_npc_one_shot_completed_0104(
    player: &AnimationPlayer,
    applied: &NetworkNpcAnimationApplied0104,
    visual: &NetworkNpcVisual0104,
) -> bool {
    player.animation(applied.node).is_none_or(|active| {
        active.is_finished()
            || visual
                .animation_ends
                .get(&applied.clip)
                .is_some_and(|end| active.seek_time() >= *end)
    })
}

/// Reports whether the applied idle clip crossed its `end` event since the
/// previous observation, plus the cursor to store when playback moved. A clip
/// without the event continues after each completed loop instead.
pub(super) fn network_npc_idle_end_crossed_0104(
    player: &AnimationPlayer,
    applied: &NetworkNpcAnimationApplied0104,
    cursor: Option<&NetworkNpcIdleEventCursor0104>,
    end: Option<f32>,
) -> (bool, Option<NetworkNpcIdleEventCursor0104>) {
    let Some(active) = player.animation(applied.node) else {
        return (false, None);
    };
    let observed = NetworkNpcIdleEventCursor0104 {
        node: applied.node,
        seek_time: active.seek_time(),
        completions: active.completions(),
    };
    let Some(previous) = cursor
        .filter(|cursor| cursor.node == applied.node && cursor.completions <= observed.completions)
    else {
        return (false, Some(observed));
    };
    let crossed = match end {
        Some(end) => {
            animation_event_crossings(
                previous.seek_time,
                previous.completions,
                observed.seek_time,
                observed.completions,
                active.repeat_mode(),
                end,
            ) > 0
        }
        None => observed.completions > previous.completions,
    };
    (crossed, (observed != *previous).then_some(observed))
}

pub(super) fn network_npc_idle_cursor_0104(
    player: &AnimationPlayer,
    node: AnimationNodeIndex,
) -> NetworkNpcIdleEventCursor0104 {
    let (seek_time, completions) = player.animation(node).map_or((0.0, 0), |active| {
        (active.seek_time(), active.completions())
    });
    NetworkNpcIdleEventCursor0104 {
        node,
        seek_time,
        completions,
    }
}

pub(super) fn store_network_npc_idle_cursor_0104(
    commands: &mut Commands,
    entity: Entity,
    current: Option<&mut NetworkNpcIdleEventCursor0104>,
    next: NetworkNpcIdleEventCursor0104,
) {
    match current {
        Some(current) => *current = next,
        None => {
            commands.entity(entity).insert(next);
        }
    }
}

pub(super) fn restart_network_npc_additive_pair_0104(
    player: &mut AnimationPlayer,
    prepared: &NetworkNpcPreparedAnimationGraph0104,
    pair: NetworkNpcAdditivePair0104,
) -> bool {
    // StopSameLayer: melee variants share layer 102; wound owns layer 104.
    // A wound must not cancel an attack, nor an attack the wound's tail.
    let wound = prepared
        .additive_nodes
        .iter()
        .find(|(_, candidate)| **candidate == pair)
        .is_some_and(|(name, _)| name.contains("wound"));
    for (name, other) in &prepared.additive_nodes {
        if name.contains("wound") == wound && *other != pair {
            player.stop(other.clip);
            player.stop(other.reference);
        }
    }
    // Animation.Play does not rewind a state which is already playing.
    // Frequent damage packets must not trap wound at its first few frames.
    if player
        .animation(pair.clip)
        .is_some_and(|active| !active.is_finished())
    {
        return false;
    }
    // The subtracted first frame is held for the whole playback.
    player
        .start(pair.reference)
        .set_repeat(RepeatAnimation::Never)
        .set_seek_time(0.0)
        .set_weight(1.0)
        .pause();
    player
        .start(pair.clip)
        .set_repeat(RepeatAnimation::Never)
        .set_weight(1.0)
        .resume();
    true
}

/// A `WrapMode.Once` high-layer state keeps playing its tail after `end`
/// hands the low layer back to AttackReady, then leaves the composition.
pub(super) fn stop_finished_network_npc_additive_pairs_0104(
    player: &mut AnimationPlayer,
    prepared: &NetworkNpcPreparedAnimationGraph0104,
    applied: AnimationNodeIndex,
) {
    for pair in prepared.additive_nodes.values() {
        if pair.clip != applied
            && player.animation(pair.reference).is_some()
            && player
                .animation(pair.clip)
                .is_none_or(|active| active.is_finished())
        {
            player.stop(pair.clip);
            player.stop(pair.reference);
        }
    }
}

/// Targets that a high-layer clip animates with a property which not every
/// low-layer clip animates. Bevy's Add node would write the bare difference
/// to such a property (collapsing, for example, highpriest's scaled bones),
/// so the additive pair leaves those targets to the low layer.
pub(super) fn network_npc_uncovered_high_layer_targets_0104(
    named_animations: &[(&str, &Handle<AnimationClip>, &AnimationClip)],
) -> HashSet<AnimationTargetId> {
    let low_layers = named_animations
        .iter()
        .filter(|(name, ..)| NETWORK_NPC_LOW_LAYER_CLIPS_0104.contains(name))
        .map(|(.., clip)| *clip)
        .collect::<Vec<_>>();
    let covered = |target: AnimationTargetId, property: (TypeId, usize)| {
        !low_layers.is_empty()
            && low_layers.iter().all(|clip| {
                clip.curves_for_target(target).is_some_and(|curves| {
                    curves
                        .iter()
                        .any(|curve| network_npc_curve_property_0104(curve) == property)
                })
            })
    };
    let mut uncovered = HashSet::new();
    for (name, _, clip) in named_animations {
        if !network_npc_clip_is_additive_0104(name) {
            continue;
        }
        for (target, curves) in clip.curves() {
            if curves
                .iter()
                .any(|curve| !covered(*target, network_npc_curve_property_0104(curve)))
            {
                uncovered.insert(*target);
            }
        }
    }
    uncovered
}

#[must_use]
pub(super) fn network_npc_curve_property_0104(curve: &VariableCurve) -> (TypeId, usize) {
    match curve.0.evaluator_id() {
        EvaluatorId::ComponentField(field) => **field,
        EvaluatorId::Type(type_id) => (type_id, usize::MAX),
    }
}

pub(super) fn surface_has_named_ancestor(
    mut entity: Entity,
    root: Entity,
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
        if entity == root {
            return false;
        }
        let Ok(parent) = parents.get(entity) else {
            return false;
        };
        entity = parent.parent();
    }
}

pub(super) fn network_npc_playbacks_0104<'a>(
    low: &'a NetworkNpcAnimationApplied0104,
    high: Option<&'a NetworkNpcHighAnimations0104>,
) -> impl Iterator<Item = (usize, &'a NetworkNpcAnimationApplied0104)> {
    [
        Some(low),
        high.and_then(|high| high.applied[0].as_ref()),
        high.and_then(|high| high.applied[1].as_ref()),
    ]
    .into_iter()
    .enumerate()
    .filter_map(|(index, applied)| Some((index, applied?)))
}

pub(super) fn network_npc_effect_ancestor<'a>(
    mut entity: Entity,
    parents: &Query<&ChildOf>,
    roots: &'a Query<(Entity, &NetworkNpc0104, &NetworkNpcVisual0104)>,
) -> Option<(Entity, &'a NetworkNpc0104, &'a NetworkNpcVisual0104)> {
    loop {
        if let Ok(root) = roots.get(entity) {
            return Some(root);
        }
        entity = parents.get(entity).ok()?.parent();
    }
}

pub(super) fn network_npc_ancestor<'a>(
    entity: Entity,
    parents: &Query<&ChildOf>,
    roots: &'a Query<(
        Entity,
        &NetworkNpcVisual0104,
        &NetworkNpcAppearance0104,
        Option<&NetworkNpcMotion0104>,
        Option<&NetworkNpcCombatAnimation0104>,
        Option<&NetworkNpcReadyAnimation0104>,
    )>,
) -> Option<(
    Entity,
    &'a NetworkNpcVisual0104,
    &'a NetworkNpcAppearance0104,
    Option<&'a NetworkNpcMotion0104>,
    Option<&'a NetworkNpcCombatAnimation0104>,
    Option<&'a NetworkNpcReadyAnimation0104>,
)> {
    let mut current = entity;
    loop {
        if let Ok((root, visual, appearance, motion, combat, ready)) = roots.get(current) {
            return Some((root, visual, appearance, motion, combat, ready));
        }
        current = parents.get(current).ok()?.parent();
    }
}

pub(super) fn required_string<'a>(value: &'a Value, field: &str, context: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{context}.{field} must be a string"))
}

pub(super) fn required_i64(value: &Value, field: &str, context: &str) -> Result<i64, String> {
    value
        .get(field)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("{context}.{field} must be an integer"))
}

pub(super) fn optional_i64(value: &Value, field: &str, context: &str) -> Result<Option<i64>, String> {
    value
        .get(field)
        .map(|value| {
            value
                .as_i64()
                .ok_or_else(|| format!("{context}.{field} must be an integer"))
        })
        .transpose()
}

pub(super) fn required_i32(value: &Value, field: &str, context: &str) -> Result<i32, String> {
    let value = required_i64(value, field, context)?;
    i32::try_from(value).map_err(|_| format!("{context}.{field} is outside i32: {value}"))
}

pub(super) fn required_f32(value: &Value, field: &str, context: &str) -> Result<f32, String> {
    let value = value
        .get(field)
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("{context}.{field} must be a number"))?;
    let value = value as f32;
    if !value.is_finite() {
        return Err(format!("{context}.{field} must be finite"));
    }
    Ok(value)
}
