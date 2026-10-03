use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn audit_text_contract(
    mode: PreviewMode,
    localization: &Localization,
    language: &Language,
    language_slug: &str,
    assets: &PreviewAssets,
    all_texts: &Query<'_, '_, &Text>,
    styled_texts: &Query<
        '_,
        '_,
        (
            Entity,
            &Text,
            &LocalizedText,
            (&TextFont, &LineHeight),
            &TextLayout,
            &TextLayoutInfo,
            &ResurrectTextStyle,
            &ComputedNode,
            &UiTransform,
            &InheritedVisibility,
            &ChildOf,
        ),
    >,
    computed_nodes: &Query<'_, '_, &ComputedNode>,
) -> Result<String, String> {
    let all_count = all_texts.iter().count();
    let styled_count = styled_texts.iter().count();
    if all_count != 7 || styled_count != all_count {
        return Err(format!(
            "every-Text ownership incomplete: styled={styled_count}, all={all_count}, expected=7"
        ));
    }

    let expected = [
        (
            RESURRECT_TITLE_LOCALIZATION_KEY,
            RESURRECT_TITLE,
            ResurrectTextStyle::Label,
            1usize,
        ),
        (
            RESURRECT_QUESTION_LOCALIZATION_KEY,
            RESURRECT_QUESTION,
            ResurrectTextStyle::ImageWindow,
            1,
        ),
        (
            RESURRECT_COUNTDOWN_LOCALIZATION_KEY,
            "You will go automatically in : {seconds} seconds.",
            ResurrectTextStyle::ImageWindow,
            1,
        ),
        (
            RESURRECT_GO_LOCALIZATION_KEY,
            RESURRECT_GO_LABEL,
            ResurrectTextStyle::Button,
            1,
        ),
        (
            RESURRECT_REVIVE_LOCALIZATION_KEY,
            RESURRECT_REVIVE_LABEL,
            ResurrectTextStyle::Button,
            2,
        ),
        (
            RESURRECT_USE_ITEM_LOCALIZATION_KEY,
            RESURRECT_USE_ITEM_LABEL,
            ResurrectTextStyle::Button,
            1,
        ),
    ];
    let mut counts = [0usize; 6];
    let mut visible_count = 0usize;
    let mut glyph_count = 0usize;
    let mut line_count = 0usize;
    let mut saw_cyrillic = false;

    for (
        entity,
        text,
        localized,
        font,
        text_layout,
        layout_info,
        style,
        computed,
        transform,
        inherited_visibility,
        parent,
    ) in styled_texts.iter()
    {
        let Some((index, (_, fallback, expected_style, _))) = expected
            .iter()
            .enumerate()
            .find(|(_, (key, _, _, _))| localized.key == *key)
        else {
            return Err(format!(
                "unexpected Resurrect localization key {}",
                localized.key
            ));
        };
        counts[index] += 1;
        if localized.fallback != *fallback || style != expected_style {
            return Err(format!(
                "{} lost its clean fallback or source GUIStyle",
                localized.key
            ));
        }
        if localized.key == RESURRECT_COUNTDOWN_LOCALIZATION_KEY {
            if localized.args.len() != 1
                || localized.args.get("seconds").map(String::as_str) != Some("60")
            {
                return Err("countdown must carry only the clean {seconds} argument".to_owned());
            }
        } else if !localized.args.is_empty() {
            return Err(format!(
                "{} has unexpected template arguments",
                localized.key
            ));
        }
        let expected_copy = localization.text(language, localized);
        if text.0 != expected_copy {
            return Err(format!(
                "{} rendered {:?}, expected {:?}",
                localized.key, text.0, expected_copy
            ));
        }

        let spec = style.spec();
        let expected_font = match spec.font_role {
            ResurrectFontRole::Jeffe => &assets.fonts[0],
            ResurrectFontRole::Chalet => &assets.fonts[1],
        };
        if font.0.font != bevy::text::FontSource::Handle(expected_font.clone())
            || font.0.font_size.eval(Vec2::ZERO, 16.0) != spec.font_size
            || (*font.1) != LineHeight::Px(spec.line_height)
        {
            return Err(format!(
                "{} has wrong replacement font metrics: size={}, line={:?}",
                localized.key,
                font.0.font_size.eval(Vec2::ZERO, 16.0),
                (*font.1)
            ));
        }
        let expected_justify =
            if spec.anchor == ffone_client::resurrect_ui::ResurrectTextAnchor::MiddleCenter {
                Justify::Center
            } else {
                Justify::Left
            };
        let expected_linebreak = if spec.word_wrap {
            LineBreak::WordBoundary
        } else {
            LineBreak::NoWrap
        };
        if text_layout.justify != expected_justify || text_layout.linebreak != expected_linebreak {
            return Err(format!(
                "{} does not preserve {} alignment/wrapping",
                localized.key, spec.source_style
            ));
        }
        if transform.translation != Val2::px(0.0, spec.y_offset) {
            return Err(format!(
                "{} replacement baseline offset {:?} != {}",
                localized.key, transform.translation, spec.y_offset
            ));
        }

        if !inherited_visibility.get()
            || computed.size().x <= 0.0
            || computed.size().y <= 0.0
            || layout_info.glyphs.is_empty()
        {
            continue;
        }
        visible_count += 1;
        glyph_count += layout_info.glyphs.len();
        line_count += layout_info.run_geometry.len();
        saw_cyrillic |= text.0.chars().any(|character| {
            ('\u{0400}'..='\u{052f}').contains(&character)
                || ('\u{2de0}'..='\u{2dff}').contains(&character)
        });

        let source_size = if *style == ResurrectTextStyle::Button {
            computed_nodes
                .get(parent.parent())
                .map_err(|_| format!("Text {entity:?} has no source button parent"))?
                .size()
        } else {
            computed.size()
        };
        let [left, right, top, bottom] = spec.padding;
        let available = Vec2::new(
            (source_size.x - left - right).max(0.0),
            (source_size.y - top - bottom).max(0.0),
        );
        if !layout_info.size.is_finite()
            || layout_info.size.x > available.x + 1.0
            || layout_info.size.y > available.y + 1.0
        {
            return Err(format!(
                "{} GPU layout {:?} exceeds source content Rect {:?}",
                localized.key, layout_info.size, available
            ));
        }

        let mut minimum = Vec2::splat(f32::INFINITY);
        let mut maximum = Vec2::splat(f32::NEG_INFINITY);
        for glyph in &layout_info.glyphs {
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
        for run in &layout_info.run_geometry {
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
                (font.0.font_size.eval(Vec2::ZERO, 16.0) * 1.5 - spec.line_height).max(0.0) + 0.55;
            if !line.min.is_finite()
                || !line.max.is_finite()
                || line.min.y < -overhang
                || line.max.y > layout_info.size.y + overhang
                || line.height() + 0.55 < spec.line_height
                || line.height() > spec.line_height + overhang
            {
                return Err(format!(
                    "{} invalid GPU line/baseline box {:?}",
                    localized.key, line
                ));
            }
        }
    }

    for (index, (_, _, _, expected_count)) in expected.iter().enumerate() {
        if counts[index] != *expected_count {
            return Err(format!(
                "incomplete semantic key tree: counts={counts:?}, expected={:?}",
                expected.map(|(_, _, _, count)| count)
            ));
        }
    }
    let expected_visible = if mode == PreviewMode::GroupItem { 6 } else { 5 };
    if visible_count != expected_visible || glyph_count == 0 || line_count == 0 {
        return Err(format!(
            "incomplete visible text tree: visible={visible_count}/{expected_visible}, glyphs={glyph_count}, lines={line_count}"
        ));
    }
    if language_slug == "ru" && !saw_cyrillic {
        return Err("RU production bundle produced no visible Cyrillic glyphs".to_owned());
    }

    Ok(format!(
        "ResurrectMode GPU text audit: mode={}, locale={language_slug}, key-first={styled_count}/{all_count}, visible={visible_count}, glyphs={glyph_count}, line-boxes={line_count}, styles=[label, Imagewindow, button]",
        mode.cli_name()
    ))
}
