use super::*;

#[test]
fn production_weapon_rows_select_every_legacy_animation_family() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(root).unwrap();
    let catalog = PlayerWeaponAnimationCatalog::open(&locator).unwrap();

    assert!(catalog.len() > 700);
    assert_eq!(
        catalog.profile_for_item(43),
        Some(PlayerWeaponAnimationProfile::Stick)
    );
    assert_eq!(
        catalog.profile_for_item(197),
        Some(PlayerWeaponAnimationProfile::Pistol)
    );
    assert_eq!(
        catalog.profile_for_item(328),
        Some(PlayerWeaponAnimationProfile::Rifle)
    );
    assert_eq!(
        catalog.profile_for_item(1),
        Some(PlayerWeaponAnimationProfile::Bomb)
    );
    assert_eq!(
        catalog.profile_for_item(365),
        Some(PlayerWeaponAnimationProfile::Rocket)
    );
    assert_eq!(catalog.profile_for_item(639), None);
    assert_eq!(catalog.attack_sounds.len(), 806);
    assert_eq!(catalog.profiles.len(), 797);
    assert_eq!(
        catalog.attack_sound_variants_for_item(43, 0),
        Some(&["MeleeLtUser-01".to_owned()][..])
    );
    assert_eq!(
        catalog.attack_sound_variants_for_item(43, 100),
        Some(&["MeleeMedUser-01".to_owned()][..])
    );
    assert_eq!(
        catalog.attack_sound_variants_for_item(197, 100),
        Some(&["SonicUser-03".to_owned()][..])
    );
    assert_eq!(
        catalog.attack_sound_variants_for_item(328, 0),
        Some(&["LaserHvyUser-02".to_owned()][..])
    );
    assert_eq!(
        catalog.attack_sound_variants_for_item(365, 100),
        Some(&["TorpedoUser-04".to_owned()][..])
    );
    assert_eq!(
        catalog.attack_sound_variants_for_item(1, 100),
        Some(&["ThrownUser-01".to_owned()][..]),
        "the orphaned primary ThrownUesr-05 row must fall back to this weapon's Sound1"
    );

    let audio = crate::semantic_audio::NativeAudioCatalog::open(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"),
        false,
    )
    .unwrap();
    for item_id in catalog.profiles.keys().copied() {
        for weapon_battery in [0, 1] {
            let variants = catalog
                .attack_sound_variants_for_item(item_id, weapon_battery)
                .unwrap_or_else(|| {
                    panic!(
                        "equippable weapon {item_id} has no attack sound at battery {weapon_battery}"
                    )
                });
            assert!(!variants.is_empty());
            for true_name in variants {
                let candidates = audio
                    .by_true_name(true_name)
                    .into_iter()
                    .filter(|asset| {
                        asset.category == crate::semantic_audio::NativeAudioCategory::Sfx
                    })
                    .collect::<Vec<_>>();
                assert_eq!(
                    candidates.len(),
                    1,
                    "weapon {item_id} attack sound {true_name:?} at battery {weapon_battery} must resolve to exactly one native SFX"
                );
            }
        }
    }
    assert_eq!(
        catalog.combat_profile_for_item(43),
        Some(PlayerWeaponCombatProfile {
            attack_half_angle_degrees: 90.0,
            attack_range: 3.0,
            blast_radius: 0.5,
            attack_cooldown_seconds: 1.0,
            target_capacity: 3,
            target_mode: LegacyWeaponTargetMode::Normal,
            effect1_bullet_type: 5,
            effect2_bullet_type: 145,
            warhead_duration_seconds: 0.0,
            grenade_initial_vertical_speed: 15.0,
        })
    );
    assert_eq!(
        catalog.combat_profile_for_item(197),
        Some(PlayerWeaponCombatProfile {
            attack_half_angle_degrees: 20.0,
            attack_range: 12.0,
            blast_radius: 0.5,
            attack_cooldown_seconds: 0.8,
            target_capacity: 1,
            target_mode: LegacyWeaponTargetMode::Normal,
            effect1_bullet_type: 113,
            effect2_bullet_type: 151,
            warhead_duration_seconds: 0.0,
            grenade_initial_vertical_speed: 80.0,
        })
    );
    assert_eq!(
        catalog.combat_profile_for_item(1).map(|profile| (
            profile.target_mode,
            profile.attack_range,
            profile.warhead_duration_seconds,
            profile.grenade_initial_vertical_speed,
            profile.effect1_bullet_type,
            profile.effect2_bullet_type,
        )),
        Some((LegacyWeaponTargetMode::Grenade, 8.0, 2.5, 6.0, 72, 163))
    );
    assert_eq!(
        catalog
            .combat_profile_for_item(365)
            .map(|profile| profile.target_mode),
        Some(LegacyWeaponTargetMode::Rocket)
    );
    // Equip type 5 has no animation family, but its authoritative attack
    // cooldown must still be available to the network action gate.
    assert_eq!(
        catalog
            .combat_profile_for_item(639)
            .map(|profile| (profile.attack_cooldown_seconds, profile.target_mode)),
        Some((2.0, LegacyWeaponTargetMode::Grenade))
    );
    // Equip type 10 is an animation family, not an attack ABI. This row
    // is deliberately a normal target-mode weapon despite using Bomb
    // locomotion and catches accidental equip-type routing.
    assert_eq!(
        catalog
            .combat_profile_for_item(691)
            .map(|profile| profile.target_mode),
        Some(LegacyWeaponTargetMode::Normal)
    );
    let pistol = catalog.combat_profile_for_item(197).unwrap();
    assert_eq!(pistol.bullet_type(0), 113);
    assert_eq!(pistol.bullet_type(-1), 113);
    assert_eq!(pistol.bullet_type(1), 151);
    assert_eq!(pistol.bullet_type(999), 151);
    assert_eq!(
        PlayerWeaponCombatProfile {
            effect2_bullet_type: 0,
            ..pistol
        }
        .bullet_type(1),
        113
    );

    let published = ffone_runtime_contracts::RETROBUTION_TUTORIAL_BULLET_TYPES;
    for profile in catalog.combat_profiles.values().copied() {
        for battery in [0, 1] {
            assert!(
                published.contains(&profile.native_presentation_bullet_type(battery)),
                "weapon profile {profile:?} resolves outside the native projectile catalog"
            );
        }
    }
    assert_eq!(
        catalog
            .combat_profile_for_item(357)
            .unwrap()
            .native_presentation_bullet_type(0),
        13
    );
    assert_eq!(
        catalog
            .combat_profile_for_item(357)
            .unwrap()
            .native_presentation_bullet_type(100),
        152
    );
    assert_eq!(
        catalog
            .combat_profile_for_item(758)
            .unwrap()
            .native_presentation_bullet_type(0),
        164
    );
    assert_eq!(
        catalog
            .combat_profile_for_item(759)
            .unwrap()
            .native_presentation_bullet_type(0),
        136
    );
    assert_eq!(
        catalog
            .combat_profile_for_item(759)
            .unwrap()
            .native_presentation_bullet_type(100),
        165
    );
}

