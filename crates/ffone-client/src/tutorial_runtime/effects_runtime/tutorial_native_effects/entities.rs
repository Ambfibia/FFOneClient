use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    runtime: &mut TutorialEffectRuntime,
    registry: Option<&TutorialActorRegistry>,
    names: &Query<&Name>,
    children: &Query<&Children>,
    globals: &Query<&GlobalTransform>,
    instance_id: u64,
    effect_id: i32,
    streamed_world: bool,
    placement: TutorialEffectPlacement,
    scale: f32,
    name: Option<String>,
    destroy_after_seconds: Option<f32>,
    source_line: u32,
    plan: NativeEffectPlan,
    preload_cache: &mut NativeEffectPreloadCache,
) {
    let (stream_owned, stream_owner) = streamed_effect_ownership(streamed_world, &placement);
    let direct_parent = match &placement {
        TutorialEffectPlacement::ExactEntityWorld { root_entity, .. } => Some(*root_entity),
        _ => None,
    };
    let parent = if let Some(parent) = direct_parent {
        Some(parent)
    } else if let Some(proof) = placement.attachment_proof() {
        let (root, node_name) = match &proof {
            TutorialEffectAttachmentProof::Actor {
                actor_id,
                node_name,
            } => (
                registry.and_then(|registry| registry.entity(*actor_id)),
                node_name,
            ),
            TutorialEffectAttachmentProof::EntityRoot {
                root_entity,
                node_name,
            } => (Some(*root_entity), node_name),
        };
        let mut matches = Vec::new();
        if let Some(root) = root {
            collect_named(root, node_name, names, children, &mut matches);
        }
        if matches.len() != 1 {
            match proof {
                TutorialEffectAttachmentProof::Actor {
                    actor_id,
                    node_name,
                } => runtime.issues.push_back(
                    TutorialEffectRuntimeIssue::ExactBoneAttachmentUnavailable {
                        actor_id,
                        node_name,
                        match_count: matches.len(),
                        source_line,
                    },
                ),
                TutorialEffectAttachmentProof::EntityRoot {
                    root_entity,
                    node_name,
                } => runtime.issues.push_back(
                    TutorialEffectRuntimeIssue::ExactEntityBoneAttachmentUnavailable {
                        root_entity,
                        node_name,
                        match_count: matches.len(),
                        source_line,
                    },
                ),
            }
            runtime.forget_instance(instance_id);
            return;
        }
        Some(matches[0])
    } else {
        None
    };
    let transform = match placement {
        TutorialEffectPlacement::World { position, rotation } => {
            Transform::from_translation(position)
                .with_rotation(rotation)
                .with_scale(Vec3::splat(scale))
        }
        TutorialEffectPlacement::ExactEntityWorld {
            root_entity,
            position,
            rotation,
        } => {
            let world = Transform::from_translation(position)
                .with_rotation(rotation)
                .with_scale(Vec3::splat(scale));
            globals
                .get(root_entity)
                .map(|parent| {
                    Transform::from_matrix(parent.to_matrix().inverse() * world.to_matrix())
                })
                .unwrap_or(world)
        }
        TutorialEffectPlacement::ExactBone {
            local_translation_after_parenting,
            local_rotation_after_parenting,
            ..
        }
        | TutorialEffectPlacement::ExactEntityBone {
            local_translation_after_parenting,
            local_rotation_after_parenting,
            ..
        } => {
            // Unity's `transform.parent = bone` keeps world scale. Bevy's
            // `ChildOf` uses a local transform, so cancel inherited rig scale.
            let parent_scale = parent
                .and_then(|parent| globals.get(parent).ok())
                .map(|global| global.to_scale_rotation_translation().0)
                .unwrap_or(Vec3::ONE);
            let local_scale = if parent_scale.abs().min_element() > 0.000_001 {
                Vec3::splat(scale) / parent_scale
            } else {
                Vec3::splat(scale)
            };
            Transform::from_translation(local_translation_after_parenting)
                .with_rotation(local_rotation_after_parenting)
                .with_scale(local_scale)
        }
    };
    let waiting_for_mesh_surface = plan.mesh_scene.is_some();
    let components = (
        Name::new(name.unwrap_or_else(|| format!("Tutorial effect {effect_id}"))),
        NativeRoot { instance_id },
        NativeEffectRoot {
            stream_owner,
            age: 0.0,
            destroy_after: destroy_after_seconds,
            natural_destroy_after: (!plan.disable_update && plan.maximum_timer > 0.0)
                .then_some(plan.maximum_timer + plan.longest_lifetime),
            material_animation: plan.material_animation.clone(),
            waiting_for_mesh_surface,
        },
        transform,
        Visibility::Inherited,
    );
    let root = if let Some(parent) = parent {
        commands.spawn((components, ChildOf(parent))).id()
    } else {
        commands.spawn(components).id()
    };
    // The native mission marker meshes contain the animated symbol rig, but
    // no BillboardCamera node. Give their attachment root the camera-facing
    // behavior while leaving the authored bobbing and spin below it intact.
    if matches!(effect_id, 865 | 866) && parent.is_some() {
        commands.entity(root).insert(NativeEffectBillboard { upright: true });
    }
    runtime.bind_native_root(instance_id, root);
    if let Some(path) = plan.mesh_scene {
        let gltf = preload_cache
            .gltfs
            .entry(effect_id)
            .or_insert_with(|| asset_server.load(path))
            .clone();
        let scene = preload_cache
            .scenes
            .entry(effect_id)
            .or_insert_with(|| asset_server.load(GltfAssetLabel::Scene(0).from_asset(path)))
            .clone();
        commands.spawn((
            Name::new(format!("Original Unity mesh effect es{effect_id}")),
            WorldAssetRoot(scene),
            NativeMeshEffectLifetime::from_plan(&plan),
            NativeMeshEffectPlayback {
                gltf,
                graph: None,
                repeat: mesh_effect_repeats_standard_animation(effect_id),
                resolved: false,
                started: false,
            },
            Transform::IDENTITY,
            Visibility::Hidden,
            ChildOf(root),
        ));
    }
    for emitter in plan.emitters {
        let initial_translation = emitter.initial_translation;
        let preserve_when_streamed_budget_is_full =
            streamed_world && persistent_streamed_landmark_particle(&emitter);
        commands.spawn((
            Name::new(format!(
                "Unity particle {}#{}",
                emitter.source_asset, emitter.source_path_id
            )),
            NativeEmitter {
                owner: instance_id,
                stream_owner,
                plan: Arc::new(emitter),
                static_material: None,
                stream_owned,
                preserve_when_streamed_budget_is_full,
                scale,
                age: 0.0,
                timer: 2.0,
                maximum_timer: plan.maximum_timer,
                disable_update: plan.disable_update,
            },
            Transform::from_translation(initial_translation),
            Visibility::Inherited,
            ChildOf(root),
        ));
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_projectile(
    commands: &mut Commands,
    asset_server: &AssetServer,
    runtime: &mut TutorialEffectRuntime,
    meshes: &mut Assets<Mesh>,
    images: &mut Assets<Image>,
    materials: &mut Assets<TutorialParticleMaterial>,
    visual_assets: &mut NativeVisualAssets,
    instance_id: u64,
    effect_id: i32,
    source: Vec3,
    target: Vec3,
    scale: f32,
    reverse: bool,
    sampled_initial_velocity: Vec3,
    plan: NativeProjectilePlan,
) {
    let Some(state) = TutorialOniProjectileState::new(source, sampled_initial_velocity, reverse)
    else {
        runtime.forget_instance(instance_id);
        return;
    };
    spawn_projectile_visual(
        commands,
        asset_server,
        meshes,
        images,
        materials,
        visual_assets,
        instance_id,
        effect_id,
        format!("Tutorial Oni projectile es{effect_id}"),
        source,
        target,
        scale,
        NativeProjectileMotion::Oni(state),
        None,
        plan,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_linear_projectile(
    commands: &mut Commands,
    asset_server: &AssetServer,
    runtime: &mut TutorialEffectRuntime,
    meshes: &mut Assets<Mesh>,
    images: &mut Assets<Image>,
    materials: &mut Assets<TutorialParticleMaterial>,
    visual_assets: &mut NativeVisualAssets,
    instance_id: u64,
    bullet_type: i32,
    effect_id: i32,
    source: Vec3,
    target: Vec3,
    scale: f32,
    motion: NativeLinearProjectileMotion,
    impact: Option<NativeLinearImpactPlan>,
    plan: Option<NativeProjectilePlan>,
    carried_effect: Option<NativeEffectPlan>,
) {
    if !source.is_finite() || !target.is_finite() || !scale.is_finite() {
        runtime.forget_instance(instance_id);
        return;
    }
    let direction = (target - source).normalize_or_zero();
    let motion = match motion {
        NativeLinearProjectileMotion::BulletMove {
            hide_seconds,
            duration_seconds,
        } if hide_seconds.is_finite()
            && hide_seconds >= 0.0
            && duration_seconds.is_finite()
            && duration_seconds >= 0.0 =>
        {
            NativeProjectileMotion::Linear {
                position: source,
                hide_remaining: hide_seconds,
                elapsed: 0.0,
                duration: duration_seconds,
            }
        }
        NativeLinearProjectileMotion::Warhead {
            speed,
            initial_vertical_speed,
            duration_seconds,
            authority,
        } if speed.is_finite()
            && speed >= 0.0
            && initial_vertical_speed.is_none_or(|speed| speed.is_finite())
            && duration_seconds.is_finite()
            && duration_seconds >= 0.0 =>
        {
            let mut velocity = direction * speed;
            if let Some(vertical_speed) = initial_vertical_speed {
                velocity.y += vertical_speed;
            }
            NativeProjectileMotion::Warhead {
                position: source,
                velocity,
                gravity: initial_vertical_speed.is_some(),
                authority,
                elapsed: 0.0,
                duration: duration_seconds,
            }
        }
        _ => {
            runtime.forget_instance(instance_id);
            return;
        }
    };
    if let Some(plan) = plan {
        spawn_projectile_visual(
            commands,
            asset_server,
            meshes,
            images,
            materials,
            visual_assets,
            instance_id,
            effect_id,
            format!("Tutorial player bullet {bullet_type} es{effect_id}"),
            source,
            target,
            scale,
            motion,
            impact,
            plan,
        );
    } else {
        // BulletMoveScript parents any ordinary particle effect to its moving
        // carrier. BulletTable 106 has no particle script and remains the one
        // exact invisible melee carrier.
        let initial_direction = target - source;
        let initial_rotation = (initial_direction.length_squared() > f32::EPSILON)
            .then(|| Quat::from_rotation_arc(Vec3::Z, initial_direction.normalize()))
            .unwrap_or(Quat::IDENTITY);
        let projectile = commands
            .spawn((
                Name::new(if carried_effect.is_some() {
                    format!("Tutorial player bullet {bullet_type} es{effect_id}")
                } else {
                    format!("Tutorial invisible mob bullet {bullet_type}")
                }),
                NativeRoot { instance_id },
                NativeProjectile {
                    target,
                    motion,
                    impact,
                    waiting_for_mesh_surface: false,
                    waiting_for_trail_prewarm: false,
                },
                Transform::from_translation(source)
                    .with_rotation(initial_rotation)
                    .with_scale(Vec3::splat(scale)),
                Visibility::Inherited,
            ))
            .id();
        if let Some(plan) = carried_effect {
            spawn_carried_effect(
                commands,
                asset_server,
                projectile,
                instance_id,
                effect_id,
                scale,
                plan,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_carried_effect(
    commands: &mut Commands,
    asset_server: &AssetServer,
    projectile: Entity,
    instance_id: u64,
    effect_id: i32,
    scale: f32,
    plan: NativeEffectPlan,
) {
    let effect_root = commands
        .spawn((
            Name::new(format!("Carried Unity particle effect es{effect_id}")),
            NativeEffectRoot {
                stream_owner: None,
                age: 0.0,
                destroy_after: None,
                natural_destroy_after: (!plan.disable_update && plan.maximum_timer > 0.0)
                    .then_some(plan.maximum_timer + plan.longest_lifetime),
                material_animation: plan.material_animation.clone(),
                waiting_for_mesh_surface: plan.mesh_scene.is_some(),
            },
            Transform::IDENTITY,
            Visibility::Inherited,
            ChildOf(projectile),
        ))
        .id();
    if let Some(path) = plan.mesh_scene {
        commands.spawn((
            Name::new(format!("Original Unity carried mesh effect es{effect_id}")),
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(path))),
            NativeMeshEffectLifetime::from_plan(&plan),
            NativeMeshEffectPlayback {
                gltf: asset_server.load(path),
                graph: None,
                repeat: mesh_effect_repeats_standard_animation(effect_id),
                resolved: false,
                started: false,
            },
            Transform::IDENTITY,
            Visibility::Hidden,
            ChildOf(effect_root),
        ));
    }
    for emitter in plan.emitters {
        let initial_translation = emitter.initial_translation;
        commands.spawn((
            Name::new(format!(
                "Unity carried particle {}#{}",
                emitter.source_asset, emitter.source_path_id
            )),
            NativeEmitter {
                owner: instance_id,
                stream_owner: None,
                plan: Arc::new(emitter),
                static_material: None,
                stream_owned: false,
                preserve_when_streamed_budget_is_full: false,
                scale,
                age: 0.0,
                timer: 2.0,
                maximum_timer: plan.maximum_timer,
                disable_update: plan.disable_update,
            },
            Transform::from_translation(initial_translation),
            Visibility::Inherited,
            ChildOf(effect_root),
        ));
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_projectile_visual(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    images: &mut Assets<Image>,
    materials: &mut Assets<TutorialParticleMaterial>,
    visual_assets: &mut NativeVisualAssets,
    instance_id: u64,
    effect_id: i32,
    name: String,
    source: Vec3,
    target: Vec3,
    scale: f32,
    motion: NativeProjectileMotion,
    impact: Option<NativeLinearImpactPlan>,
    plan: NativeProjectilePlan,
) {
    let waiting_for_mesh_surface = plan.mesh_scene.is_some();
    let waiting_for_trail_prewarm = plan.trail.is_some();
    let initial_direction = match &motion {
        NativeProjectileMotion::Oni(state) => state.velocity,
        NativeProjectileMotion::Linear { position, .. } => target - *position,
        NativeProjectileMotion::Warhead { velocity, .. } => *velocity,
    };
    let initial_rotation = (initial_direction.length_squared() > f32::EPSILON)
        .then(|| Quat::from_rotation_arc(Vec3::Z, initial_direction.normalize()))
        .unwrap_or(Quat::IDENTITY);
    let projectile = commands
        .spawn((
            Name::new(name),
            NativeRoot { instance_id },
            NativeProjectile {
                target,
                motion,
                impact,
                waiting_for_mesh_surface,
                waiting_for_trail_prewarm,
            },
            Transform::from_translation(source)
                .with_rotation(initial_rotation)
                .with_scale(Vec3::splat(scale)),
            Visibility::Inherited,
        ))
        .id();
    let NativeProjectilePlan {
        trail,
        mesh_scene,
        material_animation,
        ..
    } = plan;
    if let Some(path) = mesh_scene {
        let gltf = asset_server.load(path);
        let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset(path));
        let mut mesh = commands.spawn((
            Name::new(format!("Original Unity projectile mesh es{effect_id}")),
            WorldAssetRoot(scene),
            NativeMeshEffectPlayback {
                gltf,
                graph: None,
                repeat: projectile_mesh_repeats_standard_animation(effect_id),
                resolved: false,
                started: false,
            },
            Transform::IDENTITY,
            Visibility::Inherited,
            ChildOf(projectile),
        ));
        if let Some(plan) = material_animation {
            mesh.insert(
                TutorialEffectMaterialAnimation {
                    age: 0.0,
                    plan,
                    paused: false,
                }
                .paused(),
            );
        }
    }
    let Some(plan) = trail else {
        return;
    };
    let material = shared_trail_material(&plan, images, materials, visual_assets);
    let points = vec![source; plan.length];
    let mesh = meshes.add(trail_mesh(&points, Vec3::X, 0.0, plan.height * scale));
    commands.spawn((
        Name::new(format!(
            "Unity trail {}#{}",
            plan.source_asset, plan.source_path_id
        )),
        Mesh3d(mesh),
        MeshMaterial3d(material),
        NativeTrail {
            projectile,
            plan,
            scale,
            points,
            elapsed: 0.0,
        },
        NativeDetachedOwner(instance_id),
        Transform::IDENTITY,
        Visibility::Inherited,
        NoFrustumCulling,
    ));
}

pub(super) fn spawn_world_impact(
    commands: &mut Commands,
    asset_server: Option<&AssetServer>,
    target: Vec3,
    rotation: Quat,
    impact: NativeLinearImpactPlan,
) {
    let NativeLinearImpactPlan {
        instance_id,
        effect_id,
        scale,
        plan,
        ..
    } = impact;
    let waiting_for_mesh_surface = plan.mesh_scene.is_some() && asset_server.is_some();
    let root = commands
        .spawn((
            Name::new(format!("Tutorial bullet impact es{effect_id}")),
            NativeRoot { instance_id },
            NativeEffectRoot {
                stream_owner: None,
                age: 0.0,
                destroy_after: None,
                natural_destroy_after: (!plan.disable_update && plan.maximum_timer > 0.0)
                    .then_some(plan.maximum_timer + plan.longest_lifetime),
                material_animation: plan.material_animation.clone(),
                waiting_for_mesh_surface,
            },
            Transform::from_translation(target)
                .with_rotation(rotation)
                .with_scale(Vec3::splat(scale)),
            Visibility::Inherited,
        ))
        .id();
    if let (Some(path), Some(asset_server)) = (plan.mesh_scene, asset_server) {
        let gltf = asset_server.load(path);
        let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset(path));
        commands.spawn((
            Name::new(format!("Original Unity impact mesh es{effect_id}")),
            WorldAssetRoot(scene),
            NativeMeshEffectLifetime::from_plan(&plan),
            NativeMeshEffectPlayback {
                gltf,
                graph: None,
                repeat: mesh_effect_repeats_standard_animation(effect_id),
                resolved: false,
                started: false,
            },
            Transform::IDENTITY,
            Visibility::Hidden,
            ChildOf(root),
        ));
    }
    for emitter in plan.emitters {
        let initial_translation = emitter.initial_translation;
        commands.spawn((
            Name::new(format!(
                "Unity particle {}#{}",
                emitter.source_asset, emitter.source_path_id
            )),
            NativeEmitter {
                owner: instance_id,
                stream_owner: None,
                plan: Arc::new(emitter),
                static_material: None,
                stream_owned: false,
                preserve_when_streamed_budget_is_full: false,
                scale,
                age: 0.0,
                timer: 2.0,
                maximum_timer: plan.maximum_timer,
                disable_update: plan.disable_update,
            },
            Transform::from_translation(initial_translation),
            Visibility::Inherited,
            ChildOf(root),
        ));
    }
}

pub(super) fn despawn_native_instance(
    id: u64,
    commands: &mut Commands,
    roots: &Query<(Entity, &NativeRoot)>,
    detached: &Query<(Entity, &NativeDetachedOwner)>,
) {
    for (entity, root) in roots {
        if root.instance_id == id {
            commands.entity(entity).despawn();
        }
    }
    for (entity, owner) in detached {
        if owner.0 == id {
            commands.entity(entity).despawn();
        }
    }
}
