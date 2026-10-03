use super::chat_log::chat_log_shows_latest;
use super::chat_model::{CHAT_SCROLL_TO_LATEST, CHAT_SCROLL_WHEEL_LINE};
use super::chat_scrollbar::chat_scrollbar_geometry;
use super::chat_spawn::{ChatLogViewport, ChatScrollbar, ChatScrollbarPart};
use super::*;
use bevy::input::mouse::MouseScrollUnit;
use bevy::input::mouse::MouseWheel;

#[test]
fn wheel_scrolls_over_log_or_scrollbar_and_growth_keeps_a_scrolled_back_log() {
    let mut model = GameplayUiModel::default();
    model.visible = true;
    model.chat.visible = true;
    model.chat.input_enabled = true;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<MouseWheel>()
        .insert_resource(model)
        .add_systems(Update, scroll_chat_history);
    let viewport = app
        .world_mut()
        .spawn((
            ChatLogViewport,
            Interaction::Hovered,
            ScrollPosition(Vec2::new(0.0, CHAT_SCROLL_TO_LATEST)),
            ComputedNode {
                size: Vec2::new(200.0, 100.0),
                content_size: Vec2::new(200.0, 500.0),
                inverse_scale_factor: 1.0,
                ..default()
            },
        ))
        .id();
    let thumb = app
        .world_mut()
        .spawn((ChatScrollbarPart::Thumb, Interaction::None))
        .id();
    let wheel = |app: &mut App| {
        app.world_mut().write_message(MouseWheel {
            phase: bevy::input::touch::TouchPhase::Moved,
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: 1.0,
            window: Entity::PLACEHOLDER,
        });
        app.update();
        app.world().get::<ScrollPosition>(viewport).unwrap().y
    };
    assert_eq!(wheel(&mut app), 400.0 - CHAT_SCROLL_WHEEL_LINE);
    *app.world_mut().get_mut::<Interaction>(viewport).unwrap() = Interaction::None;
    *app.world_mut().get_mut::<Interaction>(thumb).unwrap() = Interaction::Hovered;
    assert_eq!(wheel(&mut app), 400.0 - CHAT_SCROLL_WHEEL_LINE * 2.0);
    *app.world_mut().get_mut::<Interaction>(thumb).unwrap() = Interaction::None;
    assert_eq!(wheel(&mut app), 400.0 - CHAT_SCROLL_WHEEL_LINE * 2.0);

    // New history re-pins only a log that already shows its newest line.
    assert!(chat_log_shows_latest(CHAT_SCROLL_TO_LATEST, 400.0));
    assert!(chat_log_shows_latest(399.5, 400.0));
    assert!(chat_log_shows_latest(0.0, 0.0));
    assert!(!chat_log_shows_latest(
        400.0 - CHAT_SCROLL_WHEEL_LINE,
        400.0
    ));
}

fn fixture(scale: f32) -> (App, Entity, Entity, Entity, Vec<Entity>) {
    let mut app = App::new();
    let mut model = GameplayUiModel::default();
    model.visible = true;
    model.chat.visible = true;
    model.chat.input_enabled = true;
    app.add_plugins(MinimalPlugins)
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f32(0.1),
        ))
        .insert_resource(model)
        .init_resource::<ButtonInput<MouseButton>>()
        .add_systems(Update, handle_chat_scrollbar);
    let window = app
        .world_mut()
        .spawn((
            Window {
                focused: true,
                ..default()
            },
            PrimaryWindow,
        ))
        .id();
    let viewport = app
        .world_mut()
        .spawn((
            ChatLogViewport,
            ScrollPosition(Vec2::new(0.0, 200.0)),
            ComputedNode {
                size: Vec2::new(200.0, 100.0) * scale,
                content_size: Vec2::new(200.0, 500.0) * scale,
                inverse_scale_factor: 1.0 / scale,
                ..default()
            },
        ))
        .id();
    let bar = app
        .world_mut()
        .spawn((
            ChatScrollbar,
            Node::default(),
            bevy::ui::RelativeCursorPosition {
                cursor_over: true,
                normalized: Some(Vec2::ZERO),
            },
        ))
        .id();
    let parts = [
        ChatScrollbarPart::Up,
        ChatScrollbarPart::Down,
        ChatScrollbarPart::Track,
        ChatScrollbarPart::Thumb,
    ]
    .into_iter()
    .map(|part| app.world_mut().spawn((part, Interaction::None)).id())
    .collect();
    (app, viewport, bar, window, parts)
}

