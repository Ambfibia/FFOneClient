//! Cash mall, user store and upsell production integration.

use super::gameplay_ui_actions;
use super::option_runtime::{
    OptionProductionRuntime, clear_option_action_press, option_action_just_pressed,
};
use super::runtime_status::RuntimeStatus;
use super::state::ClientState;
use bevy::{
    audio::Volume,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use ffone_client::{
    cashmall_ui::{
        CASHMALL_HIDDEN_CHAT_COMMAND_0104, CashmallAudioCue0104, CashmallCloseGate0104,
        CashmallCloseSource0104, CashmallLifecyclePhase0104, CashmallLocalEffect0104,
        CashmallModalState0104, CashmallModeProjection0104, CashmallUiOutbox0104,
        CashmallUiState0104,
    },
    gameplay_audio::GameplayAudioRuntime,
    gameplay_ui::GameplayUiModel,
    mission_ui::MissionUiModel,
    option_ui::LegacyOptionAction,
    resurrect_ui::ResurrectUiModel,
    system_message_ui::SystemMessageUiModel,
    upsell_ui::{UpsellUiAction, UpsellUiAudioOutbox, UpsellUiModel, UpsellUiOutbox},
    user_equip_ui::{
        UserEquipItemModeProjection, UserEquipModalState, UserEquipUiOutbox, UserEquipUiState,
    },
    user_store_runtime::{UserStoreGuardRejection0104, UserStoreProductionRuntime0104},
    user_store_ui::{
        UserStoreAuthority0104, UserStorePopupPresentation0104, UserStoreUiOutbox0104,
        UserStoreUiState0104,
    },
};

pub(super) fn reset_upsell_shell(
    model: &mut UpsellUiModel,
    outbox: &mut UpsellUiOutbox,
    audio: &mut UpsellUiAudioOutbox,
) {
    model.close();
    outbox.clear();
    audio.clear();
}

pub(super) fn sync_upsell_ui_context(
    state: Res<State<ClientState>>,
    gameplay_ui: Res<GameplayUiModel>,
    system_messages: Res<SystemMessageUiModel>,
    resurrect_ui: Res<ResurrectUiModel>,
    status: Res<RuntimeStatus>,
    mut model: ResMut<UpsellUiModel>,
    mut outbox: ResMut<UpsellUiOutbox>,
    mut audio: ResMut<UpsellUiAudioOutbox>,
) {
    model.set_ui_scale(gameplay_ui.ui_scale);
    model.set_external_modes(system_messages.is_popup(), false);
    // Clean `GameFrame` reports an exit-capable main-game mode here, while
    // the inventory PopupExit result is zero for this native shell.
    model.set_escape_context(*state.get() == ClientState::World, 0);

    // ResurrectMode owns the death surface and the Upsell owner is linked only
    // to MainGame. Leaving World or entering death therefore tears it down
    // without fabricating the unreachable legacy LeaveFuture branch.
    if *state.get() != ClientState::World
        || resurrect_ui.visible
        || status.hp.is_some_and(|hp| hp <= 0)
    {
        reset_upsell_shell(&mut model, &mut outbox, &mut audio);
    }
}

pub(super) fn consume_upsell_ui_outbox(mut outbox: ResMut<UpsellUiOutbox>, mut status: ResMut<RuntimeStatus>) {
    while let Some(action) = outbox.pop_front() {
        match action {
            UpsellUiAction::Exit => {
                status.message = "Upsell returned to the linked MainGame mode".to_owned();
            }
            UpsellUiAction::OpenPayPage => {
                // `cnUpsell.PayPage` tears down nearly every live client
                // manager and transfers to the external billing callback.
                // Until that owner exists natively, retain the visible Upgrade
                // screen and its typed intent instead of partially destroying
                // the active world.
                status.message =
                    "Upsell PayPage retained: exact billing callback and teardown owner are unavailable"
                        .to_owned();
            }
        }
    }
}

pub(super) fn consume_upsell_audio_outbox(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut outbox: ResMut<UpsellUiAudioOutbox>,
    audio_runtime: Option<ResMut<GameplayAudioRuntime>>,
) {
    let mut audio_runtime = audio_runtime;
    while let Some(cue) = outbox.pop_front() {
        let Some(path) = cue.exact_path() else {
            if let Some(audio_runtime) = audio_runtime.as_deref_mut() {
                audio_runtime.queue_legacy_button_sound();
            }
            continue;
        };
        commands.spawn((
            Name::new(format!("Upsell UI audio {path}")),
            ffone_client::audio_channel::GameplayAudioChannel::ui_sfx(),
            AudioPlayer::new(asset_server.load(path)),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.7)),
        ));
    }
}

