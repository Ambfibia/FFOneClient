use super::*;

pub(super) fn setup_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cli: Res<PreviewCli>,
    mut model: ResMut<EmailUiModel>,
    mut actions: ResMut<EmailUiOutbox>,
    mut transport: ResMut<EmailTransportOutbox>,
    mut audio: ResMut<EmailUiAudioOutbox>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    spawn_world_backdrop(&mut commands);

    model.buddies = preview_buddies(&cli.language);
    model.inventory[0] = Some(inventory_item(7, 200, 25, ICON_GENERAL, Some("25")));
    model.inventory[1] = Some(inventory_item(0, 100, 0, ICON_WEAPON, None));
    model.inventory[4] = Some(inventory_item(1, 101, 0, ICON_COSMETIC, None));
    model.current_local_time = EmailSystemTime {
        year: 2026,
        month: 7,
        day: 31,
        hour: 19,
        ..default()
    };

    open_email_ui(
        &mut model,
        &mut actions,
        vec![EmailGuideMessage {
            mode: 0,
            mission_task_id: 2_001,
            sender_npc_id: 100,
            sender_name: "Numbuh Two".to_owned(),
            subject: "Welcome to the KND Treehouse".to_owned(),
            content: "Use your email to keep up with mission guides.".to_owned(),
            sender_icon_path: None,
            auto_delete_note: true,
            ..default()
        }],
        8_450,
    );
    match cli.scene {
        PreviewScene::Player => setup_player_scene(
            &mut model,
            &mut actions,
            &mut transport,
            &mut audio,
            &cli.language,
        ),
        PreviewScene::Compose | PreviewScene::Buddy | PreviewScene::Calculator => {
            setup_compose_scene(&mut model, &mut audio, cli.scene, &cli.language)
        }
    }

    commands.insert_resource(PreviewAssets {
        images: EMAIL_UI_IMAGE_PATHS
            .iter()
            .copied()
            .chain(PREVIEW_ICON_PATHS)
            .map(|path| asset_server.load(path))
            .collect(),
        fonts: [EMAIL_UI_FONT_PATH, EMAIL_UI_BODY_FONT_PATH].map(|path| asset_server.load(path)),
    });
}

pub(super) fn preview_buddies(language: &str) -> Vec<email_ui::EmailBuddy> {
    let names: &[(&str, &str)] = if language == "ru" {
        &[
            ("Декстер", "Макферсон"),
            ("Ди Ди", ""),
            ("Бен", "Теннисон"),
            ("Самурай", "Джек"),
            ("Номер", "Два"),
            ("Коко", "Фостер"),
            ("Гвен", "Теннисон"),
        ]
    } else {
        &[
            ("Dexter", "McPherson"),
            ("Dee Dee", ""),
            ("Ben", "Tennyson"),
            ("Samurai", "Jack"),
            ("Numbuh", "Two"),
            ("Coco", "Foster"),
            ("Gwen", "Tennyson"),
        ]
    };
    names
        .iter()
        .enumerate()
        .map(|(index, (first_name, last_name))| email_ui::EmailBuddy {
            pc_uid: 3_001 + index as i64,
            first_name: (*first_name).to_owned(),
            last_name: (*last_name).to_owned(),
            name_check_flag: 1,
        })
        .collect()
}

pub(super) fn setup_player_scene(
    model: &mut EmailUiModel,
    actions: &mut EmailUiOutbox,
    transport: &mut EmailTransportOutbox,
    audio: &mut EmailUiAudioOutbox,
    language: &str,
) {
    assert!(switch_email_folder(
        EmailFolder::Player,
        model,
        transport,
        audio,
    ));
    assert!(matches!(
        transport.pop(),
        Some(email_ui::EmailRequest::PageList { page: 1 })
    ));
    apply_email_reply(
        EmailReply::PageListSuccess {
            page: 1,
            messages: preview_summaries(language),
        },
        model,
        actions,
        transport,
        audio,
    );
    assert!(matches!(
        transport.pop(),
        Some(email_ui::EmailRequest::Read { email_index: 7_001 })
    ));
    let content = if language == "ru" {
        "Встретимся у фонтана на Техплощади. Я приложил капсулу нано, которую ты просил."
    } else {
        "Meet me by the Tech Square fountain. I attached the nano capsule you asked for."
    };
    apply_email_reply(
        EmailReply::ReadSuccess(EmailReadMessage {
            email_index: 7_001,
            content: if env::var_os("FFONE_REVIEW_LONG_EMAIL").is_some() { content.repeat(12) } else { content.to_owned() },
            items: [
                EmailWireItem {
                    item_type: 7,
                    item_id: 200,
                    option: 3,
                    time_limit: 0,
                },
                EmailWireItem::default(),
                EmailWireItem::default(),
                EmailWireItem::default(),
            ],
            cash: 250,
        }),
        model,
        actions,
        transport,
        audio,
    );
    model.tick_opening(email_ui::EMAIL_UI_OPEN_SECONDS);
}

