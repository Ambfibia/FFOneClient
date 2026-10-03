//! Server-authoritative quest-drop notices and reward icon queues.

use super::*;
use crate::{
    localization::{
        Language, Localization, LocalizedTextCase, localized_tabledata_npc_name,
        localized_tabledata_quest_item_name,
    },
    quick_slot_ui::QuickSlotUiConfig,
    tutorial_mission_content::TutorialMissionContent,
    world_mission_runtime::WorldMissionRuntime,
};
use ffone_protocol::RewardItemReply0104;

mod shiny;
mod status;

pub(super) fn install(app: &mut App) {
    app.init_resource::<RewardNotices>().add_systems(
        Update,
        (update, status::update, shiny::update)
            .chain()
            .in_set(GameplayUiSet::Rewards)
            .before(LocalizationSet::Apply),
    );
}

pub const REWARD_TEXTURES: [(&str, usize, &str); 3] = [
    (
        "ui/en/gameplay/rewards/crate.png",
        39937,
        "3272717257362843fa19dca30cc2cef83c756ce5528358369cc9cc80723de2ba",
    ),
    (
        "ui/en/gameplay/rewards/potion.png",
        40183,
        "60ec405991ba8209f5e1e5cf16fbe3a78725c1ed64af114eccf7656a7b99ed85",
    ),
    (
        "ui/en/gameplay/rewards/boost.png",
        39367,
        "9de3d9effa33ddb456170b136b97d30417b019f8ab62b7bc4bb529360724c94e",
    ),
];

#[derive(Clone, Debug)]
struct QuestNotice {
    npc_type: i32,
    npc_name: String,
    item_id: i32,
    item_name: String,
    found: bool,
    remaining: f32,
}

#[derive(Clone, Debug)]
struct IconNotice {
    count: i32,
    remaining: f32,
}

#[derive(Resource, Default, Debug)]
pub struct RewardNotices {
    shiny: VecDeque<shiny::Notice>,
    quest: VecDeque<QuestNotice>,
    icons: [VecDeque<IconNotice>; 3],
    status: status::RewardStatus,
}

impl RewardNotices {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Call only after inventory validation, before replacing battery totals
    /// and before mission completion consumes the active task.
    pub fn receive(
        &mut self,
        reply: &RewardItemReply0104,
        content: &TutorialMissionContent,
        mission: &WorldMissionRuntime,
        previous_nano_battery: i32,
        previous_weapon_battery: i32,
    ) {
        let mut has_quest_item = false;
        for reward in &reply.items {
            if reward.item.item_type == 8 {
                has_quest_item = true;
                if reply.npc_type_id > 0 {
                    self.push_quest(
                        content,
                        reply.npc_type_id,
                        i32::from(reward.item.item_id),
                        true,
                    );
                }
            } else if reward.inventory_location == 1
                && reward.item.item_type == 9
                && content.reward_is_crate(reward.item.item_id)
            {
                // Wire iOpt is the complete slot post-state, not a drop count.
                self.icons[0].push_back(IconNotice {
                    count: 1,
                    remaining: 2.0,
                });
            }
        }
        if !has_quest_item
            && reply.npc_type_id > 0
            && mission
                .active_tasks()
                .iter()
                .any(|task| task.task_id == reply.task_id)
            && let Some(item) = content.reward_quest_item(reply.task_id, reply.npc_type_id)
        {
            self.push_quest(content, reply.npc_type_id, item, false);
        }
        for (lane, current, previous) in [
            (1, reply.nano_battery, previous_nano_battery),
            (2, reply.weapon_battery, previous_weapon_battery),
        ] {
            let count = current.saturating_sub(previous);
            if count > 0 {
                self.icons[lane].push_back(IconNotice {
                    count,
                    remaining: 2.0,
                });
            }
        }
    }

    fn push_quest(
        &mut self,
        content: &TutorialMissionContent,
        npc_type: i32,
        item_id: i32,
        found: bool,
    ) {
        let (Some(item_name), Some(npc)) = (
            content.quest_item_name(item_id),
            content.gameplay_npc(npc_type),
        ) else {
            return;
        };
        self.quest.push_back(QuestNotice {
            npc_type,
            npc_name: npc.name.clone(),
            item_id,
            item_name: item_name.to_owned(),
            found,
            remaining: 3.0,
        });
    }

    pub fn advance(&mut self, delta: f32) {
        if let Some(front) = self.shiny.front_mut() {
            front.remaining -= delta;
            if front.remaining < -2.0 {
                self.shiny.pop_front();
            }
        }
        self.status.advance(delta, !self.icons[0].is_empty());
        if let Some(front) = self.quest.front_mut() {
            front.remaining -= delta;
            if front.remaining <= 0.0 {
                self.quest.pop_front();
            }
        }
        for lane in &mut self.icons {
            if let Some(front) = lane.front_mut() {
                front.remaining -= delta;
                if front.remaining < -2.0 {
                    lane.pop_front();
                }
            }
        }
    }
}

