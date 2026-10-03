use super::*;

pub(super) fn summary(index: i64, subject: &str) -> EmailSummary {
    EmailSummary {
        email_index: index,
        from_pc_uid: 10_000 + index,
        first_name: format!("Sender{index}"),
        subject: subject.to_owned(),
        send_time: EmailSystemTime {
            year: 2026,
            month: 7,
            day: index as i32,
            ..default()
        },
        ..default()
    }
}

#[test]
fn guide_detail_labels_sender_icon_and_shifts_from_subject_like_panel_email_list() {
    use bevy::ecs::system::RunSystemOnce;

    fn icon_display(app: &mut App, layer: EmailUiGuideIcon) -> Display {
        let mut query = app.world_mut().query::<(&EmailUiGuideIcon, &Node)>();
        query
            .iter(app.world())
            .find(|(candidate, _)| **candidate == layer)
            .map(|(_, node)| node.display)
            .unwrap()
    }
    fn label_lefts(app: &mut App) -> Vec<(f32, Val)> {
        let mut query = app.world_mut().query::<(&EmailUiGuideIconShift, &Node)>();
        let mut lefts = query
            .iter(app.world())
            .map(|(shift, node)| (shift.left, node.left))
            .collect::<Vec<_>>();
        lefts.sort_by(|left, right| left.0.total_cmp(&right.0));
        lefts
    }

    assert_eq!(
        EMAIL_UI_DETAIL_GUIDE_ICON_IMAGE_RECT,
        EmailUiRect::new(
            EMAIL_UI_DETAIL_ICON_RECT.left,
            EMAIL_UI_DETAIL_ICON_RECT.top + EMAIL_UI_LABEL_PADDING_TOP,
            62.0,
            EMAIL_UI_DETAIL_ICON_RECT.height
                - EMAIL_UI_LABEL_PADDING_TOP
                - EMAIL_UI_LABEL_PADDING_BOTTOM,
        )
    );
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_systems(Update, sync_email_guide_sender_icon);
    let mut model = EmailUiModel::default();
    model.guide_messages = vec![EmailGuideMessage {
        mode: 2,
        sender_name: "Computress".to_owned(),
        sender_icon_path: Some("icons/entities/hnpc/hnpcicon_153.png".to_owned()),
        ..default()
    }];
    model.selected_row = Some(0);
    app.insert_resource(model);
    app.world_mut()
        .run_system_once(|mut commands: Commands, asset_server: Res<AssetServer>| {
            let assets = EmailUiAssets::load(&asset_server);
            commands
                .spawn(Node::default())
                .with_children(|parent| spawn_email_detail(parent, &assets));
        })
        .unwrap();
    app.update();

    assert_eq!(
        icon_display(&mut app, EmailUiGuideIcon::SlotBackdrop),
        Display::Flex
    );
    assert_eq!(
        icon_display(&mut app, EmailUiGuideIcon::Sender),
        Display::Flex
    );
    let mut query = app
        .world_mut()
        .query::<(&EmailUiGuideIcon, &Node, &ImageNode)>();
    for (layer, node, image) in query.iter(app.world()) {
        assert_eq!(
            (node.left, node.top, node.width, node.height),
            (px(10.0), px(13.0), px(62.0), px(62.0))
        );
        if *layer == EmailUiGuideIcon::Sender {
            assert_eq!(
                image.image.path().map(ToString::to_string).as_deref(),
                Some("icons/entities/hnpc/hnpcicon_153.png")
            );
        }
    }
    assert_eq!(
        label_lefts(&mut app),
        vec![
            (10.0, px(89.0)),
            (10.0, px(89.0)),
            (70.0, px(149.0)),
            (95.0, px(174.0))
        ]
    );

    app.world_mut().resource_mut::<EmailUiModel>().folder = EmailFolder::Player;
    app.update();
    assert_eq!(
        icon_display(&mut app, EmailUiGuideIcon::SlotBackdrop),
        Display::None
    );
    assert_eq!(
        icon_display(&mut app, EmailUiGuideIcon::Sender),
        Display::None
    );
    assert_eq!(
        label_lefts(&mut app),
        vec![
            (10.0, px(10.0)),
            (10.0, px(10.0)),
            (70.0, px(70.0)),
            (95.0, px(95.0))
        ]
    );

    // A selected guide row without a resolved icon keeps only the backdrop.
    {
        let mut model = app.world_mut().resource_mut::<EmailUiModel>();
        model.folder = EmailFolder::Guide;
        model.guide_messages[0].sender_icon_path = None;
    }
    app.update();
    assert_eq!(
        icon_display(&mut app, EmailUiGuideIcon::SlotBackdrop),
        Display::Flex
    );
    assert_eq!(
        icon_display(&mut app, EmailUiGuideIcon::Sender),
        Display::None
    );
}

