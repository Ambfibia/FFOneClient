use bevy::asset::AssetPlugin;
use sha2::{Digest, Sha256};
use tempfile::tempdir;

use crate::system_message_ui::*;

#[test]
fn newly_opened_and_rebuilt_dialogs_are_centered_at_unchanged_resolution() {
    let mut app = App::new();
    app.init_resource::<SystemMessageUiModel>()
        .add_systems(Update, update_system_message_layout);
    app.world_mut().spawn((
        Window {
            resolution: bevy::window::WindowResolution::new(1264, 681),
            ..default()
        },
        PrimaryWindow,
    ));
    app.update(); // Populate the viewport cache while the dialog is closed.
    for index in [0, 1] {
        let entity = app
            .world_mut()
            .spawn((
                SystemMessageLayer(index),
                Node::default(),
                UiTransform::default(),
            ))
            .id();
        app.update();
        let expected = system_message_layer_layout(
            Vec2::new(1264., 681.),
            index,
            clean_system_message_ui_scale(681.),
        );
        let node = app.world().get::<Node>(entity).unwrap();
        assert_eq!(node.left, px(expected.left));
        assert_eq!(node.top, px(expected.top));
        let changed = app
            .world()
            .entity(entity)
            .get_ref::<Node>()
            .unwrap()
            .last_changed();
        app.update();
        assert_eq!(
            app.world()
                .entity(entity)
                .get_ref::<Node>()
                .unwrap()
                .last_changed(),
            changed
        );
    }
}

#[test]
fn selective_drain_preserves_unrelated_fifo_actions() {
    let action = |request_id| SystemMessageUiAction::Chosen {
        request_id,
        button_type: SystemMessageButtonType::Ok,
        choice: SystemMessageChoice::Primary,
    };
    let mut outbox = SystemMessageUiOutbox::default();
    for request_id in [10, 20, 11, 21] {
        outbox.push(action(request_id));
    }

    assert_eq!(
        outbox.drain_matching(|candidate| matches!(
            candidate,
            SystemMessageUiAction::Chosen {
                request_id: 20 | 21,
                ..
            }
        )),
        vec![action(20), action(21)]
    );
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![action(10), action(11)]
    );
}

#[test]
fn system_message_audio_routes_match_clean_special_case() {
    assert_eq!(
        system_message_audio_cue(
            SystemMessageButtonType::DeleteMission,
            SystemMessageChoice::Primary,
        ),
        SystemMessageUiAudioCue::AbandonMission
    );
    assert_eq!(
        system_message_audio_cue(
            SystemMessageButtonType::OkCancel,
            SystemMessageChoice::Primary,
        ),
        SystemMessageUiAudioCue::YesButton
    );
    assert_eq!(
        system_message_audio_cue(
            SystemMessageButtonType::DeleteMission,
            SystemMessageChoice::Secondary,
        ),
        SystemMessageUiAudioCue::NoButton
    );
}

