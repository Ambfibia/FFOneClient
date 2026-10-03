use super::*;

pub(super) struct Fixture {
    pub(super) _temp: TempDir,
    pub(super) assets: AssetLocator,
    pub(super) table_path: PathBuf,
}

impl Fixture {
    pub(super) fn new(document: Value) -> Self {
        Self::build(document, false, &[])
    }

    pub(super) fn with_icon_paths(document: Value, icon_paths: &[&str]) -> Self {
        Self::build(document, false, icon_paths)
    }

    pub(super) fn with_duplicate_table_route(document: Value) -> Self {
        Self::build(document, true, &[])
    }

    pub(super) fn build(document: Value, duplicate_table_route: bool, icon_paths: &[&str]) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let relative = TABLE_SET_PATH;
        let table_path = temp.path().join(relative);
        fs::create_dir_all(table_path.parent().unwrap()).unwrap();
        let bytes = serde_json::to_vec(&document).unwrap();
        fs::write(&table_path, &bytes).unwrap();
        if duplicate_table_route {
            // Obsolete content-addressed routes no longer participate in
            // resolution; the one stable table path remains authoritative.
            let second_relative = "data/tables/table-set--obsolete.json";
            fs::write(temp.path().join(second_relative), &bytes).unwrap();
        }
        for &icon_path in icon_paths {
            let icon_bytes = format!("fixture texture at {icon_path}").into_bytes();
            let absolute = temp.path().join(icon_path);
            fs::create_dir_all(absolute.parent().unwrap()).unwrap();
            fs::write(&absolute, &icon_bytes).unwrap();
        }
        // The production locator now owns semantic icons below `icons/`.
        // Keep the compact table fixtures structurally valid even when a
        // test intentionally provides no icon payloads.
        fs::create_dir_all(temp.path().join("icons")).unwrap();
        let assets = AssetLocator::open(temp.path()).unwrap();
        Self {
            _temp: temp,
            assets,
            table_path,
        }
    }
}
