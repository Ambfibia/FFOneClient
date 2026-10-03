use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch_network_events(
    events: impl IntoIterator<Item = NetworkEvent>,
    mut commands: Commands,
    config: Res<ClientConfig>,
    bridge: Res<NetworkBridge>,
    mut runtime: ResMut<RuntimeStatus>,
    mut selection_ui: ResMut<CharacterSelectionUiModel>,
    mut creation_session: ResMut<CharacterCreationSession>,
    mut tutorial_session: ResMut<TutorialSession>,
    mut creation_ui: ResMut<CharacterCreationUiModel>,
    mut next_state: ResMut<NextState<ClientState>>,
    mut lifecycle_ingress: ResMut<NetworkEntityLifecycleIngress0104>,
    mut lifecycle_session: ResMut<NetworkLifecycleSession>,
    asset_server: Res<AssetServer>,
    world_catalog: Res<NativeWorldCatalog>,
    character_data: Res<LoadedCharacterCreationData>,
    rig_catalog: Res<NativePlayerRigCatalog>,
    mut world_spawn: WorldSpawnRuntime,
    world_slice: Query<Entity, With<WorldSliceEntity>>,
) {
    for event in events {
        match event {
            NetworkEvent::Connecting => {
                // Clear login metadata in event order, before a later LoginMetadata
                // in the same poll can install the new session entitlement.
                world_spawn.npc_modes.guide_production.reset_session();
                world_spawn
                    .session
                    .music_requests
                    .request("stop".into(), false);
                world_spawn.session.world_combat.clear();
                world_spawn.session.instance_audio.active = false;
                if let Some(epoch) = lifecycle_session.take() {
                    lifecycle_ingress.disconnect(epoch);
                }
                world_spawn
                    .session
                    .session_resets
                    .write(NetworkSessionReset::new(NetworkSessionBoundary::Connecting));
                runtime.clear_world();
                world_spawn.world_modes.world_nano_authority.clear();
                runtime.roster = default();
                selection_ui.reset_delete_confirmation();
                creation_session.reset();
                runtime.message = format!("Connecting to {}...", config.login_address);
                next_state.set(ClientState::Login);
            }
            NetworkEvent::LoginMetadata { payment_flag } => {
                world_spawn.npc_modes.guide_production.payment_flag = Some(payment_flag);
            }
            NetworkEvent::ReturnedToCharacterSelection(characters) => {
                // The retained login connection only knows the entry location.
                // Capture the departing controller before its world is removed.
                if let Ok((_, transform, _, _, _)) = world_spawn.queries.local_player.single() {
                    runtime.retain_local_position(
                        ProtocolPosition::from_native(transform.translation).raw(),
                    );
                }
                // Only the shard closed: the worker retains the authenticated
                // login session and does not send another LoginMetadata event.
                world_spawn.npc_modes.guide_production.reset_world();
                world_spawn
                    .session
                    .music_requests
                    .request("stop".into(), false);
                world_spawn.session.world_combat.clear();
                world_spawn.session.instance_audio.active = false;
                world_spawn.session.gameplay_loading.finish();
                world_spawn
                    .session
                    .session_resets
                    .write(NetworkSessionReset::new(
                        NetworkSessionBoundary::Disconnected,
                    ));
                if let Some(epoch) = lifecycle_session.take() {
                    lifecycle_ingress.disconnect(epoch);
                }
                runtime.clear_world();
                world_spawn.world_modes.world_nano_authority.clear();
                for entity in &world_slice {
                    commands.entity(entity).despawn();
                }
                world_spawn
                    .session
                    .gameplay_loading
                    .begin(ResourceLoadingScope::CharacterSelection);
                runtime.roster.selected_uid = characters
                    .iter()
                    .min_by_key(|character| character.slot)
                    .map(|character| character.pc_uid);
                runtime.message = format!("Received {} character(s)", characters.len());
                runtime.roster.replace_retained_login_characters(characters);
                for character in &runtime.roster.characters {
                    match character_data
                        .0
                        .resolve_character_summary(character)
                        .map_err(|error| error.to_string())
                        .and_then(|look| {
                            prewarm_native_player_look(
                                &asset_server,
                                &mut world_spawn.session.rig_assets,
                                &rig_catalog,
                                &look,
                            )
                        }) {
                        Ok(()) => {}
                        Err(error) => warn!(
                            "character-selection avatar {} could not be prewarmed: {error}",
                            character.pc_uid
                        ),
                    }
                }
                next_state.set(ClientState::CharacterSelect);
            }
            NetworkEvent::Characters(characters) => {
                let pending_entry_uid = runtime.roster.pending_character_entry_uid;
                let pending_created_uid = creation_session.pending_created_character_entry_uid;
                if let Some(pc_uid) = pending_entry_uid {
                    runtime.message =
                        format!("Character {pc_uid} is already entering the shard...");
                } else {
                    world_spawn
                        .session
                        .gameplay_loading
                        .begin(ResourceLoadingScope::CharacterSelection);
                    runtime.message = format!("Received {} character(s)", characters.len());
                }
                runtime.roster.replace_retained_login_characters(characters);
                for character in &runtime.roster.characters {
                    match character_data
                        .0
                        .resolve_character_summary(character)
                        .map_err(|error| error.to_string())
                        .and_then(|look| {
                            prewarm_native_player_look(
                                &asset_server,
                                &mut world_spawn.session.rig_assets,
                                &rig_catalog,
                                &look,
                            )
                        }) {
                        Ok(()) => {}
                        Err(error) => warn!(
                            "character-selection avatar {} could not be prewarmed: {error}",
                            character.pc_uid
                        ),
                    }
                }
                runtime.roster.selected_uid =
                    pending_entry_uid.or(pending_created_uid).or_else(|| {
                        runtime
                            .roster
                            .characters
                            .iter()
                            .min_by_key(|character| character.slot)
                            .map(|character| character.pc_uid)
                    });
                if pending_entry_uid.is_none() {
                    match creation_session
                        .take_created_character_from_roster(&runtime.roster.characters)
                    {
                        Ok(Some(character)) => {
                            request_character_entry(
                                &character,
                                &bridge,
                                &mut runtime,
                                &mut world_spawn.session.gameplay_loading,
                                &mut creation_session,
                                &mut creation_ui,
                                &mut world_spawn.session.character_preview,
                                &mut tutorial_session,
                                &mut next_state,
                            );
                            continue;
                        }
                        Ok(None) => {}
                        Err(pc_uid) => {
                            runtime.roster.selected_uid = runtime
                                .roster
                                .characters
                                .iter()
                                .min_by_key(|character| character.slot)
                                .map(|character| character.pc_uid);
                            runtime.message = format!("Character UID {pc_uid} was not returned");
                            next_state.set(ClientState::CharacterSelect);
                            continue;
                        }
                    }
                    next_state.set(ClientState::CharacterSelect);
                    if let Some(pc_uid) = config.character_uid {
                        if let Some(character) = runtime
                            .roster
                            .characters
                            .iter()
                            .find(|character| character.pc_uid == pc_uid)
                            .cloned()
                        {
                            request_character_entry(
                                &character,
                                &bridge,
                                &mut runtime,
                                &mut world_spawn.session.gameplay_loading,
                                &mut creation_session,
                                &mut creation_ui,
                                &mut world_spawn.session.character_preview,
                                &mut tutorial_session,
                                &mut next_state,
                            );
                        } else {
                            runtime.message = format!("Character UID {pc_uid} was not returned");
                        }
                    }
                }
            }
            NetworkEvent::CharacterNameChecked(checked) => {
                let Some(request) = creation_session.pending_name_check.as_ref() else {
                    runtime.message =
                        "OpenFusion returned a name check without a pending creator request"
                            .to_owned();
                    continue;
                };
                let Some(slot) = creation_ui.slot.and_then(|slot| i8::try_from(slot).ok()) else {
                    runtime.message =
                        "Character name was checked, but the creator has no protocol slot"
                            .to_owned();
                    continue;
                };
                // The original 0104 client always reserves the name with male
                // gender first. The appearance screen may change it before
                // `P_CL2LS_REQ_CHAR_CREATE`.
                let save = ffone_protocol::CharacterNameSaveRequest0104::from_check(
                    slot, 1, request, &checked,
                );
                runtime.message = format!(
                    "Name {} {} accepted; reserving slot {}...",
                    checked.first_name.to_string_lossy(),
                    checked.last_name.to_string_lossy(),
                    slot
                );
                if let Err(error) = bridge.send(NetworkCommand::SaveCharacterName(save)) {
                    runtime.message = error;
                }
            }
            NetworkEvent::CharacterNameSaved(saved) => {
                character_flow::accept_reserved_character_name(
                    saved,
                    &mut creation_session,
                    &mut creation_ui,
                    &mut world_spawn.session.gameplay_loading,
                    &mut runtime,
                );
            }
            NetworkEvent::CharacterCreated { slot, response } => {
                world_spawn
                    .session
                    .gameplay_loading
                    .begin(ResourceLoadingScope::CharacterSelection);
                runtime.roster.selected_uid = Some(response.style.pc_uid);
                runtime.message = format!(
                    "Character {} {} created in slot {}",
                    response.style.first_name.to_string_lossy(),
                    response.style.last_name.to_string_lossy(),
                    slot
                );
                creation_session.await_created_character_roster(response.style.pc_uid);
            }
            NetworkEvent::CharacterDeleted { pc_uid, response } => {
                accept_character_delete(&mut runtime, &mut selection_ui, pc_uid);
                runtime.message =
                    format!("Deleted character UID {pc_uid} from slot {}", response.slot);
            }
            NetworkEvent::CharacterNameChanged(response) => {
                runtime.roster.selected_uid = Some(response.pc_uid);
                runtime.message = format!(
                    "Character renamed to {} {} in slot {}",
                    response.first_name.to_string_lossy(),
                    response.last_name.to_string_lossy(),
                    response.slot
                );
            }
            NetworkEvent::DuplicateSessionExitRequested => {
                runtime.message = "Previous account session disconnect requested".to_owned();
            }
            NetworkEvent::CharacterOperationRejected { stage, error_code } => {
                if matches!(
                    stage,
                    CharacterOperationStage::NameCheck | CharacterOperationStage::NameSave
                ) {
                    creation_session.pending_name_check = None;
                }
                if stage == CharacterOperationStage::Delete {
                    selection_ui.reject_delete_request();
                }
                let operation = match stage {
                    CharacterOperationStage::NameCheck => "name check",
                    CharacterOperationStage::NameSave => "name reservation",
                    CharacterOperationStage::Appearance => "appearance save",
                    CharacterOperationStage::Delete => "character deletion",
                    CharacterOperationStage::Rename => "character rename",
                };
                runtime.message =
                    format!("OpenFusion rejected {operation} with error {error_code}");
            }
            NetworkEvent::TutorialExitFailed { pc_uid, error } => {
                recover_tutorial_completion_to_character_selection(
                    &mut tutorial_session,
                    &mut runtime,
                    &mut next_state,
                    format!(
                        "Tutorial completion for character {pc_uid} failed; choose a character to retry: {error}"
                    ),
                );
                let _ = bridge.send(NetworkCommand::RefreshCharacters);
            }
            NetworkEvent::LoginFrame(_) => {
                // The retained login reader consumes heartbeat traffic. Any other
                // login frame remains classified here instead of entering gameplay.
            }
            NetworkEvent::MalformedFrame0104 {
                stream,
                frame,
                expected_payload_size,
            } => {
                runtime.message = format!(
                    "Rejected malformed {stream} packet 0x{:08x}: expected {} payload bytes, got {}",
                    frame.packet_type,
                    expected_payload_size,
                    frame.payload.len()
                );
            }
            NetworkEvent::EnteringWorld { pc_uid } => {
                runtime.roster.selected_uid = Some(pc_uid);
                runtime.message = format!("Entering OpenFusion world as {pc_uid}...");
            }
            NetworkEvent::WorldReady(world) => {
                handle_world_ready(
                    &mut commands,
                    &config,
                    &bridge,
                    &mut runtime,
                    &mut tutorial_session,
                    &mut next_state,
                    &mut lifecycle_ingress,
                    &mut lifecycle_session,
                    &asset_server,
                    &world_catalog,
                    &character_data,
                    &rig_catalog,
                    &mut world_spawn,
                    &world_slice,
                    world,
                );
                if let Some(inventory) = world_spawn.inventory.inventory.snapshot() {
                    runtime.retain_local_equipment(inventory);
                }
            }
            NetworkEvent::Frame(frame) => {
                handle_gameplay_frame(
                    &mut commands,
                    &bridge,
                    &mut runtime,
                    &mut next_state,
                    &mut lifecycle_ingress,
                    &mut lifecycle_session,
                    &asset_server,
                    &mut world_spawn,
                    tutorial_session.owns_local_hp(),
                    frame,
                );
                // Do this for each packet, not in a later render system: the
                // same poll can contain the final equip reply and menu return.
                if let Some(inventory) = world_spawn.inventory.inventory.snapshot() {
                    runtime.retain_local_equipment(inventory);
                }
            }
            NetworkEvent::Error(error) => {
                // Clear login metadata in event order, before a later LoginMetadata
                // in the same poll can install the new session entitlement.
                world_spawn.npc_modes.guide_production.reset_session();
                if world_spawn.npc_modes.quit_runtime.is_waiting_for_server() {
                    let _ = bridge.send(NetworkCommand::Disconnect);
                }
                world_spawn.npc_modes.quit_runtime.finish();
                world_spawn.npc_modes.quit_menu.set_enabled(true);
                world_spawn
                    .session
                    .music_requests
                    .request("stop".into(), false);
                world_spawn.session.instance_audio.active = false;
                selection_ui.reject_delete_request();
                let character_entry_failed =
                    runtime.roster.pending_character_entry_uid.take().is_some();
                if character_entry_failed && lifecycle_session.active.is_none() {
                    world_spawn.session.gameplay_loading.finish();
                    next_state.set(ClientState::CharacterSelect);
                }
                runtime.message = format!("Network error: {error}");
                world_spawn
                    .session
                    .session_resets
                    .write(NetworkSessionReset::new(NetworkSessionBoundary::Error));
                if let Some(epoch) = lifecycle_session.take() {
                    lifecycle_ingress.disconnect(epoch);
                    runtime.clear_world();
                    world_spawn.world_modes.world_nano_authority.clear();
                    world_spawn.session.gameplay_loading.finish();
                    next_state.set(ClientState::Login);
                    for entity in &world_slice {
                        commands.entity(entity).despawn();
                    }
                }
            }
            NetworkEvent::Disconnected { reason } => {
                // Clear login metadata in event order, before a later LoginMetadata
                // in the same poll can install the new session entitlement.
                world_spawn.npc_modes.guide_production.reset_session();
                world_spawn.npc_modes.quit_runtime.finish();
                world_spawn.npc_modes.quit_menu.set_enabled(true);
                world_spawn
                    .session
                    .music_requests
                    .request("stop".into(), false);
                world_spawn.session.world_combat.clear();
                world_spawn.session.instance_audio.active = false;
                selection_ui.reset_delete_confirmation();
                runtime.message = format!("Disconnected from OpenFusion: {reason}");
                runtime.clear_world();
                world_spawn.world_modes.world_nano_authority.clear();
                // WorldReady clears pending_character_entry_uid before assets
                // finish loading. A disconnect in that window still owns the
                // tutorial/world overlay and must release it before Login.
                world_spawn.session.gameplay_loading.finish();
                world_spawn
                    .session
                    .session_resets
                    .write(NetworkSessionReset::new(
                        NetworkSessionBoundary::Disconnected,
                    ));
                if let Some(epoch) = lifecycle_session.take() {
                    lifecycle_ingress.disconnect(epoch);
                }
                next_state.set(ClientState::Login);
                for entity in &world_slice {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

pub(in super::super) fn accept_character_delete(
    runtime: &mut RuntimeStatus,
    selection_ui: &mut CharacterSelectionUiModel,
    pc_uid: i64,
) {
    runtime
        .roster
        .characters
        .retain(|character| character.pc_uid != pc_uid);
    runtime.roster.world_snapshot_uids.remove(&pc_uid);
    if runtime.roster.selected_uid == Some(pc_uid) {
        runtime.roster.selected_uid = runtime
            .roster
            .characters
            .iter()
            .min_by_key(|character| character.slot)
            .map(|character| character.pc_uid);
    }
    selection_ui.complete_delete_request(pc_uid);
}

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn poll_network(
    commands: Commands,
    config: Res<ClientConfig>,
    bridge: Res<NetworkBridge>,
    roster: NetworkRosterIngress,
    creation_session: ResMut<CharacterCreationSession>,
    tutorial_session: ResMut<TutorialSession>,
    creation_ui: ResMut<CharacterCreationUiModel>,
    next_state: ResMut<NextState<ClientState>>,
    lifecycle_ingress: ResMut<NetworkEntityLifecycleIngress0104>,
    lifecycle_session: ResMut<NetworkLifecycleSession>,
    asset_server: Res<AssetServer>,
    world_catalog: Res<NativeWorldCatalog>,
    character_data: Res<LoadedCharacterCreationData>,
    rig_catalog: Res<NativePlayerRigCatalog>,
    world_spawn: WorldSpawnRuntime,
    world_slice: Query<Entity, With<WorldSliceEntity>>,
) {
    let events = bridge.drain();
    let NetworkRosterIngress {
        runtime,
        selection_ui,
    } = roster;
    dispatch_network_events(
        events,
        commands,
        config,
        bridge,
        runtime,
        selection_ui,
        creation_session,
        tutorial_session,
        creation_ui,
        next_state,
        lifecycle_ingress,
        lifecycle_session,
        asset_server,
        world_catalog,
        character_data,
        rig_catalog,
        world_spawn,
        world_slice,
    );
}