#[test]
fn geometry_uses_effective_serialized_component_values() {
    assert_eq!(
        SYSTEM_MESSAGE_WINDOW_RECT,
        SystemMessageUiRect::new(0.0, 0.0, 600.0, 164.0)
    );
    assert_eq!(
        SYSTEM_MESSAGE_CONTENT_RECT,
        SystemMessageUiRect::new(140.0, 20.0, 420.0, 104.0)
    );
    assert_eq!(
        SYSTEM_MESSAGE_OK_RECT,
        SystemMessageUiRect::new(454.0, 124.0, 110.0, 25.0)
    );
    assert_eq!(
        SYSTEM_MESSAGE_CANCEL_RECT,
        SystemMessageUiRect::new(44.0, 124.0, 150.0, 25.0)
    );
    assert_eq!(
        SYSTEM_MESSAGE_EXIT_CHARACTER_CREATION_RECT,
        SystemMessageUiRect::new(320.0, 124.0, 245.0, 25.0)
    );
    assert_eq!(
        SYSTEM_MESSAGE_DIALOG_BORDER,
        BorderRect {
            min_inset: Vec2::new(40.0, 16.0),
            max_inset: Vec2::new(40.0, 40.0)
        }
    );
    assert_eq!(
        SYSTEM_MESSAGE_DIALOG_PATH,
        "ui/en/gameplay/system/systemDialogBox.png"
    );
    assert_eq!(SYSTEM_MESSAGE_FONT_PATH, "fonts/jeffe.otf");
    assert_eq!(SYSTEM_MESSAGE_JEFFE_14_PATH_ID, 903);
    assert_eq!(SYSTEM_MESSAGE_JEFFE_14_FONT_SIZE, 12.0);
    assert_eq!(SYSTEM_MESSAGE_JEFFE_14_LINE_HEIGHT, 13.710_000_04);
    assert_eq!(SYSTEM_MESSAGE_JEFFE_12_PATH_ID, 953);
    assert_eq!(SYSTEM_MESSAGE_JEFFE_12_FONT_SIZE, 10.0);
    assert_eq!(SYSTEM_MESSAGE_JEFFE_12_LINE_HEIGHT, 12.338_999_75);
    assert_eq!(SYSTEM_MESSAGE_CHALET_SMALL_PATH_ID, 1_018);
    assert_eq!(SYSTEM_MESSAGE_CHALET_SMALL_FONT_SIZE, 12.0);
    assert_eq!(SYSTEM_MESSAGE_CHALET_SMALL_LINE_HEIGHT, 12.071_999_55);
    assert_eq!(
        SYSTEM_MESSAGE_BODY_FONT_PATH,
        "fonts/chaletbook-regular.ttf"
    );
    assert_eq!(
        SYSTEM_MESSAGE_ICON_QUANTITY_RECT,
        SystemMessageUiRect::new(65.0, 32.0, 62.0, 62.0)
    );
    assert_eq!(
        SYSTEM_MESSAGE_COMPARISON_ICON_RECTS,
        [
            SystemMessageUiRect::new(371.0, 27.0, 62.0, 62.0),
            SystemMessageUiRect::new(444.0, 27.0, 62.0, 62.0),
        ]
    );
    assert_eq!(
        SYSTEM_MESSAGE_COMPARISON_BADGE_RECTS,
        [
            SystemMessageUiRect::new(405.0, 61.0, 26.0, 26.0),
            SystemMessageUiRect::new(478.0, 61.0, 26.0, 26.0),
        ]
    );

    let first = system_message_layer_layout(Vec2::new(1_280.0, 720.0), 0, 1.0);
    assert_eq!(first.left, 350.0);
    assert_eq!(first.top, 288.0);
    let second = system_message_layer_layout(Vec2::new(1_280.0, 720.0), 1, 1.0);
    assert_eq!(second.left, 360.0);
    assert_eq!(second.top, 298.0);
}