fn press(app: &mut App, part: Entity) {
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    *app.world_mut().get_mut::<Interaction>(part).unwrap() = Interaction::Pressed;
}

#[test]
fn minimap_new_zoom_out_covers_twice_the_default_world_width() {
    let normal = minimap_tiles(6320.32, 1871.77, 8.0);
    let wide = minimap_tiles(6320.32, 1871.77, 16.0);
    let normal_area: f32 = normal
        .iter()
        .map(|t| t.source.width * t.source.height)
        .sum();
    let wide_area: f32 = wide.iter().map(|t| t.source.width * t.source.height).sum();
    assert!((wide_area / normal_area - 4.0).abs() < 0.001);
    assert_eq!(wide, minimap_tiles(6320.32, 1871.77, 100.0));
}

#[test]
fn arrows_repeat_and_track_pages_with_clamped_offsets() {
    let (mut app, viewport, bar, _, parts) = fixture(1.0);
    press(&mut app, parts[0]);
    app.update();
    assert_eq!(
        app.world().get::<ScrollPosition>(viewport).unwrap().y,
        190.0
    );
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(
        app.world().get::<ScrollPosition>(viewport).unwrap().y,
        180.0
    );
    *app.world_mut().get_mut::<Interaction>(parts[0]).unwrap() = Interaction::None;
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .reset_all();
    app.update();
    press(&mut app, parts[2]);
    app.world_mut()
        .get_mut::<bevy::ui::RelativeCursorPosition>(bar)
        .unwrap()
        .normalized = Some(Vec2::new(0.0, 0.45));
    app.update();
    assert_eq!(
        app.world().get::<ScrollPosition>(viewport).unwrap().y,
        270.0
    );
}

#[test]
fn thumb_preserves_grab_offset_and_drag_cancels_on_release_or_focus_loss() {
    for scale in [1.0, 1.5, 2.0] {
        let (mut app, viewport, bar, window, parts) = fixture(scale);
        press(&mut app, parts[3]);
        app.update();
        assert_eq!(
            app.world().get::<ScrollPosition>(viewport).unwrap().y,
            200.0
        );
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        let (track, thumb) = chat_scrollbar_geometry(100.0, 400.0);
        app.world_mut()
            .get_mut::<bevy::ui::RelativeCursorPosition>(bar)
            .unwrap()
            .normalized = Some(Vec2::new(0.0, (track - thumb) * 0.25 / 100.0));
        app.update();
        assert!((app.world().get::<ScrollPosition>(viewport).unwrap().y - 300.0).abs() < 1e-4);
        app.world_mut()
            .get_mut::<bevy::ui::RelativeCursorPosition>(bar)
            .unwrap()
            .normalized = Some(Vec2::splat(2.0));
        app.update();
        assert_eq!(
            app.world().get::<ScrollPosition>(viewport).unwrap().y,
            400.0
        );
        app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
        app.update();
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        app.world_mut()
            .get_mut::<bevy::ui::RelativeCursorPosition>(bar)
            .unwrap()
            .normalized = Some(Vec2::splat(-2.0));
        app.update();
        assert_eq!(
            app.world().get::<ScrollPosition>(viewport).unwrap().y,
            400.0
        );
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .reset_all();
        app.update();
        assert_eq!(
            app.world().get::<ScrollPosition>(viewport).unwrap().y,
            400.0
        );
    }
}

#[test]
fn disabled_chat_and_channel_change_cancel_pointer_ownership() {
    let (mut app, viewport, _, _, parts) = fixture(1.0);
    app.world_mut()
        .resource_mut::<GameplayUiModel>()
        .chat
        .input_enabled = false;
    press(&mut app, parts[0]);
    app.update();
    assert_eq!(
        app.world().get::<ScrollPosition>(viewport).unwrap().y,
        200.0
    );
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .reset_all();
    app.world_mut()
        .resource_mut::<GameplayUiModel>()
        .chat
        .input_enabled = true;
    app.update();
    press(&mut app, parts[3]);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    app.world_mut()
        .resource_mut::<GameplayUiModel>()
        .chat
        .selected = ChatChannel::Group;
    app.update();
    assert_eq!(
        app.world().get::<ScrollPosition>(viewport).unwrap().y,
        200.0
    );
}
