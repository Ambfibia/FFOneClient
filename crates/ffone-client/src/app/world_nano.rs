//! World Nano authority inbox, activation, projection and presentation.

use super::LocalPlayer;
use super::loading_screen::GameplayLoadingState;
use super::mission_indicators::{
    WorldMissionIndicatorRuntime, WorldMissionWaypointRuntime, world_npc_game_icon_name,
    world_quest_symbol_name, world_smart_indicator_name,
};
use super::npc_warp::apply_authoritative_world_teleport;
use super::option_runtime::{OptionProductionRuntime, option_action_just_pressed};
use super::runtime_status::{RuntimeNanoSlot, RuntimeStatus};
use super::user_equip::LocalVehiclePresentationRuntime;
use super::world_map::WorldMapProductionRuntime;
use bevy::prelude::*;
use ffone_client::{
    avatar_action::{LegacyAvatarActionContext, LegacyMoveMode, LegacyVehiclePresentationFamily},
    coordinates::ProtocolPosition,
    gameplay_audio::GameplayAudioRuntime,
    gameplay_nano_portraits::GameplayNanoPortraitCatalog,
    legacy_environment::LegacyAvatarEnvironmentState,
    legacy_world_location::legacy_world_location_name_or_last,
    movement::LegacyPlayerController,
    network::{NetworkBridge, NetworkCommand},
    option_ui::{InputSettings, LegacyOptionAction},
    skill_buff_ui::SkillBuffUiModel,
    tutorial_effects_runtime::{TutorialEffectRuntime, TutorialEffectRuntimeCommand},
    tutorial_mission_content::{GameplaySkillUiDefinition, TutorialMissionContent},
    tutorial_nano_gameplay::{
        TutorialNanoGameplayCommandQueue, TutorialNanoGameplayLoadout, TutorialNanoGameplayState,
        WorldNanoGameplayPresentation,
    },
    world_map::WorldMapPresentation,
    world_nano_authority::{
        WorldNanoAuthoritativeProjection0104, WorldNanoEntityKind0104, WorldNanoMovePostState0104,
        decode_world_nano_authority_0104,
    },
    world_nano_cooldown::WorldNanoCooldownRuntime,
    world_npc_skill_authority::{
        WorldNpcSkillAuthoritativeProjection0104, WorldNpcSkillEntityKind0104,
        decode_world_npc_skill_authority_0104,
    },
};
use ffone_protocol::{
    DecodedFrame, NanoActiveRequest0104, NanoActiveSuccess0104, NanoSkillUseDelivery0104,
    decode_nano_active_success_0104,
};
use std::collections::VecDeque;

/// Fully validated local Nano Move records waiting for the Bevy controller.
/// Scalar authority is committed in network order during polling; remote
/// entity authority stays in the ordered lifecycle ingress.
#[derive(Debug, Default, Resource)]
pub(super) struct WorldNanoAuthorityInbox0104 {
    pub(super) local_movements: VecDeque<WorldNanoMovePostState0104>,
}

impl WorldNanoAuthorityInbox0104 {
    pub(super) fn push_local_movement(&mut self, movement: WorldNanoMovePostState0104) {
        self.local_movements.push_back(movement);
    }

    pub(super) fn take_local_movements(&mut self) -> VecDeque<WorldNanoMovePostState0104> {
        std::mem::take(&mut self.local_movements)
    }

    pub(super) fn clear(&mut self) {
        self.local_movements.clear();
    }
}

pub(super) fn requested_world_nano_slot(
    input: &InputSettings,
    keys: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
) -> Option<usize> {
    [
        LegacyOptionAction::Nano1,
        LegacyOptionAction::Nano2,
        LegacyOptionAction::Nano3,
    ]
    .into_iter()
    .position(|action| option_action_just_pressed(input, action, keys, mouse))
}