#[test]
fn player_tab_requests_page_one_and_page_success_immediately_reads_first_mail() {
    let mut model = EmailUiModel::default();
    let mut actions = EmailUiOutbox::default();
    let mut transport = EmailTransportOutbox::default();
    let mut audio = EmailUiAudioOutbox::default();
    open_email_ui(&mut model, &mut actions, vec![], 0);
    assert!(switch_email_folder(
        EmailFolder::Player,
        &mut model,
        &mut transport,
        &mut audio
    ));
    assert_eq!(transport.pop(), Some(EmailRequest::PageList { page: 1 }));
    apply_email_reply(
        EmailReply::PageListSuccess {
            page: 1,
            messages: vec![summary(11, "First"), summary(0, "empty")],
        },
        &mut model,
        &mut actions,
        &mut transport,
        &mut audio,
    );
    assert_eq!(model.player_messages.len(), 1);
    assert_eq!(model.selected_row, Some(0));
    assert!(model.send_in_flight);
    assert_eq!(
        transport.pop(),
        Some(EmailRequest::Read { email_index: 11 })
    );
}

#[test]
fn player_next_requires_a_full_five_mail_page_and_moves_only_one_page() {
    let (mut model, _, mut transport, mut audio) = opened_player_model();
    assert!(!model.player_can_next());
    assert!(!change_email_page(
        1,
        &mut model,
        &mut transport,
        &mut audio
    ));
    model.player_messages = (1..=EMAIL_PAGE_SIZE)
        .map(|index| summary(index as i64, "Mail"))
        .collect();
    assert!(model.player_can_next());
    assert!(!change_email_page(
        2,
        &mut model,
        &mut transport,
        &mut audio
    ));
    assert!(change_email_page(1, &mut model, &mut transport, &mut audio));
    assert_eq!(model.player_page, 2);
    assert_eq!(transport.pop(), Some(EmailRequest::PageList { page: 2 }));
}

#[test]
fn empty_guide_folder_and_legacy_relative_dates_keep_clean_labels() {
    let model = EmailUiModel::default();
    assert_eq!(email_text(&model, EmailUiTextRole::PageNumber), "0");
    assert_eq!(
        email_text(&model, EmailUiTextRole::FolderEmpty),
        "There are no messages in this folder."
    );
    let now = EmailSystemTime {
        year: 2026,
        month: 7,
        day: 31,
        ..default()
    };
    assert_eq!(now.legacy_day_label(now), "today");
    assert_eq!(
        EmailSystemTime {
            year: 2026,
            month: 7,
            day: 30,
            ..default()
        }
        .legacy_day_label(now),
        "1 day"
    );
    assert_eq!(
        EmailSystemTime {
            year: 2026,
            month: 6,
            day: 31,
            ..default()
        }
        .legacy_day_label(now),
        "31 day"
    );
}

#[test]
fn delete_success_refreshes_the_same_player_page() {
    let (mut model, mut actions, mut transport, mut audio) = opened_player_model();
    model.player_page = 3;
    model.send_in_flight = true;
    apply_email_reply(
        EmailReply::DeleteSuccess {
            email_indices: [7, 0, 0, 0, 0],
        },
        &mut model,
        &mut actions,
        &mut transport,
        &mut audio,
    );
    assert!(model.send_in_flight);
    assert_eq!(transport.pop(), Some(EmailRequest::PageList { page: 3 }));
}

