use crate::upsell_ui::*;
use bevy::asset::AssetPlugin;
use tempfile::tempdir;

fn command(
    model: &mut UpsellUiModel,
    outbox: &mut UpsellUiOutbox,
    audio: &mut UpsellUiAudioOutbox,
    command: UpsellUiCommand,
) -> bool {
    apply_upsell_ui_command(model, outbox, audio, command)
}

#[test]
fn clean_mode_registration_and_retained_dead_reachability_are_explicit() {
    assert_eq!(
        [
            UpsellUiMode::Upgrade.source_index(),
            UpsellUiMode::NewsPayZone.source_index(),
            UpsellUiMode::NewsFreeZone.source_index(),
        ],
        [0, 1, 2]
    );
    assert_eq!(
        UpsellUiMode::from_source_index(3),
        None,
        "LeaveFuture has no registered mode index"
    );
    assert_eq!(
        UPSELL_UPGRADE_REACHABILITY,
        UpsellSourceReachability::RetainedExplicit
    );
    assert_eq!(
        UPSELL_LEAVE_FUTURE_REACHABILITY,
        UpsellSourceReachability::UnregisteredDead
    );
}

#[test]
fn serialized_and_initialized_upgrade_geometry_and_assets_are_exact() {
    assert_eq!(
        UPSELL_SERIALIZED_ADVERTIS_RECT,
        UpsellUiRect::new(300.0, 100.0, 92.0, 18.0)
    );
    let level_one = upsell_upgrade_geometry(1).unwrap();
    assert_eq!(level_one.dialog, UPSELL_UPGRADE_DIALOG_RECT);
    assert_eq!(level_one.close, UPSELL_SERIALIZED_CLOSE_RECT);
    assert_eq!(level_one.continue_playing, UPSELL_LEVEL_ONE_CANCEL_RECT);
    assert_eq!(level_one.get_upgrade, UPSELL_LEVEL_ONE_GET_RECT);
    assert_eq!(level_one.not_right_now, UPSELL_LEVEL_ONE_NOT_NOW_RECT);
    assert_eq!(level_one.advertis, UPSELL_LEVEL_ONE_ADVERTIS_RECT);

    for level in 2..=4 {
        let geometry = upsell_upgrade_geometry(level).unwrap();
        assert_eq!(geometry.continue_playing, UPSELL_OTHER_LEVEL_CANCEL_RECT);
        assert_eq!(geometry.get_upgrade, UPSELL_OTHER_LEVEL_GET_RECT);
        assert_eq!(geometry.not_right_now, UPSELL_OTHER_LEVEL_NOT_NOW_RECT);
        assert_eq!(geometry.advertis, UPSELL_OTHER_LEVEL_ADVERTIS_RECT);
    }
    assert_eq!(upsell_upgrade_geometry(0), None);
    assert_eq!(upsell_upgrade_geometry(5), None);
    assert_eq!(UPSELL_LEVEL_IMAGE_PATHS.len(), 4);
}

#[test]
fn centered_screen_pivot_layout_preserves_integer_source_rects_and_scale() {
    let upgrade = upsell_ui_layout(Vec2::new(1_264.0, 681.0), 1.0, UpsellUiMode::Upgrade);
    assert_eq!(upgrade.viewport, Vec2::new(1_264.0, 681.0));
    assert_eq!(upgrade.pivot, Vec2::new(632.0, 340.0));
    assert_eq!(
        upgrade.background.source,
        UpsellUiRect::new(-328.0, -379.0, 1_920.0, 1_440.0)
    );
    assert_eq!(
        upgrade.dialog.source,
        UpsellUiRect::new(76.0, 20.0, 1_112.0, 641.0)
    );
    assert_eq!(upgrade.dialog.source, upgrade.dialog.painted);

    let news = upsell_ui_layout(Vec2::new(1_264.0, 681.0), 1.0, UpsellUiMode::NewsFreeZone);
    assert_eq!(
        news.dialog.source,
        UpsellUiRect::new(116.0, 15.0, 1_032.0, 650.0)
    );

    let scale = clean_upsell_ui_scale(1_080.0);
    assert_eq!(scale, (1_080.0_f32 / 768.0) * 1.05);
    let scaled = upsell_ui_layout(Vec2::new(1_920.0, 1_080.0), scale, UpsellUiMode::Upgrade);
    assert_eq!(
        scaled.dialog.painted.center(),
        scaled.pivot + Vec2::new(0.0, -0.5 * scale)
    );
    assert_eq!(scaled.dialog.painted.width, 1_112.0 * scale);
    assert_eq!(scaled.dialog.painted.height, 641.0 * scale);
}

#[test]
fn receive_init_always_selects_news_from_pay_zone_without_local_fallback_or_audio() {
    for (is_pay_zone, expected) in [
        (true, UpsellUiMode::NewsPayZone),
        (false, UpsellUiMode::NewsFreeZone),
    ] {
        let mut model = UpsellUiModel::default();
        model.receive_init(1, is_pay_zone).unwrap();
        assert!(model.visible());
        assert_eq!(model.active_mode(), Some(expected));
        assert_eq!(model.news_page_count(), 0);
        assert_eq!(model.current_news_page_path(), None);
        assert_eq!(model.page_alpha(), 0.0);
    }
}

