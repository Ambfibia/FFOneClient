//! Nano wheel slots, cooldown layers, stamina and battery counters.

use super::assets::GameplayUiAssets;
use super::hud::GameplayUiRect;
use super::model::{GameplayNanoPortraitImages, GameplayUiModel};
use crate::localization::LocalizedText;
use bevy::{prelude::*, text::LineHeight, ui::widget::NodeImageMode};

pub const NANO_EMPTY_SLOTS_RECT: GameplayUiRect = GameplayUiRect::new(1037.0, 673.0, 223.0, 37.0);

pub const NANO_SLOT_KEY_RECTS: [GameplayUiRect; 3] = [
    GameplayUiRect::new(1055.0, 684.0, 36.0, 36.0),
    GameplayUiRect::new(1133.0, 684.0, 36.0, 36.0),
    GameplayUiRect::new(1209.0, 684.0, 36.0, 36.0),
];
pub const NANO_WHEEL_WINDOW_RECT: GameplayUiRect = GameplayUiRect::new(0.0, 64.0, 223.0, 37.0);
pub const NANO_AFFINITY_BACK_RECTS: [GameplayUiRect; 3] = [
    GameplayUiRect::new(7.0, 16.0, 56.0, 72.0),
    GameplayUiRect::new(83.0, 16.0, 56.0, 72.0),
    GameplayUiRect::new(160.0, 16.0, 56.0, 72.0),
];
pub const NANO_GUMBALL_RECTS: [GameplayUiRect; 3] = [
    // Clean Retrobution's serialized cnNanoWheel overrides the constructor's
    // stale (7, 30)/(83, 30)/(160, 30) defaults with these live rectangles.
    GameplayUiRect::new(-3.0, -27.0, 71.0, 104.0),
    GameplayUiRect::new(74.0, -27.0, 71.0, 104.0),
    GameplayUiRect::new(151.0, -27.0, 71.0, 104.0),
];
pub const NANO_PORTRAIT_RECTS: [GameplayUiRect; 3] = [
    GameplayUiRect::new(8.0, 20.0, 64.0, 64.0),
    GameplayUiRect::new(85.0, 20.0, 64.0, 64.0),
    GameplayUiRect::new(162.0, 20.0, 64.0, 64.0),
];
/// Extend the animated camera above the old square by 36 logical pixels.
/// The matching off-axis render view preserves the original scale and bottom edge.
pub const NANO_ANIMATED_PORTRAIT_RECTS: [GameplayUiRect; 3] = [
    GameplayUiRect::new(0.0, -24.0, 72.0, 108.0),
    GameplayUiRect::new(77.0, -24.0, 72.0, 108.0),
    GameplayUiRect::new(154.0, -24.0, 72.0, 108.0),
];
pub const NANO_AFFINITY_ICON_RECTS: [GameplayUiRect; 3] = [
    GameplayUiRect::new(50.0, 35.0, 15.0, 15.0),
    GameplayUiRect::new(127.0, 35.0, 15.0, 15.0),
    GameplayUiRect::new(204.0, 35.0, 15.0, 15.0),
];
pub const NANO_SKILL_RECTS: [GameplayUiRect; 3] = [
    GameplayUiRect::new(-5.0, 5.0, 26.0, 26.0),
    GameplayUiRect::new(72.0, 5.0, 26.0, 26.0),
    GameplayUiRect::new(149.0, 5.0, 26.0, 26.0),
];
/// `cnNanoWheel.DoWindow` moves the authored SkillRect one pixel left before
/// drawing every cooldown wedge.
pub const NANO_COOLDOWN_RECTS: [GameplayUiRect; 3] = [
    GameplayUiRect::new(-6.0, 5.0, 26.0, 26.0),
    GameplayUiRect::new(71.0, 5.0, 26.0, 26.0),
    GameplayUiRect::new(148.0, 5.0, 26.0, 26.0),
];
pub(super) const NANO_COOLDOWN_FINE_STEP_DEGREES: f32 = 5.0;
// In the source's first-sector path `num6` can approach 96.43 degrees before
// `num4` reaches zero, so j=0..18 (19 sprites) is the exact safe capacity.
pub(super) const NANO_COOLDOWN_FINE_STEP_CAPACITY: usize = 19;

