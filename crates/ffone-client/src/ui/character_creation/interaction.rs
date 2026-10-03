use super::*;

pub(super) const CC_NAME_BUTTON: &str = "ui/en/character/creation/name/CCNameTabButtonNormal.png";

pub(super) const CC_NAME_BUTTON_OVER: &str = "ui/en/character/creation/name/CCNameTabButtonOver.png";

pub(super) const CC_SCROLL_BG: &str = "ui/en/character/creation/name/CCScrollBG.png";

pub(super) const CC_SCROLL_UP: &str = "ui/en/character/creation/name/CCScrollUpNormal.png";

pub(super) const CC_SCROLL_UP_OVER: &str = "ui/en/character/creation/name/CCScrollUpOver.png";

pub(super) const CC_SCROLL_DOWN: &str = "ui/en/character/creation/name/CCScrollDownUp.png";

pub(super) const CC_SCROLL_DOWN_OVER: &str = "ui/en/character/creation/name/CCScrollDownOver.png";

pub(super) const CC_BLUE_BUTTON: &str = "ui/en/character/selection/controls/blue_button_normal.png";

pub(super) const CC_BLUE_BUTTON_OVER: &str = "ui/en/character/selection/controls/blue_button_over.png";

pub(super) const CC_RED_BUTTON: &str = "ui/en/character/selection/controls/red_button_normal.png";

pub(super) const CC_RED_BUTTON_OVER: &str = "ui/en/character/selection/controls/red_button_over.png";

#[derive(Component)]
pub(super) struct CharacterCreationButton(pub(super) CharacterCreationControl);

#[derive(Component)]
pub(super) struct CreationNameModeButton(pub(super) CharacterNameMode);

