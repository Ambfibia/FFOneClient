use super::*;

/// Keep the published needed-jump wing mesh turning around the world's upright
/// axis. Its authored local Y axis lies horizontally after placement rotation.
/// The global clock restores the same phase after its tile streams back.
pub fn update_world_floating_icons(
    time: Res<Time>,
    mut icons: Query<(&mut WorldFloatingIconSpin, &mut Transform)>,
) {
    let angle = time.elapsed_secs() * std::f32::consts::FRAC_PI_2;
    for (mut spin, mut transform) in &mut icons {
        let initial = *spin.initial_rotation.get_or_insert(transform.rotation);
        transform.rotation = Quat::from_rotation_y(angle) * initial;
    }
}

/// Evaluate every auto-playing legacy world AnimationClip against the native
/// pivot hierarchy built from its exact source paths. Cubic sampling is the
/// expensive part and is independent per authored player, so it runs through
/// Bevy's parallel query executor. The following apply system performs the
/// short, deterministic target write phase.
pub(super) fn update_world_animations(
    time: Res<Time>,
    mut players: Query<(
        &mut WorldAnimationPlayer,
        &WorldAnimationCompiledTracks,
        &mut WorldAnimationSamples,
        &WorldAnimationVisibilityBindings,
    )>,
    view_visibility: Query<&ViewVisibility, With<Mesh3d>>,
) {
    let delta = time.delta_secs();
    players
        .par_iter_mut()
        .for_each(|(mut player, tracks, mut samples, visibility)| {
            samples.0.clear();
            if !player.play_automatically {
                return;
            }
            let has_renderers = visibility.ready && !visibility.renderers.is_empty();
            let any_renderer_visible = has_renderers
                && visibility.renderers.iter().any(|renderer| {
                    view_visibility
                        .get(*renderer)
                        .is_ok_and(|visibility| visibility.get())
                });
            // Unity's Animation.m_AnimateIfVisible pauses the animation clock,
            // not merely its transform writes. In particular the Marquee Row
            // orbiting billboard must remain at its authored bind pose until
            // its renderer becomes visible. Native scene admission can take
            // several frames, so advancing before bindings are ready also
            // introduced a load-time-dependent positional phase error.
            if world_animation_clock_is_paused(
                player.animate_only_if_visible,
                visibility.ready,
                has_renderers,
                any_renderer_visible,
            ) {
                return;
            }
            player.elapsed_seconds += delta;
            // Non-physics animation can be sampled lazily: its clock keeps
            // advancing and the exact current pose is restored the first frame
            // a renderer becomes visible. This avoids dirtying thousands of
            // off-screen transform trees while preserving moving colliders and
            // gameplay platforms through the explicit animate_physics owner.
            if !player.animate_physics && has_renderers && !any_renderer_visible {
                return;
            }
            let elapsed = f64::from(player.elapsed_seconds);
            let Some(clip) = player.active_clip() else {
                return;
            };
            let sample_time =
                world_animation_sample_time(elapsed, clip.duration, player.transform_repeats);
            for track in &tracks.0 {
                let translation = track.translation.and_then(|channel_index| {
                    let channel = clip.channels.get(channel_index)?;
                    sample_world_animation_vector_fixed::<3>(
                        channel,
                        world_channel_sample_time(channel, elapsed, sample_time),
                    )
                    .map(|value| Vec3::new(value[0] as f32, value[1] as f32, value[2] as f32))
                });
                let rotation = track.rotation.and_then(|channel_index| {
                    let channel = clip.channels.get(channel_index)?;
                    let value = sample_world_animation_vector_fixed::<4>(
                        channel,
                        world_channel_sample_time(channel, elapsed, sample_time),
                    )?;
                    let rotation = Quat::from_xyzw(
                        value[0] as f32,
                        value[1] as f32,
                        value[2] as f32,
                        value[3] as f32,
                    );
                    (rotation.is_finite() && rotation.length_squared() > f32::EPSILON)
                        .then(|| rotation.normalize())
                });
                let scale = track.scale.and_then(|channel_index| {
                    let channel = clip.channels.get(channel_index)?;
                    sample_world_animation_vector_fixed::<3>(
                        channel,
                        world_channel_sample_time(channel, elapsed, sample_time),
                    )
                    .map(|value| Vec3::new(value[0] as f32, value[1] as f32, value[2] as f32))
                });
                samples.0.push(WorldAnimationTargetSample {
                    entity: track.entity,
                    translation,
                    rotation,
                    scale,
                });
            }
        });
}

