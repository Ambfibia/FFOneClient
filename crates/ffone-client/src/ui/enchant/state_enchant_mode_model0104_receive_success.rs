use super::*;

impl EnchantModeModel0104 {


    pub fn receive_success(
        &mut self,
        reply: EnchantSuccess0104,
    ) -> Result<EnchantReplyDisposition0104, EnchantModelError0104> {
        if matches!(self.phase, EnchantPhase0104::Closed) {
            return Err(EnchantModelError0104::ModeClosed);
        }
        if reply.success_flag == 1 {
            let retained_send_lock = self.send_locked;
            let mut mutations = vec![EnchantInventoryMutation0104 {
                slot: reply.enchant_item_slot,
                item: reply.enchant_item,
            }];
            if reply.weapon_material_item_slot != -1 {
                mutations.push(EnchantInventoryMutation0104 {
                    slot: reply.weapon_material_item_slot,
                    item: reply.weapon_material_item,
                });
            }
            if reply.defence_material_item_slot != -1 {
                mutations.push(EnchantInventoryMutation0104 {
                    slot: reply.defence_material_item_slot,
                    item: reply.defence_material_item,
                });
            }
            self.refresh_selected_item(
                EnchantAttachmentSlot0104::Target,
                reply.enchant_item_slot,
                reply.enchant_item,
            );
            self.refresh_selected_item(
                EnchantAttachmentSlot0104::WeaponMaterial,
                reply.weapon_material_item_slot,
                reply.weapon_material_item,
            );
            self.refresh_selected_item(
                EnchantAttachmentSlot0104::ArmorMaterial,
                reply.defence_material_item_slot,
                reply.defence_material_item,
            );
            self.taros = reply.taros;
            self.intents.push_back(EnchantIntent0104::Audio(
                EnchantAudioIntent0104::CrocPotSuccess,
            ));
            self.intents
                .push_back(EnchantIntent0104::AuthoritativeReceipt(
                    EnchantAuthoritativeReceipt0104 {
                        taros: reply.taros,
                        mutations,
                        success_flag: 1,
                    },
                ));
            self.intents.push_back(EnchantIntent0104::RefreshInventory);
            // Clean does not touch Send here; the success overlay forces its
            // own buttons enabled over whatever send state was already set.
            self.send_locked = retained_send_lock;
            self.phase = EnchantPhase0104::Success { reply };
            Ok(EnchantReplyDisposition0104::SuccessOverlay)
        } else if reply.success_flag == 0 {
            let prior_level = self
                .selection
                .visual_item(EnchantAttachmentSlot0104::Target)
                .map_or(0, |item| item.item.option & 0xFFFF);
            let new_level = reply.enchant_item.option & 0xFFFF;
            let level_loss = prior_level - new_level;
            // Clean failure sends all three inventory events even when either
            // material slot is -1.
            let mutations = vec![
                EnchantInventoryMutation0104 {
                    slot: reply.enchant_item_slot,
                    item: reply.enchant_item,
                },
                EnchantInventoryMutation0104 {
                    slot: reply.weapon_material_item_slot,
                    item: reply.weapon_material_item,
                },
                EnchantInventoryMutation0104 {
                    slot: reply.defence_material_item_slot,
                    item: reply.defence_material_item,
                },
            ];
            self.refresh_selected_item(
                EnchantAttachmentSlot0104::Target,
                reply.enchant_item_slot,
                reply.enchant_item,
            );
            self.refresh_selected_item(
                EnchantAttachmentSlot0104::WeaponMaterial,
                reply.weapon_material_item_slot,
                reply.weapon_material_item,
            );
            self.refresh_selected_item(
                EnchantAttachmentSlot0104::ArmorMaterial,
                reply.defence_material_item_slot,
                reply.defence_material_item,
            );
            self.taros = reply.taros;
            self.send_locked = false;
            self.intents.push_back(EnchantIntent0104::Audio(
                EnchantAudioIntent0104::CrocPotFail,
            ));
            self.intents
                .push_back(EnchantIntent0104::AuthoritativeReceipt(
                    EnchantAuthoritativeReceipt0104 {
                        taros: reply.taros,
                        mutations,
                        success_flag: 0,
                    },
                ));
            self.intents.push_back(EnchantIntent0104::RefreshInventory);
            self.show_system_message(
                ENCHANT_MESSAGE_FAILURE_0104,
                EnchantSystemCallback0104::EnchantFailed,
                Some(level_loss.to_string()),
            );
            Ok(EnchantReplyDisposition0104::FailureMessage)
        } else {
            // The managed method has no else branch.  It leaves bSend true and
            // gives the player no overlay or recovery affordance.
            self.intents
                .push_back(EnchantIntent0104::IgnoredSuccessFlag(reply.success_flag));
            Ok(EnchantReplyDisposition0104::IgnoredLegacyFlag)
        }
    }

