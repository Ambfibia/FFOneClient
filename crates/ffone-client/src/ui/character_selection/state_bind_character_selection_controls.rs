use super::*;

pub(super) fn bind_character_selection_slot_text(
    model: Res<CharacterSelectionUiModel>,
    localization: Option<Res<Localization>>,
    language: Option<Res<Language>>,
    mut names: Query<(
        &SelectionSlotName,
        &mut LocalizedText,
        &mut TextColor,
        &mut CharacterSelectionTextStyle,
    )>,
    mut levels: Query<
        (
            &SelectionSlotLevel,
            &mut TextColor,
            &mut LocalizedText,
            &mut Visibility,
            &mut CharacterSelectionTextStyle,
        ),
        Without<SelectionSlotName>,
    >,
    mut locations: Query<
        (
            &SelectionSlotLocation,
            &mut LocalizedText,
            &mut TextColor,
            &mut CharacterSelectionTextStyle,
        ),
        (Without<SelectionSlotName>, Without<SelectionSlotLevel>),
    >,
    mut empty_labels: Query<
        (
            &SelectionSlotEmptyLabel,
            &mut Node,
            &mut LocalizedText,
            &mut Visibility,
        ),
        (
            Without<SelectionSlotName>,
            Without<SelectionSlotLevel>,
            Without<SelectionSlotLocation>,
        ),
    >,
) {
    for (marker, mut localized_component, mut color, mut style) in &mut names {
        let selected =
            model.selected_slot == Some(marker.0) || model.hovered_slot == Some(marker.0);
        let value = match &model.slots[marker.0] {
            CharacterSlotUi::Occupied(slot) => slot.display_name.clone(),
            CharacterSlotUi::Empty
            | CharacterSlotUi::SubscriptionLocked
            | CharacterSlotUi::SubscriptionLockedOccupied(_) => String::new(),
        };
        let localized =
            LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", &value);
        *localized_component = localized;
        *style = if selected {
            CharacterSelectionTextStyle::CharNameDown
        } else {
            CharacterSelectionTextStyle::CharNameUp
        };
        *color = character_selection_text_color(*style);
    }
    for (marker, mut color, mut localized_component, mut visibility, mut style) in &mut levels {
        let selected =
            model.selected_slot == Some(marker.0) || model.hovered_slot == Some(marker.0);
        let (localized, next_visibility) = match &model.slots[marker.0] {
            CharacterSlotUi::Occupied(slot) => (
                LocalizedText::new("ui.character_select.level", "LEVEL {level}")
                    .with_arg("level", slot.level.to_string()),
                Visibility::Inherited,
            ),
            CharacterSlotUi::Empty
            | CharacterSlotUi::SubscriptionLocked
            | CharacterSlotUi::SubscriptionLockedOccupied(_) => (
                LocalizedText::new("ui.character_select.level", "LEVEL {level}")
                    .with_arg("level", ""),
                Visibility::Hidden,
            ),
        };
        *localized_component = localized;
        *visibility = next_visibility;
        *style = if selected {
            CharacterSelectionTextStyle::CharNameDown
        } else {
            CharacterSelectionTextStyle::CharLevelUp
        };
        *color = character_selection_text_color(*style);
    }
    for (marker, mut localized_component, mut color, mut style) in &mut locations {
        let selected =
            model.selected_slot == Some(marker.0) || model.hovered_slot == Some(marker.0);
        let localized = match &model.slots[marker.0] {
            CharacterSlotUi::Occupied(slot) => {
                slot.location_localized_in(selected, localization.as_deref(), language.as_deref())
            }
            CharacterSlotUi::Empty
            | CharacterSlotUi::SubscriptionLocked
            | CharacterSlotUi::SubscriptionLockedOccupied(_) => {
                LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", "")
            }
        };
        *localized_component = localized;
        *style = if selected {
            CharacterSelectionTextStyle::CharNameDown
        } else {
            CharacterSelectionTextStyle::CharLevelUp
        };
        *color = character_selection_text_color(*style);
    }
    for (marker, mut node, mut localized_component, mut visibility) in &mut empty_labels {
        match model.slots[marker.0] {
            CharacterSlotUi::Empty => {
                node.left = px(220.0);
                node.top = px(SLOT_EMPTY_Y[marker.0]);
                node.width = px(63.0);
                node.height = px(15.0);
                let localized = LocalizedText::new("ui.character_select.empty", "EMPTY");
                *localized_component = localized;
                *visibility = Visibility::Inherited;
            }
            CharacterSlotUi::SubscriptionLocked
            | CharacterSlotUi::SubscriptionLockedOccupied(_) => {
                node.left = px(165.0);
                node.top = px([71.0, 162.0, 264.0, 361.0][marker.0]);
                node.width = px(172.0);
                node.height = px(16.0);
                let localized = LocalizedText::new(
                    "ui.character_select.unlimited_only",
                    "UNLIMITED ACCESS ONLY",
                );
                *localized_component = localized;
                *visibility = Visibility::Inherited;
            }
            CharacterSlotUi::Occupied(_) => {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

pub(super) fn bind_character_selection_decorations(
    model: Res<CharacterSelectionUiModel>,
    mut disks_back: Query<(&SelectionSlotDiskBack, &mut Visibility)>,
    mut disks_front: Query<
        (&SelectionSlotDiskFront, &mut Visibility),
        Without<SelectionSlotDiskBack>,
    >,
    mut locks: Query<
        (&SelectionSlotLock, &mut Visibility),
        (
            Without<SelectionSlotDiskBack>,
            Without<SelectionSlotDiskFront>,
        ),
    >,
    mut avatar_name: Query<&mut LocalizedText, With<SelectionAvatarName>>,
) {
    for (marker, mut visibility) in &mut disks_back {
        *visibility = if matches!(
            model.slots[marker.0],
            CharacterSlotUi::Empty
                | CharacterSlotUi::Occupied(_)
                | CharacterSlotUi::SubscriptionLockedOccupied(_)
        ) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for (marker, mut visibility) in &mut disks_front {
        *visibility = if matches!(
            model.slots[marker.0],
            CharacterSlotUi::Empty
                | CharacterSlotUi::Occupied(_)
                | CharacterSlotUi::SubscriptionLockedOccupied(_)
        ) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for (marker, mut visibility) in &mut locks {
        *visibility = if model.slots[marker.0].subscription_locked() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut localized_component) = avatar_name.single_mut() {
        let value = model
            .selected_character()
            .map_or_else(String::new, |slot| slot.display_name.clone());
        *localized_component =
            LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", value);
    }
}

pub(super) fn sync_character_selection_camera_activity(
    model: Res<CharacterSelectionUiModel>,
    phase: Option<Res<State<crate::ui_startup::NativeUiStartupPhase>>>,
    mut cameras: Query<
        (
            &mut Camera,
            Option<&SelectionBaseCamera>,
            Option<&SelectionModalCamera>,
            Option<&SelectionPortraitOverlayCamera>,
        ),
        Or<(
            With<SelectionBaseCamera>,
            With<SelectionModalCamera>,
            With<SelectionPortraitOverlayCamera>,
        )>,
    >,
) {
    // Production keeps this UI resident between roster visits. `model.visible`
    // is synchronized from ClientState in Update and can therefore retain its
    // previous value for the transition frame. NativeUiStartupPhase is the
    // authoritative camera owner and prevents that frame from overlapping the
    // gameplay or creation order-100 camera. Standalone UI tests do not install
    // the production phase, so preserve their model-only behavior.
    let owns_camera = phase.as_deref().is_none_or(|phase| {
        phase.get() == &crate::ui_startup::NativeUiStartupPhase::CharacterSelection
    });
    for (mut camera, base, modal, portrait_overlay) in &mut cameras {
        camera.is_active = owns_camera
            && model.visible
            && (base.is_some()
                || portrait_overlay.is_some()
                || (modal.is_some() && model.delete_confirmation_pc_uid.is_some()));
    }
}

pub(super) fn bind_character_selection_control_visibility(
    model: Res<CharacterSelectionUiModel>,
    mut controls: Query<
        (
            &mut Visibility,
            Option<&SelectionEnter>,
            Option<&SelectionCreate>,
            Option<&SelectionDelete>,
        ),
        Or<(
            With<SelectionEnter>,
            With<SelectionCreate>,
            With<SelectionDelete>,
        )>,
    >,
) {
    let has_selected_character = model.selected_character().is_some();
    let can_create = model.create.enabled() && model.first_creatable_protocol_slot().is_some();
    let can_delete = model.delete.enabled() && has_selected_character;
    for (mut visibility, enter, create, delete) in &mut controls {
        let rendered = if enter.is_some() {
            has_selected_character
        } else if create.is_some() {
            can_create
        } else if delete.is_some() {
            can_delete
        } else {
            false
        };
        *visibility = if rendered {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

pub(super) fn bind_character_selection_preview_loading(
    model: Res<CharacterSelectionUiModel>,
    mut areas: Query<&mut BackgroundColor, With<SelectionPreviewArea>>,
    mut labels: Query<(&mut Visibility, &mut LocalizedText), With<SelectionPreviewLoading>>,
) {
    let pending = model.visible
        && model.selected_character().is_some()
        && model.preview != CharacterPreviewStatus::Ready;
    for mut background in &mut areas {
        *background = if pending {
            BackgroundColor(Color::srgb(0.025, 0.045, 0.08))
        } else {
            BackgroundColor(Color::NONE)
        };
    }
    for (mut visibility, mut label) in &mut labels {
        *visibility = if pending {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        *label = if model.preview == CharacterPreviewStatus::Unavailable {
            LocalizedText::new(
                "status.player_preview.unavailable",
                "Native player preview unavailable: {error}",
            )
            .with_arg("error", model.status.as_deref().unwrap_or("unknown error"))
        } else {
            LocalizedText::new(
                "status.player_preview.loading",
                "Loading native player preview...",
            )
        };
    }
}

#[cfg(test)]
mod preview_loading_tests {
    use super::*;

    #[test]
    fn selected_preview_stays_covered_until_the_current_look_is_ready() {
        let mut app = App::new();
        let mut model = CharacterSelectionUiModel::default();
        model.visible = true;
        model.slots[0] = CharacterSlotUi::Occupied(OccupiedCharacterSlotUi {
            pc_uid: 1,
            display_name: "First".into(),
            level: 1,
            district: String::new(),
            zone: String::new(),
            background: CharacterLocationBackground::Future,
        });
        model.slots[1] = CharacterSlotUi::Occupied(OccupiedCharacterSlotUi {
            pc_uid: 2,
            display_name: "Second".into(),
            level: 1,
            district: String::new(),
            zone: String::new(),
            background: CharacterLocationBackground::Future,
        });
        model.selected_slot = Some(0);
        app.insert_resource(model);
        let area = app
            .world_mut()
            .spawn((SelectionPreviewArea, BackgroundColor(Color::NONE)))
            .id();
        let label = app
            .world_mut()
            .spawn((
                SelectionPreviewLoading,
                Visibility::Hidden,
                LocalizedText::new("status.player_preview.loading", "Loading native player preview..."),
            ))
            .id();
        app.add_systems(Update, bind_character_selection_preview_loading);

        app.update();
        assert_ne!(app.world().get::<BackgroundColor>(area).unwrap().0, Color::NONE);
        assert_eq!(app.world().get::<Visibility>(label), Some(&Visibility::Inherited));

        app.world_mut().resource_mut::<CharacterSelectionUiModel>().preview =
            CharacterPreviewStatus::Ready;
        app.update();
        assert_eq!(app.world().get::<BackgroundColor>(area).unwrap().0, Color::NONE);
        assert_eq!(app.world().get::<Visibility>(label), Some(&Visibility::Hidden));

        let mut model = app.world_mut().resource_mut::<CharacterSelectionUiModel>();
        model.preview = CharacterPreviewStatus::PlayerAssemblyPending;
        model.selected_slot = Some(1);
        drop(model);
        app.update();
        assert_eq!(app.world().get::<Visibility>(label), Some(&Visibility::Inherited));
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn bind_character_selection_controls(
    model: Res<CharacterSelectionUiModel>,
    assets: Res<CharacterSelectionAssets>,
    mut buttons: Query<(
        &Interaction,
        &mut ImageNode,
        Option<&SelectionEnter>,
        Option<&SelectionCreate>,
        Option<&SelectionDelete>,
        Option<&SelectionMusicToggle>,
        Option<&SelectionFullscreen>,
        Option<&SelectionQuit>,
        Option<&SelectionRotateLeft>,
        Option<&SelectionRotateRight>,
    )>,
    mut labels: Query<
        (
            &mut TextColor,
            Option<&SelectionEnterText>,
            Option<&SelectionCreateText>,
            Option<&SelectionDeleteText>,
            Option<&SelectionQuitText>,
        ),
        Or<(
            With<SelectionEnterText>,
            With<SelectionCreateText>,
            With<SelectionDeleteText>,
            With<SelectionQuitText>,
        )>,
    >,
    enter_interaction: Query<&Interaction, With<SelectionEnter>>,
    create_interaction: Query<&Interaction, With<SelectionCreate>>,
    delete_interaction: Query<&Interaction, With<SelectionDelete>>,
    quit_interaction: Query<&Interaction, With<SelectionQuit>>,
) {
    for (
        interaction,
        mut image,
        enter,
        create,
        delete,
        music,
        fullscreen,
        quit,
        rotate_left,
        rotate_right,
    ) in &mut buttons
    {
        let over = matches!(*interaction, Interaction::Hovered | Interaction::Pressed);
        if enter.is_some() {
            image.image = if model.selected_character().is_some() && over {
                assets.enter_over.clone()
            } else {
                assets.enter.clone()
            };
        } else if create.is_some() {
            image.image = if over {
                assets.blue_button_over.clone()
            } else {
                assets.blue_button.clone()
            };
        } else if delete.is_some() {
            image.image = if *interaction == Interaction::Hovered {
                assets.red_button_over.clone()
            } else {
                assets.red_button.clone()
            };
        } else if music.is_some() {
            // `GUI.Toggle(..., bMusic, MusicToggle)` selects the style's
            // onNormal texture when music is enabled. The clean asset names
            // describe the action/icon, so onNormal is `musicoff` pathId 438
            // and normal is `musicon` pathId 200.
            image.image = if model.music_enabled {
                assets.music_toggle_off.clone()
            } else {
                assets.music_toggle_on.clone()
            };
        } else if fullscreen.is_some() {
            image.image = match (model.fullscreen, over) {
                (true, true) => assets.windowed_over.clone(),
                (true, false) => assets.windowed.clone(),
                (false, true) => assets.fullscreen_over.clone(),
                (false, false) => assets.fullscreen.clone(),
            };
        } else if quit.is_some() {
            image.image = if *interaction == Interaction::Hovered {
                assets.red_button_over.clone()
            } else {
                assets.red_button.clone()
            };
        } else if rotate_left.is_some() {
            image.image = if *interaction == Interaction::Hovered {
                assets.rotate_left_over.clone()
            } else {
                assets.rotate_left.clone()
            };
        } else if rotate_right.is_some() {
            image.image = if *interaction == Interaction::Hovered {
                assets.rotate_right_over.clone()
            } else {
                assets.rotate_right.clone()
            };
        }
    }

    for (mut color, enter, create, delete, quit) in &mut labels {
        let interaction = if enter.is_some() {
            enter_interaction.single().copied().ok()
        } else if create.is_some() {
            create_interaction.single().copied().ok()
        } else if delete.is_some() {
            delete_interaction.single().copied().ok()
        } else if quit.is_some() {
            quit_interaction.single().copied().ok()
        } else {
            None
        }
        .unwrap_or(Interaction::None);

        color.0 = if enter.is_some() {
            if model.selected_character().is_some()
                && matches!(interaction, Interaction::Hovered | Interaction::Pressed)
            {
                Color::srgb(0.0, 0.278_431_39, 0.478_431_37)
            } else {
                Color::srgb(0.8, 1.0, 1.0)
            }
        } else if create.is_some() {
            if matches!(interaction, Interaction::Hovered | Interaction::Pressed) {
                Color::srgb(0.0, 0.278_431_39, 0.478_431_37)
            } else {
                Color::srgb(0.898_039_2, 0.898_039_2, 0.898_039_2)
            }
        } else if delete.is_some() || quit.is_some() {
            if interaction == Interaction::Pressed {
                Color::srgb(0.898_039_2, 0.898_039_2, 0.898_039_2)
            } else {
                Color::WHITE
            }
        } else {
            Color::WHITE
        };
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn bind_character_selection_delete_modal(
    model: Res<CharacterSelectionUiModel>,
    assets: Res<CharacterSelectionAssets>,
    mut modal: Query<&mut Visibility, With<SelectionDeleteModal>>,
    mut field: Query<&mut LocalizedText, With<SelectionDeleteFieldText>>,
    mut cancel: Query<
        (&Interaction, &mut ImageNode),
        (With<SelectionDeleteCancel>, Without<SelectionDeleteConfirm>),
    >,
    mut confirm: Query<
        (&Interaction, &mut ImageNode),
        (With<SelectionDeleteConfirm>, Without<SelectionDeleteCancel>),
    >,
    mut cancel_text: Query<
        &mut TextColor,
        (
            With<SelectionDeleteCancelText>,
            Without<SelectionDeleteConfirmText>,
        ),
    >,
    mut confirm_text: Query<
        &mut TextColor,
        (
            With<SelectionDeleteConfirmText>,
            Without<SelectionDeleteCancelText>,
        ),
    >,
) {
    let open = model.delete_confirmation_pc_uid.is_some();
    if let Ok(mut visibility) = modal.single_mut() {
        *visibility = if open {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut localized_component) = field.single_mut() {
        let value = if open {
            format!("{}|", model.delete_name_input)
        } else {
            String::new()
        };
        *localized_component =
            LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", value);
    }
    if let Ok((interaction, mut image)) = cancel.single_mut() {
        image.image = if *interaction == Interaction::None {
            assets.cancel_normal.clone()
        } else {
            // Exact `cancel` style: Hover and Active both use pathId 640,
            // `blue_button_normal`.
            assets.blue_button.clone()
        };
    }
    if let Ok((interaction, mut image)) = confirm.single_mut() {
        image.image = if *interaction == Interaction::Hovered {
            assets.red_button_over.clone()
        } else {
            // QuitButton Active returns to red_button_normal.
            assets.red_button.clone()
        };
    }
    if let Ok(mut color) = cancel_text.single_mut() {
        color.0 = Color::srgb(0.8, 1.0, 1.0);
    }
    if let (Ok((interaction, _)), Ok(mut color)) = (confirm.single(), confirm_text.single_mut()) {
        color.0 = if *interaction == Interaction::Pressed {
            Color::srgb(0.898_039_2, 0.898_039_2, 0.898_039_2)
        } else {
            Color::WHITE
        };
    }
}

pub(super) fn control_character_selection_music(
    model: Res<CharacterSelectionUiModel>,
    loading: Option<Res<crate::world_audio::RetrobutionLoadingAudioState>>,
    phase: Option<Res<State<crate::ui_startup::NativeUiStartupPhase>>>,
    mut sources: Query<(&mut PlaybackSettings, Option<&AudioSink>), With<SelectionMusic>>,
) {
    // Selection audio must observe the transition into creation/cutscenes even
    // after its presentation set stops. Also silence a source still decoding.
    let active = model.visible
        && !loading.is_some_and(|loading| loading.active)
        && model.music_enabled
        && phase.as_ref().is_none_or(|phase| {
            matches!(
                phase.get(),
                crate::ui_startup::NativeUiStartupPhase::CharacterSelection
                    | crate::ui_startup::NativeUiStartupPhase::Gameplay
            )
        });
    for (mut settings, sink) in &mut sources {
        settings.paused = !active;
        if let Some(sink) = sink {
            if active {
                sink.play();
            } else {
                sink.pause();
            }
        }
    }
}