#[test]
fn invalid_levels_and_page_paths_fail_closed_without_mutating_state() {
    let mut model = UpsellUiModel::default();
    let before = model.clone();
    assert_eq!(
        model.receive_init(0, false),
        Err(UpsellUiError::InvalidLevel(0))
    );
    assert_eq!(model, before);
    assert_eq!(
        model.open_retained_upgrade(5),
        Err(UpsellUiError::InvalidLevel(5))
    );
    assert_eq!(model, before);

    model.receive_init(2, false).unwrap();
    model
        .set_news_page_paths(vec!["textures/page-a.png".to_owned()])
        .unwrap();
    let before = model.clone();
    assert_eq!(
        model.set_news_page_paths(vec!["../outside.png".to_owned()]),
        Err(UpsellUiError::InvalidNewsPagePath { index: 0 })
    );
    assert_eq!(model, before);

    model.open_retained_upgrade(2).unwrap();
    let before = model.clone();
    assert_eq!(
        model.set_news_page_paths(Vec::new()),
        Err(UpsellUiError::NotNewsMode)
    );
    assert_eq!(model, before);
}

#[test]
fn popup_and_help_disable_mouse_while_escape_keeps_its_clean_independent_gates() {
    let mut model = UpsellUiModel::default();
    model.receive_init(1, true).unwrap();
    model.set_escape_context(true, 0);
    assert!(model.input_boundary().mouse_controls_enabled);
    assert!(model.input_boundary().escape_dismiss_enabled);

    model.set_external_modes(false, true);
    assert!(!model.input_boundary().mouse_controls_enabled);
    assert!(
        model.input_boundary().escape_dismiss_enabled,
        "GameFrame.bHelp is not a direct cnUpsell Escape condition"
    );
    model.set_external_modes(true, false);
    assert!(!model.input_boundary().mouse_controls_enabled);
    assert!(!model.input_boundary().escape_dismiss_enabled);
    model.set_external_modes(false, false);
    model.set_escape_context(false, 0);
    assert!(!model.input_boundary().escape_dismiss_enabled);
    model.set_escape_context(true, 1);
    assert!(!model.input_boundary().escape_dismiss_enabled);
}

#[test]
fn escape_requires_both_external_event_results_and_emits_only_exit() {
    let mut model = UpsellUiModel::default();
    model.receive_init(1, false).unwrap();
    let mut outbox = UpsellUiOutbox::default();
    model.set_escape_context(false, 0);
    assert!(!dismiss_upsell_with_escape(&mut model, &mut outbox));
    assert!(model.visible());
    model.set_escape_context(true, 0);
    assert!(dismiss_upsell_with_escape(&mut model, &mut outbox));
    assert!(!model.visible());
    assert_eq!(outbox.pop_front(), Some(UpsellUiAction::Exit));
    assert!(outbox.is_empty());
}

#[test]
fn zero_or_single_news_page_continue_exits_without_explicit_sound() {
    for pages in [Vec::new(), vec!["textures/only-page.png".to_owned()]] {
        let mut model = UpsellUiModel::default();
        model.receive_init(1, false).unwrap();
        model.set_news_page_paths(pages).unwrap();
        let mut outbox = UpsellUiOutbox::default();
        let mut audio = UpsellUiAudioOutbox::default();
        assert!(command(
            &mut model,
            &mut outbox,
            &mut audio,
            UpsellUiCommand::Continue
        ));
        assert!(!model.visible());
        assert_eq!(outbox.pop_front(), Some(UpsellUiAction::Exit));
        assert!(audio.is_empty());
    }
}

#[test]
fn multipage_continue_uses_exact_audio_fade_and_wrap_exit_order() {
    let mut model = UpsellUiModel::default();
    model.receive_init(1, true).unwrap();
    model
        .set_news_page_paths(vec![
            "textures/page-0.png".to_owned(),
            "textures/page-1.png".to_owned(),
            "textures/page-2.png".to_owned(),
        ])
        .unwrap();
    let mut outbox = UpsellUiOutbox::default();
    let mut audio = UpsellUiAudioOutbox::default();

    assert!(command(
        &mut model,
        &mut outbox,
        &mut audio,
        UpsellUiCommand::Continue
    ));
    assert_eq!(model.news_page(), 1);
    assert_eq!(model.page_alpha(), 0.01);
    assert_eq!(audio.pop_front(), Some(UpsellUiAudioCue::ButtonSound));
    assert_eq!(audio.pop_front(), Some(UpsellUiAudioCue::ActionSuccess));
    model.advance_page_fade(0.25);
    assert_eq!(model.page_alpha(), 0.51);
    model.advance_page_fade(1.0);
    assert_eq!(model.page_alpha(), 1.0);

    assert!(command(
        &mut model,
        &mut outbox,
        &mut audio,
        UpsellUiCommand::Continue
    ));
    assert_eq!(model.news_page(), 2);
    assert_eq!(audio.pop_front(), Some(UpsellUiAudioCue::ButtonSound));
    assert_eq!(audio.pop_front(), Some(UpsellUiAudioCue::ActionSuccess));

    assert!(command(
        &mut model,
        &mut outbox,
        &mut audio,
        UpsellUiCommand::Continue
    ));
    assert!(!model.visible());
    assert_eq!(audio.pop_front(), Some(UpsellUiAudioCue::ButtonSound));
    assert!(audio.is_empty());
    assert_eq!(outbox.pop_front(), Some(UpsellUiAction::Exit));
    assert_eq!(
        UpsellUiAudioCue::ActionSuccess.exact_path(),
        Some(UPSELL_ACTION_SUCCESS_SOUND_PATH)
    );
    assert_eq!(UpsellUiAudioCue::ButtonSound.exact_path(), None);
}

