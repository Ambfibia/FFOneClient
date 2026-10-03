use super::*;

pub(super) fn skyway_socket_full_path(gender: PlayerRigGender) -> String {
    let root = match gender {
        PlayerRigGender::Male => "m",
        PlayerRigGender::Female => "w",
    };
    format!("{root}/Bip01/Bip01 NonAccum/Bip01 Pelvis/Bip01 Broomstick")
}

#[must_use]
pub fn tutorial_player_weapon_socket_full_path(gender: PlayerRigGender) -> String {
    let root = match gender {
        PlayerRigGender::Male => "m",
        PlayerRigGender::Female => "w",
    };
    format!(
        "{root}/{}",
        LegacyPlayerAttachmentSlot::RightPistol.socket_path()
    )
}
