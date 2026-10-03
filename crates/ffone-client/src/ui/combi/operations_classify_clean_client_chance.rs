use super::*;

pub(super) fn recipe_i32(
    object: &serde_json::Map<String, Value>,
    index: usize,
    field: &'static str,
) -> Result<i32, CombiRecipeTableError0104> {
    let value = object
        .get(field)
        .and_then(Value::as_i64)
        .ok_or(CombiRecipeTableError0104::InvalidField { index, field })?;
    i32::try_from(value).map_err(|_| CombiRecipeTableError0104::InvalidField { index, field })
}

pub(super) fn recipe_f32(
    object: &serde_json::Map<String, Value>,
    index: usize,
    field: &'static str,
) -> Result<f32, CombiRecipeTableError0104> {
    let value = object
        .get(field)
        .and_then(Value::as_f64)
        .ok_or(CombiRecipeTableError0104::InvalidField { index, field })?;
    let value = value as f32;
    if !value.is_finite() {
        return Err(CombiRecipeTableError0104::InvalidField { index, field });
    }
    Ok(value)
}

#[must_use]
pub const fn combined_appearance_item_id(item: ItemBase0104) -> Option<i16> {
    let raw = ((item.option >> 16) & 0xffff) as u16;
    if raw == 0 { None } else { Some(raw as i16) }
}

#[must_use]
pub const fn combi_equipment_item_type(item_type: i16) -> bool {
    item_type >= 0 && item_type < 4
}

pub(super) fn compare_stat(candidate: i32, equipped: i32) -> CombiStatComparison0104 {
    match candidate.cmp(&equipped) {
        std::cmp::Ordering::Less => CombiStatComparison0104::Lower,
        std::cmp::Ordering::Equal => CombiStatComparison0104::Equal,
        std::cmp::Ordering::Greater => CombiStatComparison0104::Higher,
    }
}

#[must_use]
pub fn clean_range_label(item_type: i16, equip_type: i32, delay_time: i32) -> String {
    if item_type != 0 {
        return "N/A".to_owned();
    }
    match equip_type {
        0 | 1 | 2 | 3 | 7 | 8 => "Short".to_owned(),
        5 | 6 | 9 | 10 => "Medium".to_owned(),
        4 | 11 => "Long".to_owned(),
        _ if delay_time == 0 => "Infinity".to_owned(),
        _ => format!("{:.2}", 1.0 / (delay_time as f32 * 0.1)),
    }
}

pub(super) fn classify_clean_client_chance(
    raw_percent: f32,
) -> Result<CombiChance0104, CombiProjectionError0104> {
    if !raw_percent.is_finite() || raw_percent < 0.0 {
        return Err(CombiProjectionError0104::InvalidChance { raw_percent });
    }
    // Clean has no `else` for exactly 100 or above and would retain the
    // previous GUI value. The authoritative Retrobution rows all stay below
    // 100; a future divergent table is rejected rather than inventing state.
    if raw_percent >= 100.0 {
        return Err(CombiProjectionError0104::UnsupportedChanceRetention { raw_percent });
    }
    Ok(if raw_percent < 10.0 {
        CombiChance0104::VeryLow { raw_percent }
    } else if raw_percent < 30.0 {
        CombiChance0104::Low { raw_percent }
    } else if raw_percent < 70.0 {
        CombiChance0104::Medium { raw_percent }
    } else if raw_percent < 90.0 {
        CombiChance0104::Good { raw_percent }
    } else {
        CombiChance0104::VeryGood { raw_percent }
    })
}

#[must_use]
pub const fn empty_item_0104() -> ItemBase0104 {
    ItemBase0104 {
        item_type: 0,
        item_id: 0,
        option: 0,
        time_limit: 0,
    }
}

#[must_use]
pub const fn expected_success_style_item(
    style_before: ItemBase0104,
    stats_before: ItemBase0104,
) -> ItemBase0104 {
    let appearance = match combined_appearance_item_id(style_before) {
        Some(item_id) => item_id as u16 as i32,
        None => style_before.item_id as u16 as i32,
    };
    ItemBase0104 {
        item_type: style_before.item_type,
        item_id: stats_before.item_id,
        option: appearance << 16,
        time_limit: style_before.time_limit,
    }
}

pub(super) fn combi_fallback_text(localized: &LocalizedText) -> String {
    let mut value = localized.fallback.clone();
    for (name, replacement) in &localized.args {
        value = value.replace(&format!("{{{name}}}"), replacement);
    }
    value
}

pub(super) fn combi_passthrough_text(value: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", value)
}

pub(super) fn combi_cost_text(cost: i32) -> LocalizedText {
    LocalizedText::new("ui.combi.cost", "{cost}").with_arg("cost", cost.to_string())
}

pub(super) fn combi_chance_level_text(level: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.combi.chance.level", "{level}").with_arg("level", level)
}

pub(super) fn combi_item_level_text(level: i32) -> LocalizedText {
    LocalizedText::new("ui.combi.item.level", "LEVEL {level}").with_arg("level", level.to_string())
}

pub(super) fn combi_stat_value_text(value: i32) -> LocalizedText {
    LocalizedText::new("ui.combi.item.stat_value", "{value}").with_arg("value", value.to_string())
}

pub(super) fn combi_range_text(value: &str) -> LocalizedText {
    let key = match value {
        "N/A" => Some("ui.combi.range.not_applicable"),
        "Short" => Some("ui.combi.range.short"),
        "Medium" => Some("ui.combi.range.medium"),
        "Long" => Some("ui.combi.range.long"),
        "Infinity" => Some("ui.combi.range.infinity"),
        _ => None,
    };
    key.map_or_else(
        || combi_passthrough_text(value),
        |key| LocalizedText::new(key, value),
    )
}

pub(super) fn combi_type_text(item_type: i16) -> Option<LocalizedText> {
    let (key, fallback) = match item_type {
        0 => ("ui.combi.type.weapon", "Weapon"),
        1 => ("ui.combi.type.body", "Body"),
        2 => ("ui.combi.type.legs", "Legs"),
        3 => ("ui.combi.type.shoes", "Shoes"),
        _ => return None,
    };
    Some(LocalizedText::new(key, fallback))
}

/// Decoration layered over or around a control. Bevy UI focus treats a node
/// without `FocusPolicy` as `Block`, so an icon, label, or backdrop without
/// `Pass` swallows the hover and press meant for the cell or button under it.
pub(super) fn combi_passive() -> (Pickable, FocusPolicy) {
    (Pickable::IGNORE, FocusPolicy::Pass)
}

pub(super) fn cancel_combi_carry(pointer: &mut CombiPointerState0104, outbox: &mut CombiUiOutbox0104) {
    if pointer.carried.take().is_some() {
        outbox.push(CombiUiCommand0104::CancelInventoryDrag);
    }
}
