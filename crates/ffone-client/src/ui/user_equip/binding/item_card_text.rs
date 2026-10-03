//! Item card text case binding.

use super::super::catalog::UserEquipSlotEndpoint;
use super::super::components::{UserEquipBoundTextTarget, UserEquipUiElement};
use super::super::item_projection::{UserEquipItemModeProjection, user_equip_item_rating_colors};
use super::super::popup_state::{
    UserEquipItemPopupState, UserEquipPopupLayoutVariant, user_equip_popup_layout_variant,
};
use crate::tutorial_mission_content::TutorialMissionContent;
use bevy::prelude::*;

pub(in super::super) fn bind_item_card_text_case(
    mut commands: Commands,
    popup: Res<UserEquipItemPopupState>,
    projection: Option<Res<UserEquipItemModeProjection>>,
    content: Option<Res<TutorialMissionContent>>,
    labels: Query<(&UserEquipUiElement, &UserEquipBoundTextTarget)>,
    cases: Query<&crate::localization::LocalizedTextCase>,
    mut colors: Query<&mut TextColor, Without<UserEquipUiElement>>,
) {
    let uppercase = projection
        .as_deref()
        .and_then(|p| user_equip_popup_layout_variant(&popup, p, content.as_deref()))
        .is_some_and(|v| {
            matches!(
                v,
                UserEquipPopupLayoutVariant::GeneralStack | UserEquipPopupLayoutVariant::GeneralGum
            )
        });
    let item = popup
        .selected()
        .and_then(|e| projection.as_deref()?.item_at(e));
    let rating_colors = item.zip(content.as_deref()).map(|(item, content)| {
        user_equip_item_rating_colors(
            content,
            Color::WHITE,
            item.item,
            usize::try_from(item.item.item_type)
                .ok()
                .and_then(|slot| projection.as_deref()?.equipped_item(slot)),
            matches!(
                popup.selected(),
                Some(UserEquipSlotEndpoint::Equipment { .. })
            ),
        )
    });
    for (element, target) in &labels {
        // The layout owner has no TextColor; the visible text is its bound child.
        if let UserEquipUiElement::ItemPopupField(index @ 2..=4) = element
            && let Some(rating_colors) = rating_colors
            && let Ok(mut color) = colors.get_mut(target.0)
        {
            color.set_if_neq(TextColor(rating_colors[index - 2]));
        }
        if let Some(item) = item
            && let UserEquipUiElement::ItemPopupField(index @ (11 | 13)) = element
            && let Ok(mut color) = colors.get_mut(target.0)
        {
            let combined =
                (0..=3).contains(&item.item.item_type) && (item.item.option >> 16) as i16 > 0;
            let value = if *index == 11 {
                if matches!(item.item.item_type, 0 | 10) {
                    Color::WHITE
                } else {
                    Color::srgb(0., 0., 1.)
                }
            } else if !combined
                && content
                    .as_deref()
                    .and_then(|c| {
                        c.gameplay_user_equip_item_detail(item.item.item_type, item.item.item_id)
                    })
                    .is_some_and(|d| d.tradeable)
            {
                Color::srgb(0., 1., 0.)
            } else {
                Color::srgb(1., 0., 0.)
            };
            if color.0 != value {
                color.0 = value;
            }
        }

        if matches!(element, UserEquipUiElement::NanoViewerRequirementLabel(_, 1)) {
            if cases.get(target.0).is_err() {
                commands.entity(target.0)
                    .insert(crate::localization::LocalizedTextCase::Uppercase);
            }
            continue;
        }
        if !matches!(element, UserEquipUiElement::ItemPopupIdentity) {
            continue;
        }
        let present = cases.get(target.0).is_ok();
        if uppercase && !present {
            commands
                .entity(target.0)
                .insert(crate::localization::LocalizedTextCase::Uppercase);
        } else if !uppercase && present {
            commands
                .entity(target.0)
                .remove::<crate::localization::LocalizedTextCase>();
        }
    }
}
