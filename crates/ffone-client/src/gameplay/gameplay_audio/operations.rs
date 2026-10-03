use super::*;

pub(super) fn actor_clips(npc_type: i32) -> Option<&'static [ActorClipAudio]> {
    match npc_type {
        2666 => Some(SAMURAI_JACK_CLIPS),
        2667 => Some(NUMBUH_ONE_CLIPS),
        2668 => Some(DEXTER_PISTOL_CLIPS),
        2669 => Some(NUMBUH_FIVE_CLIPS),
        2670 => Some(BEN_CLIPS),
        2673 => Some(DEXTER_CLIPS),
        2674 | 2897 => Some(SPAWN_CLIPS),
        2675 => Some(CERBERUS_CLIPS),
        2676 => Some(OIL_CLIPS),
        2677 => Some(BAT_CLIPS),
        2678 => Some(BUTTERCUP_CLIPS),
        2902 => Some(DEXTER_SWORD_CLIPS),
        _ => None,
    }
}

pub(super) fn player_damage_cue(gender: PlayerRigGender, critical: bool) -> SoundCue {
    match (gender, critical) {
        (PlayerRigGender::Male, false) => SoundCue::Random(&["M_Avatar_Hurt01", "M_Avatar_Hurt02"]),
        (PlayerRigGender::Female, false) => {
            SoundCue::Random(&["F_Avatar_Hurt01", "F_Avatar_Hurt02"])
        }
        (PlayerRigGender::Male, true) => {
            SoundCue::Random(&["M_Avatar_Critical01", "M_Avatar_Critical02"])
        }
        (PlayerRigGender::Female, true) => {
            SoundCue::Random(&["F_Avatar_Critical01", "F_Avatar_Critical02"])
        }
    }
}

pub(super) fn player_infection_damage_cue(gender: PlayerRigGender) -> SoundCue {
    match gender {
        PlayerRigGender::Male => SoundCue::Random(&["M_Avatar_GooDmg01", "M_Avatar_GooDmg02"]),
        PlayerRigGender::Female => SoundCue::Random(&["F_Avatar_GooDmg01", "F_Avatar_GooDmg02"]),
    }
}

pub(super) fn semantic_numeric_family<'a>(
    catalog: &'a NativeAudioCatalog,
    requested_true_name: &str,
) -> Vec<&'a crate::semantic_audio::NativeAudioAsset> {
    let folded = requested_true_name.to_ascii_lowercase();
    let prefix_len = folded
        .trim_end_matches(|character: char| character.is_ascii_digit())
        .len();
    if prefix_len == folded.len() || prefix_len == 0 {
        return Vec::new();
    }
    let prefix = &requested_true_name[..prefix_len];
    let suffix_width = requested_true_name.len() - prefix_len;
    let mut siblings: Vec<&crate::semantic_audio::NativeAudioAsset> = Vec::new();
    for take in 0..=99 {
        let candidate = format!("{prefix}{take:0suffix_width$}");
        for asset in catalog.by_true_name(&candidate) {
            if !siblings.iter().any(|sibling| std::ptr::eq(*sibling, asset)) {
                siblings.push(asset);
            }
        }
    }
    siblings
}

