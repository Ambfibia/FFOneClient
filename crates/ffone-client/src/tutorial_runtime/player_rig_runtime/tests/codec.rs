use super::*;

#[test]
fn same_frame_layer_requests_keep_only_the_final_base_pose() {
    let mut base = None;
    let mut upper = None;
    coalesce_legacy_layer_animation(
        &mut base,
        &mut upper,
        LegacyAnimationLayer::FullBody,
        CoalescedLegacyLayerAnimation {
            semantic_clip: LegacyVisualClip::AttackFull(1),
            animation_clip: TutorialPlayerClip::RifleAttack1,
            blend_seconds: 0.15,
        },
    );
    coalesce_legacy_layer_animation(
        &mut base,
        &mut upper,
        LegacyAnimationLayer::UpperBody,
        CoalescedLegacyLayerAnimation {
            semantic_clip: LegacyVisualClip::AttackUpper(1),
            animation_clip: TutorialPlayerClip::RifleAttack1Upper,
            blend_seconds: 0.15,
        },
    );
    coalesce_legacy_layer_animation(
        &mut base,
        &mut upper,
        LegacyAnimationLayer::FullBody,
        CoalescedLegacyLayerAnimation {
            semantic_clip: LegacyVisualClip::Run,
            animation_clip: TutorialPlayerClip::RifleRun,
            blend_seconds: 0.15,
        },
    );

    assert_eq!(base.unwrap().semantic_clip, LegacyVisualClip::Run);
    assert_eq!(
        upper.unwrap().semantic_clip,
        LegacyVisualClip::AttackUpper(1)
    );
}
