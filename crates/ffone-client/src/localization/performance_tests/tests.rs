use crate::localization::{
    Language, Localization, LocalizedText, LocalizedTextCase, apply_localized_texts,
};
use bevy::prelude::*;
use std::collections::BTreeMap;

fn app() -> App {
    let mut app = App::new();
    app.insert_resource(Localization {
        fallback: "en".into(),
        bundles: BTreeMap::from([
            (
                "en".into(),
                BTreeMap::from([("test.value".into(), "Value {value}".into())]),
            ),
            (
                "ru".into(),
                BTreeMap::from([("test.value".into(), "Значение {value}".into())]),
            ),
        ]),
        fallback_keys_by_source: BTreeMap::new(),
    })
    .insert_resource(Language {
        requested: "en".into(),
        effective: "en".into(),
    });
    app
}

#[test]
fn incremental_resolution_keeps_arguments_casing_removal_and_external_writes_live() {
    let mut app = app();
    app.add_systems(Update, apply_localized_texts);
    let entity = app
        .world_mut()
        .spawn((
            Text::new(""),
            LocalizedText::new("test.value", "Value {value}").with_arg("value", "one"),
        ))
        .id();
    app.update();
    app.update();
    assert_eq!(app.world().get::<Text>(entity).unwrap().0, "Value one");
    app.world_mut()
        .get_mut::<LocalizedText>(entity)
        .unwrap()
        .args
        .insert("value".into(), "two".into());
    app.update();
    assert_eq!(app.world().get::<Text>(entity).unwrap().0, "Value two");
    app.world_mut()
        .entity_mut(entity)
        .insert(LocalizedTextCase::Uppercase);
    app.update();
    assert_eq!(app.world().get::<Text>(entity).unwrap().0, "VALUE TWO");
    app.world_mut()
        .entity_mut(entity)
        .remove::<LocalizedTextCase>();
    app.update();
    assert_eq!(app.world().get::<Text>(entity).unwrap().0, "Value two");
    app.world_mut().get_mut::<Text>(entity).unwrap().0 = "external stale label".into();
    app.update();
    assert_eq!(app.world().get::<Text>(entity).unwrap().0, "Value two");
    app.world_mut().resource_mut::<Language>().effective = "ru".into();
    app.update();
    assert_eq!(app.world().get::<Text>(entity).unwrap().0, "Значение two");
    app.world_mut()
        .resource_mut::<Localization>()
        .bundles
        .get_mut("ru")
        .unwrap()
        .insert("test.value".into(), "Обновлено {value}".into());
    app.update();
    assert_eq!(app.world().get::<Text>(entity).unwrap().0, "Обновлено two");
}

/// Manual comparison with the previous production loop. Timing has no flaky
/// pass/fail threshold; the ordinary test above owns behavior correctness.
#[test]
#[ignore = "explicit CPU microbenchmark; run with --ignored --nocapture"]
fn localization_idle_frame_benchmark() {
    fn old_loop(
        localization: Res<Localization>,
        language: Res<Language>,
        mut texts: Query<(&LocalizedText, &mut Text)>,
    ) {
        for (localized, mut text) in &mut texts {
            let resolved = localization.text(&language, localized);
            if text.0 != resolved {
                text.0 = resolved;
            }
        }
    }
    for count in [1000, 5000] {
        for incremental in [false, true] {
            let mut app = app();
            if incremental {
                app.add_systems(Update, apply_localized_texts);
            } else {
                app.add_systems(Update, old_loop);
            }
            for index in 0..count {
                app.world_mut().spawn((
                    Text::new(""),
                    LocalizedText::new("test.value", "Value {value}")
                        .with_arg("value", index.to_string()),
                ));
            }
            for _ in 0..10 {
                app.update();
            }
            let start = std::time::Instant::now();
            for _ in 0..300 {
                app.update();
            }
            println!(
                "localization count={count} incremental={incremental} mean_ms={:.4}",
                start.elapsed().as_secs_f64() * 1000.0 / 300.0
            );
        }
    }
}
