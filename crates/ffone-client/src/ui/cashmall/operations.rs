use super::*;

#[must_use]
pub fn cashmall_tab_view_0104(
    selected: CashmallTab0104,
    tab: CashmallTab0104,
    hovered: bool,
    pressed: bool,
) -> CashmallTabView0104 {
    let is_selected = selected == tab;
    CashmallTabView0104 {
        tab,
        rect: CASHMALL_TAB_RECTS[tab.index()],
        hit_rect: CASHMALL_TAB_HIT_RECTS[tab.index()],
        visual: if is_selected {
            CashmallTabVisual0104::Selected
        } else if hovered || pressed {
            CashmallTabVisual0104::Hover
        } else {
            CashmallTabVisual0104::Normal
        },
        label: tab.legacy_name(),
        label_source: if is_selected {
            CashmallTabLabelSource0104::RawEnumName
        } else {
            CashmallTabLabelSource0104::LocalizedKey
        },
        pressed_visual_is_distinct: false,
    }
}

#[must_use]
pub fn cashmall_scan_slot9_0104(
    player_inventory: &[CashmallPlayerInventoryItem0104],
    taros: i32,
) -> Vec<CashmallRowProjection0104> {
    (0..CASHMALL_SLOT_SCAN_COUNT)
        .filter_map(|slot_id| {
            let source = player_inventory.iter().find(|entry| {
                entry.slot_type == CASHMALL_SLOT_TYPE && entry.slot_id == slot_id as i32
            })?;
            let restricted = cashmall_item_needs_equip_validation(source.item.item_type)
                && source.equip_eligible == Some(false);
            let insufficient_taros = source.item_price > taros;
            Some(CashmallRowProjection0104 {
                scan_slot_id: slot_id,
                source: source.clone(),
                icon: source
                    .icon
                    .clone()
                    .map_or(CashmallPresentationIcon0104::MissingChecker, |icon| {
                        CashmallPresentationIcon0104::Resolved(icon)
                    }),
                frame_visual: if restricted {
                    CashmallSlotFrameVisual0104::Restricted
                } else {
                    CashmallSlotFrameVisual0104::Normal
                },
                frame_alpha_percent: if insufficient_taros { 40 } else { 100 },
                icon_alpha_percent: 100,
                level_label: format!("LEVEL {}", source.level),
                price_text_visible: false,
            })
        })
        .collect()
}

#[must_use]
pub fn cashmall_opening_eased_fraction_0104(elapsed_seconds: f32) -> f32 {
    if elapsed_seconds.is_nan() || elapsed_seconds <= 0.0 {
        return 0.0;
    }
    if !elapsed_seconds.is_finite() || elapsed_seconds >= CASHMALL_OPEN_SECONDS {
        return 1.0;
    }
    (elapsed_seconds / CASHMALL_OPEN_SECONDS * std::f32::consts::FRAC_PI_2).sin()
}

#[must_use]
pub const fn cashmall_cash_digits_0104() -> [u8; CASHMALL_CASH_DIGIT_COUNT] {
    [0; CASHMALL_CASH_DIGIT_COUNT]
}

#[must_use]
pub fn cashmall_cash_text_0104() -> String {
    cashmall_cash_digits_0104()
        .into_iter()
        .map(|digit| char::from(b'0' + digit))
        .collect()
}

#[must_use]
pub fn cashmall_taros_counter_digit_0104(mut value: i32, index: usize) -> Option<i32> {
    if index >= CASHMALL_PC_STUFF_TAROS_DIGIT_RECTS.len() {
        return None;
    }
    let mut divisor = 100_000_000;
    for current in 0..=index {
        let digit = value / divisor;
        if current == index {
            return Some(digit);
        }
        value -= digit * divisor;
        divisor /= 10;
    }
    None
}

