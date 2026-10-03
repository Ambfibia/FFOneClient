use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct NetworkNpcAnimationEffectEvent0104 {
    pub(super) clip: String,
    pub(super) time: f32,
    pub(super) effect_id: i32,
    pub(super) node_name: Option<String>,
}

pub(super) fn parse_network_npc_animation_document(bytes: &[u8]) -> Result<Value, String> {
    if bytes.len() < 20 || &bytes[0..4] != b"glTF" {
        return Err("semantic character is not a GLB 2.0 container".to_owned());
    }
    let version = u32::from_le_bytes(bytes[4..8].try_into().expect("checked GLB header"));
    let declared_length =
        u32::from_le_bytes(bytes[8..12].try_into().expect("checked GLB header")) as usize;
    if version != 2 || declared_length != bytes.len() {
        return Err(format!(
            "semantic character GLB header mismatch: version={version}, declared={declared_length}, actual={}",
            bytes.len()
        ));
    }
    let json_length =
        u32::from_le_bytes(bytes[12..16].try_into().expect("checked JSON chunk")) as usize;
    let json_type = u32::from_le_bytes(bytes[16..20].try_into().expect("checked JSON chunk"));
    let json_end = 20usize
        .checked_add(json_length)
        .ok_or_else(|| "semantic character GLB JSON length overflowed".to_owned())?;
    if json_type != 0x4e4f_534a || json_end > bytes.len() {
        return Err("semantic character GLB has no valid leading JSON chunk".to_owned());
    }
    serde_json::from_slice(&bytes[20..json_end])
        .map_err(|error| format!("semantic character GLB JSON is invalid: {error}"))
}

pub(super) fn parse_network_npc_animation_effect_events(
    bytes: &[u8],
) -> Result<Vec<NetworkNpcAnimationEffectEvent0104>, String> {
    let document = parse_network_npc_animation_document(bytes)?;
    let animations = document
        .get("animations")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let mut parsed = Vec::new();
    for (animation_index, animation) in animations.iter().enumerate() {
        let context = format!("animations[{animation_index}]");
        let clip = required_string(animation, "name", &context)?;
        let Some(events) = animation
            .pointer("/extras/nonTrs/events")
            .and_then(Value::as_array)
        else {
            continue;
        };
        for (event_index, event) in events.iter().enumerate() {
            let function = required_string(
                event,
                "functionName",
                &format!("{context}.events[{event_index}]"),
            )?;
            if function != "particle" && function != "tag_p" {
                continue;
            }
            let parameter = required_string(
                event,
                "stringParameter",
                &format!("{context}.events[{event_index}]"),
            )?;
            let (effect_text, node_name) = if function == "tag_p" {
                let (effect, node) = parameter.split_once(':').ok_or_else(|| {
                    format!("{context}.events[{event_index}] tag_p has no effect:node parameter")
                })?;
                if node.is_empty() {
                    return Err(format!(
                        "{context}.events[{event_index}] tag_p has an empty node"
                    ));
                }
                (effect, Some(node.to_owned()))
            } else {
                (parameter, None)
            };
            let effect_id = effect_text.parse::<i32>().map_err(|error| {
                format!(
                    "{context}.events[{event_index}] has invalid effect ID {effect_text:?}: {error}"
                )
            })?;
            if !RETROBUTION_TUTORIAL_EFFECT_IDS.contains(&effect_id)
                && !RETROBUTION_FUSION_ACTOR_EFFECT_IDS.contains(&effect_id)
                && !RETROBUTION_CHARACTER_ACTOR_EFFECT_IDS.contains(&effect_id)
            {
                return Err(format!(
                    "{context}.events[{event_index}] references ES{effect_id} outside the exact character actor catalog"
                ));
            }
            let seconds = event
                .get("time")
                .and_then(Value::as_f64)
                .ok_or_else(|| format!("{context}.events[{event_index}].time must be numeric"))?;
            if !seconds.is_finite() || seconds < 0.0 || seconds > f64::from(f32::MAX) {
                return Err(format!(
                    "{context}.events[{event_index}].time must be finite and non-negative"
                ));
            }
            parsed.push(NetworkNpcAnimationEffectEvent0104 {
                clip: clip.to_owned(),
                time: seconds as f32,
                effect_id,
                node_name,
            });
        }
    }
    Ok(parsed)
}

