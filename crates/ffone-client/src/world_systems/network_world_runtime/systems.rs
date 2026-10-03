use super::*;

pub fn advance_network_npc_motion_0104(
    mut commands: Commands,
    time: Option<Res<Time>>,
    hnpcs: Query<(), With<NetworkHnpcVisual0104>>,
    mut npcs: Query<
        (
            Entity,
            &mut Transform,
            Ref<NetworkNpcMotion0104>,
            Option<&NetworkNpcAnimationLayers0104>,
            Option<&NetworkNpcMotionSettled0104>,
        ),
        With<NetworkNpc0104>,
    >,
) {
    let Some(time) = time else {
        return;
    };
    let delta_seconds = time.delta_secs().max(0.0);
    for (entity, mut transform, motion, layers, settled) in &mut npcs {
        if layers.is_some_and(|layers| layers.stand_attack) && !hnpcs.contains(entity) {
            continue;
        }
        // Primary NpcMoveController stores all three packet coordinates, but
        // zeroes the target delta's Y before ordinary locomotion. Its ground
        // ray and gravity own the root height independently.
        let horizontal_destination = Vec3::new(
            motion.destination.x,
            transform.translation.y,
            motion.destination.z,
        );
        if advance_server_entity_motion(
            &mut transform,
            horizontal_destination,
            motion.speed,
            delta_seconds,
            true,
        ) {
            if motion.is_changed() || settled.is_none() {
                commands.entity(entity).insert(NetworkNpcMotionSettled0104 {
                    since: time.elapsed_secs_f64(),
                });
                continue;
            }
            if time.elapsed_secs_f64() - settled.unwrap().since < 0.15 {
                continue;
            }
            let mut entity = commands.entity(entity);
            entity.remove::<NetworkNpcMotion0104>();
            entity.remove::<NetworkNpcMotionSettled0104>();
            if motion.move_style == 0 {
                entity.remove::<NetworkNpcReadyAnimation0104>();
            } else {
                // `NpcMoveController.EndAnimation`: walking settles to stand,
                // while combat running settles into AttackReady.
                entity.insert(NetworkNpcReadyAnimation0104);
            }
        } else if settled.is_some() {
            commands
                .entity(entity)
                .remove::<NetworkNpcMotionSettled0104>();
        }
    }
}

pub fn advance_network_transportation_motion_0104(
    mut commands: Commands,
    time: Option<Res<Time>>,
    mut transportation: Query<
        (Entity, &mut Transform, &NetworkTransportationMotion0104),
        With<NetworkTransportation0104>,
    >,
) {
    let Some(time) = time else {
        return;
    };
    let delta_seconds = time.delta_secs().max(0.0);
    for (entity, mut transform, motion) in &mut transportation {
        if advance_server_entity_motion(
            &mut transform,
            motion.destination,
            motion.speed,
            delta_seconds,
            // Clean cnBusMoveController.ForceUpdate changes position only;
            // its packet has no facing and SetupBus retains prefab rotation.
            false,
        ) {
            commands
                .entity(entity)
                .remove::<NetworkTransportationMotion0104>();
        }
    }
}

pub(super) fn advance_server_entity_motion(
    transform: &mut Transform,
    destination: Vec3,
    speed: f32,
    delta_seconds: f32,
    turn_to_destination: bool,
) -> bool {
    let delta = destination - transform.translation;
    let distance = delta.length();
    if !distance.is_finite() || distance <= ARRIVAL_EPSILON {
        transform.translation = destination;
        return true;
    }
    let horizontal = Vec3::new(delta.x, 0.0, delta.z);
    if turn_to_destination && horizontal.length_squared() > f32::EPSILON {
        transform.rotation = Transform::IDENTITY
            .looking_to(horizontal.normalize(), Vec3::Y)
            .rotation;
    }
    let maximum_step = speed.max(0.0) * delta_seconds;
    if maximum_step <= 0.0 || maximum_step + ARRIVAL_EPSILON >= distance {
        transform.translation = destination;
        true
    } else {
        transform.translation += delta / distance * maximum_step;
        false
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_network_npc_high_layers_0104(
    root: Entity,
    visual: &NetworkNpcVisual0104,
    alive: bool,
    layers: &mut NetworkNpcAnimationLayers0104,
    high: &mut NetworkNpcHighAnimations0104,
    player: &mut AnimationPlayer,
    prepared: &NetworkNpcPreparedAnimationGraph0104,
    random: &mut LegacyNanoStandRandomStream,
) -> bool {
    if high.reset_revision != layers.reset_revision {
        for pair in prepared.additive_nodes.values() {
            player.stop(pair.clip);
            player.stop(pair.reference);
        }
        *high = NetworkNpcHighAnimations0104 {
            reset_revision: layers.reset_revision,
            ..default()
        };
    }
    let mut completed_owner = false;
    for index in 0..2 {
        let Some(request) = layers.high[index] else {
            continue;
        };
        let same = high.requests[index] == request.revision;
        if !same {
            high.requests[index] = request.revision;
            let clip = if index == 0 {
                network_npc_melee_clip_0104(random.next_index(2), |name| {
                    prepared.nodes.contains_key(name)
                })
            } else {
                "wound"
            };
            if let Some(pair) = prepared.additive_nodes.get(clip).copied() {
                let started = restart_network_npc_additive_pair_0104(player, prepared, pair);
                if started || high.applied[index].is_none() {
                    high.applied[index] = Some(NetworkNpcAnimationApplied0104 {
                        root,
                        requested_clip: request.clip.name().to_owned(),
                        clip: clip.to_owned(),
                        node: pair.clip,
                        additive: true,
                        combat_revision: Some(request.revision),
                        playback_revision: Some(request.revision),
                    });
                }
                if started || high.cursors[index].is_none() {
                    high.cursors[index] = Some(network_npc_idle_cursor_0104(player, pair.clip));
                }
            } else {
                high.applied[index] = None;
            }
        }
        let completed = high.applied[index].as_ref().is_none_or(|applied| {
            let (crossed, observed) = network_npc_idle_end_crossed_0104(
                player,
                applied,
                high.cursors[index].as_ref(),
                visual.animation_ends.get(&applied.clip).copied(),
            );
            if let Some(observed) = observed {
                high.cursors[index] = Some(observed);
            }
            crossed
                || player
                    .animation(applied.node)
                    .is_none_or(|active| active.is_finished())
        });
        if completed {
            layers.high[index] = None;
            if layers.high_owner == Some(request.revision) {
                layers.high_owner = None;
                if alive {
                    layers.low = None;
                    layers.stand_attack = false;
                    completed_owner = true;
                }
            }
        }
    }
    completed_owner
}

pub(super) fn sync_network_pc_vehicle_weapon_visibility(
    players: Query<(&NetworkPcVisual0104, &NetworkPcAppearance0104)>,
    parents: Query<&ChildOf>,
    mut weapons: Query<(Entity, &mut Visibility), With<NetworkPcWeaponAttachment0104>>,
) {
    for (entity, mut visibility) in &mut weapons {
        let Some((_, appearance)) = players
            .iter()
            .find(|(v, _)| is_descendant_of(entity, v.rig_root, &parents))
        else {
            continue;
        };
        let desired = if appearance.0.pc_state == 8 {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if *visibility != desired {
            *visibility = desired;
        }
    }
}
