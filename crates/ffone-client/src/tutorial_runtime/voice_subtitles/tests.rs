use std::collections::BTreeSet;

use bevy::asset::AssetPlugin;

use crate::tutorial_voice_subtitles::*;

fn approximate(left: f32, right: f32) {
    assert!((left - right).abs() < 0.001, "{left} differs from {right}");
}

#[test]
fn mapping_is_unique_complete_and_case_insensitive() {
    assert_eq!(
        TUTORIAL_VOICE_SUBTITLE_SPECS.len(),
        TUTORIAL_VOICE_SUBTITLE_SPEC_COUNT
    );
    let cues = TUTORIAL_VOICE_SUBTITLE_SPECS
        .iter()
        .map(|spec| spec.cue.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    assert_eq!(cues.len(), TUTORIAL_VOICE_SUBTITLE_SPEC_COUNT);

    let lines = TUTORIAL_VOICE_SUBTITLE_SPECS
        .iter()
        .map(|spec| {
            assert_eq!(spec.event, 2);
            assert!(spec.max_seconds > 0.0);
            spec.line
        })
        .collect::<BTreeSet<_>>();
    let expected = (1..=91)
        .filter(|line| !TUTORIAL_VOICE_SUBTITLE_LINE_GAPS.contains(line))
        .collect::<BTreeSet<_>>();
    assert_eq!(lines, expected);
    assert_eq!(
        TUTORIAL_VOICE_SUBTITLE_SPECS
            .iter()
            .filter(|spec| spec.line == 91)
            .count(),
        1
    );
    assert_eq!(
        tutorial_voice_subtitle_spec("cOmPuTrEsS_TuT62")
            .unwrap()
            .line,
        91
    );
    assert!(tutorial_voice_subtitle_spec("NumTwo_Tut01").is_none());
}

#[test]
fn source_durations_and_literal_overrides_are_exact() {
    let expected = [
        ("Computress_tut01", 1, 5.0),
        ("Btrcup_Tut01", 2, 2.0),
        ("NumFive_TutFollowme_Rev", 3, 3.0),
        ("Ben_Tut01", 4, 3.0),
        ("Computress_Tut16", 20, 14.0),
        ("NumFive_TutFuseExpl", 26, 12.0),
        ("Btrcup_Tut08", 61, 12.0),
        ("Dexter_Tut03", 62, 11.0),
        ("Dexter_Tut10", 66, 17.0),
        ("Dexter_Tut12", 86, 24.0),
        ("Computress_Tut62", 91, 3.0),
    ];
    for (cue, line, seconds) in expected {
        let spec = tutorial_voice_subtitle_spec(cue).unwrap();
        assert_eq!(spec.line, line);
        assert_eq!(spec.max_seconds, seconds);
    }
    assert_eq!(
        tutorial_voice_subtitle_spec("Dexter_Tut10")
            .unwrap()
            .english_literal_override,
        Some(DEXTER_TUT10_ENGLISH_LITERAL)
    );
    assert_eq!(
        tutorial_voice_subtitle_spec("Dexter_Tut12")
            .unwrap()
            .english_literal_override,
        Some(DEXTER_TUT12_ENGLISH_LITERAL)
    );
    assert_eq!(
        TUTORIAL_VOICE_SUBTITLE_SPECS
            .iter()
            .filter(|spec| spec.english_literal_override.is_some())
            .count(),
        2
    );
}

#[test]
fn resolver_splits_only_first_colon_and_does_not_trim_dialogue() {
    let resolved = resolve_tutorial_voice_subtitle_with(
        "computress_tut01",
        TutorialLocaleBranch::OriginalEnglish,
        |_, _| Some("COMPUTRESS: first: second".to_owned()),
    )
    .unwrap()
    .unwrap();
    assert_eq!(resolved.cue, "Computress_tut01");
    assert_eq!(resolved.speaker, "COMPUTRESS:");
    assert_eq!(resolved.dialogue, " first: second");
    assert_eq!(resolved.full_text, "COMPUTRESS: first: second");
    assert!(!resolved.used_english_literal_override);

    let mut called = false;
    assert!(
        resolve_tutorial_voice_subtitle_with(
            "unknown-cue",
            TutorialLocaleBranch::OriginalEnglish,
            |_, _| {
                called = true;
                None
            },
        )
        .unwrap()
        .is_none()
    );
    assert!(!called);
}

#[test]
fn resolver_requires_mapped_text_and_separator_even_for_literal_override() {
    let missing = resolve_tutorial_voice_subtitle_with(
        "Dexter_Tut10",
        TutorialLocaleBranch::OriginalEnglish,
        |_, _| None,
    )
    .unwrap_err();
    assert!(matches!(
        missing,
        TutorialVoiceSubtitleError::MissingMappedSceneText {
            cue: "Dexter_Tut10",
            event: 2,
            line: 66
        }
    ));

    let malformed = resolve_tutorial_voice_subtitle_with(
        "Computress_Tut01",
        TutorialLocaleBranch::OriginalEnglish,
        |_, _| Some("no separator".to_owned()),
    )
    .unwrap_err();
    assert!(matches!(
        malformed,
        TutorialVoiceSubtitleError::MissingSpeakerSeparator { .. }
    ));

    let dexter10 = resolve_tutorial_voice_subtitle_with(
        "Dexter_Tut10",
        TutorialLocaleBranch::OriginalEnglish,
        |_, _| Some("DEXTER: truncated source".to_owned()),
    )
    .unwrap()
    .unwrap();
    assert_eq!(dexter10.full_text, DEXTER_TUT10_ENGLISH_LITERAL);
    assert!(dexter10.used_english_literal_override);

    let dexter12 = resolve_tutorial_voice_subtitle_with(
        "Dexter_Tut12",
        TutorialLocaleBranch::OriginalEnglish,
        |_, _| Some("DEXTER: truncated source".to_owned()),
    )
    .unwrap()
    .unwrap();
    assert_eq!(dexter12.full_text, DEXTER_TUT12_ENGLISH_LITERAL);
    assert!(dexter12.used_english_literal_override);

    let localized = resolve_tutorial_voice_subtitle_with(
        "Dexter_Tut10",
        TutorialLocaleBranch::Localized,
        |_, _| Some("DEXTER: localized source".to_owned()),
    )
    .unwrap()
    .unwrap();
    assert_eq!(localized.full_text, "DEXTER: localized source");
    assert!(!localized.used_english_literal_override);
}

#[test]
fn every_tutorial_voice_resolves_production_text_in_both_languages() {
    use crate::localization::{Localization, localized_tutorial_scene_text};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = crate::assets::AssetLocator::open(&root).unwrap();
    let content = TutorialMissionContent::open(&locator).unwrap();
    for locale in ["en", "ru"] {
        let (localization, language) = Localization::open(&root, locale).unwrap();
        for spec in TUTORIAL_VOICE_SUBTITLE_SPECS {
            let resolved = resolve_tutorial_voice_subtitle_with(
                spec.cue,
                TutorialLocaleBranch::Localized,
                |event, line| {
                    let fallback = content.scene_text(event, line).ok()?;
                    Some(localization.text(
                        &language,
                        &localized_tutorial_scene_text(event, line, fallback),
                    ))
                },
            )
            .unwrap()
            .unwrap();
            assert!(
                !resolved.dialogue.trim().is_empty(),
                "{} {locale}",
                spec.cue
            );
        }
    }
}
#[test]
fn state_clears_only_after_strict_source_timeout() {
    let mut state = TutorialVoiceSubtitleState::default();
    state
        .start_from_cue_with(
            "Computress_Tut01",
            TutorialLocaleBranch::OriginalEnglish,
            100.0,
            |_, _| Some("COMPUTRESS: exact".to_owned()),
        )
        .unwrap();
    assert_eq!(state.active().unwrap().resolved.dialogue, " exact");
    assert!(!state.tick(105.0).unwrap());
    assert!(state.active().is_some());
    assert!(state.tick(105.000_001).unwrap());
    assert!(state.active().is_none());

    state
        .start_from_cue_with(
            "Computress_Tut01",
            TutorialLocaleBranch::OriginalEnglish,
            200.0,
            |_, _| Some("COMPUTRESS: exact".to_owned()),
        )
        .unwrap();
    assert!(matches!(
        state.tick(199.0),
        Err(TutorialVoiceSubtitleError::NonMonotonicTime { .. })
    ));
    assert!(
        state
            .start_from_cue_with(
                "NumTwo_Tut01",
                TutorialLocaleBranch::OriginalEnglish,
                201.0,
                |_, _| unreachable!(),
            )
            .unwrap()
            .is_none()
    );
    assert!(state.active().is_none());
}

#[test]
fn localized_recording_retains_subtitle_until_audio_finishes() {
    let mut state = TutorialVoiceSubtitleState::default();
    state
        .start_from_cue_with(
            "Computress_Tut01",
            TutorialLocaleBranch::Localized,
            100.0,
            |_, _| Some("КОМПЬЮТРЕСС: Длинная реплика".to_owned()),
        )
        .unwrap();
    state.retain_until(115.0);
    assert!(!state.tick(110.0).unwrap());
    assert!(!state.tick(115.0).unwrap());
    assert!(state.tick(115.01).unwrap());
    state.retain_until(120.0);
    assert!(
        state.active().is_none(),
        "a finished or skipped subtitle cannot be resurrected"
    );
}

#[test]
fn translated_text_length_does_not_change_source_timeout() {
    let mut state = TutorialVoiceSubtitleState::default();
    state
        .start_from_cue_with(
            "Computress_Tut01",
            TutorialLocaleBranch::Localized,
            100.0,
            |_, _| Some(format!("COMPUTRESS: {}", "x".repeat(1800))),
        )
        .unwrap();
    assert!(!state.tick(105.0).unwrap());
    assert!(state.tick(105.01).unwrap());
}

#[test]
fn geometry_applies_squared_margin_and_scale_around_bottom_area_pivot() {
    let unscaled = TutorialVoiceSubtitleGeometry::for_viewport(800.0, 600.0).unwrap();
    approximate(unscaled.ui_scale, 1.0);
    approximate(unscaled.raw_area.x, 250.0);
    approximate(unscaled.raw_area.y, 505.956_12);
    approximate(unscaled.raw_area.width, 300.0);
    approximate(unscaled.raw_area.height, 94.043_89);
    approximate(unscaled.scaled_area.x, unscaled.raw_area.x);
    approximate(unscaled.scaled_area.y, unscaled.raw_area.y);
    approximate(unscaled.scaled_area.width, unscaled.raw_area.width);
    approximate(unscaled.scaled_area.height, unscaled.raw_area.height);

    let scaled = TutorialVoiceSubtitleGeometry::for_viewport(1024.0, 768.0).unwrap();
    approximate(scaled.ui_scale, 1.05);
    approximate(scaled.raw_area.x, 275.625);
    approximate(scaled.raw_area.width, 472.75);
    approximate(scaled.legacy_pivot.x, 512.0);
    approximate(scaled.legacy_pivot.y, scaled.raw_area.y);
    approximate(scaled.scaled_area.x, 263.806_24);
    approximate(scaled.scaled_area.y, scaled.raw_area.y);
    approximate(scaled.scaled_area.width, 496.387_48);
    approximate(
        scaled.scaled_area.height,
        scaled.raw_area.height * scaled.ui_scale,
    );
}

#[test]
fn visibility_restricts_voice_subtitles_to_active_cutscenes() {
    let mut context = TutorialVoiceSubtitleUiContext {
        event_scene: true,
        scene: Some(1),
        cinematic: true,
        cinematic_alpha: 0.9,
        locale: TutorialLocaleBranch::Localized,
    };
    assert_eq!(
        tutorial_voice_subtitle_visibility(true, &context),
        TutorialVoiceSubtitleVisibility {
            subtitle: false,
            skip_label: true
        }
    );
    context.cinematic_alpha = 0.900_001;
    assert!(tutorial_voice_subtitle_visibility(true, &context).subtitle);
    assert!(!tutorial_voice_subtitle_visibility(false, &context).subtitle);
    assert!(tutorial_voice_subtitle_visibility(false, &context).skip_label);

    context.scene = None;
    assert_eq!(
        tutorial_voice_subtitle_visibility(true, &context),
        TutorialVoiceSubtitleVisibility::default()
    );

    context.event_scene = false;
    context.scene = Some(1);
    context.cinematic = true;
    context.locale = TutorialLocaleBranch::OriginalEnglish;
    assert!(!tutorial_voice_subtitle_visibility(true, &context).subtitle);
    context.locale = TutorialLocaleBranch::Localized;
    assert!(
        !tutorial_voice_subtitle_visibility(true, &context).subtitle,
        "localized voice text must not leak into interactive tutorial stages"
    );
    assert!(!tutorial_voice_subtitle_visibility(true, &context).skip_label);
    // Interactive Computress/Numbuh Two VoiceOut: no scene and no bars.
    context.scene = None;
    context.cinematic = false;
    context.cinematic_alpha = 0.0;
    assert_eq!(
        tutorial_voice_subtitle_visibility(true, &context),
        TutorialVoiceSubtitleVisibility::default()
    );

    assert!(is_english_locale("en-US"));
    assert!(is_english_locale("EN_gb"));
    assert!(!is_english_locale("ru-RU"));
}

#[test]
fn voice_subtitles_sort_after_cinematic_bars_and_location_title() {
    // Native choreography uses 1901 for the bars and the independently
    // rendered location title uses 1902, matching their source OnGUI call
    // order immediately before VOSubTitle.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .init_asset::<Font>()
        .add_plugins(TutorialVoiceSubtitlePlugin);
    app.update();

    let mut roots = app
        .world_mut()
        .query::<(&TutorialVoiceSubtitleUiElement, &GlobalZIndex)>();
    let (element, global) = roots.single(app.world()).unwrap();
    assert_eq!(*element, TutorialVoiceSubtitleUiElement::Root);
    assert_eq!(*global, GlobalZIndex(1_903));
}

#[test]
fn every_voice_subtitle_text_entity_is_key_first() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .init_asset::<Font>()
        .add_plugins(TutorialVoiceSubtitlePlugin);
    app.update();

    let mut text_count = app.world_mut().query::<&Text>();
    assert_eq!(text_count.iter(app.world()).count(), 3);

    let mut localized = app
        .world_mut()
        .query::<(&TutorialVoiceSubtitleUiElement, &LocalizedText)>();
    let rows = localized.iter(app.world()).collect::<Vec<_>>();
    assert_eq!(rows.len(), 3);
    for (element, localized) in rows {
        match element {
            TutorialVoiceSubtitleUiElement::Speaker
            | TutorialVoiceSubtitleUiElement::Dialogue => {
                assert_eq!(localized.key, "ui.content.passthrough");
                assert_eq!(localized.fallback, "{text}");
                assert_eq!(localized.args.get("text").map(String::as_str), Some(""));
            }
            TutorialVoiceSubtitleUiElement::SkipLabelText => {
                assert_eq!(localized.key, "ui.tutorial.skip");
                assert_eq!(localized.fallback, TUTORIAL_SKIP_LABEL);
            }
            other => panic!("unexpected localized subtitle element: {other:?}"),
        }
    }
}

