use super::*;

#[test]
fn production_character_glbs_expose_exact_runtime_effect_events() {
    let locator = AssetLocator::open(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"),
    )
    .unwrap();
    for (path, expected) in [
        (
            "characters/fusions/fusion_buttercup/fusion_buttercup.glb",
            &[("corruptak", 547), ("skill0", 548)] as &[_],
        ),
        (
            "characters/fusions/fusion_blooexpression/fusion_blooexpression.glb",
            &[("corruptak", 601), ("skill0", 602)],
        ),
        (
            "characters/fusions/fusion_mac/fusion_mac.glb",
            &[("corruptak", 626), ("skill0", 627)],
        ),
        (
            "characters/mobs/mob_fuselord/mob_fuselord.glb",
            &[("melee1", 689), ("skill0", 695)],
        ),
        (
            "characters/npcs/npc_scamper/npc_scamper.glb",
            &[("fly", 768)],
        ),
    ] {
        let events =
            parse_network_npc_animation_effect_events(&locator.read(path).unwrap()).unwrap();
        for &(clip, effect_id) in expected {
            assert!(
                events
                    .iter()
                    .any(|event| event.clip == clip && event.effect_id == effect_id),
                "{path} lost {clip} ES{effect_id}: {events:#?}"
            );
        }
    }

    let sounds = parse_network_npc_animation_sound_events(
        &locator
            .read("characters/fusions/fusion_ace/fusion_ace.glb")
            .unwrap(),
    )
    .unwrap();
    assert!(sounds.iter().any(|event| {
        event.clip == "corruptak"
            && (event.time - 0.1).abs() < f32::EPSILON
            && event.payload == "FusionAce_Corruptak.wav"
    }));

    for (path, summon_voice) in [
        (
            "characters/nanos/nano_buttercup/nano_buttercup.glb",
            "Btrcup_NanSummon01_0(RAND:1-3).wav",
        ),
        (
            "characters/nanos/nano_bloo/nano_bloo.glb",
            "Bloo_NanSummon0(RAND:1-3).wav",
        ),
    ] {
        let sounds =
            parse_network_npc_animation_sound_events(&locator.read(path).unwrap()).unwrap();
        assert!(
            sounds
                .iter()
                .any(|event| event.clip.eq_ignore_ascii_case("call")
                    && event.payload == summon_voice),
            "{path} lost its clean Nano acquisition summon voice: {sounds:#?}"
        );
        assert!(
            sounds
                .iter()
                .any(|event| event.clip.eq_ignore_ascii_case("call")
                    && event.payload == "Nano Ability 06.wav"),
            "{path} lost its clean Nano acquisition call SFX: {sounds:#?}"
        );
    }
}
