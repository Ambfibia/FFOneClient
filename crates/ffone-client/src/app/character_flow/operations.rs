use super::*;

pub(super) fn character_summary_ui_slot(character: &CharacterSummary) -> CharacterSlotUi {
    let style = character.style;
    let display_name = if style.name_check == 1 {
        format!("{} {}", character.first_name, character.last_name)
            .trim()
            .to_owned()
    } else {
        format!("Player {}", character.pc_uid)
    };
    let (district, zone, background) = if style.appearance_flag == 0 {
        (
            "CHARACTER CREATION".to_owned(),
            String::new(),
            CharacterLocationBackground::Future,
        )
    } else if style.tutorial_flag == 0 {
        (
            "TECH SQUARE".to_owned(),
            "THE FUTURE".to_owned(),
            CharacterLocationBackground::Future,
        )
    } else if let Some(location) = resolve_character_selection_location(character.position) {
        (
            location.district.to_uppercase(),
            location.zone.to_uppercase(),
            location.background,
        )
    } else {
        (
            "UNKNOWN".to_owned(),
            "UNKNOWN".to_owned(),
            CharacterLocationBackground::Future,
        )
    };
    CharacterSlotUi::Occupied(OccupiedCharacterSlotUi {
        pc_uid: character.pc_uid,
        display_name,
        level: character.level,
        district,
        zone,
        background,
    })
}

pub(in super::super) fn resume_incomplete_character(
    character: &CharacterSummary,
    session: &mut CharacterCreationSession,
    model: &mut CharacterCreationUiModel,
) -> Result<(), String> {
    let slot = u8::try_from(character.slot)
        .ok()
        .filter(|slot| (1..=4).contains(slot))
        .ok_or_else(|| format!("invalid unfinished-character slot {}", character.slot))?;
    let first_name = FixedUtf16::<9>::from_str(&character.first_name)
        .map_err(|error| format!("invalid unfinished first name: {error}"))?;
    let last_name = FixedUtf16::<17>::from_str(&character.last_name)
        .map_err(|error| format!("invalid unfinished last name: {error}"))?;
    session.reset();
    session.generated_name = character.style.name_check == 1;
    session.saved_name = Some(CharacterNameSaveSuccess0104 {
        pc_uid: character.pc_uid,
        slot: character.slot,
        gender: character.style.gender,
        first_name,
        last_name,
    });
    let mut resumed = CharacterCreationUiModel::default();
    resumed.visible = true;
    resumed.screen = CharacterCreationScreen::Appearance;
    resumed.slot = Some(slot);
    resumed.pc_uid = Some(character.pc_uid);
    resumed.reserve_name = CharacterCreationCapability::Enabled;
    resumed.custom_name_filter = CharacterCreationCapability::Enabled;
    resumed.appearance = CharacterAppearance::default();
    resumed.randomize_on_appearance_open = true;
    *model = resumed;
    Ok(())
}

pub(in super::super) fn accept_reserved_character_name(
    saved: CharacterNameSaveSuccess0104,
    session: &mut CharacterCreationSession,
    model: &mut CharacterCreationUiModel,
    loading: &mut GameplayLoadingState,
    runtime: &mut RuntimeStatus,
) {
    loading.begin(ResourceLoadingScope::CharacterCreation);
    session.pending_name_check = None;
    model.pc_uid = Some(saved.pc_uid);
    // InitCreationMode starts male and randomizes once, after the name reply.
    model.appearance = CharacterAppearance::default();
    model.randomize_on_appearance_open = true;
    model.screen = CharacterCreationScreen::Appearance;
    model.blocker = None;
    runtime.message = format!(
        "Name {} {} reserved; choose the exact character appearance",
        saved.first_name.to_string_lossy(),
        saved.last_name.to_string_lossy()
    );
    session.saved_name = Some(saved);
}

