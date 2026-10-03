use super::*;

pub(super) fn popup_position(
    layout: VendorModeLayout,
    source: VendorItemPopupSource0104,
    viewport: Vec2,
    height: f32,
    try_on: bool,
) -> Option<Vec2> {
    let (anchor, inventory) = match source {
        VendorItemPopupSource0104::CatalogRow { row_index }
        | VendorItemPopupSource0104::BuybackRow { row_index } => {
            (layout.row_rect(row_index)?, false)
        }
        VendorItemPopupSource0104::InventorySlot { inventory_slot } => (
            layout.item_mode.inventory_slot_rect(inventory_slot)?.into(),
            true,
        ),
    };
    let width = if try_on { 525. } else { 310. };
    let left = if inventory {
        anchor.left - 318.
    } else {
        anchor.left + 173. - if try_on { 200. } else { 0. }
    };
    Some(Vec2::new(
        left.clamp(0., (viewport.x - width).max(0.)),
        (anchor.top - 40.).clamp(0., (viewport.y - height).max(0.)),
    ))
}

pub(super) fn bind(
    mut commands: Commands,
    popup: Res<VendorItemPopupState>,
    content: Option<Res<crate::tutorial_mission_content::TutorialMissionContent>>,
    vendor_projection: Option<Res<VendorModeProjection0104>>,
    vendor_state: Option<Res<VendorUiState>>,
    try_image: Option<Res<crate::player_preview::NativePlayerTryOnPreviewImage>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    server: Res<AssetServer>,
    assets: Res<VendorUiAssets>,
    mut parts: Query<(
        Entity,
        &Part,
        &mut Node,
        Option<&mut ImageNode>,
        Option<&mut LocalizedText>,
        Option<&Interaction>,
        Option<&crate::localization::LocalizedTextCase>,
        Option<&mut TextColor>,
    )>,
) {
    // Compare at writers so an idle popup does not dirty Bevy layout or images.
    macro_rules! assign {
        ($field:expr, $value:expr) => {{
            let value = $value;
            if $field != value {
                $field = value;
            }
        }};
    }
    let Some(window) = windows.iter().next() else {
        return;
    };
    let selected = popup.selection.as_ref();
    let layout =
        vendor_projection
            .as_ref()
            .zip(vendor_state.as_ref())
            .map(|(projection, vendor_state)| {
                vendor_mode_layout(
                    window.width() as u32,
                    window.height() as u32,
                    vendor_state.opening_elapsed_seconds,
                    vendor_state.vendor_scroll_y,
                    vendor_state.inventory_scroll_y,
                    projection.rows_for_tab(vendor_state.tab),
                )
            });
    let general = selected.is_some_and(|s| s.item.item_type == 7 && !s.contract.buyback);
    let chest = selected.is_some_and(|s| s.item.item_type == 9 && !s.contract.buyback);
    let calculator = matches!(
        popup.quantity(),
        Some(VendorQuantityContract0104::Calculator { .. })
    );
    let detail = selected.and_then(|s| {
        content
            .as_ref()?
            .gameplay_user_equip_item_detail(s.item.item_type, s.item.item_id)
    });
    let combined = selected
        .is_some_and(|s| (0..=3).contains(&s.item.item_type) && (s.item.option >> 16) as i16 > 0);
    let display_icon = selected.and_then(|s| content.as_ref()?.gameplay_item_display_icon(s.item));
    let item_text = selected.and_then(|s| {
        content.as_ref()?.gameplay_user_equip_item_text(
            s.item.item_type,
            crate::user_equip_ui::user_equip_display_text_id(s.item),
        )
    });
    // Clean EquipPopup.ValueColor compares with the player's same-slot item.
    let rating_colors = selected.zip(content.as_deref()).map(|(s, content)| {
        let same_slot_item = usize::try_from(s.item.item_type).ok().and_then(|slot| {
            vendor_projection
                .as_deref()?
                .equipment
                .iter()
                .find(|equipped| equipped.wire_slot_index == slot && !equipped.item.empty)
                .map(|equipped| equipped.item.item)
        });
        crate::user_equip_ui::user_equip_item_rating_colors(
            content,
            Color::srgb(0.8, 1., 1.),
            s.item,
            same_slot_item,
            false,
        )
    });
    for (entity, part, mut node, image, copy, interaction, case, color) in &mut parts {
        if matches!(part, Part::Description) {
            if general && case.is_none() {
                commands
                    .entity(entity)
                    .insert(crate::localization::LocalizedTextCase::Uppercase);
            } else if !general && case.is_some() {
                commands
                    .entity(entity)
                    .remove::<crate::localization::LocalizedTextCase>();
            }
        }
        let visible = match part {
            Part::CombinedBadge => selected.is_some_and(|s| {
                (0..=3).contains(&s.item.item_type) && ((s.item.option >> 16) as i16) > 0
            }),
            Part::TryPanel => popup.try_on_item().is_some(),
            Part::TryOn => popup.selected_try_on_item().is_some(),
            Part::Info | Part::Detail(0..=3) => {
                !general && !chest && selected.is_some_and(|s| s.item.item_type != 10)
            }
            Part::Detail(_) => !general && !chest,
            Part::Root => selected.is_some(),
            Part::Pad | Part::Amount | Part::AmountLabel | Part::Clear | Part::Digit(_) => {
                calculator
            }
            Part::Delete => selected.is_some_and(|s| s.contract.delete),
            Part::Accept => selected.is_some_and(|s| {
                s.contract.buy.is_some()
                    || s.contract.buyback
                    || s.contract.sell.is_some()
                    || s.contract.open_chest
            }),
            _ => true,
        };
        let display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
        let Some(selected) = selected else {
            continue;
        };
        if let Some(mut color) = color {
            let value = match part {
                Part::Level => Color::srgb(1., 1., 0.),
                Part::Detail(10)
                    if selected.item.item_type != 0 && selected.item.item_type != 10 =>
                {
                    Color::srgb(0., 0., 1.)
                }
                Part::Detail(12) => {
                    if !combined && detail.as_ref().is_some_and(|d| d.tradeable) {
                        Color::srgb(0., 1., 0.)
                    } else {
                        Color::srgb(1., 0., 0.)
                    }
                }
                _ => Color::srgb(0.8, 1., 1.),
            };
            if matches!(part, Part::Level | Part::Detail(10 | 12)) {
                assign!(color.0, value);
            }
            if let (Part::Detail(index @ 1..=3), Some(colors)) = (part, rating_colors) {
                assign!(color.0, colors[usize::from(*index) - 1]);
            }
        }
        match part {
            Part::TryLeft | Part::TryRight => {
                use crate::user_equip_ui::*;
                if let Some(mut image) = image {
                    let hover = matches!(
                        interaction,
                        Some(Interaction::Hovered | Interaction::Pressed)
                    );
                    let path = match (part, hover) {
                        (Part::TryLeft, true) => USER_EQUIP_TURN_RIGHT_HOVER_PATH,
                        (Part::TryLeft, false) => USER_EQUIP_TURN_RIGHT_PATH,
                        (_, true) => USER_EQUIP_TURN_LEFT_HOVER_PATH,
                        (_, false) => USER_EQUIP_TURN_LEFT_PATH,
                    };
                    assign!(image.image, server.load(path));
                }
            }
            Part::TryAvatar => {
                if let (Some(mut image), Some(target)) = (image, try_image.as_ref()) {
                    assign!(image.image, target.0.clone());
                }
            }
            Part::TryOn => {
                if let Some(mut image) = image {
                    assign!(
                        image.color,
                        if popup.try_on_allowed {
                            Color::WHITE
                        } else {
                            Color::srgba(1., 1., 1., 0.4)
                        }
                    );
                    assign!(
                        image.image,
                        server.load(
                            if popup.try_on_allowed
                                && matches!(
                                    interaction,
                                    Some(Interaction::Hovered | Interaction::Pressed)
                                )
                            {
                                USER_EQUIP_BUTTON_HOVER_PATH
                            } else {
                                USER_EQUIP_BUTTON_NORMAL_PATH
                            }
                        )
                    );
                }
            }
            Part::Description if item_text.is_some() => {
                bind_vendor_text_value(copy, item_text.as_ref().unwrap().1.clone());
            }
            Part::Name if item_text.is_some() => {
                bind_vendor_text_value(copy, item_text.as_ref().unwrap().0.clone());
            }
            Part::Description => bind_vendor_text_value(
                copy,
                LocalizedText::new("ui.inventory.popup.description", "{description}")
                    .with_arg("description", ""),
            ),
            Part::Detail(index) => {
                use crate::user_equip_ui::{
                    user_equip_item_type_localized, user_equip_range_localized,
                    user_equip_rarity_localized, user_equip_weapon_type_localized,
                };
                let spec = match index {
                    1..=3 => {
                        let value = detail.as_ref().map(|d| match index {
                            1 => d.point_rating,
                            2 => d.group_rating,
                            _ => d.defense_rating,
                        });
                        LocalizedText::new(
                            match index {
                                1 => "ui.inventory.popup.point_value",
                                2 => "ui.inventory.popup.group_value",
                                _ => "ui.inventory.popup.defense_value",
                            },
                            "{value}",
                        )
                        .with_arg("value", value.map(|v| v.to_string()).unwrap_or_default())
                    }
                    6 => LocalizedText::new(
                        if selected.item.item_type == 10 {
                            "ui.inventory.popup.speed"
                        } else {
                            "ui.inventory.popup.range"
                        },
                        if selected.item.item_type == 10 {
                            "Speed"
                        } else {
                            "Range"
                        },
                    ),
                    10 if selected.item.item_type == 10 => {
                        LocalizedText::new("ui.inventory.vehicle_class", "{speed} Class").with_arg(
                            "speed",
                            detail
                                .as_ref()
                                .and_then(|d| d.vehicle_speed_class)
                                .unwrap_or_default()
                                .to_string(),
                        )
                    }
                    9 if selected.item.item_type == 0 => user_equip_weapon_type_localized(
                        detail.as_ref().and_then(|d| d.target_mode),
                    ),
                    9 => user_equip_item_type_localized(selected.item.item_type),
                    10 => user_equip_range_localized(
                        detail
                            .as_ref()
                            .filter(|_| selected.item.item_type == 0)
                            .and_then(|d| d.equip_type),
                    ),
                    11 if combined => LocalizedText::new("ui.inventory.rarity.special", "Special"),
                    11 => user_equip_rarity_localized(detail.as_ref().and_then(|d| d.rarity)),
                    12 => {
                        if !combined && detail.as_ref().is_some_and(|d| d.tradeable) {
                            LocalizedText::new("ui.inventory.popup.trade_value", "Tradable")
                        } else {
                            LocalizedText::new("ui.inventory.popup.not_tradable", "Not tradable")
                        }
                    }
                    _ => continue,
                };
                bind_vendor_text_value(copy, spec);
            }
            Part::Root => {
                let height = if general {
                    355.
                } else if chest {
                    235.
                } else {
                    449.
                };
                if let Some(layout) = layout {
                    if let Some(position) = popup_position(
                        layout,
                        selected.contract.source,
                        Vec2::new(window.width(), window.height()),
                        height,
                        popup.try_on_item().is_some(),
                    ) {
                        assign!(node.left, px(position.x));
                        assign!(node.top, px(position.y));
                    }
                }
                assign!(node.height, px(height));
            }
            Part::Back => {
                if let Some(mut image) = image {
                    let (path, top, height) = if general {
                        (USER_EQUIP_GENERAL_DIALOG_PATH, 0., 355.)
                    } else if chest {
                        (USER_EQUIP_USE_DIALOG_PATH, 0., 235.)
                    } else {
                        (USER_EQUIP_EQUIP_POPUP_PATH, 14., 435.)
                    };
                    assign!(node.top, px(top));
                    assign!(node.height, px(height));
                    assign!(image.image, server.load(path));
                }
            }
            Part::Icon => {
                if let Some(mut image) = image {
                    let handle = if let Some(path) = display_icon {
                        server.load(path.to_owned())
                    } else if combined && content.is_some() {
                        assets.missing_checker.clone()
                    } else {
                        match &selected.icon {
                            VendorPresentationIcon0104::Resolved(path) => {
                                server.load(path.runtime_path().to_owned())
                            }
                            VendorPresentationIcon0104::MissingChecker(_) => {
                                assets.missing_checker.clone()
                            }
                            VendorPresentationIcon0104::Empty => Handle::default(),
                        }
                    };
                    assign!(image.image, handle);
                    assign!(
                        node.display,
                        if display_icon.is_none()
                            && !combined
                            && matches!(selected.icon, VendorPresentationIcon0104::Empty)
                        {
                            Display::None
                        } else {
                            Display::Flex
                        }
                    );
                }
            }
            Part::Name => bind_vendor_text_value(
                copy,
                vendor_item_name_localized(
                    selected.metadata.as_ref().map_or("", |m| m.name.as_str()),
                ),
            ),
            Part::Level if general => bind_vendor_text_value(
                copy,
                LocalizedText::new("ui.inventory.popup.cost", "COST {cost}")
                    .with_arg("cost", popup.cost_label().unwrap_or_default()),
            ),
            Part::Level => bind_vendor_text_value(
                copy,
                LocalizedText::new("ui.inventory.popup.level", "Level {level}").with_arg(
                    "level",
                    selected
                        .metadata
                        .as_ref()
                        .map_or(0, |m| m.level)
                        .to_string(),
                ),
            ),
            Part::Amount => bind_vendor_text_value(
                copy,
                LocalizedText::new("ui.inventory.popup.amount_value", "{amount}")
                    .with_arg("amount", popup.amount.to_string()),
            ),
            Part::Delete => {
                assign!(
                    node.top,
                    px(if general {
                        302.
                    } else if chest {
                        180.
                    } else {
                        375.
                    })
                );
            }
            Part::Accept => {
                let equip_button = selected.contract.buyback || selected.contract.sell.is_some();
                let width = if general {
                    150.
                } else if chest || equip_button {
                    130.
                } else {
                    85.
                };
                assign!(node.width, px(width));
                if let Some(mut image) = image {
                    assign!(
                        node.left,
                        px(if general {
                            79.
                        } else if chest || equip_button {
                            160.
                        } else {
                            210.
                        })
                    );
                    assign!(
                        node.top,
                        px(if general {
                            301.
                        } else if chest {
                            180.
                        } else if equip_button {
                            380.
                        } else {
                            390.
                        })
                    );
                    assign!(
                        image.image,
                        server.load(
                            if interaction == Some(&Interaction::Hovered)
                                || interaction == Some(&Interaction::Pressed)
                            {
                                USER_EQUIP_BUTTON_HOVER_PATH
                            } else {
                                USER_EQUIP_BUTTON_NORMAL_PATH
                            },
                        )
                    );
                }
                bind_vendor_text_value(
                    copy,
                    if selected.contract.open_chest {
                        LocalizedText::new("ui.inventory.action.open", "OPEN")
                    } else if selected.contract.buyback {
                        LocalizedText::new("ui.vendor.popup.buyback", "BUY BACK")
                    } else if selected.contract.sell.is_some() {
                        LocalizedText::new("ui.vendor.popup.sell", "SELL")
                    } else {
                        LocalizedText::new("ui.vendor.popup.buy", "BUY")
                    },
                );
            }
            _ => {}
        }
    }
}
