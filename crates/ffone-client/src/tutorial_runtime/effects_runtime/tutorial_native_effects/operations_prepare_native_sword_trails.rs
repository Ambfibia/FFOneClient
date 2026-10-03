use super::*;

pub(super) fn prewarm_native_projectile_visuals(
    mut commands: Commands,
    runtime: Res<TutorialEffectRuntime>,
    cameras: Query<(), With<Camera3d>>,
    mut state: ResMut<NativeProjectileVisualPrewarm>,
    mut readiness: ResMut<TutorialProjectileVisualReadiness>,
    surfaces: Query<Entity, With<NativeProjectileVisualPrewarmSurface>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TutorialParticleMaterial>>,
    mut visual_assets: ResMut<NativeVisualAssets>,
) {
    if state.initialized {
        if !cameras.is_empty() {
            state.camera_frames = state.camera_frames.saturating_add(1);
        }
        // Keep the clipped warm-up draws alive for several complete render
        // extracts. This covers a camera created after the effect plugin and
        // avoids making the first 0.3 s shot pay shader/texture preparation.
        if state.camera_frames >= 8 {
            for entity in &surfaces {
                commands.entity(entity).despawn();
            }
            readiness.ready = true;
        }
        return;
    }
    let Some(library) = runtime.library.as_deref() else {
        return;
    };
    for (effect_id, effect) in &library.projectile_effects {
        let compiled = compile_projectile_plan(*effect_id, effect);
        let Some(trail) = compiled.plan.and_then(|plan| plan.trail) else {
            continue;
        };
        let material =
            shared_trail_material(&trail, &mut images, &mut materials, &mut visual_assets);
        let length = trail.length.max(2);
        let points = (0..length)
            .map(|index| Vec3::X * index as f32 * 0.01)
            .collect::<Vec<_>>();
        let mesh = meshes.add(trail_mesh(&points, Vec3::Y, 0.0, trail.height));
        commands.spawn((
            Name::new(format!("Projectile trail GPU prewarm es{effect_id}")),
            NativeProjectileVisualPrewarmSurface,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_xyz(0.0, -1_000_000.0, 0.0),
            Visibility::Inherited,
            NoFrustumCulling,
        ));
    }
    state.initialized = true;
    readiness.ready = false;
}

pub(super) fn prepare_native_sword_trails(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TutorialParticleMaterial>>,
    mut visual_assets: ResMut<NativeVisualAssets>,
    attachments: Query<(Entity, &TutorialPlayerWeaponAttachment), Without<NativeSwordTrailBound>>,
    rigs: Query<&TutorialSelectedPlayerRig>,
    names: Query<&Name>,
    children: Query<&Children>,
) {
    for (attachment_entity, attachment) in &attachments {
        let Ok(rig) = rigs.get(attachment.rig_root) else {
            continue;
        };
        if rig.attached_weapon() != Some(attachment_entity) {
            continue;
        }

        let mut top_matches = Vec::new();
        let mut bottom_matches = Vec::new();
        collect_named(
            attachment_entity,
            "Stag01",
            &names,
            &children,
            &mut top_matches,
        );
        collect_named(
            attachment_entity,
            "Stag02",
            &names,
            &children,
            &mut bottom_matches,
        );
        let ([top_point], [bottom_point]) = (top_matches.as_slice(), bottom_matches.as_slice())
        else {
            // The GLTF scene may still be materializing. Retrobution also
            // requires one sequential Stag pair and creates no trail until it
            // can resolve both points.
            continue;
        };

        let texture = visual_assets
            .sword_trail_texture
            .get_or_insert_with(|| asset_server.load(RETROBUTION_SWORD_TRAIL_TEXTURE))
            .clone();
        let material = visual_assets
            .sword_trail_material
            .get_or_insert_with(|| {
                materials.add(TutorialParticleMaterial {
                    uniform: TutorialParticleUniform {
                        tint: RETROBUTION_SWORD_TRAIL_TINT,
                        // SwordTrailMaterial PathID 879 serializes
                        // `_MainTex` scale (1,-1) with a zero offset.
                        uv_scale_offset: Vec4::new(1.0, -1.0, 0.0, 0.0),
                        legacy_gamma_accumulation_gain: 1.0,
                    },
                    texture,
                    blend_mode: ParticleBlendMode::SrcAlphaOne,
                })
            })
            .clone();
        let edges = vec![NativeSwordTrailEdge::default(); RETROBUTION_SWORD_TRAIL_LENGTH];
        let mesh = meshes.add(native_sword_trail_mesh(&edges));
        commands.spawn((
            Name::new("Retrobution SwordTrail Stag01/Stag02"),
            Mesh3d(mesh),
            MeshMaterial3d(material),
            NativeSwordTrail {
                attachment: attachment_entity,
                controller_root: rig.controller_root,
                rig_root: attachment.rig_root,
                top_point: *top_point,
                bottom_point: *bottom_point,
                edges,
                emitting: false,
                attack_generation: None,
                elapsed_seconds: 0.0,
            },
            ChildOf(attachment.rig_root),
            Transform::IDENTITY,
            Visibility::Hidden,
            NoFrustumCulling,
        ));
        commands
            .entity(attachment_entity)
            .insert(NativeSwordTrailBound);
    }
}

