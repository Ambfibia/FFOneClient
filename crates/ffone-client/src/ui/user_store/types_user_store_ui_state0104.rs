use super::*;

impl UserStoreUiState0104 {
    #[must_use]
    pub const fn busy(&self) -> bool {
        self.pending.is_some()
    }

    #[must_use]
    pub fn input_boundary(&self) -> UserStoreInputBoundary0104 {
        let modal = self.modal.system_popup || self.modal.item_popup || self.modal.help;
        let close_enabled = self.active
            && !self.busy()
            && !modal
            && !self.modal.external_close_gate
            && self.mode != UserStoreMode0104::ReturnStore;
        UserStoreInputBoundary0104 {
            blocks_lower_ui: self.active,
            blocks_gameplay_input: self.active,
            requires_pointer: self.active,
            controls_enabled: close_enabled,
            close_enabled,
            escape_enabled: close_enabled && !self.modal.external_gameplay_gate,
        }
    }

    pub fn begin_my_store(
        &mut self,
        authority: &mut UserStoreAuthority0104,
        owner_pc_id: i32,
        open_item_inventory_slot: i32,
        previous_cursor_locked: bool,
        outbox: &mut UserStoreUiOutbox0104,
    ) -> UserStoreActionOutcome0104 {
        if self.active || inventory_slot_index(open_item_inventory_slot).is_none() {
            return UserStoreActionOutcome0104::Blocked(if self.active {
                UserStoreRequestBlock0104::Busy
            } else {
                UserStoreRequestBlock0104::InvalidInventorySlot
            });
        }
        *self = Self {
            active: true,
            mode: UserStoreMode0104::MyStore,
            pending: Some(UserStorePendingRequest0104::Ready {
                open_item_inventory_slot,
            }),
            previous_cursor_locked,
            ..default()
        };
        authority.owner_pc_id = owner_pc_id;
        authority.target_pc_id = None;
        authority.listings = [None; USER_STORE_LIST_CAPACITY];
        authority.maximum_list_slots = 0;
        authority.store_open = false;
        authority.open_item_inventory_slot = open_item_inventory_slot;
        let packet = UserStorePacket0104::ready(open_item_inventory_slot);
        push_entry_commands(outbox, packet.clone());
        UserStoreActionOutcome0104::Sent(packet)
    }