#[test]
fn reached_text_styles_match_the_serialized_skin_and_font_metrics() {
    let label = SystemMessageTextStyle::Label.spec();
    assert_eq!(label.source_style, "label");
    assert_eq!(label.source_font_path_id, 903);
    assert_eq!(label.font_role, SystemMessageFontRole::Jeffe);
    assert_eq!(label.font_size, 12.0);
    assert_eq!(label.line_height, 13.710_000_04);
    assert_eq!(label.margin, [4.0; 4]);
    assert_eq!(label.padding, [0.0, 0.0, 3.0, 3.0]);
    assert_eq!(label.justify, Justify::Left);
    assert_eq!(label.linebreak, LineBreak::WordBoundary);
    assert!(!label.stretch_width);
    assert_eq!(label.y_offset, 0.0);

    let body = SystemMessageTextStyle::ImageWindow.spec();
    assert_eq!(body.source_style, "Imagewindow");
    assert_eq!(body.source_font_path_id, 1_018);
    assert_eq!(body.font_role, SystemMessageFontRole::Chalet);
    assert_eq!(body.font_size, 12.0);
    assert_eq!(body.line_height, 12.071_999_55);
    assert_eq!(body.margin, [0.0; 4]);
    assert_eq!(body.padding, [0.0; 4]);
    assert_eq!(body.linebreak, LineBreak::WordBoundary);
    assert!(body.stretch_width);

    let button = SystemMessageTextStyle::Button.spec();
    assert_eq!(button.source_style, "button");
    assert_eq!(button.source_font_path_id, 903);
    assert_eq!(button.padding, [0.0; 4]);
    assert_eq!(button.justify, Justify::Center);
    assert_eq!(button.linebreak, LineBreak::NoWrap);

    let cancel = SystemMessageTextStyle::CancelButton.spec();
    assert_eq!(cancel.source_style, "CancelButton");
    assert_eq!(cancel.source_font_path_id, 953);
    assert_eq!(cancel.font_size, 10.0);
    assert_eq!(cancel.line_height, 12.338_999_75);
    assert_eq!(cancel.padding, [10.0, 6.0, 4.0, 6.0]);
    assert_eq!(cancel.linebreak, LineBreak::WordBoundary);

    let destructive = SystemMessageTextStyle::RedButton.spec();
    assert_eq!(destructive.source_style, "RedButton");
    assert_eq!(destructive.source_font_path_id, 903);
    assert_eq!(destructive.padding, [10.0, 6.0, 4.0, 6.0]);
    assert_eq!(destructive.y_offset, 0.0);
}

#[test]
fn clean_button_switch_routes_every_renderable_type() {
    use SystemMessageButtonType as Type;
    use SystemMessageButtonVisual as Visual;

    let cases = [
        (
            Type::None,
            "OKAY",
            None,
            Visual::Standard,
            SYSTEM_MESSAGE_OK_RECT,
        ),
        (
            Type::Ok,
            "OKAY",
            None,
            Visual::Standard,
            SYSTEM_MESSAGE_OK_RECT,
        ),
        (
            Type::OkCancel,
            "OKAY",
            Some("CANCEL"),
            Visual::Standard,
            SYSTEM_MESSAGE_OK_RECT,
        ),
        (
            Type::YesNo,
            "YES",
            Some("NO"),
            Visual::Standard,
            SYSTEM_MESSAGE_OK_RECT,
        ),
        (
            Type::ExitCharacterCreation,
            "EXIT CHARACTER CREATION",
            Some("CANCEL"),
            Visual::Destructive,
            SYSTEM_MESSAGE_EXIT_CHARACTER_CREATION_RECT,
        ),
        (
            Type::DeleteMission,
            "DELETE MISSION",
            Some("CANCEL"),
            Visual::Destructive,
            SYSTEM_MESSAGE_EXIT_CHARACTER_CREATION_RECT,
        ),
        (
            Type::TutorialSkip,
            "SKIP",
            Some("CONTINUE"),
            Visual::Standard,
            SYSTEM_MESSAGE_OK_RECT,
        ),
        (
            Type::LeaveGroup,
            "LEAVE",
            Some("CANCEL"),
            Visual::Destructive,
            SYSTEM_MESSAGE_OK_RECT,
        ),
        (
            Type::DeleteItem,
            "DELETE",
            Some("CANCEL"),
            Visual::Destructive,
            SYSTEM_MESSAGE_OK_RECT,
        ),
        (
            Type::CancelWarp,
            "WARP",
            Some("CANCEL"),
            Visual::Standard,
            SYSTEM_MESSAGE_OK_RECT,
        ),
        (
            Type::CombinationFailure,
            "OKAY",
            None,
            Visual::Standard,
            SYSTEM_MESSAGE_OK_RECT,
        ),
        (
            Type::CombinationConfirm,
            "CONTINUE",
            Some("CANCEL"),
            Visual::Standard,
            SYSTEM_MESSAGE_OK_RECT,
        ),
    ];

    for (button_type, primary, secondary, visual, rect) in cases {
        let layout = system_message_button_layout(button_type);
        assert_eq!(layout.primary.label, primary);
        assert_eq!(layout.primary.visual, visual);
        assert_eq!(layout.primary.rect, rect);
        assert_eq!(layout.secondary.map(|button| button.label), secondary);
        assert_eq!(layout.count(), usize::from(secondary.is_some()) + 1);
        for spec in std::iter::once(layout.primary).chain(layout.secondary) {
            let localized = system_message_button_localized(spec);
            assert_ne!(localized.key, "ui.content.passthrough");
            assert_eq!(localized.fallback, spec.label);
            assert!(localized.args.is_empty());
            match spec.visual {
                Visual::Cancel => {
                    assert_eq!(
                        system_message_button_font_size(spec.visual),
                        SYSTEM_MESSAGE_JEFFE_12_FONT_SIZE
                    );
                    assert_eq!(
                        system_message_button_line_height(spec.visual),
                        SYSTEM_MESSAGE_JEFFE_12_LINE_HEIGHT
                    );
                }
                Visual::Standard | Visual::Destructive => {
                    assert_eq!(
                        system_message_button_font_size(spec.visual),
                        SYSTEM_MESSAGE_JEFFE_14_FONT_SIZE
                    );
                    assert_eq!(
                        system_message_button_line_height(spec.visual),
                        SYSTEM_MESSAGE_JEFFE_14_LINE_HEIGHT
                    );
                }
            }
        }
    }

    assert_eq!(
        system_message_button_layout(Type::OkCancel)
            .secondary
            .unwrap()
            .visual,
        Visual::Cancel
    );
    assert_eq!(
        system_message_button_layout(Type::DeleteMission)
            .secondary
            .unwrap()
            .visual,
        Visual::Standard
    );
}