/// Authored `AnimationEventHandler.end` times. `NpcAnimation.EndAnimation`
/// chooses the next state there, a quarter second before most last keys.
/// Several `end` events use the earliest. Four clean NPC stand clips carry an
/// `end` at 0 s; such an event would fire on every restart and is ignored.
pub(super) fn parse_network_npc_animation_end_events(bytes: &[u8]) -> Result<BTreeMap<String, f32>, String> {
    let document = parse_network_npc_animation_document(bytes)?;
    let animations = document
        .get("animations")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let mut ends = BTreeMap::new();
    for (animation_index, animation) in animations.iter().enumerate() {
        let context = format!("animations[{animation_index}]");
        let clip = required_string(animation, "name", &context)?;
        let Some(events) = animation
            .pointer("/extras/nonTrs/events")
            .and_then(Value::as_array)
        else {
            continue;
        };
        for (event_index, event) in events.iter().enumerate() {
            let event_context = format!("{context}.events[{event_index}]");
            if required_string(event, "functionName", &event_context)? != "end" {
                continue;
            }
            let seconds = event
                .get("time")
                .and_then(Value::as_f64)
                .ok_or_else(|| format!("{event_context}.time must be numeric"))?;
            if !seconds.is_finite() || seconds < 0.0 || seconds > f64::from(f32::MAX) {
                return Err(format!(
                    "{event_context}.time must be finite and non-negative"
                ));
            }
            if seconds <= 0.0 {
                continue;
            }
            let seconds = seconds as f32;
            ends.entry(clip.to_owned())
                .and_modify(|end: &mut f32| *end = end.min(seconds))
                .or_insert(seconds);
        }
    }
    Ok(ends)
}

#[derive(Default, Resource)]
pub(super) struct NetworkPcAnimationRevision0104(pub(super) u64);

#[derive(Default, Resource)]
pub(super) struct NetworkHnpcAnimationRevision0104(pub(super) u64);

#[derive(Clone, Debug, PartialEq, Eq, Component)]
pub(crate) enum NetworkPcRigAppearanceStatus0104 {
    Loading,
    Ready,
    Blocked(String),
}

pub(super) fn sync_network_pc_animation_0104(
    mut commands: Commands,
    mut revisions: ResMut<NetworkPcAnimationRevision0104>,
    weapon_catalog: Res<PlayerWeaponAnimationCatalog>,
    content: Option<Res<crate::tutorial_mission_content::TutorialMissionContent>>,
    requests: Query<&NativePlayerRigAnimationRequest>,
    players: Query<
        (
            &NetworkPcVisual0104,
            Ref<RemoteAnimation>,
            &NetworkPcAppearance0104,
        ),
        Or<(
            Changed<NetworkPcVisual0104>,
            Changed<RemoteAnimation>,
            Changed<NetworkPcAppearance0104>,
        )>,
    >,
) {
    for (visual, animation, appearance) in &players {
        revisions.0 = revisions.0.wrapping_add(1).max(1);
        let profile = weapon_catalog.profile_for_item(visual.request.equipment[0].item_id);
        let vehicle_type = (appearance.0.pc_state == 8
            && animation.state != RemoteAnimationState::Attacking)
            .then(|| {
                content
                    .as_ref()
                    .and_then(|c| c.gameplay_vehicle_equip_type(appearance.0.equipment[8].item_id))
            })
            .flatten();
        let (clip, repeat) = network_pc_vehicle_animation_clip(animation.state, vehicle_type)
            .unwrap_or_else(|| network_pc_animation_clip(animation.state, profile));
        // Movement packets and HP updates must not rewind an unchanged loop.
        if requests.get(visual.rig_root).is_ok_and(|request| {
            request.clip == clip && request.repeat == repeat && (repeat || !animation.is_changed())
        }) {
            continue;
        }
        commands
            .entity(visual.rig_root)
            .insert(NativePlayerRigAnimationRequest {
                clip: clip.to_owned(),
                revision: revisions.0,
                repeat,
            });
    }
}

