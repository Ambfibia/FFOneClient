use super::*;

#[test]
fn inventory_and_outgoing_slot_controls_attach_first_free_then_detach() {
    let (mut model, _, transport, audio) = opened_player_model();
    model.inventory[9] = Some(EmailInventorySlotView {
        item: EmailWireItem {
            item_type: 7,
            item_id: 123,
            option: 4,
            time_limit: 0,
        },
        icon_path: Some("icons/items/general/generalitemicon_00.png".to_owned()),
        count_label: Some("4".to_owned()),
    });
    let mut opening_audio = EmailUiAudioOutbox::default();
    assert!(begin_email_compose(None, &mut model, &mut opening_audio));
    model.tick_opening(EMAIL_UI_OPEN_SECONDS);

    let mut app = App::new();
    app.insert_resource(model)
        .insert_resource(transport)
        .insert_resource(audio)
        .insert_resource(ButtonInput::<MouseButton>::default())
        .init_resource::<EmailUiItemDragState>()
        .add_systems(Update, handle_email_item_interactions);
    let inventory_source = app
        .world_mut()
        .spawn((Interaction::Pressed, EmailUiInventorySlot { slot: 9 }))
        .id();
    let attachment_drop = app
        .world_mut()
        .spawn((
            Interaction::None,
            EmailUiAttachmentSlot {
                slot: 2,
                outgoing: true,
            },
        ))
        .id();
    let attachment_source = app
        .world_mut()
        .spawn((
            Interaction::None,
            EmailUiAttachmentSlot {
                slot: 0,
                outgoing: true,
            },
        ))
        .id();
    let inventory_drop = app
        .world_mut()
        .spawn((Interaction::None, EmailUiInventorySlot { slot: 0 }))
        .id();

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    app.world_mut()
        .entity_mut(inventory_source)
        .insert(Interaction::None);
    app.world_mut()
        .entity_mut(attachment_drop)
        .insert(Interaction::Hovered);
    {
        let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
        mouse.clear_just_pressed(MouseButton::Left);
        mouse.release(MouseButton::Left);
    }
    app.update();
    {
        let model = app.world().resource::<EmailUiModel>();
        let attached = model.draft.attachments[0].expect("first free attachment slot");
        assert_eq!(attached.inventory_slot, 9);
        assert!(email_inventory_slot_staged(model, 9));
        assert!(model.draft.attachments[2].is_none());
    }

    app.world_mut()
        .entity_mut(attachment_drop)
        .insert(Interaction::None);
    app.world_mut()
        .entity_mut(attachment_source)
        .insert(Interaction::Pressed);
    {
        let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
        mouse.clear_just_released(MouseButton::Left);
        mouse.press(MouseButton::Left);
    }
    app.update();
    app.world_mut()
        .entity_mut(attachment_source)
        .insert(Interaction::None);
    app.world_mut()
        .entity_mut(inventory_drop)
        .insert(Interaction::Hovered);
    {
        let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
        mouse.clear_just_pressed(MouseButton::Left);
        mouse.release(MouseButton::Left);
    }
    app.update();
    let model = app.world().resource::<EmailUiModel>();
    assert!(model.draft.attachments[0].is_none());
    assert!(!email_inventory_slot_staged(model, 9));
    assert_eq!(app.world().resource::<EmailUiAudioOutbox>().0.len(), 2);
}

#[test]
fn incoming_item_control_accepts_one_based_slot_into_first_free_inventory() {
    let (model, _, transport, audio) = opened_player_model();
    let mut app = App::new();
    app.insert_resource(model)
        .insert_resource(transport)
        .insert_resource(audio)
        .insert_resource(ButtonInput::<MouseButton>::default())
        .init_resource::<EmailUiItemDragState>()
        .add_systems(Update, handle_email_item_interactions);
    app.world_mut().spawn((
        Interaction::Hovered,
        EmailUiAttachmentSlot {
            slot: 0,
            outgoing: false,
        },
    ));
    {
        let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
        mouse.press(MouseButton::Right);
        mouse.clear_just_pressed(MouseButton::Right);
        mouse.release(MouseButton::Right);
    }
    app.update();
    assert_eq!(
        app.world_mut().resource_mut::<EmailTransportOutbox>().pop(),
        Some(EmailRequest::ReceiveItem {
            email_index: 7,
            inventory_slot: 0,
            email_item_slot: 1,
        })
    );
    assert!(app.world().resource::<EmailUiModel>().send_in_flight);
}

