use super::*;

pub(super) fn audit_visible_text(
    all_texts: &Query<'_, '_, &Text>,
    styled_texts: &Query<
        '_,
        '_,
        (
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
        ),
    >,
    computed_nodes: &Query<'_, '_, &ComputedNode>,
    assets: &PreviewAssets,
    scene: PreviewScene,
    language: &str,
) -> Result<String, String> {
    let all_count = all_texts.iter().count();
    let styled_count = styled_texts.iter().count();
    if all_count == 0 || styled_count != all_count {
        return Err(format!(
            "every-Text source ownership incomplete: styled={styled_count}, all={all_count}"
        ));
    }

    let mut visible_count = 0usize;
    let mut glyph_count = 0usize;
    let mut line_count = 0usize;
    let mut fitted_count = 0usize;
    let mut visible_styles = Vec::new();
    let mut saw_cyrillic = false;
    let mut glyph_min_y = f32::INFINITY;
    let mut glyph_max_y = f32::NEG_INFINITY;

    for (entity, text, localized, style, font, layout, computed, transform, parent, role) in styled_texts
    {
        if localized.key.trim().is_empty() {
            return Err(format!("Text {entity:?} has an empty semantic key"));
        }
        let (font_index, expected_size, expected_line_height) = expected_text_metrics(*style);
        if font.0.font != bevy::text::FontSource::Handle(assets.fonts[font_index].clone()) {
            return Err(format!(
                "{} uses the wrong replacement font for {style:?}",
                localized.key
            ));
        }
        let minimum_size = if role.is_some_and(|element| element.role == EmailUiTextRole::DetailBody) {
            expected_size // Long mail must scroll at its source font size.
        } else { (expected_size * 0.7).max(6.0).min(expected_size) };
        if font.0.font_size.eval(Vec2::ZERO, 16.0) > expected_size + f32::EPSILON
            || font.0.font_size.eval(Vec2::ZERO, 16.0) < minimum_size - f32::EPSILON
        {
            return Err(format!(
                "{} font size {} is outside {}..={} for {style:?}",
                localized.key,
                font.0.font_size.eval(Vec2::ZERO, 16.0),
                minimum_size,
                expected_size
            ));
        }
        if font.0.font_size.eval(Vec2::ZERO, 16.0) + f32::EPSILON < expected_size {
            fitted_count += 1;
        }
        let current_line_height =
            expected_line_height * font.0.font_size.eval(Vec2::ZERO, 16.0) / expected_size;
        if (*font.1) != LineHeight::Px(current_line_height) {
            return Err(format!(
                "{} line height {:?} != {} for {style:?}",
                localized.key,
                (*font.1),
                current_line_height
            ));
        }
        if transform.translation.y != px(style.y_offset()) {
            return Err(format!(
                "{} replacement baseline offset {:?} != {} for {style:?}",
                localized.key,
                transform.translation.y,
                style.y_offset()
            ));
        }

        if computed.size().x <= 0.0
            || computed.size().y <= 0.0
            || text.0.is_empty()
            || layout.glyphs.is_empty()
        {
            continue;
        }
        visible_count += 1;
        glyph_count += layout.glyphs.len();
        line_count += layout.run_geometry.len();
        saw_cyrillic |= text.0.chars().any(|character| {
            ('\u{0400}'..='\u{052f}').contains(&character)
                || ('\u{2de0}'..='\u{2dff}').contains(&character)
        });
        if !visible_styles.contains(style) {
            visible_styles.push(*style);
        }

        let parent_computed = computed_nodes
            .get(parent.parent())
            .map_err(|_| format!("{} Text has no computed source-Rect parent", localized.key))?;
        if computed.size().x > parent_computed.size().x + 1.0
            || (computed.size().y > parent_computed.size().y + 1.0 && !role.is_some_and(|r|r.role==email_ui::EmailUiTextRole::DetailBody))
        {
            return Err(format!(
                "{} text node {:?} exceeds parent {:?}",
                localized.key,
                computed.size(),
                parent_computed.size()
            ));
        }
        if !layout.size.is_finite()
            || layout.size.x > computed.size().x + 1.0
            || layout.size.y > computed.size().y + 1.0
        {
            return Err(format!(
                "{} GPU layout {:?} exceeds text node {:?}",
                localized.key,
                layout.size,
                computed.size()
            ));
        }

        let mut minimum = Vec2::splat(f32::INFINITY);
        let mut maximum = Vec2::splat(f32::NEG_INFINITY);
        for glyph in &layout.glyphs {
            minimum = minimum.min(glyph.position - glyph.atlas_info.rect.size() * 0.5);
            maximum = maximum.max(glyph.position + glyph.atlas_info.rect.size() * 0.5);
        }
        if minimum.x < -2.0
            || minimum.y < -2.0
            || maximum.x > computed.size().x + 2.0
            || maximum.y > computed.size().y + 2.0
        {
            return Err(format!(
                "{} glyph bounds {:?}..{:?} exceed text node {:?}",
                localized.key,
                minimum,
                maximum,
                computed.size()
            ));
        }
        glyph_min_y = glyph_min_y.min(minimum.y);
        glyph_max_y = glyph_max_y.max(maximum.y);

        for run in &layout.run_geometry {
            let line = run.bounds;
            // Bevy 0.19 replaced the per-section line rect with `run_geometry`,
            // whose `bounds` is the decoration/selection box, not the line box.
            // Parley clamps that box to at least `ascent + descent`, so an
            // authored line height tighter than the font's em box legitimately
            // reports a taller box that overhangs the line: a 12 px font on a
            // 13.71 px line height measures 15 px starting at -1.0. The authored
            // height stays under contract on the `LineHeight` component checked
            // above, so allow the decoration box exactly that documented
            // overhang and still fail a collapsed or oversized run.
            let overhang =
                (font.0.font_size.eval(Vec2::ZERO, 16.0) * 1.5 - current_line_height).max(0.0) + 0.55;
            if !line.min.is_finite()
                || !line.max.is_finite()
                || line.min.y < -overhang
                || line.max.y > layout.size.y + overhang
                || line.height() + 0.55 < current_line_height
                || line.height() > current_line_height + overhang
            {
                return Err(format!(
                    "{} invalid GPU line/baseline box {:?} for line height {}",
                    localized.key, line, current_line_height
                ));
            }
        }
    }

    if visible_count == 0 || glyph_count == 0 || line_count == 0 {
        return Err("visible GPU text has not been laid out yet".to_owned());
    }
    let required_styles: &[EmailUiTextStyle] = match scene {
        PreviewScene::Player => &[
            EmailUiTextStyle::BlankBoxUpperLeft,
            EmailUiTextStyle::LabelMiddleCenter,
            EmailUiTextStyle::ChaletLabelMiddleLeft,
            EmailUiTextStyle::TextArea,
            EmailUiTextStyle::Button,
            EmailUiTextStyle::MailPageButton,
        ],
        PreviewScene::Compose => &[
            EmailUiTextStyle::BlankBoxUpperLeft,
            EmailUiTextStyle::LabelMiddleRight,
            EmailUiTextStyle::ButtonLabelFont,
            EmailUiTextStyle::RedButtonLabelFont,
            EmailUiTextStyle::PostageLabel,
        ],
        PreviewScene::Buddy => &[
            EmailUiTextStyle::RightLabel,
            EmailUiTextStyle::ButtonLabelFont,
            EmailUiTextStyle::PostageLabel,
        ],
        PreviewScene::Calculator => &[
            EmailUiTextStyle::CalculatorButton,
            EmailUiTextStyle::CenterLabel,
            EmailUiTextStyle::RightLabel,
        ],
    };
    for required in required_styles {
        if !visible_styles.contains(required) {
            return Err(format!(
                "scene {} has no visible {required:?} text",
                scene.name()
            ));
        }
    }
    if language == "ru" && !saw_cyrillic {
        return Err("RU production bundle produced no visible Cyrillic glyphs".to_owned());
    }

    visible_styles.sort_by_key(|style| format!("{style:?}"));
    Ok(format!(
        "Email GPU text audit: scene={}, locale={}, key-first={}/{}, visible={}, auto-fit={}, \
         glyphs={}, line-boxes={}, glyph-y={:.1}..{:.1}, styles={:?}",
        scene.name(),
        language,
        styled_count,
        all_count,
        visible_count,
        fitted_count,
        glyph_count,
        line_count,
        glyph_min_y,
        glyph_max_y,
        visible_styles
    ))
}
