use super::*;

/// Only paths explicitly authored in TableData participate in playback.
/// Enumerate language roots once, then probe each exact declared path.
pub(super) fn load(root: &Path) -> Result<EditableCatalogDocument, String> {
    let table = crate::asset_tables::read(root)?;
    let rows = table["m_pAudioData"]
        .as_array()
        .ok_or("TableData has no m_pAudioData")?;
    let mut locales = BTreeSet::from(["en".to_owned(), "ru".to_owned()]);
    let voice_root = root.join("audio/voice");
    if voice_root.is_dir() {
        for entry in fs::read_dir(voice_root).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
                let locale = entry.file_name().to_string_lossy().into_owned();
                validate_locale_id(&locale)?;
                locales.insert(locale);
            }
        }
    }
    let mut assets = Vec::new();
    for row in rows {
        let mut row = row.clone();
        let path = row["path"]
            .as_str()
            .ok_or("audio row has no path")?
            .to_owned();
        validate_relative_path(&path)?;
        let voice = row["category"] == "voice";
        let canonical_path = if voice {
            format!("audio/voice/en/{path}")
        } else {
            path.clone()
        };
        row.as_object_mut()
            .ok_or("invalid audio row")?
            .remove("path");
        row["files"] = serde_json::json!([]);
        let mut asset: EditableCatalogAsset =
            serde_json::from_value(row).map_err(|e| e.to_string())?;
        asset.canonical_path = canonical_path;
        validate_editable_file(
            &asset,
            asset.category.into(),
            &EditableCatalogFile {
                locale: voice.then(|| "en".to_owned()),
                path: asset.canonical_path.clone(),
            },
        )?;
        asset.files = if voice {
            let mut files = Vec::new();
            for locale in &locales {
                let relative = format!("audio/voice/{locale}/{path}");
                match fs::metadata(root.join(&relative)) {
                    Ok(meta) if meta.is_file() => files.push(EditableCatalogFile {
                        locale: Some(locale.clone()),
                        path: relative,
                    }),
                    Ok(_) => return Err(format!("voice path is not a file: {relative}")),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => return Err(format!("{relative}: {e}")),
                }
            }
            files
        } else {
            vec![EditableCatalogFile { locale: None, path }]
        };
        // Missing takes remain selectable: selecting one produces silence.
        assets.push(asset);
    }
    let mut counts = EditableCatalogCounts {
        assets: assets.len() as u64,
        music: 0,
        ambient: 0,
        voice: 0,
        sfx: 0,
    };
    for asset in &assets {
        match asset.category {
            CatalogCategory::Music => counts.music += 1,
            CatalogCategory::Ambient => counts.ambient += 1,
            CatalogCategory::Voice => counts.voice += 1,
            CatalogCategory::Sfx => counts.sfx += 1,
        }
    }
    Ok(EditableCatalogDocument {
        fallback_locale: "en".into(),
        locale_fallbacks: BTreeMap::new(),
        counts,
        assets,
    })
}

pub(super) fn numbered(value: &str) -> Option<(&str, &str)> {
    let start = value.trim_end_matches(|c: char| c.is_ascii_digit()).len();
    (start < value.len()).then(|| (&value[..start], &value[start..]))
}
