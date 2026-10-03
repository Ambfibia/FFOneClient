use super::*;

pub(in super::super) fn request_character_entry(
    character: &CharacterSummary,
    bridge: &NetworkBridge,
    runtime: &mut RuntimeStatus,
    loading: &mut GameplayLoadingState,
    creation_session: &mut CharacterCreationSession,
    creation_ui: &mut CharacterCreationUiModel,
    preview: &mut NativePlayerPreviewModel,
    tutorial: &mut TutorialSession,
    next_state: &mut NextState<ClientState>,
) {
    if let Some(pending_uid) = runtime.roster.pending_character_entry_uid {
        runtime.message = if pending_uid == character.pc_uid {
            format!(
                "Character {} is already entering the shard...",
                character.pc_uid
            )
        } else {
            format!("Character {pending_uid} is already entering the shard...")
        };
        return;
    }
    runtime.roster.selected_uid = Some(character.pc_uid);
    match character_entry_route(character) {
        Ok(CharacterEntryRoute::ResumeAppearance) => {
            tutorial.clear();
            match resume_incomplete_character(character, creation_session, creation_ui) {
                Ok(()) => {
                    preview.yaw_degrees = 0.0;
                    preview.camera_distance = NATIVE_PLAYER_CREATION_CAMERA_DISTANCE;
                    runtime.message = "Resuming unfinished character appearance".to_owned();
                    next_state.set(ClientState::CharacterCreate);
                }
                Err(error) => runtime.message = error,
            }
        }
        Ok(CharacterEntryRoute::TutorialSequence) => {
            tutorial.clear();
            tutorial.character = Some(character.clone());
            runtime.message = format!(
                "Starting Dexter's time-capsule sequence for {}...",
                character.pc_uid
            );
            next_state.set(ClientState::TutorialIntro);
        }
        Ok(CharacterEntryRoute::ShardWorld) => {
            tutorial.clear();
            runtime.message = format!("Selecting character {}...", character.pc_uid);
            runtime.roster.pending_character_entry_uid = Some(character.pc_uid);
            match bridge.send(NetworkCommand::SelectCharacter {
                pc_uid: character.pc_uid,
                location: CharacterEntryLocation0104::Saved,
            }) {
                Ok(()) => loading.begin(ResourceLoadingScope::World),
                Err(error) => {
                    runtime.roster.pending_character_entry_uid = None;
                    runtime.message = error;
                }
            }
        }
        Err(value) => {
            runtime.message = format!(
                "Character {} has unsupported tutorial flag {value}; expected 0 or 1",
                character.pc_uid
            );
        }
    }
}

pub(super) fn submit_character_name_request(
    request: Result<CharacterNameCheckRequest0104, ffone_protocol::PayloadError>,
    slot: Option<u8>,
    bridge: &NetworkBridge,
    session: &mut CharacterCreationSession,
    runtime: &mut RuntimeStatus,
) {
    let request = match request {
        Ok(request) => request,
        Err(error) => {
            runtime.message = format!("Character name cannot be encoded: {error}");
            return;
        }
    };
    let Some(slot) = slot
        .and_then(|slot| i8::try_from(slot).ok())
        .filter(|slot| (1..=4).contains(slot))
    else {
        runtime.message =
            "Character name cannot be reserved: creator has no valid protocol slot".to_owned();
        return;
    };
    runtime.message = format!(
        "Checking and reserving name {} {} with OpenFusion...",
        request.first_name.to_string_lossy(),
        request.last_name.to_string_lossy()
    );
    session.pending_name_check = Some(request.clone());
    // The check and reservation belong to one user action. Keeping both packets
    // on the worker prevents Bevy frame/state changes from losing the second
    // half of the legacy OpenFusion handshake.
    if let Err(error) = bridge.send(NetworkCommand::ReserveCharacterName {
        request,
        slot,
        gender: 1,
    }) {
        session.pending_name_check = None;
        runtime.message = error;
    }
}
