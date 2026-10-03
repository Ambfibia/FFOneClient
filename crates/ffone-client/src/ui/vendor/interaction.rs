use super::*;

pub const VENDOR_TAB_BUY_HOVER_PATH: &str = "ui/en/vendor/tab-buy-hover.png";

pub const VENDOR_TAB_BUYBACK_HOVER_PATH: &str = "ui/en/vendor/tab-buyback-hover.png";

pub const VENDOR_BUTTON_NORMAL_PATH: &str = "ui/en/vendor/button-normal.png";

pub const VENDOR_BUTTON_HOVER_PATH: &str = "ui/en/vendor/button-hover.png";

pub const VENDOR_SCROLL_TRACK_PATH: &str = "ui/en/vendor/scroll-track.png";

pub const VENDOR_SCROLL_THUMB_PATH: &str = "ui/en/vendor/scroll-thumb.png";

pub const VENDOR_SCROLL_UP_PATH: &str = "ui/en/vendor/scroll-up.png";

pub const VENDOR_SCROLL_DOWN_PATH: &str = "ui/en/vendor/scroll-down.png";

pub const VENDOR_SCROLL_SHADOW_PATH: &str = "ui/en/vendor/scroll-shadow.png";

pub const VENDOR_BUTTON_SOURCE_FONT_PATH_ID: i64 = 933;

pub const VENDOR_BUTTON_FONT_SIZE: f32 = 11.0;

pub const VENDOR_BUTTON_PADDING_LEFT: f32 = 6.0;

pub const VENDOR_BUTTON_PADDING_RIGHT: f32 = 6.0;

pub const VENDOR_BUTTON_PADDING_TOP: f32 = 3.0;

pub const VENDOR_BUTTON_PADDING_BOTTOM: f32 = 3.0;

pub const VENDOR_BUTTON_NORMAL_TEXT_RGB: [f32; 3] = [0.9, 0.9, 0.9];

pub const VENDOR_BUTTON_HOVER_TEXT_RGB: [f32; 3] = [0.229_838_71, 0.463_709_68, 1.0];

pub const VENDOR_SCROLL_VELOCITY: f32 = 200.0;

