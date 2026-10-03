use super::*;

#[derive(Component)]
pub struct BankItemDeleteButton;

pub const BANK_SCROLL_TRACK_PATH: &str = "ui/en/bank-mode/scroll-track.png";

pub const BANK_SCROLL_THUMB_PATH: &str = "ui/en/bank-mode/scroll-thumb.png";

pub const BANK_SCROLL_UP_PATH: &str = "ui/en/bank-mode/scroll-up.png";

pub const BANK_SCROLL_DOWN_PATH: &str = "ui/en/bank-mode/scroll-down.png";

pub const BANK_SCROLL_SHADOW_PATH: &str = "ui/en/bank-mode/scroll-shadow.png";

/// Byte-different PNG encoding, pixel-identical to clean pathId 640.
pub const BANK_SLOT_BUTTON_PATH: &str = "ui/en/gameplay/chat/blue_button_normal.png";

pub const BANK_SLOT_BUTTON_SOURCE_PATH_ID: i64 = 640;

pub const BANK_SLOT_BUTTON_SHA256: &str =
    "506EA52CF108EEB14A035FA52A4C650BA1A5255B6E4FB62A10ADC3AEDFC72C8E";

pub const BANK_SCROLL_VELOCITY: f32 = 200.0;

pub const BANK_BUTTON_SOURCE_FONT_PATH_ID: i64 = 933;

/// Approved JEFFE replacement calibration for clean pathId 933
/// (`JEFFE___14`) used by the default `button` style.
pub const BANK_BUTTON_FONT_SIZE: f32 = 11.0;

pub const BANK_BUTTON_PADDING_LEFT: f32 = 6.0;

pub const BANK_BUTTON_PADDING_RIGHT: f32 = 6.0;

pub const BANK_BUTTON_PADDING_TOP: f32 = 3.0;

