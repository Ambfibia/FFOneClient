//! Active Nano info panel spawning and binding.

use super::assets::GameplayUiAssets;
use super::hud::GameplayUiRect;
use super::model::GameplayUiModel;
use super::nano_wheel::{
    NANO_COOLDOWN_FINE_STEP_CAPACITY, NANO_COOLDOWN_FINE_STEP_DEGREES, NanoWheelTransientUi,
    nano_cooldown_layers, skill_icon_asset_path,
};
use super::text::hud_font;
use crate::localization::LocalizedText;
use bevy::{prelude::*, ui::widget::NodeImageMode};

/// Clean `CnGuiNano_info.RenderRect`. This is a separate top-left HUD surface,
/// not part of the bottom-right `cnNanoWheel`.
pub const ACTIVE_NANO_INFO_RECT: GameplayUiRect = GameplayUiRect::new(101.0, 42.0, 154.0, 29.0);
pub const ACTIVE_NANO_INFO_WINDOW_RECT: GameplayUiRect = GameplayUiRect::new(0.0, 0.0, 154.0, 29.0);
pub const ACTIVE_NANO_NAME_RECT: GameplayUiRect = GameplayUiRect::new(20.0, 0.0, 100.0, 20.0);
pub const ACTIVE_NANO_SKILL_RECT: GameplayUiRect = GameplayUiRect::new(125.0, 2.0, 26.0, 26.0);
/// `CnGuiNano_info.DoWindow` subtracts one pixel from the authored skill Rect
/// before drawing cooldown slices.
pub const ACTIVE_NANO_COOLDOWN_RECT: GameplayUiRect = GameplayUiRect::new(124.0, 2.0, 26.0, 26.0);
pub const ACTIVE_NANO_STAMINA_RECT: GameplayUiRect = GameplayUiRect::new(18.0, 19.0, 104.0, 4.0);

pub const ACTIVE_NANO_INFO_TEXTURE_PATH: &str = "ui/en/gameplay/nano/active/nano_info.png";
pub const ACTIVE_NANO_INFO_TEXTURE_BYTES: u64 = 1486;
pub const ACTIVE_NANO_INFO_TEXTURE_SHA256: &str =
    "7ad1e55cb1d8d983fd2df3a559a0cdb88880940c25b51f9c1f7539d7fe8f5be6";

#[derive(Component)]
pub(super) struct ActiveNanoInfoRoot;
#[derive(Component)]
pub(super) struct ActiveNanoInfoName;
#[derive(Component)]
pub(super) struct ActiveNanoInfoStaminaFill;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ActiveNanoInfoLayer {
    SkillIcon,
    CooldownBase,
    CooldownRotating,
    CooldownFine(u8),
}
#[derive(Component)]
pub(super) struct ActiveNanoInfoImage(pub(super) ActiveNanoInfoLayer);

