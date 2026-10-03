use bevy::{asset::AssetPlugin, input::keyboard::Key};
use tempfile::tempdir;

use crate::quit_menu_ui::*;

fn pressed_escape() -> KeyboardInput {
    KeyboardInput {
        key_code: KeyCode::Escape,
        logical_key: Key::Escape,
        state: ButtonState::Pressed,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    }
}

#[test]
fn cancel_visual_resolves_by_kind_after_logout_button_removal() {
    let mut app = App::new();
    let mut images = Assets::<Image>::default();
    let cancel_hover = images.add(Image::default());
    app.insert_resource(QuitMenuUiModel {
        visible: true,
        enabled: true,
        ..default()
    })
    .insert_resource(QuitMenuUiAssets {
        backdrop: default(),
        dialog: default(),
        button_normal: default(),
        button_hover: default(),
        button_active: None,
        cancel_normal: default(),
        cancel_hover: cancel_hover.clone(),
        cancel_active: None,
        font: default(),
    })
    .add_systems(Update, update_quit_menu_button_visuals);
    let button = app
        .world_mut()
        .spawn((
            Interaction::Hovered,
            QuitMenuButton {
                kind: QuitMenuButtonKind::Cancel,
            },
            ImageNode::default(),
            Pickable::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                QuitMenuButtonLabel {
                    kind: QuitMenuButtonKind::Cancel,
                },
                TextColor::default(),
            ));
        })
        .id();
    app.update();
    assert_eq!(
        app.world().get::<ImageNode>(button).unwrap().image,
        cancel_hover
    );
    assert_eq!(QUIT_MENU_BUTTONS.len(), 3);
    assert!(
        !QUIT_MENU_BUTTONS
            .iter()
            .any(|spec| spec.kind == QuitMenuButtonKind::QuitAndLogout)
    );
}

#[test]
fn clean_button_order_labels_geometry_and_styles_are_exact() {
    let expected = [
        (
            QuitMenuButtonKind::ChangeCharacter,
            "CHANGE CHARACTER",
            16.0,
            QuitMenuButtonVisual::Standard,
        ),
        (
            QuitMenuButtonKind::QuitGame,
            "QUIT GAME",
            69.0,
            QuitMenuButtonVisual::Standard,
        ),
        (
            QuitMenuButtonKind::Cancel,
            "CANCEL",
            122.0,
            QuitMenuButtonVisual::Cancel,
        ),
    ];
    for (spec, (kind, label, y, visual)) in QUIT_MENU_BUTTONS.iter().zip(expected) {
        assert_eq!(spec.kind, kind);
        assert_eq!(spec.label, label);
        assert_eq!(spec.visual, visual);
        assert_eq!(spec.rect, QuitMenuUiRect::new(13.0, y, 175.0, 45.0));
    }
    assert_eq!(
        QUIT_MENU_DIALOG_RECT,
        QuitMenuUiRect::new(0.0, 0.0, 206.0, 188.0)
    );
    assert_eq!(QUIT_MENU_BUTTON_VERTICAL_GAP, 8.0);
    assert_eq!(QUIT_MENU_BACKDROP_TEXTURE_PATH_ID, 535);
    assert_eq!(QUIT_MENU_DIALOG_TEXTURE_PATH_ID, 47);
    assert_eq!(QUIT_MENU_BUTTON_NORMAL_TEXTURE_PATH_ID, 449);
    assert_eq!(QUIT_MENU_BUTTON_HOVER_TEXTURE_PATH_ID, 122);
    assert_eq!(QUIT_MENU_CANCEL_NORMAL_TEXTURE_PATH_ID, 178);
    assert_eq!(QUIT_MENU_CANCEL_HOVER_TEXTURE_PATH_ID, 640);

    let standard = QuitMenuButtonVisual::Standard.text_style();
    assert_eq!(standard.source_style, "Button");
    assert_eq!(standard.source_font_path_id, 903);
    assert_eq!(standard.padding, [6.0, 6.0, 3.0, 3.0]);
    assert!(standard.word_wrap);
    assert!(standard.clips_text);
    assert_eq!(standard.y_offset, 0.0);

    let cancel = QuitMenuButtonVisual::Cancel.text_style();
    assert_eq!(cancel.source_style, "CancelButton");
    assert_eq!(cancel.source_font_path_id, 903);
    assert_eq!(cancel.padding, [0.0; 4]);
    assert!(!cancel.word_wrap);
    assert!(cancel.clips_text);
    assert_eq!(cancel.y_offset, 0.0);
}