/// Rotate every billboard toward the active camera.
pub fn update_world_billboards(
    camera: Query<&GlobalTransform, With<Camera3d>>,
    mut billboards: Query<(&WorldBillboard, &ChildOf, &mut Transform)>,
    parents: Query<&GlobalTransform, Without<WorldBillboard>>,
) {
    let Ok(camera) = camera.single() else {
        return;
    };
    let camera_rotation = camera.rotation() * Quat::from_rotation_y(std::f32::consts::PI);
    let mut forward = camera.forward().as_vec3();
    forward.y = 0.0;
    let up_rotation = (forward.length_squared() > f32::EPSILON).then(|| {
        Transform::IDENTITY
            .looking_to(forward.normalize(), Vec3::Y)
            .rotation
            * Quat::from_rotation_y(std::f32::consts::PI)
    });
    for (billboard, child_of, mut transform) in &mut billboards {
        if !billboard.runtime_enabled {
            continue;
        }
        let parent_rotation = parents
            .get(child_of.parent())
            .map(GlobalTransform::rotation)
            .unwrap_or(Quat::IDENTITY);
        match billboard.mode {
            BillboardMode::Camera => {
                let desired_rotation = parent_rotation.inverse() * camera_rotation;
                if transform.rotation != desired_rotation {
                    transform.rotation = desired_rotation;
                }
            }
            BillboardMode::Up => {
                if let Some(rotation) = up_rotation {
                    let desired_rotation = parent_rotation.inverse() * rotation;
                    if transform.rotation != desired_rotation {
                        transform.rotation = desired_rotation;
                    }
                }
            }
            BillboardMode::RigidCamera | BillboardMode::Center | BillboardMode::RigidCenter => {}
        }
    }
}

/// Reproduce `VisibleSwitch.OnBecameVisible`'s observable state without
/// incorrectly hiding the renderer. Bevy already culls off-screen meshes; an
/// always-enabled billboard pivot produces identical visible frames and only
/// spends a small transform update while it is outside the frustum.
pub fn update_world_visibility_switches(
    switches: Query<&WorldVisibilitySwitch>,
    view_visibility: Query<&ViewVisibility, With<Mesh3d>>,
    children: Query<&Children>,
    mut billboards: Query<&mut WorldBillboard>,
) {
    for switch in &switches {
        let mut stack = switch.renderer_entities.clone();
        let mut visible = false;
        while let Some(entity) = stack.pop() {
            if view_visibility
                .get(entity)
                .is_ok_and(|visibility| visibility.get())
            {
                visible = true;
                break;
            }
            if let Ok(descendants) = children.get(entity) {
                stack.extend(descendants.iter());
            }
        }
        for entity in &switch.controlled_billboards {
            if let Ok(mut billboard) = billboards.get_mut(*entity) {
                if billboard.runtime_enabled != visible {
                    billboard.runtime_enabled = visible;
                }
            }
        }
    }
}

