use std::collections::BTreeMap;

use crate::character_scene::*;

#[test]
fn only_character_container_turns_and_neither_role_changes_origin_or_scale() {
    let world = native_scene_container_transform(NativeSceneRole::WorldAuthored);
    assert_eq!(world, Transform::IDENTITY);

    let character = native_scene_container_transform(NativeSceneRole::CharacterGameplay);
    assert_eq!(character.translation, Vec3::ZERO);
    assert_eq!(character.scale, Vec3::ONE);
    assert!((character.rotation * Vec3::Z).abs_diff_eq(Vec3::NEG_Z, 0.000_01));
}

#[test]
fn exact_named_root_search_records_every_intermediary_wrapper() {
    let scene = Entity::from_raw_u32(1).expect("valid test entity row");
    let wrapper = Entity::from_raw_u32(2).expect("valid test entity row");
    let exact = Entity::from_raw_u32(3).expect("valid test entity row");
    let similar = Entity::from_raw_u32(4).expect("valid test entity row");
    let mut children =
        BTreeMap::from([(scene, vec![wrapper, similar]), (wrapper, vec![exact])]);
    let names = BTreeMap::from([(exact, "nano_coco"), (similar, "nano_coco(Clone)")]);

    let matches = collect_topmost_exact_named_descendants(
        scene,
        |entity| children.remove(&entity).unwrap_or_default(),
        |entity| names.get(&entity).is_some_and(|name| *name == "nano_coco"),
    );
    assert_eq!(
        matches,
        vec![ExactNamedRootCandidate {
            root: exact,
            wrappers: vec![wrapper],
        }]
    );
}

#[test]
fn duplicate_true_names_are_reportable_instead_of_order_dependent() {
    let scene = Entity::from_raw_u32(1).expect("valid test entity row");
    let first = Entity::from_raw_u32(2).expect("valid test entity row");
    let second = Entity::from_raw_u32(3).expect("valid test entity row");
    let mut children = BTreeMap::from([(scene, vec![first, second])]);

    let matches = collect_topmost_exact_named_descendants(
        scene,
        |entity| children.remove(&entity).unwrap_or_default(),
        |entity| entity == first || entity == second,
    );
    assert_eq!(matches.len(), 2);
}

#[test]
fn nested_reuse_of_true_name_does_not_compete_with_the_topmost_root() {
    let scene = Entity::from_raw_u32(1).expect("valid test entity row");
    let logical_root = Entity::from_raw_u32(2).expect("valid test entity row");
    let nested_mesh = Entity::from_raw_u32(3).expect("valid test entity row");
    let mut children = BTreeMap::from([
        (scene, vec![logical_root]),
        (logical_root, vec![nested_mesh]),
    ]);

    let matches = collect_topmost_exact_named_descendants(
        scene,
        |entity| children.remove(&entity).unwrap_or_default(),
        |entity| entity == logical_root || entity == nested_mesh,
    );
    assert_eq!(
        matches,
        vec![ExactNamedRootCandidate {
            root: logical_root,
            wrappers: Vec::new(),
        }]
    );
}

#[test]
fn only_identity_scene_wrappers_are_accepted() {
    assert!(is_identity_scene_wrapper(Transform::IDENTITY));
    assert!(!is_identity_scene_wrapper(Transform::from_translation(
        Vec3::new(0.25, 0.0, 0.0)
    )));
    assert!(!is_identity_scene_wrapper(Transform::from_scale(
        Vec3::splat(1.01)
    )));
    assert!(!is_identity_scene_wrapper(Transform::from_rotation(
        Quat::from_rotation_y(0.01)
    )));
}

#[test]
fn an_external_reveal_gate_survives_successful_root_normalization() {
    assert!(matches!(
        normalized_character_scene_visibility(true),
        Visibility::Hidden
    ));
    assert!(matches!(
        normalized_character_scene_visibility(false),
        Visibility::Inherited
    ));
}
