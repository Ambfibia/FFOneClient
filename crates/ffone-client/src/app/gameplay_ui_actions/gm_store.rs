use crate::app::*;
use ffone_client::user_store_runtime::prove_user_store_packet_0104;
use ffone_client::user_store_ui::*;

#[derive(SystemParam)]
pub(crate) struct Context<'w, 's> {
    production: ResMut<'w, UserStoreProductionRuntime0104>,
    state: ResMut<'w, UserStoreUiState0104>,
    authority: ResMut<'w, UserStoreAuthority0104>,
    popup: ResMut<'w, UserStorePopupPresentation0104>,
    outbox: ResMut<'w, UserStoreUiOutbox0104>,
    inventory: ResMut<'w, LocalInventoryRuntime>,
    runtime: ResMut<'w, RuntimeStatus>,
    bridge: Res<'w, NetworkBridge>,
    catalog: ResMut<'w, UserStoreItemCatalog0104>,
    content: Res<'w, TutorialMissionContent>,
    localization: Res<'w, Localization>,
    language: Res<'w, Language>,
    audio: ResMut<'w, GameplayAudioRuntime>,
    rule: ResMut<'w, RuleUiModel>,
    rules: ResMut<'w, RuleRuntime>,
    cursors: Query<'w, 's, &'static mut CursorOptions, With<PrimaryWindow>>,
}

fn report(c: &mut Context, error: impl ToString) {
    super::gm_runtime::feedback(
        &mut c.runtime,
        &c.localization,
        &c.language,
        LocalizedText::new(
            "ui.chat.command.gm.failed",
            "Could not send command: {error}",
        )
        .with_arg("error", error.to_string()),
    );
}