    pub fn receive_failure(
        &mut self,
        failure: EnchantFailure0104,
    ) -> Result<(), EnchantModelError0104> {
        if matches!(self.phase, EnchantPhase0104::Closed) {
            return Err(EnchantModelError0104::ModeClosed);
        }
        // The packet is parsed and discarded; only Send(false) is observable.
        self.send_locked = false;
        self.intents
            .push_back(EnchantIntent0104::TransportFailure(failure));
        Ok(())
    }

    pub fn receive_auxiliary_unlock(&mut self, packet_id: u32) -> bool {
        let handled = matches!(
            packet_id,
            ENCHANT_DELETE_SUCCESS_PACKET_ID_0104
                | ENCHANT_DISASSEMBLE_SUCCESS_PACKET_ID_0104
                | ENCHANT_DISASSEMBLE_FAILURE_PACKET_ID_0104
        );
        if handled {
            // Exact Send(false): waiting/awaiting phase itself is not changed.
            self.send_locked = false;
            self.intents
                .push_back(EnchantIntent0104::AuxiliaryUnlock(packet_id));
        }
        handled
    }

    pub fn enchant_more_items(&mut self) -> Result<(), EnchantModelError0104> {
        if !matches!(self.phase, EnchantPhase0104::Success { .. }) {
            return Err(EnchantModelError0104::NotSuccessOverlay);
        }
        self.send_locked = false;
        self.phase = EnchantPhase0104::Ready;
        // Exact EnchantMoreItem: flags clear, but InventoryManager slot objects
        // are neither detached nor returned to inventory.
        for slot in EnchantAttachmentSlot0104::ALL {
            self.selection.clear_flag_only(slot);
        }
        Ok(())
    }

    pub fn go_to_my_stuff(&mut self) -> Result<(), EnchantModelError0104> {
        if !matches!(self.phase, EnchantPhase0104::Success { .. }) {
            return Err(EnchantModelError0104::NotSuccessOverlay);
        }
        self.send_locked = false;
        self.exit_common();
        self.intents.push_back(EnchantIntent0104::Lifecycle(
            EnchantLifecycleIntent0104::GoToMyStuff,
        ));
        self.intents.push_back(EnchantIntent0104::Lifecycle(
            EnchantLifecycleIntent0104::EnterGameMode(6),
        ));
        self.intents.push_back(EnchantIntent0104::Lifecycle(
            EnchantLifecycleIntent0104::DispatchLegacyEvent {
                channel: 2,
                element_func: 3,
                argument: Some(0),
                dispatch: Some(2),
            },
        ));
        self.intents.push_back(EnchantIntent0104::Lifecycle(
            EnchantLifecycleIntent0104::CheckFirstUse(3),
        ));
        self.intents.push_back(EnchantIntent0104::Lifecycle(
            EnchantLifecycleIntent0104::DispatchLegacyEvent {
                channel: 11,
                element_func: 18,
                argument: None,
                dispatch: None,
            },
        ));
        Ok(())
    }

    pub fn close(&mut self, inventory_accepts_close: bool) -> Result<(), EnchantModelError0104> {
        if !self.input_capabilities().close_enabled {
            return Err(EnchantModelError0104::InputBlocked);
        }
        if !inventory_accepts_close {
            return Err(EnchantModelError0104::CloseRejectedByInventory);
        }
        self.exit_common();
        Ok(())
    }

    pub(super) fn exit_common(&mut self) {
        self.intents.push_back(EnchantIntent0104::Lifecycle(
            EnchantLifecycleIntent0104::RestoreGameplayInventory {
                inventory_event_dispatch: 10,
            },
        ));
        self.intents.push_back(EnchantIntent0104::Lifecycle(
            EnchantLifecycleIntent0104::SetCursorLock(self.old_cursor_lock),
        ));
        self.intents.push_back(EnchantIntent0104::Lifecycle(
            EnchantLifecycleIntent0104::ExitToMainGame,
        ));
        self.intents
            .push_back(EnchantIntent0104::Audio(EnchantAudioIntent0104::StopUiMode));
        self.send_locked = false;
        for slot in EnchantAttachmentSlot0104::ALL {
            self.selection.clear_flag_only(slot);
        }
        self.phase = EnchantPhase0104::Closed;
    }

