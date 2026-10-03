use super::*;

#[derive(Debug)]
pub(super) struct PreparedAnimation {
    pub(super) graph: Handle<AnimationGraph>,
    pub(super) node: AnimationNodeIndex,
    pub(super) fixed_sample_time: Option<f32>,
}

pub(super) fn prepare_selected_animation(
    mut state: ResMut<RuntimeState>,
    config: Res<PreviewConfig>,
    handles: Option<Res<ModelHandles>>,
    gltfs: Res<Assets<Gltf>>,
    clips: Res<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    shared: Res<SharedReport>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if state.terminal || state.gltf_inspected {
        return;
    }
    let Some(handles) = handles else {
        return;
    };
    let Some(gltf) = gltfs.get(&handles.gltf) else {
        return;
    };
    state.animations_loaded = gltf.animations.len() as u64;
    let mut exact_animation_names = gltf
        .named_animations
        .keys()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    exact_animation_names.sort();
    if let Some(evidence) = config.evidence.as_ref() {
        let mut expected = evidence.facts.standard_animation_names.clone();
        expected.sort();
        if exact_animation_names != expected || gltf.animations.len() != expected.len() {
            fail_runtime(
                &mut state,
                &shared,
                &mut app_exit,
                format!(
                    "loaded GLTF exact animation names differ: expected {expected:?}, got {exact_animation_names:?}"
                ),
            );
            return;
        }
    }
    state.exact_animation_names = exact_animation_names.clone();
    shared.update(|report| {
        report.animations = gltf.animations.len();
        report.exact_animation_names = exact_animation_names;
    });

    let Some(selection) = config.animation.as_deref() else {
        state.gltf_inspected = true;
        state.animation_resolved = true;
        return;
    };
    let resolved = if config.animation_exact_name {
        gltf.named_animations
            .get(selection)
            .cloned()
            .map(|handle| (handle, format!("name:{selection}")))
    } else if let Ok(index) = selection.parse::<usize>() {
        gltf.animations
            .get(index)
            .cloned()
            .map(|handle| (handle, format!("index:{index}")))
    } else {
        gltf.named_animations
            .get(selection)
            .cloned()
            .map(|handle| (handle, format!("name:{selection}")))
    };
    let Some((clip, label)) = resolved else {
        fail_runtime(
            &mut state,
            &shared,
            &mut app_exit,
            format!(
                "requested animation {selection:?} does not exist ({} clips, named: {})",
                gltf.animations.len(),
                gltf.named_animations.len()
            ),
        );
        return;
    };
    let fixed_sample_time = if config.evidence.is_some() || config.sample_midpoint {
        let Some(clip_asset) = clips.get(&clip) else {
            return;
        };
        let duration = clip_asset.duration();
        if !duration.is_finite() || duration < 0.0 {
            fail_runtime(
                &mut state,
                &shared,
                &mut app_exit,
                format!("exact animation {selection:?} has an invalid duration"),
            );
            return;
        }
        Some(duration * 0.5)
    } else {
        None
    };
    let (graph, node) = AnimationGraph::from_clip(clip);
    state.prepared_animation = Some(PreparedAnimation {
        graph: graphs.add(graph),
        node,
        fixed_sample_time,
    });
    state.gltf_inspected = true;
    state.animation_resolved = true;
    shared.update(|report| report.selected_animation = Some(label));
}

pub(super) fn start_selected_animation(
    mut commands: Commands,
    mut state: ResMut<RuntimeState>,
    mut players: Query<(Entity, &mut AnimationPlayer)>,
) {
    if state.terminal || state.animation_started {
        return;
    }
    let Some(prepared) = state.prepared_animation.as_ref() else {
        return;
    };
    let graph = prepared.graph.clone();
    let node = prepared.node;
    let fixed_sample_time = prepared.fixed_sample_time;
    let mut found = false;
    let mut sampled_players = 0_u64;
    for (entity, mut player) in &mut players {
        let active = player.play(node).repeat();
        if let Some(sample_time) = fixed_sample_time {
            active.set_seek_time(sample_time).pause();
            sampled_players = sampled_players.saturating_add(1);
        }
        commands
            .entity(entity)
            .insert(AnimationGraphHandle(graph.clone()));
        found = true;
    }
    if found {
        state.animation_started = true;
        state.animation_started_frame = Some(state.frames);
        state.sampled_players = sampled_players;
    }
}