#[test]
fn every_button_has_a_semantic_localization_key_at_spawn_time() {
    let keys = QUIT_MENU_BUTTONS
        .iter()
        .map(|spec| spec.localization_key)
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(keys.len(), QUIT_MENU_BUTTONS.len());
    assert!(keys.iter().all(|key| key.starts_with("ui.")));

    // The production-tree test below checks actual LocalizedText ownership.
}

#[test]
fn production_tree_is_key_first_and_preserves_each_used_gui_style() {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(QuitMenuUiPlugin);
    app.update();

    let world = app.world_mut();
    let mut query = world.query_filtered::<(
        &Text,
        &LocalizedText,
        &QuitMenuTextStyle,
        (&TextFont, &LineHeight),
        &TextLayout,
        &UiTransform,
    ), With<Text>>();
    let labels = query.iter(world).collect::<Vec<_>>();
    assert_eq!(labels.len(), 3);
    assert!(labels.iter().all(|(_, localized, _, _, _, _)| {
        !localized.key.is_empty() && localized.args.is_empty()
    }));

    for (_, _, style, font, layout, transform) in labels {
        let spec = style.0.text_style();
        assert_eq!(font.0.font_size.eval(Vec2::ZERO, 16.0), spec.font_size);
        assert_eq!((*font.1), LineHeight::Px(spec.line_height));
        assert_eq!(layout.justify, Justify::Center);
        assert_eq!(
            layout.linebreak,
            if spec.word_wrap {
                LineBreak::WordBoundary
            } else {
                LineBreak::NoWrap
            }
        );
        assert_eq!(transform.translation, Val2::px(0.0, spec.y_offset));
    }
}

#[test]
fn canonical_1264x681_layout_is_integer_centered_and_unscaled() {
    assert_eq!(clean_quit_menu_ui_scale(681.0), 1.0);
    let layout = quit_menu_ui_layout(Vec2::new(1_264.0, 681.0), 1.0);
    assert_eq!(layout.viewport, Vec2::new(1_264.0, 681.0));
    assert_eq!(layout.pivot, Vec2::new(632.0, 340.0));
    assert_eq!(layout.dialog.node_left, 529.0);
    assert_eq!(layout.dialog.node_top, 246.0);
    assert_eq!(
        layout.dialog.visual,
        QuitMenuUiRect::new(529.0, 246.0, 206.0, 188.0)
    );
    assert_eq!(
        layout.backdrop.visual,
        QuitMenuUiRect::new(0.0, 0.0, 1_264.0, 681.0)
    );
}

#[test]
fn high_resolution_layout_scales_about_legacy_integer_screen_center() {
    let scale = clean_quit_menu_ui_scale(1_080.0);
    assert_eq!(scale, (1_080.0_f32 / 768.0) * 1.05);
    let layout = quit_menu_ui_layout(Vec2::new(1_920.0, 1_080.0), scale);
    let source_center = Vec2::new(960.0, 540.0);
    let expected_center = layout.pivot + (source_center - layout.pivot) * scale;
    assert_eq!(layout.dialog.visual.center(), expected_center);
    assert_eq!(layout.dialog.visual.width, 206.0 * scale);
    assert_eq!(layout.dialog.visual.height, 188.0 * scale);
    assert_eq!(layout.dialog.scale, scale);
}

