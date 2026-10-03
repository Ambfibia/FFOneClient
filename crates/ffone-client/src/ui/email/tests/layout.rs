use super::*;

#[test]
fn player_tab_stays_below_list_rim_and_guide_in_normal_active_and_hover_states() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(EmailUiPlugin);
    app.update();
    let assets = app.world().resource::<EmailUiAssets>().clone();
    let world = app.world_mut();
    let mut buttons = world.query::<(Entity, &EmailUiButton, &ChildOf)>();
    let (player, parent) = buttons
        .iter(world)
        .find(|(_, button, _)| button.kind == EmailUiButtonKind::PlayerTab)
        .map(|(entity, _, parent)| (entity, parent.parent()))
        .unwrap();
    let guide = buttons
        .iter(world)
        .find(|(_, button, _)| button.kind == EmailUiButtonKind::GuideTab)
        .map(|(entity, _, _)| entity)
        .unwrap();
    let children = world.get::<Children>(parent).unwrap();
    let order = |entity| children.iter().position(|child| child == entity).unwrap();
    let rim = children
        .iter()
        .find(|child| {
            world
                .get::<ImageNode>(*child)
                .is_some_and(|image| image.image == assets.list)
        })
        .unwrap();
    assert!(order(player) < order(rim) && order(rim) < order(guide));
    let original_children = children.to_vec();
    for (folder, interaction, expected) in [
        (
            EmailFolder::Guide,
            Interaction::None,
            assets.player_tab.clone(),
        ),
        (
            EmailFolder::Guide,
            Interaction::Hovered,
            assets.player_tab_hover.clone(),
        ),
        (
            EmailFolder::Player,
            Interaction::None,
            assets.player_tab_hover.clone(),
        ),
        (
            EmailFolder::Player,
            Interaction::Hovered,
            assets.player_tab_hover.clone(),
        ),
    ] {
        let world = app.world_mut();
        let mut model = world.resource_mut::<EmailUiModel>();
        model.visible = true;
        model.opening_elapsed_seconds = EMAIL_UI_OPEN_SECONDS;
        model.folder = folder;
        *world.get_mut::<Interaction>(player).unwrap() = interaction;
        app.update();
        assert_eq!(
            app.world().get::<ImageNode>(player).unwrap().image,
            expected
        );
        assert_eq!(
            app.world().get::<Children>(parent).unwrap().to_vec(),
            original_children
        );
    }
}

#[test]
fn clean_authority_ids_components_and_geometry_are_frozen() {
    assert_eq!(EMAIL_UI_GAME_MODE_VALUE, 18);
    assert_eq!(EMAIL_UI_GAME_OBJECT_PATH_ID, 1_275);
    assert_eq!(
        [
            EMAIL_UI_LIST_COMPONENT_PATH_ID,
            EMAIL_UI_COMPOSE_COMPONENT_PATH_ID,
            EMAIL_UI_MODE_COMPONENT_PATH_ID,
            EMAIL_UI_PC_STUFF_COMPONENT_PATH_ID,
        ],
        [1_553, 1_554, 1_555, 1_556]
    );
    assert_eq!(
        EMAIL_UI_LIST_WINDOW_RECT,
        EmailUiRect::new(0.0, 0.0, 567.0, 634.0)
    );
    assert_eq!(
        EMAIL_UI_COMPOSE_INNER_RECT,
        EmailUiRect::new(6.0, 3.0, 565.0, 600.0)
    );
    assert_eq!(
        EMAIL_UI_LIST_BACK_RECT,
        EmailUiRect::new(0.0, 15.0, 561.0, 229.0)
    );
    assert_eq!(
        EMAIL_UI_DATA_BACK_RECT,
        EmailUiRect::new(0.0, 248.0, 560.0, 382.0)
    );
    assert_eq!(
        EMAIL_UI_BUDDY_LIST_VIEWPORT_RECT,
        EmailUiRect::new(20.0, 54.0, 310.0, 178.0)
    );
    assert_eq!(EMAIL_UI_BUDDY_LIST_CONTENT_WIDTH, 270.0);
    assert_eq!(EMAIL_UI_BUDDY_ROW_WIDTH, 280.0);
    assert_eq!(EMAIL_UI_BUDDY_ROW_HEIGHT, 30.0);
    assert_eq!(
        EMAIL_UI_DETAIL_ITEM_RECT,
        EmailUiRect::new(40.0, 230.0, 67.0, 67.0)
    );
    assert_eq!(
        EMAIL_UI_COMPOSE_ITEM_RECT,
        EmailUiRect::new(30.0, 392.0, 68.0, 64.0)
    );
    assert_eq!(EMAIL_UI_POPUP_Z_INDEX, EMAIL_UI_Z_INDEX + 1);
}