    pub fn begin_user_store(
        &mut self,
        authority: &mut UserStoreAuthority0104,
        owner_pc_id: i32,
        target_pc_id: i32,
        previous_cursor_locked: bool,
        outbox: &mut UserStoreUiOutbox0104,
    ) -> UserStoreActionOutcome0104 {
        if self.active {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::Busy);
        }
        *self = Self {
            active: true,
            mode: UserStoreMode0104::UserStore,
            pending: Some(UserStorePendingRequest0104::ItemList { target_pc_id }),
            previous_cursor_locked,
            ..default()
        };
        authority.owner_pc_id = owner_pc_id;
        authority.target_pc_id = Some(target_pc_id);
        authority.listings = [None; USER_STORE_LIST_CAPACITY];
        authority.maximum_list_slots = USER_STORE_LIST_CAPACITY;
        authority.store_open = true;
        let packet = UserStorePacket0104::item_list(target_pc_id);
        push_entry_commands(outbox, packet.clone());
        UserStoreActionOutcome0104::Sent(packet)
    }

    pub fn tick_opening(&mut self, delta_seconds: f32) {
        if self.active && delta_seconds.is_finite() && delta_seconds > 0.0 {
            self.opening_elapsed_seconds =
                (self.opening_elapsed_seconds + delta_seconds).min(USER_STORE_OPEN_SECONDS);
        }
    }

    pub fn scroll_store_dead(&mut self, delta: f32) {
        if delta.is_finite() {
            self.inert_store_scroll_y -= delta.clamp(
                -USER_STORE_SCROLL_INPUT_CLAMP,
                USER_STORE_SCROLL_INPUT_CLAMP,
            );
        }
    }

    pub fn scroll_inventory(&mut self, delta: f32) {
        if delta.is_finite() {
            self.inventory_scroll_y = clamp_user_store_inventory_scroll(
                self.inventory_scroll_y
                    - delta.clamp(
                        -USER_STORE_SCROLL_INPUT_CLAMP,
                        USER_STORE_SCROLL_INPUT_CLAMP,
                    ),
            );
        }
    }

    pub fn activate_inventory_row(
        &mut self,
        authority: &UserStoreAuthority0104,
        inventory_slot: i32,
        _button: UserStoreRowButton0104,
        outbox: &mut UserStoreUiOutbox0104,
    ) -> UserStoreActionOutcome0104 {
        if !self.input_boundary().controls_enabled {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::Busy);
        }
        if self.mode != UserStoreMode0104::MyStore {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::WrongMode);
        }
        if authority.store_open {
            outbox.push(UserStoreUiCommand0104::SystemMessage(
                USER_STORE_CANNOT_REGISTER.into(),
            ));
            return UserStoreActionOutcome0104::SystemMessage(USER_STORE_CANNOT_REGISTER);
        }
        let Some(slot) = inventory_slot_index(inventory_slot) else {
            return UserStoreActionOutcome0104::Blocked(
                UserStoreRequestBlock0104::InvalidInventorySlot,
            );
        };
        let item = authority.inventory[slot];
        if user_store_item_is_empty(item) {
            return UserStoreActionOutcome0104::Blocked(
                UserStoreRequestBlock0104::EmptyInventorySlot,
            );
        }
        if authority.first_empty_listing_slot().is_none() {
            outbox.push(UserStoreUiCommand0104::SystemMessage(
                USER_STORE_REGISTER_OVER_MAX.into(),
            ));
            return UserStoreActionOutcome0104::SystemMessage(USER_STORE_REGISTER_OVER_MAX);
        }
        let popup = UserStoreItemPopup0104 {
            kind: if item.item_type == USER_STORE_GENERAL_ITEM_TYPE {
                UserStorePopupKind0104::RegisterQuantity
            } else {
                UserStorePopupKind0104::RegisterPrice
            },
            slot_type: 1,
            slot: inventory_slot,
            item,
            maximum_quantity: if item.item_type == USER_STORE_GENERAL_ITEM_TYPE {
                item.option.max(0)
            } else {
                1
            },
            price: 0,
        };
        self.modal.item_popup = true;
        outbox.push(UserStoreUiCommand0104::PlayButtonSound);
        outbox.push(UserStoreUiCommand0104::OpenItemPopup(popup));
        UserStoreActionOutcome0104::Popup(popup)
    }

    pub fn commit_register_quantity(
        &mut self,
        authority: &UserStoreAuthority0104,
        popup: UserStoreItemPopup0104,
        quantity: i32,
        outbox: &mut UserStoreUiOutbox0104,
    ) -> UserStoreActionOutcome0104 {
        if popup.kind != UserStorePopupKind0104::RegisterQuantity
            || quantity <= 0
            || quantity > popup.maximum_quantity
        {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::InvalidQuantity);
        }
        self.modal.item_popup = false;
        let mut item = popup.item;
        item.option = quantity;
        let price_popup = UserStoreItemPopup0104 {
            kind: UserStorePopupKind0104::RegisterPrice,
            slot_type: 1,
            slot: popup.slot,
            item,
            maximum_quantity: quantity,
            price: 0,
        };
        if authority.first_empty_listing_slot().is_none() {
            outbox.push(UserStoreUiCommand0104::SystemMessage(
                USER_STORE_REGISTER_OVER_MAX.into(),
            ));
            return UserStoreActionOutcome0104::SystemMessage(USER_STORE_REGISTER_OVER_MAX);
        }
        self.modal.item_popup = true;
        outbox.push(UserStoreUiCommand0104::OpenItemPopup(price_popup));
        UserStoreActionOutcome0104::Popup(price_popup)
    }

    pub fn commit_register_price(
        &mut self,
        authority: &UserStoreAuthority0104,
        popup: UserStoreItemPopup0104,
        price: i32,
        outbox: &mut UserStoreUiOutbox0104,
    ) -> UserStoreActionOutcome0104 {
        // Clean `GumPopup.DrawStack` only raises the registration event when
        // both the selected quantity and the entered Taros value are > 0.
        if popup.kind != UserStorePopupKind0104::RegisterPrice || price <= 0 {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::InvalidPrice);
        }
        self.modal.item_popup = false;
        self.request_register(authority, popup.slot, popup.item, price, outbox)
    }

    pub fn request_register(
        &mut self,
        authority: &UserStoreAuthority0104,
        inventory_slot: i32,
        item: ItemBase0104,
        price: i32,
        outbox: &mut UserStoreUiOutbox0104,
    ) -> UserStoreActionOutcome0104 {
        if !self.active {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::Inactive);
        }
        if self.busy() {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::Busy);
        }
        if self.mode != UserStoreMode0104::MyStore {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::WrongMode);
        }
        if authority.store_open {
            outbox.push(UserStoreUiCommand0104::SystemMessage(
                USER_STORE_CANNOT_REGISTER.into(),
            ));
            return UserStoreActionOutcome0104::SystemMessage(USER_STORE_CANNOT_REGISTER);
        }
        if price < 0 {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::InvalidPrice);
        }
        let Some(slot) = inventory_slot_index(inventory_slot) else {
            return UserStoreActionOutcome0104::Blocked(
                UserStoreRequestBlock0104::InvalidInventorySlot,
            );
        };
        let source = authority.inventory[slot];
        if user_store_item_is_empty(source) {
            return UserStoreActionOutcome0104::Blocked(
                UserStoreRequestBlock0104::EmptyInventorySlot,
            );
        }
        if source.item_type != item.item_type
            || source.item_id != item.item_id
            || source.time_limit != item.time_limit
            || (item.item_type == USER_STORE_GENERAL_ITEM_TYPE
                && (item.option <= 0 || item.option > source.option))
            || (item.item_type != USER_STORE_GENERAL_ITEM_TYPE && item != source)
        {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::InvalidQuantity);
        }
        let Some(list_slot) = authority.first_empty_listing_slot() else {
            outbox.push(UserStoreUiCommand0104::SystemMessage(
                USER_STORE_REGISTER_OVER_MAX.into(),
            ));
            return UserStoreActionOutcome0104::SystemMessage(USER_STORE_REGISTER_OVER_MAX);
        };
        let packet = UserStorePacket0104::register_item(list_slot, inventory_slot, item, price);
        self.pending = Some(UserStorePendingRequest0104::Register {
            list_slot,
            inventory_slot,
            item,
            price,
        });
        outbox.push(UserStoreUiCommand0104::SendPacket(packet.clone()));
        UserStoreActionOutcome0104::Sent(packet)
    }

    pub fn activate_store_row(
        &mut self,
        authority: &UserStoreAuthority0104,
        list_slot: i32,
        _button: UserStoreRowButton0104,
        outbox: &mut UserStoreUiOutbox0104,
    ) -> UserStoreActionOutcome0104 {
        if !self.input_boundary().controls_enabled {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::Busy);
        }
        let Some(slot) = usize::try_from(list_slot)
            .ok()
            .filter(|slot| *slot < USER_STORE_LIST_CAPACITY)
        else {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::InvalidListSlot);
        };
        let Some(listing) = authority.listings[slot] else {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::EmptyListSlot);
        };
        if listing.price < 0 {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::Sold);
        }
        if self.mode == UserStoreMode0104::MyStore && authority.store_open {
            outbox.push(UserStoreUiCommand0104::SystemMessage(
                USER_STORE_CANNOT_UNREGISTER.into(),
            ));
            return UserStoreActionOutcome0104::SystemMessage(USER_STORE_CANNOT_UNREGISTER);
        }
        let popup = UserStoreItemPopup0104 {
            kind: match self.mode {
                UserStoreMode0104::MyStore => UserStorePopupKind0104::Unregister,
                UserStoreMode0104::UserStore => UserStorePopupKind0104::Buy,
                UserStoreMode0104::ReturnStore => {
                    return UserStoreActionOutcome0104::Blocked(
                        UserStoreRequestBlock0104::ReturnStoreUnreachable,
                    );
                }
            },
            slot_type: self.mode.listing_slot_type().expect("observed mode"),
            slot: list_slot,
            item: listing.item,
            // `GumPopup.ShowPopup` copies `mClickItem.iOpt` into `iMaxNums`
            // for the disabled listing calculator display.
            maximum_quantity: listing.item.option.max(0),
            price: listing.price,
        };
        self.modal.item_popup = true;
        outbox.push(UserStoreUiCommand0104::PlayButtonSound);
        outbox.push(UserStoreUiCommand0104::OpenItemPopup(popup));
        UserStoreActionOutcome0104::Popup(popup)
    }

    pub fn commit_store_popup(
        &mut self,
        authority: &UserStoreAuthority0104,
        popup: UserStoreItemPopup0104,
        outbox: &mut UserStoreUiOutbox0104,
    ) -> UserStoreActionOutcome0104 {
        self.modal.item_popup = false;
        match popup.kind {
            UserStorePopupKind0104::Unregister => {
                self.request_unregister(authority, popup.slot, outbox)
            }
            UserStorePopupKind0104::Buy => self.request_buy(authority, popup.slot, outbox),
            _ => UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::InvalidListSlot),
        }
    }

    pub fn request_unregister(
        &mut self,
        authority: &UserStoreAuthority0104,
        list_slot: i32,
        outbox: &mut UserStoreUiOutbox0104,
    ) -> UserStoreActionOutcome0104 {
        if !self.active {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::Inactive);
        }
        if self.busy() {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::Busy);
        }
        if self.mode != UserStoreMode0104::MyStore {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::WrongMode);
        }
        if authority.store_open {
            outbox.push(UserStoreUiCommand0104::SystemMessage(
                USER_STORE_CANNOT_UNREGISTER.into(),
            ));
            return UserStoreActionOutcome0104::SystemMessage(USER_STORE_CANNOT_UNREGISTER);
        }
        let Ok(slot) = list_slot_index(list_slot) else {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::InvalidListSlot);
        };
        if authority.listings[slot].is_none() {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::EmptyListSlot);
        }
        let packet = UserStorePacket0104::unregister_item(list_slot);
        self.pending = Some(UserStorePendingRequest0104::Unregister { list_slot });
        outbox.push(UserStoreUiCommand0104::SendPacket(packet.clone()));
        UserStoreActionOutcome0104::Sent(packet)
    }

    pub fn request_open_store(
        &mut self,
        authority: &UserStoreAuthority0104,
        outbox: &mut UserStoreUiOutbox0104,
    ) -> UserStoreActionOutcome0104 {
        if !self.active {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::Inactive);
        }
        if self.busy() {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::Busy);
        }
        if self.mode != UserStoreMode0104::MyStore {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::WrongMode);
        }
        if authority.store_open {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::StoreOpen);
        }
        if authority.occupied_listing_count() == 0 {
            return UserStoreActionOutcome0104::NoOp;
        }
        let packet = UserStorePacket0104::sale_start(authority.open_item_inventory_slot);
        self.pending = Some(UserStorePendingRequest0104::SaleStart {
            open_item_inventory_slot: authority.open_item_inventory_slot,
        });
        outbox.push(UserStoreUiCommand0104::SendPacket(packet.clone()));
        UserStoreActionOutcome0104::Sent(packet)
    }

    pub fn request_buy(
        &mut self,
        authority: &UserStoreAuthority0104,
        list_slot: i32,
        outbox: &mut UserStoreUiOutbox0104,
    ) -> UserStoreActionOutcome0104 {
        if !self.active {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::Inactive);
        }
        if self.busy() {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::Busy);
        }
        if self.mode != UserStoreMode0104::UserStore {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::WrongMode);
        }
        let Ok(slot) = list_slot_index(list_slot) else {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::InvalidListSlot);
        };
        let Some(listing) = authority.listings[slot] else {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::EmptyListSlot);
        };
        if listing.price < 0 {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::Sold);
        }
        if listing.price > authority.taros {
            return UserStoreActionOutcome0104::Blocked(
                UserStoreRequestBlock0104::InsufficientTaros,
            );
        }
        let Some(target_pc_id) = authority.target_pc_id else {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::WrongMode);
        };
        let Some(empty_inventory_slot) = authority.first_empty_inventory_slot() else {
            return UserStoreActionOutcome0104::Blocked(UserStoreRequestBlock0104::InventoryFull);
        };
        let packet = UserStorePacket0104::item_buy(target_pc_id, list_slot, empty_inventory_slot);
        self.pending = Some(UserStorePendingRequest0104::Buy {
            target_pc_id,
            list_slot,
            empty_inventory_slot,
        });
        outbox.push(UserStoreUiCommand0104::SendPacket(packet.clone()));
        UserStoreActionOutcome0104::Sent(packet)
    }

    pub fn click_primary_button(
        &mut self,
        authority: &UserStoreAuthority0104,
        outbox: &mut UserStoreUiOutbox0104,
    ) -> UserStoreActionOutcome0104 {
        if self.mode == UserStoreMode0104::MyStore && !authority.store_open {
            self.request_open_store(authority, outbox)
        } else {
            self.request_close(authority, false, outbox)
        }
    }

    /// Clean handler exists but its body is empty.
    #[must_use]
    pub const fn click_go_to_game(&self) -> UserStoreActionOutcome0104 {
        UserStoreActionOutcome0104::NoOp
    }

    pub fn request_close(
        &mut self,
        authority: &UserStoreAuthority0104,
        via_escape: bool,
        outbox: &mut UserStoreUiOutbox0104,
    ) -> UserStoreActionOutcome0104 {
        let boundary = self.input_boundary();
        let allowed = if via_escape {
            boundary.escape_enabled
        } else {
            boundary.close_enabled
        };
        if !allowed {
            return UserStoreActionOutcome0104::Blocked(if !self.active {
                UserStoreRequestBlock0104::Inactive
            } else {
                UserStoreRequestBlock0104::Busy
            });
        }
        match self.mode {
            UserStoreMode0104::UserStore => {
                self.finish_exit(outbox);
                UserStoreActionOutcome0104::LocalExit
            }
            UserStoreMode0104::MyStore => {
                let packet = UserStorePacket0104::cancel(authority.owner_pc_id);
                self.pending = Some(UserStorePendingRequest0104::Cancel {
                    owner_pc_id: authority.owner_pc_id,
                });
                outbox.push(UserStoreUiCommand0104::SendPacket(packet.clone()));
                UserStoreActionOutcome0104::Sent(packet)
            }
            UserStoreMode0104::ReturnStore => UserStoreActionOutcome0104::Blocked(
                UserStoreRequestBlock0104::ReturnStoreUnreachable,
            ),
        }
    }

    pub fn target_store_state_change(
        &mut self,
        authority: &UserStoreAuthority0104,
        target_pc_id: i32,
        state: i32,
        outbox: &mut UserStoreUiOutbox0104,
    ) -> bool {
        if self.active
            && self.mode == UserStoreMode0104::UserStore
            && authority.target_pc_id == Some(target_pc_id)
            && state == 0
        {
            outbox.push(UserStoreUiCommand0104::SystemMessage(
                USER_STORE_TARGET_CLOSED_MESSAGE_KEY.into(),
            ));
            self.finish_exit(outbox);
            true
        } else {
            false
        }
    }

    pub fn open_help(&mut self, outbox: &mut UserStoreUiOutbox0104) -> bool {
        if !self.input_boundary().controls_enabled {
            return false;
        }
        self.modal.help = true;
        outbox.push(UserStoreUiCommand0104::OpenHelp(USER_STORE_HELP_EVENT_ID));
        true
    }

    pub fn finish_exit(&mut self, outbox: &mut UserStoreUiOutbox0104) {
        let restore_locked = self.previous_cursor_locked;
        *self = Self {
            previous_cursor_locked: restore_locked,
            ..default()
        };
        outbox.push(UserStoreUiCommand0104::ResetInventoryUiMode);
        if restore_locked {
            outbox.push(UserStoreUiCommand0104::RestoreCursor);
        }
        outbox.push(UserStoreUiCommand0104::ChangeToGameplay);
        outbox.push(UserStoreUiCommand0104::StopUiModeSound);
        outbox.push(UserStoreUiCommand0104::FreeAssets);
    }

    pub fn receive_reply(
        &mut self,
        authority: &mut UserStoreAuthority0104,
        packet_id: u32,
        payload: &[u8],
        outbox: &mut UserStoreUiOutbox0104,
    ) -> Result<UserStoreReply0104, UserStoreReplyReject0104> {
        let reply = decode_user_store_reply_0104(packet_id, payload)
            .map_err(UserStoreReplyReject0104::Codec)?;
        if let UserStoreReply0104::BuySuccessSeller(seller) = reply {
            if !self.active || self.mode != UserStoreMode0104::MyStore {
                return Err(UserStoreReplyReject0104::NoPendingRequest { packet_id });
            }
            let mut transaction = authority.clone();
            transaction
                .apply_seller(seller)
                .map_err(UserStoreReplyReject0104::Apply)?;
            *authority = transaction;
            return Ok(UserStoreReply0104::BuySuccessSeller(seller));
        }

        let pending = self
            .pending
            .ok_or(UserStoreReplyReject0104::NoPendingRequest { packet_id })?;
        if packet_id != pending.expected_success_packet()
            && packet_id != pending.expected_failure_packet()
        {
            return Err(UserStoreReplyReject0104::UnexpectedForPending { packet_id, pending });
        }

        let failure = match reply {
            UserStoreReply0104::ReadyFail(code)
            | UserStoreReply0104::CancelFail(code)
            | UserStoreReply0104::RegisterFail(code)
            | UserStoreReply0104::UnregisterFail(code)
            | UserStoreReply0104::SaleStartFail(code)
            | UserStoreReply0104::ItemListFail(code)
            | UserStoreReply0104::BuyFail(code) => Some(code),
            _ => None,
        };
        if let Some(code) = failure {
            self.pending = None;
            self.last_error = Some(code);
            outbox.push(UserStoreUiCommand0104::ServerFailure(code));
            return Ok(reply);
        }

        let mut transaction = authority.clone();
        match (pending, &reply) {
            (
                UserStorePendingRequest0104::Ready {
                    open_item_inventory_slot,
                },
                UserStoreReply0104::ReadySuccess(success),
            ) if success.open_item_inventory_slot == open_item_inventory_slot => {
                transaction
                    .setup_ready(*success)
                    .map_err(UserStoreReplyReject0104::Apply)?;
                self.mode = UserStoreMode0104::MyStore;
            }
            (
                UserStorePendingRequest0104::Cancel { .. },
                UserStoreReply0104::CancelSuccess { .. },
            ) => {}
            (
                UserStorePendingRequest0104::Register {
                    list_slot,
                    inventory_slot,
                    item,
                    price,
                },
                UserStoreReply0104::RegisterSuccess(success),
            ) if success.list_slot == list_slot
                && success.inventory_slot == inventory_slot
                && success.item == item
                && success.price == price =>
            {
                transaction
                    .apply_register(*success)
                    .map_err(UserStoreReplyReject0104::Apply)?;
            }
            (
                UserStorePendingRequest0104::Unregister { list_slot },
                UserStoreReply0104::UnregisterSuccess {
                    list_slot: reply_slot,
                },
            ) if list_slot == *reply_slot => {
                transaction
                    .apply_unregister(*reply_slot)
                    .map_err(UserStoreReplyReject0104::Apply)?;
            }
            (
                UserStorePendingRequest0104::SaleStart {
                    open_item_inventory_slot,
                },
                UserStoreReply0104::SaleStartSuccess(success),
            ) if success.open_item_inventory_slot == open_item_inventory_slot => {
                transaction
                    .apply_sale_start(*success)
                    .map_err(UserStoreReplyReject0104::Apply)?;
            }
            (
                UserStorePendingRequest0104::ItemList { target_pc_id },
                UserStoreReply0104::ItemListSuccess(success),
            ) if success.target_pc_id == target_pc_id => {
                transaction
                    .setup_item_list(success)
                    .map_err(UserStoreReplyReject0104::Apply)?;
                self.mode = UserStoreMode0104::UserStore;
            }
            (
                UserStorePendingRequest0104::Buy {
                    target_pc_id,
                    list_slot,
                    empty_inventory_slot,
                },
                UserStoreReply0104::BuySuccessBuyer(success),
            ) if success.target_pc_id == target_pc_id
                && success.list_slot == list_slot
                && success.buyer_inventory_slot == empty_inventory_slot =>
            {
                transaction
                    .apply_buyer(*success)
                    .map_err(UserStoreReplyReject0104::Apply)?;
            }
            _ => {
                return Err(UserStoreReplyReject0104::CorrelationMismatch(
                    "reply fields do not match the request lock",
                ));
            }
        }

        let cancel_success = matches!(reply, UserStoreReply0104::CancelSuccess { .. });
        *authority = transaction;
        self.pending = None;
        self.last_error = None;
        if cancel_success {
            self.finish_exit(outbox);
        }
        Ok(reply)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserStoreItemMetadata0104 {
    pub name: String,
    pub level: i32,
    pub description: String,
    pub icon_path: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct UserStoreItemCatalog0104 {
    pub(super) entries: BTreeMap<(i16, i16), UserStoreItemMetadata0104>,
}

impl UserStoreItemCatalog0104 {
    pub fn insert(&mut self, item_type: i16, item_id: i16, metadata: UserStoreItemMetadata0104) {
        self.entries.insert((item_type, item_id), metadata);
    }

    #[must_use]
    pub fn resolve(&self, item: ItemBase0104) -> Option<&UserStoreItemMetadata0104> {
        self.entries.get(&(item.item_type, item.item_id))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UserStorePresentationIcon0104 {
    Empty,
    Resolved(String),
    MissingChecker,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserStoreProjectedItem0104 {
    pub item: ItemBase0104,
    pub name: String,
    pub level: i32,
    pub description: String,
    pub icon: UserStorePresentationIcon0104,
    pub count_label: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserStoreListingRowProjection0104 {
    pub visual_row: usize,
    pub list_slot: i32,
    pub slot_type: i32,
    pub item: Option<UserStoreProjectedItem0104>,
    pub price: Option<i32>,
    pub seller_display_taros: Option<i32>,
    pub sold: bool,
    pub affordable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct UserStoreUiProjection0104 {
    pub title: String,
    pub listing_rows: Vec<UserStoreListingRowProjection0104>,
    pub inventory: Vec<UserStoreInventoryProjection0104>,
    pub primary_button: String,
    pub show_go_to_game: bool,
    pub busy: bool,
    pub error: Option<i32>,
}

impl Default for UserStoreUiProjection0104 {
    fn default() -> Self {
        Self {
            title: USER_STORE_TITLE_SUFFIX_TYPO.into(),
            listing_rows: Vec::new(),
            inventory: Vec::new(),
            primary_button: "OPEN STORE".into(),
            show_go_to_game: false,
            busy: false,
            error: None,
        }
    }
}
