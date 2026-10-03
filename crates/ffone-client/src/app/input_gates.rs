//! Tutorial input/action gates, NPC subtarget camera and gameplay cursor.

use super::LocalPlayer;
use super::guide::GuideProductionRuntime;
use super::local_inventory::LocalInventoryRuntime;
use super::modal_gates::GameplayModalModels;
use super::race::RaceProductionRuntime;
use super::runtime_status::RuntimeStatus;
use super::state::ClientState;
use super::transportation::TransportationProductionRuntime;
use super::tutorial_session::{TutorialLogicRuntime, TutorialSession};
use super::tutorial_startup::tutorial_actor_needs_early_modal_lock;
use super::world_combat::{
    WorldCombatGateInputs, apply_ordinary_world_weapon_combat_profile,
    ordinary_world_weapon_combat_profile, tutorial_weapon_combat_profile,
};
use super::world_nano::{active_world_nano_slot, ordinary_world_nano_skill_supported};
use bevy::{
    prelude::*,
    window::{CursorOptions, PrimaryWindow},
};
use ffone_client::{
    avatar_action::{
        LegacyAvatarActionContext, LegacyAvatarPresentationContext, LegacyMoveMode,
        LegacyNanoTargetPolicy, LegacyVehiclePresentationFamily, LegacyWeaponTargetMode,
    },
    bank_runtime::BankProductionRuntime0104,
    entity_lifecycle::NetworkNpcAppearance0104,
    legacy_environment::LegacyAvatarEnvironmentState,
    mission_ui::{MissionUiModel, TUTORIAL_EXIT_KEY, TutorialExitDialogGate},
    movement::{
        LegacyInputGate, LegacyOrbitCamera, LegacyPlayerController, LegacyWorldColliderPending,
        SERVER_TO_CLIENT_SCALE,
    },
    rule_runtime::RuleRuntime,
    transportation_ui::{TransportationModel, TransportationPhase, TransportationTarget},
    tutorial::TutorialInputLock,
    tutorial_actors::TutorialActor,
    tutorial_choreography_runtime::TutorialChoreographyPresentation,
    tutorial_mission_content::TutorialMissionContent,
    tutorial_nano_gameplay::{
        TUTORIAL_BUTTERCUP_SKILL_RANGE, TUTORIAL_BUTTERCUP_SKILL_TARGET_CAPACITY,
        TutorialNanoGameplayState,
    },
    tutorial_native_mechanics::TutorialNativeMechanics,
    tutorial_player_presentation::PlayerWeaponAnimationCatalog,
    vendor_runtime::VendorProductionRuntime0104,
    world_behaviour::{WorldRopeTraversal, WorldSlopeTraversal, WorldZiplineTraversal},
    world_nano_cooldown::WorldNanoCooldownRuntime,
};
use ffone_protocol::NanoSkillUseRequest0104;

