use super::*;

#[test]
fn reward_totals_start_from_authority_accumulate_and_expire_without_mutating_wallet() {
    let mut notices = RewardNotices::default();
    assert!(!notices.status.needs_tick());
    notices.receive_currencies(100, 200, 120, 240);
    assert_eq!(notices.status.counters[0].current, 200);
    notices.advance(0.1);
    assert_eq!(notices.status.counters[0].current, 200);
    notices.advance(0.001);
    assert_eq!(notices.status.counters[0].current, 208);
    assert_eq!(notices.status.counters[1].current, 104);
    notices.receive_currencies(120, 240, 150, 260);
    assert_eq!(notices.status.counters[0].current, 208);
    assert_eq!(notices.status.counters[0].target, 260);
    for _ in 0..100 {
        notices.advance(0.11);
    }
    assert_eq!(notices.status.counters[0].current, 260);
    assert_eq!(notices.status.counters[1].current, 150);
    assert!(!notices.status.counters.iter().any(Counter::visible));
    // Spending between rewards must clamp the display to the real wallet.
    notices.receive_currencies(10, 30, 15, 35);
    assert_eq!(notices.status.counters[0].current, 30);
    assert_eq!(notices.status.counters[1].current, 10);
    notices.clear();
    assert!(!notices.status.initialized);
    assert!(!notices.status.needs_tick());
}

#[test]
fn unchanged_or_capped_reward_does_not_flash_and_shared_timer_quirk_is_preserved() {
    let mut notices = RewardNotices::default();
    notices.receive_currencies(100, 200, 100, 200);
    assert!(!notices.status.counters.iter().any(Counter::visible));
    notices.receive_currencies(100, 200, 110, 200);
    assert!(!notices.status.counters[0].visible());
    assert!(notices.status.counters[1].visible());
    notices.status.counters[1].remaining = 0.5;
    assert_eq!(currency_alpha(&notices.status, 1), 0.5);
    notices.status.counters[0].remaining = 1.5;
    assert_eq!(currency_alpha(&notices.status, 1), 1.0);
}

#[test]
fn full_inventory_persists_after_bounce_yields_to_crates_and_clears_on_free_slot() {
    let mut notices = RewardNotices::default();
    notices.set_inventory_full(Some(true));
    notices.advance(1.5);
    assert!(!notices.status.full);
    notices.advance(0.01);
    assert!(notices.status.full);
    assert_eq!(notices.status.full_remaining, 2.0);
    notices.advance(3.0);
    assert!(notices.status.full);
    assert_eq!(notices.status.full_remaining, 0.0);
    notices.icons[0].push_back(IconNotice {
        count: 1,
        remaining: 2.0,
    });
    notices.set_inventory_full(Some(false));
    notices.advance(4.01);
    assert!(
        notices.status.full,
        "crate holds inventory polling for this frame"
    );
    notices.advance(0.01);
    assert!(!notices.status.full);
    notices.set_inventory_full(Some(true));
    notices.advance(1.51);
    assert!(notices.status.full);
    notices.set_inventory_full(None);
    assert!(!notices.status.full);
    notices.clear();
    assert_eq!(notices.status.inventory_full, None);
}

#[test]
fn status_entities_show_localized_counters_and_persistent_badge_then_hide_for_chat() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()))
        .init_asset::<Image>()
        .init_asset::<Font>()
        .insert_resource(GameplayUiModel {
            visible: true,
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
        .add_systems(Update, update);
    {
        let mut notices = app.world_mut().resource_mut::<RewardNotices>();
        notices.receive_currencies(12300, 4500, 12345, 4567);
        notices.set_inventory_full(Some(true));
        notices.advance(1.51);
    }
    app.update();
    let world = app.world_mut();
    let mut panels = world.query::<(&StatusImage, &Node, &ImageNode)>();
    assert_eq!(panels.iter(world).count(), 3);
    assert!(
        panels
            .iter(world)
            .all(|(_, n, image)| n.display == Display::Flex
                && matches!(image.image_mode, NodeImageMode::Stretch))
    );
    let mut digits = world.query::<(&CurrencyDigit, &LocalizedText)>();
    assert_eq!(digits.iter(world).count(), 18);
    assert!(
        digits
            .iter(world)
            .all(|(_, text)| text.key == "ui.gameplay.reward.counter_digit")
    );
    world.resource_mut::<RewardNotices>().icons[0].push_back(IconNotice {
        count: 1,
        remaining: 2.0,
    });
    app.update();
    let world = app.world_mut();
    assert!(
        panels
            .iter(world)
            .any(|(role, node, _)| matches!(role, StatusImage::FullInventory)
                && node.display == Display::None)
    );
    world.resource_mut::<GameplayUiModel>().chat.active = true;
    app.update();
    assert!(
        panels
            .iter(app.world())
            .all(|(_, node, _)| node.display == Display::None)
    );
}
