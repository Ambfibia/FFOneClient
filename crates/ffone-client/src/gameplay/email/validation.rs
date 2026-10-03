use super::*;

pub(super) fn validate_player_authority(
    player: EmailPlayerAuthority0104,
    inventory: &EmailInventoryAuthority0104,
) -> Result<(), EmailProductionError0104> {
    if inventory.owner_pc_id != player.owner_pc_id {
        return Err(EmailProductionError0104::InventoryOwnerMismatch {
            expected_pc_id: player.owner_pc_id,
            inventory_pc_id: inventory.owner_pc_id,
        });
    }
    if player.taros < 0 {
        return Err(EmailProductionError0104::NegativeTaros {
            taros: player.taros,
        });
    }
    Ok(())
}

pub(super) fn validate_staged_attachments_unchanged(
    model: &EmailUiModel,
    inventory: &EmailInventoryAuthority0104,
) -> Result<(), EmailProductionError0104> {
    for attachment in model.draft.attachments.iter().flatten() {
        let Ok(inventory_slot) = usize::try_from(attachment.inventory_slot) else {
            return Err(EmailProductionError0104::RequestNotReachable {
                request: EmailRequest::Send {
                    recipient_pc_uid: model.draft.recipient_pc_uid,
                    subject: model.draft.subject.clone(),
                    content: model.draft.content.clone(),
                    items: array::from_fn(|index| {
                        model.draft.attachments[index].unwrap_or_default()
                    }),
                    cash: model.draft.cash,
                },
            });
        };
        let actual = inventory.slot(inventory_slot).unwrap_or(ItemBase0104 {
            item_type: 0,
            item_id: 0,
            option: 0,
            time_limit: 0,
        });
        if email_item_from_base(actual) != attachment.item {
            return Err(EmailProductionError0104::AttachmentSourceChanged {
                inventory_slot,
                expected: attachment.item,
                actual,
            });
        }
    }
    Ok(())
}

pub(super) fn validate_request_reachable(
    session: &EmailProductionSession0104,
    model: &EmailUiModel,
    request: &EmailRequest,
    catalog: &impl EmailItemCatalog0104,
) -> Result<(), EmailProductionError0104> {
    if !model.visible {
        return Err(EmailProductionError0104::RequestNotReachable {
            request: request.clone(),
        });
    }
    match request {
        EmailRequest::UpdateCheck => Ok(()),
        EmailRequest::PageList { page }
            if model.folder == EmailFolder::Player && model.player_page == *page =>
        {
            Ok(())
        }
        EmailRequest::Read { email_index }
            if model.folder == EmailFolder::Player
                && model
                    .selected_summary()
                    .is_some_and(|summary| summary.email_index == *email_index) =>
        {
            Ok(())
        }
        EmailRequest::Delete { email_indices }
            if model.folder == EmailFolder::Player
                && email_indices[1..].iter().all(|index| *index == 0)
                && model
                    .read_message
                    .as_ref()
                    .is_some_and(|read| read.email_index == email_indices[0]) =>
        {
            Ok(())
        }
        EmailRequest::ReceiveCash { email_index }
            if model.folder == EmailFolder::Player
                && model
                    .read_message
                    .as_ref()
                    .is_some_and(|read| read.email_index == *email_index && read.cash > 0) =>
        {
            Ok(())
        }
        EmailRequest::ReceiveItem {
            email_index,
            inventory_slot,
            email_item_slot,
        } => {
            require_fresh_inventory(session)?;
            let Ok(inventory_slot_index) = usize::try_from(*inventory_slot) else {
                return request_unreachable(request);
            };
            let Ok(email_item_index) = usize::try_from(*email_item_slot - 1) else {
                return request_unreachable(request);
            };
            let Some(read) = model.read_message.as_ref() else {
                return request_unreachable(request);
            };
            if model.folder != EmailFolder::Player
                || read.email_index != *email_index
                || email_item_index >= EMAIL_ATTACHMENT_COUNT
                || read.items[email_item_index].is_empty()
                || session
                    .inventory
                    .slot(inventory_slot_index)
                    .is_none_or(|item| !InventoryRuntime0104::item_is_empty(item))
            {
                return request_unreachable(request);
            }
            Ok(())
        }
        EmailRequest::ReceiveAllItems { email_index } => {
            require_fresh_inventory(session)?;
            let Some(read) = model.read_message.as_ref() else {
                return request_unreachable(request);
            };
            let item_count = read.items.iter().filter(|item| !item.is_empty()).count();
            if model.folder != EmailFolder::Player
                || read.email_index != *email_index
                || item_count == 0
                || session.inventory.free_slot_count() < item_count
            {
                return request_unreachable(request);
            }
            Ok(())
        }
        EmailRequest::Send { .. } => {
            require_fresh_inventory(session)?;
            if model.screen != EmailScreen::Compose {
                return request_unreachable(request);
            }
            if model.available_cash != session.context.player.taros {
                return Err(EmailProductionError0104::TarosProjectionMismatch {
                    authoritative: session.context.player.taros,
                    projected: model.available_cash,
                });
            }
            let expected = model
                .draft
                .request(session.context.player.taros)
                .map_err(|_| EmailProductionError0104::RequestNotReachable {
                    request: request.clone(),
                })?;
            if &expected != request {
                return request_unreachable(request);
            }
            validate_send_attachments(session, request, catalog)
        }
        _ => request_unreachable(request),
    }
}