#[test]
fn production_buddy_rows_preserve_clean_geometry_font_copy_and_selection() {
    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(EmailUiPlugin);
    app.update();
    {
        let mut model = app.world_mut().resource_mut::<EmailUiModel>();
        model.visible = true;
        model.screen = EmailScreen::Compose;
        model.popup = EmailPopup::BuddyList;
        model.opening_elapsed_seconds = EMAIL_UI_OPEN_SECONDS;
        model.buddies = vec![
            EmailBuddy {
                pc_uid: 77,
                first_name: "Dexter".to_owned(),
                last_name: "McPherson".to_owned(),
                name_check_flag: 1,
            },
            EmailBuddy {
                pc_uid: 88,
                name_check_flag: 0,
                ..default()
            },
        ];
    }
    app.update();

    let assets = app.world().resource::<EmailUiAssets>().clone();
    let mut rows = {
        let world = app.world_mut();
        let mut query = world.query::<(Entity, &EmailUiBuddyRow, &Node, &Children)>();
        query
            .iter(world)
            .map(|(entity, row, node, children)| {
                let text_entity = children[0];
                (
                    entity,
                    *row,
                    node.clone(),
                    world
                        .get::<LocalizedText>(text_entity)
                        .expect("buddy row Text must be key-first")
                        .clone(),
                    world
                        .get::<TextFont>(text_entity)
                        .expect("buddy row Text must carry its source font style")
                        .clone(),
                    world
                        .get::<TextLayout>(text_entity)
                        .expect("buddy row Text must carry its source alignment")
                        .clone(),
                    *world
                        .get::<EmailUiTextStyle>(text_entity)
                        .expect("buddy row Text must identify rightLabel"),
                )
            })
            .collect::<Vec<_>>()
    };
    rows.sort_by_key(|row| row.1.row);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].1.row, 0);
    assert_eq!(rows[0].2.left, px(0.0));
    assert_eq!(rows[0].2.top, px(0.0));
    assert_eq!(rows[0].2.width, px(280.0));
    assert_eq!(rows[0].2.height, px(30.0));
    assert_eq!(rows[0].3.key, "ui.content.passthrough");
    assert_eq!(rows[0].3.args["text"], "Dexter McPherson");
    assert_eq!(
        rows[0].4.font,
        bevy::text::FontSource::Handle(assets.body_font.clone())
    );
    assert_eq!(rows[0].4.font_size, bevy::text::FontSize::Px(12.0));
    assert_eq!(rows[0].5.justify, Justify::Right);
    assert_eq!(rows[0].5.linebreak, LineBreak::WordBoundary);
    assert_eq!(rows[0].6, EmailUiTextStyle::RightLabel);
    assert_eq!(rows[1].2.top, px(30.0));
    assert_eq!(rows[1].3.key, "ui.email.player_label");
    assert_eq!(rows[1].3.args["pc_uid"], "88");

    app.world_mut()
        .entity_mut(rows[0].0)
        .insert(Interaction::Pressed);
    app.update();
    let model = app.world().resource::<EmailUiModel>();
    assert_eq!(model.draft.recipient_pc_uid, 77);
    assert_eq!(model.draft.recipient_name, "DexterMcPherson");
    assert_eq!(model.popup, EmailPopup::None);
}

#[test]
fn clean_layout_matches_1264_by_681_integer_centering_and_compose_slide() {
    let closed = email_ui_layout_with_opening(Vec2::new(1_264.0, 681.0), 1.0, 0.0, 0.0);
    assert_eq!(closed.list_window.left, 122.0);
    assert_eq!(closed.list_window.top, 21.0);
    assert_eq!(closed.left_backplate.left, 114.0);
    assert_eq!(closed.left_backplate.top, 14.0);
    assert_eq!(closed.right_window.left, EMAIL_UI_RIGHT_OFFSCREEN_X);
    assert_eq!(closed.compose_window.left, -460.0);
    let open = email_ui_layout_with_opening(Vec2::new(1_264.0, 681.0), 1.0, 1.0, 1.0);
    assert_eq!(open.compose_window.left, 122.0);
    assert_eq!(open.right_window.left, 707.0);
    assert_eq!(open.scale, Vec2::ONE);
}
