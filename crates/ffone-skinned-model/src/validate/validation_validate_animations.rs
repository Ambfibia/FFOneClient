use super::*;

pub(super) fn validate_animations(model: &NativeModel) -> Result<()> {
    for (clip_index, clip) in model.animations.iter().enumerate() {
        validate_true_asset_name("animation", &clip.name)?;
        if !clip.duration.is_finite() || !(clip.duration as f32).is_finite() || clip.duration < 0.0
        {
            return invalid(format!("animation {clip_index} duration is invalid"));
        }
        for (label, value) in [
            ("declaredDuration", clip.declared_duration),
            ("keyedDuration", clip.keyed_duration),
            ("eventDuration", clip.event_duration),
        ] {
            if value.is_some_and(|value| {
                !value.is_finite() || !(value as f32).is_finite() || value < 0.0
            }) {
                return invalid(format!("animation {clip_index} {label} is invalid"));
            }
        }
        if clip
            .sample_rate
            .is_some_and(|value| !value.is_finite() || !(value as f32).is_finite() || value <= 0.0)
        {
            return invalid(format!("animation {clip_index} sample rate is invalid"));
        }
        if !clip.metadata.unsupported.is_empty() {
            return invalid(format!(
                "animation {clip_index} contains unsupported source curves; refusing lossy publish"
            ));
        }
        for (channel_index, channel) in clip.channels.iter().enumerate() {
            let context = format!("animation {clip_index} channel {channel_index}");
            validate_channel(model, clip.duration, channel, &context)?;
        }
        validate_trs_source_metadata(model, clip_index, clip)?;
        validate_time_recoveries(model, clip_index, clip)?;
        validate_curve_recoveries(model, clip_index, clip)?;
        for (curve_index, curve) in clip.metadata.float_curves.iter().enumerate() {
            validate_float_curve(
                model,
                clip.duration,
                curve,
                &format!("animation {clip_index} float curve {curve_index}"),
            )?;
        }
        for (curve_index, curve) in clip.metadata.object_curves.iter().enumerate() {
            let context = format!("animation {clip_index} object curve {curve_index}");
            if curve.target_node as usize >= model.nodes.len() || curve.keys.is_empty() {
                return invalid(format!("{context} has an invalid target or no keys"));
            }
            valid_name("object curve property", &curve.property)?;
            let times = curve.keys.iter().map(|key| key.time).collect::<Vec<_>>();
            validate_times(&times, clip.duration, &context)?;
            for key in &curve.keys {
                if let Some(value) = &key.value {
                    validate_asset_reference(value, &context)?;
                }
            }
        }
        for (event_index, event) in clip.metadata.events.iter().enumerate() {
            let context = format!("animation {clip_index} event {event_index}");
            if !event.time.is_finite()
                || !(event.time as f32).is_finite()
                || event.time < 0.0
                || event.time > clip.duration + 1.0e-6
            {
                return invalid(format!("{context} time is invalid"));
            }
            valid_name("animation event function", &event.function_name)?;
            finite("animation event float parameter", [&event.float_parameter])?;
            if let Some(provenance) = &event.float_parameter_provenance {
                let bits = provenance
                    .raw_float32_bits
                    .strip_prefix("0x")
                    .filter(|hex| hex.len() == 8)
                    .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                    .ok_or_else(|| {
                        crate::ModelError::Invalid(format!(
                            "{context} float parameter provenance has invalid rawFloat32Bits"
                        ))
                    })?;
                if provenance.source_asset_format != 6
                    || provenance.interpretation
                        != AnimationEventFloatParameterInterpretation::LegacyUnusedNonfiniteNormalizedToZero
                    || event.float_parameter != 0.0
                    || !f32::from_bits(bits).is_nan()
                    || provenance.raw_float32_bits != format!("0x{bits:08x}")
                {
                    return invalid(format!(
                        "{context} float parameter provenance is inconsistent"
                    ));
                }
            }
            if event.string_parameter.len() > 65_535 {
                return invalid(format!("{context} string parameter is too long"));
            }
            if let Some(value) = &event.object_parameter {
                validate_asset_reference(value, &context)?;
            }
            match &event.object_parameter_provenance {
                AnimationEventObjectParameterProvenance::Missing { .. } => {
                    if event.object_parameter.is_some() {
                        return invalid(format!(
                            "{context} missing object provenance contradicts an object parameter"
                        ));
                    }
                }
                AnimationEventObjectParameterProvenance::SerializedPointer {
                    source_asset_index,
                    file_id,
                    path_id,
                    interpretation,
                } => match interpretation {
                    SerializedEventObjectParameterInterpretation::NullPathId => {
                        if *path_id != 0 || event.object_parameter.is_some() {
                            return invalid(format!(
                                "{context} null-path-id provenance is inconsistent"
                            ));
                        }
                    }
                    SerializedEventObjectParameterInterpretation::NonNullUnresolved => {
                        return invalid(format!(
                            "{context} contains an unresolved non-null Unity object pointer"
                        ));
                    }
                    SerializedEventObjectParameterInterpretation::NonNullMetadataOnly => {
                        if *path_id == 0
                            || event.object_parameter.is_some()
                            || event.function_name != "sound"
                            || event.string_parameter.trim().is_empty()
                        {
                            return invalid(format!(
                                "{context} metadata-only Unity object pointer is inconsistent"
                            ));
                        }
                    }
                    SerializedEventObjectParameterInterpretation::NonNullLegacyUnused => {
                        let exact_larry_event = *source_asset_index == 5
                            && *file_id == 1
                            && event.object_parameter.is_none()
                            && event.function_name == "end"
                            && event.string_parameter.is_empty()
                            && event.int_parameter == 0
                            && matches!(
                                (
                                    clip.name.as_str(),
                                    event.time,
                                    *path_id,
                                    event.message_options,
                                ),
                                ("stand2", 4.730000019073486, 6, 13_156)
                                    | ("stand3", 2.5, 4, 0)
                                    | ("walk", 3.0, 2_816, 1_852_788_223)
                            );
                        if !exact_larry_event {
                            return invalid(format!(
                                "{context} legacy-unused Unity object pointer is inconsistent"
                            ));
                        }
                    }
                },
            }
        }
        let keyed_duration = clip
            .channels
            .iter()
            .filter_map(|channel| channel.times.last().copied())
            .chain(
                clip.metadata
                    .float_curves
                    .iter()
                    .filter_map(|curve| curve.times.last().copied()),
            )
            .chain(clip.metadata.curve_recoveries.iter().flat_map(|recovery| {
                std::iter::once(&recovery.canonical.keys)
                    .chain(recovery.rejected.iter().map(|track| &track.keys))
                    .filter_map(curve_recovery_keys_last_time)
            }))
            .reduce(f64::max);
        let event_duration = clip
            .metadata
            .events
            .iter()
            .map(|event| event.time)
            .reduce(f64::max);
        if !same_optional_duration(clip.keyed_duration, keyed_duration)
            || !same_optional_duration(clip.event_duration, event_duration)
        {
            return invalid(format!(
                "animation {clip_index} duration provenance differs from exact key/event maxima"
            ));
        }
        let effective = [
            clip.declared_duration,
            clip.keyed_duration,
            clip.event_duration,
        ]
        .into_iter()
        .flatten()
        .reduce(f64::max)
        .unwrap_or(0.0);
        if !same_duration(clip.duration, effective) {
            return invalid(format!(
                "animation {clip_index} effective duration is not the exact source maximum"
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_trs_source_metadata(
    model: &NativeModel,
    clip_index: usize,
    clip: &crate::AnimationClip,
) -> Result<()> {
    let hierarchy_paths = (0..model.nodes.len())
        .map(|node| model_node_path(model, node))
        .collect::<Vec<_>>();
    let standard_targets = clip
        .channels
        .iter()
        .map(|channel| {
            let kind = match &channel.values {
                TrackValues::Translation(_) => EmptyTrsBindingKind::Translation,
                TrackValues::Rotation(_) => EmptyTrsBindingKind::Rotation,
                TrackValues::Scale(_) => EmptyTrsBindingKind::Scale,
            };
            (channel.target_node, kind)
        })
        .collect::<BTreeSet<_>>();
    let mut source_indices = BTreeSet::new();
    let mut previous_channel_order = None;
    for (channel_index, channel) in clip.channels.iter().enumerate() {
        let kind = channel_kind(channel);
        validate_channel_key_provenance(
            channel,
            &format!("animation {clip_index} channel {channel_index}"),
        )?;
        let Some(rank) = trs_source_rank(kind, channel.source_encoding) else {
            return invalid(format!(
                "animation {clip_index} channel {channel_index} uses an impossible source encoding"
            ));
        };
        let order = (rank, channel.source_index);
        if previous_channel_order.is_some_and(|previous| previous >= order) {
            return invalid(format!(
                "animation {clip_index} channels are not in canonical source order"
            ));
        }
        previous_channel_order = Some(order);
        if !source_indices.insert((kind, channel.source_encoding, channel.source_index)) {
            return invalid(format!(
                "animation {clip_index} channel {channel_index} repeats a source identity"
            ));
        }
    }

    for (binding_index, binding) in clip.metadata.empty_trs_bindings.iter().enumerate() {
        let context = format!("animation {clip_index} empty TRS binding {binding_index}");
        validate_metadata_target(
            &hierarchy_paths,
            model.nodes.len(),
            binding.target_node,
            &binding.target_path,
            &context,
        )?;
        if !source_indices.insert((binding.kind, binding.source_encoding, binding.source_index)) {
            return invalid(format!(
                "{context} duplicates a (kind, sourceEncoding, sourceIndex) identity"
            ));
        }
        if matches!(
            (binding.kind, binding.source_encoding),
            (
                EmptyTrsBindingKind::Translation | EmptyTrsBindingKind::Scale,
                EmptyTrsSourceEncoding::Compressed
            )
        ) {
            return invalid(format!(
                "{context} uses compressed encoding for a non-rotation curve"
            ));
        }
        if standard_targets.contains(&(binding.target_node, binding.kind)) {
            return invalid(format!(
                "{context} collides with a non-empty standard animation channel"
            ));
        }
    }
    let mut previous_duplicate_order = None;
    for (binding_index, binding) in clip.metadata.duplicate_trs_bindings.iter().enumerate() {
        let context = format!("animation {clip_index} duplicate TRS binding {binding_index}");
        validate_metadata_target(
            &hierarchy_paths,
            model.nodes.len(),
            binding.target_node,
            &binding.target_path,
            &context,
        )?;
        let Some(rank) = trs_source_rank(binding.kind, binding.source_encoding) else {
            return invalid(format!("{context} uses an impossible source encoding"));
        };
        let order = (rank, binding.source_index);
        if previous_duplicate_order.is_some_and(|previous| previous >= order) {
            return invalid(format!(
                "animation {clip_index} duplicateTrsBindings are not in canonical source order"
            ));
        }
        previous_duplicate_order = Some(order);
        if !source_indices.insert((binding.kind, binding.source_encoding, binding.source_index)) {
            return invalid(format!("{context} repeats a source identity"));
        }
        let canonical = clip
            .channels
            .iter()
            .filter(|channel| channel_kind(channel) == binding.kind)
            .nth(binding.canonical_track_index as usize)
            .ok_or_else(|| {
                crate::ModelError::Invalid(format!("{context} canonicalTrackIndex is out of range"))
            })?;
        if canonical.target_node != binding.target_node
            || canonical.source_index != binding.canonical_source_index
            || canonical.source_encoding != binding.canonical_source_encoding
        {
            return invalid(format!(
                "{context} canonical target/source identity does not match its runtime channel"
            ));
        }
        if binding.source_index == binding.canonical_source_index
            && binding.source_encoding == binding.canonical_source_encoding
        {
            return invalid(format!("{context} repeats its canonical source identity"));
        }
        match binding.relation {
            DuplicateTrsRelation::Identical => {
                if binding.resolution_proof.is_some()
                    || canonical.source_key_count != binding.source_key_count
                    || canonical.duplicate_keys != binding.duplicate_keys
                {
                    return invalid(format!(
                        "{context} identical relation does not exactly match canonical provenance"
                    ));
                }
            }
            DuplicateTrsRelation::SerializedLastWriteWins => {
                let proof = binding.resolution_proof.as_ref().ok_or_else(|| {
                    crate::ModelError::Invalid(format!(
                        "{context} serialized-last-write relation has no proof"
                    ))
                })?;
                if proof.rule
                    != SerializedCurveOverwriteRule::LaterSerializedBindingOverwritesEarlier
                    || proof.clip_name != clip.name
                    || !proof.exact_target_path
                    || !proof.exact_source_array_order
                    || binding.source_encoding != binding.canonical_source_encoding
                    || binding.source_index >= binding.canonical_source_index
                {
                    return invalid(format!(
                        "{context} serialized-last-write proof/order is invalid"
                    ));
                }
            }
        }
        validate_duplicate_keys(binding, canonical, clip.duration, &context)?;
    }
    Ok(())
}

pub(super) fn validate_metadata_target(
    hierarchy_paths: &[String],
    node_count: usize,
    target_node: u32,
    target_path: &str,
    context: &str,
) -> Result<()> {
    if target_node as usize >= node_count {
        return invalid(format!("{context} target is out of bounds"));
    }
    if target_path.trim_matches('/') != target_path
        || target_path.contains(['\\', '\0'])
        || target_path.contains("//")
        || target_path.chars().any(char::is_control)
    {
        return invalid(format!("{context} source path is invalid"));
    }
    let roots = hierarchy_paths
        .iter()
        .enumerate()
        .filter(|(_, path)| !path.contains('/'))
        .collect::<Vec<_>>();
    if target_path.is_empty() {
        return match roots.as_slice() {
            [(root_node, _)] if *root_node == target_node as usize => Ok(()),
            [(_, _)] => invalid(format!(
                "{context} empty root-relative path resolves to a different hierarchy target"
            )),
            _ => invalid(format!(
                "{context} empty root-relative path does not have one exact hierarchy root"
            )),
        };
    }
    if let [(root_node, root_path)] = roots.as_slice() {
        let relative = format!("{root_path}/{target_path}");
        if let Some((relative_node, _)) = hierarchy_paths
            .iter()
            .enumerate()
            .find(|(_, candidate)| **candidate == relative)
        {
            if relative_node == target_node as usize {
                return Ok(());
            }
            return invalid(format!(
                "{context} exact root-relative path resolves to a different hierarchy target"
            ));
        }
        if *root_node == target_node as usize && *root_path == target_path {
            return Ok(());
        }
    }
    let suffix = format!("/{target_path}");
    let matches = hierarchy_paths
        .iter()
        .enumerate()
        .filter_map(|(node, candidate)| {
            (candidate == target_path || candidate.ends_with(&suffix)).then_some(node)
        })
        .collect::<Vec<_>>();
    if matches.as_slice() != [target_node as usize] {
        return invalid(format!(
            "{context} path does not resolve uniquely to its exact hierarchy target"
        ));
    }
    Ok(())
}

pub(super) fn validate_channel_key_provenance(channel: &AnimationChannel, context: &str) -> Result<()> {
    validate_key_identity_coverage(
        &channel.source_key_indices,
        &channel.duplicate_keys,
        channel.source_key_count,
        context,
    )?;
    if channel.source_key_indices.len() != channel.times.len() {
        return invalid(format!(
            "{context} sourceKeyIndices are not aligned with runtime keys"
        ));
    }
    validate_same_time_duplicate_keys(channel, &channel.duplicate_keys, context)
}

pub(super) fn validate_duplicate_binding_key_provenance(
    binding: &DuplicateTrsBinding,
    context: &str,
) -> Result<()> {
    let source_indices = match &binding.keys {
        DuplicateTrsKeys::Vec3(keys) => keys
            .iter()
            .map(|key| key.source_key_index)
            .collect::<Vec<_>>(),
        DuplicateTrsKeys::Quaternion(keys) => keys
            .iter()
            .map(|key| key.source_key_index)
            .collect::<Vec<_>>(),
    };
    validate_key_identity_coverage(
        &source_indices,
        &binding.duplicate_keys,
        binding.source_key_count,
        context,
    )?;
    for (duplicate_index, duplicate) in binding.duplicate_keys.iter().enumerate() {
        let canonical_index = duplicate.canonical_key_index as usize;
        let expected = match &binding.keys {
            DuplicateTrsKeys::Vec3(keys) => keys.get(canonical_index).map(|key| {
                ExactTrsKeyPayload::Vec3(ExactVec3KeyPayload {
                    time: key.time,
                    value: key.value,
                    in_tangent: key.in_tangent,
                    out_tangent: key.out_tangent,
                    tangent_mode: key.tangent_mode,
                })
            }),
            DuplicateTrsKeys::Quaternion(keys) => keys.get(canonical_index).map(|key| {
                ExactTrsKeyPayload::Quaternion(ExactQuaternionKeyPayload {
                    time: key.time,
                    value: key.value,
                    in_tangent: key.in_tangent,
                    out_tangent: key.out_tangent,
                    tangent_mode: key.tangent_mode,
                })
            }),
        };
        if expected.as_ref() != Some(&duplicate.key) {
            return invalid(format!(
                "{context} duplicateKeys[{duplicate_index}] differs from its canonical key"
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_key_identity_coverage(
    canonical_indices: &[u32],
    duplicate_keys: &[DuplicateAnimationKey],
    source_key_count: u32,
    context: &str,
) -> Result<()> {
    if canonical_indices.is_empty() || source_key_count == 0 {
        return invalid(format!("{context} has no canonical/source keys"));
    }
    let mut identities = BTreeSet::new();
    let mut previous = None;
    for &source_index in canonical_indices {
        if source_index >= source_key_count
            || previous.is_some_and(|previous| previous >= source_index)
            || !identities.insert(source_index)
        {
            return invalid(format!(
                "{context} canonical sourceKeyIndex order is invalid"
            ));
        }
        previous = Some(source_index);
    }
    previous = None;
    for (duplicate_index, duplicate) in duplicate_keys.iter().enumerate() {
        if duplicate.source_key_index >= source_key_count
            || previous.is_some_and(|previous| previous >= duplicate.source_key_index)
            || !identities.insert(duplicate.source_key_index)
        {
            return invalid(format!(
                "{context} duplicateKeys[{duplicate_index}] source identity/order is invalid"
            ));
        }
        previous = Some(duplicate.source_key_index);
    }
    if identities.len() != source_key_count as usize
        || (0..source_key_count).any(|source_index| !identities.contains(&source_index))
    {
        return invalid(format!(
            "{context} sourceKeyCount is not fully covered by canonical and duplicate keys"
        ));
    }
    Ok(())
}

pub(super) fn validate_same_time_duplicate_keys(
    channel: &AnimationChannel,
    duplicate_keys: &[DuplicateAnimationKey],
    context: &str,
) -> Result<()> {
    for (duplicate_index, duplicate) in duplicate_keys.iter().enumerate() {
        let Some(expected) = channel_key_payload(channel, duplicate.canonical_key_index as usize)
        else {
            return invalid(format!(
                "{context} duplicateKeys[{duplicate_index}] canonicalKeyIndex is out of range"
            ));
        };
        if duplicate.key != expected {
            return invalid(format!(
                "{context} duplicateKeys[{duplicate_index}] differs from its canonical key"
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_duplicate_keys(
    binding: &DuplicateTrsBinding,
    canonical: &AnimationChannel,
    duration: f64,
    context: &str,
) -> Result<()> {
    validate_duplicate_binding_key_provenance(binding, context)?;
    match (&binding.keys, binding.kind, &canonical.values) {
        (
            DuplicateTrsKeys::Vec3(keys),
            EmptyTrsBindingKind::Translation,
            TrackValues::Translation(values),
        )
        | (DuplicateTrsKeys::Vec3(keys), EmptyTrsBindingKind::Scale, TrackValues::Scale(values)) => {
            if binding.relation == DuplicateTrsRelation::Identical {
                validate_duplicate_vec3_keys(keys, values, canonical, duration, context)
            } else {
                validate_displaced_vec3_keys(keys, duration, context)?;
                if duplicate_vec3_keys_match_channel(keys, values, canonical, context)? {
                    return invalid(format!(
                        "{context} serialized-last-write displaced keys are actually identical"
                    ));
                }
                Ok(())
            }
        }
        (
            DuplicateTrsKeys::Quaternion(keys),
            EmptyTrsBindingKind::Rotation,
            TrackValues::Rotation(values),
        ) => {
            if binding.relation == DuplicateTrsRelation::Identical {
                validate_duplicate_quaternion_keys(keys, values, canonical, duration, context)
            } else {
                validate_displaced_quaternion_keys(keys, duration, context)?;
                if duplicate_quaternion_keys_match_channel(keys, values, canonical, context)? {
                    return invalid(format!(
                        "{context} serialized-last-write displaced keys are actually identical"
                    ));
                }
                Ok(())
            }
        }
        _ => invalid(format!(
            "{context} key component shape/kind differs from its canonical channel"
        )),
    }
}

pub(super) fn validate_duplicate_vec3_keys(
    keys: &[ExactVec3Key],
    values: &[[f64; 3]],
    canonical: &AnimationChannel,
    duration: f64,
    context: &str,
) -> Result<()> {
    validate_displaced_vec3_keys(keys, duration, context)?;
    if !duplicate_vec3_keys_match_channel(keys, values, canonical, context)? {
        return invalid(format!(
            "{context} complete keys differ from the canonical runtime channel"
        ));
    }
    Ok(())
}

pub(super) fn validate_duplicate_quaternion_keys(
    keys: &[ExactQuaternionKey],
    values: &[[f64; 4]],
    canonical: &AnimationChannel,
    duration: f64,
    context: &str,
) -> Result<()> {
    validate_displaced_quaternion_keys(keys, duration, context)?;
    if !duplicate_quaternion_keys_match_channel(keys, values, canonical, context)? {
        return invalid(format!(
            "{context} complete keys differ from the canonical runtime channel"
        ));
    }
    Ok(())
}

pub(super) fn validate_displaced_vec3_keys(keys: &[ExactVec3Key], duration: f64, context: &str) -> Result<()> {
    if keys.is_empty() {
        return invalid(format!("{context} has no duplicate keys"));
    }
    validate_times(
        &keys.iter().map(|key| key.time).collect::<Vec<_>>(),
        duration,
        context,
    )?;
    if keys
        .iter()
        .flat_map(|key| key.value)
        .any(|component| !component.is_finite())
    {
        return invalid(format!("{context} displaced vec3 values are non-finite"));
    }
    for tangents in [
        exact_vec3_tangents(keys, true, context)?,
        exact_vec3_tangents(keys, false, context)?,
    ] {
        if tangents
            .into_iter()
            .flatten()
            .flat_map(|value| value)
            .any(|component| !component.is_finite())
        {
            return invalid(format!("{context} displaced vec3 tangents are non-finite"));
        }
    }
    exact_tangent_modes(keys.iter().map(|key| key.tangent_mode), context)?;
    Ok(())
}

pub(super) fn validate_displaced_quaternion_keys(
    keys: &[ExactQuaternionKey],
    duration: f64,
    context: &str,
) -> Result<()> {
    if keys.is_empty() {
        return invalid(format!("{context} has no duplicate keys"));
    }
    validate_times(
        &keys.iter().map(|key| key.time).collect::<Vec<_>>(),
        duration,
        context,
    )?;
    if keys
        .iter()
        .flat_map(|key| key.value)
        .any(|component| !component.is_finite())
    {
        return invalid(format!(
            "{context} displaced quaternion values are non-finite"
        ));
    }
    for tangents in [
        exact_quaternion_tangents(keys, true, context)?,
        exact_quaternion_tangents(keys, false, context)?,
    ] {
        if tangents
            .into_iter()
            .flatten()
            .flat_map(|value| value)
            .any(|component| !component.is_finite())
        {
            return invalid(format!(
                "{context} displaced quaternion tangents are non-finite"
            ));
        }
    }
    exact_tangent_modes(keys.iter().map(|key| key.tangent_mode), context)?;
    Ok(())
}
