use super::*;

impl TutorialEffectRuntime {


    pub(super) fn process_one(&mut self, command: TutorialEffectRuntimeCommand, streamed_world: bool) {
        if self.queue_native_skill_projectile(&command) { return; }
        let native_request = match &command {
            TutorialEffectRuntimeCommand::Preload { effect_id, .. } | TutorialEffectRuntimeCommand::Add { effect_id, .. } => self.native_catalog.contains_key(effect_id),
            _ => false,
        };
        if self.library.is_none() && !native_request {
            if let TutorialEffectRuntimeCommand::Preload { effect_id, .. } = &command {
                // `PreloadEffect` returns null when the catalog cannot serve
                // the request; Retrobution treats that as non-blocking.
                self.mark_native_preload_complete(*effect_id);
            }
            self.issues
                .push_back(TutorialEffectRuntimeIssue::ExactCatalogUnavailable {
                    source_line: command.source_line(),
                });
            self.record(
                command,
                TutorialEffectRuntimeDisposition::RejectedFailClosed,
            );
            return;
        }
        match command.clone() {
            TutorialEffectRuntimeCommand::Preload {
                effect_id,
                source_line,
            } => {
                let Some(compiled) = self.compile_available_effect(effect_id)
                else {
                    self.mark_native_preload_complete(effect_id);
                    self.issues
                        .push_back(TutorialEffectRuntimeIssue::EffectOutsideExactCatalog {
                            effect_id,
                            source_line,
                        });
                    self.record(
                        command,
                        TutorialEffectRuntimeDisposition::RejectedFailClosed,
                    );
                    return;
                };
                if self.preloaded.insert(effect_id) {

                    self.push_blockers(effect_id, source_line, &compiled.blockers);
                    if let Some(plan) = compiled.plan {
                        self.native_preloads.push_back(
                            tutorial_native_effects::NativePreloadRequest { effect_id, plan },
                        );
                    } else {
                        // A failed native compile is terminal just like a
                        // completed-but-unsuccessful AssetBundleRequest.
                        self.mark_native_preload_complete(effect_id);
                    }
                }
                self.record(
                    command,
                    TutorialEffectRuntimeDisposition::SerializedClosurePreloaded,
                );
            }
            TutorialEffectRuntimeCommand::Add {
                effect_id,
                placement,
                scale,
                tracked,
                name,
                destroy_after_seconds,
                source_line,
            } => {
                let Some(compiled) = self.compile_available_effect(effect_id)
                else {
                    self.issues
                        .push_back(TutorialEffectRuntimeIssue::EffectOutsideExactCatalog {
                            effect_id,
                            source_line,
                        });
                    self.record(
                        command,
                        TutorialEffectRuntimeDisposition::RejectedFailClosed,
                    );
                    return;
                };
                if !placement.is_valid()
                    || !scale.is_finite()
                    || scale <= 0.0
                    || destroy_after_seconds
                        .is_some_and(|seconds| !seconds.is_finite() || seconds < 0.0)
                {
                    self.issues
                        .push_back(TutorialEffectRuntimeIssue::InvalidEffectTransform {
                            effect_id,
                            source_line,
                        });
                    self.record(
                        command,
                        TutorialEffectRuntimeDisposition::RejectedFailClosed,
                    );
                    return;
                }

                self.push_blockers(effect_id, source_line, &compiled.blockers);
                let Some(plan) = compiled.plan else {
                    self.record(
                        command,
                        TutorialEffectRuntimeDisposition::RejectedFailClosed,
                    );
                    return;
                };
                if self.preloaded.insert(effect_id) {
                    self.native_preloads
                        .push_back(tutorial_native_effects::NativePreloadRequest {
                            effect_id,
                            plan: plan.clone(),
                        });
                }
                let rendered_nodes = plan.rendered_nodes;
                let blocked_nodes = compiled.blockers.len();
                let stream_owner = placement.stream_owner();
                let instance_id = self.allocate_instance(tracked, name.clone());
                self.active
                    .get_mut(&instance_id)
                    .expect("new native effect instance is active")
                    .stream_owner = stream_owner;
                self.native_spawns
                    .push_back(tutorial_native_effects::NativeSpawnRequest::Effect {
                        instance_id,
                        effect_id,
                        streamed_world,
                        placement,
                        scale,
                        name,
                        destroy_after_seconds,
                        source_line,
                        plan,
                    });
                self.record(
                    command,
                    TutorialEffectRuntimeDisposition::NativeEffectQueued {
                        instance_id,
                        rendered_nodes,
                        blocked_nodes,
                    },
                );
            }
            TutorialEffectRuntimeCommand::ClearTracked { .. } => {
                let ids = std::mem::take(&mut self.tracked)
                    .into_iter()
                    .collect::<Vec<_>>();
                let count = ids.len();
                for id in ids {
                    self.forget_instance(id);
                    self.native_despawns.push_back(id);
                }
                self.record(
                    command,
                    TutorialEffectRuntimeDisposition::ClearedTrackedInstances { count },
                );
            }
            TutorialEffectRuntimeCommand::DestroyNamed { name, source_line } => {
                if let Some(instance_id) = self.named.remove(&name) {
                    self.forget_instance(instance_id);
                    self.native_despawns.push_back(instance_id);
                    self.record(
                        command,
                        TutorialEffectRuntimeDisposition::DestroyedNamedInstance { instance_id },
                    );
                } else {
                    self.issues.push_back(
                        TutorialEffectRuntimeIssue::NamedEffectWasNotInstantiated {
                            name,
                            source_line,
                        },
                    );
                    self.record(
                        command,
                        TutorialEffectRuntimeDisposition::RejectedFailClosed,
                    );
                }
            }
            TutorialEffectRuntimeCommand::Projectile {
                bullet_type,
                source,
                target,
                target_exists,
                source_style,
                target_style,
                motion,
                source_line,
            } => {
                if !source.is_finite() || !target.is_finite() || !motion.is_valid() {
                    self.issues
                        .push_back(TutorialEffectRuntimeIssue::InvalidProjectileEndpoints {
                            source_line,
                        });
                    self.record(
                        command,
                        TutorialEffectRuntimeDisposition::RejectedFailClosed,
                    );
                    return;
                }
                let inputs = self.library.as_ref().and_then(|library| {
                    let row = library.bullets.get(&bullet_type)?.clone();
                    let effect = |effect_id: i32| {
                        library
                            .projectile_effects
                            .get(&effect_id)
                            .or_else(|| library.effects.get(&effect_id))
                            .cloned()
                    };
                    let particle = (row.parameters.particle_script > 0)
                        .then(|| effect(row.parameters.particle_script))
                        .flatten();
                    let fire = (matches!(motion, TutorialProjectileMotion::BulletMove)
                        && row.parameters.fire_script > 0)
                        .then(|| effect(row.parameters.fire_script))
                        .flatten();
                    let success_script = exact_projectile_success_script(
                        target_exists || matches!(motion, TutorialProjectileMotion::Warhead { .. }),
                        row.parameters.success_script,
                    );
                    let impact = success_script.and_then(effect);
                    if matches!(motion, TutorialProjectileMotion::BulletMove)
                        && row.parameters.fire_script > 0
                        && fire.is_none()
                        || success_script.is_some() && impact.is_none()
                    {
                        return None;
                    }
                    Some((row, particle, fire, impact))
                });
                let Some((row, particle, fire, impact)) = inputs else {
                    self.issues.push_back(
                        TutorialEffectRuntimeIssue::ProjectileOutsideExactContract {
                            bullet_type,
                            source_line,
                        },
                    );
                    self.record(
                        command,
                        TutorialEffectRuntimeDisposition::RejectedFailClosed,
                    );
                    return;
                };

                let projectile = particle.as_ref().map(|particle| {
                    tutorial_native_effects::compile_projectile_plan(
                        particle.entry.effect_id,
                        particle,
                    )
                });
                let exact_invisible_particle = particle.as_ref().is_some_and(|particle| {
                    exact_invisible_projectile_carrier(particle.entry.effect_id)
                });
                // Weapon effects such as Rifle ES[808] compile through the
                // exact ParticleTrailController path below. Mob ES[17] instead
                // has an ordinary emitter and therefore no trail plan.
                // BulletMoveScript still parents that representation to the
                // moving carrier, so retain its ordinary effect plan there
                // instead of spawning it at the attack socket.
                let projectile_effect = match (&particle, &projectile) {
                    (Some(particle), Some(projectile))
                        if projectile.plan.is_none() && !exact_invisible_particle =>
                    {
                        Some((
                            particle.clone(),
                            tutorial_native_effects::compile_effect_plan(
                                particle.entry.effect_id,
                                particle,
                            ),
                        ))
                    }
                    _ => None,
                };
                if let (Some(particle), Some(projectile)) = (&particle, &projectile) {
                    if projectile_effect.is_none() && !exact_invisible_particle {
                        self.push_blockers(
                            particle.entry.effect_id,
                            source_line,
                            &projectile.blockers,
                        );
                    }
                }
                if let Some((effect, compiled)) = &projectile_effect {
                    self.push_blockers(effect.entry.effect_id, source_line, &compiled.blockers);
                }
                let fire = fire.map(|effect| {
                    let compiled = tutorial_native_effects::compile_effect_plan(
                        effect.entry.effect_id,
                        &effect,
                    );
                    (effect, compiled)
                });
                if let Some((effect, compiled)) = &fire {
                    self.push_blockers(effect.entry.effect_id, source_line, &compiled.blockers);
                }
                let impact = impact.map(|effect| {
                    let compiled = tutorial_native_effects::compile_effect_plan(
                        effect.entry.effect_id,
                        &effect,
                    );
                    (effect, compiled)
                });
                if let Some((effect, compiled)) = &impact {
                    self.push_blockers(effect.entry.effect_id, source_line, &compiled.blockers);
                }
                let projectile_plan = projectile
                    .as_ref()
                    .and_then(|compiled| compiled.plan.clone());
                if particle.is_some()
                    && projectile_plan.is_none()
                    && projectile_effect
                        .as_ref()
                        .is_none_or(|(_, compiled)| compiled.plan.is_none())
                    && !exact_invisible_particle
                {
                    self.record(
                        command,
                        TutorialEffectRuntimeDisposition::RejectedFailClosed,
                    );
                    return;
                }
                if fire
                    .as_ref()
                    .is_some_and(|(_, compiled)| compiled.plan.is_none())
                    || impact
                        .as_ref()
                        .is_some_and(|(_, compiled)| compiled.plan.is_none())
                {
                    self.record(
                        command,
                        TutorialEffectRuntimeDisposition::RejectedFailClosed,
                    );
                    return;
                }

                let direction = (target - source).normalize_or_zero();
                let rotation = if direction == Vec3::ZERO {
                    Quat::IDENTITY
                } else {
                    Quat::from_rotation_arc(Vec3::Z, direction)
                };
                let mut rendered_nodes = projectile_plan
                    .as_ref()
                    .map_or(0, |plan| plan.rendered_nodes);
                let mut blocked_nodes = if projectile_effect.is_some() || exact_invisible_particle {
                    0
                } else {
                    projectile
                        .as_ref()
                        .map_or(0, |compiled| compiled.blockers.len())
                };
                let carried_effect_plan = projectile_effect.map(|(_effect, compiled)| {
                    rendered_nodes += compiled.plan.as_ref().unwrap().rendered_nodes;
                    blocked_nodes += compiled.blockers.len();
                    compiled.plan.unwrap()
                });
                if let Some((effect, compiled)) = fire {
                    rendered_nodes += compiled.plan.as_ref().unwrap().rendered_nodes;
                    blocked_nodes += compiled.blockers.len();
                    let instance_id = self.allocate_instance(false, None);
                    self.native_spawns.push_back(
                        tutorial_native_effects::NativeSpawnRequest::Effect {
                            instance_id,
                            effect_id: effect.entry.effect_id,
                            streamed_world: false,
                            placement: TutorialEffectPlacement::World {
                                position: source,
                                rotation,
                            },
                            scale: row.parameters.fire_model_scale as f32,
                            name: None,
                            destroy_after_seconds: None,
                            source_line,
                            plan: compiled.plan.unwrap(),
                        },
                    );
                }
                let impact = impact.map(|(effect, compiled)| {
                    rendered_nodes += compiled.plan.as_ref().unwrap().rendered_nodes;
                    blocked_nodes += compiled.blockers.len();
                    tutorial_native_effects::NativeLinearImpactPlan {
                        instance_id: self.allocate_instance(false, None),
                        effect_id: effect.entry.effect_id,
                        scale: match motion {
                            TutorialProjectileMotion::BulletMove => exact_projectile_success_scale(
                                row.parameters.success_model_scale as f32,
                                source_style,
                                target_style,
                            ),
                            TutorialProjectileMotion::Warhead { .. } => {
                                row.parameters.success_model_scale as f32
                            }
                        },
                        sound_path: exact_tutorial_success_sound_path(
                            &row.parameters.success_sound,
                        )
                        .map(str::to_owned),
                        plan: compiled.plan.unwrap(),
                    }
                });
                let instance_id = self.allocate_instance(false, None);
                self.native_spawns.push_back(
                    tutorial_native_effects::NativeSpawnRequest::LinearProjectile {
                        instance_id,
                        bullet_type,
                        effect_id: particle
                            .as_ref()
                            .map_or(0, |particle| particle.entry.effect_id),
                        source,
                        target,
                        scale: row.parameters.bullet_model_scale as f32,
                        motion: match motion {
                            TutorialProjectileMotion::BulletMove => {
                                tutorial_native_effects::NativeLinearProjectileMotion::BulletMove {
                                    hide_seconds: row.parameters.hide_time_seconds as f32,
                                    duration_seconds: row.parameters.maximum_time_seconds as f32,
                                }
                            }
                            TutorialProjectileMotion::Warhead {
                                speed,
                                initial_vertical_speed,
                                duration_seconds,
                                authority,
                            } => tutorial_native_effects::NativeLinearProjectileMotion::Warhead {
                                speed,
                                initial_vertical_speed,
                                duration_seconds,
                                authority,
                            },
                        },
                        impact,
                        plan: projectile_plan,
                        carried_effect: carried_effect_plan,
                    },
                );
                self.record(
                    command,
                    TutorialEffectRuntimeDisposition::NativeProjectileQueued {
                        instance_id,
                        rendered_nodes,
                        blocked_nodes,
                    },
                );
            }
            TutorialEffectRuntimeCommand::ProjectilePair {
                types,
                source,
                target,
                oni,
                priority,
                source_line,
            } => {
                if exact_projectile_inputs(self.library.as_ref().unwrap(), types, oni, priority)
                    .is_none()
                {
                    self.issues.push_back(
                        TutorialEffectRuntimeIssue::ProjectilePairOutsideExactContract {
                            types,
                            source_line,
                        },
                    );
                    self.record(
                        command,
                        TutorialEffectRuntimeDisposition::RejectedFailClosed,
                    );
                    return;
                }
                if !source.is_finite() || !target.is_finite() {
                    self.issues
                        .push_back(TutorialEffectRuntimeIssue::InvalidProjectileEndpoints {
                            source_line,
                        });
                    self.record(
                        command,
                        TutorialEffectRuntimeDisposition::RejectedFailClosed,
                    );
                    return;
                }
                self.issues.push_back(
                    TutorialEffectRuntimeIssue::ProjectileRuntimeSamplesRequired {
                        types,
                        reverse: None,
                        required_draw_count_normal: 6,
                        required_draw_count_reverse: 4,
                        source_line,
                    },
                );
                self.record(
                    command,
                    TutorialEffectRuntimeDisposition::RejectedFailClosed,
                );
            }
            TutorialEffectRuntimeCommand::ProjectilePairSampled {
                types,
                source,
                target,
                oni,
                priority,
                reverse,
                sampled_initial_velocity,
                source_line,
            } => {
                let Some(inputs) =
                    exact_projectile_inputs(self.library.as_ref().unwrap(), types, oni, priority)
                else {
                    self.issues.push_back(
                        TutorialEffectRuntimeIssue::ProjectilePairOutsideExactContract {
                            types,
                            source_line,
                        },
                    );
                    self.record(
                        command,
                        TutorialEffectRuntimeDisposition::RejectedFailClosed,
                    );
                    return;
                };
                if !source.is_finite() || !target.is_finite() {
                    self.issues
                        .push_back(TutorialEffectRuntimeIssue::InvalidProjectileEndpoints {
                            source_line,
                        });
                    self.record(
                        command,
                        TutorialEffectRuntimeDisposition::RejectedFailClosed,
                    );
                    return;
                }
                if let Some((index, sample)) = sampled_initial_velocity
                    .iter()
                    .copied()
                    .enumerate()
                    .find(|(_, sample)| !valid_oni_velocity_sample(*sample, reverse))
                {
                    self.issues.push_back(
                        TutorialEffectRuntimeIssue::InvalidProjectileVelocitySample {
                            projectile_index: index,
                            sample,
                            reverse,
                            source_line,
                        },
                    );
                    self.record(
                        command,
                        TutorialEffectRuntimeDisposition::RejectedFailClosed,
                    );
                    return;
                }

                let first = tutorial_native_effects::compile_projectile_plan(
                    inputs[0].0.entry.effect_id,
                    &inputs[0].0,
                );
                let second = tutorial_native_effects::compile_projectile_plan(
                    inputs[1].0.entry.effect_id,
                    &inputs[1].0,
                );
                self.push_blockers(inputs[0].0.entry.effect_id, source_line, &first.blockers);
                self.push_blockers(inputs[1].0.entry.effect_id, source_line, &second.blockers);
                let (Some(first_plan), Some(second_plan)) = (first.plan, second.plan) else {
                    self.record(
                        command,
                        TutorialEffectRuntimeDisposition::RejectedFailClosed,
                    );
                    return;
                };
                let rendered_nodes = first_plan.rendered_nodes + second_plan.rendered_nodes;
                let blocked_nodes = first.blockers.len() + second.blockers.len();
                let ids = [
                    self.allocate_instance(false, None),
                    self.allocate_instance(false, None),
                ];
                for (index, (plan, scale)) in
                    [(first_plan, inputs[0].1), (second_plan, inputs[1].1)]
                        .into_iter()
                        .enumerate()
                {
                    self.native_spawns.push_back(
                        tutorial_native_effects::NativeSpawnRequest::Projectile {
                            instance_id: ids[index],
                            effect_id: inputs[index].0.entry.effect_id,
                            source,
                            target,
                            scale,
                            reverse,
                            sampled_initial_velocity: sampled_initial_velocity[index],
                            plan,
                        },
                    );
                }
                self.record(
                    command,
                    TutorialEffectRuntimeDisposition::NativeProjectilePairQueued {
                        instance_ids: ids,
                        rendered_nodes,
                        blocked_nodes,
                    },
                );
            }
        }
    }