pub(super) fn network_pc_animation_clip(
    state: RemoteAnimationState,
    weapon_profile: Option<PlayerWeaponAnimationProfile>,
) -> (&'static str, bool) {
    if let RemoteAnimationState::Emoting { clip } = state {
        return (clip.name(), false);
    }
    let (base_clip, repeat) = match state {
        RemoteAnimationState::Idle => (TutorialPlayerClip::Stand1, true),
        RemoteAnimationState::Attacking => (
            weapon_profile.map_or(TutorialPlayerClip::Attack1, |p| p.attack(false)),
            false,
        ),
        RemoteAnimationState::Moving { direction_key } => (
            if (4..=6).contains(&direction_key) {
                TutorialPlayerClip::RunBack
            } else {
                TutorialPlayerClip::Run
            },
            true,
        ),
        RemoteAnimationState::Jumping { .. } => (TutorialPlayerClip::Jump, false),
        RemoteAnimationState::Emoting { .. } => unreachable!("handled above"),
    };
    let clip = weapon_profile.map_or(base_clip, |profile| profile.locomotion_variant(base_clip));
    (clip.name(), repeat)
}

pub(super) fn block_network_pc_rig(
    commands: &mut Commands,
    rig: &NetworkPcRig0104,
    status: &mut NetworkPcRigAppearanceStatus0104,
    detail: String,
) {
    if matches!(status, NetworkPcRigAppearanceStatus0104::Blocked(_)) {
        return;
    }
    *status = NetworkPcRigAppearanceStatus0104::Blocked(detail.clone());
    commands
        .entity(rig.pc_root)
        .insert(NetworkPcVisualIssue0104 {
            pc_id: rig.pc_id,
            detail,
        });
}

#[derive(Clone, Debug, PartialEq, Eq, Component)]
pub enum NetworkHnpcRigAppearanceStatus0104 {
    Loading,
    Ready,
    Blocked(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Component)]
pub(super) struct NetworkHnpcAnimationState0104 {
    pub(super) rig_root: Entity,
    pub(super) requested_clip: String,
    pub(super) clip: String,
    pub(super) combat_revision: Option<u64>,
    pub(super) rig_revision: u64,
    pub(super) repeat: bool,
    pub(super) idle: bool,
}

