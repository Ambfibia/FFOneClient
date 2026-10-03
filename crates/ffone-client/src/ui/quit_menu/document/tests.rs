use super::*;
#[test]
fn loaded_native_document_binds_geometry_and_real_ecs_actions() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/game").into(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(QuitMenuUiPlugin);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        app.update();
        let loaded = app
            .world()
            .get_resource::<DocumentHandle>()
            .is_some_and(|h| {
                app.world()
                    .resource::<Assets<DocumentAsset>>()
                    .contains(&h.0)
            });
        if loaded {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "native document did not load"
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    app.update();
    let handle = app.world().resource::<DocumentHandle>().0.clone();
    {
        let mut documents = app.world_mut().resource_mut::<Assets<DocumentAsset>>();
        let mut asset = documents.get_mut(&handle).unwrap();
        let document = &mut asset.0;
        document.buttons[0].rect[0] = 17.0;
        document.styles[0].content_offset = [1.0, 2.0];
        document.styles[0].font_compensation = [0.0, -3.0];
        document.styles[0].active = Some(document.styles[0].hover.clone());
        document.styles[1].active = Some(document.styles[1].normal.clone());
    }
    app.update();
    let mut labels = app
        .world_mut()
        .query::<(&QuitMenuButtonLabel, &UiTransform)>();
    for (label, transform) in labels.iter(app.world()) {
        if label.kind == QuitMenuButtonKind::ChangeCharacter {
            assert_eq!(transform.translation, Val2::px(1.0, -1.0));
        }
    }
    let assets = app.world().resource::<QuitMenuUiAssets>();
    assert_eq!(
        assets.button_image(QuitMenuButtonVisual::Standard, Interaction::Pressed),
        assets.button_hover
    );
    assert_eq!(
        assets.button_image(QuitMenuButtonVisual::Cancel, Interaction::Pressed),
        assets.cancel_normal
    );
    app.update();
    let mut labels = app
        .world_mut()
        .query::<(&QuitMenuButtonLabel, &UiTransform)>();
    for (label, transform) in labels.iter(app.world()) {
        if label.kind == QuitMenuButtonKind::ChangeCharacter {
            assert_eq!(transform.translation, Val2::px(1.0, -1.0));
        }
    }
    let mut backgrounds = app.world_mut().query::<(&QuitMenuButton, &ImageNode)>();
    for (_, image) in backgrounds.iter(app.world()) {
        assert_eq!(image.visual_box, bevy::ui::VisualBox::BorderBox);
    }
    let mut query = app.world_mut().query::<(&QuitMenuButton, &Node)>();
    assert_eq!(
        query
            .iter(app.world())
            .find(|(b, _)| b.kind == QuitMenuButtonKind::ChangeCharacter)
            .unwrap()
            .1
            .left,
        px(17.0)
    );
    for (kind, expected) in [
        (
            QuitMenuButtonKind::ChangeCharacter,
            QuitMenuUiAction::ChangeCharacter,
        ),
        (QuitMenuButtonKind::QuitGame, QuitMenuUiAction::QuitGame),
        (
            QuitMenuButtonKind::Cancel,
            QuitMenuUiAction::Cancel {
                source: QuitMenuDismissalSource::CancelButton,
            },
        ),
    ] {
        let mut q = app.world_mut().query::<&mut Interaction>();
        for mut i in q.iter_mut(app.world_mut()) {
            *i = Interaction::None;
        }
        app.world_mut().resource_mut::<QuitMenuUiModel>().open();
        app.update();
        let mut q = app
            .world_mut()
            .query::<(&QuitMenuButton, &mut Interaction)>();
        for (b, mut i) in q.iter_mut(app.world_mut()) {
            if b.kind == kind {
                *i = Interaction::Pressed;
            }
        }
        app.update();
        assert_eq!(
            app.world_mut()
                .resource_mut::<QuitMenuUiOutbox>()
                .pop_front(),
            Some(expected)
        );
        assert!(!app.world().resource::<QuitMenuUiModel>().visible);
    }
    app.world_mut().resource_mut::<QuitMenuUiModel>().open();
    app.world_mut()
        .resource_mut::<QuitMenuUiModel>()
        .set_enabled(false);
    let mut q = app.world_mut().query::<&mut Interaction>();
    for mut i in q.iter_mut(app.world_mut()) {
        *i = Interaction::Pressed;
    }
    app.update();
    assert!(app.world().resource::<QuitMenuUiOutbox>().is_empty());
}
