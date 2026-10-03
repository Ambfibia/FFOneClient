use super::*;

pub(crate) const LEGACY_SPATIAL_SCALE: SpatialScale = SpatialScale::new(0.1);

pub(super) const SPAWN_MELEE: SoundCue = SoundCue::Random(&[
    "Spawn_CS_Melee1_1",
    "Spawn_CS_Melee1_2",
    "Spawn_CS_Melee1_3",
]);

pub(super) const SPAWN_WALK: SoundCue = SoundCue::Random(&["Spawn_CS_Walk1_1", "Spawn_CS_Walk1_2"]);

pub(super) const SPAWN_STAND1: SoundCue = SoundCue::Random(&["Spawn_CS_Stand1_1", "Spawn_CS_Stand1_2"]);

pub(super) const SPAWN_STAND_ALT: SoundCue =
    SoundCue::Random(&["Spawn_CS_Stand2", "Spawn_CS_Stand3", "Spawn_CS_Stand4"]);

pub(super) const SPAWN_CLIPS: &[ActorClipAudio] = &[
    ActorClipAudio {
        clip: "corruptak",
        duration_seconds: 1.100_000_3,
        events: &[
            sound_event!(0.1, SoundCue::Exact("Spawn_CS_Corruptak")),
            sound_event!(0.1, SoundCue::Exact("Spawn_CS_Skill1")),
        ],
    },
    audio_clip!("death", 1.599_999_8, 0.1, SoundCue::Exact("Spawn_CS_Death")),
    ActorClipAudio {
        clip: "dodge",
        duration_seconds: 0.633_333_44,
        events: &[
            sound_event!(0.1, SPAWN_MELEE),
            sound_event!(0.1, SoundCue::Exact("Spawn_CS_Dodge")),
        ],
    },
    audio_clip!("melee1", 1.000_000_4, 0.1, SPAWN_MELEE),
    audio_clip!("ready", 0.766_666_9, 0.1, SoundCue::Exact("Spawn_CS_Ready")),
    audio_clip!("run", 0.633_333_44, 0.1, SPAWN_WALK),
    audio_clip!(
        "skill1",
        1.100_000_3,
        0.1,
        SoundCue::Exact("Spawn_CS_Skill1")
    ),
    audio_clip!("stand1", 0.966_667_06, 0.1, SPAWN_STAND1),
    audio_clip!("stand2", 1.333_333_4, 0.1, SPAWN_STAND_ALT),
    audio_clip!("stand3", 1.333_333_4, 0.1, SPAWN_STAND_ALT),
    audio_clip!("stand4", 1.333_333_4, 0.1, SPAWN_STAND_ALT),
    audio_clip!("walk", 0.4, 0.1, SPAWN_WALK),
];

pub(super) const CERBERUS_MELEE1: SoundCue = SoundCue::Random(&[
    "DexbotCerberus_CB_Melee1_1",
    "DexbotCerberus_CB_Melee1_2",
    "DexbotCerberus_CB_Melee1_3",
]);

pub(super) const CERBERUS_READY: SoundCue =
    SoundCue::Random(&["DexbotCerberus_CB_Ready1_1", "DexbotCerberus_CB_Ready1_2"]);

