use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VendorInventorySlotProjection0104 {
    pub slot_index: usize,
    pub column: usize,
    pub row: usize,
    pub item: VendorProjectedItem0104,
}

pub(super) fn projected_inventory_item(
    item: ItemBase0104,
    catalog: &impl VendorItemCatalog0104,
) -> VendorProjectedItem0104 {
    let empty = InventoryRuntime0104::item_is_empty(item);
    let metadata = (!empty).then(|| catalog.resolve(item)).flatten();
    let ids = UserEquipItemIds::from_item(item);
    VendorProjectedItem0104 {
        item,
        empty,
        icon: projected_icon(item, metadata.as_ref()),
        show_combined_badge: !empty
            && (0..4).contains(&item.item_type)
            && ids.combined_look_id.is_some(),
        count_label: (!empty && item.item_type == 7).then(|| item.option.to_string()),
        quest_item_id: (!empty && item.item_type == 8).then_some(item.item_id),
        metadata,
    }
}

/// Complete immutable authority needed while VendorMode is visible.
#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct VendorModeProjection0104 {
    pub owner_pc_id: i32,
    pub session: VendorSession0104,
    pub taros: i32,
    pub weapon_battery: i32,
    pub nano_battery: i32,
    pub npc_name: String,
    pub npc_service: String,
    pub catalog_rows: Vec<VendorCatalogRowProjection0104>,
    pub recent_buy_rows: Vec<VendorRecentBuyRowProjection0104>,
    pub inventory: [VendorInventorySlotProjection0104; INVENTORY_SLOT_COUNT_0104],
    pub equipment: [VendorEquipmentSlotProjection0104; EQUIPMENT_SLOT_COUNT_0104],
    pub(super) inventory_items: [ItemBase0104; INVENTORY_SLOT_COUNT_0104],
}

impl Default for VendorModeProjection0104 {
    fn default() -> Self {
        let empty = ItemBase0104 {
            item_type: 0,
            item_id: 0,
            option: 0,
            time_limit: 0,
        };
        let inventory = array::from_fn(|slot_index| VendorInventorySlotProjection0104 {
            slot_index,
            column: slot_index % 5,
            row: slot_index / 5,
            item: VendorProjectedItem0104 {
                item: empty,
                empty: true,
                icon: VendorPresentationIcon0104::Empty,
                show_combined_badge: false,
                count_label: None,
                quest_item_id: None,
                metadata: None,
            },
        });
        let equipment = array::from_fn(|visual_index| {
            let spec = USER_EQUIP_EQUIPMENT_STRIP_ORDER[visual_index];
            VendorEquipmentSlotProjection0104 {
                visual_index,
                wire_slot_index: spec.wire_slot_index,
                item: VendorProjectedItem0104 {
                    item: empty,
                    empty: true,
                    icon: VendorPresentationIcon0104::Empty,
                    show_combined_badge: false,
                    count_label: None,
                    quest_item_id: None,
                    metadata: None,
                },
            }
        });
        Self {
            owner_pc_id: 0,
            session: VendorSession0104::default(),
            taros: 0,
            weapon_battery: 0,
            nano_battery: 0,
            npc_name: String::new(),
            npc_service: String::new(),
            catalog_rows: Vec::new(),
            recent_buy_rows: Vec::new(),
            inventory,
            equipment,
            inventory_items: [empty; INVENTORY_SLOT_COUNT_0104],
        }
    }
}

