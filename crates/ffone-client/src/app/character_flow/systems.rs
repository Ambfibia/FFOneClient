use super::*;

pub(super) fn sync_character_creation_ui(
    data: Res<LoadedCharacterCreationData>,
    options: Res<OptionProductionRuntime>,
    state: Res<State<ClientState>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut model: ResMut<CharacterCreationUiModel>,
) {
    model.visible = *state.get() == ClientState::CharacterCreate;
    let window = windows.single().ok();
    model.fullscreen = window.is_some_and(|window| !matches!(window.mode, WindowMode::Windowed));
    model.ui_scale = window.map_or(1.0, |window| options.effective_ui_scale(window.height()));
    if model.visible {
        model.name_table = CharacterCreationCapability::Enabled;
        model.creation_items = CharacterCreationCapability::Enabled;
        model.reserve_name = CharacterCreationCapability::Enabled;
        model.save_appearance = CharacterCreationCapability::Enabled;
        // The server-side OpenFusion check remains authoritative while the
        // native local slang table is published.
        model.custom_name_filter = CharacterCreationCapability::Enabled;
        if let Ok(counts) = data.0.option_counts() {
            model.option_counts = counts;
            model.color_palettes = data.0.ui_palettes();
        }
        let appearance = model.appearance.clone();
        if let Ok(label) = data.0.appearance_label(&appearance, AppearanceField::Hair) {
            model.hair_label = label;
        }
        if let Ok(label) = data.0.appearance_label(&appearance, AppearanceField::Face) {
            model.face_label = label;
        }
        let mut icon_paths = std::array::from_fn(|_| std::array::from_fn(|_| None));
        let mut icons_ready = true;
        for (row, field) in [
            AppearanceField::Shirt,
            AppearanceField::Pants,
            AppearanceField::Shoes,
        ]
        .into_iter()
        .enumerate()
        {
            match data.0.clothing_icon_window(&appearance, field) {
                Ok(paths) => {
                    icon_paths[row] = paths.map(Some);
                }
                Err(_) => {
                    icons_ready = false;
                }
            }
        }
        model.starter_icon_paths = icon_paths;
        model.starter_icons = if icons_ready {
            CharacterCreationCapability::Enabled
        } else {
            CharacterCreationCapability::Pending(
                ffone_client::character_creation_ui::CharacterCreationPending::StarterClothingIconsNotPublished,
            )
        };
    }
}

