use super::active_nano::ActiveNanoInfoRoot;
use super::chat_model::chat_layout;
use super::chat_spawn::ChatLogViewport;
use super::chat_spawn::ChatRoot;
use super::chat_spawn::ChatScrollbar;
use super::chat_spawn::ChatScrollbarPart;
use super::minimap::MinimapCameraOverlay;
use super::minimap::MinimapRoot;
use super::minimap::MinimapTileImage;
use super::nano_wheel::NanoWheelRoot;
use super::player_status::PlayerHealthFill;
use super::player_status::PlayerStatusRoot;
use super::*;

#[derive(Resource, Default)]
struct LayoutChanges(Vec<usize>);

fn record_changes(nodes: Query<(), Changed<Node>>, mut changes: ResMut<LayoutChanges>) {
    changes.0.push(nodes.iter().count());
}

#[test]
fn hud_layout_preserves_idle_ticks_and_reacts_to_scale_resize_and_external_edits() {
    let mut app = App::new();
    app.init_resource::<GameplayUiModel>()
        .init_resource::<GameplayMenuTransition>()
        .init_resource::<LayoutChanges>()
        .add_systems(
            Update,
            (
                update_gameplay_hud_layout,
                bind_chat_scrollbar,
                record_changes,
            )
                .chain(),
        );
    let status = app
        .world_mut()
        .spawn((Node::default(), UiTransform::default(), PlayerStatusRoot))
        .id();
    app.world_mut()
        .spawn((Node::default(), UiTransform::default(), ActiveNanoInfoRoot));
    app.world_mut()
        .spawn((Node::default(), UiTransform::default(), MinimapRoot));
    let chat = app
        .world_mut()
        .spawn((Node::default(), UiTransform::default(), ChatRoot))
        .id();
    app.world_mut()
        .spawn((Node::default(), UiTransform::default(), NanoWheelRoot));
    app.world_mut().spawn((Node::default(), ChatLogViewport));
    let scrollbar = app.world_mut().spawn((Node::default(), ChatScrollbar)).id();
    for part in [
        ChatScrollbarPart::Track,
        ChatScrollbarPart::Up,
        ChatScrollbarPart::Down,
        ChatScrollbarPart::Thumb,
    ] {
        app.world_mut().spawn((Node::default(), part));
    }
    app.update();
    app.update();
    assert_eq!(app.world().resource::<LayoutChanges>().0.last(), Some(&0));
    app.world_mut().resource_mut::<GameplayUiModel>().ui_scale = 1.5;
    app.update();
    assert_eq!(
        app.world().get::<UiTransform>(status).unwrap().scale,
        Vec2::splat(1.5)
    );
    assert!(
        app.world()
            .resource::<LayoutChanges>()
            .0
            .last()
            .copied()
            .unwrap()
            > 0
    );
    app.update();
    assert_eq!(app.world().resource::<LayoutChanges>().0.last(), Some(&0));
    let previous_bottom = app.world().get::<Node>(chat).unwrap().bottom;
    let previous_height = app.world().get::<Node>(scrollbar).unwrap().height;
    app.world_mut()
        .resource_mut::<GameplayUiModel>()
        .chat
        .window_size += Vec2::new(40.0, 40.0);
    app.update();
    assert_ne!(
        app.world().get::<Node>(chat).unwrap().bottom,
        previous_bottom
    );
    assert_ne!(
        app.world().get::<Node>(scrollbar).unwrap().height,
        previous_height
    );
    let expected_left = app.world().get::<Node>(status).unwrap().left;
    app.world_mut().get_mut::<Node>(status).unwrap().left = px(-999.0);
    app.update();
    assert_eq!(app.world().get::<Node>(status).unwrap().left, expected_left);
    app.update();
    assert_eq!(app.world().resource::<LayoutChanges>().0.last(), Some(&0));
}

fn hud_binding_fixture() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()))
        .init_asset::<Image>()
        .init_asset::<Font>()
        .init_asset::<CircularMinimapTileMaterial>()
        .init_asset::<FusionMatterMeterMaterial>()
        .init_resource::<GameplayUiModel>()
        .init_resource::<GameplayMenuTransition>()
        .init_resource::<LayoutChanges>()
        .add_systems(Startup, spawn_gameplay_hud)
        .add_systems(
            Update,
            (bind_hud_player_minimap, bind_chat, record_changes).chain(),
        );
    app.world_mut()
        .resource_mut::<GameplayUiModel>()
        .minimap
        .tiles = vec![MinimapTileSample {
        tile_number: 1,
        source: GameplayUiRect {
            x: 20.0,
            y: 30.0,
            width: 100.0,
            height: 110.0,
        },
        destination: GameplayUiRect {
            x: 0.0,
            y: 0.0,
            width: 148.0,
            height: 148.0,
        },
    }];
    app.update();
    app
}