pub(super) fn setup_compose_scene(
    model: &mut EmailUiModel,
    audio: &mut EmailUiAudioOutbox,
    scene: PreviewScene,
    language: &str,
) {
    assert!(begin_email_compose(
        Some((3_001, String::new())),
        model,
        audio,
    ));
    model.tick_opening(email_ui::EMAIL_UI_OPEN_SECONDS);
    if language == "ru" {
        model.draft.set_subject("Капсула нано для Техплощади");
        model
            .draft
            .set_content("Спасибо! Приложение получено, встретимся у фонтана после задания.");
    } else {
        model.draft.set_subject("Nano capsule for Tech Square");
        model
            .draft
            .set_content("Thanks! Attachment received; meet me by the fountain after the mission.");
    }
    model.draft.attachments[0] = Some(EmailOutgoingItem {
        inventory_slot: 0,
        item: EmailWireItem {
            item_type: 7,
            item_id: 200,
            option: 25,
            time_limit: 0,
        },
    });
    model.draft.cash = 245;
    model.calculator_value = 245;
    model.popup = match scene {
        PreviewScene::Buddy => EmailPopup::BuddyList,
        PreviewScene::Calculator => EmailPopup::AddTaros,
        PreviewScene::Compose | PreviewScene::Player => EmailPopup::None,
    };
}