/// Reproduce the clean `EpPlatformTrigger.Update` phase and transform rules.
/// A global clock keeps independently streamed instances synchronized, as the
/// clean `EpSynchronizer` did after server-time initialization.
pub fn update_world_platforms(
    time: Res<Time>,
    mut platforms: Query<(&WorldPlatformMotion, &mut Transform)>,
) {
    let elapsed = time.elapsed_secs();
    platforms
        .par_iter_mut()
        .for_each(|(motion, mut transform)| {
            if !motion.velocity.is_finite() || motion.velocity.abs() <= f32::EPSILON {
                return;
            }
            let phase = elapsed * motion.velocity.abs();
            let path_motion = matches!(motion.move_type, 8..=10);
            let (timer, returning) = if path_motion {
                (phase.rem_euclid(1.0), false)
            } else {
                let cycle = phase.rem_euclid(2.0);
                if cycle < 1.0 {
                    (cycle, false)
                } else {
                    (2.0 - cycle, true)
                }
            };
            // The exact clean endpoint contract is
            // `initialPosition + initialRotation * (to - from)`: line motion
            // does not apply localScale.x. The old native path scaled that
            // delta, which moved Marquee Row's 1.25-scale pad past its source
            // endpoint. Curves and waypoint paths retain their separate scale.
            transform.translation = legacy_platform_world_translation(motion, timer, returning);
            let rotation_progress = if path_motion {
                timer * std::f32::consts::TAU * motion.spin_velocity
            } else {
                let direction = if returning { -1.0 } else { 1.0 };
                direction
                    * legacy_platform_eased_progress(timer)
                    * std::f32::consts::PI
                    * motion.spin_velocity
            };
            transform.rotation = match motion.move_type {
                4 | 6 | 9 => motion.initial_rotation * Quat::from_rotation_x(rotation_progress),
                // Native space reflects Unity X, so Unity Z rotation changes sign.
                5 | 7 | 10 => motion.initial_rotation * Quat::from_rotation_z(-rotation_progress),
                _ => motion.initial_rotation,
            };
            transform.scale = motion.initial_scale;
        });
}

/// Detect local-player entry into every authored sphere, box and capsule
/// trigger volume in the collider's exact local space.
pub fn update_world_trigger_volumes(
    mut commands: Commands,
    mut player: Query<(
        Entity,
        &GlobalTransform,
        &mut crate::movement::LegacyPlayerController,
        &mut LegacyAvatarTargetFeed,
    )>,
    mut volumes: Query<(
        Entity,
        &WorldTriggerVolume,
        &GlobalTransform,
        &mut WorldTriggerVolumeOccupied,
    )>,
    triggers: Query<(Entity, &WorldTrigger, &GlobalTransform)>,
) {
    let Ok((player_entity, player_global, mut controller, mut target_feed)) = player.single_mut()
    else {
        return;
    };
    let position = player_global.translation();
    let mut nearest_interaction: Option<(Entity, f32, f32)> = None;
    let mut has_volume_source = false;
    for (_entity, volume, global, mut occupied) in &mut volumes {
        if !volume.is_trigger {
            continue;
        }
        has_volume_source = true;
        let local_position = global.affine().inverse().transform_point3(position);
        let inside = trigger_volume_contains_local_point(volume, local_position);
        let entered = inside && !occupied.0;
        if inside != occupied.0 {
            occupied.0 = inside;
        }
        let Some(trigger_entity) = volume.trigger else {
            continue;
        };
        let Ok((_, trigger, trigger_global)) = triggers.get(trigger_entity) else {
            continue;
        };
        if entered && trigger.kind == WorldTriggerKind::Jumppad {
            let power = trigger.add_power.max(0.0);
            if power.is_finite() && power > 0.0 {
                commands.entity(player_entity).insert(WorldJumppadArmed {
                    trigger: trigger_entity,
                    power,
                    remaining_seconds: 3.0,
                });
            }
        }
        if entered && trigger.kind == WorldTriggerKind::Slope && trigger.path_points.len() >= 2 {
            let points = trigger
                .path_points
                .iter()
                .map(|point| trigger_global.transform_point(*point))
                .collect::<Vec<_>>();
            let nearest = points
                .iter()
                .enumerate()
                .min_by(|(_, left), (_, right)| {
                    left.distance_squared(position)
                        .total_cmp(&right.distance_squared(position))
                })
                .map_or(0, |(index, _)| index)
                .min(points.len() - 2);
            controller.begin_scripted_traversal();
            commands.entity(player_entity).insert(WorldSlopeTraversal {
                points,
                speed: trigger.speed.max(0.01),
                segment: nearest,
                drift: Vec3::ZERO,
                packet_elapsed: f32::INFINITY,
                slope_id: trigger.object_id as i32,
            });
        }
        if inside
            && matches!(
                trigger.kind,
                WorldTriggerKind::Launcher
                    | WorldTriggerKind::Zipline
                    | WorldTriggerKind::Switch
                    | WorldTriggerKind::Rope
            )
        {
            let distance = global.translation().distance(position);
            if nearest_interaction.is_none_or(|(_, nearest, _)| distance < nearest) {
                nearest_interaction = Some((trigger_entity, distance, trigger.radius));
            }
        }
    }
    for (trigger_entity, trigger, global) in &triggers {
        if trigger.kind != WorldTriggerKind::Rope || trigger.path_points.len() < 2 {
            continue;
        }
        let distance = trigger
            .path_points
            .windows(2)
            .map(|segment| {
                let start = global.transform_point(segment[0]);
                let end = global.transform_point(segment[1]);
                distance_to_segment(position, start, end)
            })
            .fold(f32::INFINITY, f32::min);
        if distance <= trigger.radius.max(1.0)
            && nearest_interaction.is_none_or(|(_, nearest, _)| distance < nearest)
        {
            nearest_interaction = Some((trigger_entity, distance, trigger.radius));
            has_volume_source = true;
        }
    }
    if has_volume_source {
        target_feed.source_connected = true;
        target_feed.trigger =
            nearest_interaction.map(|(entity, _, radius)| LegacyTriggerSample { entity, radius });
    }
}