pub(super) fn native_sword_attack_active(action: &LegacyAvatarActionState) -> bool {
    matches!(action.base_action(), Some(LegacyVisualClip::AttackFull(_)))
        || matches!(action.upper_action, Some(LegacyVisualClip::AttackUpper(_)))
}

pub(super) fn push_native_sword_trail_edge(edges: &mut [NativeSwordTrailEdge], edge: NativeSwordTrailEdge) {
    edges.rotate_left(1);
    if let Some(last) = edges.last_mut() {
        *last = edge;
    }
}

pub(super) fn particle_quad() -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [-0.5, -0.5, 0.0],
            [0.5, -0.5, 0.0],
            [0.5, 0.5, 0.0],
            [-0.5, 0.5, 0.0],
        ],
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0_f32; 4]; 4])
    .with_inserted_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]))
}

/// `ParticleEmitterController.timer` is serialized by code with an initial
/// value of `2f`. Once `EffectEmitterController.Start` finishes attaching an
/// emitter and enables `bTimer`, its first `Update` therefore emits
/// immediately. Deferred Bevy spawning already supplies the one frame that
/// Unity needs to instantiate the emitter; adding another native warm-up
/// frame made short Nano summon effects visibly late.
pub(super) fn legacy_emission_due(timer: &mut f32, delta: f32, generations_per_second: f32) -> bool {
    if *timer + delta * generations_per_second < 1.0 {
        *timer += delta * generations_per_second;
        false
    } else {
        true
    }
}

pub(super) fn persistent_streamed_landmark_particle(plan: &EmitterPlan) -> bool {
    plan.lifetime >= 300.0
        && plan.generations_per_second <= 1.0 / 300.0
        && plan.number_per_generation > 0.0
        && plan.number_per_generation <= 1.0
}

pub(super) fn streamed_particle_spawn_count(
    requested: usize,
    remaining: &mut usize,
    preserve_when_full: bool,
) -> usize {
    let counted = requested.min(*remaining);
    *remaining -= counted;
    if preserve_when_full {
        requested
    } else {
        counted
    }
}

