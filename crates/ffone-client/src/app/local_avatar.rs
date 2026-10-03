//! Local avatar traversal/vehicle presentation.

use super::local_inventory::LocalInventoryRuntime;
use super::modal_gates::GameplayModalModels;
use super::option_runtime::{OptionProductionRuntime, option_action_just_pressed};
use super::runtime_status::RuntimeStatus;
use super::state::ClientState;
use super::transportation::TransportationProductionRuntime;
use super::user_equip::{
    LocalVehiclePresentationRuntime, USER_EQUIP_COMPUTER_EFFECT_NAME,
    USER_EQUIP_SPECIAL_STATE_FLAG_0104, UserEquipAvatarPresentationRuntime,
    user_equip_computer_effect,
};
use super::{LocalPlayer, vehicle_transition};
use bevy::prelude::*;
use ffone_client::{
    avatar_action::{
        LegacyAvatarActionState, LegacyAvatarPresentationContext,
        LegacyAvatarTraversalPresentation, LegacyVehiclePresentationFamily,
    },
    gameplay_audio::GameplayAudioRuntime,
    legacy_environment::LegacyAvatarEnvironmentState,
    mission_ui::{MissionJournalUi, MissionUiModel},
    movement::{LegacyInputState, LegacyPlayerController},
    network::{NetworkBridge, NetworkCommand},
    option_ui::LegacyOptionAction,
    tutorial_effects_runtime::{
        TutorialEffectPlacement, TutorialEffectRuntime, TutorialEffectRuntimeCommand,
    },
    tutorial_mission_content::TutorialMissionContent,
    tutorial_player_rig_runtime::TutorialSkywayPresentation,
    user_equip_ui::UserEquipUiState,
    world_audio::RetrobutionInstanceAudioState,
    world_behaviour::{WorldRopeTraversal, WorldSlopeTraversal, WorldZiplineTraversal},
};
use ffone_protocol::{
    PcSpecialStateSwitchRequest0104, PcVehicleOffRequest0104, PcVehicleOnRequest0104, WirePayload,
    packet,
};
use std::time::Instant;

#[must_use]
pub(super) fn rope_traversal_presentation(
    move_type: i64,
    local_axis: Vec2,
) -> LegacyAvatarTraversalPresentation {
    if move_type == 2 {
        if local_axis.x != 0.0 {
            LegacyAvatarTraversalPresentation::RopeTurn
        } else if local_axis.y < 0.0 {
            LegacyAvatarTraversalPresentation::RopeDrop
        } else if local_axis.y > 0.0 {
            LegacyAvatarTraversalPresentation::RopeUp
        } else {
            LegacyAvatarTraversalPresentation::RopeStand2
        }
    } else {
        let direction = if move_type == 0 {
            -local_axis.x
        } else {
            local_axis.x
        };
        if direction < 0.0 {
            LegacyAvatarTraversalPresentation::RopeLeft
        } else if direction > 0.0 {
            LegacyAvatarTraversalPresentation::RopeRight
        } else {
            LegacyAvatarTraversalPresentation::RopeStand1
        }
    }
}

#[must_use]
pub(super) const fn traversal_allows_inventory_pose(traversal: LegacyAvatarTraversalPresentation) -> bool {
    matches!(
        traversal,
        LegacyAvatarTraversalPresentation::None | LegacyAvatarTraversalPresentation::Slope
    )
}

