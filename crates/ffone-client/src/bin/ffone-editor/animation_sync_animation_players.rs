use super::*;

pub(super) const ANIMATION_SLOTS: usize = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EditorPoseMode {
    /// First frame of the model's authored default clip, frozen for inspection.
    Default,
    /// Unanimated GLB rest/bind transforms. For character rigs this is the T-pose view.
    TPose,
    /// Normal playback of the selected named clip.
    Clip,
}

#[derive(Debug)]
pub(super) struct PreparedAnimationSet {
    pub(super) generation: u64,
    pub(super) graph: Handle<AnimationGraph>,
    pub(super) nodes: BTreeMap<String, AnimationNodeIndex>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct EditorAnimationApplied {
    pub(super) generation: u64,
    pub(super) revision: u64,
    pub(super) node: AnimationNodeIndex,
}

pub(super) fn prepare_animation_graph(
    preview: Res<ModelPreview>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut prepared: ResMut<PreparedAnimations>,
) {
    if prepared
        .0
        .as_ref()
        .is_some_and(|value| value.generation == preview.generation)
    {
        return;
    }
    let Some(gltf_handle) = preview.gltf.as_ref() else {
        return;
    };
    let Some(gltf) = gltfs.get(gltf_handle) else {
        return;
    };
    let mut graph = AnimationGraph::new();
    let mut named = gltf.named_animations.iter().collect::<Vec<_>>();
    named.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
    let nodes = named
        .into_iter()
        .map(|(name, clip)| {
            (
                name.to_string(),
                graph.add_clip(clip.clone(), 1.0, graph.root),
            )
        })
        .collect();
    prepared.0 = Some(PreparedAnimationSet {
        generation: preview.generation,
        graph: graphs.add(graph),
        nodes,
    });
}

pub(super) fn sync_animation_players(
    mut commands: Commands,
    catalog: Res<EditorCatalog>,
    state: Res<EditorState>,
    preview: Res<ModelPreview>,
    prepared: Res<PreparedAnimations>,
    parents: Query<&ChildOf>,
    mut players: Query<(
        Entity,
        &mut AnimationPlayer,
        Option<&EditorAnimationApplied>,
    )>,
) {
    if state.kind == CatalogKind::Equipment {
        return;
    }
    if state.pose_mode == EditorPoseMode::TPose {
        for (entity, mut player, applied) in &mut players {
            if !belongs_to_root(entity, preview.root, &parents) {
                continue;
            }
            player.stop_all();
            if applied.is_some() {
                commands.entity(entity).remove::<EditorAnimationApplied>();
            }
        }
        return;
    }
    let Some(prepared) = prepared.0.as_ref() else {
        return;
    };
    let Some(clip) = catalog.entries[state.selected]
        .animations
        .get(state.clip_index)
    else {
        return;
    };
    let Some(&node) = prepared.nodes.get(clip) else {
        return;
    };
    for (entity, mut player, applied) in &mut players {
        if !belongs_to_root(entity, preview.root, &parents) {
            continue;
        }
        let needs_restart = applied.is_none_or(|applied| {
            applied.generation != preview.generation
                || applied.revision != state.playback_revision
                || applied.node != node
        });
        if needs_restart {
            let active = player
                .play(node)
                .seek_to(0.0)
                .set_speed(state.speed)
                .set_repeat(if state.looping {
                    RepeatAnimation::Forever
                } else {
                    RepeatAnimation::Never
                });
            if state.paused || state.pose_mode == EditorPoseMode::Default {
                active.pause();
            }
            commands.entity(entity).insert((
                AnimationGraphHandle(prepared.graph.clone()),
                EditorAnimationApplied {
                    generation: preview.generation,
                    revision: state.playback_revision,
                    node,
                },
            ));
            continue;
        }
        if let Some(active) = player.animation_mut(node) {
            active.set_speed(state.speed).set_repeat(if state.looping {
                RepeatAnimation::Forever
            } else {
                RepeatAnimation::Never
            });
            if state.pose_mode == EditorPoseMode::Default {
                active.seek_to(0.0).pause();
            } else if state.paused {
                active.pause();
            } else {
                active.resume();
            }
        }
    }
}

/// Converts ordinary biped bind/A poses into a geometric T-pose without
/// replacing the native rig. Models with differently named arm chains remain
/// in their exact published bind pose instead of receiving a guessed edit.
pub(super) fn apply_editor_t_pose(
    state: Res<EditorState>,
    icons: Res<icon_generator::IconGenerator>,
    segments: Query<(Entity, &Name, &ChildOf, &Children, &GlobalTransform)>,
    globals: Query<&GlobalTransform>,
    mut transforms: ParamSet<(Query<(&Name, &Transform)>, Query<&mut Transform>)>,
) {
    if state.pose_mode != EditorPoseMode::TPose || (state.kind == CatalogKind::Equipment && !icons.active) {
        return;
    }

    let mut corrected_rotations = Vec::new();
    {
        let nodes = transforms.p0();
        for (entity, name, parent, children, global) in &segments {
            let Some(child_segment) = t_pose_child_segment(name.as_str()) else {
                continue;
            };
            let Some(child_translation) = children.iter().find_map(|child| {
                let (child_name, child_transform) = nodes.get(child).ok()?;
                compact_bone_name(child_name.as_str())
                    .contains(child_segment)
                    .then_some(child_transform.translation)
            }) else {
                continue;
            };
            let Ok(parent_global) = globals.get(parent.parent()) else {
                continue;
            };
            let direction = global
                .affine()
                .transform_vector3(child_translation)
                .normalize_or_zero();
            let parent_rotation = parent_global.to_scale_rotation_translation().1;
            let segment_rotation = global.to_scale_rotation_translation().1;
            if let Some(rotation) =
                horizontal_t_pose_rotation(parent_rotation, segment_rotation, direction)
            {
                corrected_rotations.push((entity, rotation));
            }
        }
    }

    let mut nodes = transforms.p1();
    for (entity, rotation) in corrected_rotations {
        if let Ok(mut transform) = nodes.get_mut(entity) {
            transform.rotation = rotation;
        }
    }
}

pub(super) fn compact_bone_name(name: &str) -> String {
    name.chars()
        .filter(|character| !character.is_ascii_whitespace() && *character != '_')
        .flat_map(char::to_lowercase)
        .collect()
}

pub(super) fn t_pose_child_segment(name: &str) -> Option<&'static str> {
    let name = compact_bone_name(name);
    if name.contains("upperarm") {
        Some("forearm")
    } else if name.contains("forearm") {
        Some("hand")
    } else {
        None
    }
}

pub(super) fn horizontal_t_pose_rotation(
    parent_rotation: Quat,
    segment_rotation: Quat,
    direction: Vec3,
) -> Option<Quat> {
    (direction != Vec3::ZERO).then(|| {
        let target = if direction.x.is_sign_negative() {
            Vec3::NEG_X
        } else {
            Vec3::X
        };
        let correction = Quat::from_rotation_arc(direction, target);
        (parent_rotation.inverse() * correction * segment_rotation).normalize()
    })
}

pub(super) fn select_relative_clip(catalog: &EditorCatalog, state: &mut EditorState, direction: isize) {
    let count = catalog.entries[state.selected].animations.len();
    if count == 0 {
        return;
    }
    let next = (state.clip_index as isize + direction).rem_euclid(count as isize) as usize;
    state.select_clip(catalog, next);
    state.animation_page = next / ANIMATION_SLOTS;
}

#[derive(Component)]
pub(super) struct AnimationSlot(pub(super) usize);