/// Keep ambient emission bounded by the clean client's camera distance without
/// making its lifetime depend on the direction in which the player happens to
/// look. Unity's `ParticleEmitterController.Update` runs independently of the
/// camera cone; normal renderer bounds/frustum culling decides whether an
/// emitted card contributes to the frame.
pub(super) fn world_effect_is_within_camera_distance(
    effect_position: Vec3,
    camera: &GlobalTransform,
    projection: &Projection,
) -> bool {
    let far = match projection {
        Projection::Perspective(perspective) => perspective.far.min(LEGACY_WORLD_CAMERA_FAR_NATIVE),
        Projection::Orthographic(orthographic) => {
            orthographic.far.min(LEGACY_WORLD_CAMERA_FAR_NATIVE)
        }
        Projection::Custom(_) => return true,
    };
    let radius = far + STREAMED_WORLD_EFFECT_FAR_MARGIN;
    let offset = effect_position - camera.translation();
    offset.length_squared() <= radius * radius
}

pub(super) fn native_emitted_particle_position(
    emitter_world_position: Vec3,
    generated_position: Vec3,
) -> Vec3 {
    emitter_world_position + generated_position
}

pub(super) fn simulate_particles(
    time: Res<Time>,
    cameras: Query<(&GlobalTransform, &Projection), (With<Camera3d>, With<LegacyOrbitCamera>)>,
    live_stream_owners: Query<Entity, Without<crate::world::PendingNativeWorldSceneUnload>>,
    mut particles: Query<(&mut Transform, &mut Visibility, &mut NativeParticle)>,
) {
    let delta = time.delta_secs();
    let camera = cameras
        .iter()
        .next()
        .map(|(transform, projection)| (transform.clone(), projection.clone()));
    particles
        .par_iter_mut()
        .for_each(|(mut transform, mut visibility, mut particle)| {
            if !particle.alive {
                return;
            }
            if particle
                .stream_owner
                .is_some_and(|owner| live_stream_owners.get(owner).is_err())
            {
                particle.alive = false;
                if *visibility != Visibility::Hidden {
                    *visibility = Visibility::Hidden;
                }
                return;
            }
            particle.age += delta;
            if particle.age > particle.plan.lifetime {
                particle.alive = false;
                if *visibility != Visibility::Hidden {
                    *visibility = Visibility::Hidden;
                }
                return;
            }
            // Do not hide or freeze a live card by testing only its centre
            // against a camera cone. Large 20--27 m authored cards can still
            // intersect the frustum while their centre is outside it; freezing
            // here made their late animation phase pop in after a camera turn.
            // Keep the Unity simulation continuous and let Bevy cull the
            // transformed quad from its real AABB.
            if *visibility != Visibility::Inherited {
                *visibility = Visibility::Inherited;
            }
            let force = particle.plan.force;
            if force != Vec3::ZERO {
                particle.velocity += force * delta;
            }
            let translation_delta = particle.velocity * delta;
            if translation_delta != Vec3::ZERO {
                transform.translation += translation_delta;
            }
            let normalized = (particle.age / particle.plan.lifetime).clamp(0.0, 1.0);
            let rotation = particle
                .plan
                .rotation_curve
                .evaluate(normalized, 0.0)
                .to_radians();
            let desired_rotation =
                camera
                    .as_ref()
                    .map_or(Quat::from_rotation_z(rotation), |(camera, _)| {
                        particle_billboard_rotation(
                            particle.plan.render_mode,
                            camera,
                            transform.translation,
                            rotation,
                        )
                    });
            if transform.rotation != desired_rotation {
                transform.rotation = desired_rotation;
            }
            let size = particle_size(&particle.plan, normalized, particle.scale);
            let desired_scale = Vec3::new(size.x, size.y, 1.0);
            if transform.scale != desired_scale {
                transform.scale = desired_scale;
            }
        });
}

pub(super) fn admit_particle_despawn(stream_owned: bool, remaining_streamed: &mut usize) -> bool {
    if !stream_owned {
        return true;
    }
    if *remaining_streamed == 0 {
        return false;
    }
    *remaining_streamed -= 1;
    true
}

