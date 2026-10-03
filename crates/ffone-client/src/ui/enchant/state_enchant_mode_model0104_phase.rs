use super::*;

pub const ENCHANT_MODE_MANAGED_SOURCE_SHA256: &str =
    "4BF6C33E03E55D0F9FB17BAE9D835529EDD4B98BA0346621736B3E3A195265E7";

pub const ENCHANT_INVENTORY_MANAGER_MANAGED_SOURCE_SHA256: &str =
    "55096F59C1C1E041EF97E38172EDCE48F0439183BB280AB782BEEBB504905C7B";

pub const ENCHANT_GAME_MODE_0104: i32 = 29;

pub const ENCHANT_INVENTORY_GUI_MODE_0104: i32 = 9;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnchantLegacyStateKind0104 {
    UnreachableCashWarning,
    ArmorPreviewUsesWeaponTexture,
    HelperTwoPreviewChecksHelperOneTexture,
    HelperTwoIdleDragCarriesTarget,
    IntendedHelperIdsRejected,
    OtherGeneralItemsAcceptedAsHelpers,
    FailureMutatesMinusOneMaterialSlots,
    UnsupportedSuccessFlagLeavesSendLock,
    EnchantMoreLeavesSlotObjects,
    MaterialLabelsUseEquipmentTable,
    DeclaredScrollDelegateNeverAssigned,
    KoreanHammerControlHiddenByCleanConfiguration,
    SuccessEquipCheckBranchesUseSameColors,
}