/// One cable Move awaiting authored collision and the post-Move hang offset.
#[derive(Component)]
pub struct WorldZiplineStep {
    detached: bool,
    finished: bool,
}

/// Submit the cable's CharacterController.Move. EpUpdate first moves the root
/// to the cable (with the CCT bias), then subtracts the measured hang height.
/// Keeping that second operation after collision preserves walls/ceilings and
/// makes the ZIPLINE packet describe the rendered position.
pub fn update_world_zipline_traversals(
    mut commands: Commands,
    time: Res<Time>,
    input: Res<LegacyInputState>,
    mut traversals: Query<(
        Entity,
        &mut Transform,
        &mut crate::movement::LegacyPlayerController,
        &mut WorldZiplineTraversal,
    )>,
    rigs: Query<(&TutorialSelectedPlayerRig, &NativePlayerRigBones)>,
    bone_transforms: Query<&GlobalTransform>,
) {
    let delta = time.delta_secs().max(0.0);
    for (entity, mut transform, mut controller, mut traversal) in &mut traversals {
        if let Some(height) = rigs.iter().find_map(|(rig, bones)| {
            (rig.controller_root == entity).then(|| {
                let hand = bones.unique_by_true_name("Bip01 L Hand")?;
                let toe = bones.unique_by_true_name("Bip01 L Toe0")?;
                let height = bone_transforms.get(hand).ok()?.translation().y
                    - bone_transforms.get(toe).ok()?.translation().y;
                height.is_finite().then_some(height)
            })?
        }) {
            traversal.hang_height = height;
        }
        let delta_position = traversal.end - traversal.start;
        let maximum_distance = delta_position.length();
        let direction = delta_position.normalize_or_zero();
        traversal.travelled += traversal.speed * delta;
        traversal.packet_elapsed += delta;
        let cable_position = traversal.start + direction * traversal.travelled
            - Vec3::splat(crate::movement::LEGACY_CHARACTER_MOVE_BIAS);
        controller.submit_scripted_move(cable_position - transform.translation, delta);
        transform.translation = cable_position;
        let horizontal = Vec3::new(direction.x, 0.0, direction.z).normalize_or_zero();
        if horizontal.length_squared() > f32::EPSILON {
            let unity_direction = unity_to_native_vector(horizontal);
            controller.yaw_degrees = unity_direction.x.atan2(unity_direction.z).to_degrees();
            let target =
                LegacyUnityHeadingDegrees::new(controller.yaw_degrees).native_root_rotation();
            transform.rotation = transform
                .rotation
                .slerp(target, (delta * 4.0).clamp(0.0, 1.0));
        }
        // Source steps the cable before checking jump and clamps only the packet
        // distance at its endpoint. No horizontal launch impulse on detachment.
        let finished = traversal.travelled > maximum_distance || input.jump_just_pressed;
        commands.entity(entity).insert(WorldZiplineStep {
            detached: input.jump_just_pressed,
            finished,
        });
    }
}

