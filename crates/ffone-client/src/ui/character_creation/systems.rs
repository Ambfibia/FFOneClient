use super::*;

pub(super) fn sync_character_creation_camera_activity(
    model: Res<CharacterCreationUiModel>,
    phase: Option<Res<State<crate::ui_startup::NativeUiStartupPhase>>>,
    mut cameras: Query<
        (
            &mut Camera,
            Option<&CharacterCreationBaseCamera>,
            Option<&CharacterCreationForegroundCamera>,
        ),
        Or<(
            With<CharacterCreationBaseCamera>,
            With<CharacterCreationForegroundCamera>,
        )>,
    >,
) {
    // The model is updated from ClientState in the same schedule and may keep
    // its old visibility for one transition frame. The native phase is the
    // authoritative owner of the shared order-100 UI camera.
    let owns_camera = phase.as_deref().is_none_or(|phase| {
        phase.get() == &crate::ui_startup::NativeUiStartupPhase::CharacterCreation
    });
    for (mut camera, base, foreground) in &mut cameras {
        camera.is_active = owns_camera
            && model.visible
            && (base.is_some()
                || (foreground.is_some() && model.screen == CharacterCreationScreen::Appearance));
    }
}

pub(super) fn apply_pending_appearance_randomization(
    mut model: ResMut<CharacterCreationUiModel>,
    mut outbox: ResMut<CharacterCreationUiOutbox>,
    mut random: ResMut<CharacterCreationRandom>,
) {
    if !model.visible
        || model.screen != CharacterCreationScreen::Appearance
        || !model.randomize_on_appearance_open
    {
        return;
    }
    if let CharacterCreationCapability::Pending(reason) = model.creation_items {
        model.blocker = Some(reason);
        return;
    }

    randomize_appearance(&mut model, &mut random);
    model.randomize_on_appearance_open = false;
    model.blocker = None;
    outbox
        .actions
        .push_back(CharacterCreationUiAction::RandomizeAppearance(
            model.appearance.clone(),
        ));
}