#[test]
fn animated_hud_does_not_dirty_unchanged_layout_or_minimap_materials() {
    let mut app = hud_binding_fixture();
    // The shared model changes each frame during camera orbit, even when the
    // player/map tile/chat geometry is stationary.
    for heading in [10.0, 20.0, 30.0] {
        app.world_mut()
            .resource_mut::<GameplayUiModel>()
            .minimap
            .camera_heading_degrees = heading;
        app.update();
        assert_eq!(app.world().resource::<LayoutChanges>().0.last(), Some(&0));
        let rotation = app
            .world_mut()
            .query_filtered::<&UiTransform, With<MinimapCameraOverlay>>()
            .single(app.world())
            .unwrap()
            .rotation;
        assert_eq!(rotation, Rot2::radians(heading.to_radians()));
    }
    let mut tile_events = app
        .world_mut()
        .resource_mut::<Messages<AssetEvent<CircularMinimapTileMaterial>>>();
    tile_events.clear();
    app.world_mut()
        .resource_mut::<Messages<AssetEvent<FusionMatterMeterMaterial>>>()
        .clear();
    for _ in 0..3 {
        app.world_mut()
            .resource_mut::<GameplayUiModel>()
            .set_changed();
        app.update();
    }
    let events = app
        .world()
        .resource::<Messages<AssetEvent<CircularMinimapTileMaterial>>>();
    assert!(
        !events
            .iter_current_update_messages()
            .any(|event| matches!(event, AssetEvent::Modified { .. }))
    );
    let events = app
        .world()
        .resource::<Messages<AssetEvent<FusionMatterMeterMaterial>>>();
    assert!(
        !events
            .iter_current_update_messages()
            .any(|event| matches!(event, AssetEvent::Modified { .. }))
    );
}

#[test]
fn guarded_hud_layout_updates_geometry_and_repairs_external_edits() {
    let mut app = hud_binding_fixture();
    let health = app
        .world_mut()
        .query_filtered::<Entity, With<PlayerHealthFill>>()
        .single(app.world())
        .unwrap();
    app.world_mut().get_mut::<Node>(health).unwrap().width = px(-999.0);
    app.world_mut()
        .resource_mut::<GameplayUiModel>()
        .set_changed();
    app.update();
    let model = app.world().resource::<GameplayUiModel>();
    assert_eq!(
        app.world().get::<Node>(health).unwrap().width,
        px(PLAYER_HEALTH_RECT.width * model.player.health_fraction())
    );
    app.world_mut()
        .resource_mut::<GameplayUiModel>()
        .chat
        .window_size += Vec2::new(25.0, 35.0);
    app.update();
    let expected = chat_layout(
        app.world().resource::<GameplayUiModel>().chat.window_size,
        false,
    );
    let node = app
        .world_mut()
        .query_filtered::<&Node, With<ChatRoot>>()
        .single(app.world())
        .unwrap();
    assert_eq!(node.width, px(expected.size.x));
    assert_eq!(node.height, px(expected.size.y));
    app.world_mut()
        .resource_mut::<GameplayUiModel>()
        .set_changed();
    app.update();
    assert_eq!(app.world().resource::<LayoutChanges>().0.last(), Some(&0));
}

#[test]
fn minimap_material_guards_preserve_changed_uv_texture_and_meter_values() {
    let mut app = hud_binding_fixture();
    {
        let mut model = app.world_mut().resource_mut::<GameplayUiModel>();
        model.minimap.tiles[0].tile_number = 2;
        model.minimap.tiles[0].source.x = 64.0;
        model.minimap.tiles[0].destination.width = 74.0;
        model.minimap.fusion_matter = 3;
        model.minimap.max_fusion_matter = 4;
    }
    app.update();
    let tile = app
        .world_mut()
        .query::<(
            &MinimapTileImage,
            &MaterialNode<CircularMinimapTileMaterial>,
        )>()
        .iter(app.world())
        .find(|(marker, _)| marker.0 == 0)
        .unwrap()
        .1
        .0
        .clone();
    let material = app
        .world()
        .resource::<Assets<CircularMinimapTileMaterial>>()
        .get(&tile)
        .unwrap();
    assert_eq!(material.source_uv.x, 64.0 / 512.0);
    assert_eq!(material.destination_uv.z, 74.0 / MINIMAP_MAP_RECT.width);
    assert_eq!(
        material.color_texture,
        app.world().resource::<GameplayUiAssets>().minimap_tiles[1]
    );
    let alpha_texture = material.alpha_texture.clone();
    let meter = app
        .world_mut()
        .query::<&MaterialNode<FusionMatterMeterMaterial>>()
        .single(app.world())
        .unwrap()
        .0
        .clone();
    assert_eq!(
        app.world()
            .resource::<Assets<FusionMatterMeterMaterial>>()
            .get(&meter)
            .unwrap()
            .parameters
            .x,
        0.75
    );
    // Mutating an existing asset still gets repaired by the regular binder.
    app.world_mut()
        .resource_mut::<Assets<CircularMinimapTileMaterial>>()
        .get_mut(&tile)
        .unwrap()
        .source_uv
        .x = -1.0;
    app.world_mut()
        .resource_mut::<GameplayUiModel>()
        .set_changed();
    app.update();
    let material = app
        .world()
        .resource::<Assets<CircularMinimapTileMaterial>>()
        .get(&tile)
        .unwrap();
    assert_eq!(material.source_uv.x, 64.0 / 512.0);
    assert_eq!(material.alpha_texture, alpha_texture);
}
