use super::*;

#[test]
fn ui_sfx_money_families_resolve_exactly_and_keep_legacy_pitch_ranges() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = NativeAudioCatalog::open(&root, false).unwrap();
    let mut runtime = GameplayAudioRuntime::default();
    runtime.random = GameplayAudioRandom::with_seed(42);
    for _ in 0..64 {
        runtime.queue_legacy_purchase_sound(true);
        runtime.queue_legacy_money_sound();
        runtime.queue_legacy_money_transfer_sound();
    }
    let mut names = std::collections::BTreeSet::new();
    for cue in &runtime.ui_sounds {
        assert!((0.9..=1.1).contains(&cue.pitch));
        assert_eq!(cue.clean_gain, 0.7);
        let clips = catalog.by_true_name(&cue.true_name);
        let [clip] = clips.as_slice() else { panic!("ambiguous/missing UI SFX {}", cue.true_name) };
        assert_eq!(clip.category, NativeAudioCategory::Sfx);
        assert!(root.join(&clip.path).is_file());
        names.insert(cue.true_name.as_str());
    }
    assert_eq!(names.len(), 7, "two purchases, three money jingles, two transfers");
    runtime.ui_sounds.clear();
    runtime.queue_legacy_purchase_sound(false);
    runtime.queue_gameplay_ui_sound("Yes_Button");
    assert!(runtime.ui_sounds.iter().all(|cue| cue.pitch == 1.0));
    for name in ["Height_Up", "Height_Down", "Character_Rotate_Left", "Character_Limit_Max", "Buddy_Warp", "Incoming_Tell", "Outgoing_Chat"] {
        let clips = catalog.by_true_name(name);
        assert_eq!(clips.len(), 1, "missing/ambiguous {name}");
        assert!(root.join(&clips[0].path).is_file());
    }
}

#[test]
fn tutorial_set_player_null_uses_the_primary_camera_distance_gate() {
    let player = Vec3::new(100.0, 0.0, 0.0);
    let camera = Vec3::new(5.0, 0.0, 0.0);

    assert_eq!(
        legacy_spatial_sound_gate(SpatialAudioTarget::None, Some(player), Some(camera), 5,),
        (Some(camera), 40.0)
    );
    assert_eq!(
        legacy_spatial_sound_gate(SpatialAudioTarget::None, Some(player), Some(camera), 6,),
        (Some(camera), 30.0)
    );
    assert_eq!(
        legacy_spatial_sound_gate(SpatialAudioTarget::Player, Some(player), Some(camera), 5,),
        (Some(player), 18.0)
    );
    assert_eq!(
        legacy_spatial_sound_gate(SpatialAudioTarget::Player, Some(player), Some(camera), 6,),
        (Some(player), 16.0)
    );
}

pub(super) fn all_cues() -> Vec<SoundCue> {
    let mut cues = Vec::new();
    for npc_type in [
        2666, 2667, 2668, 2669, 2670, 2672, 2673, 2674, 2675, 2676, 2677, 2678, 2897, 2902,
    ] {
        if let Some(clips) = actor_clips(npc_type) {
            for clip in clips {
                cues.extend(clip.events.iter().map(|event| event.cue));
            }
        }
    }
    cues.extend([
        SoundCue::Exact("MeleeLtUser-01"),
        SoundCue::Exact("SonicUser-02"),
        SoundCue::Exact("LaserHvyUser-02"),
        SoundCue::Exact("M_Avatar_Attack1"),
        SoundCue::Exact("M_Avatar_Attack1upper"),
        SoundCue::Exact("PlyrAvtF_Attack1"),
        SoundCue::Exact("PlyrAvtF_Attack1upper"),
        SoundCue::Random(&["M_Avatar_Hurt01", "M_Avatar_Hurt02"]),
        SoundCue::Random(&["F_Avatar_Hurt01", "F_Avatar_Hurt02"]),
        SoundCue::Random(&["M_Avatar_Critical01", "M_Avatar_Critical02"]),
        SoundCue::Random(&["F_Avatar_Critical01", "F_Avatar_Critical02"]),
        SoundCue::Exact("SFX_PoisonDamage"),
        SoundCue::Random(&["M_Avatar_GooDmg01", "M_Avatar_GooDmg02"]),
        SoundCue::Random(&["F_Avatar_GooDmg01", "F_Avatar_GooDmg02"]),
    ]);
    for gender in [PlayerRigGender::Male, PlayerRigGender::Female] {
        cues.push(player_locomotion_cue(gender, LegacyLocomotionState::JumpStart).unwrap());
        cues.push(player_locomotion_cue(gender, LegacyLocomotionState::Landing).unwrap());
        cues.push(player_locomotion_cue(gender, LegacyLocomotionState::Slide).unwrap());
        for state in [
            LegacyLocomotionState::Swim,
            LegacyLocomotionState::SwimBack,
            LegacyLocomotionState::SwimIdle,
            LegacyLocomotionState::SwimLeft,
            LegacyLocomotionState::SwimRight,
        ] {
            cues.push(player_locomotion_cue(gender, state).unwrap());
        }
    }
    cues.extend([
        player_locomotion_cue(PlayerRigGender::Male, LegacyLocomotionState::RopeDown).unwrap(),
        SoundCue::Exact("SFX_bouncetech"),
        SoundCue::Exact("Vehicle_GetOn"),
        SoundCue::Exact("Vehicle_GetOff"),
        SoundCue::Exact("InvenLooping"),
        SoundCue::Exact("Open_Screen"),
        SoundCue::Exact("Close_Screen"),
        SoundCue::Random(&["Spawn_CS_Wound_2", "Spawn_CS_Wound_1"]),
        SoundCue::Random(&["DexbotCerberus_CB_Wound_2", "DexbotCerberus_CB_Wound_1"]),
        SoundCue::Random(&["OilMonster_OO_Wound_2", "OilMonster_OO_Wound_1"]),
        SoundCue::Exact("DexbotBat_TW_Wound"),
        SoundCue::Exact("FusionButtercup_Wound"),
    ]);
    cues
}

