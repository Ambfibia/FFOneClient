use bevy::{asset::AssetPlugin, image::Image, text::Font};
use tempfile::tempdir;

use crate::localization::Localization;
use crate::resurrect_ui::*;

fn ready_context() -> ResurrectUiContext {
    ResurrectUiContext {
        ready_for_play: true,
        player_available: true,
        skill_icon_back_available: true,
        phoenix_group_icon_available: true,
        phoenix_self_icon_available: true,
        nearest_xcom_index: Some(17),
        ..default()
    }
}

fn entered_model() -> (ResurrectUiModel, ResurrectUiOutbox) {
    let mut model = ResurrectUiModel::default();
    let mut outbox = ResurrectUiOutbox::default();
    model.enter(0, &mut outbox);
    outbox.clear();
    (model, outbox)
}

#[test]
fn serialized_geometry_includes_the_clean_unused_item_rect_quirk() {
    assert_eq!(
        RESURRECT_WINDOW_RECT,
        ResurrectUiRect::new(0.0, 0.0, 520.0, 164.0)
    );
    assert_eq!(
        RESURRECT_PHOENIX_BUTTON_RECT,
        ResurrectUiRect::new(44.0, 124.0, 150.0, 25.0)
    );
    assert_eq!(
        RESURRECT_UNUSED_ITEM_BUTTON_RECT,
        ResurrectUiRect::new(209.0, 124.0, 150.0, 25.0)
    );
    assert_eq!(
        ResurrectChoice::UseItem.rect(),
        RESURRECT_PHOENIX_BUTTON_RECT
    );
    assert_ne!(
        ResurrectChoice::UseItem.rect(),
        RESURRECT_UNUSED_ITEM_BUTTON_RECT
    );
}

#[test]
fn clean_default_scale_and_center_pivot_layout_are_exact() {
    assert_eq!(clean_resurrect_ui_scale(720.0), 1.0);
    assert_eq!(clean_resurrect_ui_scale(768.0), 1.05);
    let layout = resurrect_ui_layout(Vec2::new(1280.0, 720.0), 1.0);
    assert_eq!(layout.dialog_left, 380.0);
    assert_eq!(layout.dialog_top, 278.0);
    assert_eq!(
        layout.visual_rect,
        ResurrectUiRect::new(380.0, 278.0, 520.0, 164.0)
    );
}

#[test]
fn invalid_scale_and_viewport_inputs_fail_to_safe_values() {
    assert_eq!(clean_resurrect_ui_scale(f32::NAN), 1.0);
    let layout = resurrect_ui_layout(Vec2::new(f32::NAN, -1.0), f32::NAN);
    assert_eq!(layout.viewport, Vec2::ZERO);
    assert_eq!(layout.scale, 1.0);
}

#[test]
fn group_phoenix_precedes_self_and_item_is_topmost() {
    let context = ResurrectUiContext {
        phoenix_group_skill: true,
        phoenix_self_skill_bit_16: true,
        resurrection_item_slot: Some(8),
        ..ready_context()
    };
    assert_eq!(
        context.choice_draw_order(),
        [
            Some(ResurrectChoice::NearestResurrectEm),
            Some(ResurrectChoice::PhoenixGroup),
            Some(ResurrectChoice::UseItem)
        ]
    );
    assert_eq!(
        context.topmost_choice_at(Vec2::new(60.0, 130.0)),
        Some(ResurrectChoice::UseItem)
    );
    assert!(!context.choice_is_visible(ResurrectChoice::PhoenixSelf));
}

#[test]
fn clean_master_gate_requires_the_group_icon_even_for_go() {
    let context = ResurrectUiContext {
        phoenix_group_icon_available: false,
        ..ready_context()
    };
    assert!(!context.clean_controls_gate_open());
    assert_eq!(context.choice_draw_order(), [None, None, None]);
}

