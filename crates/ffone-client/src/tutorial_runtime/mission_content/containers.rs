use super::*;

pub(super) fn indexed_table_object<'a>(
    rows: &'a [Value],
    source_id: i32,
    table: &str,
    context: &str,
) -> TutorialMissionContentResult<&'a Map<String, Value>> {
    let index = usize::try_from(source_id)
        .map_err(|_| invalid(format!("{context} has negative {table} row ID {source_id}")))?;
    let row = rows.get(index).ok_or_else(|| {
        invalid(format!(
            "{context} references missing {table} row {source_id}"
        ))
    })?;
    value_object(row, &format!("{context} -> {table}[{source_id}]"))
}

pub(super) fn value_object<'a>(
    value: &'a Value,
    context: &str,
) -> TutorialMissionContentResult<&'a Map<String, Value>> {
    value
        .as_object()
        .ok_or_else(|| invalid(format!("{context} must be a JSON object")))
}

pub(super) fn required_object<'a>(
    object: &'a Map<String, Value>,
    field: &str,
    context: &str,
) -> TutorialMissionContentResult<&'a Map<String, Value>> {
    object
        .get(field)
        .ok_or_else(|| invalid(format!("{context} has no {field}")))?
        .as_object()
        .ok_or_else(|| invalid(format!("{context}.{field} must be an object")))
}
