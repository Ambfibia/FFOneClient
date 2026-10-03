use super::*;

#[test]
fn direct_character_model_rotation_is_converted_to_the_native_gameplay_root() {
    let model_rotation = Quat::from_euler(EulerRot::XYZ, 0.3, -1.1, 0.2);
    let root_rotation = choreography_character_root_rotation(model_rotation);
    let rendered_forward = root_rotation * native_model_forward_child_rotation() * Vec3::Z;
    assert!(rendered_forward.abs_diff_eq(model_rotation * Vec3::Z, 0.000_01));
}

#[test]
fn character_model_contract_rejects_unsafe_paths() {
    for path in [
        "../outside.glb",
        "/absolute.glb",
        "models\\npc.glb",
        "models/npc.gltf",
    ] {
        assert!(
            ClientConfig::from_args(["--character-model".to_owned(), path.to_owned(),]).is_err(),
            "unsafe or non-GLB character model path {path:?} was accepted"
        );
    }
}