pub(super) fn sync_network_hnpc_animation_0104(
    mut commands: Commands,
    mut revisions: ResMut<NetworkHnpcAnimationRevision0104>,
    mut random: Local<crate::legacy_npc_nano_animation::LegacyNanoStandRandomStream>,
    roots: Query<(
        Entity,
        &NetworkHnpcVisual0104,
        &NetworkNpcAppearance0104,
        Option<&NetworkNpcMotion0104>,
        Option<&NetworkNpcCombatAnimation0104>,
        Option<&NetworkNpcReadyAnimation0104>,
        Option<&NetworkHnpcAnimationState0104>,
    )>,
    rigs: Query<(
        &NetworkHnpcRig0104,
        Option<&NativePlayerRigAnimationApplied>,
        Option<&NativePlayerRigStand1Playback>,
        Option<&NativePlayerRigAnimationIssue>,
        Option<&NetworkHnpcIdleEventCursor0104>,
    )>,
    players: Query<&AnimationPlayer>,
) {
    for (root, visual, appearance, motion, combat, ready, current) in &roots {
        let current = current.filter(|current| current.rig_root == visual.rig_root);
        let Ok((rig, applied, playback, issue, cursor)) = rigs.get(visual.rig_root) else {
            continue;
        };
        let base_clip = match (appearance.0.hp, combat, motion, ready) {
            (hp, _, _, _) if hp <= 0 => "death",
            (_, Some(combat), _, _) => combat.clip.name(),
            (_, None, Some(motion), _) if motion.move_style == 0 => "walk",
            (_, None, Some(_), _) => "run",
            (_, None, None, Some(_)) => "ready",
            (_, None, None, None) => "stand1",
        };
        let idle = appearance.0.hp > 0 && combat.is_none() && motion.is_none() && ready.is_none();
        let idle_finished = idle
            && current.is_some_and(|current| {
                current.idle
                    && applied.is_some_and(|applied| {
                        applied.clip == current.clip && applied.revision == current.rig_revision
                    })
                    && playback.is_some_and(|playback| {
                        players.get(playback.animation_player).is_ok_and(|player| {
                            let Some(active) = player.animation(playback.animation_node) else {
                                return false;
                            };
                            let previous = cursor.filter(|cursor| {
                                cursor.revision == current.rig_revision
                                    && cursor.completions <= active.completions()
                            });
                            let (seek, completions) = previous
                                .map(|cursor| (cursor.seek, cursor.completions))
                                .unwrap_or((-f32::EPSILON, 0));
                            // The end event can precede the final pose. Idle
                            // gestures select their next state only after a
                            // full loop, just like ordinary NPC idle clips.
                            let finished = active.completions() > completions;
                            if previous.is_none()
                                || seek != active.seek_time()
                                || completions != active.completions()
                            {
                                commands.entity(visual.rig_root).insert(
                                    NetworkHnpcIdleEventCursor0104 {
                                        revision: current.rig_revision,
                                        seek: active.seek_time(),
                                        completions: active.completions(),
                                    },
                                );
                            }
                            finished
                        })
                    })
            });
        let desired = if idle {
            if let Some(current) = current.filter(|current| current.idle && !idle_finished) {
                current.requested_clip.as_str()
            } else {
                hnpc_idle_clip(
                    rig.idle_clips.as_ref(),
                    current.map(|current| current.clip.as_str()).unwrap_or(""),
                    random.next_index(100),
                )
            }
        } else {
            network_hnpc_animation_clip(base_clip, rig.look.weapon_animation_profile)
        };
        let combat_revision = combat.map(|combat| combat.revision);
        let repeat = appearance.0.hp > 0 && combat.is_none_or(|combat| combat.clip.repeats());

        if let (Some(combat), Some(current)) = (combat, current)
            && !combat.clip.repeats()
            && current.combat_revision == Some(combat.revision)
            && let (Some(applied), Some(playback)) = (applied, playback)
            && applied.clip == current.clip
            && applied.revision == current.rig_revision
            && !applied.repeat
            && players
                .get(playback.animation_player)
                .is_ok_and(|player| player.all_finished())
        {
            commands
                .entity(root)
                .remove::<NetworkNpcCombatAnimation0104>()
                .insert(NetworkNpcReadyAnimation0104);
            continue;
        }

        if let Some(current) = current
            && current.requested_clip == desired
            && current.combat_revision == combat_revision
            && let Some(issue) = issue
            && issue.0.contains(current.clip.as_str())
        {
            if combat.is_some_and(|combat| !combat.clip.repeats()) {
                commands
                    .entity(root)
                    .remove::<NetworkNpcCombatAnimation0104>()
                    .insert(NetworkNpcReadyAnimation0104);
                continue;
            }
            let fallback = if combat.is_some() && current.clip == desired {
                Some("ready")
            } else if desired != "death" && current.clip != "stand1" {
                Some("stand1")
            } else {
                None
            };
            if let Some(fallback) = fallback {
                commands.entity(visual.rig_root).insert(
                    crate::player_shared_rig::NativePlayerRigAnimationBlend(Duration::from_millis(
                        if idle { 300 } else { 200 },
                    )),
                );
                revisions.0 = revisions.0.wrapping_add(1).max(1);
                let rig_revision = revisions.0;
                commands
                    .entity(visual.rig_root)
                    .insert(NativePlayerRigAnimationRequest {
                        clip: fallback.to_owned(),
                        revision: rig_revision,
                        repeat,
                    })
                    .remove::<NativePlayerRigAnimationIssue>();
                commands.entity(root).insert(NetworkHnpcAnimationState0104 {
                    rig_root: visual.rig_root,
                    requested_clip: desired.to_owned(),
                    clip: fallback.to_owned(),
                    combat_revision,
                    rig_revision,
                    repeat,
                    idle,
                });
            }
            continue;
        }

        // CrossFade on the already playing ambient clip preserves its time.
        // Let it finish the tail and wrap, without a new request.
        if current.is_some_and(|current| {
            current.requested_clip == desired
                && current.combat_revision == combat_revision
                && current.repeat == repeat
                && current.idle == idle
        }) {
            continue;
        }
        revisions.0 = revisions.0.wrapping_add(1).max(1);
        let rig_revision = revisions.0;
        commands.entity(visual.rig_root).insert(
            crate::player_shared_rig::NativePlayerRigAnimationBlend(Duration::from_millis(
                if idle { 300 } else { 200 },
            )),
        );
        debug!(
            npc_type = visual.npc_type,
            clip = desired,
            idle,
            rig_revision,
            "HNPC animation request"
        );
        commands
            .entity(visual.rig_root)
            .insert(NativePlayerRigAnimationRequest {
                clip: desired.to_owned(),
                revision: rig_revision,
                repeat,
            });
        commands.entity(root).insert(NetworkHnpcAnimationState0104 {
            rig_root: visual.rig_root,
            requested_clip: desired.to_owned(),
            clip: desired.to_owned(),
            combat_revision,
            rig_revision,
            repeat,
            idle,
        });
    }
}

