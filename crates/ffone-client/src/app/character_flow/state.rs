use super::*;

pub(in super::super) fn character_slots_from_runtime(
    characters: &[CharacterSummary],
) -> [CharacterSlotUi; 4] {
    let mut slots = std::array::from_fn(|_| CharacterSlotUi::Empty);
    for character in characters {
        let Some(zero_based_slot) = character.slot.checked_sub(1) else {
            continue;
        };
        let Ok(index) = usize::try_from(zero_based_slot) else {
            continue;
        };
        if let Some(slot) = slots.get_mut(index) {
            *slot = character_summary_ui_slot(character);
        }
    }
    slots
}

pub(in super::super) fn sync_character_selection_ui(
    runtime: Res<RuntimeStatus>,
    inventory: Res<LocalInventoryRuntime>,
    options: Res<OptionProductionRuntime>,
    state: Res<State<ClientState>>,
    data: Res<LoadedCharacterCreationData>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut model: ResMut<CharacterSelectionUiModel>,
    mut portraits: ResMut<CharacterSelectionPortraitsModel>,
    mut gameplay_portrait: ResMut<GameplayPlayerPortraitModel>,
) {
    model.visible = *state.get() == ClientState::CharacterSelect;
    model.create = CharacterSelectionCapability::Enabled;
    model.delete = CharacterSelectionCapability::Enabled;
    model.preview = CharacterPreviewStatus::PlayerAssemblyPending;
    let window = windows.single().ok();
    model.fullscreen = window.is_some_and(|window| !matches!(window.mode, WindowMode::Windowed));
    model.ui_scale = window.map_or(1.0, |window| options.effective_ui_scale(window.height()));
    model.set_slots(
        character_slots_from_runtime(&runtime.roster.characters),
        runtime.roster.selected_uid,
    );
    gameplay_portrait.visible = matches!(*state.get(), ClientState::Tutorial | ClientState::World);
    gameplay_portrait.slot = gameplay_portrait.visible.then_some(()).and_then(|()| {
        runtime
            .roster
            .selected_uid
            .and_then(|uid| {
                runtime
                    .roster
                    .characters
                    .iter()
                    .find(|character| character.pc_uid == uid)
            })
            .and_then(|character| character.slot.checked_sub(1))
            .and_then(|slot| usize::try_from(slot).ok())
            .filter(|slot| *slot < 4)
    });
    portraits.visible = model.visible;
    // The roster is a selection-time snapshot. The independent HUD actor must
    // follow the same server-owned equipment as the inventory preview, even
    // while the inventory UI is closed. Keep other account slots untouched.
    let gameplay_character = gameplay_portrait
        .slot
        .and_then(|_| inventory.snapshot())
        .map(|inventory| authoritative_user_equip_preview_character(&runtime, inventory));
    for slot in 0..4 {
        let protocol_slot = i8::try_from(slot + 1).expect("four portrait slots fit protocol i8");
        let Some(character) = runtime
            .roster
            .characters
            .iter()
            .find(|character| character.slot == protocol_slot)
        else {
            portraits
                .clear_slot(slot)
                .expect("fixed portrait slot index");
            continue;
        };
        let character = if gameplay_portrait.slot == Some(slot) {
            match gameplay_character.as_ref() {
                Some(Ok(character)) => character,
                Some(Err(error)) => {
                    portraits
                        .block_slot(slot, error.clone())
                        .expect("fixed portrait slot index");
                    continue;
                }
                None => character,
            }
        } else {
            character
        };
        match data.0.resolve_character_summary(character) {
            Ok(look) => {
                if let Err(error) = portraits.set_look(slot, look) {
                    portraits
                        .block_slot(slot, error)
                        .expect("fixed portrait slot index");
                }
            }
            Err(error) => {
                let error = error.to_string();
                if portraits.slots[slot].status
                    != CharacterSelectionPortraitStatus::Blocked(error.clone())
                {
                    warn!(
                        "character-selection portrait slot {} (uid {}) blocked: {error}",
                        slot + 1,
                        character.pc_uid
                    );
                }
                portraits
                    .block_slot(slot, error)
                    .expect("fixed portrait slot index");
            }
        }
    }
}

