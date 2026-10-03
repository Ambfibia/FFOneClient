use super::*;

#[test]
fn mission_ui_and_loaded_tutorial_content_resolve_in_en_and_ru() {
    let root = asset_root();
    let locator = AssetLocator::open(&root).unwrap();
    let content = TutorialMissionContent::open(&locator).unwrap();
    let (localization, mut language) = Localization::open(&root, "en").unwrap();

    for (source, _) in MISSION_UI_SOURCE_KEYS {
        let localized = mission_ui_localized_text(*source);
        assert_eq!(localization.text(&language, &localized), *source);
    }

    let mut mission_texts = Vec::new();
    for task_id in TUTORIAL_MISSION_TASK_IDS {
        let mission = content.mission(task_id).unwrap();
        for (field, fallback) in [
            ("title", mission.title.as_str()),
            ("objective", mission.objective.as_str()),
            (
                "offer_description",
                mission.journal.offer_description.as_str(),
            ),
            (
                "task_description",
                mission.journal.active_task_description.as_str(),
            ),
            ("mission_summary", mission.journal.mission_summary.as_str()),
            (
                "mission_complete_summary",
                mission.journal.mission_complete_summary.as_str(),
            ),
            (
                "completion_description",
                mission.journal.completion_description.as_str(),
            ),
        ] {
            let localized = mission_content_text(task_id, field, fallback);
            assert_eq!(localization.text(&language, &localized), fallback);
            mission_texts.push((localized, fallback.to_owned()));
        }
    }

    let nano = content.journal_entry(2250, "Pokey Oaks North").unwrap();
    let nano = nano.nano.expect("task 2250 owns Buttercup Nano content");
    let mut nano_texts = vec![
        nano_content_text(nano.nano_id, "name", &nano.name),
        nano_content_text(nano.nano_id, "attribute", &nano.attribute),
        nano_content_text(nano.nano_id, "description", &nano.description),
    ];
    for skill in &nano.skills {
        nano_texts.extend([
            nano_skill_content_text(skill.tune_id, "name", &skill.name),
            nano_skill_content_text(skill.tune_id, "type_label", &skill.type_label),
            nano_skill_content_text(skill.tune_id, "description", &skill.description),
        ]);
    }
    localization.select(&mut language, "ru");
    for (localized, fallback) in mission_texts {
        let translated = localization.text(&language, &localized);
        assert_ne!(translated, fallback, "{} stayed English", localized.key);
    }
    for localized in nano_texts {
        let translated = localization.text(&language, &localized);
        assert_ne!(
            translated, localized.fallback,
            "{} stayed English",
            localized.key
        );
    }
}