pub(super) const CASHMALL_TAB_CLICK_AUDIO_PATH_0104: &str = "audio/sfx/ui/tab_click01.ogg";

pub(super) fn sync_cashmall_projection_0104(
    projection: &mut CashmallModeProjection0104,
    shared_item_mode: &UserEquipItemModeProjection,
    taros: i32,
) {
    // Clean `Panel_Cashmall.DoSlot` scans the retained global slot type 9.
    // FFOne owns no authoritative slot-9 cache, and protocol-0104 exposes no
    // Cash Mall catalog reply, so keeping this list empty is the only exact
    // fail-closed projection. Normal inventory/equipment and Taros still come
    // from their real production owners.
    projection.player_inventory.clear();
    projection.shared_item_mode.clone_from(shared_item_mode);
    projection.taros = taros;
}

pub(super) fn sync_cashmall_production_context_0104(
    runtime: Res<RuntimeStatus>,
    mission_ui: Res<MissionUiModel>,
    system_messages: Res<SystemMessageUiModel>,
    shared_item_mode: Res<UserEquipItemModeProjection>,
    state: Res<CashmallUiState0104>,
    mut modal: ResMut<CashmallModalState0104>,
    mut close_gate: ResMut<CashmallCloseGate0104>,
    mut projection: ResMut<CashmallModeProjection0104>,
) {
    let system_popup_active = mission_ui.system_popup_active() || system_messages.is_popup();
    *modal = CashmallModalState0104 {
        system_popup_active,
        // The bounded production shell has no Help, generic-popup, redeem or
        // InventoryManager popup owner. Those routes stay typed fail-closed
        // in `cashmall_ui` instead of inventing modal state here.
        ..default()
    };
    *close_gate = CashmallCloseGate0104 {
        // The active mode is the exact clean `(2, 24)` receiver. System popup
        // gating is applied separately by `CashmallModalState0104`.
        mode_accepts_escape: state.phase() != CashmallLifecyclePhase0104::Hidden,
        // No native InventoryManager/PopupControll window is active, so clean
        // `(11, 13)[0]` is zero. Future popup owners must replace this input.
        exit_arbitration_clear: true,
    };
    sync_cashmall_projection_0104(&mut projection, &shared_item_mode, runtime.candy);
}

