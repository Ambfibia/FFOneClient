use super::*;

/// `cnEnchantMode.Update` has no configurable-key or Escape close branch;
/// the reachable close input is `Panel_PCStuffScript`'s close button.
pub const ENCHANT_KEYBOARD_CLOSE_REACHABLE_0104: bool = false;

/// `cnEnchantMode.pDelScroll` is declared but never assigned or called. Unity's
/// `GUI.BeginScrollView` can still update its own scroll position from pointer
/// input, but the configurable inventory-scroll axis is unreachable here.
pub const ENCHANT_CONFIGURABLE_SCROLL_REACHABLE_0104: bool = false;

pub const ENCHANT_BUTTON_NORMAL_PATH: &str = "ui/en/combi/button-normal.png";

pub const ENCHANT_BUTTON_HOVER_PATH: &str = "ui/en/combi/button-hover.png";

#[must_use]
pub fn clamp_enchant_inventory_scroll_0104(scroll_y: f32) -> f32 {
    if scroll_y.is_nan() || scroll_y <= 0.0 {
        0.0
    } else if !scroll_y.is_finite() {
        ENCHANT_INVENTORY_SCROLL_MAX_0104
    } else {
        scroll_y.min(ENCHANT_INVENTORY_SCROLL_MAX_0104)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EnchantInputCapabilities0104 {
    pub draw: bool,
    pub base_gui_enabled: bool,
    pub preview_enabled: bool,
    pub clear_enabled: bool,
    pub enchant_enabled: bool,
    pub close_enabled: bool,
    pub success_buttons_enabled: bool,
}

pub const ENCHANT_INVENTORY_SCROLL_VELOCITY_0104: f32 = 200.0;

pub const ENCHANT_INVENTORY_SCROLL_MAX_0104: f32 = 190.0;

pub const ENCHANT_INVENTORY_BUTTON_FONT_PATH_ID_0104: i64 = 933;

#[derive(Component)]
pub(super) struct EnchantButtonLabel0104;

pub(super) fn collect_enchant_ui_input_0104(
    projection: Res<EnchantModeProjection0104>,
    inventory_ui: Res<EnchantInventoryUiState0104>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    mut dragging: Local<bool>,
    controls: Query<(&EnchantInteractiveControl0104, Ref<Interaction>)>,
    mut outbox: ResMut<EnchantUiOutbox0104>,
) {
    let panel_controls_enabled = inventory_ui.panel_controls_enabled();
    let base_controls_enabled = projection.capabilities.base_gui_enabled && panel_controls_enabled;
    if !base_controls_enabled
        || mouse.as_ref().is_some_and(|mouse| {
            !mouse.pressed(MouseButton::Left) && !mouse.just_released(MouseButton::Left)
        })
    {
        *dragging = false;
    }
    if mouse
        .as_ref()
        .is_some_and(|mouse| mouse.just_released(MouseButton::Left))
        && *dragging
    {
        *dragging = false;
        if base_controls_enabled {
            for (control, interaction) in &controls {
                if *interaction != Interaction::Hovered {
                    continue;
                }
                let command = match *control {
                    EnchantInteractiveControl0104::Attachment(slot) => {
                        Some(EnchantUiCommand0104::DropOnAttachment(slot))
                    }
                    EnchantInteractiveControl0104::Trash => Some(EnchantUiCommand0104::DropOnTrash),
                    _ => None,
                };
                if let Some(command) = command {
                    outbox.push(command);
                    break;
                }
            }
        }
    }
    for (control, interaction) in &controls {
        if !interaction.is_changed() || *interaction != Interaction::Pressed {
            continue;
        }
        let command = match *control {
            EnchantInteractiveControl0104::InventorySlot(slot) if base_controls_enabled => {
                *dragging = true;
                Some(EnchantUiCommand0104::BeginInventoryDrag(slot))
            }
            EnchantInteractiveControl0104::Attachment(slot) if base_controls_enabled => {
                Some(if projection.selection.visual_item(slot).is_some() {
                    EnchantUiCommand0104::DetachAttachment(slot)
                } else {
                    EnchantUiCommand0104::DropOnAttachment(slot)
                })
            }
            EnchantInteractiveControl0104::Preview
                if projection.capabilities.preview_enabled && panel_controls_enabled =>
            {
                Some(EnchantUiCommand0104::Preview)
            }
            EnchantInteractiveControl0104::Clear
                if projection.capabilities.clear_enabled && panel_controls_enabled =>
            {
                Some(EnchantUiCommand0104::Clear)
            }
            EnchantInteractiveControl0104::Enchant
                if projection.capabilities.enchant_enabled && panel_controls_enabled =>
            {
                Some(EnchantUiCommand0104::Enchant)
            }
            EnchantInteractiveControl0104::Close
                if projection.capabilities.close_enabled && panel_controls_enabled =>
            {
                Some(EnchantUiCommand0104::Close)
            }
            EnchantInteractiveControl0104::Trash if base_controls_enabled => {
                Some(EnchantUiCommand0104::DropOnTrash)
            }
            EnchantInteractiveControl0104::Help if base_controls_enabled => {
                Some(EnchantUiCommand0104::Help)
            }
            EnchantInteractiveControl0104::RedeemCode if base_controls_enabled => {
                Some(EnchantUiCommand0104::OpenRedeemCode)
            }
            EnchantInteractiveControl0104::EnchantMoreItems
                if projection.capabilities.success_buttons_enabled =>
            {
                Some(EnchantUiCommand0104::EnchantMoreItems)
            }
            EnchantInteractiveControl0104::GoToMyStuff
                if projection.capabilities.success_buttons_enabled =>
            {
                Some(EnchantUiCommand0104::GoToMyStuff)
            }
            _ => None,
        };
        if let Some(command) = command {
            outbox.push(command);
        }
    }
}

pub(super) fn enchant_bind_button_0104(
    image: Option<Mut<ImageNode>>,
    interaction: Option<&Interaction>,
    enabled: bool,
    assets: &EnchantUiAssets0104,
) {
    let Some(mut image) = image else {
        return;
    };
    let hovered =
        enabled && interaction.is_some_and(|interaction| *interaction == Interaction::Hovered);
    image.image = assets.image(if hovered {
        EnchantStaticAssetRole0104::ButtonHover
    } else {
        EnchantStaticAssetRole0104::ButtonNormal
    });
    image.color = if enabled {
        Color::WHITE
    } else {
        Color::srgba(1.0, 1.0, 1.0, 0.45)
    };
}

pub(super) fn bind_enchant_button_labels_0104(
    assets: If<Res<EnchantUiAssets0104>>,
    buttons: Query<&ImageNode, With<EnchantInteractiveControl0104>>,
    mut labels: Query<(&ChildOf, &mut TextColor), With<EnchantButtonLabel0104>>,
) {
    for (parent, mut label) in &mut labels {
        let Ok(image) = buttons.get(parent.parent()) else {
            continue;
        };
        let value = if image.image == assets.image(EnchantStaticAssetRole0104::ButtonHover) {
            1.0
        } else {
            0.9
        };
        let color = Color::srgba(value, value, value, image.color.alpha());
        if label.0 != color {
            label.0 = color;
        }
    }
}