/// Projects authoritative traversal/UI facts into the one local-avatar FSM
/// and drives the exact UserEquip computer AnimationEvent boundary. A public
/// Skyway is a dedicated broomstick traversal. Only confirmed personal
/// board/scooter state selects the vehicle-specific inventory clips and
/// ES833/ES834.
pub(super) fn sync_local_avatar_presentation(
    time: Res<Time>,
    input: Res<LegacyInputState>,
    user_equip_ui: Res<UserEquipUiState>,
    mission_ui: Res<MissionUiModel>,
    vehicle: Res<LocalVehiclePresentationRuntime>,
    inventory: Res<LocalInventoryRuntime>,
    content: Res<TutorialMissionContent>,
    mut vehicle_presentation: ResMut<
        ffone_client::tutorial_player_rig_runtime::PersonalVehiclePresentation,
    >,
    transportation: Res<TransportationProductionRuntime>,
    mut skyway_presentation: ResMut<TutorialSkywayPresentation>,
    bridge: Res<NetworkBridge>,
    mut status: ResMut<RuntimeStatus>,
    mut mode: ResMut<UserEquipAvatarPresentationRuntime>,
    mut audio: ResMut<GameplayAudioRuntime>,
    mut effects: ResMut<TutorialEffectRuntime>,
    mut players: Query<
        (
            Entity,
            &GlobalTransform,
            &LegacyAvatarEnvironmentState,
            &mut LegacyAvatarPresentationContext,
            &mut LegacyAvatarActionState,
            Option<&WorldSlopeTraversal>,
            Option<&WorldZiplineTraversal>,
            Option<&WorldRopeTraversal>,
        ),
        With<LocalPlayer>,
    >,
) {
    skyway_presentation.active = transportation.skyway_active;
    let Ok((player, transform, environment, mut presentation, mut actions, slope, zipline, rope)) =
        players.single_mut()
    else {
        return;
    };

    let user_equip_open = user_equip_ui.is_active();
    if mode.mode_open != user_equip_open {
        audio.queue_user_equip_mode_edge(user_equip_open);
        mode.mode_open = user_equip_open;
    }

    let traversal = if transportation.skyway_active {
        LegacyAvatarTraversalPresentation::BroomStick
    } else if zipline.is_some() {
        LegacyAvatarTraversalPresentation::Zipline
    } else if let Some(rope) = rope {
        rope_traversal_presentation(rope.move_type, input.local_axis)
    } else if slope.is_some() {
        LegacyAvatarTraversalPresentation::Slope
    } else {
        LegacyAvatarTraversalPresentation::None
    };
    presentation.traversal = traversal;
    if presentation.mounted_vehicle != vehicle.family {
        actions.invalidate_visual();
        presentation.mounted_vehicle = vehicle.family;
    }
    let vehicle_item = (vehicle.family != LegacyVehiclePresentationFamily::None)
        .then(|| {
            inventory.snapshot().map(|s| {
                s.equipment()[ffone_protocol::CharacterEquipSlot0104::Vehicle as usize].item_id
            })
        })
        .flatten();
    if vehicle_presentation.item_id != vehicle_item || vehicle_presentation.family != vehicle.family
    {
        vehicle_presentation.item_id = vehicle_item;
        vehicle_presentation.family = vehicle.family;
        vehicle_presentation.engine_sound = vehicle_item
            .and_then(|id| content.gameplay_vehicle_engine_sound(id))
            .map(str::to_owned);
        vehicle_presentation.speed_server_units = vehicle_item
            .and_then(|id| content.gameplay_vehicle_speed(id))
            .unwrap_or(0);
    }
    let journal_open = matches!(mission_ui.journal, MissionJournalUi::Other(_));
    presentation.inventory_open = (user_equip_open || journal_open)
        && !environment.in_water
        && traversal_allows_inventory_pose(traversal);

    let desired_family = presentation
        .inventory_open
        .then_some(presentation.mounted_vehicle);
    if mode.active_family != desired_family {
        if mode.effect_spawned {
            effects.enqueue(TutorialEffectRuntimeCommand::DestroyNamed {
                name: USER_EQUIP_COMPUTER_EFFECT_NAME.to_owned(),
                source_line: line!(),
            });
        }
        mode.active_family = desired_family;
        mode.effect_elapsed_seconds = 0.0;
        mode.effect_spawned = false;
    }

    if let Some(family) = mode.active_family {
        mode.effect_elapsed_seconds += time.delta_secs().max(0.0);
        let (effect_id, event_seconds) = user_equip_computer_effect(family);
        if !mode.effect_spawned && mode.effect_elapsed_seconds >= event_seconds {
            effects.enqueue(TutorialEffectRuntimeCommand::Add {
                effect_id,
                placement: TutorialEffectPlacement::ExactEntityWorld {
                    root_entity: player,
                    position: transform.translation(),
                    rotation: transform.rotation(),
                },
                scale: 1.0,
                tracked: false,
                name: Some(USER_EQUIP_COMPUTER_EFFECT_NAME.to_owned()),
                destroy_after_seconds: None,
                source_line: line!(),
            });
            mode.effect_spawned = true;
        }
    }

    // Water blocks only `SetInvenMotion`; clean `SetInvenMode(true)` still
    // owns CnEquip's mode loop and special-state switch there. Zipline/rope
    // remain the actual special-animation exclusion.
    let desired_special_state = user_equip_open && traversal_allows_inventory_pose(traversal);
    if mode.special_state_active != desired_special_state {
        let result = status
            .player_id
            .ok_or_else(|| "UserEquip computer mode has no authoritative local PC ID".to_owned())
            .and_then(|pc_id| {
                bridge.send(NetworkCommand::SwitchSpecialState(
                    PcSpecialStateSwitchRequest0104 {
                        pc_id,
                        special_state_flag: USER_EQUIP_SPECIAL_STATE_FLAG_0104,
                    },
                ))
            });
        match result {
            Ok(()) => mode.special_state_active = desired_special_state,
            Err(error) => status.message = error,
        }
    }
}

pub(super) fn advance_skyway_traversal(
    time: Res<Time>,
    transportation: Res<TransportationProductionRuntime>,
    mut players: Query<(&mut Transform, &mut LegacyPlayerController), With<LocalPlayer>>,
) {
    if !transportation.skyway_active {
        return;
    }
    let Some(motion) = transportation.skyway_motion else {
        return;
    };
    let Ok((mut transform, mut controller)) = players.single_mut() else {
        return;
    };
    motion.advance(&mut transform, time.delta_secs());
    let forward = transform.rotation * Vec3::NEG_Z;
    controller.yaw_degrees = (-forward.x).atan2(forward.z).to_degrees();
    controller.movement_enabled = false;
}

