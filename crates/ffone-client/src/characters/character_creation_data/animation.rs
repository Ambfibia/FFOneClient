use super::*;

pub(super) fn player_rig_gender(gender: DataGender) -> PlayerRigGender {
    match gender {
        DataGender::Male => PlayerRigGender::Male,
        DataGender::Female => PlayerRigGender::Female,
    }
}
