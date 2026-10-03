//! World combat lifecycle, weapon combat profiles, projectiles and NPC attack visuals.

use super::asset_residency::SharedTutorialEffectLibrary;
use super::local_inventory::LocalInventoryRuntime;
use super::runtime_status::RuntimeStatus;
use super::state::ClientState;
use super::tutorial_combat::{active_world_nano_style, tutorial_named_descendant_position};
use super::{LocalPlayer, movement_buffs};
use bevy::{ecs::system::SystemParam, prelude::*};
use ffone_client::{
    avatar_action::{
        LegacyAttackTarget, LegacyAvatarActionContext, LegacyAvatarActionState,
        LegacyWeaponTargetMode,
    },
    entity_lifecycle::{
        NetworkNpcAppearance0104, NetworkNpcAttackEventQueue0104, NetworkPcAppearance0104,
        NetworkRemotePc0104,
    },
    gameplay_audio::GameplayAudioRuntime,
    inventory_runtime::InventoryLocation0104,
    movement::{LegacyPlayerController, SERVER_TO_CLIENT_SCALE},
    nanocom_message_ui::NanocomMessageUiModel,
    network::{NetworkBridge, NetworkCommand},
    overheat_ui::{LegacyClassWeaponOverheatTable, OverheatUiModel},
    skill_buff_ui::SkillBuffUiModel,
    tutorial_effects_runtime::{
        TutorialEffectLibrary, TutorialEffectRuntime, TutorialEffectRuntimeCommand,
        TutorialProjectileMotion,
    },
    tutorial_mission_content::TutorialMissionContent,
    tutorial_player_presentation::{
        PlayerWeaponAnimationCatalog, PlayerWeaponCombatProfile, TutorialPlayerAnimationRequest,
        TutorialPlayerClip, TutorialPlayerPresentationCommandQueue,
        tutorial_player_gender_from_protocol,
    },
    tutorial_player_rig_runtime::{TutorialSelectedPlayerRig, TutorialSelectedPlayerRigActive},
    user_equip_runtime::UserEquipProductionRuntime0104,
    world_combat::WorldPrimaryAttack0104,
};
use ffone_protocol::{
    AttackResult0104, DecodedFrame, ItemBase0104, ItemMoveRequest0104, NpcCombatPacket0104,
    PcAttackNpcsRequest0104, TimeBuffDotDamageTick0104, WirePayload, decode_npc_combat_packet_0104,
    packet,
};
use std::time::Duration;

pub(super) const WORLD_COMBAT_TIMEOUT_SECONDS: f32 = 5.0;

/// Clean `GameFrame.SendCombatMode` ownership for the ordinary world.
/// Network frames only mark activity; the deterministic update below owns the
/// begin/end edge packets and the five-second inactivity timeout.
#[derive(Debug, Default, Resource)]
pub(super) struct WorldCombatLifecycle {
    pub(super) active: bool,
    pub(super) activity_observed: bool,
    pub(super) inactive_seconds: f32,
}

impl WorldCombatLifecycle {
    pub(super) fn observe(&mut self) {
        self.activity_observed = true;
    }

    pub(super) fn clear(&mut self) {
        *self = Self::default();
    }

    pub(super) fn advance(&mut self, delta_seconds: f32, dead: bool) -> (bool, bool) {
        let mut begin = false;
        if self.activity_observed {
            begin = !self.active;
            self.active = true;
            self.inactive_seconds = 0.0;
            self.activity_observed = false;
        } else if self.active {
            self.inactive_seconds += delta_seconds.max(0.0);
        }

        let end = self.active && (dead || self.inactive_seconds > WORLD_COMBAT_TIMEOUT_SECONDS);
        if end {
            self.active = false;
            self.inactive_seconds = 0.0;
        }
        (begin, end)
    }
}