/// HNPC idle sets use three equally weighted table slots; repeating an
/// alternate selects slot zero. Missing rig clips fall back to stand1 in the
/// request owner. Equipment never prefixes these full-body ambient clips.
pub(super) fn hnpc_idle_clip<'a>(clips: Option<&'a [String; 3]>, current: &str, roll: usize) -> &'a str {
    if let Some(clips) = clips {
        let index = if roll < 34 {
            0
        } else if roll < 67 {
            1
        } else {
            2
        };
        return if index != 0 && clips[index] == current {
            &clips[0]
        } else {
            &clips[index]
        };
    }
    let clip = match roll {
        0..40 => "stand1",
        40..70 => "stand2",
        70..90 => "stand3",
        _ => "stand4",
    };
    if clip == current { "stand1" } else { clip }
}

pub(super) fn network_hnpc_animation_clip(
    base_clip: &'static str,
    profile: Option<PlayerWeaponAnimationProfile>,
) -> &'static str {
    let Some(profile) = profile else {
        return base_clip;
    };
    match base_clip {
        "run" => profile.run(false).name(),
        "ready" => profile.ready().name(),
        "melee1" => profile.attack(false).name(),
        _ => base_clip,
    }
}

pub(super) fn block_network_hnpc_rig(
    commands: &mut Commands,
    rig: &NetworkHnpcRig0104,
    status: &mut NetworkHnpcRigAppearanceStatus0104,
    detail: String,
) {
    if matches!(status, NetworkHnpcRigAppearanceStatus0104::Blocked(_)) {
        return;
    }
    *status = NetworkHnpcRigAppearanceStatus0104::Blocked(detail.clone());
    commands
        .entity(rig.npc_root)
        .insert(NetworkNpcVisualIssue0104 {
            npc_type: rig.npc_type,
            detail,
        });
}

#[must_use]
pub(super) fn network_npc_clip_is_additive_0104(name: &str) -> bool {
    !name.contains("upper")
        && !name.contains("event")
        && (name.contains("melee") || name.contains("wound"))
}