#[test]
fn inventory_scan_keeps_the_last_matching_column_and_ignores_unknown_rows() {
    let slots = [
        ResurrectInventorySlot {
            column: 2,
            item_type: 7,
            general_item_type: Some(10),
            ..default()
        },
        ResurrectInventorySlot {
            column: 5,
            item_type: 7,
            general_item_type: None,
            ..default()
        },
        ResurrectInventorySlot {
            column: 9,
            item_type: 7,
            general_item_type: Some(10),
            ..default()
        },
    ];
    assert_eq!(clean_resurrection_item_slot(slots), Some(9));
}

#[test]
fn entry_code_22_latches_nano_restore_and_emits_exact_effects() {
    let mut model = ResurrectUiModel::default();
    let mut outbox = ResurrectUiOutbox::default();
    model.enter(22, &mut outbox);
    assert!(model.visible);
    assert!(model.nano_selection_was_open);
    assert_eq!(
        outbox.pop_front(),
        Some(ResurrectUiAction::Entered {
            effects: CLEAN_RESURRECT_ENTER_EFFECTS
        })
    );
}

#[test]
fn unresolved_xcom_fails_closed_without_latching_request_sent() {
    let (mut model, mut outbox) = entered_model();
    let context = ResurrectUiContext {
        nearest_xcom_index: None,
        ..ready_context()
    };
    assert_eq!(
        model.activate_choice(ResurrectChoice::NearestResurrectEm, context, &mut outbox),
        Err(ResurrectRequestBlocker::NearestXcomUnresolved)
    );
    assert!(!model.request_sent);
    assert!(outbox.is_empty());
}

#[test]
fn item_request_uses_type_six_eil_one_and_inventory_column() {
    let (mut model, mut outbox) = entered_model();
    let context = ResurrectUiContext {
        nearest_xcom_index: None,
        resurrection_item_slot: Some(12),
        ..ready_context()
    };
    let request = model
        .activate_choice(ResurrectChoice::UseItem, context, &mut outbox)
        .expect("known item slot should not require XCom");
    assert_eq!(
        request,
        ResurrectRegenRequest {
            choice: ResurrectChoice::UseItem,
            regen_type: 6,
            e_il: 1,
            index: 12,
            origin: ResurrectRequestOrigin::Manual,
        }
    );
    assert!(model.request_sent);
    assert_eq!(
        outbox.pop_front(),
        Some(ResurrectUiAction::RequestRegen(request))
    );
}

#[test]
fn manual_controls_obey_system_popup_but_timeout_does_not() {
    let (mut manual_model, mut manual_outbox) = entered_model();
    let context = ResurrectUiContext {
        system_popup_active: true,
        ..ready_context()
    };
    assert_eq!(
        manual_model.activate_choice(
            ResurrectChoice::NearestResurrectEm,
            context,
            &mut manual_outbox
        ),
        Err(ResurrectRequestBlocker::SystemPopup)
    );

    let (mut timeout_model, mut timeout_outbox) = entered_model();
    timeout_model.elapsed_seconds = RESURRECT_TIMEOUT_SECONDS;
    let request = timeout_model
        .advance(0.001, context, &mut timeout_outbox)
        .expect("clean Update is independent of GUI.enabled");
    assert_eq!(request.origin, ResurrectRequestOrigin::StrictTimeout);
    assert_eq!(request.regen_type, 1);
}

#[test]
fn repeated_dead_status_preserves_timeout_and_pending_request() {
    let (mut model, mut outbox) = entered_model();
    let context = ready_context();
    for _ in 0..60 {
        model.enter_from_authoritative_death(0, &mut outbox);
        assert!(model.advance(1.0, context, &mut outbox).is_none());
    }
    model.enter_from_authoritative_death(0, &mut outbox);
    assert!(model.advance(0.001, context, &mut outbox).is_some());
    for _ in 0..100 {
        model.enter_from_authoritative_death(0, &mut outbox);
        assert!(model.advance(1.0, context, &mut outbox).is_none());
        assert_eq!(model.countdown_seconds(), 0);
    }
    assert_eq!(outbox.len(), 1);
    assert!(model.request_sent);
    assert!(model.accept_regen_success(&mut outbox));
    model.enter_from_authoritative_death(22, &mut outbox);
    assert_eq!(model.elapsed_seconds, 0.0);
    assert!(!model.request_sent);
    assert!(model.nano_selection_was_open);
}