/// Exact clean shortcut contract: pressing the already-active slot sends -1
/// to recall the Nano; pressing any other valid slot sends that slot index.
pub(super) fn world_nano_active_request_slot(
    runtime: &RuntimeStatus,
    index: usize,
    maximum_stamina: i16,
) -> Option<i16> {
    let slot = runtime.nano_slots.get(index)?;
    if slot.nano_id.is_none() || slot.skill_id <= 0 {
        return None;
    }
    if slot.active {
        Some(-1)
    } else if maximum_stamina <= 0 || i32::from(slot.stamina) * 5 <= i32::from(maximum_stamina) {
        // cnOwnAvatarStatus requires strictly more than 20% of Battery1.
        // Recall above remains available even after the Nano is exhausted.
        None
    } else {
        i16::try_from(index).ok()
    }
}

pub(super) fn apply_world_nano_active_success(
    success: NanoActiveSuccess0104,
    runtime: &mut RuntimeStatus,
) -> Result<(), String> {
    if success.active_nano_slot == -1 {
        runtime.pending_passive_nano_voice = None;
        for slot in &mut runtime.nano_slots {
            slot.active = false;
        }
        runtime.pending_nano_activation = None;
        return Ok(());
    }
    let index = usize::try_from(success.active_nano_slot).map_err(|_| {
        format!(
            "Nano activation reply has invalid slot {}",
            success.active_nano_slot
        )
    })?;
    let slot = runtime.nano_slots.get(index).copied().ok_or_else(|| {
        format!(
            "Nano activation reply has invalid slot {}",
            success.active_nano_slot
        )
    })?;
    if runtime.pending_nano_activation != Some(index) {
        return Err(format!(
            "uncorrelated Nano activation reply for slot {index}; pending slot is {:?}",
            runtime.pending_nano_activation
        ));
    }
    if slot.nano_id.is_none() || slot.skill_id <= 0 {
        return Err(format!(
            "Nano activation reply selected unavailable or untuned slot {index}"
        ));
    }

    for slot in &mut runtime.nano_slots {
        slot.active = false;
    }
    runtime.nano_slots[index].active = true;
    runtime.pending_passive_nano_voice =
        (success.condition_status_add != 0).then_some(TutorialNanoGameplayLoadout {
            nano_id: slot.nano_id.expect("validated Nano slot"),
            skill_id: slot.skill_id,
        });
    runtime.pending_nano_activation = None;
    Ok(())
}

pub(super) fn validate_world_nano_projection_correlation(
    projection: &WorldNanoAuthoritativeProjection0104,
    runtime: &RuntimeStatus,
) -> Result<(), String> {
    if projection.delivery != NanoSkillUseDelivery0104::LocalSuccess {
        return Ok(());
    }
    if runtime.player_id != Some(projection.caster.pc_id) {
        return Err(format!(
            "Nano skill success belongs to PC {}, local PC is {:?}",
            projection.caster.pc_id, runtime.player_id
        ));
    }
    if !runtime
        .nano_slots
        .iter()
        .any(|slot| slot.nano_id == Some(projection.caster.nano_id))
    {
        return Err(format!(
            "Nano skill success references unequipped Nano {}",
            projection.caster.nano_id
        ));
    }
    Ok(())
}

pub(super) fn apply_world_nano_response_frame(
    frame: &DecodedFrame,
    runtime: &mut RuntimeStatus,
    skill_buffs: &mut SkillBuffUiModel,
    inbox: &mut WorldNanoAuthorityInbox0104,
) -> Result<bool, String> {
    if let Some(success) = decode_nano_active_success_0104(frame.packet_type, &frame.payload)
        .map_err(|error| format!("malformed protocol-0104 Nano activation reply: {error}"))?
    {
        apply_world_nano_active_success(success, runtime)?;
        return Ok(true);
    }
    if let Some(projection) = decode_world_nano_authority_0104(frame.packet_type, &frame.payload)
        .map_err(|error| format!("malformed protocol-0104 Nano skill result: {error}"))?
    {
        validate_world_nano_projection_correlation(&projection, runtime)?;
        if let Some(movement) =
            apply_world_nano_projection_to_local_status(&projection, runtime, skill_buffs)
        {
            inbox.push_local_movement(movement);
        }
        return Ok(true);
    }
    Ok(false)
}

