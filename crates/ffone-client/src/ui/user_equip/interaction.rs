//! Pointer, keyboard and scrollbar input collection and command execution.

use super::actions::{
    UserEquipUiAction, UserEquipUiAudioCue, UserEquipUiAudioOutbox, UserEquipUiOutbox,
};
use super::catalog::UserEquipSlotEndpoint;
use super::components::{
    UserEquipAvatarTurnControl, UserEquipCloseControl, UserEquipGumNanoButtonControl,
    UserEquipHelpCloseControl, UserEquipHelpControl, UserEquipItemTabControl,
    UserEquipNanoSlotControl, UserEquipNanoTabControl, UserEquipNanoViewerCloseControl,
    UserEquipPopupButtonControl, UserEquipPopupCloseControl, UserEquipPopupTrashControl,
    UserEquipScrollDownControl, UserEquipScrollThumbControl, UserEquipScrollTrackControl,
    UserEquipScrollUpControl, UserEquipSlotControl, UserEquipTrashControl,
};
use super::geometry::USER_EQUIP_INVENTORY_VIEWPORT_RECT;
use super::item_projection::UserEquipItemModeProjection;
use super::layout::{UserEquipScrollbarMetrics, user_equip_bevy_wheel_to_legacy_axis};
use super::nano_projection::UserEquipNanoModeProjection;
use super::popup_state::{
    UserEquipDragState, UserEquipItemPopupState, UserEquipNanoViewerState, UserEquipPopupCommand,
    user_equip_gum_target_enabled, user_equip_trash_drop,
};
use super::state::{
    UserEquipAvatarPreviewPresentation, UserEquipCloseSource, UserEquipModalState, UserEquipMode,
    UserEquipUiState, user_equip_equipment_endpoint_for_item,
};
use crate::tutorial_mission_content::TutorialMissionContent;
use bevy::{
    ecs::system::SystemParam, input::mouse::AccumulatedMouseScroll, prelude::*,
    window::PrimaryWindow,
};

pub(super) fn advance_user_equip_lifecycle(time: Res<Time>, mut state: ResMut<UserEquipUiState>) {
    state.tick(time.delta_secs());
}

pub(super) fn collect_user_equip_controller_actions(
    input: Option<Res<crate::ui::shared::controller::ControllerUiInput>>,
    state: Res<UserEquipUiState>,
    projection: Res<UserEquipItemModeProjection>,
    mut modal: ResMut<UserEquipModalState>,
    mut popup: ResMut<UserEquipItemPopupState>,
    mut drag: ResMut<UserEquipDragState>,
    mut audio: ResMut<UserEquipUiAudioOutbox>,
    slots: Query<&UserEquipSlotControl>,
) {
    let Some(entity) = input.and_then(|input| input.confirmed) else { return; };
    if state.mode() != UserEquipMode::Item || !state.input_capabilities(*modal).slot_pointer {
        return;
    }
    let Ok(control) = slots.get(entity) else { return; };
    if projection.endpoint_is_empty(control.0) { return; }
    drag.cancel();
    popup.open(control.0);
    modal.item_popup_active = true;
    audio.push(UserEquipUiAudioCue::OpenScreen);
}

pub(super) fn clear_closed_user_equip_selection(
    state: Res<UserEquipUiState>,
    mut popup: ResMut<UserEquipItemPopupState>,
    mut nano: ResMut<UserEquipNanoViewerState>,
    mut drag: ResMut<UserEquipDragState>,
    mut modal: ResMut<UserEquipModalState>,
) {
    if !state.is_active() && state.is_changed() {
        popup.close();
        nano.close();
        *drag = UserEquipDragState::default();
        modal.item_popup_active = false;
        modal.inventory_popup_modal = false;
    }
}

pub(super) fn collect_user_equip_avatar_rotation(
    state: Res<UserEquipUiState>,
    modal: Res<UserEquipModalState>,
    controls: Query<(&Interaction, &UserEquipAvatarTurnControl)>,
    mut presentation: ResMut<UserEquipAvatarPreviewPresentation>,
) {
    if !state.input_capabilities(*modal).panel_controls {
        return;
    }
    for (interaction, control) in &controls {
        if *interaction == Interaction::Pressed {
            presentation.rotate_one_gui_frame(control.0.yaw_delta_degrees());
        }
    }
}

