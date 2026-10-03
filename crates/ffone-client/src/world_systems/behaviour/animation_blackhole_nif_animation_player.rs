use super::*;

pub(super) const WORLD_EFFECT_NIF_ANIMATION_SCHEMA: &str = "ffone.world-effect-nif-animation.v1";

pub(super) const BLACKHOLE_NIF_ANIMATION_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../assets/game/map/shared/effects/blackhole.nif-animation.json"
));

/// An authored legacy `Animation` player backed by source-decoded TRS and
/// material curves.
#[derive(Component, Clone, Debug)]
pub struct WorldAnimationPlayer {
    pub play_automatically: bool,
    pub animate_physics: bool,
    pub animate_only_if_visible: bool,
    pub wrap_mode: i64,
    pub clip_count: usize,
    pub model_entities: Vec<Entity>,
    pub default_clip: Option<serde_json::Value>,
    pub clip_refs: Vec<serde_json::Value>,
    pub default_clip_id: Option<String>,
    pub clip_ids: Vec<String>,
    pub targets: HashMap<String, WorldAnimationTargetBinding>,
    pub elapsed_seconds: f32,
    pub(super) transform_repeats: bool,
    pub(super) active_clip: Option<Arc<WorldAnimationClip>>,
}

impl WorldAnimationPlayer {
    pub(super) fn active_clip(&self) -> Option<&WorldAnimationClip> {
        self.active_clip.as_deref()
    }
}

#[derive(Clone, Debug)]
pub struct WorldAnimationTargetBinding {
    pub entity: Entity,
    pub model_entities: Vec<Entity>,
}

/// Partial state for one unusually large legacy Animation record. The largest
/// published record owns 190 targets and more than one hundred model
/// reparentings; retaining this cursor keeps even that single record inside the
/// ordinary per-frame behaviour budget.
pub(super) struct PendingWorldAnimationRecord {
    pub(super) record_index: usize,
    pub(super) active_clip_index: Option<usize>,
    pub(super) anchor: Option<Entity>,
    pub(super) next_target: usize,
    pub(super) target_entities: HashMap<String, Entity>,
    pub(super) target_bindings: HashMap<String, WorldAnimationTargetBinding>,
    pub(super) deferred_parent_bindings: Vec<(Entity, String)>,
}

#[derive(Clone, Debug)]
pub(super) struct WorldAnimationCompiledTrack {
    pub(super) entity: Entity,
    pub(super) translation: Option<usize>,
    pub(super) rotation: Option<usize>,
    pub(super) scale: Option<usize>,
}

#[derive(Component, Debug, Default)]
pub(super) struct WorldAnimationCompiledTracks(pub(super) Vec<WorldAnimationCompiledTrack>);

#[derive(Clone, Copy, Debug)]
pub(super) struct WorldAnimationTargetSample {
    pub(super) entity: Entity,
    pub(super) translation: Option<Vec3>,
    pub(super) rotation: Option<Quat>,
    pub(super) scale: Option<Vec3>,
}

#[derive(Component, Debug, Default)]
pub(crate) struct WorldAnimationSamples(pub(super) Vec<WorldAnimationTargetSample>);

#[derive(Component, Debug, Default)]
pub(super) struct WorldAnimationVisibilityBindings {
    pub(super) ready: bool,
    pub(super) renderers: Vec<Entity>,
    pub(super) initialized: bool,
    pub(super) pending_stack: Vec<Entity>,
}

