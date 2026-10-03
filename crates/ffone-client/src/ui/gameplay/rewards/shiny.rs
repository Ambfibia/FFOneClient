//! Coco reward queue: server skill/buff icon, four-second bounce/fade.
use super::*;
use ffone_protocol::wire_0104::ShinyPickupSuccess0104;

#[derive(Debug, Clone)]
pub(super) struct Notice {
    title: LocalizedText,
    icon: String,
    handle: Option<Handle<Image>>,
    pub(super) remaining: f32,
}

impl RewardNotices {
    pub fn receive_shiny(
        &mut self,
        reply: &ShinyPickupSuccess0104,
        content: &TutorialMissionContent,
        localization: &Localization,
        language: &Language,
    ) -> Option<LocalizedText> {
        let name = content.gameplay_skill_name(reply.skill_id)?;
        let icon = if reply.cstb == 0 {
            content.gameplay_skill(reply.skill_id)?.icon_number
        } else {
            content.gameplay_skill_buff(reply.cstb)?.icon_number
        };
        let title = LocalizedText::new(
            format!(
                "content.tabledata.skill.skill_string.{}.str_name",
                reply.skill_id
            ),
            name,
        );
        let bonus = localization.text(language, &title);
        self.shiny.push_back(Notice {
            title,
            icon: format!("icons/skills/skillicon_{icon:02}.png"),
            handle: None,
            remaining: 2.0,
        });
        Some(
            LocalizedText::new(
                "ui.gameplay.reward.coco_received",
                "You receive <{bonus}> from Coco's Egg",
            )
            .with_arg("bonus", bonus),
        )
    }
}

#[derive(Component)]
pub(super) struct CocoNotice;
#[derive(Component)]
pub(super) struct CocoIcon;
#[derive(Component)]
pub(super) struct CocoLabel {
    shadow: bool,
}

pub(super) fn spawn(
    hud: &mut ChildSpawnerCommands,
    assets: &GameplayUiAssets,
    server: &AssetServer,
) {
    hud.spawn((
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            width: px(178),
            height: px(135),
            ..default()
        },
        ImageNode::new(load_nano_wheel_image(
            server,
            "ui/en/gameplay/rewards/coco.png",
        )),
        CocoNotice,
        Pickable::IGNORE,
        ZIndex(940),
    ))
    .with_children(|group| {
        group.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(71),
                top: px(72),
                width: px(22),
                height: px(22),
                ..default()
            },
            ImageNode::default(),
            CocoIcon,
            Pickable::IGNORE,
        ));
        for shadow in [true, false] {
            group.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(if shadow { 1 } else { 0 }),
                    top: px(if shadow { 111 } else { 110 }),
                    width: px(178),
                    height: px(25),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                Text::new(""),
                LocalizedText::new("ui.gameplay.reward.coco_title", "{bonus}")
                    .with_arg("bonus", ""),
                chat_jeffe_14_font(&assets.jeffe_font),
                UiTransform::from_scale(Vec2::new(1.0, CHAT_JEFFE_14_VERTICAL_SCALE)),
                TextLayout::default().with_justify(Justify::Center),
                TextColor(Color::WHITE),
                CocoLabel { shadow },
                Pickable::IGNORE,
            ));
        }
    });
}

fn pose(remaining: f32, viewport: Vec2, scale: f32, quick_slot: bool) -> (Vec2, f32) {
    let height = 135.0 + if quick_slot { 40.0 } else { 0.0 };
    let fraction = remaining / 2.0;
    let bounce = if remaining > 0.0 {
        (std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * (2.0 - fraction).powi(3))
            .sin()
            .abs()
            * fraction.powi(2)
            * height
    } else {
        0.0
    };
    let alpha = if remaining > 0.0 {
        1.0
    } else {
        ((1.0 + remaining / 2.0) * 2.0).clamp(0.0, 1.0)
    };
    (
        Vec2::new(
            viewport.x * 0.5 - 89.0 * scale,
            viewport.y + (-height + bounce) * scale,
        ),
        alpha,
    )
}

pub(super) fn update(
    windows: Query<&Window, With<PrimaryWindow>>,
    model: Res<GameplayUiModel>,
    mut notices: ResMut<RewardNotices>,
    server: Res<AssetServer>,
    localization: Res<Localization>,
    language: Res<Language>,
    quick_slot: Option<Res<QuickSlotUiConfig>>,
    transition: Option<Res<GameplayMenuTransition>>,
    mut roots: Query<
        (&mut Node, &mut ImageNode, &mut UiTransform),
        (With<CocoNotice>, Without<CocoIcon>),
    >,
    mut icons: Query<&mut ImageNode, (With<CocoIcon>, Without<CocoNotice>)>,
    mut labels: Query<(&CocoLabel, &mut LocalizedText, &mut TextColor)>,
) {
    if let Some(front) = notices.shiny.front_mut() {
        if front.handle.is_none() {
            front.handle = Some(
                server
                    .load_builder()
                    .with_settings::<ImageLoaderSettings>(configure_nano_wheel_image)
                    .load::<Image>(front.icon.clone()),
            );
        }
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let chat_editing = model.chat.active && transition.as_ref().is_none_or(|v| v.remaining <= 0.0);
    let current = notices
        .shiny
        .front()
        .filter(|_| model.visible && !chat_editing);
    for (mut node, mut image, mut transform) in &mut roots {
        let Some(notice) = current else {
            if node.display != Display::None {
                node.display = Display::None;
            }
            continue;
        };
        let (p, alpha) = pose(
            notice.remaining,
            Vec2::new(window.width(), window.height()),
            model.ui_scale,
            quick_slot
                .as_ref()
                .is_some_and(|q| q.localized_mode.draws_component()),
        );
        if node.display != Display::Flex {
            node.display = Display::Flex;
        }
        let left = px(p.x + 89.0 * (model.ui_scale - 1.0));
        let top = px(p.y + 67.5 * (model.ui_scale - 1.0));
        if node.left != left {
            node.left = left;
        }
        if node.top != top {
            node.top = top;
        }
        let scale = Vec2::splat(model.ui_scale);
        if transform.scale != scale {
            transform.scale = scale;
        }
        let tint = Color::srgba(1.0, 1.0, 1.0, alpha);
        if image.color != tint {
            image.color = tint;
        }
        for mut icon in &mut icons {
            let handle = notice
                .handle
                .as_ref()
                .expect("loaded current notice")
                .clone();
            if icon.image != handle {
                icon.image = handle;
            }
            if icon.color != tint {
                icon.color = tint;
            }
        }
        for (label, mut text, mut color) in &mut labels {
            text.set_if_neq(
                LocalizedText::new("ui.gameplay.reward.coco_title", "{bonus}")
                    .with_arg("bonus", localization.text(&language, &notice.title)),
            );
            let value = if label.shadow { 0.0 } else { 1.0 };
            let tint = Color::srgba(value, value, value, alpha);
            if color.0 != tint {
                color.0 = tint;
            }
        }
    }
}

#[cfg(test)]
mod tests;
