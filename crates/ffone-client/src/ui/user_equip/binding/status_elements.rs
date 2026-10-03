//! Binding of boost/potion/taros values, player status, tabs and Nano status panels.

use super::super::asset_contract::UserEquipStaticAssetRole;
use super::super::components::UserEquipUiElement;
use super::super::geometry::user_equip_taros_digits;
use super::super::images::presentation_icon_handle;
use super::super::state::UserEquipMode;
use super::{UserEquipBindContext, bounded_status_fraction, inventory_count_localized};
use crate::localization::LocalizedText;
use bevy::prelude::*;

/// Boost/potion/taros values, player status, tabs and Nano status panels.
#[allow(clippy::too_many_arguments)]
pub(super) fn bind_status_element(
    context: &UserEquipBindContext<'_, '_, '_>,
    element: &UserEquipUiElement,
    mut node: Mut<Node>,
    image: Option<Mut<ImageNode>>,
    localized: Option<Mut<LocalizedText>>,
    text_color: Option<Mut<TextColor>>,
    _text_layout: Option<Mut<TextLayout>>,
    _interaction: Option<&Interaction>,
) {
    let UserEquipBindContext {
        asset_server,
        assets,
        state,
        presentation,
        nano_portraits,
        nano_projection,
        nano_mode,
        item_tab_hovered,
        nano_tab_hovered,
        ..
    } = *context;
    match *element {
        UserEquipUiElement::BoostValue => {
            if let Some(mut localized) = localized {
                *localized = inventory_count_localized(presentation.weapon_battery.to_string());
            }
        }
        UserEquipUiElement::PotionValue => {
            if let Some(mut localized) = localized {
                *localized = inventory_count_localized(presentation.nano_battery.to_string());
            }
        }
        UserEquipUiElement::TarosDigit(index) => {
            if let Some(mut localized) = localized {
                let digits = user_equip_taros_digits(presentation.taros);
                let digit = digits.as_bytes().get(index).copied().unwrap_or(b'0') as char;
                *localized = LocalizedText::new("ui.inventory.status.taros_digit", "{digit}")
                    .with_arg("digit", digit.to_string());
            }
        }
        UserEquipUiElement::StatusName => {
            if let Some(mut localized) = localized {
                *localized = LocalizedText::new("ui.inventory.status.name", "{name}")
                    .with_arg("name", presentation.player_name.clone());
            }
        }
        UserEquipUiElement::StatusLevel => {
            if let Some(mut localized) = localized {
                *localized = LocalizedText::new("ui.inventory.status.level", "LEVEL : {level}")
                    .with_arg("level", presentation.level.to_string());
            }
        }
        UserEquipUiElement::StatusHpValue => {
            if let Some(mut localized) = localized {
                *localized =
                    LocalizedText::new("ui.inventory.status.hp_value", "{current}/{maximum}")
                        .with_arg("current", presentation.hp.to_string())
                        .with_arg("maximum", presentation.max_hp.to_string());
            }
        }
        UserEquipUiElement::StatusFusionMatterValue => {
            if let Some(mut localized) = localized {
                *localized = LocalizedText::new(
                    "ui.inventory.status.fusion_matter_value",
                    "{current}/{maximum}",
                )
                .with_arg("current", presentation.fusion_matter.to_string())
                .with_arg("maximum", presentation.max_fusion_matter.to_string());
            }
        }
        UserEquipUiElement::StatusGuideName => {
            if let Some(mut localized) = localized {
                *localized = presentation.guide_name_key.as_ref().map_or_else(
                    || {
                        LocalizedText::new("ui.inventory.status.guide_name", "{name}")
                            .with_arg("name", presentation.guide_name.clone())
                    },
                    |key| LocalizedText::new(key.clone(), presentation.guide_name.clone()),
                );
            }
        }
        UserEquipUiElement::StatusGuideIcon => {
            if let Some(mut image) = image {
                if let Some(path) = presentation.guide_icon_path.as_ref() {
                    node.display = Display::Flex;
                    image.image = asset_server.load::<Image>(path.clone());
                } else {
                    node.display = Display::None;
                    image.image = Handle::default();
                }
            }
        }
        UserEquipUiElement::StatusHpBar => {
            node.width =
                px(470.0 * bounded_status_fraction(presentation.hp, presentation.max_hp));
        }
        UserEquipUiElement::StatusFusionBar => {
            node.width = px(470.0
                * bounded_status_fraction(
                    presentation.fusion_matter,
                    presentation.max_fusion_matter,
                ));
        }
        UserEquipUiElement::ItemTab => {
            if let Some(mut image) = image {
                node.display = if state.mode() == UserEquipMode::Item {
                    Display::None
                } else {
                    Display::Flex
                };
                image.image = assets.0.image(if item_tab_hovered {
                    UserEquipStaticAssetRole::ItemTabHover
                } else {
                    UserEquipStaticAssetRole::ItemTab
                });
            }
        }
        UserEquipUiElement::NanoTab => {
            if let Some(mut image) = image {
                node.display = if nano_mode {
                    Display::None
                } else {
                    Display::Flex
                };
                image.image = assets.0.image(if nano_tab_hovered {
                    UserEquipStaticAssetRole::NanoTabHover
                } else {
                    UserEquipStaticAssetRole::NanoTab
                });
            }
        }
        UserEquipUiElement::NanoStatusPanel(slot) => {
            node.display = Display::Flex;
            debug_assert_eq!(nano_projection.status[slot].slot_index, slot);
        }
        UserEquipUiElement::NanoStatusPortrait(slot) => {
            let portrait = nano_portraits
                .as_deref()
                .and_then(|portraits| portraits.0[slot].as_ref());
            if nano_projection.status[slot].nano_id.is_some() {
                if let (Some(mut image), Some(portrait)) = (image, portrait) {
                    node.display = Display::Flex;
                    image.image = portrait.clone();
                } else {
                    node.display = Display::None;
                }
            } else {
                node.display = Display::None;
            }
        }
        UserEquipUiElement::NanoStatusType(slot) => {
            let status = &nano_projection.status[slot];
            if let (Some(style), Some(mut image)) = (status.style, image) {
                node.display = Display::Flex;
                image.image = assets.0.image(match style {
                    0 => UserEquipStaticAssetRole::NanoBlue,
                    1 => UserEquipStaticAssetRole::NanoRed,
                    _ => UserEquipStaticAssetRole::NanoYellow,
                });
            } else {
                node.display = Display::None;
            }
        }
        UserEquipUiElement::NanoStatusSlotLabel(slot) => {
            let status = &nano_projection.status[slot];
            node.display = if status.nano_id.is_some() {
                Display::Flex
            } else {
                Display::None
            };
            if let Some(mut localized) = localized {
                *localized = LocalizedText::new("ui.inventory.nano.slot", "NANO {ordinal}")
                    .with_arg("ordinal", (slot + 1).to_string());
            }
        }
        UserEquipUiElement::NanoStatusName(slot) => {
            let status = &nano_projection.status[slot];
            if let Some(mut localized) = localized {
                *localized = if status.nano_id.is_some() {
                    LocalizedText::new(
                        format!("content.nano.{}.name", status.nano_id.unwrap_or_default()),
                        &status.name,
                    )
                } else {
                    LocalizedText::new("ui.inventory.nano.empty", "EMPTY")
                };
            }
        }
        UserEquipUiElement::NanoStatusAttribute(slot) => {
            let status = &nano_projection.status[slot];
            node.display = if status.nano_id.is_some() {
                Display::Flex
            } else {
                Display::None
            };
            if let Some(mut localized) = localized {
                *localized = LocalizedText::new(
                    format!(
                        "content.nano.{}.attribute",
                        status.nano_id.unwrap_or_default()
                    ),
                    &status.attribute,
                );
            }
            if let (Some(style), Some(mut text_color)) = (status.style, text_color) {
                text_color.0 = match style {
                    0 => Color::srgb(0.0, 90.0 / 255.0, 1.0),
                    1 => Color::srgb(1.0, 0.0, 0.0),
                    _ => Color::srgb(1.0, 1.0, 0.0),
                };
            }
        }
        UserEquipUiElement::NanoStatusSkill(slot) => {
            if let Some(mut image) = image {
                let (handle, visible) = presentation_icon_handle(
                    &nano_projection.status[slot].skill_icon,
                    &asset_server,
                    &assets.0,
                );
                node.display = if nano_projection.status[slot].nano_id.is_some() && visible {
                    Display::Flex
                } else {
                    Display::None
                };
                image.image = handle;
            }
        }
        UserEquipUiElement::NanoStatusStamina(slot) => {
            node.display = if nano_projection.status[slot].nano_id.is_some() {
                Display::Flex
            } else {
                Display::None
            };
            node.width = px(90.0 * nano_projection.status[slot].stamina_fraction());
        }
        _ => {}
    }
}