#[test]
fn message_body_is_key_first_for_both_semantic_and_legacy_copy() {
    let passthrough = SystemMessageRequest::new(1, "server copy", SystemMessageButtonType::Ok);
    assert_eq!(passthrough.localized.key, "ui.content.passthrough");
    assert_eq!(
        passthrough.localized.args.get("text").map(String::as_str),
        Some("server copy")
    );
    assert_eq!(
        passthrough.icon_path.as_deref(),
        Some(SYSTEM_MESSAGE_WARNING_ICON_PATH)
    );

    let semantic = SystemMessageRequest::new_localized(
        2,
        LocalizedText::new("ui.loading.error", "LOADING STOPPED: {error}")
            .with_arg("error", "Scene"),
        SystemMessageButtonType::Ok,
    );
    assert_eq!(semantic.text, "LOADING STOPPED: Scene");
    assert_eq!(semantic.localized.key, "ui.loading.error");
    assert_eq!(
        semantic.localized.args.get("error"),
        Some(&"Scene".to_owned())
    );
}

#[test]
fn serialized_icon_slots_preserve_null_entries_and_published_bytes() {
    for (raw, expected) in [
        (0, Some(SYSTEM_MESSAGE_WARNING_ICON_PATH)),
        (1, Some(SYSTEM_MESSAGE_TRADE_ICON_PATH)),
        (2, Some(SYSTEM_MESSAGE_BUDDY_ICON_PATH)),
        (3, Some(SYSTEM_MESSAGE_GROUP_ICON_PATH)),
        (4, None),
        (5, None),
        (6, None),
        (7, Some(SYSTEM_MESSAGE_COMBI_ICON_PATH)),
    ] {
        assert_eq!(
            SystemMessageIconIndex::try_from(raw)
                .unwrap()
                .runtime_path(),
            expected
        );
        assert_eq!(
            SystemMessageRequest::new(raw as u64, "icon", SystemMessageButtonType::Ok)
                .try_with_legacy_icon_index(raw)
                .unwrap()
                .icon_path
                .as_deref(),
            expected
        );
    }
    assert_eq!(
        SystemMessageIconIndex::try_from(8),
        Err(SystemMessageIconIndexError(8))
    );

    let game_assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets")
        .join("game");
    for contract in SYSTEM_MESSAGE_ICON_ASSET_CONTRACTS {
        assert_eq!(contract.index.runtime_path(), Some(contract.runtime_path));
        let bytes = std::fs::read(game_assets.join(contract.runtime_path)).unwrap();
        assert_eq!(
            format!("{:X}", Sha256::digest(bytes)),
            contract.png_sha256,
            "{} must remain byte-exact to primary path ID {}",
            contract.runtime_path,
            contract.source_path_id
        );
    }
}

