use super::*;

pub const CHARACTER_SELECTION_RED_BUTTON_PATH: &str =
    "ui/en/character/selection/controls/red_button_normal.png";

pub const CHARACTER_SELECTION_RED_BUTTON_OVER_PATH: &str =
    "ui/en/character/selection/controls/red_button_over.png";

pub const CHARACTER_SELECTION_BLUE_BUTTON_PATH: &str =
    "ui/en/character/selection/controls/blue_button_normal.png";

pub const CHARACTER_SELECTION_BLUE_BUTTON_OVER_PATH: &str =
    "ui/en/character/selection/controls/blue_button_over.png";

pub const CHARACTER_SELECTION_QUIT_BUTTON_Y_OFFSET: f32 = 0.0;

pub const CHARACTER_SELECTION_CREATE_BUTTON_Y_OFFSET: f32 = 0.0;

// Bevy's legacy `Interaction` focus pass does not use `Camera::order` when it
// compares independent UI roots. Keep a matching root Z order so the visual
// foreground is also the input foreground. The base root historically used
// 500; leaving the overlay and detached modal below it made the full-screen
// selection panel capture clicks through the visually higher DELETE UI.
pub(super) const CHARACTER_SELECTION_BASE_INTERACTION_Z_INDEX: i32 = 500;

pub(super) const CHARACTER_SELECTION_PORTRAIT_OVERLAY_INTERACTION_Z_INDEX: i32 = 600;

pub(super) const CHARACTER_SELECTION_MODAL_INTERACTION_Z_INDEX: i32 = 700;

#[derive(Component)]
pub(super) struct SelectionSlotButton(pub(super) usize);

