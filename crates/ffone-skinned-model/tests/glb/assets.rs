use super::*;

#[test]
fn canonical_path_keeps_only_the_true_name() {
    assert_eq!(
        model_relative_path("npc", &["heroes"], "Rex").unwrap(),
        PathBuf::from("models/npc/heroes/Rex.glb")
    );
    assert_eq!(
        model_relative_path("npc", &["heroes"], "Fusion: Finn.").unwrap(),
        PathBuf::from("models/npc/heroes/Fusion_ Finn.glb")
    );
    assert_eq!(
        model_relative_path("npc", &[], "CON").unwrap(),
        PathBuf::from("models/npc/_CON.glb")
    );
    assert!(model_relative_path("npc", &[], "Rex--b7f8cca850c45cf0").is_err());
    assert!(model_relative_path("npc", &[], "Mesh#123").is_err());
}