#[test]
fn raw_types_normalize_failures_and_reject_unrendered_or_invalid_values() {
    assert_eq!(
        SystemMessageButtonType::try_from(10),
        Ok(SystemMessageButtonType::Ok)
    );
    assert_eq!(
        SystemMessageButtonType::try_from(11),
        Ok(SystemMessageButtonType::OkCancel)
    );
    assert_eq!(
        SystemMessageButtonType::try_from(4),
        Err(SystemMessageButtonTypeError::Unrendered(4))
    );
    assert_eq!(
        SystemMessageButtonType::try_from(5),
        Err(SystemMessageButtonTypeError::Unrendered(5))
    );
    assert_eq!(
        SystemMessageButtonType::try_from(-1),
        Err(SystemMessageButtonTypeError::Invalid(-1))
    );
    assert_eq!(
        SystemMessageButtonType::try_from(16),
        Err(SystemMessageButtonTypeError::Invalid(16))
    );
}

#[test]
fn model_is_lifo_and_secondary_choice_fails_closed_for_one_button() {
    let mut model = SystemMessageUiModel::default();
    model.push(SystemMessageRequest::new(
        1,
        "older",
        SystemMessageButtonType::Ok,
    ));
    model.push(SystemMessageRequest::new(
        2,
        "newer",
        SystemMessageButtonType::YesNo,
    ));
    assert_eq!(model.current().unwrap().request_id, 2);
    assert!(model.is_popup());

    assert_eq!(
        model.choose(SystemMessageChoice::Secondary),
        Some(SystemMessageUiAction::Chosen {
            request_id: 2,
            button_type: SystemMessageButtonType::YesNo,
            choice: SystemMessageChoice::Secondary,
        })
    );
    assert_eq!(model.current().unwrap().request_id, 1);
    assert_eq!(model.choose(SystemMessageChoice::Secondary), None);
    assert_eq!(model.len(), 1);
    assert!(model.choose(SystemMessageChoice::Primary).is_some());
    assert!(!model.is_popup());

    model.set_focus_out(true);
    assert!(model.is_popup());
    model.push(SystemMessageRequest::new(
        3,
        "frozen",
        SystemMessageButtonType::Ok,
    ));
    assert_eq!(model.choose(SystemMessageChoice::Primary), None);
    assert_eq!(model.len(), 1);
}

#[test]
fn correlated_remove_preserves_newer_and_unrelated_modal_requests() {
    let mut model = SystemMessageUiModel::default();
    for request_id in [10, 20, 30] {
        model.push(SystemMessageRequest::new(
            request_id,
            request_id.to_string(),
            SystemMessageButtonType::Ok,
        ));
    }
    assert!(model.remove(20));
    assert_eq!(
        model
            .stack()
            .iter()
            .map(|request| request.request_id)
            .collect::<Vec<_>>(),
        vec![10, 30]
    );
    assert_eq!(model.current().unwrap().request_id, 30);
    assert!(!model.remove(20));
}

fn plugin_app() -> (App, tempfile::TempDir) {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(SystemMessageUiPlugin);
    (app, asset_root)
}