#[test]
fn invalid_or_small_scale_inputs_fail_closed_to_one() {
    for height in [f32::NAN, f32::NEG_INFINITY, -1.0, 0.0, 681.0] {
        assert_eq!(clean_quit_menu_ui_scale(height), 1.0);
    }
    let layout = quit_menu_ui_layout(Vec2::new(1_264.0, 681.0), f32::NAN);
    assert_eq!(layout.scale, 1.0);
}

#[test]
fn passive_boundary_reports_cursor_and_input_needs_without_owning_them() {
    let mut model = QuitMenuUiModel::default();
    assert_eq!(model.input_boundary(), QuitMenuInputBoundary::default());

    model.open();
    assert_eq!(
        model.input_boundary(),
        QuitMenuInputBoundary {
            blocks_lower_ui: true,
            blocks_gameplay_input: true,
            requires_pointer: true,
            mouse_controls_enabled: true,
            escape_dismiss_enabled: true,
        }
    );
    model.set_enabled(false);
    assert!(!model.input_boundary().mouse_controls_enabled);
    assert!(model.input_boundary().escape_dismiss_enabled);
    assert!(model.input_boundary().requires_pointer);
}

#[test]
fn disabled_model_rejects_mouse_activation_without_action_or_audio() {
    let mut model = QuitMenuUiModel {
        visible: true,
        enabled: false,
        ..default()
    };
    let mut outbox = QuitMenuUiOutbox::default();
    let mut audio = QuitMenuAudioOutbox::default();
    let mut sequence = QuitMenuClickSoundSequence::seeded(7);
    assert!(!activate_quit_menu_button(
        QuitMenuButtonKind::QuitGame,
        &mut model,
        &mut outbox,
        &mut audio,
        &mut sequence,
    ));
    assert!(model.visible);
    assert!(outbox.is_empty());
    assert!(audio.is_empty());
}

#[test]
fn enabled_mouse_activation_closes_first_and_emits_typed_action_and_click() {
    let mut model = QuitMenuUiModel {
        visible: true,
        enabled: true,
        ..default()
    };
    let mut outbox = QuitMenuUiOutbox::default();
    let mut audio = QuitMenuAudioOutbox::default();
    let mut sequence = QuitMenuClickSoundSequence::seeded(7);
    assert!(activate_quit_menu_button(
        QuitMenuButtonKind::QuitAndLogout,
        &mut model,
        &mut outbox,
        &mut audio,
        &mut sequence,
    ));
    assert!(!model.visible);
    assert_eq!(outbox.pop_front(), Some(QuitMenuUiAction::QuitAndLogout));
    let cue = audio.pop_front().expect("click cue");
    assert!(matches!(
        cue,
        QuitMenuAudioCue::ButtonClick {
            gain: QUIT_MENU_BUTTON_SOUND_GAIN,
            ..
        }
    ));
    assert!(QUIT_MENU_BUTTON_SOUND_PATHS.contains(&cue.path()));
}

#[test]
fn escape_dismisses_even_when_mouse_controls_are_disabled_and_has_no_click_cue() {
    let mut app = App::new();
    app.add_message::<KeyboardInput>()
        .init_resource::<QuitMenuUiModel>()
        .init_resource::<QuitMenuUiOutbox>()
        .add_systems(Update, handle_quit_menu_keyboard);
    {
        let mut model = app.world_mut().resource_mut::<QuitMenuUiModel>();
        model.open();
        model.set_enabled(false);
    }
    app.world_mut().write_message(pressed_escape());
    app.update();
    assert!(!app.world().resource::<QuitMenuUiModel>().visible);
    assert_eq!(
        app.world_mut()
            .resource_mut::<QuitMenuUiOutbox>()
            .pop_front(),
        Some(QuitMenuUiAction::Cancel {
            source: QuitMenuDismissalSource::EscapeKey,
        })
    );
}