fn quest_text(
    notice: &QuestNotice,
    localization: Option<&Localization>,
    language: Option<&Language>,
) -> (LocalizedText, String) {
    let resolve = |value: LocalizedText| match (localization, language) {
        (Some(catalog), Some(language)) => catalog.text(language, &value),
        _ => value.fallback,
    };
    let (key, fallback) = if notice.found {
        ("ui.gameplay.reward.quest_found", "The {mob} had: {item}!")
    } else {
        (
            "ui.gameplay.reward.quest_missing",
            "The {mob} did not have: {item}.",
        )
    };
    let mob = resolve(localized_tabledata_npc_name(
        notice.npc_type,
        &notice.npc_name,
    ));
    let item = resolve(localized_tabledata_quest_item_name(
        notice.item_id,
        &notice.item_name,
    ));
    let text = LocalizedText::new(key, fallback)
        .with_arg("mob", &mob)
        .with_arg("item", &item);
    let resolved = match (localization, language) {
        (Some(catalog), Some(language)) => catalog.text(language, &text),
        _ => fallback.replace("{mob}", &mob).replace("{item}", &item),
    }
    .to_uppercase();
    (text, resolved)
}

fn icon_text(lane: usize, count: i32) -> LocalizedText {
    match lane {
        0 => LocalizedText::new("ui.gameplay.reward.crate", "+1 C.R.A.T.E"),
        1 => LocalizedText::new("ui.gameplay.reward.potions", "+{count} POTIONS")
            .with_arg("count", count.to_string()),
        _ => LocalizedText::new("ui.gameplay.reward.boosts", "+{count} BOOSTS")
            .with_arg("count", count.to_string()),
    }
}

#[derive(Component)]
pub(super) struct QuestNoticeLabel {
    shadow: bool,
}
#[derive(Component)]
pub(super) struct RewardIcon {
    lane: usize,
}
#[derive(Component)]
pub(super) struct RewardIconLabel {
    lane: usize,
    shadow: bool,
}

pub(super) fn spawn(
    hud: &mut ChildSpawnerCommands,
    assets: &GameplayUiAssets,
    server: &AssetServer,
) {
    status::spawn(hud, assets, server);
    shiny::spawn(hud, assets, server);
    for shadow in [true, false] {
        hud.spawn((
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                height: px(40),
                padding: UiRect::new(px(10), px(6), px(4), px(6)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Text::new(""),
            LocalizedText::new("ui.gameplay.reward.quest_found", "The {mob} had: {item}!")
                .with_arg("mob", "")
                .with_arg("item", ""),
            LocalizedTextCase::Uppercase,
            (
                TextFont {
                    font: (assets.jeffe_font.clone()).into(),
                    font_size: (14.0).into(),
                    ..default()
                },
                LineHeight::Px(16.451_999_66),
            ),
            TextLayout::default().with_justify(Justify::Center),
            TextColor(Color::WHITE),
            QuestNoticeLabel { shadow },
            Pickable::IGNORE,
            ZIndex(940),
        ));
    }
    for (lane, (path, _, _)) in REWARD_TEXTURES.iter().enumerate() {
        hud.spawn((
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: px(178),
                height: px(142),
                ..default()
            },
            ImageNode {
                // These three textures have the same proven single-mip,
                // bilinear/repeat sampling contract as the HUD Nano wheel.
                image: load_nano_wheel_image(server, path),
                image_mode: NodeImageMode::Stretch,
                ..default()
            },
            RewardIcon { lane },
            Pickable::IGNORE,
            ZIndex(940),
        ))
        .with_children(|icon| {
            for shadow in [true, false] {
                icon.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(if shadow { 1 } else { 0 }),
                        top: px(if shadow { 111 } else { 110 }),
                        width: px(178),
                        height: px(30),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    Text::new(""),
                    icon_text(lane, 0),
                    chat_jeffe_14_font(&assets.jeffe_font),
                    UiTransform::from_scale(Vec2::new(1.0, CHAT_JEFFE_14_VERTICAL_SCALE)),
                    TextLayout::default().with_justify(Justify::Center),
                    TextColor(if shadow { Color::BLACK } else { Color::WHITE }),
                    RewardIconLabel { lane, shadow },
                    Pickable::IGNORE,
                ));
            }
        });
    }
}