pub(super) fn clip_contract(
    gender: PlayerRigGender,
    clip: TutorialPlayerClip,
    animation_index: u32,
    runtime_status: &str,
) -> PlayerRigClipContract {
    PlayerRigClipContract {
        name: clip.name().to_owned(),
        source_path_id: clip.source_path_id(gender),
        gltf_animation_index: animation_index,
        channel_count: 1,
        source_key_count: 1,
        duration_seconds_bits: 1.0_f64.to_bits(),
        playback: clip.playback().contract_value().to_owned(),
        runtime_status: runtime_status.to_owned(),
    }
}

#[test]
fn exact_gender_clip_path_ids_are_preserved() {
    let expected = [
        (PlayerRigGender::Male, TutorialPlayerClip::Run, 34_268),
        (PlayerRigGender::Male, TutorialPlayerClip::Staying, 34_269),
        (PlayerRigGender::Male, TutorialPlayerClip::Standup, 34_404),
        (PlayerRigGender::Female, TutorialPlayerClip::Run, 34_191),
        (PlayerRigGender::Female, TutorialPlayerClip::Staying, 34_194),
        (PlayerRigGender::Female, TutorialPlayerClip::Standup, 34_533),
        (PlayerRigGender::Male, TutorialPlayerClip::RunBack, 34_249),
        (PlayerRigGender::Male, TutorialPlayerClip::JumpStart, 34_390),
        (PlayerRigGender::Male, TutorialPlayerClip::Jump, 34_314),
        (PlayerRigGender::Male, TutorialPlayerClip::JumpEnd, 34_224),
        (
            PlayerRigGender::Male,
            TutorialPlayerClip::JumpLandRun,
            34_256,
        ),
        (PlayerRigGender::Female, TutorialPlayerClip::RunBack, 34_611),
        (
            PlayerRigGender::Female,
            TutorialPlayerClip::JumpStart,
            34_557,
        ),
        (PlayerRigGender::Female, TutorialPlayerClip::Jump, 34_605),
        (PlayerRigGender::Female, TutorialPlayerClip::JumpEnd, 34_626),
        (
            PlayerRigGender::Female,
            TutorialPlayerClip::JumpLandRun,
            34_567,
        ),
        (PlayerRigGender::Male, TutorialPlayerClip::Stand1, 237_023),
        (PlayerRigGender::Female, TutorialPlayerClip::Stand1, 237_027),
        (PlayerRigGender::Male, TutorialPlayerClip::Attack1, 34_235),
        (
            PlayerRigGender::Male,
            TutorialPlayerClip::Attack1Upper,
            34_213,
        ),
        (PlayerRigGender::Female, TutorialPlayerClip::Attack1, 34_171),
        (
            PlayerRigGender::Female,
            TutorialPlayerClip::Attack1Upper,
            34_174,
        ),
        (
            PlayerRigGender::Male,
            TutorialPlayerClip::StickAttack1,
            34_332,
        ),
        (
            PlayerRigGender::Male,
            TutorialPlayerClip::StickAttack1Upper,
            34_412,
        ),
        (
            PlayerRigGender::Male,
            TutorialPlayerClip::PistolAttack1,
            34_380,
        ),
        (
            PlayerRigGender::Male,
            TutorialPlayerClip::PistolAttack1Upper,
            34_240,
        ),
        (
            PlayerRigGender::Female,
            TutorialPlayerClip::StickAttack1,
            34_579,
        ),
        (
            PlayerRigGender::Female,
            TutorialPlayerClip::StickAttack1Upper,
            34_315,
        ),
        (
            PlayerRigGender::Female,
            TutorialPlayerClip::PistolAttack1,
            34_414,
        ),
        (
            PlayerRigGender::Female,
            TutorialPlayerClip::PistolAttack1Upper,
            34_286,
        ),
        (PlayerRigGender::Male, TutorialPlayerClip::Swim, 34_486),
        (PlayerRigGender::Male, TutorialPlayerClip::SwimBack, 34_453),
        (PlayerRigGender::Male, TutorialPlayerClip::SwimIdle, 34_524),
        (PlayerRigGender::Male, TutorialPlayerClip::SwimLeft, 34_535),
        (PlayerRigGender::Male, TutorialPlayerClip::SwimRight, 34_514),
        (PlayerRigGender::Female, TutorialPlayerClip::Swim, 34_185),
        (
            PlayerRigGender::Female,
            TutorialPlayerClip::SwimBack,
            34_165,
        ),
        (
            PlayerRigGender::Female,
            TutorialPlayerClip::SwimIdle,
            34_512,
        ),
        (
            PlayerRigGender::Female,
            TutorialPlayerClip::SwimLeft,
            34_503,
        ),
        (
            PlayerRigGender::Female,
            TutorialPlayerClip::SwimRight,
            34_498,
        ),
    ];
    for (gender, clip, path_id) in expected {
        assert_eq!(clip.source_path_id(gender), path_id);
    }
}

