//! Option production runtime, open gates, input bindings, live settings and outbox.

#[cfg(test)]
use ffone_client::semantic_audio::NativeAudioCategory;
use super::guide::GuideProductionRuntime;
use super::nano_free_tuning::NanoFreeTuningProductionRuntime;
use super::race::RaceProductionRuntime;
use super::runtime_status::RuntimeStatus;
use super::state::ClientState;
use bevy::{
    audio::{GlobalVolume, Volume},
    ecs::system::SystemParam,
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{MonitorSelection, PrimaryWindow, WindowMode},
};
use ffone_client::{
    bank_runtime::BankProductionRuntime0104,
    bank_ui::{BankLifecyclePhase, BankUiOutbox0104, BankUiState},
    buddy_ui::{BUDDY_MAX_SLOTS, BuddyUiModel},
    cashmall_ui::{CashmallLifecyclePhase0104, CashmallUiState0104},
    combi_runtime::CombiProductionRuntime0104,
    email_runtime::EmailProductionRuntime0104,
    enchant_runtime::EnchantProductionRuntime0104,
    gameplay_audio::RetrobutionAudioMix,
    gameplay_ui::GameplayUiModel,
    guide_ui::{GuideUiModel, GuideUiOutbox},
    legacy_glow::LegacyGlowSettings,
    mission_ui::MissionUiModel,
    movement::LegacyOrbitCamera,
    nano_free_tuning_ui::NanoFreeTuningModel,
    nanocom_message_ui::NanocomMessageUiModel,
    network::{NetworkBridge, NetworkCommand},
    option_ui::{
        InputSettings, LegacyInputBinding, LegacyOptionAction, LegacyPhysicalKey, OptionBuddySlot,
        OptionCloseTrigger, OptionOpenAudioRoute, OptionSettings, OptionUiAction,
        OptionUiAssetGate, OptionUiAudioRouting, OptionUiEvent, OptionUiModel, OptionUiOutbox,
        SoundChannelSettings, SoundSettings, legacy_physical_key,
    },
    quit_menu_runtime::QuitMenuRuntime,
    quit_menu_ui::QuitMenuUiModel,
    resurrect_ui::ResurrectUiModel,
    rule_runtime::RuleRuntime,
    rule_ui::RuleUiModel,
    system_message_ui::SystemMessageUiModel,
    transportation_ui::{TransportationModel, TransportationPhase},
    upsell_ui::{UpsellUiModel, UpsellUiOutbox},
    user_equip_ui::UserEquipUiState,
    user_store_ui::UserStoreUiState0104,
    vendor_runtime::VendorProductionRuntime0104,
    vendor_ui::{VendorLifecyclePhase, VendorUiOutbox0104, VendorUiState},
    world_map::{WorldMapPhase, WorldMapPresentation},
};
use ffone_protocol::BuddyRemoveRequest0104;

/// Live counterpart to clean `cnOption` plus
/// `ConfigurableInput.WriteMappings`, backed by the versioned user-settings
/// document outside the runtime asset tree.
///
/// Draft and modal state remain inside `OptionUiModel`; only committed values
/// cross this boundary and become part of the atomic durable snapshot.
#[derive(Debug, Clone, Resource)]
pub(super) struct OptionProductionRuntime {
    pub(super) options: OptionSettings,
    pub(super) input: InputSettings,
    pub(super) initialized_from_live_runtime: bool,
}

impl Default for OptionProductionRuntime {
    fn default() -> Self {
        Self {
            // Keep the clean `cnDisplayOption.SetDefault` value for Scale UI.
            // The 1264x681 comparison viewport still resolves to scale 1.0,
            // while HD/FHD viewports receive the source height-based scale.
            options: OptionSettings::default(),
            input: InputSettings::default(),
            initialized_from_live_runtime: false,
        }
    }
}

