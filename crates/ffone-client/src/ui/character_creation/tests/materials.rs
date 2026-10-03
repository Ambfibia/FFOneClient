use super::*;

#[test]
fn preview_overlapping_controls_render_above_the_player_camera() {
    assert!(
        CHARACTER_CREATION_FOREGROUND_CAMERA_ORDER
            > crate::player_preview::NATIVE_PLAYER_PREVIEW_CAMERA_ORDER
    );

    let asset_root = project_asset("");
    let mut app = App::new();
    insert_test_localization(&mut app, &asset_root);
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .init_asset::<AudioSource>()
        .add_message::<KeyboardInput>()
        .add_plugins(NativeCharacterCreationUiPlugin);
    app.update();

    let world = app.world_mut();
    let mut camera_query =
        world.query_filtered::<Entity, With<CharacterCreationForegroundCamera>>();
    let foreground_camera = camera_query.single(world).unwrap();
    let mut root_query = world.query_filtered::<
        (&UiTargetCamera, &FocusPolicy, &Pickable, &Children),
        With<CreationPreviewControlsRoot>,
    >();
    let (target, focus_policy, pickable, children) = root_query.single(world).unwrap();
    assert_eq!(target.0, foreground_camera);
    assert_eq!(*focus_policy, FocusPolicy::Pass);
    assert_eq!(*pickable, Pickable::IGNORE);
    assert_eq!(children.len(), 5);
    assert!(children.iter().all(|child| {
        world
            .get::<CharacterCreationButton>(child)
            .is_some_and(|button| {
                matches!(
                    button.0,
                    CharacterCreationControl::Camera(_)
                        | CharacterCreationControl::RandomAppearance
                )
            })
    }));

    let continue_label = {
        let mut query = world.query::<(Entity, &CharacterCreationButton, &Children)>();
        query.iter(world).find_map(|(_, button, children)| {
            (button.0 == CharacterCreationControl::ContinueAppearance).then_some(children[0])
        })
    }
    .expect("appearance continue label");
    assert_eq!(
        world.get::<FocusPolicy>(continue_label),
        Some(&FocusPolicy::Pass),
        "button text must not capture clicks intended for Continue"
    );
    assert_eq!(
        world.get::<Pickable>(continue_label),
        Some(&Pickable::IGNORE)
    );
}