#[test]
fn gui_style_contracts_match_fusionfall_system_message_skin() {
    assert_eq!(
        TUTORIAL_VOICE_SUBTITLE_CENTERBOX_STYLE,
        TutorialVoiceSubtitleStyleContract {
            legacy_name: "centerbox",
            source_font_name: "JEFFE___14",
            source_font_path_id: 903,
            source_nominal_raster_size: 14.0,
            source_line_spacing: 13.710_000_038_146_973,
            semantic_font_path: TUTORIAL_VOICE_SUBTITLE_JEFFE_FONT_PATH,
            semantic_font_size: 12.0,
            normal_text_color: [1.0, 1.0, 1.0, 1.0],
            normal_background_path_id: 0,
            alignment: 4,
            border: TutorialVoiceSubtitleInsets::new(1, 0, 0, 0),
            margin: TutorialVoiceSubtitleInsets::new(0, 0, 0, 0),
            padding: TutorialVoiceSubtitleInsets::new(0, 0, 0, 0),
            image_position: TutorialVoiceSubtitleImagePosition::ImageAbove,
            word_wrap: true,
            clipping: TutorialVoiceSubtitleTextClipping::Clip,
            stretch_width: true,
            stretch_height: false,
            fixed_width: 0.0,
            fixed_height: 0.0,
        }
    );

    for (style, expected_name, expected_font, expected_path_id, expected_size, clipping) in [
        (
            TUTORIAL_VOICE_SUBTITLE_SMALLFONT_STYLE,
            "smallfont",
            "ChaletBook-Regular Small",
            1018,
            12.0,
            TutorialVoiceSubtitleTextClipping::Clip,
        ),
        (
            TUTORIAL_VOICE_SUBTITLE_SMALLFONT2_STYLE,
            "smallfont2",
            "ChaletBook-Regular",
            1115,
            14.0,
            TutorialVoiceSubtitleTextClipping::Overflow,
        ),
    ] {
        assert_eq!(style.legacy_name, expected_name);
        assert_eq!(style.source_font_name, expected_font);
        assert_eq!(style.source_font_path_id, expected_path_id);
        assert_eq!(style.source_nominal_raster_size, expected_size);
        assert_eq!(style.semantic_font_size, expected_size);
        assert_eq!(
            style.semantic_font_path,
            TUTORIAL_VOICE_SUBTITLE_CHALET_FONT_PATH
        );
        assert_eq!(style.normal_text_color, [1.0, 0.995_967_75, 1.0, 1.0]);
        assert_eq!(style.normal_background_path_id, 0);
        assert_eq!(style.alignment, 4);
        assert_eq!(style.border, TutorialVoiceSubtitleInsets::new(5, 5, 5, 5));
        assert_eq!(style.margin, TutorialVoiceSubtitleInsets::new(4, 4, 4, 4));
        assert_eq!(style.padding, TutorialVoiceSubtitleInsets::new(10, 6, 4, 6));
        assert_eq!(
            style.image_position,
            TutorialVoiceSubtitleImagePosition::TextOnly
        );
        assert!(style.word_wrap);
        assert_eq!(style.clipping, clipping);
        assert!(style.stretch_width);
        assert!(!style.stretch_height);
        assert_eq!((style.fixed_width, style.fixed_height), (0.0, 0.0));
    }

    approximate(
        TUTORIAL_VOICE_SUBTITLE_SMALLFONT_STYLE.source_line_spacing,
        12.071_999_55,
    );
    approximate(
        TUTORIAL_VOICE_SUBTITLE_SMALLFONT2_STYLE.source_line_spacing,
        14.083_999_63,
    );
}