pub(super) const CERBERUS_CLIPS: &[ActorClipAudio] = &[
    audio_clip!(
        "corruptak",
        0.966_667_06,
        0.25,
        SoundCue::Exact("DexbotCerberus_CB_Corruptak")
    ),
    audio_clip!(
        "death",
        1.233_333_5,
        0.25,
        SoundCue::Exact("DexbotCerberus_CB_Death")
    ),
    audio_clip!("melee1", 0.633_333_44, 0.25, CERBERUS_MELEE1),
    audio_clip!(
        "melee1upper",
        0.633_333_44,
        0.25,
        SoundCue::Exact("DexbotCerberus_CB_Melee1upper")
    ),
    audio_clip!(
        "melee2",
        0.633_333_44,
        0.25,
        SoundCue::Exact("DexbotCerberus_CB_Melee1_3")
    ),
    audio_clip!(
        "melee2upper",
        0.633_333_44,
        0.25,
        SoundCue::Exact("DexbotCerberus_CB_Melee2upper")
    ),
    audio_clip!("ready", 1.166_666_9, 0.25, CERBERUS_READY),
    audio_clip!(
        "run",
        0.766_666_9,
        0.25,
        SoundCue::Exact("DexbotCerberus_CB_Run")
    ),
    audio_clip!(
        "skill0",
        0.966_667_06,
        0.25,
        SoundCue::Exact("DexbotCerberus_CB_Skill0")
    ),
    audio_clip!(
        "stand1",
        1.233_333_5,
        0.25,
        SoundCue::Exact("DexbotCerberus_CB_Stand1")
    ),
    audio_clip!(
        "stand2",
        2.866_665_4,
        0.25,
        SoundCue::Exact("DexbotCerberus_CB_Stand2")
    ),
    audio_clip!(
        "stand3",
        3.133_331_8,
        0.25,
        SoundCue::Exact("DexbotCerberus_CB_Stand3")
    ),
    audio_clip!(
        "stand4",
        1.999_999_4,
        0.25,
        SoundCue::Exact("DexbotCerberus_CB_Stand4")
    ),
    audio_clip!(
        "walk",
        1.033_333_7,
        0.25,
        SoundCue::Exact("DexbotCerberus_CB_Walk")
    ),
];

pub(super) const OIL_MELEE1: SoundCue =
    SoundCue::Random(&["OilMonster_OO_Melee1_1", "OilMonster_OO_Melee1_2"]);

pub(super) const OIL_MELEE2: SoundCue =
    SoundCue::Random(&["OilMonster_OO_Melee2_1", "OilMonster_OO_Melee2_2"]);

pub(super) const OIL_CLIPS: &[ActorClipAudio] = &[
    audio_clip!(
        "corruptak",
        0.900_000_33,
        0.25,
        SoundCue::Exact("OilMonster_OO_Melee1_1")
    ),
    audio_clip!(
        "death",
        3.666_664_6,
        0.25,
        SoundCue::Exact("OilMonster_OO_Death")
    ),
    audio_clip!(
        "dodge",
        0.566_666_7,
        0.25,
        SoundCue::Exact("OilMonster_OO_Dodge")
    ),
    audio_clip!("melee1", 0.900_000_33, 0.25, OIL_MELEE1),
    audio_clip!(
        "melee1upper",
        1.033_333_7,
        0.25,
        SoundCue::Exact("OilMonster_OO_Melee1Upper")
    ),
    audio_clip!("melee2", 0.900_000_33, 0.25, OIL_MELEE2),
    audio_clip!(
        "melee2upper",
        1.033_333_7,
        0.25,
        SoundCue::Exact("OilMonster_OO_Melee1Upper")
    ),
    audio_clip!(
        "ready",
        1.333_333_4,
        0.25,
        SoundCue::Exact("OilMonster_OO_Ready")
    ),
    audio_clip!(
        "run",
        0.966_667_06,
        0.25,
        SoundCue::Exact("OilMonster_OO_Run")
    ),
    audio_clip!(
        "skill0",
        0.966_667_06,
        0.25,
        SoundCue::Exact("OilMonster_OO_Skill0")
    ),
    audio_clip!(
        "stand1",
        2.333_332_5,
        0.25,
        SoundCue::Exact("OilMonster_OO_Stand1")
    ),
    audio_clip!(
        "stand2",
        2.999_998_6,
        0.25,
        SoundCue::Exact("OilMonster_OO_Stand2")
    ),
    audio_clip!(
        "stand3",
        2.999_998_6,
        0.25,
        SoundCue::Exact("OilMonster_OO_Stand3")
    ),
    audio_clip!(
        "stand4",
        2.333_332_5,
        0.25,
        SoundCue::Exact("OilMonster_OO_Stand4")
    ),
    audio_clip!(
        "standup",
        2.999_998_6,
        0.25,
        SoundCue::Exact("OilMonster_OO_Standup")
    ),
    audio_clip!(
        "walk",
        1.333_333_4,
        0.25,
        SoundCue::Exact("OilMonster_OO_Walk_LOOP")
    ),
];

