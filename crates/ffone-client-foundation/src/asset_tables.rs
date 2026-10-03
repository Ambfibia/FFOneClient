//! Native asset references owned by TableData, independent of migration inventories.
use std::path::Path;

use serde_json::Value;

pub const TABLE_NAME: &str = "native_asset_routes";

pub fn read(root: &Path) -> Result<Value, String> {
    let path = root.join(crate::assets::TABLE_SET_PATH);
    let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    // Skip unrelated XDT values during deserialization instead of allocating a
    // second complete copy of the large gameplay table document.
    #[derive(serde::Deserialize)]
    struct Document {
        #[serde(default)]
        schema: String,
        #[serde(default)]
        tables: Vec<Entry>,
        #[serde(rename = "_ffone")]
        extension: Option<Box<Document>>,
    }
    #[derive(serde::Deserialize)]
    struct Entry {
        name: String,
        #[serde(default)]
        value: Routes,
    }
    #[derive(Default, serde::Deserialize)]
    struct Routes {
        #[serde(default, rename = "m_pAudioData")]
        audio: Value,
        #[serde(default, rename = "m_pCharacterModelData")]
        models: Value,
    }
    let mut document: Document = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if let Some(extension) = document.extension.take() {
        document = *extension;
    }
    if document.schema != crate::xdt::TABLE_SET_SCHEMA && document.schema != crate::xdt::SCHEMA {
        return Err("unsupported TableData schema".into());
    }
    let mut entries = document.tables.into_iter().filter(|e| e.name == TABLE_NAME);
    let entry = entries
        .next()
        .ok_or("TableData has no native_asset_routes table")?;
    if entries.next().is_some() {
        return Err("duplicate native_asset_routes table".into());
    }
    Ok(
        serde_json::json!({"m_pAudioData":entry.value.audio,"m_pCharacterModelData":entry.value.models}),
    )
}

pub fn from_document(document: &Value) -> Result<&Value, String> {
    let document = document.get("_ffone").unwrap_or(document);
    let tables = document["tables"]
        .as_array()
        .ok_or("TableData has no tables")?;
    let mut found = tables.iter().filter(|t| t["name"] == TABLE_NAME);
    let table = found
        .next()
        .ok_or("TableData has no native_asset_routes table")?;
    if found.next().is_some() {
        return Err("duplicate native_asset_routes table".to_owned());
    }
    Ok(&table["value"])
}

/// Transitional in-memory view for model consumers. No character inventory is opened.
pub fn character_models(root: &Path) -> Result<Value, String> {
    let table = read(root)?;
    model_view(&table)
}

pub fn character_models_from_document(document: &Value) -> Result<Value, String> {
    model_view(from_document(document)?)
}

fn model_view(table: &Value) -> Result<Value, String> {
    let models = table["m_pCharacterModelData"]
        .as_array()
        .ok_or("TableData has no m_pCharacterModelData")?;
    Ok(serde_json::json!({"schema": "ffone.semantic-character-registry.v2", "models": models}))
}