pub(super) fn frame_observes_local_combat(frame: &DecodedFrame, local_player_id: i32) -> bool {
    // Compatibility divergence: clean GameFrame dispatches infection tick 17
    // without calling SendCombatMode. OpenFusion's ordinary four-second heal
    // path, however, tests only `inCombat` and does not exclude an active
    // infection buff. Renewing the existing five-second lease on each local
    // infection tick prevents server healing from racing the two-second goo
    // damage cadence while retaining the normal CombatEnd edge after exit.
    if frame.packet_type == packet::P_FE2CL_CHAR_TIME_BUFF_TIME_TICK {
        return TimeBuffDotDamageTick0104::decode(&frame.payload).is_ok_and(|tick| {
            tick.character_type == 1
                && tick.character_id == local_player_id
                && tick.time_buff_id == 17
        });
    }
    let Ok(Some(packet)) = decode_npc_combat_packet_0104(frame.packet_type, &frame.payload) else {
        return false;
    };
    match packet {
        NpcCombatPacket0104::PcAttackNpcsSuccess(_)
        | NpcCombatPacket0104::PcAttackCharsSuccess(_) => true,
        NpcCombatPacket0104::NpcAttackPcs(packet) => packet
            .results
            .iter()
            .any(|result| result.id == local_player_id),
        NpcCombatPacket0104::NpcAttackChars(packet) => packet
            .results
            .iter()
            .any(|result| result.entity_type == 1 && result.id == local_player_id),
        NpcCombatPacket0104::PcAttackChars(packet) => packet
            .results
            .iter()
            .any(|result| result.entity_type == 1 && result.id == local_player_id),
        _ => false,
    }
}

pub(super) fn advance_world_combat_lifecycle(
    time: Res<Time>,
    bridge: Res<NetworkBridge>,
    mut runtime: ResMut<RuntimeStatus>,
    mut combat: ResMut<WorldCombatLifecycle>,
) {
    let Some(player_id) = runtime.player_id else {
        combat.clear();
        return;
    };

    let dead = runtime.hp.is_some_and(|hp| hp <= 0);
    let (begin, end) = combat.advance(time.delta_secs(), dead);
    if begin && let Err(error) = bridge.send(NetworkCommand::CombatBegin(player_id)) {
        runtime.message = format!("Combat begin send failed: {error}");
    }
    if end {
        if let Err(error) = bridge.send(NetworkCommand::CombatEnd(player_id)) {
            runtime.message = format!("Combat end send failed: {error}");
        }
    }
}

#[derive(SystemParam)]
pub(super) struct WorldCombatGateInputs<'w> {
    pub(super) skill_buffs: Option<Res<'w, SkillBuffUiModel>>,
    pub(super) lifecycle: Option<Res<'w, WorldCombatLifecycle>>,
    pub(super) overheat_model: Option<Res<'w, OverheatUiModel>>,
    pub(super) overheat_table: Option<Res<'w, LegacyClassWeaponOverheatTable>>,
    pub(super) item_move_owner: Option<Res<'w, UserEquipProductionRuntime0104>>,
}

