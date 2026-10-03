//! Data-driven timing recovered from the finite tutorial coroutines in
//! `cntutorialscript`.
//!
//! This module deliberately contains no Bevy audio entities. It is the exact
//! presentation contract consumed by the native client and can be unit-tested
//! without an asset server or renderer.

use crate::tutorial::TutorialScene;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialAudioCueKind {
    /// Replaces the currently playing tutorial voice, matching `VoiceOut`.
    Voice,
    /// Overlapping one-shot music or sound effect.
    OneShot,
    /// Loop until a later `StopLoops` cue or scene cleanup.
    StartLoop,
    /// Stop every loop owned by the current tutorial scene.
    StopLoops,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TutorialAudioCue {
    pub seconds: f32,
    /// Legacy cue name below `Tut Sound/` or `Sound/`, and the clip's exact
    /// Unity true name. The runtime resolves it through the audio catalog by
    /// that name, so the cue never encodes where the file sits.
    pub cue: &'static str,
    pub kind: TutorialAudioCueKind,
    /// Exact semantic route for the cues the true name alone cannot resolve:
    /// a voice line, or an SFX true name that more than one route publishes.
    pub semantic_path: Option<&'static str>,
}

const fn voice(seconds: f32, cue: &'static str) -> TutorialAudioCue {
    TutorialAudioCue {
        seconds,
        cue,
        kind: TutorialAudioCueKind::Voice,
        semantic_path: None,
    }
}

const fn voice_at(
    seconds: f32,
    cue: &'static str,
    semantic_path: &'static str,
) -> TutorialAudioCue {
    TutorialAudioCue {
        seconds,
        cue,
        kind: TutorialAudioCueKind::Voice,
        semantic_path: Some(semantic_path),
    }
}

const fn sound(seconds: f32, cue: &'static str) -> TutorialAudioCue {
    TutorialAudioCue {
        seconds,
        cue,
        kind: TutorialAudioCueKind::OneShot,
        semantic_path: None,
    }
}

const fn sound_at(
    seconds: f32,
    cue: &'static str,
    semantic_path: &'static str,
) -> TutorialAudioCue {
    TutorialAudioCue {
        seconds,
        cue,
        kind: TutorialAudioCueKind::OneShot,
        semantic_path: Some(semantic_path),
    }
}

const fn start_loop(seconds: f32, cue: &'static str) -> TutorialAudioCue {
    TutorialAudioCue {
        seconds,
        cue,
        kind: TutorialAudioCueKind::StartLoop,
        semantic_path: None,
    }
}

const fn stop_loops(seconds: f32) -> TutorialAudioCue {
    TutorialAudioCue {
        seconds,
        cue: "",
        kind: TutorialAudioCueKind::StopLoops,
        semantic_path: None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TutorialScenePresentation {
    pub scene: TutorialScene,
    /// Natural completion time after all finite `WaitForSeconds`/fixed loops.
    /// Asset-preload waits are intentionally excluded, exactly as they are from
    /// the reference coroutine's presentation clock once loading completes.
    pub duration_seconds: f32,
    pub audio: &'static [TutorialAudioCue],
}

const BASIC_MOVE_AUDIO: &[TutorialAudioCue] = &[
    voice(1.0, "Computress_Tut01"),
    sound(2.0, "TechSquareText_Typed"),
    sound(5.15, "Flythru_Sting"),
    sound(6.15, "Explosion_01"),
    sound(6.25, "Explosion_01"),
    sound(6.55, "Explosion_02"),
    sound(6.85, "Explosion_03"),
    sound(7.15, "Explosion_04"),
    sound(8.15, "Buttercup_Flyby_Whoosh"),
    voice(8.15, "Btrcup_Tut01"),
    sound(9.65, "Buttercup_Flyby_Whoosh"),
    voice(23.15, "NumFive_TutFollowMe_REV"),
    voice_at(25.15, "Ben_Tut01", "audio/voice/en/ben/tut01.ogg"),
];

const BASIC_COMBAT_A_AUDIO: &[TutorialAudioCue] = &[
    sound(0.0, "FusionSpawns_Sting"),
    voice_at(1.0, "Ben_Tut02", "audio/voice/en/ben/tut02.ogg"),
    voice(6.0, "NumFive_TutHoldThem_REV"),
];

const BASIC_COMBAT_B_AUDIO: &[TutorialAudioCue] = &[
    voice_at(3.3, "Ben_Tut03", "audio/voice/en/ben/tut03.ogg"),
    voice_at(8.8, "Ben_Tut04", "audio/voice/en/ben/tut04.ogg"),
    voice(15.9, "Computress_Tut16"),
    voice(30.1, "Computress_Tut17"),
    sound(32.6, "Cyberus_Sting"),
    voice(32.6, "Ben_Tut05"),
    sound_at(
        34.1,
        "Cyberus_Landing",
        "audio/sfx/world_events/cyberus_landing.ogg",
    ),
    voice_at(35.1, "Ben_Tut06", "audio/voice/en/ben/tut06.ogg"),
];

const BASIC_COMBAT_C_AUDIO: &[TutorialAudioCue] = &[
    voice_at(1.0, "Ben_Tut08", "audio/voice/en/ben/tut08.ogg"),
    voice(5.0, "NumFive_TutIKnowYou"),
    voice(14.5, "NumFive_TutFuseExpl"),
    sound(15.5, "PlanetFusion_Sting"),
    voice_at(25.5, "Ben_Tut09", "audio/voice/en/ben/tut09.ogg"),
    voice_at(39.0, "Ben_Tut10", "audio/voice/en/ben/tut10.ogg"),
];

const INFECTION_A_AUDIO: &[TutorialAudioCue] =
    &[sound(1.0, "InfectedZone_Sting"), voice(1.0, "BtrCup_Tut08")];

const INFECTION_B_AUDIO: &[TutorialAudioCue] = &[
    voice(1.0, "Dexter_Tut03"),
    voice(12.0, "Dexter_Tut04"),
    voice(17.1, "Dexter_Tut08"),
    sound_at(
        21.1,
        "FusionButtercup_Stand1",
        "audio/voice/en/fusionbuttercup/stand1.ogg",
    ),
    voice(24.6, "Dexter_Tut09"),
];

const INFECTION_C_AUDIO: &[TutorialAudioCue] = &[
    sound(1.5, "Nano_Creation"),
    // Buttercup `call2` AnimationClip pathId 968 fires this exact voice event
    // at 0.25s. The presentation starts call2 at scene time 3.5s.
    voice_at(
        3.75,
        "Btrcup_NanSummon01_01",
        "audio/voice/en/nano_buttercup/nansummon01_01.ogg",
    ),
    voice(8.9, "Dexter_Tut10"),
    voice(26.4, "Computress_Tut55"),
    start_loop(33.4, "LairCollapse_Quake_LOOP"),
    voice(34.4, "Dexter_Tut11"),
];

const NANO_POWER_A_AUDIO: &[TutorialAudioCue] = &[
    sound(0.0, "SCAMPER_Sting"),
    start_loop(0.0, "RumbleQuake_LOOP"),
    sound(4.0, "Explosion_01"),
    sound(4.3, "Explosion_02"),
    sound(4.6, "Explosion_03"),
    sound(4.9, "Explosion_04"),
    sound(5.2, "Explosion_01"),
    sound(5.5, "Explosion_04"),
    sound(5.5, "Explosion_03"),
    voice(8.5, "NumTwo_Tut05"),
    stop_loops(13.5),
];

const NANO_POWER_A2_AUDIO: &[TutorialAudioCue] = &[sound(0.0, "TechWing_Sting")];

const NANO_POWER_B_AUDIO: &[TutorialAudioCue] = &[
    sound(0.0, "DexCarrier_Sting"),
    voice(2.0, "NumTwo_Tut06"),
    sound(13.25, "Explosion_01"),
    sound(13.45, "Explosion_02"),
    sound(13.45, "SCAMPER_Jet"),
    sound(13.75, "Explosion_03"),
    sound(13.95, "Explosion_04"),
    sound(14.15, "Explosion_01"),
    sound(14.35, "Explosion_02"),
    sound(14.45, "Explosion_03"),
    start_loop(16.65, "DexterHologram_LOOP"),
    voice(16.65, "Dexter_Tut12"),
    stop_loops(40.65),
    sound_at(
        40.65,
        "HologramOff",
        "audio/sfx/environment/hologramoff.ogg",
    ),
    voice(41.15, "NumTwo_Tut07"),
];

pub const TUTORIAL_SCENE_PRESENTATIONS: &[TutorialScenePresentation] = &[
    TutorialScenePresentation {
        scene: TutorialScene::BasicMove,
        duration_seconds: 31.15,
        audio: BASIC_MOVE_AUDIO,
    },
    TutorialScenePresentation {
        scene: TutorialScene::BasicCombatA,
        duration_seconds: 12.0,
        audio: BASIC_COMBAT_A_AUDIO,
    },
    TutorialScenePresentation {
        scene: TutorialScene::BasicCombatB,
        duration_seconds: 39.6,
        audio: BASIC_COMBAT_B_AUDIO,
    },
    TutorialScenePresentation {
        scene: TutorialScene::BasicCombatC,
        duration_seconds: 45.0,
        audio: BASIC_COMBAT_C_AUDIO,
    },
    TutorialScenePresentation {
        scene: TutorialScene::InfectionA,
        duration_seconds: 15.0,
        audio: INFECTION_A_AUDIO,
    },
    TutorialScenePresentation {
        scene: TutorialScene::InfectionB,
        duration_seconds: 33.1,
        audio: INFECTION_B_AUDIO,
    },
    TutorialScenePresentation {
        scene: TutorialScene::InfectionC,
        duration_seconds: 47.9,
        audio: INFECTION_C_AUDIO,
    },
    TutorialScenePresentation {
        scene: TutorialScene::NanoPowerA,
        duration_seconds: 14.5,
        audio: NANO_POWER_A_AUDIO,
    },
    TutorialScenePresentation {
        scene: TutorialScene::NanoPowerA2,
        duration_seconds: 5.0,
        audio: NANO_POWER_A2_AUDIO,
    },
    TutorialScenePresentation {
        scene: TutorialScene::NanoPowerB,
        duration_seconds: 47.15,
        audio: NANO_POWER_B_AUDIO,
    },
];

pub fn tutorial_scene_presentation(
    scene: TutorialScene,
) -> Option<&'static TutorialScenePresentation> {
    TUTORIAL_SCENE_PRESENTATIONS
        .iter()
        .find(|presentation| presentation.scene == scene)
}

#[cfg(test)]
mod tests;