pub(super) fn apply_world_nano_projection_to_local_status(
    projection: &WorldNanoAuthoritativeProjection0104,
    runtime: &mut RuntimeStatus,
    skill_buffs: &mut SkillBuffUiModel,
) -> Option<WorldNanoMovePostState0104> {
    let local_player_id = runtime.player_id?;

    if projection.caster.pc_id == local_player_id {
        for slot in &mut runtime.nano_slots {
            if slot.nano_id == Some(projection.caster.nano_id) {
                slot.skill_id = projection.caster.skill_id;
                slot.stamina = projection.caster.nano_stamina;
            }
        }
        if projection.caster.nano_deactivated {
            for slot in &mut runtime.nano_slots {
                slot.active = false;
            }
        }
        if let Some(hp) = projection.caster.absolute_hp {
            runtime.hp = Some(hp);
        }
    }

    let mut movement = None;
    for target in &projection.targets {
        if target.target.kind != WorldNanoEntityKind0104::Player
            || target.target.id != local_player_id
        {
            continue;
        }
        if let Some(hp) = target.absolute_hp {
            runtime.hp = Some(hp);
        }
        if let Some(condition_bit_flag) = target.absolute_condition_bit_flag {
            skill_buffs.local_condition_bit_flag = condition_bit_flag as u32;
        }
        if let Some(weapon_battery) = target.absolute_weapon_battery {
            runtime.weapon_battery = weapon_battery;
        }
        if let Some(nano_battery) = target.absolute_nano_battery {
            runtime.nano_battery = nano_battery;
        }
        if let Some(nano_stamina) = target.absolute_nano_stamina
            && let Some(slot) = runtime.nano_slots.iter_mut().find(|slot| slot.active)
        {
            slot.stamina = nano_stamina;
        }
        if target.nano_deactivated == Some(true) {
            for slot in &mut runtime.nano_slots {
                slot.active = false;
            }
        }
        if target.movement.is_some() {
            movement = target.movement;
        }
    }
    movement
}

pub(super) fn apply_world_npc_skill_projection_to_local_status(
    projection: &WorldNpcSkillAuthoritativeProjection0104,
    runtime: &mut RuntimeStatus,
    skill_buffs: &mut SkillBuffUiModel,
) -> Option<WorldNanoMovePostState0104> {
    let local_player_id = runtime.player_id?;
    let mut movement = None;
    for target in &projection.targets {
        if target.target.kind != WorldNpcSkillEntityKind0104::Player
            || target.target.id != local_player_id
        {
            continue;
        }
        if let Some(hp) = target.absolute_hp {
            // Corruption damage is intentionally signed and may pass below
            // zero before the regeneration mode takes ownership.
            runtime.hp = Some(hp);
        }
        if let Some(condition_bit_flag) = target.absolute_condition_bit_flag {
            skill_buffs.local_condition_bit_flag = condition_bit_flag as u32;
        }
        if let Some(weapon_battery) = target.absolute_weapon_battery {
            runtime.weapon_battery = weapon_battery;
        }
        if let Some(nano_battery) = target.absolute_nano_battery {
            runtime.nano_battery = nano_battery;
        }

        let authoritative_slot = target
            .absolute_active_nano_slot
            .and_then(|slot| usize::try_from(slot).ok())
            .filter(|slot| *slot < runtime.nano_slots.len());
        if target.absolute_active_nano_slot == Some(-1) {
            for slot in &mut runtime.nano_slots {
                slot.active = false;
            }
        } else if let Some(index) = authoritative_slot {
            for slot in &mut runtime.nano_slots {
                slot.active = false;
            }
            let slot = &mut runtime.nano_slots[index];
            if let Some(nano_id) = target.absolute_nano_id.filter(|nano_id| *nano_id > 0) {
                if slot.nano_id != Some(nano_id) {
                    slot.nano_id = Some(nano_id);
                    // Corruption tails do not serialize the tune skill. Never
                    // attach the previous occupant's skill to a new Nano ID.
                    slot.skill_id = 0;
                }
            }
            if let Some(stamina) = target.absolute_nano_stamina {
                slot.stamina = stamina;
            }
            slot.active = target.nano_deactivated != Some(true) && slot.nano_id.is_some();
        } else if let Some(stamina) = target.absolute_nano_stamina
            && let Some(slot) = runtime.nano_slots.iter_mut().find(|slot| slot.active)
        {
            slot.stamina = stamina;
        }
        if target.nano_deactivated == Some(true) {
            for slot in &mut runtime.nano_slots {
                slot.active = false;
            }
        }
        if let Some(target_movement) = target.movement {
            movement = Some(WorldNanoMovePostState0104 {
                map_number: target_movement.map_number,
                position: target_movement.position,
            });
        }
    }
    movement
}