pub(super) fn cashmall_missing_checker_image_0104() -> Image {
    Image::new(
        Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        user_equip_missing_checker_rgba(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

pub(super) fn cashmall_passthrough_text_0104(value: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", value)
}

pub(super) fn cashmall_item_level_text_0104(level: i32) -> LocalizedText {
    LocalizedText::new("ui.cashmall.item.level", "LEVEL {level}")
        .with_arg("level", level.to_string())
}

pub(super) fn cashmall_resolved_fallback_0104(localized: &LocalizedText) -> String {
    let mut resolved = localized.fallback.clone();
    for (name, value) in &localized.args {
        resolved = resolved.replace(&format!("{{{name}}}"), value);
    }
    resolved
}

#[allow(clippy::too_many_arguments)]
pub(super) fn bind_cashmall_ui_0104(
    asset_server: Res<AssetServer>,
    assets: Res<CashmallUiAssets0104>,
    state: Res<CashmallUiState0104>,
    modal: Res<CashmallModalState0104>,
    projection: Res<CashmallModeProjection0104>,
    hover: Res<CashmallHoverState0104>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut asset_status: ResMut<CashmallUiAssetStatus0104>,
    mut roots: Query<(&mut Node, &mut Visibility), With<CashmallUiRoot0104>>,
    mut elements: Query<
        (
            &CashmallUiElement0104,
            &mut Node,
            Option<&mut ImageNode>,
            Option<&mut LocalizedText>,
            Option<&mut TextColor>,
            Option<&mut ZIndex>,
        ),
        Without<CashmallUiRoot0104>,
    >,
) {
    let readiness = assets.readiness(&asset_server);
    asset_status.0 = readiness;
    let Ok(window) = windows.single() else {
        for (_, mut visibility) in &mut roots {
            *visibility = Visibility::Hidden;
        }
        return;
    };
    let width = window.width().max(0.0) as u32;
    let height = window.height().max(0.0) as u32;
    let view = cashmall_mode_view_0104(
        width,
        height,
        &state,
        *modal,
        &projection,
        hover.hovered_tabs,
        hover.pressed_tabs,
        readiness == CashmallStaticAssetReadiness0104::Ready,
    );
    for (mut node, mut visibility) in &mut roots {
        node.width = px(width);
        node.height = px(height);
        *visibility = if view.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    let Some(view) = view else {
        return;
    };

    for (element, mut node, image, text, text_color, z_index) in &mut elements {
        match *element {
            CashmallUiElement0104::Backdrop => {
                bind_cashmall_rect_0104(&mut node, view.layout.full_backdrop)
            }
            CashmallUiElement0104::CashmallBackplate => {
                bind_cashmall_rect_0104(&mut node, view.layout.cashmall_backplate)
            }
            CashmallUiElement0104::RightBackplate => {
                bind_cashmall_rect_0104(&mut node, view.layout.right_backplate)
            }
            CashmallUiElement0104::CashmallPanel => {
                bind_cashmall_rect_0104(&mut node, view.layout.cashmall_panel)
            }
            CashmallUiElement0104::PcStuffPanel => {
                bind_cashmall_rect_0104(&mut node, view.layout.item_mode.pc_stuff_panel.into())
            }
            CashmallUiElement0104::EquipmentPanel => {
                bind_cashmall_rect_0104(&mut node, view.layout.item_mode.equipment_panel.into())
            }
            CashmallUiElement0104::ListContent => {
                node.left = px(0);
                node.top = px(-state.cashmall_scroll_y());
                node.width = px(CASHMALL_ROW_WIDTH);
                // Clean feeds BeginScrollView the never-populated cached count,
                // not the number of rows `DoSlot` subsequently draws.
                node.height = px(CASHMALL_LEGACY_SCROLL_CONTENT_HEIGHT);
            }
            CashmallUiElement0104::InventoryContent => {
                node.left = px(0);
                node.top = px(-state.inventory_scroll_y());
                node.width = px(USER_EQUIP_INVENTORY_CONTENT_WIDTH);
                node.height = px(USER_EQUIP_INVENTORY_CONTENT_HEIGHT);
            }
            CashmallUiElement0104::TabVisual(tab) => {
                let tab_view = view.tabs[tab.index()];
                if let Some(mut image) = image {
                    image.image = assets.image(cashmall_tab_asset_role_0104(tab, tab_view.visual));
                }
                if let Some(mut z_index) = z_index {
                    z_index.0 = if tab_view.visual == CashmallTabVisual0104::Selected {
                        3
                    } else {
                        1
                    };
                }
            }
            CashmallUiElement0104::TabLabel(tab) => {
                let tab_view = view.tabs[tab.index()];
                bind_cashmall_localized_text_0104(
                    text,
                    if tab_view.label_source == CashmallTabLabelSource0104::RawEnumName {
                        cashmall_passthrough_text_0104(tab_view.label)
                    } else {
                        cashmall_tab_localized_text_0104(tab)
                    },
                );
                bind_cashmall_text_color_0104(
                    text_color,
                    cashmall_blankbox_text_color_0104(
                        view.controls.tabs,
                        hover.hovered_tabs[tab.index()],
                        hover.pressed_tabs[tab.index()],
                    ),
                );
            }
            CashmallUiElement0104::GoToStuff => {
                if let Some(mut image) = image {
                    image.image = assets.image(if hover.go_to_stuff {
                        CashmallStaticAssetRole0104::ButtonHover
                    } else {
                        CashmallStaticAssetRole0104::ButtonNormal
                    });
                }
            }
            CashmallUiElement0104::GoToStuffLabel => bind_cashmall_text_color_0104(
                text_color,
                cashmall_button_text_color_0104(
                    view.controls.go_to_stuff,
                    hover.go_to_stuff,
                    hover.go_to_stuff_pressed,
                ),
            ),
            CashmallUiElement0104::NanoTab => {
                if let Some(mut image) = image {
                    image.image = assets.image(if hover.nano_tab {
                        CashmallStaticAssetRole0104::NanoTabHover
                    } else {
                        CashmallStaticAssetRole0104::NanoTab
                    });
                }
            }
            CashmallUiElement0104::NanoTabLabel => bind_cashmall_text_color_0104(
                text_color,
                cashmall_blankbox_text_color_0104(
                    view.controls.pc_stuff_controls,
                    hover.nano_tab,
                    false,
                ),
            ),
            CashmallUiElement0104::RedeemCode => {
                if let Some(mut image) = image {
                    image.image = assets.image(if hover.redeem_code {
                        CashmallStaticAssetRole0104::ButtonHover
                    } else {
                        CashmallStaticAssetRole0104::ButtonNormal
                    });
                }
            }
            CashmallUiElement0104::RedeemCodeLabel => bind_cashmall_text_color_0104(
                text_color,
                cashmall_button_text_color_0104(
                    view.controls.pc_stuff_controls,
                    hover.redeem_code,
                    hover.redeem_code_pressed,
                ),
            ),
            CashmallUiElement0104::NpcName => bind_cashmall_localized_text_0104(
                text,
                cashmall_passthrough_text_0104(&view.npc_name),
            ),
            CashmallUiElement0104::CashDigit(index) => {
                let digit = view.cash_digits.get(index).copied().unwrap_or(0);
                let value = char::from(b'0' + digit.min(9)).to_string();
                bind_cashmall_localized_text_0104(text, cashmall_passthrough_text_0104(value));
            }
            CashmallUiElement0104::Row(row) => {
                node.display = display_cashmall_if_0104(row < view.rows.len());
            }
            CashmallUiElement0104::RowFrame(row) => {
                let row_view = view.rows.get(row);
                node.display = display_cashmall_if_0104(row_view.is_some());
                if let (Some(row_view), Some(mut image)) = (row_view, image) {
                    image.image = assets.image(match row_view.frame_visual {
                        CashmallSlotFrameVisual0104::Normal => {
                            CashmallStaticAssetRole0104::SlotOccupied
                        }
                        CashmallSlotFrameVisual0104::Restricted => {
                            CashmallStaticAssetRole0104::Restricted
                        }
                    });
                    image.color =
                        Color::srgba(1.0, 1.0, 1.0, row_view.frame_alpha_percent as f32 / 100.0);
                    if let Some(mut z_index) = z_index {
                        // Clean reverses icon/frame draw order only in the
                        // insufficient-Taros branch.
                        z_index.0 = if row_view.frame_alpha_percent < 100 {
                            2
                        } else {
                            0
                        };
                    }
                }
            }
            CashmallUiElement0104::RowIcon(row) => {
                if let Some(row_view) = view.rows.get(row) {
                    bind_cashmall_row_icon_0104(
                        &mut node,
                        image,
                        &row_view.icon,
                        &asset_server,
                        &assets,
                        Color::srgba(1.0, 1.0, 1.0, row_view.icon_alpha_percent as f32 / 100.0),
                    );
                } else {
                    node.display = Display::None;
                }
            }
            CashmallUiElement0104::RowName(row) => {
                bind_optional_cashmall_localized_text_0104(
                    &mut node,
                    text,
                    view.rows
                        .get(row)
                        .map(|row| cashmall_passthrough_text_0104(&row.source.name)),
                );
            }
            CashmallUiElement0104::RowLevel(row) => {
                bind_optional_cashmall_localized_text_0104(
                    &mut node,
                    text,
                    view.rows
                        .get(row)
                        .map(|row| cashmall_item_level_text_0104(row.source.level)),
                );
            }
            CashmallUiElement0104::InventorySlotFrame(slot) => {
                if let Some(mut image) = image {
                    let visual = if projection.shared_item_mode.inventory[slot].item.empty {
                        UserEquipSlotFrameVisual::Empty
                    } else {
                        UserEquipSlotFrameVisual::Occupied
                    };
                    image.image = cashmall_shared_slot_frame_image_0104(visual, &assets);
                }
            }
            CashmallUiElement0104::InventorySlotIcon(slot) => {
                let icon = UserEquipPresentationIcon::from_projection(
                    &projection.shared_item_mode.inventory[slot].item.icon,
                );
                bind_cashmall_shared_icon_0104(&mut node, image, &icon, &asset_server, &assets);
            }
            CashmallUiElement0104::InventorySlotBadge(slot) => {
                node.display = display_cashmall_if_0104(
                    projection.shared_item_mode.inventory[slot]
                        .item
                        .show_combined_badge,
                );
            }
            CashmallUiElement0104::InventorySlotCount(slot) => {
                bind_optional_cashmall_localized_text_0104(
                    &mut node,
                    text,
                    cashmall_shared_item_overlay_text_0104(
                        &projection.shared_item_mode.inventory[slot].item,
                    ),
                );
            }
            CashmallUiElement0104::EquipmentSlotFrame(slot) => {
                if let Some(mut image) = image {
                    let visual = if projection.shared_item_mode.equipment[slot].item.empty {
                        UserEquipSlotFrameVisual::Empty
                    } else {
                        UserEquipSlotFrameVisual::Occupied
                    };
                    image.image = cashmall_shared_slot_frame_image_0104(visual, &assets);
                }
            }
            CashmallUiElement0104::EquipmentSlotIcon(slot) => {
                let icon = UserEquipPresentationIcon::from_projection(
                    &projection.shared_item_mode.equipment[slot].item.icon,
                );
                bind_cashmall_shared_icon_0104(&mut node, image, &icon, &asset_server, &assets);
            }
            CashmallUiElement0104::EquipmentSlotBadge(slot) => {
                node.display = display_cashmall_if_0104(
                    projection.shared_item_mode.equipment[slot]
                        .item
                        .show_combined_badge,
                );
            }
            CashmallUiElement0104::TarosDigit(index) => {
                let value = cashmall_taros_counter_digit_0104(projection.taros, index)
                    .unwrap_or_default()
                    .to_string();
                bind_cashmall_localized_text_0104(text, cashmall_passthrough_text_0104(value));
            }
            CashmallUiElement0104::Info
            | CashmallUiElement0104::Title
            | CashmallUiElement0104::Cash
            | CashmallUiElement0104::Dialog
            | CashmallUiElement0104::ListBack
            | CashmallUiElement0104::ListDivider
            | CashmallUiElement0104::Table
            | CashmallUiElement0104::ListViewport
            | CashmallUiElement0104::TableShadow
            | CashmallUiElement0104::NanoTabHit
            | CashmallUiElement0104::InventoryPanel
            | CashmallUiElement0104::ItemTabLabel
            | CashmallUiElement0104::InventoryViewport
            | CashmallUiElement0104::DexlabsBanner
            | CashmallUiElement0104::TarosCounter
            | CashmallUiElement0104::Close
            | CashmallUiElement0104::Trash
            | CashmallUiElement0104::Help
            | CashmallUiElement0104::EquipmentTitle
            | CashmallUiElement0104::EquipmentTitleLabel
            | CashmallUiElement0104::EquipmentSlotLabel(_) => {}
        }
    }
}

pub(super) fn bind_cashmall_text_color_0104(color: Option<Mut<TextColor>>, value: Color) {
    let Some(mut color) = color else {
        return;
    };
    color.0 = value;
}

pub(super) fn cashmall_blankbox_text_color_0104(enabled: bool, hovered: bool, pressed: bool) -> Color {
    if !enabled || pressed {
        Color::BLACK
    } else if hovered {
        Color::srgb(0.189_516_13, 0.044_354_84, 1.0)
    } else {
        Color::srgb(0.794_354_86, 1.0, 1.0)
    }
}

pub(super) fn bind_cashmall_row_icon_0104(
    node: &mut Node,
    image: Option<Mut<ImageNode>>,
    icon: &CashmallPresentationIcon0104,
    asset_server: &AssetServer,
    assets: &CashmallUiAssets0104,
    tint: Color,
) {
    let Some(mut image) = image else {
        return;
    };
    node.display = Display::Flex;
    image.color = tint;
    image.image = match icon {
        CashmallPresentationIcon0104::MissingChecker => assets.missing_checker.clone(),
        CashmallPresentationIcon0104::Resolved(path) => {
            let handle = asset_server.load::<Image>(path.runtime_path().to_owned());
            if matches!(asset_server.load_state(handle.id()), LoadState::Loaded) {
                handle
            } else {
                assets.missing_checker.clone()
            }
        }
    };
}

pub(super) fn bind_cashmall_shared_icon_0104(
    node: &mut Node,
    image: Option<Mut<ImageNode>>,
    icon: &UserEquipPresentationIcon,
    asset_server: &AssetServer,
    assets: &CashmallUiAssets0104,
) {
    let Some(mut image) = image else {
        return;
    };
    image.color = Color::WHITE;
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

pub(super) fn cashmall_shared_item_overlay_text_0104(
    item: &crate::user_equip_ui::UserEquipItemProjection,
) -> Option<LocalizedText> {
    if item.empty {
        None
    } else {
        match item.item.item_type {
            7 => Some(cashmall_passthrough_text_0104(item.item.option.to_string())),
            8 => Some(
                LocalizedText::new("ui.cashmall.item.quest", "Quest {item_id}")
                    .with_arg("item_id", item.item.item_id.to_string()),
            ),
            _ => None,
        }
    }
}
