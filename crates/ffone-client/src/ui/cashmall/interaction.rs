use super::*;

pub const CASHMALL_BUTTON_FONT_PATH_ID: i64 = 933;

/// `mItemNums` is never populated, so Unity's automatic BeginScrollView
/// scrollbar has a zero-height content rect and is not drawn.
pub const CASHMALL_AUTOMATIC_SCROLLBAR_REACHABLE: bool = false;

pub const CASHMALL_BUTTON_FONT_SIZE: f32 = 11.0;

pub const CASHMALL_BUTTON_PADDING_LEFT: f32 = 6.0;

pub const CASHMALL_BUTTON_PADDING_RIGHT: f32 = 6.0;

pub const CASHMALL_BUTTON_PADDING_TOP: f32 = 3.0;

pub const CASHMALL_BUTTON_PADDING_BOTTOM: f32 = 3.0;

pub const CASHMALL_FIRST_TAB_HOVER_PATH: &str = "ui/en/cashmall/first-tab-button-hover.png";

pub const CASHMALL_SECOND_TAB_HOVER_PATH: &str = "ui/en/cashmall/second-tab-button-hover.png";

pub const CASHMALL_NANO_TAB_HOVER_PATH: &str = "ui/en/gameplay/journal/nanotabover.png";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CashmallInputCapabilities0104 {
    pub draw: bool,
    pub cashmall_controls: bool,
    pub pc_stuff_controls: bool,
    pub tabs: bool,
    pub row_actions: bool,
    pub go_to_stuff: bool,
    pub close_button: bool,
    pub update_scroll: bool,
    pub escape_request: bool,
    pub cursor_forced_unlocked: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CashmallScrollTarget0104 {
    Cashmall,
    #[default]
    PcStuff,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CashmallPointerButton0104 {
    Primary,
    Secondary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CashmallVendorClickBoundary0104 {
    pub slot_type: i32,
    pub slot_id: i32,
    pub item: ItemBase0104,
    pub popup_action: i32,
    pub popup_rect: [i32; 4],
}

#[derive(Clone, Copy, Debug, Default, Resource)]
pub(super) struct CashmallHoverState0104 {
    pub(super) hovered_tabs: [bool; CASHMALL_TAB_COUNT],
    pub(super) pressed_tabs: [bool; CASHMALL_TAB_COUNT],
    pub(super) go_to_stuff: bool,
    pub(super) go_to_stuff_pressed: bool,
    pub(super) nano_tab: bool,
    pub(super) redeem_code: bool,
    pub(super) redeem_code_pressed: bool,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn collect_cashmall_ui_input_0104(
    mouse_buttons: Option<Res<ButtonInput<MouseButton>>>,
    mouse_scroll: Option<Res<AccumulatedMouseScroll>>,
    modal: Res<CashmallModalState0104>,
    close_gate: Res<CashmallCloseGate0104>,
    projection: Res<CashmallModeProjection0104>,
    windows: Query<&Window, With<PrimaryWindow>>,
    controls: Query<(&CashmallInteractiveControl0104, &Interaction), Changed<Interaction>>,
    all_controls: Query<(&CashmallInteractiveControl0104, &Interaction)>,
    mut state: ResMut<CashmallUiState0104>,
    mut hover: ResMut<CashmallHoverState0104>,
    mut outbox: ResMut<CashmallUiOutbox0104>,
) {
    hover.hovered_tabs = [false; CASHMALL_TAB_COUNT];
    hover.pressed_tabs = [false; CASHMALL_TAB_COUNT];
    hover.go_to_stuff = false;
    hover.go_to_stuff_pressed = false;
    hover.nano_tab = false;
    hover.redeem_code = false;
    hover.redeem_code_pressed = false;
    for (control, interaction) in &all_controls {
        match *control {
            CashmallInteractiveControl0104::Tab(tab) => {
                hover.hovered_tabs[tab.index()] =
                    matches!(*interaction, Interaction::Hovered | Interaction::Pressed);
                hover.pressed_tabs[tab.index()] = *interaction == Interaction::Pressed;
            }
            CashmallInteractiveControl0104::GoToStuff => {
                hover.go_to_stuff = *interaction == Interaction::Hovered;
                hover.go_to_stuff_pressed = *interaction == Interaction::Pressed;
            }
            CashmallInteractiveControl0104::NanoTab => {
                hover.nano_tab =
                    matches!(*interaction, Interaction::Hovered | Interaction::Pressed);
            }
            CashmallInteractiveControl0104::RedeemCode => {
                hover.redeem_code = *interaction == Interaction::Hovered;
                hover.redeem_code_pressed = *interaction == Interaction::Pressed;
            }
            _ => {}
        }
    }

    for (control, interaction) in &controls {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *control {
            CashmallInteractiveControl0104::Tab(tab) => {
                let _ = state.select_tab(*modal, tab, &mut outbox);
            }
            CashmallInteractiveControl0104::Row(row) => {
                let _ = state.activate_row(
                    *modal,
                    &projection,
                    row,
                    CashmallPointerButton0104::Primary,
                    &mut outbox,
                );
            }
            CashmallInteractiveControl0104::GoToStuff => {
                if state.request_go_to_stuff(*modal, &mut outbox).is_ok() {
                    return;
                }
            }
            CashmallInteractiveControl0104::NanoTab => {
                let _ = state.request_nano_tab(*modal);
            }
            CashmallInteractiveControl0104::RedeemCode => {
                let _ = state.request_redeem_code(*modal);
            }
            CashmallInteractiveControl0104::Close => {
                if state
                    .request_close(
                        CashmallCloseSource0104::PcStuffCloseButton,
                        *modal,
                        *close_gate,
                        &mut outbox,
                    )
                    .is_ok()
                {
                    return;
                }
            }
            CashmallInteractiveControl0104::Help => {
                let _ = state.request_help(*modal, &mut outbox);
            }
        }
    }

    if mouse_buttons
        .as_ref()
        .is_some_and(|mouse| mouse.just_released(MouseButton::Right))
    {
        let mut hovered_row = None;
        let mut ambiguous = false;
        for (control, interaction) in &all_controls {
            if *interaction != Interaction::Hovered {
                continue;
            }
            let CashmallInteractiveControl0104::Row(row) = *control else {
                continue;
            };
            if hovered_row.replace(row).is_some() {
                ambiguous = true;
                break;
            }
        }
        if !ambiguous {
            if let Some(row) = hovered_row {
                let _ = state.activate_row(
                    *modal,
                    &projection,
                    row,
                    CashmallPointerButton0104::Secondary,
                    &mut outbox,
                );
            }
        }
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
    let layout = cashmall_mode_layout_0104(
        window.width().max(0.0) as u32,
        window.height().max(0.0) as u32,
        state.opening_elapsed_seconds,
        state.inventory_scroll_y,
    );
    state.note_scroll_areas(
        CashmallUiRect0104::from(layout.item_mode.inventory_viewport).contains(cursor),
        layout.table.contains(cursor),
    );
    let _ = state.apply_scroll_axis(*modal, axis);
}

pub(super) fn cashmall_button_text_color_0104(enabled: bool, hovered: bool, pressed: bool) -> Color {
    if !enabled {
        Color::BLACK
    } else if hovered && !pressed {
        Color::srgb(0.229_838_71, 0.463_709_68, 1.0)
    } else {
        Color::srgb(0.9, 0.9, 0.9)
    }
}