#[test]
fn end_animation_uses_the_source_point_three_second_cross_fade() {
    let request = TutorialPlayerAnimationRequest::end_animation_cross_fade(
        PlayerRigGender::Male,
        TutorialPlayerClip::RifleReady,
    );
    assert_eq!(
        request.dispatch,
        TutorialPlayerAnimationDispatch::CrossFade(Duration::from_secs_f32(0.3))
    );
    assert!(!request.sets_emote_state);
    assert!(!request.hide_hand_attachment);
}

#[test]
fn missing_clip_resolution_never_substitutes_an_available_alias() {
    let published_today = TutorialPlayerClipAvailability::from_names([
        "stand1",
        "height_Add",
        "height",
        "shape_Add",
        "shape",
        "riflestand1",
    ]);
    let standing =
        TutorialPlayerAnimationRequest::avatar_emote(PlayerRigGender::Male, "staying").unwrap();
    let missing = published_today.resolve(standing).unwrap_err();
    assert_eq!(missing.requested_clip, TutorialPlayerClip::Staying);
    assert_eq!(missing.source_path_id, 34_269);
    assert!(missing.available_names.iter().any(|name| name == "stand1"));

    let stand_force = TutorialPlayerAnimationRequest::stand_force(PlayerRigGender::Male);
    assert_eq!(
        published_today.resolve(stand_force).unwrap().request,
        stand_force
    );
}

