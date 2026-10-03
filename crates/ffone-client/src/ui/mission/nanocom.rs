//! Nanocom menu and chat quick-menu spawning, hover states and keyboard toggle.

use super::assets::MissionUiAssets;
use super::buttons::menu_interaction_state;
use super::components::{
    ChatQuickMenuButton, MenuButtonStyle, MissionUiControl, MissionUiView, NanocomCloseButton,
    NanocomMenuButton,
};
use super::geometry::{
    CHAT_EMOTE_CONTAINER_HEIGHT, CHAT_QUICK_GROUP_RECT, CHAT_QUICK_MENU_RECT,
    CHAT_QUICK_VEHICLE_RECT, CHAT_QUICK_WARP_RECT, CHAT_WINDOW_HEIGHT, NANOCOM_CLOSE_RECT,
    NANOCOM_MENU_RECT,
};
use super::labels::NANOCOM_MENU_LABELS;
use super::layout::MissionUiRect;
use super::model::MissionUiModel;
use super::widgets::{
    centered_menu_skin_label, mission_text_color, mission_text_state_color, skin_style_border,
    sliced, stretch, text_button,
};
use crate::{
    character_selection_ui::{
        CHARACTER_SELECTION_BLUE_BUTTON_OVER_PATH, CHARACTER_SELECTION_BLUE_BUTTON_PATH,
        CHARACTER_SELECTION_RED_BUTTON_OVER_PATH, CHARACTER_SELECTION_RED_BUTTON_PATH,
    },
    gameplay_ui::{GameplayUiModel, GameplayUiOutbox},
    gui_skin::gui_style as mission_gui_style,
    tutorial_native_mechanics::TutorialNativeMechanics,
    user_equip_ui::UserEquipUiState,
};
use bevy::{
    prelude::*,
    sprite::{SliceScaleMode, TextureSlicer},
    ui::widget::NodeImageMode,
};

pub(super) fn spawn_nanocom(root: &mut ChildSpawnerCommands, assets: &MissionUiAssets) {
    root.spawn((
        NANOCOM_MENU_RECT.node(),
        stretch(&assets.nanocom_menu),
        UiTransform::default(),
        MissionUiView::NanocomRoot,
        Pickable::IGNORE,
    ))
    .with_children(|menu| {
        for (index, text) in NANOCOM_MENU_LABELS.into_iter().enumerate() {
            let style = if index == NANOCOM_MENU_LABELS.len() - 1 {
                MenuButtonStyle::HudRed
            } else {
                MenuButtonStyle::HudBlue
            };
            text_button(
                menu,
                MissionUiRect::new(11.0, 71.0 + index as f32 * 29.0, 156.0, 26.0),
                text,
                nanocom_control(index),
                style,
                assets,
            );
        }
        menu.spawn((
            Button,
            NANOCOM_CLOSE_RECT.node(),
            ImageNode::new(assets.nanocom_close.clone()),
            MissionUiControl::NanocomClose,
            NanocomCloseButton,
        ));
    });
}

pub(super) fn spawn_chat_quick_menu(root: &mut ChildSpawnerCommands, assets: &MissionUiAssets) {
    let border = skin_style_border("FusionFallChatSkin", "EmoteBox")
        .expect("clean Retrobution Chat EmoteBox style must remain converted");
    root.spawn((
        CHAT_QUICK_MENU_RECT.node(),
        sliced(&assets.chat_quick_menu_box, border),
        UiTransform::default(),
        MissionUiView::ChatQuickRoot,
        Pickable::IGNORE,
    ))
    .with_children(|menu| {
        for (rect, text, style, enabled, control) in [
            (
                CHAT_QUICK_GROUP_RECT,
                "LEAVE GROUP",
                MenuButtonStyle::ChatRed,
                false,
                None,
            ),
            (
                CHAT_QUICK_WARP_RECT,
                "WARP AWAY",
                MenuButtonStyle::ChatRed,
                true,
                Some(MissionUiControl::WarpAway),
            ),
            (
                CHAT_QUICK_VEHICLE_RECT,
                "HOP ON VEHICLE",
                MenuButtonStyle::ChatBlue,
                false,
                None,
            ),
        ] {
            let (skin_name, style_name) = style.skin_and_style();
            let normal = if style.is_red() {
                &assets.red_button
            } else {
                &assets.blue_button
            };
            let mut image = skin_style_border(skin_name, style_name).map_or_else(
                || stretch(normal),
                |button_border| sliced(normal, button_border),
            );
            if !enabled {
                image.color = Color::srgba(0.52, 0.52, 0.52, 0.78);
            }
            let mut button = menu.spawn((
                Button,
                rect.node(),
                image,
                ChatQuickMenuButton { style, enabled },
            ));
            if let Some(control) = control {
                button.insert(control);
            } else {
                button.insert(Pickable::IGNORE);
            }
            button.with_children(|button| {
                centered_menu_skin_label(
                    button,
                    MissionUiRect::new(0.0, 0.0, rect.width, rect.height),
                    text,
                    skin_name,
                    style_name,
                    assets,
                    (!enabled).then_some(Color::srgba(0.72, 0.72, 0.72, 0.82)),
                );
            });
        }
    });
}

