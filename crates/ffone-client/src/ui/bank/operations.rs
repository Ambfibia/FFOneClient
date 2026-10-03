use super::*;

#[must_use]
pub fn bank_opening_eased_fraction(elapsed_seconds: f32) -> f32 {
    if elapsed_seconds.is_nan() || elapsed_seconds <= 0.0 {
        return 0.0;
    }
    if !elapsed_seconds.is_finite() || elapsed_seconds >= BANK_OPEN_SECONDS {
        return 1.0;
    }
    (elapsed_seconds / BANK_OPEN_SECONDS * std::f32::consts::FRAC_PI_2).sin()
}

/// Exact clean quotient/subtraction loop for one of the nine Taros digits.
/// Rust and C# integer division both truncate toward zero, including for a
/// malformed negative authoritative value, so no speculative clamp is added.
#[must_use]
pub fn bank_taros_counter_digit_0104(mut value: i32, index: usize) -> Option<i32> {
    if index >= BANK_PC_STUFF_TAROS_DIGIT_RECTS.len() {
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

#[must_use]
pub fn bank_taros_counter_digits_0104(value: i32) -> [String; 9] {
    array::from_fn(|index| {
        bank_taros_counter_digit_0104(value, index)
            .unwrap_or_default()
            .to_string()
    })
}

pub(super) fn equipment_slot_label(visual_index: usize) -> String {
    let spec = USER_EQUIP_EQUIPMENT_STRIP_ORDER[visual_index];
    match spec.label_ordinal {
        Some(ordinal) => format!("{} {ordinal}", spec.label_key),
        None => spec.label_key.to_owned(),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn bind_bank_ui(
    asset_server: Res<AssetServer>,
    assets: Res<BankUiRuntimeAssets>,
    state: Res<BankUiState>,
    modal: Res<BankModalState>,
    projection: Res<BankModeProjection0104>,
    pc_stuff: Res<BankPcStuffAuthority0104>,
    search: Res<BankSearch>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut roots: Query<(&mut Node, &mut Visibility), With<BankUiRoot>>,
    mut elements: Query<
        (
            &BankUiElement,
            &mut Node,
            Option<&mut ImageNode>,
            Option<&mut LocalizedText>,
            Option<&Interaction>,
        ),
        Without<BankUiRoot>,
    >,
    mut taros_digit_texts: Query<
        (&BankTarosDigitText0104, &mut LocalizedText),
        Without<BankUiElement>,
    >,
) {
    let Ok(window) = windows.single() else {
        for (_, mut visibility) in &mut roots {
            *visibility = Visibility::Hidden;
        }
        return;
    };
    let width = window.width().max(0.0) as u32;
    let height = window.height().max(0.0) as u32;
    let view = bank_mode_view_with_pc_stuff(
        width,
        height,
        *state,
        *modal,
        &projection,
        *pc_stuff,
        assets.0.all_loaded(&asset_server),
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

    for (digit, mut localized) in &mut taros_digit_texts {
        *localized = bank_taros_digit_localized(view.taros_digits[digit.0].clone());
    }

    for (element, mut node, image, localized, interaction) in &mut elements {
        match *element {
            BankUiElement::Backdrop => bind_rect(&mut node, view.layout.full_backdrop),
            BankUiElement::BankBackplate => bind_rect(&mut node, view.layout.bank_backplate),
            BankUiElement::RightBackplate => bind_rect(&mut node, view.layout.right_backplate),
            BankUiElement::BankPanel => bind_rect(&mut node, view.layout.bank_panel),
            BankUiElement::PcStuffPanel => {
                bind_rect(&mut node, view.layout.item_mode.pc_stuff_panel.into())
            }
            BankUiElement::EquipmentPanel => {
                bind_rect(&mut node, view.layout.item_mode.equipment_panel.into())
            }
            BankUiElement::BankContent => {
                node.left = px(0);
                node.top = px(-view.layout.bank_scroll_y);
                node.width = px(BANK_CONTENT_WIDTH);
                node.height = px(search.content_height());
            }
            BankUiElement::InventoryContent => {
                node.left = px(0);
                node.top = px(-view.layout.inventory_scroll_y);
                node.width = px(USER_EQUIP_INVENTORY_CONTENT_WIDTH);
                node.height = px(USER_EQUIP_INVENTORY_CONTENT_HEIGHT);
            }
            BankUiElement::BankScrollThumb => {
                let mut rect = view.layout.scroll_thumb_in_dialog();
                rect.height = crate::service_scroll::thumb_extent(
                    BANK_SCROLL_TRACK_RECT.height,
                    BANK_VIEWPORT_HEIGHT,
                    search.maximum(),
                );
                rect.width = 15.0;
                rect.top = BANK_SCROLL_TRACK_RECT.top
                    + if search.maximum() > 0.0 {
                        state.bank_scroll_y / search.maximum()
                            * (BANK_SCROLL_TRACK_RECT.height - rect.height)
                    } else {
                        0.0
                    };
                bind_rect(&mut node, rect);
                node.display = display_if(search.maximum() > 0.0);
            }
            BankUiElement::BankScrollTrack
            | BankUiElement::BankScrollUp
            | BankUiElement::BankScrollDown => {
                node.display = display_if(search.maximum() > 0.0);
            }
            BankUiElement::BankSlotFrame(slot) => {
                if let Some(index) = search.visible_index(slot) {
                    node.display = Display::Flex;
                    node.left = px((index % 6) as f32 * 67.0);
                    node.top = px((index / 6) as f32 * 67.0);
                } else {
                    node.display = Display::None;
                }
                if let Some(mut image) = image {
                    *image = bank_slot_frame_image(view.bank[slot].frame_visual, &assets.0);
                    if matches!(
                        view.bank[slot].frame_visual,
                        BankSlotFrameVisual::Empty | BankSlotFrameVisual::Occupied
                    ) {
                        bind_bank_button_visual(
                            &mut image,
                            interaction,
                            view.controls_enabled,
                            &assets.0,
                        );
                    }
                }
            }
            BankUiElement::RedeemCode => {
                if let Some(mut image) = image {
                    bind_bank_button_visual(
                        &mut image,
                        interaction,
                        view.controls_enabled,
                        &assets.0,
                    );
                }
            }
            BankUiElement::BankSlotIcon(slot) => {
                bind_presentation_icon(
                    &mut node,
                    image,
                    &view.bank[slot].icon,
                    &asset_server,
                    &assets.0,
                );
            }
            BankUiElement::BankSlotBadge(slot) => {
                node.display = display_if(view.bank[slot].show_combined_badge);
            }
            BankUiElement::BankSlotCount(slot) => {
                bind_count_text(&mut node, localized, view.bank[slot].count_label.as_deref());
            }
            BankUiElement::InventorySlotFrame(slot) => {
                if let Some(mut image) = image {
                    image.image =
                        inventory_slot_frame_image(view.inventory[slot].frame_visual, &assets.0);
                }
            }
            BankUiElement::InventorySlotIcon(slot) => {
                bind_presentation_icon(
                    &mut node,
                    image,
                    &view.inventory[slot].icon,
                    &asset_server,
                    &assets.0,
                );
            }
            BankUiElement::InventorySlotBadge(slot) => {
                node.display = display_if(view.inventory[slot].show_combined_badge);
            }
            BankUiElement::InventorySlotCount(slot) => {
                bind_count_text(
                    &mut node,
                    localized,
                    view.inventory[slot].count_label.as_deref(),
                );
            }
            BankUiElement::EquipmentSlotFrame(slot) => {
                if let Some(mut image) = image {
                    image.image =
                        inventory_slot_frame_image(view.equipment[slot].frame_visual, &assets.0);
                }
            }
            BankUiElement::EquipmentSlotIcon(slot) => {
                bind_presentation_icon(
                    &mut node,
                    image,
                    &view.equipment[slot].icon,
                    &asset_server,
                    &assets.0,
                );
            }
            BankUiElement::EquipmentSlotBadge(slot) => {
                node.display = display_if(view.equipment[slot].show_combined_badge);
            }
            BankUiElement::BankInfo
            | BankUiElement::BankTitle
            | BankUiElement::BankDialog
            | BankUiElement::BankTab
            | BankUiElement::BankViewport
            | BankUiElement::BankShadow
            | BankUiElement::ItemTabLabel
            | BankUiElement::InventoryViewport
            | BankUiElement::DexlabsBanner
            | BankUiElement::TarosCounter
            | BankUiElement::TarosDigit(_)
            | BankUiElement::RedeemCodeLabel
            | BankUiElement::Close
            | BankUiElement::Trash
            | BankUiElement::Help
            | BankUiElement::EquipmentTitle
            | BankUiElement::EquipmentTitleLabel
            | BankUiElement::EquipmentSlotLabel(_) => {}
        }
    }
}

pub(super) fn bind_count_text(node: &mut Node, localized: Option<Mut<LocalizedText>>, value: Option<&str>) {
    let Some(mut localized) = localized else {
        return;
    };
    if let Some(value) = value {
        node.display = Display::Flex;
        *localized = bank_count_localized(value);
    } else {
        node.display = Display::None;
        *localized = bank_count_localized("");
    }
}

pub(super) fn bind_presentation_icon(
    node: &mut Node,
    image: Option<Mut<ImageNode>>,
    icon: &UserEquipPresentationIcon,
    asset_server: &AssetServer,
    assets: &BankUiAssets,
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
            // Retain the strong image handle during asynchronous loading. Replacing
            // it with the checker here cancels cold loads that have no other owner.
            image.image = if matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)) {
                assets.missing_checker.clone()
            } else {
                handle
            };
        }
    }
}
