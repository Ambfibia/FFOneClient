//! Quit menu destinations, context and outboxes.

use super::guide::GuideProductionRuntime;
use super::login::ActiveLoginCredentials;
use super::nano_free_tuning::NanoFreeTuningProductionRuntime;
use super::race::RaceProductionRuntime;
use super::runtime_status::RuntimeStatus;
use super::state::ClientState;
use bevy::{audio::Volume, ecs::system::SystemParam, prelude::*};
use ffone_client::{
    bank_runtime::BankProductionRuntime0104,
    bank_ui::{BankLifecyclePhase, BankUiOutbox0104, BankUiState},
    buddy_ui::BuddyUiModel,
    cashmall_ui::{CashmallLifecyclePhase0104, CashmallUiState0104},
    combi_runtime::CombiProductionRuntime0104,
    email_runtime::EmailProductionRuntime0104,
    enchant_runtime::EnchantProductionRuntime0104,
    gameplay_ui::GameplayUiModel,
    guide_ui::{GuideUiModel, GuideUiOutbox},
    launcher_ui::LauncherUiModel,
    nano_free_tuning_ui::NanoFreeTuningModel,
    nanocom_message_ui::NanocomMessageUiModel,
    network::{NetworkBridge, NetworkCommand},
    option_ui::OptionUiModel,
    quit_menu_runtime::{QuitMenuDestination, QuitMenuRuntime},
    quit_menu_ui::{
        QuitMenuAudioOutbox, QuitMenuUiAction, QuitMenuUiModel, QuitMenuUiOutbox,
        dismiss_quit_menu_with_escape,
    },
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

pub(super) fn reset_quit_menu_shell(
    model: &mut QuitMenuUiModel,
    outbox: &mut QuitMenuUiOutbox,
    runtime: &mut QuitMenuRuntime,
) {
    model.close();
    model.set_enabled(true);
    outbox.clear();
    runtime.reset();
}

pub(super) fn complete_quit_menu_destination(
    destination: QuitMenuDestination,
    bridge: &NetworkBridge,
    active_credentials: &mut ActiveLoginCredentials,
    runtime: &mut RuntimeStatus,
    next_state: &mut NextState<ClientState>,
    app_exit: &mut MessageWriter<AppExit>,
) -> Result<(), String> {
    match destination {
        QuitMenuDestination::ChangeCharacter => {
            runtime.message = "Returning to character selection...".to_owned();
            bridge
                .send(NetworkCommand::ReturnToCharacterSelection)
                .map_err(|error| {
                    next_state.set(ClientState::Login);
                    error
                })
        }
        QuitMenuDestination::QuitGame => {
            runtime.message = "OpenFusion exit confirmed; closing FFOne.".to_owned();
            app_exit.write(AppExit::Success);
            Ok(())
        }
        QuitMenuDestination::QuitAndLogout => {
            active_credentials.credentials = None;
            runtime.message = "OpenFusion logout confirmed; closing FFOne.".to_owned();
            app_exit.write(AppExit::Success);
            Ok(())
        }
    }
}

pub(super) fn sync_quit_menu_context(
    state: Res<State<ClientState>>,
    system_messages: Res<SystemMessageUiModel>,
    runtime: Res<QuitMenuRuntime>,
    mut model: ResMut<QuitMenuUiModel>,
) {
    if *state.get() != ClientState::World {
        model.close();
    }
    model.set_enabled(!runtime.is_waiting_for_server() && !system_messages.is_popup());
}

pub(super) fn close_quit_menu_from_gamepad(
    pad: Option<Res<super::GamepadActionState>>,
    mut model: ResMut<QuitMenuUiModel>,
    mut outbox: ResMut<QuitMenuUiOutbox>,
) {
    if model.visible
        && pad.as_ref().is_some_and(|pad| {
            pad.just_pressed(ffone_client::option_ui::LegacyOptionAction::Escape)
        })
    {
        dismiss_quit_menu_with_escape(&mut model, &mut outbox);
    }
}

pub(super) fn open_quit_menu_from_nanocom(
    model: &mut QuitMenuUiModel,
    runtime: &QuitMenuRuntime,
    another_modal_owns_input: bool,
) -> bool {
    if another_modal_owns_input || model.visible || runtime.is_waiting_for_server() {
        return false;
    }
    // This is the mode-23 transition itself, not an exit destination. Opening
    // the model lets the existing visibility/audio owner observe one normal
    // edge without installing a fabricated network request.
    model.open();
    true
}

#[derive(SystemParam)]
pub(super) struct QuitMenuOpenInputs<'w> {
    pub(super) gameplay_ui: Res<'w, GameplayUiModel>,
    pub(super) launcher_ui: Res<'w, LauncherUiModel>,
    pub(super) buddy_ui: Res<'w, BuddyUiModel>,
    pub(super) system_messages: Res<'w, SystemMessageUiModel>,
    pub(super) nanocom_messages: Res<'w, NanocomMessageUiModel>,
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
    pub(super) option_ui: Res<'w, OptionUiModel>,
    pub(super) model: Res<'w, QuitMenuUiModel>,
}

