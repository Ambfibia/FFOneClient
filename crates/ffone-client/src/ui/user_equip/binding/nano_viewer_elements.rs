//! Binding of the Nano viewer popup.

use super::super::asset_contract::UserEquipStaticAssetRole;
use super::super::components::UserEquipUiElement;
use super::super::geometry::{UserEquipUiRect, user_equip_nano_popup_content_rect};
use super::super::images::{bind_rect, presentation_icon_handle};
use super::UserEquipBindContext;
use crate::localization::LocalizedText;
use bevy::prelude::*;

/// Nano viewer frame, icon, name, attribute, description and current power.
#[allow(clippy::too_many_arguments)]
pub(super) fn bind_nano_viewer_identity_element(
    context: &UserEquipBindContext<'_, '_, '_>,
    element: &UserEquipUiElement,
    mut node: Mut<Node>,
    image: Option<Mut<ImageNode>>,
    localized: Option<Mut<LocalizedText>>,
    _text_color: Option<Mut<TextColor>>,
    _text_layout: Option<Mut<TextLayout>>,
    _interaction: Option<&Interaction>,
) {
    let UserEquipBindContext {
        asset_server,
        assets,
        window,
        nano_mode,
        selected_nano,
        ..
    } = *context;
    match *element {
        UserEquipUiElement::NanoViewer => {
            node.display = if nano_mode && selected_nano.is_some() {
                Display::Flex
            } else {
                Display::None
            };
            if let Some(entry) = selected_nano {
                let rect = if !entry.owned {
                    UserEquipUiRect::new(
                        window.width() * 0.5 + 100.0,
                        window.height() * 0.5 - 230.0,
                        356.0,
                        460.0,
                    )
                } else if entry.equipped {
                    UserEquipUiRect::new(
                        window.width() * 0.5 - 402.5,
                        window.height() * 0.5 - 315.0,
                        370.0,
                        640.0,
                    )
                } else {
                    UserEquipUiRect::new(
                        window.width() * 0.5 + 100.0,
                        window.height() * 0.5 - 310.0,
                        360.0,
                        620.0,
                    )
                };
                bind_rect(&mut node, rect);
                if let Some(mut image) = image {
                    image.image = assets.0.image(if entry.equipped {
                        UserEquipStaticAssetRole::NanoPopupEquipped
                    } else if entry.owned {
                        UserEquipStaticAssetRole::NanoPopup
                    } else {
                        UserEquipStaticAssetRole::NanoPopupNext
                    });
                }
            }
        }
        UserEquipUiElement::NanoViewerInnerBackdrop => {
            node.display = if selected_nano.is_some_and(|entry| entry.equipped) {
                Display::Flex
            } else {
                Display::None
            };
        }
        UserEquipUiElement::NanoViewerEquippedTitle => {
            node.display = if selected_nano.is_some_and(|entry| entry.equipped) {
                Display::Flex
            } else {
                Display::None
            };
            if let Some(mut localized) = localized {
                *localized = LocalizedText::new("ui.inventory.equipped", " EQUIPPED");
            }
        }
        UserEquipUiElement::NanoViewerIcon => {
            if let (Some(entry), Some(mut image)) = (selected_nano, image) {
                let (handle, visible) =
                    presentation_icon_handle(&entry.viewer_icon, &asset_server, &assets.0);
                node.display = if visible {
                    Display::Flex
                } else {
                    Display::None
                };
                image.image = handle;
                bind_rect(
                    &mut node,
                    user_equip_nano_popup_content_rect(
                        UserEquipUiRect::new(17.0, 15.0, 62.0, 62.0),
                        entry.equipped,
                    ),
                );
            } else {
                node.display = Display::None;
            }
        }
        UserEquipUiElement::NanoViewerName => {
            if let (Some(entry), Some(mut localized)) = (selected_nano, localized) {
                *localized = LocalizedText::new(
                    format!("content.nano.{}.name", entry.nano_id),
                    &entry.name,
                );
                let rect = if entry.owned {
                    UserEquipUiRect::new(88.0, 15.0, 200.0, 15.0)
                } else {
                    UserEquipUiRect::new(86.0, 13.0, 200.0, 15.0)
                };
                bind_rect(
                    &mut node,
                    user_equip_nano_popup_content_rect(rect, entry.equipped),
                );
            }
        }
        UserEquipUiElement::NanoViewerAttribute => {
            if let (Some(entry), Some(mut localized)) = (selected_nano, localized) {
                *localized = LocalizedText::new(
                    format!("content.nano.{}.attribute", entry.nano_id),
                    &entry.attribute,
                );
                let rect = if entry.owned {
                    UserEquipUiRect::new(88.0, 47.0, 100.0, 15.0)
                } else {
                    UserEquipUiRect::new(86.0, 48.0, 100.0, 15.0)
                };
                bind_rect(
                    &mut node,
                    user_equip_nano_popup_content_rect(rect, entry.equipped),
                );
            }
        }
        UserEquipUiElement::NanoViewerDescription => {
            if let (Some(entry), Some(mut localized)) = (selected_nano, localized) {
                *localized = LocalizedText::new(
                    format!("content.nano.{}.description", entry.nano_id),
                    &entry.description,
                );
                let rect = if entry.owned {
                    UserEquipUiRect::new(17.0, 85.0, 335.0, 27.0)
                } else {
                    UserEquipUiRect::new(12.0, 94.0, 335.0, 27.0)
                };
                bind_rect(
                    &mut node,
                    user_equip_nano_popup_content_rect(rect, entry.equipped),
                );
            }
        }
        UserEquipUiElement::NanoViewerCurrentPower => {
            if let Some(entry) = selected_nano {
                bind_rect(
                    &mut node,
                    user_equip_nano_popup_content_rect(
                        UserEquipUiRect::new(20.0, 120.0, 100.0, 10.0),
                        entry.equipped,
                    ),
                );
            }
            node.display = if selected_nano.is_some_and(|entry| entry.owned) {
                Display::Flex
            } else {
                Display::None
            };
            if let Some(mut localized) = localized {
                *localized =
                    LocalizedText::new("ui.inventory.nano.current_power", "current power");
            }
        }
        _ => {}
    }
}