    pub(super) fn record(
        &mut self,
        command: TutorialEffectRuntimeCommand,
        disposition: TutorialEffectRuntimeDisposition,
    ) {
        self.records.push_back(TutorialEffectRuntimeRecord {
            command,
            disposition,
        });
    }

}

pub struct TutorialEffectsRuntimePlugin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum TutorialEffectRuntimeSet {
    ActorAnimationEvents,
}

impl Plugin for TutorialEffectsRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TutorialEffectRuntime>()
            .init_resource::<TutorialActorEventQueue>()
            .add_systems(
                Update,
                emit_retrobution_actor_animation_events
                    .in_set(TutorialEffectRuntimeSet::ActorAnimationEvents)
                    .after(apply_tutorial_actor_animation_playback)
                    .before(process_tutorial_effect_runtime),
            )
            .add_systems(Update, tutorial_native_effects::native_catalog::load.before(process_tutorial_effect_runtime))
            .add_systems(Update, process_tutorial_effect_runtime);
        tutorial_native_effects::install(app);
    }
}

pub fn process_tutorial_effect_runtime(
    mut runtime: ResMut<TutorialEffectRuntime>,
    live_stream_owners: Query<Entity, Without<crate::world::PendingNativeWorldSceneUnload>>,
) {
    runtime.process_pending_with_stream_liveness(|owner| live_stream_owners.get(owner).is_ok());
}

