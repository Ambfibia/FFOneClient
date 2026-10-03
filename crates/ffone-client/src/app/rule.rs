//! Rule UI context and runtime.

use super::runtime_status::RuntimeStatus;
use bevy::{audio::Volume, prelude::*};
use ffone_client::{
    gameplay_audio::GameplayAudioRuntime,
    gameplay_ui::GameplayUiModel,
    mission_ui::MissionUiModel,
    rule_runtime::{RuleRuntime, RuleRuntimeCommand, RuleRuntimeSound},
    rule_ui::{RuleUiAudioOutbox, RuleUiModel, RuleUiOutbox},
    system_message_ui::SystemMessageUiModel,
};
use std::collections::VecDeque;

pub(super) const RULE_OPEN_SCREEN_AUDIO_PATH: &str = "audio/sfx/ui/open_screen.ogg";
pub(super) const RULE_CLOSE_SCREEN_AUDIO_PATH: &str = "audio/sfx/ui/close_screen.ogg";

pub(super) fn sync_rule_ui_context(
    gameplay_ui: Res<GameplayUiModel>,
    system_messages: Res<SystemMessageUiModel>,
    runtime: Res<RuleRuntime>,
    mut model: ResMut<RuleUiModel>,
) {
    model.set_ui_scale(gameplay_ui.ui_scale);
    // FFOne does not yet own a separate Help/Computress surface. The system
    // popup is real and retains clean Rule's stronger pointer/Escape gate.
    runtime.sync_external_gates(&mut model, system_messages.is_popup(), false);
}

pub(super) fn consume_rule_runtime(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    system_messages: Res<SystemMessageUiModel>,
    mut mission_ui: ResMut<MissionUiModel>,
    mut model: ResMut<RuleUiModel>,
    mut outbox: ResMut<RuleUiOutbox>,
    mut audio: ResMut<RuleUiAudioOutbox>,
    mut rule_runtime: ResMut<RuleRuntime>,
    audio_runtime: Option<ResMut<GameplayAudioRuntime>>,
    mut status: ResMut<RuntimeStatus>,
) {
    let mut audio_runtime = audio_runtime;
    let errors = rule_runtime.flush_ui_outboxes(&model, &mut outbox, &mut audio);
    if let Some(error) = errors.last() {
        status.message = format!("RuleMode ignored invalid UI route: {error:?}");
    }

    let mut pending = rule_runtime.drain_commands().collect::<VecDeque<_>>();
    while let Some(command) = pending.pop_front() {
        match command {
            RuleRuntimeCommand::EnterMode(_) => {}
            RuleRuntimeCommand::PlaySound(sound) => {
                let path = match sound {
                    RuleRuntimeSound::OpenScreen => Some(RULE_OPEN_SCREEN_AUDIO_PATH),
                    RuleRuntimeSound::CloseScreen => Some(RULE_CLOSE_SCREEN_AUDIO_PATH),
                    RuleRuntimeSound::RandomButtonSound => {
                        if let Some(audio_runtime) = audio_runtime.as_deref_mut() {
                            audio_runtime.queue_legacy_button_sound();
                        }
                        None
                    }
                };
                if let Some(path) = path {
                    commands.spawn((
                        Name::new(format!("RuleMode audio {path}")),
                        ffone_client::audio_channel::GameplayAudioChannel::ui_sfx(),
                        AudioPlayer::new(asset_server.load(path)),
                        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(sound.gain())),
                    ));
                }
            }
            RuleRuntimeCommand::RequestComputressExitGate { token, .. } => {
                // In clean Retrobution an absent Computress owner returns
                // false from OnComputressExitPressed, which GameFrame maps to
                // an accepted local `2/24` return. A real popup still rejects
                // fail-closed if it appeared between input and consumption.
                let accepted = !system_messages.is_popup();
                if let Err(error) = rule_runtime.resolve_escape_gate(&mut model, token, accepted) {
                    status.message = format!("RuleMode Computress exit gate rejected: {error:?}");
                }
                pending.extend(rule_runtime.drain_commands());
            }
            RuleRuntimeCommand::ExitMode(exit) => {
                if exit.clear_npc_interaction_locally {
                    mission_ui.clear_npc_interaction_locally();
                }
                debug_assert!(!exit.send_npc_interaction_close_packet);
            }
        }
    }
}

pub(super) fn reset_rule_session(
    mut runtime: ResMut<RuleRuntime>,
    mut model: ResMut<RuleUiModel>,
    mut outbox: ResMut<RuleUiOutbox>,
    mut audio: ResMut<RuleUiAudioOutbox>,
) {
    runtime.reset_session(&mut model, &mut outbox, &mut audio);
}