    #[must_use]
    pub fn helper_drop_accepts(slot: EnchantAttachmentSlot0104, item: ItemBase0104) -> bool {
        if item.item_type != 7 {
            return false;
        }
        match slot {
            EnchantAttachmentSlot0104::Helper1 => item.item_id != ENCHANT_HELP_ITEM_1_ID_0104,
            EnchantAttachmentSlot0104::Helper2 => item.item_id != ENCHANT_HELP_ITEM_2_ID_0104,
            _ => false,
        }
    }

    #[must_use]
    pub fn idle_drag_payload_slot(slot: EnchantAttachmentSlot0104) -> EnchantAttachmentSlot0104 {
        if slot == EnchantAttachmentSlot0104::Helper2 {
            EnchantAttachmentSlot0104::Target
        } else {
            slot
        }
    }

    #[must_use]
    pub const fn unattached_armor_preview_role() -> EnchantAttachmentSlot0104 {
        // cnGuiEnchant checks texArmorRawItem but draws texWpnRawItem.
        EnchantAttachmentSlot0104::WeaponMaterial
    }

}

// ---------------------------------------------------------------------------
// Standalone Bevy projection and renderer
// ---------------------------------------------------------------------------

pub const ENCHANT_INVENTORY_SLOT_COUNT_0104: usize = 50;

pub const ENCHANT_INVENTORY_COLUMNS_0104: usize = 5;

pub const ENCHANT_INVENTORY_SLOT_SIZE_0104: f32 = 67.0;

pub const ENCHANT_INVENTORY_SLOT_STRIDE_0104: f32 = 69.0;

pub const ENCHANT_INVENTORY_OPEN_SECONDS_0104: f32 = 1.0;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EnchantInventorySlotProjection0104 {
    pub item: Option<ItemBase0104>,
    pub icon_path: Option<String>,
    pub restricted: bool,
    pub show_combined_badge: bool,
    pub quantity_label: Option<String>,
    pub hidden_by_selection_overlay: bool,
}

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct EnchantModeProjection0104 {
    pub visible: bool,
    pub phase: EnchantPhase0104,
    pub capabilities: EnchantInputCapabilities0104,
    pub selection: EnchantSelection0104,
    pub requirements: Option<EnchantRequirements0104>,
    pub target_error: bool,
    pub material_quantity_errors: [bool; 2],
    pub taros: i32,
    pub weapon_battery: i32,
    pub nano_battery: i32,
    pub waiting_progress_width: f32,
    pub primary_npc_camera_bound: bool,
    pub waiting_npc_camera_bound: bool,
    pub support: EnchantSupportPresentation0104,
    pub inventory: [EnchantInventorySlotProjection0104; ENCHANT_INVENTORY_SLOT_COUNT_0104],
    pub equipment: [EnchantInventorySlotProjection0104; ENCHANT_EQUIPMENT_SLOT_COUNT_0104],
    pub success: Option<EnchantSuccessPresentation0104>,
}

impl Default for EnchantModeProjection0104 {
    fn default() -> Self {
        Self {
            visible: false,
            phase: EnchantPhase0104::Closed,
            capabilities: EnchantInputCapabilities0104::default(),
            selection: EnchantSelection0104::default(),
            requirements: None,
            target_error: false,
            material_quantity_errors: [false; 2],
            taros: 0,
            weapon_battery: 0,
            nano_battery: 0,
            waiting_progress_width: 0.0,
            primary_npc_camera_bound: false,
            waiting_npc_camera_bound: false,
            support: EnchantSupportPresentation0104::default(),
            inventory: array::from_fn(|_| EnchantInventorySlotProjection0104::default()),
            equipment: array::from_fn(|_| EnchantInventorySlotProjection0104::default()),
            success: None,
        }
    }
}

