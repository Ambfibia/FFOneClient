use super::*;

#[test]
fn exact_weapon_socket_is_gender_rooted_right_pistol_path() {
    for (gender, root) in [
        (PlayerRigGender::Male, "m/"),
        (PlayerRigGender::Female, "w/"),
    ] {
        let path = tutorial_player_weapon_socket_full_path(gender);
        assert!(path.starts_with(root));
        assert!(path.ends_with("Bip01 Rweapon01"));
        assert!(path.contains(LegacyPlayerAttachmentSlot::RightPistol.socket_path()));
    }
}

#[test]
fn completed_tutorial_emote_releases_to_the_idle_locomotion_path() {
    assert!(tutorial_emote_holds_locomotion(true, false, false, false));
    assert!(!tutorial_emote_holds_locomotion(true, false, true, false));
    assert!(!tutorial_emote_holds_locomotion(true, true, false, false));
    assert!(!tutorial_emote_holds_locomotion(false, false, false, false));
    assert!(!tutorial_emote_interrupt_requested(0, false));
    assert!(tutorial_emote_interrupt_requested(1, false));
    assert!(tutorial_emote_interrupt_requested(0, true));
}