pub(super) fn sync_tutorial_input_gate(
    state: Res<State<ClientState>>,
    tutorial: Res<TutorialSession>,
    native: Res<TutorialNativeMechanics>,
    presentation: Res<TutorialChoreographyPresentation>,
    model: Res<MissionUiModel>,
    modals: GameplayModalModels,
    actors: Query<&TutorialActor>,
    mut players: Query<&mut LegacyPlayerController, With<LocalPlayer>>,
    mut gate: ResMut<LegacyInputGate>,
) {
    let tutorial_active = *state.get() == ClientState::Tutorial;
    let world_active = *state.get() == ClientState::World;
    let chat_active = modals.chat_active(*state.get());
    let system_popup = modals.system_popup();
    let nanocom_popup = modals.nanocom_popup();
    let buddy_modal = modals.buddy_modal();
    let quit_modal = modals.quit_modal();
    let option_modal = modals.option_modal();
    let resurrect_modal = modals.resurrect_modal();
    let upsell_modal = modals.upsell_modal();
    let guide_modal = modals.guide_modal() || modals.game_guide_modal();
    let bank_modal = modals.bank_modal();
    let vendor_modal = modals.vendor_modal();
    let rule_modal = modals.rule_modal();
    let nano_free_tuning_modal = modals.nano_free_tuning_modal();
    let user_equip_modal = modals.user_equip_modal();
    let pc2pc_modal = modals.pc2pc_modal();
    let world_map_modal = modals.world_map_modal();
    let transportation_modal = modals.transportation_world_owned();
    let race_modal = modals.race_modal();
    let email_modal = modals.email_modal();
    let combi_modal = modals.combi_modal() || modals.barber_modal();
    let enchant_modal = modals.enchant_modal();
    let cashmall_modal = modals.cashmall_modal();
    let user_store_modal = modals.user_store_modal();
    if tutorial_active
        || (world_active && model.stops_auto_run())
        || world_map_modal
        || transportation_modal
        || race_modal
        || email_modal
        || combi_modal
        || enchant_modal
        || cashmall_modal
        || user_store_modal
        || bank_modal
        || vendor_modal
        || rule_modal
        || nano_free_tuning_modal
        || user_equip_modal
        || pc2pc_modal
        || option_modal
        || resurrect_modal
    {
        // Retrobution suppresses Home for the entire tutorial and clears an
        // existing AutoRun as soon as MainGame yields to a modal mode.
        // Clearing it before ReadInput prevents the controller's zero-axis
        // fallback from moving through a blocked mode, including mission
        // journal and NPC conversations owned by MissionUiModel.
        for mut controller in &mut players {
            controller.set_auto_run(false);
        }
    }
    let early_npc_modal = tutorial_active
        && actors
            .iter()
            .any(|actor| tutorial_actor_needs_early_modal_lock(&model, actor));
    let world_modal = world_active
        && (chat_active
            || buddy_modal
            || model.gameplay_input_blocked()
            || system_popup
            || nanocom_popup
            || quit_modal
            || option_modal
            || resurrect_modal
            || upsell_modal
            || guide_modal
            || bank_modal
            || vendor_modal
            || rule_modal
            || nano_free_tuning_modal
            || user_equip_modal
            || pc2pc_modal
            || world_map_modal
            || transportation_modal
            || race_modal
            || email_modal
            || combi_modal
            || enchant_modal
            || cashmall_modal
            || user_store_modal);
    let next = if world_modal {
        LegacyInputGate {
            allow_forward: false,
            allow_backward: false,
            allow_strafe: false,
            allow_keyboard_turning: false,
            allow_jump: false,
            allow_mouse_camera: false,
        }
    } else if !tutorial_active {
        LegacyInputGate::default()
    } else if tutorial.completion_requested
        || presentation.movement_locked
        || model.gameplay_input_blocked()
        || buddy_modal
        || system_popup
        || nanocom_popup
        || quit_modal
        || option_modal
        || resurrect_modal
        || upsell_modal
        || guide_modal
        || bank_modal
        || vendor_modal
        || rule_modal
        || nano_free_tuning_modal
        || user_equip_modal
        || world_map_modal
        || transportation_modal
        || race_modal
        || combi_modal
        || enchant_modal
        || cashmall_modal
        || user_store_modal
        || early_npc_modal
    {
        LegacyInputGate {
            allow_forward: false,
            allow_backward: false,
            allow_strafe: false,
            allow_keyboard_turning: false,
            allow_jump: false,
            allow_mouse_camera: false,
        }
    } else {
        LegacyInputGate {
            allow_forward: !native.is_locked(TutorialInputLock::Front),
            allow_backward: !native.is_locked(TutorialInputLock::Back),
            allow_strafe: !native.is_locked(TutorialInputLock::Side),
            allow_keyboard_turning: !native.is_locked(TutorialInputLock::Turn),
            allow_jump: !native.is_locked(TutorialInputLock::Jump),
            allow_mouse_camera: !native.is_locked(TutorialInputLock::Mouse),
        }
    };
    if *gate != next {
        *gate = next;
    }
}

/// Exact native projection of `cnPlayerCamera.SubTargetCamera`.
///
/// Retrobution stores `npc.rotation * Quaternion.Euler(0, 180, 0)` as the
/// sub-target angle, places the camera at `target - angle * forward * radius*3`,
/// and looks at 60% of the NPC table height. Under the shared Unity-to-native
/// reflection, that angle's forward vector is the actor root's local `+Z`.
pub(super) fn tutorial_npc_subtarget_pose(
    target_position: Vec3,
    target_rotation: Quat,
    radius: f32,
    height: f32,
) -> (Vec3, Vec3) {
    let vertical_focus = Vec3::Y * (height * 0.6);
    let position = target_position - (target_rotation * Vec3::Z) * (radius * 3.0) + vertical_focus;
    (position, target_position + vertical_focus)
}

