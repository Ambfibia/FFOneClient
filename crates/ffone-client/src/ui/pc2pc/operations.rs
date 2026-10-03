use super::*;

pub(super) fn correlate_local_offer_stay(
    local_offer: &[Option<Pc2pcTradeItem0104>; PC2PC_OFFER_SLOT_COUNT],
    item_stay: &[Pc2pcTradeItem0104; PC2PC_PROTOCOL_TRADE_ITEM_COUNT],
) -> Result<(), Pc2pcAuthorityError0104> {
    for (slot, local) in local_offer.iter().copied().enumerate() {
        let Some(local) = local else {
            continue;
        };
        let matched = item_stay.iter().copied().any(|candidate| {
            !candidate.is_empty()
                && candidate.offer_slot == slot as i32
                && candidate.item_type == local.item_type
                && candidate.item_id == local.item_id
                && candidate.option == local.option
        });
        if !matched {
            return Err(Pc2pcAuthorityError0104::FinalStayMissingLocalOffer {
                slot,
                expected: local,
            });
        }
    }
    for candidate in item_stay.iter().copied().filter(|item| !item.is_empty()) {
        if !(0..PC2PC_OFFER_SLOT_COUNT as i32).contains(&candidate.offer_slot) {
            return Err(Pc2pcAuthorityError0104::TradeItem(
                Pc2pcTradeItemError0104::OfferSlotOutOfBounds {
                    offer_slot: candidate.offer_slot,
                },
            ));
        }
        let expected = local_offer[candidate.offer_slot as usize];
        let Some(expected) = expected else {
            return Err(Pc2pcAuthorityError0104::UnexpectedFinalStay { actual: candidate });
        };
        if candidate.item_type != expected.item_type
            || candidate.item_id != expected.item_id
            || candidate.option != expected.option
        {
            return Err(Pc2pcAuthorityError0104::UnexpectedFinalStay { actual: candidate });
        }
    }
    Ok(())
}

pub(super) fn take_local_pending_confirm(
    state: &mut Pc2pcUiState,
    envelope: Pc2pcEnvelope0104,
) -> Result<(), Pc2pcCorrelationError0104> {
    match pending_request(state)? {
        Pc2pcPendingRequest0104::Confirm(expected) if expected.envelope == envelope => {
            state.pending = None;
            Ok(())
        }
        pending => Err(Pc2pcCorrelationError0104::PendingKindMismatch {
            pending,
            outcome: Pc2pcRequestFailureKind0104::Confirm,
        }),
    }
}

pub(super) fn take_matching_failure_pending(
    state: &mut Pc2pcUiState,
    envelope: Pc2pcEnvelope0104,
    kind: Pc2pcRequestFailureKind0104,
) -> Result<(), Pc2pcCorrelationError0104> {
    let pending = pending_request(state)?;
    let matches = match (&pending, kind) {
        (
            Pc2pcPendingRequest0104::RegisterItem(value),
            Pc2pcRequestFailureKind0104::RegisterItem,
        ) => value.envelope == envelope,
        (Pc2pcPendingRequest0104::Cancel(value), Pc2pcRequestFailureKind0104::Cancel) => {
            value.envelope == envelope
        }
        (
            Pc2pcPendingRequest0104::UnregisterItem(value),
            Pc2pcRequestFailureKind0104::UnregisterItem,
        ) => value.envelope == envelope,
        (
            Pc2pcPendingRequest0104::RegisterCash(value),
            Pc2pcRequestFailureKind0104::RegisterCash,
        ) => value.envelope == envelope,
        (Pc2pcPendingRequest0104::Confirm(value), Pc2pcRequestFailureKind0104::Confirm) => {
            value.envelope == envelope
        }
        (Pc2pcPendingRequest0104::Chat(value), Pc2pcRequestFailureKind0104::Chat) => {
            value.envelope == envelope
        }
        _ => false,
    };
    if matches {
        state.pending = None;
        Ok(())
    } else {
        Err(Pc2pcCorrelationError0104::PendingKindMismatch {
            pending,
            outcome: kind,
        })
    }
}

