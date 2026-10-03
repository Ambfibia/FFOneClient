//! Spawning of the UserEquip window, labels, buttons and help panel.

use super::asset_contract::{UserEquipStaticAssetRole, UserEquipUiAssetContract};
use super::binding::inventory_static_localized;
use super::components::{
    UserEquipAvatarTurnControl, UserEquipBoundTextTarget, UserEquipCloseControl,
    UserEquipHelpCloseControl, UserEquipHelpControl, UserEquipItemTabControl,
    UserEquipNanoTabControl, UserEquipScrollDownControl, UserEquipScrollThumbControl,
    UserEquipScrollTrackControl, UserEquipScrollUpControl, UserEquipTrashControl,
    UserEquipUiAssets, UserEquipUiElement, UserEquipUiRoot, UserEquipUiRuntimeAssets,
};
use super::geometry::{
    USER_EQUIP_AVATAR_PREVIEW_RECT, USER_EQUIP_CLOSE_RECT, USER_EQUIP_DEXLABS_RECT,
    USER_EQUIP_EQUIP_STRIP_RECT, USER_EQUIP_EQUIPMENT_STRIP_COUNT, USER_EQUIP_EQUIPMENT_TITLE_RECT,
    USER_EQUIP_HELP_PANEL_RECT, USER_EQUIP_HELP_RECT, USER_EQUIP_INVENTORY_CONTENT_HEIGHT,
    USER_EQUIP_INVENTORY_CONTENT_WIDTH, USER_EQUIP_INVENTORY_PANEL_BORDER,
    USER_EQUIP_INVENTORY_SHADOW_A_RECT, USER_EQUIP_INVENTORY_SHADOW_B_RECT,
    USER_EQUIP_INVENTORY_SHADOW_BORDER, USER_EQUIP_INVENTORY_SLOT_SIZE,
    USER_EQUIP_INVENTORY_VIEWPORT_RECT, USER_EQUIP_ITEM_TAB_HIT_RECT,
    USER_EQUIP_ITEM_TAB_TEXTURE_RECT, USER_EQUIP_NANO_CONTENT_HEIGHT,
    USER_EQUIP_NANO_GALLERY_COUNT, USER_EQUIP_NANO_TAB_HIT_RECT, USER_EQUIP_NANO_TAB_TEXTURE_RECT,
    USER_EQUIP_PC_STUFF_RECT, USER_EQUIP_POPUP_FONT_SIZE, USER_EQUIP_REGULAR_FONT_LINE_HEIGHT,
    USER_EQUIP_RIGHT_PANEL_BORDER, USER_EQUIP_SCROLL_DOWN_RECT, USER_EQUIP_SCROLL_TRACK_RECT,
    USER_EQUIP_SCROLL_UP_RECT, USER_EQUIP_SMALL_FONT_LINE_HEIGHT, USER_EQUIP_SMALL_FONT_SIZE,
    USER_EQUIP_STATUS_FONT_SIZE, USER_EQUIP_STATUS_FUSION_BACK_RECT,
    USER_EQUIP_STATUS_FUSION_BAR_RECT, USER_EQUIP_STATUS_FUSION_RECT,
    USER_EQUIP_STATUS_GUIDE_ICON_RECT, USER_EQUIP_STATUS_GUIDE_LABEL_RECT,
    USER_EQUIP_STATUS_GUIDE_NAME_RECT, USER_EQUIP_STATUS_GUIDE_RECT,
    USER_EQUIP_STATUS_HP_BACK_RECT, USER_EQUIP_STATUS_HP_BAR_RECT, USER_EQUIP_STATUS_HP_RECT,
    USER_EQUIP_STATUS_LEVEL_RECT, USER_EQUIP_STATUS_NAME_RECT, USER_EQUIP_STATUS_PANEL_BORDER,
    USER_EQUIP_STATUS_PANEL_RECT, USER_EQUIP_TAB_FONT_SIZE, USER_EQUIP_TAROS_BACK_RECT,
    USER_EQUIP_TRASH_RECT, USER_EQUIP_TURN_LEFT_POSITIONED_RECT,
    USER_EQUIP_TURN_RIGHT_POSITIONED_RECT, USER_EQUIP_UI_Z_INDEX, USER_EQUIP_USER_CLOTHES_RECT,
    UserEquipUiRect, user_equip_taros_digit_rect,
};
use super::images::{sliced_image, stretched_image};
use super::item_popup::spawn_item_popup;
use super::nano_viewer::spawn_nano_viewer;
use super::redeem;
use super::spawn_slots::{
    spawn_battery_half_slots, spawn_equipment_slot, spawn_inventory_slot, spawn_nano_gallery_slot,
    spawn_nano_status_panels,
};
use super::state::UserEquipAvatarTurnDirection;
use crate::{
    inventory_runtime::INVENTORY_SLOT_COUNT_0104, localization::LocalizedText,
    player_preview::NativePlayerInventoryPreviewImage,
};
use bevy::{prelude::*, sprite::BorderRect, text::LineHeight};