#[test]
fn plugin_spawns_modal_stack_without_own_camera_and_emits_choice() {
    let (mut app, _asset_root) = plugin_app();
    app.update();
    {
        let world = app.world_mut();
        let mut roots = world.query_filtered::<&Node, With<SystemMessageUiRoot>>();
        assert_eq!(roots.single(world).unwrap().display, Display::None);
        let mut cameras = world.query::<&Camera>();
        assert_eq!(cameras.iter(world).count(), 0);
    }

    {
        let mut model = app.world_mut().resource_mut::<SystemMessageUiModel>();
        model.push(SystemMessageRequest::new(
            41,
            "first",
            SystemMessageButtonType::Ok,
        ));
        model.push(SystemMessageRequest::new(
            42,
            "second",
            SystemMessageButtonType::OkCancel,
        ));
    }
    app.update();

    let secondary = {
        let world = app.world_mut();
        let mut roots = world.query_filtered::<&Node, With<SystemMessageUiRoot>>();
        assert_eq!(roots.single(world).unwrap().display, Display::Flex);
        let mut layers = world.query::<&SystemMessageLayer>();
        assert_eq!(layers.iter(world).count(), 2);
        let mut bodies = world.query_filtered::<&LocalizedText, With<SystemMessageBody>>();
        let body = bodies.single(world).unwrap();
        assert_eq!(body.key, "ui.content.passthrough");
        assert_eq!(body.fallback, "{text}");
        assert_eq!(body.args.get("text").map(String::as_str), Some("second"));
        let mut buttons = world.query::<(Entity, &SystemMessageButton)>();
        let values = buttons.iter(world).collect::<Vec<_>>();
        assert_eq!(values.len(), 2);
        values
            .into_iter()
            .find_map(|(entity, button)| {
                (button.choice == SystemMessageChoice::Secondary).then_some(entity)
            })
            .unwrap()
    };
    app.world_mut()
        .entity_mut(secondary)
        .insert(Interaction::Pressed);
    app.update();

    assert_eq!(
        app.world_mut()
            .resource_mut::<SystemMessageUiOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![SystemMessageUiAction::Chosen {
            request_id: 42,
            button_type: SystemMessageButtonType::OkCancel,
            choice: SystemMessageChoice::Secondary,
        }]
    );
    assert_eq!(
        app.world_mut()
            .resource_mut::<SystemMessageUiAudioOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![SystemMessageUiAudioCue::NoButton]
    );
    assert_eq!(
        app.world()
            .resource::<SystemMessageUiModel>()
            .current()
            .unwrap()
            .request_id,
        41
    );
}

#[test]
fn long_body_scrolls_inside_fixed_frame_with_buttons_available() {
    let (mut app, _asset_root) = plugin_app();
    app.update();
    app.world_mut()
        .resource_mut::<SystemMessageUiModel>()
        .push(SystemMessageRequest::new(
            63,
            "Long English text and длинный русский текст ".repeat(30),
            SystemMessageButtonType::OkCancel,
        ));
    app.update();

    let world = app.world_mut();
    let mut viewport = world.query_filtered::<(&Node, &ScrollPosition), With<super::view::SystemMessageBodyViewport>>();
    let (node, _) = viewport.single(world).unwrap();
    assert_eq!(node.height, px(SYSTEM_MESSAGE_CONTENT_RECT.height));
    assert_eq!(node.overflow, Overflow::scroll_y());

    let mut body = world.query_filtered::<&Node, With<SystemMessageBody>>();
    assert_eq!(body.single(world).unwrap().height, Val::Auto);
    let mut buttons = world.query::<(&SystemMessageButton, &Node)>();
    assert_eq!(buttons.iter(world).count(), 2);
    for (_, node) in buttons.iter(world) {
        assert_eq!(node.top, px(SYSTEM_MESSAGE_OK_RECT.y));
    }
}

#[test]
fn long_body_wheel_scroll_stays_within_content_bounds() {
    use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};

    let mut app = App::new();
    app.init_resource::<SystemMessageUiModel>()
        .insert_resource(AccumulatedMouseScroll {
            unit: MouseScrollUnit::Line,
            delta: Vec2::new(0.0, -2.0),
        })
        .add_systems(Update, super::view::scroll_system_message_body);
    app.world_mut()
        .resource_mut::<SystemMessageUiModel>()
        .push(SystemMessageRequest::new(64, "long", SystemMessageButtonType::Ok));
    let viewport = app.world_mut().spawn((
        super::view::SystemMessageBodyViewport,
        ComputedNode {
            size: Vec2::new(420.0, 100.0),
            content_size: Vec2::new(420.0, 200.0),
            inverse_scale_factor: 1.0,
            ..default()
        },
        Interaction::Hovered,
        ScrollPosition::default(),
    )).id();

    app.update();
    assert_eq!(app.world().get::<ScrollPosition>(viewport).unwrap().y, 60.0);
    app.world_mut().resource_mut::<AccumulatedMouseScroll>().delta.y = -10.0;
    app.update();
    assert_eq!(app.world().get::<ScrollPosition>(viewport).unwrap().y, 100.0);
    *app.world_mut().get_mut::<Interaction>(viewport).unwrap() = Interaction::None;
    app.world_mut().resource_mut::<AccumulatedMouseScroll>().delta.y = 10.0;
    app.update();
    assert_eq!(app.world().get::<ScrollPosition>(viewport).unwrap().y, 100.0);
}

