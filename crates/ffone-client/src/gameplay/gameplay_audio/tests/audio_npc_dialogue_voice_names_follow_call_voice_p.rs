use super::*;

#[test]
fn duplicate_sfx_true_names_have_one_general_world_route() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = NativeAudioCatalog::open(&root, false).unwrap();
    let mut groups = HashMap::<String, Vec<_>>::new();
    for audio in catalog
        .assets()
        .iter()
        .filter(|audio| audio.category == NativeAudioCategory::Sfx)
    {
        groups
            .entry(audio.true_name.to_ascii_lowercase())
            .or_default()
            .push(audio);
    }
    for (true_name, candidates) in groups
        .into_iter()
        .filter(|(_, candidates)| candidates.len() > 1)
    {
        let general = candidates
            .into_iter()
            .filter(|audio| {
                !audio.logical_key.starts_with("sfx/character_creation/")
                    && !audio.logical_key.contains("/zone_local/")
                    && !audio.logical_key.contains("/tutorial_audio/")
            })
            .collect::<Vec<_>>();
        assert_eq!(
            general.len(),
            1,
            "duplicate SFX true name {true_name:?} has no unique general-world route"
        );
    }
}

#[test]
fn npc_dialogue_voice_names_follow_call_voice_play_and_resolve_both_locales() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = NativeAudioCatalog::open(&root, false).unwrap();
    let mut random = GameplayAudioRandom::with_seed(17);
    for cue in [
        LegacyNpcVoiceCue::Greeting,
        LegacyNpcVoiceCue::QuestGreeting,
        LegacyNpcVoiceCue::Farewell,
        LegacyNpcVoiceCue::QuestAccepted,
        LegacyNpcVoiceCue::QuestCompleted,
    ] {
        for _ in 0..16 {
            let true_name = legacy_npc_voice_true_name("Dexter", cue, &mut random).unwrap();
            let candidates = catalog
                .by_true_name(&true_name)
                .into_iter()
                .filter(|asset| asset.category == NativeAudioCategory::Voice)
                .collect::<Vec<_>>();
            let [audio] = candidates.as_slice() else {
                panic!("NPC dialogue {true_name:?} does not have one native voice route")
            };
            for locale in ["en", "ru"] {
                assert!(
                    root.join(catalog.path_for_locale(audio, locale).unwrap())
                        .is_file()
                );
            }
        }
    }

    // Echo Echo is the important renamed-file regression: clean
    // AvatarUtil requests the `echoecho_*` container route while the
    // editable files intentionally live below `voice/<locale>/echo`.
    for cue in [
        LegacyNpcVoiceCue::Greeting,
        LegacyNpcVoiceCue::QuestGreeting,
        LegacyNpcVoiceCue::Farewell,
    ] {
        for _ in 0..16 {
            let true_name = legacy_npc_voice_true_name("echoecho", cue, &mut random).unwrap();
            let candidates = catalog
                .by_true_name(&true_name)
                .into_iter()
                .filter(|asset| asset.category == NativeAudioCategory::Voice)
                .collect::<Vec<_>>();
            let [audio] = candidates.as_slice() else {
                panic!(
                    "renamed Echo Echo dialogue {true_name:?} does not have one native voice route"
                )
            };
            assert_eq!(audio.owner, "echo");
            for locale in ["en", "ru"] {
                let path = catalog.path_for_locale(audio, locale).unwrap();
                assert!(path.contains(&format!("/voice/{locale}/echo/")));
                assert!(root.join(path).is_file());
            }
        }
    }

    assert_eq!(
        legacy_npc_voice_true_name(
            "F_KNDOp1_Armor",
            LegacyNpcVoiceCue::QuestGreeting,
            &mut random
        )
        .as_deref(),
        Some("F_KNDOp1_qgreeting"),
        "three-part owners use the first two segments for quest/farewell VO"
    );
    assert!(legacy_npc_voice_true_name("", LegacyNpcVoiceCue::Greeting, &mut random).is_none());
    assert_eq!(
        legacy_npc_open_voice_cue(3, false, true),
        LegacyNpcVoiceCue::QuestGreeting
    );
    assert_eq!(
        legacy_npc_open_voice_cue(14, false, true),
        LegacyNpcVoiceCue::Greeting
    );
    assert_eq!(
        legacy_npc_open_voice_cue(14, true, true),
        LegacyNpcVoiceCue::RaceEnd
    );
}

