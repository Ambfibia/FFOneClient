use super::*;

pub(super) fn insert_test_localization(app: &mut App, asset_root: &Path) {
    let (localization, language) =
        Localization::open(asset_root, "en").expect("test localization catalog must load");
    app.insert_resource(localization).insert_resource(language);
}
