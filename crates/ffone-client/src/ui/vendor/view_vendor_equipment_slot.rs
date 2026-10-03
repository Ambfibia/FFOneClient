use super::*;

pub(super) fn spawn_vendor_equipment_slot(
    parent: &mut ChildSpawnerCommands,
    visual_index: usize,
    assets: &VendorUiAssets,
) {
    let top = USER_EQUIP_EQUIPMENT_CONTENT_RECT.top
        + visual_index as f32 * USER_EQUIP_EQUIPMENT_SLOT_STRIDE;
    parent
        .spawn((
            VendorUiElement::EquipmentSlotFrame(visual_index),
            VendorUiRect::new(
                USER_EQUIP_EQUIPMENT_CONTENT_RECT.left,
                top,
                USER_EQUIP_EQUIPMENT_SLOT_SIZE,
                USER_EQUIP_EQUIPMENT_SLOT_SIZE,
            )
            .node(),
            stretched_image(assets.image(VendorStaticAssetRole::SlotEmpty)),
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|slot_node| {
            slot_node.spawn((
                VendorUiElement::EquipmentSlotIcon(visual_index),
                Node {
                    display: Display::None,
                    ..VendorUiRect::new(
                        0.0,
                        0.0,
                        USER_EQUIP_EQUIPMENT_SLOT_SIZE,
                        USER_EQUIP_EQUIPMENT_SLOT_SIZE,
                    )
                    .node()
                },
                ImageNode::default(),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            slot_node.spawn((
                VendorUiElement::EquipmentSlotBadge(visual_index),
                Node {
                    display: Display::None,
                    ..VendorUiRect::new(
                        USER_EQUIP_COMBINED_BADGE_LEFT,
                        USER_EQUIP_COMBINED_BADGE_TOP,
                        USER_EQUIP_COMBINED_BADGE_SIZE,
                        USER_EQUIP_COMBINED_BADGE_SIZE,
                    )
                    .node()
                },
                stretched_image(assets.image(VendorStaticAssetRole::Combined)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            spawn_vendor_text(
                slot_node,
                VendorUiElement::EquipmentSlotLabel(visual_index),
                VendorUiRect::new(0.0, -2.0, 60.0, 19.0),
                &equipment_slot_label(visual_index),
                equipment_slot_localized(visual_index),
                assets.font.clone(),
                VendorUiTextStyle::EquipFontMiddleRight,
                Color::WHITE,
            );
        });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_vendor_text(
    parent: &mut ChildSpawnerCommands,
    marker: VendorUiElement,
    rect: VendorUiRect,
    value: &str,
    localized: LocalizedText,
    font: Handle<Font>,
    style: VendorUiTextStyle,
    color: Color,
) {
    let mut node = rect.node();
    style.apply_to_container(&mut node);
    if matches!(marker, VendorUiElement::TarosDigit(_)) {
        node.justify_content = JustifyContent::Center;
    }
    parent
        .spawn((node, Pickable::IGNORE, bevy::ui::FocusPolicy::Pass))
        .with_children(|container| {
            container.spawn((
                marker,
                Node {
                    max_width: percent(100),
                    flex_shrink: 0.0,
                    ..default()
                },
                Text::new(value),
                style.font(font),
                TextColor(color),
                style.layout(),
                style,
                UiTransform::from_translation(Val2::px(0.0, style.replacement_y_offset())),
                localized,
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
        });
}