pub(super) fn apply_world_npc_skill_response_frame(
    frame: &DecodedFrame,
    runtime: &mut RuntimeStatus,
    skill_buffs: &mut SkillBuffUiModel,
    inbox: &mut WorldNanoAuthorityInbox0104,
) -> Result<bool, String> {
    let Some(projection) = decode_world_npc_skill_authority_0104(frame.packet_type, &frame.payload)
        .map_err(|error| format!("malformed protocol-0104 NPC skill result: {error}"))?
    else {
        return Ok(false);
    };
    if let Some(movement) =
        apply_world_npc_skill_projection_to_local_status(&projection, runtime, skill_buffs)
    {
        inbox.push_local_movement(movement);
    }
    Ok(true)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_world_nano_authority_inbox(
    mut commands: Commands,
    mut inbox: ResMut<WorldNanoAuthorityInbox0104>,
    mut runtime: ResMut<RuntimeStatus>,
    mut world_map: ResMut<WorldMapPresentation>,
    mut map_production: ResMut<WorldMapProductionRuntime>,
    mut loading: ResMut<GameplayLoadingState>,
    mut local_players: Query<
        (
            Entity,
            &mut Transform,
            &mut LegacyPlayerController,
            &mut Visibility,
            &mut LegacyAvatarEnvironmentState,
        ),
        With<LocalPlayer>,
    >,
) {
    for movement in inbox.take_local_movements() {
        let map_changed = runtime.map_number != Some(movement.map_number);
        runtime.map_number = Some(movement.map_number);
        if map_changed {
            world_map.reset_session();
            map_production.server_npcs = default();
            map_production.observed_npcs.clear();
            map_production.combined_npc_sources.clear();
            map_production.npc_sources_dirty = true;
            map_production.snapshot_retry_seconds = 1.1;
        }
        let position = ProtocolPosition::new(movement.position).to_native();
        runtime.map_name =
            legacy_world_location_name_or_last(&runtime.map_name, -position.x, position.z)
                .map(str::to_owned)
                .unwrap_or_else(|| format!("Map {}", movement.map_number));
        match local_players.single_mut() {
            Ok((player, mut transform, mut controller, mut visibility, mut environment)) => {
                if map_changed {
                    // A group-target Move tail can cross maps even though
                    // OpenFusion does not emit a second GOTO packet. Match
                    // the normal warp path so chunks and collision reload.
                    apply_authoritative_world_teleport(
                        &mut commands,
                        &mut loading,
                        player,
                        &mut transform,
                        &mut controller,
                        &mut visibility,
                        &mut environment,
                        position,
                        runtime.hp,
                    );
                } else {
                    // Same-map group Recall needs only an authoritative
                    // controller snap; hiding the avatar and rebuilding the
                    // entire streamed slice would introduce a fake load.
                    transform.translation = position;
                    controller.apply_authoritative_teleport(position);
                    environment.last_observed_hp = runtime.hp;
                }
            }
            Err(error) => {
                runtime.message =
                    format!("Nano Move has no unique local player to reposition: {error}");
            }
        }
    }
}

/// Ports the tutorial's proven 1/2/3 Nano activation path to the shard world.
/// Status is committed only by `P_FE2CL_REP_NANO_ACTIVE_SUCC`: OpenFusion has
/// no failure packet and may silently reject a stale or untuned slot.
pub(super) fn activate_world_nano_from_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    options: Res<OptionProductionRuntime>,
    pad: Option<Res<super::GamepadActionState>>,
    vehicle: Res<LocalVehiclePresentationRuntime>,
    bridge: Res<NetworkBridge>,
    content: Res<TutorialMissionContent>,
    mut audio: ResMut<GameplayAudioRuntime>,
    players: Query<&LegacyAvatarActionContext, With<LocalPlayer>>,
    mut runtime: ResMut<RuntimeStatus>,
) {
    let Some(index) = requested_world_nano_slot(&options.input, &keys, &mouse).or_else(|| {
        pad.as_ref().and_then(|pad| [LegacyOptionAction::Nano1, LegacyOptionAction::Nano2, LegacyOptionAction::Nano3]
            .into_iter().position(|action| pad.just_pressed(action)))
    }) else {
        return;
    };
    let Ok(context) = players.single() else {
        return;
    };
    if !context.ready_for_play
        || !context.input_enabled
        || context.system_popup
        || context.nano_locked
        || context.vehicle_mounted
        || vehicle.family != LegacyVehiclePresentationFamily::None
        || context.move_mode != LegacyMoveMode::None
    {
        return;
    }
    let maximum_stamina = runtime.nano_slots[index]
        .nano_id
        .and_then(|id| content.gameplay_nano(id))
        .map_or(0, |nano| nano.max_stamina);
    let Some(nano_slot) = world_nano_active_request_slot(&runtime, index, maximum_stamina) else {
        let slot = runtime.nano_slots[index];
        if slot.nano_id.is_some() && slot.skill_id > 0 && !slot.active && maximum_stamina > 0 {
            audio.queue_gameplay_ui_sound("Nano_FailSummon");
        }
        return;
    };
    if let Err(error) = bridge.send(NetworkCommand::ActivateNano(NanoActiveRequest0104 {
        nano_slot,
    })) {
        runtime.message = format!("Nano activation send failed: {error}");
        return;
    }
    // Keep the pressed index for both activation and recall. The authoritative
    // -1 reply clears this pending request in `apply_world_nano_active_success`.
    runtime.pending_nano_activation = Some(index);
}