pub fn cleanup_tutorial_effect_runtime(mut runtime: ResMut<TutorialEffectRuntime>) {
    runtime.clear_scene_instances();
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TutorialOniProjectileState {
    pub position: Vec3,
    pub velocity: Vec3,
    pub move_speed: f32,
    pub age_seconds: f32,
    pub stopped_seconds: f32,
    pub reverse: bool,
    pub phase: TutorialOniMotionPhase,
}

impl TutorialOniProjectileState {
    pub fn new(position: Vec3, sampled_initial_velocity: Vec3, reverse: bool) -> Option<Self> {
        if !position.is_finite() || !sampled_initial_velocity.is_finite() {
            return None;
        }
        Some(Self {
            position,
            velocity: sampled_initial_velocity,
            move_speed: RETROBUTION_ONI_MOTION.initial_move_speed,
            age_seconds: 0.0,
            stopped_seconds: 0.0,
            reverse,
            phase: TutorialOniMotionPhase::Flying,
        })
    }

    /// Deterministic single-frame transcription of `OniMoveScript.Update`.
    pub fn step(&mut self, delta_seconds: f32, target: Vec3) {
        if !delta_seconds.is_finite()
            || delta_seconds < 0.0
            || !target.is_finite()
            || self.phase == TutorialOniMotionPhase::Destroyed
        {
            return;
        }
        if self.phase == TutorialOniMotionPhase::EmitterStopped {
            self.stopped_seconds += delta_seconds;
            if self.stopped_seconds > RETROBUTION_ONI_MOTION.destroy_delay_seconds {
                self.phase = TutorialOniMotionPhase::Destroyed;
            }
            return;
        }

        let mut delta = target - self.position;
        let distance = delta.length();
        if distance
            <= delta_seconds
                * (self.move_speed + delta_seconds * RETROBUTION_ONI_MOTION.acceleration_per_second)
            || distance < RETROBUTION_ONI_MOTION.arrival_distance
        {
            self.position = target;
            self.stop_emitter();
            return;
        }
        if distance > RETROBUTION_ONI_MOTION.maximum_distance
            || self.age_seconds > RETROBUTION_ONI_MOTION.maximum_flight_seconds
        {
            self.stop_emitter();
            return;
        }

        if self.reverse {
            self.velocity *= 1.0 - delta_seconds * RETROBUTION_ONI_MOTION.damping_per_second;
            self.velocity += delta * delta_seconds * RETROBUTION_ONI_MOTION.reverse_force;
            self.position += self.velocity * delta_seconds;
        } else {
            delta.y = 0.0;
            self.move_speed += delta_seconds * RETROBUTION_ONI_MOTION.acceleration_per_second;
            self.velocity.x *= 1.0 - delta_seconds * RETROBUTION_ONI_MOTION.damping_per_second;
            self.velocity.z *= 1.0 - delta_seconds * RETROBUTION_ONI_MOTION.damping_per_second;
            self.velocity += delta * delta_seconds * RETROBUTION_ONI_MOTION.normal_force;
            self.velocity.y += RETROBUTION_ONI_MOTION.gravity_per_second * delta_seconds;
            let vertical = self.velocity.y;
            let mut planar = Vec3::new(self.velocity.x, 0.0, self.velocity.z);
            if planar.length_squared() > 0.0 {
                planar = planar.normalize() * self.move_speed;
            }
            planar.y = vertical;
            self.position += planar * delta_seconds;
            if self.position.y < target.y {
                self.position.y = target.y;
                self.velocity.y *= RETROBUTION_ONI_MOTION.bounce_multiplier;
            }
        }
        self.age_seconds += delta_seconds;
    }

    pub(super) fn stop_emitter(&mut self) {
        self.phase = TutorialOniMotionPhase::EmitterStopped;
        self.stopped_seconds = 0.0;
    }
}
