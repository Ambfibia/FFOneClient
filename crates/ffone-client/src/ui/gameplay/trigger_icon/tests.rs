use super::*;
use bevy::ecs::system::RunSystemOnce;

fn fixture() -> (App, Entity, Entity, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .insert_resource(GameplayUiModel::retrobution_reference_frame())
        .add_systems(Update, bind);
    app.world_mut().spawn((
        Window {
            resolution: bevy::window::WindowResolution::new(1264, 681),
            ..default()
        },
        bevy::window::PrimaryWindow,
    ));
    app.world_mut()
        .run_system_once(|mut commands: Commands, assets: Res<AssetServer>| {
            commands
                .spawn(Node::default())
                .with_children(|parent| spawn(parent, &assets));
        })
        .unwrap();
    let icon = app
        .world_mut()
        .query_filtered::<Entity, With<TriggerUseIcon>>()
        .single(app.world())
        .unwrap();
    let trigger = app.world_mut().spawn_empty().id();
    let mut state = LegacyAvatarActionState::default();
    state.target_selection.trigger = Some(trigger);
    let player = app
        .world_mut()
        .spawn((state, LegacyAvatarActionContext::default()))
        .id();
    app.world_mut().spawn(LegacyOrbitCamera::new(player));
    (app, player, trigger, icon)
}

#[test]
fn trigger_use_icon_follows_selection_and_traversal_without_dirtying_idle_layout() {
    let (mut app, player, trigger, icon) = fixture();
    app.update();
    let node = app.world().get::<Node>(icon).unwrap();
    assert_eq!(node.display, Display::Flex);
    assert_eq!(
        (node.left, node.top, node.width, node.height),
        (px(614), px(314), px(37), px(52))
    );
    assert_eq!(app.world().get::<ZIndex>(icon).unwrap().0, 900);
    assert!(matches!(
        app.world().get::<ImageNode>(icon).unwrap().image_mode,
        NodeImageMode::Stretch
    ));
    app.update();
    let changed = app
        .world_mut()
        .run_system_once(|nodes: Query<Ref<Node>, With<TriggerUseIcon>>| {
            nodes.single().unwrap().last_changed()
        })
        .unwrap();
    app.update();
    let unchanged = app
        .world_mut()
        .run_system_once(|nodes: Query<Ref<Node>, With<TriggerUseIcon>>| {
            nodes.single().unwrap().last_changed()
        })
        .unwrap();
    assert_eq!(changed, unchanged);

    app.world_mut()
        .entity_mut(player)
        .insert(WorldZiplineTraversal {
            start: Vec3::ZERO,
            end: Vec3::X * 10.0,
            speed: 5.0,
            travelled: 0.0,
            packet_elapsed: 0.0,
            hang_height: 2.2,
        });
    app.update();
    assert_eq!(
        app.world().get::<Node>(icon).unwrap().display,
        Display::None
    );
    app.world_mut()
        .entity_mut(player)
        .remove::<WorldZiplineTraversal>();
    app.world_mut().resource_mut::<GameplayUiModel>().visible = false;
    app.update();
    assert_eq!(
        app.world().get::<Node>(icon).unwrap().display,
        Display::None
    );
    app.world_mut().resource_mut::<GameplayUiModel>().visible = true;

    // A traversal retains the previous target selection; it must still hide the cue.
    app.world_mut()
        .get_mut::<LegacyAvatarActionContext>(player)
        .unwrap()
        .move_mode = LegacyMoveMode::Other;
    app.update();
    assert_eq!(
        app.world().get::<Node>(icon).unwrap().display,
        Display::None
    );
    app.world_mut()
        .get_mut::<LegacyAvatarActionContext>(player)
        .unwrap()
        .move_mode = LegacyMoveMode::None;
    app.world_mut()
        .get_mut::<LegacyAvatarActionState>(player)
        .unwrap()
        .target_selection
        .trigger = None;
    app.update();
    assert_eq!(
        app.world().get::<Node>(icon).unwrap().display,
        Display::None
    );

    // Changing weapon targeting mode must not suppress interaction icons.
    app.world_mut()
        .get_mut::<LegacyAvatarActionContext>(player)
        .unwrap()
        .weapon_target_mode = crate::avatar_action::LegacyWeaponTargetMode::Rocket;
    app.world_mut()
        .get_mut::<LegacyAvatarActionState>(player)
        .unwrap()
        .target_selection
        .trigger = Some(trigger);
    app.world_mut().resource_mut::<GameplayUiModel>().ui_scale = 1.5;
    app.update();
    assert_eq!(
        app.world().get::<Node>(icon).unwrap().display,
        Display::Flex
    );
    assert_eq!(
        app.world().get::<UiTransform>(icon).unwrap().scale,
        Vec2::splat(1.5)
    );
    app.world_mut().despawn(trigger);
    app.update();
    assert_eq!(
        app.world().get::<Node>(icon).unwrap().display,
        Display::None
    );
}

#[test]
fn trigger_use_icon_hides_while_launcher_owns_the_player() {
    let (mut app, _, _, icon) = fixture();
    app.insert_resource(LauncherUiModel::default());
    app.update();
    assert_eq!(
        app.world().get::<Node>(icon).unwrap().display,
        Display::Flex
    );
    app.world_mut().resource_mut::<LauncherUiModel>().phase =
        crate::launcher_ui::LauncherUiPhase::Aiming;
    app.update();
    assert_eq!(
        app.world().get::<Node>(icon).unwrap().display,
        Display::None
    );
    app.world_mut().resource_mut::<LauncherUiModel>().phase =
        crate::launcher_ui::LauncherUiPhase::Hidden;
    app.update();
    assert_eq!(
        app.world().get::<Node>(icon).unwrap().display,
        Display::Flex
    );
}

#[test]
fn trigger_use_icon_texture_matches_native_contract() {
    use sha2::Digest;
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/game")
            .join(TEXTURE_PATH),
    )
    .unwrap();
    assert_eq!(bytes.len(), 1979);
    assert_eq!(
        format!("{:x}", sha2::Sha256::digest(bytes)),
        "03e0e32a492161f0f59137c7d755b56440eaafcec1abf50d02a2326ea21fb60b"
    );
}
