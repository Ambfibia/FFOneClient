use super::*;

pub(super) fn animation_metadata_mismatch_detail(model: &NativeModel, glb: &[u8]) -> Result<String> {
    let parsed = ParsedGlb::parse(glb)?;
    let source_standard = model
        .animations
        .iter()
        .filter(|clip| !clip.channels.is_empty())
        .collect::<Vec<_>>();
    let emitted_standard = optional_root_array(&parsed.document, "animations")?;
    if source_standard.len() != emitted_standard.len() {
        return Ok(format!(
            "standard clip count {} != {}",
            source_standard.len(),
            emitted_standard.len()
        ));
    }
    for (index, (source, emitted)) in source_standard
        .iter()
        .zip(emitted_standard.iter())
        .enumerate()
    {
        let emitted_name = required_string(emitted.get("name"), "standard animation name")?;
        if source.name != emitted_name {
            return Ok(format!(
                "animations[{index}].name {:?} != {:?}",
                source.name, emitted_name
            ));
        }
        let source_metadata = serde_json::to_value(&source.metadata).map_err(|error| {
            ModelError::Invalid(format!(
                "cannot serialize source animation metadata for diagnostic: {error}"
            ))
        })?;
        let emitted_metadata = emitted.pointer("/extras/nonTrs").ok_or_else(|| {
            ModelError::Invalid("standard animation has no nonTrs metadata".into())
        })?;
        if let Some(detail) = first_json_difference(
            &source_metadata,
            emitted_metadata,
            &format!("animations[{index}].metadata"),
        ) {
            return Ok(detail);
        }
    }

    let source_metadata_only = model
        .animations
        .iter()
        .filter(|clip| clip.channels.is_empty())
        .collect::<Vec<_>>();
    let emitted_metadata_only = required_array_value(
        parsed
            .document
            .pointer("/extras/ffone/metadataOnlyAnimations"),
        "metadata-only animation list",
    )?;
    if source_metadata_only.len() != emitted_metadata_only.len() {
        return Ok(format!(
            "metadata-only clip count {} != {}",
            source_metadata_only.len(),
            emitted_metadata_only.len()
        ));
    }
    for (index, (source, emitted)) in source_metadata_only
        .iter()
        .zip(emitted_metadata_only.iter())
        .enumerate()
    {
        let source_value = serde_json::json!({
            "name": source.name,
            "duration": source.duration,
            "declaredDuration": source.declared_duration,
            "keyedDuration": source.keyed_duration,
            "eventDuration": source.event_duration,
            "sampleRate": source.sample_rate,
            "wrapMode": source.wrap_mode,
            "loop": source.looped,
            "metadata": source.metadata,
        });
        if let Some(detail) = first_json_difference(
            &source_value,
            emitted,
            &format!("metadataOnlyAnimations[{index}]"),
        ) {
            return Ok(detail);
        }
    }
    Ok("canonical byte streams differ after structurally identical metadata".to_owned())
}

pub(super) fn animation_metadata_from_model(model: &NativeModel) -> Result<Vec<u8>> {
    let standard = model
        .animations
        .iter()
        .filter(|clip| !clip.channels.is_empty())
        .collect::<Vec<_>>();
    let metadata_only = model
        .animations
        .iter()
        .filter(|clip| clip.channels.is_empty())
        .collect::<Vec<_>>();
    let mut out = Canonical::default();
    out.tag("animation-metadata-v1");
    out.len(standard.len())?;
    for clip in standard {
        out.string(&clip.name);
        let metadata = serde_json::to_value(&clip.metadata).map_err(|error| {
            ModelError::Invalid(format!(
                "cannot canonicalize source animation metadata: {error}"
            ))
        })?;
        write_canonical_json(&mut out, &metadata)?;
    }
    out.len(metadata_only.len())?;
    for clip in metadata_only {
        out.string(&clip.name);
        out.f64(clip.duration);
        out.optional_f64(clip.declared_duration);
        out.optional_f64(clip.keyed_duration);
        out.optional_f64(clip.event_duration);
        out.optional_f64(clip.sample_rate);
        out.optional_i32(clip.wrap_mode);
        out.bool(clip.looped);
        let metadata = serde_json::to_value(&clip.metadata).map_err(|error| {
            ModelError::Invalid(format!(
                "cannot canonicalize metadata-only source clip: {error}"
            ))
        })?;
        write_canonical_json(&mut out, &metadata)?;
    }
    Ok(out.0)
}

pub(super) fn animation_metadata_from_glb(glb: &ParsedGlb<'_>) -> Result<Vec<u8>> {
    let standard = optional_root_array(&glb.document, "animations")?;
    let metadata_only = required_array_value(
        glb.document.pointer("/extras/ffone/metadataOnlyAnimations"),
        "metadata-only animation list",
    )?;
    let mut out = Canonical::default();
    out.tag("animation-metadata-v1");
    out.len(standard.len())?;
    for clip in standard {
        out.string(required_string(
            clip.get("name"),
            "standard animation name",
        )?);
        let metadata = clip.pointer("/extras/nonTrs").ok_or_else(|| {
            ModelError::Invalid("semantic proof standard animation has no nonTrs metadata".into())
        })?;
        write_canonical_json(&mut out, metadata)?;
    }
    out.len(metadata_only.len())?;
    for clip in metadata_only {
        out.string(required_string(
            clip.get("name"),
            "metadata-only animation name",
        )?);
        out.f64(required_f64(
            clip.get("duration"),
            "metadata-only animation duration",
        )?);
        out.optional_f64(optional_f64(clip.get("declaredDuration"))?);
        out.optional_f64(optional_f64(clip.get("keyedDuration"))?);
        out.optional_f64(optional_f64(clip.get("eventDuration"))?);
        out.optional_f64(optional_f64(clip.get("sampleRate"))?);
        out.optional_i32(optional_i32(clip.get("wrapMode"))?);
        out.bool(required_bool(
            clip.get("loop"),
            "metadata-only animation loop",
        )?);
        write_canonical_json(
            &mut out,
            clip.get("metadata").ok_or_else(|| {
                ModelError::Invalid("metadata-only animation has no metadata".into())
            })?,
        )?;
    }
    Ok(out.0)
}
