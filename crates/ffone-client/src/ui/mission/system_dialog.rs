//! Tutorial-exit system dialog state, spawning and button presentation.

use super::assets::{MissionUiAssets, SYSTEM_DIALOG_BOX};
use super::components::{MissionUiControl, MissionUiView, SystemDialogButton};
use super::geometry::{
    SYSTEM_DIALOG_CANCEL_RECT, SYSTEM_DIALOG_CONTENT_RECT, SYSTEM_DIALOG_ICON_RECT,
    SYSTEM_DIALOG_OKAY_RECT, SYSTEM_DIALOG_OVERLAY_ALPHA, SYSTEM_DIALOG_PANEL_RECT,
};
use super::layout::MissionUiRect;
use super::widgets::{
    centered_menu_skin_label, centered_node, mission_text_state_color, skin_label, styled_image,
};
use crate::{
    character_selection_ui::{
        CHARACTER_SELECTION_BLUE_BUTTON_OVER_PATH, CHARACTER_SELECTION_BLUE_BUTTON_PATH,
        CHARACTER_SELECTION_CANCEL_NORMAL_PATH,
    },
    gui_skin::gui_style as mission_gui_style,
};
use bevy::prelude::*;

pub const TUTORIAL_EXIT_SYSTEM_MESSAGE_ID: i32 = 47;
pub const TUTORIAL_EXIT_SYSTEM_MESSAGE_BUTTON_TYPE: i32 = 2;
pub const TUTORIAL_EXIT_SYSTEM_MESSAGE_TEXT: &str = "Are you sure you want to skip the tutorial? ";
pub const TUTORIAL_EXIT_KEY: KeyCode = KeyCode::Backquote;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TutorialSystemDialogUi {
    pub(super) message_id: i32,
    pub(super) button_type: i32,
    pub(super) text: &'static str,
}

impl TutorialSystemDialogUi {
    pub const fn tutorial_exit() -> Self {
        Self {
            message_id: TUTORIAL_EXIT_SYSTEM_MESSAGE_ID,
            button_type: TUTORIAL_EXIT_SYSTEM_MESSAGE_BUTTON_TYPE,
            text: TUTORIAL_EXIT_SYSTEM_MESSAGE_TEXT,
        }
    }

    pub const fn message_id(self) -> i32 {
        self.message_id
    }

    pub const fn button_type(self) -> i32 {
        self.button_type
    }

