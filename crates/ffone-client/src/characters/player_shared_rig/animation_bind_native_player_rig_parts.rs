use super::*;

pub(super) fn resolve_native_player_rig_bones(
    mut commands: Commands,
    mut rigs: Query<
        (Entity, &NativePlayerRigRuntime, &mut NativePlayerRigStatus),
        Without<NativePlayerRigBones>,
    >,
    ready_scenes: Query<(), With<NativePlayerRigSceneReady>>,
    children: Query<&Children>,
    names: Query<&Name>,
) {
    for (rig_entity, runtime, mut status) in &mut rigs {
        if status.is_blocked() || ready_scenes.get(runtime.skeleton_scene).is_err() {
            continue;
        }
        match map_native_player_rig_bones(
            runtime.skeleton_scene,
            &runtime.gender,
            &children,
            &names,
        ) {
            Ok(bones) => {
                commands.entity(rig_entity).insert(bones);
                *status = NativePlayerRigStatus::Loading(NativePlayerRigLoadingStage::ModularParts);
            }
            Err(error) => *status = NativePlayerRigStatus::Blocked(error),
        }
    }
}

pub(super) fn map_native_player_rig_bones(
    scene: Entity,
    gender: &PlayerGenderRigContract,
    children: &Query<&Children>,
    names: &Query<&Name>,
) -> Result<NativePlayerRigBones, String> {
    let root_name = &gender.nodes[0].true_name;
    let mut stack = children
        .get(scene)
        .map(|children| children.iter().collect::<Vec<_>>())
        .unwrap_or_default();
    let mut root_matches = Vec::new();
    while let Some(entity) = stack.pop() {
        if names
            .get(entity)
            .is_ok_and(|name| name.as_str() == root_name)
        {
            root_matches.push(entity);
            continue;
        }
        if let Ok(descendants) = children.get(entity) {
            stack.extend(descendants.iter());
        }
    }
    let [root] = root_matches.as_slice() else {
        return Err(format!(
            "{:?} shared skeleton root {root_name:?} resolved {} times",
            gender.gender,
            root_matches.len()
        ));
    };

    let mut entities = Vec::with_capacity(gender.nodes.len());
    let mut entries = Vec::with_capacity(gender.nodes.len());
    for (index, contract_node) in gender.nodes.iter().enumerate() {
        let entity = if index == 0 {
            *root
        } else {
            let parent_index = contract_node.parent_actor_bone_index.ok_or_else(|| {
                format!(
                    "{:?} non-root bone {:?} has no parent",
                    gender.gender, contract_node.full_path
                )
            })? as usize;
            let parent = entities[parent_index];
            let matches = children
                .get(parent)
                .map(|children| {
                    children
                        .iter()
                        .filter(|child| {
                            names
                                .get(*child)
                                .is_ok_and(|name| name.as_str() == contract_node.true_name)
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let [resolved] = matches.as_slice() else {
                return Err(format!(
                    "{:?} shared skeleton bone {:?} resolved {} times below its exact parent",
                    gender.gender,
                    contract_node.full_path,
                    matches.len()
                ));
            };
            *resolved
        };
        entities.push(entity);
        entries.push(NativePlayerRigBoneEntity {
            actor_bone_index: contract_node.actor_bone_index,
            true_name: contract_node.true_name.clone(),
            full_path: contract_node.full_path.clone(),
            entity,
        });
    }
    Ok(NativePlayerRigBones::new(entries))
}

pub(super) fn prepare_native_player_rig_stand1(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut asset_cache: ResMut<NativePlayerRigAssetCache>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut rigs: Query<
        (Entity, &NativePlayerRigRuntime, &mut NativePlayerRigStatus),
        (
            With<NativePlayerRigBones>,
            Without<NativePlayerRigStand1Graph>,
        ),
    >,
) {
    for (rig_entity, runtime, mut status) in &mut rigs {
        if status.is_blocked() {
            continue;
        }
        if let Some(error) = asset_load_failure(&asset_server, &runtime.skeleton_gltf) {
            *status = NativePlayerRigStatus::Blocked(format!("shared skeleton: {error}"));
            continue;
        }
        let Some(gltf) = gltfs.get(&runtime.skeleton_gltf) else {
            continue;
        };
        let Some(clip) = gltf
            .named_animations
            .get(runtime.animation_name.as_str())
            .cloned()
        else {
            *status = NativePlayerRigStatus::Blocked(format!(
                "{:?} shared skeleton GLB has no named {:?} ({} animations)",
                runtime.gender.gender,
                runtime.animation_name,
                gltf.animations.len()
            ));
            continue;
        };
        let cache_key = (runtime.skeleton_gltf.id(), runtime.animation_name.clone());
        let (graph, node) =
            if let Some((graph, node)) = asset_cache.animation_graphs.get(&cache_key) {
                (graph.clone(), *node)
            } else {
                let (graph, node) = AnimationGraph::from_clip(clip);
                let graph = graphs.add(graph);
                asset_cache
                    .animation_graphs
                    .insert(cache_key, (graph.clone(), node));
                (graph, node)
            };
        commands
            .entity(rig_entity)
            .insert(NativePlayerRigStand1Graph { graph, node });
        *status = NativePlayerRigStatus::Loading(NativePlayerRigLoadingStage::Stand1);
    }
}

pub(super) fn bind_native_player_rig_parts(
    mut commands: Commands,
    mut rigs: Query<
        (
            Entity,
            &NativePlayerRigInstance,
            &NativePlayerRigRuntime,
            &NativePlayerRigBones,
            &mut NativePlayerRigStatus,
        ),
        Without<NativePlayerRigPartsBound>,
    >,
    ready_scenes: Query<(), With<NativePlayerRigSceneReady>>,
    instantiated_scenes: Query<(), With<Children>>,
    parents: Query<&ChildOf>,
    names: Query<&Name>,
    mut skinned_meshes: Query<
        (Entity, &mut SkinnedMesh, Option<&NativePlayerRigSkinBound>),
        Without<LegacyMaterialPassCompanion>,
    >,
) {
    'rigs: for (rig_entity, instance, runtime, bones, mut status) in &mut rigs {
        if status.is_blocked() {
            continue;
        }
        // WorldInstanceSpawner writes the complete dynamic scene before attaching its
        // direct instance child. An entity-scoped WorldInstanceReady observer
        // can lag behind that hierarchy when many HNPC parts resolve together,
        // so the child itself is equivalent readiness evidence here.
        if runtime.parts.iter().any(|part| {
            ready_scenes.get(part.root).is_err() && instantiated_scenes.get(part.root).is_err()
        }) {
            continue;
        }

        let mut total_palettes = 0_usize;
        let mut total_surfaces = 0_usize;
        let mut renderer_base = 0_u16;
        let mut ordered_parts = runtime.parts.iter().collect::<Vec<_>>();
        ordered_parts.sort_by_key(|part| part.contract.actor_skin_combiner_clothes_index);
        for part in ordered_parts {
            let expected = part
                .contract
                .skins
                .iter()
                .map(|skin| skin.renderer_true_name.as_str())
                .collect::<BTreeSet<_>>();
            let mut seen = BTreeSet::new();
            for (surface, mut skinned, bound) in &mut skinned_meshes {
                if !is_descendant_of(surface, part.root, &parents) {
                    continue;
                }
                let remap = match exact_renderer_remap(
                    surface,
                    part.root,
                    &part.contract.skins,
                    &parents,
                    &names,
                ) {
                    Ok(remap) => remap,
                    Err(error) => {
                        *status = NativePlayerRigStatus::Blocked(format!(
                            "part {:?}: {error}",
                            part.contract.exact_route
                        ));
                        continue 'rigs;
                    }
                };
                let local_renderer_index = part
                    .contract
                    .skins
                    .iter()
                    .position(|candidate| std::ptr::eq(candidate, remap))
                    .expect("exact_renderer_remap returns an entry from this slice");
                let Some(renderer_index) =
                    flattened_renderer_index(renderer_base, local_renderer_index)
                else {
                    *status = NativePlayerRigStatus::Blocked(format!(
                        "part {:?} flattened renderer order exceeds u16",
                        part.contract.exact_route
                    ));
                    continue 'rigs;
                };
                // Character-selection preview teardown and ordinary-world
                // spawn can cross this deferred bind in the same Update. The
                // query proves that the surface exists now, but an earlier
                // queued despawn of its rig root may run before this command
                // is applied. A retired preview surface is no longer an
                // acceptance target, so silently discard only that stale
                // entity command instead of panicking the live world entry.
                commands
                    .entity(surface)
                    .queue_silenced(move |mut entity: EntityWorldMut| {
                        entity.insert(LegacyMaterialRendererOrder { renderer_index });
                    });
                if skinned.joints.len() != remap.actor_bone_paths.len() {
                    *status = NativePlayerRigStatus::Blocked(format!(
                        "part {:?} renderer {:?} has {} Bevy joints, contract has {}",
                        part.contract.exact_route,
                        remap.renderer_true_name,
                        skinned.joints.len(),
                        remap.actor_bone_paths.len()
                    ));
                    continue 'rigs;
                }
                let joints = match remap
                    .actor_bone_paths
                    .iter()
                    .map(|path| {
                        bones.by_full_path(path).ok_or_else(|| {
                            format!(
                                "part {:?} renderer {:?} refers to missing shared bone {path:?}",
                                part.contract.exact_route, remap.renderer_true_name
                            )
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
                {
                    Ok(joints) => joints,
                    Err(error) => {
                        *status = NativePlayerRigStatus::Blocked(error);
                        continue 'rigs;
                    }
                };
                if bound.is_none_or(|bound| {
                    bound.rig_root != rig_entity || bound.generation != instance.generation
                }) {
                    skinned.joints = joints;
                    let binding = NativePlayerRigSkinBound {
                        rig_root: rig_entity,
                        generation: instance.generation,
                    };
                    commands
                        .entity(surface)
                        .queue_silenced(move |mut entity: EntityWorldMut| {
                            entity.insert(binding);
                        });
                }
                seen.insert(remap.renderer_true_name.as_str());
                total_surfaces += 1;
            }
            if seen != expected {
                let missing = expected.difference(&seen).copied().collect::<Vec<_>>();
                *status = NativePlayerRigStatus::Blocked(format!(
                    "part {:?} did not instantiate exact renderer palettes {:?}",
                    part.contract.exact_route, missing
                ));
                continue 'rigs;
            }
            total_palettes += expected.len();
            let Ok(renderer_count) = u16::try_from(part.contract.skins.len()) else {
                *status = NativePlayerRigStatus::Blocked(format!(
                    "part {:?} renderer count exceeds u16",
                    part.contract.exact_route
                ));
                continue 'rigs;
            };
            let Some(next_renderer_base) = renderer_base.checked_add(renderer_count) else {
                *status = NativePlayerRigStatus::Blocked(
                    "flattened native player renderer order exceeds u16".to_owned(),
                );
                continue 'rigs;
            };
            renderer_base = next_renderer_base;
        }
        let parts_bound = NativePlayerRigPartsBound {
            parts: runtime.parts.len(),
            skin_palettes: total_palettes,
            skinned_surfaces: total_surfaces,
        };
        commands
            .entity(rig_entity)
            .queue_silenced(move |mut entity: EntityWorldMut| {
                entity.insert(parts_bound);
            });
    }
}

pub(super) fn play_native_player_rig_stand1(
    mut commands: Commands,
    mut rigs: Query<
        (
            Entity,
            &NativePlayerRigRuntime,
            &NativePlayerRigStand1Graph,
            &mut NativePlayerRigStatus,
        ),
        (
            With<NativePlayerRigBones>,
            With<NativePlayerRigPartsBound>,
            Without<NativePlayerRigStand1Playback>,
        ),
    >,
    parents: Query<&ChildOf>,
    scene_roots: Query<(), With<WorldAssetRoot>>,
    mut players: Query<(Entity, &mut AnimationPlayer)>,
) {
    for (rig_entity, runtime, stand1, mut status) in &mut rigs {
        if status.is_blocked() {
            continue;
        }
        let matches = players
            .iter_mut()
            .filter_map(|(entity, _)| {
                (owning_scene_root(entity, &parents, &scene_roots) == Some(runtime.skeleton_scene))
                    .then_some(entity)
            })
            .collect::<Vec<_>>();
        let [player_entity] = matches.as_slice() else {
            *status = NativePlayerRigStatus::Blocked(format!(
                "{:?} shared skeleton resolved {} AnimationPlayers",
                runtime.gender.gender,
                matches.len()
            ));
            continue;
        };
        let Ok((_, mut player)) = players.get_mut(*player_entity) else {
            *status = NativePlayerRigStatus::Blocked(
                "shared skeleton AnimationPlayer disappeared".to_owned(),
            );
            continue;
        };
        player.play(stand1.node).repeat();
        commands
            .entity(*player_entity)
            .insert(AnimationGraphHandle(stand1.graph.clone()));
        commands
            .entity(rig_entity)
            .insert(NativePlayerRigStand1Playback {
                animation_player: *player_entity,
                animation_graph: stand1.graph.clone(),
                animation_node: stand1.node,
            });
    }
}

pub(super) fn apply_native_player_rig_animation_requests(
    mut commands: Commands,
    mut asset_cache: ResMut<NativePlayerRigAssetCache>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    rigs: Query<(
        Entity,
        &NativePlayerRigRuntime,
        &NativePlayerRigStand1Playback,
        &NativePlayerRigAnimationRequest,
        Option<&NativePlayerRigAnimationApplied>,
        Option<&NativePlayerRigAnimationBlend>,
    )>,
    mut players: Query<(&mut AnimationPlayer, Option<&mut AnimationTransitions>)>,
) {
    for (rig_entity, runtime, playback, request, applied, blend) in &rigs {
        if applied.is_some_and(|applied| {
            applied.clip == request.clip
                && applied.revision == request.revision
                && applied.repeat == request.repeat
        }) {
            continue;
        }
        if request.clip.trim().is_empty() {
            commands
                .entity(rig_entity)
                .insert(NativePlayerRigAnimationIssue(
                    "shared-rig animation request has an empty clip".to_owned(),
                ));
            continue;
        }
        let Some(gltf) = gltfs.get(&runtime.skeleton_gltf) else {
            continue;
        };
        let Some(clip) = gltf.named_animations.get(request.clip.as_str()).cloned() else {
            commands
                .entity(rig_entity)
                .insert(NativePlayerRigAnimationIssue(format!(
                    "{:?} shared skeleton has no requested clip {:?}",
                    runtime.gender.gender, request.clip
                )));
            continue;
        };
        if let Some(blend) = blend {
            let (graph, nodes) = asset_cache
                .transition_graphs
                .entry(runtime.skeleton_gltf.id())
                .or_insert_with(|| {
                    let mut entries = gltf.named_animations.iter().collect::<Vec<_>>();
                    entries.sort_by(|a, b| a.0.cmp(b.0));
                    let (graph, nodes) =
                        AnimationGraph::from_clips(entries.iter().map(|(_, clip)| (*clip).clone()));
                    (
                        graphs.add(graph),
                        entries
                            .into_iter()
                            .zip(nodes)
                            .map(|((name, _), node)| (name.to_string(), node))
                            .collect(),
                    )
                });
            let node = nodes[&request.clip];
            let Ok((mut player, transitions)) = players.get_mut(playback.animation_player) else {
                continue;
            };
            if playback.animation_graph != *graph {
                player.stop_all();
            }
            let mut new_transitions = AnimationTransitions::new();
            let has_transitions = transitions.is_some();
            let transitions = match transitions {
                Some(transitions) => transitions.into_inner(),
                None => &mut new_transitions,
            };
            let active = transitions.play(&mut player, node, blend.0);
            active.replay();
            active.set_repeat(if request.repeat {
                bevy::animation::RepeatAnimation::Forever
            } else {
                bevy::animation::RepeatAnimation::Never
            });
            if !has_transitions {
                commands
                    .entity(playback.animation_player)
                    .insert(new_transitions);
            }
            commands
                .entity(playback.animation_player)
                .insert(AnimationGraphHandle(graph.clone()));
            commands
                .entity(rig_entity)
                .insert((
                    NativePlayerRigStand1Playback {
                        animation_player: playback.animation_player,
                        animation_graph: graph.clone(),
                        animation_node: node,
                    },
                    NativePlayerRigAnimationApplied {
                        clip: request.clip.clone(),
                        revision: request.revision,
                        repeat: request.repeat,
                    },
                ))
                .remove::<NativePlayerRigAnimationIssue>();
            continue;
        }
        let cache_key = (runtime.skeleton_gltf.id(), request.clip.clone());
        let (graph, node) =
            if let Some((graph, node)) = asset_cache.animation_graphs.get(&cache_key) {
                (graph.clone(), *node)
            } else {
                let (graph, node) = AnimationGraph::from_clip(clip);
                let graph = graphs.add(graph);
                asset_cache
                    .animation_graphs
                    .insert(cache_key, (graph.clone(), node));
                (graph, node)
            };
        let Ok((mut player, _)) = players.get_mut(playback.animation_player) else {
            commands
                .entity(rig_entity)
                .insert(NativePlayerRigAnimationIssue(
                    "shared-rig AnimationPlayer disappeared during clip switch".to_owned(),
                ));
            continue;
        };
        player.stop_all();
        let active = player.start(node);
        if request.repeat {
            active.repeat();
        }
        commands
            .entity(playback.animation_player)
            .insert(AnimationGraphHandle(graph));
        commands
            .entity(rig_entity)
            .insert(NativePlayerRigAnimationApplied {
                clip: request.clip.clone(),
                revision: request.revision,
                repeat: request.repeat,
            });
        commands
            .entity(rig_entity)
            .remove::<NativePlayerRigAnimationIssue>();
    }
}

pub(super) fn finalize_native_player_rig_status(
    mut rigs: Query<(
        Entity,
        &NativePlayerRigInstance,
        &NativePlayerRigRuntime,
        &mut NativePlayerRigStatus,
        Option<&NativePlayerRigBones>,
        Option<&NativePlayerRigPartsBound>,
        Option<&NativePlayerRigStand1Graph>,
        Option<&NativePlayerRigStand1Playback>,
        Option<&NativePlayerBodyShape>,
        Option<&NativePlayerBodyShapePlayback>,
    )>,
    children: Query<&Children>,
    mut descendants: Local<RigDescendantScratch>,
    mut source_index: Local<RigQueryIndex>,
    mut companion_index: Local<RigQueryIndex>,
    pass_companions: Query<
        (
            Entity,
            &LegacyMaterialPassCompanion,
            Option<&NativePlayerRigSkinBound>,
        ),
        (With<LegacyMaterialPassCompanion>, With<SkinnedMesh>),
    >,
    source_bindings: Query<&NativePlayerRigSkinBound>,
    ordered_material_sources: Query<
        (
            Entity,
            &LegacyMaterialRendererOrder,
            Option<&LegacyMaterialSortOrderApplied>,
            Option<&crate::legacy_model_material::PendingLegacyModelMaterial>,
        ),
        (
            With<NativePlayerRigSkinBound>,
            Without<LegacyMaterialPassCompanion>,
        ),
    >,
) {
    if rigs.is_empty() {
        return;
    }
    let source_rows = if source_index.enabled {
        ordered_material_sources.iter().collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    source_index.rebuild(source_rows.iter().map(|row| row.0));
    let companion_rows = if companion_index.enabled {
        pass_companions.iter().collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    companion_index.rebuild(companion_rows.iter().map(|row| row.0));
    for (
        rig_entity,
        instance,
        runtime,
        mut status,
        bones,
        parts,
        graph,
        playback,
        body_shape,
        body_playback,
    ) in &mut rigs
    {
        if status.is_blocked() {
            continue;
        }
        if body_shape.is_some() && body_playback.is_none() && playback.is_some() {
            status.set_if_neq(NativePlayerRigStatus::Loading(
                NativePlayerRigLoadingStage::Stand1,
            ));
            continue;
        }
        descendants.collect_for_query(rig_entity, &children, source_index.enabled);
        let unbound = |(_, companion, binding): (
            Entity,
            &LegacyMaterialPassCompanion,
            Option<&NativePlayerRigSkinBound>,
        )| {
            source_bindings
                .get(companion.source_mesh_entity)
                .is_ok_and(|source| {
                    source.rig_root == rig_entity && source.generation == instance.generation
                })
                && binding.is_none_or(|binding| {
                    binding.rig_root != rig_entity || binding.generation != instance.generation
                })
        };
        let has_unbound_pass_companion = if companion_index.enabled {
            descendants
                .ordered_matches(&companion_index)
                .iter()
                .any(|index| unbound(companion_rows[*index]))
        } else {
            pass_companions
                .iter()
                .filter(|(entity, _, _)| descendants.contains(*entity))
                .any(unbound)
        };
        let ordered_sources = if source_index.enabled {
            descendants
                .ordered_matches(&source_index)
                .iter()
                .map(|index| source_rows[*index])
                .collect::<Vec<_>>()
        } else {
            ordered_material_sources
                .iter()
                .filter(|(entity, _, _, _)| descendants.contains(*entity))
                .collect::<Vec<_>>()
        };
        let has_unordered_material_source = parts.is_some_and(|parts| {
            ordered_sources.len() != parts.skinned_surfaces
                || ordered_sources
                    .iter()
                    .any(|(_, renderer_order, applied, pending)| {
                        let expected_pass_count = pending
                            .map(|pending| pending.params.render_plan().passes.len())
                            .and_then(|count| u8::try_from(count).ok());
                        applied.is_none_or(|applied| {
                            applied.renderer_index != renderer_order.renderer_index
                                || Some(applied.pass_count) != expected_pass_count
                        })
                    })
        });
        let next_status = match (bones, parts, graph, playback) {
            (Some(_), Some(_), _, _)
                if has_unbound_pass_companion || has_unordered_material_source =>
            {
                NativePlayerRigStatus::Loading(NativePlayerRigLoadingStage::ModularParts)
            }
            (Some(bones), Some(parts), Some(_), Some(_)) => NativePlayerRigStatus::ReadyAnimated {
                actor_bones: bones.entries.len(),
                parts: parts.parts,
                skin_palettes: parts.skin_palettes,
                skinned_surfaces: parts.skinned_surfaces,
            },
            (Some(_), Some(_), Some(_), None) => {
                NativePlayerRigStatus::Loading(NativePlayerRigLoadingStage::Stand1)
            }
            (Some(_), None, _, _) => {
                NativePlayerRigStatus::Loading(NativePlayerRigLoadingStage::ModularParts)
            }
            (None, _, _, _) if runtime.skeleton_scene != Entity::PLACEHOLDER => {
                NativePlayerRigStatus::Loading(NativePlayerRigLoadingStage::BoneMap)
            }
            _ => NativePlayerRigStatus::Loading(NativePlayerRigLoadingStage::SkeletonScene),
        };
        status.set_if_neq(next_status);
    }
}

// Rebuild the actual rig subtree each update so reparenting is observed now.
// Query indices map those entities back to the original source/pass/error order;
// the bitset also preserves the original scan path in the explicit A/B fixture.
#[derive(Default)]
pub(super) struct RigDescendantScratch {
    pub(super) membership: Vec<u64>,
    pub(super) pending: Vec<Entity>,
    pub(super) entities: Vec<Entity>,
    pub(super) matches: Vec<usize>,
}

impl RigDescendantScratch {
    #[cfg(test)]
    pub(super) fn collect(&mut self, root: Entity, children: &Query<&Children>) {
        self.collect_for_query(root, children, true);
    }

    pub(super) fn collect_for_query(&mut self, root: Entity, children: &Query<&Children>, indexed: bool) {
        self.membership.fill(0);
        self.pending.clear();
        self.entities.clear();
        if let Ok(owned) = children.get(root) {
            self.pending.extend(owned.iter());
        }
        while let Some(entity) = self.pending.pop() {
            if entity == root {
                continue;
            }
            let index = entity.index().index() as usize;
            let word = index / 64;
            if self.membership.len() <= word {
                self.membership.resize(word + 1, 0);
            }
            let mask = 1_u64 << (index % 64);
            if self.membership[word] & mask != 0 {
                continue;
            }
            self.membership[word] |= mask;
            if indexed {
                self.entities.push(entity);
            }
            if let Ok(owned) = children.get(entity) {
                self.pending.extend(owned.iter());
            }
        }
    }

    pub(super) fn contains(&self, entity: Entity) -> bool {
        let index = entity.index().index() as usize;
        self.membership
            .get(index / 64)
            .is_some_and(|word| word & (1_u64 << (index % 64)) != 0)
    }

    pub(super) fn ordered_matches(&mut self, index: &RigQueryIndex) -> &[usize] {
        self.matches.clear();
        self.matches.extend(
            self.entities
                .iter()
                .filter_map(|entity| index.rank(*entity)),
        );
        // Preserve the original ECS query's source/pass/error order, not DFS order.
        self.matches.sort_unstable();
        &self.matches
    }
}

pub(super) struct RigQueryIndex {
    // Epochs invalidate removed sources without clearing/scanning the high-water
    // allocation each frame. The full Entity also guards generation reuse.
    pub(super) ranks: Vec<(Entity, usize, u64)>,
    pub(super) epoch: u64,
    pub(super) enabled: bool,
}

impl Default for RigQueryIndex {
    fn default() -> Self {
        Self {
            ranks: default(),
            epoch: 0,
            enabled: !(std::env::var_os("FFONE_PERF_OUTPUT").is_some()
                && std::env::var_os("FFONE_PERF_CPU_QUERY_BASELINE").is_some()),
        }
    }
}