pub(super) fn nanocom_control(index: usize) -> MissionUiControl {
    match index {
        0 => MissionUiControl::NanocomMyStuff,
        1 => MissionUiControl::NanocomJournal,
        2 => MissionUiControl::NanocomEmail,
        3 => MissionUiControl::NanocomMap,
        4 => MissionUiControl::NanocomSettings,
        5 => MissionUiControl::NanocomGameGuide,
        6 => MissionUiControl::NanocomExitGame,
        _ => unreachable!(),
    }
}

pub(super) fn chat_quick_menu_logical_rect(viewport_height: f32) -> MissionUiRect {
    MissionUiRect::new(
        CHAT_QUICK_MENU_RECT.x,
        viewport_height - CHAT_WINDOW_HEIGHT - CHAT_EMOTE_CONTAINER_HEIGHT + CHAT_QUICK_MENU_RECT.y,
        CHAT_QUICK_MENU_RECT.width,
        CHAT_QUICK_MENU_RECT.height,
    )
}

pub(super) fn nanocom_button_source_path(style: MenuButtonStyle, interaction: &Interaction) -> &'static str {
    match (style.is_red(), interaction) {
        (true, Interaction::Hovered) => CHARACTER_SELECTION_RED_BUTTON_OVER_PATH,
        (true, _) => CHARACTER_SELECTION_RED_BUTTON_PATH,
        (false, Interaction::Hovered) => CHARACTER_SELECTION_BLUE_BUTTON_OVER_PATH,
        (false, _) => CHARACTER_SELECTION_BLUE_BUTTON_PATH,
    }
}

pub(super) fn bind_nanocom_button_hover(
    assets: Res<MissionUiAssets>,
    mut buttons: Query<
        (&Interaction, &NanocomMenuButton, &Children, &mut ImageNode),
        (With<NanocomMenuButton>, Changed<Interaction>),
    >,
    child_nodes: Query<&Children, Without<NanocomMenuButton>>,
    mut text_colors: Query<&mut TextColor>,
) {
    for (interaction, marker, children, mut image) in &mut buttons {
        image.image = match nanocom_button_source_path(marker.0, interaction) {
            CHARACTER_SELECTION_RED_BUTTON_OVER_PATH => assets.red_button_over.clone(),
            CHARACTER_SELECTION_RED_BUTTON_PATH => assets.red_button.clone(),
            CHARACTER_SELECTION_BLUE_BUTTON_OVER_PATH => assets.blue_button_over.clone(),
            _ => assets.blue_button.clone(),
        };
        let (skin_name, style_name) = marker.0.skin_and_style();
        image.image_mode =
            skin_style_border(skin_name, style_name).map_or(NodeImageMode::Stretch, |border| {
                NodeImageMode::Sliced(TextureSlicer {
                    border,
                    center_scale_mode: SliceScaleMode::Stretch,
                    sides_scale_mode: SliceScaleMode::Stretch,
                    max_corner_scale: 1.0,
                })
            });
        let text_color = mission_text_state_color(
            mission_gui_style(skin_name, style_name)
                .expect("Retrobution Nanocom button style must remain converted"),
            menu_interaction_state(interaction),
        );
        for child in children.iter() {
            if let Ok(mut color) = text_colors.get_mut(child) {
                **color = text_color;
            }
            let Ok(grandchildren) = child_nodes.get(child) else {
                continue;
            };
            for descendant in grandchildren.iter() {
                if let Ok(mut color) = text_colors.get_mut(descendant) {
                    **color = text_color;
                }
            }
        }
    }
}

pub(super) fn bind_chat_quick_button_hover(
    assets: Res<MissionUiAssets>,
    mut buttons: Query<
        (&Interaction, &ChatQuickMenuButton, &mut ImageNode),
        (With<ChatQuickMenuButton>, Changed<Interaction>),
    >,
) {
    for (interaction, marker, mut image) in &mut buttons {
        let hovered =
            marker.enabled && matches!(interaction, Interaction::Hovered | Interaction::Pressed);
        image.image = if marker.style.is_red() {
            if hovered {
                assets.red_button_over.clone()
            } else {
                assets.red_button.clone()
            }
        } else if hovered {
            assets.blue_button_over.clone()
        } else {
            assets.blue_button.clone()
        };
        let (skin_name, style_name) = marker.style.skin_and_style();
        image.image_mode =
            skin_style_border(skin_name, style_name).map_or(NodeImageMode::Stretch, |border| {
                NodeImageMode::Sliced(TextureSlicer {
                    border,
                    center_scale_mode: SliceScaleMode::Stretch,
                    sides_scale_mode: SliceScaleMode::Stretch,
                    max_corner_scale: 1.0,
                })
            });
        image.color = if marker.enabled {
            Color::WHITE
        } else {
            Color::srgba(0.52, 0.52, 0.52, 0.78)
        };
    }
}