pub(super) fn validate_send_attachments(
    session: &EmailProductionSession0104,
    request: &EmailRequest,
    catalog: &impl EmailItemCatalog0104,
) -> Result<(), EmailProductionError0104> {
    let EmailRequest::Send { items, .. } = request else {
        return request_unreachable(request);
    };
    let mut seen = [false; EMAIL_INVENTORY_SLOT_COUNT];
    for outgoing in items {
        if outgoing.item.is_empty() {
            if *outgoing != EmailOutgoingItem::default() {
                return request_unreachable(request);
            }
            continue;
        }
        let Ok(inventory_slot) = usize::try_from(outgoing.inventory_slot) else {
            return request_unreachable(request);
        };
        let Some(actual) = session.inventory.slot(inventory_slot) else {
            return request_unreachable(request);
        };
        if seen[inventory_slot] || email_item_from_base(actual) != outgoing.item {
            return request_unreachable(request);
        }
        seen[inventory_slot] = true;
        validate_attachment_eligibility(
            inventory_slot,
            actual,
            session.context.item_policy,
            catalog,
        )?;
    }
    Ok(())
}

pub(super) fn validate_attachment_eligibility(
    inventory_slot: usize,
    item: ItemBase0104,
    policy: EmailItemFeaturePolicy0104,
    catalog: &impl EmailItemCatalog0104,
) -> Result<(), EmailProductionError0104> {
    if item.item_type < 0 || item.item_id <= 0 {
        return Err(EmailProductionError0104::MalformedInventoryItem {
            inventory_slot,
            item,
        });
    }
    if item.item_type == 9 {
        return Err(EmailProductionError0104::AttachmentRejected {
            inventory_slot,
            item,
            reason: EmailAttachmentRejection0104::Chest,
        });
    }
    if item.item_type != 7 && !(0..7).contains(&item.item_type) {
        return Err(EmailProductionError0104::AttachmentRejected {
            inventory_slot,
            item,
            reason: EmailAttachmentRejection0104::UnsupportedItemType,
        });
    }
    let metadata = catalog
        .resolve(item)
        .ok_or(EmailProductionError0104::MissingCatalogItem {
            inventory_slot,
            item,
        })?;
    let mut tradeable =
        metadata
            .tradeable
            .ok_or(EmailProductionError0104::MissingTradeMetadata {
                inventory_slot,
                item,
            })?;
    if item.item_type == 7 {
        let subtype =
            metadata
                .general_item_type
                .ok_or(EmailProductionError0104::MissingGeneralSubtype {
                    inventory_slot,
                    item,
                })?;
        if subtype == 3 {
            return Err(EmailProductionError0104::AttachmentRejected {
                inventory_slot,
                item,
                reason: EmailAttachmentRejection0104::GeneralEmailSubtype,
            });
        }
    } else {
        if policy.combine_enabled && ((item.option as u32 >> 16) & 0xffff) > 0 {
            tradeable = false;
        }
        // This clean branch occurs after the combine branch and deliberately
        // overrides it when both replacement-era feature flags are enabled.
        if policy.korean_enchant_enabled && (item.option as u32 & 0xffff) > 0 {
            tradeable = true;
        }
        if !tradeable && policy.combine_enabled && ((item.option as u32 >> 16) & 0xffff) > 0 {
            return Err(EmailProductionError0104::AttachmentRejected {
                inventory_slot,
                item,
                reason: EmailAttachmentRejection0104::CombinedLook,
            });
        }
    }
    if !tradeable {
        return Err(EmailProductionError0104::AttachmentRejected {
            inventory_slot,
            item,
            reason: EmailAttachmentRejection0104::NotTradeable,
        });
    }
    Ok(())
}

