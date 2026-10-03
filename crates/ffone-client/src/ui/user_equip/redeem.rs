use super::*;
use crate::shared_input_ui::{RedeemSource, SharedInputDialog, SharedRedeemCode};

#[derive(Component)]
pub(super) struct RedeemControl;
#[derive(Component)]
pub(super) struct RedeemLabel;

#[derive(Component)]
pub(super) struct RedeemImages {
    normal: Handle<Image>,
    hover: Handle<Image>,
}

// The shared inventory footer uses the same button and font adapter in every mode.
const REDEEM_RECT: UserEquipUiRect = UserEquipUiRect::new(15.0, 598.0, 149.0, 25.0);

pub(super) fn spawn_control(
    parent: &mut ChildSpawnerCommands,
    assets: &UserEquipUiAssets,
    server: &AssetServer,
) {
    use crate::bank_ui::*;
    parent
        .spawn((
            Button,
            RedeemControl,
            RedeemImages {
                normal: server.load(BANK_SLOT_BUTTON_PATH),
                hover: server.load(crate::vendor_ui::VENDOR_BUTTON_HOVER_PATH),
            },
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect {
                    left: px(BANK_BUTTON_PADDING_LEFT),
                    right: px(BANK_BUTTON_PADDING_RIGHT),
                    top: px(BANK_BUTTON_PADDING_TOP),
                    bottom: px(BANK_BUTTON_PADDING_BOTTOM),
                },
                ..REDEEM_RECT.node()
            },
            sliced_image(server.load(BANK_SLOT_BUTTON_PATH), BANK_SLOT_BUTTON_BORDER),
        ))
        .with_children(|button| {
            button.spawn((
                RedeemLabel,
                Text::new("REDEEM CODE"),
                LocalizedText::new("ui.inventory.redeem_code", "REDEEM CODE"),
                (
                    TextFont {
                        font: (assets.font.clone()).into(),
                        font_size: (BANK_BUTTON_FONT_SIZE).into(),
                        ..default()
                    },
                    LineHeight::Px(BANK_BUTTON_LINE_HEIGHT),
                ),
                TextLayout::new(Justify::Center, bevy::text::LineBreak::NoWrap),
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                UiTransform::from_translation(Val2::px(0.0, BANK_TEXT_REPLACEMENT_Y_OFFSET)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
        });
}

pub(super) fn collect_actions(
    state: Res<UserEquipUiState>,
    projection: Res<UserEquipItemModeProjection>,
    mut modal: ResMut<UserEquipModalState>,
    buttons: Query<&Interaction, (With<RedeemControl>, Changed<Interaction>)>,
    input: Option<ResMut<SharedInputDialog>>,
    redeem: Option<ResMut<SharedRedeemCode>>,
    mut audio: ResMut<UserEquipUiAudioOutbox>,
) {
    if !state.input_capabilities(*modal).panel_controls
        || !buttons.iter().any(|value| *value == Interaction::Pressed)
    {
        return;
    }
    if let (Some(mut input), Some(mut redeem)) = (input, redeem)
        && redeem.open(
            RedeemSource::Inventory {
                pc: projection.owner_pc_id,
            },
            &mut input,
        )
    {
        modal.redeem_code_view = true;
        audio.push(UserEquipUiAudioCue::ButtonSound);
    }
}

pub(super) fn bind_controls(
    state: Res<UserEquipUiState>,
    modal: Res<UserEquipModalState>,
    mut buttons: Query<(&mut ImageNode, &Interaction, &RedeemImages), With<RedeemControl>>,
    mut labels: Query<&mut TextColor, With<RedeemLabel>>,
) {
    let enabled = state.input_capabilities(*modal).panel_controls;
    let color = if enabled {
        Color::WHITE
    } else {
        Color::srgba(1.0, 1.0, 1.0, 0.5)
    };
    for (mut image, interaction, images) in &mut buttons {
        let handle = if enabled && *interaction == Interaction::Hovered {
            &images.hover
        } else {
            &images.normal
        };
        if &image.image != handle {
            image.image = handle.clone();
        }
        if image.color != color {
            image.color = color;
        }
        let text_color = if enabled && *interaction == Interaction::Hovered {
            Color::srgb(0.229838714, 0.463709682, 1.0)
        } else {
            Color::srgb(0.9, 0.9, 0.9)
        }
        .with_alpha(if enabled { 1.0 } else { 0.5 });
        for mut label in &mut labels {
            if label.0 != text_color {
                label.0 = text_color;
            }
        }
    }
}

#[cfg(test)]
mod tests;
