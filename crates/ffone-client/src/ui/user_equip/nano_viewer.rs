//! Nano viewer popup spawning.

use super::asset_contract::UserEquipStaticAssetRole;
use super::components::{UserEquipNanoViewerCloseControl, UserEquipUiAssets, UserEquipUiElement};
use super::geometry::UserEquipUiRect;
use super::images::stretched_image;
use super::nano_station;
use crate::localization::LocalizedText;
use bevy::{prelude::*, text::LineHeight};

pub(super) fn spawn_nano_viewer(parent: &mut ChildSpawnerCommands, assets: &UserEquipUiAssets) {
    parent
        .spawn((
            UserEquipUiElement::NanoViewer,
            crate::ui::shared::controller::ControllerUiBoundary,
            Node {
                display: Display::None,
                overflow: Overflow::clip(),
                ..UserEquipUiRect::new(0.0, 0.0, 360.0, 620.0).node()
            },
            stretched_image(assets.image(UserEquipStaticAssetRole::NanoPopup)),
            ZIndex(100),
        ))
        .with_children(|viewer| {
            nano_station::spawn_controls(viewer, assets);
            viewer.spawn((
                UserEquipUiElement::NanoViewerInnerBackdrop,
                Node {
                    display: Display::None,
                    ..UserEquipUiRect::new(2.5, 18.0, 360.0, 620.0).node()
                },
                stretched_image(assets.image(UserEquipStaticAssetRole::NanoPopup)),
                Pickable::IGNORE,
            ));
            spawn_nano_viewer_text(
                viewer,
                UserEquipUiElement::NanoViewerEquippedTitle,
                UserEquipUiRect::new(13.5, 3.0, 100.0, 30.0),
                assets,
                16.0,
                Color::srgb(0.0, 0.2, 0.4),
                Justify::Left,
                false,
            );
            viewer.spawn((
                Button,
                UserEquipUiElement::NanoViewerClose,
                UserEquipNanoViewerCloseControl,
                UserEquipUiRect::new(327.0, 0.0, 32.0, 32.0).node(),
                stretched_image(assets.image(UserEquipStaticAssetRole::Close)),
            ));
            viewer.spawn((
                UserEquipUiElement::NanoViewerIcon,
                UserEquipUiRect::new(17.0, 15.0, 62.0, 62.0).node(),
                ImageNode::default(),
                Pickable::IGNORE,
            ));
            spawn_nano_viewer_text(
                viewer,
                UserEquipUiElement::NanoViewerName,
                UserEquipUiRect::new(88.0, 15.0, 200.0, 15.0),
                assets,
                14.0,
                Color::srgb(0.8, 1.0, 1.0),
                Justify::Left,
                false,
            );
            spawn_nano_viewer_text(
                viewer,
                UserEquipUiElement::NanoViewerAttribute,
                UserEquipUiRect::new(88.0, 47.0, 100.0, 15.0),
                assets,
                14.0,
                Color::srgb(1.0, 1.0, 0.0),
                Justify::Left,
                false,
            );
            spawn_nano_viewer_text(
                viewer,
                UserEquipUiElement::NanoViewerDescription,
                UserEquipUiRect::new(17.0, 85.0, 335.0, 27.0),
                assets,
                8.0,
                Color::srgb(0.0, 1.0, 1.0),
                Justify::Left,
                true,
            );
            spawn_nano_viewer_text(
                viewer,
                UserEquipUiElement::NanoViewerCurrentPower,
                UserEquipUiRect::new(20.0, 120.0, 100.0, 10.0),
                assets,
                8.0,
                Color::srgb(0.0, 0.2, 0.4),
                Justify::Center,
                false,
            );
            let rows = [
                (
                    UserEquipUiRect::new(18.0, 133.0, 35.0, 35.0),
                    53.0,
                    135.0,
                    151.0,
                    20.0,
                    173.0,
                    317.0,
                    27.0,
                ),
                (
                    UserEquipUiRect::new(16.0, 222.0, 35.0, 35.0),
                    53.0,
                    227.0,
                    243.0,
                    20.0,
                    268.0,
                    310.0,
                    40.0,
                ),
                (
                    UserEquipUiRect::new(15.0, 383.0, 35.0, 35.0),
                    53.0,
                    387.0,
                    403.0,
                    20.0,
                    430.0,
                    310.0,
                    40.0,
                ),
            ];
            for (index, (icon_rect, text_x, name_y, type_y, desc_x, desc_y, desc_w, desc_h)) in
                rows.into_iter().enumerate()
            {
                viewer.spawn((
                    UserEquipUiElement::NanoViewerSkillIcon(index),
                    icon_rect.node(),
                    ImageNode::default(),
                    Pickable::IGNORE,
                ));
                spawn_nano_viewer_text(
                    viewer,
                    UserEquipUiElement::NanoViewerSkillName(index),
                    UserEquipUiRect::new(text_x, name_y, 200.0, 15.0),
                    assets,
                    14.0,
                    Color::srgb(0.8, 1.0, 1.0),
                    Justify::Left,
                    false,
                );
                spawn_nano_viewer_text(
                    viewer,
                    UserEquipUiElement::NanoViewerSkillType(index),
                    UserEquipUiRect::new(text_x, type_y, 150.0, 12.0),
                    assets,
                    8.0,
                    Color::srgb(1.0, 1.0, 0.0),
                    Justify::Left,
                    false,
                );
                spawn_nano_viewer_text(
                    viewer,
                    UserEquipUiElement::NanoViewerSkillDescription(index),
                    UserEquipUiRect::new(desc_x, desc_y, desc_w, desc_h),
                    assets,
                    8.0,
                    Color::srgb(0.0, 1.0, 1.0),
                    Justify::Left,
                    true,
                );
            }
            for (skill_index, prompt_y, label_y, bar_x, bar_y, value_y) in [
                (
                    1,
                    310.0,
                    [328.0, 345.0],
                    177.0,
                    [331.0, 348.0],
                    [329.0, 346.0],
                ),
                (
                    2,
                    474.0,
                    [493.0, 509.0],
                    176.0,
                    [492.0, 509.0],
                    [490.0, 508.0],
                ),
            ] {
                spawn_nano_viewer_text(
                    viewer,
                    UserEquipUiElement::NanoViewerRequirementPrompt(skill_index),
                    UserEquipUiRect::new(70.0, prompt_y, 250.0, 13.0),
                    assets,
                    8.0,
                    Color::srgb(0.0, 1.0, 1.0),
                    Justify::Left,
                    false,
                );
                for requirement in 0..2 {
                    spawn_nano_viewer_text(
                        viewer,
                        UserEquipUiElement::NanoViewerRequirementLabel(skill_index, requirement),
                        UserEquipUiRect::new(20.0, label_y[requirement], 140.0, 13.0),
                        assets,
                        8.0,
                        Color::srgb(0.0, 1.0, 1.0),
                        Justify::Right,
                        false,
                    );
                    viewer.spawn((
                        UserEquipUiElement::NanoViewerRequirementBar(skill_index, requirement),
                        UserEquipUiRect::new(bar_x, bar_y[requirement], 126.0, 9.0).node(),
                        stretched_image(assets.image(if requirement == 0 {
                            UserEquipStaticAssetRole::NanoFmBar
                        } else {
                            UserEquipStaticAssetRole::NanoItemBar
                        })),
                        Pickable::IGNORE,
                    ));
                    spawn_nano_viewer_text(
                        viewer,
                        UserEquipUiElement::NanoViewerRequirementValue(skill_index, requirement),
                        UserEquipUiRect::new(177.0, value_y[requirement], 129.0, 13.0),
                        assets,
                        8.0,
                        Color::srgb(0.0, 1.0, 1.0),
                        Justify::Center,
                        false,
                    );
                }
            }
            spawn_nano_viewer_text(
                viewer,
                UserEquipUiElement::NanoViewerStationNotice,
                UserEquipUiRect::new(37.0, 565.0, 280.0, 33.0),
                assets,
                8.0,
                Color::srgb(0.8, 1.0, 1.0),
                Justify::Center,
                true,
            );
        });
}

