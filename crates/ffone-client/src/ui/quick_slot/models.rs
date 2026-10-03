use super::*;

#[derive(Clone, Debug, Resource)]
pub struct QuickSlotUiModel {
    pub ready_for_play: bool,
    /// `CnGuiChat` passes false while the friend tab owns the adjacent area.
    pub clicks_enabled: bool,
    pub slots: [LegacyQuickSlotEntry; QUICK_SLOT_COUNT],
}

impl Default for QuickSlotUiModel {
    fn default() -> Self {
        Self {
            ready_for_play: false,
            clicks_enabled: true,
            // The clean static constructor initializes QuickItem to -1.
            slots: array::from_fn(|_| LegacyQuickSlotEntry {
                item_id: -1,
                ..default()
            }),
        }
    }
}

impl QuickSlotUiModel {
    #[must_use]
    pub fn hotkey_action(
        &self,
        config: QuickSlotUiConfig,
        input: QuickSlotInputFrame,
    ) -> Option<QuickSlotUiAction> {
        if config.macro_chat_mode == LegacyMacroChatMode::Use && input.macro_chat_modifier_held {
            return None;
        }

        // The clean source uses a single if/else-if chain. A higher-priority
        // pressed key consumes the frame even if its slot is empty.
        let slot = input
            .slot_just_pressed
            .iter()
            .position(|pressed| *pressed)?;
        self.hotkey_action_for_slot(slot)
    }

    #[must_use]
    pub fn hotkey_action_for_slot(&self, slot: usize) -> Option<QuickSlotUiAction> {
        let entry = self.slots.get(slot)?;
        entry.has_item_id().then_some(QuickSlotUiAction {
            logical_slot: slot,
            legacy_event_index: slot as i32,
            source: QuickSlotActivationSource::Hotkey,
        })
    }

    #[must_use]
    pub fn pointer_action_for_slot(&self, slot: usize) -> Option<QuickSlotUiAction> {
        let entry = self.slots.get(slot)?;
        (self.clicks_enabled
            && entry.has_item_id()
            && entry.has_semantic_icon()
            && !entry.inventory_empty)
            .then_some(QuickSlotUiAction {
                logical_slot: slot,
                // This is intentionally not normalized. `cnQuickSlot.DoQuick`
                // sends i + 1 while keyboard activation sends i.
                legacy_event_index: slot as i32 + 1,
                source: QuickSlotActivationSource::Pointer,
            })
    }
}
