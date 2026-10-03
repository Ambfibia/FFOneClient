use super::*;

#[test]
fn computress_comm_out_resolves_through_catalog_for_en_and_ru() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets")
        .join("game");
    let catalog = NativeAudioCatalog::open(&root, false).unwrap();
    for true_name in [
        "Computress_CommOut01",
        "Computress_CommOut02",
        "Computress_CommOut03",
    ] {
        let assets = catalog
            .by_true_name(true_name)
            .into_iter()
            .filter(|asset| asset.category == NativeAudioCategory::Voice)
            .collect::<Vec<_>>();
        let [asset] = assets.as_slice() else {
            panic!("{true_name} must resolve to one semantic voice asset")
        };
        for locale in ["en", "ru"] {
            assert!(
                root.join(catalog.path_for_locale(asset, locale).unwrap())
                    .is_file(),
                "{true_name} must resolve for {locale} through catalog fallback"
            );
        }
    }
}
