use super::*;

pub(super) fn cue_timeline(outbox: &mut OptionUiOutbox) -> Vec<OptionUiAudioCue> {
    let mut cues = Vec::new();
    while let Some(event) = outbox.pop_front() {
        if let OptionUiEvent::Audio(cue) = event {
            cues.push(cue);
        }
    }
    cues
}

#[test]
fn every_option_text_entity_has_semantic_ownership() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_systems(Startup, spawn_option_ui);
    app.update();

    let mut texts = app.world_mut().query::<(&Text, Option<&LocalizedText>)>();
    let rows = texts.iter(app.world()).collect::<Vec<_>>();
    assert!(!rows.is_empty());
    assert!(rows.iter().all(|(_, localized)| localized.is_some()));
}

#[test]
fn open_dropdown_list_covers_neighbouring_pulldowns_but_not_its_own() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .init_resource::<OptionUiModel>()
        .add_systems(Startup, spawn_option_ui)
        .add_systems(Update, bind_option_dropdowns);
    for kind in [OptionDropdownKind::Resolution, OptionDropdownKind::Texture] {
        assert!(
            app.world_mut()
                .resource_mut::<OptionUiModel>()
                .open_dropdown(kind)
        );
        app.update();
        let world = app.world_mut();
        let buttons = world
            .query::<(&OptionDropdownButton, &ZIndex)>()
            .iter(world)
            .map(|(button, z_index)| (button.0, z_index.0))
            .collect::<Vec<_>>();
        let (panel_z, panel_display) = world
            .query::<(&OptionDropdownPanel, &ZIndex, &Node)>()
            .iter(world)
            .find(|(panel, ..)| panel.0 == kind)
            .map(|(_, z_index, node)| (z_index.0, node.display))
            .unwrap();
        assert_eq!(panel_display, Display::Flex);
        for (button, z_index) in buttons {
            if button == kind {
                assert!(z_index > panel_z, "{kind:?} repaints its own pulldown");
            } else {
                assert!(z_index < panel_z, "{kind:?} list must cover {button:?}");
            }
        }
    }
    app.world_mut()
        .resource_mut::<OptionUiModel>()
        .close_dropdowns();
    app.update();
    let world = app.world_mut();
    assert!(
        world
            .query::<(&OptionDropdownButton, &ZIndex)>()
            .iter(world)
            .all(|(_, z_index)| z_index.0 == OPTION_DROPDOWN_BUTTON_Z_INDEX)
    );
}

#[test]
fn exact_shell_centers_at_both_required_viewports() {
    let compact = option_ui_layout(Vec2::new(1_264.0, 681.0), true);
    assert_eq!(compact.scale, Vec2::ONE);
    assert_eq!(compact.node_left, 122.0);
    assert_eq!(compact.node_top, 21.5);
    assert_eq!(
        compact.visual,
        OptionUiRect::new(122.0, 21.5, 1_020.0, 638.0)
    );

    let standard = option_ui_layout(Vec2::new(1_280.0, 720.0), true);
    assert_eq!(standard.scale, Vec2::ONE);
    assert_eq!(standard.node_left, 130.0);
    assert_eq!(standard.node_top, 41.0);
    assert_eq!(
        standard.visual,
        OptionUiRect::new(130.0, 41.0, 1_020.0, 638.0)
    );
}

#[test]
fn every_static_option_label_is_key_first_and_resolves_in_en_and_ru() {
    let (localization, mut language) = Localization::open(&asset_root(), "en").unwrap();
    let mut sources = HashSet::new();
    let mut keys = HashSet::new();
    for (source, key) in OPTION_LOCALIZED_SOURCE_KEYS {
        assert!(
            sources.insert(*source),
            "duplicate Option source {source:?}"
        );
        assert!(
            keys.insert(*key),
            "duplicate Option localization key {key:?}"
        );
        let localized = option_localized_text(source)
            .unwrap_or_else(|| panic!("Option source {source:?} has no localization key"));
        assert_eq!(localized.key, *key);
        assert_eq!(
            localization.text(&language, &localized),
            *source,
            "English fallback for {key:?} changed legacy copy"
        );
    }

    localization.select(&mut language, "ru");
    for (source, _) in OPTION_LOCALIZED_SOURCE_KEYS {
        let translated = localization.text(
            &language,
            &option_localized_text(source).expect("registered Option source"),
        );
        assert!(
            !translated.trim().is_empty(),
            "empty RU Option text for {source:?}"
        );
    }

    let translation_value =
        LocalizedText::new("ui.option.language.translation_value", "TEXT: {language}")
            .with_arg(
                "language",
                resolve_option_source(Some(&localization), Some(&language), "RUSSIAN"),
            );
    assert_eq!(
        localization.text(&language, &translation_value),
        "ТЕКСТ: РУССКИЙ"
    );
}