impl OptionProductionRuntime {
    pub(super) fn effective_ui_scale(&self, _viewport_height: f32) -> f32 {
        // Production applies Scale UI to the WindowResolution logical canvas.
        // Returning the legacy matrix again here would scale already-laid-out
        // Bevy nodes a second time, while their clipping and text layout still
        // belong to the unscaled coordinate system.
        1.0
    }

    pub(super) fn initialize_from_live_runtime(
        &mut self,
        window: &Window,
        glow_enabled: bool,
        camera_sensitivity: Option<f32>,
    ) {
        if self.initialized_from_live_runtime {
            return;
        }
        self.options.graphics.width = window.physical_width().max(1);
        self.options.graphics.height = window.physical_height().max(1);
        self.options.graphics.windowed = matches!(window.mode, WindowMode::Windowed);
        self.options.graphics.glow = glow_enabled;
        if let Some(sensitivity) = camera_sensitivity.filter(|value| value.is_finite()) {
            self.input.camera_sensitivity = sensitivity.clamp(1.0, 10.0);
        }
        self.initialized_from_live_runtime = true;
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub(super) struct OptionOpenBlockers {
    pub(super) chat: bool,
    pub(super) buddy: bool,
    pub(super) mission: bool,
    pub(super) system_popup: bool,
    pub(super) nanocom: bool,
    pub(super) quit: bool,
    pub(super) resurrect: bool,
    pub(super) upsell: bool,
    pub(super) guide: bool,
    pub(super) bank: bool,
    pub(super) vendor: bool,
    pub(super) rule: bool,
    pub(super) nano_free_tuning: bool,
    pub(super) user_equip: bool,
    pub(super) world_map: bool,
    pub(super) transportation: bool,
    pub(super) race: bool,
    pub(super) email: bool,
    pub(super) combi: bool,
    pub(super) enchant: bool,
    pub(super) cashmall: bool,
    pub(super) user_store: bool,
}

impl OptionOpenBlockers {
    #[must_use]
    pub(super) const fn blocked(self) -> bool {
        self.chat
            || self.buddy
            || self.mission
            || self.system_popup
            || self.nanocom
            || self.quit
            || self.resurrect
            || self.upsell
            || self.guide
            || self.bank
            || self.vendor
            || self.rule
            || self.nano_free_tuning
            || self.user_equip
            || self.world_map
            || self.transportation
            || self.race
            || self.email
            || self.combi
            || self.enchant
            || self.cashmall
            || self.user_store
    }
}

#[derive(SystemParam)]
pub(super) struct OptionOpenInputs<'w> {
    pub(super) gameplay_ui: ResMut<'w, GameplayUiModel>,
    pub(super) buddy_ui: Res<'w, BuddyUiModel>,
    pub(super) mission_ui: Res<'w, MissionUiModel>,
    pub(super) system_messages: Res<'w, SystemMessageUiModel>,
    pub(super) nanocom_messages: Res<'w, NanocomMessageUiModel>,
    pub(super) quit_model: Res<'w, QuitMenuUiModel>,
    pub(super) quit_runtime: Res<'w, QuitMenuRuntime>,
    pub(super) resurrect_ui: Res<'w, ResurrectUiModel>,
    pub(super) guide_ui: Res<'w, GuideUiModel>,
    pub(super) guide_outbox: Res<'w, GuideUiOutbox>,
    pub(super) guide_production: Res<'w, GuideProductionRuntime>,
    pub(super) bank_state: Res<'w, BankUiState>,
    pub(super) bank_outbox: Res<'w, BankUiOutbox0104>,
    pub(super) bank_production: Res<'w, BankProductionRuntime0104>,
    pub(super) vendor_state: Res<'w, VendorUiState>,
    pub(super) vendor_outbox: Res<'w, VendorUiOutbox0104>,
    pub(super) vendor_production: Res<'w, VendorProductionRuntime0104>,
    pub(super) rule_ui: Res<'w, RuleUiModel>,
    pub(super) rule_runtime: Res<'w, RuleRuntime>,
    pub(super) nano_free_tuning: Res<'w, NanoFreeTuningModel>,
    pub(super) nano_free_tuning_production: Res<'w, NanoFreeTuningProductionRuntime>,
    pub(super) upsell_ui: Res<'w, UpsellUiModel>,
    pub(super) upsell_outbox: Res<'w, UpsellUiOutbox>,
    pub(super) user_equip_ui: Res<'w, UserEquipUiState>,
    pub(super) world_map: Res<'w, WorldMapPresentation>,
    pub(super) transportation: Option<Res<'w, TransportationModel>>,
    pub(super) race_production: Res<'w, RaceProductionRuntime>,
    pub(super) email_runtime: Res<'w, EmailProductionRuntime0104>,
    pub(super) combi_runtime: Res<'w, CombiProductionRuntime0104>,
    pub(super) enchant_runtime: Res<'w, EnchantProductionRuntime0104>,
    pub(super) cashmall_ui: Option<Res<'w, CashmallUiState0104>>,
    pub(super) user_store_ui: Option<Res<'w, UserStoreUiState0104>>,
    pub(super) asset_gate: Res<'w, OptionUiAssetGate>,
}

pub(super) fn option_binding_just_pressed(
    binding: LegacyInputBinding,
    keyboard: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
) -> bool {
    let LegacyInputBinding::Key(physical) = binding else {
        return false;
    };
    match physical {
        LegacyPhysicalKey::Mouse0 => mouse.just_pressed(MouseButton::Left),
        LegacyPhysicalKey::Mouse1 => mouse.just_pressed(MouseButton::Right),
        _ => keyboard
            .get_just_pressed()
            .any(|key| legacy_physical_key(*key) == Some(physical)),
    }
}

pub(super) fn option_action_just_pressed(
    input: &InputSettings,
    action: LegacyOptionAction,
    keyboard: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
) -> bool {
    input
        .mappings
        .iter()
        .find(|row| row.action == action)
        .is_some_and(|row| {
            option_binding_just_pressed(row.primary, keyboard, mouse)
                || option_binding_just_pressed(row.alternate, keyboard, mouse)
        })
}

pub(super) fn option_binding_just_released(
    binding: LegacyInputBinding,
    keyboard: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
) -> bool {
    let LegacyInputBinding::Key(physical) = binding else {
        return false;
    };
    match physical {
        LegacyPhysicalKey::Mouse0 => mouse.just_released(MouseButton::Left),
        LegacyPhysicalKey::Mouse1 => mouse.just_released(MouseButton::Right),
        _ => keyboard
            .get_just_released()
            .any(|key| legacy_physical_key(*key) == Some(physical)),
    }
}

pub(super) fn option_action_just_released(
    input: &InputSettings,
    action: LegacyOptionAction,
    keyboard: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
) -> bool {
    input
        .mappings
        .iter()
        .find(|row| row.action == action)
        .is_some_and(|row| {
            option_binding_just_released(row.primary, keyboard, mouse)
                || option_binding_just_released(row.alternate, keyboard, mouse)
        })
}

pub(super) fn clear_option_action_press(
    input: &InputSettings,
    action: LegacyOptionAction,
    keyboard: &mut ButtonInput<KeyCode>,
    mouse: &mut ButtonInput<MouseButton>,
) {
    let Some(row) = input.mappings.iter().find(|row| row.action == action) else {
        return;
    };
    for binding in [row.primary, row.alternate] {
        let LegacyInputBinding::Key(physical) = binding else {
            continue;
        };
        match physical {
            LegacyPhysicalKey::Mouse0 => {
                mouse.clear_just_pressed(MouseButton::Left);
            }
            LegacyPhysicalKey::Mouse1 => {
                mouse.clear_just_pressed(MouseButton::Right);
            }
            _ => {
                let pressed = keyboard.get_just_pressed().copied().collect::<Vec<_>>();
                for key in pressed {
                    if legacy_physical_key(key) == Some(physical) {
                        keyboard.clear_just_pressed(key);
                    }
                }
            }
        }
    }
}

pub(super) fn sync_option_live_runtime(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&LegacyOrbitCamera, Option<&LegacyGlowSettings>)>,
    mut runtime: ResMut<OptionProductionRuntime>,
) {
    if runtime.initialized_from_live_runtime {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some((camera, glow)) = cameras.iter().next() else {
        return;
    };
    runtime.initialize_from_live_runtime(
        window,
        glow.is_some(),
        Some(camera.configurable_sensitivity),
    );
}

/// Projects loaded settings to owners that may be created after startup.
/// Window geometry is already applied by `WindowPlugin`; avoiding it here also
/// prevents a no-op swapchain resize. Newly spawned gameplay cameras still
/// need their persisted glow state, while master volume and buddy policy are
/// refreshed whenever the committed option owner changes.
pub(super) fn sync_option_settings_to_live_owners(
    mut commands: Commands,
    runtime: Res<OptionProductionRuntime>,
    cameras: Query<(Entity, Ref<LegacyOrbitCamera>, Option<&LegacyGlowSettings>)>,
    mut buddy_ui: ResMut<BuddyUiModel>,
    mut global: ResMut<GlobalVolume>,
) {
    let runtime_changed = runtime.is_changed();
    if runtime_changed {
        buddy_ui.set_social_buddy_enabled(runtime.options.social.allow_buddy_requests);
        set_option_global_master(&runtime.options.sound, &mut global);
    }
    for (entity, camera, glow) in &cameras {
        if !runtime_changed && !camera.is_added() {
            continue;
        }
        if runtime.options.graphics.glow && glow.is_none() {
            commands
                .entity(entity)
                .insert(LegacyGlowSettings::default());
        } else if !runtime.options.graphics.glow && glow.is_some() {
            commands.entity(entity).remove::<LegacyGlowSettings>();
        }
    }
}

pub(super) fn sync_option_buddy_projection(buddy_ui: Res<BuddyUiModel>, mut option_ui: ResMut<OptionUiModel>) {
    // Buddy data changes on network events, not every rendered frame. Build
    // the small projection without mutably dereferencing OptionUiModel, then
    // dirty that resource only when the value really changed. Otherwise every
    // idle world frame wakes the entire hidden OptionMode binding/layout set.
    if !buddy_ui.is_changed() && option_ui.buddy_slots.len() == BUDDY_MAX_SLOTS {
        return;
    }
    let projected = (0..BUDDY_MAX_SLOTS)
        .map(|slot| {
            buddy_ui
                .slot(slot)
                .map_or_else(OptionBuddySlot::default, |entry| OptionBuddySlot {
                    pc_uid: entry.pc_uid,
                    blocked: entry.blocked,
                    name_check_flag: entry.name_check_flag,
                    first_name: entry.first_name.clone(),
                    last_name: entry.last_name.clone(),
                })
        })
        .collect::<Vec<_>>();
    if option_ui.buddy_slots != projected {
        option_ui.buddy_slots = projected;
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn route_option_mode_input(
    state: Res<State<ClientState>>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    runtime: Res<OptionProductionRuntime>,
    pad: Option<Res<super::GamepadActionState>>,
    routing: Res<OptionUiAudioRouting>,
    mut inputs: OptionOpenInputs,
    mut model: ResMut<OptionUiModel>,
    mut outbox: ResMut<OptionUiOutbox>,
    mut status: ResMut<RuntimeStatus>,
) {
    if *state.get() != ClientState::World {
        return;
    }

    if model.visible {
        // Remap capture owns its entire input frame. Clean OptionMode also
        // refuses the local close keys while its SystemMessage is active.
        if model.key_capture.is_some() || model.modal.system_popup {
            return;
        }
        let escape = keyboard.just_pressed(KeyCode::Escape) || pad.as_ref().is_some_and(|pad| pad.just_pressed(LegacyOptionAction::Escape)) || option_action_just_pressed(
            &model.draft_input,
            LegacyOptionAction::Escape,
            &keyboard,
            &mouse,
        );
        let option = pad.as_ref().is_some_and(|pad| pad.just_pressed(LegacyOptionAction::Option)) || option_action_just_pressed(
            &model.draft_input,
            LegacyOptionAction::Option,
            &keyboard,
            &mouse,
        );
        if !escape && !option {
            return;
        }
        if escape {
            keyboard.clear_just_pressed(KeyCode::Escape);
            clear_option_action_press(
                &model.draft_input,
                LegacyOptionAction::Escape,
                &mut keyboard,
                &mut mouse,
            );
        }
        if option {
            clear_option_action_press(
                &model.draft_input,
                LegacyOptionAction::Option,
                &mut keyboard,
                &mut mouse,
            );
        }
        model.cancel(OptionCloseTrigger::Shortcut, routing.close, &mut outbox);
        return;
    }

    if !pad.as_ref().is_some_and(|pad| pad.just_pressed(LegacyOptionAction::Option)) && !option_action_just_pressed(
        &runtime.input,
        LegacyOptionAction::Option,
        &keyboard,
        &mouse,
    ) {
        return;
    }

    let chat_active = inputs.gameplay_ui.visible
        && inputs.gameplay_ui.chat.input_enabled
        && inputs.gameplay_ui.chat.active;
    let blockers = OptionOpenBlockers {
        chat: chat_active,
        buddy: inputs.buddy_ui.add_dialog_open(),
        mission: inputs.mission_ui.gameplay_input_blocked(),
        system_popup: inputs.system_messages.is_popup() || inputs.mission_ui.system_popup_active(),
        nanocom: inputs.nanocom_messages.expanded_visible(),
        quit: inputs.quit_model.visible || inputs.quit_runtime.is_waiting_for_server(),
        resurrect: inputs.resurrect_ui.visible,
        upsell: inputs.upsell_ui.visible() || !inputs.upsell_outbox.is_empty(),
        guide: inputs.guide_production.modal_active(&inputs.guide_ui)
            || !inputs.guide_outbox.is_empty(),
        bank: inputs.bank_state.phase != BankLifecyclePhase::Hidden
            || inputs.bank_production.modal_active()
            || !inputs.bank_outbox.is_empty(),
        vendor: inputs.vendor_state.phase != VendorLifecyclePhase::Hidden
            || inputs.vendor_production.modal_active()
            || !inputs.vendor_outbox.is_empty(),
        rule: inputs.rule_runtime.modal_active(&inputs.rule_ui),
        nano_free_tuning: inputs
            .nano_free_tuning_production
            .modal_active(&inputs.nano_free_tuning),
        user_equip: inputs.user_equip_ui.is_active(),
        world_map: inputs.world_map.model.phase() != WorldMapPhase::Closed,
        transportation: inputs
            .transportation
            .as_ref()
            .is_some_and(|model| model.phase() != TransportationPhase::Hidden),
        race: inputs.race_production.modal_active(),
        email: inputs.email_runtime.modal_active(),
        combi: inputs.combi_runtime.modal_active(),
        enchant: inputs.enchant_runtime.is_active(),
        cashmall: inputs
            .cashmall_ui
            .as_ref()
            .is_some_and(|state| state.phase() != CashmallLifecyclePhase0104::Hidden),
        user_store: inputs
            .user_store_ui
            .as_ref()
            .is_some_and(|state| state.active),
    };
    if blockers.blocked() {
        return;
    }

    clear_option_action_press(
        &runtime.input,
        LegacyOptionAction::Option,
        &mut keyboard,
        &mut mouse,
    );
    if inputs.asset_gate.failed {
        status.message = "OptionMode blocked by missing exact native UI assets".to_owned();
        return;
    }
    if !inputs.asset_gate.ready {
        status.message = "OptionMode is waiting for its exact native UI assets".to_owned();
        return;
    }
    if !runtime.initialized_from_live_runtime {
        status.message = "OptionMode is waiting for its live window and camera owners".to_owned();
        return;
    }

    inputs.gameplay_ui.chat.input_enabled = false;
    inputs.gameplay_ui.chat.active = false;
    inputs.gameplay_ui.chat.input.clear();
    model.open(
        runtime.options.clone(),
        runtime.input.clone(),
        OptionOpenAudioRoute {
            main_game_transition: true,
            inventory_transition: false,
        },
        &mut outbox,
    );
}

pub(super) fn option_channel_gain(channel: SoundChannelSettings) -> f32 {
    if !channel.enabled || !channel.volume.is_finite() {
        0.0
    } else {
        channel.volume.clamp(0.0, 1.0)
    }
}

pub(super) fn sync_retrobution_audio_mix(
    options: Res<OptionProductionRuntime>,
    mut mix: ResMut<RetrobutionAudioMix>,
) {
    mix.set_if_neq(RetrobutionAudioMix {
        music: option_channel_gain(options.options.sound.music),
        ambient: option_channel_gain(options.options.sound.ambient),
        effects: option_channel_gain(options.options.sound.effects),
        voice: option_channel_gain(options.options.sound.voice),
    });
}

#[cfg(test)]
pub(super) fn option_sound_gain(sound: &SoundSettings, category: Option<NativeAudioCategory>) -> f32 {
    let master = option_channel_gain(sound.master);
    let channel = match category {
        Some(NativeAudioCategory::Music) => option_channel_gain(sound.music),
        Some(NativeAudioCategory::Ambient) => option_channel_gain(sound.ambient),
        Some(NativeAudioCategory::Voice) => option_channel_gain(sound.voice),
        Some(NativeAudioCategory::Sfx) => option_channel_gain(sound.effects),
        None => 1.0,
    };
    master * channel
}

pub(super) fn set_option_global_master(sound: &SoundSettings, global: &mut GlobalVolume) {
    global.volume = Volume::Linear(option_channel_gain(sound.master));
}

pub(super) fn option_window_mode_changed(current: &OptionSettings, next: &OptionSettings) -> bool {
    current.graphics.width != next.graphics.width
        || current.graphics.height != next.graphics.height
        || current.graphics.windowed != next.graphics.windowed
}

pub(super) fn apply_option_camera_input_settings(
    runtime: Res<OptionProductionRuntime>,
    motion: Option<ResMut<AccumulatedMouseMotion>>,
    mut cameras: Query<&mut LegacyOrbitCamera>,
) {
    let sensitivity = if runtime.input.camera_sensitivity.is_finite() {
        runtime.input.camera_sensitivity.clamp(1.0, 10.0)
    } else {
        InputSettings::default().camera_sensitivity
    };
    for mut camera in &mut cameras {
        camera.configurable_sensitivity = sensitivity;
    }
    if runtime.input.invert_y
        && let Some(mut motion) = motion
    {
        motion.delta.y = -motion.delta.y;
    }
}

pub(super) fn apply_option_settings_to_live_runtime(
    options: &OptionSettings,
    apply_window_mode: bool,
    commands: &mut Commands,
    windows: &mut Query<&mut Window, With<PrimaryWindow>>,
    cameras: &mut Query<(Entity, &mut LegacyOrbitCamera, Option<&LegacyGlowSettings>)>,
    buddy_ui: &mut BuddyUiModel,
    global: &mut GlobalVolume,
) {
    if apply_window_mode && let Ok(mut window) = windows.single_mut() {
        window.resolution.set_physical_resolution(
            options.graphics.width.max(1),
            options.graphics.height.max(1),
        );
        window.mode = if options.graphics.windowed {
            WindowMode::Windowed
        } else {
            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
        };
    }
    for (entity, _, glow) in cameras.iter_mut() {
        if options.graphics.glow && glow.is_none() {
            commands
                .entity(entity)
                .insert(LegacyGlowSettings::default());
        } else if !options.graphics.glow && glow.is_some() {
            commands.entity(entity).remove::<LegacyGlowSettings>();
        }
    }
    buddy_ui.set_social_buddy_enabled(options.social.allow_buddy_requests);
    set_option_global_master(&options.sound, global);
}

#[allow(clippy::too_many_arguments)]
pub(super) fn consume_option_ui_outbox(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    bridge: Res<NetworkBridge>,
    mut outbox: ResMut<OptionUiOutbox>,
    mut production: ResMut<OptionProductionRuntime>,
    mut global: ResMut<GlobalVolume>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut cameras: Query<(Entity, &mut LegacyOrbitCamera, Option<&LegacyGlowSettings>)>,
    mut buddy_ui: ResMut<BuddyUiModel>,
    mut status: ResMut<RuntimeStatus>,
) {
    while let Some(event) = outbox.pop_front() {
        match event {
            OptionUiEvent::Action(OptionUiAction::PersistOptions(options)) => {
                // Applying an unrelated option must not ask the window backend
                // to recreate the swapchain or re-apply the current client
                // area. On Windows that observable no-op can still resize a
                // DPI-scaled window and was especially easy to trigger after
                // changing only the text/voice locale.
                let apply_window_mode = option_window_mode_changed(&production.options, &options);
                production.options = options;
                apply_option_settings_to_live_runtime(
                    &production.options,
                    apply_window_mode,
                    &mut commands,
                    &mut windows,
                    &mut cameras,
                    &mut buddy_ui,
                    &mut global,
                );
                status.message = "Options applied to the live client session".to_owned();
            }
            OptionUiEvent::Action(OptionUiAction::PersistInput(input)) => {
                production.input = input;
                for (_, mut camera, _) in &mut cameras {
                    camera.configurable_sensitivity =
                        production.input.camera_sensitivity.clamp(1.0, 10.0);
                }
                status.message =
                    "Option, camera sensitivity, and invert-Y input settings applied".to_owned();
            }
            OptionUiEvent::Action(OptionUiAction::ApplySoundImmediately(sound)) => {
                production.options.sound = sound;
                set_option_global_master(&production.options.sound, &mut global);
            }
            OptionUiEvent::Action(OptionUiAction::RemoveBuddy { slot, pc_uid }) => {
                let valid = buddy_ui
                    .slot(slot)
                    .is_some_and(|entry| entry.pc_uid == pc_uid && entry.blocked);
                let Some(buddy_slot) = i8::try_from(slot).ok().filter(|slot| *slot >= 0) else {
                    status.message = format!("OptionMode rejected invalid Buddy slot {slot}");
                    continue;
                };
                if !valid {
                    status.message =
                        format!("OptionMode rejected stale blocked Buddy {slot}/{pc_uid}");
                    continue;
                }
                if let Err(error) =
                    bridge.send(NetworkCommand::RemoveBuddy(BuddyRemoveRequest0104 {
                        buddy_pc_uid: pc_uid,
                        buddy_slot,
                    }))
                {
                    status.message = error;
                }
            }
            OptionUiEvent::Action(OptionUiAction::Closed) => {}
            OptionUiEvent::Audio(cue) => {
                let effects_gain = option_channel_gain(production.options.sound.effects);
                commands.spawn((
                    Name::new(format!("OptionMode audio {}", cue.path())),
                    ffone_client::audio_channel::GameplayAudioChannel::ui_sfx(),
                    AudioPlayer::new(asset_server.load(cue.path())),
                    PlaybackSettings::DESPAWN
                        .with_volume(Volume::Linear(cue.gain() * effects_gain)),
                ));
            }
        }
    }
}

pub(super) fn reset_option_session(mut model: ResMut<OptionUiModel>, mut outbox: ResMut<OptionUiOutbox>) {
    model.reset_runtime_session();
    outbox.clear();
}