pub(super) fn bind_chat_quick_warp_state(
    model: Res<MissionUiModel>,
    native: Res<TutorialNativeMechanics>,
    mut buttons: Query<(
        &MissionUiControl,
        &mut ChatQuickMenuButton,
        &mut ImageNode,
        &Children,
    )>,
    child_nodes: Query<&Children, Without<ChatQuickMenuButton>>,
    mut text_colors: Query<&mut TextColor>,
) {
    if !model.is_changed() && !native.is_changed() {
        return;
    }
    for (control, mut marker, mut image, children) in &mut buttons {
        if *control != MissionUiControl::WarpAway {
            continue;
        }
        let enabled = native.bound_stage().is_none() && model.warp_away_available();
        marker.enabled = enabled;
        image.color = if enabled {
            Color::WHITE
        } else {
            Color::srgba(0.52, 0.52, 0.52, 0.78)
        };
        let (skin_name, style_name) = marker.style.skin_and_style();
        let text_color = if enabled {
            mission_text_color(
                mission_gui_style(skin_name, style_name)
                    .expect("clean Retrobution quick-menu style"),
            )
        } else {
            Color::srgba(0.72, 0.72, 0.72, 0.82)
        };
        for child in children.iter() {
            let Ok(grandchildren) = child_nodes.get(child) else {
                continue;
            };
            for descendant in grandchildren.iter() {
                if let Ok(mut color) = text_colors.get_mut(descendant) {
                    **color = text_color;
                }
            }
        }
    }
}

pub(super) fn bind_nanocom_close_hover(
    assets: Res<MissionUiAssets>,
    mut buttons: Query<
        (&Interaction, &mut ImageNode),
        (With<NanocomCloseButton>, Changed<Interaction>),
    >,
) {
    for (interaction, mut image) in &mut buttons {
        image.image = if matches!(interaction, Interaction::Hovered | Interaction::Pressed) {
            assets.nanocom_close_over.clone()
        } else {
            assets.nanocom_close.clone()
        };
    }
}

pub(super) fn toggle_nanocom_with_enter(
    keys: Res<ButtonInput<KeyCode>>,
    gameplay_ui: Option<Res<GameplayUiModel>>,
    user_equip_ui: Option<Res<UserEquipUiState>>,
    mut model: ResMut<MissionUiModel>,
    mut outbox: ResMut<GameplayUiOutbox>,
) {
    if !keys.just_pressed(KeyCode::Enter) && !keys.just_pressed(KeyCode::NumpadEnter) {
        return;
    }
    // In a ready world `CnGuiChat.Update` owns Enter and emits the matching
    // Nanocom transition. Keeping this fallback only for the chat-gated
    // tutorial prevents one keypress from toggling the menu twice.
    if gameplay_ui
        .as_ref()
        .is_some_and(|gameplay_ui| gameplay_ui.chat.input_enabled)
    {
        return;
    }
    if model.system_popup_active() {
        return;
    }
    if model.nanocom_foreign_modal_suppressed() {
        return;
    }
    if user_equip_ui
        .as_ref()
        .is_some_and(|user_equip_ui| user_equip_ui.is_active())
    {
        return;
    }
    if model.nanocom_main_menu_visible {
        model.close_nanocom_menu(&mut outbox);
    } else {
        model.open_nanocom_menu(&mut outbox);
    }
}

pub(super) const fn mission_ui_control_uses_nanocom_surface(control: MissionUiControl) -> bool {
    matches!(
        control,
        MissionUiControl::NanocomMyStuff
            | MissionUiControl::NanocomJournal
            | MissionUiControl::NanocomEmail
            | MissionUiControl::NanocomMap
            | MissionUiControl::NanocomSettings
            | MissionUiControl::NanocomGameGuide
            | MissionUiControl::NanocomExitGame
            | MissionUiControl::NanocomClose
            | MissionUiControl::WarpAway
    )
}

pub(super) const fn mission_ui_control_is_nanocom_menu_row(control: MissionUiControl) -> bool {
    matches!(
        control,
        MissionUiControl::NanocomMyStuff
            | MissionUiControl::NanocomJournal
            | MissionUiControl::NanocomEmail
            | MissionUiControl::NanocomMap
            | MissionUiControl::NanocomSettings
            | MissionUiControl::NanocomGameGuide
            | MissionUiControl::NanocomExitGame
    )
}