pub(super) fn route_cashmall_production_input_0104(
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    option_runtime: Res<OptionProductionRuntime>,
    modal: Res<CashmallModalState0104>,
    close_gate: Res<CashmallCloseGate0104>,
    mut state: ResMut<CashmallUiState0104>,
    mut outbox: ResMut<CashmallUiOutbox0104>,
    mut runtime: ResMut<RuntimeStatus>,
) {
    if state.phase() == CashmallLifecyclePhase0104::Hidden
        || !state.input_capabilities(*modal).escape_request
        || !option_action_just_pressed(
            &option_runtime.input,
            LegacyOptionAction::Escape,
            &keyboard,
            &mouse,
        )
    {
        return;
    }
    // Cashmall is ConfigurableInput Key(4), the same exact Escape action used
    // by the other production mode drivers. Consume only that mapped edge so
    // it cannot fall through to QuitMenu after the local close commits.
    clear_option_action_press(
        &option_runtime.input,
        LegacyOptionAction::Escape,
        &mut keyboard,
        &mut mouse,
    );
    if let Err(error) = state.request_close(
        CashmallCloseSource0104::ConfigurableKey4,
        *modal,
        *close_gate,
        &mut outbox,
    ) {
        runtime.message = format!("Cashmall configurable close rejected: {error:?}");
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn consume_cashmall_production_effects_0104(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut outbox: ResMut<CashmallUiOutbox0104>,
    mut user_equip_state: ResMut<UserEquipUiState>,
    mut user_equip_modal: ResMut<UserEquipModalState>,
    mut user_equip_outbox: ResMut<UserEquipUiOutbox>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
    audio_runtime: Option<ResMut<GameplayAudioRuntime>>,
    mut runtime: ResMut<RuntimeStatus>,
) {
    let mut audio_runtime = audio_runtime;
    for effect in outbox.drain().collect::<Vec<_>>() {
        match effect {
            CashmallLocalEffect0104::EnterMode {
                ui_input_event,
                ui_input_enter_value,
                force_inventory_tab,
                force_cursor_unlocked,
            } => {
                if ui_input_event != [11, 0]
                    || ui_input_enter_value != 1
                    || force_inventory_tab != 0
                    || !force_cursor_unlocked
                {
                    runtime.message =
                        "Cashmall rejected a malformed local mode-entry boundary".to_owned();
                    continue;
                }
                if let Ok(mut cursor) = cursors.single_mut() {
                    cursor.grab_mode = CursorGrabMode::None;
                    cursor.visible = true;
                }
            }
            CashmallLocalEffect0104::PlayAudio(CashmallAudioCue0104::TabClick01) => {
                commands.spawn((
                    Name::new("Cashmall Tab_Click01 audio"),
                    ffone_client::audio_channel::GameplayAudioChannel::ui_sfx(),
                    AudioPlayer::new(asset_server.load(CASHMALL_TAB_CLICK_AUDIO_PATH_0104)),
                    PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.7)),
                ));
            }
            CashmallLocalEffect0104::PlayAudio(CashmallAudioCue0104::ButtonSound) => {
                if let Some(audio_runtime) = audio_runtime.as_deref_mut() {
                    audio_runtime.queue_legacy_button_sound();
                }
            }
            CashmallLocalEffect0104::VendorClickItem(boundary) => {
                // This is not a purchase packet. It targets the absent shared
                // Vendor/Inventory popup owner and is unreachable while the
                // authoritative slot-9 projection remains empty.
                runtime.message = format!(
                    "Cashmall retained local Vendor ClickItem for slot {}/{}; exact shared popup owner is unavailable",
                    boundary.slot_type, boundary.slot_id
                );
            }
            CashmallLocalEffect0104::RetainedRightClickEquipmentEvent {
                event,
                item_id,
                item_type,
                option,
            } => {
                runtime.message = format!(
                    "Cashmall retained local equipment event {event:?} for item {item_type}/{item_id}/{option}; exact shared owner is unavailable"
                );
            }
            CashmallLocalEffect0104::GoToMyStuff(boundary) => {
                let exact = boundary.close_inventory_event == [2, 3, 5]
                    && boundary.request_game_mode_event == [2, 0]
                    && boundary.next_game_mode == 6
                    && boundary.receive_init_event == [2, 3, 0]
                    && boundary.next_mode_init_argument == 2
                    && boundary.first_use_condition == 3
                    && boundary.final_refresh_event == [11, 18];
                if !exact {
                    runtime.message =
                        "Cashmall rejected a malformed GO TO MY STUFF transition".to_owned();
                    continue;
                }
                *user_equip_modal = UserEquipModalState::default();
                user_equip_outbox.clear();
                // This is clean mode 6 / ReceiveInit argument 2. Calling the
                // public Item-mode owner directly avoids falsely labelling it
                // as the separate NanocomMyStuff source.
                user_equip_state.open_item_mode();
            }
            CashmallLocalEffect0104::Close(boundary) => {
                if boundary.ui_input_event != [11, 0]
                    || boundary.ui_input_exit_value != 10
                    || boundary.notify_game_mode_exit != [2, 1]
                    || !boundary.stop_ui_mode_sound
                    || !boundary.request_asset_gc
                    || boundary.loaded_textures_actually_cleared
                {
                    runtime.message =
                        "Cashmall rejected a malformed local mode-exit boundary".to_owned();
                    continue;
                }
                if let Ok(mut cursor) = cursors.single_mut() {
                    cursor.grab_mode = if boundary.restore_cursor_locked {
                        CursorGrabMode::Locked
                    } else {
                        CursorGrabMode::None
                    };
                    cursor.visible = !boundary.restore_cursor_locked;
                }
            }
            CashmallLocalEffect0104::DeadHelpSendMessage => {
                runtime.message =
                    "Cashmall ClickHelp has no receiver on the clean CashmallMode owner".to_owned();
            }
        }
    }
}

