// Native player emote timing and semantic sound events.
// Maintained by FusionForge tools/native/export-player-emote-events.py.
use super::{PlayerEmoteEvents, PlayerRigGender, TutorialPlayerClip};
pub(super) const fn events(
    gender: PlayerRigGender,
    clip: TutorialPlayerClip,
) -> Option<PlayerEmoteEvents> {
    match (gender, clip) {
        (PlayerRigGender::Male, TutorialPlayerClip::Cry) => Some(PlayerEmoteEvents {
            end: 2.3500001430511475,
            sounds: &[(0.25, "M_Avatar_Sad0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Female, TutorialPlayerClip::Cry) => Some(PlayerEmoteEvents {
            end: 2.2833333015441895,
            sounds: &[(0.25, "F_Avatar_Sad0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Male, TutorialPlayerClip::Angry) => Some(PlayerEmoteEvents {
            end: 1.0166666507720947,
            sounds: &[(0.25, "M_Avatar_Angry0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Female, TutorialPlayerClip::Angry) => Some(PlayerEmoteEvents {
            end: 1.0166666507720947,
            sounds: &[(0.25, "F_Avatar_Angry0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Male, TutorialPlayerClip::Shocked) => Some(PlayerEmoteEvents {
            end: 1.4833333492279053,
            sounds: &[(0.25, "M_Avatar_Scared0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Female, TutorialPlayerClip::Shocked) => Some(PlayerEmoteEvents {
            end: 1.4833333492279053,
            sounds: &[(0.25, "F_Avatar_Scared0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Male, TutorialPlayerClip::Hello) => Some(PlayerEmoteEvents {
            end: 2.4833333492279053,
            sounds: &[(0.25, "M_Avatar_Hello0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Female, TutorialPlayerClip::Hello) => Some(PlayerEmoteEvents {
            end: 2.4833333492279053,
            sounds: &[(0.25, "F_Avatar_Hello0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Male, TutorialPlayerClip::Thank) => Some(PlayerEmoteEvents {
            end: 1.350000023841858,
            sounds: &[(0.25, "M_Avatar_Thanks0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Female, TutorialPlayerClip::Thank) => Some(PlayerEmoteEvents {
            end: 1.350000023841858,
            sounds: &[(0.25, "F_Avatar_Thanks0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Male, TutorialPlayerClip::Dance1) => Some(PlayerEmoteEvents {
            end: 3.4833333492279053,
            sounds: &[
                (0.25, "M_Avatar_Dance01.wav"),
                (0.25, "M_Avatar_SFX_Dance1_0(RAND:1-2).wav"),
            ],
        }),
        (PlayerRigGender::Female, TutorialPlayerClip::Dance1) => Some(PlayerEmoteEvents {
            end: 3.883333206176758,
            sounds: &[
                (0.25, "F_Avatar_Dance01.wav"),
                (0.25, "F_Avatar_SFX_Dance1_0(RAND:1-2).wav"),
            ],
        }),
        (PlayerRigGender::Male, TutorialPlayerClip::Kiss) => Some(PlayerEmoteEvents {
            end: 1.6166666746139526,
            sounds: &[(0.25, "M_Avatar_Love0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Female, TutorialPlayerClip::Kiss) => Some(PlayerEmoteEvents {
            end: 1.4833333492279053,
            sounds: &[(0.25, "F_Avatar_Love0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Male, TutorialPlayerClip::Agree) => Some(PlayerEmoteEvents {
            end: 1.149999976158142,
            sounds: &[(0.25, "M_Avatar_Yes0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Female, TutorialPlayerClip::Agree) => Some(PlayerEmoteEvents {
            end: 1.149999976158142,
            sounds: &[(0.25, "F_Avatar_Yes0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Male, TutorialPlayerClip::Laugh) => Some(PlayerEmoteEvents {
            end: 3.0833334922790527,
            sounds: &[(0.25, "M_Avatar_Laugh0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Female, TutorialPlayerClip::Laugh) => Some(PlayerEmoteEvents {
            end: 2.4166667461395264,
            sounds: &[(0.25, "F_Avatar_Laugh0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Male, TutorialPlayerClip::No) => Some(PlayerEmoteEvents {
            end: 1.4166667461395264,
            sounds: &[(0.25, "M_Avatar_No0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Female, TutorialPlayerClip::No) => Some(PlayerEmoteEvents {
            end: 1.4166667461395264,
            sounds: &[(0.25, "F_Avatar_No0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Male, TutorialPlayerClip::Flex) => Some(PlayerEmoteEvents {
            end: 1.4833333492279053,
            sounds: &[(0.25, "M_Avatar_Flex0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Female, TutorialPlayerClip::Flex) => Some(PlayerEmoteEvents {
            end: 1.4833333492279053,
            sounds: &[(0.25, "F_Avatar_Flex0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Male, TutorialPlayerClip::Tease) => Some(PlayerEmoteEvents {
            end: 2.4833333492279053,
            sounds: &[(0.25, "M_Avatar_Tease0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Female, TutorialPlayerClip::Tease) => Some(PlayerEmoteEvents {
            end: 2.4833333492279053,
            sounds: &[(0.25, "F_Avatar_Tease0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Male, TutorialPlayerClip::Ok) => Some(PlayerEmoteEvents {
            end: 1.4166667461395264,
            sounds: &[(0.25, "M_Avatar_Yes0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Female, TutorialPlayerClip::Ok) => Some(PlayerEmoteEvents {
            end: 1.4166667461395264,
            sounds: &[(0.25, "F_Avatar_Yes0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Male, TutorialPlayerClip::Applaud) => Some(PlayerEmoteEvents {
            end: 2.0166666507720947,
            sounds: &[(0.25, "M_Avatar_Applause.wav")],
        }),
        (PlayerRigGender::Female, TutorialPlayerClip::Applaud) => Some(PlayerEmoteEvents {
            end: 2.4166667461395264,
            sounds: &[(0.25, "F_Avatar_Applause.wav")],
        }),
        (PlayerRigGender::Male, TutorialPlayerClip::Cheer) => Some(PlayerEmoteEvents {
            end: 1.283333420753479,
            sounds: &[(0.25, "M_Avatar_Happy0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Female, TutorialPlayerClip::Cheer) => Some(PlayerEmoteEvents {
            end: 0.9500000476837158,
            sounds: &[(0.25, "F_Avatar_Happy0(RAND:1-3).wav")],
        }),
        (PlayerRigGender::Male, TutorialPlayerClip::Dance2) => Some(PlayerEmoteEvents {
            end: 2.75,
            sounds: &[
                (0.25, "M_Avatar_Dance02.wav"),
                (0.25, "M_Avatar_SFX_Dance2_0(RAND:1-2).wav"),
            ],
        }),
        (PlayerRigGender::Female, TutorialPlayerClip::Dance2) => Some(PlayerEmoteEvents {
            end: 3.4166667461395264,
            sounds: &[
                (0.25, "F_Avatar_SFX_Dance2_0(RAND:1-2).wav"),
                (0.25, "F_Avatar_Dance02.wav"),
            ],
        }),
        (PlayerRigGender::Male, TutorialPlayerClip::Dance3) => Some(PlayerEmoteEvents {
            end: 2.75,
            sounds: &[
                (0.25, "M_Avatar_Dance03.wav"),
                (0.25, "M_Avatar_SFX_Dance3_0(RAND:1-2).wav"),
            ],
        }),
        (PlayerRigGender::Female, TutorialPlayerClip::Dance3) => Some(PlayerEmoteEvents {
            end: 2.683333396911621,
            sounds: &[
                (0.25, "F_Avatar_Dance03.wav"),
                (0.25, "F_Avatar_SFX_Dance3_0(RAND:1-2).wav"),
            ],
        }),
        (PlayerRigGender::Male, TutorialPlayerClip::Dance4) => Some(PlayerEmoteEvents {
            end: 3.616666793823242,
            sounds: &[
                (0.25, "M_Avatar_SFX_Dance4_0(RAND:1-2).wav"),
                (0.25, "M_Avatar_Dance04.wav"),
            ],
        }),
        (PlayerRigGender::Female, TutorialPlayerClip::Dance4) => Some(PlayerEmoteEvents {
            end: 3.75,
            sounds: &[
                (0.25, "F_Avatar_SFX_Dance4_0(RAND:1-2).wav"),
                (0.25, "F_Avatar_Dance04.wav"),
            ],
        }),
        (PlayerRigGender::Male, TutorialPlayerClip::Dance5) => Some(PlayerEmoteEvents {
            end: 2.75,
            sounds: &[
                (0.25, "M_Avatar_SFX_Dance5_0(RAND:1-2).wav"),
                (0.25, "M_Avatar_Dance05.wav"),
            ],
        }),
        (PlayerRigGender::Female, TutorialPlayerClip::Dance5) => Some(PlayerEmoteEvents {
            end: 4.016666889190674,
            sounds: &[
                (0.25, "F_Avatar_SFX_Dance5_0(RAND:1-2).wav"),
                (0.25, "F_Avatar_Dance05.wav"),
            ],
        }),
        (PlayerRigGender::Male, TutorialPlayerClip::Goodbye) => Some(PlayerEmoteEvents {
            end: 2.4833333492279053,
            sounds: &[],
        }),
        (PlayerRigGender::Female, TutorialPlayerClip::Goodbye) => Some(PlayerEmoteEvents {
            end: 2.4833333492279053,
            sounds: &[],
        }),
        (PlayerRigGender::Male, TutorialPlayerClip::Beach1) => Some(PlayerEmoteEvents {
            end: 2.950000047683716,
            sounds: &[],
        }),
        (PlayerRigGender::Female, TutorialPlayerClip::Beach1) => Some(PlayerEmoteEvents {
            end: 2.950000047683716,
            sounds: &[],
        }),
        (PlayerRigGender::Male, TutorialPlayerClip::Beach2) => Some(PlayerEmoteEvents {
            end: 2.950000047683716,
            sounds: &[],
        }),
        (PlayerRigGender::Female, TutorialPlayerClip::Beach2) => Some(PlayerEmoteEvents {
            end: 2.950000047683716,
            sounds: &[],
        }),
        (PlayerRigGender::Male, TutorialPlayerClip::Beach3) => Some(PlayerEmoteEvents {
            end: 2.950000047683716,
            sounds: &[],
        }),
        (PlayerRigGender::Female, TutorialPlayerClip::Beach3) => Some(PlayerEmoteEvents {
            end: 2.950000047683716,
            sounds: &[],
        }),
        _ => None,
    }
}
