//! NPC letterbox, quest list and warp/service panels and their hover states.

use super::assets::{
    MISSION_BODY, MISSION_BOTTOM, MISSION_BUTTON, MISSION_TOP, MissionUiAssets, NPC_MULTI_WINDOW,
};
use super::components::{
    JournalCloseButton, MissionUiControl, MissionUiView, NpcActionButton, NpcMissionButton,
};
use super::geometry::{NPC_SINGLE_MISSION_RECT, NPC_WARP_AND_CLOSE_RECT};
use super::layout::{MissionUiRect, NPC_MISSION_ROW_LIMIT, NPC_UTILITY_ROW_LIMIT};
use super::widgets::{
    mission_text_color, mission_text_font, mission_text_state_color, npc_action_button, skin_label,
    stretch, styled_image,
};
use crate::{gui_skin::gui_style as mission_gui_style, localization::LocalizedText};
use bevy::{prelude::*, text::LineBreak};

/// `NpcIconMode.DoNpcWindow` paints `TopString` with `pMenuSkin.BigFont16`;
/// the NPC window's `M_Top`/`Sel_Bar` styles identify that skin.
pub(super) const NPC_TOP_NOTICE_SKIN: &str = "FusionFallInteractionSkin";
pub(super) const NPC_TOP_NOTICE_STYLE: &str = "BigFont16";

pub(super) fn spawn_npc_letterbox(root: &mut ChildSpawnerCommands, assets: &MissionUiAssets) {
    let notice_style = mission_gui_style(NPC_TOP_NOTICE_SKIN, NPC_TOP_NOTICE_STYLE)
        .expect("clean Retrobution NpcIconMode BigFont16 style must remain converted");
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: percent(100),
            height: percent(100),
            ..default()
        },
        MissionUiView::NpcLetterboxRoot,
        Pickable::IGNORE,
    ))
    .with_children(|bars| {
        bars.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(15.673_982),
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ));
        bars.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                bottom: px(0),
                width: percent(100),
                height: percent(15.673_982),
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ));
        // `GUI.Label(Rect(0, 0, Screen.width, 0.15673982 * Screen.height))`,
        // a MiddleCenter, zero-padding, word-wrapped white JEFFE 16 style.
        bars.spawn((
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(15.673_982),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                overflow: Overflow::clip(),
                ..default()
            },
            MissionUiView::NpcTopNotice,
            Pickable::IGNORE,
        ))
        .with_children(|bar| {
            bar.spawn((
                Text::new(""),
                LocalizedText::new(
                    "ui.transportation.registered.hub",
                    "New transportation hub registered in your NanoCom!",
                ),
                mission_text_font(assets, NPC_TOP_NOTICE_SKIN, NPC_TOP_NOTICE_STYLE),
                TextColor(mission_text_color(notice_style)),
                TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                MissionUiView::NpcTopNoticeText,
                Pickable::IGNORE,
            ));
        });
    });
}

pub(super) fn spawn_npc_quest(root: &mut ChildSpawnerCommands, assets: &MissionUiAssets) {
    root.spawn((
        NPC_SINGLE_MISSION_RECT.node(),
        UiTransform::default(),
        MissionUiView::NpcQuestRoot,
        Pickable::IGNORE,
    ))
    .with_children(|panel| {
        panel.spawn((
            MissionUiRect::new(0.0, 0.0, 332.0, 50.0).node(),
            styled_image(&assets.mission_top, MISSION_TOP),
            Pickable::IGNORE,
        ));
        panel.spawn((
            MissionUiRect::new(0.0, 50.0, 332.0, 25.0).node(),
            styled_image(&assets.mission_body, MISSION_BODY),
            MissionUiView::NpcCompletedBody,
            Pickable::IGNORE,
        ));
        panel.spawn((
            MissionUiRect::new(0.0, 50.0, 332.0, 25.0).node(),
            styled_image(&assets.mission_body, MISSION_BODY),
            MissionUiView::NpcAvailableBody,
            Pickable::IGNORE,
        ));
        panel.spawn((
            MissionUiRect::new(0.0, 107.0, 332.0, 76.0).node(),
            styled_image(&assets.mission_bottom, MISSION_BOTTOM),
            MissionUiView::NpcQuestBottom,
            Pickable::IGNORE,
        ));
        skin_label(
            panel,
            MissionUiRect::new(15.0, 10.0, 277.0, 40.0),
            "",
            "FusionFallInteractionSkin",
            "label",
            assets,
            Some(MissionUiView::NpcName),
            None,
        );
        panel.spawn((
            MissionUiRect::new(0.0, 50.0, 328.0, 20.0).node(),
            stretch(&assets.npc_sel_bar),
            MissionUiView::NpcCompletedHeader,
            Pickable::IGNORE,
        ));
        skin_label(
            panel,
            MissionUiRect::new(0.0, 50.0, 328.0, 20.0),
            "Active missions:",
            "FusionFallInteractionSkin",
            "Sel_Bar",
            assets,
            Some(MissionUiView::NpcCompletedHeader),
            None,
        );
        panel.spawn((
            MissionUiRect::new(0.0, 50.0, 328.0, 20.0).node(),
            stretch(&assets.npc_sel_bar),
            MissionUiView::NpcAvailableHeader,
            Pickable::IGNORE,
        ));
        skin_label(
            panel,
            MissionUiRect::new(0.0, 50.0, 328.0, 20.0),
            "Available missions:",
            "FusionFallInteractionSkin",
            "Sel_Bar",
            assets,
            Some(MissionUiView::NpcAvailableHeader),
            None,
        );
        for slot in 0..NPC_MISSION_ROW_LIMIT {
            panel
                .spawn((
                    Button,
                    MissionUiRect::new(7.0, 75.0 + slot as f32 * 32.0, 312.0, 27.0).node(),
                    styled_image(&assets.mission_button, MISSION_BUTTON),
                    MissionUiControl::NpcMissionRow(slot),
                    MissionUiView::NpcMissionRow(slot),
                    NpcMissionButton,
                ))
                .with_children(|row| {
                    row.spawn((
                        MissionUiRect::new(10.0, 5.0, 17.0, 17.0).node(),
                        ImageNode::new(assets.npc_world_mission_icon.clone()),
                        MissionUiView::NpcMissionRowIcon(slot),
                        Pickable::IGNORE,
                    ));
                    skin_label(
                        row,
                        MissionUiRect::new(25.0, 0.0, 287.0, 27.0),
                        "",
                        "FusionFallInteractionSkin",
                        "m_button",
                        assets,
                        Some(MissionUiView::NpcMissionRowText(slot)),
                        None,
                    );
                });
        }
        npc_action_button(
            panel,
            MissionUiRect::new(60.0, 121.0, 212.0, 48.0),
            "WARP",
            &assets.npcicon_warp,
            MissionUiControl::NpcUtility(0),
            Some(MissionUiView::NpcQuestUtilityButton),
            Some((
                MissionUiView::NpcQuestUtilityText,
                MissionUiView::NpcQuestUtilityIcon,
            )),
            assets,
        );
        npc_action_button(
            panel,
            MissionUiRect::new(60.0, 121.0, 212.0, 48.0),
            "CLOSE",
            &assets.npcicon_exit,
            MissionUiControl::NpcClose,
            Some(MissionUiView::NpcQuestCloseButton),
            None,
            assets,
        );
        panel.spawn((
            Button,
            MissionUiRect::new(296.0, 0.5, 32.0, 32.0).node(),
            stretch(&assets.journal_close),
            MissionUiControl::NpcClose,
            JournalCloseButton,
        ));
    });
}

