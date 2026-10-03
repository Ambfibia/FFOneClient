use super::*;
use crate::assets::AssetLocator;
use crate::world_mission_runtime::WorldMissionServerEvent0104;
use ffone_protocol::{ItemBase0104, ItemReward0104, PcTaskStartSuccess0104};
use std::collections::BTreeSet;
use std::{path::PathBuf, sync::OnceLock};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game")
}
fn content() -> &'static TutorialMissionContent {
    static CONTENT: OnceLock<TutorialMissionContent> = OnceLock::new();
    CONTENT
        .get_or_init(|| TutorialMissionContent::open(&AssetLocator::open(root()).unwrap()).unwrap())
}
fn active() -> WorldMissionRuntime {
    let mut mission = WorldMissionRuntime::default();
    mission
        .apply_event(
            &WorldMissionServerEvent0104::TaskStartSuccess(PcTaskStartSuccess0104 {
                task_id: 2248,
                remaining_time: 0,
            }),
            content(),
        )
        .unwrap();
    mission
}
fn reply() -> RewardItemReply0104 {
    RewardItemReply0104 {
        candy: 100,
        fusion_matter: 200,
        nano_battery: 10,
        weapon_battery: 20,
        pack_padding: [0; 3],
        fatigue: 0,
        fatigue_level: 0,
        npc_type_id: 2676,
        task_id: 2248,
        items: vec![],
    }
}
fn item(kind: i16, id: i16, location: i32) -> ItemReward0104 {
    ItemReward0104 {
        item: ItemBase0104 {
            item_type: kind,
            item_id: id,
            option: 5,
            time_limit: 0,
        },
        inventory_location: location,
        slot: 0,
    }
}

#[test]
fn production_reward_distinguishes_missing_and_found_without_inventing_drops() {
    let content = content();
    assert_eq!(content.reward_quest_item(2248, 2676), Some(537));
    assert_eq!(content.reward_quest_item(2248, 274), None);
    let mut notices = RewardNotices::default();
    let mut packet = reply();
    notices.receive(&packet, content, &active(), 10, 20);
    assert_eq!(notices.quest.len(), 1);
    assert!(!notices.quest[0].found);
    assert_eq!(notices.quest[0].item_id, 537);
    packet.items.push(item(8, 537, 2));
    notices.receive(&packet, content, &active(), 10, 20);
    assert_eq!(notices.quest.len(), 2);
    assert!(notices.quest[1].found);
    assert!(notices.icons.iter().all(VecDeque::is_empty));
    notices.advance(3.0);
    assert!(notices.quest[0].found);
    assert_eq!(notices.quest[0].remaining, 3.0);
    notices.advance(3.0);
    assert!(notices.quest.is_empty());
    packet.items.clear();
    notices.receive(&packet, content, &WorldMissionRuntime::default(), 10, 20);
    assert!(
        notices.quest.is_empty(),
        "inactive task must not produce missing-item copy"
    );
    packet.task_id = 4;
    notices.receive(&packet, content, &active(), 10, 20);
    assert!(
        notices.quest.is_empty(),
        "a different task must not borrow the active objective"
    );
}

#[test]
fn crate_uses_packet_edge_not_post_state_quantity_and_batteries_use_deltas() {
    let mut notices = RewardNotices::default();
    let mut packet = reply();
    packet.items = vec![item(9, 1, 1), item(9, 1, 0)];
    notices.receive(&packet, content(), &active(), 7, 18);
    assert_eq!(notices.icons[0].len(), 1);
    assert_eq!(notices.icons[0][0].count, 1);
    assert_eq!(notices.icons[1][0].count, 3);
    assert_eq!(notices.icons[2][0].count, 2);
    notices.receive(&packet, content(), &active(), 10, 20);
    assert_eq!(notices.icons[0].len(), 2);
    assert_eq!(notices.icons[1].len(), 1);
    notices.advance(4.0);
    assert_eq!(
        notices.icons[0].len(),
        2,
        "source expires strictly below minus two"
    );
    notices.advance(0.01);
    assert_eq!(notices.icons[0].len(), 1);
    assert_eq!(notices.icons[0][0].remaining, 2.0);
    notices.clear();
    assert!(notices.quest.is_empty());
    assert!(notices.icons.iter().all(VecDeque::is_empty));
}

#[test]
fn icon_bounce_fade_and_quick_slot_pivot_match_source() {
    let viewport = Vec2::new(1280.0, 720.0);
    assert_eq!(
        icon_position(0.0, 0, viewport, 1.0, false),
        (Vec2::new(415.0, 550.0), 1.0)
    );
    assert_eq!(
        icon_position(-1.5, 0, viewport, 1.0, false),
        (Vec2::new(415.0, 550.0), 0.5)
    );
    assert_eq!(
        icon_position(-2.0, 0, viewport, 1.0, true),
        (Vec2::new(415.0, 510.0), 0.0)
    );
    assert!((icon_position(2.0, 0, viewport, 1.0, false).0.y - 720.0).abs() < 0.001);
    assert_eq!(
        icon_position(0.0, 2, viewport, 1.5, true).0,
        Vec2::new(752.5, 405.0)
    );
}