pub(super) fn pc2pc_fallback_text(localized: &LocalizedText) -> String {
    let mut value = localized.fallback.clone();
    for (name, replacement) in &localized.args {
        value = value.replace(&format!("{{{name}}}"), replacement);
    }
    value
}

pub(super) fn pc2pc_passthrough_text(value: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", value)
}

pub(super) fn pc2pc_count_text(value: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.inventory.item.count", "{count}").with_arg("count", value)
}

pub(super) fn pc2pc_taros_amount_text(amount: i32) -> LocalizedText {
    LocalizedText::new("ui.pc2pc.taros.amount", "{amount}").with_arg("amount", amount.to_string())
}

pub(super) fn pc2pc_remote_offer_text(name: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.pc2pc.offer.remote", "{name}'S OFFER").with_arg("name", name)
}

pub(super) fn pc2pc_chat_log_text(chat: &VecDeque<Pc2pcChatLine0104>) -> LocalizedText {
    let log = chat
        .iter()
        .map(|line| format!("{}: {}", line.display_name, line.text))
        .collect::<Vec<_>>()
        .join("\n");
    LocalizedText::new("ui.pc2pc.chat.log", "{log}").with_arg("log", log)
}

pub(super) fn equipment_slot_label(visual_index: usize) -> String {
    let spec = USER_EQUIP_EQUIPMENT_STRIP_ORDER[visual_index];
    match spec.label_ordinal {
        Some(ordinal) => format!("{} {ordinal}", spec.label_key),
        None => spec.label_key.to_owned(),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn bind_pc2pc_ui(
    asset_server: Res<AssetServer>,
    assets: Res<Pc2pcUiAssets>,
    mut readiness: ResMut<Pc2pcStaticAssetReadiness>,
    model: Res<Pc2pcUiModel0104>,
    modal: Res<Pc2pcModalState>,
    capabilities: Res<Pc2pcBackendCapabilities>,
    portraits: Res<Pc2pcPortraitBindings>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut roots: Query<(&mut Node, &mut Visibility), With<Pc2pcUiRoot>>,
    mut elements: Query<
        (
            &Pc2pcUiElement,
            &mut Node,
            Option<&mut ImageNode>,
            Option<&mut LocalizedText>,
            Option<&mut Visibility>,
        ),
        Without<Pc2pcUiRoot>,
    >,
) {
    *readiness = assets.readiness(&asset_server);
    let Ok(window) = windows.single() else {
        for (_, mut visibility) in &mut roots {
            *visibility = Visibility::Hidden;
        }
        return;
    };
    let render = model.state.phase.renders_shell()
        && model.snapshot.is_some()
        && *readiness == Pc2pcStaticAssetReadiness::Ready;
    for (mut node, mut visibility) in &mut roots {
        node.width = px(window.width());
        node.height = px(window.height());
        *visibility = if render {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if !render {
        return;
    }

    let layout = pc2pc_mode_layout(
        window.width().max(0.0) as u32,
        window.height().max(0.0) as u32,
        model.state.opening_elapsed,
        0.0,
    );
    let local_visual_ready = model.state.local_visual_ready();
    let remote_ready = model.state.remote_ready;
    let main_button_enabled = model.state.button_enabled(*modal);

    for (element, mut node, image, text, visibility) in &mut elements {
        match *element {
            Pc2pcUiElement::Backdrop => bind_rect(&mut node, layout.full_backdrop),
            Pc2pcUiElement::TradeBackplate => bind_rect(&mut node, layout.trade_backplate),
            Pc2pcUiElement::RightBackplate => bind_rect(&mut node, layout.right_backplate),
            Pc2pcUiElement::TradePanel => bind_rect(&mut node, layout.trade_panel),
            Pc2pcUiElement::PcStuffPanel => {
                bind_rect(&mut node, layout.item_mode.pc_stuff_panel.into())
            }
            Pc2pcUiElement::EquipmentPanel => {
                bind_rect(&mut node, layout.item_mode.equipment_panel.into());
            }
            Pc2pcUiElement::InventoryContent => {
                node.left = px(0);
                node.top = px(-layout.item_mode.scroll_y);
                node.width = px(USER_EQUIP_INVENTORY_CONTENT_WIDTH);
                node.height = px(USER_EQUIP_INVENTORY_CONTENT_HEIGHT);
            }
            Pc2pcUiElement::LocalOfferBox => {
                if let Some(mut image) = image {
                    image.image = assets.image(if local_visual_ready {
                        Pc2pcStaticAssetRole::LocalOfferReady
                    } else {
                        Pc2pcStaticAssetRole::LocalOffer
                    });
                }
            }
            Pc2pcUiElement::RemoteOfferBox => {
                if let Some(mut image) = image {
                    image.image = assets.image(if remote_ready {
                        Pc2pcStaticAssetRole::RemoteOfferReady
                    } else {
                        Pc2pcStaticAssetRole::RemoteOffer
                    });
                }
            }
            Pc2pcUiElement::LocalOfferFrame(slot) => {
                bind_offer_frame(image, &model.projection.local_offer[slot], &assets);
            }
            Pc2pcUiElement::RemoteOfferFrame(slot) => {
                bind_offer_frame(image, &model.projection.remote_offer[slot], &assets);
            }
            Pc2pcUiElement::LocalOfferIcon(slot) => bind_presentation_icon(
                &mut node,
                image,
                &presentation_icon(&model.projection.local_offer[slot].icon),
                &asset_server,
                &assets,
            ),
            Pc2pcUiElement::RemoteOfferIcon(slot) => bind_presentation_icon(
                &mut node,
                image,
                &presentation_icon(&model.projection.remote_offer[slot].icon),
                &asset_server,
                &assets,
            ),
            Pc2pcUiElement::LocalOfferCount(slot) => bind_count_text(
                &mut node,
                text,
                model.projection.local_offer[slot].count_label.as_deref(),
            ),
            Pc2pcUiElement::RemoteOfferCount(slot) => bind_count_text(
                &mut node,
                text,
                model.projection.remote_offer[slot].count_label.as_deref(),
            ),
            Pc2pcUiElement::LocalMoneyText => bind_localized_text(
                text,
                pc2pc_taros_amount_text(model.projection.local_offer_taros),
            ),
            Pc2pcUiElement::RemoteMoneyText => bind_localized_text(
                text,
                pc2pc_taros_amount_text(model.projection.remote_offer_taros),
            ),
            Pc2pcUiElement::LocalTitle => {
                bind_localized_text(text, LocalizedText::new("ui.pc2pc.offer.local", "MY OFFER"))
            }
            Pc2pcUiElement::RemoteTitle => bind_localized_text(
                text,
                pc2pc_remote_offer_text(model.projection.remote_name.to_uppercase()),
            ),
            Pc2pcUiElement::MainButton => {
                if let Some(mut image) = image {
                    image.image = assets.image(Pc2pcStaticAssetRole::Button);
                    let pulse = if remote_ready {
                        1.0 - (std::f32::consts::FRAC_PI_2 * model.state.accept_pulse).sin()
                    } else {
                        1.0
                    };
                    image.color = if main_button_enabled {
                        Color::srgba(1.0, 1.0, 1.0, pulse.max(0.28))
                    } else {
                        Color::srgba(0.55, 0.55, 0.55, 0.82)
                    };
                }
            }
            Pc2pcUiElement::MainButtonText => {
                bind_localized_text(text, model.state.button_label().localized_text())
            }
            Pc2pcUiElement::ReadyName => {
                node.display = display_if(local_visual_ready || remote_ready);
            }
            Pc2pcUiElement::ReadyWaitingPrefix => {
                node.display = display_if(local_visual_ready);
            }
            Pc2pcUiElement::ReadyPlayerName => {
                node.display = display_if(local_visual_ready || remote_ready);
                bind_localized_text(
                    text,
                    pc2pc_passthrough_text(model.projection.remote_name.clone()),
                );
            }
            Pc2pcUiElement::ReadySubject => {
                if local_visual_ready {
                    node.display = Display::Flex;
                    bind_localized_text(
                        text,
                        LocalizedText::new(
                            "ui.pc2pc.ready.local_subject",
                            " to accept your offer.",
                        ),
                    );
                } else if remote_ready {
                    node.display = Display::Flex;
                    bind_localized_text(
                        text,
                        LocalizedText::new(
                            "ui.pc2pc.ready.remote_subject",
                            "WOULD LIKE YOU TO ACCEPT THIS TRADE.",
                        ),
                    );
                } else {
                    node.display = Display::None;
                }
            }
            Pc2pcUiElement::ChatList => {
                node.top = px(PC2PC_CHAT_LIST_VIEW_RECT.top - model.state.chat_scroll_y);
                bind_localized_text(text, pc2pc_chat_log_text(&model.chat));
            }
            Pc2pcUiElement::LocalPortrait => bind_portrait(
                &mut node,
                image,
                capabilities.portrait_backend,
                portraits.local.as_ref(),
                &asset_server,
            ),
            Pc2pcUiElement::RemotePortrait => bind_portrait(
                &mut node,
                image,
                capabilities.portrait_backend,
                portraits.remote.as_ref(),
                &asset_server,
            ),
            Pc2pcUiElement::LocalFreeChat => {
                if let Some(mut visibility) = visibility {
                    *visibility = if capabilities.portrait_backend
                        && portraits.local_free_chat_allowed == Some(true)
                    {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    };
                }
            }
            Pc2pcUiElement::RemoteFreeChat => {
                if let Some(mut visibility) = visibility {
                    *visibility = if capabilities.portrait_backend
                        && portraits.remote_free_chat_allowed == Some(true)
                    {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    };
                }
            }
            Pc2pcUiElement::InventorySlotFrame(slot) => {
                if let Some(mut image) = image {
                    image.image =
                        assets.image(if model.projection.item_mode.inventory[slot].item.empty {
                            Pc2pcStaticAssetRole::SlotEmpty
                        } else {
                            Pc2pcStaticAssetRole::SlotOccupied
                        });
                }
            }
            Pc2pcUiElement::InventorySlotIcon(slot) => bind_presentation_icon(
                &mut node,
                image,
                &presentation_icon(&model.projection.item_mode.inventory[slot].item.icon),
                &asset_server,
                &assets,
            ),
            Pc2pcUiElement::InventorySlotBadge(slot) => {
                node.display = display_if(
                    model.projection.item_mode.inventory[slot]
                        .item
                        .show_combined_badge,
                );
            }
            Pc2pcUiElement::InventorySlotCount(slot) => {
                let item = &model.projection.item_mode.inventory[slot].item;
                let label =
                    (!item.empty && item.item.item_type == 7).then(|| item.item.option.to_string());
                bind_count_text(&mut node, text, label.as_deref());
            }
            Pc2pcUiElement::EquipmentSlotFrame(slot) => {
                if let Some(mut image) = image {
                    image.image =
                        assets.image(if model.projection.item_mode.equipment[slot].item.empty {
                            Pc2pcStaticAssetRole::SlotEmpty
                        } else {
                            Pc2pcStaticAssetRole::SlotOccupied
                        });
                }
            }
            Pc2pcUiElement::EquipmentSlotIcon(slot) => bind_presentation_icon(
                &mut node,
                image,
                &presentation_icon(&model.projection.item_mode.equipment[slot].item.icon),
                &asset_server,
                &assets,
            ),
            Pc2pcUiElement::EquipmentSlotBadge(slot) => {
                node.display = display_if(
                    model.projection.item_mode.equipment[slot]
                        .item
                        .show_combined_badge,
                );
            }
            Pc2pcUiElement::TradeArea
            | Pc2pcUiElement::LocalMoney
            | Pc2pcUiElement::RemoteMoney
            | Pc2pcUiElement::LocalTaros
            | Pc2pcUiElement::RemoteTaros
            | Pc2pcUiElement::AddTaros
            | Pc2pcUiElement::ChatBox
            | Pc2pcUiElement::ChatInput
            | Pc2pcUiElement::ChatSend
            | Pc2pcUiElement::InventoryViewport
            | Pc2pcUiElement::ItemTabLabel
            | Pc2pcUiElement::NanoTab
            | Pc2pcUiElement::NanoTabLabel
            | Pc2pcUiElement::Close
            | Pc2pcUiElement::Trash
            | Pc2pcUiElement::Help
            | Pc2pcUiElement::EquipmentContent
            | Pc2pcUiElement::EquipmentTitle
            | Pc2pcUiElement::EquipmentTitleLabel
            | Pc2pcUiElement::EquipmentSlotLabel(_) => {}
        }
    }
}

pub(super) fn presentation_icon(icon: &UserEquipProjectedIcon) -> UserEquipPresentationIcon {
    match icon {
        UserEquipProjectedIcon::Empty => UserEquipPresentationIcon::Empty,
        UserEquipProjectedIcon::Resolved(icon) => {
            UserEquipPresentationIcon::Resolved(icon.runtime_path().to_owned())
        }
        UserEquipProjectedIcon::MissingChecker(_) => UserEquipPresentationIcon::MissingChecker,
    }
}

pub(super) fn bind_portrait(
    node: &mut Node,
    image: Option<Mut<ImageNode>>,
    backend_available: bool,
    portrait: Option<&Pc2pcPortraitRef>,
    asset_server: &AssetServer,
) {
    let Some(mut image) = image else {
        return;
    };
    image.image = Handle::default();
    image.color = Color::NONE;
    if !backend_available {
        node.display = Display::Flex;
        return;
    }
    let Some(portrait) = portrait else {
        node.display = Display::Flex;
        return;
    };
    let handle = asset_server.load::<Image>(portrait.runtime_path().to_owned());
    if matches!(asset_server.load_state(handle.id()), LoadState::Loaded) {
        image.image = handle;
        image.color = Color::WHITE;
    }
    node.display = Display::Flex;
}

pub(super) fn bind_presentation_icon(
    node: &mut Node,
    image: Option<Mut<ImageNode>>,
    icon: &UserEquipPresentationIcon,
    asset_server: &AssetServer,
    assets: &Pc2pcUiAssets,
) {
    let Some(mut image) = image else {
        return;
    };
    match icon {
        UserEquipPresentationIcon::Empty => {
            node.display = Display::None;
            image.image = Handle::default();
        }
        UserEquipPresentationIcon::MissingChecker => {
            node.display = Display::Flex;
            image.image = assets.missing_checker.clone();
        }
        UserEquipPresentationIcon::Resolved(path) => {
            let handle = asset_server.load::<Image>(path.clone());
            node.display = Display::Flex;
            image.image = if matches!(asset_server.load_state(handle.id()), LoadState::Loaded) {
                handle
            } else {
                assets.missing_checker.clone()
            };
        }
    }
}

pub(super) fn bind_count_text(node: &mut Node, text: Option<Mut<LocalizedText>>, value: Option<&str>) {
    if let Some(value) = value {
        node.display = Display::Flex;
        bind_localized_text(text, pc2pc_count_text(value));
    } else {
        node.display = Display::None;
        bind_localized_text(text, pc2pc_count_text(""));
    }
}