pub const BANK_BUTTON_PADDING_BOTTOM: f32 = 3.0;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BankInputCapabilities {
    pub draw: bool,
    pub close: bool,
    pub scroll: bool,
    pub item_move: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankScrollTarget {
    Bank,
    Inventory,
}

#[must_use]
pub const fn bank_scroll_max() -> f32 {
    BANK_CONTENT_HEIGHT - BANK_VIEWPORT_HEIGHT
}

#[must_use]
pub fn clamp_bank_scroll(scroll_y: f32) -> f32 {
    if scroll_y.is_nan() || scroll_y <= 0.0 {
        0.0
    } else if !scroll_y.is_finite() {
        bank_scroll_max()
    } else {
        scroll_y.min(bank_scroll_max())
    }
}

#[derive(Clone, Copy)]
pub(super) struct BankPointerDraft {
    pub(super) source: BankSlotRef0104,
    pub(super) item: ItemBase0104,
    pub(super) owner_pc_id: i32,
    pub(super) npc_id: i32,
    pub(super) button: MouseButton,
    pub(super) origin: Vec2,
    pub(super) press_position: Vec2,
    pub(super) dragged: bool,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn collect_bank_ui_input(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mouse_scroll: Option<Res<AccumulatedMouseScroll>>,
    modal: Res<BankModalState>,
    search: Res<BankSearch>,
    windows: Query<&Window, With<PrimaryWindow>>,
    close_buttons: Query<&Interaction, (With<BankCloseControl>, Changed<Interaction>)>,
    mut state: ResMut<BankUiState>,
    mut outbox: ResMut<BankUiOutbox0104>,
) {
    let capabilities = state.input_capabilities(*modal);
    let close_requested = capabilities.close
        && (keyboard
            .as_ref()
            .is_some_and(|keyboard| keyboard.just_pressed(KeyCode::Escape))
            || close_buttons
                .iter()
                .any(|interaction| *interaction == Interaction::Pressed));
    if close_requested && state.request_close(*modal, &mut outbox).is_ok() {
        return;
    }
    if !capabilities.scroll {
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
    let layout = bank_mode_layout(
        window.width().max(0.0) as u32,
        window.height().max(0.0) as u32,
        state.opening_elapsed_seconds,
        state.bank_scroll_y,
        state.inventory_scroll_y,
    );
    if layout.bank_dialog.contains(cursor) {
        state.apply_scroll_axis(BankScrollTarget::Bank, axis);
        state.bank_scroll_y = state.bank_scroll_y.min(search.maximum());
    } else if BankUiRect::from(layout.item_mode.pc_stuff_panel).contains(cursor) {
        state.apply_scroll_axis(BankScrollTarget::Inventory, axis);
    }
}

pub(super) fn collect_bank_slot_input(
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    slots: Query<(
        &BankSlotControl,
        &Interaction,
        Option<&UiGlobalTransform>,
        Option<&ComputedNode>,
    )>,
    modal: Res<BankModalState>,
    projection: Res<BankModeProjection0104>,
    mut state: ResMut<BankUiState>,
    mut outbox: ResMut<BankUiOutbox0104>,
    mut draft: Local<Option<BankPointerDraft>>,
    mut visual: ResMut<drag::DragVisual>,
    mut requests: ResMut<BankLocalRequests>,
    mut popup: ResMut<BankItemPopupState>,
    mut audio: Option<ResMut<crate::gameplay_audio::GameplayAudioRuntime>>,
) {
    let Some(mouse) = mouse else {
        *draft = None;
        visual.cancel();
        return;
    };
    if !state.input_capabilities(*modal).item_move || windows.iter().any(|window| !window.focused) {
        *draft = None;
        if !visual.dropping
            || modal.any()
            || state.phase == BankLifecyclePhase::Hidden
            || windows.iter().any(|window| !window.focused)
        {
            visual.cancel();
        }
        return;
    }
    // Bevy's UI picking respects the scroll viewport and overlaid controls.
    // Ambiguous hits never select an arbitrary item by ECS iteration order.
    if draft.is_none()
        && !mouse.just_pressed(MouseButton::Left)
        && !mouse.just_pressed(MouseButton::Right)
    {
        return;
    }
    let mut hits = slots
        .iter()
        .filter(|(_, interaction, _, _)| **interaction != Interaction::None);
    let hovered = hits.next().map(|(slot, _, _, _)| slot.0);
    let hovered = if hits.next().is_none() { hovered } else { None };
    let center = |target| {
        slots
            .iter()
            .find(|(slot, _, _, _)| slot.0 == target)
            .and_then(|(_, _, transform, node)| {
                Some(transform?.translation * node?.inverse_scale_factor())
            })
    };
    let cursor = windows
        .iter()
        .next()
        .and_then(|w| w.physical_cursor_position())
        .map(|p| {
            let scale = slots
                .iter()
                .find_map(|(_, _, _, node)| node.map(|n| n.inverse_scale_factor()))
                .unwrap_or(1.);
            p * scale
        });
    if let Some(button) = [MouseButton::Left, MouseButton::Right]
        .into_iter()
        .find(|button| mouse.just_pressed(*button))
    {
        *draft = hovered
            .filter(|slot| !projection.slot_locked(*slot))
            .filter(|slot| projection.item_at(*slot).item_id > 0)
            .map(|source| BankPointerDraft {
                source,
                item: projection.item_at(source),
                owner_pc_id: projection.owner_pc_id,
                npc_id: projection.npc_id,
                button,
                origin: center(source).or(cursor).unwrap_or(Vec2::ZERO),
                press_position: cursor.unwrap_or(Vec2::ZERO),
                dragged: false,
            });
        if draft.is_some() && let Some(audio) = audio.as_mut() {
            audio.queue_legacy_button_sound();
        }
    }
    let Some(pressed) = *draft else {
        return;
    };
    if projection.owner_pc_id != pressed.owner_pc_id
        || projection.npc_id != pressed.npc_id
        || projection.item_at(pressed.source) != pressed.item
        || projection.slot_locked(pressed.source)
    {
        *draft = None;
        visual.cancel();
        return;
    }
    if pressed.button == MouseButton::Left && mouse.pressed(MouseButton::Left) {
        if let Some(cursor) = cursor {
            if !pressed.dragged && cursor.distance_squared(pressed.press_position) > 1. {
                visual.start(pressed.source, &projection, pressed.origin);
                if let Some(draft) = draft.as_mut() {
                    draft.dragged = true;
                }
            }
            visual.follow(cursor);
        }
    }
    if mouse.just_released(pressed.button) {
        *draft = None;
        match (pressed.button, hovered) {
            (MouseButton::Right, Some(target)) if target == pressed.source => {
                match state.request_one_click(*modal, &projection, pressed.source, &mut outbox) {
                    Err(BankActionBlocked::Transfer(BankTransferError0104::BankFull)) => {
                        requests.full = Some(BankSlotLocation0104::Bank)
                    }
                    Err(BankActionBlocked::Transfer(BankTransferError0104::InventoryFull)) => {
                        requests.full = Some(BankSlotLocation0104::Inventory)
                    }
                    _ => {}
                }
            }
            (MouseButton::Left, Some(target)) if target == pressed.source => {
                if !pressed.dragged {
                    popup.open(&projection, target);
                }
                visual.drop_at(center(target));
            }
            (MouseButton::Left, Some(target)) if target != pressed.source => {
                let accepted = state
                    .request_transfer(*modal, &projection, pressed.source, target, &mut outbox)
                    .is_ok();
                visual.drop_at(if accepted { center(target) } else { None });
            }
            _ => {
                visual.drop_at(None);
            }
        }
    } else if !mouse.pressed(pressed.button) {
        *draft = None;
        visual.cancel();
    }
}

pub(super) fn bind_bank_button_visual(
    image: &mut ImageNode,
    interaction: Option<&Interaction>,
    enabled: bool,
    assets: &BankUiAssets,
) {
    let hovered = enabled && interaction.is_some_and(|i| *i == Interaction::Hovered);
    let handle = assets.image(if hovered {
        BankStaticAssetRole::ButtonHover
    } else {
        BankStaticAssetRole::BankSlotButton
    });
    if image.image != handle {
        image.image = handle;
    }
    let color = if enabled {
        Color::WHITE
    } else {
        Color::srgba(1.0, 1.0, 1.0, 0.5)
    };
    if image.color != color {
        image.color = color;
    }
}

pub(super) fn bind_bank_button_labels(
    assets: If<Res<BankUiAssets>>,
    buttons: Query<&ImageNode, With<BankRedeemControl>>,
    mut labels: Query<(&BankUiElement, &ChildOf, &mut TextColor)>,
) {
    for (element, parent, mut label) in &mut labels {
        if *element != BankUiElement::RedeemCodeLabel {
            continue;
        }
        let Ok(image) = buttons.get(parent.parent()) else {
            continue;
        };
        let color = if image.image == assets.image(BankStaticAssetRole::ButtonHover) {
            Color::srgb(0.229838714, 0.463709682, 1.0)
        } else {
            Color::srgb(0.9, 0.9, 0.9)
        }
        .with_alpha(image.color.alpha());
        if label.0 != color {
            label.0 = color;
        }
    }
}
