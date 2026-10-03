use super::*;

pub(super) fn prepare_tutorial_nano_gameplay_asset(
    asset_server: Res<AssetServer>,
    locator: Option<Res<AssetLocator>>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut assets: ResMut<TutorialNanoGameplayAssets>,
    mut state: ResMut<TutorialNanoGameplayState>,
    mut issues: ResMut<TutorialNanoGameplayIssueQueue>,
) {
    if !matches!(state.status, TutorialNanoGameplayStatus::Loading) || state.asset_contract_ready {
        return;
    }
    let Some(gltf_handle) = assets.gltf.clone() else {
        return;
    };
    if let Some(error) = gameplay_nano_asset_load_failure(&asset_server, &gltf_handle) {
        state.status = TutorialNanoGameplayStatus::Blocked(error.clone());
        issues.push(TutorialNanoGameplayIssue::AssetBlocked(error));
        return;
    }
    let Some(gltf) = gltfs.get(&gltf_handle) else {
        return;
    };
    if assets.sound_events.is_none()
        && let (Some(locator), Some(presentation)) =
            (locator.as_deref(), state.world_presentation.as_ref())
    {
        let sound_events = match locator
            .read(&presentation.model_path)
            .and_then(|bytes| parse_network_npc_animation_sound_events(&bytes))
        {
            Ok(events) => events,
            Err(error) => {
                let error = format!("gameplay Nano AnimationEvent audio is invalid: {error}");
                state.status = TutorialNanoGameplayStatus::Blocked(error.clone());
                issues.push(TutorialNanoGameplayIssue::AssetBlocked(error));
                return;
            }
        };
        assets.sound_events = Some(sound_events.into());
    }
    let skill_clip = state
        .world_presentation
        .as_ref()
        .and_then(WorldNanoGameplayPresentation::skill_clip)
        .unwrap_or(SKILL_CLIP);
    let required_clips = [CALL_CLIP, "stand1", "stand2", "stand3", skill_clip];
    let asset_clips = required_clips.map(|logical_clip| {
        gameplay_nano_asset_clip_name(
            state.loadout.map(|loadout| loadout.nano_id),
            state
                .world_presentation
                .as_ref()
                .map(|presentation| presentation.model_path.as_str()),
            logical_clip,
        )
    });
    let missing = required_clips
        .iter()
        .zip(asset_clips.iter())
        .filter(|(logical_name, asset_name)| {
            !gltf.named_animations.contains_key(**asset_name)
                && !(state.world_presentation.is_some()
                    && matches!(
                        **logical_name,
                        "stand2" | "stand3" | "skill1" | "skill2" | "skill3"
                    ))
        })
        .map(|(logical_name, _)| *logical_name)
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        let error = format!("exact gameplay Nano GLB is missing required clips: {missing:?}");
        state.status = TutorialNanoGameplayStatus::Blocked(error.clone());
        issues.push(TutorialNanoGameplayIssue::AssetBlocked(error));
        return;
    }
    if assets.graph.is_none() {
        let mut available_clips = required_clips
            .iter()
            .zip(asset_clips.iter())
            .filter_map(|(logical_name, asset_name)| {
                gltf.named_animations
                    .get(*asset_name)
                    .map(|clip| (*logical_name, clip.clone()))
            })
            .collect::<Vec<_>>();
        for name in ["withdraw", "discharge"] {
            if let Some(clip) = gltf.named_animations.get(name) {
                available_clips.push((name, clip.clone()));
            }
        }
        let clips = available_clips.iter().map(|(_, clip)| clip.clone());
        let (graph, nodes) = AnimationGraph::from_clips(clips);
        assets.nodes = available_clips
            .into_iter()
            .map(|(name, _)| name)
            .zip(nodes)
            .collect();
        assets.graph = Some(graphs.add(graph));
    }
    state.asset_contract_ready = true;
}

pub(super) fn gameplay_nano_asset_load_failure(
    asset_server: &AssetServer,
    handle: &Handle<Gltf>,
) -> Option<String> {
    match asset_server.load_state(handle.id()) {
        LoadState::Failed(error) => Some(format!("gameplay Nano GLB load failed: {error}")),
        _ => asset_server
            .get_recursive_dependency_load_state(handle.id())
            .and_then(|state| match state {
                RecursiveDependencyLoadState::Failed(error) => {
                    Some(format!("gameplay Nano GLB dependency load failed: {error}"))
                }
                _ => None,
            }),
    }
}
