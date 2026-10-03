use super::*;

pub(super) fn evaluate_ready_job(
    job: &Job,
    look: &NativePlayerLook,
    target_kind: NativePlayerPartKind,
    identity: &str,
    audit: &NativePlayerPreviewAudit,
    frames: u32,
    started: Instant,
) -> ItemResult {
    let target = look.parts.iter().find(|part| part.kind == target_kind);
    let target_route = target.map(|part| part.exact_route.clone());
    let expected_primary_texture = target.and_then(|part| part.primary_texture.as_ref());
    let expected_secondary_texture = target.and_then(|part| part.secondary_texture.as_ref());
    let expected_primary = expected_primary_texture.map(|texture| texture.path.clone());
    let expected_secondary = expected_secondary_texture.map(|texture| texture.path.clone());
    let surfaces = audit
        .surfaces
        .iter()
        .filter(|surface| {
            surface.kind == target_kind && target_route.as_ref() == Some(&surface.exact_route)
        })
        .map(|surface| SurfaceEvidence {
            kind: format!("{:?}", surface.kind).to_ascii_lowercase(),
            exact_route: surface.exact_route.clone(),
            material_true_name: surface.material_true_name.clone(),
            role: role_label(surface.role).to_owned(),
            bound_texture_path: surface.bound_texture_path.clone(),
            base_texture_assigned: surface.base_texture_assigned,
            source_main_texture: surface.source_main_texture.clone(),
            hidden: surface.hidden,
        })
        .collect::<Vec<_>>();
    let visible = surfaces
        .iter()
        .filter(|surface| !surface.hidden)
        .collect::<Vec<_>>();
    let mut failures = Vec::new();
    if audit.identity.as_deref() != Some(identity) || audit.gender != Some(job.gender.rig()) {
        failures.push("ready audit snapshot belongs to another preview generation".to_owned());
    }
    if target.is_none() {
        failures.push("resolved look has no target equipment part".to_owned());
    }
    if surfaces.is_empty() {
        failures.push("target equipment produced no audited material surfaces".to_owned());
    }
    if visible.is_empty() {
        failures.push("target equipment has no visible material surfaces".to_owned());
    }
    if !visible.iter().any(|surface| surface.base_texture_assigned) {
        failures.push("no visible target surface has a GPU base texture".to_owned());
    }
    if let Some(expected) = &expected_primary {
        if !visible.iter().any(|surface| {
            surface.bound_texture_path.as_ref() == Some(expected)
                || (surface.base_texture_assigned
                    && expected_primary_texture.is_some_and(|texture| {
                        surface
                            .source_main_texture
                            .as_deref()
                            .is_some_and(|source| {
                                source_texture_matches(source, &texture.contract.true_name)
                            })
                    }))
        }) {
            failures.push(format!(
                "declared primary texture {expected:?} never reached a visible material"
            ));
        }
    } else if !visible.iter().any(|surface| {
        surface
            .source_main_texture
            .as_deref()
            .is_some_and(meaningful_source_texture)
    }) {
        failures.push(
            "item has no table primary texture and no meaningful authored _MainTex".to_owned(),
        );
    }
    if let Some(expected) = &expected_secondary
        && !visible
            .iter()
            .any(|surface| surface.bound_texture_path.as_ref() == Some(expected))
    {
        failures.push(format!(
            "declared secondary texture {expected:?} never reached a visible material"
        ));
    }
    for surface in &visible {
        if surface.role == "primary" && !surface.base_texture_assigned {
            failures.push(format!(
                "visible primary material {:?} is using the implicit white slot",
                surface.material_true_name
            ));
        }
    }
    failures.sort();
    failures.dedup();
    item_result(
        job,
        target_route,
        expected_primary,
        expected_secondary,
        true,
        failures,
        surfaces,
        frames,
        started.elapsed(),
    )
}