#[test]
fn item_is_not_removed_until_matching_success_arrives() {
    let (mut model, mut actions, mut transport, mut audio) = opened_player_model();
    assert!(accept_email_item(
        0,
        4,
        &mut model,
        &mut transport,
        &mut audio
    ));
    assert_eq!(model.read_message.as_ref().unwrap().items[0].item_id, 100);
    assert_eq!(
        transport.pop(),
        Some(EmailRequest::ReceiveItem {
            email_index: 7,
            inventory_slot: 4,
            email_item_slot: 1,
        })
    );
    apply_email_reply(
        EmailReply::ReceiveItemSuccess {
            email_index: 7,
            inventory_slot: 4,
            email_item_slot: 1,
        },
        &mut model,
        &mut actions,
        &mut transport,
        &mut audio,
    );
    assert_eq!(model.read_message.as_ref().unwrap().items[0].item_id, 0);
    assert!(matches!(
        actions.0.back(),
        Some(EmailUiAction::RefreshInventory { .. })
    ));
}

#[test]
fn receive_all_preserves_clean_capacity_and_empty_attachment_messages() {
    let (mut model, mut actions, mut transport, mut audio) = opened_player_model();
    assert!(accept_all_email_items(
        0,
        &mut model,
        &mut actions,
        &mut transport,
        &mut audio
    ));
    assert!(transport.0.is_empty());
    assert!(matches!(
        actions.0.back(),
        Some(EmailUiAction::SystemMessage {
            message_id: Some(178),
            ..
        })
    ));
    model.read_message.as_mut().unwrap().items = [EmailWireItem::default(); 4];
    actions.0.clear();
    assert!(accept_all_email_items(
        5,
        &mut model,
        &mut actions,
        &mut transport,
        &mut audio
    ));
    assert!(matches!(
        actions.0.back(),
        Some(EmailUiAction::SystemMessage {
            message_id: Some(179),
            ..
        })
    ));
}

#[test]
fn buddy_calculator_and_attachment_staging_match_clean_compose_semantics() {
    let (mut model, _, _, mut audio) = opened_player_model();
    model.buddies = vec![EmailBuddy {
        pc_uid: 77,
        first_name: "Dexter".to_owned(),
        last_name: "McPherson".to_owned(),
        name_check_flag: 1,
    }];
    model.inventory[9] = Some(EmailInventorySlotView {
        item: EmailWireItem {
            item_type: 7,
            item_id: 123,
            option: 4,
            time_limit: 0,
        },
        icon_path: None,
        count_label: Some("4".to_owned()),
    });
    assert!(begin_email_compose(None, &mut model, &mut audio));
    model.tick_opening(EMAIL_UI_OPEN_SECONDS);

    model.popup = EmailPopup::BuddyList;
    assert!(select_email_buddy(0, &mut model, &mut audio));
    assert_eq!(model.draft.recipient_pc_uid, 77);
    assert_eq!(model.draft.recipient_name, "DexterMcPherson");

    model.popup = EmailPopup::AddTaros;
    model.available_cash = 245;
    for digit in [9, 9, 9] {
        assert!(input_email_calculator(
            EmailCalculatorInput::Digit(digit),
            &mut model
        ));
    }
    assert_eq!(model.calculator_value, 245);
    assert!(commit_email_calculator(&mut model, &mut audio));
    assert_eq!(model.draft.cash, 245);

    assert!(attach_email_inventory_item(9, 0, &mut model));
    assert!(model.inventory[9].is_some());
    assert_eq!(model.draft.postage(), 70);
    assert!(!attach_email_inventory_item(9, 1, &mut model));
    assert!(detach_email_inventory_item(0, &mut model));
    assert_eq!(model.draft.postage(), 50);
}

#[test]
fn compose_popup_accepts_only_its_own_controls() {
    let mut model = EmailUiModel {
        visible: true,
        screen: EmailScreen::Compose,
        opening_elapsed_seconds: EMAIL_UI_OPEN_SECONDS,
        right_opening_elapsed_seconds: EMAIL_UI_OPEN_SECONDS,
        popup: EmailPopup::BuddyList,
        ..default()
    };
    assert!(email_ui_button_enabled(
        &model,
        EmailUiButtonKind::BuddyPopupClose
    ));
    assert!(!email_ui_button_enabled(
        &model,
        EmailUiButtonKind::ComposeSend
    ));
    model.popup = EmailPopup::AddTaros;
    assert!(email_ui_button_enabled(
        &model,
        EmailUiButtonKind::CalculatorDigit(8)
    ));
    assert!(!email_ui_button_enabled(
        &model,
        EmailUiButtonKind::ComposeClose
    ));
}