    pub const fn text(self) -> &'static str {
        self.text
    }

    pub(super) fn is_tutorial_exit(self) -> bool {
        self.message_id == TUTORIAL_EXIT_SYSTEM_MESSAGE_ID
            && self.button_type == TUTORIAL_EXIT_SYSTEM_MESSAGE_BUTTON_TYPE
            && self.text == TUTORIAL_EXIT_SYSTEM_MESSAGE_TEXT
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TutorialExitDialogGate {
    pub tutorial_active: bool,
    pub world_ready: bool,
    pub startup_ready: bool,
    pub delay_active: bool,
    pub system_popup_active: bool,
    pub event_scene_active: bool,
}

impl TutorialExitDialogGate {
    pub const fn allows_open(self) -> bool {
        self.tutorial_active
            && self.world_ready
            && self.startup_ready
            && !self.delay_active
            && !self.system_popup_active
            && !self.event_scene_active
    }
}

pub(super) fn spawn_system_dialog(commands: &mut Commands, assets: &MissionUiAssets) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, SYSTEM_DIALOG_OVERLAY_ALPHA)),
            GlobalZIndex(i32::MAX - 32),
            MissionUiView::SystemDialogRoot,
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        width: px(SYSTEM_DIALOG_PANEL_RECT.width),
                        height: px(SYSTEM_DIALOG_PANEL_RECT.height),
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    styled_image(&assets.system_dialog_box, SYSTEM_DIALOG_BOX),
                    UiTransform::default(),
                    MissionUiView::SystemDialogPanel,
                ))
                .with_children(|dialog| {
                    // Even the overload without an explicit icon passes icon
                    // index zero in the clean client. The ItemBox and warning
                    // icon deliberately occupy the same 62x62 rectangle.
                    dialog.spawn((
                        SYSTEM_DIALOG_ICON_RECT.node(),
                        ImageNode::new(assets.system_dialog_icon_box.clone()),
                        Pickable::IGNORE,
                    ));
                    dialog.spawn((
                        SYSTEM_DIALOG_ICON_RECT.node(),
                        ImageNode::new(assets.system_dialog_warning.clone()),
                        Pickable::IGNORE,
                    ));
                    skin_label(
                        dialog,
                        SYSTEM_DIALOG_CONTENT_RECT,
                        TUTORIAL_EXIT_SYSTEM_MESSAGE_TEXT,
                        "FusionFallSysMessageSkin",
                        "label",
                        assets,
                        Some(MissionUiView::SystemDialogText),
                        None,
                    );
                    dialog
                        .spawn((
                            Button,
                            centered_node(SYSTEM_DIALOG_OKAY_RECT),
                            styled_image(&assets.blue_button, CHARACTER_SELECTION_BLUE_BUTTON_PATH),
                            MissionUiControl::SystemDialogOkay,
                            SystemDialogButton,
                        ))
                        .with_children(|button| {
                            centered_menu_skin_label(
                                button,
                                MissionUiRect::new(
                                    0.0,
                                    0.0,
                                    SYSTEM_DIALOG_OKAY_RECT.width,
                                    SYSTEM_DIALOG_OKAY_RECT.height,
                                ),
                                "OKAY",
                                "FusionFallSysMessageSkin",
                                "button",
                                assets,
                                None,
                            );
                        });
                    dialog
                        .spawn((
                            Button,
                            centered_node(SYSTEM_DIALOG_CANCEL_RECT),
                            styled_image(
                                &assets.cancel_normal,
                                CHARACTER_SELECTION_CANCEL_NORMAL_PATH,
                            ),
                            MissionUiControl::SystemDialogCancel,
                            SystemDialogButton,
                        ))
                        .with_children(|button| {
                            skin_label(
                                button,
                                MissionUiRect::new(
                                    0.0,
                                    0.0,
                                    SYSTEM_DIALOG_CANCEL_RECT.width,
                                    SYSTEM_DIALOG_CANCEL_RECT.height,
                                ),
                                "CANCEL",
                                "FusionFallSysMessageSkin",
                                "CancelButton",
                                assets,
                                None,
                                None,
                            );
                        });
                });
        });
}

pub(super) fn system_dialog_button_source_path(
    control: MissionUiControl,
    interaction: &Interaction,
) -> &'static str {
    match (control, interaction) {
        (MissionUiControl::SystemDialogOkay, Interaction::Hovered) => {
            CHARACTER_SELECTION_BLUE_BUTTON_OVER_PATH
        }
        (MissionUiControl::SystemDialogOkay, _) => CHARACTER_SELECTION_BLUE_BUTTON_PATH,
        (MissionUiControl::SystemDialogCancel, Interaction::None) => {
            CHARACTER_SELECTION_CANCEL_NORMAL_PATH
        }
        (MissionUiControl::SystemDialogCancel, _) => CHARACTER_SELECTION_BLUE_BUTTON_PATH,
        _ => CHARACTER_SELECTION_BLUE_BUTTON_PATH,
    }
}

pub(super) fn bind_system_dialog_button_hover(
    assets: Res<MissionUiAssets>,
    mut buttons: Query<
        (&Interaction, &MissionUiControl, &Children, &mut ImageNode),
        (With<SystemDialogButton>, Changed<Interaction>),
    >,
    mut text_colors: Query<&mut TextColor>,
) {
    for (interaction, control, children, mut image) in &mut buttons {
        image.image = match system_dialog_button_source_path(*control, interaction) {
            CHARACTER_SELECTION_BLUE_BUTTON_OVER_PATH => assets.blue_button_over.clone(),
            CHARACTER_SELECTION_CANCEL_NORMAL_PATH => assets.cancel_normal.clone(),
            _ => assets.blue_button.clone(),
        };
        let style_name = if *control == MissionUiControl::SystemDialogCancel {
            "CancelButton"
        } else {
            "button"
        };
        let state = match interaction {
            Interaction::Hovered => "hover",
            Interaction::Pressed => "active",
            Interaction::None => "normal",
        };
        let style = mission_gui_style("FusionFallSysMessageSkin", style_name)
            .expect("Retrobution system dialog button style");
        let color = mission_text_state_color(style, state);
        for child in children.iter() {
            if let Ok(mut text_color) = text_colors.get_mut(child) {
                **text_color = color;
            }
        }
    }
}
