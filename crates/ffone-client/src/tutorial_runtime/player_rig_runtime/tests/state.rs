use super::*;

#[test]
fn traversal_and_inventory_semantics_map_exactly_without_weapon_prefixes() {
    let expected = [
        (LegacyVisualClip::Die, TutorialPlayerClip::Die),
        (LegacyVisualClip::Death, TutorialPlayerClip::Death),
        (LegacyVisualClip::Slide, TutorialPlayerClip::Slide),
        (LegacyVisualClip::RopeDown, TutorialPlayerClip::RopeDown),
        (LegacyVisualClip::Launcher, TutorialPlayerClip::Launcher),
        (LegacyVisualClip::RopeDrop, TutorialPlayerClip::RopeDrop),
        (LegacyVisualClip::RopeLeft, TutorialPlayerClip::RopeLeft),
        (LegacyVisualClip::RopeRight, TutorialPlayerClip::RopeRight),
        (LegacyVisualClip::RopeStand1, TutorialPlayerClip::RopeStand1),
        (LegacyVisualClip::RopeStand2, TutorialPlayerClip::RopeStand2),
        (LegacyVisualClip::RopeTurn, TutorialPlayerClip::RopeTurn),
        (LegacyVisualClip::RopeUp, TutorialPlayerClip::RopeUp),
        (LegacyVisualClip::Inventory, TutorialPlayerClip::Inventory),
        (
            LegacyVisualClip::BoardInventory,
            TutorialPlayerClip::BoardInventory,
        ),
        (
            LegacyVisualClip::ScooterInventory,
            TutorialPlayerClip::ScooterInventory,
        ),
    ];
    let profiles = [
        None,
        Some(PlayerWeaponAnimationProfile::Stick),
        Some(PlayerWeaponAnimationProfile::Pistol),
        Some(PlayerWeaponAnimationProfile::Rifle),
        Some(PlayerWeaponAnimationProfile::Bomb),
        Some(PlayerWeaponAnimationProfile::Rocket),
    ];

    for (semantic, concrete) in expected {
        for profile in profiles {
            for tutorial_stand_semantics in [false, true] {
                assert_eq!(
                    legacy_visual_tutorial_clip(semantic, profile, tutorial_stand_semantics,),
                    Some(concrete),
                    "semantic={semantic:?}, profile={profile:?}, tutorial={tutorial_stand_semantics}"
                );
            }
        }
    }
}

#[test]
fn bevy_adapter_restarts_the_real_node_and_sets_contract_repeat_mode() {
    let (_, nodes) = AnimationGraph::from_clips([
        Handle::<AnimationClip>::default(),
        Handle::<AnimationClip>::default(),
    ]);
    let mut player = AnimationPlayer::default();
    let mut transitions = AnimationTransitions::new();
    let immediate =
        TutorialPlayerAnimationRequest::avatar_emote(PlayerRigGender::Male, "staying").unwrap();
    apply_resolved_tutorial_animation(
        &mut player,
        &mut transitions,
        nodes[0],
        ContractResolvedTutorialPlayerAnimation {
            request: immediate,
            gltf_animation_index: 7,
            playback: TutorialPlayerClipPlayback::Clamp,
        },
    );
    assert_eq!(transitions.get_main_animation(), Some(nodes[0]));
    assert_eq!(
        player.animation(nodes[0]).unwrap().repeat_mode(),
        RepeatAnimation::Never
    );
    player
        .animation_mut(nodes[0])
        .unwrap()
        .set_seek_time(0.75)
        .pause();
    apply_resolved_tutorial_animation(
        &mut player,
        &mut transitions,
        nodes[0],
        ContractResolvedTutorialPlayerAnimation {
            request: immediate,
            gltf_animation_index: 7,
            playback: TutorialPlayerClipPlayback::Clamp,
        },
    );
    assert_eq!(player.animation(nodes[0]).unwrap().seek_time(), 0.0);
    assert!(!player.animation(nodes[0]).unwrap().is_paused());
}
