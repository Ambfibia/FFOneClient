use super::*;

#[test]
fn every_tutorial_weapon_uses_its_published_exact_impact_sound_route() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let library = TutorialEffectLibrary::load(&root).expect("exact tutorial effect library");
    let audio_catalog = crate::semantic_audio::NativeAudioCatalog::open(&root, false)
        .expect("native semantic audio catalog");
    for row in &library.projectile_catalog().rows {
        let success_sound = row.parameters.success_sound.as_str();
        if matches!(success_sound, "\"" | "......") {
            assert!(exact_tutorial_success_sound_path(success_sound).is_none());
            continue;
        }
        let relative = exact_tutorial_success_sound_path(success_sound).unwrap_or_else(|| {
            panic!(
                "bullet {} has no native path for exact success sound {success_sound:?}",
                row.bullet_type
            )
        });
        let logical = relative.strip_prefix("audio/").unwrap_or(relative);
        assert!(
            root.join(relative).is_file(),
            "bullet {} sound is not published at {logical}",
            row.bullet_type
        );
        let audio = audio_catalog.by_legacy_path(relative).unwrap_or_else(|| {
            panic!(
                "bullet {} exact success sound {success_sound:?} has no semantic route {relative:?}",
                row.bullet_type
            )
        });
        assert_eq!(
            audio.category,
            crate::semantic_audio::NativeAudioCategory::Sfx,
            "bullet {} success sound must route through the SFX mixer",
            row.bullet_type
        );
    }
}
