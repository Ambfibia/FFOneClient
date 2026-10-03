use super::*;

pub(super) fn value_object<'a>(
    value: &'a Value,
    context: &str,
) -> Result<&'a Map<String, Value>, TransportationCatalogError> {
    value
        .as_object()
        .ok_or_else(|| invalid(format!("{context} must be an object")))
}

pub(super) fn required_object<'a>(
    object: &'a Map<String, Value>,
    field: &str,
    context: &str,
) -> Result<&'a Map<String, Value>, TransportationCatalogError> {
    object
        .get(field)
        .ok_or_else(|| invalid(format!("{context}.{field} is missing")))
        .and_then(|value| value_object(value, &format!("{context}.{field}")))
}
