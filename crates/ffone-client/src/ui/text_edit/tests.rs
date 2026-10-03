use super::*;

fn shape(text: &str, scale: f32) -> parley::Layout<()> {
    use parley::{FontContext, LayoutContext, StyleProperty};
    let mut fonts = FontContext::new();
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/game/fonts/jeffe.otf"
    ))
    .unwrap();
    fonts.collection.register_fonts(
        bytes.into(),
        Some(parley::fontique::FontInfoOverride {
            family_name: Some("Fixture"),
            ..default()
        }),
    );
    let mut context = LayoutContext::new();
    let mut builder = context.ranged_builder(&mut fonts, text, scale, true);
    builder.push_default(StyleProperty::FontFamily("Fixture".into()));
    builder.push_default(StyleProperty::FontSize(20.0));
    let mut layout = builder.build(text);
    layout.break_all_lines(None);
    layout
}

#[test]
fn shaped_scalar_boundaries_include_cyrillic_spaces_and_scaled_advances() {
    for text in ["office space", "Привет мир"] {
        let layout = shape(text, 2.0);
        let positions = layout_advances(&layout, text, 2.0);
        assert_eq!(positions.len(), text.chars().count() + 1);
        assert!(positions.windows(2).all(|pair| pair[0] <= pair[1]));
        assert!(positions.last().unwrap() > &0.0);
        assert!((positions.last().unwrap() * 2.0 - layout.width()).abs() < 0.01);
        for (index, x) in positions.iter().enumerate() {
            assert_eq!(nearest(&positions, *x), index);
        }
    }
}

#[test]
fn scrolling_recovers_after_initial_layout_resize_and_shorter_replacement() {
    assert_eq!(scroll_to_cursor(42.0, 43.0, 43.0, 270.0), 0.0);
    assert_eq!(scroll_to_cursor(0.0, 300.0, 300.0, 100.0), 200.0);
    assert_eq!(scroll_to_cursor(200.0, 40.0, 300.0, 100.0), 40.0);
    assert_eq!(scroll_to_cursor(200.0, 20.0, 20.0, 100.0), 0.0);
}

#[test]
fn selection_replacement_and_unicode_navigation() {
    let mut text = "Привет😀!".to_owned();
    let mut edit = TextEdit::default();
    edit.key(&mut text, KeyCode::ArrowLeft, false, false);
    edit.key(&mut text, KeyCode::ArrowLeft, false, true);
    edit.insert(&mut text, "ёж", 64, true);
    assert_eq!(text, "Приветёж!");
    edit.key(&mut text, KeyCode::KeyA, true, false);
    edit.insert(&mut text, "новый", 32, false);
    assert_eq!(text, "новый");
    edit.key(&mut text, KeyCode::Home, false, false);
    edit.key(&mut text, KeyCode::Delete, false, false);
    assert_eq!(text, "овый");
}

#[test]
fn selected_text_releases_utf16_capacity_and_controls_do_not_erase() {
    let mut text = "😀abc".to_owned();
    let mut edit = TextEdit::default();
    edit.key(&mut text, KeyCode::KeyA, true, false);
    edit.insert(&mut text, "\u{1}", 5, true);
    assert_eq!(text, "😀abc");
    edit.insert(&mut text, "😀😀😀", 5, true);
    assert_eq!(text, "😀😀");
    edit.key(&mut text, KeyCode::KeyA, true, false);
    edit.key(&mut text, KeyCode::Backspace, false, false);
    assert!(text.is_empty());
}