pub(in super::super) fn handle_character_selection_ui_actions(
    state: Res<State<ClientState>>,
    mut outbox: ResMut<CharacterSelectionUiOutbox>,
    mut loading: ResMut<GameplayLoadingState>,
    mut buffered_entry: ResMut<BufferedCharacterEntry>,
    mut model: ResMut<CharacterSelectionUiModel>,
    mut preview: ResMut<NativePlayerPreviewModel>,
    bridge: Res<NetworkBridge>,
    mut runtime: ResMut<RuntimeStatus>,
    mut creation_session: ResMut<CharacterCreationSession>,
    mut tutorial_session: ResMut<TutorialSession>,
    mut creation_ui: ResMut<CharacterCreationUiModel>,
    mut option_runtime: ResMut<OptionProductionRuntime>,
    mut next_state: ResMut<NextState<ClientState>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut exit: MessageWriter<AppExit>,
) {
    if *state.get() != ClientState::CharacterSelect {
        buffered_entry.clear();
        outbox.drain().for_each(drop);
        return;
    }
    if loading.visible {
        if loading.scope != Some(ResourceLoadingScope::CharacterSelection) {
            buffered_entry.clear();
        }
        for action in outbox.drain() {
            if action == CharacterSelectionUiAction::EnterSelected
                && let Some(pc_uid) = model.selected_character().map(|character| character.pc_uid)
            {
                buffered_entry.queue(pc_uid, &loading);
            }
        }
        return;
    }

    if let Some(pc_uid) = buffered_entry.take_when_ready(&loading) {
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
                &mut loading,
                &mut creation_session,
                &mut creation_ui,
                &mut preview,
                &mut tutorial_session,
                &mut next_state,
            );
            return;
        }
    }
    for action in outbox.drain() {
        if loading.visible {
            continue;
        }
        match action {
            CharacterSelectionUiAction::SelectSlot(index) => {
                if let Some(character) = model.slots.get(index).and_then(CharacterSlotUi::occupied)
                {
                    runtime.roster.selected_uid = Some(character.pc_uid);
                    runtime.message = format!(
                        "Selected {} (UID {})",
                        character.display_name, character.pc_uid
                    );
                }
            }
            CharacterSelectionUiAction::EnterSelected => {
                if let Some(pc_uid) = model.selected_character().map(|character| character.pc_uid) {
                    let character = runtime
                        .roster
                        .characters
                        .iter()
                        .find(|character| character.pc_uid == pc_uid)
                        .cloned();
                    if let Some(character) = character {
                        request_character_entry(
                            &character,
                            &bridge,
                            &mut runtime,
                            &mut loading,
                            &mut creation_session,
                            &mut creation_ui,
                            &mut preview,
                            &mut tutorial_session,
                            &mut next_state,
                        );
                    }
                }
            }
            CharacterSelectionUiAction::CreateCharacter { slot } => {
                let Ok(slot) = u8::try_from(slot) else {
                    runtime.message = format!("Invalid character slot {slot}");
                    continue;
                };
                creation_session.reset();
                preview.yaw_degrees = 0.0;
                preview.camera_distance = NATIVE_PLAYER_CREATION_CAMERA_DISTANCE;
                let mut next_creator = CharacterCreationUiModel::default();
                next_creator.slot = Some(slot);
                next_creator.fullscreen = windows
                    .single()
                    .is_ok_and(|window| !matches!(window.mode, WindowMode::Windowed));
                next_creator.music_enabled = model.music_enabled;
                next_creator.ui_scale = model.ui_scale;
                next_creator.reserve_name = CharacterCreationCapability::Enabled;
                // OpenFusion performs the authoritative name/content check.
                // A native copy of the old local SlangChecker remains a
                // separate parity task, but custom-name creation is not
                // replaced by a permissive local-only decision.
                next_creator.custom_name_filter = CharacterCreationCapability::Enabled;
                *creation_ui = next_creator;
                runtime.message = format!("Starting Dexter's introduction for slot {slot}");
                next_state.set(ClientState::CharacterCreateIntro);
            }
            CharacterSelectionUiAction::DeleteSelected { pc_uid } => {
                runtime.message = format!("Deleting character UID {pc_uid}...");
                if let Err(error) = bridge.send(NetworkCommand::DeleteCharacter { pc_uid }) {
                    runtime.message = error;
                    model.reject_delete_request();
                }
            }
            CharacterSelectionUiAction::ToggleMusic(enabled) => {
                runtime.message = if enabled {
                    "Character selection music enabled".to_owned()
                } else {
                    "Character selection music disabled".to_owned()
                };
            }
            CharacterSelectionUiAction::ToggleFullscreen => {
                if let Ok(mut window) = windows.single_mut() {
                    window.mode = if matches!(window.mode, WindowMode::Windowed) {
                        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
                    } else {
                        WindowMode::Windowed
                    };
                    let windowed = matches!(window.mode, WindowMode::Windowed);
                    model.fullscreen = !windowed;
                    option_runtime.options.graphics.windowed = windowed;
                }
            }
            CharacterSelectionUiAction::Quit => {
                let _ = bridge.send(NetworkCommand::Disconnect);
                exit.write(AppExit::Success);
            }
        }
    }
}