impl EnchantModeProjection0104 {
    #[must_use]
    pub fn from_model(
        model: &EnchantModeModel0104,
        support: EnchantSupportPresentation0104,
        inventory: [EnchantInventorySlotProjection0104; ENCHANT_INVENTORY_SLOT_COUNT_0104],
        equipment: [EnchantInventorySlotProjection0104; ENCHANT_EQUIPMENT_SLOT_COUNT_0104],
        weapon_battery: i32,
        nano_battery: i32,
    ) -> Self {
        let success = match model.phase() {
            EnchantPhase0104::Success { reply } => model
                .selection()
                .visual_item(EnchantAttachmentSlot0104::Target)
                .map(|target| EnchantSuccessPresentation0104 {
                    item: reply.enchant_item,
                    presentation: target.presentation.clone(),
                    displayed_enchant_level: (reply.enchant_item.option & 0xFFFF) - 1,
                }),
            _ => None,
        };
        Self {
            visible: model.input_capabilities().draw,
            phase: model.phase().clone(),
            capabilities: model.input_capabilities(),
            selection: model.selection().clone(),
            requirements: model.requirements().ok().flatten(),
            target_error: model.target_error(),
            material_quantity_errors: [
                model.material_quantity_error(EnchantAttachmentSlot0104::WeaponMaterial),
                model.material_quantity_error(EnchantAttachmentSlot0104::ArmorMaterial),
            ],
            taros: model.taros(),
            weapon_battery,
            nano_battery,
            waiting_progress_width: model.waiting_progress_width(),
            primary_npc_camera_bound: false,
            waiting_npc_camera_bound: false,
            support,
            inventory,
            equipment,
            success,
        }
    }

    #[must_use]
    pub const fn dead_cash_warning_visible(&self) -> bool {
        // Exact duplicated condition in cnGuiEnchant: `else if (error == 0)`
        // follows an `if (error == 0)`, so the warning branch cannot execute.
        false
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Resource)]
pub struct EnchantInventoryUiState0104 {
    pub(super) opening_elapsed_seconds: f32,
    pub(super) scroll_y: f32,
    pub(super) was_visible: bool,
}

impl Default for EnchantInventoryUiState0104 {
    fn default() -> Self {
        Self {
            opening_elapsed_seconds: 0.0,
            scroll_y: 0.0,
            was_visible: false,
        }
    }
}

impl EnchantInventoryUiState0104 {
    #[must_use]
    pub const fn opening_elapsed_seconds(&self) -> f32 {
        self.opening_elapsed_seconds
    }

    #[must_use]
    pub const fn scroll_y(&self) -> f32 {
        self.scroll_y
    }

    #[must_use]
    pub fn panel_controls_enabled(&self) -> bool {
        self.opening_elapsed_seconds >= ENCHANT_INVENTORY_OPEN_SECONDS_0104
    }

    pub fn tick(&mut self, delta_seconds: f32, visible: bool) {
        if visible && !self.was_visible {
            self.opening_elapsed_seconds = 0.0;
            self.scroll_y = 0.0;
        }
        if visible && delta_seconds.is_finite() && delta_seconds > 0.0 {
            self.opening_elapsed_seconds = (self.opening_elapsed_seconds + delta_seconds)
                .min(ENCHANT_INVENTORY_OPEN_SECONDS_0104);
        } else if visible && delta_seconds.is_infinite() && delta_seconds.is_sign_positive() {
            self.opening_elapsed_seconds = ENCHANT_INVENTORY_OPEN_SECONDS_0104;
        } else if !visible {
            self.opening_elapsed_seconds = 0.0;
            self.scroll_y = 0.0;
        }
        self.was_visible = visible;
    }

    pub fn set_scroll_y(&mut self, scroll_y: f32) {
        self.scroll_y = clamp_enchant_inventory_scroll_0104(scroll_y);
    }

    /// Mirrors the shared `Panel_PCStuff.Scroll` calculation for a host that
    /// supplies a reachable pointer-scroll update. Clean `cnEnchantMode`
    /// itself never assigns or invokes its declared configurable-axis delegate.
    pub fn apply_legacy_scroll_axis(&mut self, axis: f32) {
        let axis = if axis.is_finite() {
            axis.clamp(-1.0, 1.0)
        } else {
            0.0
        };
        self.set_scroll_y(self.scroll_y - axis * ENCHANT_INVENTORY_SCROLL_VELOCITY_0104);
    }
}

pub(super) fn advance_enchant_inventory_ui_0104(
    time: Option<Res<Time>>,
    projection: Res<EnchantModeProjection0104>,
    mut inventory_ui: ResMut<EnchantInventoryUiState0104>,
) {
    let delta_seconds = time.as_deref().map_or(0.0, |time| time.delta_secs());
    inventory_ui.tick(delta_seconds, projection.visible);
}
