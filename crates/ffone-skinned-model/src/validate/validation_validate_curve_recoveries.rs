use super::*;

pub(super) fn validate_time_recoveries(
    model: &NativeModel,
    clip_index: usize,
    clip: &crate::AnimationClip,
) -> Result<()> {
    let hierarchy_paths = (0..model.nodes.len())
        .map(|node| model_node_path(model, node))
        .collect::<Vec<_>>();
    let mut identities = BTreeSet::new();
    let mut previous_order = None;
    for (recovery_index, recovery) in clip.metadata.time_recoveries.iter().enumerate() {
        let context = format!("animation {clip_index} time recovery {recovery_index}");
        validate_metadata_target(
            &hierarchy_paths,
            model.nodes.len(),
            recovery.target_node,
            &recovery.target_path,
            &context,
        )?;
        let Some(rank) = trs_source_rank(recovery.kind, recovery.source_encoding) else {
            return invalid(format!("{context} uses an impossible source encoding"));
        };
        if recovery.source_encoding != EmptyTrsSourceEncoding::Plain {
            return invalid(format!(
                "{context} recovery is only proven for plain curves"
            ));
        }
        let identity = (
            recovery.kind,
            recovery.source_encoding,
            recovery.source_index,
        );
        if !identities.insert(identity) {
            return invalid(format!("{context} repeats a recovered source identity"));
        }
        let order = (rank, recovery.source_index);
        if previous_order.is_some_and(|previous| previous >= order) {
            return invalid(format!(
                "animation {clip_index} timeRecoveries are not in canonical source order"
            ));
        }
        previous_order = Some(order);

        validate_recovery_times(recovery, clip.duration, &context)?;
        if !same_optional_f64_bits(recovery.source_sample_rate, clip.sample_rate)
            || !same_optional_f64_bits(recovery.reference.sample_rate, recovery.source_sample_rate)
        {
            return invalid(format!(
                "{context} sample rates do not exactly match source clip/reference"
            ));
        }
        for (label, sample_rate) in [
            ("sourceSampleRate", recovery.source_sample_rate),
            ("reference.sampleRate", recovery.reference.sample_rate),
        ] {
            if sample_rate.is_some_and(|value| {
                !value.is_finite() || !(value as f32).is_finite() || value <= 0.0
            }) {
                return invalid(format!("{context} {label} is invalid"));
            }
        }
        validate_true_asset_name("time recovery reference asset", &recovery.reference.asset)?;
        validate_true_asset_name(
            "time recovery reference animation",
            &recovery.reference.clip_name,
        )?;
        if recovery.reference.path_id <= 0 {
            return invalid(format!("{context} reference pathId is invalid"));
        }
        if recovery.reference.kind != recovery.kind
            || recovery.reference.path != recovery.target_path
            || recovery.reference.source_encoding != recovery.source_encoding
        {
            return invalid(format!(
                "{context} reference kind/path/encoding differs from recovered curve"
            ));
        }
        if !recovery.proof.exact_key_payload_excluding_time
            || !recovery.proof.exact_path
            || !recovery.proof.exact_sample_rate
            || recovery.proof.unique_recovered_time_vector_count != 1
            || recovery.proof.matching_reference_count == 0
        {
            return invalid(format!("{context} proof is incomplete or ambiguous"));
        }

        let canonical = clip
            .channels
            .iter()
            .filter(|channel| {
                channel_kind(channel) == recovery.kind
                    && channel.target_node == recovery.target_node
                    && channel.source_encoding == recovery.source_encoding
                    && channel.source_index == recovery.source_index
            })
            .collect::<Vec<_>>();
        let [canonical] = canonical.as_slice() else {
            return invalid(format!(
                "{context} does not resolve to exactly one canonical runtime channel"
            ));
        };
        if canonical.source_key_count as usize != recovery.recovered_times.len()
            || !canonical.duplicate_keys.is_empty()
            || canonical.source_key_indices.len() != recovery.recovered_times.len()
            || canonical.source_key_indices.len() != canonical.times.len()
        {
            return invalid(format!(
                "{context} sourceKeyCount/channel alignment differs from recoveredTimes"
            ));
        }
        for (&source_key_index, &time) in canonical.source_key_indices.iter().zip(&canonical.times)
        {
            let recovered = recovery
                .recovered_times
                .get(source_key_index as usize)
                .copied();
            if recovered.is_none_or(|recovered| recovered.to_bits() != time.to_bits()) {
                return invalid(format!(
                    "{context} recoveredTimes do not align with canonical sampler keys"
                ));
            }
        }
        for duplicate in &canonical.duplicate_keys {
            let recovered = recovery
                .recovered_times
                .get(duplicate.source_key_index as usize)
                .copied();
            if recovered.is_none_or(|recovered| {
                !same_duration(recovered, exact_payload_time(&duplicate.key))
            }) {
                return invalid(format!(
                    "{context} recoveredTimes do not align with duplicate source keys"
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn validate_curve_recoveries(
    model: &NativeModel,
    clip_index: usize,
    clip: &crate::AnimationClip,
) -> Result<()> {
    let hierarchy_paths = (0..model.nodes.len())
        .map(|node| model_node_path(model, node))
        .collect::<Vec<_>>();
    let mut base_identities = clip
        .channels
        .iter()
        .map(|channel| {
            (
                channel_kind(channel),
                channel.source_encoding,
                channel.source_index,
            )
        })
        .chain(
            clip.metadata
                .empty_trs_bindings
                .iter()
                .map(|binding| (binding.kind, binding.source_encoding, binding.source_index)),
        )
        .chain(
            clip.metadata
                .duplicate_trs_bindings
                .iter()
                .map(|binding| (binding.kind, binding.source_encoding, binding.source_index)),
        )
        .collect::<BTreeSet<_>>();
    let time_recovery_identities = clip
        .metadata
        .time_recoveries
        .iter()
        .map(|recovery| {
            (
                recovery.kind,
                recovery.source_encoding,
                recovery.source_index,
            )
        })
        .collect::<BTreeSet<_>>();
    let mut previous_order = None;
    for (recovery_index, recovery) in clip.metadata.curve_recoveries.iter().enumerate() {
        let context = format!("animation {clip_index} curve recovery {recovery_index}");
        validate_metadata_target(
            &hierarchy_paths,
            model.nodes.len(),
            recovery.target_node,
            &recovery.target_path,
            &context,
        )?;
        let Some(rank) = trs_source_rank(recovery.kind, recovery.source_encoding) else {
            return invalid(format!("{context} uses an impossible source encoding"));
        };
        if recovery.source_encoding != EmptyTrsSourceEncoding::Plain
            || recovery.source.source_encoding != EmptyTrsSourceEncoding::Plain
        {
            return invalid(format!(
                "{context} recovery is only proven for plain curves"
            ));
        }
        let expected_field = curve_source_field(recovery.kind);
        if recovery.source.field != expected_field
            || recovery.reference.field != expected_field
            || recovery.reference.kind != recovery.kind
            || recovery.reference.path != recovery.target_path
            || recovery.reference.source_encoding != recovery.source_encoding
        {
            return invalid(format!(
                "{context} source/reference identity differs from the recovered curve"
            ));
        }
        let source_indices = &recovery.source.source_indices;
        if source_indices.len() < 2
            || source_indices.windows(2).any(|pair| pair[0] >= pair[1])
            || recovery.source.source_target_curve_count != source_indices.len() as u64
        {
            return invalid(format!(
                "{context} source indices/count are not strict and complete"
            ));
        }
        let order = (rank, source_indices[0]);
        if previous_order.is_some_and(|previous| previous >= order) {
            return invalid(format!(
                "animation {clip_index} curveRecoveries are not in canonical source order"
            ));
        }
        // Serialized curves for different targets may interleave. Canonical
        // group order follows first occurrence; individual identities are
        // still checked for uniqueness and complete source coverage.
        previous_order = Some(order);

        if !same_optional_f64_bits(recovery.source_sample_rate, clip.sample_rate)
            || !same_optional_f64_bits(recovery.reference.sample_rate, recovery.source_sample_rate)
        {
            return invalid(format!(
                "{context} sample rates do not exactly match source clip/reference"
            ));
        }
        for (label, sample_rate) in [
            ("sourceSampleRate", recovery.source_sample_rate),
            ("reference.sampleRate", recovery.reference.sample_rate),
        ] {
            if sample_rate.is_some_and(|value| {
                !value.is_finite() || !(value as f32).is_finite() || value <= 0.0
            }) {
                return invalid(format!("{context} {label} is invalid"));
            }
        }
        validate_true_asset_name(
            "curve recovery reference animation",
            &recovery.reference.clip_name,
        )?;

        let canonical_channel = clip
            .channels
            .iter()
            .filter(|channel| channel_kind(channel) == recovery.kind)
            .nth(recovery.canonical.track_index as usize)
            .ok_or_else(|| {
                crate::ModelError::Invalid(format!(
                    "{context} canonical trackIndex is out of range"
                ))
            })?;
        if canonical_channel.target_node != recovery.target_node
            || canonical_channel.source_index != recovery.canonical.source_index
            || canonical_channel.source_encoding != recovery.canonical.source_encoding
            || recovery.canonical.path != recovery.target_path
            || recovery.canonical.interpolation != canonical_channel.interpolation
            || recovery.canonical.source_key_count != canonical_channel.source_key_count
            || recovery.canonical.source_encoding != EmptyTrsSourceEncoding::Plain
        {
            return invalid(format!(
                "{context} canonical provenance differs from the runtime channel"
            ));
        }
        let canonical_payload = validate_curve_recovery_canonical_track(
            &recovery.canonical,
            recovery.kind,
            canonical_channel,
            clip.duration,
            &format!("{context} canonical"),
        )?;
        let canonical_identity = (
            recovery.kind,
            recovery.canonical.source_encoding,
            recovery.canonical.source_index,
        );
        if !source_indices.contains(&recovery.canonical.source_index)
            || !base_identities.contains(&canonical_identity)
            || time_recovery_identities.contains(&canonical_identity)
        {
            return invalid(format!(
                "{context} canonical source is absent, outside sourceIndices, or multiply recovered"
            ));
        }

        let expected_rejected = source_indices
            .iter()
            .copied()
            .filter(|source_index| *source_index != recovery.canonical.source_index)
            .collect::<Vec<_>>();
        if recovery.rejected.len() != expected_rejected.len() {
            return invalid(format!("{context} rejected provenance is incomplete"));
        }
        let mut source_payloads =
            vec![(recovery.canonical.source_index, canonical_payload.clone())];
        let mut has_conflict = false;
        for (rejected_index, (track, expected_source_index)) in
            recovery.rejected.iter().zip(expected_rejected).enumerate()
        {
            if track.source_index != expected_source_index
                || track.path != recovery.target_path
                || track.source_encoding != EmptyTrsSourceEncoding::Plain
            {
                return invalid(format!(
                    "{context} rejected track {rejected_index} identity/order differs"
                ));
            }
            let payload = validate_curve_recovery_track(
                track,
                recovery.kind,
                clip.duration,
                &format!("{context} rejected track {rejected_index}"),
            )?;
            has_conflict |= payload != canonical_payload;
            let identity = (recovery.kind, track.source_encoding, track.source_index);
            if base_identities.contains(&identity) || !base_identities.insert(identity) {
                return invalid(format!(
                    "{context} rejected track {rejected_index} repeats a published source identity"
                ));
            }
            source_payloads.push((track.source_index, payload));
        }
        if recovery.rejected.is_empty() || !has_conflict {
            return invalid(format!(
                "{context} has no rejected conflicting source curve"
            ));
        }

        if !recovery.proof.exact_path
            || !recovery.proof.exact_sample_rate
            || !recovery.proof.all_source_curves_constant
            || !recovery.proof.canonical_matches_reference
            || recovery.proof.unique_canonical_candidate_count != 1
            || recovery.proof.matching_reference_count != 1
            || recovery.proof.source_target_curve_count != source_indices.len() as u64
        {
            return invalid(format!(
                "{context} proof differs from recomputed strict requirements"
            ));
        }

        let mut matching_pairs = Vec::new();
        for (candidate_clip_index, candidate_clip) in model.animations.iter().enumerate() {
            if candidate_clip_index == clip_index
                || !same_optional_f64_bits(candidate_clip.sample_rate, recovery.source_sample_rate)
                || native_target_binding_count(candidate_clip, recovery.kind, recovery.target_node)
                    != 1
            {
                continue;
            }
            for candidate in candidate_clip.channels.iter().filter(|channel| {
                channel_kind(channel) == recovery.kind
                    && channel.target_node == recovery.target_node
                    && channel.source_encoding == EmptyTrsSourceEncoding::Plain
            }) {
                let Some(candidate_payload) = constant_channel_payload(candidate) else {
                    continue;
                };
                for (target_source_index, target_payload) in &source_payloads {
                    if target_payload == &candidate_payload {
                        matching_pairs.push((
                            *target_source_index,
                            candidate_clip_index,
                            candidate.source_index,
                        ));
                    }
                }
            }
        }
        let declared_match = matching_pairs.iter().any(
            |(target_source_index, candidate_clip_index, candidate_source_index)| {
                *target_source_index == recovery.canonical.source_index
                    && *candidate_source_index == recovery.reference.source_index
                    && model.animations[*candidate_clip_index].name == recovery.reference.clip_name
            },
        );
        if matching_pairs.len() != 1 || !declared_match {
            return invalid(format!(
                "{context} does not have one unique candidate/reference authority"
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_curve_recovery_canonical_track(
    track: &AnimationCurveRecoveryCanonicalTrack,
    kind: EmptyTrsBindingKind,
    canonical: &AnimationChannel,
    duration: f64,
    context: &str,
) -> Result<ConstantCurvePayload> {
    if !track.duplicate_keys.is_empty() {
        return invalid(format!("{context} contains duplicate source keys"));
    }
    match (&track.keys, kind, &canonical.values) {
        (
            DuplicateTrsKeys::Vec3(keys),
            EmptyTrsBindingKind::Translation,
            TrackValues::Translation(values),
        )
        | (DuplicateTrsKeys::Vec3(keys), EmptyTrsBindingKind::Scale, TrackValues::Scale(values)) => {
            validate_duplicate_vec3_keys(keys, values, canonical, duration, context)?;
        }
        (
            DuplicateTrsKeys::Quaternion(keys),
            EmptyTrsBindingKind::Rotation,
            TrackValues::Rotation(values),
        ) => {
            validate_duplicate_quaternion_keys(keys, values, canonical, duration, context)?;
        }
        _ => return invalid(format!("{context} key shape differs from its kind/channel")),
    }
    validate_constant_curve_keys(
        &track.keys,
        &track.duplicate_keys,
        track.source_key_count,
        track.interpolation,
        duration,
        context,
    )
}

pub(super) fn validate_curve_recovery_track(
    track: &AnimationCurveRecoveryTrack,
    kind: EmptyTrsBindingKind,
    duration: f64,
    context: &str,
) -> Result<ConstantCurvePayload> {
    if !matches!(
        (&track.keys, kind),
        (
            DuplicateTrsKeys::Vec3(_),
            EmptyTrsBindingKind::Translation | EmptyTrsBindingKind::Scale
        ) | (
            DuplicateTrsKeys::Quaternion(_),
            EmptyTrsBindingKind::Rotation
        )
    ) {
        return invalid(format!("{context} key shape differs from its kind"));
    }
    validate_constant_curve_keys(
        &track.keys,
        &track.duplicate_keys,
        track.source_key_count,
        track.interpolation,
        duration,
        context,
    )
}

pub(super) fn validate_constant_curve_keys(
    keys: &DuplicateTrsKeys,
    duplicate_keys: &[DuplicateAnimationKey],
    source_key_count: u32,
    interpolation: Interpolation,
    duration: f64,
    context: &str,
) -> Result<ConstantCurvePayload> {
    if !duplicate_keys.is_empty() || keys.is_empty() || source_key_count as usize != keys.len() {
        return invalid(format!(
            "{context} is not a complete non-duplicate constant curve"
        ));
    }
    match keys {
        DuplicateTrsKeys::Vec3(keys) => {
            let times = keys.iter().map(|key| key.time).collect::<Vec<_>>();
            validate_times(&times, duration, context)?;
            if keys
                .iter()
                .enumerate()
                .any(|(index, key)| key.source_key_index as usize != index)
            {
                return invalid(format!(
                    "{context} key provenance is not complete source order"
                ));
            }
            let expected_interpolation = interpolation_from_vec3_keys(keys, context)?;
            if interpolation != expected_interpolation {
                return invalid(format!(
                    "{context} interpolation differs from exact tangents"
                ));
            }
            let mut payload = None;
            for key in keys {
                finite("curve recovery vec3 value", &key.value)?;
                if let Some(value) = &key.in_tangent {
                    finite("curve recovery vec3 in tangent", value)?;
                }
                if let Some(value) = &key.out_tangent {
                    finite("curve recovery vec3 out tangent", value)?;
                }
                let current = ConstantCurvePayload::Vec3 {
                    value: key.value,
                    in_tangent: key.in_tangent,
                    out_tangent: key.out_tangent,
                    tangent_mode: key.tangent_mode,
                };
                if payload
                    .as_ref()
                    .is_some_and(|previous| previous != &current)
                {
                    return invalid(format!("{context} is not constant"));
                }
                payload.get_or_insert(current);
            }
            Ok(payload.expect("non-empty keys"))
        }
        DuplicateTrsKeys::Quaternion(keys) => {
            let times = keys.iter().map(|key| key.time).collect::<Vec<_>>();
            validate_times(&times, duration, context)?;
            if keys
                .iter()
                .enumerate()
                .any(|(index, key)| key.source_key_index as usize != index)
            {
                return invalid(format!(
                    "{context} key provenance is not complete source order"
                ));
            }
            let expected_interpolation = interpolation_from_quaternion_keys(keys, context)?;
            if interpolation != expected_interpolation {
                return invalid(format!(
                    "{context} interpolation differs from exact tangents"
                ));
            }
            let mut payload = None;
            for key in keys {
                finite("curve recovery quaternion value", &key.value)?;
                if let Some(value) = &key.in_tangent {
                    finite("curve recovery quaternion in tangent", value)?;
                }
                if let Some(value) = &key.out_tangent {
                    finite("curve recovery quaternion out tangent", value)?;
                }
                let current = ConstantCurvePayload::Quaternion {
                    value: key.value,
                    in_tangent: key.in_tangent,
                    out_tangent: key.out_tangent,
                    tangent_mode: key.tangent_mode,
                };
                if payload
                    .as_ref()
                    .is_some_and(|previous| previous != &current)
                {
                    return invalid(format!("{context} is not constant"));
                }
                payload.get_or_insert(current);
            }
            Ok(payload.expect("non-empty keys"))
        }
    }
}

pub(super) fn validate_recovery_times(
    recovery: &AnimationTimeRecovery,
    duration: f64,
    context: &str,
) -> Result<()> {
    if recovery.original_times.is_empty()
        || recovery.original_times.len() != recovery.recovered_times.len()
        || recovery.original_times.iter().any(|value| {
            !value.is_finite()
                || !(*value as f32).is_finite()
                || *value < 0.0
                || *value > duration + 1.0e-6
        })
    {
        return invalid(format!("{context} originalTimes are invalid"));
    }
    if recovery
        .original_times
        .windows(2)
        .all(|pair| pair[0] < pair[1])
    {
        return invalid(format!(
            "{context} originalTimes were already strict and need no recovery"
        ));
    }
    validate_times(&recovery.recovered_times, duration, context)
}

pub(super) fn validate_channel(
    model: &NativeModel,
    duration: f64,
    channel: &AnimationChannel,
    context: &str,
) -> Result<()> {
    if channel.target_node as usize >= model.nodes.len() || channel.times.is_empty() {
        return invalid(format!("{context} has invalid target/times"));
    }
    validate_times(&channel.times, duration, context)?;
    let value_count = validate_track(&channel.values, true)?;
    if value_count != channel.times.len() {
        return invalid(format!("{context} key count mismatch"));
    }
    validate_tangent_modes(&channel.tangent_modes, value_count, context)?;
    validate_track_tangents(
        channel.interpolation,
        &channel.values,
        channel.in_tangents.as_ref(),
        channel.out_tangents.as_ref(),
        value_count,
        context,
    )
}

pub(super) fn validate_float_curve(
    model: &NativeModel,
    duration: f64,
    curve: &FloatCurve,
    context: &str,
) -> Result<()> {
    if curve.target_node as usize >= model.nodes.len() {
        return invalid(format!("{context} has invalid target"));
    }
    let hierarchy_paths = (0..model.nodes.len())
        .map(|node| model_node_path(model, node))
        .collect::<Vec<_>>();
    validate_metadata_target(
        &hierarchy_paths,
        model.nodes.len(),
        curve.target_node,
        &curve.target_path,
        context,
    )?;
    if curve.class_id <= 0 {
        return invalid(format!("{context} has invalid Unity class id"));
    }
    if curve.source_key_count as usize != curve.times.len()
        || curve.source_key_indices.len() != curve.times.len()
        || curve
            .source_key_indices
            .iter()
            .enumerate()
            .any(|(index, source)| usize::try_from(*source).ok() != Some(index))
    {
        return invalid(format!(
            "{context} does not preserve exact float source-key identities"
        ));
    }
    valid_name("float curve property", &curve.property)?;
    validate_float_curve_times(&curve.times, duration, context)?;
    finite("float curve values", &curve.values)?;
    if curve.values.len() != curve.times.len() {
        return invalid(format!("{context} key count mismatch"));
    }
    validate_tangent_modes(&curve.tangent_modes, curve.values.len(), context)?;
    match curve.interpolation {
        Interpolation::CubicSpline => {
            let (Some(input), Some(output)) = (&curve.in_tangents, &curve.out_tangents) else {
                return invalid(format!("{context} CUBICSPLINE tangents are missing"));
            };
            finite("float curve in tangents", input)?;
            finite("float curve out tangents", output)?;
            if input.len() != curve.values.len() || output.len() != curve.values.len() {
                return invalid(format!("{context} tangent count mismatch"));
            }
        }
        Interpolation::Linear | Interpolation::Step => {
            if curve.in_tangents.is_some() || curve.out_tangents.is_some() {
                return invalid(format!("{context} non-cubic curve contains tangents"));
            }
        }
    }
    Ok(())
}

pub(super) fn validate_track_tangents(
    interpolation: Interpolation,
    values: &TrackValues,
    input: Option<&TrackValues>,
    output: Option<&TrackValues>,
    count: usize,
    context: &str,
) -> Result<()> {
    match interpolation {
        Interpolation::CubicSpline => {
            let (Some(input), Some(output)) = (input, output) else {
                return invalid(format!("{context} CUBICSPLINE tangents are missing"));
            };
            if track_kind(values) != track_kind(input) || track_kind(values) != track_kind(output) {
                return invalid(format!("{context} tangent track type mismatch"));
            }
            if validate_track(input, false)? != count || validate_track(output, false)? != count {
                return invalid(format!("{context} tangent count mismatch"));
            }
        }
        Interpolation::Linear | Interpolation::Step => {
            if input.is_some() || output.is_some() {
                return invalid(format!("{context} non-cubic channel contains tangents"));
            }
        }
    }
    Ok(())
}

pub(super) fn validate_track(values: &TrackValues, normalize_rotations: bool) -> Result<usize> {
    match values {
        TrackValues::Translation(values) | TrackValues::Scale(values) => {
            finite("animation vec3", values.iter().flatten())?;
            Ok(values.len())
        }
        TrackValues::Rotation(values) => {
            for value in values {
                finite("animation rotation", value)?;
                if normalize_rotations {
                    normalized_quat(value, "animation rotation")?;
                }
            }
            Ok(values.len())
        }
    }
}

pub(super) fn validate_tangent_modes(values: &[i32], count: usize, context: &str) -> Result<()> {
    // Some legacy serialized Keyframe layouts have slopes but no tangentMode
    // field. Empty therefore means "not present in the source"; a partially
    // populated vector would be lossy and remains invalid.
    if !values.is_empty() && values.len() != count {
        return invalid(format!("{context} tangentMode count mismatch"));
    }
    Ok(())
}

pub(super) fn validate_times(values: &[f64], duration: f64, context: &str) -> Result<()> {
    if values.iter().any(|value| {
        !value.is_finite()
            || !(*value as f32).is_finite()
            || *value < 0.0
            || *value > duration + 1.0e-6
    }) || values.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return invalid(format!("{context} times are invalid"));
    }
    Ok(())
}

pub(super) fn validate_float_curve_times(values: &[f64], duration: f64, context: &str) -> Result<()> {
    // Unity material curves can contain an exact pre-roll key before t=0.
    // Keep it in metadata so UV/material animation playback can reproduce the
    // serialized curve instead of silently clamping or dropping that key.
    if values.iter().any(|value| {
        !value.is_finite() || !(*value as f32).is_finite() || *value > duration + 1.0e-6
    }) || values.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return invalid(format!("{context} times are invalid"));
    }
    Ok(())
}

pub(super) fn validate_asset_reference(value: &NativeAssetReference, context: &str) -> Result<()> {
    valid_name("asset reference kind", &value.kind)?;
    valid_name("asset reference name", &value.name)?;
    if value
        .uri
        .as_ref()
        .is_some_and(|uri| uri.trim().is_empty() || uri.chars().any(char::is_control))
    {
        return invalid(format!("{context} asset reference URI is invalid"));
    }
    Ok(())
}
