use super::*;

pub(super) fn build_jobs(data: &CharacterCreationData, cli: &Cli) -> Vec<Job> {
    let mut jobs = Vec::new();
    for item in &data.avatar_items_document().items {
        if !is_wearable(item.category)
            || cli
                .category
                .is_some_and(|category| category != item.category)
        {
            continue;
        }
        for (gender, visual) in [
            (AuditGender::Male, &item.male),
            (AuditGender::Female, &item.female),
        ] {
            if !gender_allowed(item.required_gender, gender)
                || matches!(
                    (cli.gender, gender),
                    (GenderFilter::Male, AuditGender::Female)
                        | (GenderFilter::Female, AuditGender::Male)
                )
            {
                continue;
            }
            jobs.push(Job {
                ordinal: jobs.len(),
                category: item.category,
                item_number: item.item_number,
                name: item.name.clone(),
                gender,
                declared_model_status: visual.model_status,
                declared_primary_status: visual.primary_texture.as_ref().map(|item| item.status),
                declared_secondary_status: visual
                    .secondary_texture
                    .as_ref()
                    .map(|item| item.status),
            });
        }
    }
    jobs.into_iter()
        .skip(cli.start)
        .take(cli.limit.unwrap_or(usize::MAX))
        .enumerate()
        .map(|(ordinal, mut job)| {
            job.ordinal = ordinal;
            job
        })
        .collect()
}

pub(super) fn blocked_result(
    job: &Job,
    look: &NativePlayerLook,
    target_kind: NativePlayerPartKind,
    error: String,
    frames: u32,
    started: Instant,
) -> ItemResult {
    let target = look.parts.iter().find(|part| part.kind == target_kind);
    item_result(
        job,
        target.map(|part| part.exact_route.clone()),
        target
            .and_then(|part| part.primary_texture.as_ref())
            .map(|texture| texture.path.clone()),
        target
            .and_then(|part| part.secondary_texture.as_ref())
            .map(|texture| texture.path.clone()),
        false,
        vec![error],
        Vec::new(),
        frames,
        started.elapsed(),
    )
}

pub(super) fn resolution_failure(job: &Job, error: String) -> ItemResult {
    item_result(
        job,
        None,
        None,
        None,
        false,
        vec![format!("look resolution failed: {error}")],
        Vec::new(),
        0,
        Duration::ZERO,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn item_result(
    job: &Job,
    target_route: Option<String>,
    expected_primary_texture: Option<String>,
    expected_secondary_texture: Option<String>,
    rendered: bool,
    failure_reasons: Vec<String>,
    surfaces: Vec<SurfaceEvidence>,
    frames: u32,
    elapsed: Duration,
) -> ItemResult {
    ItemResult {
        ordinal: job.ordinal,
        category: category_label(job.category).to_owned(),
        item_number: job.item_number,
        name: job.name.clone(),
        gender: job.gender,
        identity: job.identity(),
        declared_model_status: status_label(job.declared_model_status).to_owned(),
        declared_primary_status: job
            .declared_primary_status
            .map(status_label)
            .map(str::to_owned),
        declared_secondary_status: job
            .declared_secondary_status
            .map(status_label)
            .map(str::to_owned),
        target_route,
        expected_primary_texture,
        expected_secondary_texture,
        rendered,
        passed: failure_reasons.is_empty(),
        failure_reasons,
        surfaces,
        screenshot: None,
        capture_visible_pixels: None,
        capture_total_pixels: None,
        frames,
        elapsed_seconds: elapsed.as_secs_f64(),
    }
}

pub(super) fn finish_result(
    state: &mut AuditState,
    result: ItemResult,
    cache: &mut NativePlayerRigAssetCache,
) {
    let passed = result.passed;
    if !passed {
        eprintln!(
            "FAIL {}/{} {} {} {:?}: {}",
            state.cursor + 1,
            state.jobs.len(),
            result.category,
            result.item_number,
            result.gender,
            result.failure_reasons.join("; ")
        );
    }
    state.results.push(result);
    state.cursor += 1;
    state.transient_retries = 0;
    if state.cursor % 100 == 0 {
        cache.release_cached_handles();
    }
    if state.cursor % REPORT_CHECKPOINT_INTERVAL == 0 || !passed {
        if let Err(error) = write_report(state) {
            eprintln!("cannot write audit checkpoint: {error}");
        }
    }
    if state.cursor % 25 == 0 || !passed {
        let passed_count = state.results.iter().filter(|result| result.passed).count();
        println!(
            "equipment audit progress {}/{} passed={} failed={}",
            state.cursor,
            state.jobs.len(),
            passed_count,
            state.results.len() - passed_count
        );
    }
    state.phase = Phase::Start;
}

pub(super) const fn is_wearable(category: AvatarItemCategory) -> bool {
    matches!(
        category,
        AvatarItemCategory::Shirt
            | AvatarItemCategory::Pants
            | AvatarItemCategory::Shoes
            | AvatarItemCategory::Hat
            | AvatarItemCategory::Glasses
            | AvatarItemCategory::Back
    )
}

pub(super) const fn gender_allowed(required: u8, gender: AuditGender) -> bool {
    required == 0
        || matches!(
            (required, gender),
            (1, AuditGender::Male) | (2, AuditGender::Female)
        )
}

pub(super) const fn category_slot(
    category: AvatarItemCategory,
) -> Option<(CharacterEquipSlot0104, NativePlayerPartKind)> {
    Some(match category {
        AvatarItemCategory::Shirt => (
            CharacterEquipSlot0104::UpperBody,
            NativePlayerPartKind::Shirt,
        ),
        AvatarItemCategory::Pants => (
            CharacterEquipSlot0104::LowerBody,
            NativePlayerPartKind::Pants,
        ),
        AvatarItemCategory::Shoes => (CharacterEquipSlot0104::Foot, NativePlayerPartKind::Shoes),
        AvatarItemCategory::Hat => (CharacterEquipSlot0104::Head, NativePlayerPartKind::Hat),
        AvatarItemCategory::Glasses => {
            (CharacterEquipSlot0104::Face, NativePlayerPartKind::Glasses)
        }
        AvatarItemCategory::Back => (CharacterEquipSlot0104::Back, NativePlayerPartKind::Back),
        AvatarItemCategory::Face
        | AvatarItemCategory::Head
        | AvatarItemCategory::Vehicle
        | AvatarItemCategory::Weapon => return None,
    })
}

pub(super) const fn category_label(category: AvatarItemCategory) -> &'static str {
    match category {
        AvatarItemCategory::Back => "back",
        AvatarItemCategory::Glasses => "glasses",
        AvatarItemCategory::Hat => "hat",
        AvatarItemCategory::Pants => "pants",
        AvatarItemCategory::Shirt => "shirt",
        AvatarItemCategory::Shoes => "shoes",
        AvatarItemCategory::Face => "face",
        AvatarItemCategory::Head => "head",
        AvatarItemCategory::Vehicle => "vehicle",
        AvatarItemCategory::Weapon => "weapon",
    }
}

pub(super) const fn role_label(role: ActorSkinTextureRole) -> &'static str {
    match role {
        ActorSkinTextureRole::Primary => "primary",
        ActorSkinTextureRole::SecondarySkin => "secondary_skin",
        ActorSkinTextureRole::PreserveOverride => "preserve_override",
    }
}