#[test]
fn barber_confirmation_voice_resolves_declared_route_in_en_and_ru() {
    let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog=NativeAudioCatalog::open(&root,false).unwrap();
    let name=legacy_npc_voice_true_name("m_barber",LegacyNpcVoiceCue::BarberOk,&mut GameplayAudioRandom::with_seed(7)).unwrap();
    assert_eq!(name,"m_barber_clickbarb01");
    let voices=catalog.by_true_name(&name);assert_eq!(voices.len(),1);
    for locale in ["en","ru"] {assert!(root.join(catalog.path_for_locale(voices[0],locale).unwrap()).is_file());}
}

#[test]
fn npc_warp_sound_uses_the_unique_spatial_sfx_catalog_route() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = NativeAudioCatalog::open(&root, false).unwrap();
    let candidates = catalog
        .by_true_name("Dexbot_Warp")
        .into_iter()
        .filter(|asset| asset.category == NativeAudioCategory::Sfx)
        .collect::<Vec<_>>();
    let [audio] = candidates.as_slice() else {
        panic!("Dexbot_Warp must resolve to one native SFX route")
    };
    assert!(
        root.join(catalog.path_for_locale(audio, "en").unwrap())
            .is_file()
    );

    let anchor = Entity::from_bits(77);
    let mut runtime = GameplayAudioRuntime::default();
    runtime.queue_legacy_world_sound(anchor, "Dexbot_Warp");
    assert!(matches!(
        runtime.animation_sounds.as_slice(),
        [QueuedAnimationSound {
            anchor: queued_anchor,
            true_name,
            voice_route: false,
            ..
        }] if *queued_anchor == anchor && true_name == "Dexbot_Warp"
    ));
}

#[test]
fn larry_dialogue_uses_the_production_npc_owner_and_both_voice_locales() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(&root).unwrap();
    let content = TutorialMissionContent::open(&locator).unwrap();
    let npc = content.gameplay_npc(692).expect("Larry 3000");
    assert_eq!(npc.move_voice_owner, "larry");
    let catalog = NativeAudioCatalog::open(&root, false).unwrap();
    let mut random = GameplayAudioRandom::with_seed(17);
    for cue in [
        legacy_npc_open_voice_cue(npc.service_category, false, false),
        legacy_npc_open_voice_cue(npc.service_category, false, true),
        LegacyNpcVoiceCue::Farewell,
        LegacyNpcVoiceCue::QuestAccepted,
        LegacyNpcVoiceCue::QuestCompleted,
    ] {
        for _ in 0..32 {
            let name = legacy_npc_voice_true_name(&npc.move_voice_owner, cue, &mut random)
                .expect("nonempty voice owner");
            let candidates = catalog
                .by_true_name(&name)
                .into_iter()
                .filter(|asset| asset.category == NativeAudioCategory::Voice)
                .collect::<Vec<_>>();
            let [audio] = candidates.as_slice() else {
                panic!("Larry cue {name:?} must have exactly one native voice route")
            };
            for locale in ["en", "ru"] {
                let path = catalog.path_for_locale(audio, locale).unwrap();
                assert!(path.contains(&format!("/voice/{locale}/")));
                assert!(
                    root.join(path).is_file(),
                    "missing Larry cue {name:?} in {locale}"
                );
            }
        }
    }
}

