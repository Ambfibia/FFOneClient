use super::*;

#[test]
fn unsupported_or_mixed_source_curves_block_lossy_publish() {
    let mut model = rex();
    model.animations[0]
        .metadata
        .unsupported
        .push(UnsupportedAnimationBinding {
            kind: "mixed-tangent-curve".into(),
            target_path: "Rex/Bip01".into(),
            property: "localRotation".into(),
            reason: "mixed legacy modes cannot be represented exactly".into(),
        });
    assert!(
        encode_glb(&model)
            .unwrap_err()
            .to_string()
            .contains("refusing lossy publish")
    );
}