#[test]
fn timeout_comparison_is_strictly_greater_than_sixty() {
    let (mut model, mut outbox) = entered_model();
    let context = ready_context();
    assert_eq!(
        model.advance(RESURRECT_TIMEOUT_SECONDS, context, &mut outbox),
        None
    );
    assert!(!model.request_sent);
    let request = model
        .advance(0.001, context, &mut outbox)
        .expect("strict boundary should fire only after 60");
    assert_eq!(request.origin, ResurrectRequestOrigin::StrictTimeout);
}

#[test]
fn not_ready_resets_elapsed_like_rewriting_start_time_each_frame() {
    let (mut model, mut outbox) = entered_model();
    model.elapsed_seconds = 59.0;
    let context = ResurrectUiContext {
        ready_for_play: false,
        ..ready_context()
    };
    assert_eq!(model.advance(20.0, context, &mut outbox), None);
    assert_eq!(model.elapsed_seconds, 0.0);
}

#[test]
fn countdown_keeps_unity_floor_semantics_but_never_displays_negative_seconds() {
    let mut model = ResurrectUiModel {
        visible: true,
        elapsed_seconds: 59.1,
        ..default()
    };
    assert_eq!(model.countdown_seconds(), 0);
    model.elapsed_seconds = 60.001;
    assert_eq!(model.countdown_seconds(), 0);
    assert_eq!(
        ResurrectUiText::default().countdown(model.countdown_seconds()),
        "You will go automatically in : 0 seconds."
    );
}

#[test]
fn timeout_retries_unavailable_context_then_waits_for_one_authoritative_reply() {
    let (mut model, mut outbox) = entered_model();
    let unresolved = ResurrectUiContext {
        nearest_xcom_index: None,
        ..ready_context()
    };
    assert!(model.advance(61.0, unresolved, &mut outbox).is_none());
    assert!(!model.request_sent);
    assert_eq!(model.countdown_seconds(), 0);

    let resolved = ResurrectUiContext {
        nearest_xcom_index: Some(0),
        ..ready_context()
    };
    let request = model.advance(0.1, resolved, &mut outbox).unwrap();
    assert_eq!(request.index, 0);
    assert_eq!(request.regen_type, 1);
    assert_eq!(
        outbox.pop_front(),
        Some(ResurrectUiAction::RequestRegen(request))
    );
    for _ in 0..100 {
        assert!(model.advance(1.0, resolved, &mut outbox).is_none());
        assert_eq!(model.countdown_seconds(), 0);
    }
    assert!(outbox.is_empty());
    assert!(model.visible);
    assert!(model.accept_regen_success(&mut outbox));
    assert!(!model.visible);
}

#[test]
fn production_bundles_resolve_every_resurrect_key_in_english_and_russian() {
    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let (localization, mut language) = Localization::open(&asset_root, "en").unwrap();
    let copy = ResurrectUiText::default();
    let english = [
        (copy.title_localized(), "You have been defeated!"),
        (
            copy.question_localized(),
            "Would you like to proceed to the nearest Resurrect 'Em?",
        ),
        (
            copy.countdown_localized(42),
            "You will go automatically in : 42 seconds.",
        ),
        (
            copy.choice_localized(ResurrectChoice::NearestResurrectEm),
            "GO",
        ),
        (
            copy.choice_localized(ResurrectChoice::PhoenixSelf),
            "REVIVE",
        ),
        (copy.choice_localized(ResurrectChoice::UseItem), "USE ITEM"),
    ];
    for (localized, expected) in &english {
        assert_eq!(localization.text(&language, localized), *expected);
    }

    localization.select(&mut language, "ru");
    let russian = [
        (copy.title_localized(), "Вы потерпели поражение!"),
        (
            copy.question_localized(),
            "Переместиться к ближайшему «Воскресителю»?",
        ),
        (
            copy.countdown_localized(42),
            "Автоматическое перемещение через: 42 сек.",
        ),
        (
            copy.choice_localized(ResurrectChoice::NearestResurrectEm),
            "ВПЕРЁД",
        ),
        (
            copy.choice_localized(ResurrectChoice::PhoenixSelf),
            "ВОСКРЕСНУТЬ",
        ),
        (
            copy.choice_localized(ResurrectChoice::UseItem),
            "ИСП. ПРЕДМЕТ",
        ),
    ];
    for (localized, expected) in &russian {
        assert_eq!(localization.text(&language, localized), *expected);
    }
}