/// Complete EpUpdate after the actual CCT result, before avatar presentation.
pub fn finish_world_zipline_steps(
    mut commands: Commands,
    mut traversals: Query<(Entity, &mut Transform, &mut crate::movement::LegacyPlayerController,
        &mut WorldZiplineTraversal, &WorldZiplineStep)>,
    mut gameplay: ResMut<WorldGameplayIntentQueue>,
) {
    for (entity, mut transform, mut controller, mut traversal, step) in &mut traversals {
        transform.translation.y -= traversal.hang_height;
        // Cable contact never releases its movement owner; ordinary floor
        // grounding must not show weapons or replace ropedown mid-ride.
        controller.set_grounded(false);
        controller.velocity = (traversal.end - traversal.start).normalize_or_zero() * traversal.speed;
        let finished = step.finished;
        traversal.travelled = traversal.travelled.min(traversal.start.distance(traversal.end));
        if finished || traversal.packet_elapsed >= crate::movement::LEGACY_PACKET_SEND_INTERVAL {
            let request = world_zipline_request(&traversal, transform.translation,
                controller.yaw_degrees, step.detached);
            let _ = gameplay.push(packet::P_CL2FE_REQ_PC_ZIPLINE, &request);
            traversal.packet_elapsed = 0.0;
        }
        if finished {
            controller.finish_scripted_traversal_with_jump();
            commands.entity(entity).remove::<WorldZiplineTraversal>();
        }
        commands.entity(entity).remove::<WorldZiplineStep>();
    }
}

pub(super) fn world_zipline_request(
    traversal: &WorldZiplineTraversal, position: Vec3, yaw: f32, detached: bool,
) -> PcZiplineRequest0104 {
    PcZiplineRequest0104 {
        client_time: 0,
        start_position: ProtocolPosition::from_native(traversal.start).raw(),
        moved_distance: traversal.travelled,
        maximum_distance: traversal.start.distance(traversal.end),
        dummy: 0.0,
        position: ProtocolPosition::from_native(position).raw(),
        velocity: ProtocolMoveVelocity::from_native((traversal.end - traversal.start).normalize_or_zero()).raw(),
        down: i32::from(detached), roll_max: 30, roll: 0,
        angle: LegacyUnityHeadingDegrees::new(yaw).to_protocol().degrees(),
        speed: (traversal.speed * 100.0) as i32,
    }
}

pub fn update_world_rope_traversals(
    mut commands: Commands,
    time: Res<Time>,
    input: Res<LegacyInputState>,
    mut traversals: Query<(
        Entity,
        &mut Transform,
        &mut crate::movement::LegacyPlayerController,
        &mut WorldRopeTraversal,
    )>,
) {
    for (entity, mut transform, mut controller, mut traversal) in &mut traversals {
        let axis = match traversal.move_type {
            2 => input.local_axis.y,
            0 => -input.local_axis.x,
            _ => input.local_axis.x,
        };
        traversal.distance = (traversal.distance + axis * traversal.speed * time.delta_secs())
            .clamp(0.0, traversal.total_length);
        let Some((path_position, direction)) =
            sample_polyline(&traversal.points, traversal.distance)
        else {
            controller.finish_scripted_traversal(Vec3::ZERO);
            commands.entity(entity).remove::<WorldRopeTraversal>();
            continue;
        };
        // Clean horizontal ropes place the controller one capsule-height
        // below the authored hand path. The serialized controller is 2 units.
        transform.translation = if traversal.move_type == 2 {
            path_position
        } else {
            path_position - Vec3::Y * 2.2
        };
        controller.velocity = direction * axis * traversal.speed;
        if direction.length_squared() > f32::EPSILON && traversal.move_type != 2 {
            let side = Vec3::Y.cross(direction).normalize_or_zero();
            let look = if traversal.move_type == 0 {
                -side
            } else {
                side
            };
            if look.length_squared() > f32::EPSILON {
                transform.look_to(look, Vec3::Y);
            }
        }
        if input.jump_just_pressed {
            let jump = if input.local_axis.y < 0.0 {
                Vec3::ZERO
            } else {
                Vec3::Y
                    * controller.jump_height_server_units as f32
                    * crate::movement::SERVER_TO_CLIENT_SCALE
            };
            controller.finish_scripted_traversal(jump);
            commands.entity(entity).remove::<WorldRopeTraversal>();
        }
    }
}