#[test]
fn production_nodes_render_both_notices_and_respect_chat_visibility() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()))
        .init_asset::<Image>()
        .init_asset::<Font>()
        .insert_resource(GameplayUiModel {
            visible: true,
            chat: ChatUi {
                input_enabled: true,
                active: false,
                ..default()
            },
            ..default()
        })
        .init_resource::<RewardNotices>()
        .add_systems(
            Startup,
            |mut commands: Commands, server: Res<AssetServer>| {
                let assets = GameplayUiAssets::load(&server);
                commands
                    .spawn(Node::default())
                    .with_children(|hud| spawn(hud, &assets, &server));
                commands.spawn((Window::default(), PrimaryWindow));
            },
        )
        .add_systems(Update, (update, status::update).chain());
    let mut packet = reply();
    packet.items.push(item(9, 1, 1));
    app.world_mut()
        .resource_mut::<RewardNotices>()
        .receive(&packet, content(), &active(), 7, 18);
    app.world_mut()
        .resource_mut::<RewardNotices>()
        .receive_currencies(90, 180, 100, 200);
    app.update();
    {
        let world = app.world_mut();
        let mut labels = world
            .query_filtered::<(&Node, &LocalizedText, &LocalizedTextCase), With<QuestNoticeLabel>>(
            );
        assert_eq!(labels.iter(world).count(), 2);
        for (node, text, case) in labels.iter(world) {
            assert_eq!(node.display, Display::Flex);
            assert_eq!(text.key, "ui.gameplay.reward.quest_missing");
            assert_eq!(*case, LocalizedTextCase::Uppercase);
        }
        let mut icons = world.query_filtered::<(&Node, &ImageNode), With<RewardIcon>>();
        assert_eq!(icons.iter(world).count(), 3);
        for (node, image) in icons.iter(world) {
            assert_eq!(node.display, Display::Flex);
            assert_eq!(node.width, px(178));
            assert!(matches!(image.image_mode, NodeImageMode::Stretch));
        }
    }
    app.world_mut()
        .resource_mut::<GameplayUiModel>()
        .chat
        .active = true;
    app.update();
    {
        let world = app.world_mut();
        let mut nodes =
            world.query_filtered::<&Node, Or<(With<RewardIcon>, With<QuestNoticeLabel>)>>();
        assert!(nodes.iter(world).all(|node| node.display == Display::None));
        assert_eq!(world.resource::<RewardNotices>().quest.len(), 1);
    }
    app.world_mut().resource_mut::<RewardNotices>().clear();
    app.world_mut()
        .resource_mut::<GameplayUiModel>()
        .chat
        .active = false;
    app.update();
    let world = app.world_mut();
    let mut nodes = world.query_filtered::<&Node, Or<(With<RewardIcon>, With<QuestNoticeLabel>)>>();
    assert!(nodes.iter(world).all(|node| node.display == Display::None));
}

#[test]
fn production_assets_and_localized_notices_have_exact_contracts() {
    use sha2::{Digest, Sha256};
    for (path, bytes, hash) in REWARD_TEXTURES.into_iter().chain(status::STATUS_TEXTURES) {
        let data = std::fs::read(root().join(path)).unwrap();
        assert_eq!(data.len(), bytes);
        assert_eq!(format!("{:x}", Sha256::digest(&data)), hash);
        assert!(image::load_from_memory(&data).unwrap().width() > 0);
    }
    let (localization, en) = Localization::open(&root(), "en").unwrap();
    let (_, ru) = Localization::open(&root(), "ru").unwrap();
    let mut notices = RewardNotices::default();
    notices.receive(&reply(), content(), &active(), 7, 18);
    let notice = &notices.quest[0];
    for found in [false, true] {
        let notice = QuestNotice {
            found,
            ..notice.clone()
        };
        let (text_en, en_copy) = quest_text(&notice, Some(&localization), Some(&en));
        let (text_ru, ru_copy) = quest_text(&notice, Some(&localization), Some(&ru));
        assert_eq!(text_en.key, text_ru.key);
        assert_ne!(en_copy, ru_copy);
        assert!(!en_copy.contains('{') && !ru_copy.contains('{'));
        assert!(ru_copy.chars().any(|c| ('А'..='Я').contains(&c)));
    }
    let en_bundle: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root().join("localization/en.json")).unwrap())
            .unwrap();
    let ru_bundle: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root().join("localization/ru.json")).unwrap())
            .unwrap();
    let en_entries = en_bundle["entries"].as_object().unwrap();
    let ru_entries = ru_bundle["entries"].as_object().unwrap();
    assert_eq!(
        en_entries.keys().collect::<BTreeSet<_>>(),
        ru_entries.keys().collect::<BTreeSet<_>>()
    );
    for (key, value) in en_entries {
        let placeholders = |s: &str| {
            s.split('{')
                .skip(1)
                .filter_map(|v| v.split_once('}').map(|(key, _)| key.to_owned()))
                .collect::<BTreeSet<_>>()
        };
        assert_eq!(
            placeholders(value.as_str().unwrap()),
            placeholders(ru_entries[key].as_str().unwrap()),
            "{key}"
        );
    }
    for lane in 0..3 {
        let text = icon_text(lane, 3);
        assert!(en_entries.contains_key(&text.key) && ru_entries.contains_key(&text.key));
        assert_ne!(localization.text(&en, &text), localization.text(&ru, &text));
    }
    assert_eq!(en_entries["ui.gameplay.reward.counter_digit"], "{digit}");
    assert_eq!(ru_entries["ui.gameplay.reward.counter_digit"], "{digit}");
}