pub(super) fn authoritative_world_hand_weapon_item_id(inventory: &LocalInventoryRuntime) -> Option<i16> {
    let hand =
        inventory.snapshot()?.equipment()[ffone_protocol::CharacterEquipSlot0104::Hand as usize];
    (hand.item_type == 0 && hand.item_id > 0).then_some(hand.item_id)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum OrdinaryWorldWeaponCombatProfile {
    /// The authoritative inventory has not loaded, the Hand slot is malformed,
    /// or its weapon row has no validated native combat profile yet.
    Unavailable,
    /// The authoritative Hand slot is the protocol's empty weapon item.
    Unarmed,
    Armed(PlayerWeaponCombatProfile),
}

pub(super) fn ordinary_world_weapon_combat_profile(
    inventory: &LocalInventoryRuntime,
    weapon_catalog: &PlayerWeaponAnimationCatalog,
) -> OrdinaryWorldWeaponCombatProfile {
    let Some(snapshot) = inventory.snapshot() else {
        return OrdinaryWorldWeaponCombatProfile::Unavailable;
    };
    let hand = snapshot.equipment()[ffone_protocol::CharacterEquipSlot0104::Hand as usize];
    match (hand.item_type, hand.item_id) {
        (0, 0) => OrdinaryWorldWeaponCombatProfile::Unarmed,
        (0, item_id) if item_id > 0 => weapon_catalog.combat_profile_for_item(item_id).map_or(
            OrdinaryWorldWeaponCombatProfile::Unavailable,
            OrdinaryWorldWeaponCombatProfile::Armed,
        ),
        _ => OrdinaryWorldWeaponCombatProfile::Unavailable,
    }
}

pub(super) fn apply_ordinary_world_weapon_combat_profile(
    context: &mut LegacyAvatarActionContext,
    profile: OrdinaryWorldWeaponCombatProfile,
) {
    let profile = match profile {
        OrdinaryWorldWeaponCombatProfile::Unavailable => {
            // A missing or still-loading hand item is not permission to emit an
            // unthrottled hitscan request. The inventory authority will unlock
            // combat as soon as the exact XDT row becomes available.
            context.attack_locked = true;
            context.attack_half_angle_degrees = 90.0;
            context.attack_range = 0.0;
            context.attack_cooldown_seconds = f32::INFINITY;
            context.target_capacity = 0;
            context.weapon_target_mode = LegacyWeaponTargetMode::Normal;
            return;
        }
        OrdinaryWorldWeaponCombatProfile::Unarmed => {
            // Clean `cnAvatarStatus.UpdateAttribute` retains AvatarTable
            // `m_pAvatarData[1]` when Hand is empty, and
            // `cnAvatarAttack.EquipItems` keeps its base delay: attack
            // angle/range/delay/targets = 90/200/10/1. Distances are protocol
            // centiunits and delay is multiplied by 0.1 seconds.
            context.attack_locked = false;
            context.attack_half_angle_degrees = 90.0;
            context.attack_range = 2.0;
            context.attack_cooldown_seconds = 1.0;
            context.target_capacity = 1;
            context.weapon_target_mode = LegacyWeaponTargetMode::Normal;
            return;
        }
        OrdinaryWorldWeaponCombatProfile::Armed(profile) => profile,
    };
    context.attack_locked = false;
    context.attack_half_angle_degrees = profile.attack_half_angle_degrees;
    context.attack_range = profile.attack_range;
    context.attack_cooldown_seconds = profile.attack_cooldown_seconds;
    context.target_capacity = if profile.target_mode == LegacyWeaponTargetMode::Normal {
        profile
            .target_capacity
            .min(PcAttackNpcsRequest0104::MAX_TARGETS)
    } else {
        profile.target_capacity
    };
    context.weapon_target_mode = profile.target_mode;
}

#[derive(Clone, Copy)]
pub(super) struct TutorialWeaponCombatProfile {
    pub(super) half_angle_degrees: f32,
    pub(super) range: f32,
    pub(super) cooldown_seconds: f32,
    pub(super) target_capacity: usize,
    pub(super) target_mode: LegacyWeaponTargetMode,
    pub(super) bullet_type: Option<i32>,
    pub(super) fire_link: Option<&'static str>,
}

/// Exact tutorial weapon rows from `ItemTable`: melee 43, pistol 197 and
/// sniper 328. Protocol attack ranges are converted from centiunits.
pub(super) fn tutorial_weapon_combat_profile(item_id: Option<i32>) -> TutorialWeaponCombatProfile {
    match item_id {
        Some(43) => TutorialWeaponCombatProfile {
            half_angle_degrees: 90.0,
            range: 3.0,
            cooldown_seconds: 1.0,
            target_capacity: 3,
            target_mode: LegacyWeaponTargetMode::Normal,
            bullet_type: Some(5),
            fire_link: None,
        },
        Some(197) => TutorialWeaponCombatProfile {
            half_angle_degrees: 20.0,
            range: 12.0,
            cooldown_seconds: 0.8,
            target_capacity: 1,
            target_mode: LegacyWeaponTargetMode::Normal,
            bullet_type: Some(113),
            fire_link: Some("Gtag01"),
        },
        Some(328) => TutorialWeaponCombatProfile {
            half_angle_degrees: 20.0,
            range: 16.0,
            cooldown_seconds: 1.0,
            target_capacity: 1,
            target_mode: LegacyWeaponTargetMode::Normal,
            bullet_type: Some(13),
            fire_link: Some("Gtag01"),
        },
        _ => TutorialWeaponCombatProfile {
            half_angle_degrees: 90.0,
            range: 2.0,
            cooldown_seconds: 0.0,
            target_capacity: 1,
            target_mode: LegacyWeaponTargetMode::Normal,
            bullet_type: None,
            fire_link: None,
        },
    }
}

pub(super) fn world_weapon_swap_request(
    equipment: &[ItemBase0104],
    mut supported: impl FnMut(i16) -> bool,
) -> Option<ItemMoveRequest0104> {
    use ffone_protocol::CharacterEquipSlot0104::{Hand, ExtendedHand};
    let hand = *equipment.get(Hand as usize)?;
    let reserve = *equipment.get(ExtendedHand as usize)?;
    let weapon = |item: ItemBase0104| item.item_type == 0 && item.item_id > 0;
    // Only the two equipment slots participate, including holstering a single
    // weapon in ExtendedHand. The inventory is not a weapon-cycle list.
    let (from, to) = if weapon(reserve) && supported(reserve.item_id) {
        (ExtendedHand, Hand)
    } else if reserve.item_id == 0 && weapon(hand) {
        (Hand, ExtendedHand)
    } else {
        return None;
    };
    Some(ItemMoveRequest0104 {
        from_location: InventoryLocation0104::Equipment.wire_value(),
        from_slot_num: from as i32,
        to_location: InventoryLocation0104::Equipment.wire_value(),
        to_slot_num: to as i32,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct WorldWeaponProjectileEndpoint {
    pub(super) target: Vec3,
    pub(super) target_exists: bool,
    pub(super) target_style: i32,
}

/// Returns only the local `MakeBullet` endpoints owned by the clean hitscan
/// branch. Rocket and grenade modes deliberately remain empty: their local
/// presentation belongs to the authoritative warhead response, while this
/// input path only emits the special fire packet.
pub(super) fn world_weapon_projectile_endpoints(
    attack: &WorldPrimaryAttack0104,
    actor_transform: &Transform,
    attack_range: f32,
    targets: &[LegacyAttackTarget],
    mut npc_endpoint: impl FnMut(Entity) -> Option<(i32, Vec3, i32)>,
) -> Vec<WorldWeaponProjectileEndpoint> {
    let WorldPrimaryAttack0104::Hitscan(request) = attack else {
        return Vec::new();
    };
    if request.npc_ids.is_empty() {
        if !actor_transform.translation.is_finite()
            || !actor_transform.rotation.is_finite()
            || !attack_range.is_finite()
            || attack_range < 0.0
        {
            return Vec::new();
        }
        let mut forward = actor_transform.rotation * Vec3::NEG_Z;
        forward.y = 0.0;
        let forward = forward.try_normalize().unwrap_or(Vec3::NEG_Z);
        return vec![WorldWeaponProjectileEndpoint {
            // `cnAvatarAttack.MakeBullet(null)` adds one Unity unit to the
            // forward range endpoint before handing it to BulletContainer.
            target: actor_transform.translation + forward * attack_range + Vec3::Y,
            target_exists: false,
            target_style: -1,
        }];
    }

    let live_targets = targets
        .iter()
        .filter_map(|target| npc_endpoint(target.entity))
        .collect::<Vec<_>>();
    request
        .npc_ids
        .iter()
        .filter_map(|npc_id| {
            live_targets
                .iter()
                .find(|(candidate_id, _, _)| candidate_id == npc_id)
                .map(|(_, target, target_style)| WorldWeaponProjectileEndpoint {
                    target: *target,
                    target_exists: true,
                    target_style: *target_style,
                })
        })
        .collect()
}

pub(super) fn validated_world_weapon_bullet_links(
    library: &TutorialEffectLibrary,
    bullet_type: i32,
) -> Option<(&str, &str)> {
    if !library.contains_bullet(bullet_type) {
        return None;
    }
    library
        .projectile_catalog()
        .rows
        .iter()
        .find(|row| row.bullet_type == bullet_type)
        .map(|row| {
            (
                row.parameters.fire_link.as_str(),
                row.parameters.success_link.as_str(),
            )
        })
}

#[derive(SystemParam)]
pub(super) struct WorldNpcInteractionCombatVisuals<'w, 's> {
    pub(super) nano_notices: ResMut<'w, NanocomMessageUiModel>,
    pub(super) movement_buffs: Res<'w, movement_buffs::MovementBuffs>,
    pub(super) nano_controllers: Query<
        'w,
        's,
        (
            &'static mut LegacyPlayerController,
            Option<&'static movement_buffs::MovementBuffBase>,
        ),
        With<LocalPlayer>,
    >,
    pub(super) weapon_catalog: Res<'w, PlayerWeaponAnimationCatalog>,
    pub(super) effect_library: Res<'w, SharedTutorialEffectLibrary>,
    pub(super) npc_transforms: Query<
        'w,
        's,
        &'static mut Transform,
        (With<NetworkNpcAppearance0104>, Without<LocalPlayer>),
    >,
    pub(super) rigs: Query<'w, 's, &'static TutorialSelectedPlayerRig, With<TutorialSelectedPlayerRigActive>>,
    pub(super) parents: Query<'w, 's, &'static ChildOf>,
    pub(super) named_transforms: Query<'w, 's, (Entity, &'static Name, &'static GlobalTransform)>,
    pub(super) action_states: Query<'w, 's, &'static LegacyAvatarActionState, With<LocalPlayer>>,
    pub(super) nano_npcs: Query<
        'w,
        's,
        (
            Entity,
            &'static NetworkNpcAppearance0104,
            &'static GlobalTransform,
        ),
    >,
    pub(super) nano_players: Query<
        'w,
        's,
        (
            Entity,
            &'static NetworkRemotePc0104,
            &'static NetworkPcAppearance0104,
            &'static GlobalTransform,
        ),
    >,
    pub(super) effect_runtime: ResMut<'w, TutorialEffectRuntime>,
    pub(super) gameplay_audio: ResMut<'w, GameplayAudioRuntime>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn collect_world_npc_attack_visuals(
    mut attacks: ResMut<NetworkNpcAttackEventQueue0104>,
    runtime: Res<RuntimeStatus>,
    content: Res<TutorialMissionContent>,
    npcs: Query<(Entity, &NetworkNpcAppearance0104, &GlobalTransform)>,
    local_players: Query<(Entity, &GlobalTransform), With<LocalPlayer>>,
    remote_players: Query<(
        Entity,
        &NetworkRemotePc0104,
        &NetworkPcAppearance0104,
        &GlobalTransform,
    )>,
    parents: Query<&ChildOf>,
    named_transforms: Query<(Entity, &Name, &GlobalTransform)>,
    mut effects: ResMut<TutorialEffectRuntime>,
) {
    let local = local_players.single().ok();
    for attack in attacks.take_all() {
        let Some((source_entity, source_appearance, source_transform)) = npcs
            .iter()
            .find(|(_, appearance, _)| appearance.0.npc_id == attack.npc_id)
        else {
            continue;
        };
        let Some(definition) = content.gameplay_npc(source_appearance.0.npc_type) else {
            continue;
        };
        if definition.attack_effect <= 0 {
            continue;
        }
        let source = if definition.attack_effect == 66 {
            tutorial_named_descendant_position(source_entity, "tag01", &parents, &named_transforms)
                .unwrap_or_else(|| {
                    source_transform.translation()
                        + Vec3::Y
                            * (definition.height_server_units as f32 * SERVER_TO_CLIENT_SCALE * 0.5)
                })
        } else {
            source_transform.translation()
                + Vec3::Y * (definition.height_server_units as f32 * SERVER_TO_CLIENT_SCALE * 0.5)
        };

        for result in attack
            .results
            .iter()
            .filter(|result| result.entity_type == 1)
        {
            let target = if runtime.player_id == Some(result.id) {
                local.map(|(entity, transform)| {
                    (
                        entity,
                        transform,
                        active_world_nano_style(&runtime, &content),
                    )
                })
            } else {
                remote_players
                    .iter()
                    .find(|(_, remote, _, _)| remote.pc_id == result.id)
                    .map(|(entity, _, appearance, transform)| {
                        let target_style = content
                            .gameplay_nano(appearance.0.nano.id)
                            .map_or(-1, |nano| i32::from(nano.style));
                        (entity, transform, target_style)
                    })
            };
            let Some((target_entity, target_transform, target_style)) = target else {
                continue;
            };
            let target = tutorial_named_descendant_position(
                target_entity,
                "Bip01 Spine1",
                &parents,
                &named_transforms,
            )
            .unwrap_or_else(|| target_transform.translation() + Vec3::Y * 0.8);
            effects.enqueue(TutorialEffectRuntimeCommand::Projectile {
                bullet_type: definition.attack_effect,
                source,
                target,
                target_exists: true,
                source_style: definition.npc_style,
                target_style,
                motion: TutorialProjectileMotion::BulletMove,
                source_line: line!(),
            });
        }
    }
}

pub(super) fn local_player_npc_attack_result(
    results: &[AttackResult0104],
    player_id: i32,
) -> Option<&AttackResult0104> {
    // `NPC_ATTACK_PCS` is already a typed player-target packet family.
    // OpenFusion leaves legacy sAttackResult.eCT zero-initialized, so using
    // entity_type as a second discriminator discards every real local hit.
    results.iter().find(|result| result.id == player_id)
}

pub(super) fn local_player_npc_attack_result_from_frame(
    frame: &DecodedFrame,
    player_id: i32,
) -> Option<AttackResult0104> {
    match decode_npc_combat_packet_0104(frame.packet_type, &frame.payload)
        .ok()
        .flatten()?
    {
        NpcCombatPacket0104::NpcAttackPcs(packet) => {
            local_player_npc_attack_result(&packet.results, player_id).copied()
        }
        // The mixed PvP family does populate eCT; only `1` is a player hit.
        NpcCombatPacket0104::NpcAttackChars(packet) => packet
            .results
            .iter()
            .find(|result| result.entity_type == 1 && result.id == player_id)
            .copied(),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LocalPlayerDamageAudioEdge {
    Damage { critical: bool },
    Death { critical: bool },
}

pub(super) fn local_player_damage_audio_edge(
    previous_hp: Option<i32>,
    current_hp: i32,
    critical: bool,
) -> Option<LocalPlayerDamageAudioEdge> {
    let previous_hp = previous_hp?;
    if previous_hp <= 0 || current_hp >= previous_hp {
        return None;
    }
    Some(if current_hp <= 0 {
        LocalPlayerDamageAudioEdge::Death { critical }
    } else {
        LocalPlayerDamageAudioEdge::Damage { critical }
    })
}

pub(super) fn queue_local_player_damage_audio(
    runtime: &RuntimeStatus,
    player: Entity,
    edge: LocalPlayerDamageAudioEdge,
    audio: &mut GameplayAudioRuntime,
) {
    let Some(protocol_gender) = runtime
        .player_gender
        .and_then(|gender| i8::try_from(gender).ok())
    else {
        return;
    };
    let Ok(gender) = tutorial_player_gender_from_protocol(protocol_gender) else {
        return;
    };
    match edge {
        LocalPlayerDamageAudioEdge::Damage { critical } => {
            audio.queue_player_damage(player, gender, critical);
        }
        LocalPlayerDamageAudioEdge::Death { critical } => {
            audio.queue_player_death(player, gender, critical);
        }
    }
}

pub(super) fn animate_local_player_damage(
    state: Res<State<ClientState>>,
    runtime: Res<RuntimeStatus>,
    mut previous: Local<Option<(i32, i32)>>,
    mut animations: ResMut<TutorialPlayerPresentationCommandQueue>,
) {
    if !matches!(state.get(), ClientState::World | ClientState::Tutorial) {
        *previous = None;
        return;
    }
    let current = runtime.player_id.zip(runtime.hp);
    let damaged = local_player_wound_edge(*previous, current);
    *previous = current;
    if !damaged { return; }
    let Some(gender) = runtime.player_gender.and_then(|g| i8::try_from(g).ok())
        .and_then(|g| tutorial_player_gender_from_protocol(g).ok()) else { return; };
    animations.push_animation(TutorialPlayerAnimationRequest::runtime_cross_fade_with_duration(
        gender, TutorialPlayerClip::WoundUpper, Duration::ZERO,
    ));
}

pub(super) fn local_player_wound_edge(
    previous: Option<(i32, i32)>,
    current: Option<(i32, i32)>,
) -> bool {
    matches!((previous, current),
        (Some((old_id, old_hp)), Some((id, hp))) if old_id == id && hp > 0 && old_hp > hp)
}
