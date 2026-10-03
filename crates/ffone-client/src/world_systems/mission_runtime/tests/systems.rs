use super::*;

#[test]
fn minimap_advance_matches_an_active_terminator_before_objectives_complete() {
    let content = production_content();
    let definition = content.mission(2_254).expect("primary tutorial hunt task");
    let runtime = WorldMissionRuntime {
        active_tasks: vec![active_task(definition)],
        ..WorldMissionRuntime::default()
    };
    assert!(!runtime.can_complete_task(definition, &[]));
    let (_, advance) = runtime.npc_has_available_or_completable_mission(
        definition.provenance.terminator_npc_type,
        100,
        0,
        &BTreeSet::new(),
        &[],
        content,
    );
    assert!(advance);
}
