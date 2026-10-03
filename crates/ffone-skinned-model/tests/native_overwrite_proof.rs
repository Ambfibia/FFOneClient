use ffone_skinned_model::{
    AnimationCurveRecoveryReference, SerializedCurveOverwriteProof, SerializedCurveOverwriteRule,
};

#[test]
fn overwrite_proof_publishes_native_order_without_legacy_object_identity() {
    let original = serde_json::json!({
        "rule": "later-serialized-binding-overwrites-earlier",
        "asset": "old-source-container",
        "pathId": 1181,
        "clipName": "skill1",
        "exactTargetPath": true,
        "exactSourceArrayOrder": true
    });
    let proof: SerializedCurveOverwriteProof = serde_json::from_value(original).unwrap();
    // The Editor can still validate its source identity before native publication.
    assert_eq!(proof.path_id, 1181);
    let native = serde_json::to_value(&proof).unwrap();
    assert!(native.get("asset").is_none());
    assert!(native.get("pathId").is_none());
    let decoded: SerializedCurveOverwriteProof = serde_json::from_value(native).unwrap();
    assert_eq!(
        decoded.rule,
        SerializedCurveOverwriteRule::LaterSerializedBindingOverwritesEarlier
    );
    assert_eq!(decoded.clip_name, "skill1");
    assert!(decoded.exact_target_path && decoded.exact_source_array_order);
}

#[test]
fn recovery_reference_uses_clip_and_curve_identity_in_native_data() {
    let original = serde_json::json!({
        "asset": "old-source-container", "pathId": 414,
        "clipName": "stand3", "field": "m_PositionCurves",
        "kind": "translation", "path": "Bip01/Spine",
        "sourceEncoding": "plain", "sourceIndex": 3, "sampleRate": 30.0
    });
    let reference: AnimationCurveRecoveryReference = serde_json::from_value(original).unwrap();
    assert_eq!(reference.path_id, 414);
    let native = serde_json::to_value(&reference).unwrap();
    assert!(native.get("asset").is_none());
    assert!(native.get("pathId").is_none());
    let decoded: AnimationCurveRecoveryReference = serde_json::from_value(native).unwrap();
    assert_eq!(decoded.clip_name, "stand3");
    assert_eq!(decoded.path, "Bip01/Spine");
    assert_eq!(decoded.source_index, 3);
}