pub(super) fn sync_native_player_preview(
    mut try_on: VendorTryOnOwners,
    data: Res<LoadedCharacterCreationData>,
    runtime: Res<RuntimeStatus>,
    inventory: Res<LocalInventoryRuntime>,
    user_equip: Res<UserEquipUiState>,
    barber: Res<ffone_client::barber::BarberModel>,
    vehicle: Res<LocalVehiclePresentationRuntime>,
    mut user_equip_preview: ResMut<UserEquipAvatarPreviewPresentation>,
    state: Res<State<ClientState>>,
    session: Res<CharacterCreationSession>,
    mut selection: ResMut<CharacterSelectionUiModel>,
    mut creator: ResMut<CharacterCreationUiModel>,
    mut preview: ResMut<NativePlayerPreviewModel>,
) {
    if let Some(popup) = try_on.popup.as_mut() {
        let allowed = *state.get() == ClientState::World
            && try_on
                .vendor
                .as_ref()
                .is_some_and(|v| v.phase == VendorLifecyclePhase::Visible && !v.send_pending)
            && popup.selected_try_on_item().is_some_and(|item| {
                try_on.content.as_ref().is_some_and(|content| {
                    content.gameplay_try_on_allowed(
                        item,
                        runtime.player_gender.unwrap_or(0),
                        try_on
                            .guide
                            .as_ref()
                            .and_then(|g| g.authoritative())
                            .map_or(0, |g| i32::from(g.raw_mentor())),
                    )
                })
            });
        if popup.try_on_allowed != allowed {
            popup.set_try_on_allowed(allowed);
        }
    }
    selection.preview = CharacterPreviewStatus::PlayerAssemblyPending;
    creator.preview = CharacterCreationPreviewStatus::PlayerAssemblyPending;
    match state.get() {
        ClientState::CharacterSelect => {
            preview.stage = NativePlayerPreviewStage::Selection;
            preview.yaw_degrees = selection.preview_yaw_degrees;
            preview.camera_distance = NATIVE_PLAYER_SELECTION_CAMERA_DISTANCE;
            let character = selection.selected_character().and_then(|selected| {
                runtime
                    .roster
                    .characters
                    .iter()
                    .find(|character| character.pc_uid == selected.pc_uid)
            });
            let Some(character) = character else {
                preview.visible = false;
                preview.clear_look();
                selection.status = None;
                return;
            };
            match data.0.resolve_character_summary(character) {
                Ok(look) => {
                    preview.visible = true;
                    if let Err(error) = preview.set_look(look) {
                        preview.visible = false;
                        selection.preview = CharacterPreviewStatus::Unavailable;
                        selection.status = Some(error);
                        return;
                    }
                    match &preview.status {
                        NativePlayerPreviewStatus::ReadyAnimated { .. } => {
                            selection.preview = CharacterPreviewStatus::Ready;
                            selection.status = None;
                        }
                        NativePlayerPreviewStatus::Blocked(error) => {
                            selection.preview = CharacterPreviewStatus::Unavailable;
                            selection.status = Some(error.clone());
                        }
                        NativePlayerPreviewStatus::Empty | NativePlayerPreviewStatus::Loading => {
                            selection.status = Some("Loading native player preview...".to_owned());
                        }
                    }
                }
                Err(error) => {
                    preview.visible = false;
                    preview.clear_look();
                    selection.preview = CharacterPreviewStatus::Unavailable;
                    selection.status = Some(error.to_string());
                }
            }
        }
        ClientState::CharacterCreate
            if creator.visible && creator.screen == CharacterCreationScreen::Appearance =>
        {
            preview.stage = NativePlayerPreviewStage::Creation;
            // `CnGuiCharCreation` randomizes once when the appearance screen
            // opens. Do not build the default look and replace it again in the
            // next frame: that overlaps two asynchronous rig/material graphs
            // exactly during the selection-to-creation camera transition.
            // Clear the previous selection preview now; the final randomized
            // look is submitted on the following frame.
            if creator.randomize_on_appearance_open {
                preview.visible = false;
                preview.clear_look();
                return;
            }
            preview.visible = true;
            match resolve_current_creator(&data.0, &session, &creator.appearance) {
                Ok(resolved) => {
                    if preview.set_look(resolved.look).is_err() {
                        preview.visible = false;
                        return;
                    }
                    if matches!(
                        preview.status,
                        NativePlayerPreviewStatus::ReadyAnimated { .. }
                    ) {
                        creator.preview = CharacterCreationPreviewStatus::Ready;
                    }
                }
                Err(_) => {
                    preview.visible = false;
                    preview.clear_look();
                }
            }
        }
        ClientState::World if barber.active() => {
            preview.stage = NativePlayerPreviewStage::Barber;
            preview.yaw_degrees = barber.yaw;
            preview.camera_distance = barber.distance;
            let character = inventory.snapshot().and_then(|inventory| {
                authoritative_user_equip_preview_character(&runtime, inventory).ok()
            });
            if let (Some(mut character), Some(style)) = (character, &barber.draft) {
                ffone_client::barber::apply_appearance(&mut character.style, style);
                // Hats and glasses must not hide the edited face/hair preview.
                for slot in [4usize, 5, 6, 7, 8] { character.equipment[slot] = Default::default(); }
                if barber.original.as_ref().is_some_and(|original| original.gender != style.gender) {
                    use ffone_runtime_contracts::AvatarItemCategory::{Shirt,Pants,Shoes};
                    for (slot,category) in [(1usize,Shirt),(2,Pants),(3,Shoes)] {
                        let item=character.equipment[slot];
                        if data.0.avatar_items_document().items.iter().find(|row|row.category==category && row.item_number==item.item_id as u32)
                            .is_some_and(|row| row.required_gender>0 && row.required_gender as i8!=style.gender) {
                            character.equipment[slot]=Default::default();
                        }
                    }
                }
                match data.0.resolve_character_summary(&character) {
                    Ok(look) => preview.visible = preview.set_look(look).is_ok(),
                    Err(_) => { preview.visible = false; preview.clear_look(); }
                }
            } else { preview.visible = false; preview.clear_look(); }
        }
        ClientState::World
            if try_on
                .popup
                .as_ref()
                .and_then(|p| p.try_on_item())
                .is_some() =>
        {
            let popup = try_on.popup.as_ref().unwrap();
            let item = popup.try_on_item().unwrap();
            preview.stage = NativePlayerPreviewStage::TryOn;
            preview.yaw_degrees = popup.try_on_yaw;
            preview.camera_distance = 2.2;
            let character = inventory.snapshot().and_then(|inventory| {
                authoritative_user_equip_preview_character(&runtime, inventory).ok()
            });
            if let Some(character) = character {
                match data.0.resolve_try_on_character(&character, item) {
                    Ok(look) => {
                        preview.visible = preview.set_look(look).is_ok();
                    }
                    Err(_) => {
                        preview.visible = false;
                        preview.clear_look();
                    }
                }
            } else {
                preview.visible = false;
                preview.clear_look();
            }
        }
        ClientState::World if user_equip.is_active() => {
            user_equip_preview
                .set_vehicle_mounted(vehicle.family != LegacyVehiclePresentationFamily::None);
            preview.stage = NativePlayerPreviewStage::Inventory;
            preview.yaw_degrees = user_equip_preview.yaw_degrees();
            preview.camera_distance = NATIVE_PLAYER_INVENTORY_CAMERA_DISTANCE;
            selection.status = None;
            let Some(inventory) = inventory.snapshot() else {
                preview.visible = false;
                preview.clear_look();
                return;
            };
            let character = match authoritative_user_equip_preview_character(&runtime, inventory) {
                Ok(character) => character,
                Err(_) => {
                    preview.visible = false;
                    preview.clear_look();
                    return;
                }
            };
            match data.0.resolve_character_summary(&character) {
                Ok(look) => {
                    preview.visible = true;
                    if preview.set_look(look).is_err() {
                        preview.visible = false;
                        preview.clear_look();
                    }
                }
                Err(_) => {
                    preview.visible = false;
                    preview.clear_look();
                }
            }
        }
        _ => {
            preview.visible = false;
            preview.clear_look();
            if *state.get() == ClientState::World {
                preview.stage = NativePlayerPreviewStage::Selection;
            }
            selection.status = None;
        }
    }
}