impl VendorModeProjection0104 {
    #[allow(clippy::too_many_arguments)]
    pub fn from_authoritative(
        owner_pc_id: i32,
        session: VendorSession0104,
        taros: i32,
        weapon_battery: i32,
        nano_battery: i32,
        npc_name: impl Into<String>,
        npc_service: impl Into<String>,
        vendor_entries: &[VendorCatalogEntry0104],
        recent_buy_entries: &[VendorRecentBuyEntry0104],
        inventory: &InventoryRuntime0104,
        catalog: &impl VendorItemCatalog0104,
        eligibility: &impl VendorEquipEligibility0104,
    ) -> Result<Self, VendorProjectionError0104> {
        if inventory.owner_pc_id() != owner_pc_id {
            return Err(VendorProjectionError0104::OwnerMismatch {
                expected_pc_id: owner_pc_id,
                inventory_pc_id: inventory.owner_pc_id(),
            });
        }

        let catalog_rows = vendor_entries
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, entry)| {
                entry.source_slot_id < VENDOR_CATALOG_CAPACITY
                    && !InventoryRuntime0104::item_is_empty(entry.item)
            })
            .take(VENDOR_CATALOG_CAPACITY)
            .map(|(packet_index, entry)| {
                let metadata = catalog.resolve(entry.item);
                let price = metadata.as_ref().map(|value| value.buy_price);
                let vehicle_speed_class = (entry.item.item_type == 10)
                    .then(|| catalog.vehicle_speed_class(entry.item))
                    .flatten();
                VendorCatalogRowProjection0104 {
                    packet_index,
                    source_slot_id: entry.source_slot_id,
                    item: entry.item,
                    icon: projected_icon(entry.item, metadata.as_ref()),
                    affordable: price.map(|price| price <= taros),
                    price,
                    equip_validation: equip_validation(entry.item, eligibility),
                    vehicle_speed_class,
                    metadata,
                }
            })
            .collect();

        let recent_buy_rows = recent_buy_entries
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, entry)| {
                entry.source_slot_id < VENDOR_RECENT_BUY_CAPACITY
                    && !InventoryRuntime0104::item_is_empty(entry.item)
            })
            .take(VENDOR_RECENT_BUY_CAPACITY)
            .map(|(fifo_index, entry)| {
                let metadata = catalog.resolve(entry.item);
                let price = metadata
                    .as_ref()
                    .map(|metadata| recent_price(entry.item, metadata));
                let vehicle_speed_class = (entry.item.item_type == 10)
                    .then(|| catalog.vehicle_speed_class(entry.item))
                    .flatten();
                VendorRecentBuyRowProjection0104 {
                    fifo_index,
                    source_slot_id: entry.source_slot_id,
                    restore_list_id: entry.source_slot_id as i32 + 1,
                    item: entry.item,
                    icon: projected_icon(entry.item, metadata.as_ref()),
                    affordable: price.map(|price| price <= taros),
                    price,
                    vehicle_speed_class,
                    metadata,
                }
            })
            .collect();

        let inventory_projection = array::from_fn(|slot_index| VendorInventorySlotProjection0104 {
            slot_index,
            column: slot_index % 5,
            row: slot_index / 5,
            item: projected_inventory_item(inventory.inventory()[slot_index], catalog),
        });
        let equipment = array::from_fn(|visual_index| {
            let spec = USER_EQUIP_EQUIPMENT_STRIP_ORDER[visual_index];
            VendorEquipmentSlotProjection0104 {
                visual_index,
                wire_slot_index: spec.wire_slot_index,
                item: projected_inventory_item(
                    inventory.equipment()[spec.wire_slot_index],
                    catalog,
                ),
            }
        });

        Ok(Self {
            owner_pc_id,
            session,
            taros,
            weapon_battery,
            nano_battery,
            npc_name: npc_name.into(),
            npc_service: npc_service.into(),
            catalog_rows,
            recent_buy_rows,
            inventory: inventory_projection,
            equipment,
            inventory_items: *inventory.inventory(),
        })
    }

    #[must_use]
    pub const fn inventory_items(&self) -> &[ItemBase0104; INVENTORY_SLOT_COUNT_0104] {
        &self.inventory_items
    }

    #[must_use]
    pub fn first_empty_inventory_slot(&self) -> Option<usize> {
        self.inventory_items
            .iter()
            .copied()
            .position(InventoryRuntime0104::item_is_empty)
    }

    #[must_use]
    pub fn rows_for_tab(&self, tab: VendorTab0104) -> usize {
        match tab {
            VendorTab0104::Buy => self.catalog_rows.len(),
            VendorTab0104::Buyback => self.recent_buy_rows.len(),
        }
    }

    pub(super) fn catalog_quantity_contract(
        &self,
        row_index: usize,
    ) -> Result<VendorQuantityContract0104, VendorSilentBlock0104> {
        let row = self
            .catalog_rows
            .get(row_index)
            .ok_or(VendorSilentBlock0104::InvalidCatalogRow { row_index })?;
        if row.item.item_type != 7 {
            return Ok(VendorQuantityContract0104::Fixed(0));
        }
        let metadata =
            row.metadata
                .as_ref()
                .ok_or(VendorSilentBlock0104::MissingCatalogMetadata {
                    item_type: row.item.item_type,
                    item_id: row.item.item_id,
                })?;
        let mut maximum = metadata
            .stack_size
            .ok_or(VendorSilentBlock0104::MissingStackMetadata {
                item_id: row.item.item_id,
            })?
            .max(0);
        if metadata.general_item_type == Some(2) {
            maximum = match row.item.item_id {
                3 => maximum.saturating_sub(self.weapon_battery.max(0)),
                4 => maximum.saturating_sub(self.nano_battery.max(0)),
                _ => maximum,
            };
        }
        maximum = if metadata.buy_price > 0 {
            maximum.min(self.taros.max(0) / metadata.buy_price)
        } else {
            0
        };
        Ok(VendorQuantityContract0104::Calculator { maximum })
    }

    /// Primary `GUI.Button` path: open the clean item popup, do not transact.
    #[must_use]
    pub fn primary_row_activation(
        &self,
        tab: VendorTab0104,
        row_index: usize,
    ) -> VendorActivationOutcome0104 {
        match tab {
            VendorTab0104::Buy => match self.catalog_quantity_contract(row_index) {
                Ok(quantity) => VendorActivationOutcome0104::Popup(VendorItemActionPopup0104 {
                    source: VendorItemPopupSource0104::CatalogRow { row_index },
                    buy: Some(quantity),
                    buyback: false,
                    sell: None,
                    delete: false,
                    open_chest: false,
                }),
                Err(blocked) => VendorActivationOutcome0104::Action(
                    VendorActionOutcome0104::SilentBlocked(blocked),
                ),
            },
            VendorTab0104::Buyback => {
                if self.recent_buy_rows.get(row_index).is_none() {
                    return VendorActivationOutcome0104::Action(
                        VendorActionOutcome0104::SilentBlocked(
                            VendorSilentBlock0104::InvalidRecentBuyRow { row_index },
                        ),
                    );
                }
                VendorActivationOutcome0104::Popup(VendorItemActionPopup0104 {
                    source: VendorItemPopupSource0104::BuybackRow { row_index },
                    buy: None,
                    buyback: true,
                    sell: None,
                    delete: false,
                    open_chest: false,
                })
            }
        }
    }

    /// Secondary-button release from clean `Panel_Vendor.DoSlot`.
    #[must_use]
    pub fn secondary_row_activation(
        &self,
        tab: VendorTab0104,
        row_index: usize,
    ) -> VendorActivationOutcome0104 {
        match tab {
            VendorTab0104::Buyback => {
                VendorActivationOutcome0104::Action(self.request_restore(row_index))
            }
            VendorTab0104::Buy => {
                let Some(row) = self.catalog_rows.get(row_index) else {
                    return VendorActivationOutcome0104::Action(
                        VendorActionOutcome0104::SilentBlocked(
                            VendorSilentBlock0104::InvalidCatalogRow { row_index },
                        ),
                    );
                };
                if row.item.item_type < 7 || matches!(row.item.item_type, 9 | 10) {
                    VendorActivationOutcome0104::Action(self.request_buy(row_index, 0))
                } else {
                    self.primary_row_activation(tab, row_index)
                }
            }
        }
    }

    /// Primary inventory click through `PopupControll` in Vendor mode.
    #[must_use]
    pub fn primary_inventory_activation(
        &self,
        inventory_slot: usize,
    ) -> VendorActivationOutcome0104 {
        let Some(projected) = self.inventory.get(inventory_slot) else {
            return VendorActivationOutcome0104::Action(VendorActionOutcome0104::SilentBlocked(
                VendorSilentBlock0104::InvalidInventorySlot { inventory_slot },
            ));
        };
        if projected.item.empty {
            return VendorActivationOutcome0104::Action(VendorActionOutcome0104::SilentBlocked(
                VendorSilentBlock0104::EmptyInventorySlot { inventory_slot },
            ));
        }
        let item = projected.item.item;
        if item.item_type == 7 {
            let Some(metadata) = projected.item.metadata.as_ref() else {
                return VendorActivationOutcome0104::Action(
                    VendorActionOutcome0104::SilentBlocked(
                        VendorSilentBlock0104::MissingCatalogMetadata {
                            item_type: item.item_type,
                            item_id: item.item_id,
                        },
                    ),
                );
            };
            if !metadata.sellable {
                return VendorActivationOutcome0104::Action(
                    VendorActionOutcome0104::SystemMessage(VendorSystemMessage0104::plain(
                        VendorSystemMessageId0104::ItemNotSellable,
                    )),
                );
            }
            return VendorActivationOutcome0104::Popup(VendorItemActionPopup0104 {
                source: VendorItemPopupSource0104::InventorySlot { inventory_slot },
                buy: None,
                buyback: false,
                sell: Some(VendorQuantityContract0104::Calculator {
                    maximum: item.option.max(0),
                }),
                // GumPopup type 3 has no delete button. General items are
                // still deletable through the clean trash drop target.
                delete: false,
                open_chest: false,
            });
        }
        VendorActivationOutcome0104::Popup(VendorItemActionPopup0104 {
            source: VendorItemPopupSource0104::InventorySlot { inventory_slot },
            buy: None,
            buyback: false,
            sell: (item.item_type < 7).then_some(VendorQuantityContract0104::Fixed(1)),
            delete: true,
            open_chest: item.item_type == 9,
        })
    }

    /// Secondary-button release from clean `InventoryManagerScript.OneClickItem`.
    #[must_use]
    pub fn secondary_inventory_activation(
        &self,
        inventory_slot: usize,
    ) -> VendorActivationOutcome0104 {
        VendorActivationOutcome0104::Action(self.request_sell(inventory_slot, 1))
    }

    /// Exact `Panel_PCStuffScript` trash/hammer drop targets.
    #[must_use]
    pub fn inventory_drop_activation(
        &self,
        inventory_slot: usize,
        target: VendorInventoryDropTarget0104,
    ) -> VendorActionOutcome0104 {
        match target {
            VendorInventoryDropTarget0104::Trash => self.request_delete(inventory_slot),
            VendorInventoryDropTarget0104::Hammer => self.request_disassemble(inventory_slot),
        }
    }

    /// Resolves a button/calculator choice produced by the typed popup owner.
    #[must_use]
    pub fn commit_item_popup(
        &self,
        popup: VendorItemActionPopup0104,
        commit: VendorPopupCommit0104,
    ) -> VendorActionOutcome0104 {
        match (popup.source, commit) {
            (
                VendorItemPopupSource0104::CatalogRow { row_index },
                VendorPopupCommit0104::Buy { selected_option },
            ) if popup
                .buy
                .is_some_and(|contract| contract.accepts(selected_option)) =>
            {
                self.request_buy(row_index, selected_option)
            }
            (
                VendorItemPopupSource0104::BuybackRow { row_index },
                VendorPopupCommit0104::Buyback,
            ) if popup.buyback => self.request_restore(row_index),
            (
                VendorItemPopupSource0104::InventorySlot { inventory_slot },
                VendorPopupCommit0104::Sell { count },
            ) if popup.sell.is_some_and(|contract| contract.accepts(count)) => {
                self.request_sell(inventory_slot, count)
            }
            (
                VendorItemPopupSource0104::InventorySlot { inventory_slot },
                VendorPopupCommit0104::Delete,
            ) if popup.delete => self.request_delete(inventory_slot),
            _ => VendorActionOutcome0104::SilentBlocked(
                VendorSilentBlock0104::PopupActionUnavailable,
            ),
        }
    }

    #[must_use]
    pub fn request_buy(&self, row_index: usize, selected_option: i32) -> VendorActionOutcome0104 {
        let Some(row) = self.catalog_rows.get(row_index) else {
            return VendorActionOutcome0104::SilentBlocked(
                VendorSilentBlock0104::InvalidCatalogRow { row_index },
            );
        };
        if selected_option < 0 {
            return VendorActionOutcome0104::SilentBlocked(VendorSilentBlock0104::InvalidCount {
                requested: selected_option,
                maximum: None,
            });
        }

        if selected_option == 0 {
            return self.normal_buy_outcome(row.item, 1);
        }

        let Some(metadata) = row.metadata.as_ref() else {
            return VendorActionOutcome0104::SilentBlocked(
                VendorSilentBlock0104::MissingCatalogMetadata {
                    item_type: row.item.item_type,
                    item_id: row.item.item_id,
                },
            );
        };
        if metadata.general_item_type != Some(2) {
            return self.normal_buy_outcome(row.item, selected_option);
        }

        let Some(battery_recharge) = metadata.battery_recharge else {
            return VendorActionOutcome0104::SilentBlocked(
                VendorSilentBlock0104::MissingBatteryMetadata {
                    item_id: row.item.item_id,
                },
            );
        };
        let mut item = row.item;
        item.option = selected_option;
        if battery_recharge == 0 {
            let Some(inventory_slot) = self.first_empty_inventory_slot() else {
                // Clean `BuyGeneral` writes option=-1 and returns without a
                // message when this branch has no destination.
                return VendorActionOutcome0104::SilentBlocked(
                    VendorSilentBlock0104::InventoryFullForGeneralBuy,
                );
            };
            VendorActionOutcome0104::Intent(VendorIntent0104::BuyGeneral(
                VendorBuyGeneralIntent0104 {
                    identity: self.session.request_identity(),
                    item,
                    inventory_slot,
                },
            ))
        } else {
            VendorActionOutcome0104::Intent(VendorIntent0104::Battery(VendorBatteryIntent0104 {
                identity: self.session.request_identity(),
                item,
                battery_recharge,
            }))
        }
    }

    pub(super) fn normal_buy_outcome(&self, mut item: ItemBase0104, option: i32) -> VendorActionOutcome0104 {
        let Some(inventory_slot) = self.first_empty_inventory_slot() else {
            return VendorActionOutcome0104::SystemMessage(VendorSystemMessage0104::plain(
                VendorSystemMessageId0104::InventoryFull,
            ));
        };
        item.option = option;
        VendorActionOutcome0104::Intent(VendorIntent0104::Buy(VendorBuyIntent0104 {
            identity: self.session.request_identity(),
            item,
            inventory_slot,
        }))
    }

    #[must_use]
    pub fn request_restore(&self, row_index: usize) -> VendorActionOutcome0104 {
        let Some(row) = self.recent_buy_rows.get(row_index) else {
            return VendorActionOutcome0104::SilentBlocked(
                VendorSilentBlock0104::InvalidRecentBuyRow { row_index },
            );
        };
        let Some(inventory_slot) = self.first_empty_inventory_slot() else {
            // Clean `RecentBuy` silently skips the request when full.
            return VendorActionOutcome0104::SilentBlocked(
                VendorSilentBlock0104::InventoryFullForRestore,
            );
        };
        let mut item = row.item;
        item.time_limit = 0;
        VendorActionOutcome0104::Intent(VendorIntent0104::Restore(VendorRestoreIntent0104 {
            identity: self.session.request_identity(),
            restore_list_id: row.restore_list_id,
            item,
            inventory_slot,
        }))
    }

    #[must_use]
    pub fn request_sell(
        &self,
        inventory_slot: usize,
        calculator_count: i32,
    ) -> VendorActionOutcome0104 {
        let Some(projected) = self.inventory.get(inventory_slot) else {
            return VendorActionOutcome0104::SilentBlocked(
                VendorSilentBlock0104::InvalidInventorySlot { inventory_slot },
            );
        };
        let item = projected.item.item;
        if projected.item.empty {
            return VendorActionOutcome0104::SilentBlocked(
                VendorSilentBlock0104::EmptyInventorySlot { inventory_slot },
            );
        }
        match item.item_type {
            9 => {
                return VendorActionOutcome0104::SystemMessage(VendorSystemMessage0104::plain(
                    VendorSystemMessageId0104::CannotSellChest,
                ));
            }
            8 => {
                return VendorActionOutcome0104::SystemMessage(VendorSystemMessage0104::plain(
                    VendorSystemMessageId0104::CannotSellQuestItem,
                ));
            }
            _ => {}
        }
        let Some(metadata) = projected.item.metadata.as_ref() else {
            return VendorActionOutcome0104::SilentBlocked(
                VendorSilentBlock0104::MissingCatalogMetadata {
                    item_type: item.item_type,
                    item_id: item.item_id,
                },
            );
        };
        if !metadata.sellable {
            return VendorActionOutcome0104::SystemMessage(VendorSystemMessage0104::plain(
                VendorSystemMessageId0104::ItemNotSellable,
            ));
        }
        let count = if item.item_type < 7 {
            1
        } else {
            let maximum = (item.item_type == 7).then_some(item.option.max(0));
            if calculator_count <= 0 || maximum.is_some_and(|maximum| calculator_count > maximum) {
                return VendorActionOutcome0104::SilentBlocked(
                    VendorSilentBlock0104::InvalidCount {
                        requested: calculator_count,
                        maximum,
                    },
                );
            }
            calculator_count
        };
        VendorActionOutcome0104::Intent(VendorIntent0104::Sell(VendorSellIntent0104 {
            item,
            inventory_slot,
            count,
        }))
    }

    #[must_use]
    pub fn request_delete(&self, inventory_slot: usize) -> VendorActionOutcome0104 {
        let Some(projected) = self.inventory.get(inventory_slot) else {
            return VendorActionOutcome0104::SilentBlocked(
                VendorSilentBlock0104::InvalidInventorySlot { inventory_slot },
            );
        };
        if projected.item.empty {
            return VendorActionOutcome0104::SilentBlocked(
                VendorSilentBlock0104::EmptyInventorySlot { inventory_slot },
            );
        }
        let item = projected.item.item;
        let intent = VendorIntent0104::Delete(VendorDeleteIntent0104 {
            location: VendorInventoryLocation0104::Inventory,
            inventory_slot,
            item,
        });
        VendorActionOutcome0104::Confirmation(VendorConfirmation0104 {
            message_id: VendorSystemMessageId0104::ConfirmDelete,
            callback: VendorConfirmationCallback0104::DeleteItemOk,
            icon: projected.item.icon.clone(),
            delete_count: (item.item_type == 7).then_some(item.option),
            intent,
        })
    }

    #[must_use]
    pub fn request_disassemble(&self, inventory_slot: usize) -> VendorActionOutcome0104 {
        let Some(projected) = self.inventory.get(inventory_slot) else {
            return VendorActionOutcome0104::SilentBlocked(
                VendorSilentBlock0104::InvalidInventorySlot { inventory_slot },
            );
        };
        if projected.item.empty {
            return VendorActionOutcome0104::SilentBlocked(
                VendorSilentBlock0104::EmptyInventorySlot { inventory_slot },
            );
        }
        let item = projected.item.item;
        if !matches!(item.item_type, 0 | 1 | 2 | 3 | 7) {
            return VendorActionOutcome0104::SilentBlocked(
                VendorSilentBlock0104::DisassembleIneligible {
                    item_type: item.item_type,
                },
            );
        }
        let intent = VendorIntent0104::Disassemble(VendorDisassembleIntent0104 {
            inventory_slot,
            item,
        });
        VendorActionOutcome0104::Confirmation(VendorConfirmation0104 {
            message_id: VendorSystemMessageId0104::ConfirmDisassemble,
            callback: VendorConfirmationCallback0104::HammerItemOk,
            icon: projected.item.icon.clone(),
            delete_count: None,
            intent,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum VendorInventoryLocation0104 {
    Inventory = 1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VendorInventoryDropTarget0104 {
    Trash,
    Hammer,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct VendorModalState {
    pub help: bool,
    pub inventory_popup: bool,
    pub system_popup: bool,
    pub generic_popup: bool,
}

impl VendorModalState {
    #[must_use]
    pub const fn vendor_panel_blocked(self) -> bool {
        self.help || self.inventory_popup || self.system_popup || self.generic_popup
    }

    #[must_use]
    pub const fn pc_stuff_blocked(self) -> bool {
        self.vendor_panel_blocked() || self.inventory_popup
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Resource)]
pub struct VendorUiState {
    pub phase: VendorLifecyclePhase,
    pub opening_elapsed_seconds: f32,
    pub tab: VendorTab0104,
    pub vendor_scroll_y: f32,
    pub inventory_scroll_y: f32,
    pub send_pending: bool,
}

impl Default for VendorUiState {
    fn default() -> Self {
        Self {
            phase: VendorLifecyclePhase::Hidden,
            opening_elapsed_seconds: 0.0,
            tab: VendorTab0104::Buy,
            vendor_scroll_y: 0.0,
            inventory_scroll_y: 0.0,
            send_pending: false,
        }
    }
}