#[test]
fn send_success_returns_to_guide_mail_and_emits_authority_refreshes() {
    let (mut model, mut actions, mut transport, mut audio) = opened_player_model();
    model.screen = EmailScreen::Compose;
    model.send_in_flight = true;
    model.mail_send_in_flight = true;
    apply_email_reply(
        EmailReply::SendSuccess {
            recipient_pc_uid: 99,
            authoritative_cash: 500,
            items: [EmailOutgoingItem::default(); 4],
        },
        &mut model,
        &mut actions,
        &mut transport,
        &mut audio,
    );
    assert_eq!(model.screen, EmailScreen::List);
    assert_eq!(model.folder, EmailFolder::Guide);
    assert!(!model.send_in_flight);
    assert!(!model.mail_send_in_flight);
    assert!(
        actions
            .0
            .iter()
            .any(|action| matches!(action, EmailUiAction::ApplySendSuccessItems(_)))
    );
    assert!(
        actions
            .0
            .iter()
            .any(|action| matches!(action, EmailUiAction::RefreshGuideEmail { .. }))
    );
    assert_eq!(audio.pop(), Some(EmailUiAudioCue::ActionSuccess));
}

#[test]
fn escape_then_computress_gate_preserves_the_clean_two_stage_exit_order() {
    let (mut model, mut actions, _, _) = opened_player_model();
    actions.0.clear();
    assert!(request_email_close(
        EmailCloseSource::Escape,
        &mut model,
        &mut actions
    ));
    assert_eq!(
        actions.pop(),
        Some(EmailUiAction::RequestEscapeCloseGate {
            event_group: 2,
            event_function: 24,
        })
    );
    assert!(resolve_email_escape_gate(true, &mut model, &mut actions));
    assert_eq!(
        actions.pop(),
        Some(EmailUiAction::QueryComputressExitGate {
            event_group: 11,
            event_function: 13,
        })
    );
    assert!(resolve_email_computress_gate(
        false,
        &mut model,
        &mut actions
    ));
    assert!(!model.visible);
    assert_eq!(
        actions.0.into_iter().collect::<Vec<_>>(),
        vec![
            EmailUiAction::SetInventoryMailMode {
                event_group: 11,
                event_function: 0,
                value: 10,
            },
            EmailUiAction::ExitMode {
                event_group: 2,
                event_function: 1,
            },
            EmailUiAction::StopUiModeSound,
            EmailUiAction::SetCursorLocked(false),
        ]
    );
}

#[test]
fn failure_codes_retain_clean_message_ids_and_time_travel_text() {
    assert_eq!(receive_item_failure_message(1, "item").0, Some(221));
    assert_eq!(receive_item_failure_message(5, "item").0, Some(222));
    assert_eq!(receive_cash_failure_message(1).0, Some(225));
    assert_eq!(send_failure_message(5).0, Some(171));
    assert!(email_fallback_text(&send_failure_message(9).1).contains("TIME TRAVEL"));
}

#[test]
fn system_message_fallbacks_retain_semantic_keys_and_dynamic_arguments() {
    let localized = send_failure_message(17).1;
    assert_eq!(localized.key, "ui.email.error.send.generic");
    assert_eq!(localized.args["error_code"], "17");
    assert_eq!(email_fallback_text(&localized), "ERROR!\nMail Error. 17");

    let action = email_system_message(
        None,
        LocalizedText::new(
            "ui.email.error.reply_rejected",
            "Email reply was rejected: {error}",
        )
        .with_arg("error", "bad packet"),
    );
    let EmailUiAction::SystemMessage {
        fallback,
        localized,
        ..
    } = action
    else {
        panic!("expected system message")
    };
    assert_eq!(fallback, "Email reply was rejected: bad packet");
    assert_eq!(localized.args["error"], "bad packet");
}