pub(super) fn validate_reply_against_request(
    pending: Option<&EmailRequest>,
    reply: &EmailReply,
) -> Result<(), EmailProductionError0104> {
    if matches!(reply, EmailReply::NewEmail { .. }) {
        if pending.is_none() || matches!(pending, Some(EmailRequest::UpdateCheck)) {
            return Ok(());
        }
        // New-email is an unsolicited push while any other request remains
        // pending; it does not participate in that correlation.
        return Ok(());
    }
    let request = pending.ok_or(EmailProductionError0104::NoPendingRequest)?;
    let matches = match (request, reply) {
        (
            EmailRequest::Read { email_index },
            EmailReply::ReadSuccess(EmailReadMessage {
                email_index: actual,
                ..
            })
            | EmailReply::ReadFailure {
                email_index: actual,
                ..
            },
        ) => email_index == actual,
        (
            EmailRequest::PageList { page },
            EmailReply::PageListSuccess { page: actual, .. }
            | EmailReply::PageListFailure { page: actual, .. },
        ) => page == actual,
        (
            EmailRequest::Delete { email_indices },
            EmailReply::DeleteSuccess {
                email_indices: actual,
            }
            | EmailReply::DeleteFailure {
                email_indices: actual,
                ..
            },
        ) => email_indices == actual,
        (
            EmailRequest::Send {
                recipient_pc_uid,
                items,
                ..
            },
            EmailReply::SendSuccess {
                recipient_pc_uid: actual_recipient,
                items: actual_items,
                ..
            },
        ) => recipient_pc_uid == actual_recipient && items == actual_items,
        (
            EmailRequest::Send {
                recipient_pc_uid, ..
            },
            EmailReply::SendFailure {
                recipient_pc_uid: actual,
                ..
            },
        ) => recipient_pc_uid == actual,
        (
            EmailRequest::ReceiveItem {
                email_index,
                inventory_slot,
                email_item_slot,
            },
            EmailReply::ReceiveItemSuccess {
                email_index: actual_email,
                inventory_slot: actual_inventory,
                email_item_slot: actual_item,
            }
            | EmailReply::ReceiveItemFailure {
                email_index: actual_email,
                inventory_slot: actual_inventory,
                email_item_slot: actual_item,
                ..
            },
        ) => {
            email_index == actual_email
                && inventory_slot == actual_inventory
                && email_item_slot == actual_item
        }
        (
            EmailRequest::ReceiveCash { email_index },
            EmailReply::ReceiveCashSuccess {
                email_index: actual,
                ..
            }
            | EmailReply::ReceiveCashFailure {
                email_index: actual,
                ..
            },
        ) => email_index == actual,
        (
            EmailRequest::ReceiveAllItems { email_index },
            EmailReply::ReceiveAllItemsSuccess {
                email_index: actual,
            }
            | EmailReply::ReceiveAllItemsFailure {
                email_index: actual,
                ..
            },
        ) => email_index == actual,
        _ => false,
    };
    if matches {
        Ok(())
    } else {
        Err(EmailProductionError0104::ReplyDoesNotMatchRequest {
            request: request.clone(),
            reply: reply.clone(),
        })
    }
}

pub(super) fn require_fresh_inventory(
    session: &EmailProductionSession0104,
) -> Result<(), EmailProductionError0104> {
    if session.inventory_projection_stale {
        Err(EmailProductionError0104::InventoryAuthorityStale)
    } else {
        Ok(())
    }
}

pub(super) fn require_non_negative_authoritative_taros(taros: i32) -> Result<(), EmailProductionError0104> {
    if taros < 0 {
        Err(EmailProductionError0104::NegativeAuthoritativeTaros { taros })
    } else {
        Ok(())
    }
}

pub(super) fn require_size(packet_id: u32, body: &[u8], expected: usize) -> Result<(), EmailDecodeError0104> {
    if body.len() == expected {
        Ok(())
    } else {
        Err(EmailDecodeError0104::UnexpectedBodySize {
            packet_id,
            expected,
            actual: body.len(),
        })
    }
}