pub(super) fn cleanup_expired_particles(mut commands: Commands, particles: Query<(Entity, &NativeParticle)>) {
    let mut remaining_streamed = STREAMED_WORLD_PARTICLE_DESPAWNS_PER_FRAME;
    for (entity, particle) in &particles {
        if !particle.alive && admit_particle_despawn(particle.stream_owned, &mut remaining_streamed)
        {
            commands.entity(entity).despawn();
        }
    }
}

pub(super) fn particle_billboard_rotation(
    mode: LegacyParticleRenderMode,
    camera: &GlobalTransform,
    particle_position: Vec3,
    roll_radians: f32,
) -> Quat {
    let base = match mode {
        LegacyParticleRenderMode::Billboard | LegacyParticleRenderMode::SortedBillboard => {
            camera.compute_transform().rotation
        }
        LegacyParticleRenderMode::HorizontalBillboard => {
            // The authored quad lies in local XY with +Z as its front.
            Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)
        }
        LegacyParticleRenderMode::VerticalBillboard => {
            let to_camera = camera.translation() - particle_position;
            let horizontal = Vec2::new(to_camera.x, to_camera.z);
            if horizontal.length_squared() <= f32::EPSILON {
                Quat::IDENTITY
            } else {
                // Rotate only around world Y: unlike a normal billboard this
                // never inherits camera pitch. ES669/ES686 depend on this.
                Quat::from_rotation_y(horizontal.x.atan2(horizontal.y))
            }
        }
    };
    base * Quat::from_rotation_z(roll_radians)
}

/// Unity's legacy ParticleRenderer walks atlas cells left-to-right and then
/// bottom-to-top in UV space. The exact texture decoder flips source rows for
/// Bevy, so cell zero remains the first top-memory row after conversion.
pub(super) fn particle_uv_scale_offset(plan: &EmitterPlan, normalized: f32) -> Vec4 {
    let tiles = plan.uv_tiles.max(UVec2::ONE);
    let frame = particle_material_animation_sample(plan, normalized).atlas_frame;
    let scale = Vec2::new(1.0 / tiles.x as f32, 1.0 / tiles.y as f32);
    let offset = Vec2::new(
        (frame % tiles.x) as f32 * scale.x,
        (frame / tiles.x) as f32 * scale.y,
    );
    Vec4::new(scale.x, scale.y, offset.x, offset.y)
}

pub(super) fn particle_color(plan: &EmitterPlan, normalized: f32) -> Vec4 {
    if !plan.animate_color {
        return Vec4::ONE;
    }
    let scaled = normalized.clamp(0.0, 1.0) * 4.0;
    let index = (scaled.floor() as usize).min(3);
    plan.colors[index].lerp(plan.colors[index + 1], scaled - index as f32)
}

pub(super) fn particle_size(plan: &EmitterPlan, normalized: f32, scale: f32) -> Vec2 {
    let width = (!plan.width_curve.0.is_empty())
        .then(|| plan.width_curve.evaluate(normalized, plan.initial_size));
    let height = (!plan.height_curve.0.is_empty())
        .then(|| plan.height_curve.evaluate(normalized, plan.initial_size));
    // Unity's legacy ParticleRenderer mirrors the authored dimension when
    // only one of m_WidthCurve/m_HeightCurve is serialized. ES767 has a
    // 20->30 width curve and an empty height curve; treating the latter as
    // initialSize=1 flattened every explosion into a horizontal strip.
    match (width, height) {
        (Some(width), Some(height)) => Vec2::new(width, height) * scale,
        (Some(size), None) | (None, Some(size)) => Vec2::splat(size * scale),
        (None, None) => Vec2::splat(plan.initial_size * scale),
    }
}

pub(super) fn warhead_sphere_hit(start: Vec3, end: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let offset = start - center;
    let c = offset.length_squared() - radius * radius;
    if c <= 0.0 { return Some(0.0); }
    let delta = end - start;
    let a = delta.length_squared();
    if a <= f32::EPSILON { return None; }
    let b = offset.dot(delta);
    let discriminant = b * b - a * c;
    if discriminant < 0.0 { return None; }
    let fraction = (-b - discriminant.sqrt()) / a;
    (0.0..=1.0).contains(&fraction).then_some(fraction)
}

