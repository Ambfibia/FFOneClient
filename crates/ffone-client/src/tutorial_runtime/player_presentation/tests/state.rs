use super::*;

#[test]
fn primary_traversal_and_inventory_clips_keep_exact_names_sources_and_playback() {
    let expected = [
        (TutorialPlayerClip::Slide, "slide", 34_408, 34_572),
        (TutorialPlayerClip::RopeDown, "ropedown", 34_214, 34_618),
        (TutorialPlayerClip::RopeDrop, "ropedrop", 34_348, 34_190),
        (TutorialPlayerClip::RopeLeft, "ropeleft", 34_334, 34_169),
        (TutorialPlayerClip::RopeRight, "roperight", 34_350, 34_162),
        (TutorialPlayerClip::RopeStand1, "ropestand1", 34_361, 34_152),
        (TutorialPlayerClip::RopeStand2, "ropestand2", 34_327, 34_204),
        (TutorialPlayerClip::RopeTurn, "ropeturn", 34_606, 34_507),
        (TutorialPlayerClip::RopeUp, "ropeup", 34_335, 34_193),
        (TutorialPlayerClip::Mount1, "mount1", 34_330, 34_280),
        (TutorialPlayerClip::Mount2, "mount2", 34_338, 34_294),
        (TutorialPlayerClip::Inventory, "inven", 34_276, 34_614),
        (
            TutorialPlayerClip::BoardInventory,
            "board_inven",
            34_399,
            34_413,
        ),
        (
            TutorialPlayerClip::ScooterInventory,
            "scooter_inven",
            34_583,
            34_435,
        ),
    ];

    for (clip, name, male_path_id, female_path_id) in expected {
        assert_eq!(clip.name(), name);
        assert_eq!(TutorialPlayerClip::from_exact_name(name), Some(clip));
        assert_eq!(
            clip.playback(),
            TutorialPlayerClipPlayback::Loop,
            "{name} owns a persistent source state"
        );
        for (gender, path_id) in [
            (PlayerRigGender::Male, male_path_id),
            (PlayerRigGender::Female, female_path_id),
        ] {
            assert_eq!(clip.source_path_id(gender), path_id);
            let request = TutorialPlayerAnimationRequest::locomotion(gender, clip)
                .expect("authoritative special presentation is a runtime locomotion clip");
            assert_eq!(request.clip, clip);
            assert_eq!(request.source_path_id, path_id);
        }
    }
}
