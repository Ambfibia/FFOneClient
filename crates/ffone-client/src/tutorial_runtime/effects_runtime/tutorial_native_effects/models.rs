use super::*;

pub(super) fn exact_effect_mesh_scene(effect_id: i32) -> Option<&'static str> {
    match effect_id {
        // ES425 and ES740 point at the same exact `m_inven` subtree: 201 of
        // their 204 closure objects are byte-identical, with only the ES
        // wrapper/root/controller differing. Keep one native publication.
        425 => Some("map/shared/effects/models/es740/m_inven.glb"),
        461 => Some("map/shared/effects/models/es461/launcher_symbol.glb"),
        464 => Some("map/shared/effects/models/es464/jumppad_S_power.glb"),
        465 => Some("map/shared/effects/models/es465/jumppad_M_power.glb"),
        466 => Some("map/shared/effects/models/es466/jumppad_B_power.glb"),
        533 => Some("map/shared/effects/models/es533/run_symbol.glb"),
        534 => Some("map/shared/effects/models/es534/roket_symbol.glb"),
        547 => Some("map/shared/effects/models/es547/f_buttercup_corruptak.glb"),
        548 => Some("map/shared/effects/models/es548/f_buttercup_megatak.glb"),
        601 => Some("map/shared/effects/models/es601/f_bloo_corruptak.glb"),
        602 => Some("map/shared/effects/models/es602/f_bloo_megatak.glb"),
        626 => Some("map/shared/effects/models/es626/f_mac_corruptak.glb"),
        627 => Some("map/shared/effects/models/es627/f_mac_megatak.glb"),
        668 => Some("map/shared/effects/models/es668/patrol_elena7.glb"),
        705 => Some("map/shared/effects/models/es705/nano_get.glb"),
        734 => Some("map/shared/effects/models/es734/T_pistol.glb"),
        736 => Some("map/shared/effects/models/es736/t_bubbles.glb"),
        739 => Some("map/shared/effects/models/es739/Tutorial_dexterhologram.glb"),
        740 => Some("map/shared/effects/models/es740/m_inven.glb"),
        741 => Some("map/shared/effects/models/es741/Tutorial_dexterhologram_1.glb"),
        742 => Some("map/shared/effects/models/es742/T_electric gun.glb"),
        751 => Some("map/shared/effects/models/es751/npc_samuraijack_event_eff.glb"),
        771 => Some("map/shared/effects/models/es771/tutorialstone.glb"),
        804 => Some("map/shared/effects/models/es804/models/effect/protection_target_e02.glb"),
        833 => Some("map/shared/effects/models/es833/vehicle_board_inven0.glb"),
        834 => Some("map/shared/effects/models/es834/m_vehicle_scooter_inven.glb"),
        865 => Some("map/shared/effects/models/es865/marker_progress.glb"),
        866 => Some("map/shared/effects/models/es866/marker_acquire.glb"),
        _ => None,
    }
}

pub(super) fn exact_projectile_mesh_scene(effect_id: i32) -> Option<&'static str> {
    match effect_id {
        15 => Some("map/shared/projectiles/effects/models/es15/tail1.glb"),
        31 => Some(
            "map/shared/projectiles/effects/models/es31/models/projectile/EnergyGrenades_e01.glb",
        ),
        378 => Some(
            "map/shared/projectiles/effects/models/es378/models/projectile/cnanoProjectile_a3.glb",
        ),
        379 => Some("map/shared/projectiles/effects/models/es379/models/projectile/Rocket2.glb"),
        391 => Some("map/shared/projectiles/effects/models/es391/tail2.glb"),
        718 => Some("map/shared/projectiles/effects/models/es718/sonic_ware_ta_m01.glb"),
        729 => Some("map/shared/projectiles/effects/models/es729/models/projectile/Ice_L.glb"),
        754 => {
            Some("map/shared/projectiles/effects/models/es754/models/projectile/weapon_fluid_0.glb")
        }
        755 => Some("map/shared/projectiles/effects/models/es755/models/projectile/Ballisitic.glb"),
        787 => Some("map/shared/projectiles/effects/models/es787/models/projectile/Rocket2_up.glb"),
        790 => {
            Some("map/shared/projectiles/effects/models/es790/models/projectile/Ballisitic_up.glb")
        }
        791 => Some("map/shared/projectiles/effects/models/es791/models/projectile/Ice_L_up.glb"),
        792 => Some(
            "map/shared/projectiles/effects/models/es792/models/projectile/weapon_fluid_up.glb",
        ),
        793 => Some("map/shared/projectiles/effects/models/es793/models/projectile/needset_up.glb"),
        794 => Some(
            "map/shared/projectiles/effects/models/es794/models/projectile/cnanoProjectile_a3_up.glb",
        ),
        _ => None,
    }
}