pub const NANO_STAMINA_BACK_RECTS: [GameplayUiRect; 3] = [
    GameplayUiRect::new(22.0, 17.0, 42.0, 6.0),
    GameplayUiRect::new(99.0, 17.0, 42.0, 6.0),
    GameplayUiRect::new(176.0, 17.0, 42.0, 6.0),
];
pub const NANO_STAMINA_FILL_RECTS: [GameplayUiRect; 3] = [
    GameplayUiRect::new(23.0, 18.0, 40.0, 4.0),
    GameplayUiRect::new(100.0, 18.0, 40.0, 4.0),
    GameplayUiRect::new(177.0, 18.0, 40.0, 4.0),
];
pub(super) const NANO_WHEEL_EXACT_SAMPLER_PATHS: [&str; 10] = [
    "ui/en/gameplay/nano/skill/cooldown/cooltime01.png",
    "ui/en/gameplay/nano/skill/cooldown/cooltime02.png",
    "ui/en/gameplay/nano/skill/cooldown/cooltime03.png",
    "ui/en/gameplay/nano/skill/cooldown/cooltime04.png",
    "ui/en/gameplay/nano/skill/cooldown/cooltime05.png",
    "ui/en/gameplay/nano/skill/cooldown/cooltime06.png",
    "ui/en/gameplay/nano/skill/cooldown/cooltime07.png",
    "ui/en/gameplay/nano/skill/cooldown/CoolTime_00.png",
    "ui/en/gameplay/nano/stamina/nano_st_bar.png",
    "ui/en/gameplay/nano/stamina/HP_BAR.png",
];
pub const NANO_BATTERY_COUNTER_RECT: GameplayUiRect = GameplayUiRect::new(-81.0, 26.0, 76.0, 29.0);
pub const WEAPON_BATTERY_COUNTER_RECT: GameplayUiRect =
    GameplayUiRect::new(-81.0, 61.0, 76.0, 29.0);
pub const NANO_BATTERY_LABEL_RECT: GameplayUiRect = NANO_BATTERY_COUNTER_RECT;
pub const WEAPON_BATTERY_LABEL_RECT: GameplayUiRect = WEAPON_BATTERY_COUNTER_RECT;
pub(super) const BATTERY_COUNTER_FONT_SIZE: f32 = 9.0;
pub(super) const BATTERY_COUNTER_LINE_HEIGHT: f32 = 13.71;
pub(super) const BATTERY_COUNTER_TEXT_Y_OFFSET: f32 = 1.0;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NanoSlotUi {
    pub nano_id: Option<i16>,
    /// Localized TableData name projected by the gameplay owner. The HUD uses
    /// it only as the argument of a stable semantic localization template.
    pub name: String,
    pub skill_id: Option<i16>,
    pub style: Option<u8>,
    /// Validated native GLB route selected through the Nano table and semantic
    /// character registry. Animated HUD slots never substitute the 2D icon.
    pub model_path: Option<String>,
    pub nano_icon_number: Option<u16>,
    pub skill_icon_number: Option<u16>,
    pub active_skill: bool,
    pub stamina_fraction: f32,
    pub active: bool,
}

/// Transient `cnNanoWheel` state whose clean source owner is
/// `GameCondition`/`cnMainGame`, not the equipped-Nano table.
///
/// The resource defaults to no effects, so incomplete or unavailable network
/// state cannot leave a stale gumball or cooldown overlay on screen.
#[derive(Clone, Debug, Default, PartialEq, Resource)]
pub struct NanoWheelTransientUi {
    pub slots: [NanoWheelTransientSlotUi; 3],
}