pub(super) fn spawn_user_equip_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut image_assets: ResMut<Assets<Image>>,
    contract: Res<UserEquipUiAssetContract>,
    inventory_preview: Option<Res<NativePlayerInventoryPreviewImage>>,
) {
    let assets = UserEquipUiAssets::load(&asset_server, &mut image_assets, &contract);
    commands.insert_resource(UserEquipUiRuntimeAssets(assets.clone()));
    commands
        .spawn((
            UserEquipUiRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                overflow: Overflow::clip(),
                ..default()
            },
            Visibility::Hidden,
            GlobalZIndex(USER_EQUIP_UI_Z_INDEX),
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            root.spawn((
                UserEquipUiElement::Backdrop,
                UserEquipUiRect::default().node(),
                stretched_image(assets.image(UserEquipStaticAssetRole::Backdrop)),
                Pickable::IGNORE,
            ));
            root.spawn((
                UserEquipUiElement::ClothesBackplate,
                UserEquipUiRect::default().node(),
                stretched_image(assets.image(UserEquipStaticAssetRole::ClothesPanel)),
                Pickable::IGNORE,
            ));
            root.spawn((
                UserEquipUiElement::RightBackplate,
                UserEquipUiRect::default().node(),
                sliced_image(
                    assets.image(UserEquipStaticAssetRole::RightPanel),
                    USER_EQUIP_RIGHT_PANEL_BORDER,
                ),
                Pickable::IGNORE,
            ));
            root.spawn((
                UserEquipUiElement::UserClothesPanel,
                Node {
                    overflow: Overflow::clip(),
                    ..USER_EQUIP_USER_CLOTHES_RECT.node()
                },
                Pickable::IGNORE,
            ))
            .with_children(|panel| {
                panel.spawn((
                    UserEquipUiElement::AvatarPreview,
                    USER_EQUIP_AVATAR_PREVIEW_RECT.node(),
                    stretched_image(
                        inventory_preview
                            .as_deref()
                            .map(|preview| preview.0.clone())
                            .unwrap_or_default(),
                    ),
                    Pickable::IGNORE,
                ));
                panel.spawn((
                    UserEquipUiElement::StatusPanel,
                    USER_EQUIP_STATUS_PANEL_RECT.node(),
                    sliced_image(
                        assets.image(UserEquipStaticAssetRole::UserStatusPanel),
                        USER_EQUIP_STATUS_PANEL_BORDER,
                    ),
                    Pickable::IGNORE,
                ));
                for (element, rect, role) in [
                    (
                        UserEquipUiElement::StatusHpBack,
                        USER_EQUIP_STATUS_HP_BACK_RECT,
                        UserEquipStaticAssetRole::HpBack,
                    ),
                    (
                        UserEquipUiElement::StatusHpBar,
                        USER_EQUIP_STATUS_HP_BAR_RECT,
                        UserEquipStaticAssetRole::HpBar,
                    ),
                    (
                        UserEquipUiElement::StatusFusionBack,
                        USER_EQUIP_STATUS_FUSION_BACK_RECT,
                        UserEquipStaticAssetRole::HpBack,
                    ),
                    (
                        UserEquipUiElement::StatusFusionBar,
                        USER_EQUIP_STATUS_FUSION_BAR_RECT,
                        UserEquipStaticAssetRole::FusionMatterBar,
                    ),
                    (
                        UserEquipUiElement::StatusGuideBox,
                        USER_EQUIP_STATUS_GUIDE_RECT,
                        UserEquipStaticAssetRole::GuideBox,
                    ),
                ] {
                    panel.spawn((
                        element,
                        rect.node(),
                        stretched_image(assets.image(role)),
                        Pickable::IGNORE,
                    ));
                }
                spawn_bound_text_styled(
                    panel,
                    UserEquipUiElement::StatusName,
                    USER_EQUIP_STATUS_NAME_RECT,
                    "",
                    "ui.inventory.status.name",
                    "{name}",
                    assets.font.clone(),
                    USER_EQUIP_STATUS_FONT_SIZE,
                    USER_EQUIP_REGULAR_FONT_LINE_HEIGHT,
                    Color::srgb(0.8, 1.0, 1.0),
                    Justify::Left,
                );
                spawn_bound_text_styled(
                    panel,
                    UserEquipUiElement::StatusLevel,
                    USER_EQUIP_STATUS_LEVEL_RECT,
                    "LEVEL : 0",
                    "ui.inventory.status.level",
                    "LEVEL : {level}",
                    assets.font.clone(),
                    USER_EQUIP_STATUS_FONT_SIZE,
                    USER_EQUIP_REGULAR_FONT_LINE_HEIGHT,
                    Color::srgb(1.0, 1.0, 0.0),
                    Justify::Left,
                );
                spawn_bound_text_styled(
                    panel,
                    UserEquipUiElement::StatusHp,
                    USER_EQUIP_STATUS_HP_RECT,
                    "HEALTH",
                    "ui.inventory.status.health_label",
                    "HEALTH",
                    assets.font.clone(),
                    USER_EQUIP_SMALL_FONT_SIZE,
                    USER_EQUIP_SMALL_FONT_LINE_HEIGHT,
                    Color::WHITE,
                    Justify::Left,
                );
                spawn_bound_text_styled(
                    panel,
                    UserEquipUiElement::StatusHpValue,
                    USER_EQUIP_STATUS_HP_RECT,
                    "0/0",
                    "ui.inventory.status.hp_value",
                    "{current}/{maximum}",
                    assets.font.clone(),
                    USER_EQUIP_SMALL_FONT_SIZE,
                    USER_EQUIP_SMALL_FONT_LINE_HEIGHT,
                    Color::WHITE,
                    Justify::Right,
                );
                spawn_bound_text_styled(
                    panel,
                    UserEquipUiElement::StatusFusionMatter,
                    USER_EQUIP_STATUS_FUSION_RECT,
                    "FUSION MATTER",
                    "ui.inventory.status.fusion_matter_label",
                    "FUSION MATTER",
                    assets.font.clone(),
                    USER_EQUIP_SMALL_FONT_SIZE,
                    USER_EQUIP_SMALL_FONT_LINE_HEIGHT,
                    Color::WHITE,
                    Justify::Left,
                );
                spawn_bound_text_styled(
                    panel,
                    UserEquipUiElement::StatusFusionMatterValue,
                    USER_EQUIP_STATUS_FUSION_RECT,
                    "0/0",
                    "ui.inventory.status.fusion_matter_value",
                    "{current}/{maximum}",
                    assets.font.clone(),
                    USER_EQUIP_SMALL_FONT_SIZE,
                    USER_EQUIP_SMALL_FONT_LINE_HEIGHT,
                    Color::WHITE,
                    Justify::Right,
                );
                spawn_bound_text_styled(
                    panel,
                    UserEquipUiElement::StatusGuideLabel,
                    USER_EQUIP_STATUS_GUIDE_LABEL_RECT,
                    "GUIDE:",
                    "ui.inventory.status.guide",
                    "GUIDE:",
                    assets.font.clone(),
                    USER_EQUIP_SMALL_FONT_SIZE,
                    USER_EQUIP_SMALL_FONT_LINE_HEIGHT,
                    Color::srgb(0.806_569_34, 1.0, 1.0),
                    Justify::Left,
                );
                spawn_bound_text_styled(
                    panel,
                    UserEquipUiElement::StatusGuideName,
                    USER_EQUIP_STATUS_GUIDE_NAME_RECT,
                    "",
                    "ui.inventory.status.guide_name",
                    "{name}",
                    assets.font.clone(),
                    USER_EQUIP_SMALL_FONT_SIZE,
                    USER_EQUIP_SMALL_FONT_LINE_HEIGHT,
                    Color::srgb(0.645_255_45, 1.0, 1.0),
                    Justify::Left,
                );
                panel.spawn((
                    UserEquipUiElement::StatusGuideIcon,
                    Node {
                        display: Display::None,
                        ..USER_EQUIP_STATUS_GUIDE_ICON_RECT.node()
                    },
                    ImageNode::default(),
                    Pickable::IGNORE,
                ));
                spawn_nano_status_panels(panel, &assets);
                panel.spawn((
                    Button,
                    UserEquipUiElement::TurnLeftPositioned,
                    UserEquipAvatarTurnControl(UserEquipAvatarTurnDirection::LeftPositioned),
                    USER_EQUIP_TURN_LEFT_POSITIONED_RECT.node(),
                    stretched_image(assets.image(UserEquipStaticAssetRole::TurnRight)),
                ));
                panel.spawn((
                    Button,
                    UserEquipUiElement::TurnRightPositioned,
                    UserEquipAvatarTurnControl(UserEquipAvatarTurnDirection::RightPositioned),
                    USER_EQUIP_TURN_RIGHT_POSITIONED_RECT.node(),
                    stretched_image(assets.image(UserEquipStaticAssetRole::TurnLeft)),
                ));
            });
            root.spawn((
                UserEquipUiElement::PcStuffPanel,
                USER_EQUIP_PC_STUFF_RECT.node(),
                Pickable::IGNORE,
            ))
            .with_children(|panel| {
                // Clean draws the inactive tab first, then begins the active
                // FFInvBack/FFNanoBack group. Keeping the background as a
                // later child is what makes the inactive tab recede behind
                // the active silhouette instead of floating above it.
                panel.spawn((
                    UserEquipUiElement::ItemTab,
                    USER_EQUIP_ITEM_TAB_TEXTURE_RECT.node(),
                    stretched_image(assets.image(UserEquipStaticAssetRole::ItemTab)),
                    Pickable::IGNORE,
                ));
                panel.spawn((
                    UserEquipUiElement::NanoTab,
                    USER_EQUIP_NANO_TAB_TEXTURE_RECT.node(),
                    stretched_image(assets.image(UserEquipStaticAssetRole::NanoTab)),
                    Pickable::IGNORE,
                ));
                panel.spawn((
                    UserEquipUiElement::PcStuffPanelBackground,
                    USER_EQUIP_PC_STUFF_RECT.node(),
                    sliced_image(
                        assets.image(UserEquipStaticAssetRole::InventoryPanel),
                        USER_EQUIP_INVENTORY_PANEL_BORDER,
                    ),
                    Pickable::IGNORE,
                ));
                spawn_tab_label(
                    panel,
                    UserEquipUiElement::ItemTabLabel,
                    USER_EQUIP_ITEM_TAB_HIT_RECT,
                    "EQUIPMENT",
                    assets.font.clone(),
                );
                spawn_nano_tab_label(
                    panel,
                    UserEquipUiElement::NanoTabLabel,
                    USER_EQUIP_NANO_TAB_HIT_RECT,
                    "NANOS",
                    assets.font.clone(),
                );
                for (element, rect) in [
                    (
                        UserEquipUiElement::InventoryShadowA,
                        USER_EQUIP_INVENTORY_SHADOW_A_RECT,
                    ),
                    (
                        UserEquipUiElement::InventoryShadowB,
                        USER_EQUIP_INVENTORY_SHADOW_B_RECT,
                    ),
                ] {
                    panel.spawn((
                        element,
                        rect.node(),
                        ImageNode {
                            color: Color::BLACK,
                            ..sliced_image(
                                assets.image(UserEquipStaticAssetRole::InventoryShadow),
                                USER_EQUIP_INVENTORY_SHADOW_BORDER,
                            )
                        },
                        Pickable::IGNORE,
                    ));
                }
                panel
                    .spawn((
                        UserEquipUiElement::InventoryViewport,
                        Node {
                            overflow: Overflow::clip(),
                            ..USER_EQUIP_INVENTORY_VIEWPORT_RECT.node()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|viewport| {
                        viewport
                            .spawn((
                                UserEquipUiElement::InventoryContent,
                                UserEquipUiRect::new(
                                    0.0,
                                    0.0,
                                    USER_EQUIP_INVENTORY_CONTENT_WIDTH,
                                    USER_EQUIP_INVENTORY_CONTENT_HEIGHT,
                                )
                                .node(),
                                Pickable::IGNORE,
                            ))
                            .with_children(|content| {
                                for slot in 0..INVENTORY_SLOT_COUNT_0104 {
                                    spawn_inventory_slot(content, slot, &assets);
                                }
                            });
                    });
                panel
                    .spawn((
                        UserEquipUiElement::NanoViewport,
                        Node {
                            display: Display::None,
                            overflow: Overflow::clip(),
                            ..USER_EQUIP_INVENTORY_VIEWPORT_RECT.node()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|viewport| {
                        viewport
                            .spawn((
                                UserEquipUiElement::NanoContent,
                                UserEquipUiRect::new(
                                    0.0,
                                    0.0,
                                    USER_EQUIP_INVENTORY_CONTENT_WIDTH,
                                    USER_EQUIP_NANO_CONTENT_HEIGHT,
                                )
                                .node(),
                                Pickable::IGNORE,
                            ))
                            .with_children(|content| {
                                for slot in 0..USER_EQUIP_NANO_GALLERY_COUNT {
                                    spawn_nano_gallery_slot(content, slot, &assets);
                                }
                            });
                    });
                panel.spawn((
                    UserEquipUiElement::ScrollTrack,
                    USER_EQUIP_SCROLL_TRACK_RECT.node(),
                    sliced_image(
                        assets.image(UserEquipStaticAssetRole::ScrollTrack),
                        BorderRect {
                            min_inset: Vec2::new(2.0, 4.0),
                            max_inset: Vec2::new(2.0, 4.0),
                        },
                    ),
                    Button,
                    UserEquipScrollTrackControl,
                    ZIndex(3),
                ));
                panel.spawn((
                    Button,
                    UserEquipUiElement::ScrollUp,
                    UserEquipScrollUpControl,
                    USER_EQUIP_SCROLL_UP_RECT.node(),
                    stretched_image(assets.image(UserEquipStaticAssetRole::ScrollUp)),
                    ZIndex(4),
                ));
                panel.spawn((
                    Button,
                    UserEquipUiElement::ScrollDown,
                    UserEquipScrollDownControl,
                    USER_EQUIP_SCROLL_DOWN_RECT.node(),
                    stretched_image(assets.image(UserEquipStaticAssetRole::ScrollDown)),
                    ZIndex(4),
                ));
                panel.spawn((
                    UserEquipUiElement::ScrollThumb,
                    UserEquipUiRect::new(357.0, 48.0, 13.0, 15.0).node(),
                    sliced_image(
                        assets.image(UserEquipStaticAssetRole::ScrollThumb),
                        BorderRect {
                            min_inset: Vec2::new(2.0, 4.0),
                            max_inset: Vec2::new(2.0, 4.0),
                        },
                    ),
                    Button,
                    UserEquipScrollThumbControl,
                    ZIndex(5),
                ));
                spawn_close_button(
                    panel,
                    UserEquipUiElement::Close,
                    USER_EQUIP_CLOSE_RECT,
                    assets.image(UserEquipStaticAssetRole::Close),
                );
                panel.spawn((
                    Button,
                    UserEquipUiElement::Trash,
                    UserEquipTrashControl,
                    USER_EQUIP_TRASH_RECT.node(),
                    stretched_image(assets.image(UserEquipStaticAssetRole::Trash)),
                ));
                panel.spawn((
                    Button,
                    UserEquipUiElement::Help,
                    UserEquipHelpControl,
                    USER_EQUIP_HELP_RECT.node(),
                    stretched_image(assets.image(UserEquipStaticAssetRole::Help)),
                ));
                panel.spawn((
                    UserEquipUiElement::TarosBack,
                    USER_EQUIP_TAROS_BACK_RECT.node(),
                    stretched_image(assets.image(UserEquipStaticAssetRole::Taros)),
                    Pickable::IGNORE,
                ));
                panel.spawn((
                    UserEquipUiElement::Dexlabs,
                    USER_EQUIP_DEXLABS_RECT.node(),
                    stretched_image(assets.image(UserEquipStaticAssetRole::Dexlabs)),
                    Pickable::IGNORE,
                ));
                for index in 0..9 {
                    spawn_bound_text_styled(
                        panel,
                        UserEquipUiElement::TarosDigit(index),
                        user_equip_taros_digit_rect(index),
                        "0",
                        "ui.inventory.status.taros_digit",
                        "{digit}",
                        assets.font.clone(),
                        12.0,
                        USER_EQUIP_REGULAR_FONT_LINE_HEIGHT,
                        Color::WHITE,
                        Justify::Center,
                    );
                }
                spawn_help_panel(panel, &assets);
                redeem::spawn_control(panel, &assets, &asset_server);
            });
            root.spawn((
                UserEquipUiElement::EquipmentPanel,
                USER_EQUIP_EQUIP_STRIP_RECT.node(),
                Pickable::IGNORE,
            ))
            .with_children(|panel| {
                panel
                    .spawn((
                        UserEquipUiElement::EquipmentTitle,
                        Node {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..USER_EQUIP_EQUIPMENT_TITLE_RECT.node()
                        },
                        stretched_image(assets.image(UserEquipStaticAssetRole::EquipTitle)),
                        Pickable::IGNORE,
                    ))
                    .with_children(|title| {
                        title.spawn((
                            UserEquipUiElement::EquipmentTitleLabel,
                            Text::new(" EQUIPPED"),
                            (
                                TextFont {
                                    font: (assets.font.clone()).into(),
                                    font_size: (USER_EQUIP_SMALL_FONT_SIZE).into(),
                                    ..default()
                                },
                                LineHeight::Px(USER_EQUIP_SMALL_FONT_LINE_HEIGHT),
                            ),
                            TextColor(Color::srgb(0.0, 0.2, 0.4)),
                            TextLayout::new(Justify::Center, LineBreak::NoWrap),
                            LocalizedText::new("ui.inventory.equipped", " EQUIPPED"),
                            Pickable::IGNORE,
                        ));
                    });
                for visual_index in 0..USER_EQUIP_EQUIPMENT_STRIP_COUNT {
                    spawn_equipment_slot(panel, visual_index, &assets);
                }
                spawn_battery_half_slots(panel, &assets);
            });
            spawn_item_popup(root, &assets);
            spawn_nano_viewer(root, &assets);
            root.spawn((
                UserEquipUiElement::DraggedItem,
                Node {
                    display: Display::None,
                    width: px(USER_EQUIP_INVENTORY_SLOT_SIZE),
                    height: px(USER_EQUIP_INVENTORY_SLOT_SIZE),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ImageNode::default(),
                ZIndex(200),
                Pickable::IGNORE,
            ));
        });
}

pub(super) fn spawn_tab_label(
    parent: &mut ChildSpawnerCommands,
    marker: UserEquipUiElement,
    rect: UserEquipUiRect,
    text: &'static str,
    font: Handle<Font>,
) {
    parent.spawn((
        Button,
        UserEquipItemTabControl,
        marker,
        Node {
            // Clean `blankbox` is UpperLeft with zero padding.
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Start,
            ..rect.node()
        },
        Text::new(text),
        (
            TextFont {
                font: (font).into(),
                font_size: (USER_EQUIP_TAB_FONT_SIZE).into(),
                ..default()
            },
            LineHeight::Px(USER_EQUIP_REGULAR_FONT_LINE_HEIGHT),
        ),
        TextColor(Color::srgb(0.794_354_86, 1.0, 1.0)),
        TextLayout::new(Justify::Left, LineBreak::NoWrap),
        inventory_static_localized(text),
    ));
}

pub(super) fn spawn_nano_tab_label(
    parent: &mut ChildSpawnerCommands,
    marker: UserEquipUiElement,
    rect: UserEquipUiRect,
    text: &'static str,
    font: Handle<Font>,
) {
    parent.spawn((
        Button,
        UserEquipNanoTabControl,
        marker,
        Node {
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Start,
            ..rect.node()
        },
        Text::new(text),
        (
            TextFont {
                font: (font).into(),
                font_size: (USER_EQUIP_TAB_FONT_SIZE).into(),
                ..default()
            },
            LineHeight::Px(USER_EQUIP_REGULAR_FONT_LINE_HEIGHT),
        ),
        TextColor(Color::srgb(0.794_354_86, 1.0, 1.0)),
        TextLayout::new(Justify::Left, LineBreak::NoWrap),
        inventory_static_localized(text),
    ));
}

pub(super) fn spawn_close_button(
    parent: &mut ChildSpawnerCommands,
    marker: UserEquipUiElement,
    rect: UserEquipUiRect,
    image: Handle<Image>,
) {
    parent.spawn((
        Button,
        marker,
        UserEquipCloseControl,
        rect.node(),
        stretched_image(image),
    ));
}

pub(super) fn user_equip_equipfont_node(rect: UserEquipUiRect) -> Node {
    // FusionFallInvenSkin/pathId 1366 `equipfont` uses Unity TextAnchor 5
    // (MiddleRight), with zero padding. The clean equipment-caption Rect is
    // already y = slot_y - 2, so no additional baseline offset belongs here.
    Node {
        justify_content: JustifyContent::End,
        align_items: AlignItems::Center,
        ..rect.node()
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_bound_text_styled(
    parent: &mut ChildSpawnerCommands,
    element: UserEquipUiElement,
    rect: UserEquipUiRect,
    text: &'static str,
    key: &'static str,
    fallback: &'static str,
    font: Handle<Font>,
    font_size: f32,
    line_height: f32,
    color: Color,
    justify: Justify,
) {
    let vertically_centered = matches!(
        element,
        UserEquipUiElement::StatusHp
            | UserEquipUiElement::StatusHpValue
            | UserEquipUiElement::StatusFusionMatter
            | UserEquipUiElement::StatusFusionMatterValue
            | UserEquipUiElement::BoostLabel
            | UserEquipUiElement::PotionLabel
            | UserEquipUiElement::TarosDigit(_)
            | UserEquipUiElement::ItemPopupField(2..=4)
            | UserEquipUiElement::ItemPopupField(6..=13)
            | UserEquipUiElement::ItemPopupAmountLabel
            | UserEquipUiElement::ItemPopupAmountValue
            | UserEquipUiElement::ItemPopupKeypadLabel(_)
    );
    let default_label_top_padding = user_equip_default_label_top_padding(element);
    let wraps = matches!(
        element,
        UserEquipUiElement::ItemPopupTitle | UserEquipUiElement::ItemPopupIdentity
    );
    let mut text_entity = None;
    let mut container = parent.spawn((
        element,
        Node {
            justify_content: match justify {
                Justify::Left | Justify::Start => JustifyContent::Start,
                Justify::Center => JustifyContent::Center,
                Justify::Right | Justify::End => JustifyContent::End,
                Justify::Justified => JustifyContent::SpaceBetween,
            },
            align_items: if vertically_centered {
                AlignItems::Center
            } else {
                AlignItems::Start
            },
            padding: UiRect::top(px(default_label_top_padding)),
            ..rect.node()
        },
        Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
    ));
    container.with_children(|text_parent| {
        text_entity = Some(
            text_parent
                .spawn((
                    Node {
                        width: if wraps { percent(100) } else { Val::Auto },
                        ..default()
                    },
                    Text::new(text),
                    (
                        TextFont {
                            font: (font).into(),
                            font_size: (font_size).into(),
                            ..default()
                        },
                        LineHeight::Px(line_height),
                    ),
                    TextColor(color),
                    TextLayout::new(
                        if wraps { justify } else { Justify::Left },
                        if wraps {
                            LineBreak::WordBoundary
                        } else {
                            LineBreak::NoWrap
                        },
                    ),
                    LocalizedText::new(key, fallback),
                    Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ))
                .id(),
        );
    });
    if wraps {
        let fit_font = (
            TextFont {
                font_size: (font_size).into(),
                ..default()
            },
            LineHeight::Px(line_height),
        );
        container.commands().entity(text_entity.unwrap()).insert(
            crate::localization::UiTextAutoFit::new(
                if matches!(element, UserEquipUiElement::ItemPopupTitle) {
                    170.
                } else {
                    280.
                },
                if matches!(element, UserEquipUiElement::ItemPopupTitle) {
                    28.
                } else {
                    40.
                },
                &fit_font,
            ),
        );
    }
    container.insert(UserEquipBoundTextTarget(
        text_entity.expect("bound text child is spawned synchronously"),
    ));
}

pub(super) fn user_equip_default_label_top_padding(element: UserEquipUiElement) -> f32 {
    match element {
        // FusionFallInvenSkin's built-in label style is UpperLeft with exact
        // 3 px top/bottom padding. Battery baseline placement comes from the
        // explicit DoEquipPanel value Rects, not an invented font offset.
        UserEquipUiElement::BoostValue
        | UserEquipUiElement::PotionValue
        | UserEquipUiElement::ItemPopupTitle
        | UserEquipUiElement::ItemPopupField(0 | 1 | 5) => 3.0,
        _ => 0.0,
    }
}

pub(super) fn spawn_help_panel(parent: &mut ChildSpawnerCommands, assets: &UserEquipUiAssets) {
    parent
        .spawn((
            UserEquipUiElement::HelpPanel,
            crate::ui::shared::controller::ControllerUiBoundary,
            Node {
                display: Display::None,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                padding: UiRect::all(px(16)),
                row_gap: px(16),
                ..USER_EQUIP_HELP_PANEL_RECT.node()
            },
            BackgroundColor(Color::srgba(0.02, 0.08, 0.14, 0.97)),
        ))
        .with_children(|help| {
            help.spawn((
                UserEquipUiElement::HelpTitle,
                Text::new("INVENTORY HELP"),
                (TextFont { font: (assets.font.clone()).into(), font_size: (15.0).into(), ..default() }, LineHeight::Px(17.0)),
                TextColor(Color::srgb(0.8, 1.0, 1.0)),
                TextLayout::new(Justify::Center, LineBreak::NoWrap),
                LocalizedText::new("ui.inventory.help.title", "INVENTORY HELP"),
                Pickable::IGNORE,
            ));
            help.spawn((
                UserEquipUiElement::HelpBody,
                Text::new("Left-click an item to choose an action. Right-click to equip or use it immediately. Drag items between inventory and equipment slots to move or swap them."),
                (TextFont { font: (assets.font.clone()).into(), font_size: (USER_EQUIP_POPUP_FONT_SIZE).into(), ..default() }, LineHeight::Px(16.0)),
                TextColor(Color::WHITE),
                TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                LocalizedText::new(
                    "ui.inventory.help.body",
                    "Left-click an item to choose an action. Right-click to equip or use it immediately. Drag items between inventory and equipment slots to move or swap them.",
                ),
                Pickable::IGNORE,
            ));
            help
                .spawn((
                    Button,
                    UserEquipUiElement::HelpClose,
                    UserEquipHelpCloseControl,
                    Node {
                        height: px(38),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.06, 0.25, 0.36)),
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("CLOSE"),
                        (TextFont { font: (assets.font.clone()).into(), font_size: (USER_EQUIP_POPUP_FONT_SIZE).into(), ..default() }, LineHeight::Px(USER_EQUIP_REGULAR_FONT_LINE_HEIGHT)),
                        TextColor(Color::WHITE),
                        LocalizedText::new("ui.common.close", "CLOSE"),
                        Pickable::IGNORE,
                    ));
                });
        });
}
