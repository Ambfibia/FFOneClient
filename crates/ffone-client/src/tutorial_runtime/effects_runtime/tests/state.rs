use super::*;

#[test]
fn scene_cleanup_despawns_instances_but_retains_preload_state() {
    let mut runtime = TutorialEffectRuntime::with_library(library());
    runtime.preloaded.insert(372);
    runtime.mark_native_preload_complete(372);
    let effect_name = "tutorial scene effect";
    let instance_id = runtime.allocate_instance(false, Some(effect_name.to_owned()));

    runtime.clear_scene_instances();

    assert_eq!(runtime.active_native_instance_count(), 0);
    assert!(!runtime.has_named_native_instance(effect_name));
    assert_eq!(runtime.native_despawns.pop_front(), Some(instance_id));
    assert!(runtime.is_native_preload_complete(372));
}