pub(super) fn reset_local_vehicle_presentation(
    mut vehicle: ResMut<LocalVehiclePresentationRuntime>,
    mut presentation: ResMut<
        ffone_client::tutorial_player_rig_runtime::PersonalVehiclePresentation,
    >,
) {
    *vehicle = LocalVehiclePresentationRuntime::default();
    *presentation = Default::default();
}

pub(super) fn drive_local_vehicle_toggle(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    pad: Option<Res<super::GamepadActionState>>,
    option_runtime: Res<OptionProductionRuntime>,
    inventory: Res<LocalInventoryRuntime>,
    mission_ui: Res<MissionUiModel>,
    modals: GameplayModalModels,
    players: Query<
        (
            &LegacyAvatarEnvironmentState,
            &LegacyAvatarPresentationContext,
        ),
        With<LocalPlayer>,
    >,
    bridge: Res<NetworkBridge>,
    mut status: ResMut<RuntimeStatus>,
    mut vehicle: ResMut<LocalVehiclePresentationRuntime>,
    instance: Res<RetrobutionInstanceAudioState>,
    state: Res<State<ClientState>>,
    mut audio: ResMut<GameplayAudioRuntime>,
) {
    vehicle_transition::expire_pending(&mut vehicle.pending, status.map_number, Instant::now());
    if vehicle.pending.is_some() {
        return;
    }
    let Ok((environment, presentation)) = players.single() else {
        return;
    };
    let mounting = vehicle.family == LegacyVehiclePresentationFamily::None;
    let forced_dismount = !mounting
        && (!traversal_allows_inventory_pose(presentation.traversal)
            || inventory.snapshot().is_none_or(|s| {
                let item = s.equipment()[ffone_protocol::CharacterEquipSlot0104::Vehicle as usize];
                item.item_type != 10 || item.item_id <= 0
            }));
    if !forced_dismount
        && (modals.chat_active(*state.get())
            || mission_ui.gameplay_input_blocked()
            || modals.system_popup()
            || modals.nanocom_popup()
            || modals.buddy_modal()
            || modals.quit_modal()
            || modals.option_modal()
            || modals.resurrect_modal()
            || modals.upsell_modal()
            || modals.guide_modal()
            || modals.game_guide_modal()
            || modals.bank_modal()
            || modals.vendor_modal()
            || modals.rule_modal()
            || modals.nano_free_tuning_modal()
            || modals.user_equip_modal()
            || modals.pc2pc_modal()
            || modals.world_map_modal()
            || modals.transportation_modal()
            || modals.race_modal()
            || modals.email_modal()
            || (modals.combi_modal() || modals.barber_modal())
            || modals.enchant_modal()
            || modals.cashmall_modal()
            || modals.user_store_modal()
            || !(pad.as_ref().is_some_and(|pad| pad.just_pressed(LegacyOptionAction::VehicleToggle))
                || option_action_just_pressed(
                &option_runtime.input,
                LegacyOptionAction::VehicleToggle,
                &keyboard,
                &mouse,
            )))
    {
        return;
    }
    if mounting
        && (instance.active
            || *state.get() == ClientState::Tutorial
            || environment.in_water
            || !traversal_allows_inventory_pose(presentation.traversal))
    {
        audio.queue_gameplay_ui_sound("Action_Failure01");
        status.message = "Vehicle toggle rejected by the clean special-animation gate".to_owned();
        return;
    }

    if mounting {
        let equipped_vehicle = inventory.snapshot().map(|snapshot| {
            snapshot.equipment()[ffone_protocol::CharacterEquipSlot0104::Vehicle as usize]
        });
        if equipped_vehicle.is_none_or(|item| item.item_type != 10 || item.item_id <= 0) {
            audio.queue_gameplay_ui_sound("Action_Failure01");
            status.message = "Vehicle toggle requires an equipped vehicle".to_owned();
            return;
        }
    }
    let send = if mounting {
        ffone_protocol::RegisteredGameplayRequest0104::new(
            packet::P_CL2FE_REQ_PC_VEHICLE_ON,
            PcVehicleOnRequest0104 { unused: 0 }.encode(),
        )
        .map_err(|error| error.to_string())
        .and_then(|request| bridge.send(NetworkCommand::SendRegisteredGameplay0104(request)))
    } else {
        bridge.send(NetworkCommand::VehicleOff(PcVehicleOffRequest0104 {
            unused: 0,
        }))
    };
    match send {
        Ok(()) => {
            vehicle.pending = Some(vehicle_transition::PendingVehicleRequest::new(
                mounting,
                status.map_number,
                Instant::now(),
            ))
        }
        Err(error) => status.message = format!("Vehicle toggle send failed: {error}"),
    }
}
