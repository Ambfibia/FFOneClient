use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) enum SoundCue {
    Exact(&'static str),
    Random(&'static [&'static str]),
}

impl SoundCue {
    pub(super) fn choose(self, random: &mut GameplayAudioRandom) -> SelectedSound {
        match self {
            Self::Exact(true_name) => SelectedSound {
                true_name: true_name.to_owned(),
                voice_route: false,
                npc_dialogue: false,
                random_voice: false,
            },
            Self::Random(variants) => {
                debug_assert!(!variants.is_empty());
                SelectedSound {
                    true_name: variants[(random.next_u32() as usize) % variants.len()].to_owned(),
                    voice_route: false,
                    npc_dialogue: false,
                    random_voice: true,
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct SelectedSound {
    pub(super) true_name: String,
    pub(super) voice_route: bool,
    pub(super) npc_dialogue: bool,
    pub(super) random_voice: bool,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct LegacySoundEvent {
    pub(super) seconds: f32,
    pub(super) cue: SoundCue,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ActorClipAudio {
    pub(super) clip: &'static str,
    pub(super) duration_seconds: f32,
    pub(super) events: &'static [LegacySoundEvent],
}

pub(super) fn actor_clip_audio(npc_type: i32, clip: &str) -> Option<&'static ActorClipAudio> {
    actor_clips(npc_type)?
        .iter()
        .find(|audio| audio.clip == clip)
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ScheduledSound {
    pub(super) anchor: Entity,
    pub(super) remaining_seconds: f32,
    pub(super) cue: SoundCue,
    pub(super) priority: bool,
}

#[derive(Clone, Debug)]
pub(super) struct ScheduledNamedSound {
    pub(super) anchor: Entity,
    pub(super) remaining_seconds: f32,
    pub(super) true_name: String,
}

#[derive(Clone, Debug)]
pub(super) struct QueuedAnimationSound {
    pub(super) random_voice: bool,
    pub(super) anchor: Entity,
    pub(super) true_name: String,
    pub(super) voice_route: bool,
}

#[derive(Clone, Debug)]
pub(super) struct QueuedUiSound {
    pub(super) true_name: String,
    pub(super) clean_gain: f32,
    pub(super) pitch: f32,
}

#[derive(Debug)]
pub(super) struct GameplayAudioRandom {
    pub(super) state: u32,
}

impl Default for GameplayAudioRandom {
    fn default() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let folded = nanos ^ (nanos >> 32) ^ (nanos >> 64) ^ (nanos >> 96);
        Self::with_seed(folded as u32)
    }
}

impl GameplayAudioRandom {
    pub(super) const FALLBACK_SEED: u32 = 0x6a09_e667;

    pub(super) const fn with_seed(seed: u32) -> Self {
        Self {
            state: if seed == 0 { Self::FALLBACK_SEED } else { seed },
        }
    }

    pub(super) fn next_u32(&mut self) -> u32 {
        let mut value = self.state;
        value ^= value << 13;
        value ^= value >> 17;
        value ^= value << 5;
        self.state = value;
        value
    }
}

/// The exact `eVoice` branches used by `AvatarUtil.CallVoicePlay` for NPC
/// interaction and mission dialogue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyNpcVoiceCue {
    Greeting,
    QuestGreeting,
    Farewell,
    QuestAccepted,
    QuestCompleted,
    QuestStartEscort,
    WarpOk,
    MoveOk,
    RaceStart,
    RaceEnd,
    BarberOk,
}

/// Exact `NpcIconMode.NpcGreetingBubble` choice. Race-finish NPCs own their
/// special branch before the ordinary quest-greeting test.
#[must_use]
pub const fn legacy_npc_open_voice_cue(
    npc_service_category: i32,
    ring_race_active: bool,
    has_quest: bool,
) -> LegacyNpcVoiceCue {
    if npc_service_category == 14 {
        if ring_race_active {
            LegacyNpcVoiceCue::RaceEnd
        } else {
            LegacyNpcVoiceCue::Greeting
        }
    } else if has_quest {
        LegacyNpcVoiceCue::QuestGreeting
    } else {
        LegacyNpcVoiceCue::Greeting
    }
}

pub(super) fn legacy_npc_voice_true_name(
    owner: &str,
    cue: LegacyNpcVoiceCue,
    random: &mut GameplayAudioRandom,
) -> Option<String> {
    // `CallVoicePlay` rejects the same empty/sentinel string-table owners.
    if owner.len() <= 1 {
        return None;
    }
    let parts = owner.split('_').collect::<Vec<_>>();
    let base = if parts.len() >= 3 {
        format!("{}_{}", parts[0], parts[1])
    } else {
        owner.to_owned()
    };
    let random_take = |random: &mut GameplayAudioRandom, total: u32| random.next_u32() % total + 1;
    Some(match cue {
        // Unity `Random.Range(int, int)` excludes its upper bound.
        LegacyNpcVoiceCue::Greeting => {
            format!("{owner}_greeting0{}", random_take(random, 2))
        }
        LegacyNpcVoiceCue::QuestGreeting => format!("{base}_qgreeting"),
        LegacyNpcVoiceCue::Farewell => {
            format!("{base}_farewell0{}", random_take(random, 3))
        }
        LegacyNpcVoiceCue::QuestAccepted => {
            format!("{base}_goodluck0{}", random_take(random, 3))
        }
        LegacyNpcVoiceCue::QuestCompleted => {
            format!("{base}_nicejob0{}", random_take(random, 3))
        }
        LegacyNpcVoiceCue::QuestStartEscort => {
            format!("{owner}_laugh0{}", random_take(random, 3))
        }
        LegacyNpcVoiceCue::WarpOk => {
            format!("{owner}_clickwarp0{}", random_take(random, 3))
        }
        LegacyNpcVoiceCue::MoveOk => {
            format!("{owner}_clickmove0{}", random_take(random, 3))
        }
        LegacyNpcVoiceCue::BarberOk => format!("{owner}_clickbarb0{}", random_take(random, 1)),
        LegacyNpcVoiceCue::RaceStart => {
            format!("{owner}_clickstart0{}", random_take(random, 3))
        }
        LegacyNpcVoiceCue::RaceEnd => {
            format!("{owner}_racefinished0{}", random_take(random, 3))
        }
    })
}

#[derive(Debug, Resource, Default)]
pub struct GameplayAudioRuntime {
    pub(super) nano: nano::NanoVoiceRuntime,
    pub(super) random: GameplayAudioRandom,
    pub(super) scheduled: Vec<ScheduledSound>,
    pub(super) scheduled_named: Vec<ScheduledNamedSound>,
    pub(super) animation_sounds: Vec<QueuedAnimationSound>,
    pub(super) npc_voice_changes: HashMap<Entity, Option<String>>,
    pub(super) closing_npc_dialogues: HashSet<Entity>,
    pub(super) pending_npc_voices: HashMap<Entity, (SelectedSound, Handle<AudioSource>)>,
    pub(super) active_npc_voices: HashMap<Entity, Entity>,
    pub(super) ui_sounds: Vec<QueuedUiSound>,
    pub(super) player_locomotion: HashMap<Entity, PlayerLocomotionAudioCursor>,
    pub(super) player_death_poses: HashSet<Entity>,
    pub(super) player_inventory_audio: HashMap<Entity, Entity>,
    pub(super) service_inventory_audio_active: bool,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PlayerLocomotionAudioCursor {
    pub(super) locomotion: LegacyLocomotionState,
    pub(super) elapsed_seconds: f32,
    pub(super) next_loop_event_seconds: Option<f32>,
}

impl GameplayAudioRuntime {
    /// Bank, Enchant, player trade and street stores call PlayUIModeSound
    /// without opening the ordinary UserEquip screen.
    pub fn set_service_inventory_audio_active(&mut self, active: bool) {
        self.service_inventory_audio_active = active;
    }

    #[cfg(test)]
    pub(crate) fn queued_nano_dismissal_count(&self) -> usize {
        self.nano.dismissal_count()
    }
    /// NanoContainer owns farewell independently of the disappearing model.
    pub(crate) fn queue_nano_dismissal(&mut self, nano: Entity, owner: Entity, voice_owner: &str) {
        let take = self.random.next_u32();
        self.nano.dismiss(nano, owner, voice_owner, take);
    }
    #[cfg(test)]
    pub(crate) fn queued_animation_sound_names(&self) -> Vec<(&str, bool)> {
        self.animation_sounds
            .iter()
            .map(|sound| (sound.true_name.as_str(), sound.voice_route))
            .collect()
    }

    /// Queues one non-spatial legacy UI SFX by its semantic Unity true name.
    /// Resolution remains catalog-owned; callers never depend on a physical
    /// native audio path.
    pub fn queue_gameplay_ui_sound(&mut self, true_name: &'static str) {
        self.ui_sounds.push(QueuedUiSound {
            true_name: true_name.to_owned(),
            clean_gain: 0.7,
            pitch: 1.0,
        });
    }

    /// InventoryManager's Buy button chooses Purchase01/02; 0.1 is
    /// SoundUtil's pitchOffset, not a volume or a playback delay.
    pub fn queue_legacy_purchase_sound(&mut self, random_pitch: bool) {
        self.queue_legacy_money_family(&["Purchase01", "Purchase02"], if random_pitch { 0.1 } else { 0.0 });
    }

    pub fn queue_legacy_money_sound(&mut self) {
        self.queue_legacy_money_family(&["Money_Misc01", "Money_Misc02", "Money_Misc03"], 0.1);
    }

    /// cnVendor.ReceivePacket plays this only on buy/battery/sell/restore success.
    pub fn queue_legacy_money_transfer_sound(&mut self) {
        self.queue_legacy_money_family(&["Money_Transfer01", "Money_Transfer02"], 0.1);
    }

    fn queue_legacy_money_family(&mut self, variants: &[&str], pitch_offset: f32) {
        let index = self.random.next_u32() as usize % variants.len();
        let pitch = 1.0 - pitch_offset + 2.0 * pitch_offset * (self.random.next_u32() as f32 / u32::MAX as f32);
        self.ui_sounds.push(QueuedUiSound {
            true_name: variants[index].to_owned(),
            clean_gain: 0.7,
            pitch,
        });
    }

    /// Exact shared `SoundUtil.ButtonSound()` family. Unity chooses the upper
    /// bound exclusively, yielding mouse_click01 through mouse_click05.
    pub fn queue_legacy_button_sound(&mut self) {
        const BUTTONS: [&str; 5] = [
            "mouse_click01",
            "mouse_click02",
            "mouse_click03",
            "mouse_click04",
            "mouse_click05",
        ];
        let index = self.random.next_u32() as usize % BUTTONS.len();
        self.queue_gameplay_ui_sound(BUTTONS[index]);
    }

    /// Queues one spatial legacy world SFX at its owning entity. This is the
    /// `SoundUtil.Playsound(target, trueName)` route used by clean NPC modes;
    /// physical audio paths remain catalog-owned.
    pub fn queue_legacy_world_sound(&mut self, anchor: Entity, true_name: &'static str) {
        self.animation_sounds.push(QueuedAnimationSound {
            random_voice: false,
            anchor,
            true_name: true_name.to_owned(),
            voice_route: false,
        });
    }

    /// Clean `GameFrame.ChangeGameMode(UserEquip)` owns one generic screen
    /// edge and `cnAvatarAnimation.SetInvenMode` owns the second.  Both route
    /// through `PlayUIModeSound` at the legacy 0.7 SFX gain.
    pub fn queue_user_equip_mode_edge(&mut self, opening: bool) {
        let true_name = if opening {
            "Open_Screen"
        } else {
            "Close_Screen"
        };
        self.queue_gameplay_ui_sound(true_name);
        self.queue_gameplay_ui_sound(true_name);
    }

    /// Queues the spatial, localized VO name constructed by clean
    /// `AvatarUtil.CallVoicePlay`. The stable Unity true name is resolved by
    /// [`NativeAudioCatalog`], so editable physical filenames remain free to
    /// change.
    pub fn queue_legacy_npc_voice(&mut self, anchor: Entity, owner: &str, cue: LegacyNpcVoiceCue) {
        if self.closing_npc_dialogues.contains(&anchor)
            || (cue != LegacyNpcVoiceCue::Farewell
                && self
                    .npc_voice_changes
                    .get(&anchor)
                    .is_some_and(Option::is_none))
        {
            // A close committed this frame. A stale input/packet callback
            // must not resurrect the voice before that cancellation is driven.
            return;
        }
        let Some(true_name) = legacy_npc_voice_true_name(owner, cue, &mut self.random) else {
            return;
        };
        // One voice source per NPC: the last request in a frame replaces
        // the previous request, including clips still waiting for asset load.
        self.npc_voice_changes.insert(anchor, Some(true_name));
        if cue == LegacyNpcVoiceCue::Farewell {
            // EndMode is terminal for this frame: a stale greeting must not
            // replace its farewell. A later, genuine reopen can speak again.
            self.closing_npc_dialogues.insert(anchor);
        }
    }

    pub fn stop_npc_dialogue_voice(&mut self, anchor: Entity) {
        self.npc_voice_changes.insert(anchor, None);
    }

    pub(super) fn schedule(&mut self, anchor: Entity, seconds: f32, cue: SoundCue) {
        self.schedule_with_priority(anchor, seconds, cue, false);
    }

    pub(super) fn schedule_with_priority(
        &mut self,
        anchor: Entity,
        seconds: f32,
        cue: SoundCue,
        priority: bool,
    ) {
        self.scheduled.push(ScheduledSound {
            anchor,
            remaining_seconds: seconds.max(0.0),
            cue,
            priority,
        });
    }

    pub(super) fn schedule_named(&mut self, anchor: Entity, seconds: f32, true_name: String) {
        self.scheduled_named.push(ScheduledNamedSound {
            anchor,
            remaining_seconds: seconds.max(0.0),
            true_name,
        });
    }

    /// Queue one clean `AnimationEventHandler.sound` payload.
    ///
    /// Retrobution expands every `(RAND:min-max)` token when the event fires,
    /// then asks `AssetLoader` for `Sound/<expanded name>`.  FFOne keeps that
    /// timing and lookup identity, but resolves the resulting Unity true name
    /// through [`NativeAudioCatalog`] so a renamed native file remains valid.
    pub fn queue_legacy_animation_sound(&mut self, anchor: Entity, payload: &str) {
        self.queue_legacy_animation_sound_route(anchor, payload, false, false);
    }

    /// Queues an animation-owned character sound for an NPC, mob or fusion.
    /// Character clips can reference either ordinary SFX or localized vocal
    /// takes (and a small number of names exist in both catalog categories),
    /// so this route prefers the semantic voice asset and falls back to SFX.
    pub fn queue_legacy_character_animation_sound(&mut self, anchor: Entity, payload: &str) {
        self.queue_legacy_animation_sound_route(anchor, payload, true, false);
    }

    /// Queue the Nano variant of `AnimationEventHandler.sound`. Retrobution
    /// resolves `_Nan` payloads through the localized VO bundle and all other
    /// Nano payloads through ordinary SFX.
    pub fn queue_legacy_nano_animation_sound(&mut self, anchor: Entity, payload: &str) {
        self.queue_legacy_animation_sound_route(anchor, payload, false, true);
    }

    pub(super) fn queue_legacy_animation_sound_route(
        &mut self,
        anchor: Entity,
        payload: &str,
        prefer_character_voice: bool,
        nano: bool,
    ) {
        let Some(true_name) = expand_legacy_random_sound(payload, &mut self.random) else {
            warn!("Ignored malformed Retrobution animation sound payload {payload:?}");
            return;
        };
        let voice_route =
            prefer_character_voice || (nano && true_name.to_ascii_lowercase().contains("_nan"));
        self.animation_sounds.push(QueuedAnimationSound {
            random_voice: payload.contains("(RAND:"),
            anchor,
            true_name,
            voice_route,
        });
    }

    pub fn queue_player_weapon_attack(
        &mut self,
        player: Entity,
        gender: PlayerRigGender,
        item_id: i16,
        weapon_battery: i32,
        weapon_catalog: &PlayerWeaponAnimationCatalog,
    ) {
        let Some(profile) = weapon_catalog.profile_for_item(item_id) else {
            return;
        };
        let Some(variants) = weapon_catalog.attack_sound_variants_for_item(item_id, weapon_battery)
        else {
            warn!("Weapon item {item_id} has no usable attack-sound table row");
            return;
        };
        let index = self.random.next_u32() as usize % variants.len();
        let true_name = variants[index].clone();
        self.schedule_named(
            player,
            player_weapon_attack_event_seconds(profile, gender),
            true_name,
        );
    }

    /// Queues the clean actor-owned `attack1` animation events used when the
    /// Hand inventory slot is empty. `AvatarAttack` starts both clips while
    /// standing/ready and only the layer-101 upper clip while moving, so the
    /// full-body event is conditional but the upper event is always present.
    pub fn queue_player_unarmed_attack(
        &mut self,
        player: Entity,
        gender: PlayerRigGender,
        full_body: bool,
    ) {
        let (full, upper) = match gender {
            PlayerRigGender::Male => ("M_Avatar_Attack1", "M_Avatar_Attack1upper"),
            PlayerRigGender::Female => ("PlyrAvtF_Attack1", "PlyrAvtF_Attack1upper"),
        };
        // CharacterSelection.resourceFile AnimationClip pathIds
        // F: 34171/34174, M: 34235/34213 all fire `sound` at 0.25 s.
        if full_body {
            self.schedule(player, 0.25, SoundCue::Exact(full));
        }
        self.schedule(player, 0.25, SoundCue::Exact(upper));
    }

    /// Clean `cnAvatarStatus.Damage` voice family. NPC attack result bit 1
    /// promotes damage type 0 (Hurt) to type 7 (Critical); Unity then chooses
    /// one of the two gender-owned variants.
    pub fn queue_player_damage(&mut self, player: Entity, gender: PlayerRigGender, critical: bool) {
        self.schedule(player, 0.0, player_damage_cue(gender, critical));
    }

    /// Exact clean `cnOwnAvatarStatus.BuffTimeTick` infection branch. Unity
    /// starts the shared poison SFX and one randomized gender-owned GooDmg
    /// voice on the same tick.
    pub fn queue_player_infection_damage(&mut self, player: Entity, gender: PlayerRigGender) {
        self.schedule(player, 0.0, SoundCue::Exact("SFX_PoisonDamage"));
        self.schedule(player, 0.0, player_infection_damage_cue(gender));
    }

    /// The same clean damage voice family on the alive-to-dead edge. Local
    /// terminal reactions are priority audio and cannot be discarded by the
    /// general simultaneous-world-SFX budget.
    pub fn queue_player_death(&mut self, player: Entity, gender: PlayerRigGender, critical: bool) {
        self.schedule_with_priority(player, 0.0, player_damage_cue(gender, critical), true);
    }

    pub(super) fn observe_player_death_pose(
        &mut self,
        player: Entity,
        gender: PlayerRigGender,
        clip: crate::avatar_action::LegacyVisualClip,
    ) {
        use crate::avatar_action::LegacyVisualClip;
        if !matches!(clip, LegacyVisualClip::Die | LegacyVisualClip::Death) {
            self.player_death_poses.remove(&player);
            return;
        }
        if !self.player_death_poses.insert(player) || clip != LegacyVisualClip::Die {
            return;
        }
        // The native die pose also owns a sound event at 0.25 seconds.
        // Damage() alone only supplies Hurt/Critical, never this defeat voice.
        let defeat = match gender {
            PlayerRigGender::Male => SoundCue::Random(&[
                "M_Avatar_DefeatLng01",
                "M_Avatar_DefeatLng02",
                "M_Avatar_DefeatLng03",
            ]),
            PlayerRigGender::Female => SoundCue::Random(&[
                "F_Avatar_DefeatLng01",
                "F_Avatar_DefeatLng02",
                "F_Avatar_DefeatLng03",
            ]),
        };
        self.schedule_with_priority(player, 0.25, defeat, true);
    }

    pub fn queue_player_hurt(&mut self, player: Entity, gender: PlayerRigGender) {
        self.queue_player_damage(player, gender, false);
    }

    /// Clean `EpJumppadTrigger` plays this immediately after the armed pad is
    /// consumed by `OnControllerColliderHit`.
    pub fn queue_player_jumppad(&mut self, player: Entity) {
        self.schedule(player, 0.0, SoundCue::Exact("SFX_bouncetech"));
    }

    /// Clean local `PC_VEHICLE_*_SUCC` callbacks play these only after the
    /// shard commits the mount transition.
    pub fn queue_player_vehicle_mode_edge(&mut self, player: Entity, mounted: bool) {
        self.schedule(
            player,
            0.0,
            SoundCue::Exact(if mounted {
                "Vehicle_GetOn"
            } else {
                "Vehicle_GetOff"
            }),
        );
    }

    pub fn queue_tutorial_actor_damage(&mut self, actor: Entity, npc_type: i32) {
        let cue = match npc_type {
            2674 | 2897 => SoundCue::Random(&["Spawn_CS_Wound_2", "Spawn_CS_Wound_1"]),
            2675 => SoundCue::Random(&["DexbotCerberus_CB_Wound_2", "DexbotCerberus_CB_Wound_1"]),
            2676 => SoundCue::Random(&["OilMonster_OO_Wound_2", "OilMonster_OO_Wound_1"]),
            2677 => SoundCue::Exact("DexbotBat_TW_Wound"),
            2678 => SoundCue::Exact("FusionButtercup_Wound"),
            _ => return,
        };
        self.schedule(actor, 0.0, cue);
    }
}

#[derive(Component)]
pub struct GameplaySfxAudio;

/// Per-channel clean `cnSoundOption` gain. The executable mirrors its live
/// OptionMode state here; library tests and embedders retain the clean 0.5
/// defaults. Master volume remains owned by Bevy's `GlobalVolume`.
#[derive(Clone, Copy, Debug, PartialEq, Resource)]
pub struct RetrobutionAudioMix {
    pub music: f32,
    pub ambient: f32,
    pub effects: f32,
    pub voice: f32,
}

impl Default for RetrobutionAudioMix {
    fn default() -> Self {
        Self {
            music: 0.5,
            ambient: 0.5,
            effects: 0.5,
            voice: 0.5,
        }
    }
}

#[derive(SystemSet, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GameplayAudioSet {
    Collect,
    Drive,
}

pub struct GameplayAudioPlugin;

impl Plugin for GameplayAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GameplayChannelMixPlugin)
            .init_resource::<GameplayAudioRuntime>()
            .init_resource::<RetrobutionAudioMix>()
            .add_systems(
                Update,
                emit_gameplay_actor_audio_events
                    .after(apply_tutorial_actor_animation_playback)
                    .in_set(GameplayAudioSet::Collect),
            )
            .add_systems(
                Update,
                drive_gameplay_audio
                    .in_set(GameplayAudioSet::Drive)
                    .after(GameplayAudioSet::Collect)
                    .after(crate::player_emote::PlayerEmoteAdvance)
                    .after(LegacyAvatarActionSet::Locomotion),
            );
    }
}

pub(super) fn expand_legacy_random_sound(payload: &str, random: &mut GameplayAudioRandom) -> Option<String> {
    let mut sound = payload.trim().to_owned();
    while let Some(start) = sound.find("(RAND:") {
        let tail = &sound[start + 6..];
        let close = tail.find(')')?;
        let range = &tail[..close];
        let (minimum, maximum) = range.split_once('-')?;
        let minimum = minimum.parse::<i32>().ok()?;
        let maximum = maximum.parse::<i32>().ok()?;
        if maximum < minimum {
            return None;
        }
        let width = u32::try_from(i64::from(maximum) - i64::from(minimum) + 1).ok()?;
        let selected = minimum + i32::try_from(random.next_u32() % width).ok()?;
        let suffix = start + 6 + close + 1;
        sound.replace_range(start..suffix, &selected.to_string());
    }
    let sound = sound
        .strip_suffix(".wav")
        .or_else(|| sound.strip_suffix(".ogg"))
        .unwrap_or(&sound)
        .trim();
    (!sound.is_empty()).then(|| sound.to_owned())
}

#[derive(Clone, Copy, Debug, PartialEq, Component)]
pub(super) struct GameplayActorAudioCursor {
    pub(super) request_serial: u64,
    pub(super) restart_serial: u64,
    pub(super) node: AnimationNodeIndex,
    pub(super) seek_time: f32,
    pub(super) completions: u32,
}

pub(super) fn emit_gameplay_actor_audio_events(
    mut commands: Commands,
    actors: Query<(Entity, &TutorialActor, Option<&GameplayActorAudioCursor>)>,
    players: Query<(&AnimationPlayer, &TutorialActorAnimationPlayback)>,
    mut runtime: ResMut<GameplayAudioRuntime>,
) {
    for (actor_root, actor, cursor) in &actors {
        let Some((active, playback, node)) = players.iter().find_map(|(player, playback)| {
            if playback.actor_root != actor_root || playback.terminally_unavailable {
                return None;
            }
            let node = playback.node?;
            player
                .animation(node)
                .map(|active| (active, playback, node))
        }) else {
            continue;
        };
        let Some(clip) = playback.resolved_clip.or(playback.clip) else {
            continue;
        };
        let Some(audio) = actor_clip_audio(actor.npc_type, clip) else {
            continue;
        };
        debug_assert!(
            audio
                .events
                .iter()
                .all(|event| event.seconds <= audio.duration_seconds)
        );

        let same_playback = cursor.is_some_and(|cursor| {
            cursor.request_serial == playback.request_serial
                && cursor.restart_serial == playback.restart_serial
                && cursor.node == node
                && active.completions() >= cursor.completions
        });
        let (previous_seek, previous_completions) = if same_playback {
            let cursor = cursor.expect("same playback requires an audio cursor");
            (cursor.seek_time, cursor.completions)
        } else {
            (0.0, 0)
        };

        for event in audio.events {
            let crossings = animation_event_crossings(
                previous_seek,
                previous_completions,
                active.seek_time(),
                active.completions(),
                active.repeat_mode(),
                event.seconds,
            );
            for _ in 0..crossings {
                runtime.schedule(actor_root, 0.0, event.cue);
            }
        }

        commands
            .entity(actor_root)
            .insert(GameplayActorAudioCursor {
                request_serial: playback.request_serial,
                restart_serial: playback.restart_serial,
                node,
                seek_time: active.seek_time(),
                completions: active.completions(),
            });
    }
}
