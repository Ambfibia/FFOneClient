use super::*;

pub(super) fn sanitize_runtime_environment(value: &mut serde_json::Value) {
    let object = value.as_object_mut().unwrap();
    object.remove("mapScene");
    object.remove("placementAudit");
    object.remove("sourceCodeEvidence");
    object
        .get_mut("ambience")
        .and_then(serde_json::Value::as_object_mut)
        .unwrap()
        .remove("sourceObject");
    let terrain_detail = object
        .get_mut("terrainDetail")
        .and_then(serde_json::Value::as_object_mut)
        .unwrap();
    terrain_detail.remove("sourceObject");
    terrain_detail.remove("terrainRendererSourceObject");
    remove_key_recursively(value, "parsedData");
}
