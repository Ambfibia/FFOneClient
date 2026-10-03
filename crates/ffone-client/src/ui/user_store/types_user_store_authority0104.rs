use super::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(i32)]
pub enum UserStoreMode0104 {
    #[default]
    MyStore = 0,
    UserStore = 1,
    /// Declared by clean `cnStore`, but no setup or transition reaches it.
    ReturnStore = 2,
}

impl UserStoreMode0104 {
    #[must_use]
    pub const fn observed_setup(self) -> bool {
        matches!(self, Self::MyStore | Self::UserStore)
    }

    #[must_use]
    pub const fn listing_slot_type(self) -> Option<i32> {
        match self {
            Self::MyStore => Some(USER_STORE_MY_SLOT_TYPE),
            Self::UserStore => Some(USER_STORE_OTHER_SLOT_TYPE),
            Self::ReturnStore => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserStorePacket0104 {
    pub packet_id: u32,
    pub payload: Vec<u8>,
}

impl UserStorePacket0104 {
    #[must_use]
    pub fn ready(open_item_inventory_slot: i32) -> Self {
        Self::single_i32(STREETSTALL_REQ_READY, open_item_inventory_slot)
    }

    #[must_use]
    pub fn cancel(pc_id: i32) -> Self {
        Self::single_i32(STREETSTALL_REQ_CANCEL, pc_id)
    }

    #[must_use]
    pub fn register_item(
        list_slot: i32,
        inventory_slot: i32,
        item: ItemBase0104,
        price: i32,
    ) -> Self {
        let mut payload = vec![0_u8; STREETSTALL_REGISTER_REQUEST_SIZE];
        write_i32(&mut payload, 0, list_slot);
        write_i32(&mut payload, 4, inventory_slot);
        encode_item(item, &mut payload[8..20]);
        write_i32(&mut payload, 20, price);
        Self {
            packet_id: STREETSTALL_REQ_REGISTER_ITEM,
            payload,
        }
    }

    #[must_use]
    pub fn unregister_item(list_slot: i32) -> Self {
        Self::single_i32(STREETSTALL_REQ_UNREGISTER_ITEM, list_slot)
    }

    #[must_use]
    pub fn sale_start(open_item_inventory_slot: i32) -> Self {
        Self::single_i32(STREETSTALL_REQ_SALE_START, open_item_inventory_slot)
    }

    #[must_use]
    pub fn item_list(target_pc_id: i32) -> Self {
        Self::single_i32(STREETSTALL_REQ_ITEM_LIST, target_pc_id)
    }

    #[must_use]
    pub fn item_buy(target_pc_id: i32, list_slot: i32, empty_inventory_slot: i32) -> Self {
        let mut payload = vec![0_u8; STREETSTALL_ITEM_BUY_REQUEST_SIZE];
        write_i32(&mut payload, 0, target_pc_id);
        write_i32(&mut payload, 4, list_slot);
        write_i32(&mut payload, 8, empty_inventory_slot);
        Self {
            packet_id: STREETSTALL_REQ_ITEM_BUY,
            payload,
        }
    }

    pub(super) fn single_i32(packet_id: u32, value: i32) -> Self {
        Self {
            packet_id,
            payload: value.to_le_bytes().to_vec(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UserStoreReadySuccess0104 {
    pub open_item_inventory_slot: i32,
    pub item_list_count_max: i32,
    pub tax_percentage: f32,
    pub pc_char_state: i8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UserStoreListingRecord0104 {
    pub list_slot: i32,
    pub item: ItemBase0104,
    pub price: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserStoreItemListSuccess0104 {
    pub target_pc_id: i32,
    pub records: Vec<UserStoreListingRecord0104>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UserStoreRegisterSuccess0104 {
    pub list_slot: i32,
    pub inventory_slot: i32,
    pub item: ItemBase0104,
    pub price: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UserStoreSaleStartSuccess0104 {
    pub open_item_inventory_slot: i32,
    pub open_item: ItemBase0104,
    pub pc_char_state: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UserStoreBuyBuyerSuccess0104 {
    pub target_pc_id: i32,
    pub buyer_taros: i32,
    pub buyer_inventory_slot: i32,
    pub item: ItemBase0104,
    pub list_slot: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UserStoreBuySellerSuccess0104 {
    pub buyer_pc_id: i32,
    pub seller_taros: i32,
    pub seller_inventory_slot: i32,
    pub item: ItemBase0104,
    pub list_slot: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UserStoreReply0104 {
    ReadySuccess(UserStoreReadySuccess0104),
    ReadyFail(i32),
    CancelSuccess { pc_char_state: i8 },
    CancelFail(i32),
    RegisterSuccess(UserStoreRegisterSuccess0104),
    RegisterFail(i32),
    UnregisterSuccess { list_slot: i32 },
    UnregisterFail(i32),
    SaleStartSuccess(UserStoreSaleStartSuccess0104),
    SaleStartFail(i32),
    ItemListSuccess(UserStoreItemListSuccess0104),
    ItemListFail(i32),
    BuySuccessBuyer(UserStoreBuyBuyerSuccess0104),
    BuySuccessSeller(UserStoreBuySellerSuccess0104),
    BuyFail(i32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UserStoreListing0104 {
    pub list_slot: i32,
    /// The original inventory slot is known for the owner's own listings and
    /// intentionally absent for another player's packet-projected list.
    pub inventory_slot: Option<i32>,
    pub item: ItemBase0104,
    /// A negative price is clean-client's retained "sold" representation.
    pub price: i32,
}

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct UserStoreAuthority0104 {
    pub owner_pc_id: i32,
    pub target_pc_id: Option<i32>,
    pub taros: i32,
    pub inventory: [ItemBase0104; USER_STORE_INVENTORY_CAPACITY],
    pub equipment: [ItemBase0104; USER_STORE_EQUIPMENT_CAPACITY],
    pub listings: [Option<UserStoreListing0104>; USER_STORE_LIST_CAPACITY],
    pub maximum_list_slots: usize,
    pub tax_percentage: f32,
    pub open_item_inventory_slot: i32,
    pub open_item: ItemBase0104,
    pub store_open: bool,
}

impl Default for UserStoreAuthority0104 {
    fn default() -> Self {
        Self {
            owner_pc_id: 0,
            target_pc_id: None,
            taros: 0,
            inventory: [EMPTY_ITEM_0104; USER_STORE_INVENTORY_CAPACITY],
            equipment: [EMPTY_ITEM_0104; USER_STORE_EQUIPMENT_CAPACITY],
            listings: [None; USER_STORE_LIST_CAPACITY],
            maximum_list_slots: 0,
            tax_percentage: 0.0,
            open_item_inventory_slot: -1,
            open_item: EMPTY_ITEM_0104,
            store_open: false,
        }
    }
}

impl UserStoreAuthority0104 {
    #[must_use]
    pub fn first_empty_inventory_slot(&self) -> Option<i32> {
        self.inventory
            .iter()
            .copied()
            .position(user_store_item_is_empty)
            .map(|slot| slot as i32)
    }

    #[must_use]
    pub fn first_empty_listing_slot(&self) -> Option<i32> {
        let maximum = self.maximum_list_slots.min(USER_STORE_LIST_CAPACITY);
        self.listings[..maximum]
            .iter()
            .position(Option::is_none)
            .map(|slot| slot as i32)
    }

    #[must_use]
    pub fn occupied_listing_count(&self) -> usize {
        self.listings
            .iter()
            .filter(|listing| listing.is_some())
            .count()
    }

    #[must_use]
    pub fn visible_listing_count(&self, mode: UserStoreMode0104) -> usize {
        match mode {
            UserStoreMode0104::MyStore => self.maximum_list_slots.min(USER_STORE_LIST_CAPACITY),
            UserStoreMode0104::UserStore => self.occupied_listing_count(),
            UserStoreMode0104::ReturnStore => 0,
        }
    }

    pub(super) fn setup_ready(
        &mut self,
        reply: UserStoreReadySuccess0104,
    ) -> Result<(), UserStoreApplyError0104> {
        let maximum = usize::try_from(reply.item_list_count_max)
            .ok()
            .filter(|value| *value <= USER_STORE_LIST_CAPACITY)
            .ok_or(UserStoreApplyError0104::InvalidMaximumListSlots(
                reply.item_list_count_max,
            ))?;
        if inventory_slot_index(reply.open_item_inventory_slot).is_none() {
            return Err(UserStoreApplyError0104::InvalidInventorySlot(
                reply.open_item_inventory_slot,
            ));
        }
        self.target_pc_id = None;
        self.maximum_list_slots = maximum;
        self.tax_percentage = reply.tax_percentage;
        self.open_item_inventory_slot = reply.open_item_inventory_slot;
        self.open_item = self.inventory[reply.open_item_inventory_slot as usize];
        // Clean `Panel_UserStore.SetReady` unconditionally sets storeFlag=0;
        // the returned character-state byte is retained only as wire evidence.
        self.store_open = false;
        self.listings = [None; USER_STORE_LIST_CAPACITY];
        Ok(())
    }

    pub(super) fn setup_item_list(
        &mut self,
        reply: &UserStoreItemListSuccess0104,
    ) -> Result<(), UserStoreApplyError0104> {
        let mut listings = [None; USER_STORE_LIST_CAPACITY];
        for record in &reply.records {
            let slot = list_slot_index(record.list_slot)
                .map_err(|_| UserStoreApplyError0104::InvalidListSlot(record.list_slot))?;
            listings[slot] = Some(UserStoreListing0104 {
                list_slot: record.list_slot,
                inventory_slot: None,
                item: record.item,
                price: record.price,
            });
        }
        self.target_pc_id = Some(reply.target_pc_id);
        self.maximum_list_slots = USER_STORE_LIST_CAPACITY;
        self.listings = listings;
        self.store_open = true;
        Ok(())
    }

    pub(super) fn apply_register(
        &mut self,
        reply: UserStoreRegisterSuccess0104,
    ) -> Result<(), UserStoreApplyError0104> {
        let list_slot = list_slot_index(reply.list_slot)
            .map_err(|_| UserStoreApplyError0104::InvalidListSlot(reply.list_slot))?;
        if list_slot >= self.maximum_list_slots.min(USER_STORE_LIST_CAPACITY) {
            return Err(UserStoreApplyError0104::ListSlotOverMaximum(
                reply.list_slot,
            ));
        }
        if self.listings[list_slot].is_some() {
            return Err(UserStoreApplyError0104::OccupiedListSlot(reply.list_slot));
        }
        let inventory_slot = inventory_slot_index(reply.inventory_slot).ok_or(
            UserStoreApplyError0104::InvalidInventorySlot(reply.inventory_slot),
        )?;
        let source = self.inventory[inventory_slot];
        if user_store_item_is_empty(source) {
            return Err(UserStoreApplyError0104::EmptyInventorySlot(
                reply.inventory_slot,
            ));
        }
        if source.item_type != reply.item.item_type
            || source.item_id != reply.item.item_id
            || source.time_limit != reply.item.time_limit
        {
            return Err(UserStoreApplyError0104::ItemMismatch);
        }
        if reply.item.item_type == USER_STORE_GENERAL_ITEM_TYPE {
            if reply.item.option <= 0 || source.option < reply.item.option {
                return Err(UserStoreApplyError0104::InvalidStackQuantity {
                    available: source.option,
                    requested: reply.item.option,
                });
            }
            if source.option == reply.item.option {
                self.inventory[inventory_slot] = EMPTY_ITEM_0104;
            } else {
                self.inventory[inventory_slot].option -= reply.item.option;
            }
        } else {
            if source != reply.item {
                return Err(UserStoreApplyError0104::ItemMismatch);
            }
            self.inventory[inventory_slot] = EMPTY_ITEM_0104;
        }
        self.listings[list_slot] = Some(UserStoreListing0104 {
            list_slot: reply.list_slot,
            inventory_slot: Some(reply.inventory_slot),
            item: reply.item,
            price: reply.price,
        });
        Ok(())
    }

    pub(super) fn apply_unregister(&mut self, list_slot: i32) -> Result<(), UserStoreApplyError0104> {
        let list_index = list_slot_index(list_slot)
            .map_err(|_| UserStoreApplyError0104::InvalidListSlot(list_slot))?;
        let listing =
            self.listings[list_index].ok_or(UserStoreApplyError0104::EmptyListSlot(list_slot))?;
        let inventory_slot = listing
            .inventory_slot
            .and_then(inventory_slot_index)
            .ok_or(UserStoreApplyError0104::MissingOriginalInventorySlot(
                list_slot,
            ))?;
        let destination = self.inventory[inventory_slot];
        if listing.item.item_type == USER_STORE_GENERAL_ITEM_TYPE {
            if user_store_item_is_empty(destination) {
                self.inventory[inventory_slot] = listing.item;
            } else if destination.item_type == listing.item.item_type
                && destination.item_id == listing.item.item_id
                && destination.time_limit == listing.item.time_limit
            {
                self.inventory[inventory_slot].option = destination
                    .option
                    .checked_add(listing.item.option)
                    .ok_or(UserStoreApplyError0104::StackOverflow)?;
            } else {
                return Err(UserStoreApplyError0104::OccupiedInventorySlot(
                    inventory_slot as i32,
                ));
            }
        } else if user_store_item_is_empty(destination) {
            self.inventory[inventory_slot] = listing.item;
        } else {
            return Err(UserStoreApplyError0104::OccupiedInventorySlot(
                inventory_slot as i32,
            ));
        }
        self.listings[list_index] = None;
        Ok(())
    }

    pub(super) fn apply_sale_start(
        &mut self,
        reply: UserStoreSaleStartSuccess0104,
    ) -> Result<(), UserStoreApplyError0104> {
        let slot = inventory_slot_index(reply.open_item_inventory_slot).ok_or(
            UserStoreApplyError0104::InvalidInventorySlot(reply.open_item_inventory_slot),
        )?;
        if self.inventory[slot] != reply.open_item {
            return Err(UserStoreApplyError0104::ItemMismatch);
        }
        self.open_item_inventory_slot = reply.open_item_inventory_slot;
        self.open_item = reply.open_item;
        self.store_open = true;
        Ok(())
    }

    pub(super) fn apply_buyer(
        &mut self,
        reply: UserStoreBuyBuyerSuccess0104,
    ) -> Result<(), UserStoreApplyError0104> {
        let target = self
            .target_pc_id
            .ok_or(UserStoreApplyError0104::MissingTargetPc)?;
        if target != reply.target_pc_id {
            return Err(UserStoreApplyError0104::TargetPcMismatch {
                expected: target,
                actual: reply.target_pc_id,
            });
        }
        let list_slot = list_slot_index(reply.list_slot)
            .map_err(|_| UserStoreApplyError0104::InvalidListSlot(reply.list_slot))?;
        let listing = self.listings[list_slot]
            .ok_or(UserStoreApplyError0104::EmptyListSlot(reply.list_slot))?;
        if listing.price < 0 {
            return Err(UserStoreApplyError0104::AlreadySold(reply.list_slot));
        }
        if listing.item != reply.item {
            return Err(UserStoreApplyError0104::ItemMismatch);
        }
        let inventory_slot = inventory_slot_index(reply.buyer_inventory_slot).ok_or(
            UserStoreApplyError0104::InvalidInventorySlot(reply.buyer_inventory_slot),
        )?;
        if !user_store_item_is_empty(self.inventory[inventory_slot]) {
            return Err(UserStoreApplyError0104::OccupiedInventorySlot(
                reply.buyer_inventory_slot,
            ));
        }
        self.taros = reply.buyer_taros;
        self.inventory[inventory_slot] = reply.item;
        self.listings[list_slot]
            .as_mut()
            .expect("validated listing")
            .price = -1;
        Ok(())
    }

    pub(super) fn apply_seller(
        &mut self,
        reply: UserStoreBuySellerSuccess0104,
    ) -> Result<(), UserStoreApplyError0104> {
        let list_slot = list_slot_index(reply.list_slot)
            .map_err(|_| UserStoreApplyError0104::InvalidListSlot(reply.list_slot))?;
        let listing = self.listings[list_slot]
            .ok_or(UserStoreApplyError0104::EmptyListSlot(reply.list_slot))?;
        if listing.price < 0 {
            return Err(UserStoreApplyError0104::AlreadySold(reply.list_slot));
        }
        if listing.item != reply.item {
            return Err(UserStoreApplyError0104::ItemMismatch);
        }
        if listing.inventory_slot != Some(reply.seller_inventory_slot) {
            return Err(UserStoreApplyError0104::OriginalInventorySlotMismatch {
                expected: listing.inventory_slot,
                actual: reply.seller_inventory_slot,
            });
        }
        self.taros = reply.seller_taros;
        self.listings[list_slot]
            .as_mut()
            .expect("validated listing")
            .price = -1;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserStorePendingRequest0104 {
    Ready {
        open_item_inventory_slot: i32,
    },
    Cancel {
        owner_pc_id: i32,
    },
    Register {
        list_slot: i32,
        inventory_slot: i32,
        item: ItemBase0104,
        price: i32,
    },
    Unregister {
        list_slot: i32,
    },
    SaleStart {
        open_item_inventory_slot: i32,
    },
    ItemList {
        target_pc_id: i32,
    },
    Buy {
        target_pc_id: i32,
        list_slot: i32,
        empty_inventory_slot: i32,
    },
}

impl UserStorePendingRequest0104 {
    #[must_use]
    pub const fn expected_success_packet(self) -> u32 {
        match self {
            Self::Ready { .. } => STREETSTALL_REP_READY_SUCCESS,
            Self::Cancel { .. } => STREETSTALL_REP_CANCEL_SUCCESS,
            Self::Register { .. } => STREETSTALL_REP_REGISTER_ITEM_SUCCESS,
            Self::Unregister { .. } => STREETSTALL_REP_UNREGISTER_ITEM_SUCCESS,
            Self::SaleStart { .. } => STREETSTALL_REP_SALE_START_SUCCESS,
            Self::ItemList { .. } => STREETSTALL_REP_ITEM_LIST,
            Self::Buy { .. } => STREETSTALL_REP_ITEM_BUY_SUCCESS_BUYER,
        }
    }

    #[must_use]
    pub const fn expected_failure_packet(self) -> u32 {
        match self {
            Self::Ready { .. } => STREETSTALL_REP_READY_FAIL,
            Self::Cancel { .. } => STREETSTALL_REP_CANCEL_FAIL,
            Self::Register { .. } => STREETSTALL_REP_REGISTER_ITEM_FAIL,
            Self::Unregister { .. } => STREETSTALL_REP_UNREGISTER_ITEM_FAIL,
            Self::SaleStart { .. } => STREETSTALL_REP_SALE_START_FAIL,
            Self::ItemList { .. } => STREETSTALL_REP_ITEM_LIST_FAIL,
            Self::Buy { .. } => STREETSTALL_REP_ITEM_BUY_FAIL,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UserStoreModalState0104 {
    pub system_popup: bool,
    pub item_popup: bool,
    pub help: bool,
    pub external_close_gate: bool,
    pub external_gameplay_gate: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserStorePopupKind0104 {
    RegisterQuantity,
    RegisterPrice,
    Unregister,
    Buy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UserStoreItemPopup0104 {
    pub kind: UserStorePopupKind0104,
    pub slot_type: i32,
    pub slot: i32,
    pub item: ItemBase0104,
    pub maximum_quantity: i32,
    pub price: i32,
}

/// Native presentation state for the clean `GumPopup` calculator embedded in
/// the UserStore item dialog. Network-authoritative store state remains in
/// `UserStoreAuthority0104`; this resource only owns the locally typed digits
/// and the currently displayed typed popup contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct UserStorePopupPresentation0104 {
    pub popup: Option<UserStoreItemPopup0104>,
    pub value: i32,
    pub first_digit: bool,
}

impl Default for UserStorePopupPresentation0104 {
    fn default() -> Self {
        Self {
            popup: None,
            value: 0,
            first_digit: true,
        }
    }
}

impl UserStorePopupPresentation0104 {
    pub fn open(&mut self, popup: UserStoreItemPopup0104) {
        self.value = match popup.kind {
            UserStorePopupKind0104::Unregister | UserStorePopupKind0104::Buy => {
                popup.maximum_quantity.max(0)
            }
            UserStorePopupKind0104::RegisterQuantity | UserStorePopupKind0104::RegisterPrice => 0,
        };
        self.popup = Some(popup);
        self.first_digit = true;
    }

    pub fn close(&mut self) {
        *self = Self::default();
    }

    #[must_use]
    pub fn keypad_enabled(&self) -> bool {
        self.popup.is_some_and(|popup| {
            matches!(
                popup.kind,
                UserStorePopupKind0104::RegisterQuantity | UserStorePopupKind0104::RegisterPrice
            )
        })
    }

    #[must_use]
    pub fn maximum_input(&self) -> i32 {
        self.popup.map_or(0, |popup| match popup.kind {
            UserStorePopupKind0104::RegisterQuantity => popup.maximum_quantity.max(0),
            UserStorePopupKind0104::RegisterPrice => USER_STORE_POPUP_MAX_PRICE,
            UserStorePopupKind0104::Unregister | UserStorePopupKind0104::Buy => {
                popup.maximum_quantity.max(0)
            }
        })
    }

    pub fn append_digit(&mut self, digit: u8) {
        if digit > 9 || !self.keypad_enabled() {
            return;
        }
        let next = if self.first_digit {
            i32::from(digit)
        } else {
            self.value
                .saturating_mul(10)
                .saturating_add(i32::from(digit))
        };
        self.value = next.min(self.maximum_input());
        self.first_digit = false;
    }

    pub fn clear_value(&mut self) {
        if self.keypad_enabled() {
            self.value = 0;
            self.first_digit = true;
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum UserStoreUiCommand0104 {
    SendPacket(UserStorePacket0104),
    SystemMessage(String),
    ServerFailure(i32),
    RejectedReply(UserStoreReplyReject0104),
    OpenItemPopup(UserStoreItemPopup0104),
    OpenHelp(i32),
    PlayButtonSound,
    UnlockCursor,
    RestoreCursor,
    ActivateStoreInventoryMode,
    ResetInventoryUiMode,
    ActivateSharedPanels,
    ChangeToGameplay,
    StartUiModeSound,
    StopUiModeSound,
    FreeAssets,
}

#[derive(Clone, Debug, Default, PartialEq, Resource)]
pub struct UserStoreUiOutbox0104(pub VecDeque<UserStoreUiCommand0104>);

impl UserStoreUiOutbox0104 {
    pub fn push(&mut self, command: UserStoreUiCommand0104) {
        self.0.push_back(command);
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct UserStoreUiState0104 {
    pub active: bool,
    pub mode: UserStoreMode0104,
    pub pending: Option<UserStorePendingRequest0104>,
    pub modal: UserStoreModalState0104,
    pub opening_elapsed_seconds: f32,
    /// Preserved clean dead state: the value changes but does not offset rows.
    pub inert_store_scroll_y: f32,
    pub inventory_scroll_y: f32,
    pub previous_cursor_locked: bool,
    pub last_error: Option<i32>,
}

impl Default for UserStoreUiState0104 {
    fn default() -> Self {
        Self {
            active: false,
            mode: UserStoreMode0104::MyStore,
            pending: None,
            modal: UserStoreModalState0104::default(),
            opening_elapsed_seconds: 0.0,
            inert_store_scroll_y: 0.0,
            inventory_scroll_y: 0.0,
            previous_cursor_locked: true,
            last_error: None,
        }
    }
}