pub(super) fn preview_summaries(language: &str) -> Vec<EmailSummary> {
    let en = [
        (
            7_001,
            3_001,
            "Dexter",
            "McPherson",
            "Nano capsule delivery",
            31,
        ),
        (7_002, 3_002, "Dee Dee", "", "Meet me at Peach Creek", 30),
        (
            7_003,
            3_003,
            "Ben",
            "Tennyson",
            "The Null Void needs heroes",
            29,
        ),
        (7_004, 3_004, "Mac", "", "Foster's rooftop race", 28),
        (
            7_005,
            3_005,
            "Samurai",
            "Jack",
            "A message from the past",
            27,
        ),
    ];
    let ru = [
        (
            7_001,
            3_001,
            "Декстер",
            "Макферсон",
            "Доставка капсулы нано",
            31,
        ),
        (7_002, 3_002, "Ди Ди", "", "Встретимся у Ручья", 30),
        (
            7_003,
            3_003,
            "Бен",
            "Теннисон",
            "Нуль-пространству нужны герои",
            29,
        ),
        (7_004, 3_004, "Мак", "", "Гонка по крыше Фостера", 28),
        (7_005, 3_005, "Самурай", "Джек", "Послание из прошлого", 27),
    ];
    (if language == "ru" { &ru } else { &en })
        .iter()
        .copied()
        .map(
            |(email_index, from_pc_uid, first_name, last_name, subject, day)| EmailSummary {
                email_index,
                from_pc_uid,
                first_name: first_name.to_owned(),
                last_name: last_name.to_owned(),
                subject: subject.to_owned(),
                read_flag: if email_index == 7_001 { 1 } else { 0 },
                send_time: EmailSystemTime {
                    year: 2026,
                    month: 7,
                    day,
                    hour: 18,
                    minute: 42,
                    ..default()
                },
                delete_time: EmailSystemTime::default(),
                item_cash_flag: if email_index == 7_001 { 1 } else { 0 },
            },
        )
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    preview_assets: Res<PreviewAssets>,
    cli: Res<PreviewCli>,
    model: Res<EmailUiModel>,
    mut state: ResMut<PreviewState>,
    roots: Query<(&ComputedNode, &Node), With<EmailUiRoot>>,
    list_panels: Query<(&ComputedNode, &UiTransform, &Node), With<EmailUiListPanel>>,
    compose_panels: Query<(&ComputedNode, &UiTransform, &Node), With<EmailUiComposePanel>>,
    right_panels: Query<(&ComputedNode, &UiTransform, &Node), With<EmailUiRightPanel>>,
    buttons: Query<(&EmailUiButton, &Interaction, &ComputedNode)>,
    role_texts: Query<(&EmailUiTextElement, &Text)>,
    all_texts: Query<&Text>,
    styled_texts: Query<(
        Entity,
        &Text,
        &LocalizedText,
        &EmailUiTextStyle,
        (&TextFont, &LineHeight),
        &TextLayoutInfo,
        &ComputedNode,
        &UiTransform,
        &ChildOf,
        Option<&EmailUiTextElement>,
    )>,
    computed_nodes: Query<&ComputedNode>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);

    let failed = preview_assets
        .images
        .iter()
        .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
        || preview_assets
            .fonts
            .iter()
            .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)));
    if failed {
        eprintln!("Email acceptance asset failed to load");
        exit.write(AppExit::error());
        return;
    }

    let assets_loaded = preview_assets
        .images
        .iter()
        .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
        && preview_assets
            .fonts
            .iter()
            .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded));
    let root_exact = roots.single().is_ok_and(|(computed, node)| {
        computed.size() == Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32)
            && node.display == Display::Flex
    });
    let list_exact = list_panels
        .single()
        .is_ok_and(|(computed, transform, node)| {
            if cli.scene == PreviewScene::Player {
                computed.size() == Vec2::new(567.0, 634.0)
                    && transform.scale == Vec2::ONE
                    && node.left == px(122.0)
                    && node.top == px(21.0)
                    && node.display == Display::Flex
            } else {
                node.display == Display::None
            }
        });
    let compose_exact = compose_panels
        .single()
        .is_ok_and(|(computed, transform, node)| {
            if cli.scene == PreviewScene::Player {
                node.display == Display::None
            } else {
                computed.size() == Vec2::new(940.0, 634.0)
                    && transform.scale == Vec2::ONE
                    && node.left == px(122.0)
                    && node.top == px(21.0)
                    && node.display == Display::Flex
            }
        });
    let right_exact = right_panels
        .single()
        .is_ok_and(|(computed, transform, node)| {
            computed.size() == Vec2::new(380.0, 632.0)
                && transform.scale == Vec2::ONE
                && node.left == px(707.0)
                && node.top == px(21.0)
        });
    let expected_hover_size = match cli.scene {
        PreviewScene::Player => Vec2::new(160.0, 25.0),
        PreviewScene::Compose | PreviewScene::Buddy => Vec2::new(130.0, 25.0),
        PreviewScene::Calculator => Vec2::new(44.0, 19.0),
    };
    let button_contract = buttons.iter().any(|(button, interaction, computed)| {
        button.kind == cli.scene.hover()
            && *interaction == Interaction::Hovered
            && computed.size() == expected_hover_size
    }) && buttons.iter().any(|(button, _, computed)| {
        button.kind == EmailUiButtonKind::RightClose && computed.size() == Vec2::new(30.0, 30.0)
    });
    let text_contract = match cli.scene {
        PreviewScene::Player => role_texts.iter().any(|(element, text)| {
            element.role == EmailUiTextRole::DetailBody
                && if cli.language == "ru" {
                    text.0.contains("Техплощади")
                } else {
                    text.0.contains("Tech Square fountain")
                }
        }),
        PreviewScene::Compose | PreviewScene::Buddy => role_texts.iter().any(|(element, text)| {
            element.role == EmailUiTextRole::ComposeSubject
                && if cli.language == "ru" {
                    text.0.contains("Капсула нано")
                } else {
                    text.0.contains("Nano capsule")
                }
        }),
        PreviewScene::Calculator => role_texts.iter().any(|(element, text)| {
            element.role == EmailUiTextRole::CalculatorValue && text.0 == "245"
        }),
    };
    let boundary_exact = model.input_boundary()
        == EmailInputBoundary {
            blocks_lower_ui: true,
            blocks_gameplay_input: true,
            requires_pointer: true,
            cursor_locked_while_visible: false,
            cursor_locked_after_exit: false,
            mouse_controls_enabled: true,
            escape_close_gate_enabled: true,
        };
    let authority_exact = match cli.scene {
        PreviewScene::Player => {
            model.screen == EmailScreen::List
                && model.folder == EmailFolder::Player
                && model.player_page == 1
                && model.player_messages.len() == 5
                && model.selected_row == Some(0)
                && model
                    .read_message
                    .as_ref()
                    .is_some_and(|message| message.email_index == 7_001 && message.cash == 250)
        }
        PreviewScene::Compose => {
            model.screen == EmailScreen::Compose
                && model.popup == EmailPopup::None
                && model.draft.cash == 245
                && model.draft.postage() == 70
        }
        PreviewScene::Buddy => {
            model.screen == EmailScreen::Compose
                && model.popup == EmailPopup::BuddyList
                && model.buddies.len() == 7
        }
        PreviewScene::Calculator => {
            model.screen == EmailScreen::Compose
                && model.popup == EmailPopup::AddTaros
                && model.calculator_value == 245
        }
    };
    let text_audit = audit_visible_text(
        &all_texts,
        &styled_texts,
        &computed_nodes,
        &preview_assets,
        cli.scene,
        &cli.language,
    );
    if assets_loaded && !state.text_audited {
        match &text_audit {
            Ok(report) => {
                println!("{report}");
                state.text_audited = true;
            }
            Err(error) if state.frames % 120 == 0 => eprintln!("Email text audit waiting: {error}"),
            Err(_) => {}
        }
    }
    let text_audit_exact = text_audit.is_ok();
    let ready = assets_loaded
        && root_exact
        && list_exact
        && compose_exact
        && right_exact
        && button_contract
        && text_contract
        && text_audit_exact
        && boundary_exact
        && authority_exact;
    if ready && state.ready_frame.is_none() {
        state.ready_frame = Some(state.frames);
        state.ready_at = Some(Instant::now());
    }

    let warmed = state
        .ready_frame
        .is_some_and(|frame| state.frames.saturating_sub(frame) >= WARMUP_FRAMES_AFTER_LOAD)
        && state
            .ready_at
            .is_some_and(|ready_at| ready_at.elapsed() >= GPU_UPLOAD_GRACE);
    if ready && warmed && !state.capture_issued {
        state.capture_issued = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }

    if state.capture_saved {
        exit.write(AppExit::Success);
    } else if state.capture_failed {
        exit.write(AppExit::error());
    } else if state.started_at.elapsed() >= CAPTURE_TIMEOUT {
        eprintln!("Email capture timed out");
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

pub(super) fn expected_text_metrics(style: EmailUiTextStyle) -> (usize, f32, f32) {
    match style {
        EmailUiTextStyle::LabelUpperLeft
        | EmailUiTextStyle::LabelMiddleCenter
        | EmailUiTextStyle::LabelMiddleRight
        | EmailUiTextStyle::BlankBoxUpperLeft
        | EmailUiTextStyle::BlankBoxUpperCenter
        | EmailUiTextStyle::BlankBoxMiddleLeft
        | EmailUiTextStyle::BlankBoxMiddleRight
        | EmailUiTextStyle::ButtonLabelFont
        | EmailUiTextStyle::RedButtonLabelFont => (
            0,
            EMAIL_UI_JEFFE_12_FONT_SIZE,
            EMAIL_UI_JEFFE_12_LINE_HEIGHT,
        ),
        EmailUiTextStyle::Button | EmailUiTextStyle::MailPageButton => (
            0,
            EMAIL_UI_JEFFE_14_FONT_SIZE,
            EMAIL_UI_JEFFE_14_LINE_HEIGHT,
        ),
        EmailUiTextStyle::CalculatorButton => (
            0,
            EMAIL_UI_JEFFE_16_FONT_SIZE,
            EMAIL_UI_JEFFE_16_LINE_HEIGHT,
        ),
        EmailUiTextStyle::PostageLabel => (
            0,
            EMAIL_UI_JEFFE_06_FONT_SIZE,
            EMAIL_UI_JEFFE_06_LINE_HEIGHT,
        ),
        EmailUiTextStyle::CenterLabel
        | EmailUiTextStyle::RightLabel
        | EmailUiTextStyle::ChaletLabelMiddleLeft
        | EmailUiTextStyle::ChaletLabelMiddleCenter
        | EmailUiTextStyle::TextArea => (
            1,
            EMAIL_UI_CHALET_SMALL_FONT_SIZE,
            EMAIL_UI_CHALET_SMALL_LINE_HEIGHT,
        ),
    }
}