pub(super) fn reset_cashmall_session_0104(
    mut state: ResMut<CashmallUiState0104>,
    mut modal: ResMut<CashmallModalState0104>,
    mut close_gate: ResMut<CashmallCloseGate0104>,
    mut projection: ResMut<CashmallModeProjection0104>,
    mut outbox: ResMut<CashmallUiOutbox0104>,
) {
    *state = CashmallUiState0104::default();
    *modal = CashmallModalState0104::default();
    *close_gate = CashmallCloseGate0104::default();
    *projection = CashmallModeProjection0104::default();
    outbox.clear();
}

pub(super) fn report_user_store_guard_rejection_0104(
    rejection: UserStoreGuardRejection0104,
    runtime: &mut RuntimeStatus,
) {
    runtime.message = format!(
        "UserStore production fail-closed: {}; dropped {} unowned UI command(s)",
        rejection.error, rejection.dropped_commands
    );
}

pub(super) fn guard_user_store_production_boundary_0104(
    production: &mut UserStoreProductionRuntime0104,
    state: &mut UserStoreUiState0104,
    popup: &mut UserStorePopupPresentation0104,
    outbox: &mut UserStoreUiOutbox0104,
    runtime: &mut RuntimeStatus,
) {
    if let Some(rejection) = production.guard_unowned_ui(state, popup, outbox) {
        report_user_store_guard_rejection_0104(rejection, runtime);
    }
}

pub(super) fn guard_user_store_before_interaction_0104(
    mut production: ResMut<UserStoreProductionRuntime0104>,
    mut state: ResMut<UserStoreUiState0104>,
    mut popup: ResMut<UserStorePopupPresentation0104>,
    mut outbox: ResMut<UserStoreUiOutbox0104>,
    mut runtime: ResMut<RuntimeStatus>,
) {
    guard_user_store_production_boundary_0104(
        &mut production,
        &mut state,
        &mut popup,
        &mut outbox,
        &mut runtime,
    );
}

pub(super) fn consume_user_store_outbox_0104(mut context: gameplay_ui_actions::gm_store::Context) {
    gameplay_ui_actions::gm_store::pump(&mut context);
}

pub(super) fn reset_user_store_session_0104(
    mut production: ResMut<UserStoreProductionRuntime0104>,
    mut state: ResMut<UserStoreUiState0104>,
    mut authority: ResMut<UserStoreAuthority0104>,
    mut popup: ResMut<UserStorePopupPresentation0104>,
    mut outbox: ResMut<UserStoreUiOutbox0104>,
) {
    production.reset();
    *state = UserStoreUiState0104::default();
    *authority = UserStoreAuthority0104::default();
    *popup = UserStorePopupPresentation0104::default();
    outbox.clear();
}

pub(super) const CASHMALL_CHAT_MAX_USER_LEVEL_0104: i16 = 50;

pub(super) fn is_cashmall_hidden_chat_command_0104(message: &str, user_level: i16) -> bool {
    // Clean `CnGuiChat.CheckCheatKey` first rejects account levels above 50,
    // then calls `ChatString.Split(' ')` and compares element zero with a
    // case-sensitive String equality. Preserve empty tokens and do not trim.
    user_level <= CASHMALL_CHAT_MAX_USER_LEVEL_0104
        && message.split(' ').next() == Some(CASHMALL_HIDDEN_CHAT_COMMAND_0104)
}