/// Returns physical pixels after the bottom-center UI scale pivot.
pub fn icon_position(
    remaining: f32,
    lane: usize,
    viewport: Vec2,
    scale: f32,
    quick_slot: bool,
) -> (Vec2, f32) {
    let height = 170.0 + if quick_slot { 40.0 } else { 0.0 };
    let fraction = remaining / 2.0;
    let phase = 2.0 - fraction;
    let bounce = if remaining > 0.0 {
        (std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * phase.powi(3))
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
            viewport.x * 0.5 + (-225.0 + lane as f32 * 150.0) * scale,
            viewport.y + (bounce - height) * scale,
        ),
        alpha,
    )
}

pub(super) fn update(
    time: Res<Time>,
    windows: Query<&Window, With<PrimaryWindow>>,
    model: Res<GameplayUiModel>,
    mut notices: ResMut<RewardNotices>,
    localization: Option<Res<Localization>>,
    language: Option<Res<Language>>,
    quick_slot: Option<Res<QuickSlotUiConfig>>,
    transition: Option<Res<GameplayMenuTransition>>,
    mut quest_labels: Query<
        (
            &QuestNoticeLabel,
            &mut Node,
            &mut LocalizedText,
            &mut TextColor,
            &mut UiTransform,
        ),
        (Without<RewardIcon>, Without<RewardIconLabel>),
    >,
    mut icons: Query<
        (&RewardIcon, &mut Node, &mut ImageNode, &mut UiTransform),
        Without<RewardIconLabel>,
    >,
    mut icon_labels: Query<(&RewardIconLabel, &mut LocalizedText, &mut TextColor)>,
) {
    if !notices.shiny.is_empty()
        || !notices.quest.is_empty()
        || notices.icons.iter().any(|lane| !lane.is_empty())
        || notices.status.needs_tick()
    {
        notices.advance(time.delta_secs());
    }
    let Ok(window) = windows.single() else { return };
    // IsInputEnabled reads bChatEnable after fSlideMenu settles. Native
    // input_enabled is permission to use chat, not its active editing state.
    let chat_editing = model.chat.active
        && transition
            .as_ref()
            .is_none_or(|transition| transition.remaining <= 0.0);
    let visible = model.visible && !chat_editing;
    let quest = notices.quest.front().filter(|_| visible);
    let resolved =
        quest.map(|notice| quest_text(notice, localization.as_deref(), language.as_deref()));
    for (label, mut node, mut text, mut color, mut transform) in &mut quest_labels {
        let Some((notice, (localized, resolved))) = quest.zip(resolved.as_ref()) else {
            if node.display != Display::None {
                node.display = Display::None;
            }
            continue;
        };
        let t = (1.0 - notice.remaining).clamp(0.0, 1.0);
        let half = resolved.encode_utf16().count() as f32 * 8.0;
        let shadow = if label.shadow { 1.0 } else { 0.0 };
        node.display = Display::Flex;
        node.width = px(half * 2.0 + 20.0);
        node.left =
            px(window.width() * 0.5 - half + if notice.found { 100.0 * t } else { 0.0 } + shadow);
        node.top = px(window.height() * 0.25
            + if notice.found {
                (280.0 - window.height() * 0.25) * t
            } else {
                150.0 * t
            }
            + shadow);
        transform.scale = Vec2::splat(model.ui_scale);
        text.set_if_neq(localized.clone());
        let rgb = if label.shadow {
            [0.0; 3]
        } else if notice.found {
            [166.0 / 255.0, 176.0 / 255.0, 1.0]
        } else {
            [234.0 / 255.0, 52.0 / 255.0, 36.0 / 255.0]
        };
        color.0 = Color::srgba(rgb[0], rgb[1], rgb[2], notice.remaining.min(1.0));
    }
    for (icon, mut node, mut image, mut transform) in &mut icons {
        let Some(notice) = notices.icons[icon.lane].front().filter(|_| visible) else {
            if node.display != Display::None {
                node.display = Display::None;
            }
            continue;
        };
        let (position, alpha) = icon_position(
            notice.remaining,
            icon.lane,
            Vec2::new(window.width(), window.height()),
            model.ui_scale,
            quick_slot
                .as_ref()
                .is_some_and(|config| config.localized_mode.draws_component()),
        );
        node.display = Display::Flex;
        // UiTransform scales around the node center; compensate to retain the
        // source rectangle's top-left after its screen-pivot transform.
        node.left = px(position.x + 89.0 * (model.ui_scale - 1.0));
        node.top = px(position.y + 71.0 * (model.ui_scale - 1.0));
        transform.scale = Vec2::splat(model.ui_scale);
        image.color = Color::srgba(1.0, 1.0, 1.0, alpha);
    }
    for (label, mut text, mut color) in &mut icon_labels {
        if let Some(notice) = notices.icons[label.lane].front() {
            text.set_if_neq(icon_text(label.lane, notice.count));
            // Original labels restore opaque black/white after icon fading.
            color.0 = if label.shadow {
                Color::BLACK
            } else {
                Color::WHITE
            };
        }
    }
}

#[cfg(test)]
mod tests;