pub(super) fn native_impact_playback_settings(effects_gain: f32) -> PlaybackSettings {
    PlaybackSettings::DESPAWN
        .with_volume(Volume::Linear(effects_gain.clamp(0.0, 1.0)))
        .with_spatial(true)
        .with_spatial_scale(LEGACY_SPATIAL_SCALE)
}

pub(super) fn push_interpolated_trail_points(points: &mut [Vec3], current: Vec3, step: f32) {
    let previous = *points.last().unwrap();
    let mut fraction = step;
    while fraction < 1.0 {
        push_trail_point(points, previous.lerp(current, fraction));
        fraction += step;
    }
    push_trail_point(points, current);
}

pub(super) fn push_trail_point(points: &mut [Vec3], point: Vec3) {
    points.rotate_left(1);
    *points.last_mut().unwrap() = point;
}

pub(super) fn cleanup_effect_roots(
    mut commands: Commands,
    time: Res<Time>,
    mut runtime: ResMut<TutorialEffectRuntime>,
    mut roots: Query<(Entity, Option<&NativeRoot>, &mut NativeEffectRoot)>,
    detached: Query<(Entity, &NativeDetachedOwner)>,
    live_stream_owners: Query<Entity, Without<crate::world::PendingNativeWorldSceneUnload>>,
) {
    for (entity, root, mut effect) in &mut roots {
        if effect
            .stream_owner
            .is_some_and(|owner| live_stream_owners.get(owner).is_err())
        {
            // The tile's post-order traversal owns this rooted subtree now.
            // Advancing its lifetime could trigger an unbudgeted recursive
            // despawn while that traversal is still in progress.
            continue;
        }
        if effect.age == 0.0
            && !effect.waiting_for_mesh_surface
            && let Some(root) = root
        {
            runtime.mark_native_presentation_ready(root.instance_id);
        }
        effect.advance_age(time.delta_secs());
        let expired = effect
            .destroy_after
            .is_some_and(|seconds| effect.age >= seconds)
            || effect
                .natural_destroy_after
                .is_some_and(|seconds| effect.age > seconds);
        if expired {
            commands.entity(entity).despawn();
            if let Some(root) = root {
                for (detached_entity, owner) in &detached {
                    if owner.0 == root.instance_id {
                        commands.entity(detached_entity).despawn();
                    }
                }
                runtime.forget_instance(root.instance_id);
            }
        }
    }
}

pub(super) fn cleanup_effect_meshes(
    mut commands: Commands,
    roots: Query<&NativeEffectRoot>,
    meshes: Query<(Entity, &ChildOf, &NativeMeshEffectLifetime)>,
) {
    for (entity, parent, lifetime) in &meshes {
        let Ok(root) = roots.get(parent.parent()) else {
            continue;
        };
        if lifetime.0.is_some_and(|deadline| root.age > deadline) {
            commands.entity(entity).despawn();
        }
    }
}

pub(super) fn cleanup_stream_owned_native_effects(
    mut runtime: ResMut<TutorialEffectRuntime>,
    live_stream_owners: Query<Entity, Without<crate::world::PendingNativeWorldSceneUnload>>,
) {
    let orphaned = runtime
        .active
        .iter()
        .filter_map(|(instance_id, instance)| {
            instance
                .stream_owner
                .is_some_and(|owner| live_stream_owners.get(owner).is_err())
                .then_some(*instance_id)
        })
        .collect::<Vec<_>>();
    for instance_id in orphaned {
        // Rooted ambient effects remain ChildOf the tile and are removed by
        // the world's global post-order unload budget. Detached particles
        // carry the same stream owner, stop simulating immediately, and leave
        // through the ordinary particle-lifetime cleanup instead of a burst.
        runtime.forget_instance(instance_id);
    }
}
