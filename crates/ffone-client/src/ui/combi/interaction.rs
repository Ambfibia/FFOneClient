use super::*;

pub const COMBI_NPC_BUTTON_TYPE_0104: i32 = 20;

pub const COMBI_BUTTON_NORMAL_PATH: &str = "ui/en/combi/button-normal.png";

pub const COMBI_BUTTON_HOVER_PATH: &str = "ui/en/combi/button-hover.png";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CombiInputCapabilities0104 {
    pub draw: bool,
    pub main_controls: bool,
    pub selection_controls: bool,
    pub clear: bool,
    pub combine: bool,
    pub close: bool,
    pub success_controls: bool,
}

/// UI-local pointer state of the Croc-Pot bag. The runtime stays the
/// authority for drops; this only tracks the pointer and the scroll offset.
#[derive(Clone, Copy, Debug, Default, PartialEq, Resource)]
pub struct CombiPointerState0104 {
    pub(super) carried: Option<CombiCarriedItem0104>,
    pub(super) scroll_y: f32,
}

impl CombiPointerState0104 {
    #[must_use]
    pub const fn carried(&self) -> Option<CombiCarriedItem0104> {
        self.carried
    }

    #[must_use]
    pub const fn scroll_y(&self) -> f32 {
        self.scroll_y
    }