pub(super) fn player_locomotion_cue(
    gender: PlayerRigGender,
    locomotion: LegacyLocomotionState,
) -> Option<SoundCue> {
    match (gender, locomotion) {
        (PlayerRigGender::Male, LegacyLocomotionState::Stun) => Some(SoundCue::Random(&[
            "M_Avatar_Stun01", "M_Avatar_Stun02", "M_Avatar_Stun03",
        ])),
        (PlayerRigGender::Female, LegacyLocomotionState::Stun) => Some(SoundCue::Random(&[
            "F_Avatar_Stun01", "F_Avatar_Stun02", "F_Avatar_Stun03",
        ])),
        (PlayerRigGender::Male, LegacyLocomotionState::JumpStart) => Some(SoundCue::Random(&[
            "M_Avatar_JumpStart01",
            "M_Avatar_JumpStart02",
            "M_Avatar_JumpStart03",
            "M_Avatar_JumpStart04",
            "M_Avatar_JumpStart05",
            "M_Avatar_JumpStart06",
            "M_Avatar_JumpStart07",
            "M_Avatar_JumpStart08",
            "M_Avatar_JumpStart09",
        ])),
        (PlayerRigGender::Female, LegacyLocomotionState::JumpStart) => Some(SoundCue::Random(&[
            "F_Avatar_JumpStart01",
            "F_Avatar_JumpStart02",
            "F_Avatar_JumpStart03",
            "F_Avatar_JumpStart04",
            "F_Avatar_JumpStart05",
            "F_Avatar_JumpStart06",
            "F_Avatar_JumpStart07",
            "F_Avatar_JumpStart08",
            "F_Avatar_JumpStart09",
        ])),
        (PlayerRigGender::Male, LegacyLocomotionState::Landing) => Some(SoundCue::Random(&[
            "M_Avatar_JumpLnd01",
            "M_Avatar_JumpLnd02",
            "M_Avatar_JumpLnd03",
        ])),
        (PlayerRigGender::Female, LegacyLocomotionState::Landing) => Some(SoundCue::Random(&[
            "F_Avatar_JmpLnd01",
            "F_Avatar_JmpLnd02",
            "F_Avatar_JmpLnd03",
        ])),
        (PlayerRigGender::Male, LegacyLocomotionState::Slide) => {
            Some(SoundCue::Exact("M_Avatar_Sliding"))
        }
        (PlayerRigGender::Female, LegacyLocomotionState::Slide) => {
            Some(SoundCue::Exact("F_Avatar_Sliding"))
        }
        (_, LegacyLocomotionState::RopeDown) => Some(SoundCue::Exact("SFX_Zipline_Final")),
        (PlayerRigGender::Male, LegacyLocomotionState::Swim) => {
            Some(SoundCue::Exact("M_Avatar_SwimForward"))
        }
        (PlayerRigGender::Female, LegacyLocomotionState::Swim) => {
            Some(SoundCue::Exact("F_Avatar_SwimForward"))
        }
        (_, LegacyLocomotionState::SwimBack) => Some(SoundCue::Exact("Avatar_SwimBack")),
        (_, LegacyLocomotionState::SwimIdle) => Some(SoundCue::Exact("Avatar_SwimIdle")),
        (
            PlayerRigGender::Male,
            LegacyLocomotionState::SwimLeft | LegacyLocomotionState::SwimRight,
        ) => Some(SoundCue::Exact("M_Avatar_SwimTurn")),
        (
            PlayerRigGender::Female,
            LegacyLocomotionState::SwimLeft | LegacyLocomotionState::SwimRight,
        ) => Some(SoundCue::Exact("F_Avatar_SwimTurn")),
        // Primary jumplandrun clips contain only `end`, with no `sound`
        // AnimationEvent. Run/RunBack also have no player footstep event.
        _ => None,
    }
}

pub(super) fn player_locomotion_loop_duration(
    gender: PlayerRigGender,
    locomotion: LegacyLocomotionState,
) -> Option<f32> {
    match (gender, locomotion) {
        // Exact primary AnimationClip `end` events. The sound callback is at
        // 0.25 s in each cycle, so repeated Play/loop ownership re-fires it.
        (_, LegacyLocomotionState::Slide) => Some(1.483_333_3),
        (_, LegacyLocomotionState::RopeDown) => Some(0.483_333_35),
        (PlayerRigGender::Female, LegacyLocomotionState::Swim) => Some(0.950_000_05),
        (
            _,
            LegacyLocomotionState::Swim
            | LegacyLocomotionState::SwimBack
            | LegacyLocomotionState::SwimIdle,
        ) => Some(0.816_666_7),
        (_, LegacyLocomotionState::SwimLeft | LegacyLocomotionState::SwimRight) => Some(0.75),
        _ => None,
    }
}