#[test]
fn graphics_presets_keep_the_clean_balanced_constructor_quirk() {
    let mut graphics = GraphicsSettings::default();
    assert!(graphics.soft_vegetation);
    graphics.apply_preset(GraphicsDetail::Balanced);
    assert!(!graphics.soft_vegetation);
    graphics.apply_preset(GraphicsDetail::BestPerformance);
    assert_eq!(graphics.visibility, 0.0);
    assert_eq!(graphics.particle_level, 0);
    assert!(!graphics.toon_shading);
    assert_eq!(graphics.texture, TextureQuality::Low);
    assert_eq!(graphics.shadow, ShadowQuality::PlayerOnly);
}

#[test]
fn game_ui_old_chat_is_inverted_and_owns_the_restart_modal_boundary() {
    let mut model = OptionUiModel::default();
    model.visible = true;
    model.selected_tab = OptionTab::GameUi;
    assert!(!model.display_value(OptionDisplayElement::OldChat));
    assert!(model.set_display_value(OptionDisplayElement::OldChat, true));
    assert!(!model.draft_options.display.new_chat);
    assert_eq!(model.popup, Some(OptionSystemPopup::OldChatRestartRequired));
    assert!(model.modal.system_popup);
    assert!(!model.chrome_enabled());
    assert!(model.acknowledge_popup());
    assert!(model.chrome_enabled());
    assert!(model.set_text_color(OptionTextColorChannel::General, 17));
    assert!(!model.set_text_color(OptionTextColorChannel::General, 18));
}

#[test]
fn controls_expose_all_39_actions_in_four_source_groups() {
    let grouped = OptionControlGroup::ALL
        .into_iter()
        .flat_map(OptionControlGroup::actions)
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(grouped.len(), OPTION_INPUT_ACTION_COUNT);
    assert_eq!(
        grouped.iter().copied().collect::<HashSet<_>>().len(),
        OPTION_INPUT_ACTION_COUNT
    );
    assert!(
        LegacyOptionAction::ALL
            .into_iter()
            .all(|action| grouped.contains(&action))
    );
}

#[test]
fn three_column_capture_mutates_draft_and_rejects_duplicate_bindings() {
    let mut model = OptionUiModel::default();
    model.visible = true;
    model.selected_tab = OptionTab::Controls;
    assert!(
        model
            .begin_binding_capture(LegacyOptionAction::Jump, OptionInputMappingSlot::Alternate)
    );
    assert!(model.submit_captured_binding(LegacyInputBinding::Key(LegacyPhysicalKey::H)));
    let jump = model
        .draft_input
        .mappings
        .iter()
        .find(|row| row.action == LegacyOptionAction::Jump)
        .expect("Jump row");
    assert_eq!(
        jump.alternate,
        LegacyInputBinding::Key(LegacyPhysicalKey::H)
    );
    assert!(
        model.begin_binding_capture(
            LegacyOptionAction::Nano1,
            OptionInputMappingSlot::Alternate
        )
    );
    assert!(!model.submit_captured_binding(LegacyInputBinding::Key(LegacyPhysicalKey::H)));
    assert_eq!(model.popup, Some(OptionSystemPopup::DuplicateInputBinding));

    model.popup = None;
    model.modal.system_popup = false;
    assert!(model.begin_binding_capture(LegacyOptionAction::Nano1, OptionInputMappingSlot::Primary));
    assert!(!model.submit_captured_binding(LegacyInputBinding::Key(LegacyPhysicalKey::H)));
    assert_eq!(model.popup, Some(OptionSystemPopup::DuplicateInputBinding));
}

