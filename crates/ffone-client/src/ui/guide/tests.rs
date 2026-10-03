    use crate::guide_ui::*;
    use tempfile::tempdir;

    fn command(
        model: &mut GuideUiModel,
        outbox: &mut GuideUiOutbox,
        audio: &mut GuideUiAudioOutbox,
        command: GuideUiCommand,
    ) -> bool {
        apply_guide_ui_command(model, outbox, audio, command)
    }

    #[test]
    fn clean_mentor_order_ids_names_and_tabledata_descriptions_are_exact() {
        assert_eq!(
            GuideMentor::CLEAN_ORDER.map(GuideMentor::wire_id),
            [4, 2, 3, 1]
        );
        assert_eq!(
            GuideMentor::CLEAN_ORDER.map(GuideMentor::name),
            ["BEN TENNYSON", "DEXTER", "MOJO JOJO", "EDD"]
        );
        assert_eq!(
            GuideMentor::BenTennyson.description(),
            "Ben must prevent Fuse from getting his claws on hidden Plumber technology, and only \
you can help him!"
        );
        assert_eq!(
            GuideMentor::Edd.description(),
            "Edd is on the hunt for hidden candy treasure! Help him dig up some sweet riches, and \
dirty secrets!"
        );
        assert_eq!(GuideMentor::from_wire_id(5), None);
        assert_eq!(GuideMentor::from_wire_id(0), None);
    }

    #[test]
    fn clean_component_geometry_and_canonical_layout_are_exact() {
        assert_eq!(GUIDE_UI_PRIMARY_MAIN_BYTES, 7_000_415);
        assert_eq!(
            GUIDE_UI_PRIMARY_MAIN_SHA256,
            "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F"
        );
        assert_eq!(
            GUIDE_WINDOW_RECT,
            GuideUiRect::new(0.0, 0.0, 1_036.0, 654.0)
        );
        assert_eq!(GUIDE_CARD_RECT, GuideUiRect::new(0.0, 166.0, 200.0, 124.0));
        assert_eq!(
            GUIDE_MODAL_RECT,
            GuideUiRect::new(258.0, 245.0, 520.0, 164.0)
        );
        assert_eq!(GUIDE_MODAL_TITLE_RECT.x, 398.0);
        assert_eq!(GUIDE_MODAL_TITLE_RECT.y, 265.0);
        assert!((GUIDE_MODAL_TITLE_RECT.height - 19.710_001).abs() < 0.000_1);
        assert!((GUIDE_MODAL_BODY_RECT.y - 294.710_02).abs() < 0.000_1);
        assert!((GUIDE_MODAL_BODY_RECT.height - 74.289_99).abs() < 0.000_1);
        let layout = guide_ui_layout(Vec2::new(1_264.0, 681.0), 1.0);
        assert_eq!(
            layout.window.source,
            GuideUiRect::new(114.0, 13.0, 1_036.0, 654.0)
        );
        assert_eq!(
            layout.background.source,
            GuideUiRect::new(-328.0, -379.0, 1_920.0, 1_440.0)
        );
        assert_eq!(layout.window.painted, layout.window.source);
    }

    #[test]
    fn mentor_portrait_icon_confirm_and_selection_geometry_is_source_backed() {
        let expected = [
            (
                GuideMentor::BenTennyson,
                GuideUiRect::new(55.0, 243.0, 160.0, 287.0),
                Vec2::new(34.0, 34.0),
                Vec2::new(116.0, 182.0),
            ),
            (
                GuideMentor::Dexter,
                GuideUiRect::new(278.0, 243.0, 224.0, 280.0),
                Vec2::new(32.0, 30.0),
                Vec2::new(160.0, 195.0),
            ),
            (
                GuideMentor::MojoJojo,
                GuideUiRect::new(512.0, 251.0, 281.0, 272.0),
                Vec2::new(30.0, 35.0),
                Vec2::new(285.0, 196.0),
            ),
            (
                GuideMentor::Edd,
                GuideUiRect::new(807.0, 245.0, 180.0, 295.0),
                Vec2::new(28.0, 29.0),
                Vec2::new(109.0, 162.0),
            ),
        ];
        for (mentor, portrait, icon, confirm) in expected {
            assert_eq!(mentor.portrait_rect(), portrait);
            assert_eq!(mentor.icon_size(), icon);
            assert_eq!(mentor.confirm_size(), confirm);
            assert_eq!(mentor.selected_effect_rect().x, mentor.guide_position().x - 133.0);
            assert_eq!(mentor.selected_effect_rect().width, 267.0);
            assert_eq!(mentor.selected_effect_rect().height, 414.0);
        }
    }

    #[test]
    fn initial_flow_preserves_warp_selection_and_confirmation_sequence() {
        let mut model = GuideUiModel::default();
        let mut outbox = GuideUiOutbox::default();
        let mut audio = GuideUiAudioOutbox::default();
        model.open_initial_selection();
        assert_eq!(model.phase, GuideUiPhase::WarpWarning);
        assert!(command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::AcceptWarpWarning
        ));
        assert_eq!(model.phase, GuideUiPhase::MentorSelection);
        assert_eq!(outbox.pop_front(), Some(GuideUiAction::WarpWarningAccepted));
        assert_eq!(audio.pop_front(), Some(GuideUiAudioCue::YesButton));
        assert_eq!(
            model.primary_blocker(),
            Some(GuideUiBlocker::NoMentorSelected)
        );
        assert!(command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::SelectMentor(GuideMentor::MojoJojo)
        ));
        assert!(command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::OpenConfirmation
        ));
        assert!(command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::ConfirmMentor
        ));
        assert!(model.awaiting_server);
        assert_eq!(model.pending_mentor(), Some(GuideMentor::MojoJojo));
        assert_eq!(
            outbox.pop_front(),
            Some(GuideUiAction::ChangeMentorRequested {
                mentor: GuideMentor::MojoJojo,
            })
        );
    }

    #[test]
    fn nonzero_future_service_opens_initial_selection_without_synthetic_warp_acceptance() {
        let mut model = GuideUiModel::default();
        model.open_initial_mentor_selection();
        assert!(model.visible);
        assert_eq!(model.purpose, GuideUiPurpose::InitialSelection);
        assert_eq!(model.phase, GuideUiPhase::MentorSelection);
        assert_eq!(model.current, None);
        assert_eq!(model.selected, None);
        assert!(!model.awaiting_server);
    }

    #[test]
    fn local_transport_cancellation_reenables_selection_without_closing() {
        let mut model = GuideUiModel::default();
        let mut outbox = GuideUiOutbox::default();
        let mut audio = GuideUiAudioOutbox::default();
        model.open_change(GuideMentor::Dexter);
        assert!(apply_guide_ui_command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::SelectMentor(GuideMentor::Edd),
        ));
        assert!(apply_guide_ui_command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::OpenConfirmation,
        ));
        assert!(apply_guide_ui_command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::ConfirmMentor,
        ));
        assert!(model.awaiting_server);
        model.cancel_pending_request();
        assert!(model.visible);
        assert!(!model.awaiting_server);
        assert_eq!(model.pending_mentor(), None);
    }

    #[test]
    fn change_flow_uses_exact_zero_cost_and_confirmation_copy_without_quote_action() {
        let mut model = GuideUiModel::default();
        model.open_change(GuideMentor::Dexter);
        assert_eq!(model.displayed_price(), Some(0));
        let mut outbox = GuideUiOutbox::default();
        let mut audio = GuideUiAudioOutbox::default();
        assert!(command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::SelectMentor(GuideMentor::BenTennyson)
        ));
        assert_eq!(model.displayed_price(), Some(0));
        assert!(outbox.is_empty());
        assert!(command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::OpenConfirmation
        ));
        let view = guide_ui_view(&model, Vec2::new(1_264.0, 681.0)).unwrap();
        let confirmation = view.confirmation.unwrap();
        assert_eq!(confirmation.title, "ALERT");
        assert_eq!(confirmation.mentor, GuideMentor::BenTennyson);
        assert!(confirmation.body.contains("your currently have equipped"));
        assert_eq!(view.displayed_cost.as_deref(), Some("cost : 0"));
    }

    #[test]
    fn confirming_current_mentor_emits_message_155_without_network_wait() {
        let mut model = GuideUiModel::default();
        model.open_change(GuideMentor::Dexter);
        let mut outbox = GuideUiOutbox::default();
        let mut audio = GuideUiAudioOutbox::default();
        assert!(command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::OpenConfirmation
        ));
        assert!(command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::ConfirmMentor
        ));
        assert!(!model.awaiting_server);
        assert_eq!(
            outbox.pop_front(),
            Some(GuideUiAction::CurrentMentorRejected {
                mentor: GuideMentor::Dexter,
                system_message_id: 155,
            })
        );
    }

    #[test]
    fn reply_correlation_fails_closed_and_non_first_success_refreshes_missions() {
        let mut model = GuideUiModel::default();
        model.open_change(GuideMentor::Dexter);
        let mut outbox = GuideUiOutbox::default();
        let mut audio = GuideUiAudioOutbox::default();
        command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::SelectMentor(GuideMentor::Edd),
        );
        command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::OpenConfirmation,
        );
        command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::ConfirmMentor,
        );
        assert!(!resolve_guide_change_success(
            &mut model,
            &mut outbox,
            GuideChangeSuccess {
                mentor: GuideMentor::BenTennyson,
                mentor_count: 2,
                fusion_matter: 800,
            }
        ));
        assert!(model.awaiting_server);
        assert!(resolve_guide_change_success(
            &mut model,
            &mut outbox,
            GuideChangeSuccess {
                mentor: GuideMentor::Edd,
                mentor_count: 2,
                fusion_matter: 800,
            }
        ));
        assert!(!model.visible);
        assert_eq!(
            outbox.pop_front(),
            Some(GuideUiAction::ChangeMentorRequested {
                mentor: GuideMentor::Edd,
            })
        );
        assert_eq!(
            outbox.pop_front(),
            Some(GuideUiAction::MentorChangeSucceeded {
                mentor: GuideMentor::Edd,
                mentor_count: 2,
                fusion_matter: 800,
                refresh_guide_missions: true,
            })
        );
    }

    #[test]
    fn profile_correlated_later_success_preserves_raw_count_one_without_first_warp() {
        let mut model = GuideUiModel::default();
        model.open_change(GuideMentor::Dexter);
        let mut outbox = GuideUiOutbox::default();
        let mut audio = GuideUiAudioOutbox::default();
        assert!(command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::SelectMentor(GuideMentor::Edd),
        ));
        assert!(command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::OpenConfirmation,
        ));
        assert!(command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::ConfirmMentor,
        ));
        assert_eq!(
            outbox.pop_front(),
            Some(GuideUiAction::ChangeMentorRequested {
                mentor: GuideMentor::Edd,
            })
        );

        assert!(resolve_correlated_guide_change_success(
            &mut model,
            &mut outbox,
            GuideChangeSuccess {
                mentor: GuideMentor::Edd,
                mentor_count: 1,
                fusion_matter: 800,
            },
            false,
        ));
        assert_eq!(
            outbox.pop_front(),
            Some(GuideUiAction::MentorChangeSucceeded {
                mentor: GuideMentor::Edd,
                mentor_count: 1,
                fusion_matter: 800,
                refresh_guide_missions: true,
            })
        );
    }

    #[test]
    fn first_change_and_failure_preserve_clean_mode_and_message_ids() {
        let mut model = GuideUiModel::default();
        let mut outbox = GuideUiOutbox::default();
        let mut audio = GuideUiAudioOutbox::default();
        model.open_initial_selection();
        command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::AcceptWarpWarning,
        );
        command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::SelectMentor(GuideMentor::BenTennyson),
        );
        command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::OpenConfirmation,
        );
        command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::ConfirmMentor,
        );
        assert!(resolve_guide_change_success(
            &mut model,
            &mut outbox,
            GuideChangeSuccess {
                mentor: GuideMentor::BenTennyson,
                mentor_count: 1,
                fusion_matter: 1_000,
            }
        ));
        assert!(matches!(
            outbox.pop_front(),
            Some(GuideUiAction::WarpWarningAccepted)
        ));
        assert!(matches!(
            outbox.pop_front(),
            Some(GuideUiAction::ChangeMentorRequested { .. })
        ));
        assert_eq!(
            outbox.pop_front(),
            Some(GuideUiAction::FirstMentorChangeSucceeded {
                mentor: GuideMentor::BenTennyson,
                mentor_count: 1,
                fusion_matter: 1_000,
                next_mode: 10,
                warp_npc_table_id: 1_425,
            })
        );

        model.open_change(GuideMentor::Dexter);
        command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::SelectMentor(GuideMentor::MojoJojo),
        );
        command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::OpenConfirmation,
        );
        command(
            &mut model,
            &mut outbox,
            &mut audio,
            GuideUiCommand::ConfirmMentor,
        );
        assert!(resolve_guide_change_failure(
            &mut model,
            &mut outbox,
            GuideMentor::MojoJojo,
            7,
        ));
        assert_eq!(model.current, Some(GuideMentor::Dexter));
        assert!(matches!(
            outbox.pop_front(),
            Some(GuideUiAction::ChangeMentorRequested { .. })
        ));
        assert_eq!(
            outbox.pop_front(),
            Some(GuideUiAction::MentorChangeFailed {
                mentor: GuideMentor::MojoJojo,
                error_code: 7,
                system_message_id: 154,
            })
        );
    }

    #[test]
    fn external_popup_and_in_flight_request_preserve_input_boundary() {
        let mut model = GuideUiModel::default();
        model.open_change(GuideMentor::Dexter);
        model.set_external_modes(true, false);
        assert_eq!(
            model.input_boundary(),
            GuideUiInputBoundary {
                blocks_lower_ui: true,
                blocks_gameplay_input: true,
                requires_pointer: true,
                mouse_controls_enabled: false,
                escape_dismiss_enabled: false,
            }
        );
        model.set_external_modes(false, true);
        assert!(!model.controls_enabled());
        assert!(model.input_boundary().escape_dismiss_enabled);
    }

    #[test]
    fn cost_icon_formula_and_exact_source_assets_are_stable() {
        let (text, icon) = guide_cost_text_and_icon_rects(72.0);
        assert_eq!(text, GuideUiRect::new(467.0, 572.0, 72.0, 20.0));
        assert_eq!(icon, GuideUiRect::new(575.0, 567.0, 31.0, 31.0));
        assert_eq!(GUIDE_CHANGE_PRICES, [0; 4]);
        assert_eq!(
            GUIDE_CHANGE_FAILURE_MESSAGE,
            "ERROR!\nYou do not have enough Fusion Matter to change your guide."
        );
        assert_eq!(GUIDE_ALREADY_CURRENT_MESSAGE, "Can not change same guide.");
        assert_eq!(
            GuideMentor::CLEAN_ORDER.map(GuideMentor::portrait_path),
            [
                "ui/en/gameplay/guide/ben.png",
                "ui/en/gameplay/guide/dexter.png",
                "ui/en/gameplay/guide/mojo.png",
                "ui/en/gameplay/guide/edd.png",
            ]
        );
    }

    #[test]
    fn passive_renderer_pickability_tracks_phase_confirmation_and_external_gates() {
        let mut model = GuideUiModel::default();
        assert!(!guide_ui_button_enabled(
            &model,
            GuideUiCommand::AcceptWarpWarning
        ));

        model.open_initial_selection();
        assert!(guide_ui_button_enabled(
            &model,
            GuideUiCommand::AcceptWarpWarning
        ));
        assert!(guide_ui_button_enabled(
            &model,
            GuideUiCommand::Dismiss(GuideUiDismissalSource::WarpCancelButton)
        ));
        assert!(!guide_ui_button_enabled(
            &model,
            GuideUiCommand::SelectMentor(GuideMentor::Edd)
        ));

        model.phase = GuideUiPhase::MentorSelection;
        assert!(guide_ui_button_enabled(
            &model,
            GuideUiCommand::SelectMentor(GuideMentor::Edd)
        ));
        model.selected = Some(GuideMentor::Edd);
        model.confirmation_open = true;
        assert!(!guide_ui_button_enabled(
            &model,
            GuideUiCommand::SelectMentor(GuideMentor::BenTennyson)
        ));
        assert!(guide_ui_button_enabled(
            &model,
            GuideUiCommand::ConfirmMentor
        ));

        model.set_external_modes(true, false);
        assert!(!guide_ui_button_enabled(
            &model,
            GuideUiCommand::ConfirmMentor
        ));
    }

    #[test]
    fn guide_badges_straddle_card_without_clipping_and_follow_reopening() {
        let asset_root = tempdir().unwrap();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin {
                file_path: asset_root.path().to_string_lossy().into_owned(),
                ..default()
            })
            .init_asset::<Image>()
            .init_asset::<Font>()
            .add_plugins(GuideUiPlugin);
        app.update();

        let assets = app.world().resource::<GuideUiAssets>().clone();
        let world = app.world_mut();
        let mut windows = world.query_filtered::<&Children, With<GuideUiSelectionWindow>>();
        let siblings: Vec<Entity> = windows.single(world).unwrap().iter().collect();
        for mentor in GuideMentor::CLEAN_ORDER {
            let portrait = siblings
                .iter()
                .position(|&entity| {
                    world.get::<ImageNode>(entity).is_some_and(|image| {
                        image.image == assets.portraits[mentor.slot()]
                    })
                })
                .unwrap();
            let effect = siblings
                .iter()
                .position(|&entity| {
                    world.get::<GuideUiMentorSelectedEffect>(entity)
                        .is_some_and(|effect| effect.mentor == mentor)
                })
                .unwrap();
            let frame = siblings
                .iter()
                .position(|&entity| {
                    world.get::<GuideUiMentorCurrentFrame>(entity)
                        .is_some_and(|frame| frame.mentor == mentor)
                })
                .unwrap();
            let card = siblings.iter().position(|&entity| {
                world.get::<GuideUiMentorCard>(entity)
                    .is_some_and(|card| card.mentor == mentor)
            }).unwrap();
            let icon = siblings.iter().position(|&entity| {
                world.get::<ImageNode>(entity)
                    .is_some_and(|image| image.image == assets.icons[mentor.slot()])
            }).unwrap();
            assert!(frame < card && card < icon && icon < effect && effect < portrait,
                "{mentor:?} badge must paint over frame and outside clipped content");
            let card_entity = siblings[card];
            assert_eq!(world.get::<Node>(card_entity).unwrap().overflow, Overflow::clip());
            let icon_node = world.get::<Node>(siblings[icon]).unwrap();
            assert_eq!(icon_node.left, px(mentor.card_rect().x - mentor.icon_size().x / 2.0));
            assert_eq!(icon_node.top, px(mentor.card_rect().y));
            assert_eq!(icon_node.width, px(mentor.icon_size().x));
            assert_eq!(icon_node.height, px(mentor.icon_size().y));
        }

        for current in GuideMentor::CLEAN_ORDER {
            app.world_mut().resource_mut::<GuideUiModel>().open_change(current);
            app.update();
            let world = app.world_mut();
            let mut frames = world.query::<(&GuideUiMentorCurrentFrame, &Node)>();
            let visible: Vec<_> = frames
                .iter(world)
                .filter_map(|(frame, node)| (node.display != Display::None).then_some(frame.mentor))
                .collect();
            assert_eq!(visible, vec![current]);

            let mut effects = world.query::<(&GuideUiMentorSelectedEffect, &Node)>();
            let selected: Vec<_> = effects
                .iter(world)
                .filter_map(|(effect, node)| (node.display != Display::None).then_some(effect.mentor))
                .collect();
            assert_eq!(selected, vec![current]);

            let proposed = GuideMentor::CLEAN_ORDER[(current.slot() + 1) % 4];
            app.world_mut().resource_mut::<GuideUiModel>().selected = Some(proposed);
            app.update();
            let world = app.world_mut();
            let mut frames = world.query::<(&GuideUiMentorCurrentFrame, &Node)>();
            assert_eq!(
                frames
                    .iter(world)
                    .filter_map(|(frame, node)|
                        (node.display != Display::None).then_some(frame.mentor))
                    .collect::<Vec<_>>(),
                vec![current]
            );
            let mut effects = world.query::<(&GuideUiMentorSelectedEffect, &Node)>();
            assert_eq!(
                effects
                    .iter(world)
                    .filter_map(|(effect, node)|
                        (node.display != Display::None).then_some(effect.mentor))
                    .collect::<Vec<_>>(),
                vec![proposed]
            );

            // Dismissing an unconfirmed choice and reopening from server state
            // restores the one confirmed marker and its initial highlight.
            {
                let mut model = app.world_mut().resource_mut::<GuideUiModel>();
                model.close();
                model.open_change(current);
            }
            app.update();
            let world = app.world_mut();
            let mut effects = world.query::<(&GuideUiMentorSelectedEffect, &Node)>();
            assert_eq!(
                effects
                    .iter(world)
                    .filter_map(|(effect, node)|
                        (node.display != Display::None).then_some(effect.mentor))
                    .collect::<Vec<_>>(),
                vec![current]
            );
        }

        app.world_mut().resource_mut::<GuideUiModel>().close();
        app.update();
        let world = app.world_mut();
        let mut frames = world.query::<(&GuideUiMentorCurrentFrame, &Node)>();
        assert!(frames.iter(world).all(|(_, node)| node.display == Display::None));
    }

    #[test]
    fn presentation_tree_is_key_first_and_preserves_exact_source_style_metrics() {
        let asset_root = tempdir().unwrap();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin {
                file_path: asset_root.path().to_string_lossy().into_owned(),
                ..default()
            })
            .init_asset::<Image>()
            .init_asset::<Font>()
            .add_plugins(GuideUiPlugin);
        app.update();

        let (jeffe, chalet) = {
            let assets = app.world().resource::<GuideUiAssets>();
            (assets.jeffe.clone(), assets.chalet.clone())
        };
        let world = app.world_mut();
        let mut controls = world.query_filtered::<(&Node, &ImageNode), With<Button>>();
        let padded = controls
            .iter(world)
            .filter(|(node, _)| node.padding != UiRect::ZERO)
            .collect::<Vec<_>>();
        assert!(
            !padded.is_empty(),
            "production Guide buttons must exercise caption padding"
        );
        for (_, image) in padded {
            assert_eq!(image.visual_box, bevy::ui::VisualBox::BorderBox);
        }
        let mut every_text = world.query::<(&Text, Option<&LocalizedText>)>();
        let every_text_count = every_text.iter(world).count();
        assert_eq!(every_text_count, 26);
        assert!(
            every_text
                .iter(world)
                .all(|(_, localized)| localized.is_some_and(|text| !text.key.is_empty()))
        );

        let mut styled = world.query::<(
            &GuideUiTextElement,
            &Node,
            (&TextFont, &LineHeight),
            &TextLayout,
            &LocalizedText,
        )>();
        let styled = styled.iter(world).collect::<Vec<_>>();
        assert_eq!(styled.len(), every_text_count);
        assert_eq!(
            styled
                .iter()
                .filter(|(element, ..)| element.role == GuideUiTextRole::CurrentGuide)
                .count(),
            4
        );
        assert_eq!(
            styled
                .iter()
                .filter(|(element, ..)| element.role == GuideUiTextRole::CommonCancel)
                .count(),
            3
        );

        for (element, node, font, layout, localized) in &styled {
            let (expected_font, expected_size, expected_line_height) = match element.role {
                GuideUiTextRole::SelectionHeading | GuideUiTextRole::PrimaryButton => {
                    (&jeffe, GUIDE_JEFFE_16_FONT_SIZE, GUIDE_JEFFE_16_LINE_HEIGHT)
                }
                GuideUiTextRole::SelectionIntro
                | GuideUiTextRole::MentorDescription(_)
                | GuideUiTextRole::WarpBody
                | GuideUiTextRole::ConfirmationBody => (
                    &chalet,
                    GUIDE_CHALET_SMALL_FONT_SIZE,
                    GUIDE_CHALET_SMALL_LINE_HEIGHT,
                ),
                _ => (&jeffe, GUIDE_JEFFE_14_FONT_SIZE, GUIDE_JEFFE_14_LINE_HEIGHT),
            };
            assert_eq!(
                &font.0.font,
                &bevy::text::FontSource::Handle(expected_font.clone()),
                "font for {:?}",
                element.role
            );
            assert_eq!(
                font.0.font_size.eval(Vec2::ZERO, 16.0),
                expected_size,
                "font size for {:?}",
                element.role
            );
            assert_eq!(
                (*font.1),
                LineHeight::Px(expected_line_height),
                "line height for {:?}",
                element.role
            );

            let expected_justify = match element.role {
                GuideUiTextRole::SelectionHeading
                | GuideUiTextRole::CurrentGuide
                | GuideUiTextRole::PrimaryButton
                | GuideUiTextRole::WarpButton
                | GuideUiTextRole::CommonCancel
                | GuideUiTextRole::CommonConfirm => Justify::Center,
                _ => Justify::Left,
            };
            assert_eq!(
                layout.justify, expected_justify,
                "alignment for {:?}",
                element.role
            );
            let expected_linebreak = match element.role {
                GuideUiTextRole::MentorName(_)
                | GuideUiTextRole::WarpButton
                | GuideUiTextRole::CommonCancel
                | GuideUiTextRole::CommonConfirm => LineBreak::NoWrap,
                _ => LineBreak::WordBoundary,
            };
            assert_eq!(
                layout.linebreak, expected_linebreak,
                "wrapping for {:?}",
                element.role
            );

            let expected_padding = match element.role {
                GuideUiTextRole::Computress => GUIDE_COMPUTRESS_TEXT_PADDING,
                GuideUiTextRole::SelectionHeading => GUIDE_BIG_FONT_TEXT_PADDING,
                GuideUiTextRole::MentorName(_) => GUIDE_CARD_TEXT_PADDING,
                GuideUiTextRole::CurrentGuide => GUIDE_CURRENT_TEXT_PADDING,
                GuideUiTextRole::Cost
                | GuideUiTextRole::WarpTitle
                | GuideUiTextRole::ConfirmationTitle => GUIDE_LABEL_TEXT_PADDING,
                _ => UiRect::ZERO,
            };
            assert_eq!(
                node.padding, expected_padding,
                "content padding for {:?}",
                element.role
            );

            let (expected_key, expected_fallback) = match element.role {
                GuideUiTextRole::Computress => ("ui.guide.computress", GUIDE_COMPUTRESS_LABEL),
                GuideUiTextRole::SelectionHeading => {
                    ("ui.guide.heading.choose", GUIDE_CHOOSE_HEADING)
                }
                GuideUiTextRole::SelectionIntro => ("ui.guide.intro.choose", GUIDE_CHOOSE_INTRO),
                GuideUiTextRole::MentorName(mentor) => (
                    GUIDE_MENTOR_NAME_LOCALIZATION_KEYS[mentor.slot()],
                    mentor.name(),
                ),
                GuideUiTextRole::MentorDescription(mentor) => (
                    match mentor {
                        GuideMentor::BenTennyson => "content.tabledata.guide.guide_string.9.sz_string",
                        GuideMentor::Dexter => "content.tabledata.guide.guide_string.7.sz_string",
                        GuideMentor::MojoJojo => "content.tabledata.guide.guide_string.8.sz_string",
                        GuideMentor::Edd => "content.tabledata.guide.guide_string.6.sz_string",
                    },
                    mentor.description(),
                ),
                GuideUiTextRole::CurrentGuide => ("ui.guide.current", GUIDE_CURRENT_LABEL),
                GuideUiTextRole::Cost => ("ui.guide.cost", "cost : {price}"),
                GuideUiTextRole::PrimaryButton => {
                    ("ui.guide.action.choose", GUIDE_CHOOSE_BUTTON_LABEL)
                }
                GuideUiTextRole::WarpTitle => ("ui.guide.warp.title", GUIDE_WARP_TITLE),
                GuideUiTextRole::WarpBody => ("ui.guide.warp.body", GUIDE_WARP_BODY),
                GuideUiTextRole::WarpButton => ("ui.guide.warp.action", GUIDE_WARP_BUTTON_LABEL),
                GuideUiTextRole::ConfirmationTitle => {
                    ("ui.guide.confirm.initial.title", GUIDE_CONFIRM_TITLE)
                }
                GuideUiTextRole::ConfirmationBody => (
                    "ui.guide.confirm.initial.body",
                    "Are you sure that you want to choose {mentor} as your guide?",
                ),
                GuideUiTextRole::CommonCancel => ("ui.common.cancel", GUIDE_CANCEL_LABEL),
                GuideUiTextRole::CommonConfirm => ("ui.common.confirm", GUIDE_CONFIRM_LABEL),
            };
            assert_eq!(localized.key, expected_key, "key for {:?}", element.role);
            assert_eq!(
                localized.fallback, expected_fallback,
                "fallback for {:?}",
                element.role
            );
            match element.role {
                GuideUiTextRole::MentorDescription(_) => assert!(localized.args.is_empty()),
                GuideUiTextRole::Cost => {
                    assert_eq!(localized.args.get("price").map(String::as_str), Some("0"));
                }
                GuideUiTextRole::ConfirmationBody => assert_eq!(
                    localized.args.get("mentor").map(String::as_str),
                    Some(GuideMentor::BenTennyson.name())
                ),
                _ => assert!(localized.args.is_empty(), "args for {:?}", element.role),
            }
        }

        let (_, heading_node, heading_font, heading_layout, heading_text) = styled
            .iter()
            .find(|(element, ..)| element.role == GuideUiTextRole::SelectionHeading)
            .copied()
            .unwrap();
        assert_eq!(heading_node.padding, GUIDE_BIG_FONT_TEXT_PADDING);
        assert_eq!(
            heading_font.0.font_size.eval(Vec2::ZERO, 16.0),
            GUIDE_JEFFE_16_FONT_SIZE
        );
        assert_eq!(
            (*heading_font.1),
            LineHeight::Px(GUIDE_JEFFE_16_LINE_HEIGHT)
        );
        assert_eq!(heading_layout.justify, Justify::Center);
        assert_eq!(heading_text.key, "ui.guide.heading.choose");

        let (_, _, description_font, description_layout, description_text) = styled
            .iter()
            .find(|(element, ..)| {
                element.role == GuideUiTextRole::MentorDescription(GuideMentor::BenTennyson)
            })
            .copied()
            .unwrap();
        assert_eq!(
            description_font.0.font_size.eval(Vec2::ZERO, 16.0),
            GUIDE_CHALET_SMALL_FONT_SIZE
        );
        assert_eq!(
            (*description_font.1),
            LineHeight::Px(GUIDE_CHALET_SMALL_LINE_HEIGHT)
        );
        assert_eq!(description_layout.justify, Justify::Left);
        assert_eq!(description_text.key, "content.tabledata.guide.guide_string.9.sz_string");
        assert_eq!(description_text.fallback, GuideMentor::BenTennyson.description());
        assert!(description_text.args.is_empty());

        let (_, cost_node, cost_font, _, cost_text) = styled
            .iter()
            .find(|(element, ..)| element.role == GuideUiTextRole::Cost)
            .copied()
            .unwrap();
        assert_eq!(cost_node.padding, GUIDE_LABEL_TEXT_PADDING);
        assert_eq!(
            cost_font.0.font_size.eval(Vec2::ZERO, 16.0),
            GUIDE_JEFFE_14_FONT_SIZE
        );
        assert_eq!((*cost_font.1), LineHeight::Px(GUIDE_JEFFE_14_LINE_HEIGHT));
        assert_eq!(cost_text.key, "ui.guide.cost");
        assert_eq!(cost_text.args.get("price").map(String::as_str), Some("0"));

        let mut heading =
            world.query_filtered::<(Entity, &ChildOf), With<GuideUiSelectionHeading>>();
        let (_, heading_parent) = heading.single(world).unwrap();
        let heading_parent = world.get::<Node>(heading_parent.parent()).unwrap();
        assert_eq!(heading_parent.top, px(GUIDE_HEADING_RECT.y));
        assert_eq!(heading_parent.height, px(GUIDE_HEADING_RECT.height));
        assert_eq!(heading_parent.align_items, AlignItems::Center);

        let mut intro = world.query_filtered::<&Node, With<GuideUiSelectionIntro>>();
        assert_eq!(intro.single(world).unwrap().top, px(GUIDE_INTRO_RECT.y));
        let mut modal_title = world.query::<(&GuideUiTextElement, &Node)>();
        for (_, node) in modal_title.iter(world).filter(|(element, _)| {
            matches!(
                element.role,
                GuideUiTextRole::WarpTitle | GuideUiTextRole::ConfirmationTitle
            )
        }) {
            assert_eq!(node.top, px(GUIDE_MODAL_TITLE_RECT.y));
            assert_eq!(node.height, px(GUIDE_MODAL_TITLE_RECT.height));
        }
        let mut modal_body = world.query::<(&GuideUiTextElement, &Node)>();
        for (_, node) in modal_body.iter(world).filter(|(element, _)| {
            matches!(
                element.role,
                GuideUiTextRole::WarpBody | GuideUiTextRole::ConfirmationBody
            )
        }) {
            assert_eq!(node.top, px(GUIDE_MODAL_BODY_RECT.y));
            assert_eq!(node.height, px(GUIDE_MODAL_BODY_RECT.height));
        }

        assert_eq!(GUIDE_JEFFE_14_SOURCE_FONT_PATH_ID, 903);
        assert_eq!(GUIDE_JEFFE_16_SOURCE_FONT_PATH_ID, 1_012);
        assert_eq!(GUIDE_CHALET_SMALL_SOURCE_FONT_PATH_ID, 1_018);
    }

    #[test]
    fn dynamic_selection_and_confirmation_copy_updates_semantic_templates_before_localization() {
        let asset_root = tempdir().unwrap();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin {
                file_path: asset_root.path().to_string_lossy().into_owned(),
                ..default()
            })
            .init_asset::<Image>()
            .init_asset::<Font>()
            .add_plugins(GuideUiPlugin);
        app.update();

        {
            let mut model = app.world_mut().resource_mut::<GuideUiModel>();
            model.open_change(GuideMentor::Dexter);
            model.selected = Some(GuideMentor::Edd);
            model.confirmation_open = true;
        }
        app.update();

        let world = app.world_mut();
        let mut texts = world.query::<(&GuideUiTextElement, &LocalizedText)>();
        let texts = texts.iter(world).collect::<Vec<_>>();
        let find = |role| {
            texts
                .iter()
                .find(|(element, _)| element.role == role)
                .map(|(_, text)| *text)
                .unwrap()
        };
        assert_eq!(
            find(GuideUiTextRole::SelectionHeading).key,
            "ui.guide.heading.change"
        );
        assert_eq!(
            find(GuideUiTextRole::SelectionIntro).key,
            "ui.guide.intro.change"
        );
        assert_eq!(
            find(GuideUiTextRole::PrimaryButton).key,
            "ui.guide.action.change"
        );
        assert_eq!(
            find(GuideUiTextRole::ConfirmationTitle).key,
            "ui.guide.confirm.change.title"
        );
        let body = find(GuideUiTextRole::ConfirmationBody);
        assert_eq!(body.key, "ui.guide.confirm.change.body");
        assert_eq!(body.args.get("mentor").map(String::as_str), Some("EDD"));
        assert!(body.fallback.contains("your currently have equipped"));
    }

    #[test]
    fn guide_card_descriptions_resolve_maintained_en_and_ru_content() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
        let (localization, ru) = crate::localization::Localization::open(&root, "ru").unwrap();
        let (_, en) = crate::localization::Localization::open(&root, "en").unwrap();
        for mentor in GuideMentor::CLEAN_ORDER {
            let text = super::guide_static_localized_text(GuideUiTextRole::MentorDescription(mentor), mentor.description());
            assert_eq!(localization.text(&en, &text), mentor.description());
            assert_ne!(localization.text(&ru, &text), mentor.description());
        }
    }