#[test]
fn player_movement_does_not_invent_unproven_footsteps() {
    for gender in [PlayerRigGender::Male, PlayerRigGender::Female] {
        assert!(player_locomotion_cue(gender, LegacyLocomotionState::Run).is_none());
        assert!(player_locomotion_cue(gender, LegacyLocomotionState::LandingRun).is_none());
    }
}

#[test]
fn samurai_jack_sword_sounds_match_primary_melee1event_timing() {
    let clip = actor_clip_audio(2666, "melee1event")
        .expect("Samurai Jack cutscene melee1event audio contract");
    assert_eq!(clip.duration_seconds, 3.333_331_6);
    let events = clip
        .events
        .iter()
        .map(|event| {
            let SoundCue::Exact(true_name) = event.cue else {
                panic!("Samurai Jack clip must use exact primary sound names")
            };
            (event.seconds, true_name)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        events,
        vec![
            (0.133_333, "SamJack_AttackGrunt1"),
            (0.233_333, "SamJack_SwordSFX_1"),
            (0.7, "SamJack_AttackGrunt2"),
            (0.766_667, "SamJack_SwordSFX_2"),
            (1.233_333, "SamJack_SwordSFX_3"),
            (1.266_667, "SamJack_AttackGrunt3"),
        ]
    );
    assert!(actor_clip_audio(2666, "melee1").is_none());

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = NativeAudioCatalog::open(&root, false).unwrap();
    assert!(
        catalog.by_true_name("windUser-04").is_empty(),
        "the unavailable primary windUser-04 event must stay silent"
    );
}

#[test]
fn belladonna_dialogue_resolves_from_production_owner_in_both_languages() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(&root).unwrap();
    let content = TutorialMissionContent::open(&locator).unwrap();
    let npc = content.gameplay_npc(720).expect("Belladonna");
    assert_eq!(npc.move_voice_owner, "Belladonna");
    let catalog = NativeAudioCatalog::open(&root, false).unwrap();
    let anchor = Entity::from_bits(77);
    for cue in [
        legacy_npc_open_voice_cue(npc.service_category, false, false),
        legacy_npc_open_voice_cue(npc.service_category, false, true),
        LegacyNpcVoiceCue::Farewell,
        LegacyNpcVoiceCue::QuestAccepted,
        LegacyNpcVoiceCue::QuestCompleted,
    ] {
        let mut runtime = GameplayAudioRuntime::default();
        runtime.queue_legacy_npc_voice(anchor, &npc.move_voice_owner, cue);
        let name = runtime
            .npc_voice_changes
            .get(&anchor)
            .unwrap()
            .as_ref()
            .unwrap();
        let candidates = catalog.by_true_name(name);
        assert_eq!(candidates.len(), 1, "{name}");
        let audio = candidates[0];
        assert_eq!(audio.owner, "belladonna");
        assert_eq!(audio.category, NativeAudioCategory::Voice);
        for locale in ["en", "ru"] {
            let path = catalog.path_for_locale(audio, locale).unwrap();
            assert!(root.join(path).is_file(), "{name} ({locale})");
        }
    }
}
