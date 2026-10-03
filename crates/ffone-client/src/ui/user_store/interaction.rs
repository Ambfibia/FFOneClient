use super::*;

pub const USER_STORE_SCROLL_VELOCITY: f32 = 200.0;

pub const USER_STORE_SCROLL_INPUT_CLAMP: f32 = 30.0;

pub const USER_STORE_BUTTON_NORMAL_PATH: &str = "ui/en/vendor/button-normal.png";

pub const USER_STORE_BUTTON_HOVER_PATH: &str = "ui/en/vendor/button-hover.png";

pub const USER_STORE_POPUP_CLOSE_HOVER_PATH: &str = "ui/en/email/close-hover.png";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserStoreRowButton0104 {
    Primary,
    Secondary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UserStoreInputBoundary0104 {
    pub blocks_lower_ui: bool,
    pub blocks_gameplay_input: bool,
    pub requires_pointer: bool,
    pub controls_enabled: bool,
    pub close_enabled: bool,
    pub escape_enabled: bool,
}

#[must_use]
pub const fn user_store_inventory_scroll_max() -> f32 {
    USER_STORE_INVENTORY_CONTENT_HEIGHT - USER_STORE_INVENTORY_VIEWPORT_HEIGHT
}

#[must_use]
pub fn clamp_user_store_inventory_scroll(value: f32) -> f32 {
    if value.is_nan() || value <= 0.0 {
        0.0
    } else if !value.is_finite() {
        user_store_inventory_scroll_max()
    } else {
        value.min(user_store_inventory_scroll_max())
    }
}

pub(super) fn user_store_primary_button_text(value: &str) -> LocalizedText {
    match value {
        "OPEN STORE" => LocalizedText::new("ui.user_store.open_store", value),
        "CLOSE STORE" => LocalizedText::new("ui.user_store.close_store", value),
        _ => user_store_passthrough_text(value),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn collect_user_store_input_0104(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mouse_buttons: Option<Res<ButtonInput<MouseButton>>>,
    mouse_scroll: Option<Res<AccumulatedMouseScroll>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    changed_controls: Query<(&UserStoreInteractiveControl0104, &Interaction), Changed<Interaction>>,
    all_controls: Query<(&UserStoreInteractiveControl0104, &Interaction)>,
    mut state: ResMut<UserStoreUiState0104>,
    authority: Res<UserStoreAuthority0104>,
    catalog: Res<UserStoreItemCatalog0104>,
    mut outbox: ResMut<UserStoreUiOutbox0104>,
    mut popup_presentation: ResMut<UserStorePopupPresentation0104>,
) {
    if !state.active {
        return;
    }
    if keyboard
        .as_ref()
        .is_some_and(|keyboard| keyboard.just_pressed(KeyCode::Escape))
    {
        let _ = state.request_close(&authority, true, &mut outbox);
        return;
    }
    let projection = project_user_store_ui_0104(&state, &authority, &catalog);
    let popup_was_open = popup_presentation.popup.is_some();
    for (control, interaction) in &changed_controls {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if popup_was_open {
            match *control {
                UserStoreInteractiveControl0104::PopupClose => {
                    state.modal.item_popup = false;
                    popup_presentation.close();
                }
                UserStoreInteractiveControl0104::PopupDigit(digit) => {
                    popup_presentation.append_digit(digit);
                }
                UserStoreInteractiveControl0104::PopupClear => {
                    popup_presentation.clear_value();
                }
                UserStoreInteractiveControl0104::PopupAction => {
                    let _ = apply_user_store_popup_action_0104(
                        &mut popup_presentation,
                        &mut state,
                        &authority,
                        &mut outbox,
                    );
                }
                UserStoreInteractiveControl0104::Row(_)
                | UserStoreInteractiveControl0104::InventorySlot(_)
                | UserStoreInteractiveControl0104::PrimaryButton
                | UserStoreInteractiveControl0104::GoToGame
                | UserStoreInteractiveControl0104::Close
                | UserStoreInteractiveControl0104::Help => {}
            }
            continue;
        }
        match *control {
            UserStoreInteractiveControl0104::Row(visual_row) => {
                if let Some(row) = projection.listing_rows.get(visual_row) {
                    let outcome = state.activate_store_row(
                        &authority,
                        row.list_slot,
                        UserStoreRowButton0104::Primary,
                        &mut outbox,
                    );
                    adopt_user_store_popup_outcome_0104(&mut popup_presentation, &outcome);
                }
            }
            UserStoreInteractiveControl0104::InventorySlot(slot) => {
                let outcome = state.activate_inventory_row(
                    &authority,
                    slot as i32,
                    UserStoreRowButton0104::Primary,
                    &mut outbox,
                );
                adopt_user_store_popup_outcome_0104(&mut popup_presentation, &outcome);
            }
            UserStoreInteractiveControl0104::PrimaryButton => {
                let _ = state.click_primary_button(&authority, &mut outbox);
            }
            UserStoreInteractiveControl0104::GoToGame => {
                let _ = state.click_go_to_game();
            }
            UserStoreInteractiveControl0104::Close => {
                let _ = state.request_close(&authority, false, &mut outbox);
            }
            UserStoreInteractiveControl0104::Help => {
                let _ = state.open_help(&mut outbox);
            }
            UserStoreInteractiveControl0104::PopupClose
            | UserStoreInteractiveControl0104::PopupDigit(_)
            | UserStoreInteractiveControl0104::PopupClear
            | UserStoreInteractiveControl0104::PopupAction => {}
        }
    }

    if mouse_buttons
        .as_ref()
        .is_some_and(|buttons| buttons.just_released(MouseButton::Right))
    {
        let mut hovered = all_controls.iter().filter_map(|(control, interaction)| {
            (*interaction == Interaction::Hovered
                && matches!(
                    control,
                    UserStoreInteractiveControl0104::Row(_)
                        | UserStoreInteractiveControl0104::InventorySlot(_)
                ))
            .then_some(*control)
        });
        let first = hovered.next();
        if first.is_some() && hovered.next().is_none() {
            match first.expect("checked") {
                UserStoreInteractiveControl0104::Row(visual_row) => {
                    if let Some(row) = projection.listing_rows.get(visual_row) {
                        let outcome = state.activate_store_row(
                            &authority,
                            row.list_slot,
                            UserStoreRowButton0104::Secondary,
                            &mut outbox,
                        );
                        adopt_user_store_popup_outcome_0104(&mut popup_presentation, &outcome);
                    }
                }
                UserStoreInteractiveControl0104::InventorySlot(slot) => {
                    let outcome = state.activate_inventory_row(
                        &authority,
                        slot as i32,
                        UserStoreRowButton0104::Secondary,
                        &mut outbox,
                    );
                    adopt_user_store_popup_outcome_0104(&mut popup_presentation, &outcome);
                }
                _ => {}
            }
        }
    }

    let axis = mouse_scroll.as_ref().map_or(0.0, |scroll| scroll.delta.y);
    if !axis.is_finite() || axis == 0.0 || !state.input_boundary().controls_enabled {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let layout = user_store_layout_0104(
        window.width().max(0.0) as u32,
        window.height().max(0.0) as u32,
        state.opening_elapsed_seconds,
        state.inventory_scroll_y,
    );
    if layout.list_viewport.contains(cursor) {
        state.scroll_store_dead(axis);
    } else if layout.inventory_viewport.contains(cursor) {
        state.scroll_inventory(axis);
    }
}
