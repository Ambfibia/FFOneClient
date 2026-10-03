use super::*;

#[test]
fn unarmed_and_each_tutorial_weapon_resolve_their_original_attack_prefix() {
    assert_eq!(
        legacy_visual_tutorial_clip(LegacyVisualClip::AttackFull(1), None, false),
        Some(TutorialPlayerClip::Attack1)
    );
    assert_eq!(
        legacy_visual_tutorial_clip(LegacyVisualClip::AttackUpper(1), None, false),
        Some(TutorialPlayerClip::Attack1Upper)
    );
    assert_eq!(
        legacy_visual_tutorial_clip(
            LegacyVisualClip::Stand1,
            Some(PlayerWeaponAnimationProfile::Rifle),
            true,
        ),
        Some(TutorialPlayerClip::Stand1),
        "tutorial StandName must not gain the rifle prefix"
    );
    assert_eq!(
        legacy_visual_tutorial_clip(
            LegacyVisualClip::Run,
            Some(PlayerWeaponAnimationProfile::Rifle),
            true,
        ),
        Some(TutorialPlayerClip::RifleRun)
    );
    assert_eq!(
        legacy_visual_tutorial_clip(
            LegacyVisualClip::AttackFull(3),
            Some(PlayerWeaponAnimationProfile::Rifle),
            true,
        ),
        Some(TutorialPlayerClip::RifleAttack1)
    );
    assert_eq!(
        legacy_visual_tutorial_clip(
            LegacyVisualClip::AttackUpper(3),
            Some(PlayerWeaponAnimationProfile::Rifle),
            true,
        ),
        Some(TutorialPlayerClip::RifleAttack1Upper)
    );
    assert_eq!(
        legacy_visual_tutorial_clip(
            LegacyVisualClip::AttackFull(3),
            Some(PlayerWeaponAnimationProfile::Stick),
            true,
        ),
        Some(TutorialPlayerClip::StickAttack1)
    );
    assert_eq!(
        legacy_visual_tutorial_clip(
            LegacyVisualClip::AttackUpper(3),
            Some(PlayerWeaponAnimationProfile::Stick),
            true,
        ),
        Some(TutorialPlayerClip::StickAttack1Upper)
    );
    assert_eq!(
        legacy_visual_tutorial_clip(
            LegacyVisualClip::AttackFull(3),
            Some(PlayerWeaponAnimationProfile::Pistol),
            true,
        ),
        Some(TutorialPlayerClip::PistolAttack1)
    );
    assert_eq!(
        legacy_visual_tutorial_clip(
            LegacyVisualClip::AttackUpper(3),
            Some(PlayerWeaponAnimationProfile::Pistol),
            true,
        ),
        Some(TutorialPlayerClip::PistolAttack1Upper)
    );
    assert_eq!(
        legacy_visual_tutorial_clip(
            LegacyVisualClip::Stand1,
            Some(PlayerWeaponAnimationProfile::Stick),
            false,
        ),
        Some(TutorialPlayerClip::StickStand1)
    );
    assert_eq!(
        legacy_visual_tutorial_clip(
            LegacyVisualClip::Stand1,
            Some(PlayerWeaponAnimationProfile::Pistol),
            false,
        ),
        Some(TutorialPlayerClip::PistolStand1)
    );
    assert_eq!(
        legacy_visual_tutorial_clip(
            LegacyVisualClip::Stand1,
            Some(PlayerWeaponAnimationProfile::Rifle),
            false,
        ),
        Some(TutorialPlayerClip::RifleStand1)
    );
    assert_eq!(
        legacy_visual_tutorial_clip(
            LegacyVisualClip::Stand1,
            Some(PlayerWeaponAnimationProfile::Bomb),
            false,
        ),
        Some(TutorialPlayerClip::BombStand1)
    );
    assert_eq!(
        legacy_visual_tutorial_clip(
            LegacyVisualClip::AttackFull(3),
            Some(PlayerWeaponAnimationProfile::Rocket),
            false,
        ),
        Some(TutorialPlayerClip::RocketAttack1)
    );
    for alpha in [0.25_f32, 0.5, 0.75] {
        let relative = normalized_upper_layer_weight(alpha);
        assert!((relative / (1.0 + relative) - alpha).abs() <= f32::EPSILON);
    }
    assert_eq!(tutorial_upper_layer_alpha(0.0, 0.15, false), (0.0, false));
    assert_eq!(tutorial_upper_layer_alpha(0.075, 0.15, false), (0.5, false));
    assert_eq!(tutorial_upper_layer_alpha(0.0, 0.15, true), (1.0, false));
    assert_eq!(tutorial_upper_layer_alpha(0.075, 0.15, true), (0.5, false));
    assert_eq!(tutorial_upper_layer_alpha(0.15, 0.15, true), (0.0, true));
}