#[test]
fn every_spawned_resurrect_label_is_key_first_and_keeps_exact_source_style() {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(ResurrectUiPlugin);
    app.update();

    let world = app.world_mut();
    {
        let mut all_text = world.query_filtered::<
            (
                Entity,
                Option<&LocalizedText>,
                Option<&ResurrectTextStyle>,
            ),
            With<Text>,
        >();
        let mut count = 0;
        for (_, localized, style) in all_text.iter(world) {
            count += 1;
            assert!(
                localized.is_some(),
                "every user-visible resurrect Text entity must carry LocalizedText"
            );
            assert!(
                style.is_some(),
                "every user-visible resurrect Text entity must retain its clean GUIStyle"
            );
        }
        assert_eq!(count, 7);
    }

    {
        let mut body_text = world.query::<(
            &ResurrectTextNode,
            &ResurrectTextStyle,
            &Text,
            &LocalizedText,
            (&TextFont, &LineHeight),
            &TextLayout,
            &Node,
        )>();
        let rows = body_text.iter(world).collect::<Vec<_>>();
        assert_eq!(rows.len(), 3);
        for (role, style, text, localized, font, layout, node) in rows {
            let spec = style.spec();
            assert_eq!(font.0.font_size.eval(Vec2::ZERO, 16.0), spec.font_size);
            assert_eq!((*font.1), LineHeight::Px(spec.line_height));
            assert_eq!(layout.justify, Justify::Left);
            assert_eq!(layout.linebreak, LineBreak::WordBoundary);
            assert!(spec.word_wrap);
            assert_eq!(spec.y_offset, 0.0);
            match role.0 {
                ResurrectTextRole::Title => {
                    assert_eq!(*style, ResurrectTextStyle::Label);
                    assert_eq!(text.0, RESURRECT_TITLE);
                    assert_eq!(localized.key, RESURRECT_TITLE_LOCALIZATION_KEY);
                    assert_eq!(localized.fallback, RESURRECT_TITLE);
                    assert_eq!(node.align_self, AlignSelf::FlexStart);
                    assert_eq!(node.margin.left, px(RESURRECT_LABEL_MARGIN[0]));
                    assert_eq!(node.margin.right, px(RESURRECT_LABEL_MARGIN[1]));
                    assert_eq!(node.margin.top, px(0));
                    assert_eq!(node.margin.bottom, px(RESURRECT_LABEL_MARGIN[3]));
                    assert_eq!(node.padding.top, px(RESURRECT_LABEL_PADDING[2]));
                    assert_eq!(node.padding.bottom, px(RESURRECT_LABEL_PADDING[3]));
                }
                ResurrectTextRole::Question => {
                    assert_eq!(*style, ResurrectTextStyle::ImageWindow);
                    assert_eq!(text.0, RESURRECT_QUESTION);
                    assert_eq!(localized.key, RESURRECT_QUESTION_LOCALIZATION_KEY);
                    assert_eq!(localized.fallback, RESURRECT_QUESTION);
                }
                ResurrectTextRole::Countdown => {
                    assert_eq!(*style, ResurrectTextStyle::ImageWindow);
                    assert_eq!(text.0, "You will go automatically in : 60 seconds.");
                    assert_eq!(localized.key, RESURRECT_COUNTDOWN_LOCALIZATION_KEY);
                    assert_eq!(
                        localized.fallback,
                        "You will go automatically in : {seconds} seconds."
                    );
                    assert_eq!(
                        localized.args.get("seconds").map(String::as_str),
                        Some("60")
                    );
                }
            }
        }
    }

    {
        let mut button_text = world.query::<(
            &ResurrectUiButtonLabel,
            &ResurrectTextStyle,
            &Text,
            &LocalizedText,
            (&TextFont, &LineHeight),
            &TextLayout,
        )>();
        let rows = button_text.iter(world).collect::<Vec<_>>();
        assert_eq!(rows.len(), 4);
        assert_eq!(
            rows.iter()
                .filter(|(_, _, _, localized, _, _)| {
                    localized.key == RESURRECT_REVIVE_LOCALIZATION_KEY
                })
                .count(),
            2,
            "Phoenix group and Phoenix self share the clean REVIVE copy"
        );
        for (label, style, text, localized, font, layout) in rows {
            assert_eq!(*style, ResurrectTextStyle::Button);
            let spec = style.spec();
            assert_eq!(font.0.font_size.eval(Vec2::ZERO, 16.0), spec.font_size);
            assert_eq!((*font.1), LineHeight::Px(spec.line_height));
            assert_eq!(layout.justify, Justify::Center);
            assert_eq!(layout.linebreak, LineBreak::NoWrap);
            assert!(!spec.word_wrap);
            let (key, fallback) = match label.choice {
                ResurrectChoice::NearestResurrectEm => {
                    (RESURRECT_GO_LOCALIZATION_KEY, RESURRECT_GO_LABEL)
                }
                ResurrectChoice::PhoenixSelf | ResurrectChoice::PhoenixGroup => {
                    (RESURRECT_REVIVE_LOCALIZATION_KEY, RESURRECT_REVIVE_LABEL)
                }
                ResurrectChoice::UseItem => (
                    RESURRECT_USE_ITEM_LOCALIZATION_KEY,
                    RESURRECT_USE_ITEM_LABEL,
                ),
            };
            assert_eq!(text.0, fallback);
            assert_eq!(localized.key, key);
            assert_eq!(localized.fallback, fallback);
            assert!(localized.args.is_empty());
        }
    }

    {
        let mut buttons = world.query::<(&ResurrectUiButton, &ZIndex)>();
        let rows = buttons.iter(world).collect::<Vec<_>>();
        assert_eq!(rows.len(), 4);
        for (button, z_index) in rows {
            assert_eq!(
                z_index.0,
                if button.choice == ResurrectChoice::UseItem {
                    2
                } else {
                    0
                },
                "clean draws USE ITEM after the Phoenix icon pair"
            );
        }
        let mut visuals = world.query::<(&ResurrectPhoenixVisual, &ZIndex)>();
        let rows = visuals.iter(world).collect::<Vec<_>>();
        assert_eq!(rows.len(), 4);
        assert!(rows.iter().all(|(_, z_index)| z_index.0 == 1));
    }
}