/// Exact `CnGuiNano_info` child of clean `GameHUD`: active Nano name,
/// stamina, tuned-skill icon, and the active-skill cooldown overlay.
pub(super) fn spawn_active_nano_info(parent: &mut ChildSpawnerCommands, assets: &GameplayUiAssets) {
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                display: Display::None,
                left: px(ACTIVE_NANO_INFO_RECT.x),
                top: px(ACTIVE_NANO_INFO_RECT.y),
                width: px(ACTIVE_NANO_INFO_RECT.width),
                height: px(ACTIVE_NANO_INFO_RECT.height),
                ..default()
            },
            UiTransform::default(),
            ActiveNanoInfoRoot,
            ZIndex(11),
        ))
        .with_children(|panel| {
            panel.spawn((
                ACTIVE_NANO_INFO_WINDOW_RECT.node(),
                ImageNode {
                    image: assets.active_nano_info.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                ZIndex(0),
            ));
            panel.spawn((
                Node {
                    display: Display::None,
                    ..ACTIVE_NANO_SKILL_RECT.node()
                },
                ImageNode {
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                ActiveNanoInfoImage(ActiveNanoInfoLayer::SkillIcon),
                UiTransform::default(),
                ZIndex(1),
            ));
            for layer in [
                ActiveNanoInfoLayer::CooldownRotating,
                ActiveNanoInfoLayer::CooldownBase,
            ] {
                panel.spawn((
                    Node {
                        display: Display::None,
                        ..ACTIVE_NANO_COOLDOWN_RECT.node()
                    },
                    ImageNode {
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                    ActiveNanoInfoImage(layer),
                    UiTransform::default(),
                    ZIndex(2),
                ));
            }
            for step in 0..NANO_COOLDOWN_FINE_STEP_CAPACITY {
                panel.spawn((
                    Node {
                        display: Display::None,
                        ..ACTIVE_NANO_COOLDOWN_RECT.node()
                    },
                    ImageNode {
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                    ActiveNanoInfoImage(ActiveNanoInfoLayer::CooldownFine(step as u8)),
                    UiTransform::default(),
                    ZIndex(2),
                ));
            }
            panel
                .spawn((Node {
                    align_items: AlignItems::Center,
                    ..ACTIVE_NANO_NAME_RECT.node()
                },))
                .with_child((
                    Text::new(""),
                    hud_font(&assets.chalet_font, 12.0),
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    TextLayout::default().with_justify(Justify::Left),
                    LocalizedText::new("ui.gameplay.nano_info.name", "{name}").with_arg("name", ""),
                    ActiveNanoInfoName,
                ));
            crate::damage_bar::spawn_damage_bar(
                panel,
                ACTIVE_NANO_STAMINA_RECT.node(),
                ImageNode {
                    image: assets.nano_stamina_fill.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                (ActiveNanoInfoStaminaFill, ZIndex(3)),
            );
        });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn bind_active_nano_info(
    model: Res<GameplayUiModel>,
    transient: Res<NanoWheelTransientUi>,
    assets: Res<GameplayUiAssets>,
    asset_server: Res<AssetServer>,
    mut root: Single<
        &mut Node,
        (
            With<ActiveNanoInfoRoot>,
            Without<ActiveNanoInfoStaminaFill>,
            Without<ActiveNanoInfoImage>,
        ),
    >,
    mut name: Single<&mut LocalizedText, With<ActiveNanoInfoName>>,
    mut stamina: Single<
        (&mut Node, &mut crate::damage_bar::DamageBarOwner),
        (
            With<ActiveNanoInfoStaminaFill>,
            Without<ActiveNanoInfoRoot>,
            Without<ActiveNanoInfoImage>,
        ),
    >,
    mut images: Query<
        (
            &ActiveNanoInfoImage,
            &mut Node,
            &mut ImageNode,
            &mut UiTransform,
        ),
        (
            Without<ActiveNanoInfoRoot>,
            Without<ActiveNanoInfoStaminaFill>,
        ),
    >,
) {
    if !model.is_changed() && !transient.is_changed() && !assets.is_changed() {
        return;
    }
    let Some((slot_index, slot)) = model
        .nanos
        .iter()
        .enumerate()
        .find(|(_, slot)| slot.active && slot.nano_id.is_some())
    else {
        root.display = Display::None;
        return;
    };
    if !model.visible {
        root.display = Display::None;
        return;
    }

    root.display = Display::Flex;
    **name = LocalizedText::new(
        format!("content.nano.{}.name", slot.nano_id.unwrap_or_default()),
        &slot.name,
    );
    stamina.0.width = px(active_nano_stamina_fill_width(slot.stamina_fraction));
    stamina.1.0 = slot.nano_id.unwrap_or_default() as u64;

    let has_skill = slot.skill_id.is_some() && slot.skill_icon_number.is_some();
    let cooldown = nano_cooldown_layers(
        slot.active_skill && has_skill,
        transient.slots[slot_index].active_skill_cooldown_remaining,
    );
    for (marker, mut node, mut image, mut transform) in &mut images {
        let visible = match marker.0 {
            ActiveNanoInfoLayer::SkillIcon => has_skill,
            ActiveNanoInfoLayer::CooldownBase => {
                cooldown.is_some_and(|layers| layers.base_texture_index.is_some())
            }
            ActiveNanoInfoLayer::CooldownRotating => {
                cooldown.is_some_and(|layers| layers.rotating_texture_index.is_some())
            }
            ActiveNanoInfoLayer::CooldownFine(step) => {
                cooldown.is_some_and(|layers| usize::from(step) < layers.fine_step_count)
            }
        };
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        if !visible {
            continue;
        }
        transform.rotation = Rot2::IDENTITY;
        image.image = match marker.0 {
            ActiveNanoInfoLayer::SkillIcon => asset_server.load(skill_icon_asset_path(
                slot.skill_icon_number
                    .expect("visible active Nano skill has an icon"),
            )),
            ActiveNanoInfoLayer::CooldownBase => assets.nano_cooldown_slices[cooldown
                .expect("visible active Nano cooldown has validated layers")
                .base_texture_index
                .expect("visible active Nano cooldown base has a texture index")]
            .clone(),
            ActiveNanoInfoLayer::CooldownRotating => {
                let layers = cooldown.expect("visible active Nano cooldown has validated layers");
                transform.rotation = Rot2::radians(layers.rotating_degrees.to_radians());
                assets.nano_cooldown_slices[layers
                    .rotating_texture_index
                    .expect("visible active Nano rotating cooldown has a texture index")]
                .clone()
            }
            ActiveNanoInfoLayer::CooldownFine(step) => {
                transform.rotation =
                    Rot2::radians(f32::from(step) * NANO_COOLDOWN_FINE_STEP_DEGREES.to_radians());
                assets.nano_cooldown_fine.clone()
            }
        };
    }
}

/// Exact `CnGuiNano_info.DoWindow` active-Nano stamina geometry. Unlike the
/// wheel, this surface always draws the fill, including at full stamina.
pub(super) fn active_nano_stamina_fill_width(stamina_fraction: f32) -> f32 {
    if stamina_fraction.is_finite() {
        104.0 * stamina_fraction.clamp(0.0, 1.0)
    } else {
        0.0
    }
}