pub(super) fn spawn_nano_viewer_text(
    parent: &mut ChildSpawnerCommands,
    element: UserEquipUiElement,
    rect: UserEquipUiRect,
    assets: &UserEquipUiAssets,
    font_size: f32,
    color: Color,
    justify: Justify,
    wrap: bool,
) {
    let mut node = rect.node();
    node.overflow = Overflow::clip();
    let uppercase = matches!(
        element,
        UserEquipUiElement::NanoViewerName
            | UserEquipUiElement::NanoViewerAttribute
            | UserEquipUiElement::NanoViewerDescription
            | UserEquipUiElement::NanoViewerSkillName(_)
            | UserEquipUiElement::NanoViewerSkillType(_)
            | UserEquipUiElement::NanoViewerSkillDescription(_)
    );
    let mut entity = parent.spawn((
        element,
        node,
        Text::new(""),
        (
            TextFont {
                font: (assets.font.clone()).into(),
                font_size: (font_size).into(),
                ..default()
            },
            LineHeight::Px(if font_size <= 8.0 { 10.968 } else { 13.71 }),
        ),
        TextColor(color),
        TextLayout::new(
            justify,
            if wrap {
                LineBreak::WordBoundary
            } else {
                LineBreak::NoWrap
            },
        ),
        LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", ""),
        Pickable::IGNORE,
    ));
    if uppercase {
        entity.insert(crate::localization::LocalizedTextCase::Uppercase);
    }
}
