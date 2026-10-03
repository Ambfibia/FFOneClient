use super::*;

pub(super) fn path_id(value: &JsonValue, key: &str) -> Result<i64, String> {
    object(value, key)?
        .get("pathId")
        .and_then(JsonValue::as_i64)
        .ok_or_else(|| format!("{key}.pathId is absent"))
}

pub(super) fn game_object_path(value: &JsonValue) -> Option<i64> {
    value.get("m_GameObject")?.get("pathId")?.as_i64()
}

pub(super) fn unique_path_type<'a>(
    closure: &'a TutorialEffectClosureFile,
    path: i64,
    object_type: &str,
) -> Result<&'a TutorialUnityObjectProof, String> {
    let matches = closure
        .objects
        .iter()
        .filter(|object| object.path_id == path && object.object_type == object_type)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [object] => Ok(*object),
        [] => Err(format!("closure has no {object_type} pathId {path}")),
        _ => Err(format!("closure has ambiguous {object_type} pathId {path}")),
    }
}