pub(super) fn world_npc_subtarget_pose(
    grounded_root_position: Vec3,
    target_rotation: Quat,
    radius_server_units: i32,
    height_server_units: i32,
) -> (Vec3, Vec3) {
    tutorial_npc_subtarget_pose(
        grounded_root_position,
        target_rotation,
        radius_server_units as f32 * SERVER_TO_CLIENT_SCALE,
        height_server_units as f32 * SERVER_TO_CLIENT_SCALE,
    )
}

pub(super) fn apply_tutorial_npc_subtarget_camera(
    model: Res<MissionUiModel>,
    content: Res<TutorialMissionContent>,
    actors: Query<(&TutorialActor, &GlobalTransform)>,
    mut cameras: Query<&mut Transform, With<LegacyOrbitCamera>>,
) {
    let Some(actor_id) = model.npc_subtarget_actor_id() else {
        return;
    };
    let Some((actor, actor_transform)) = actors
        .iter()
        .find(|(actor, _)| actor.id == actor_id && actor.is_alive())
    else {
        return;
    };
    let Some(definition) = content.gameplay_npc(actor.npc_type) else {
        return;
    };
    let (position, focus) = tutorial_npc_subtarget_pose(
        actor_transform.translation(),
        actor_transform.rotation(),
        definition.radius(),
        definition.height(),
    );
    for mut camera in &mut cameras {
        camera.translation = position;
        camera.look_at(focus, Vec3::Y);
    }
}

pub(super) fn apply_world_npc_subtarget_camera(
    model: Res<MissionUiModel>,
    production: Res<GuideProductionRuntime>,
    bank_production: Res<BankProductionRuntime0104>,
    vendor_production: Res<VendorProductionRuntime0104>,
    rule_runtime: Res<RuleRuntime>,
    transportation: Res<TransportationModel>,
    transportation_production: Res<TransportationProductionRuntime>,
    race_production: Res<RaceProductionRuntime>,
    content: Res<TutorialMissionContent>,
    npcs: Query<(&NetworkNpcAppearance0104, &GlobalTransform)>,
    mut cameras: Query<&mut Transform, With<LegacyOrbitCamera>>,
) {
    let target_npc_id = bank_production
        .session()
        .map(|session| session.source().npc_id)
        .or_else(|| bank_production.opening().map(|source| source.npc_id))
        .or_else(|| {
            vendor_production
                .active_source_npc()
                .map(|source| source.runtime_npc_id)
        })
        .or_else(|| rule_runtime.camera_sub_target_npc_id())
        .or_else(|| {
            transportation_production
                .camera_subtarget_active
                .then(|| match transportation.target() {
                    Some(TransportationTarget::Npc {
                        npc_instance_id, ..
                    }) if transportation.phase() != TransportationPhase::Hidden => {
                        Some(npc_instance_id)
                    }
                    _ => None,
                })
                .flatten()
        })
        .or_else(|| race_production.camera_subtarget_npc_id())
        .or_else(|| {
            production
                .active_source_npc
                .map(|source| source.runtime_npc_id)
        })
        .or_else(|| {
            model
                .npc_icon_mode_visible
                .then(|| {
                    model
                        .npc_interaction
                        .as_ref()
                        .map(|interaction| interaction.npc_id)
                })
                .flatten()
        });
    let Some(target_npc_id) = target_npc_id else {
        return;
    };
    let Some((appearance, transform)) = npcs
        .iter()
        .find(|(appearance, _)| appearance.0.npc_id == target_npc_id)
    else {
        return;
    };
    let Some(definition) = content.gameplay_npc(appearance.0.npc_type) else {
        return;
    };
    let (position, focus) = world_npc_subtarget_pose(
        transform.translation(),
        transform.rotation(),
        definition.radius_server_units,
        definition.height_server_units,
    );
    for mut camera in &mut cameras {
        camera.translation = position;
        camera.look_at(focus, Vec3::Y);
    }
}

pub(super) const fn legacy_gameplay_cursor_locked(
    state: ClientState,
    gameplay_ready: bool,
    pointer_ui_active: bool,
) -> bool {
    matches!(state, ClientState::Tutorial | ClientState::World)
        && gameplay_ready
        && !pointer_ui_active
}

