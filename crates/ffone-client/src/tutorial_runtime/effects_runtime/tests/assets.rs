use super::*;

#[test]
fn exact_world_serialized_effect_queues_without_catalog_identity() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let library = TutorialEffectLibrary::load(&root).expect("exact native effect library");
    let mut closure = library
        .effects
        .get(&594)
        .expect("world particle ES594 closure")
        .closure
        .clone();
    closure.effect_id = None;
    let mut runtime = TutorialEffectRuntime::default();
    let instance_id = runtime
        .enqueue_world_serialized_effect(
            closure,
            TutorialEffectPlacement::ExactEntityWorld {
                root_entity: Entity::PLACEHOLDER,
                position: Vec3::new(10.0, 20.0, 30.0),
                rotation: Quat::IDENTITY,
            },
            1.5,
        )
        .expect("world serialized effect native plan");

    assert!(runtime.active.contains_key(&instance_id));
    assert!(runtime.native_preloads.is_empty());
    assert_eq!(runtime.native_spawns.len(), 1);
    assert_eq!(
        runtime.active[&instance_id].stream_owner,
        Some(Entity::PLACEHOLDER)
    );
}

#[test]
fn missing_catalog_never_reports_success() {
    let mut runtime = TutorialEffectRuntime::default();
    runtime.enqueue(TutorialEffectRuntimeCommand::Preload {
        effect_id: 767,
        source_line: 2451,
    });
    runtime.process_pending();
    assert!(matches!(
        runtime.drain_issues().next(),
        Some(TutorialEffectRuntimeIssue::ExactCatalogUnavailable { source_line: 2451 })
    ));
    assert!(!runtime.is_serialized_closure_preloaded(767));
    assert!(runtime.is_native_preload_complete(767));
}

#[test]
fn effect_outside_loaded_catalog_is_terminal_without_claiming_preload_success() {
    let mut runtime = TutorialEffectRuntime::with_library(library());
    runtime.enqueue(TutorialEffectRuntimeCommand::Preload {
        effect_id: 999_999,
        source_line: 3065,
    });
    runtime.process_pending();

    assert!(!runtime.is_serialized_closure_preloaded(999_999));
    assert!(runtime.is_native_preload_complete(999_999));
    assert!(matches!(
        runtime.drain_issues().next(),
        Some(TutorialEffectRuntimeIssue::EffectOutsideExactCatalog {
            effect_id: 999_999,
            source_line: 3065,
        })
    ));
}