#[test]
fn cancel_button_and_escape_remain_distinguishable_in_the_outbox() {
    let mut model = QuitMenuUiModel {
        visible: true,
        ..default()
    };
    let mut outbox = QuitMenuUiOutbox::default();
    let mut audio = QuitMenuAudioOutbox::default();
    let mut sequence = QuitMenuClickSoundSequence::seeded(11);
    assert!(activate_quit_menu_button(
        QuitMenuButtonKind::Cancel,
        &mut model,
        &mut outbox,
        &mut audio,
        &mut sequence,
    ));
    assert_eq!(
        outbox.pop_front(),
        Some(QuitMenuUiAction::Cancel {
            source: QuitMenuDismissalSource::CancelButton,
        })
    );

    model.open();
    assert!(dismiss_quit_menu_with_escape(&mut model, &mut outbox));
    assert_eq!(
        outbox.pop_front(),
        Some(QuitMenuUiAction::Cancel {
            source: QuitMenuDismissalSource::EscapeKey,
        })
    );
}

#[test]
fn click_sound_sequence_is_deterministic_bounded_and_keeps_exact_gain() {
    let mut left = QuitMenuClickSoundSequence::seeded(0x1234_5678);
    let mut right = QuitMenuClickSoundSequence::seeded(0x1234_5678);
    let left_values = (0..12).map(|_| left.next_clip_index()).collect::<Vec<_>>();
    let right_values = (0..12).map(|_| right.next_clip_index()).collect::<Vec<_>>();
    assert_eq!(left_values, right_values);
    assert!(left_values.iter().all(|index| *index < 5));
    assert!(
        left_values
            .iter()
            .copied()
            .collect::<std::collections::HashSet<_>>()
            .len()
            > 1
    );
    for clip_index in 0..5 {
        let cue = QuitMenuAudioCue::ButtonClick {
            clip_index,
            gain: QUIT_MENU_BUTTON_SOUND_GAIN,
        };
        assert_eq!(
            cue.path(),
            QUIT_MENU_BUTTON_SOUND_PATHS[usize::from(clip_index)]
        );
        assert_eq!(cue.gain(), 0.7);
    }
}

#[test]
fn visibility_transitions_emit_one_open_or_close_cue_without_mixer_coupling() {
    let mut app = App::new();
    app.init_resource::<QuitMenuUiModel>()
        .init_resource::<QuitMenuAudioOutbox>()
        .init_resource::<QuitMenuPresentationState>()
        .add_systems(Update, sync_quit_menu_visibility);

    app.world_mut().resource_mut::<QuitMenuUiModel>().open();
    app.update();
    assert_eq!(
        app.world_mut()
            .resource_mut::<QuitMenuAudioOutbox>()
            .pop_front(),
        Some(QuitMenuAudioCue::OpenScreen)
    );
    app.update();
    assert!(app.world().resource::<QuitMenuAudioOutbox>().is_empty());

    app.world_mut().resource_mut::<QuitMenuUiModel>().close();
    app.update();
    assert_eq!(
        app.world_mut()
            .resource_mut::<QuitMenuAudioOutbox>()
            .pop_front(),
        Some(QuitMenuAudioCue::CloseScreen)
    );
}

#[test]
fn serialized_asset_and_font_provenance_stays_explicitly_partial() {
    assert_eq!(QUIT_MENU_GAME_OBJECT_PATH_ID, 1_346);
    assert_eq!(QUIT_MENU_COMPONENT_PATH_ID, 1_418);
    assert_eq!(QUIT_MENU_SKIN_PATH_ID, 1_394);
    assert_eq!(QUIT_MENU_BUTTON_FONT_PATH_ID, 903);
    assert_eq!(QUIT_MENU_ROOT_FONT_PATH_ID, 10_102);
    assert_eq!(QUIT_MENU_PARITY_STATUS, "partial");
    assert!(QUIT_MENU_ROOT_FONT_CAVEAT.contains("no text"));
    assert!(QUIT_MENU_ROOT_FONT_CAVEAT.contains("JEFFE___14"));
    assert!(QUIT_MENU_ROOT_FONT_CAVEAT.contains("unused"));
}