#[must_use]
pub(crate) fn network_npc_animation_uses_delta_additive_layer_0104(
    glb_path: &str,
    clip_name: &str,
) -> bool {
    matches!(
        glb_path,
        "characters/mobs/mob_cerberus/mob_cerberus.glb"
            | "characters/mobs/mob_oilmonster/mob_oilmonster.glb"
            | "characters/mobs/mob_sneakyspawn/mob_sneakyspawn.glb"
    ) && network_npc_clip_is_additive_0104(clip_name)
}

#[derive(Clone, Debug, PartialEq, Eq, Component)]
pub(super) struct NetworkNpcAnimationApplied0104 {
    pub(super) root: Entity,
    /// Semantic state requested by the lifecycle machine. This remains the
    /// comparison key when a proven ready/idle fallback is selected.
    pub(super) requested_clip: String,
    /// Exact GLTF clip selected for playback and AnimationEvent lookup.
    pub(super) clip: String,
    pub(super) node: AnimationNodeIndex,
    pub(super) additive: bool,
    pub(super) combat_revision: Option<u64>,
    pub(super) playback_revision: Option<u64>,
}

pub(super) struct NetworkNpcPreparedAnimationGraph0104 {
    pub(super) graph: Handle<AnimationGraph>,
    pub(super) nodes: HashMap<String, AnimationNodeIndex>,
    pub(super) additive_nodes: HashMap<String, NetworkNpcAdditivePair0104>,
    pub(super) base_nodes: Vec<AnimationNodeIndex>,
    pub(super) fallback_base_node: Option<AnimationNodeIndex>,
}

/// One stable graph per NPC GLTF. The clean Unity `Animation` component owns
/// every state together; swapping a separate Bevy graph on each transition
/// can race the pose request and restart fallback clips every frame.
#[derive(Default, Resource)]
pub(super) struct NetworkNpcAnimationAssets0104 {
    pub(super) graphs: HashMap<AssetId<Gltf>, NetworkNpcPreparedAnimationGraph0104>,
}

#[cfg(test)]
mod pc_animation_regression {
    use super::*;
    #[test]
    fn repeated_movement_and_hp_updates_do_not_restart_run() {
        let mut app = App::new();
        app.init_resource::<NetworkPcAnimationRevision0104>()
            .init_resource::<PlayerWeaponAnimationCatalog>()
            .add_systems(Update, sync_network_pc_animation_0104);
        let pc =
            ffone_protocol::PcAppearance0104::decode(&[0; ffone_protocol::PcAppearance0104::SIZE])
                .unwrap();
        let rig = app.world_mut().spawn_empty().id();
        let root = app
            .world_mut()
            .spawn((
                NetworkPcVisual0104 {
                    pc_id: 17,
                    rig_root: rig,
                    generation: 1,
                    request: PendingPcVisual0104::from(&pc),
                },
                RemoteAnimation {
                    state: RemoteAnimationState::Moving { direction_key: 1 },
                },
                NetworkPcAppearance0104(pc),
            ))
            .id();
        app.update();
        let revision = app
            .world()
            .get::<NativePlayerRigAnimationRequest>(rig)
            .unwrap()
            .revision;
        app.world_mut().entity_mut(root).insert(RemoteAnimation {
            state: RemoteAnimationState::Moving { direction_key: 1 },
        });
        app.world_mut()
            .get_mut::<NetworkPcAppearance0104>(root)
            .unwrap()
            .0
            .hp = 50;
        app.update();
        let request = app
            .world()
            .get::<NativePlayerRigAnimationRequest>(rig)
            .unwrap();
        assert_eq!(request.clip, "run");
        assert_eq!(request.revision, revision);
        app.world_mut()
            .entity_mut(root)
            .insert(RemoteAnimation::default());
        app.update();
        assert_eq!(
            app.world()
                .get::<NativePlayerRigAnimationRequest>(rig)
                .unwrap()
                .clip,
            "stand1"
        );
    }
}