#[must_use]
pub fn enchant_inventory_opening_eased_fraction_0104(elapsed_seconds: f32) -> f32 {
    if elapsed_seconds.is_nan() || elapsed_seconds <= 0.0 {
        return 0.0;
    }
    if !elapsed_seconds.is_finite() || elapsed_seconds >= ENCHANT_INVENTORY_OPEN_SECONDS_0104 {
        return 1.0;
    }
    (elapsed_seconds / ENCHANT_INVENTORY_OPEN_SECONDS_0104 * std::f32::consts::FRAC_PI_2).sin()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnchantInventoryMutation0104 {
    pub slot: i32,
    pub item: ItemBase0104,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnchantSelectionIntent0104 {
    Reserve {
        slot: EnchantAttachmentSlot0104,
        source_slot: i32,
        quantity: i32,
    },
    Release {
        slot: EnchantAttachmentSlot0104,
        source_slot: i32,
        quantity: i32,
    },
}

#[derive(Clone, Debug)]
pub struct EnchantModeModel0104 {
    pub(super) phase: EnchantPhase0104,
    pub(super) selection: EnchantSelection0104,
    pub(super) cached_requirements: Option<EnchantRequirements0104>,
    pub(super) taros: i32,
    pub(super) send_locked: bool,
    pub(super) old_cursor_lock: bool,
    pub(super) external_gates: EnchantExternalGates0104,
    pub(super) next_request_token: u64,
    pub(super) last_request: Option<EnchantRequest0104>,
    pub(super) intents: VecDeque<EnchantIntent0104>,
}

impl Default for EnchantModeModel0104 {
    fn default() -> Self {
        Self {
            phase: EnchantPhase0104::Closed,
            selection: EnchantSelection0104::default(),
            cached_requirements: None,
            taros: 0,
            send_locked: false,
            old_cursor_lock: false,
            external_gates: EnchantExternalGates0104::default(),
            next_request_token: 1,
            last_request: None,
            intents: VecDeque::new(),
        }
    }
}

impl EnchantModeModel0104 {
    #[must_use]
    pub fn phase(&self) -> &EnchantPhase0104 {
        &self.phase
    }

    #[must_use]
    pub const fn selection(&self) -> &EnchantSelection0104 {
        &self.selection
    }

    #[must_use]
    pub const fn taros(&self) -> i32 {
        self.taros
    }

    #[must_use]
    pub const fn send_locked(&self) -> bool {
        self.send_locked
    }

    #[must_use]
    pub const fn external_gates(&self) -> EnchantExternalGates0104 {
        self.external_gates
    }

    pub fn set_external_gates(&mut self, gates: EnchantExternalGates0104) {
        self.external_gates = gates;
    }

    pub fn open_redeem_code(&mut self) -> Result<(), EnchantModelError0104> {
        self.require_base_input()?;
        self.external_gates.inventory_popup_open = true;
        self.intents
            .push_back(EnchantIntent0104::Audio(EnchantAudioIntent0104::Button));
        self.intents.push_back(EnchantIntent0104::Popup(
            EnchantPopupIntent0104::RedeemCode {
                window_id: ENCHANT_REDEEM_WINDOW_ID_0104,
                max_code_chars: ENCHANT_REDEEM_CODE_MAX_CHARS_0104,
            },
        ));
        Ok(())
    }

    pub fn cancel_redeem_code(&mut self) -> Result<(), EnchantModelError0104> {
        if !self.external_gates.inventory_popup_open {
            return Err(EnchantModelError0104::RedeemNotOpen);
        }
        self.external_gates.inventory_popup_open = false;
        self.intents
            .push_back(EnchantIntent0104::Audio(EnchantAudioIntent0104::NoButton));
        Ok(())
    }

    pub fn submit_redeem_code(
        &mut self,
        code: &str,
    ) -> Result<Option<EnchantRedeemWire0104>, EnchantModelError0104> {
        if !self.external_gates.inventory_popup_open {
            return Err(EnchantModelError0104::RedeemNotOpen);
        }
        let utf16_len = code.encode_utf16().count();
        if utf16_len < ENCHANT_REDEEM_CODE_MIN_CHARS_0104 {
            // The clean modal remains open but still plays Yes_Button.
            self.intents
                .push_back(EnchantIntent0104::Audio(EnchantAudioIntent0104::YesButton));
            return Ok(None);
        }
        if code.contains(' ') {
            self.external_gates.inventory_popup_open = false;
            self.external_gates.system_popup_open = true;
            self.intents.push_back(EnchantIntent0104::Popup(
                EnchantPopupIntent0104::RedeemCodeSpaceError {
                    message: ENCHANT_REDEEM_SPACE_ERROR_TEXT_0104,
                },
            ));
            self.intents
                .push_back(EnchantIntent0104::Audio(EnchantAudioIntent0104::YesButton));
            return Err(EnchantModelError0104::Redeem(
                EnchantRedeemError0104::ContainsSpace,
            ));
        }
        let wire = enchant_redeem_wire_0104(code).map_err(EnchantModelError0104::Redeem)?;
        self.external_gates.inventory_popup_open = false;
        self.intents
            .push_back(EnchantIntent0104::RedeemWire(wire.clone()));
        self.intents.push_back(EnchantIntent0104::Audio(
            EnchantAudioIntent0104::ActionSuccess,
        ));
        self.intents
            .push_back(EnchantIntent0104::Audio(EnchantAudioIntent0104::YesButton));
        Ok(Some(wire))
    }

    #[must_use]
    pub fn intents(&self) -> impl Iterator<Item = &EnchantIntent0104> {
        self.intents.iter()
    }

    pub fn pop_intent(&mut self) -> Option<EnchantIntent0104> {
        self.intents.pop_front()
    }

    pub fn clear_intents(&mut self) {
        self.intents.clear();
    }

    pub fn open(&mut self, taros: i32, old_cursor_lock: bool) {
        self.phase = EnchantPhase0104::Ready;
        self.selection.reset();
        self.cached_requirements = None;
        self.taros = taros;
        self.send_locked = false;
        self.old_cursor_lock = old_cursor_lock;
        self.external_gates = EnchantExternalGates0104::default();
        self.last_request = None;
        self.intents
            .push_back(EnchantIntent0104::Audio(EnchantAudioIntent0104::PlayUiMode));
        self.intents.push_back(EnchantIntent0104::Lifecycle(
            EnchantLifecycleIntent0104::InitializeInventoryMode {
                gui_mode: ENCHANT_INVENTORY_GUI_MODE_0104,
                active_inventory_tab: 0,
                show_inventory_panel: true,
                show_equipment_panel: true,
                inventory_event_dispatch: 8,
            },
        ));
        self.intents.push_back(EnchantIntent0104::Lifecycle(
            EnchantLifecycleIntent0104::SetCursorLock(false),
        ));
        self.intents.push_back(EnchantIntent0104::Lifecycle(
            EnchantLifecycleIntent0104::LoadPanelBackdrop {
                logical_path: "TutorialAssets/panelback.png",
            },
        ));
        self.intents.push_back(EnchantIntent0104::Camera(
            EnchantCameraIntent0104::BindPrimaryNpcAvatar {
                camera_path_id: ENCHANT_PRIMARY_CAMERA_PATH_ID,
                controller_path_id: ENCHANT_PRIMARY_CAMERA_CONTROLLER_PATH_ID,
                distance_millimetres: 1_300,
                height_millimetres: 550,
                euler_degrees: [0, -20, 0],
                target: EnchantCameraTarget0104::Neck,
            },
        ));
        self.intents.push_back(EnchantIntent0104::Camera(
            EnchantCameraIntent0104::BindWaitingNpcAvatar {
                camera_path_id: ENCHANT_WAITING_CAMERA_PATH_ID,
                controller_path_id: ENCHANT_WAITING_CAMERA_CONTROLLER_PATH_ID,
                distance_millimetres: 1_300,
                height_millimetres: 550,
                euler_degrees: [0, -20, 0],
                target: EnchantCameraTarget0104::Neck,
            },
        ));
    }

    #[must_use]
    pub fn requirements(&self) -> Result<Option<EnchantRequirements0104>, EnchantModelError0104> {
        if !self
            .selection
            .is_attached(EnchantAttachmentSlot0104::Target)
        {
            return Ok(None);
        }
        self.selection
            .visual_item(EnchantAttachmentSlot0104::Target)
            .ok_or(EnchantModelError0104::TargetRequired)?;
        self.cached_requirements
            .map(Some)
            .ok_or(EnchantModelError0104::TargetRequired)
    }

    #[must_use]
    pub fn target_error(&self) -> bool {
        self.selection
            .is_attached(EnchantAttachmentSlot0104::Target)
            && self
                .selection
                .visual_item(EnchantAttachmentSlot0104::Target)
                .is_some_and(|item| item.presentation.cashable > 0)
    }

    #[must_use]
    pub fn material_quantity_error(&self, slot: EnchantAttachmentSlot0104) -> bool {
        let Ok(Some(requirements)) = self.requirements() else {
            return false;
        };
        let required = match slot {
            EnchantAttachmentSlot0104::WeaponMaterial => requirements.weapon_material_count,
            EnchantAttachmentSlot0104::ArmorMaterial => requirements.armor_material_count,
            _ => return false,
        };
        self.selection.is_attached(slot)
            && self
                .selection
                .visual_item(slot)
                .is_some_and(|item| item.item.option < required)
    }

    #[must_use]
    pub fn input_capabilities(&self) -> EnchantInputCapabilities0104 {
        let draw = !matches!(self.phase, EnchantPhase0104::Closed);
        let overlay = matches!(
            self.phase,
            EnchantPhase0104::SystemMessage { .. }
                | EnchantPhase0104::Waiting { .. }
                | EnchantPhase0104::Success { .. }
        );
        let base_gui_enabled = draw && !self.send_locked && !self.external_gates.any() && !overlay;
        let target_attached = self
            .selection
            .is_attached(EnchantAttachmentSlot0104::Target);
        let target_valid = target_attached && !self.target_error();
        let requirements = self.requirements().ok().flatten();
        let material_ready = requirements.is_some_and(|requirements| {
            let weapon_ready = requirements.weapon_material_id == 0
                || (self
                    .selection
                    .is_attached(EnchantAttachmentSlot0104::WeaponMaterial)
                    && !self.material_quantity_error(EnchantAttachmentSlot0104::WeaponMaterial));
            let armor_ready = requirements.armor_material_id == 0
                || (self
                    .selection
                    .is_attached(EnchantAttachmentSlot0104::ArmorMaterial)
                    && !self.material_quantity_error(EnchantAttachmentSlot0104::ArmorMaterial));
            weapon_ready && armor_ready
        });
        EnchantInputCapabilities0104 {
            draw,
            base_gui_enabled,
            preview_enabled: base_gui_enabled && target_valid,
            clear_enabled: base_gui_enabled && self.selection.any_attached(),
            enchant_enabled: base_gui_enabled && target_valid && material_ready,
            close_enabled: base_gui_enabled,
            // DoEnchantSucc explicitly forces GUI.enabled=true, even if Send or
            // an unrelated modal gate is active.
            success_buttons_enabled: matches!(self.phase, EnchantPhase0104::Success { .. }),
        }
    }

    pub(super) fn require_base_input(&self) -> Result<(), EnchantModelError0104> {
        if matches!(self.phase, EnchantPhase0104::Closed) {
            Err(EnchantModelError0104::ModeClosed)
        } else if matches!(
            self.phase,
            EnchantPhase0104::SystemMessage { .. }
                | EnchantPhase0104::Waiting { .. }
                | EnchantPhase0104::Success { .. }
        ) {
            Err(EnchantModelError0104::OverlayOwnsInput)
        } else if self.send_locked || self.external_gates.any() {
            Err(EnchantModelError0104::InputBlocked)
        } else {
            Ok(())
        }
    }

    pub fn attach(
        &mut self,
        slot: EnchantAttachmentSlot0104,
        mut item: EnchantSelectableItem0104,
    ) -> Result<(), EnchantModelError0104> {
        self.require_base_input()?;
        match slot {
            EnchantAttachmentSlot0104::Target => {
                if item.target_kind().is_none() {
                    return Err(EnchantModelError0104::UnsupportedTargetType(
                        item.item.item_type,
                    ));
                }
                let requirements =
                    enchant_requirements_0104(item.item).map_err(|error| match error {
                        EnchantProjectionError0104::UnsupportedTargetItemType(item_type) => {
                            EnchantModelError0104::UnsupportedTargetType(item_type)
                        }
                        EnchantProjectionError0104::MissingRecipe { recipe_index } => {
                            EnchantModelError0104::MissingRecipe(recipe_index)
                        }
                    })?;
                // InventoryManager detaches the existing target first.  Its
                // callback clears only secondary slots whose flags are true.
                if self.selection.visual_item(slot).is_some() {
                    self.detach_visual(slot);
                    self.clear_flagged_secondaries();
                } else {
                    self.clear_flagged_secondaries();
                }
                self.reserve(slot, &item);
                self.selection.set(slot, item);
                // `ReceiveSetItemAttached` caches the recipe-derived fields.
                // `Preview` later increments the attached object's `iOpt`
                // without running that callback again.
                self.cached_requirements = Some(requirements);
            }
            EnchantAttachmentSlot0104::WeaponMaterial
            | EnchantAttachmentSlot0104::ArmorMaterial => {
                let requirements = self
                    .requirements()?
                    .ok_or(EnchantModelError0104::TargetRequired)?;
                if item.item.item_type != 7 {
                    return Err(EnchantModelError0104::GeneralItemRequired(
                        item.item.item_type,
                    ));
                }
                let (expected, needed) = if slot == EnchantAttachmentSlot0104::WeaponMaterial {
                    (
                        requirements.weapon_material_id,
                        requirements.weapon_material_count,
                    )
                } else {
                    (
                        requirements.armor_material_id,
                        requirements.armor_material_count,
                    )
                };
                if expected == 0 {
                    return Err(EnchantModelError0104::MaterialNotRequired(slot));
                }
                if item.item.item_id != expected {
                    return Err(EnchantModelError0104::WrongMaterial {
                        expected,
                        actual: item.item.item_id,
                    });
                }
                if self.selection.visual_item(slot).is_some() {
                    self.detach_visual(slot);
                }
                // Exact stack split: reserve the requested count when the
                // source is larger, otherwise move the whole insufficient stack.
                item.item.option = item.item.option.min(needed);
                self.reserve(slot, &item);
                self.selection.set(slot, item);
            }
            EnchantAttachmentSlot0104::Helper1 | EnchantAttachmentSlot0104::Helper2 => {
                self.requirements()?
                    .ok_or(EnchantModelError0104::TargetRequired)?;
                if item.item.item_type != 7 {
                    return Err(EnchantModelError0104::GeneralItemRequired(
                        item.item.item_type,
                    ));
                }
                let rejected = if slot == EnchantAttachmentSlot0104::Helper1 {
                    ENCHANT_HELP_ITEM_1_ID_0104
                } else {
                    ENCHANT_HELP_ITEM_2_ID_0104
                };
                // Clean InventoryManagerScript accidentally returns for the
                // intended helper ID, and accepts every other General item.
                if item.item.item_id == rejected {
                    return Err(EnchantModelError0104::IntendedHelperRejected {
                        slot,
                        item_id: rejected,
                    });
                }
                if self.selection.visual_item(slot).is_some() {
                    self.detach_visual(slot);
                }
                self.reserve(slot, &item);
                self.selection.set(slot, item);
            }
        }
        Ok(())
    }

    pub(super) fn reserve(&mut self, slot: EnchantAttachmentSlot0104, item: &EnchantSelectableItem0104) {
        self.intents.push_back(EnchantIntent0104::Selection(
            EnchantSelectionIntent0104::Reserve {
                slot,
                source_slot: item.source_slot,
                quantity: if item.item.item_type == 7 {
                    item.item.option
                } else {
                    1
                },
            },
        ));
    }

    pub(super) fn detach_visual(&mut self, slot: EnchantAttachmentSlot0104) {
        if let Some(item) = self.selection.take_visual(slot) {
            self.intents.push_back(EnchantIntent0104::Selection(
                EnchantSelectionIntent0104::Release {
                    slot,
                    source_slot: item.source_slot,
                    quantity: if item.item.item_type == 7 {
                        item.item.option
                    } else {
                        1
                    },
                },
            ));
        }
        if slot == EnchantAttachmentSlot0104::Target {
            self.cached_requirements = None;
        }
    }

    pub(super) fn clear_flagged_secondaries(&mut self) {
        for slot in [
            EnchantAttachmentSlot0104::WeaponMaterial,
            EnchantAttachmentSlot0104::ArmorMaterial,
            EnchantAttachmentSlot0104::Helper1,
            EnchantAttachmentSlot0104::Helper2,
        ] {
            if self.selection.is_attached(slot) {
                self.detach_visual(slot);
            }
        }
    }

    pub fn detach(&mut self, slot: EnchantAttachmentSlot0104) -> Result<(), EnchantModelError0104> {
        self.require_base_input()?;
        if slot == EnchantAttachmentSlot0104::Target {
            self.detach_visual(slot);
            self.clear_flagged_secondaries();
        } else {
            self.detach_visual(slot);
        }
        Ok(())
    }

    pub fn clear_all(&mut self) -> Result<(), EnchantModelError0104> {
        self.require_base_input()?;
        if !self.selection.any_attached() {
            return Err(EnchantModelError0104::NothingAttached);
        }
        for slot in EnchantAttachmentSlot0104::ALL {
            if self.selection.is_attached(slot) {
                self.detach_visual(slot);
            }
        }
        Ok(())
    }

    pub fn preview(&mut self) -> Result<EnchantSelectableItem0104, EnchantModelError0104> {
        if !self.input_capabilities().preview_enabled {
            return Err(if self.target_error() {
                EnchantModelError0104::TargetCannotBeEnchanted
            } else {
                EnchantModelError0104::InputBlocked
            });
        }
        let target = self
            .selection
            .visual_item_mut(EnchantAttachmentSlot0104::Target)
            .ok_or(EnchantModelError0104::TargetRequired)?;
        // Exact cnEnchantMode.Preview side effect on the attached object.
        target.item.option = target.item.option.wrapping_add(1);
        let popup_item = target.clone();
        self.intents
            .push_back(EnchantIntent0104::Popup(EnchantPopupIntent0104::Preview {
                gui_mode: ENCHANT_INVENTORY_GUI_MODE_0104,
                source_slot_type: ENCHANT_POPUP_SOURCE_SLOT_0104,
                action: ENCHANT_POPUP_ACTION_0104,
                popup_rect: (-1, -1, 0, 0),
                item: popup_item.clone(),
            }));
        Ok(popup_item)
    }

    pub fn activate_enchant(&mut self) -> Result<(), EnchantModelError0104> {
        if !self.input_capabilities().enchant_enabled {
            if self.target_error() {
                return Err(EnchantModelError0104::TargetCannotBeEnchanted);
            }
            for slot in [
                EnchantAttachmentSlot0104::WeaponMaterial,
                EnchantAttachmentSlot0104::ArmorMaterial,
            ] {
                if self.material_quantity_error(slot) {
                    return Err(EnchantModelError0104::MaterialQuantityInsufficient(slot));
                }
            }
            return Err(EnchantModelError0104::InputBlocked);
        }
        let requirements = self
            .requirements()?
            .ok_or(EnchantModelError0104::TargetRequired)?;
        if self.taros >= requirements.cost {
            self.show_system_message(
                ENCHANT_MESSAGE_CONFIRM_0104,
                EnchantSystemCallback0104::EnchantConfirmed,
                None,
            );
            Ok(())
        } else {
            self.show_system_message(
                ENCHANT_MESSAGE_NOT_ENOUGH_TAROS_0104,
                EnchantSystemCallback0104::None,
                None,
            );
            Err(EnchantModelError0104::NotEnoughTaros)
        }
    }

    pub(super) fn show_system_message(
        &mut self,
        message_id: i32,
        callback: EnchantSystemCallback0104,
        argument: Option<String>,
    ) {
        self.phase = EnchantPhase0104::SystemMessage {
            message_id,
            argument: argument.clone(),
            callback,
        };
        self.intents.push_back(EnchantIntent0104::Popup(
            EnchantPopupIntent0104::SystemMessage {
                message_id,
                callback,
                argument,
            },
        ));
    }

    pub fn accept_system_message(&mut self) -> Result<(), EnchantModelError0104> {
        let EnchantPhase0104::SystemMessage { callback, .. } = self.phase.clone() else {
            return Err(EnchantModelError0104::WrongSystemMessage);
        };
        match callback {
            EnchantSystemCallback0104::None => {
                self.phase = EnchantPhase0104::Ready;
            }
            EnchantSystemCallback0104::EnchantConfirmed => {
                let token = self.next_request_token;
                self.next_request_token = self.next_request_token.wrapping_add(1).max(1);
                self.send_locked = true;
                self.phase = EnchantPhase0104::Waiting {
                    elapsed_seconds: 0.0,
                    request_token: token,
                };
                self.intents.push_back(EnchantIntent0104::Animation(
                    EnchantAnimationIntent0104::SetEventInMakeOut,
                ));
            }
            EnchantSystemCallback0104::EnchantFailed => {
                // EnchantFailed -> Send(false), ClearAll(0), then flags false.
                self.send_locked = false;
                for slot in EnchantAttachmentSlot0104::ALL {
                    if self.selection.is_attached(slot) {
                        self.detach_visual(slot);
                    }
                    self.selection.clear_flag_only(slot);
                }
                self.phase = EnchantPhase0104::Ready;
            }
        }
        Ok(())
    }

    pub fn dismiss_system_message(&mut self) -> Result<(), EnchantModelError0104> {
        let EnchantPhase0104::SystemMessage { callback, .. } = self.phase else {
            return Err(EnchantModelError0104::WrongSystemMessage);
        };
        if callback == EnchantSystemCallback0104::EnchantFailed {
            self.accept_system_message()
        } else {
            self.phase = EnchantPhase0104::Ready;
            Ok(())
        }
    }

    pub fn advance(&mut self, delta_seconds: f32) -> Result<(), EnchantModelError0104> {
        let EnchantPhase0104::Waiting {
            elapsed_seconds,
            request_token,
        } = &mut self.phase
        else {
            return Ok(());
        };
        if delta_seconds.is_finite() && delta_seconds > 0.0 {
            *elapsed_seconds += delta_seconds;
        }
        if *elapsed_seconds <= ENCHANT_WAIT_SECONDS_0104 {
            return Ok(());
        }
        let token = *request_token;
        let request = self.build_request();
        self.phase = EnchantPhase0104::AwaitingReply {
            request_token: token,
        };
        self.send_locked = true;
        let outcome = match request {
            Ok(request) => {
                self.last_request = Some(request);
                self.intents.push_back(EnchantIntent0104::Wire {
                    request_token: token,
                    packet_id: ENCHANT_REQUEST_PACKET_ID_0104,
                    payload: request.encode(),
                    request,
                });
                Ok(())
            }
            Err(error) => Err(error),
        };
        // `Enchant()` invokes StandMotion even when no request was emitted.
        self.intents.push_back(EnchantIntent0104::Animation(
            EnchantAnimationIntent0104::StandMotion,
        ));
        outcome
    }

    #[must_use]
    pub fn waiting_progress_width(&self) -> f32 {
        match self.phase {
            EnchantPhase0104::Waiting {
                elapsed_seconds, ..
            } => 414.0 * (elapsed_seconds / ENCHANT_WAIT_SECONDS_0104).clamp(0.0, 1.0),
            _ => 0.0,
        }
    }

    pub(super) fn build_request(&self) -> Result<EnchantRequest0104, EnchantModelError0104> {
        let target = self
            .selection
            .visual_item(EnchantAttachmentSlot0104::Target)
            .ok_or(EnchantModelError0104::TargetRequired)?;
        let weapon = self
            .selection
            .visual_item(EnchantAttachmentSlot0104::WeaponMaterial);
        let armor = self
            .selection
            .visual_item(EnchantAttachmentSlot0104::ArmorMaterial);
        if weapon.is_none() && armor.is_none() {
            return Err(EnchantModelError0104::InputBlocked);
        }
        Ok(EnchantRequest0104 {
            enchant_item_slot: target.source_slot,
            weapon_material_item_slot: weapon.map_or(-1, |item| item.source_slot),
            defence_material_item_slot: armor.map_or(-1, |item| item.source_slot),
            cash_item_slot_1: self
                .selection
                .visual_item(EnchantAttachmentSlot0104::Helper1)
                .map_or(-1, |item| item.source_slot),
            cash_item_slot_2: self
                .selection
                .visual_item(EnchantAttachmentSlot0104::Helper2)
                .map_or(-1, |item| item.source_slot),
        })
    }

    pub(super) fn refresh_selected_item(
        &mut self,
        attachment: EnchantAttachmentSlot0104,
        source_slot: i32,
        item: ItemBase0104,
    ) {
        if source_slot == -1 {
            return;
        }
        if let Some(selected) = self.selection.visual_item_mut(attachment)
            && selected.source_slot == source_slot
        {
            selected.item = item;
        }
    }
}