#[test]
fn every_spawned_text_is_key_first_and_owns_a_reached_source_style() {
    let (mut app, _asset_root) = plugin_app();
    app.update();
    app.world_mut().resource_mut::<SystemMessageUiModel>().push(
        SystemMessageRequest::new(
            61,
            "MISSION:\nDelete the mission?\nAll progress is lost.\nThis cannot be undone.",
            SystemMessageButtonType::DeleteMission,
        )
        .with_icon_path("icons/items/general/generalitemicon_01.png"),
    );
    app.update();

    let world = app.world_mut();
    let mut texts = world.query::<(
        &Text,
        Option<&LocalizedText>,
        Option<&SystemMessageTextStyle>,
    )>();
    let values = texts.iter(world).collect::<Vec<_>>();
    assert_eq!(values.len(), 6, "4 body + 2 buttons");
    for (_text, localized, style) in values {
        assert!(localized.is_some_and(|value| !value.key.is_empty()));
        assert!(style.is_some());
    }

    let mut lines = world.query::<(
        &SystemMessageBodyLine,
        &LocalizedText,
        &SystemMessageTextStyle,
        (&TextFont, &LineHeight),
        &TextLayout,
    )>();
    let mut lines = lines.iter(world).collect::<Vec<_>>();
    lines.sort_by_key(|(line, _, _, _, _)| line.index);
    assert_eq!(lines.len(), 4);
    assert_eq!(*lines[0].2, SystemMessageTextStyle::Label);
    assert_eq!(*lines[1].2, SystemMessageTextStyle::Label);
    assert_eq!(*lines[2].2, SystemMessageTextStyle::ImageWindow);
    assert_eq!(*lines[3].2, SystemMessageTextStyle::ImageWindow);
    for (line, localized, style, font, layout) in lines {
        let spec = style.spec();
        assert_eq!(localized.key, "ui.content.passthrough");
        assert_eq!(localized.fallback, "{text}");
        assert!(localized.args.contains_key("text"), "line {}", line.index);
        assert_eq!(font.0.font_size.eval(Vec2::ZERO, 16.0), spec.font_size);
        assert_eq!((*font.1), LineHeight::Px(spec.line_height));
        assert_eq!(layout.justify, spec.justify);
        assert_eq!(layout.linebreak, spec.linebreak);
    }
}