#[test]
fn semantic_font_assets_used_by_the_recovered_styles_are_published() {
    let game_assets =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    for path in [
        TUTORIAL_VOICE_SUBTITLE_JEFFE_FONT_PATH,
        TUTORIAL_VOICE_SUBTITLE_CHALET_FONT_PATH,
    ] {
        assert!(
            game_assets.join(path).is_file(),
            "missing semantic font {path}"
        );
    }
}

#[test]
fn legacy_gui_matrix_scales_font_metrics_and_insets_with_the_area() {
    let scale = TutorialVoiceSubtitleGeometry::for_viewport(1024.0, 768.0)
        .unwrap()
        .ui_scale;
    approximate(scale, 1.05);
    let margin = TUTORIAL_VOICE_SUBTITLE_SMALLFONT2_STYLE
        .margin
        .scaled_ui_rect(scale);
    let Val::Px(margin_left) = margin.left else {
        panic!("expected pixel margin");
    };
    let Val::Px(margin_right) = margin.right else {
        panic!("expected pixel margin");
    };
    approximate(margin_left, 4.2);
    approximate(margin_right, 4.2);
    let padding = TUTORIAL_VOICE_SUBTITLE_SMALLFONT2_STYLE
        .padding
        .scaled_ui_rect(scale);
    let Val::Px(padding_left) = padding.left else {
        panic!("expected pixel padding");
    };
    let Val::Px(padding_right) = padding.right else {
        panic!("expected pixel padding");
    };
    approximate(padding_left, 10.5);
    approximate(padding_right, 6.3);
    approximate(
        TUTORIAL_VOICE_SUBTITLE_SMALLFONT2_STYLE.semantic_font_size * scale,
        14.7,
    );
    approximate(
        TUTORIAL_VOICE_SUBTITLE_SMALLFONT_STYLE.source_line_spacing * scale,
        12.675_599,
    );
}