#[derive(Component)]
pub(super) struct NativeMeshEffectPlayback {
    pub(super) gltf: Handle<Gltf>,
    pub(super) graph: Option<(Handle<AnimationGraph>, AnimationNodeIndex)>,
    pub(super) repeat: bool,
    pub(super) resolved: bool,
    pub(super) started: bool,
}

/// The mesh stops at the emission deadline; already-emitted particles have
/// their own remaining lifetime on the parent effect.
#[derive(Component)]
pub(super) struct NativeMeshEffectLifetime(pub(super) Option<f32>);

impl NativeMeshEffectLifetime {
    pub(super) fn from_plan(plan: &NativeEffectPlan) -> Self {
        Self((!plan.disable_update && plan.maximum_timer > 0.0).then_some(plan.maximum_timer))
    }
}

/// GLTF materials are shared by default. Every mesh effect needs an independent
/// runtime material both for Unity render-queue ordering and, when present,
/// renderer-instance material animation curves.
#[derive(Component)]
pub(super) struct NativePreparedMeshEffectSurface;

pub(super) fn prepare_native_mesh_effect_materials(
    mut commands: Commands,
    mut material_cache: Option<ResMut<crate::legacy_model_material::LegacyStaticMaterialCache>>,
    sharing: Option<Res<crate::legacy_model_material::StaticAssetSharing>>,
    model_sharing: Option<Res<crate::legacy_model_material::ModelMaterialSharing>>,
    mut roots: Query<(Entity, &mut NativeEffectRoot)>,
    mut projectiles: Query<(Entity, &mut NativeProjectile)>,
    parents: Query<&ChildOf>,
    mut surfaces: Query<(
        Entity,
        &Name,
        &mut MeshMaterial3d<LegacyModelMaterial>,
        Option<&NativePreparedMeshEffectSurface>,
    )>,
    mut standalone_animations: Query<(Entity, &mut TutorialEffectMaterialAnimation)>,
    mut materials: ResMut<Assets<LegacyModelMaterial>>,
) {
    let share = sharing.as_ref().is_none_or(|sharing| sharing.enabled)
        && model_sharing.as_ref().is_none_or(|sharing| sharing.0);
    for (root_entity, mut root) in &mut roots {
        // Particle-only effects start with this barrier open, while mesh
        // effects open it permanently after their scene surfaces are ready.
        // Scanning every LegacyModelMaterial surface for every already-ready
        // root made dense world tiles O(effect roots * world surfaces) on
        // every frame (hundreds by thousands in Firepits).
        if !root.requires_mesh_surface_preparation() {
            continue;
        }
        let mut has_surface = false;
        let target_nodes = root
            .material_animation
            .as_ref()
            .map(|animation| {
                animation
                    .curves
                    .iter()
                    .map(|curve| curve.node_name.as_str())
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        let mut ready_target_nodes = BTreeSet::new();
        for (surface_entity, name, mut handle, prepared) in &mut surfaces {
            if !is_descendant_of(surface_entity, root_entity, &parents) {
                continue;
            }
            has_surface = true;
            for target_node in &target_nodes {
                if material_surface_matches_node(name.as_str(), target_node) {
                    ready_target_nodes.insert((*target_node).to_owned());
                }
            }
            if prepared.is_some() {
                continue;
            }
            let Some(source) = materials.get(&handle.0) else {
                continue;
            };
            let may_share = share && !source.is_instance_private();
            let mut material = source.clone();
            material.sort_bias =
                mesh_effect_render_queue_sort_bias(material.render_mode.source_queue);
            handle.0 = if may_share && let Some(cache) = material_cache.as_deref_mut() {
                cache.admit_immutable(material, &mut materials)
            } else {
                materials.add(material)
            };
            commands
                .entity(surface_entity)
                .insert(NativePreparedMeshEffectSurface);
        }
        let animated_surface_ready = target_nodes.is_empty() || !ready_target_nodes.is_empty();
        if has_surface && animated_surface_ready {
            // Unity starts the EffectEmitterController lifetime after its NIF
            // object has been instantiated. Keep both the GLTF transform clip
            // and material curves behind the same barrier: ES751 otherwise
            // reaches its zero-scale keys while the exact PNG material is
            // still loading and appears invisible. A serialized material curve
            // may legally target a node with no Renderer (ES734 Cylinder04),
            // which is a Unity no-op and must not hold this barrier forever.
            root.waiting_for_mesh_surface = false;
        }
    }

    for (root_entity, mut projectile) in &mut projectiles {
        if !projectile.waiting_for_mesh_surface {
            continue;
        }
        let mut has_surface = false;
        for (surface_entity, _, mut handle, prepared) in &mut surfaces {
            if !is_descendant_of(surface_entity, root_entity, &parents) {
                continue;
            }
            has_surface = true;
            if prepared.is_some() {
                continue;
            }
            let Some(source) = materials.get(&handle.0) else {
                continue;
            };
            let may_share = share && !source.is_instance_private();
            let mut material = source.clone();
            material.sort_bias =
                mesh_effect_render_queue_sort_bias(material.render_mode.source_queue);
            handle.0 = if may_share && let Some(cache) = material_cache.as_deref_mut() {
                cache.admit_immutable(material, &mut materials)
            } else {
                materials.add(material)
            };
            commands
                .entity(surface_entity)
                .insert(NativePreparedMeshEffectSurface);
        }
        if !has_surface {
            continue;
        }
        projectile.waiting_for_mesh_surface = false;
        for (animation_entity, mut animation) in &mut standalone_animations {
            if is_descendant_of(animation_entity, root_entity, &parents) {
                animation.set_paused(false);
            }
        }
    }
}

pub(super) fn animate_native_mesh_effect_materials(
    time: Res<Time>,
    roots: Query<(Entity, &NativeEffectRoot)>,
    mut standalone: Query<(Entity, &mut TutorialEffectMaterialAnimation)>,
    children: Query<&Children>,
    mut surfaces: Query<(Entity, &Name, &mut MeshMaterial3d<LegacyModelMaterial>)>,
    mut materials: ResMut<Assets<LegacyModelMaterial>>,
    mut uniform_updates: Option<ResMut<crate::legacy_model_material::NativeMaterialUniformUpdates>>,
) {
    let mut changed = BTreeSet::new();
    for (root_entity, root) in &roots {
        let Some(animation) = root.material_animation.as_ref() else {
            continue;
        };
        let animation_time = root.age.rem_euclid(animation.duration);
        apply_material_animation(
            root_entity,
            animation,
            animation_time,
            &children,
            &mut surfaces,
            &mut materials,
            &mut changed,
        );
    }
    for (root_entity, mut playback) in &mut standalone {
        if !playback.paused {
            playback.age += time.delta_secs();
        }
        let animation_time = playback.age.rem_euclid(playback.plan.duration);
        apply_material_animation(
            root_entity,
            &playback.plan,
            animation_time,
            &children,
            &mut surfaces,
            &mut materials,
            &mut changed,
        );
    }
    for id in changed {
        if !uniform_updates
            .as_mut()
            .is_some_and(|updates| updates.enqueue(id))
        {
            // Bevy 0.19 tracks mutable dereferences, not get_mut calls.
            // Publish the preceding untracked edits when a full rebuild is required.
            let _ = materials.get_mut(id).map(|material| material.into_inner());
        }
    }
}

pub(super) fn play_native_mesh_effect_animations(
    mut commands: Commands,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    parents: Query<&ChildOf>,
    roots: Query<&NativeEffectRoot>,
    projectiles: Query<&NativeProjectile>,
    vehicle_tips: Query<&vehicle_trails::Tip>,
    mut effects: Query<(
        Entity,
        &ChildOf,
        &mut NativeMeshEffectPlayback,
        &mut Visibility,
    )>,
    mut players: Query<(Entity, &mut AnimationPlayer)>,
) {
    for (effect_entity, parent, mut effect, mut visibility) in &mut effects {
        let owner = parent.parent();
        let waiting_for_mesh_surface = roots
            .get(owner)
            .map(|root| root.waiting_for_mesh_surface)
            .or_else(|_| {
                projectiles
                    .get(owner)
                    .map(|projectile| projectile.waiting_for_mesh_surface)
            })
            .or_else(|_| vehicle_tips.get(effect_entity).map(|tip| !tip.ready));
        let Ok(waiting_for_mesh_surface) = waiting_for_mesh_surface else {
            continue;
        };
        if waiting_for_mesh_surface {
            continue;
        }
        if !effect.resolved {
            let Some(gltf) = gltfs.get(&effect.gltf) else {
                continue;
            };
            let clip = gltf
                .named_animations
                .get("nif-default")
                .cloned()
                .or_else(|| gltf.animations.first().cloned());
            effect.graph = clip.map(|clip| {
                let (graph, node) = AnimationGraph::from_clip(clip);
                (graphs.add(graph), node)
            });
            effect.resolved = true;
        }
        if effect.started {
            continue;
        }
        let Some((graph, node)) = effect.graph.clone() else {
            *visibility = Visibility::Inherited;
            effect.started = true;
            continue;
        };
        let mut found = false;
        for (player_entity, mut player) in &mut players {
            if !is_descendant_of(player_entity, effect_entity, &parents) {
                continue;
            }
            let playback = player.play(node);
            if effect.repeat {
                playback.repeat();
            }
            commands
                .entity(player_entity)
                .insert(AnimationGraphHandle(graph.clone()));
            found = true;
        }
        if found {
            effect.started = true;
            *visibility = Visibility::Inherited;
        }
    }
}

pub(super) fn native_sword_trail_mesh(edges: &[NativeSwordTrailEdge]) -> Mesh {
    let mut positions = Vec::with_capacity(edges.len() * 2);
    let mut colors = Vec::with_capacity(edges.len() * 2);
    let mut uvs = Vec::with_capacity(edges.len() * 2);
    for (index, edge) in edges.iter().enumerate() {
        positions.push(edge.top.to_array());
        positions.push(edge.bottom.to_array());
        let last = index + 1 == edges.len();
        colors.push(if last {
            [1.0, 1.0, 1.0, 1.0]
        } else {
            [0.5, 0.5, 0.5, 0.5]
        });
        colors.push([0.0, 0.0, 0.0, 0.0]);
        let uv_factor = if last {
            1.0
        } else {
            (index + 1) as f32 / edges.len() as f32
        };
        uvs.push([uv_factor, 0.0]);
        uvs.push([uv_factor, 1.0]);
    }
    // `SwordTrailController.Init` allocates `trailLength * 6` indices but
    // writes only 29 quads, leaving the final six entries at zero.
    let mut indices = vec![0_u32; edges.len() * 6];
    for index in 0..edges.len().saturating_sub(1) {
        let base = (index * 2) as u32;
        indices[index * 6..index * 6 + 6].copy_from_slice(&[
            base,
            base + 1,
            base + 2,
            base + 2,
            base + 1,
            base + 3,
        ]);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(Indices::U32(indices))
}

pub(super) fn trail_mesh(points: &[Vec3], camera_right: Vec3, elapsed: f32, height: f32) -> Mesh {
    let length = points.len();
    let direction = (points[length - 1] - points[length - 2]).normalize_or_zero();
    let mut positions = Vec::with_capacity(length * 2);
    let mut uvs = Vec::with_capacity(length * 2);
    let mut colors = Vec::with_capacity(length * 2);
    for (index, point) in points.iter().copied().enumerate() {
        let fraction = index as f32 / (length - 1) as f32;
        let axis = if direction == Vec3::ZERO {
            camera_right
        } else {
            Quat::from_axis_angle(
                direction,
                fraction * std::f32::consts::TAU + elapsed * 720_f32.to_radians(),
            ) * camera_right
        };
        let half = axis * height * 0.5;
        positions.push((point + half).to_array());
        // ParticleTrailController's final source edge contains the original
        // PLUS/PLUS typo; preserve it rather than repairing the mesh.
        positions.push(if index + 1 == length {
            (point + half).to_array()
        } else {
            (point - half).to_array()
        });
        // The legacy strip advances along texture U; V selects the two sides
        // of each edge. The terminal pair is explicitly reset to U=0.
        let uv_factor = if index + 1 == length {
            0.0
        } else {
            1.0 - (index + 1) as f32 / length as f32
        };
        uvs.push([uv_factor, 0.0]);
        uvs.push([uv_factor, 1.0]);
        let shade = index.min(length - 2) as f32 / (length - 1) as f32;
        colors.push([shade, shade, shade, shade]);
        colors.push([shade, shade, shade, shade]);
    }
    let mut indices = vec![0_u32; length * 6];
    for index in 0..length - 1 {
        let base = (index * 2) as u32;
        indices[index * 6..index * 6 + 6].copy_from_slice(&[
            base,
            base + 1,
            base + 2,
            base + 2,
            base + 1,
            base + 3,
        ]);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(Indices::U32(indices))
}
