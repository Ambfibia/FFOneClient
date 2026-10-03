use super::*;

#[test]
fn accept_capacity_is_the_only_check_that_blocks_a_published_world_task() {
    let content = production_content();
    let world = content
        .missions()
        .filter(|definition| definition.mission_type == TutorialMissionType::World)
        .take(5)
        .collect::<Vec<_>>();
    assert_eq!(world.len(), 5);
    let mut runtime = WorldMissionRuntime::default();
    runtime.active_tasks = world[..4]
        .iter()
        .map(|definition| active_task(definition))
        .collect();
    assert!(matches!(
        runtime.check_accept_task(world[4].provenance.task_id, content),
        Err(WorldMissionAcceptRejection::CategoryFull {
            mission_type: TutorialMissionType::World,
            capacity: 4,
        })
    ));

    // Instanced and escort/defence contracts are ordinary acceptable
    // tasks. The shard owns the DEF NPC path and the leave-instance
    // failure, so failing them closed here only stranded the player: every
    // published Nano-mission chain ends on an instanced task.
    runtime.clear();
    let instance = content
        .missions()
        .find(|definition| definition.provenance.required_instance_id > 0)
        .expect("published catalog has an instance mission");
    assert_eq!(
        runtime.check_accept_task(instance.provenance.task_id, content),
        Ok(())
    );
    let escort = content
        .missions()
        .find(|definition| definition.provenance.task_type == 6)
        .expect("published catalog has an escort mission");
    assert_eq!(
        runtime.check_accept_task(escort.provenance.task_id, content),
        Ok(())
    );
}