pub(crate) fn pump(c: &mut Context) {
    if let Some(target) = c.runtime.chat.gm.store_open.take() {
        if !c.state.active && c.runtime.user_level <= 50 && c.runtime.player_id.is_some() {
            // Admission occurs only after every packet of this server-owned
            // transaction family is registered. No preview authority is used.
            let admitted = ffone_client::user_store_runtime::USER_STORE_REQUEST_CAPABILITIES_0104
                .iter()
                .all(|entry| {
                    ffone_protocol::RegisteredGameplayRequest0104::new(
                        entry.packet_type,
                        vec![0; entry.payload_size],
                    )
                    .is_ok()
                });
            if !admitted {
                report(c, "Street-stall server handlers are not registered");
                return;
            }
            let Some(snapshot) = c.inventory.snapshot() else {
                report(c, "Inventory is not loaded");
                return;
            };
            let owner = c.runtime.player_id.unwrap();
            *c.authority = UserStoreAuthority0104::default();
            *c.catalog = UserStoreItemCatalog0104::default();
            c.authority.owner_pc_id = owner;
            c.authority.taros = c.runtime.candy;
            c.authority.inventory = *snapshot.inventory();
            c.authority
                .equipment
                .copy_from_slice(&snapshot.equipment()[..USER_STORE_EQUIPMENT_CAPACITY]);
            let locked = c
                .cursors
                .single()
                .is_ok_and(|cursor| cursor.grab_mode != CursorGrabMode::None);
            c.production.gm_owner = Some(owner);
            if target == 0 || target == owner {
                // The admin extension requires no shop consumable. Reserve an
                // empty UI slot as the existing READY protocol's owner index.
                let slot = c.authority.first_empty_inventory_slot().unwrap_or(49);
                c.state
                    .begin_my_store(&mut c.authority, owner, slot, locked, &mut c.outbox);
            } else {
                c.state
                    .begin_user_store(&mut c.authority, owner, target, locked, &mut c.outbox);
            }
        }
    }
    if c.production.gm_owner.is_none() {
        c.runtime.chat.gm.store_frames.clear();
        guard_user_store_production_boundary_0104(
            &mut c.production,
            &mut c.state,
            &mut c.popup,
            &mut c.outbox,
            &mut c.runtime,
        );
        return;
    }
    if c.production.gm_owner != c.runtime.player_id {
        c.production.reset();
        *c.state = default();
        *c.authority = default();
        *c.popup = default();
        c.outbox.clear();
        c.runtime.chat.gm.store_frames.clear();
        return;
    }
    while let Some(frame) = c.runtime.chat.gm.store_frames.pop_front() {
        match c.state.receive_reply(
            &mut c.authority,
            frame.packet_type,
            &frame.payload,
            &mut c.outbox,
        ) {
            Ok(UserStoreReply0104::BuySuccessBuyer(reply)) => {
                if let Some(snapshot) = c.inventory.snapshot_mut() {
                    if let Err(error) =
                        snapshot.apply_reward_item_post_state(ffone_protocol::ItemReward0104 {
                            inventory_location: 1,
                            slot: reply.buyer_inventory_slot,
                            item: reply.item,
                        })
                    {
                        report(c, error);
                    }
                }
                c.runtime.candy = reply.buyer_taros;
            }
            Ok(UserStoreReply0104::BuySuccessSeller(reply)) => c.runtime.candy = reply.seller_taros,
            Ok(UserStoreReply0104::ReadyFail(_) | UserStoreReply0104::ItemListFail(_)) => {
                c.state.finish_exit(&mut c.outbox)
            }
            Ok(_) => {}
            Err(error) => report(c, error),
        }
    }
    // Resolve names/descriptions from the normal keyed table projection.
    // Shop-local inventory includes reservations; never write it wholesale
    // into the authoritative inventory or infer a completed purchase from it.
    if c.state.active {
        if c.authority.taros != c.runtime.candy {
            c.authority.taros = c.runtime.candy;
        }
        if c.state.mode == UserStoreMode0104::UserStore && c.state.pending.is_none() {
            if let Some(snapshot) = c.inventory.snapshot() {
                if c.authority.inventory != *snapshot.inventory() {
                    c.authority.inventory = *snapshot.inventory();
                }
            }
        }
        let items = c
            .authority
            .inventory
            .iter()
            .copied()
            .chain(c.authority.equipment.iter().copied())
            .chain(
                c.authority
                    .listings
                    .iter()
                    .flatten()
                    .map(|listing| listing.item),
            )
            .collect::<Vec<_>>();
        for item in items {
            if item.item_id <= 0 {
                continue;
            }
            if c.catalog.resolve(item).is_some() && !c.language.is_changed() {
                continue;
            }
            if let Some((name, description)) = c
                .content
                .gameplay_user_equip_item_text(item.item_type, item.item_id)
            {
                let metadata = UserStoreItemMetadata0104 {
                    name: c.localization.text(&c.language, &name),
                    description: c.localization.text(&c.language, &description),
                    level: c
                        .content
                        .gameplay_user_equip_item_detail(item.item_type, item.item_id)
                        .map(|d| d.level)
                        .unwrap_or_default(),
                    icon_path: c
                        .content
                        .gameplay_item_display_icon(item)
                        .map(str::to_owned),
                };
                c.catalog.insert(item.item_type, item.item_id, metadata);
            }
        }
    }
    while let Some(command) = c.outbox.0.pop_front() {
        match command {
            UserStoreUiCommand0104::SendPacket(packet) => {
                match prove_user_store_packet_0104(&packet) {
                    Ok(packet) => {
                        if let Err(error) = c
                            .bridge
                            .send(NetworkCommand::SendRegisteredGameplay0104(packet))
                        {
                            report(c, error);
                        }
                    }
                    Err(error) => report(c, error),
                }
            }
            UserStoreUiCommand0104::OpenItemPopup(popup) => c.popup.open(popup),
            UserStoreUiCommand0104::OpenHelp(page) => {
                if let Err(error) = c.rules.open(
                    &mut c.rule,
                    ffone_client::rule_runtime::RuleOpenRequest::GmChatCommand {
                        user_level: i32::from(c.runtime.user_level),
                        table_index: page,
                    },
                ) {
                    report(c, format!("{error:?}"));
                }
            }
            UserStoreUiCommand0104::PlayButtonSound => c.audio.queue_legacy_button_sound(),
            UserStoreUiCommand0104::StartUiModeSound => c.audio.queue_user_equip_mode_edge(true),
            UserStoreUiCommand0104::StopUiModeSound => c.audio.queue_user_equip_mode_edge(false),
            UserStoreUiCommand0104::UnlockCursor => {
                for mut cursor in &mut c.cursors {
                    cursor.grab_mode = CursorGrabMode::None;
                    cursor.visible = true;
                }
            }
            UserStoreUiCommand0104::RestoreCursor => {
                for mut cursor in &mut c.cursors {
                    cursor.grab_mode = CursorGrabMode::Locked;
                    cursor.visible = false;
                }
            }
            UserStoreUiCommand0104::SystemMessage(message) => report(c, message),
            UserStoreUiCommand0104::ServerFailure(code) => {
                report(c, format!("Street-stall error {code}"))
            }
            UserStoreUiCommand0104::RejectedReply(error) => report(c, error),
            UserStoreUiCommand0104::ChangeToGameplay => {
                c.production.gm_owner = None;
                c.popup.close();
            }
            // The native store owns both panels in its existing UI root.
            UserStoreUiCommand0104::ActivateStoreInventoryMode
            | UserStoreUiCommand0104::ResetInventoryUiMode
            | UserStoreUiCommand0104::ActivateSharedPanels
            | UserStoreUiCommand0104::FreeAssets => {}
        }
    }
}