/// Nano viewer skills, requirement prompts/bars/values, station notice and close button.
#[allow(clippy::too_many_arguments)]
pub(super) fn bind_nano_viewer_skill_element(
    context: &UserEquipBindContext<'_, '_, '_>,
    element: &UserEquipUiElement,
    mut node: Mut<Node>,
    image: Option<Mut<ImageNode>>,
    localized: Option<Mut<LocalizedText>>,
    _text_color: Option<Mut<TextColor>>,
    _text_layout: Option<Mut<TextLayout>>,
    _interaction: Option<&Interaction>,
) {
    let UserEquipBindContext {
        asset_server,
        assets,
        state,
        presentation,
        projection,
        environment,
        selected_nano,
        ..
    } = *context;
    match *element {
        UserEquipUiElement::NanoViewerSkillIcon(index) => {
            if let (Some(entry), Some(mut image)) = (selected_nano, image) {
                let skill = &entry.skills[index];
                let (handle, visible) =
                    presentation_icon_handle(&skill.icon, &asset_server, &assets.0);
                image.image = handle;
                let owned = [
                    UserEquipUiRect::new(18.0, 133.0, 35.0, 35.0),
                    UserEquipUiRect::new(16.0, 222.0, 35.0, 35.0),
                    UserEquipUiRect::new(15.0, 383.0, 35.0, 35.0),
                ];
                let next = [
                    UserEquipUiRect::new(18.0, 139.0, 35.0, 35.0),
                    UserEquipUiRect::new(18.0, 221.0, 35.0, 35.0),
                    UserEquipUiRect::new(18.0, 304.0, 35.0, 35.0),
                ];
                let rect = if entry.owned {
                    owned[index]
                } else {
                    next[index]
                };
                bind_rect(
                    &mut node,
                    user_equip_nano_popup_content_rect(rect, entry.equipped),
                );
                node.display = if visible {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        UserEquipUiElement::NanoViewerSkillName(index)
        | UserEquipUiElement::NanoViewerSkillType(index)
        | UserEquipUiElement::NanoViewerSkillDescription(index) => {
            if let (Some(entry), Some(mut localized)) = (selected_nano, localized) {
                let skill = &entry.skills[index];
                let (value, rect) = match *element {
                    UserEquipUiElement::NanoViewerSkillName(_) => {
                        let owned = [(53.0, 135.0), (53.0, 227.0), (53.0, 387.0)];
                        let next = [(50.0, 143.0), (51.0, 226.0), (51.0, 309.0)];
                        let (x, y) = if entry.owned {
                            owned[index]
                        } else {
                            next[index]
                        };
                        (
                            skill.name.to_uppercase(),
                            UserEquipUiRect::new(x, y, 200.0, 15.0),
                        )
                    }
                    UserEquipUiElement::NanoViewerSkillType(_) => {
                        let owned = [(53.0, 151.0), (53.0, 243.0), (53.0, 403.0)];
                        let next = [(50.0, 158.0), (51.0, 241.0), (51.0, 324.0)];
                        let (x, y) = if entry.owned {
                            owned[index]
                        } else {
                            next[index]
                        };
                        (
                            skill.type_label.to_uppercase(),
                            UserEquipUiRect::new(x, y, 150.0, 12.0),
                        )
                    }
                    _ => {
                        let owned = [
                            (20.0, 173.0, 317.0, 27.0),
                            (20.0, 268.0, 310.0, 40.0),
                            (20.0, 430.0, 310.0, 40.0),
                        ];
                        let next = [
                            (20.0, 180.0, 317.0, 25.0),
                            (20.0, 261.0, 315.0, 25.0),
                            (20.0, 345.0, 315.0, 25.0),
                        ];
                        let (x, y, w, h) = if entry.owned {
                            owned[index]
                        } else {
                            next[index]
                        };
                        (
                            skill.description.to_uppercase(),
                            UserEquipUiRect::new(x, y, w, h),
                        )
                    }
                };
                bind_rect(
                    &mut node,
                    user_equip_nano_popup_content_rect(rect, entry.equipped),
                );
                let field = match *element {
                    UserEquipUiElement::NanoViewerSkillName(_) => "name",
                    UserEquipUiElement::NanoViewerSkillType(_) => "type_label",
                    _ => "description",
                };
                *localized = LocalizedText::new(
                    if skill.tune_id > 0 {
                        format!("content.nano_tune.{}.{field}", skill.tune_id)
                    } else {
                        format!("content.nano_skill.{}.{field}", skill.skill_id)
                    },
                    value,
                );
            }
        }
        UserEquipUiElement::NanoViewerRequirementPrompt(skill_index) => {
            node.display = if selected_nano.is_some_and(|entry| entry.owned) {
                Display::Flex
            } else {
                Display::None
            };
            if let Some(entry) = selected_nano {
                let y = if skill_index == 1 { 310.0 } else { 474.0 };
                bind_rect(
                    &mut node,
                    user_equip_nano_popup_content_rect(
                        UserEquipUiRect::new(70.0, y, 250.0, 13.0),
                        entry.equipped,
                    ),
                );
            }
            if let Some(mut localized) = localized {
                *localized = LocalizedText::new(
                    "ui.inventory.nano.requirements_prompt",
                    "To activate this power you need:",
                );
            }
            node.display = if selected_nano.is_some_and(|entry| entry.owned) {
                Display::Flex
            } else {
                Display::None
            };
        }
        UserEquipUiElement::NanoViewerRequirementLabel(skill_index, requirement) => {
            node.display = if selected_nano.is_some_and(|entry| entry.owned) {
                Display::Flex
            } else {
                Display::None
            };
            if let Some(entry) = selected_nano {
                let y = match (skill_index, requirement) {
                    (1, 0) => 328.0,
                    (1, _) => 345.0,
                    (2, 0) => 493.0,
                    _ => 509.0,
                };
                bind_rect(
                    &mut node,
                    user_equip_nano_popup_content_rect(
                        UserEquipUiRect::new(20.0, y, 140.0, 13.0),
                        entry.equipped,
                    ),
                );
                if let Some(mut localized) = localized {
                    *localized = if requirement == 0 {
                        LocalizedText::new("ui.inventory.nano.fusion_matter", "FUSION MATTER")
                    } else {
                        let skill = &entry.skills[skill_index];
                        environment.content.as_deref()
                            .and_then(|content| {
                                content.gameplay_user_equip_item_text(
                                    7, i16::try_from(skill.required_item_id).ok()?,
                                )
                            })
                            .map(|text| text.0)
                            .unwrap_or_else(|| LocalizedText::new(
                                format!("content.tabledata.general_item.item_string.{}.str_name", skill.required_item_id),
                                &skill.required_item_name,
                            ))
                    };
                }
            }
            node.display = if selected_nano.is_some_and(|entry| entry.owned) {
                Display::Flex
            } else {
                Display::None
            };
        }
        UserEquipUiElement::NanoViewerRequirementBar(skill_index, requirement) => {
            node.display = if selected_nano.is_some_and(|entry| entry.owned) {
                Display::Flex
            } else {
                Display::None
            };
            if let Some(entry) = selected_nano {
                let skill = &entry.skills[skill_index];
                let current = if requirement == 0 {
                    presentation.fusion_matter
                } else {
                    projection
                        .as_deref()
                        .map(|projection| {
                            projection
                                .inventory
                                .iter()
                                .filter(|slot| {
                                    slot.item.item.item_type == 7
                                        && i32::from(slot.item.item.item_id)
                                            == skill.required_item_id
                                })
                                .map(|slot| slot.item.item.option.max(0))
                                .sum::<i32>()
                        })
                        .unwrap_or_default()
                };
                let required = if requirement == 0 {
                    environment
                        .content
                        .as_deref()
                        .and_then(|content| {
                            content.gameplay_nano_tune_fusion_matter(presentation.level)
                        })
                        .unwrap_or_default()
                } else {
                    skill.required_item_count
                };
                let ratio = if required <= 0 {
                    1.0
                } else {
                    (current.max(0) as f32 / required as f32).min(1.0)
                };
                let (x, y) = match (skill_index, requirement) {
                    (1, 0) => (177.0, 331.0),
                    (1, _) => (177.0, 348.0),
                    (2, 0) => (176.0, 492.0),
                    _ => (176.0, 509.0),
                };
                bind_rect(
                    &mut node,
                    user_equip_nano_popup_content_rect(
                        UserEquipUiRect::new(x, y, 126.0 * ratio, 9.0),
                        entry.equipped,
                    ),
                );
            }
            node.display = if selected_nano.is_some_and(|entry| entry.owned) {
                Display::Flex
            } else {
                Display::None
            };
        }
        UserEquipUiElement::NanoViewerRequirementValue(skill_index, requirement) => {
            node.display = if selected_nano.is_some_and(|entry| entry.owned) {
                Display::Flex
            } else {
                Display::None
            };
            if let Some(entry) = selected_nano {
                let skill = &entry.skills[skill_index];
                let current = if requirement == 0 {
                    presentation.fusion_matter
                } else {
                    projection
                        .as_deref()
                        .map(|projection| {
                            projection
                                .inventory
                                .iter()
                                .filter(|slot| {
                                    slot.item.item.item_type == 7
                                        && i32::from(slot.item.item.item_id)
                                            == skill.required_item_id
                                })
                                .map(|slot| slot.item.item.option.max(0))
                                .sum::<i32>()
                        })
                        .unwrap_or_default()
                };
                let required = if requirement == 0 {
                    environment
                        .content
                        .as_deref()
                        .and_then(|content| {
                            content.gameplay_nano_tune_fusion_matter(presentation.level)
                        })
                        .unwrap_or_default()
                } else {
                    skill.required_item_count
                };
                let y = match (skill_index, requirement) {
                    (1, 0) => 329.0,
                    (1, _) => 346.0,
                    (2, 0) => 490.0,
                    _ => 508.0,
                };
                bind_rect(
                    &mut node,
                    user_equip_nano_popup_content_rect(
                        UserEquipUiRect::new(177.0, y, 129.0, 13.0),
                        entry.equipped,
                    ),
                );
                if let Some(mut localized) = localized {
                    *localized = LocalizedText::new(
                        "ui.inventory.nano.requirement_ratio",
                        "{current}/{required}",
                    )
                    .with_arg("current", current.to_string())
                    .with_arg("required", required.to_string());
                }
            }
            node.display = if selected_nano.is_some_and(|entry| entry.owned) {
                Display::Flex
            } else {
                Display::None
            };
        }
        UserEquipUiElement::NanoViewerStationNotice => {
            node.display = if state.nano_station_npc().is_none()
                && selected_nano.is_some_and(|entry| entry.owned)
            {
                Display::Flex
            } else {
                Display::None
            };
            if let Some(mut localized) = localized {
                *localized = LocalizedText::new(
                    "ui.inventory.nano.station_notice",
                    "To equip this Nano or change its current power, you must use a Nano Station.",
                );
            }
            if let Some(entry) = selected_nano {
                bind_rect(
                    &mut node,
                    user_equip_nano_popup_content_rect(
                        UserEquipUiRect::new(37.0, 565.0, 280.0, 33.0),
                        entry.equipped,
                    ),
                );
            }
            node.display = if state.nano_station_npc().is_none()
                && selected_nano.is_some_and(|entry| entry.owned)
            {
                Display::Flex
            } else {
                Display::None
            };
        }
        UserEquipUiElement::NanoViewerClose => {
            bind_rect(
                &mut node,
                if let Some(entry) = selected_nano.filter(|entry| entry.owned) {
                    user_equip_nano_popup_content_rect(
                        UserEquipUiRect::new(327.0, 0.0, 32.0, 32.0),
                        entry.equipped,
                    )
                } else {
                    UserEquipUiRect::new(323.0, 0.0, 32.0, 32.0)
                },
            );
        }
        _ => {}
    }
}