#[test]
fn exact_clip_resolution_is_case_sensitive() {
    let availability = TutorialPlayerClipAvailability::from_names(["Run", "stand1"]);
    let request =
        TutorialPlayerAnimationRequest::avatar_emote(PlayerRigGender::Female, "run").unwrap();
    assert!(availability.resolve(request).is_err());
}

#[test]
fn death_cancels_only_pending_direct_pose_overrides() {
    let mut queue = TutorialPlayerPresentationCommandQueue::default();
    queue
        .avatar_emote(PlayerRigGender::Male, "staying")
        .unwrap();
    queue.stand_force(PlayerRigGender::Male);
    let weapon = queue.tutorial_weapon(false, 2).unwrap();
    let death = TutorialPlayerAnimationRequest::runtime_cross_fade(
        PlayerRigGender::Male,
        TutorialPlayerClip::Die,
    );
    queue.push_animation(death);

    queue.cancel_pending_direct_pose_overrides();

    assert_eq!(queue.len(), 2);
    assert_eq!(
        queue.pop_front(),
        Some(TutorialPlayerPresentationCommand::Equipment(weapon))
    );
    assert_eq!(
        queue.pop_front(),
        Some(TutorialPlayerPresentationCommand::Animation(death))
    );
}

#[test]
fn damage_and_traversal_discard_bridged_attacks_but_keep_wound() {
    let mut queue = TutorialPlayerPresentationCommandQueue::default();
    let gender = PlayerRigGender::Female;
    let wound = TutorialPlayerAnimationRequest::runtime_cross_fade(
        gender, TutorialPlayerClip::WoundUpper);
    for clip in [TutorialPlayerClip::Attack1, TutorialPlayerClip::Attack1Upper,
        TutorialPlayerClip::RifleAttack1, TutorialPlayerClip::RifleAttack1Upper] {
        queue.push_animation(TutorialPlayerAnimationRequest::runtime_cross_fade(gender, clip));
    }
    queue.push_animation(wound);
    queue.cancel_pending_runtime_attacks();
    assert_eq!(queue.len(), 1);
    queue.cancel_pending_upper_layers();
    assert!(queue.is_empty(), "traversal must not inherit a pending wound overlay");
    queue.push_animation(wound);
    assert_eq!(queue.pop_front(), Some(TutorialPlayerPresentationCommand::Animation(wound)));
}

#[test]
fn animation_delay_stays_ordered_with_the_attack_layer_commands() {
    let gender = PlayerRigGender::Male;
    let capabilities = TutorialPlayerRigCapabilities::from_contract(
        gender,
        &[
            clip_contract(
                gender,
                TutorialPlayerClip::RifleAttack1,
                1,
                TUTORIAL_PLAYER_RUNTIME_READY_STATUS,
            ),
            clip_contract(
                gender,
                TutorialPlayerClip::RifleAttack1Upper,
                2,
                TUTORIAL_PLAYER_RUNTIME_READY_STATUS,
            ),
        ],
    )
    .unwrap();
    let consumer = TutorialPlayerPresentationConsumer::new(capabilities);
    let mut queue = TutorialPlayerPresentationCommandQueue::default();
    queue.push_animation(TutorialPlayerAnimationRequest::runtime_cross_fade(
        gender,
        TutorialPlayerClip::RifleAttack1,
    ));
    queue.push_animation(TutorialPlayerAnimationRequest::runtime_cross_fade(
        gender,
        TutorialPlayerClip::RifleAttack1Upper,
    ));
    queue.push_animation_delay(Duration::from_secs_f32(0.15));

    let resolved = consumer.consume_available(&mut queue);
    assert_eq!(resolved.len(), 3);
    assert!(matches!(
        resolved[0],
        TutorialPlayerPresentationResolution::AnimationContractResolved(
            ContractResolvedTutorialPlayerAnimation {
                request: TutorialPlayerAnimationRequest {
                    clip: TutorialPlayerClip::RifleAttack1,
                    ..
                },
                ..
            }
        )
    ));
    assert!(matches!(
        resolved[1],
        TutorialPlayerPresentationResolution::AnimationContractResolved(
            ContractResolvedTutorialPlayerAnimation {
                request: TutorialPlayerAnimationRequest {
                    clip: TutorialPlayerClip::RifleAttack1Upper,
                    ..
                },
                ..
            }
        )
    ));
    assert_eq!(
        resolved[2],
        TutorialPlayerPresentationResolution::AnimationDelay(Duration::from_secs_f32(0.15))
    );
}