pub fn update_world_slope_traversals(
    mut commands: Commands,
    time: Res<Time>,
    input: Res<LegacyInputState>,
    mut traversals: Query<(
        Entity,
        &mut Transform,
        &mut crate::movement::LegacyPlayerController,
        &mut WorldSlopeTraversal,
        Option<&crate::world::NativeWorldGroundSupport>,
    )>,
    mut gameplay: ResMut<WorldGameplayIntentQueue>,
) {
    const SLOPE_INPUT_FACTOR: f32 = 0.3;
    const SLOPE_SURFACE_TILT_FACTOR: f32 = 0.135;
    const SLOPE_DRIFT_COEFFICIENT: f32 = 0.065;

    for (entity, mut transform, mut controller, mut traversal, support) in &mut traversals {
        if traversal.segment + 1 >= traversal.points.len() {
            let exit_velocity = controller.velocity;
            controller.finish_scripted_traversal(exit_velocity);
            commands.entity(entity).remove::<WorldSlopeTraversal>();
            continue;
        }
        let delta = time.delta_secs().max(0.0);
        let previous = traversal.points[traversal.segment];
        let target = traversal.points[traversal.segment + 1];
        let path_direction = (target - previous).normalize_or_zero();
        if transform.translation.distance(target) < 0.3 * traversal.speed {
            traversal.segment += 1;
        }

        // cnAvatarThirdPersonMove.EpSlope constants and update order. Native
        // collision retains the exact supporting triangle normal, matching
        // clean `ControllerColliderHit.normal` after the first contact frame.
        let damping = (1.0 - SLOPE_DRIFT_COEFFICIENT * delta * 30.0).max(0.0);
        traversal.drift *= damping;
        if controller.grounded || support.is_some() {
            traversal.drift += path_direction * SLOPE_DRIFT_COEFFICIENT * delta * 30.0;
        }
        let normal_up_dot = support
            .map(|support| support.surface_normal.dot(Vec3::Y).clamp(-1.0, 1.0))
            // The first frame can precede the native contact sample. Preserve
            // a deterministic authored-path estimate only for that frame.
            .unwrap_or_else(|| {
                Vec2::new(path_direction.x, path_direction.z)
                    .length()
                    .clamp(0.0, 1.0)
            });
        let surface_speed = (2.0 - normal_up_dot) * traversal.speed * SLOPE_SURFACE_TILT_FACTOR;

        if path_direction.length_squared() > f32::EPSILON {
            let unity_direction = unity_to_native_vector(path_direction);
            let path_yaw = unity_direction.x.atan2(unity_direction.z).to_degrees();
            let target_yaw = path_yaw + input.local_axis.x * 35.0;
            controller.yaw_degrees = lerp_degrees(controller.yaw_degrees, target_yaw, 1.8 * delta);
            transform.rotation =
                LegacyUnityHeadingDegrees::new(controller.yaw_degrees).native_root_rotation();
        }

        let forward = traversal.drift
            * surface_speed
            * (2.0 + input.local_axis.y)
            * traversal.speed
            * SLOPE_INPUT_FACTOR;
        let lateral = transform.right()
            * input.local_axis.x
            * SLOPE_INPUT_FACTOR
            * crate::movement::LEGACY_GRAVITY;
        controller.velocity = forward + lateral - Vec3::Y * crate::movement::LEGACY_GRAVITY;
        transform.translation += controller.velocity * delta;

        traversal.packet_elapsed += delta;
        if traversal.packet_elapsed >= crate::movement::LEGACY_PACKET_SEND_INTERVAL {
            let request = PcSlopeRequest0104 {
                client_time: 0,
                position: ProtocolPosition::from_native(transform.translation).raw(),
                angle: LegacyUnityHeadingDegrees::new(controller.yaw_degrees)
                    .to_protocol()
                    .degrees(),
                speed: (surface_speed * 100.0) as i32,
                key_value: controller.current_direction_key(),
                velocity: ProtocolMoveVelocity::from_native(traversal.drift).raw(),
                slope_id: traversal.slope_id,
            };
            let _ = gameplay.push(packet::P_CL2FE_REQ_PC_SLOPE, &request);
            traversal.packet_elapsed = 0.0;
        }
        if input.jump_just_pressed {
            let jump_speed = controller.jump_height_server_units as f32
                * crate::movement::SERVER_TO_CLIENT_SCALE;
            let mut exit_velocity = forward + lateral;
            exit_velocity.y = jump_speed;
            controller.finish_scripted_traversal(exit_velocity);
            commands.entity(entity).remove::<WorldSlopeTraversal>();
        }
    }
}

