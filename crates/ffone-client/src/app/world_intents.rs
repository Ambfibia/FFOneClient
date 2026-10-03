//! Movement/world gameplay intent flushing, NPC visibility and player interaction toggles.

use super::npc_warp::NormalNpcWarpRuntime;
use super::runtime_status::RuntimeStatus;
use super::state::ClientState;
use super::transportation::TransportationProductionRuntime;
use super::{LocalPlayer, TutorialSuppressedNetworkNpc, client_state_sends_movement_intents};
use bevy::prelude::*;
use ffone_client::{
    bank_runtime::BankProductionRuntime0104,
    combi_runtime::CombiProductionRuntime0104,
    email_runtime::EmailProductionRuntime0104,
    entity_lifecycle::{NetworkNpc0104, NetworkNpcAppearance0104},
    gameplay_audio::{GameplayAudioRuntime, LegacyNpcVoiceCue},
    gameplay_ui::GameplayUiAudioOutbox,
    movement::{LegacyWorldColliderPending, MovementIntent, MovementIntentQueue},
    network::{NetworkBridge, NetworkCommand},
    option_ui::OptionUiModel,
    tutorial_mission_content::TutorialMissionContent,
    world_behaviour::WorldGameplayIntentQueue,
};

pub(super) fn sync_tutorial_network_npc_visibility(
    mut commands: Commands,
    state: Res<State<ClientState>>,
    mut npcs: Query<
        (
            Entity,
            &mut Visibility,
            Option<&TutorialSuppressedNetworkNpc>,
        ),
        With<NetworkNpc0104>,
    >,
) {
    let tutorial_owns_presentation = *state.get() == ClientState::Tutorial;
    for (entity, mut visibility, suppressed) in &mut npcs {
        if tutorial_owns_presentation {
            *visibility = Visibility::Hidden;
            if suppressed.is_none() {
                commands.entity(entity).insert(TutorialSuppressedNetworkNpc);
            }
        } else if suppressed.is_some() {
            *visibility = Visibility::Inherited;
            commands
                .entity(entity)
                .remove::<TutorialSuppressedNetworkNpc>();
        }
    }
}

pub(super) fn flush_movement_intents(
    mut intents: ResMut<MovementIntentQueue>,
    bridge: Res<NetworkBridge>,
    state: Res<State<ClientState>>,
    transportation: Option<Res<TransportationProductionRuntime>>,
    normal_npc_warp: Option<Res<NormalNpcWarpRuntime>>,
    mut runtime: ResMut<RuntimeStatus>,
) {
    if !client_state_sends_movement_intents(*state.get())
        || transportation
            .as_ref()
            .is_some_and(|runtime| !runtime.movement_packet_emission)
        || normal_npc_warp
            .as_ref()
            .is_some_and(|runtime| !runtime.movement_packet_emission)
    {
        let _ = intents.take_all();
        return;
    }
    for intent in intents.take_all() {
        let command = match intent {
            MovementIntent::Move(request) => NetworkCommand::Move(request),
            MovementIntent::Stop(request) => NetworkCommand::Stop(request),
            MovementIntent::Jump(request) => NetworkCommand::Jump(request),
        };
        if let Err(error) = bridge.send(command) {
            runtime.message = error;
            break;
        }
    }
}

pub(super) fn flush_world_gameplay_intents(
    mut intents: ResMut<WorldGameplayIntentQueue>,
    bridge: Res<NetworkBridge>,
    state: Res<State<ClientState>>,
    mut runtime: ResMut<RuntimeStatus>,
) {
    if !client_state_sends_movement_intents(*state.get()) {
        let _ = intents.take_all();
        return;
    }
    for request in intents.take_all() {
        if let Err(error) = bridge.send(NetworkCommand::SendRegisteredGameplay0104(request)) {
            runtime.message = error;
            break;
        }
    }
}

pub(super) fn toggle_player_interaction(
    state: Res<State<ClientState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    option_ui: Res<OptionUiModel>,
    bank_production: Res<BankProductionRuntime0104>,
    email_runtime: Res<EmailProductionRuntime0104>,
    combi_runtime: Res<CombiProductionRuntime0104>,
    ready_players: Query<(), (With<LocalPlayer>, Without<LegacyWorldColliderPending>)>,
    mut runtime: ResMut<RuntimeStatus>,
    mut notice: ResMut<ffone_client::gameplay_ui::CombatModeNotice>,
    mut audio: ResMut<GameplayUiAudioOutbox>,
) {
    if matches!(state.get(), ClientState::Tutorial | ClientState::World)
        && ready_players.single().is_ok()
        && !option_ui.visible
        && !bank_production.modal_active()
        && !email_runtime.modal_active()
        && !combi_runtime.modal_active()
        && keyboard.just_pressed(KeyCode::F1)
    {
        runtime.allow_player_interaction = !runtime.allow_player_interaction;
        notice.show(!runtime.allow_player_interaction);
        audio.push(ffone_client::gameplay_ui::GameplayUiAudioCue::TabClick01);
    }
}

pub(super) fn queue_service_farewell(commands: &mut Commands, npc_id: i32) {
    commands.queue(move |world: &mut World| {
        let npc = world.query::<(Entity, &NetworkNpcAppearance0104)>().iter(world)
            .find(|(_, appearance)| appearance.0.npc_id == npc_id)
            .map(|(entity, appearance)| (entity, appearance.0.npc_type));
        let Some((entity, npc_type)) = npc else { return; };
        let owner = world.get_resource::<TutorialMissionContent>()
            .and_then(|content| content.gameplay_npc(npc_type))
            .map(|definition| definition.move_voice_owner.clone());
        if let Some(owner) = owner {
            if let Some(mut audio) = world.get_resource_mut::<GameplayAudioRuntime>() {
                audio.queue_legacy_npc_voice(entity, &owner, LegacyNpcVoiceCue::Farewell);
            }
        }
    });
}