/// Reproduce `cnMainGame.Start/Update`: gameplay uses a hidden, locked cursor
/// for relative camera input, while menu, journal and system-dialog surfaces
/// restore the visible pointer. The local player readiness check corresponds
/// to the original `GameFrame.IsReadyForPlay()` gate.
pub(super) fn sync_legacy_gameplay_cursor(
    state: Res<State<ClientState>>,
    tutorial: Res<TutorialSession>,
    model: Res<MissionUiModel>,
    modals: GameplayModalModels,
    actors: Query<&TutorialActor>,
    ready_players: Query<(), (With<LocalPlayer>, Without<LegacyWorldColliderPending>)>,
    focus: Res<ffone_client::input_focus::GameInputFocus>,
    mut cursor_options: Query<(&mut Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    let chat_active = modals.chat_active(*state.get());
    let pointer_ui_active = tutorial.completion_requested
        || model.gameplay_input_blocked()
        || chat_active
        || modals.buddy_modal()
        || modals.system_popup()
        || modals.nanocom_popup()
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
        || actors
            .iter()
            .any(|actor| tutorial_actor_needs_early_modal_lock(&model, actor));
    let locked = legacy_gameplay_cursor_locked(
        *state.get(),
        ready_players.single().is_ok(),
        pointer_ui_active,
    );
    let Ok((mut window, mut cursor)) = cursor_options.single_mut() else {
        return;
    };
    let locked = locked && window.focused && !focus.suppressed;
    ffone_client::input_focus::apply_gameplay_cursor(
        &mut window,
        &mut cursor,
        locked,
        focus.regained,
    );
}

pub(super) fn sync_tutorial_action_gate(
    state: Res<State<ClientState>>,
    tutorial: Res<TutorialSession>,
    native: Res<TutorialNativeMechanics>,
    gameplay_nano: Res<TutorialNanoGameplayState>,
    model: Res<MissionUiModel>,
    modals: GameplayModalModels,
    runtime: Res<RuntimeStatus>,
    combat_gate: WorldCombatGateInputs,
    world_nano_cooldowns: Res<WorldNanoCooldownRuntime>,
    inventory: Res<LocalInventoryRuntime>,
    content: Res<TutorialMissionContent>,
    actors: Query<&TutorialActor>,
    tutorial_presentation: Option<Res<TutorialChoreographyPresentation>>,
    weapon_catalog: Res<PlayerWeaponAnimationCatalog>,
    transportation: Option<Res<TransportationProductionRuntime>>,
    mut players: Query<
        (
            &mut LegacyAvatarActionContext,
            Option<&LegacyAvatarPresentationContext>,
            Option<&LegacyWorldColliderPending>,
            Option<&LegacyAvatarEnvironmentState>,
            Has<WorldZiplineTraversal>,
            Has<WorldRopeTraversal>,
            Has<WorldSlopeTraversal>,
        ),
        With<LocalPlayer>,
    >,
) {
    let tutorial_active = *state.get() == ClientState::Tutorial;
    let system_popup = model.system_popup_active()
        || modals.buddy_modal()
        || modals.system_popup()
        || modals.nanocom_popup()
        || modals.quit_modal()
        || modals.option_modal()
        || modals.resurrect_modal()
        || modals.upsell_modal()
        || modals.guide_modal()
        || modals.bank_modal()
        || modals.vendor_modal()
        || modals.rule_modal()
        || modals.nano_free_tuning_modal()
        || modals.user_equip_modal()
        || modals.pc2pc_modal()
        || modals.world_map_modal()
        || modals.transportation_world_owned()
        || modals.race_modal()
        || modals.email_modal()
        || (modals.combi_modal() || modals.barber_modal())
        || modals.enchant_modal()
        || modals.cashmall_modal()
        || modals.user_store_modal();
    let stage = tutorial_active.then(|| tutorial.progress.stage()).flatten();
    let modal_input_blocked = tutorial_active
        && (system_popup
            || model.gameplay_input_blocked()
            || actors
                .iter()
                .any(|actor| tutorial_actor_needs_early_modal_lock(&model, actor)));
    let tutorial_weapon = tutorial_weapon_combat_profile(runtime.tutorial_weapon_id);
    for (mut context, presentation, collider_pending, environment, zipline, rope, slope) in &mut players {
        context.player_interaction_allowed = runtime.allow_player_interaction;
        context.dead = runtime.hp.is_some_and(|hp| hp <= 0);
        context.system_popup = system_popup;
        // Gameplay admission owns this gate in both shared-world states. The
        // tutorial player is spawned while `LegacyWorldColliderPending` is
        // present, so a loading frame can set the flag false before the state
        // changes to Tutorial. Recompute it every frame or the action resolver
        // keeps returning before attack input is handled after admission.
        context.ready_for_play = collider_pending.is_none();
        // Read movement ownership directly: presentation is updated later in
        // the frame and can still describe the previous ride on dismount.
        // Scripted travel blocks the whole action branch, including Nano use
        // and weapon cycling, without overwriting inventory/tutorial locks.
        context.move_mode = if zipline
            || rope
            || slope
            || transportation
                .as_ref()
                .is_some_and(|transport| transport.skyway_active)
        {
            LegacyMoveMode::Other
        } else {
            LegacyMoveMode::None
        };
        if !tutorial_active {
            apply_ordinary_world_weapon_combat_profile(
                &mut context,
                ordinary_world_weapon_combat_profile(&inventory, &weapon_catalog),
            );
            // NpcIconMode and the other mission surfaces call
            // MainGame.SetAvatarInputEnable(false) in the clean client. Keep
            // ordinary-world action input behind that same gate so a UI click
            // cannot also talk to the focused NPC through the modal.
            context.input_enabled = !system_popup
                && !model.gameplay_input_blocked()
                && runtime.hp.is_none_or(|hp| hp > 0);
            context.combat_condition = combat_gate
                .lifecycle
                .as_ref()
                .is_some_and(|combat| combat.active);
            context.time_buff_condition = combat_gate
                .skill_buffs
                .as_ref()
                .map_or(0, |buffs| buffs.local_control_condition());
            context.special_state_four = runtime.special_state & 4 != 0;
            // Equipment slot 8 only says a vehicle item is equipped. Clean
            // `iVehicle` changes after the mount lifecycle succeeds, so never
            // turn inventory equipment into a fabricated riding state.
            context.vehicle_mounted = presentation.is_some_and(|presentation| {
                presentation.mounted_vehicle != LegacyVehiclePresentationFamily::None
            });
            context.attack_players_enabled = false;
            context.attack_cooldown_blocked = false;
            context.weapon_change_cooldown_blocked = false;
            context.weapon_change_in_progress = combat_gate
                .item_move_owner
                .as_ref()
                .is_some_and(|owner| owner.active_hand_move_pending());
            context.overheat_allows_attack = combat_gate
                .overheat_model
                .as_ref()
                .zip(combat_gate.overheat_table.as_ref())
                .is_none_or(|(model, table)| model.allows_attack(table));
            context.primary_weapon_shootable = context.weapon_target_mode
                == LegacyWeaponTargetMode::Normal
                || runtime.weapon_battery > 0;
            let active_skill = active_world_nano_slot(&runtime).and_then(|(index, slot)| {
                let skill = content.gameplay_skill(slot.skill_id)?;
                (slot.stamina > 0
                    && ordinary_world_nano_skill_supported(skill)
                    && !world_nano_cooldowns.is_active(index, slot.skill_id))
                .then_some(skill)
            });
            context.nano_locked = runtime.nano_slots.iter().all(|slot| slot.nano_id.is_none());
            context.weapon_change_locked = false;
            context.tutorial_event = false;
            context.nano_skill_usable = active_skill.is_some();
            context.nano_target_policy = active_skill
                .filter(|skill| {
                    matches!(skill.target, 1 | 3 | 5 | 6) && matches!(skill.target_type, 1 | 2)
                })
                .map(|skill| LegacyNanoTargetPolicy {
                    effect_target: skill.target,
                    target_type: skill.target_type,
                    range: skill.range as f32 * SERVER_TO_CLIENT_SCALE,
                    area: skill.area as f32 * SERVER_TO_CLIENT_SCALE,
                    half_angle_degrees: skill.angle as f32,
                    capacity: usize::try_from(skill.target_number)
                        .unwrap_or_default()
                        .min(NanoSkillUseRequest0104::MAX_TARGETS),
                });
            context.nano_target_range = context
                .nano_target_policy
                .map_or(0.0, |policy| policy.range);
            context.nano_target_capacity = context
                .nano_target_policy
                .map_or(0, |policy| policy.capacity);
            context.weapon_swap_available = inventory.snapshot().is_some_and(|inventory| {
                super::world_combat::world_weapon_swap_request(
                    inventory.equipment(),
                    |item_id| weapon_catalog.profile_for_item(item_id).is_some(),
                )
                .is_some()
            });
            continue;
        }
        context.attack_half_angle_degrees = tutorial_weapon.half_angle_degrees;
        context.attack_range = tutorial_weapon.range;
        context.attack_cooldown_seconds = tutorial_weapon.cooldown_seconds;
        context.target_capacity = tutorial_weapon.target_capacity;
        context.weapon_target_mode = tutorial_weapon.target_mode;
        context.input_enabled = !modal_input_blocked && !tutorial.completion_requested;
        if tutorial.completion_requested {
            context.attack_locked = true;
            context.nano_locked = true;
            context.weapon_change_locked = true;
            context.tutorial_event = false;
            context.nano_skill_usable = false;
            context.nano_target_range = 0.0;
            context.nano_target_capacity = 0;
            context.nano_target_policy = None;
            context.weapon_swap_available = false;
            context.combat_condition = false;
            context.player_interaction_allowed = false;
            continue;
        }
        if stage.is_none() {
            context.attack_locked = true;
            context.nano_locked = true;
            context.weapon_change_locked = true;
            context.tutorial_event = false;
            context.nano_skill_usable = false;
            context.nano_target_range = 0.0;
            context.nano_target_capacity = 0;
            context.nano_target_policy = None;
            context.weapon_swap_available = false;
            context.combat_condition = false;
            continue;
        }
        context.attack_locked = native.is_locked(TutorialInputLock::Attack);
        context.nano_locked = native.is_locked(TutorialInputLock::NanoPower);
        context.weapon_change_locked = native.is_locked(TutorialInputLock::WeaponChange);
        // cntutorialscript.IsEvent reads bEventScene, which the choreography
        // can release before its tutorial stage changes.
        context.tutorial_event = tutorial_presentation
            .as_deref()
            .is_some_and(|presentation| presentation.event_scene);
        context.nano_skill_usable =
            !native.is_locked(TutorialInputLock::NanoPower) && gameplay_nano.can_use_skill();
        context.nano_target_range = TUTORIAL_BUTTERCUP_SKILL_RANGE;
        context.nano_target_capacity = TUTORIAL_BUTTERCUP_SKILL_TARGET_CAPACITY;
        context.nano_target_policy = None;
        context.weapon_swap_available = !native.is_locked(TutorialInputLock::WeaponChange);
        // GameFrame.SendCombatMode/EndCombatMode use five seconds since the
        // last incoming/outgoing combat result, independently of the chapter
        // or distance to a monster. Share the tutorial's HP-regeneration lease.
        context.combat_condition = !context.dead
            && environment.is_some_and(|environment| {
                environment.local_combat_timeout_remaining_seconds > 0.0
            });
    }
}

pub(super) fn open_tutorial_exit_dialog_on_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<ClientState>>,
    tutorial: Res<TutorialSession>,
    runtime: Res<RuntimeStatus>,
    logic: Res<TutorialLogicRuntime>,
    presentation: Res<TutorialChoreographyPresentation>,
    ready_players: Query<(), (With<LocalPlayer>, Without<LegacyWorldColliderPending>)>,
    mut model: ResMut<MissionUiModel>,
) {
    if tutorial.completion_requested || !keys.just_pressed(TUTORIAL_EXIT_KEY) {
        return;
    }
    let gate = TutorialExitDialogGate {
        tutorial_active: *state.get() == ClientState::Tutorial,
        world_ready: runtime.player_id.is_some() && runtime.map_number.is_some(),
        startup_ready: ready_players.single().is_ok(),
        delay_active: logic.delay_remaining_seconds.is_some(),
        system_popup_active: model.system_popup_active(),
        event_scene_active: presentation.event_scene,
    };
    if gate.allows_open() {
        model.open_tutorial_exit_dialog();
    }
}
