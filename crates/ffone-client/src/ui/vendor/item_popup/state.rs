use super::*;

#[derive(Clone, Debug)]
pub(super) struct Selection {
    pub(super) contract: VendorItemActionPopup0104,
    pub(super) owner: i32,
    pub(super) session: VendorSession0104,
    pub(super) source_slot: usize,
    pub(super) item: ItemBase0104,
    pub(super) unit_price: Option<i32>,
    pub(super) metadata: Option<VendorItemMetadata0104>,
    pub(super) icon: VendorPresentationIcon0104,
}

#[derive(Default, Resource)]
pub struct VendorItemPopupState {
    pub(super) selection: Option<Selection>,
    pub(super) amount: i32,
    pub(super) try_on: bool,
    pub try_on_allowed: bool,
    pub try_on_yaw: f32,
}

pub(super) fn selection(
    projection: &VendorModeProjection0104,
    contract: VendorItemActionPopup0104,
) -> Option<Selection> {
    let (item, metadata, icon, source_slot, unit_price) = match contract.source {
        VendorItemPopupSource0104::CatalogRow { row_index } => {
            let row = projection.catalog_rows.get(row_index)?;
            (
                row.item,
                row.metadata.clone(),
                row.icon.clone(),
                row.source_slot_id,
                row.price,
            )
        }
        VendorItemPopupSource0104::BuybackRow { row_index } => {
            let row = projection.recent_buy_rows.get(row_index)?;
            (
                row.item,
                row.metadata.clone(),
                row.icon.clone(),
                row.source_slot_id,
                row.price,
            )
        }
        VendorItemPopupSource0104::InventorySlot { inventory_slot } => {
            let row = projection.inventory.get(inventory_slot)?;
            if row.item.empty {
                return None;
            }
            (
                row.item.item,
                row.item.metadata.clone(),
                row.item.icon.clone(),
                row.slot_index,
                row.item
                    .metadata
                    .as_ref()
                    .map(|metadata| metadata.sell_price),
            )
        }
    };
    Some(Selection {
        contract,
        owner: projection.owner_pc_id,
        session: projection.session,
        source_slot,
        item,
        unit_price,
        metadata,
        icon,
    })
}

impl VendorItemPopupState {
    pub(crate) fn card_snapshot(&self) -> Option<crate::item_card::CardSnapshot> {
        let s = self.selection.as_ref()?;
        let catalog = matches!(
            s.contract.source,
            VendorItemPopupSource0104::CatalogRow { .. }
        );
        let price = if ((0..=6).contains(&s.item.item_type) || s.item.item_type == 10)
            && (catalog || s.contract.buyback || s.contract.sell.is_some())
        {
            s.metadata.as_ref().map(|m| {
                if catalog || s.contract.buyback {
                    m.buy_price
                } else {
                    m.sell_price
                }
            })
        } else {
            None
        };
        Some(crate::item_card::CardSnapshot {
            item: s.item,
            catalog,
            price,
        })
    }

    pub fn set_try_on_allowed(&mut self, allowed: bool) {
        if self.try_on_allowed != allowed {
            self.try_on_allowed = allowed;
        }
        if !allowed {
            self.try_on = false;
        }
    }
    pub fn selected_try_on_item(&self) -> Option<ItemBase0104> {
        let s = self.selection.as_ref()?;
        (matches!(
            s.contract.source,
            VendorItemPopupSource0104::CatalogRow { .. }
        ) && (0..=6).contains(&s.item.item_type))
        .then_some(s.item)
    }
    pub fn try_on_item(&self) -> Option<ItemBase0104> {
        (self.try_on && self.try_on_allowed)
            .then(|| self.selected_try_on_item())
            .flatten()
    }
    pub fn is_open(&self) -> bool {
        self.selection.is_some()
    }
    pub fn close(&mut self) {
        self.selection = None;
        self.amount = 0;
        self.try_on = false;
        self.try_on_allowed = false;
        self.try_on_yaw = 0.;
    }
    pub fn open(
        &mut self,
        contract: VendorItemActionPopup0104,
        projection: &VendorModeProjection0104,
    ) {
        self.close();
        if current_contract(projection, contract.source) == Some(contract) {
            self.selection = selection(projection, contract);
        }
    }
    pub(super) fn valid(&self, projection: &VendorModeProjection0104) -> bool {
        let Some(old) = &self.selection else {
            return false;
        };
        let Some(new) = selection(projection, old.contract) else {
            return false;
        };
        old.owner == new.owner
            && old.session == new.session
            && old.source_slot == new.source_slot
            && old.item == new.item
            && old.unit_price == new.unit_price
            && old
                .metadata
                .as_ref()
                .map(|metadata| (metadata.general_item_type, metadata.battery_recharge))
                == new
                    .metadata
                    .as_ref()
                    .map(|metadata| (metadata.general_item_type, metadata.battery_recharge))
            && current_contract(projection, old.contract.source) == Some(old.contract)
    }
    pub(super) fn quantity(&self) -> Option<VendorQuantityContract0104> {
        let contract = self.selection.as_ref()?.contract;
        contract.buy.or(contract.sell)
    }
    pub(super) fn cost_label(&self) -> Option<String> {
        let selected = self.selection.as_ref()?;
        let unit = selected.unit_price?;
        Some(
            if matches!(
                self.quantity(),
                Some(VendorQuantityContract0104::Calculator { .. })
            ) {
                format!(
                    "{unit} × {} = {}",
                    self.amount,
                    unit.saturating_mul(self.amount)
                )
            } else {
                unit.to_string()
            },
        )
    }
    pub(super) fn digit(&mut self, digit: u8) {
        if digit <= 9
            && let Some(VendorQuantityContract0104::Calculator { maximum }) = self.quantity()
        {
            self.amount = self
                .amount
                .saturating_mul(10)
                .saturating_add(i32::from(digit))
                .min(maximum.max(0));
        }
    }
    pub(super) fn commit(
        &mut self,
        delete: bool,
        projection: &VendorModeProjection0104,
    ) -> Option<(VendorItemPopupSource0104, VendorActionOutcome0104)> {
        if !self.valid(projection) {
            self.close();
            return None;
        }
        let contract = self.selection.as_ref()?.contract;
        let amount = match self.quantity() {
            Some(VendorQuantityContract0104::Fixed(value)) => value,
            _ => self.amount,
        };
        let commit = if delete && contract.delete {
            VendorPopupCommit0104::Delete
        } else if delete {
            return None;
        } else if contract.buy.is_some_and(|q| q.accepts(amount)) {
            VendorPopupCommit0104::Buy {
                selected_option: amount,
            }
        } else if contract.buyback {
            VendorPopupCommit0104::Buyback
        } else if contract.sell.is_some_and(|q| q.accepts(amount)) {
            VendorPopupCommit0104::Sell { count: amount }
        } else {
            return None;
        };
        let result = projection.commit_item_popup(contract, commit);
        self.close();
        Some((contract.source, result))
    }
}