pub(super) const BAT_READY: SoundCue = SoundCue::Random(&["DexbotBat_TW_Ready1", "DexbotBat_TW_Ready2"]);

pub(super) const BAT_RUN: SoundCue = SoundCue::Random(&["DexbotBat_TW_Run1", "DexbotBat_TW_Run2"]);

pub(super) const BAT_WALK: SoundCue = SoundCue::Random(&["DexbotBat_TW_Walk1", "DexbotBat_TW_Walk2"]);

pub(super) const BAT_CLIPS: &[ActorClipAudio] = &[
    audio_clip!(
        "corruptak",
        1.033_333_7,
        0.25,
        SoundCue::Exact("DexbotBat_TW_Corruptak")
    ),
    audio_clip!(
        "death",
        0.900_000_33,
        0.25,
        SoundCue::Exact("DexbotBat_TW_Death")
    ),
    audio_clip!(
        "dodge",
        0.766_666_9,
        0.25,
        SoundCue::Exact("DexbotBat_TW_Dodge")
    ),
    audio_clip!(
        "melee1",
        1.033_333_7,
        0.25,
        SoundCue::Exact("DexbotBat_TW_Range1")
    ),
    audio_clip!(
        "melee2",
        1.033_333_7,
        0.25,
        SoundCue::Exact("DexbotBat_TW_Range2")
    ),
    audio_clip!("ready", 1.799_999_6, 0.25, BAT_READY),
    audio_clip!("run", 1.166_666_9, 0.25, BAT_RUN),
    audio_clip!(
        "skill0",
        1.033_333_7,
        0.25,
        SoundCue::Exact("DexbotBat_TW_Skill0")
    ),
    audio_clip!(
        "stand1",
        2.399_999_1,
        0.25,
        SoundCue::Exact("DexbotBat_TW_Stand1")
    ),
    audio_clip!(
        "stand2",
        2.399_999_1,
        0.25,
        SoundCue::Exact("DexbotBat_TW_Stand2")
    ),
    audio_clip!(
        "stand3",
        4.999_996_7,
        0.25,
        SoundCue::Exact("DexbotBat_TW_Stand3")
    ),
    audio_clip!(
        "stand4",
        1.166_666_9,
        0.25,
        SoundCue::Exact("DexbotBat_TW_Stand4")
    ),
    audio_clip!("walk", 2.399_999_1, 0.25, BAT_WALK),
];