#[derive(SystemParam)]
pub(super) struct UserEquipPointerInput<'w> {
    pub(super) keyboard: Option<Res<'w, ButtonInput<KeyCode>>>,
    pub(super) mouse_buttons: Option<Res<'w, ButtonInput<MouseButton>>>,
    pub(super) mouse_scroll: Option<Res<'w, AccumulatedMouseScroll>>,
}

#[derive(SystemParam)]
pub(super) struct UserEquipInteractionQueries<'w, 's> {
    pub(super) close_buttons:
        Query<'w, 's, &'static Interaction, (With<UserEquipCloseControl>, Changed<Interaction>)>,
    pub(super) slots: Query<'w, 's, (&'static Interaction, &'static UserEquipSlotControl)>,
    pub(super) nano_slots: Query<'w, 's, (&'static Interaction, &'static UserEquipNanoSlotControl)>,
    pub(super) nano_viewer_close_buttons: Query<
        'w,
        's,
        &'static Interaction,
        (With<UserEquipNanoViewerCloseControl>, Changed<Interaction>),
    >,
    pub(super) popup_buttons: Query<
        'w,
        's,
        (&'static Interaction, &'static UserEquipPopupButtonControl),
        Changed<Interaction>,
    >,
    pub(super) gum_nano_buttons: Query<
        'w,
        's,
        (&'static Interaction, &'static UserEquipGumNanoButtonControl),
        Changed<Interaction>,
    >,
    pub(super) popup_close_buttons: Query<
        'w,
        's,
        &'static Interaction,
        (With<UserEquipPopupCloseControl>, Changed<Interaction>),
    >,
    pub(super) popup_trash_buttons: Query<
        'w,
        's,
        &'static Interaction,
        (With<UserEquipPopupTrashControl>, Changed<Interaction>),
    >,
    pub(super) item_tabs:
        Query<'w, 's, &'static Interaction, (With<UserEquipItemTabControl>, Changed<Interaction>)>,
    pub(super) nano_tabs:
        Query<'w, 's, &'static Interaction, (With<UserEquipNanoTabControl>, Changed<Interaction>)>,
    pub(super) trash_buttons: Query<'w, 's, &'static Interaction, With<UserEquipTrashControl>>,
    pub(super) help_buttons:
        Query<'w, 's, &'static Interaction, (With<UserEquipHelpControl>, Changed<Interaction>)>,
    pub(super) help_close_buttons: Query<
        'w,
        's,
        &'static Interaction,
        (With<UserEquipHelpCloseControl>, Changed<Interaction>),
    >,
    pub(super) scroll_up_buttons:
        Query<'w, 's, &'static Interaction, (With<UserEquipScrollUpControl>, Changed<Interaction>)>,
    pub(super) scroll_down_buttons: Query<
        'w,
        's,
        &'static Interaction,
        (With<UserEquipScrollDownControl>, Changed<Interaction>),
    >,
}

