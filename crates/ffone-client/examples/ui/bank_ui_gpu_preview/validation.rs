use super::*;

pub(super) fn audit_bank_text(
    texts: &Query<
        '_,
        '_,
        (
            Entity,
            &Text,
            &LocalizedText,
            &BankUiTextStyle,
            (&TextFont, &LineHeight, &ComputedTextBlock),
            &TextLayoutInfo,
            &ComputedNode,
            &UiTransform,
            &UiGlobalTransform,
            Option<&ChildOf>,
        ),
    >,
    computed_nodes: &Query<'_, '_, (&ComputedNode, &UiGlobalTransform)>,
    replacement_font: &Handle<Font>,
    search_font: &Handle<Font>,
    language: &str,
) -> Result<String, String> {
    let rows = texts.iter().collect::<Vec<_>>();
    if rows.len() != EXPECTED_TEXT_COUNT {
        return Err(format!(
            "key-first Text count {}, expected {EXPECTED_TEXT_COUNT}",
            rows.len()
        ));
    }
    let key_first = rows.len();
    if let Some((_, _, localized, _, _, _, _, _, _, _)) = rows
        .iter()
        .find(|(_, _, localized, _, _, _, _, _, _, _)| localized.key.trim().is_empty())
    {
        return Err(format!("empty localization key on {}", localized.fallback));
    }

    let mut visible = 0usize;
    let mut glyphs = 0usize;
    let mut line_boxes = 0usize;
    let mut saw_cyrillic = false;
    let mut styles = Vec::new();
    let mut glyph_min_y = f32::INFINITY;
    let mut glyph_max_y = f32::NEG_INFINITY;
    for (entity, text, localized, style, font, layout, computed, transform, global, parent) in rows
    {
        let expected_font = if *style == BankUiTextStyle::SearchUpperLeft {
            search_font
        } else {
            replacement_font
        };
        if font.0.font != bevy::text::FontSource::Handle(expected_font.clone()) {
            return Err(format!("{} uses the wrong replacement font", localized.key));
        }
        let (expected_size, expected_line_height) = match *style {
            BankUiTextStyle::SearchUpperLeft => (12.0, 13.56000042),
            BankUiTextStyle::LabelUpperLeft
            | BankUiTextStyle::BlankBoxUpperLeft
            | BankUiTextStyle::BlankBoxMiddleRight => {
                (BANK_LABEL_FONT_SIZE, BANK_LABEL_LINE_HEIGHT)
            }
            BankUiTextStyle::ButtonMiddleCenter => (BANK_BUTTON_FONT_SIZE, BANK_BUTTON_LINE_HEIGHT),
            BankUiTextStyle::EquipBarMiddleCenter | BankUiTextStyle::EquipFontMiddleRight => {
                (BANK_EQUIP_FONT_SIZE, BANK_EQUIP_LINE_HEIGHT)
            }
        };
        if (font.0.font_size.eval(Vec2::ZERO, 16.0) - expected_size).abs() > 0.01 {
            return Err(format!(
                "{} font size {} != {expected_size}",
                localized.key,
                font.0.font_size.eval(Vec2::ZERO, 16.0)
            ));
        }
        match *font.1 {
            LineHeight::Px(actual) if (actual - expected_line_height).abs() <= 0.01 => {}
            actual => {
                return Err(format!(
                    "{} line height {actual:?} != {expected_line_height}",
                    localized.key
                ));
            }
        }
        if transform.translation != Val2::px(0.0, BANK_TEXT_REPLACEMENT_Y_OFFSET) {
            return Err(format!(
                "{} has unexpected replacement-font Y offset {:?}",
                localized.key, transform.translation
            ));
        }
        if localized.key == "ui.inventory.item.count"
            && localized.args.keys().map(String::as_str).ne(["count"])
        {
            return Err("dynamic item count bypasses {count} template args".to_owned());
        }
        if localized.key == "ui.inventory.slot.weapon"
            && localized.args.keys().map(String::as_str).ne(["ordinal"])
        {
            return Err("weapon ordinal bypasses {ordinal} template args".to_owned());
        }
        if localized.key == "ui.bank.taros.digit"
            && localized.args.keys().map(String::as_str).ne(["digit"])
        {
            return Err("Taros digit bypasses {digit} template args".to_owned());
        }
        if text.0.is_empty() || computed.size().x <= 0.0 || layout.glyphs.is_empty() {
            continue;
        }

        visible += 1;
        glyphs += layout.glyphs.len();
        line_boxes += layout.run_geometry.len();
        saw_cyrillic |= text.0.chars().any(|character| {
            ('\u{0400}'..='\u{052f}').contains(&character)
                || ('\u{2de0}'..='\u{2dff}').contains(&character)
        });
        if !styles.contains(style) {
            styles.push(*style);
        }
        if layout.run_geometry.len() != 1 {
            return Err(format!(
                "{} wrapped to {} lines inside the primary Rect",
                localized.key,
                layout.run_geometry.len()
            ));
        }
        if !layout.size.is_finite()
            || layout.size.x > computed.size().x + 1.0
            || layout.size.y > computed.size().y + 1.0
        {
            return Err(format!(
                "{} layout {:?} exceeds text node {:?}",
                localized.key,
                layout.size,
                computed.size()
            ));
        }
        if matches!(
            *style,
            BankUiTextStyle::BlankBoxMiddleRight
                | BankUiTextStyle::ButtonMiddleCenter
                | BankUiTextStyle::EquipBarMiddleCenter
                | BankUiTextStyle::EquipFontMiddleRight
        ) && let Some(parent) = parent
        {
            let (parent_node, parent_global) = computed_nodes
                .get(parent.parent())
                .map_err(|_| format!("{entity:?} has no style container"))?;
            if computed.size().x > parent_node.size().x + 1.0
                || computed.size().y > parent_node.size().y + 1.0
            {
                return Err(format!(
                    "{} node {:?} exceeds primary style container {:?}",
                    localized.key,
                    computed.size(),
                    parent_node.size()
                ));
            }
            let child_center = global.translation;
            let parent_center = parent_global.translation;
            let child_top_left = child_center - computed.size() * 0.5;
            let parent_top_left = parent_center - parent_node.size() * 0.5;
            let actual = child_top_left - parent_top_left;
            let expected = match *style {
                BankUiTextStyle::BlankBoxMiddleRight if localized.key == "ui.bank.taros.digit" => {
                    (parent_node.size() - computed.size()) * 0.5
                }
                BankUiTextStyle::BlankBoxMiddleRight => Vec2::new(
                    parent_node.size().x - computed.size().x,
                    (parent_node.size().y - computed.size().y) * 0.5,
                ),
                BankUiTextStyle::ButtonMiddleCenter => Vec2::new(
                    (parent_node.size().x - computed.size().x) * 0.5,
                    BANK_BUTTON_PADDING_TOP
                        + (parent_node.size().y
                            - BANK_BUTTON_PADDING_TOP
                            - BANK_BUTTON_PADDING_BOTTOM
                            - computed.size().y)
                            * 0.5,
                ),
                BankUiTextStyle::EquipBarMiddleCenter => {
                    (parent_node.size() - computed.size()) * 0.5
                }
                BankUiTextStyle::EquipFontMiddleRight => Vec2::new(
                    parent_node.size().x - computed.size().x,
                    (parent_node.size().y - computed.size().y) * 0.5,
                ),
                _ => unreachable!(),
            };
            if (actual - expected).abs().max_element() > 0.75 {
                return Err(format!(
                    "{} baseline/alignment offset {:?} != {:?} inside {:?}",
                    localized.key,
                    actual,
                    expected,
                    parent_node.size()
                ));
            }
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
                "{} glyphs {:?}..{:?} exceed text node {:?}",
                localized.key,
                minimum,
                maximum,
                computed.size()
            ));
        }
        glyph_min_y = glyph_min_y.min(minimum.y);
        glyph_max_y = glyph_max_y.max(maximum.y);
        // Parley decoration bounds include ascent/descent even for negative leading.
        for shaped_line in font.2.buffer().lines() {
            let line = shaped_line.metrics();
            if !line.baseline.is_finite()
                || !line.line_height.is_finite()
                || (line.line_height - expected_line_height).abs() > 0.75
            {
                return Err(format!(
                    "{} invalid line box {:?} for {expected_line_height}",
                    localized.key, line
                ));
            }
        }
    }
    if visible != EXPECTED_VISIBLE_TEXT_COUNT || glyphs == 0 || line_boxes == 0 {
        return Err(format!(
            "visible BankMode GPU text count {visible}, expected {EXPECTED_VISIBLE_TEXT_COUNT}"
        ));
    }
    for required in [
        BankUiTextStyle::LabelUpperLeft,
        BankUiTextStyle::BlankBoxUpperLeft,
        BankUiTextStyle::BlankBoxMiddleRight,
        BankUiTextStyle::ButtonMiddleCenter,
        BankUiTextStyle::EquipBarMiddleCenter,
        BankUiTextStyle::EquipFontMiddleRight,
    ] {
        if !styles.contains(&required) {
            return Err(format!("visible BankMode has no {required:?}"));
        }
    }
    if language == "ru" && !saw_cyrillic {
        return Err("RU BankMode produced no visible Cyrillic glyphs".to_owned());
    }
    styles.sort_by_key(|style| format!("{style:?}"));
    Ok(format!(
        "BankMode GPU text audit: locale={language}, key-first={}, visible={visible}, \
         glyphs={glyphs}, line-boxes={line_boxes}, glyph-y={glyph_min_y:.1}..{glyph_max_y:.1}, \
         y-offset={BANK_TEXT_REPLACEMENT_Y_OFFSET:.1}, styles={styles:?}",
        key_first
    ))
}