pub const VENDOR_SCROLL_INPUT_CLAMP: f32 = 30.0;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VendorInputCapabilities {
    pub draw: bool,
    pub vendor_controls: bool,
    pub pc_stuff_controls: bool,
    pub tabs: bool,
    pub scroll: bool,
    pub row_actions: bool,
    pub close: bool,
    pub help: bool,
    pub go_to_stuff: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VendorScrollTarget {
    Vendor,
    Inventory,
}

#[must_use]
pub fn vendor_scroll_max(row_count: usize) -> f32 {
    (row_count as f32 * VENDOR_ROW_HEIGHT - VENDOR_LIST_VIEWPORT_RECT.height).max(0.0)
}

#[must_use]
pub fn clamp_vendor_scroll(scroll_y: f32, row_count: usize) -> f32 {
    if scroll_y.is_nan() || scroll_y <= 0.0 {
        0.0
    } else if !scroll_y.is_finite() {
        vendor_scroll_max(row_count)
    } else {
        scroll_y.min(vendor_scroll_max(row_count))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub(super) struct VendorHoverState {
    pub(super) buy: bool,
    pub(super) buyback: bool,
    pub(super) go_to_stuff: bool,
    pub(super) redeem_code: bool,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn collect_vendor_ui_input(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mouse_buttons: Option<Res<ButtonInput<MouseButton>>>,
    mouse_scroll: Option<Res<AccumulatedMouseScroll>>,
    modal: Res<VendorModalState>,
    close_gate: Res<VendorCloseGate0104>,
    projection: Res<VendorModeProjection0104>,
    windows: Query<&Window, With<PrimaryWindow>>,
    controls: Query<(&VendorInteractiveControl, &Interaction), Changed<Interaction>>,
    all_controls: Query<(&VendorInteractiveControl, &Interaction)>,
    mut state: ResMut<VendorUiState>,
    mut hover: ResMut<VendorHoverState>,
    mut outbox: ResMut<VendorUiOutbox0104>,
    mut audio: ResMut<VendorUiAudioOutbox0104>,
) {
    hover.buy = all_controls.iter().any(|(control, interaction)| {
        *control == VendorInteractiveControl::BuyTab && *interaction == Interaction::Hovered
    });
    hover.buyback = all_controls.iter().any(|(control, interaction)| {
        *control == VendorInteractiveControl::BuybackTab && *interaction == Interaction::Hovered
    });
    hover.go_to_stuff = all_controls.iter().any(|(control, interaction)| {
        *control == VendorInteractiveControl::GoToStuff && *interaction == Interaction::Hovered
    });
    hover.redeem_code = all_controls.iter().any(|(control, interaction)| {
        *control == VendorInteractiveControl::RedeemCode && *interaction == Interaction::Hovered
    });

    if keyboard
        .as_ref()
        .is_some_and(|keyboard| keyboard.just_pressed(KeyCode::Escape))
        && state
            .request_escape(*modal, *close_gate, &mut outbox)
            .is_ok()
    {
        return;
    }

    for (control, interaction) in &controls {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *control {
            VendorInteractiveControl::BuyTab => {
                let changed = state.tab != VendorTab0104::Buy;
                if state.switch_tab(*modal, VendorTab0104::Buy).is_ok() && changed {
                    audio.push(VendorUiAudioCue0104::TabClick01);
                }
            }
            VendorInteractiveControl::BuybackTab => {
                let changed = state.tab != VendorTab0104::Buyback;
                if state.switch_tab(*modal, VendorTab0104::Buyback).is_ok() && changed {
                    audio.push(VendorUiAudioCue0104::TabClick01);
                }
            }
            VendorInteractiveControl::Row(row_index) => {
                if state.input_capabilities(*modal).row_actions
                    && row_index < projection.rows_for_tab(state.tab)
                {
                    audio.push(VendorUiAudioCue0104::ButtonSound);
                    let tab = state.tab;
                    let activation = projection.primary_row_activation(tab, row_index);
                    dispatch_vendor_activation(&mut state, *modal, activation, false, &mut outbox);
                }
            }
            VendorInteractiveControl::InventorySlot(inventory_slot) => {
                if state.input_capabilities(*modal).pc_stuff_controls
                    && projection
                        .inventory
                        .get(inventory_slot)
                        .is_some_and(|slot| !slot.item.empty)
                {
                    audio.push(VendorUiAudioCue0104::ButtonSound);
                    dispatch_vendor_activation(
                        &mut state,
                        *modal,
                        projection.primary_inventory_activation(inventory_slot),
                        true,
                        &mut outbox,
                    );
                }
            }
            VendorInteractiveControl::GoToStuff => {
                if state.request_go_to_stuff(*modal, &mut outbox).is_ok() {
                    return;
                }
            }
            VendorInteractiveControl::RedeemCode => {
                if state.request_redeem_code(*modal, &mut outbox).is_ok() {
                    audio.push(VendorUiAudioCue0104::ButtonSound);
                }
            }
            VendorInteractiveControl::Close => {
                if state
                    .request_close_button(*modal, *close_gate, &mut outbox)
                    .is_ok()
                {
                    return;
                }
            }
            VendorInteractiveControl::Help => {
                if state.request_help(*modal, &mut outbox).is_ok() {
                    audio.push(VendorUiAudioCue0104::ButtonSound);
                }
            }
        }
    }

    if mouse_buttons
        .as_ref()
        .is_some_and(|mouse| mouse.just_released(MouseButton::Right))
    {
        let capabilities = state.input_capabilities(*modal);
        let mut secondary_control = None;
        let mut ambiguous = false;
        for (control, interaction) in &all_controls {
            if *interaction != Interaction::Hovered {
                continue;
            }
            let eligible = match *control {
                VendorInteractiveControl::Row(row_index) => {
                    capabilities.row_actions && row_index < projection.rows_for_tab(state.tab)
                }
                VendorInteractiveControl::InventorySlot(inventory_slot) => {
                    capabilities.pc_stuff_controls
                        && projection
                            .inventory
                            .get(inventory_slot)
                            .is_some_and(|slot| !slot.item.empty)
                }
                _ => false,
            };
            if !eligible {
                continue;
            }
            if secondary_control.replace(*control).is_some() {
                ambiguous = true;
                break;
            }
        }
        if !ambiguous {
            match secondary_control {
                Some(VendorInteractiveControl::Row(row_index)) => {
                    audio.push(VendorUiAudioCue0104::ButtonSound);
                    let tab = state.tab;
                    let activation = projection.secondary_row_activation(tab, row_index);
                    dispatch_vendor_activation(&mut state, *modal, activation, false, &mut outbox);
                }
                Some(VendorInteractiveControl::InventorySlot(inventory_slot)) => {
                    audio.push(VendorUiAudioCue0104::ButtonSound);
                    dispatch_vendor_activation(
                        &mut state,
                        *modal,
                        projection.secondary_inventory_activation(inventory_slot),
                        true,
                        &mut outbox,
                    );
                }
                _ => {}
            }
        }
    }

    if !state.input_capabilities(*modal).scroll {
        return;
    }
    let axis = mouse_scroll.as_ref().map_or(0.0, |scroll| scroll.delta.y);
    if !axis.is_finite() || axis == 0.0 {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let layout = vendor_mode_layout(
        window.width().max(0.0) as u32,
        window.height().max(0.0) as u32,
        state.opening_elapsed_seconds,
        state.vendor_scroll_y,
        state.inventory_scroll_y,
        projection.rows_for_tab(state.tab),
    );
    let target = if layout.list_viewport.contains(cursor) {
        Some(VendorScrollTarget::Vendor)
    } else if VendorUiRect::from(layout.item_mode.pc_stuff_panel).contains(cursor) {
        Some(VendorScrollTarget::Inventory)
    } else {
        None
    };
    if let Some(target) = target {
        let row_count = projection.rows_for_tab(state.tab);
        let _ = state.apply_scroll_axis(*modal, target, axis, row_count);
    }
}

pub(super) fn vendor_button_text_color(hovered: bool) -> Color {
    let [red, green, blue] = if hovered {
        VENDOR_BUTTON_HOVER_TEXT_RGB
    } else {
        VENDOR_BUTTON_NORMAL_TEXT_RGB
    };
    Color::srgb(red, green, blue)
}