pub(super) fn collect_user_equip_ui_actions(
    input: UserEquipPointerInput,
    mut state: ResMut<UserEquipUiState>,
    mut modal: ResMut<UserEquipModalState>,
    projection: Res<UserEquipItemModeProjection>,
    nano_projection: Res<UserEquipNanoModeProjection>,
    content: Option<Res<TutorialMissionContent>>,
    mut popup: ResMut<UserEquipItemPopupState>,
    mut drag: ResMut<UserEquipDragState>,
    mut nano_viewer: ResMut<UserEquipNanoViewerState>,
    queries: UserEquipInteractionQueries,
    mut outbox: ResMut<UserEquipUiOutbox>,
    mut audio: ResMut<UserEquipUiAudioOutbox>,
) {
    // A send lock, shared SystemMessage or redeem view owns the complete
    // pointer/keyboard boundary. Do not let an underlying Help/item popup
    // consume Escape or mutate its local state while that hard modal is up.
    if state.nano_station_send_pending()
        || modal.send_pending
        || modal.system_popup_active
        || modal.redeem_code_view
    {
        drag.cancel();
        return;
    }
    let escape = input
        .keyboard
        .as_ref()
        .is_some_and(|keyboard| keyboard.just_pressed(KeyCode::Escape));
    if escape && modal.help_active {
        modal.help_active = false;
    } else if escape && nano_viewer.selected_visual_index.is_some() {
        nano_viewer.close();
    } else if escape && popup.selected().is_some() {
        popup.close();
        modal.item_popup_active = false;
    } else {
        let capabilities = state.input_capabilities(*modal);
        if capabilities.keyboard_close_request && escape {
            outbox.push(UserEquipUiAction::RequestClose {
                source: UserEquipCloseSource::Escape,
            });
        }
    }

    if queries
        .help_close_buttons
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        audio.push(UserEquipUiAudioCue::ButtonSound);
        modal.help_active = false;
    }
    if queries
        .nano_viewer_close_buttons
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        audio.push(UserEquipUiAudioCue::ButtonSound);
        nano_viewer.close();
    }
    if nano_viewer.selected_visual_index().is_some() {
        drag.cancel();
        return;
    }

    if modal.item_popup_active {
        for (interaction, control) in &queries.gum_nano_buttons {
            if *interaction != Interaction::Pressed {
                continue;
            }
            let Some(UserEquipSlotEndpoint::Inventory { slot_index }) = popup.selected() else {
                continue;
            };
            let Some(item) = projection
                .inventory
                .get(slot_index)
                .map(|slot| &slot.item)
                .filter(|item| !item.empty)
            else {
                continue;
            };
            let enabled = content.as_deref().is_some_and(|content| {
                user_equip_gum_target_enabled(
                    item.item.item_id,
                    control.0,
                    &nano_projection,
                    content,
                )
            });
            if enabled {
                audio.push(UserEquipUiAudioCue::ButtonSound);
                outbox.push(UserEquipUiAction::UseInventoryItemOnNano {
                    slot_index,
                    nano_slot: control.0,
                });
                popup.close();
                modal.item_popup_active = false;
                return;
            }
        }
        let commands = popup.commands(&projection, content.as_deref());
        for (interaction, button) in &queries.popup_buttons {
            if *interaction != Interaction::Pressed {
                continue;
            }
            if let Some(command) = commands.get(button.0).copied() {
                audio.push(UserEquipUiAudioCue::ButtonSound);
                execute_user_equip_popup_command(
                    command,
                    &projection,
                    &mut popup,
                    &mut modal,
                    &mut outbox,
                );
            }
        }
        if queries
            .popup_close_buttons
            .iter()
            .any(|interaction| *interaction == Interaction::Pressed)
        {
            audio.push(UserEquipUiAudioCue::ButtonSound);
            popup.close();
            modal.item_popup_active = false;
        }
        if queries
            .popup_trash_buttons
            .iter()
            .any(|interaction| *interaction == Interaction::Pressed)
        {
            if let Some(UserEquipSlotEndpoint::Inventory { slot_index }) = popup.selected() {
                audio.push(UserEquipUiAudioCue::ButtonSound);
                outbox.push(UserEquipUiAction::DeleteInventoryItem { slot_index });
                popup.close();
                modal.item_popup_active = false;
            }
        }
    }

    let capabilities = state.input_capabilities(*modal);
    if capabilities.close_button
        && queries
            .close_buttons
            .iter()
            .any(|interaction| *interaction == Interaction::Pressed)
    {
        outbox.push(UserEquipUiAction::RequestClose {
            source: UserEquipCloseSource::CloseButton,
        });
    }
    if capabilities.panel_controls
        && queries
            .help_buttons
            .iter()
            .any(|interaction| *interaction == Interaction::Pressed)
    {
        audio.push(UserEquipUiAudioCue::ButtonSound);
        modal.help_active = true;
    }
    if capabilities.panel_controls
        && queries
            .item_tabs
            .iter()
            .any(|interaction| *interaction == Interaction::Pressed)
    {
        audio.push(UserEquipUiAudioCue::TabClick01);
        popup.close();
        modal.item_popup_active = false;
        state.select_item_tab();
        nano_viewer.close();
    }
    if capabilities.panel_controls
        && queries
            .nano_tabs
            .iter()
            .any(|interaction| *interaction == Interaction::Pressed)
    {
        audio.push(UserEquipUiAudioCue::TabClick01);
        popup.close();
        modal.item_popup_active = false;
        drag.cancel();
        state.select_nano_tab();
    }

    if capabilities.scroll
        && queries
            .scroll_up_buttons
            .iter()
            .any(|interaction| *interaction == Interaction::Pressed)
    {
        audio.push(UserEquipUiAudioCue::ButtonSound);
        outbox.push(UserEquipUiAction::ApplyLegacyScrollAxis { axis: 1.0 });
    }
    if capabilities.scroll
        && queries
            .scroll_down_buttons
            .iter()
            .any(|interaction| *interaction == Interaction::Pressed)
    {
        audio.push(UserEquipUiAudioCue::ButtonSound);
        outbox.push(UserEquipUiAction::ApplyLegacyScrollAxis { axis: -1.0 });
    }

    if capabilities.slot_pointer && state.mode() == UserEquipMode::Nano {
        if let Some((_, control)) = queries
            .nano_slots
            .iter()
            .find(|(interaction, _)| **interaction == Interaction::Pressed)
        {
            nano_viewer.open(control.0);
        }
    }

    if capabilities.slot_pointer {
        let left_pressed = input
            .mouse_buttons
            .as_ref()
            .is_some_and(|buttons| buttons.just_pressed(MouseButton::Left));
        let left_released = input
            .mouse_buttons
            .as_ref()
            .is_some_and(|buttons| buttons.just_released(MouseButton::Left));
        let right_pressed = input
            .mouse_buttons
            .as_ref()
            .is_some_and(|buttons| buttons.just_pressed(MouseButton::Right));
        if right_pressed {
            if let Some((_, control)) = queries
                .slots
                .iter()
                .find(|(interaction, _)| **interaction == Interaction::Hovered)
            {
                audio.push(UserEquipUiAudioCue::ButtonSound);
                execute_user_equip_one_click(control.0, &projection, &mut outbox);
            }
        } else if left_pressed {
            if let Some((_, control)) = queries.slots.iter().find(|(interaction, _)| {
                matches!(**interaction, Interaction::Hovered | Interaction::Pressed)
            }) {
                audio.push(UserEquipUiAudioCue::ButtonSound);
                drag.begin(control.0);
            } else if queries.trash_buttons.iter().any(|interaction| {
                matches!(*interaction, Interaction::Hovered | Interaction::Pressed)
            }) {
                audio.push(UserEquipUiAudioCue::ButtonSound);
            }
        } else if left_released {
            let source = drag.source();
            let trash_drop = queries.trash_buttons.iter().any(|interaction| {
                matches!(*interaction, Interaction::Hovered | Interaction::Pressed)
            });
            if trash_drop {
                drag.cancel();
                if let Some(action) =
                    source.and_then(|source| user_equip_trash_drop(source, &projection))
                {
                    outbox.push(action);
                    popup.close();
                    modal.item_popup_active = false;
                }
                return;
            }
            let destination = queries
                .slots
                .iter()
                .find(|(interaction, _)| {
                    matches!(**interaction, Interaction::Hovered | Interaction::Pressed)
                })
                .map(|(_, control)| control.0);
            if let Some(action) = drag.finish(destination, &projection) {
                outbox.push(action);
                popup.close();
                modal.item_popup_active = false;
            } else if source.is_some() && source == destination {
                let endpoint = source.expect("checked source");
                if projection.endpoint_is_empty(endpoint) {
                    popup.close();
                    modal.item_popup_active = false;
                } else {
                    popup.open(endpoint);
                    modal.item_popup_active = true;
                    audio.push(UserEquipUiAudioCue::OpenScreen);
                }
            }
        }
    } else {
        drag.cancel();
    }
    let axis = input.mouse_scroll.as_ref().map_or(0.0, |scroll| {
        user_equip_bevy_wheel_to_legacy_axis(scroll.delta.y)
    });
    if capabilities.scroll && axis.is_finite() && axis != 0.0 {
        outbox.push(UserEquipUiAction::ApplyLegacyScrollAxis { axis });
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum UserEquipScrollbarPart {
    Track,
    Thumb,
}

/// Pointer capture for the shared Item/Nano scroll-view scrollbar.
#[derive(Default)]
pub(super) struct UserEquipScrollbarPointer {
    pub(super) held: Option<UserEquipScrollbarPart>,
    pub(super) mode: Option<UserEquipMode>,
    pub(super) anchor: f32,
    pub(super) value: f32,
    pub(super) page_direction: f32,
    pub(super) repeat_at: f32,
}

/// Unity `SliderHandler.PageMovementValue` pages towards the pointer side.
pub(super) fn user_equip_scroll_page_direction(cursor_y: f32, thumb_top: f32, thumb_height: f32) -> f32 {
    if cursor_y < thumb_top {
        -1.0
    } else if cursor_y > thumb_top + thumb_height {
        1.0
    } else {
        0.0
    }
}

/// Unity IMGUI scrollbar input for `Panel_PCStuff`'s `GUI.BeginScrollView`:
/// the thumb drags with its grab offset; the trough pages by 90% of the view
/// after 250 ms, then every 30 ms while the pointer stays on that side.
#[allow(clippy::type_complexity)]
pub(super) fn collect_user_equip_scrollbar_pointer(
    time: Option<Res<Time>>,
    input: UserEquipPointerInput,
    windows: Query<&Window, With<PrimaryWindow>>,
    state: Res<UserEquipUiState>,
    modal: Res<UserEquipModalState>,
    parts: Query<
        (
            &Interaction,
            &ComputedNode,
            &UiGlobalTransform,
            Has<UserEquipScrollThumbControl>,
        ),
        Or<(
            With<UserEquipScrollTrackControl>,
            With<UserEquipScrollThumbControl>,
        )>,
    >,
    mut outbox: ResMut<UserEquipUiOutbox>,
    mut pointer: Local<UserEquipScrollbarPointer>,
) {
    let now = time.map_or(0.0, |time| time.elapsed_secs());
    let buttons = input.mouse_buttons.as_deref();
    let held = buttons.is_some_and(|buttons| buttons.pressed(MouseButton::Left));
    let just_pressed = buttons.is_some_and(|buttons| buttons.just_pressed(MouseButton::Left));
    let escape = input
        .keyboard
        .as_ref()
        .is_some_and(|keyboard| keyboard.just_pressed(KeyCode::Escape));
    let metrics = UserEquipScrollbarMetrics::for_mode(state.mode());
    let window = windows.single().ok().filter(|window| window.focused);
    let Some(window) = window.filter(|_| {
        held && !escape && metrics.scroll_max > 0.0 && state.input_capabilities(*modal).scroll
    }) else {
        *pointer = UserEquipScrollbarPointer::default();
        return;
    };
    // Like Unity's hot control, a held drag survives the pointer leaving the
    // window and resumes from its original grab once the pointer returns.
    let Some(cursor) = window.physical_cursor_position() else {
        return;
    };
    let hovered = parts
        .iter()
        .filter(|(interaction, ..)| **interaction != Interaction::None)
        .map(|(.., thumb)| {
            if thumb {
                UserEquipScrollbarPart::Thumb
            } else {
                UserEquipScrollbarPart::Track
            }
        })
        .min_by_key(|part| *part != UserEquipScrollbarPart::Thumb);
    if just_pressed {
        *pointer = UserEquipScrollbarPointer {
            held: hovered,
            mode: Some(state.mode()),
            repeat_at: now + 0.25,
            ..default()
        };
    }
    let Some(part) = pointer.held else {
        return;
    };
    let track = parts
        .iter()
        .find(|(.., thumb)| !thumb)
        .filter(|_| pointer.mode == Some(state.mode()));
    let Some((_, track, transform, _)) = track else {
        *pointer = UserEquipScrollbarPointer::default();
        return;
    };
    let cursor_y =
        (cursor.y - transform.translation.y + track.size().y * 0.5) * track.inverse_scale_factor();
    let current = state.scroll_y();
    let side = user_equip_scroll_page_direction(
        cursor_y,
        metrics.thumb_offset(current),
        metrics.thumb_height,
    );
    if just_pressed {
        pointer.anchor = cursor_y;
        pointer.value = current;
        pointer.page_direction = side;
    }
    let scroll_y = match part {
        UserEquipScrollbarPart::Thumb => {
            metrics.drag_value(pointer.value, cursor_y - pointer.anchor)
        }
        UserEquipScrollbarPart::Track => {
            if hovered != Some(UserEquipScrollbarPart::Track)
                || (!just_pressed && now < pointer.repeat_at)
            {
                return;
            }
            if !just_pressed {
                pointer.repeat_at = now + 0.03;
            }
            if side != pointer.page_direction {
                pointer.held = None;
                return;
            }
            (current + USER_EQUIP_INVENTORY_VIEWPORT_RECT.height * 0.9 * side)
                .clamp(0.0, metrics.scroll_max)
        }
    };
    if scroll_y != current {
        outbox.push(UserEquipUiAction::SetScrollY { scroll_y });
    }
}

pub(super) fn execute_user_equip_one_click(
    endpoint: UserEquipSlotEndpoint,
    projection: &UserEquipItemModeProjection,
    outbox: &mut UserEquipUiOutbox,
) {
    let Some(projected) = projection.item_at(endpoint).filter(|item| !item.empty) else {
        return;
    };
    // `InventoryManagerScript.OneClickItem` moves into the opposite location:
    // an equipment cell always returns its item to the first empty
    // authoritative bag slot, exactly like the popup's Unequip command and the
    // equipment-to-bag drag. Without this branch the secondary button could
    // only equip.
    let UserEquipSlotEndpoint::Inventory { slot_index } = endpoint else {
        if let Some(slot_index) = projection.first_empty_inventory_slot() {
            outbox.push(UserEquipUiAction::MoveItem {
                from: endpoint,
                to: UserEquipSlotEndpoint::Inventory { slot_index },
            });
        }
        return;
    };
    match projected.item.item_type {
        0..=6 | 10 => {
            if let Some(to) = user_equip_equipment_endpoint_for_item(projected.item, false) {
                outbox.push(UserEquipUiAction::MoveItem { from: endpoint, to });
            }
        }
        7 => outbox.push(UserEquipUiAction::UseInventoryItem { slot_index }),
        9 => outbox.push(UserEquipUiAction::OpenInventoryChest { slot_index }),
        _ => {}
    }
}

pub(super) fn execute_user_equip_popup_command(
    command: UserEquipPopupCommand,
    projection: &UserEquipItemModeProjection,
    popup: &mut UserEquipItemPopupState,
    modal: &mut UserEquipModalState,
    outbox: &mut UserEquipUiOutbox,
) {
    let Some(from) = popup.selected() else {
        return;
    };
    let Some(projected) = projection.item_at(from).filter(|item| !item.empty) else {
        popup.close();
        modal.item_popup_active = false;
        return;
    };
    match command {
        UserEquipPopupCommand::Equip
        | UserEquipPopupCommand::EquipPrimary
        | UserEquipPopupCommand::EquipSecondary => {
            let secondary = command == UserEquipPopupCommand::EquipSecondary;
            if let Some(to) = user_equip_equipment_endpoint_for_item(projected.item, secondary) {
                outbox.push(UserEquipUiAction::MoveItem { from, to });
            }
        }
        UserEquipPopupCommand::Unequip => {
            if let Some(slot_index) = projection.first_empty_inventory_slot() {
                outbox.push(UserEquipUiAction::MoveItem {
                    from,
                    to: UserEquipSlotEndpoint::Inventory { slot_index },
                });
            }
        }
        UserEquipPopupCommand::Use => {
            if let UserEquipSlotEndpoint::Inventory { slot_index } = from {
                outbox.push(UserEquipUiAction::UseInventoryItem { slot_index });
            }
        }
        UserEquipPopupCommand::Open => {
            if let UserEquipSlotEndpoint::Inventory { slot_index } = from {
                outbox.push(UserEquipUiAction::OpenInventoryChest { slot_index });
            }
        }
        UserEquipPopupCommand::Delete => {
            if let UserEquipSlotEndpoint::Inventory { slot_index } = from {
                outbox.push(UserEquipUiAction::DeleteInventoryItem { slot_index });
            }
        }
        UserEquipPopupCommand::Cancel => {}
    }
    popup.close();
    modal.item_popup_active = false;
}