/// Apply `EnvironmentCollision.beltMoveDelta` for the two authored conveyor
/// paths. The nearest-segment rule and `speed * deltaTime` displacement are
/// the clean implementation; the vertical gate prevents a belt on another
/// floor from claiming the player.
pub fn update_world_belts(
    time: Res<Time>,
    belts: Query<(&WorldTrigger, &GlobalTransform)>,
    mut player: Query<(&mut Transform, &mut crate::movement::LegacyPlayerController)>,
) {
    let Ok((mut transform, mut controller)) = player.single_mut() else {
        return;
    };
    if !controller.grounded {
        return;
    }
    let mut best: Option<(f32, Vec3, f32)> = None;
    for (trigger, global) in &belts {
        if trigger.kind != WorldTriggerKind::Belt || trigger.path_points.len() < 2 {
            continue;
        }
        let points = trigger
            .path_points
            .iter()
            .map(|point| global.transform_point(*point))
            .collect::<Vec<_>>();
        let distance = nearest_polyline_distance(&points, transform.translation);
        let Some((surface, direction)) = sample_polyline(&points, distance) else {
            continue;
        };
        let horizontal = Vec2::new(
            surface.x - transform.translation.x,
            surface.z - transform.translation.z,
        )
        .length();
        let vertical = (surface.y - transform.translation.y).abs();
        if horizontal <= trigger.radius.max(1.0) + 0.75
            && vertical <= 1.0
            && best.is_none_or(|(nearest, _, _)| horizontal < nearest)
        {
            best = Some((horizontal, direction, trigger.speed.max(0.0)));
        }
    }
    if let Some((_, direction, speed)) = best {
        let belt_velocity = direction * speed;
        transform.translation += belt_velocity * time.delta_secs().max(0.0);
        controller.velocity += belt_velocity;
    }
}

pub fn sync_world_launcher_input(
    input: Res<LegacyInputState>,
    mut external: ResMut<LauncherUiExternalState>,
) {
    external.aim_vertical_axis = input.local_axis.y;
    external.aim_horizontal_axis = input.local_axis.x;
}

/// Keep the gameplay camera on the authored launcher pivot while the existing
/// launcher model owns aim. This runs after the ordinary orbit pose.
pub fn apply_world_launcher_camera(
    launcher: Res<LauncherUiModel>,
    active: Res<ActiveWorldLauncher>,
    mut cameras: Query<&mut Transform, With<LegacyOrbitCamera>>,
) {
    if !active.custom_camera || !launcher.visible() {
        return;
    }
    let Some(position) = active.camera_position else {
        return;
    };
    let Ok(mut camera) = cameras.single_mut() else {
        return;
    };
    let yaw = launcher.start_rotation_degrees.y + launcher.current_rotation_degrees.y + 180.0;
    let forward = launcher_forward(launcher.current_rotation_degrees.x, yaw);
    camera.translation = position;
    camera.look_to(forward, Vec3::Y);
}