impl NanoWheelTransientUi {
    /// Exact `SetNanoGumBallEffect`: -1 clears every slot; 0..=2 enables the
    /// requested slot without implicitly clearing any other slot.
    ///
    /// Invalid packet values fail closed and do not mutate the last proven
    /// state.
    pub fn set_gumball_effect(&mut self, slot: i32) -> bool {
        match slot {
            -1 => {
                for state in &mut self.slots {
                    state.gumball_enabled = false;
                }
                true
            }
            0..=2 => {
                self.slots[slot as usize].gumball_enabled = true;
                true
            }
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NanoWheelTransientSlotUi {
    pub gumball_enabled: bool,
    /// Normalized remaining duration from `GameCondition` (`1.0` immediately
    /// after activation, decreasing toward zero). `None`, non-finite and
    /// out-of-range values render no cooldown.
    pub active_skill_cooldown_remaining: Option<f32>,
}

#[derive(Component)]
pub(super) struct NanoWheelRoot;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NanoSlotLayer {
    Gumball,
    AffinityBackLower,
    Portrait,
    StaminaBack,
    StaminaFill,
    AffinityBackUpper,
    AffinityIcon,
    SkillBack,
    SkillIcon,
    CooldownBase,
    CooldownRotating,
    CooldownFine(u8),
}
#[derive(Component)]
pub(super) struct NanoSlotImage {
    pub(super) index: usize,
    pub(super) layer: NanoSlotLayer,
}
#[derive(Component)]
pub(super) struct NanoBatteryCounter;
#[derive(Component)]
pub(super) struct NanoBatteryCounterText;
#[derive(Component)]
pub(super) struct WeaponBatteryCounter;
#[derive(Component)]
pub(super) struct WeaponBatteryCounterText;

pub(super) fn spawn_nano_wheel(parent: &mut ChildSpawnerCommands, assets: &GameplayUiAssets) {
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: px(20),
                bottom: px(10),
                width: px(223),
                height: px(101),
                ..default()
            },
            UiTransform::default(),
            NanoWheelRoot,
        ))
        .with_children(|wheel| {
            // `DrawNanoCamera` explicitly calls `GUIClip.Unclip`; the expanded
            // third camera and the first authored SkillRect at x=-5 must not be
            // cropped by the native composition node. The two counters remain
            // separate because the source draws them after `GUI.EndGroup`.
            wheel
                .spawn((Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: px(223),
                    height: px(101),
                    ..default()
                },))
                .with_children(|group| {
                    group.spawn((
                        NANO_WHEEL_WINDOW_RECT.node(),
                        ImageNode {
                            image: assets.nano_empty.clone(),
                            image_mode: NodeImageMode::Stretch,
                            ..default()
                        },
                    ));
                    for index in 0..3 {
                        for (layer, rect, z_index) in [
                            (NanoSlotLayer::Gumball, NANO_GUMBALL_RECTS[index], 1),
                            (
                                NanoSlotLayer::AffinityBackLower,
                                NANO_AFFINITY_BACK_RECTS[index],
                                2,
                            ),
                            (
                                NanoSlotLayer::Portrait,
                                NANO_ANIMATED_PORTRAIT_RECTS[index],
                                3,
                            ),
                            (
                                NanoSlotLayer::StaminaBack,
                                NANO_STAMINA_BACK_RECTS[index],
                                4,
                            ),
                            (
                                NanoSlotLayer::StaminaFill,
                                NANO_STAMINA_FILL_RECTS[index],
                                5,
                            ),
                            (
                                NanoSlotLayer::AffinityBackUpper,
                                NANO_AFFINITY_BACK_RECTS[index],
                                6,
                            ),
                            (
                                NanoSlotLayer::AffinityIcon,
                                NANO_AFFINITY_ICON_RECTS[index],
                                7,
                            ),
                            (NanoSlotLayer::SkillBack, NANO_SKILL_RECTS[index], 8),
                            (NanoSlotLayer::SkillIcon, NANO_SKILL_RECTS[index], 9),
                            (
                                // `DoWindow` draws this rotated partial slice
                                // first and the completed sectors over it.
                                NanoSlotLayer::CooldownRotating,
                                NANO_COOLDOWN_RECTS[index],
                                10,
                            ),
                            (NanoSlotLayer::CooldownBase, NANO_COOLDOWN_RECTS[index], 10),
                        ] {
                            if matches!(layer, NanoSlotLayer::StaminaFill) {
                                crate::damage_bar::spawn_damage_bar(
                                    group,
                                    Node {
                                        display: Display::None,
                                        ..rect.node()
                                    },
                                    ImageNode {
                                        image: assets.nano_stamina_fill.clone(),
                                        image_mode: NodeImageMode::Stretch,
                                        ..default()
                                    },
                                    (
                                        NanoSlotImage { index, layer },
                                        UiTransform::default(),
                                        ZIndex(z_index),
                                    ),
                                );
                                continue;
                            }
                            group.spawn((
                                Node {
                                    display: Display::None,
                                    ..rect.node()
                                },
                                ImageNode {
                                    image_mode: NodeImageMode::Stretch,
                                    ..default()
                                },
                                NanoSlotImage { index, layer },
                                UiTransform::default(),
                                ZIndex(z_index),
                            ));
                        }
                        for step in 0..NANO_COOLDOWN_FINE_STEP_CAPACITY {
                            group.spawn((
                                Node {
                                    display: Display::None,
                                    ..NANO_COOLDOWN_RECTS[index].node()
                                },
                                ImageNode {
                                    image_mode: NodeImageMode::Stretch,
                                    ..default()
                                },
                                NanoSlotImage {
                                    index,
                                    layer: NanoSlotLayer::CooldownFine(step as u8),
                                },
                                UiTransform::default(),
                                ZIndex(10),
                            ));
                        }
                        group.spawn((
                            [
                                GameplayUiRect::new(18.0, 75.0, 36.0, 36.0),
                                GameplayUiRect::new(96.0, 75.0, 36.0, 36.0),
                                GameplayUiRect::new(172.0, 75.0, 36.0, 36.0),
                            ][index]
                                .node(),
                            ImageNode {
                                image: assets.nano_slot_keys[index].clone(),
                                image_mode: NodeImageMode::Stretch,
                                ..default()
                            },
                            ZIndex(11),
                        ));
                    }
                });

            for (rect, image, marker) in [
                (
                    NANO_BATTERY_COUNTER_RECT,
                    assets.nano_battery_counter.clone(),
                    true,
                ),
                (
                    WEAPON_BATTERY_COUNTER_RECT,
                    assets.weapon_battery_counter.clone(),
                    false,
                ),
            ] {
                let mut counter = wheel.spawn((
                    Node {
                        display: Display::None,
                        ..rect.node()
                    },
                    ImageNode {
                        image,
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                    ZIndex(10),
                ));
                if marker {
                    counter.insert(NanoBatteryCounter);
                } else {
                    counter.insert(WeaponBatteryCounter);
                }
            }

            for (rect, marker) in [
                (NANO_BATTERY_LABEL_RECT, true),
                (WEAPON_BATTERY_LABEL_RECT, false),
            ] {
                let mut label = wheel.spawn((
                    Node {
                        align_items: AlignItems::Start,
                        justify_content: JustifyContent::Center,
                        ..rect.node()
                    },
                    ZIndex(11),
                ));
                label.with_children(|label| {
                    let mut text = label.spawn((
                        Text::new("0000"),
                        LocalizedText::new("ui.content.passthrough", "{text}")
                            .with_arg("text", "0000"),
                        battery_counter_font(&assets.jeffe_font),
                        TextColor(Color::WHITE),
                        TextLayout::default().with_justify(Justify::Center),
                        UiTransform::from_translation(Val2::px(0.0, BATTERY_COUNTER_TEXT_Y_OFFSET)),
                    ));
                    if marker {
                        text.insert(NanoBatteryCounterText);
                    } else {
                        text.insert(WeaponBatteryCounterText);
                    }
                });
            }
        });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn bind_nano_wheel(
    model: Res<GameplayUiModel>,
    transient: Res<NanoWheelTransientUi>,
    assets: Res<GameplayUiAssets>,
    asset_server: Res<AssetServer>,
    portraits: Res<GameplayNanoPortraitImages>,
    mut slot_images: Query<
        (
            &NanoSlotImage,
            &mut Node,
            &mut ImageNode,
            &mut UiTransform,
            Option<&mut crate::damage_bar::DamageBarOwner>,
        ),
        (
            Without<NanoBatteryCounter>,
            Without<WeaponBatteryCounter>,
            Without<NanoBatteryCounterText>,
            Without<WeaponBatteryCounterText>,
        ),
    >,
    mut nano_counter: Single<
        (&mut Node, &mut ImageNode),
        (
            With<NanoBatteryCounter>,
            Without<WeaponBatteryCounter>,
            Without<NanoSlotImage>,
            Without<NanoBatteryCounterText>,
            Without<WeaponBatteryCounterText>,
        ),
    >,
    mut weapon_counter: Single<
        (&mut Node, &mut ImageNode),
        (
            With<WeaponBatteryCounter>,
            Without<NanoBatteryCounter>,
            Without<NanoSlotImage>,
            Without<NanoBatteryCounterText>,
            Without<WeaponBatteryCounterText>,
        ),
    >,
    mut nano_text: Single<
        (&mut Node, &mut LocalizedText, &mut TextColor),
        (
            With<NanoBatteryCounterText>,
            Without<WeaponBatteryCounterText>,
            Without<NanoBatteryCounter>,
            Without<WeaponBatteryCounter>,
            Without<NanoSlotImage>,
        ),
    >,
    mut weapon_text: Single<
        (&mut Node, &mut LocalizedText, &mut TextColor),
        (
            With<WeaponBatteryCounterText>,
            Without<NanoBatteryCounterText>,
            Without<NanoBatteryCounter>,
            Without<WeaponBatteryCounter>,
            Without<NanoSlotImage>,
        ),
    >,
) {
    if !model.is_changed()
        && !transient.is_changed()
        && !assets.is_changed()
        && !portraits.is_changed()
    {
        return;
    }
    for (marker, mut node, mut image, mut transform, owner) in &mut slot_images {
        if let Some(mut owner) = owner {
            let id = model.nanos[marker.index].nano_id.unwrap_or_default() as u64;
            if owner.0 != id {
                owner.0 = id;
            }
        }
        let slot = &model.nanos[marker.index];
        let transient = transient.slots[marker.index];
        let style = slot.style.filter(|style| *style <= 2);
        let has_nano = slot.nano_id.is_some() && slot.nano_icon_number.is_some() && style.is_some();
        let has_skill = slot.skill_id.is_some() && slot.skill_icon_number.is_some();
        let stamina_fill_width =
            nano_stamina_fill_width(has_nano, slot.active, slot.stamina_fraction);
        let stamina_visible = stamina_fill_width.is_some();
        let cooldown = nano_cooldown_layers(
            slot.active_skill && has_skill,
            transient.active_skill_cooldown_remaining,
        );
        let visible = match marker.layer {
            NanoSlotLayer::Gumball => has_nano && transient.gumball_enabled,
            NanoSlotLayer::Portrait => has_nano && portraits.0[marker.index].is_some(),
            NanoSlotLayer::AffinityBackLower
            | NanoSlotLayer::AffinityBackUpper
            | NanoSlotLayer::AffinityIcon => has_nano,
            NanoSlotLayer::StaminaBack | NanoSlotLayer::StaminaFill => stamina_visible,
            NanoSlotLayer::SkillBack | NanoSlotLayer::SkillIcon => has_skill,
            NanoSlotLayer::CooldownBase => {
                cooldown.is_some_and(|layers| layers.base_texture_index.is_some())
            }
            NanoSlotLayer::CooldownRotating => {
                cooldown.is_some_and(|layers| layers.rotating_texture_index.is_some())
            }
            NanoSlotLayer::CooldownFine(step) => {
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
        image.image = match marker.layer {
            NanoSlotLayer::Gumball => {
                assets.nano_gumballs[usize::from(style.expect("validated style"))].clone()
            }
            NanoSlotLayer::AffinityBackLower | NanoSlotLayer::AffinityBackUpper => {
                assets.nano_affinity_backs[usize::from(style.expect("validated style"))].clone()
            }
            NanoSlotLayer::Portrait => portraits.0[marker.index]
                .clone()
                .expect("visible animated Nano portrait has a render target"),
            NanoSlotLayer::StaminaBack => assets.nano_stamina_back.clone(),
            NanoSlotLayer::StaminaFill => {
                node.width = px(stamina_fill_width.expect("visible stamina has a source width"));
                assets.nano_stamina_fill.clone()
            }
            NanoSlotLayer::AffinityIcon => {
                assets.nano_affinity_icons[usize::from(style.expect("validated style"))].clone()
            }
            NanoSlotLayer::SkillBack => assets.nano_skill_back.clone(),
            NanoSlotLayer::SkillIcon => asset_server.load(skill_icon_asset_path(
                slot.skill_icon_number.expect("validated Skill icon"),
            )),
            NanoSlotLayer::CooldownBase => assets.nano_cooldown_slices[cooldown
                .expect("visible cooldown has validated layers")
                .base_texture_index
                .expect("visible cooldown base has a texture index")]
            .clone(),
            NanoSlotLayer::CooldownRotating => {
                let layers = cooldown.expect("visible cooldown has validated layers");
                transform.rotation = Rot2::radians(layers.rotating_degrees.to_radians());
                assets.nano_cooldown_slices[layers
                    .rotating_texture_index
                    .expect("visible rotating cooldown has a texture index")]
                .clone()
            }
            NanoSlotLayer::CooldownFine(step) => {
                transform.rotation =
                    Rot2::radians(f32::from(step) * NANO_COOLDOWN_FINE_STEP_DEGREES.to_radians());
                assets.nano_cooldown_fine.clone()
            }
        };
    }

    let (nano_counter_node, nano_counter_image) = &mut *nano_counter;
    let (nano_label_node, nano_label_localized, nano_label_color) = &mut *nano_text;
    bind_battery_counter(
        model.nano_battery,
        nano_counter_node,
        nano_counter_image,
        nano_label_node,
        nano_label_localized,
        nano_label_color,
    );
    let (weapon_counter_node, weapon_counter_image) = &mut *weapon_counter;
    let (weapon_label_node, weapon_label_localized, weapon_label_color) = &mut *weapon_text;
    bind_battery_counter(
        model.weapon_battery,
        weapon_counter_node,
        weapon_counter_image,
        weapon_label_node,
        weapon_label_localized,
        weapon_label_color,
    );
}

/// Exact `cnNanoWheel.DoWindow` inactive-slot stamina rule. The clean IMGUI
/// call changes the destination Rect width and stretches the complete 186px
/// gradient into it; it does not crop UVs.
pub(super) fn nano_stamina_fill_width(has_nano: bool, active: bool, stamina_fraction: f32) -> Option<f32> {
    let stamina_fraction = stamina_fraction.clamp(0.0, 1.0);
    (has_nano && !active && stamina_fraction < 0.99).then_some(40.0 * stamina_fraction)
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct NanoCooldownLayers {
    pub(super) base_texture_index: Option<usize>,
    pub(super) rotating_texture_index: Option<usize>,
    pub(super) rotating_degrees: f32,
    pub(super) fine_step_count: usize,
}

/// Exact normalized translation of `cnNanoWheel.DoWindow`'s seven coarse
/// slices plus repeated five-degree `CoolTime_00` first-sector fill.
///
/// Returning `None` for malformed or completed cooldown state intentionally
/// mirrors `GameCondition.IsCoolTime == false` and prevents stale overlays.
pub(super) fn nano_cooldown_layers(
    active_skill: bool,
    remaining_fraction: Option<f32>,
) -> Option<NanoCooldownLayers> {
    if !active_skill {
        return None;
    }
    let remaining_fraction = remaining_fraction?;
    if !remaining_fraction.is_finite() || !(0.0..=1.0).contains(&remaining_fraction) {
        return None;
    }
    if remaining_fraction == 0.0 {
        return None;
    }

    let sector = (remaining_fraction * 7.0).floor().min(7.0) as i32 - 1;
    let remaining_degrees = remaining_fraction * 360.0;
    let degrees_within_sector = remaining_degrees - sector as f32 * 45.0;
    if remaining_degrees > 45.0 && sector >= 0 {
        Some(NanoCooldownLayers {
            base_texture_index: Some(sector.max(1) as usize),
            rotating_texture_index: Some((sector + 1).min(6) as usize),
            rotating_degrees: degrees_within_sector - 45.0,
            fine_step_count: 0,
        })
    } else {
        Some(NanoCooldownLayers {
            fine_step_count: ((degrees_within_sector / NANO_COOLDOWN_FINE_STEP_DEGREES).floor()
                as usize)
                .min(NANO_COOLDOWN_FINE_STEP_CAPACITY),
            ..default()
        })
    }
}

pub(super) fn bind_battery_counter(
    count: i32,
    counter_node: &mut Node,
    counter_image: &mut ImageNode,
    label_node: &mut Node,
    label_localized: &mut LocalizedText,
    label_color: &mut TextColor,
) {
    let visible = count > 0;
    counter_node.display = if visible {
        Display::Flex
    } else {
        Display::None
    };
    label_node.display = counter_node.display;
    if !visible {
        return;
    }
    let value = battery_counter_text(count);
    *label_localized =
        LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", value);
    let color = if count < 100 {
        Color::srgb(1.0, 0.0, 0.0)
    } else {
        Color::WHITE
    };
    counter_image.color = color;
    label_color.0 = color;
}

pub(super) fn battery_counter_text(count: i32) -> String {
    format!("{count:04}")
}

pub(super) fn battery_counter_font(font: &Handle<Font>) -> (TextFont, LineHeight) {
    (
        TextFont {
            font: (font.clone()).into(),
            font_size: (BATTERY_COUNTER_FONT_SIZE).into(),
            ..default()
        },
        LineHeight::Px(BATTERY_COUNTER_LINE_HEIGHT),
    )
}

pub(super) fn skill_icon_asset_path(icon_number: u16) -> String {
    format!("ui/en/gameplay/nano/icons/skill/skillicon_{icon_number:02}.png")
}