pub(super) fn active_world_nano_slot(runtime: &RuntimeStatus) -> Option<(usize, RuntimeNanoSlot)> {
    runtime
        .nano_slots
        .iter()
        .copied()
        .enumerate()
        .find(|(_, slot)| slot.active && slot.nano_id.is_some() && slot.skill_id > 0)
}

pub(super) fn world_nano_skill_slot(
    content: &TutorialMissionContent,
    nano_id: i16,
    skill_id: i16,
) -> Option<u8> {
    content
        .journal_nano(i32::from(nano_id))?
        .skills
        .iter()
        .position(|skill| skill.skill_id == i32::from(skill_id))
        .and_then(|index| u8::try_from(index + 1).ok())
}

pub(super) fn ordinary_world_nano_skill_supported(skill: &GameplaySkillUiDefinition) -> bool {
    skill.active
        && skill.effect_type == 1
        && match skill.target {
            1 => matches!(skill.target_type, 1 | 2) && skill.range >= 0,
            2 => true,
            3 => {
                matches!(skill.target_type, 1 | 2)
                    && skill.range >= 0
                    && (0..=180).contains(&skill.angle)
            }
            5 => matches!(skill.target_type, 1..=3) && skill.area >= 0,
            6 => matches!(skill.target_type, 1 | 2) && skill.area >= 0,
            _ => false,
        }
        && skill.target_number > 0
        && skill.cooldown >= 0
}

