use super::*;

pub(super) fn player_weapon_attack_event_seconds(
    profile: PlayerWeaponAnimationProfile,
    gender: PlayerRigGender,
) -> f32 {
    // Primary CharacterSelection.resourceFile `attacksound` AnimationEvents:
    // stickattack1upper M 34412 / F 34315, pistol M 34240 / F 34286,
    // rifle M 34305 / F 34466, bomb M 34436 / F 34542, and rocket
    // M 34210 / F 34199. Keep the original float32 event times instead of
    // snapping them to an arbitrary common delay.
    match (profile, gender) {
        (PlayerWeaponAnimationProfile::Stick, PlayerRigGender::Male) => 0.296,
        (PlayerWeaponAnimationProfile::Stick, PlayerRigGender::Female) => 0.392,
        (PlayerWeaponAnimationProfile::Rocket, PlayerRigGender::Female) => 0.293_332_994,
        (
            PlayerWeaponAnimationProfile::Pistol
            | PlayerWeaponAnimationProfile::Rifle
            | PlayerWeaponAnimationProfile::Bomb
            | PlayerWeaponAnimationProfile::Rocket,
            _,
        ) => 0.25,
    }
}