pub(super) fn legacy_sliced_button_image(handle: Handle<Image>) -> ImageNode {
    ImageNode {
        image: handle,
        // GUIStyle padding restricts the label, never the background/hit rectangle.
        visual_box: bevy::ui::VisualBox::BorderBox,
        image_mode: NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(5.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 1.0,
        }),
        ..default()
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_character_selection_interactions(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<CharacterSelectionAssets>,
    mut model: ResMut<CharacterSelectionUiModel>,
    mut outbox: ResMut<CharacterSelectionUiOutbox>,
    mut random: ResMut<CharacterSelectionRandom>,
    slots: Query<(&SelectionSlotButton, &Interaction), Changed<Interaction>>,
    enter: Query<&Interaction, (Changed<Interaction>, With<SelectionEnter>)>,
    create: Query<&Interaction, (Changed<Interaction>, With<SelectionCreate>)>,
    delete: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<SelectionDelete>,
            Without<SelectionCreate>,
        ),
    >,
    delete_modal_buttons: Query<
        (
            &Interaction,
            Option<&SelectionDeleteCancel>,
            Option<&SelectionDeleteConfirm>,
        ),
        Changed<Interaction>,
    >,
    music: Query<&Interaction, (Changed<Interaction>, With<SelectionMusicToggle>)>,
    fullscreen: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<SelectionFullscreen>,
            Without<SelectionMusicToggle>,
        ),
    >,
    quit: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<SelectionQuit>,
            Without<SelectionFullscreen>,
        ),
    >,
    rotate_left: Query<&Interaction, With<SelectionRotateLeft>>,
    rotate_right: Query<&Interaction, With<SelectionRotateRight>>,
) {
    if !model.visible {
        model.hovered_slot = None;
        return;
    }
    if model.delete_confirmation_pc_uid.is_some() {
        if model.delete_request_pending {
            return;
        }
        if delete_modal_buttons.iter().any(|(interaction, cancel, _)| {
            cancel.is_some() && *interaction == Interaction::Pressed
        }) {
            play_character_selection_sound(&mut commands, assets.delete_no_sound.clone(), 0.7);
            model.delete_confirmation_pc_uid = None;
            model.delete_name_input.clear();
        } else if delete_modal_buttons
            .iter()
            .any(|(interaction, _, confirm)| {
                confirm.is_some() && *interaction == Interaction::Pressed
            })
        {
            play_character_selection_sound(&mut commands, assets.delete_yes_sound.clone(), 0.7);
            // Exact local gate from `CnCharSelectionMode::DeleteCharacter`:
            // a case-sensitive first-name match precedes the delete packet.
            if model.delete_confirmation_name_matches() {
                if let Some(pc_uid) = model.delete_confirmation_pc_uid {
                    outbox
                        .actions
                        .push_back(CharacterSelectionUiAction::DeleteSelected { pc_uid });
                    model.delete_request_pending = true;
                }
            } else {
                model.delete_name_input.clear();
            }
        }
        return;
    }
    for (slot, interaction) in &slots {
        match interaction {
            Interaction::Pressed if model.select_slot(slot.0) => {
                play_character_selection_button_sound(&mut commands, &assets, &mut random);
                outbox
                    .actions
                    .push_back(CharacterSelectionUiAction::SelectSlot(slot.0));
            }
            Interaction::Pressed if model.slots[slot.0].subscription_locked() => {
                model.status = Some("Subscription only".to_owned());
            }
            Interaction::Hovered if model.is_selectable(slot.0) => {
                model.hovered_slot = Some(slot.0);
            }
            Interaction::None if model.hovered_slot == Some(slot.0) => {
                model.hovered_slot = None;
            }
            Interaction::Pressed | Interaction::Hovered | Interaction::None => {}
        }
    }
    if enter
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
        && model.selected_character().is_some()
    {
        play_character_selection_button_sound(&mut commands, &assets, &mut random);
        outbox
            .actions
            .push_back(CharacterSelectionUiAction::EnterSelected);
    }
    if create
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        play_character_selection_button_sound(&mut commands, &assets, &mut random);
        match model.create {
            CharacterSelectionCapability::Enabled => {
                if let Some(slot) = model.first_creatable_protocol_slot() {
                    outbox
                        .actions
                        .push_back(CharacterSelectionUiAction::CreateCharacter { slot });
                } else {
                    model.status = Some("No available character slot".to_owned());
                }
            }
            CharacterSelectionCapability::Pending(reason) => {
                model.status = Some(reason.message().to_owned());
            }
        }
    }
    if delete
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        play_character_selection_button_sound(&mut commands, &assets, &mut random);
        match model.delete {
            CharacterSelectionCapability::Enabled => {
                let pc_uid = model.selected_slot.and_then(|index| {
                    model.slots.get(index).and_then(|slot| match slot {
                        CharacterSlotUi::Occupied(character) => Some(character.pc_uid),
                        CharacterSlotUi::Empty
                        | CharacterSlotUi::SubscriptionLocked
                        | CharacterSlotUi::SubscriptionLockedOccupied(_) => None,
                    })
                });
                if let Some(pc_uid) = pc_uid {
                    model.delete_confirmation_pc_uid = Some(pc_uid);
                    model.delete_name_input.clear();
                    model.delete_request_pending = false;
                } else {
                    model.status = Some("No selectable character to delete".to_owned());
                }
            }
            CharacterSelectionCapability::Pending(reason) => {
                model.status = Some(reason.message().to_owned());
            }
        }
    }
    if music
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        model.music_enabled = !model.music_enabled;
        outbox
            .actions
            .push_back(CharacterSelectionUiAction::ToggleMusic(model.music_enabled));
    }
    if fullscreen
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        outbox
            .actions
            .push_back(CharacterSelectionUiAction::ToggleFullscreen);
    }
    if quit
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        outbox.actions.push_back(CharacterSelectionUiAction::Quit);
    }
    let rotation_axis = if rotate_right
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        1.0
    } else {
        0.0
    } - if rotate_left
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        1.0
    } else {
        0.0
    };
    model.preview_yaw_degrees = (model.preview_yaw_degrees
        + rotation_axis * time.delta_secs() * CHARACTER_SELECTION_PREVIEW_ROTATION_SPEED_DEGREES)
        .rem_euclid(360.0);
}