#[test]
fn backspace_capture_clears_a_binding_so_its_key_can_be_reassigned() {
    let mut model = OptionUiModel::default();
    model.visible = true;
    model.selected_tab = OptionTab::Controls;
    assert!(model.begin_binding_capture(LegacyOptionAction::Skill2, OptionInputMappingSlot::Primary));
    let mut keyboard = ButtonInput::<KeyCode>::default();
    keyboard.press(KeyCode::Backspace);
    let mut app = App::new();
    app.insert_resource(model)
        .insert_resource(keyboard)
        .init_resource::<ButtonInput<MouseButton>>()
        .add_systems(Update, capture_option_binding_input);
    app.update();
    let model = app.world().resource::<OptionUiModel>();
    assert!(model.key_capture.is_none());
    assert_eq!(model.draft_input.mappings.iter()
        .find(|row| row.action == LegacyOptionAction::Skill2).unwrap().primary,
        LegacyInputBinding::Unbound);
}

#[test]
fn modal_matrix_disables_all_but_key_capture_is_controls_body_only() {
    for modal in [
        OptionModalState {
            system_popup: true,
            ..default()
        },
        OptionModalState {
            resolution_dropdown: true,
            ..default()
        },
        OptionModalState {
            detail_dropdown: true,
            ..default()
        },
        OptionModalState {
            texture_dropdown: true,
            ..default()
        },
        OptionModalState {
            shadow_dropdown: true,
            ..default()
        },
        OptionModalState {
            help: true,
            ..default()
        },
        OptionModalState {
            pad_dropdown: true,
            ..default()
        },
    ] {
        assert!(modal.disables_all());
        let model = OptionUiModel { modal, ..default() };
        assert!(!model.chrome_enabled());
        assert!(!model.page_body_enabled(OptionTab::Social));
    }

    let mut model = OptionUiModel::default();
    assert!(model.begin_key_capture(LegacyOptionAction::Jump));
    assert!(model.chrome_enabled());
    assert!(!model.page_body_enabled(OptionTab::Controls));
    assert!(model.page_body_enabled(OptionTab::Social));
    assert!(model.select_tab(OptionTab::Social));
    assert!(model.key_capture.is_some());
    model.tick_key_capture(OPTION_KEY_CAPTURE_TIMEOUT_SECONDS);
    assert!(model.key_capture.is_none());
}

#[test]
fn fifth_tab_reset_branch_is_explicitly_dead_for_typed_tabs() {
    assert!(legacy_raw_fifth_tab_reset_visible(4));
    assert!(!legacy_raw_fifth_tab_reset_visible(3));
    for tab in OptionTab::ALL {
        assert!(!tab.dead_reset_visible());
    }
    assert_eq!(
        OPTION_DEAD_RESET_RECT,
        OptionUiRect::new(435.0, 600.0, 160.0, 25.0)
    );
}

#[test]
fn blocked_projection_caps_fifty_and_uses_clean_name_fallback() {
    let mut slots = vec![OptionBuddySlot::default(); 51];
    slots[2] = OptionBuddySlot {
        pc_uid: 42,
        blocked: true,
        name_check_flag: 1,
        first_name: "Dexter".into(),
        last_name: "Morgan".into(),
    };
    slots[7] = OptionBuddySlot {
        pc_uid: 77,
        blocked: true,
        name_check_flag: 0,
        first_name: "Ignored".into(),
        last_name: "Name".into(),
    };
    slots[8] = OptionBuddySlot {
        pc_uid: 88,
        blocked: false,
        ..default()
    };
    slots[50] = OptionBuddySlot {
        pc_uid: 500,
        blocked: true,
        name_check_flag: 1,
        first_name: "Outside".into(),
        last_name: "Cap".into(),
    };
    assert_eq!(
        project_blocked_players(&slots),
        vec![
            BlockedPlayerRow {
                slot: 2,
                pc_uid: 42,
                display_name: "Dexter Morgan".into(),
            },
            BlockedPlayerRow {
                slot: 7,
                pc_uid: 77,
                display_name: "Player 77".into(),
            },
        ]
    );
}