#[test]
fn localized_multiline_copy_rebuilds_clean_layout_for_ru_and_en() {
    let game_assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets")
        .join("game");
    let (localization, language) = Localization::open(&game_assets, "ru").unwrap();
    let localized = LocalizedText::new(
        "ui.combi.modal.combination_failed",
        "OOPS!\nThe combination failed. But you can always try again.",
    );
    let expected_ru = localization.text(&language, &localized);

    let (mut app, _asset_root) = plugin_app();
    app.insert_resource(localization);
    app.insert_resource(language);
    app.update();
    app.world_mut().resource_mut::<SystemMessageUiModel>().push(
        SystemMessageRequest::new_localized(
            62,
            localized.clone(),
            SystemMessageButtonType::CombinationFailure,
        ),
    );
    app.update();

    let body_args = |world: &mut World| {
        let mut query = world.query::<(&SystemMessageBodyLine, &LocalizedText)>();
        let mut values = query
            .iter(world)
            .map(|(line, localized)| {
                (
                    line.index,
                    localized.args.get("text").cloned().unwrap_or_default(),
                )
            })
            .collect::<Vec<_>>();
        values.sort_by_key(|(index, _)| *index);
        values.into_iter().map(|(_, text)| text).collect::<Vec<_>>()
    };
    assert_eq!(
        body_args(app.world_mut()),
        expected_ru
            .split('\n')
            .map(str::to_owned)
            .collect::<Vec<_>>()
    );

    let localization = app.world().resource::<Localization>().clone();
    {
        let mut language = app.world_mut().resource_mut::<Language>();
        localization.select(&mut language, "en");
    }
    let expected_en = localization.text(app.world().resource::<Language>(), &localized);
    app.update();
    assert_eq!(
        body_args(app.world_mut()),
        expected_en
            .split('\n')
            .map(str::to_owned)
            .collect::<Vec<_>>()
    );
}

#[test]
fn delete_quantity_and_comparison_icons_follow_clean_layering() {
    let (mut app, _asset_root) = plugin_app();
    app.update();
    app.world_mut().resource_mut::<SystemMessageUiModel>().push(
        SystemMessageRequest::new(70, "DELETE THIS ITEM?", SystemMessageButtonType::DeleteItem)
            .with_icon_path("icons/items/general/generalitemicon_01.png")
            .with_icon_quantity(3)
            .with_comparison_icons(
                "icons/items/equipment/cosicon_1.png",
                true,
                "icons/items/equipment/cosicon_2.png",
                false,
            ),
    );
    app.update();

    let world = app.world_mut();
    let mut primary = world.query::<&SystemMessagePrimaryIcon>();
    assert_eq!(primary.iter(world).count(), 1);
    let mut frames = world.query::<&SystemMessageIconFrame>();
    assert_eq!(frames.iter(world).count(), 3);
    let mut comparisons = world.query::<&SystemMessageComparisonIcon>();
    assert_eq!(
        comparisons
            .iter(world)
            .map(|marker| marker.0)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    let mut badges = world.query::<&SystemMessageCombinedBadge>();
    assert_eq!(badges.single(world).unwrap().0, 0);
    let mut quantities = world.query_filtered::<
        (&LocalizedText, &SystemMessageTextStyle),
        With<SystemMessageIconQuantity>,
    >();
    let (quantity, style) = quantities.single(world).unwrap();
    assert_eq!(quantity.key, "ui.content.passthrough");
    assert_eq!(quantity.fallback, "{text}");
    assert_eq!(quantity.args.get("text").map(String::as_str), Some("3"));
    assert_eq!(*style, SystemMessageTextStyle::Label);
}

#[test]
fn cursor_unlocks_for_stack_and_restores_after_last_choice() {
    let (mut app, _asset_root) = plugin_app();
    app.world_mut().spawn((
        PrimaryWindow,
        CursorOptions {
            visible: false,
            grab_mode: CursorGrabMode::Locked,
            ..default()
        },
    ));
    app.update();
    app.world_mut()
        .resource_mut::<SystemMessageUiModel>()
        .push(SystemMessageRequest::new(
            5,
            "cursor",
            SystemMessageButtonType::Ok,
        ));
    app.update();
    {
        let world = app.world_mut();
        let mut cursors = world.query_filtered::<&CursorOptions, With<PrimaryWindow>>();
        let cursor = cursors.single(world).unwrap();
        assert!(cursor.visible);
        assert_eq!(cursor.grab_mode, CursorGrabMode::None);
    }

    app.world_mut()
        .resource_mut::<SystemMessageUiModel>()
        .choose(SystemMessageChoice::Primary);
    app.update();
    let world = app.world_mut();
    let mut cursors = world.query_filtered::<&CursorOptions, With<PrimaryWindow>>();
    let cursor = cursors.single(world).unwrap();
    assert!(!cursor.visible);
    assert_eq!(cursor.grab_mode, CursorGrabMode::Locked);
}
