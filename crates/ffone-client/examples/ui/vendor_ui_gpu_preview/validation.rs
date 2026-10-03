use super::*;

pub(super) fn audit_vendor_text(
    texts: &Query<
        '_,
        '_,
        (
            Entity,
            &Text,
            &LocalizedText,
            &VendorUiTextStyle,
            (&TextFont, &LineHeight),
            &TextLayoutInfo,
            &ComputedNode,
            &UiTransform,
        ),
    >,
    preview_assets: &PreviewAssets,
    language: &str,
) -> Result<String, String> {
    let rows = texts.iter().collect::<Vec<_>>();
    if rows.len() != EXPECTED_TEXT_COUNT {
        return Err(format!(
            "key-first Text count {}, expected {EXPECTED_TEXT_COUNT}",
            rows.len()
        ));
    }

    let mut visible = 0usize;
    let mut glyphs = 0usize;
    let mut line_boxes = 0usize;
    let mut saw_cyrillic = false;
    let mut styles = Vec::new();
    let mut glyph_min_y = f32::INFINITY;
    let mut glyph_max_y = f32::NEG_INFINITY;
    for (_entity, text, localized, style, font, layout, computed, transform) in rows {
        if localized.key.trim().is_empty() {
            return Err(format!("empty localization key on {}", localized.fallback));
        }
        let expected_font = if *style == VendorUiTextStyle::LabelUpperLeftService {
            &preview_assets.service_font
        } else {
            &preview_assets.font
        };
        if font.0.font != bevy::text::FontSource::Handle(expected_font.clone()) {
            return Err(format!("{} uses the wrong replacement font", localized.key));
        }
        if (font.0.font_size.eval(Vec2::ZERO, 16.0) - style.font_size()).abs() > 0.01 {
            return Err(format!(
                "{} font size {} != {}",
                localized.key,
                font.0.font_size.eval(Vec2::ZERO, 16.0),
                style.font_size()
            ));
        }
        match (*font.1) {
            LineHeight::Px(actual) if (actual - style.line_height()).abs() <= 0.01 => {}
            actual => {
                return Err(format!(
                    "{} line height {actual:?} != {}",
                    localized.key,
                    style.line_height()
                ));
            }
        }
        if transform.translation != Val2::px(0.0, style.replacement_y_offset()) {
            return Err(format!(
                "{} has unexpected replacement-font Y offset {:?}",
                localized.key, transform.translation
            ));
        }
        let expected_args: &[&str] = match localized.key.as_str() {
            "ui.vendor.shopkeeper" | "ui.vendor.item.name" => &["name"],
            "ui.vendor.service" => &["service"],
            "ui.vendor.item.level" => &["level"],
            "ui.vendor.item.vehicle_speed" => &["speed"],
            "ui.vendor.item.price" => &["price"],
            "ui.vendor.taros.digit" => &["digit"],
            "ui.inventory.item.count" | "ui.enchant.battery.count" => &["count"],
            "ui.inventory.item.quest" => &["item_id"],
            "ui.inventory.slot.weapon" => &["ordinal"],
            _ => &[],
        };
        if localized
            .args
            .keys()
            .map(String::as_str)
            .ne(expected_args.iter().copied())
        {
            return Err(format!(
                "{} template args {:?} != {:?}",
                localized.key,
                localized.args.keys().collect::<Vec<_>>(),
                expected_args
            ));
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
        let expected_lines = if localized.key == "ui.inventory.item.quest" {
            2
        } else {
            1
        };
        if layout.run_geometry.len() != expected_lines {
            return Err(format!(
                "{} produced {} lines, expected {expected_lines} inside the primary Rect",
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
        for run in &layout.run_geometry {
            let line = run.bounds;
            if !line.min.is_finite()
                || !line.max.is_finite()
                || line.min.y < -0.5
                || line.max.y > layout.size.y + 0.5
                || (line.height() - style.line_height()).abs() > 0.75
            {
                return Err(format!(
                    "{} invalid line box {:?} for {}",
                    localized.key,
                    line,
                    style.line_height()
                ));
            }
        }
    }
    let expected_visible = if env::var("FFONE_VENDOR_POPUP").is_ok_and(|v| v.starts_with("chest")) {
        EXPECTED_VISIBLE_TEXT_COUNT - 1 // chest replaces the fixture's stack-count label
    } else {
        EXPECTED_VISIBLE_TEXT_COUNT
    };
    if visible != expected_visible || glyphs == 0 || line_boxes == 0 {
        return Err(format!(
            "visible VendorMode GPU text count {visible}, expected {expected_visible}"
        ));
    }
    for required in [
        VendorUiTextStyle::LabelUpperLeft,
        VendorUiTextStyle::LabelUpperLeftService,
        VendorUiTextStyle::BlankBoxUpperLeft,
        VendorUiTextStyle::BlankBoxMiddleRight,
        VendorUiTextStyle::ButtonMiddleCenter,
        VendorUiTextStyle::EquipBarMiddleCenter,
        VendorUiTextStyle::EquipFontMiddleRight,
    ] {
        if !styles.contains(&required) {
            return Err(format!("visible VendorMode has no {required:?}"));
        }
    }
    if language == "ru" && !saw_cyrillic {
        return Err("RU VendorMode produced no visible Cyrillic glyphs".to_owned());
    }
    styles.sort_by_key(|style| format!("{style:?}"));
    Ok(format!(
        "VendorMode GPU text audit: locale={language}, key-first={}, visible={visible}, \
         glyphs={glyphs}, line-boxes={line_boxes}, glyph-y={glyph_min_y:.1}..{glyph_max_y:.1}, \
         styles={styles:?}",
        EXPECTED_TEXT_COUNT
    ))
}