pub(super) const BUTTERCUP_CLIPS: &[ActorClipAudio] = &[
    audio_clip!(
        "corruptak",
        0.966_667_06,
        0.25,
        SoundCue::Exact("FusionButtercup_Corruptak")
    ),
    audio_clip!(
        "corruptak_readyspell",
        1.499_999_9,
        0.25,
        SoundCue::Exact("FusionButtercup_Corruptak_Readyspell")
    ),
    audio_clip!(
        "death",
        1.599_999_8,
        0.25,
        SoundCue::Exact("FusionButtercup_Death")
    ),
    audio_clip!(
        "megatak_readyspell",
        1.499_999_9,
        0.25,
        SoundCue::Exact("FusionButtercup_Megatak_Readyspell")
    ),
    audio_clip!(
        "melee1",
        1.100_000_3,
        0.25,
        SoundCue::Exact("FusionButtercup_Melee1")
    ),
    audio_clip!(
        "melee1upper",
        1.100_000_3,
        0.25,
        SoundCue::Exact("FusionButtercup_Melee1Upper")
    ),
    audio_clip!(
        "melee2",
        2.099_999_4,
        0.25,
        SoundCue::Exact("FusionButtercup_Melee2")
    ),
    audio_clip!(
        "melee2upper",
        2.099_999_4,
        0.25,
        SoundCue::Exact("FusionButtercup_Melee2Upper")
    ),
    audio_clip!(
        "ready",
        1.999_999_4,
        0.25,
        SoundCue::Exact("FusionButtercup_Ready")
    ),
    audio_clip!(
        "run",
        0.966_667_06,
        0.25,
        SoundCue::Exact("FusionButtercup_Run")
    ),
    audio_clip!(
        "skill0",
        0.966_667_06,
        0.25,
        SoundCue::Exact("FusionButtercup_Skill0")
    ),
    audio_clip!(
        "stand1",
        1.999_999_4,
        0.25,
        SoundCue::Exact("FusionButtercup_Stand1")
    ),
    audio_clip!(
        "stand2",
        3.999_997_6,
        0.25,
        SoundCue::Exact("FusionButtercup_Stand2")
    ),
    audio_clip!(
        "stand3",
        3.999_997_6,
        0.25,
        SoundCue::Exact("FusionButtercup_Stand3")
    ),
    audio_clip!(
        "stand4",
        1.166_666_9,
        0.25,
        SoundCue::Exact("FusionButtercup_Stand4")
    ),
    audio_clip!(
        "walk",
        1.999_999_4,
        0.25,
        SoundCue::Exact("FusionButtercup_Walk")
    ),
];

pub(super) const NUMBUH_ONE_WEAPON: SoundCue =
    SoundCue::Random(&["NumbuhOne_WeaponSFX_1", "NumbuhOne_WeaponSFX_2"]);

pub(super) const DEXTER_PISTOL_WEAPON: SoundCue =
    SoundCue::Random(&["Dexter_WeaponSFX_1", "Dexter_WeaponSFX_2"]);

pub(super) const NUMBUH_FIVE_WEAPON: SoundCue =
    SoundCue::Random(&["NumbuhFive_WeaponSFX_01", "NumbuhFive_WeaponSFX_02"]);

pub(super) const DEXTER_ATTACK: SoundCue = SoundCue::Random(&["Dexter_Atk_1", "Dexter_Atk_2"]);

pub(super) const BEN_ATTACK_GRUNT: SoundCue = SoundCue::Random(&[
    "Ben_AttackGrunt_01",
    "Ben_AttackGrunt_02",
    "Ben_AttackGrunt_03",
    "Ben_AttackGrunt_04",
    "Ben_AttackGrunt_05",
    "Ben_AttackGrunt_06",
    "Ben_AttackGrunt_07",
]);

pub(super) const NUMBUH_ONE_CLIPS: &[ActorClipAudio] =
    &[audio_clip!("melee1", 1.166_666_9, 0.24, NUMBUH_ONE_WEAPON)];

pub(super) const DEXTER_PISTOL_CLIPS: &[ActorClipAudio] = &[audio_clip!(
    "melee1",
    1.166_666_9,
    0.24,
    DEXTER_PISTOL_WEAPON
)];

pub(super) const NUMBUH_FIVE_CLIPS: &[ActorClipAudio] = &[
    audio_clip!("melee1", 0.900_000_33, 0.186_667, NUMBUH_FIVE_WEAPON),
    audio_clip!("melee1sitdown", 1.999_999_4, 0.16, NUMBUH_FIVE_WEAPON),
];

