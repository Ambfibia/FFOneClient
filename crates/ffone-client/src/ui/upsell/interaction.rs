use super::*;

pub const UPSELL_CLOSE_HOVER_PATH_ID: i64 = 664;

pub const UPSELL_GET_HOVER_PATH_ID: i64 = 232;

pub const UPSELL_CONTINUE_HOVER_PATH_ID: i64 = 145;

pub const UPSELL_NOT_NOW_HOVER_PATH_ID: i64 = 597;

pub const UPSELL_NEWS_BUTTON_NORMAL_PATH_ID: i64 = 640;

pub const UPSELL_NEWS_BUTTON_HOVER_PATH_ID: i64 = 309;

pub const UPSELL_CLOSE_HOVER_PATH: &str = "ui/en/shared/close_over.png";

pub const UPSELL_GET_HOVER_PATH: &str = "ui/en/gameplay/upsell/us_get_up.png";

pub const UPSELL_CONTINUE_HOVER_PATH: &str = "ui/en/gameplay/upsell/us_continue_up.png";

pub const UPSELL_NOT_NOW_HOVER_PATH: &str = "ui/en/gameplay/upsell/us_not_up.png";

pub const UPSELL_NEWS_BUTTON_NORMAL_PATH: &str = "ui/en/gameplay/chat/blue_button_normal.png";

pub const UPSELL_NEWS_BUTTON_HOVER_PATH: &str = "ui/en/gameplay/chat/blue_button_over.png";

pub const UPSELL_NEWS_BUTTON_NORMAL_TEXT_COLOR: [f32; 4] = [0.9, 1.0, 1.0, 1.0];

pub const UPSELL_NEWS_BUTTON_HOVER_TEXT_COLOR: [f32; 4] = [0.0, 0.278_225_8, 0.487_903_24, 1.0];

pub const UPSELL_NEWS_BUTTON_ACTIVE_TEXT_COLOR: [f32; 4] = [0.9, 0.9, 0.9, 1.0];

pub const UPSELL_CONTINUE_HOVER_TEXT_COLOR: [f32; 4] = [0.0, 0.196_078_43, 0.392_156_87, 1.0];

