use super::*;

#[test]
fn every_local_player_status_effect_has_a_complete_native_plan() {
    let asset_root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let library = TutorialEffectLibrary::load(asset_root).unwrap();
    let mut missing = Vec::new();
    for effect_id in ffone_runtime_contracts::RETROBUTION_PLAYER_STATUS_EFFECT_IDS {
        let compiled = compile_effect_plan(effect_id, library.effects.get(&effect_id).unwrap());
        let rendered_nodes = compiled.plan.as_ref().map_or(0, |plan| plan.rendered_nodes);
        if rendered_nodes == 0 || !compiled.blockers.is_empty() {
            missing.push((effect_id, compiled.blockers));
        }
    }
    assert!(
        missing.is_empty(),
        "every clean local-player status effect must have a complete native plan: {missing:#?}"
    );
}