#[test]
fn normal_npc_warp_voice_uses_published_locales_or_the_primary_silent_branch() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = NativeAudioCatalog::open(&root, false).unwrap();
    let locator = AssetLocator::open(&root).unwrap();
    let content = TutorialMissionContent::open(&locator).unwrap();
    let mut npc_types = content
        .gameplay_warps()
        .map(|warp| warp.npc_type)
        .collect::<Vec<_>>();
    npc_types.sort_unstable();
    npc_types.dedup();
    let mut normal_rows = 0;
    let mut published_owners = 0;
    let mut silent_owners = Vec::new();
    for npc_type in npc_types {
        if content.normal_gameplay_warp_for_npc(npc_type).is_none() {
            continue;
        }
        let owner = &content.gameplay_npc(npc_type).unwrap().move_voice_owner;
        normal_rows += 1;
        // `AvatarUtil.CallVoicePlay` returns before touching AssetLoader
        // when the table's comment owner is empty.
        if owner.trim().is_empty() {
            continue;
        }
        let mut published = 0;
        for take in 1..=3 {
            let true_name = format!("{owner}_clickwarp0{take}");
            let candidates = catalog
                .by_true_name(&true_name)
                .into_iter()
                .filter(|asset| asset.category == NativeAudioCategory::Voice)
                .collect::<Vec<_>>();
            match candidates.as_slice() {
                [] => {}
                [audio] => {
                    for locale in ["en", "ru"] {
                        assert!(
                            root.join(catalog.path_for_locale(audio, locale).unwrap())
                                .is_file()
                        );
                    }
                    published += 1;
                }
                _ => panic!(
                    "normal warp NPC {npc_type} owner {owner:?} cue {true_name:?} has {} native routes",
                    candidates.len()
                ),
            }
        }
        assert!(
            published == 0 || published == 3,
            "normal warp NPC {npc_type} owner {owner:?} must publish all three takes or preserve the clean silent branch"
        );
        if published == 0 {
            silent_owners.push((npc_type, owner.clone()));
        } else {
            published_owners += 1;
        }
    }
    assert!(
        normal_rows > 0,
        "primary data must contain normal Warp rows"
    );
    assert!(
        published_owners > 0,
        "published normal-Warp voices must retain localized catalog routes"
    );
    assert!(
        silent_owners
            .iter()
            .any(|(npc_type, owner)| *npc_type == 681 && owner == "m_plmber3"),
        "clean Retrobution publishes no m_plmber3_clickwarp01..03 assets; keep that exact silent lookup instead of inventing a replacement voice"
    );
}

#[test]
fn weapon_animation_events_keep_primary_timing_and_sound_table_rows() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = crate::assets::AssetLocator::open(root).unwrap();
    let weapon_catalog = PlayerWeaponAnimationCatalog::open(&locator).unwrap();
    let player = Entity::from_bits(1);
    let mut runtime = GameplayAudioRuntime {
        random: GameplayAudioRandom::with_seed(1),
        ..default()
    };
    runtime.queue_player_weapon_attack(player, PlayerRigGender::Male, 43, 0, &weapon_catalog);
    runtime.queue_player_weapon_attack(player, PlayerRigGender::Female, 43, 0, &weapon_catalog);
    runtime.queue_player_weapon_attack(
        player,
        PlayerRigGender::Male,
        197,
        100,
        &weapon_catalog,
    );
    runtime.queue_player_weapon_attack(player, PlayerRigGender::Male, 328, 0, &weapon_catalog);
    runtime.queue_player_weapon_attack(player, PlayerRigGender::Male, 1, 0, &weapon_catalog);
    runtime.queue_player_weapon_attack(
        player,
        PlayerRigGender::Female,
        365,
        100,
        &weapon_catalog,
    );
    let observed = runtime
        .scheduled_named
        .iter()
        .map(|sound| (sound.remaining_seconds, sound.true_name.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        observed,
        vec![
            (0.296, "MeleeLtUser-01"),
            (0.392, "MeleeLtUser-01"),
            (0.25, "SonicUser-03"),
            (0.25, "LaserHvyUser-02"),
            (0.25, "ThrownUser-01"),
            (0.293_332_994, "TorpedoUser-04"),
        ]
    );
}