/// `normal`, `hover`, `active` backgrounds from the original GUIStyle. Unity
/// uses `m_Active` while the pointer is held down; it does not reuse Hover.
pub(super) fn source_button_states(
    control: CharacterCreationControl,
) -> Option<(&'static str, &'static str, &'static str)> {
    match control {
        CharacterCreationControl::Gender(_) => {
            Some((CC_CHECK_NORMAL, CC_CHECK_OVER, CC_CHECK_NORMAL))
        }
        CharacterCreationControl::Step(field, delta) => match field {
            AppearanceField::Height
            | AppearanceField::Body
            | AppearanceField::Hair
            | AppearanceField::Face => {
                if delta < 0 {
                    Some((CC_BODY_RIGHT, CC_BODY_RIGHT_OVER, CC_BODY_RIGHT))
                } else {
                    Some((CC_BODY_LEFT, CC_BODY_LEFT_OVER, CC_BODY_LEFT))
                }
            }
            AppearanceField::Shirt => {
                if delta < 0 {
                    Some((
                        "ui/en/character/creation/clothes/CCClothesLeftTopButtonNormal.png",
                        "ui/en/character/creation/clothes/CCClothesLeftTopOver.png",
                        "ui/en/character/creation/clothes/CCClothesLeftTopButtonNormal.png",
                    ))
                } else {
                    Some((
                        "ui/en/character/creation/clothes/CCClothesRightTopButtonNormal.png",
                        "ui/en/character/creation/clothes/CCClothesRightTopOver.png",
                        "ui/en/character/creation/clothes/CCClothesRightTopButtonNormal.png",
                    ))
                }
            }
            AppearanceField::Pants => {
                if delta < 0 {
                    Some((
                        "ui/en/character/creation/clothes/CCClothesLeftButtonNormal.png",
                        "ui/en/character/creation/clothes/CCClothesLeftButtonOver.png",
                        "ui/en/character/creation/clothes/CCClothesLeftButtonNormal.png",
                    ))
                } else {
                    Some((
                        "ui/en/character/creation/clothes/CCClothesRightButtonNormal.png",
                        "ui/en/character/creation/clothes/CCClothesRightButtonOver.png",
                        "ui/en/character/creation/clothes/CCClothesRightButtonNormal.png",
                    ))
                }
            }
            AppearanceField::Shoes => {
                if delta < 0 {
                    Some((
                        "ui/en/character/creation/clothes/CCClothesLeftBottomButtonNormal.png",
                        "ui/en/character/creation/clothes/CCClothesLeftBottomButtonOver.png",
                        "ui/en/character/creation/clothes/CCClothesLeftBottomButtonNormal.png",
                    ))
                } else {
                    Some((
                        "ui/en/character/creation/clothes/CCClothesRightBottomButtonNormal.png",
                        "ui/en/character/creation/clothes/CCClothesRightBottomOver.png",
                        "ui/en/character/creation/clothes/CCClothesRightBottomButtonNormal.png",
                    ))
                }
            }
        },
        CharacterCreationControl::Camera(CharacterCreationCameraAction::RotateLeft) => {
            Some((CC_ROTATE_LEFT, CC_ROTATE_LEFT_OVER, CC_ROTATE_LEFT))
        }
        CharacterCreationControl::Camera(CharacterCreationCameraAction::RotateRight) => {
            Some((CC_ROTATE_RIGHT, CC_ROTATE_RIGHT_OVER, CC_ROTATE_RIGHT))
        }
        CharacterCreationControl::Camera(CharacterCreationCameraAction::ZoomIn) => {
            Some((CC_ZOOM_IN, CC_ZOOM_IN_OVER, CC_ZOOM_IN))
        }
        CharacterCreationControl::Camera(CharacterCreationCameraAction::ZoomOut) => {
            Some((CC_ZOOM_OUT, CC_ZOOM_OUT_OVER, CC_ZOOM_OUT))
        }
        CharacterCreationControl::NameScroll(_, delta) if delta < 0 => {
            Some((CC_SCROLL_UP, CC_SCROLL_UP_OVER, CC_SCROLL_UP))
        }
        CharacterCreationControl::NameScroll(_, _) => {
            Some((CC_SCROLL_DOWN, CC_SCROLL_DOWN_OVER, CC_SCROLL_DOWN))
        }
        CharacterCreationControl::ToggleFullscreen => None,
        CharacterCreationControl::Exit => Some((CC_RED_BUTTON, CC_RED_BUTTON_OVER, CC_RED_BUTTON)),
        CharacterCreationControl::ContinueAppearance
        | CharacterCreationControl::RandomAppearance
        | CharacterCreationControl::RandomName
        | CharacterCreationControl::ContinueName => {
            Some((CC_BLUE_BUTTON, CC_BLUE_BUTTON_OVER, CC_BLUE_BUTTON))
        }
        CharacterCreationControl::NameMode(_) => {
            Some((CC_NAME_BUTTON, CC_NAME_BUTTON_OVER, CC_NAME_BUTTON))
        }
        CharacterCreationControl::ColorPage(kind, delta) => {
            if matches!(kind, CharacterCreationColorKind::Eye) {
                if delta < 0 {
                    Some((CC_BODY_LEFT, CC_BODY_LEFT_OVER, CC_BODY_LEFT))
                } else {
                    Some((CC_BODY_RIGHT, CC_BODY_RIGHT_OVER, CC_BODY_RIGHT))
                }
            } else if delta < 0 {
                Some((
                    "ui/en/character/creation/clothes/CCClothesLeftButtonNormal.png",
                    "ui/en/character/creation/clothes/CCClothesLeftButtonOver.png",
                    "ui/en/character/creation/clothes/CCClothesLeftButtonNormal.png",
                ))
            } else {
                Some((
                    "ui/en/character/creation/clothes/CCClothesRightButtonNormal.png",
                    "ui/en/character/creation/clothes/CCClothesRightButtonOver.png",
                    "ui/en/character/creation/clothes/CCClothesRightButtonNormal.png",
                ))
            }
        }
        CharacterCreationControl::Skin(_)
        | CharacterCreationControl::HairColor(_)
        | CharacterCreationControl::EyeColor(_)
        | CharacterCreationControl::ClothingChoice(_, _)
        | CharacterCreationControl::FocusCustomName => None,
    }
}