pub(super) fn spawn_npc_warp(root: &mut ChildSpawnerCommands, assets: &MissionUiAssets) {
    root.spawn((
        NPC_WARP_AND_CLOSE_RECT.node(),
        styled_image(&assets.npc_multi_window, NPC_MULTI_WINDOW),
        UiTransform::default(),
        MissionUiView::NpcWarpRoot,
        Pickable::IGNORE,
    ))
    .with_children(|panel| {
        skin_label(
            panel,
            MissionUiRect::new(15.0, 10.0, 187.0, 40.0),
            "",
            "FusionFallInteractionSkin",
            "label",
            assets,
            Some(MissionUiView::NpcName),
            None,
        );
        for slot in 0..NPC_UTILITY_ROW_LIMIT {
            npc_action_button(
                panel,
                MissionUiRect::new(15.0, 53.0 + slot as f32 * 54.0, 212.0, 48.0),
                "",
                &assets.npcicon_warp,
                MissionUiControl::NpcUtility(slot),
                Some(MissionUiView::NpcUtilityButton(slot)),
                Some((
                    MissionUiView::NpcUtilityText(slot),
                    MissionUiView::NpcUtilityIcon(slot),
                )),
                assets,
            );
        }
        npc_action_button(
            panel,
            MissionUiRect::new(15.0, 107.0, 212.0, 48.0),
            "CLOSE",
            &assets.npcicon_exit,
            MissionUiControl::NpcClose,
            Some(MissionUiView::NpcCloseButton),
            None,
            assets,
        );
        panel.spawn((
            Button,
            MissionUiRect::new(206.0, 0.5, 32.0, 32.0).node(),
            stretch(&assets.journal_close),
            MissionUiControl::NpcClose,
            JournalCloseButton,
        ));
    });
}

pub(super) fn bind_mission_button_hover(
    assets: Res<MissionUiAssets>,
    mut buttons: Query<
        (&Interaction, &Children, &mut ImageNode),
        (With<NpcMissionButton>, Changed<Interaction>),
    >,
    mut text_colors: Query<&mut TextColor>,
) {
    let hovered_style = mission_gui_style("FusionFallInteractionSkin", "m_button")
        .expect("Retrobution mission button style");
    for (interaction, children, mut image) in &mut buttons {
        let hovered = matches!(interaction, Interaction::Hovered | Interaction::Pressed);
        image.image = if hovered {
            assets.mission_button_over.clone()
        } else {
            assets.mission_button.clone()
        };
        let color =
            mission_text_state_color(hovered_style, if hovered { "hover" } else { "normal" });
        for child in children.iter() {
            if let Ok(mut text_color) = text_colors.get_mut(child) {
                **text_color = color;
            }
        }
    }
}

pub(super) fn bind_npc_action_button_hover(
    assets: Res<MissionUiAssets>,
    mut buttons: Query<
        (&Interaction, &Children, &mut ImageNode),
        (With<NpcActionButton>, Changed<Interaction>),
    >,
    mut text_colors: Query<&mut TextColor>,
) {
    let button_style = mission_gui_style("FusionFallInteractionSkin", "button")
        .expect("Retrobution NPC action button style");
    for (interaction, children, mut image) in &mut buttons {
        let hovered = matches!(interaction, Interaction::Hovered | Interaction::Pressed);
        image.image = if hovered {
            assets.npc_blue_button_over.clone()
        } else {
            assets.npc_blue_button.clone()
        };
        let color =
            mission_text_state_color(button_style, if hovered { "hover" } else { "normal" });
        for child in children.iter() {
            if let Ok(mut text_color) = text_colors.get_mut(child) {
                **text_color = color;
            }
        }
    }
}