#[derive(Component, Clone, Debug)]
pub struct WorldAnimationTarget {
    pub path: String,
    pub base_transform: Transform,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationRecord {
    pub node: String,
    pub enabled: bool,
    #[serde(default)]
    pub play_automatically: bool,
    #[serde(default)]
    pub animate_physics: bool,
    #[serde(default)]
    pub animate_only_if_visible: bool,
    #[serde(default)]
    pub wrap_mode: i64,
    #[serde(default)]
    pub clip_count: usize,
    #[serde(default)]
    pub models: Vec<String>,
    #[serde(default)]
    pub default_clip: Option<serde_json::Value>,
    #[serde(default)]
    pub clip_refs: Vec<serde_json::Value>,
    #[serde(default)]
    pub default_clip_id: Option<String>,
    #[serde(default)]
    pub clip_ids: Vec<String>,
    #[serde(default)]
    pub targets: Vec<WorldAnimationTargetRecord>,
    #[serde(default)]
    pub world_matrix: Option<[[f64; 4]; 4]>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldAnimationTargetRecord {
    pub path: String,
    pub node: String,
    #[serde(default)]
    pub parent_path: Option<String>,
    #[serde(default)]
    pub animated: bool,
    pub base_local_trs: WorldAnimationTrs,
    pub base_world_matrix: [[f64; 4]; 4],
    #[serde(default)]
    pub root_parent_world_matrix: Option<[[f64; 4]; 4]>,
    #[serde(default)]
    pub models: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldAnimationTrs {
    pub translation: [f64; 3],
    pub rotation: [f64; 4],
    pub scale: [f64; 3],
}

impl WorldAnimationTrs {
    pub(super) fn transform(self) -> Transform {
        let rotation = Quat::from_xyzw(
            self.rotation[0] as f32,
            self.rotation[1] as f32,
            self.rotation[2] as f32,
            self.rotation[3] as f32,
        );
        // Unity's legacy Transform deserializer accepts the all-zero
        // quaternion and its quaternion-to-matrix path evaluates it as the
        // identity rotation. Glam deliberately returns NaNs when zero is
        // normalized. Three authored Candy Cove foliage children retain this
        // exact primary value; allowing NaNs into their parent hierarchy made
        // the meshes flicker, disappear or cover unrelated geometry depending
        // on camera/frustum evaluation.
        let rotation = if rotation.is_finite() && rotation.length_squared() > f32::EPSILON {
            rotation.normalize()
        } else {
            Quat::IDENTITY
        };
        Transform {
            translation: vec3(self.translation),
            rotation,
            scale: vec3(self.scale),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldAnimationClip {
    pub id: String,
    pub name: String,
    pub duration: f64,
    pub sample_rate: f64,
    pub looped: bool,
    #[serde(default)]
    pub channels: Vec<WorldAnimationChannel>,
    #[serde(default)]
    pub float_curves: Vec<WorldAnimationFloatCurve>,
    #[serde(default)]
    pub events: Vec<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldAnimationChannel {
    pub target_path: String,
    pub property: String,
    #[serde(default)]
    pub pre_infinity: i64,
    #[serde(default)]
    pub post_infinity: i64,
    pub times: Vec<f64>,
    pub values: Vec<Vec<f64>>,
    pub in_tangents: Vec<Vec<f64>>,
    pub out_tangents: Vec<Vec<f64>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldAnimationFloatCurve {
    pub target_path: String,
    pub class_id: i64,
    pub attribute: String,
    #[serde(default)]
    pub pre_infinity: i64,
    #[serde(default)]
    pub post_infinity: i64,
    pub times: Vec<f64>,
    pub values: Vec<f64>,
    pub in_tangents: Vec<f64>,
    pub out_tangents: Vec<f64>,
}

/// A small native contract for an AnimationClip owned by an external NIF
/// referenced by an `EffectEmitterController`. Tile behaviour exports retain
/// the pointer and all seven independently-published renderers, but Unity's
/// external prefab clip is not part of the tile's ordinary Animation closure.
/// The checked asset is an exact projection of the clean-primary NIF export;
/// runtime code never opens a legacy bundle or a generated extraction.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct WorldEffectNifAnimationContract {
    pub(super) schema: String,
    pub(super) id: String,
    pub(super) effect_name: String,
    pub(super) effect_route: String,
    pub(super) source: WorldEffectNifAnimationSource,
    pub(super) model_bindings: Vec<WorldEffectNifAnimationBinding>,
    pub(super) clip: WorldAnimationClip,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WorldEffectNifAnimationSource {
    pub(super) alias: String,
    pub(super) relative_container_path: String,
    pub(super) container_bytes: u64,
    pub(super) container_sha256: String,
    pub(super) root_path_id: i64,
    pub(super) animation_path_id: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct WorldEffectNifAnimationBinding {
    pub(super) target_path: String,
    pub(super) model_index: usize,
}

pub(super) fn blackhole_nif_animation_contract() -> Result<&'static WorldEffectNifAnimationContract, String> {
    static CONTRACT: OnceLock<Result<WorldEffectNifAnimationContract, String>> = OnceLock::new();
    let contract = CONTRACT.get_or_init(|| {
        let contract: WorldEffectNifAnimationContract =
            serde_json::from_str(BLACKHOLE_NIF_ANIMATION_JSON.trim_start_matches('\u{feff}'))
                .map_err(|error| {
                    format!("invalid embedded blackhole NIF animation contract: {error}")
                })?;
        validate_blackhole_nif_animation_contract(&contract)?;
        Ok(contract)
    });
    contract.as_ref().map_err(Clone::clone)
}

pub(super) fn validate_blackhole_nif_animation_contract(
    contract: &WorldEffectNifAnimationContract,
) -> Result<(), String> {
    if contract.schema != WORLD_EFFECT_NIF_ANIMATION_SCHEMA
        || contract.id != "effect/blackhole.nif#nif-default"
        || contract.effect_name != "blackhole"
        || contract.effect_route != "effect/blackhole.nif"
    {
        return Err("blackhole NIF animation identity is not the primary contract".to_owned());
    }
    let source = &contract.source;
    if source.alias != "primary"
        || source.relative_container_path != "DongResources_07_08.resourceFile"
        || source.container_bytes != 3_834_593
        || source.container_sha256
            != "1AF8A0F359C5D7231C7C503E282861C6FB8846E4A350ACE0EB37B641613B5605"
        || source.root_path_id != 9_377
        || source.animation_path_id != 2_108
    {
        return Err("blackhole NIF animation primary provenance changed".to_owned());
    }
    if contract.clip.name != "nif-default"
        || !contract.clip.looped
        || (contract.clip.duration - 3.999_997_615_814_209).abs() > 1.0e-9
        || contract.clip.float_curves.len() != 40
        || !contract.clip.channels.is_empty()
    {
        return Err("blackhole NIF animation clip shape changed".to_owned());
    }

    let binding_paths = contract
        .model_bindings
        .iter()
        .map(|binding| binding.target_path.as_str())
        .collect::<HashSet<_>>();
    let binding_indices = contract
        .model_bindings
        .iter()
        .map(|binding| binding.model_index)
        .collect::<HashSet<_>>();
    let curve_paths = contract
        .clip
        .float_curves
        .iter()
        .map(|curve| curve.target_path.as_str())
        .collect::<HashSet<_>>();
    if contract.model_bindings.len() != 7
        || binding_paths.len() != 7
        || binding_indices != HashSet::from_iter(0..7)
        || curve_paths != binding_paths
    {
        return Err("blackhole NIF animation renderer bindings changed".to_owned());
    }
    for curve in &contract.clip.float_curves {
        let samples = curve.times.len();
        if curve.class_id != 21
            || samples == 0
            || curve.values.len() != samples
            || curve.in_tangents.len() != samples
            || curve.out_tangents.len() != samples
            || curve.times.windows(2).any(|times| times[1] < times[0])
            || !curve
                .times
                .iter()
                .chain(&curve.values)
                .chain(&curve.in_tangents)
                .chain(&curve.out_tangents)
                .all(|value| value.is_finite())
        {
            return Err(format!(
                "blackhole NIF material curve {:?} is malformed",
                curve.attribute
            ));
        }
    }
    Ok(())
}

pub(super) fn blackhole_nif_animation_player(
    record: &EffectEmitterRecord,
    entities_by_model: &HashMap<String, Vec<Entity>>,
) -> Result<Option<(WorldAnimationPlayer, Vec<WorldAnimationCompiledTrack>)>, String> {
    if record.effect_name.as_deref() != Some("blackhole") {
        return Ok(None);
    }
    let nif_object = record.nif_object.as_ref().ok_or_else(|| {
        format!(
            "blackhole emitter {:?} has no external NIF pointer",
            record.node
        )
    })?;
    if nif_object.get("pathId").and_then(serde_json::Value::as_i64) != Some(9_377) {
        return Err(format!(
            "blackhole emitter {:?} does not reference the primary NIF root pathId 9377",
            record.node
        ));
    }
    let contract = blackhole_nif_animation_contract()?;
    if record.models.len() != contract.model_bindings.len() {
        return Err(format!(
            "blackhole emitter {:?} publishes {} renderers, expected {}",
            record.node,
            record.models.len(),
            contract.model_bindings.len()
        ));
    }

    let mut targets = HashMap::with_capacity(contract.model_bindings.len());
    let mut model_entities = Vec::new();
    for binding in &contract.model_bindings {
        let model_id = record.models.get(binding.model_index).ok_or_else(|| {
            format!(
                "blackhole animation binding {:?} has invalid model index {}",
                binding.target_path, binding.model_index
            )
        })?;
        let mut entities = entities_by_model.get(model_id).cloned().unwrap_or_default();
        entities.sort_unstable();
        entities.dedup();
        let Some(entity) = entities.first().copied() else {
            return Err(format!(
                "blackhole emitter {:?} has no entity for model {model_id:?}",
                record.node
            ));
        };
        model_entities.extend(entities.iter().copied());
        targets.insert(
            binding.target_path.clone(),
            WorldAnimationTargetBinding {
                // This player owns only material curves. The binding entity is
                // retained for the common target shape but never transformed.
                entity,
                model_entities: entities,
            },
        );
    }
    model_entities.sort_unstable();
    model_entities.dedup();
    let active_clip = Arc::new(contract.clip.clone());
    let player = WorldAnimationPlayer {
        play_automatically: true,
        animate_physics: false,
        animate_only_if_visible: true,
        // EffectEmitterController.Start explicitly forces its instantiated
        // nifObject Animation to Loop in the clean client.
        wrap_mode: 2,
        clip_count: 1,
        model_entities,
        default_clip: None,
        clip_refs: Vec::new(),
        default_clip_id: Some(active_clip.id.clone()),
        clip_ids: vec![active_clip.id.clone()],
        targets,
        elapsed_seconds: 0.0,
        transform_repeats: false,
        active_clip: Some(active_clip),
    };
    Ok(Some((player, Vec::new())))
}

pub(super) fn validate_world_effect_nif_animation_bindings(
    document: &NativeWorldBehaviourDocument,
    entities_by_model: &HashMap<String, Vec<Entity>>,
) -> Result<(), String> {
    for record in &document.effect_emitters {
        if record.effect_name.as_deref() != Some("blackhole") {
            continue;
        }
        if blackhole_nif_animation_player(record, entities_by_model)?.is_none() {
            return Err(format!(
                "blackhole emitter {:?} has no exact NIF animation player",
                record.node
            ));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Runtime systems
// ---------------------------------------------------------------------------

pub(super) fn compile_world_animation_tracks(
    clip: Option<&WorldAnimationClip>,
    targets: &HashMap<String, WorldAnimationTargetBinding>,
) -> Vec<WorldAnimationCompiledTrack> {
    let Some(clip) = clip else {
        return Vec::new();
    };
    let mut tracks = Vec::<WorldAnimationCompiledTrack>::new();
    let mut by_entity = HashMap::<Entity, usize>::new();
    for (channel_index, channel) in clip.channels.iter().enumerate() {
        let Some(binding) = targets.get(&channel.target_path) else {
            continue;
        };
        let index = *by_entity.entry(binding.entity).or_insert_with(|| {
            let index = tracks.len();
            tracks.push(WorldAnimationCompiledTrack {
                entity: binding.entity,
                translation: None,
                rotation: None,
                scale: None,
            });
            index
        });
        let target = match channel.property.as_str() {
            "translation" => &mut tracks[index].translation,
            "rotation" => &mut tracks[index].rotation,
            "scale" => &mut tracks[index].scale,
            _ => continue,
        };
        *target = Some(channel_index);
    }
    tracks
}

/// Renderer bindings below a keyed TRS target inherit that target's motion
/// even when their own path has no animation channel. Return the complete
/// moving closure so adaptive world-range centers never remain attached to a
/// moving ancestor.
pub(super) fn world_animation_dynamic_model_entities(
    targets: &[WorldAnimationTargetRecord],
    bindings: &HashMap<String, WorldAnimationTargetBinding>,
    compiled_tracks: &[WorldAnimationCompiledTrack],
) -> HashSet<Entity> {
    let active_entities = compiled_tracks
        .iter()
        .map(|track| track.entity)
        .collect::<HashSet<_>>();
    let path_by_entity = bindings
        .iter()
        .map(|(path, binding)| (binding.entity, path.as_str()))
        .collect::<HashMap<_, _>>();
    let parent_by_path = targets
        .iter()
        .map(|target| (target.path.as_str(), target.parent_path.as_deref()))
        .collect::<HashMap<_, _>>();
    let active_paths = active_entities
        .iter()
        .filter_map(|entity| path_by_entity.get(entity).copied())
        .collect::<HashSet<_>>();

    let mut dynamic = HashSet::new();
    for (path, binding) in bindings {
        let mut ancestor = Some(path.as_str());
        let mut visited = HashSet::new();
        let mut moves = false;
        while let Some(candidate) = ancestor {
            if !visited.insert(candidate) {
                break;
            }
            if active_paths.contains(candidate) {
                moves = true;
                break;
            }
            ancestor = parent_by_path.get(candidate).copied().flatten();
        }
        if moves {
            dynamic.extend(binding.model_entities.iter().copied());
        }
    }
    dynamic
}

pub(super) fn world_animation_clock_is_paused(
    animate_only_if_visible: bool,
    visibility_ready: bool,
    has_renderers: bool,
    any_renderer_visible: bool,
) -> bool {
    animate_only_if_visible && (!visibility_ready || (has_renderers && !any_renderer_visible))
}

/// Recover the repeating transform controller carried by imported NIF
/// `nif-default` clips. Unity's generated legacy `Animation` wrapper is
/// serialized with `WrapMode::Default`, and these clips do not retain an
/// explicit loop flag, even when the source controller closes a full cycle.
/// Only a finite, closed transform set is admitted here, so authored one-shot
/// AnimationClips keep their existing end-pose behavior.
pub(super) fn world_animation_transform_repeats(clip: Option<&WorldAnimationClip>, wrap_mode: i64) -> bool {
    let Some(clip) = clip else {
        return false;
    };
    if clip.looped || wrap_mode == 2 {
        return true;
    }
    if clip.name != "nif-default" || clip.duration <= f64::EPSILON {
        return false;
    }

    let mut has_transform_channel = false;
    let mut has_motion = false;
    for channel in &clip.channels {
        let (closed, moving) = match channel.property.as_str() {
            "rotation" => closed_rotation_animation_channel(channel),
            "translation" | "scale" => closed_vector_animation_channel(channel),
            _ => continue,
        };
        has_transform_channel = true;
        if !closed {
            return false;
        }
        has_motion |= moving;
    }
    has_transform_channel && has_motion
}

pub(super) fn closed_rotation_animation_channel(channel: &WorldAnimationChannel) -> (bool, bool) {
    let quaternion = |value: &Vec<f64>| {
        if value.len() != 4 || !value.iter().all(|component| component.is_finite()) {
            return None;
        }
        let rotation = Quat::from_xyzw(
            value[0] as f32,
            value[1] as f32,
            value[2] as f32,
            value[3] as f32,
        );
        (rotation.is_finite() && rotation.length_squared() > f32::EPSILON)
            .then(|| rotation.normalize())
    };
    let Some(first) = channel.values.first().and_then(quaternion) else {
        return (false, false);
    };
    let Some(last) = channel.values.last().and_then(quaternion) else {
        return (false, false);
    };
    // Packed legacy quaternions quantize each endpoint independently. The
    // clean spring-cooler controller closes within about one degree, so compare
    // orientation (q == -q) with a small allowance for that source precision.
    let closed = first.dot(last).abs() >= 0.9995;
    let moving = channel.values[1..]
        .iter()
        .filter_map(quaternion)
        .any(|sample| first.dot(sample).abs() < 0.9995);
    (closed, moving)
}

pub(super) fn closed_vector_animation_channel(channel: &WorldAnimationChannel) -> (bool, bool) {
    let (Some(first), Some(last)) = (channel.values.first(), channel.values.last()) else {
        return (false, false);
    };
    if first.len() != last.len()
        || first.is_empty()
        || !channel
            .values
            .iter()
            .flatten()
            .all(|value| value.is_finite())
    {
        return (false, false);
    }
    let scale = first
        .iter()
        .chain(last)
        .fold(1.0_f64, |scale, value| scale.max(value.abs()));
    let tolerance = scale * 5.0e-4;
    let closed = first
        .iter()
        .zip(last)
        .all(|(first, last)| (first - last).abs() <= tolerance);
    let moving = channel.values[1..].iter().any(|sample| {
        sample.len() == first.len()
            && sample
                .iter()
                .zip(first)
                .any(|(sample, first)| (sample - first).abs() > tolerance)
    });
    (closed, moving)
}

pub(super) fn world_animation_sample_time(elapsed_seconds: f64, duration: f64, repeats: bool) -> f64 {
    if duration <= f64::EPSILON {
        return 0.0;
    }
    if repeats {
        elapsed_seconds.rem_euclid(duration)
    } else {
        elapsed_seconds.min(duration)
    }
}

pub(crate) fn apply_world_animation_samples(
    players: Query<&WorldAnimationSamples>,
    mut targets: Query<&mut Transform, With<WorldAnimationTarget>>,
) {
    for samples in &players {
        for sample in &samples.0 {
            let Ok(mut transform) = targets.get_mut(sample.entity) else {
                continue;
            };
            if let Some(translation) = sample.translation
                && transform.translation != translation
            {
                transform.translation = translation;
            }
            if let Some(rotation) = sample.rotation
                && transform.rotation != rotation
            {
                transform.rotation = rotation;
            }
            if let Some(scale) = sample.scale
                && transform.scale != scale
            {
                transform.scale = scale;
            }
        }
    }
}

pub(super) fn sample_world_animation_vector_fixed<const N: usize>(
    channel: &WorldAnimationChannel,
    time: f64,
) -> Option<[f64; N]> {
    let times = &channel.times;
    let values = &channel.values;
    if times.len() != values.len() || times.is_empty() {
        return None;
    }
    let fixed = |index: usize| {
        let value = values.get(index)?;
        (value.len() == N).then(|| std::array::from_fn(|component| value[component]))
    };
    if times.len() == 1 || time <= times[0] {
        return fixed(0);
    }
    if time >= *times.last()? {
        return fixed(times.len() - 1);
    }
    let upper = times.partition_point(|sample| *sample <= time);
    let left = upper.saturating_sub(1);
    let right = upper.min(times.len() - 1);
    let t0 = times[left];
    let t1 = times[right];
    let duration = t1 - t0;
    if !duration.is_finite() || duration <= f64::EPSILON {
        return fixed(right);
    }
    let from = values.get(left)?;
    let to = values.get(right)?;
    if from.len() != N || to.len() != N {
        return None;
    }
    let u = ((time - t0) / duration).clamp(0.0, 1.0);
    let u2 = u * u;
    let u3 = u2 * u;
    let h00 = 2.0 * u3 - 3.0 * u2 + 1.0;
    let h10 = u3 - 2.0 * u2 + u;
    let h01 = -2.0 * u3 + 3.0 * u2;
    let h11 = u3 - u2;
    let input = channel.in_tangents.get(right);
    let output = channel.out_tangents.get(left);
    Some(std::array::from_fn(|component| {
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
    }))
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct WorldGpuUvAnimation {
    pub(super) offset_x: Option<WorldGpuUvCurve>,
    pub(super) offset_y: Option<WorldGpuUvCurve>,
    pub(super) rotation: Option<WorldGpuUvCurve>,
}