/// Rebuilds the independent inventory avatar from the selected character's
/// authoritative style and the latest server-owned 9-slot equipment snapshot.
/// UI drag/popup state is deliberately absent from this boundary.
pub(in super::super) fn authoritative_user_equip_preview_character(
    runtime: &RuntimeStatus,
    inventory: &InventoryRuntime0104,
) -> Result<CharacterSummary, String> {
    let player_id = runtime
        .player_id
        .ok_or_else(|| "UserEquip preview has no authoritative local PC ID".to_owned())?;
    if player_id != inventory.owner_pc_id() {
        return Err(format!(
            "UserEquip preview owner mismatch: runtime {player_id}, inventory {}",
            inventory.owner_pc_id()
        ));
    }
    let selected_uid = runtime
        .roster
        .selected_uid
        .ok_or_else(|| "UserEquip preview has no selected character UID".to_owned())?;
    let mut character = runtime
        .roster
        .characters
        .iter()
        .find(|character| character.pc_uid == selected_uid)
        .cloned()
        .ok_or_else(|| {
            format!("UserEquip preview has no character summary for UID {selected_uid}")
        })?;
    character.equipment = (*inventory.equipment()).map(equipped_item_from_inventory_item);
    Ok(character)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_character_creation_ui_actions(
    mut outbox: ResMut<CharacterCreationUiOutbox>,
    loading: Res<GameplayLoadingState>,
    mut model: ResMut<CharacterCreationUiModel>,
    mut session: ResMut<CharacterCreationSession>,
    data: Res<LoadedCharacterCreationData>,
    mut preview: ResMut<NativePlayerPreviewModel>,
    bridge: Res<NetworkBridge>,
    mut runtime: ResMut<RuntimeStatus>,
    mut option_runtime: ResMut<OptionProductionRuntime>,
    mut next_state: ResMut<NextState<ClientState>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if loading.visible {
        outbox.drain().for_each(drop);
        return;
    }
    for action in outbox.drain() {
        match action {
            CharacterCreationUiAction::ExitToSelection => {
                session.reset();
                model.screen = CharacterCreationScreen::Name;
                runtime.message = "Returned to character selection".to_owned();
                let _ = bridge.send(NetworkCommand::RefreshCharacters);
                next_state.set(ClientState::CharacterSelect);
            }
            CharacterCreationUiAction::ToggleFullscreen => {
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
            CharacterCreationUiAction::Camera { action, delta } => {
                preview.yaw_degrees =
                    (preview.yaw_degrees + action.rotation_delta_degrees(delta)).rem_euclid(360.0);
                preview.camera_distance = (preview.camera_distance + action.distance_delta(delta))
                    .clamp(
                        CHARACTER_CREATION_PREVIEW_MIN_DISTANCE,
                        CHARACTER_CREATION_PREVIEW_MAX_DISTANCE,
                    );
                runtime.message = match action {
                    CharacterCreationCameraAction::RotateLeft => {
                        "Rotating native player preview left".to_owned()
                    }
                    CharacterCreationCameraAction::RotateRight => {
                        "Rotating native player preview right".to_owned()
                    }
                    CharacterCreationCameraAction::ZoomIn => {
                        "Zooming native player preview in".to_owned()
                    }
                    CharacterCreationCameraAction::ZoomOut => {
                        "Zooming native player preview out".to_owned()
                    }
                };
            }
            CharacterCreationUiAction::AppearanceChanged(appearance)
            | CharacterCreationUiAction::RandomizeAppearance(appearance) => {
                runtime.message = format!(
                    "Previewing {:?}: hair={} face={} shirt={} pants={} shoes={}",
                    appearance.gender,
                    appearance.hair,
                    appearance.face,
                    appearance.shirt,
                    appearance.pants,
                    appearance.shoes
                );
                // Apply the exact appearance carried by the click event before
                // the preview rebuild set. This makes gender changes independent
                // of the periodic UI synchronizer and guarantees that the male
                // and female rig geometry receives a new preview revision.
                match resolve_current_creator(&data.0, &session, &appearance) {
                    Ok(resolved) => {
                        preview.visible = true;
                        if let Err(error) = preview.set_look(resolved.look) {
                            runtime.message = format!("Character preview cannot update: {error}");
                        }
                    }
                    Err(error) => {
                        runtime.message = format!("Character preview cannot update: {error}");
                    }
                }
            }
            CharacterCreationUiAction::SubmitGeneratedName(name) => {
                submit_generated_character_name(
                    name,
                    model.slot,
                    &bridge,
                    &mut session,
                    &mut runtime,
                );
            }
            CharacterCreationUiAction::SubmitCustomName(name) => {
                submit_custom_character_name(name, model.slot, &bridge, &mut session, &mut runtime);
            }
            CharacterCreationUiAction::ConfirmAppearance(appearance) => {
                let resolved = match resolve_current_creator(&data.0, &session, &appearance) {
                    Ok(resolved) => resolved,
                    Err(error) => {
                        runtime.message = format!("Character appearance cannot be saved: {error}");
                        continue;
                    }
                };
                runtime.message = format!(
                    "Creating {} {} with OpenFusion...",
                    resolved.style.first_name.to_string_lossy(),
                    resolved.style.last_name.to_string_lossy()
                );
                if let Err(error) = bridge.send(NetworkCommand::CreateCharacter(
                    CharacterCreateRequest0104 {
                        style: resolved.style,
                        equipped: resolved.equipped,
                        selected_indices: resolved.selected_indices,
                    },
                )) {
                    runtime.message = error;
                }
            }
        }
    }
}

pub(super) fn submit_generated_character_name(
    name: GeneratedCharacterName,
    slot: Option<u8>,
    bridge: &NetworkBridge,
    session: &mut CharacterCreationSession,
    runtime: &mut RuntimeStatus,
) {
    let request = CharacterNameCheckRequest0104::new(
        &name.first,
        &name.last,
        i32::try_from(name.first_index).unwrap_or(i32::MAX),
        i32::try_from(name.middle_index).unwrap_or(i32::MAX),
        i32::try_from(name.last_index).unwrap_or(i32::MAX),
    );
    submit_character_name_request(request, slot, bridge, session, runtime);
    session.generated_name = true;
}

pub(super) fn submit_custom_character_name(
    name: CustomCharacterName,
    slot: Option<u8>,
    bridge: &NetworkBridge,
    session: &mut CharacterCreationSession,
    runtime: &mut RuntimeStatus,
) {
    let request = CharacterNameCheckRequest0104::new(&name.first, &name.last, 0, 0, 0);
    submit_character_name_request(request, slot, bridge, session, runtime);
    session.generated_name = false;
}

pub(super) fn select_character_from_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<ClientState>>,
    mut loading: ResMut<GameplayLoadingState>,
    mut buffered_entry: ResMut<BufferedCharacterEntry>,
    bridge: Res<NetworkBridge>,
    mut runtime: ResMut<RuntimeStatus>,
    mut creation_session: ResMut<CharacterCreationSession>,
    mut creation_ui: ResMut<CharacterCreationUiModel>,
    mut preview: ResMut<NativePlayerPreviewModel>,
    mut tutorial: ResMut<TutorialSession>,
    mut next_state: ResMut<NextState<ClientState>>,
    mut model: ResMut<CharacterSelectionUiModel>,
) {
    if *state.get() != ClientState::CharacterSelect {
        return;
    }
    if loading.visible {
        if keys.just_pressed(KeyCode::Enter)
            && let Some(pc_uid) = model.selected_character().map(|character| character.pc_uid)
        {
            buffered_entry.queue(pc_uid, &loading);
        }
        return;
    }
    let direct_index = if keys.just_pressed(KeyCode::Digit1) {
        Some(0)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(1)
    } else if keys.just_pressed(KeyCode::Digit3) {
        Some(2)
    } else if keys.just_pressed(KeyCode::Digit4) {
        Some(3)
    } else {
        None
    };
    if let Some(index) = direct_index {
        if let Some(character) = model.slots.get(index).and_then(CharacterSlotUi::occupied) {
            let pc_uid = character.pc_uid;
            let display_name = character.display_name.clone();
            model.select_slot(index);
            runtime.roster.selected_uid = Some(pc_uid);
            runtime.message = format!("Selected {display_name} (UID {pc_uid})");
        }
        return;
    }

    if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::ArrowDown) {
        let selectable = model
            .slots
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| slot.occupied().map(|_| index))
            .collect::<Vec<_>>();
        if selectable.is_empty() {
            return;
        }
        let current = model
            .selected_slot
            .and_then(|selected| selectable.iter().position(|index| *index == selected))
            .unwrap_or(0);
        let next = if keys.just_pressed(KeyCode::ArrowUp) {
            (current + selectable.len() - 1) % selectable.len()
        } else {
            (current + 1) % selectable.len()
        };
        let index = selectable[next];
        if let Some(character) = model.slots[index].occupied() {
            let pc_uid = character.pc_uid;
            let display_name = character.display_name.clone();
            model.select_slot(index);
            runtime.roster.selected_uid = Some(pc_uid);
            runtime.message = format!("Selected {display_name} (UID {pc_uid})");
        }
        return;
    }

    if keys.just_pressed(KeyCode::Enter)
        && let Some(pc_uid) = model.selected_character().map(|character| character.pc_uid)
        && let Some(character) = runtime
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
            &mut tutorial,
            &mut next_state,
        );
    }
}