#[test]
fn only_reached_resurrect_text_styles_are_published_with_exact_metrics() {
    assert_eq!(
        RESURRECT_SOURCE_UNITY_ENGINE_SHA256,
        "90EF121A97F954D35A50FB27F7DBCF99EAFFEAFCBCF938EAE9910C29F3745E0C"
    );
    let label = ResurrectTextStyle::Label.spec();
    assert_eq!(label.source_style, "label");
    assert_eq!(label.source_font_path_id, 903);
    assert_eq!(label.margin, [4.0; 4]);
    assert_eq!(label.padding, [0.0, 0.0, 3.0, 3.0]);
    assert!(!label.stretch_width);

    let image_window = ResurrectTextStyle::ImageWindow.spec();
    assert_eq!(image_window.source_style, "Imagewindow");
    assert_eq!(image_window.source_font_path_id, 1_018);
    assert_eq!(image_window.margin, [0.0; 4]);
    assert_eq!(image_window.padding, [0.0; 4]);
    assert!(image_window.stretch_width);

    let button = ResurrectTextStyle::Button.spec();
    assert_eq!(button.source_style, "button");
    assert_eq!(button.source_font_path_id, 903);
    assert_eq!(button.margin, [0.0; 4]);
    assert_eq!(button.padding, [0.0; 4]);
    assert!(!button.word_wrap);
    assert!(button.stretch_width);
}