pub(super) fn handle_character_creation_interactions(
    mut commands: Commands,
    assets: Res<CharacterCreationAssets>,
    mix: Res<RetrobutionAudioMix>,
    names: Res<CharacterNameLists>,
    system_messages: Option<Res<SystemMessageUiModel>>,
    mut model: ResMut<CharacterCreationUiModel>,
    mut outbox: ResMut<CharacterCreationUiOutbox>,
    mut random: ResMut<CharacterCreationRandom>,
    buttons: Query<(&CharacterCreationButton, &Interaction), Changed<Interaction>>,
) {
    if !model.visible || system_messages.is_some_and(|messages| messages.is_popup()) {
        model.custom_name_focused = false;
        return;
    }
    for (button, interaction) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let control = button.0;
        if !control_on_current_screen(control, &model) {
            continue;
        }
        if let Some(cue) = control_sound(control, &model) {
            play_sound_cue(&mut commands, &assets, &mut random, cue, &mix);
        }
        if control != CharacterCreationControl::FocusCustomName {
            model.custom_name_focused = false;
        }
        match control {
            CharacterCreationControl::Exit => {
                outbox
                    .actions
                    .push_back(CharacterCreationUiAction::ExitToSelection);
            }
            CharacterCreationControl::ToggleFullscreen => {
                outbox
                    .actions
                    .push_back(CharacterCreationUiAction::ToggleFullscreen);
            }
            // Source `GUI.RepeatButton` camera controls are sampled every
            // rendered frame by `repeat_character_creation_camera_controls`.
            // Emitting here would add an extra fixed click on the press edge.
            CharacterCreationControl::Camera(_) => {}
            CharacterCreationControl::Gender(gender) => {
                if model.appearance.gender != gender {
                    model.set_gender(gender);
                    outbox
                        .actions
                        .push_back(CharacterCreationUiAction::AppearanceChanged(
                            model.appearance.clone(),
                        ));
                }
            }
            CharacterCreationControl::Step(field, delta) => {
                if model.creation_items.enabled()
                    || matches!(field, AppearanceField::Height | AppearanceField::Body)
                {
                    model.step_appearance(field, delta);
                    outbox
                        .actions
                        .push_back(CharacterCreationUiAction::AppearanceChanged(
                            model.appearance.clone(),
                        ));
                } else if let CharacterCreationCapability::Pending(reason) = model.creation_items {
                    model.blocker = Some(reason);
                }
            }
            CharacterCreationControl::ClothingChoice(field, delta) => {
                if model.creation_items.enabled() && model.starter_icons.enabled() {
                    model.step_appearance(field, delta);
                    outbox
                        .actions
                        .push_back(CharacterCreationUiAction::AppearanceChanged(
                            model.appearance.clone(),
                        ));
                } else {
                    let pending = match (model.creation_items, model.starter_icons) {
                        (CharacterCreationCapability::Pending(reason), _) => Some(reason),
                        (_, CharacterCreationCapability::Pending(reason)) => Some(reason),
                        _ => None,
                    };
                    model.blocker = pending;
                }
            }
            CharacterCreationControl::ColorPage(kind, delta) => {
                let index = kind.index();
                let count = model.color_palettes[index]
                    .len()
                    .div_ceil(usize::from(kind.page_size()));
                model.color_pages[index] = (i32::from(model.color_pages[index]) + i32::from(delta))
                    .rem_euclid(count as i32) as u8;
            }
            CharacterCreationControl::Skin(index) => {
                let index = index + CharacterCreationColorKind::Skin.offset(&model);
                model.set_skin_color(index);
                outbox
                    .actions
                    .push_back(CharacterCreationUiAction::AppearanceChanged(
                        model.appearance.clone(),
                    ));
            }
            CharacterCreationControl::HairColor(index) => {
                let index = index + CharacterCreationColorKind::Hair.offset(&model);
                model.set_hair_color(index);
                outbox
                    .actions
                    .push_back(CharacterCreationUiAction::AppearanceChanged(
                        model.appearance.clone(),
                    ));
            }
            CharacterCreationControl::EyeColor(index) => {
                let index = index + CharacterCreationColorKind::Eye.offset(&model);
                model.set_eye_color(index);
                outbox
                    .actions
                    .push_back(CharacterCreationUiAction::AppearanceChanged(
                        model.appearance.clone(),
                    ));
            }
            CharacterCreationControl::RandomAppearance => {
                if model.creation_items.enabled() {
                    randomize_appearance(&mut model, &mut random);
                    outbox
                        .actions
                        .push_back(CharacterCreationUiAction::RandomizeAppearance(
                            model.appearance.clone(),
                        ));
                } else if let CharacterCreationCapability::Pending(reason) = model.creation_items {
                    model.blocker = Some(reason);
                }
            }
            CharacterCreationControl::ContinueAppearance => {
                // The network appearance payload is resolved from validated
                // creator data, not from the asynchronously assembled GPU
                // preview. A missing preview must not make Continue inert.
                let pending =
                    if let CharacterCreationCapability::Pending(reason) = model.save_appearance {
                        Some(reason)
                    } else {
                        None
                    };
                if let Some(reason) = pending {
                    model.blocker = Some(reason);
                } else {
                    outbox
                        .actions
                        .push_back(CharacterCreationUiAction::ConfirmAppearance(
                            model.appearance.clone(),
                        ));
                }
            }
            CharacterCreationControl::NameMode(mode) => {
                model.name_mode = mode;
            }
            CharacterCreationControl::NameScroll(part, delta) => {
                if model.name_table.enabled() && model.scroll_name(&names, part, delta) {
                    model.validation_error = None;
                } else if let CharacterCreationCapability::Pending(reason) = model.name_table {
                    model.blocker = Some(reason);
                }
            }
            CharacterCreationControl::RandomName => {
                if !model.name_table.enabled() {
                    if let CharacterCreationCapability::Pending(reason) = model.name_table {
                        model.blocker = Some(reason);
                    }
                } else if names.valid()
                    && names.first.len() > 2
                    && names.middle.len() > 3
                    && names.last.len() > 3
                {
                    // Exact legacy integer ranges:
                    // first [1,len-1), middle/last [2,len-1).
                    model.name_indices[0] = 1 + random.below(names.first.len() - 2);
                    model.name_indices[1] = 2 + random.below(names.middle.len() - 3);
                    model.name_indices[2] = 2 + random.below(names.last.len() - 3);
                } else {
                    model.validation_error =
                        Some(CharacterNameValidationError::NameTableUnavailable);
                }
            }
            CharacterCreationControl::FocusCustomName => {
                model.custom_name_focused = true;
            }
            CharacterCreationControl::ContinueName => {
                if let CharacterCreationCapability::Pending(reason) = model.reserve_name {
                    model.blocker = Some(reason);
                    continue;
                }
                if model.name_mode == CharacterNameMode::Custom {
                    if let CharacterCreationCapability::Pending(reason) = model.custom_name_filter {
                        model.blocker = Some(reason);
                        continue;
                    }
                }
                let result = match model.name_mode {
                    CharacterNameMode::Generated => model
                        .generated_name(&names)
                        .map(CharacterCreationUiAction::SubmitGeneratedName),
                    CharacterNameMode::Custom => model
                        .custom_name()
                        .map(CharacterCreationUiAction::SubmitCustomName),
                };
                match result {
                    Ok(action) => {
                        model.validation_error = None;
                        outbox.actions.push_back(action);
                    }
                    Err(error) => model.validation_error = Some(error),
                }
            }
        }
    }
}
