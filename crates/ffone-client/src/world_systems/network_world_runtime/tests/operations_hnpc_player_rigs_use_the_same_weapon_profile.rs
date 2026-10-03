use super::*;

#[test]
fn hnpc_player_rigs_use_the_same_weapon_profile_variants() {
    assert_eq!(network_hnpc_animation_clip("stand1", None), "stand1");
    assert_eq!(
        network_hnpc_animation_clip("stand1", Some(PlayerWeaponAnimationProfile::Pistol)),
        "stand1"
    );
    assert_eq!(
        network_hnpc_animation_clip("run", Some(PlayerWeaponAnimationProfile::Stick)),
        "stickrun"
    );
    assert_eq!(
        network_hnpc_animation_clip("ready", Some(PlayerWeaponAnimationProfile::Rifle)),
        "rifleready"
    );
    assert_eq!(
        network_hnpc_animation_clip("melee1", Some(PlayerWeaponAnimationProfile::Rocket)),
        "rocketattack1"
    );
    assert_eq!(
        network_hnpc_animation_clip("death", Some(PlayerWeaponAnimationProfile::Bomb)),
        "death"
    );
}

#[test]
fn hnpc_idle_slots_preserve_weights_and_avoid_repeating_alternates() {
    let clips = ["talk", "talkexclamation", "talkquestion"].map(str::to_owned);
    for (roll, expected) in [
        (0, "talk"),
        (33, "talk"),
        (34, "talkexclamation"),
        (66, "talkexclamation"),
        (67, "talkquestion"),
        (99, "talkquestion"),
    ] {
        assert_eq!(hnpc_idle_clip(Some(&clips), "stand1", roll), expected);
    }
    assert_eq!(hnpc_idle_clip(Some(&clips), "talkexclamation", 40), "talk");
    assert_eq!(hnpc_idle_clip(Some(&clips), "talkquestion", 90), "talk");
    assert_eq!(hnpc_idle_clip(None, "stand1", 40), "stand2");
    assert_eq!(hnpc_idle_clip(None, "stand2", 40), "stand1");
    assert_eq!(hnpc_idle_clip(None, "stand1", 70), "stand3");
    assert_eq!(hnpc_idle_clip(None, "stand1", 90), "stand4");
}
