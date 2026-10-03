use super::*;
use serde_json::json;

#[test]
fn evidence_paths_are_true_stem_sidecars() {
    let (json, png) = gpu_evidence_relative_paths("models/npc/npc_max.glb").unwrap();
    assert_eq!(json, PathBuf::from("models/npc/npc_max.gpu.json"));
    assert_eq!(png, PathBuf::from("models/npc/npc_max.gpu.png"));
}

#[test]
fn evidence_paths_reject_escape_absolute_and_legacy_formats() {
    for path in ["../npc.glb", "D:/npc.glb", "models/npc/npc.unity3d"] {
        assert!(gpu_evidence_relative_paths(path).is_err(), "{path}");
    }
}

#[test]
fn facts_reject_duplicate_exact_animation_names() {
    let document = json!({
        "asset": { "version": "2.0" },
        "scene": 0,
        "scenes": [{ "name": "npc_test", "nodes": [0] }],
        "nodes": [{ "name": "npc_test" }],
        "meshes": [],
        "accessors": [],
        "animations": [{ "name": "run" }, { "name": "run" }],
        "extras": { "logicalModelName": "npc_test" }
    });
    let mut json_bytes = serde_json::to_vec(&document).unwrap();
    while json_bytes.len() % 4 != 0 {
        json_bytes.push(b' ');
    }
    let total = 20 + json_bytes.len() + 8;
    let mut glb = Vec::new();
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2_u32.to_le_bytes());
    glb.extend_from_slice(&(total as u32).to_le_bytes());
    glb.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
    glb.extend_from_slice(&0x4e4f_534a_u32.to_le_bytes());
    glb.extend_from_slice(&json_bytes);
    glb.extend_from_slice(&0_u32.to_le_bytes());
    glb.extend_from_slice(&0x004e_4942_u32.to_le_bytes());
    let error = gpu_model_facts_from_glb(&glb).expect_err("duplicate names must fail");
    assert!(
        error
            .to_string()
            .contains("duplicate exact standard animation")
    );
}
