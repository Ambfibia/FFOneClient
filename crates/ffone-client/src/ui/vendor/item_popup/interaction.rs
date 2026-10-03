use super::*;

pub(super) fn input(
    mut popup: ResMut<VendorItemPopupState>,
    projection: Res<VendorModeProjection0104>,
    mut state: ResMut<VendorUiState>,
    mut modal: ResMut<VendorModalState>,
    mut outbox: ResMut<VendorUiOutbox0104>,
    mut audio: ResMut<VendorUiAudioOutbox0104>,
    mut chest_open: ResMut<VendorChestOpenState>,
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    buttons: Query<(&Part, &Interaction), (With<Button>, Changed<Interaction>)>,
) {
    if !popup.is_open()
        || state.send_pending
        || modal.system_popup
        || modal.help
        || modal.generic_popup
    {
        return;
    }
    if keyboard
        .as_ref()
        .is_some_and(|keys| keys.just_pressed(KeyCode::Escape))
    {
        popup.close();
        modal.inventory_popup = false;
        audio.push(VendorUiAudioCue0104::ButtonSound);
        return;
    }
    let key_part = keyboard.as_ref().and_then(|keys| {
        [
            KeyCode::Digit0,
            KeyCode::Digit1,
            KeyCode::Digit2,
            KeyCode::Digit3,
            KeyCode::Digit4,
            KeyCode::Digit5,
            KeyCode::Digit6,
            KeyCode::Digit7,
            KeyCode::Digit8,
            KeyCode::Digit9,
        ]
        .into_iter()
        .enumerate()
        .find(|(_, key)| keys.just_pressed(*key))
        .map(|(digit, _)| Part::Digit(digit as u8))
        .or_else(|| {
            [
                KeyCode::Numpad0,
                KeyCode::Numpad1,
                KeyCode::Numpad2,
                KeyCode::Numpad3,
                KeyCode::Numpad4,
                KeyCode::Numpad5,
                KeyCode::Numpad6,
                KeyCode::Numpad7,
                KeyCode::Numpad8,
                KeyCode::Numpad9,
            ]
            .into_iter()
            .enumerate()
            .find(|(_, key)| keys.just_pressed(*key))
            .map(|(digit, _)| Part::Digit(digit as u8))
        })
        .or_else(|| {
            (keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter))
                .then_some(Part::Accept)
        })
        .or_else(|| keys.just_pressed(KeyCode::Backspace).then_some(Part::Clear))
    });
    for part in buttons
        .iter()
        .filter(|(_, interaction)| **interaction == Interaction::Pressed)
        .map(|(part, _)| *part)
        .chain(key_part)
    {
        let mut cue = VendorUiAudioCue0104::ButtonSound;
        match part {
            Part::Accept
                if popup
                    .selection
                    .as_ref()
                    .is_some_and(|s| s.contract.open_chest) =>
            {
                if popup.valid(&projection)
                    && state.phase == VendorLifecyclePhase::Visible
                    && let Some(selected) = popup.selection.as_ref()
                    && let VendorItemPopupSource0104::InventorySlot { inventory_slot } =
                        selected.contract.source
                {
                    let intent = VendorChestOpenIntent {
                        pc: selected.owner,
                        session: selected.session,
                        slot: inventory_slot,
                        item: selected.item,
                    };
                    if intent.valid(&projection) && chest_open.queue(intent) {
                        popup.close();
                        modal.inventory_popup = false;
                    }
                }
                // The production adapter plays the chest sound only after a valid send.
                break;
            }
            Part::TryOn => {
                if popup.try_on_allowed && !popup.try_on && popup.selected_try_on_item().is_some() {
                    popup.try_on = true;
                    audio.push(if popup.selected_try_on_item().unwrap().item_type == 0 {
                        VendorUiAudioCue0104::WeaponEquipped
                    } else {
                        VendorUiAudioCue0104::ClothingEquipped
                    });
                }
                break;
            }
            Part::TryClose => popup.try_on = false,
            Part::Close => popup.close(),
            Part::Clear => popup.amount = 0,
            Part::Digit(digit) => popup.digit(digit),
            Part::Accept | Part::Delete => {
                let random_pitch = popup.selection.as_ref().is_some_and(|s| s.item.item_type != 7);
                if let Some((source, outcome)) =
                    popup.commit(matches!(part, Part::Delete), &projection)
                {
                    if matches!(part, Part::Accept) {
                        cue = match source {
                            VendorItemPopupSource0104::CatalogRow { .. }
                            | VendorItemPopupSource0104::BuybackRow { .. } => VendorUiAudioCue0104::Purchase { random_pitch },
                            VendorItemPopupSource0104::InventorySlot { .. } => VendorUiAudioCue0104::Money,
                        };
                    }
                    modal.inventory_popup = false;
                    match source {
                        VendorItemPopupSource0104::InventorySlot { .. } => {
                            let _ = state.dispatch_inventory_outcome(*modal, outcome, &mut outbox);
                        }
                        _ => {
                            let _ = state.dispatch_outcome(*modal, outcome, &mut outbox);
                        }
                    }
                }
            }
            _ => continue,
        }
        audio.push(cue);
        modal.inventory_popup = popup.is_open();
        break;
    }
}