pub const UPSELL_NOT_NOW_HOVER_TEXT_COLOR: [f32; 4] = [0.0, 0.205_645_16, 0.399_193_56, 1.0];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UpsellInputBoundary {
    pub blocks_lower_ui: bool,
    pub blocks_gameplay_input: bool,
    pub requires_pointer: bool,
    pub mouse_controls_enabled: bool,
    pub escape_dismiss_enabled: bool,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UpsellUiButtonKind {
    Close,
    Continue,
    GetUpgrade,
    NotRightNow,
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct UpsellUiButton {
    pub kind: UpsellUiButtonKind,
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) struct UpsellUiButtonLabel {
    pub(super) kind: UpsellUiButtonKind,
}

pub(super) fn handle_upsell_keyboard(
    keys: Option<MessageReader<KeyboardInput>>,
    mut model: ResMut<UpsellUiModel>,
    mut outbox: ResMut<UpsellUiOutbox>,
) {
    let Some(mut keys) = keys else {
        return;
    };
    for key in keys.read() {
        if key.state == ButtonState::Pressed
            && !key.repeat
            && key.key_code == KeyCode::Escape
            && dismiss_upsell_with_escape(&mut model, &mut outbox)
        {
            break;
        }
    }
}

pub(super) fn handle_upsell_interactions(
    buttons: Query<(&Interaction, &UpsellUiButton), Changed<Interaction>>,
    mut model: ResMut<UpsellUiModel>,
    mut outbox: ResMut<UpsellUiOutbox>,
    mut audio: ResMut<UpsellUiAudioOutbox>,
) {
    for (interaction, button) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let command = match button.kind {
            UpsellUiButtonKind::Close => UpsellUiCommand::Close,
            UpsellUiButtonKind::Continue => UpsellUiCommand::Continue,
            UpsellUiButtonKind::GetUpgrade => UpsellUiCommand::OpenPayPage,
            UpsellUiButtonKind::NotRightNow => UpsellUiCommand::NotRightNow,
        };
        if apply_upsell_ui_command(&mut model, &mut outbox, &mut audio, command) {
            break;
        }
    }
}

pub(super) fn sync_upsell_button_visuals(
    model: Res<UpsellUiModel>,
    assets: Res<UpsellUiAssets>,
    mut buttons: Query<(
        &Interaction,
        &UpsellUiButton,
        &Children,
        &mut ImageNode,
        Option<&mut Pickable>,
    )>,
    mut labels: Query<(
        &UpsellUiButtonLabel,
        &mut TextFont,
        &mut LineHeight,
        &mut TextColor,
        &mut LocalizedText,
    )>,
) {
    for (interaction, button, children, mut image, pickable) in &mut buttons {
        let enabled = upsell_button_enabled(&model, button.kind);
        let interaction = if enabled {
            *interaction
        } else {
            Interaction::None
        };
        let Some(visual) = button_visual(&model, button.kind) else {
            if let Some(mut pickable) = pickable {
                *pickable = Pickable::IGNORE;
            }
            continue;
        };
        *image = button_image(&assets, visual, interaction);
        if let Some(mut pickable) = pickable {
            *pickable = if enabled {
                Pickable::default()
            } else {
                Pickable::IGNORE
            };
        }
        for child in children.iter() {
            if let Ok((label, mut font, mut text_line_height, mut color, mut localized)) =
                labels.get_mut(child)
                && label.kind == button.kind
            {
                let label = button_label(visual);
                *localized = upsell_label_localized(label);
                let (font_size, line_height) = button_font_metrics(visual);
                font.font_size = font_size.into();
                *text_line_height = LineHeight::Px(line_height);
                color.0 = button_text_color(visual, interaction);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum UpsellUiButtonVisual {
    Close,
    UpgradeContinue,
    GetUpgrade,
    NotRightNow,
    NewsContinue,
}

pub(super) fn button_visual(model: &UpsellUiModel, kind: UpsellUiButtonKind) -> Option<UpsellUiButtonVisual> {
    match (model.active_mode?, kind) {
        (_, UpsellUiButtonKind::Close) => Some(UpsellUiButtonVisual::Close),
        (UpsellUiMode::Upgrade, UpsellUiButtonKind::Continue) => {
            Some(UpsellUiButtonVisual::UpgradeContinue)
        }
        (UpsellUiMode::Upgrade, UpsellUiButtonKind::GetUpgrade) => {
            Some(UpsellUiButtonVisual::GetUpgrade)
        }
        (UpsellUiMode::Upgrade, UpsellUiButtonKind::NotRightNow) => {
            Some(UpsellUiButtonVisual::NotRightNow)
        }
        (UpsellUiMode::NewsPayZone | UpsellUiMode::NewsFreeZone, UpsellUiButtonKind::Continue) => {
            Some(UpsellUiButtonVisual::NewsContinue)
        }
        (
            UpsellUiMode::NewsPayZone | UpsellUiMode::NewsFreeZone,
            UpsellUiButtonKind::GetUpgrade | UpsellUiButtonKind::NotRightNow,
        ) => None,
    }
}

#[must_use]
pub fn upsell_button_enabled(model: &UpsellUiModel, kind: UpsellUiButtonKind) -> bool {
    model.controls_enabled()
        && button_rect(model, kind).is_some()
        && button_visual(model, kind).is_some()
}

pub(super) fn button_image(
    assets: &UpsellUiAssets,
    visual: UpsellUiButtonVisual,
    interaction: Interaction,
) -> ImageNode {
    let hovered = interaction == Interaction::Hovered;
    match visual {
        UpsellUiButtonVisual::Close => stretch_image(if hovered {
            assets.close_hover.clone()
        } else {
            assets.close_normal.clone()
        }),
        UpsellUiButtonVisual::GetUpgrade => stretch_image(if hovered {
            assets.get_hover.clone()
        } else {
            assets.get_normal.clone()
        }),
        UpsellUiButtonVisual::UpgradeContinue => sliced_image(
            if hovered {
                assets.continue_hover.clone()
            } else {
                assets.continue_normal.clone()
            },
            UPSELL_CONTINUE_BORDER,
        ),
        UpsellUiButtonVisual::NotRightNow => sliced_image(
            if hovered {
                assets.not_now_hover.clone()
            } else {
                assets.not_now_normal.clone()
            },
            UPSELL_NOT_NOW_BORDER,
        ),
        UpsellUiButtonVisual::NewsContinue => sliced_image(
            if hovered {
                assets.news_button_hover.clone()
            } else {
                assets.news_button_normal.clone()
            },
            UPSELL_NEWS_BUTTON_BORDER,
        ),
    }
}

pub(super) fn button_label(visual: UpsellUiButtonVisual) -> &'static str {
    match visual {
        UpsellUiButtonVisual::UpgradeContinue => UPSELL_CONTINUE_PLAYING_LABEL,
        UpsellUiButtonVisual::NotRightNow => UPSELL_NOT_RIGHT_NOW_LABEL,
        UpsellUiButtonVisual::NewsContinue => UPSELL_CONTINUE_LABEL,
        UpsellUiButtonVisual::Close | UpsellUiButtonVisual::GetUpgrade => "",
    }
}

pub(super) fn button_font_metrics(visual: UpsellUiButtonVisual) -> (f32, f32) {
    match visual {
        UpsellUiButtonVisual::UpgradeContinue => {
            (UPSELL_JEFFE_14_FONT_SIZE, UPSELL_JEFFE_14_LINE_HEIGHT)
        }
        UpsellUiButtonVisual::NotRightNow
        | UpsellUiButtonVisual::NewsContinue
        | UpsellUiButtonVisual::Close
        | UpsellUiButtonVisual::GetUpgrade => {
            (UPSELL_JEFFE_12_FONT_SIZE, UPSELL_JEFFE_12_LINE_HEIGHT)
        }
    }
}

pub(super) fn button_text_color(visual: UpsellUiButtonVisual, interaction: Interaction) -> Color {
    let rgba = match (visual, interaction) {
        (UpsellUiButtonVisual::UpgradeContinue, Interaction::Hovered) => {
            UPSELL_CONTINUE_HOVER_TEXT_COLOR
        }
        (UpsellUiButtonVisual::UpgradeContinue, Interaction::None | Interaction::Pressed) => {
            UPSELL_CONTINUE_NORMAL_TEXT_COLOR
        }
        (UpsellUiButtonVisual::NotRightNow, Interaction::Hovered) => {
            UPSELL_NOT_NOW_HOVER_TEXT_COLOR
        }
        (UpsellUiButtonVisual::NotRightNow, Interaction::Pressed) => {
            UPSELL_NOT_NOW_ACTIVE_TEXT_COLOR
        }
        (UpsellUiButtonVisual::NotRightNow, Interaction::None) => UPSELL_NOT_NOW_NORMAL_TEXT_COLOR,
        (UpsellUiButtonVisual::NewsContinue, Interaction::Hovered) => {
            UPSELL_NEWS_BUTTON_HOVER_TEXT_COLOR
        }
        (UpsellUiButtonVisual::NewsContinue, Interaction::Pressed) => {
            UPSELL_NEWS_BUTTON_ACTIVE_TEXT_COLOR
        }
        (UpsellUiButtonVisual::NewsContinue, Interaction::None) => {
            UPSELL_NEWS_BUTTON_NORMAL_TEXT_COLOR
        }
        (UpsellUiButtonVisual::Close | UpsellUiButtonVisual::GetUpgrade, _) => [1.0; 4],
    };
    Color::srgba(rgba[0], rgba[1], rgba[2], rgba[3])
}

pub(super) fn button_padding(visual: UpsellUiButtonVisual) -> UiRect {
    match visual {
        UpsellUiButtonVisual::NewsContinue => UiRect {
            left: px(6),
            right: px(6),
            top: px(3),
            bottom: px(3),
        },
        UpsellUiButtonVisual::Close
        | UpsellUiButtonVisual::UpgradeContinue
        | UpsellUiButtonVisual::GetUpgrade
        | UpsellUiButtonVisual::NotRightNow => UiRect::ZERO,
    }
}
