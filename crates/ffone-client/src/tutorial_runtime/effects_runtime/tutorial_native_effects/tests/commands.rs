use super::*;

#[test]
fn warp_departure_effect_tail_outlives_the_original_request_delay() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let library = TutorialEffectLibrary::load(root).unwrap();
    let effect = library.effects.get(&394).unwrap();
    let plan = compile_effect_plan(394, effect).plan.unwrap();
    assert!(plan.mesh_scene.is_none(), "ES394 is a particle effect");
    assert!(!plan.disable_update);
    assert_eq!(plan.maximum_timer, 1.0);
    assert_eq!(plan.longest_lifetime, 1.0);
    assert!(plan.maximum_timer + plan.longest_lifetime > 1.5);
}

#[test]
fn particle_only_roots_do_not_request_world_surface_scans() {
    let root = NativeEffectRoot {
        stream_owner: None,
        age: 0.0,
        destroy_after: None,
        natural_destroy_after: None,
        material_animation: None,
        waiting_for_mesh_surface: false,
    };

    assert!(!root.requires_mesh_surface_preparation());
}
