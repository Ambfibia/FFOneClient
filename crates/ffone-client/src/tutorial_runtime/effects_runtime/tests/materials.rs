use super::*;

#[test]
fn every_npc_game_icon_has_a_complete_native_render_plan() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let mut runtime = TutorialEffectRuntime::with_library(
        TutorialEffectLibrary::load(&root).expect("exact NPC game-icon effect library"),
    );
    let mut missing_instances = Vec::new();
    for effect_id in RETROBUTION_NPC_GAME_ICON_EFFECT_IDS {
        let name = format!("test NPC game icon {effect_id}");
        runtime.enqueue(TutorialEffectRuntimeCommand::Add {
            effect_id,
            placement: TutorialEffectPlacement::World {
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
            },
            scale: 1.0,
            tracked: false,
            name: Some(name.clone()),
            destroy_after_seconds: None,
            source_line: 187,
        });
        runtime.process_pending();
        if !runtime.has_named_native_instance(&name) {
            missing_instances.push(effect_id);
        }
    }
    let issues = runtime.drain_issues().collect::<Vec<_>>();
    assert!(
        missing_instances.is_empty(),
        "NPC game icons without native instances: {missing_instances:?}; blockers={issues:#?}"
    );
    assert!(issues.is_empty(), "NPC game-icon blockers: {issues:#?}");
}
