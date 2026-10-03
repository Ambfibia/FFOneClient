//! Server-shaped XDT on disk; lossless named table views for native consumers.
//! Gameplay values exist only once, at the JSON root. `_ffone` retains native
//! routes, table identities and all other document metadata ignored by the server.
use serde_json::{Map, Value};

pub const SCHEMA: &str = "ffone.xdt.v1";
pub const TABLE_SET_SCHEMA: &str = "ffone.table-set.v1";

pub fn from_slice(bytes: &[u8]) -> Result<Value, String> {
    let document = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    into_table_set(document)
}

pub fn into_table_set(mut document: Value) -> Result<Value, String> {
    if document.get("_ffone").is_none() {
        return Ok(document);
    }
    let root = document
        .as_object_mut()
        .ok_or("XDT root must be an object")?;
    let mut native = root
        .remove("_ffone")
        .ok_or("XDT has no native _ffone metadata")?;
    if native["schema"] != SCHEMA {
        return Err("Unsupported native XDT schema".into());
    }
    let metadata = native
        .as_object_mut()
        .ok_or("XDT metadata must be an object")?;
    let slot = metadata
        .remove("gameplay_table")
        .ok_or("Missing XDT gameplay table identity")?;
    let tables = metadata
        .get_mut("tables")
        .and_then(Value::as_array_mut)
        .ok_or("Missing native XDT tables")?;
    if !slot.is_null() {
        let index = slot
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .ok_or("Invalid XDT gameplay table index")?;
        let table = tables
            .get_mut(index)
            .and_then(Value::as_object_mut)
            .ok_or("Missing XDT gameplay table")?;
        if table.contains_key("value") {
            return Err("XDT gameplay data is duplicated in native metadata".into());
        }
        table.insert("value".into(), Value::Object(std::mem::take(root)));
    } else if !root.is_empty() {
        return Err("XDT has gameplay data without a table identity".into());
    }
    metadata.insert("schema".into(), Value::from(TABLE_SET_SCHEMA));
    Ok(native)
}

pub fn into_server_document(mut table_set: Value) -> Result<Value, String> {
    if table_set["schema"] != TABLE_SET_SCHEMA {
        return Err("Unsupported native table-set schema".into());
    }
    let metadata = table_set
        .as_object_mut()
        .ok_or("TableData root must be an object")?;
    if metadata.contains_key("gameplay_table") {
        return Err("Reserved XDT metadata key: gameplay_table".into());
    }
    let tables = metadata
        .get_mut("tables")
        .and_then(Value::as_array_mut)
        .ok_or("Missing native tables")?;
    let index = tables
        .iter()
        .position(|t| t["name"] == "npc_imports_consolidated")
        .or_else(|| {
            tables
                .iter()
                .position(|t| t["name"] != "native_asset_routes")
        });
    let mut root = Map::new();
    if let Some(index) = index {
        let value = tables[index]
            .as_object_mut()
            .ok_or("Invalid gameplay table")?
            .remove("value")
            .ok_or("Missing gameplay values")?;
        let Value::Object(values) = value else {
            return Err("Gameplay values must be an object".into());
        };
        root = values;
        if root.contains_key("_ffone") {
            return Err("Reserved XDT gameplay key: _ffone".into());
        }
    }
    metadata.insert(
        "gameplay_table".into(),
        index.map_or(Value::Null, Value::from),
    );
    metadata.insert("schema".into(), Value::from(SCHEMA));
    root.insert("_ffone".into(), table_set);
    Ok(Value::Object(root))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn server_shape_round_trip_preserves_every_value_and_native_extension() {
        let source = json!({"schema":TABLE_SET_SCHEMA,"unknown":{"keep":[1,"РУ",true]},"tables":[
            {"name":"native_asset_routes","key":"routes-id","custom":12,"value":{"m_pAudioData":[{"id":"voice","path":"audio/ru.ogg"}],"m_pCharacterModelData":[{"glb":"characters/model.glb"}]}},
            {"name":"npc_imports_consolidated","key":"accepted-id","custom":{"future":true},"value":{
                "m_pMissionTable":{"m_pMissionData":[{"m_iHTaskID":99,"m_iCSTReqMission":[7,0],"unknown":[1,2,3]}]},
                "m_pGeneralItemTable":{"m_pItemData":[{"m_iItemNumber":42,"m_iStarterZoneEnabled":1}]}}},
            {"name":"future_feature","value":{"new":[{"preserve":true}]}}
        ]});
        let flat = into_server_document(source.clone()).unwrap();
        assert_eq!(
            flat["m_pMissionTable"]["m_pMissionData"][0]["m_iHTaskID"],
            99
        );
        assert!(flat.get("tables").is_none());
        assert!(flat["_ffone"]["tables"][1].get("value").is_none());
        assert_eq!(
            flat["_ffone"]["tables"][0]["value"]["m_pAudioData"][0]["id"],
            "voice"
        );
        assert_eq!(
            from_slice(&serde_json::to_vec(&flat).unwrap()).unwrap(),
            source
        );
        assert_eq!(into_table_set(source.clone()).unwrap(), source);
    }

    #[test]
    fn malformed_or_ambiguous_extensions_fail_instead_of_dropping_data() {
        let source =
            json!({"schema":TABLE_SET_SCHEMA,"tables":[{"name":"test","value":{"m_pRows":[]}}]});
        let mut flat = into_server_document(source.clone()).unwrap();
        flat["_ffone"]["tables"][0]["value"] = json!({"must_not_disappear":true});
        assert!(into_table_set(flat).is_err());
        let mut flat = into_server_document(source.clone()).unwrap();
        flat["_ffone"]["gameplay_table"] = json!(999);
        assert!(into_table_set(flat).is_err());
        let mut source = source;
        source["tables"][0]["value"]["_ffone"] = json!({"must_not_disappear":true});
        assert!(into_server_document(source).is_err());
    }

    #[test]
    fn routing_only_documents_preserve_metadata_without_inventing_gameplay() {
        let source = json!({"schema":TABLE_SET_SCHEMA,"tables":[{"name":"native_asset_routes","value":{"m_pAudioData":[]}}]});
        let flat = into_server_document(source.clone()).unwrap();
        assert!(flat["_ffone"]["gameplay_table"].is_null());
        assert_eq!(into_table_set(flat).unwrap(), source);
    }

    #[test]
    fn published_xdt_round_trip_keeps_all_gameplay_and_native_data() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/game/data/tables/xdt.json");
        let published: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert!(published["m_pMissionTable"]["m_pMissionData"].is_array());
        assert!(published.get("tables").is_none());
        let tables = into_table_set(published.clone()).unwrap();
        let routes = crate::asset_tables::from_document(&tables).unwrap();
        assert!(!routes["m_pAudioData"].as_array().unwrap().is_empty());
        assert!(
            !routes["m_pCharacterModelData"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert_eq!(into_server_document(tables).unwrap(), published);
    }
}