pub(super) fn advance_world_nano_cooldowns(
    time: Res<Time>,
    runtime: Res<RuntimeStatus>,
    mut cooldowns: ResMut<WorldNanoCooldownRuntime>,
) {
    cooldowns.retain_equipped(
        runtime
            .nano_slots
            .map(|slot| (slot.nano_id.is_some() && slot.skill_id > 0).then_some(slot.skill_id)),
    );
    cooldowns.advance(time.delta_secs());
}

/// Presents every validated native Nano model without making presentation a
/// prerequisite for the server-authoritative normal-world skill request.
pub(super) fn sync_world_nano_presentation(
    mut runtime: ResMut<RuntimeStatus>,
    content: Res<TutorialMissionContent>,
    catalog: Res<GameplayNanoPortraitCatalog>,
    gameplay: Res<TutorialNanoGameplayState>,
    players: Query<Entity, With<LocalPlayer>>,
    mut commands: ResMut<TutorialNanoGameplayCommandQueue>,
) {
    let Some((_, slot)) = active_world_nano_slot(&runtime) else {
        if gameplay.entity().is_some() {
            commands.dismiss();
        }
        return;
    };
    let Some(nano_id) = slot.nano_id else {
        return;
    };
    if slot.stamina <= 0 {
        commands.withdraw();
        return;
    }
    let Some(nano) = content.gameplay_nano(nano_id) else {
        if gameplay.entity().is_some() {
            commands.hide();
        }
        return;
    };
    let Some(model_path) = catalog.model_path(nano_id) else {
        // Missing presentation is explicitly fail-closed, but it never
        // disables the separately planned protocol skill request.
        if gameplay.entity().is_some() {
            commands.hide();
        }
        return;
    };
    let Some(skill_slot) = world_nano_skill_slot(&content, nano_id, slot.skill_id) else {
        if gameplay.entity().is_some() {
            commands.hide();
        }
        return;
    };
    let Ok(owner) = players.single() else {
        return;
    };
    let loadout = TutorialNanoGameplayLoadout {
        nano_id,
        skill_id: slot.skill_id,
    };
    let presentation = WorldNanoGameplayPresentation {
        model_path: model_path.to_owned(),
        style: nano.style,
        skill_slot,
    };
    let replacing =
        gameplay.loadout() != Some(loadout) || gameplay.world_presentation() != Some(&presentation);
    if replacing || gameplay.stamina() != i32::from(slot.stamina) {
        commands.equip_world(
            loadout.nano_id,
            loadout.skill_id,
            i32::from(slot.stamina),
            presentation,
        );
    }
    if replacing || gameplay.entity().is_none() || gameplay.owner() != Some(owner) {
        commands.summon(owner);
    }
    if runtime.pending_passive_nano_voice == Some(loadout) {
        commands.play_world_skill(owner);
        runtime.pending_passive_nano_voice = None;
    }
}

pub(super) fn reset_world_nano_and_mission_presentation(
    mut nano_cooldowns: ResMut<WorldNanoCooldownRuntime>,
    mut nano_authority: ResMut<WorldNanoAuthorityInbox0104>,
    mut waypoint: ResMut<WorldMissionWaypointRuntime>,
    mut indicators: ResMut<WorldMissionIndicatorRuntime>,
    mut effects: ResMut<TutorialEffectRuntime>,
) {
    for npc_id in indicators.smart_indicators.iter().copied() {
        effects.enqueue(TutorialEffectRuntimeCommand::DestroyNamed {
            name: world_smart_indicator_name(npc_id),
            source_line: 2397,
        });
    }
    for npc_id in indicators.mission_symbols.keys().copied() {
        effects.enqueue(TutorialEffectRuntimeCommand::DestroyNamed {
            name: world_quest_symbol_name(npc_id),
            source_line: 3225,
        });
    }
    for npc_id in indicators.game_icons.keys().copied() {
        effects.enqueue(TutorialEffectRuntimeCommand::DestroyNamed {
            name: world_npc_game_icon_name(npc_id),
            source_line: 187,
        });
    }
    nano_cooldowns.clear();
    nano_authority.clear();
    *waypoint = WorldMissionWaypointRuntime::default();
    *indicators = WorldMissionIndicatorRuntime::default();
}