pub(super) const BEN_CLIPS: &[ActorClipAudio] = &[
    ActorClipAudio {
        clip: "kick",
        duration_seconds: 1.4,
        events: &[
            sound_event!(0.613_333, BEN_ATTACK_GRUNT),
            sound_event!(0.626_667, SoundCue::Exact("Ben_AttackWhoosh_02")),
        ],
    },
    ActorClipAudio {
        clip: "melee1event",
        duration_seconds: 2.666_665_6,
        events: &[
            sound_event!(1.387, SoundCue::Exact("Ben_AttackWhoosh_01")),
            sound_event!(1.46, BEN_ATTACK_GRUNT),
            sound_event!(2.019_667, SoundCue::Exact("Ben_AttackWhoosh_02")),
            sound_event!(2.019_667, BEN_ATTACK_GRUNT),
        ],
    },
];

// Primary Retrobution Tutorial.resourceFile
// CustomAssetBundle-b4f543c102ded400fbc6f1da25d9679a AnimationClip pathId 1103.
// The scene starts this looping `melee1event` clip on Samurai Jack (NPC type
// 2666), so each sound must follow the loaded AnimationPlayer clock and fire
// again on every completed pass. The clip's final sound event requests
// `windUser-04.wav` at 2.866667 s, but that AudioClip is absent from the clean
// primary containers; preserve the original failed lookup instead of
// substituting another wind or sword sound.
pub(super) const SAMURAI_JACK_CLIPS: &[ActorClipAudio] = &[ActorClipAudio {
    clip: "melee1event",
    duration_seconds: 3.333_331_6,
    events: &[
        sound_event!(0.133_333, SoundCue::Exact("SamJack_AttackGrunt1")),
        sound_event!(0.233_333, SoundCue::Exact("SamJack_SwordSFX_1")),
        sound_event!(0.7, SoundCue::Exact("SamJack_AttackGrunt2")),
        sound_event!(0.766_667, SoundCue::Exact("SamJack_SwordSFX_2")),
        sound_event!(1.233_333, SoundCue::Exact("SamJack_SwordSFX_3")),
        sound_event!(1.266_667, SoundCue::Exact("SamJack_AttackGrunt3")),
    ],
}];

// `t_buttercup.melee1` requests `Bubbles_AtkLng_01.wav` and
// `LaserHvyUser-01.wav`, but neither AudioClip is present in the primary
// containers. Preserve the clean client's failed loads rather than silently
// substituting nearby numbered assets.
pub(super) const DEXTER_CLIPS: &[ActorClipAudio] = &[
    audio_clip!("melee1", 0.700_000_17, 0.1, DEXTER_ATTACK),
    audio_clip!("melee1upper", 0.700_000_17, 0.1, DEXTER_ATTACK),
];

pub(super) const DEXTER_SWORD_CLIPS: &[ActorClipAudio] = &[
    audio_clip!(
        "melee1",
        0.833_333_6,
        0.303_333,
        SoundCue::Exact("MeleeLtUser-01")
    ),
    ActorClipAudio {
        clip: "melee1event",
        duration_seconds: 3.466_664_8,
        events: &[
            sound_event!(1.56, SoundCue::Exact("Dexter_WeaponSFX_01")),
            sound_event!(1.594_667, SoundCue::Exact("Dexter_AttackGrunt_01")),
            sound_event!(1.837_333, SoundCue::Exact("Dexter_WeaponTARGETSFX_01")),
            sound_event!(2.149_333, SoundCue::Exact("Dexter_WeaponSFX_02")),
            sound_event!(2.288, SoundCue::Exact("Dexter_WeaponTARGETSFX_02")),
            sound_event!(2.877_333, SoundCue::Exact("Dexter_WeaponSFX_01")),
            sound_event!(2.877_333, SoundCue::Exact("Dexter_AttackGrunt_03")),
            sound_event!(3.085_333, SoundCue::Exact("Dexter_WeaponTARGETSFX_01")),
        ],
    },
    audio_clip!(
        "melee1upper",
        0.633_333_44,
        0.293_333,
        SoundCue::Exact("MeleeLtUser-01")
    ),
    // melee2/melee2upper request `MeleeLtUser-02.wav`, which is not present
    // in the primary audio containers. Do not replace it with variant 01.
];