#[test]
fn all_runtime_assets_use_native_paths() {
    assert!(
        EMAIL_UI_IMAGE_PATHS
            .iter()
            .all(|path| path.starts_with("ui/"))
    );
}

#[test]
fn primary_inventory_skin_text_styles_keep_exact_replacement_metrics_and_padding() {
    assert_eq!(EMAIL_UI_JEFFE_12_SOURCE_FONT_PATH_ID, 977);
    assert_eq!(EMAIL_UI_JEFFE_14_SOURCE_FONT_PATH_ID, 933);
    assert_eq!(EMAIL_UI_JEFFE_16_SOURCE_FONT_PATH_ID, 1_008);
    assert_eq!(EMAIL_UI_JEFFE_06_SOURCE_FONT_PATH_ID, 970);
    assert_eq!(EMAIL_UI_CHALET_SMALL_SOURCE_FONT_PATH_ID, 949);

    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>();
    let assets = EmailUiAssets::load(app.world().resource::<AssetServer>());
    for (style, expected_font, expected_size, expected_line_height) in [
        (
            EmailUiTextStyle::LabelUpperLeft,
            assets.font.clone(),
            EMAIL_UI_JEFFE_12_FONT_SIZE,
            EMAIL_UI_JEFFE_12_LINE_HEIGHT,
        ),
        (
            EmailUiTextStyle::Button,
            assets.font.clone(),
            EMAIL_UI_JEFFE_14_FONT_SIZE,
            EMAIL_UI_JEFFE_14_LINE_HEIGHT,
        ),
        (
            EmailUiTextStyle::CalculatorButton,
            assets.font.clone(),
            EMAIL_UI_JEFFE_16_FONT_SIZE,
            EMAIL_UI_JEFFE_16_LINE_HEIGHT,
        ),
        (
            EmailUiTextStyle::PostageLabel,
            assets.font.clone(),
            EMAIL_UI_JEFFE_06_FONT_SIZE,
            EMAIL_UI_JEFFE_06_LINE_HEIGHT,
        ),
        (
            EmailUiTextStyle::RightLabel,
            assets.body_font.clone(),
            EMAIL_UI_CHALET_SMALL_FONT_SIZE,
            EMAIL_UI_CHALET_SMALL_LINE_HEIGHT,
        ),
    ] {
        let font = style.font(&assets);
        assert_eq!(
            font.0.font,
            bevy::text::FontSource::Handle(expected_font.clone()),
            "wrong font path for {style:?}"
        );
        assert_eq!(
            font.0.font_size.eval(Vec2::ZERO, 16.0),
            expected_size,
            "wrong size for {style:?}"
        );
        assert_eq!(
            font.1,
            LineHeight::Px(expected_line_height),
            "wrong line height for {style:?}"
        );
    }

    let mut label = Node::default();
    EmailUiTextStyle::LabelMiddleCenter.apply_to_container(&mut label);
    assert_eq!(label.justify_content, JustifyContent::Center);
    assert_eq!(label.align_items, AlignItems::Center);
    assert_eq!(label.padding.left, px(0));
    assert_eq!(label.padding.right, px(0));
    assert_eq!(label.padding.top, px(EMAIL_UI_LABEL_PADDING_TOP));
    assert_eq!(label.padding.bottom, px(EMAIL_UI_LABEL_PADDING_BOTTOM));

    let mut button = Node::default();
    EmailUiTextStyle::Button.apply_to_container(&mut button);
    assert_eq!(button.justify_content, JustifyContent::Center);
    assert_eq!(button.align_items, AlignItems::Center);
    assert_eq!(button.padding.left, px(EMAIL_UI_BUTTON_PADDING_LEFT));
    assert_eq!(button.padding.right, px(EMAIL_UI_BUTTON_PADDING_RIGHT));
    assert_eq!(button.padding.top, px(EMAIL_UI_BUTTON_PADDING_TOP));
    assert_eq!(button.padding.bottom, px(EMAIL_UI_BUTTON_PADDING_BOTTOM));

    assert_eq!(
        EmailUiTextStyle::LabelUpperLeft.source_style_name(),
        "label"
    );
    assert_eq!(
        EmailUiTextStyle::RightLabel.source_style_name(),
        "rightLabel"
    );
    assert_eq!(
        EmailUiTextStyle::CalculatorButton.source_style_name(),
        "cacubut*"
    );
    assert_eq!(
        EmailUiTextStyle::PostageLabel.source_style_name(),
        "vendorlistback"
    );
    assert_eq!(
        EmailUiTextStyle::PostageLabel.y_offset(),
        EMAIL_UI_POSTAGE_LABEL_Y_OFFSET
    );
}