    /// Clean `CnEquip` wheel step into `Panel_PCStuff.Scroll`, the scroll
    /// view My Stuff shares: `axis.clamp(-1, 1) * 200`, subtracted.
    pub fn apply_legacy_scroll_axis(&mut self, axis: f32) {
        let axis = if axis.is_finite() {
            axis.clamp(-1.0, 1.0)
        } else {
            0.0
        };
        self.scroll_y =
            clamp_user_equip_scroll(self.scroll_y - axis * USER_EQUIP_INVENTORY_SCROLL_VELOCITY);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn collect_combi_ui_input(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    mouse_scroll: Option<Res<AccumulatedMouseScroll>>,
    controls: Query<(&CombiInteractiveControl0104, Ref<Interaction>)>,
    state: Res<CombiUiState0104>,
    projection: Res<CombiModeProjection0104>,
    mut pointer: ResMut<CombiPointerState0104>,
    mut outbox: ResMut<CombiUiOutbox0104>,
) {
    let capabilities = state.input_capabilities(&projection);
    if !capabilities.draw {
        // Every opening starts at the top of the bag with nothing carried.
        *pointer = CombiPointerState0104::default();
        return;
    }
    if keyboard
        .as_ref()
        .is_some_and(|keyboard| keyboard.just_pressed(KeyCode::Escape))
        && capabilities.close
    {
        pointer.carried = None;
        outbox.push(CombiUiCommand0104::Close);
        return;
    }
    if !capabilities.selection_controls
        || pointer.carried.is_some_and(|carried| {
            !combi_inventory_slot_draggable(&projection, carried.inventory_index)
        })
    {
        // A popup, the wait, or an authority refresh took the item away.
        cancel_combi_carry(&mut pointer, &mut outbox);
    }
    if capabilities.main_controls {
        let axis = mouse_scroll.as_ref().map_or(0.0, |scroll| {
            user_equip_bevy_wheel_to_legacy_axis(scroll.delta.y)
        });
        if axis != 0.0 {
            pointer.apply_legacy_scroll_axis(axis);
        }
    }

    let left_pressed = mouse
        .as_ref()
        .is_some_and(|mouse| mouse.just_pressed(MouseButton::Left));
    let left_released = mouse
        .as_ref()
        .is_some_and(|mouse| mouse.just_released(MouseButton::Left));
    let left_down = mouse
        .as_ref()
        .is_some_and(|mouse| mouse.pressed(MouseButton::Left));
    if pointer.carried.is_some_and(|carried| carried.held) && !left_down && !left_released {
        // The release never arrived (focus loss, release outside the window).
        cancel_combi_carry(&mut pointer, &mut outbox);
    }

    // Presses before releases, so a press and release in one frame is a click.
    let mut press_took_carry = false;
    for (control, interaction) in &controls {
        if !interaction.is_changed() || *interaction != Interaction::Pressed {
            continue;
        }
        if let Some(slot) = control.drop_slot() {
            if !capabilities.selection_controls {
                continue;
            }
            press_took_carry = true;
            let detachable = match *control {
                CombiInteractiveControl0104::LookSelection => projection.look.is_some(),
                CombiInteractiveControl0104::StatsSelection => projection.stats.is_some(),
                _ => false,
            };
            if pointer.carried.is_some_and(|carried| !carried.held) {
                pointer.carried = None;
                outbox.push(CombiUiCommand0104::DropOnSelection { slot });
            } else if pointer.carried.is_none() && detachable {
                outbox.push(CombiUiCommand0104::DetachSelection { slot });
            }
            continue;
        }
        match *control {
            CombiInteractiveControl0104::InventorySlot(inventory_index)
                if capabilities.selection_controls =>
            {
                press_took_carry = true;
                let clicked_again = pointer.carried.is_some_and(|carried| {
                    carried.inventory_index == inventory_index && !carried.held
                });
                if clicked_again || !combi_inventory_slot_draggable(&projection, inventory_index) {
                    cancel_combi_carry(&mut pointer, &mut outbox);
                } else {
                    pointer.carried = Some(CombiCarriedItem0104 {
                        inventory_index,
                        held: true,
                    });
                    outbox.push(CombiUiCommand0104::BeginInventoryDrag { inventory_index });
                }
            }
            CombiInteractiveControl0104::EquipmentSlot(visual_index)
                if capabilities.selection_controls
                    && visual_index < USER_EQUIP_EQUIPMENT_STRIP_COUNT =>
            {
                let wire_slot = USER_EQUIP_EQUIPMENT_STRIP_ORDER[visual_index].wire_slot_index;
                if projection.equipment[wire_slot].item.is_some() {
                    outbox.push(CombiUiCommand0104::RejectEquippedItem {
                        equipment_index: wire_slot,
                        message_id: 260,
                    });
                }
            }
            CombiInteractiveControl0104::ClearAll if capabilities.clear => {
                outbox.push(CombiUiCommand0104::ClearAll);
            }
            CombiInteractiveControl0104::Combine if capabilities.combine => {
                outbox.push(CombiUiCommand0104::Combine);
            }
            CombiInteractiveControl0104::Close if capabilities.close => {
                pointer.carried = None;
                outbox.push(CombiUiCommand0104::Close);
                return;
            }
            CombiInteractiveControl0104::Help if capabilities.main_controls => {
                outbox.push(CombiUiCommand0104::Help);
            }
            CombiInteractiveControl0104::CombineMore if capabilities.success_controls => {
                outbox.push(CombiUiCommand0104::CombineMoreItems);
            }
            CombiInteractiveControl0104::GoToStuff if capabilities.success_controls => {
                outbox.push(CombiUiCommand0104::GoToMyStuff);
            }
            _ => {}
        }
    }
    if left_pressed && !press_took_carry && pointer.carried.is_some_and(|carried| !carried.held) {
        // A click anywhere else puts a click-picked item back.
        cancel_combi_carry(&mut pointer, &mut outbox);
    }

    if left_released
        && let Some(carried) = pointer.carried
        && carried.held
    {
        let mut target = None;
        let mut over_source = false;
        for (control, interaction) in &controls {
            if !matches!(*interaction, Interaction::Hovered | Interaction::Pressed) {
                continue;
            }
            if let Some(slot) = control.drop_slot() {
                target = Some(slot);
            } else if *control
                == CombiInteractiveControl0104::InventorySlot(carried.inventory_index)
            {
                over_source = true;
            }
        }
        if let Some(slot) = target {
            pointer.carried = None;
            outbox.push(CombiUiCommand0104::DropOnSelection { slot });
        } else if over_source {
            // Press and release on the item itself: it stays on the pointer
            // until the next click places it or puts it back.
            pointer.carried = Some(CombiCarriedItem0104 {
                held: false,
                ..carried
            });
        } else {
            cancel_combi_carry(&mut pointer, &mut outbox);
        }
    }
}

pub(super) fn combi_inventory_slot_draggable(
    projection: &CombiModeProjection0104,
    inventory_index: usize,
) -> bool {
    projection
        .inventory
        .get(inventory_index)
        .is_some_and(|slot| slot.item.is_some() && !slot.hidden_by_selection_overlay)
}

/// Swaps a panel icon to its `*-hover` art while it can be clicked.
pub(super) fn bind_hover_image(
    image: Option<Mut<ImageNode>>,
    interaction: Option<&Interaction>,
    enabled: bool,
    [normal, hover]: [CombiStaticAssetRole; 2],
    assets: &CombiUiAssets,
) {
    let Some(mut image) = image else {
        return;
    };
    let hovered = enabled
        && interaction.is_some_and(|interaction| {
            matches!(*interaction, Interaction::Hovered | Interaction::Pressed)
        });
    image.image = assets.image(if hovered { hover } else { normal });
}

pub(super) fn bind_button_visual(
    image: Option<Mut<ImageNode>>,
    interaction: Option<&Interaction>,
    enabled: bool,
    assets: &CombiUiAssets,
) {
    let Some(mut image) = image else {
        return;
    };
    let hovered = enabled
        && interaction.is_some_and(|interaction| {
            matches!(*interaction, Interaction::Hovered | Interaction::Pressed)
        });
    image.image = assets.image(if hovered {
        CombiStaticAssetRole::ButtonHover
    } else {
        CombiStaticAssetRole::ButtonNormal
    });
    image.color = if enabled {
        Color::WHITE
    } else {
        Color::srgba(1.0, 1.0, 1.0, 0.45)
    };
}