pub(super) fn capture_quit_menu_open_intent(
    state: Res<State<ClientState>>,
    keys: Res<ButtonInput<KeyCode>>,
    pad_focus: Res<super::gamepad_ui::PadUiFocus>,
    inputs: QuitMenuOpenInputs,
    mut runtime: ResMut<QuitMenuRuntime>,
) {
    let QuitMenuOpenInputs {
        gameplay_ui,
        launcher_ui,
        buddy_ui,
        system_messages,
        nanocom_messages,
        resurrect_ui,
        guide_ui,
        guide_outbox,
        guide_production,
        bank_state,
        bank_outbox,
        bank_production,
        vendor_state,
        vendor_outbox,
        vendor_production,
        rule_ui,
        rule_runtime,
        nano_free_tuning,
        nano_free_tuning_production,
        upsell_ui,
        upsell_outbox,
        user_equip_ui,
        world_map,
        transportation,
        race_production,
        email_runtime,
        combi_runtime,
        enchant_runtime,
        cashmall_ui,
        user_store_ui,
        option_ui,
        model,
    } = inputs;
    let chat_active =
        gameplay_ui.visible && gameplay_ui.chat.input_enabled && gameplay_ui.chat.active;
    let requested = *state.get() == ClientState::World
        && keys.just_pressed(KeyCode::Escape)
        && !pad_focus.escape_injected
        && !model.visible
        && !runtime.is_waiting_for_server()
        && !chat_active
        && !launcher_ui.visible()
        && !buddy_ui.add_dialog_open()
        && !system_messages.is_popup()
        && !nanocom_messages.expanded_visible()
        && !resurrect_ui.visible
        && !guide_production.modal_active(&guide_ui)
        && guide_outbox.is_empty()
        && bank_state.phase == BankLifecyclePhase::Hidden
        && !bank_production.modal_active()
        && bank_outbox.is_empty()
        && vendor_state.phase == VendorLifecyclePhase::Hidden
        && !vendor_production.modal_active()
        && vendor_outbox.is_empty()
        && !rule_runtime.modal_active(&rule_ui)
        && !nano_free_tuning_production.modal_active(&nano_free_tuning)
        && !upsell_ui.visible()
        && upsell_outbox.is_empty()
        && !user_equip_ui.is_active()
        && world_map.model.phase() == WorldMapPhase::Closed
        && transportation
            .as_ref()
            .is_none_or(|model| model.phase() == TransportationPhase::Hidden)
        && !race_production.modal_active()
        && !email_runtime.modal_active()
        && !combi_runtime.modal_active()
        && !enchant_runtime.is_active()
        && cashmall_ui
            .as_ref()
            .is_none_or(|state| state.phase() == CashmallLifecyclePhase0104::Hidden)
        && user_store_ui.as_ref().is_none_or(|state| !state.active)
        && !option_ui.visible;
    runtime.capture_open_intent(requested);
}

pub(super) fn consume_quit_menu_ui_outbox(
    bridge: Res<NetworkBridge>,
    mut next_state: ResMut<NextState<ClientState>>,
    mut model: ResMut<QuitMenuUiModel>,
    mut outbox: ResMut<QuitMenuUiOutbox>,
    mut runtime: ResMut<QuitMenuRuntime>,
    mut status: ResMut<RuntimeStatus>,
) {
    if runtime.exit_timed_out() {
        let destination = runtime.finish();
        model.set_enabled(true);
        outbox.clear();
        status.message = format!("Timed out waiting for {:?} PC-exit reply", destination);
        let _ = bridge.send(NetworkCommand::Disconnect);
        next_state.set(ClientState::Login);
    }
    if runtime.take_open_intent() {
        model.open();
    }
    while let Some(action) = outbox.pop_front() {
        let destination = match action {
            QuitMenuUiAction::ChangeCharacter => QuitMenuDestination::ChangeCharacter,
            QuitMenuUiAction::QuitGame => QuitMenuDestination::QuitGame,
            QuitMenuUiAction::QuitAndLogout => QuitMenuDestination::QuitAndLogout,
            QuitMenuUiAction::Cancel { .. } => continue,
        };
        if !runtime.begin(destination) {
            continue;
        }
        model.set_enabled(false);
        status.message = format!("Requesting {:?} from OpenFusion...", destination);
        if let Err(error) = bridge.send(NetworkCommand::ExitWorld) {
            runtime.finish();
            model.set_enabled(true);
            status.message = format!("QuitMenu PC-exit request failed: {error}");
        }
    }
}

pub(super) fn consume_quit_menu_audio_outbox(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut outbox: ResMut<QuitMenuAudioOutbox>,
) {
    while let Some(cue) = outbox.pop_front() {
        commands.spawn((
            Name::new(format!("QuitMenu audio {}", cue.path())),
            ffone_client::audio_channel::GameplayAudioChannel::ui_sfx(),
            AudioPlayer::new(asset_server.load(cue.path())),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(cue.gain())),
        ));
    }
}