#[test]
fn content_binding_updates_dynamic_localization_templates_without_changing_keys() {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(ResurrectUiPlugin);
    app.update();

    app.world_mut()
        .resource_mut::<ResurrectUiModel>()
        .elapsed_seconds = 18.0;
    {
        let mut copy = app.world_mut().resource_mut::<ResurrectUiText>();
        copy.countdown_prefix = "Fallback countdown".to_owned();
        copy.seconds = "ticks".to_owned();
        copy.go = "PROCEED".to_owned();
    }
    app.update();

    let world = app.world_mut();
    {
        let mut countdown = world.query::<(&ResurrectTextNode, &LocalizedText)>();
        let (_, localized) = countdown
            .iter(world)
            .find(|(role, _)| role.0 == ResurrectTextRole::Countdown)
            .expect("countdown text");
        assert_eq!(localized.key, RESURRECT_COUNTDOWN_LOCALIZATION_KEY);
        assert_eq!(localized.fallback, "Fallback countdown : {seconds} ticks.");
        assert_eq!(
            localized.args.get("seconds").map(String::as_str),
            Some("42")
        );
    }

    {
        let mut buttons = world.query::<(&ResurrectUiButtonLabel, &LocalizedText)>();
        let (_, localized) = buttons
            .iter(world)
            .find(|(label, _)| label.choice == ResurrectChoice::NearestResurrectEm)
            .expect("GO button text");
        assert_eq!(localized.key, RESURRECT_GO_LOCALIZATION_KEY);
        assert_eq!(localized.fallback, "PROCEED");
    }
}

#[test]
fn success_closes_mode_and_restores_nano_only_for_entry_22() {
    let mut model = ResurrectUiModel::default();
    let mut outbox = ResurrectUiOutbox::default();
    model.enter(22, &mut outbox);
    outbox.clear();
    assert!(model.accept_regen_success(&mut outbox));
    assert!(!model.visible);
    assert_eq!(
        outbox.pop_front(),
        Some(ResurrectUiAction::RegenSucceeded {
            effects: ResurrectSuccessEffects {
                grayscale_enabled: false,
                cursor_locked: true,
                return_to_normal_mode: true,
                restore_nano_selection: true,
            }
        })
    );
}

#[test]
fn texture_contracts_cover_every_source_owned_visual() {
    assert_eq!(RESURRECT_TEXTURE_CONTRACTS.len(), 9);
    assert!(RESURRECT_TEXTURE_CONTRACTS.iter().any(|asset| {
        asset.role == ResurrectTextureRole::Dialog
            && asset.source_path_id == Some(398)
            && asset.source_width == 110
            && asset.source_height == 164
    }));
    assert!(RESURRECT_TEXTURE_CONTRACTS.iter().any(|asset| {
        asset.role == ResurrectTextureRole::ResurrectEmIcon
            && asset.source_path_id == Some(405)
            && asset.source_width == 46
            && asset.source_height == 71
    }));
}

#[test]
fn plugin_startup_and_first_update_keep_one_hidden_passive_hierarchy() {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(ResurrectUiPlugin);
    app.update();

    assert!(app.world().contains_resource::<ResurrectUiModel>());
    assert!(app.world().contains_resource::<ResurrectUiContext>());
    assert!(app.world().contains_resource::<ResurrectUiOutbox>());
    let world = app.world_mut();
    let mut roots = world.query_filtered::<&Node, With<ResurrectUiRoot>>();
    assert_eq!(roots.iter(world).count(), 1);
    assert_eq!(roots.single(world).unwrap().display, Display::None);
    assert_eq!(roots.single(world).unwrap().overflow, Overflow::clip());
    let mut dialogs = world.query_filtered::<(&Node, &Children), With<ResurrectDialog>>();
    let (dialog, children) = dialogs.single(world).unwrap();
    assert_eq!(dialog.overflow, Overflow::visible());
    assert!(
        children.iter().any(|child| {
            world.get::<Node>(child).is_some_and(|node| {
                node.left == px(RESURRECT_GRIM_RECT.x)
                    && node.top == px(RESURRECT_GRIM_RECT.y)
                    && node.height == px(RESURRECT_GRIM_RECT.height)
            })
        }),
        "the complete Grim image must remain above the unclipped panel"
    );
}
