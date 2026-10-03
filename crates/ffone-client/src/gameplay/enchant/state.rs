use super::*;

pub const ENCHANT_ACTIVE_INVENTORY_TAB_0104: i32 = 0;

pub const ENCHANT_MY_STUFF_GAME_MODE_0104: i32 = 6;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnchantInventoryWrite0104 {
    pub inventory_index: usize,
    pub item: ItemBase0104,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnchantRuntimeModal0104 {
    DeleteItem {
        message_id: i32,
        inventory_index: usize,
        item: ItemBase0104,
        quantity: i32,
    },
}

pub(super) fn ensure_runtime_input_available(
    session: &EnchantProductionSession0104,
) -> Result<(), EnchantProductionError0104> {
    ensure_no_pending_request(session)?;
    if session.runtime_modal.is_some() || session.close_decision_pending {
        Err(EnchantProductionError0104::InputBlockedByRuntime)
    } else {
        Ok(())
    }
}

pub(super) fn begin_inventory_drag(
    session: &mut EnchantProductionSession0104,
    inventory_index: usize,
    catalog: &impl EnchantPresentationCatalog0104,
    output: &mut EnchantProductionOutput0104,
) -> Result<(), EnchantProductionError0104> {
    if !session.model.input_capabilities().base_gui_enabled {
        return Err(EnchantModelError0104::InputBlocked.into());
    }
    let Some(item) = session.snapshot.inventory.get(inventory_index).copied() else {
        return Err(EnchantProductionError0104::DragSourceOutOfBounds { inventory_index });
    };
    if EnchantAuthoritySnapshot0104::item_is_empty(item) {
        return Err(EnchantProductionError0104::EmptyDragSource { inventory_index });
    }
    let reserved = session.reserved_by_source[inventory_index];
    let available_quantity = if item.item_type == 7 {
        let available = item.option - reserved;
        if item.option <= 0 {
            return Err(EnchantProductionError0104::InvalidStackQuantity {
                inventory_index,
                quantity: item.option,
            });
        }
        if available <= 0 {
            return Err(EnchantProductionError0104::DragSourceAlreadyReserved { inventory_index });
        }
        available
    } else {
        if reserved > 0 {
            return Err(EnchantProductionError0104::DragSourceAlreadyReserved { inventory_index });
        }
        1
    };
    let presentation =
        catalog
            .presentation_for(item)
            .ok_or(EnchantProductionError0104::MissingPresentation {
                item_type: item.item_type,
                item_id: item.item_id,
            })?;
    let mut selectable_item = item;
    if selectable_item.item_type == 7 {
        selectable_item.option = available_quantity;
    }
    session.drag = Some(EnchantDrag0104 {
        inventory_index,
        authority_item: item,
        selectable: EnchantSelectableItem0104 {
            source_slot: inventory_index as i32,
            item: selectable_item,
            presentation,
        },
    });
    output.effects.push(EnchantShellEffect0104::DragCaptured {
        inventory_index,
        available_quantity,
    });
    Ok(())
}

pub(super) fn apply_selection_intent(
    session: &mut EnchantProductionSession0104,
    intent: &EnchantSelectionIntent0104,
) -> Result<(), EnchantProductionError0104> {
    match *intent {
        EnchantSelectionIntent0104::Reserve {
            slot,
            source_slot,
            quantity,
        } => {
            if quantity <= 0 {
                return Err(EnchantProductionError0104::ReservationQuantityInvalid {
                    slot,
                    quantity,
                });
            }
            let source = usize::try_from(source_slot).map_err(|_| {
                EnchantProductionError0104::ReservationSourceOutOfBounds { source_slot }
            })?;
            let Some(authority_item) = session.snapshot.inventory.get(source).copied() else {
                return Err(EnchantProductionError0104::ReservationSourceOutOfBounds {
                    source_slot,
                });
            };
            if session.reservations_by_attachment[slot.index()].is_some() {
                return Err(EnchantProductionError0104::ReservationSlotAlreadyOccupied { slot });
            }
            let authority_quantity = if authority_item.item_type == 7 {
                authority_item.option
            } else {
                1
            };
            let already = session.reserved_by_source[source];
            if authority_quantity <= 0 || quantity > authority_quantity - already {
                return Err(EnchantProductionError0104::ReservationExceedsAuthority {
                    inventory_index: source,
                    available: (authority_quantity - already).max(0),
                    requested: quantity,
                });
            }
            session.reserved_by_source[source] += quantity;
            session.reservations_by_attachment[slot.index()] = Some(EnchantReservation0104 {
                source_slot,
                quantity,
            });
        }
        EnchantSelectionIntent0104::Release {
            slot,
            source_slot,
            quantity: _,
        } => {
            let reservation = session.reservations_by_attachment[slot.index()]
                .ok_or(EnchantProductionError0104::ReservationMissing { slot })?;
            if reservation.source_slot != source_slot {
                return Err(EnchantProductionError0104::ReservationSourceMismatch {
                    slot,
                    expected: reservation.source_slot,
                    actual: source_slot,
                });
            }
            let source = usize::try_from(source_slot).map_err(|_| {
                EnchantProductionError0104::ReservationSourceOutOfBounds { source_slot }
            })?;
            let Some(total) = session.reserved_by_source.get_mut(source) else {
                return Err(EnchantProductionError0104::ReservationSourceOutOfBounds {
                    source_slot,
                });
            };
            if *total < reservation.quantity {
                return Err(EnchantProductionError0104::ReservationMissing { slot });
            }
            *total -= reservation.quantity;
            session.reservations_by_attachment[slot.index()] = None;
        }
    }
    Ok(())
}
