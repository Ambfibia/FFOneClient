use bevy::asset::AssetPlugin;

use crate::tutorial_cinematic_title::*;

#[test]
fn production_cinematic_title_is_translated_and_finishes_on_the_last_frame() {
    use crate::tutorial_choreography::{
        ChoreographyAction, FrameSequence, TUTORIAL_SCENE_CHOREOGRAPHIES,
    };
    let (key, final_count) = TUTORIAL_SCENE_CHOREOGRAPHIES
        .iter()
        .flat_map(|scene| scene.actions)
        .find_map(|timed| match timed.action {
            ChoreographyAction::Sequence(FrameSequence::SubtitleTypewriter {
                localization_key,
                canonical_english_chars,
                ..
            }) => Some((localization_key, canonical_english_chars)),
            _ => None,
        })
        .unwrap();
    assert_eq!(key, "tutorial.cinematic.tech_square_future");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    for (locale, expected) in [
        ("en", "TECH SQUARE.\nTHE FUTURE."),
        ("ru", "СКВЕР ТЕХНОЛОГИЙ.\nБУДУЩЕЕ."),
    ] {
        let (localization, language) = Localization::open(&root, locale).unwrap();
        let text = localization.text(
            &language,
            &LocalizedText::new(key, "TECH SQUARE.%sTHE FUTURE."),
        );
        assert_eq!(
            cinematic_title_view(Some(&text), final_count, 1.0, 720.0).text,
            expected
        );
    }
}

#[test]
fn normalizes_legacy_substitution_markers_before_reveal() {
    assert_eq!(
        normalize_subtitle("TECH SQUARE.%sTHE FUTURE."),
        "TECH SQUARE.\nTHE FUTURE."
    );
    assert_eq!(normalize_subtitle("A%sB%sC"), "A\nB\nC");
}

#[test]
fn reveal_count_is_unicode_character_count_not_byte_count() {
    assert_eq!(take_unicode_characters("Aé🧪Б", 0), "");
    assert_eq!(take_unicode_characters("Aé🧪Б", 3), "Aé🧪");
    assert_eq!(take_unicode_characters("Aé🧪Б", 99), "Aé🧪Б");
}

#[test]
fn computes_reference_screen_space_title_rect() {
    let rect = cinematic_title_rect(720.0);
    assert_eq!(rect.left, 50.0);
    assert_eq!(rect.width, 200.0);
    assert_eq!(rect.height, 100.0);
    assert!((rect.top - 507.147_34).abs() < 0.000_1, "{}", rect.top);
}

#[test]
fn view_normalizes_then_truncates_typewriter_text() {
    let view = cinematic_title_view(Some("TECH SQUARE.%sTHE FUTURE."), 13, 1.0, 720.0);
    assert_eq!(view.text, "TECH SQUARE.\n");
    assert!(view.visible);
    assert!(view.shadow_visible);
}

#[test]
fn visibility_and_shadow_follow_original_alpha_threshold() {
    assert!(!cinematic_title_view(None, 24, 1.0, 720.0).visible);
    assert!(!cinematic_title_view(Some("TITLE"), 0, 1.0, 720.0).visible);
    assert!(!cinematic_title_view(Some("TITLE"), 5, f32::NAN, 720.0).visible);

    let fading = cinematic_title_view(Some("TITLE"), 5, 0.9, 720.0);
    assert!(fading.visible);
    assert!(!fading.shadow_visible);
    assert_eq!(fading.alpha, 0.9);

    let opaque = cinematic_title_view(Some("TITLE"), 5, 0.900_001, 720.0);
    assert!(opaque.visible);
    assert!(opaque.shadow_visible);
}

#[test]
fn title_text_is_key_first_and_uses_the_path_1012_replacement_metrics() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .init_asset::<Font>()
        .add_systems(Startup, spawn_tutorial_cinematic_title);
    app.update();

    let mut texts = app.world_mut().query::<(
        &TutorialCinematicTitleText,
        &LocalizedText,
        (&TextFont, &LineHeight),
    )>();
    let rows = texts.iter(app.world()).collect::<Vec<_>>();
    assert_eq!(rows.len(), 2);
    for (_, localized, font) in rows {
        assert_eq!(localized.key, "ui.content.passthrough");
        assert_eq!(localized.fallback, "{text}");
        assert_eq!(localized.args.get("text").map(String::as_str), Some(""));
        assert_eq!(
            font.0.font_size.eval(Vec2::ZERO, 16.0),
            TUTORIAL_CINEMATIC_TITLE_FONT_SIZE
        );
        assert_eq!(
            (*font.1),
            LineHeight::Px(TUTORIAL_CINEMATIC_TITLE_LINE_HEIGHT)
        );
    }
}