#[test]
fn retained_upgrade_pay_page_stays_open_and_all_three_dismissals_exit_silently() {
    let mut model = UpsellUiModel::default();
    model.open_retained_upgrade(1).unwrap();
    let mut outbox = UpsellUiOutbox::default();
    let mut audio = UpsellUiAudioOutbox::default();
    assert!(command(
        &mut model,
        &mut outbox,
        &mut audio,
        UpsellUiCommand::OpenPayPage
    ));
    assert!(model.visible());
    assert_eq!(outbox.pop_front(), Some(UpsellUiAction::OpenPayPage));
    assert!(audio.is_empty());

    for dismissal in [
        UpsellUiCommand::Continue,
        UpsellUiCommand::NotRightNow,
        UpsellUiCommand::Close,
    ] {
        model.open_retained_upgrade(1).unwrap();
        assert!(command(&mut model, &mut outbox, &mut audio, dismissal));
        assert!(!model.visible());
        assert_eq!(outbox.pop_front(), Some(UpsellUiAction::Exit));
        assert!(audio.is_empty());
    }
}

#[test]
fn passive_button_boundary_tracks_mode_and_external_blockers() {
    let mut model = UpsellUiModel::default();
    assert!(!upsell_button_enabled(&model, UpsellUiButtonKind::Close));
    model.receive_init(1, false).unwrap();
    assert!(upsell_button_enabled(&model, UpsellUiButtonKind::Close));
    assert!(upsell_button_enabled(&model, UpsellUiButtonKind::Continue));
    assert!(!upsell_button_enabled(
        &model,
        UpsellUiButtonKind::GetUpgrade
    ));
    model.open_retained_upgrade(4).unwrap();
    assert!(upsell_button_enabled(
        &model,
        UpsellUiButtonKind::GetUpgrade
    ));
    assert!(upsell_button_enabled(
        &model,
        UpsellUiButtonKind::NotRightNow
    ));
    model.set_external_modes(true, false);
    assert!(!upsell_button_enabled(&model, UpsellUiButtonKind::Close));
}

#[test]
fn upsell_tree_spawns_only_real_labels_with_semantic_keys_and_exact_metrics() {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(UpsellUiPlugin);
    app.update();

    let world = app.world_mut();
    let mut labels = world.query::<(
        &UpsellUiButtonLabel,
        (&TextFont, &LineHeight),
        &LocalizedText,
    )>();
    let labels = labels.iter(world).collect::<Vec<_>>();
    assert_eq!(
        labels.len(),
        2,
        "image-only Close/Get must not spawn empty Text"
    );
    assert!(
        labels
            .iter()
            .all(|(_, _, localized)| !localized.key.is_empty())
    );
    let (_, font, localized) = labels
        .iter()
        .find(|(label, ..)| label.kind == UpsellUiButtonKind::Continue)
        .copied()
        .unwrap();
    assert_eq!(
        font.0.font_size.eval(Vec2::ZERO, 16.0),
        UPSELL_JEFFE_12_FONT_SIZE
    );
    assert_eq!((*font.1), LineHeight::Px(UPSELL_JEFFE_12_LINE_HEIGHT));
    assert_eq!(localized.key, "ui.common.continue");

    world
        .resource_mut::<UpsellUiModel>()
        .open_retained_upgrade(4)
        .unwrap();
    app.update();
    let world = app.world_mut();
    let mut localized = world.query::<(
        &UpsellUiButtonLabel,
        &LocalizedText,
        (&TextFont, &LineHeight),
    )>();
    let (_, continue_text, continue_font) = localized
        .iter(world)
        .find(|(label, ..)| label.kind == UpsellUiButtonKind::Continue)
        .unwrap();
    assert_eq!(continue_text.key, "ui.upsell.continue_playing");
    assert_eq!(
        continue_font.0.font_size.eval(Vec2::ZERO, 16.0),
        UPSELL_JEFFE_14_FONT_SIZE
    );
    assert_eq!(
        (*continue_font.1),
        LineHeight::Px(UPSELL_JEFFE_14_LINE_HEIGHT)
    );
}
